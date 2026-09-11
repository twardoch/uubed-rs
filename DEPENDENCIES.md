---
this_file: DEPENDENCIES.md
---
# Native dependencies

- uubed-core retains Rayon, once_cell, libc, rand/ChaCha/rand_distr and optional bytemuck for its tested encoders.
- uubed-tm uses bundled rusqlite for standalone SQLite files, serde/serde_json for interchange and thiserror for typed errors.
- uubed-python links both through PyO3 0.22. Macro-generated redundant error conversions have one documented Clippy exception.
- tempfile is test-only; Python inference dependencies belong to uubed-py.

## Release tools

uv runs isolated build tools; gitnextver 1.0.1 calculates the next semantic tag; tomlkit preserves Cargo formatting during version synchronization; Hatch VCS generates Python versions; Maturin builds the native Python distribution; Cargo packages/verifies Rust crates; MkDocs builds/deploys the documentation site. These are development/release tools, not new runtime dependencies.
