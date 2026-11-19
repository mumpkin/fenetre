# Contributing

## Requirements

- Rust stable (edition 2024).
- shaderc (glslang): `pacman -S shaderc` on Arch, `apt install libshaderc-dev`
  on Debian/Ubuntu. `fenetre-glsl` links the system library.

## Checks

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Conventions

- Branches: `type/topic-in-kebab-case` (`feat/glsl-compiler`).
- Commits: [Conventional Commits](https://www.conventionalcommits.org/)
  (`feat:`, `fix:`, `docs:`, `chore:`…).
- One pull request per topic, merged with "Rebase and merge" once CI is green.
