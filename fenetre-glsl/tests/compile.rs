//! Compiles real shaders with glslang.

use fenetre_glsl::{CompileError, Compiler, Severity};

const SPIRV_MAGIC: u32 = 0x0723_0203;

fn compile(source: &str) -> Result<fenetre_glsl::Compiled, CompileError> {
    Compiler::new().expect("glslang starts").compile(source)
}

#[test]
fn shadertoy_shaders_compile() {
    for source in [
        include_str!("shaders/gradient.frag"),
        include_str!("shaders/raymarch.frag"),
    ] {
        let compiled = compile(source).unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(compiled.spirv[0], SPIRV_MAGIC);
    }
}

#[test]
fn a_shader_with_its_own_version_compiles_untouched() {
    let compiled = compile(include_str!("shaders/raw.frag")).unwrap();
    assert_eq!(compiled.spirv[0], SPIRV_MAGIC);
}

#[test]
fn errors_point_to_the_users_line_and_column() {
    let Err(CompileError::Shader(diagnostics)) = compile(include_str!("shaders/broken.frag"))
    else {
        panic!("broken.frag should not compile");
    };
    let first = &diagnostics[0];
    assert_eq!(first.severity, Severity::Error);
    assert_eq!(first.line, Some(4));
    assert_eq!(first.column, Some(26));
    assert_eq!(first.message, "'wobble': undeclared identifier");
}

#[test]
fn a_missing_main_image_is_reported() {
    let Err(CompileError::Shader(diagnostics)) = compile("float unused;\n") else {
        panic!("a shader without mainImage should not compile");
    };
    assert!(
        diagnostics.iter().any(|d| d.message.contains("mainImage")),
        "{diagnostics:?}"
    );
}

#[test]
fn a_nul_character_is_an_error_not_a_crash() {
    let Err(CompileError::Shader(diagnostics)) = compile("void f() {}\n\0") else {
        panic!("NUL should be refused");
    };
    assert_eq!(diagnostics[0].line, Some(2));
}

/// wgpu hands SPIR-V to naga: make sure naga reads and validates ours.
#[test]
fn naga_accepts_the_spirv() {
    for source in [
        include_str!("shaders/gradient.frag"),
        include_str!("shaders/raymarch.frag"),
    ] {
        let words = compile(source).unwrap().spirv;
        let module = naga::front::spv::parse_u8_slice(
            to_bytes(&words).as_slice(),
            &naga::front::spv::Options::default(),
        )
        .expect("naga parses the SPIR-V");
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::default(),
        )
        .validate(&module)
        .expect("naga validates the module");
    }
}

fn to_bytes(words: &[u32]) -> Vec<u8> {
    words.iter().flat_map(|w| w.to_le_bytes()).collect()
}
