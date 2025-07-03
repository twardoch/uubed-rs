# Testing Guide for uubed-rs

This document describes the comprehensive testing strategy implemented for the uubed-rs Rust codebase.

## Testing Layers

### 1. Unit Tests

**Location**: Embedded in source files with `#[cfg(test)]`

**Purpose**: Test individual functions and modules in isolation

**Run with**:
```bash
cargo test --lib
```

**Coverage**:
- Q64 encoding/decoding roundtrip tests
- Top-k algorithm correctness
- SimHash determinism and locality
- Z-order encoding consistency
- Error handling edge cases

### 2. Integration Tests

**Location**: `tests/` directory

**Purpose**: Test interactions between modules and end-to-end workflows

**Files**:
- `integration_test.rs` - Full pipeline testing
- `thread_safety.rs` - Concurrent access testing (if PyO3 linking works)

**Run with**:
```bash
cargo test --tests
```

### 3. Property-Based Tests

**Location**: `tests/property_tests.rs`

**Purpose**: Test properties that should hold for any input using QuickCheck

**Key Properties Tested**:
- **Q64 Roundtrip**: `encode(decode(x)) == x` for all valid inputs
- **Determinism**: Same input always produces same output
- **Length Relationships**: Output length proportional to input
- **Consistency**: Original and optimized implementations match
- **Error Handling**: Functions handle invalid inputs gracefully

**Run with**:
```bash
cargo test property_
```

**Example Properties**:
```rust
#[quickcheck]
fn prop_q64_roundtrip(data: Vec<u8>) -> bool {
    let encoded = q64_encode(&data);
    q64_decode(&encoded).unwrap() == data
}

#[quickcheck]
fn prop_topk_optimized_matches_original(embedding: ValidEmbedding, k: ValidK) -> bool {
    let emb = &embedding.0;
    let k = k.0.min(emb.len());
    top_k_q64(emb, k) == topk_optimized::top_k_q64_optimized(emb, k)
}
```

### 4. Fuzzing Tests

**Location**: `fuzz/` directory

**Purpose**: Discover edge cases and ensure robustness with random inputs

**Targets**:
- `q64_roundtrip` - Tests Q64 encoding/decoding cycle
- `q64_decode` - Tests Q64 decoding with arbitrary strings
- `topk_fuzz` - Tests Top-k algorithms with arbitrary inputs
- `simhash_fuzz` - Tests SimHash with various parameters
- `zorder_fuzz` - Tests Z-order encoding robustness

**Install cargo-fuzz**:
```bash
cargo install cargo-fuzz
```

**Run fuzzing**:
```bash
cd fuzz
cargo fuzz run q64_roundtrip
cargo fuzz run topk_fuzz
# ... etc
```

**Benefits**:
- Finds edge cases not covered by manual tests
- Ensures no panics on malformed input
- Validates consistency between implementations

### 5. Performance Benchmarks

**Location**: `benches/` directory

**Purpose**: Measure performance and detect regressions

**Benchmark Suites**:
- `topk_bench.rs` - Compare original vs optimized Top-k
- `memory_bench.rs` - Memory usage profiling
- `large_embedding_bench.rs` - Scale testing with very large data

**Run with**:
```bash
cargo bench
cargo bench --bench topk_bench
```

**Memory Profiling**:
Uses custom tracking allocator to measure peak memory usage:
```rust
#[global_allocator]
static ALLOCATOR: TrackingAllocator = TrackingAllocator::new();
```

### 6. SIMD Testing

**Location**: `src/simd.rs` tests

**Purpose**: Verify SIMD implementations match scalar results

**Features**:
- Runtime CPU capability detection
- Automatic fallback to scalar implementations
- Consistency verification across architectures

**Run with**:
```bash
cargo test simd
```

## Error Handling Testing

### Comprehensive Error Types

**Location**: `src/error.rs`

**Coverage**:
- Input validation errors
- Computation failures
- Memory allocation issues
- Platform-specific errors

