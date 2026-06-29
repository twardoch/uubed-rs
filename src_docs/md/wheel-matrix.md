<!-- this_file: src_docs/md/wheel-matrix.md -->
# Wheel build matrix

Wheels are produced by [`maturin`](https://www.maturin.rs) via the
`PyO3/maturin-action` GitHub workflow (`.github/workflows/wheels.yml`). uubed-rs
is **not published to crates.io**; the wheel is the distribution artifact, bundled
into the `uubed` PyPI package.

## Platforms

| OS | Runner | Target | manylinux |
|---|---|---|---|
| Linux x86_64 | `ubuntu-latest` | `x86_64` | auto |
| Linux ARM64 | `ubuntu-latest` | `aarch64` | auto (cross) |
| macOS x86_64 | `macos-13` | `x86_64` | n/a |
| macOS ARM64 (Apple Silicon) | `macos-14` | `aarch64` | n/a |
| Windows x64 | `windows-latest` | `x64` | n/a |

A source distribution (`sdist`) is built separately.

## Build flags

All wheels are built with:

```bash
maturin build --release --features python,simd
```

Per-target CPU baselines are pinned in `pyproject.toml` under
`[tool.maturin.target.*]` so SIMD fast paths are enabled without sacrificing
portability:

| Target | `target-cpu` |
|---|---|
| `x86_64-unknown-linux-gnu` | `x86-64-v2` |
| `aarch64-unknown-linux-gnu` | `generic` |
| `x86_64-apple-darwin` | `x86-64-v2` |
| `aarch64-apple-darwin` | `apple-a14` |
| `x86_64-pc-windows-msvc` | `x86-64-v2` |

The release profile (`Cargo.toml`) additionally sets `lto = true`,
`codegen-units = 1`, `opt-level = 3`, and `strip = true`.

## Python versions

The bindings target CPython 3.8–3.12. Because no `abi3` build is configured yet,
each interpreter minor version produces its own wheel per platform. Adopting
`pyo3/abi3-py38` would reduce this to one wheel per platform.

## Local build

```bash
uvx maturin build --release        # uses pyproject [tool.maturin] features
# or explicitly:
uvx maturin build --release --features python,simd
```
