<!-- this_file: src_docs/md/pyo3-abi.md -->
# PyO3 ABI notes

The native extension is built with [PyO3](https://pyo3.rs) 0.22 and
[maturin](https://www.maturin.rs). The compiled module is `uubed_native`.

## Module surface

Functions exported to Python (see `rust/src/bindings.rs`):

| Python name | Returns | Notes |
|---|---|---|
| `q64_encode_native(data)` | `str` | Lossless Q64 encode |
| `q64_decode_native(encoded)` | `bytes` | Raises `ValueError` on bad input |
| `q64_encode_buffer_native(data)` | `bytes` | Zero-copy via buffer protocol |
| `q64_encode_batch_native(embeddings, reuse_buffers=True)` | `list[bytes]` | Batch with buffer pooling |
| `q64_encode_inplace_native(input, output)` | `int` | Caller-provided output buffer |
| `simhash_q64_native(embedding, planes=64)` | `str` | SimHash signature |
| `top_k_q64_native(embedding, k=8)` | `str` | Top-k indices |
| `top_k_q64_optimized_native(embedding, k=8)` | `str` | Optimized top-k |
| `z_order_q64_native(embedding)` | `str` | Z-order code |
| `mq64_encode_native(data, levels=None)` | `str` | Hierarchical encode |
| `mq64_decode_native(encoded)` | `bytes` | Hierarchical decode |
| `simhash_to_buffer_native(input, planes, output)` | `int` | Zero-copy SimHash |
| `top_k_to_buffer_native(input, k, output)` | `int` | Zero-copy top-k |
| `z_order_to_buffer_native(input, output)` | `int` | Zero-copy Z-order |
| `parallel_q64_encode_native(embeddings, num_threads=None)` | `list[str]` | Rayon batch |
| `parallel_simhash_encode_native(embeddings, planes, num_threads=None)` | `list[str]` | Rayon batch |
| `parallel_topk_encode_native(embeddings, k, num_threads=None)` | `list[str]` | Rayon batch |

Classes: `Q64StreamEncoder`, `Q64Stats`, `SimpleBatchProcessor`, `BufferPool`.
The module also exposes `__version__` (from `CARGO_PKG_VERSION`).

## ABI / build guidance

- **`extension-module`**: PyO3's `extension-module` feature is enabled via the
  crate's optional `pyo3` dependency, which is pulled in by the `python` Cargo
  feature. Always build the wheel with `--features python,simd`.
- **No `abi3` yet**: wheels are built per CPython minor version (3.8–3.12). The
  module therefore needs one wheel per interpreter/platform combination — see the
  [wheel build matrix](wheel-matrix.md). Enabling `pyo3/abi3-py38` later would
  collapse the matrix to one wheel per platform at a small performance cost.
- **Buffer protocol**: zero-copy entry points accept any object implementing the
  Python buffer protocol (NumPy arrays, `bytearray`, `memoryview`). Inputs are
  read-only views; output buffers must be writable and large enough
  (`len(input) * 2` for Q64).
- **GIL & threads**: parallel helpers run on a Rayon pool inside the call; the
  batch processor calls `Python::check_signals` so long runs stay interruptible.
- **Errors**: Rust `Result::Err` is converted to Python `ValueError` with the
  underlying message.