**Error Recovery Testing**:
```rust
// Test automatic input cleaning
let dirty_input = "abc!@#def$%^";
let cleaned = recovery::recover_q64_decode(dirty_input)?;
assert_eq!(cleaned, "abcdef");

// Test parameter clamping
assert_eq!(recovery::clamp_k_value(0, 100), 1);
assert_eq!(recovery::clamp_k_value(1000, 500), 500);
```

## Test Data Generation

### QuickCheck Generators

Custom `Arbitrary` implementations for test data:

```rust
#[derive(Clone, Debug)]
struct ValidEmbedding(Vec<u8>);

impl Arbitrary for ValidEmbedding {
    fn arbitrary(g: &mut Gen) -> Self {
        let size = usize::arbitrary(g) % 10000 + 1;
        let embedding = (0..size).map(|_| u8::arbitrary(g)).collect();
        ValidEmbedding(embedding)
    }
}
```

### Fuzzing Input Generation

Uses `arbitrary` crate for structured fuzzing:

```rust
#[derive(Debug)]
struct TopkInput {
    embedding: Vec<u8>,
    k: usize,
}

impl<'a> Arbitrary<'a> for TopkInput {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let embedding_size = u.int_in_range(1..=10000)?;
        // ... generate valid test cases
    }
}
```

## Continuous Integration

### Test Commands

For CI/CD pipelines:

```bash
# Run all tests
cargo test --all

# Run property tests with more iterations
QUICKCHECK_TESTS=10000 cargo test property_

# Run benchmarks (for performance regression detection)
cargo bench --no-run

# Run with different feature flags
cargo test --features simd
cargo test --no-default-features
```

### Coverage

Generate test coverage reports:

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin --out html --output-dir coverage/
```

## Performance Testing Strategy

### 1. Micro-benchmarks
- Individual function performance
- Memory allocation patterns
- SIMD vs scalar comparisons

### 2. Integration Benchmarks
- End-to-end pipeline performance
- Concurrent access patterns
- Large data handling

### 3. Regression Testing
- Automated performance monitoring
- Alerts for significant performance changes
- Historical performance tracking

## Test Organization

```
rust/
├── src/
│   ├── *.rs                 # Unit tests embedded
│   └── simd.rs             # SIMD consistency tests
├── tests/
│   ├── integration_test.rs  # End-to-end testing
│   ├── property_tests.rs    # QuickCheck properties
│   └── thread_safety.rs     # Concurrency testing
├── benches/
│   ├── topk_bench.rs       # Performance comparisons
│   ├── memory_bench.rs     # Memory profiling
│   └── large_embedding_bench.rs # Scale testing
└── fuzz/
    ├── Cargo.toml          # Fuzzing dependencies
    └── fuzz_targets/       # Individual fuzz targets
        ├── q64_roundtrip.rs
        ├── topk_fuzz.rs
        └── ...
```

## Test Quality Metrics

### Coverage Goals
- **Unit Tests**: >95% line coverage
- **Integration Tests**: All public APIs exercised
- **Property Tests**: Core invariants verified
- **Fuzzing**: 24+ hours of continuous fuzzing per release

### Performance Benchmarks
- **Regression Threshold**: <5% performance degradation
- **Memory Usage**: No memory leaks detected
- **Scaling**: Linear or sub-linear complexity verified

### Error Handling
- **Error Paths**: All error conditions tested
- **Recovery**: Graceful degradation verified
- **Robustness**: No panics on invalid input

## Running the Full Test Suite

```bash
# Complete test run
cargo test --all
cargo test property_ --release
cd fuzz && cargo fuzz run q64_roundtrip -- -max_total_time=300
cargo bench --all

# Quick test run
cargo test --lib
cargo test integration_test

# Performance-focused run
cargo test --release
cargo bench --bench topk_bench
```

This comprehensive testing strategy ensures the uubed-rs implementation is correct, performant, and robust across all supported platforms and use cases.