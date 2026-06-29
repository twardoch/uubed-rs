# uubed-rs C API

This document explains how to use the uubed encoding library from C and C++ applications.

## Overview

The uubed-rs C API provides access to high-performance encoding algorithms for embeddings and vector data:

- **Q64**: Position-safe encoding with 2:1 expansion ratio
- **SimHash**: Locality-sensitive hashing for similarity preservation  
- **Top-K**: Compressed encoding of sparse embeddings
- **Z-order**: Morton encoding for spatial locality

## Features

- Thread-safe: All functions can be called concurrently
- Zero-copy operations: Buffer reuse for performance
- Error handling: Clear error codes and messages
- Memory management: Explicit allocation and cleanup
- Cross-platform: Linux, macOS, Windows support
- SIMD optimizations: Automatic detection and use

## Quick Start

### 1. Build the Library

```bash
make build
make examples
make test-c
```

### 2. Basic Usage

```c
#include "uubed.h"
#include <stdio.h>

int main() {
    const uint8_t data[] = {0x12, 0x34, 0x56, 0x78};
    char* encoded = NULL;
    
    UubedErrorCode result = uubed_q64_encode(data, 4, &encoded);
    if (result == UUBED_SUCCESS) {
        printf("Encoded: %s\n", encoded);
        
        uint8_t* decoded = NULL;
        size_t decoded_len = 0;
        
        result = uubed_q64_decode(encoded, &decoded, &decoded_len);
        if (result == UUBED_SUCCESS) {
            printf("Decoded %zu bytes\n", decoded_len);
            uubed_free_bytes(decoded, decoded_len);
        }
        
        uubed_free_string(encoded);
    } else {
        const char* error = uubed_get_last_error_message();
        fprintf(stderr, "Error: %s\n", error);
    }
    
    return 0;
}
```

### 3. Compilation

#### Using pkg-config (Recommended)

```bash
sudo make install
gcc myapp.c $(pkg-config --cflags --libs uubed) -o myapp
```

#### Manual Compilation

```bash
gcc myapp.c -I./include -L./rust/target/release -luubed_native -o myapp
```

## API Reference

### Core Functions

#### Q64 Encoding

```c
UubedErrorCode uubed_q64_encode(const uint8_t* data, size_t data_len, char** output);
UubedErrorCode uubed_q64_decode(const char* encoded, uint8_t** output, size_t* output_len);
UubedErrorCode uubed_q64_encode_to_buffer(
    const uint8_t* data, size_t data_len,
    uint8_t* output_buffer, size_t buffer_len,
    size_t* bytes_written
);
```

#### Advanced Encodings

```c
UubedErrorCode uubed_simhash_encode(
    const uint8_t* embedding, size_t embedding_len,
    unsigned int planes, char** output
);

UubedErrorCode uubed_topk_encode(
    const uint8_t* embedding, size_t embedding_len,
    unsigned int k, char** output
);

UubedErrorCode uubed_topk_encode_optimized(
    const uint8_t* embedding, size_t embedding_len,
    unsigned int k, char** output
);

UubedErrorCode uubed_zorder_encode(
    const uint8_t* embedding, size_t embedding_len,
    char** output
);
```

### Memory Management

```c
void uubed_free_string(char* s);
void uubed_free_bytes(uint8_t* bytes, size_t len);
```

### Error Handling

```c
const char* uubed_get_last_error_message(void);
void uubed_clear_last_error(void);
```

### Utility Functions

```c
const char* uubed_get_version(void);
int uubed_has_simd_support(void);

size_t uubed_max_embedding_size(void);      // 16MB
size_t uubed_max_k_value(void);             // 100,000
size_t uubed_max_simhash_planes(void);      // 8,192
```

## Error Codes

```c
typedef enum {
    UUBED_SUCCESS = 0,
    UUBED_Q64_ERROR = 1,
    UUBED_SIMHASH_ERROR = 2,
    UUBED_TOPK_ERROR = 3,
    UUBED_ZORDER_ERROR = 4,
    UUBED_VALIDATION_ERROR = 5,
    UUBED_MEMORY_ERROR = 6,
    UUBED_COMPUTATION_ERROR = 7,
    UUBED_INVALID_PARAMETER = 8,
    UUBED_BUFFER_TOO_SMALL = 9,
    UUBED_UNKNOWN_ERROR = 10
} UubedErrorCode;
```

## Performance Considerations

### Zero-Copy Operations

For repeated operations, use buffer-based functions:

```c
size_t input_len = 1000;
uint8_t* buffer = malloc(input_len * 2);
size_t bytes_written;

UubedErrorCode result = uubed_q64_encode_to_buffer(
    input_data, input_len, buffer, input_len * 2, &bytes_written
);

// Reuse buffer for multiple operations
// ...

free(buffer);
```

### SIMD Optimizations

The library automatically uses SIMD when available:

```c
if (uubed_has_simd_support()) {
    printf("SIMD optimizations available\n");
}
```

### Thread Safety

All functions are thread-safe. Each thread has its own error state:

```c
// Thread A
uubed_q64_encode(data1, len1, &result1);
const char* error_a = uubed_get_last_error_message();

// Thread B (independent error state)
uubed_q64_encode(data2, len2, &result2);  
const char* error_b = uubed_get_last_error_message();
```

## Algorithm Selection Guide

### Q64
- General-purpose encoding with position safety
- Output size: 2x input size
- Uses position-dependent alphabets to prevent corruption

### SimHash
- Similarity-preserving hashes
- Parameter: `planes` (64-256 recommended)
- Fixed output size based on plane count

### Top-K
- Sparse embeddings
- Parameter: `k` (number of largest values to keep)
- Use `_optimized` version for k > 16

### Z-order
- Spatial/coordinate data
- Preserves spatial locality
- Best for multi-dimensional coordinate-like data

## Installation

### System-wide Installation

```bash
make install

// Installs:
// Library: /usr/local/lib/libuubed_native.*
// Header: /usr/local/include/uubed.h
// pkg-config: /usr/local/lib/pkgconfig/uubed.pc
```

### Uninstallation

```bash
sudo make uninstall
```

## Examples

See `examples/c_api_demo.c` for a complete demonstration.

## Language Bindings

This C API enables bindings for:

- Node.js (N-API)
- Go (cgo)
- C++ (direct usage)
- Any language with C FFI support

## Troubleshooting

### Compilation Issues

1. Library not found: Check `LD_LIBRARY_PATH` or use `pkg-config`
2. Header not found: Verify `-I./include` path
3. Linking errors: Confirm library was built with `make build`

### Runtime Issues

1. Segmentation faults: Validate output pointers
2. Memory leaks: Call all `uubed_free_*` functions
3. Threading issues: Each thread has separate error state

### Performance Issues

1. Use zero-copy functions for repeated operations
2. Pre-allocate buffers instead of malloc/free cycles
3. Check SIMD support with `uubed_has_simd_support()`

## License

Follows the main uubed-rs project license.

## Contributing

See main README. C API code locations:
- `rust/src/capi.rs` - Implementation
- `include/uubed.h` - Header file  
- `examples/c_api_demo.c` - Examples
- `Makefile` - Build configuration