//! Compiles prepared GLSL to SPIR-V with glslang (through `shaderc`).

use std::fmt;

use crate::diagnostic::{self, Diagnostic, Severity};
use crate::preamble::prepare;

/// Name given to glslang for the source. It shows up in its messages, which
/// [`diagnostic::parse`] reads back.
const FILE_NAME: &str = "shader.frag";

/// A shader that compiled.
#[derive(Clone, Debug)]
pub struct Compiled {
    /// SPIR-V words, ready for `wgpu::ShaderSource::SpirV`.
    pub spirv: Vec<u32>,
    /// Warnings found on the way (the shader still runs).
    pub warnings: Vec<Diagnostic>,
}

/// Why a shader did not compile.
#[derive(Debug)]
pub enum CompileError {
    /// The shader has errors; each one points into the user's source.
    Shader(Vec<Diagnostic>),
    /// glslang itself failed (not the user's fault).
    Internal(String),
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Shader(diagnostics) => {
                for (i, diagnostic) in diagnostics.iter().enumerate() {
                    if i > 0 {
                        f.write_str("\n")?;
                    }
                    write!(f, "{diagnostic}")?;
                }
                Ok(())
            }
            Self::Internal(reason) => write!(f, "internal compiler error: {reason}"),
        }
    }
}

impl std::error::Error for CompileError {}

/// A glslang instance. Create one and reuse it for every compilation.
pub struct Compiler {
    inner: shaderc::Compiler,
}

impl Compiler {
    /// Starts glslang.
    ///
    /// # Errors
    /// [`CompileError::Internal`] when shaderc cannot be initialised.
    pub fn new() -> Result<Self, CompileError> {
        shaderc::Compiler::new()
            .map(|inner| Self { inner })
            .map_err(|error| CompileError::Internal(error.to_string()))
    }

    /// Compiles a fragment shader (Shadertoy style, or complete with its own
    /// `#version`) to SPIR-V for Vulkan 1.0.
    ///
    /// # Errors
    /// [`CompileError::Shader`] with the user's mistakes, located in
    /// `source`; [`CompileError::Internal`] if glslang breaks down.
    pub fn compile(&self, source: &str) -> Result<Compiled, CompileError> {
        if let Some(line) = nul_byte_line(source) {
            return Err(CompileError::Shader(vec![Diagnostic {
                severity: Severity::Error,
                line: Some(line),
                column: None,
                message: "the file contains a NUL character".to_owned(),
            }]));
        }
        let prepared = prepare(source);
        let mut options = shaderc::CompileOptions::new()
            .map_err(|error| CompileError::Internal(error.to_string()))?;
        options.set_target_env(
            shaderc::TargetEnv::Vulkan,
            shaderc::EnvVersion::Vulkan1_0 as u32,
        );
        let artifact = self
            .inner
            .compile_into_spirv(
                &prepared.text,
                shaderc::ShaderKind::Fragment,
                FILE_NAME,
                "main",
                Some(&options),
            )
            .map_err(|error| match error {
                shaderc::Error::CompilationError(_, output) => {
                    CompileError::Shader(diagnostic::parse(&output, FILE_NAME, source))
                }
                other => CompileError::Internal(other.to_string()),
            })?;
        Ok(Compiled {
            spirv: artifact.as_binary().to_vec(),
            warnings: diagnostic::parse(&artifact.get_warning_messages(), FILE_NAME, source),
        })
    }
}

/// shaderc passes the source as a C string, which cannot hold `\0`.
fn nul_byte_line(source: &str) -> Option<u32> {
    let index = source.find('\0')?;
    let newlines = source[..index].matches('\n').count();
    u32::try_from(newlines + 1).ok()
}
