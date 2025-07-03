# uubed-rs C API

This document describes how to use the uubed encoding library from C and C++ applications.

## Overview

The uubed-rs C API provides access to high-performance encoding algorithms specifically designed for embeddings and vector data:

- **Q64**: Position-safe encoding with 2:1 expansion ratio
- **SimHash**: Locality-sensitive hashing for similarity preservation  
- **Top-K**: Compressed encoding of sparse embeddings
- **Z-order**: Morton encoding for spatial locality

## Features

- ✅ **Thread-safe**: All functions can be called concurrently
- ✅ **Zero-copy operations**: Buffer reuse for high performance
- ✅ **Comprehensive error handling**: Detailed error messages
- ✅ **Memory management**: RAII-style cleanup with proper free functions
- ✅ **Cross-platform**: Linux, macOS, Windows support
- ✅ **SIMD optimizations**: Automatic runtime detection when available

## Quick Start

### 1. Build the Library

```bash
# Build the C API
make build

# Build examples
make examples

# Run tests
make test-c
```

### 2. Basic Usage

```c
#include "uubed.h"
#include <stdio.h>

int main() {
    // Sample data
    const uint8_t data[] = {0x12, 0x34, 0x56, 0x78};
    char* encoded = NULL;
    
    // Encode
    UubedErrorCode result = uubed_q64_encode(data, 4, &encoded);
    if (result == UUBED_SUCCESS) {
        printf("Encoded: %s\n", encoded);
        
        // Decode
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
# Install the library first
sudo make install

# Compile your application
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
// Basic encoding (allocates result string)
UubedErrorCode uubed_q64_encode(const uint8_t* data, size_t data_len, char** output);

// Basic decoding  
UubedErrorCode uubed_q64_decode(const char* encoded, uint8_t** output, size_t* output_len);

// Zero-copy encoding (uses pre-allocated buffer)
UubedErrorCode uubed_q64_encode_to_buffer(
    const uint8_t* data, size_t data_len,
    uint8_t* output_buffer, size_t buffer_len,
    size_t* bytes_written
);
```

#### Advanced Encodings

```c
// SimHash encoding for locality-sensitive hashing
UubedErrorCode uubed_simhash_encode(
    const uint8_t* embedding, size_t embedding_len,
    unsigned int planes, char** output
);

// Top-K encoding for sparse embeddings
UubedErrorCode uubed_topk_encode(
    const uint8_t* embedding, size_t embedding_len,
    unsigned int k, char** output
);

// Optimized Top-K for large embeddings/k values
UubedErrorCode uubed_topk_encode_optimized(
    const uint8_t* embedding, size_t embedding_len,
    unsigned int k, char** output
);

// Z-order encoding for spatial data
UubedErrorCode uubed_zorder_encode(
    const uint8_t* embedding, size_t embedding_len,
    char** output
);
```

### Memory Management

```c
// Free strings returned by encoding functions
void uubed_free_string(char* s);

// Free byte arrays returned by decoding functions  
void uubed_free_bytes(uint8_t* bytes, size_t len);
```

### Error Handling

```c
// Get human-readable error message (thread-local)
const char* uubed_get_last_error_message(void);

// Clear error state
void uubed_clear_last_error(void);
```

### Utility Functions

```c
// Library information
const char* uubed_get_version(void);
int uubed_has_simd_support(void);

// Limits
size_t uubed_max_embedding_size(void);      // 16MB
size_t uubed_max_k_value(void);             // 100,000
size_t uubed_max_simhash_planes(void);      // 8,192
```

## Error Codes

```c
typedef enum {
    UUBED_SUCCESS = 0,              // Operation succeeded
    UUBED_Q64_ERROR = 1,            // Q64 encoding/decoding error
    UUBED_SIMHASH_ERROR = 2,        // SimHash computation error
    UUBED_TOPK_ERROR = 3,           // Top-k selection error
    UUBED_ZORDER_ERROR = 4,         // Z-order encoding error
    UUBED_VALIDATION_ERROR = 5,     // Input validation error
    UUBED_MEMORY_ERROR = 6,         // Memory allocation error
    UUBED_COMPUTATION_ERROR = 7,    // Internal computation error
    UUBED_INVALID_PARAMETER = 8,    // Invalid parameter
    UUBED_BUFFER_TOO_SMALL = 9,     // Buffer too small
    UUBED_UNKNOWN_ERROR = 10        // Unknown error
} UubedErrorCode;
```

## Performance Considerations

### Zero-Copy Operations

For high-performance scenarios, use buffer-based functions:

```c
// Pre-allocate buffer (2x input size for Q64)
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

The library automatically detects and uses SIMD instructions when available:

```c
if (uubed_has_simd_support()) {
    printf("SIMD optimizations available\n");
}
```

### Thread Safety

All functions are thread-safe. Error messages are stored per-thread:

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
- **Use for**: General-purpose encoding with position safety
- **Output size**: 2x input size
- **Features**: Position-dependent alphabets prevent corruption

### SimHash
- **Use for**: Similarity-preserving hashes
- **Parameters**: `planes` (64-256 recommended)
- **Output size**: Fixed based on plane count

### Top-K
- **Use for**: Sparse embeddings
- **Parameters**: `k` (number of largest values to keep)
- **Optimization**: Use `_optimized` version for k > 16

### Z-order
- **Use for**: Spatial/coordinate data
- **Features**: Preserves spatial locality
- **Best for**: Multi-dimensional coordinate-like data

## Installation

### System-wide Installation

```bash
# Build and install
make install

# This installs:
# - Library: /usr/local/lib/libuubed_native.*
# - Header: /usr/local/include/uubed.h
# - pkg-config: /usr/local/lib/pkgconfig/uubed.pc
```

### Uninstallation

```bash
sudo make uninstall
```

## Examples

See `examples/c_api_demo.c` for a comprehensive demonstration of all features.

## Language Bindings

This C API enables bindings for other languages:

- **Node.js**: Use N-API with the C library
- **Go**: Use cgo to interface with the C API
- **C++**: Direct usage with C++ applications
- **Other languages**: Any language with C FFI support

## Troubleshooting

### Compilation Issues

1. **Library not found**: Check `LD_LIBRARY_PATH` or use `pkg-config`
2. **Header not found**: Ensure `-I./include` points to the header location
3. **Linking errors**: Verify library was built with `make build`

### Runtime Issues

1. **Segmentation faults**: Check that all output pointers are valid
2. **Memory leaks**: Ensure all `uubed_free_*` functions are called
3. **Threading issues**: Each thread maintains separate error state

### Performance Issues

1. **Use zero-copy functions** for repeated operations
2. **Pre-allocate buffers** instead of repeated malloc/free
3. **Check SIMD support** with `uubed_has_simd_support()`

## License

This C API is part of the uubed-rs project and follows the same license terms.

## Contributing

See the main project README for contribution guidelines. The C API code is located in:
- `rust/src/capi.rs` - Implementation
- `include/uubed.h` - Header file  
- `examples/c_api_demo.c` - Examples
- `Makefile` - Build configuration