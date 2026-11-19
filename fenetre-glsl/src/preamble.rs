//! Wraps a Shadertoy-style shader into a complete Vulkan GLSL shader.

/// Lines added before the user's code. It ends with `#line 1`, so glslang
/// numbers the user's first line 1 and every error lands on the right line.
///
/// Shadertoy puts the origin of `fragCoord` at the bottom left, Vulkan at the
/// top left: `main()` flips the y axis.
const SHADERTOY_PREAMBLE: &str = "\
#version 450
layout(set = 0, binding = 0, std140) uniform FenetreGlobals {
    vec3 iResolution;
    float iTime;
    float iTimeDelta;
    int iFrame;
    vec4 iMouse;
    vec4 iDate;
};
layout(location = 0) out vec4 fenetre_FragColor;
void mainImage(out vec4 fragColor, in vec2 fragCoord);
void main() {
    vec2 fragCoord = vec2(gl_FragCoord.x, iResolution.y - gl_FragCoord.y);
    mainImage(fenetre_FragColor, fragCoord);
}
#line 1
";

/// A shader ready for the compiler.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prepared {
    /// Full GLSL text given to glslang.
    pub text: String,
    /// `true` when the Shadertoy preamble was added.
    pub wrapped: bool,
}

/// Adds the Shadertoy preamble, unless the source has its own `#version`.
#[must_use]
pub fn prepare(source: &str) -> Prepared {
    if has_version_directive(source) {
        return Prepared {
            text: source.to_owned(),
            wrapped: false,
        };
    }
    let mut text = String::with_capacity(SHADERTOY_PREAMBLE.len() + source.len());
    text.push_str(SHADERTOY_PREAMBLE);
    text.push_str(source);
    Prepared {
        text,
        wrapped: true,
    }
}

fn has_version_directive(source: &str) -> bool {
    source.lines().any(|line| {
        line.trim_start()
            .strip_prefix('#')
            .is_some_and(|rest| rest.trim_start().starts_with("version"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_a_shadertoy_shader() {
        let prepared = prepare("void mainImage(out vec4 c, in vec2 p) {}");
        assert!(prepared.wrapped);
        assert!(prepared.text.starts_with("#version 450\n"));
        assert!(
            prepared
                .text
                .ends_with("#line 1\nvoid mainImage(out vec4 c, in vec2 p) {}")
        );
    }

    #[test]
    fn keeps_a_shader_with_its_own_version() {
        let source = "// mine\n  # version 450\nvoid main() {}";
        let prepared = prepare(source);
        assert!(!prepared.wrapped);
        assert_eq!(prepared.text, source);
    }
}
