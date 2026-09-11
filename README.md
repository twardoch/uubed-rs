---
this_file: README.md
---
# Uubed Rust workspace

One native implementation for compact byte encoding and translation-memory search.

| Crate | Responsibility |
| --- | --- |
| `uubed-core` | Existing tested byte encoders, optional SIMD and C API; no Python dependency. |
| `uubed-tm` | SQLite source/pair storage, exact matches and signed int8 cosine search. |
| `uubed-python` | PyO3 bindings for both, published as the `uubed-rs` Python distribution. |

The extension is imported as `uubed_native`. Model inference, model identity,
TMX streaming and atomic index publication live in the companion `uubed` Python
package. There is no second Candle/Qwen model runtime or fixed-1024 database.

```bash
cargo test --workspace
cargo clippy --workspace --all-targets --features simd -- -D warnings
uvx maturin build --release
```

Requires Rust 1.88+; the existing core remains edition 2021. Python integration is
tested on 3.12. Native binding behavior is tested through the built wheel, rather
than linking a Rust executable against a Python extension module.

SQLite stores BINARY-collated source strings, all target variants and their path/unit
provenance. One int8 BLOB per English source has its positive scale and norm stored
alongside it. Metadata records the format and full embedding-space identity.
Query vectors must match the dimensions and contain finite, nonzero values.
Search ranks cosine similarities after filtering by target language. Scale cancels
in cosine, so no float32 corpus matrix is materialized. Search is exhaustive O(Nd),
not an approximate-nearest-neighbor service.

See [the workspace guide](../../README.md) for the complete two-consumer workflow.

## Releases and local data

`./publish.sh --dry-run` verifies the next release without pushing or uploading.
`./publish.sh` commits, tags and publishes it. See [RELEASING.md](RELEASING.md)
for credentials, same-tag retries, dependency order and private-data exclusions.
