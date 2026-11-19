# Fenêtre

A live GLSL shader editor for the desktop, in the spirit of KodeLife and
Bonzomatic, written in Rust with [GPUI](https://www.gpui.rs/) and driven by
[Helix](https://helix-editor.com/) key bindings.

> **Status: early prototyping.** Nothing usable yet. This is also a learning
> project (Rust, GPUI, shaders, and later live-coded music), so expect the
> code to evolve as its author learns.

## Goals

- **Write, save, see.** `:w` compiles the shader and shows it right away.
- **Never lose the picture.** If compilation fails, the last working shader
  keeps running and the error is shown on its line.
- **Shadertoy-friendly.** A `mainImage(out vec4, in vec2)` shader with the
  usual uniforms (`iTime`, `iResolution`, `iMouse`…) runs as is.
- **Fragment and vertex shaders** (vertexshaderart-style for the latter).
- **Three display modes**: side-by-side split, picture-in-picture, and
  full-window background with the code written on top.
- **Custom render resolution**, independent of the window size.
- **Helix editing**: modal, multiple selections, `:w` / `:q`, `Space` menus.
- Later: audio input (FFT) for shaders, and live-coded music
  (TidalCycles, Sardine) synced with the visuals.

## Platforms

Linux first (Wayland and X11).

## Building

Requires a recent stable Rust toolchain (edition 2024).

```sh
cargo build
cargo test
```

## Layout

| Crate | Purpose |
|-------|---------|
| `fenetre-glsl` | GLSL preamble, compilation and error reporting |

More crates (rendering, modal editing, the GPUI app) will join the
workspace as prototyping goes on.

## License

[MIT](LICENSE)
