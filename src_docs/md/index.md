<!-- this_file: src_docs/md/index.md -->
# uubed-rs

The Rust core of the [uubed](https://github.com/twardoch/uubed) project: a fast,
position-safe encoder for embedding vectors. It powers the `uubed` Python package
through a PyO3/maturin native extension and can also be used directly as a Rust
crate (`uubed-core`, library name `uubed_native`).

## Why position-safe encoding

Search engines and substring indexes happily match fragments of a Base64 string,
which pollutes results when embeddings are stored as text. The **QuadB64** family
breaks that by rotating through four position-dependent alphabets, so a code only
decodes correctly when read from its true offset. A fragment lifted out of
context simply isn't valid.

## Encoding variants

| Variant | Function | Purpose |
|---|---|---|
| Eq64 | `q64_encode` / `q64_decode` | Lossless, position-safe byte encoding |
| Shq64 | `simhash_q64` | Locality-sensitive SimHash signature |
| T8q64 | `top_k_q64` / `top_k_q64_optimized` | Sparse top-k index encoding |
| Zoq64 | `z_order_q64` | Z-order (Morton) spatial-locality code |
| Mq64 | `mq64_encode` / `mq64_decode` | Hierarchical (multi-level) QuadB64 |

## Cargo features

| Feature | Effect |
|---|---|
| `python` | Build the PyO3 bindings (`uubed_native` module) |
| `simd` | Enable SIMD fast paths (x86_64 SSE2 today) |
| `capi` | Build the C ABI (`staticlib`/`cdylib`) for FFI |

## Project status

uubed-rs is **not published to crates.io**; it ships as a native wheel bundled
with the `uubed` PyPI package. See the [wheel build matrix](wheel-matrix.md) for
how those artifacts are produced.

## Documentation map

- [Rust crate API](rust-api.md) — public functions, types, and usage.
- [PyO3 ABI notes](pyo3-abi.md) — Python module surface and ABI guidance.
- [Wheel build matrix](wheel-matrix.md) — supported platforms and build flags.
