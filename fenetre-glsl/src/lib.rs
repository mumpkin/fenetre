//! Turns a GLSL fragment shader written the Shadertoy way into SPIR-V.
//!
//! The user writes `void mainImage(out vec4 fragColor, in vec2 fragCoord)`.
//! [`prepare`] adds the version, the uniforms and a `main()` that calls it.
//! [`Compiler::compile`] runs glslang (through `shaderc`) on the result and
//! returns either SPIR-V words or [`Diagnostic`]s whose line numbers point
//! into the user's own file, not into the generated one.
//!
//! A source with its own `#version` line is compiled untouched: the user
//! is then in charge of the whole shader.

mod compiler;
mod diagnostic;
mod globals;
mod preamble;

pub use compiler::{CompileError, Compiled, Compiler};
pub use diagnostic::{Diagnostic, Severity};
pub use globals::Globals;
pub use preamble::{Prepared, prepare};
