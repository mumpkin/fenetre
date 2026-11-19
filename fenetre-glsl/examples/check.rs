//! Compiles shader files and prints errors the way compilers do.
//!
//! ```sh
//! cargo run -p fenetre-glsl --example check -- my-shader.frag
//! ```

use std::process::ExitCode;

use fenetre_glsl::{CompileError, Compiler};

fn main() -> ExitCode {
    let paths: Vec<String> = std::env::args().skip(1).collect();
    if paths.is_empty() {
        eprintln!("usage: check <shader.frag>...");
        return ExitCode::from(2);
    }
    let compiler = match Compiler::new() {
        Ok(compiler) => compiler,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };
    let mut failed = false;
    for path in &paths {
        let source = match std::fs::read_to_string(path) {
            Ok(source) => source,
            Err(error) => {
                eprintln!("{path}: {error}");
                failed = true;
                continue;
            }
        };
        match compiler.compile(&source) {
            Ok(shader) => {
                for warning in &shader.warnings {
                    eprintln!("{path}:{warning}");
                }
                println!("{path}: ok ({} SPIR-V words)", shader.spirv.len());
            }
            Err(CompileError::Shader(diagnostics)) => {
                for diagnostic in &diagnostics {
                    eprintln!("{path}:{diagnostic}");
                }
                failed = true;
            }
            Err(error) => {
                eprintln!("{path}: {error}");
                failed = true;
            }
        }
    }
    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
