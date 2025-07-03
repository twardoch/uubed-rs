# uubed-rs

High-performance Rust core for position-safe embedding encoding (QuadB64 family).

## Overview

This repository contains the Rust implementation of the uubed encoding library, providing:

- **Q64 Encoding**: Core position-safe encoding algorithm
- **SIMD Optimizations**: AVX2/AVX-512/NEON acceleration for maximum performance  
- **Zero-Copy Operations**: Direct buffer access for minimal overhead
- **Multiple Encoding Methods**: SimHash, Top-k, Z-order variants
- **PyO3 Bindings**: High-performance Python integration

## Features

- **Position-Safe Encoding**: Eliminates substring pollution in embeddings
- **Multiple Variants**: Eq64, Shq64, T8q64, Zoq64 for different use cases
- **High Performance**: 40-105x speedup over pure Python implementations
- **Memory Efficient**: Buffer pooling and zero-copy operations
- **Cross-Platform**: Linux, macOS, Windows support
- **Multi-Architecture**: x86_64, ARM64 with optimized SIMD

## Installation

Install via pip:

```bash
pip install uubed-rs
```

Or build from source:

```bash
maturin build --release --features simd
```

## Usage

```python
import uubed_rs

# Basic encoding
data = b"hello world"
encoded = uubed_rs.q64_encode_native(data)

# Zero-copy with buffers
import numpy as np
input_buffer = np.frombuffer(data, dtype=np.uint8)
output_buffer = np.zeros(len(data) * 2, dtype=np.uint8)
written = uubed_rs.q64_encode_inplace_native(input_buffer, output_buffer)
```

## Performance

- **Q64 Encoding**: Up to 105x faster than Python
- **SimHash**: 1.7-9.7x speedup with Rust implementation
- **Z-order**: 60-1600x performance improvement
- **Memory Usage**: 50-90% reduction through buffer pooling

## License

MIT License - see LICENSE file for details.

## Related Projects

- [uubed](https://github.com/twardoch/uubed) - Main project coordination
- [uubed-py](https://github.com/twardoch/uubed-py) - Python implementation
- [uubed-docs](https://github.com/twardoch/uubed-docs) - Comprehensive documentation