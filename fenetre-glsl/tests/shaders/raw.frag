#version 450
// A complete shader: no preamble is added.
layout(location = 0) out vec4 color;
void main() { color = vec4(gl_FragCoord.xy / 1000.0, 0.0, 1.0); }
