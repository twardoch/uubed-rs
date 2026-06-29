<!-- this_file: STYLE_GUIDE.md -->
# uubed-rs style guide

Conventions for contributing to the Rust core. Keep changes surgical and match
the surrounding code.

## File headers

Every source file starts with a `this_file` record giving its path relative to
the repository root:

```rust
// this_file: rust/src/encoders/q64.rs
```

Markdown files use the same record in an HTML comment on the first line.

## Formatting and linting (must pass before merge)

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --features simd -- -D warnings
```

CI enforces all three. Run `cargo fmt --all` to fix formatting; do not hand-format.

## Documentation comments

- Module-level docs use inner doc comments (`//!`).
  - **Exception:** files that are also pulled in with `include!` by an example or
    bench (`encoders/q64.rs`, `encoders/topk.rs`, `encoders/topk_optimized.rs`)
    use a plain `//` header, because an inner doc comment is rejected in that
    position. The reason is noted at the top of each such file.
- Public functions, types, and PyO3 exports get `///` doc comments. Document
  arguments, return values, errors, and performance characteristics where
  relevant.
- Explain *why*, not just *what* — especially for bit manipulation (e.g. the
  Z-order/Morton interleaving in `encoders/zorder.rs`).

## Error handling

- Fallible functions return `Result<_, Q64Error>` (or `UubedError` /
  `UubedResult` at the crate boundary). Never panic in library code.
- PyO3 bindings map `Err` to Python `ValueError` via `map_err`.

## Idioms

- Prefer iterators over index loops (`for (i, x) in xs.iter().enumerate()`),
  `slice::fill`, and `usize::div_ceil` over hand-rolled equivalents — clippy
  enforces these.
- Annotate integer literals (`let planes: usize = 32;`) when calling integer
  methods such as `div_ceil` so type inference stays unambiguous.
- Guard `unsafe` with a `# Safety` doc section explaining the invariants.

## Features

| Feature | Use |
|---|---|
| `python` | PyO3 bindings — required to build the wheel |
| `simd` | SIMD fast paths |
| `capi` | C ABI |

Build wheels with `--features python,simd`.

## Tests

- Unit tests live in `#[cfg(test)] mod tests` next to the code.
- Integration and property tests live in `rust/tests/`.
- Property tests that compare two implementations must only assert equality where
  the result is uniquely defined (discard ambiguous cases such as tied top-k
  boundaries).
- Add an assertion message to non-obvious asserts.
