# uubed-rs

Rust implementation of the QuadB64 family of position-safe embedding encodings.

## Overview

This library provides:

- **Q64 Encoding**: A core position-safe encoding algorithm
- **SIMD Optimizations**: Accelerated with AVX2, AVX-512, and NEON
- **Zero-Copy Operations**: Direct buffer access to reduce overhead
- **Multiple Methods**: SimHash, Top-k, and Z-order variants
- **PyO3 Bindings**: For fast Python integration

## Features

- **Position-Safe Encoding**: Prevents substring pollution in embeddings
- **Encoding Variants**: Eq64, Shq64, T8q64, Zoq64 for different needs
- **Performance**: 40–105x faster than pure Python
- **Memory Efficiency**: Uses buffer pooling and avoids unnecessary copies
- **Cross-Platform**: Supports Linux, macOS, Windows
- **Multi-Architecture**: Optimized SIMD for x86_64 and ARM64

## Installation

uubed-rs is the native core consumed by the [`uubed`](https://pypi.org/project/uubed/)
Python package — installing `uubed` pulls in this compiled extension:

```bash
pip install uubed
```

It is **not published to crates.io** (the name 404s) and there is no standalone
`uubed-rs` PyPI package to install directly. To build the wheel yourself:

```bash
# enables the PyO3 bindings (`python`) and SIMD fast paths
maturin build --release --features python,simd
```

Or use it as a Rust crate via a git/path dependency (`uubed-core`, library name
`uubed_native`).

## Usage

```python
import uubed_rs

# Basic encoding
data = b"hello world"
encoded = uubed_rs.q64_encode_native(data)

# In-place encoding with NumPy buffers
import numpy as np
input_buffer = np.frombuffer(data, dtype=np.uint8)
output_buffer = np.zeros(len(data) * 2, dtype=np.uint8)
written = uubed_rs.q64_encode_inplace_native(input_buffer, output_buffer)
```

## Performance

- **Q64 Encoding**: Up to 105x faster than Python
- **SimHash**: 1.7–9.7x speedup
- **Z-order**: 60–1600x improvement
- **Memory**: 50–90% less usage via buffer pooling

## Development

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test            # unit, integration, and property tests
cargo test --doc
```

CI (`.github/workflows/ci.yml`) runs fmt, clippy, and the test matrix across
Linux/macOS/Windows; `.github/workflows/wheels.yml` builds the maturin wheel
matrix. See [`STYLE_GUIDE.md`](STYLE_GUIDE.md) for conventions.

## Documentation

MaterialX (MkDocs + Material) sources live in `src_docs/` and render to `docs/`:

```bash
mkdocs build -f src_docs/mkdocs.yaml
```

Topics: Rust crate API, PyO3 ABI notes, and the wheel build matrix. Generate the
Rust API reference with `cargo doc --no-deps --open`.

## License

MIT – see LICENSE for details.

## Related Projects

- [uubed](https://github.com/twardoch/uubed) – Main project
- [uubed-py](https://github.com/twardoch/uubed-py) – Python version
- [uubed-docs](https://github.com/twardoch/uubed-docs) – Documentation