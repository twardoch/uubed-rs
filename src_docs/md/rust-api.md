<!-- this_file: src_docs/md/rust-api.md -->
# Rust crate API

The crate is `uubed-core`; the library name (and PyO3 module) is `uubed_native`.
Add it as a path/git dependency (it is not on crates.io):

```toml
[dependencies]
uubed-core = { git = "https://github.com/twardoch/uubed", features = ["simd"] }
```

Generate the full API reference locally with:

```bash
cargo doc --no-deps --open
```

## Core encoders (`uubed_native::encoders`)

### Q64 (Eq64) — lossless, position-safe

```rust
use uubed_native::{q64_encode, q64_decode};

let data = b"hello world";
let code = q64_encode(data);          // String, twice the input length
let back = q64_decode(&code).unwrap(); // Vec<u8>, exact round-trip
assert_eq!(&back, data);
```

`q64_encode` emits two characters per input byte, each drawn from one of four
position-dependent alphabets (`base_pos % 4`). `q64_decode` validates both the
character set and the position of every character, returning `Q64Error` on a
malformed or out-of-position string.

Zero-copy variant for hot loops:

```rust
use uubed_native::encoders::q64_encode_to_buffer;

let data = [0x12u8, 0x34, 0x56];
let mut out = vec![0u8; data.len() * 2];
let written = q64_encode_to_buffer(&data, &mut out).unwrap();
assert_eq!(written, out.len());
```

### SimHash (Shq64)

```rust
use uubed_native::encoders::simhash_q64;
let sig = simhash_q64(&embedding, 64); // 64 hyperplanes -> 16-char code
```

Projects the embedding onto a fixed set of random hyperplanes (seeded for
reproducibility, cached per shape) and Q64-encodes the sign bits. Similar inputs
yield codes with small Hamming distance.

### Top-k (T8q64)

```rust
use uubed_native::encoders::{top_k_q64, top_k_q64_optimized};
let code = top_k_q64_optimized(&embedding, 8); // indices of the 8 largest values
```

### Z-order / Morton (Zoq64)

```rust
use uubed_native::encoders::z_order_q64;
let code = z_order_q64(&embedding); // interleaves top bits per dimension
```

Bit interleaving places neighbouring dimensions in neighbouring bit-pairs so that
spatially close vectors share long common prefixes — useful for range scans.

### Hierarchical (Mq64)

```rust
use uubed_native::{mq64_encode, mq64_decode};
let code = mq64_encode(&data);
let back = mq64_decode(&code).unwrap();
```

## Error handling

Encoders that can fail return `Result<_, Q64Error>` (or the richer
`uubed_native::UubedError` / `UubedResult` at the crate boundary). Decoding
reports odd-length input, invalid characters, and characters that appear at the
wrong position.

## Parallel batch helpers (`uubed_native::parallel`)

`parallel_q64_encode`, `parallel_simhash_encode`, `parallel_topk_encode`, and
`parallel_zorder_encode` encode a slice of embeddings across a Rayon thread pool;
`BatchProcessor` adds chunked, order-preserving processing for large batches.
