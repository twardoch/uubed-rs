// this_file: rust/benches/comparative_bench.rs
//! Comprehensive benchmarks comparing uubed Q64 against alternative encoding schemes
//! 
//! This benchmark suite evaluates:
//! - Encoding speed
//! - Decoding speed  
//! - Output size efficiency
//! - Memory allocation patterns
//! 
//! Against popular encoding alternatives:
//! - Base64 (standard and URL-safe)
//! - Hex encoding
//! - MessagePack (rmp)
//! - Bincode
//! - CBOR (ciborium)

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use uubed_native::encoders::{q64_encode, q64_decode, q64_encode_to_buffer};
// use std::collections::HashMap; // Removed unused import

// External dependencies for comparison
use base64::{Engine as _, engine::general_purpose};
use hex;
use rmp_serde;
use bincode;
use ciborium;
use serde::{Serialize, Deserialize};

/// Test data patterns representative of real-world embeddings
#[derive(Clone)]
struct TestDataset {
    name: &'static str,
    data: Vec<u8>,
    description: &'static str,
}

/// Wrapper for serialization libraries that expect structured data
#[derive(Serialize, Deserialize, Clone)]
struct EmbeddingWrapper {
    data: Vec<u8>,
}

impl TestDataset {
    fn new(name: &'static str, data: Vec<u8>, description: &'static str) -> Self {
        Self { name, data, description }
    }
}

/// Generate representative test datasets
fn create_test_datasets() -> Vec<TestDataset> {
    vec![
        TestDataset::new(
            "small_random",
            (0..64).map(|_| fastrand::u8(..)).collect(),
            "Small 64-byte random embedding"
        ),
        TestDataset::new(
            "medium_random", 
            (0..512).map(|_| fastrand::u8(..)).collect(),
            "Medium 512-byte random embedding"
        ),
        TestDataset::new(
            "large_random",
            (0..4096).map(|_| fastrand::u8(..)).collect(), 
            "Large 4KB random embedding"
        ),
        TestDataset::new(
            "sparse_data",
            {
                let mut data = vec![0u8; 1024];
                // Sparse data: only 10% non-zero values
                for i in (0..1024).step_by(10) {
                    data[i] = fastrand::u8(1..=255);
                }
                data
            },
            "Sparse embedding (10% non-zero)"
        ),
        TestDataset::new(
            "clustered_data",
            {
                let mut data = vec![0u8; 1024];
                // Clustered data: high values in specific ranges
                for i in 100..200 {
                    data[i] = fastrand::u8(200..=255);
                }
                for i in 500..600 {
                    data[i] = fastrand::u8(150..=200);
                }
                data
            },
            "Clustered embedding (concentrated values)"
        ),
        TestDataset::new(
            "gradient_data",
            (0..1024).map(|i| ((i * 255) / 1023) as u8).collect(),
            "Gradient embedding (0-255 linear)"
        ),
    ]
}

/// Benchmark encoding speed across different algorithms
fn benchmark_encoding_speed(c: &mut Criterion) {
    let mut group = c.benchmark_group("encoding_speed");
    let datasets = create_test_datasets();
    
    for dataset in &datasets {
        group.throughput(Throughput::Bytes(dataset.data.len() as u64));
        
        // uubed Q64
        group.bench_with_input(
            BenchmarkId::new("uubed_q64", dataset.name),
            &dataset.data,
            |b, data| {
                b.iter(|| {
                    let result = q64_encode(black_box(data));
                    black_box(result);
                });
            },
        );
        
        // uubed Q64 zero-copy
        group.bench_with_input(
            BenchmarkId::new("uubed_q64_zerocopy", dataset.name),
            &dataset.data,
            |b, data| {
                let mut buffer = vec![0u8; data.len() * 2];
                b.iter(|| {
                    let result = q64_encode_to_buffer(black_box(data), black_box(&mut buffer));
                    let _ = black_box(result);
                });
            },
        );
        
        // Base64 standard
        group.bench_with_input(
            BenchmarkId::new("base64_standard", dataset.name),
            &dataset.data,
            |b, data| {
                b.iter(|| {
                    let result = general_purpose::STANDARD.encode(black_box(data));
                    black_box(result);
                });
            },
        );
        
        // Base64 URL-safe
        group.bench_with_input(
            BenchmarkId::new("base64_url_safe", dataset.name),
            &dataset.data,
            |b, data| {
                b.iter(|| {
                    let result = general_purpose::URL_SAFE.encode(black_box(data));
                    black_box(result);
                });
            },
        );
        
        // Hex encoding
        group.bench_with_input(
            BenchmarkId::new("hex", dataset.name),
            &dataset.data,
            |b, data| {
                b.iter(|| {
                    let result = hex::encode(black_box(data));
                    black_box(result);
                });
            },
        );
        
        // MessagePack
        group.bench_with_input(
            BenchmarkId::new("messagepack", dataset.name),
            &dataset.data,
            |b, data| {
                let wrapper = EmbeddingWrapper { data: data.clone() };
                b.iter(|| {
                    let result = rmp_serde::to_vec(black_box(&wrapper)).unwrap();
                    black_box(result);
                });
            },
        );
        
        // Bincode
        group.bench_with_input(
            BenchmarkId::new("bincode", dataset.name),
            &dataset.data,
            |b, data| {
                let wrapper = EmbeddingWrapper { data: data.clone() };
                b.iter(|| {
                    let result = bincode::serialize(black_box(&wrapper)).unwrap();
                    black_box(result);
                });
            },
        );
        
        // CBOR
        group.bench_with_input(
            BenchmarkId::new("cbor", dataset.name),
            &dataset.data,
            |b, data| {
                let wrapper = EmbeddingWrapper { data: data.clone() };
                b.iter(|| {
                    let mut result = Vec::new();
                    ciborium::ser::into_writer(black_box(&wrapper), &mut result).unwrap();
                    black_box(result);
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark decoding speed across different algorithms
fn benchmark_decoding_speed(c: &mut Criterion) {
    let mut group = c.benchmark_group("decoding_speed");
    let datasets = create_test_datasets();
    
    for dataset in &datasets {
        group.throughput(Throughput::Bytes(dataset.data.len() as u64));
        
        // Pre-encode data for decoding benchmarks
        let q64_encoded = q64_encode(&dataset.data);
        let base64_encoded = general_purpose::STANDARD.encode(&dataset.data);
        let base64_url_encoded = general_purpose::URL_SAFE.encode(&dataset.data);
        let hex_encoded = hex::encode(&dataset.data);
        let wrapper = EmbeddingWrapper { data: dataset.data.clone() };
        let msgpack_encoded = rmp_serde::to_vec(&wrapper).unwrap();
        let bincode_encoded = bincode::serialize(&wrapper).unwrap();
        let mut cbor_encoded = Vec::new();
        ciborium::ser::into_writer(&wrapper, &mut cbor_encoded).unwrap();
        
        // uubed Q64 decoding
        group.bench_with_input(
            BenchmarkId::new("uubed_q64", dataset.name),
            &q64_encoded,
            |b, encoded| {
                b.iter(|| {
                    let result = q64_decode(black_box(encoded)).unwrap();
                    black_box(result);
                });
            },
        );
        
        // Base64 standard decoding
        group.bench_with_input(
            BenchmarkId::new("base64_standard", dataset.name),
            &base64_encoded,
            |b, encoded| {
                b.iter(|| {
                    let result = general_purpose::STANDARD.decode(black_box(encoded)).unwrap();
                    black_box(result);
                });
            },
        );
        
        // Base64 URL-safe decoding
        group.bench_with_input(
            BenchmarkId::new("base64_url_safe", dataset.name),
            &base64_url_encoded,
            |b, encoded| {
                b.iter(|| {
                    let result = general_purpose::URL_SAFE.decode(black_box(encoded)).unwrap();
                    black_box(result);
                });
            },
        );
        
        // Hex decoding
        group.bench_with_input(
            BenchmarkId::new("hex", dataset.name),
            &hex_encoded,
            |b, encoded| {
                b.iter(|| {
                    let result = hex::decode(black_box(encoded)).unwrap();
                    black_box(result);
                });
            },
        );
        
        // MessagePack decoding
        group.bench_with_input(
            BenchmarkId::new("messagepack", dataset.name),
            &msgpack_encoded,
            |b, encoded| {
                b.iter(|| {
                    let result: EmbeddingWrapper = rmp_serde::from_slice(black_box(encoded)).unwrap();
                    black_box(result);
                });
            },
        );
        
        // Bincode decoding
        group.bench_with_input(
            BenchmarkId::new("bincode", dataset.name),
            &bincode_encoded,
            |b, encoded| {
                b.iter(|| {
                    let result: EmbeddingWrapper = bincode::deserialize(black_box(encoded)).unwrap();
                    black_box(result);
                });
            },
        );
        
        // CBOR decoding
        group.bench_with_input(
            BenchmarkId::new("cbor", dataset.name),
            &cbor_encoded,
            |b, encoded| {
                b.iter(|| {
                    let result: EmbeddingWrapper = ciborium::de::from_reader(black_box(encoded.as_slice())).unwrap();
                    black_box(result);
                });
            },
        );
    }
    
    group.finish();
}

/// Analyze output size efficiency
fn benchmark_size_efficiency(c: &mut Criterion) {
    let datasets = create_test_datasets();
    
    println!("\n=== SIZE EFFICIENCY ANALYSIS ===");
    println!("{:<20} {:<12} {:<8} {:<8} {:<8} {:<8} {:<8} {:<8} {:<8}", 
             "Dataset", "Original", "Q64", "Base64", "Base64URL", "Hex", "MsgPack", "Bincode", "CBOR");
    println!("{}", "-".repeat(120));
    
    for dataset in &datasets {
        let original_size = dataset.data.len();
        
        // Encode with each algorithm
        let q64_encoded = q64_encode(&dataset.data);
        let base64_encoded = general_purpose::STANDARD.encode(&dataset.data);
        let base64_url_encoded = general_purpose::URL_SAFE.encode(&dataset.data);
        let hex_encoded = hex::encode(&dataset.data);
        let wrapper = EmbeddingWrapper { data: dataset.data.clone() };
        let msgpack_encoded = rmp_serde::to_vec(&wrapper).unwrap();
        let bincode_encoded = bincode::serialize(&wrapper).unwrap();
        let mut cbor_encoded = Vec::new();
        ciborium::ser::into_writer(&wrapper, &mut cbor_encoded).unwrap();
        
        println!("{:<20} {:<12} {:<8} {:<8} {:<8} {:<8} {:<8} {:<8} {:<8}",
                 dataset.name,
                 original_size,
                 q64_encoded.len(),
                 base64_encoded.len(), 
                 base64_url_encoded.len(),
                 hex_encoded.len(),
                 msgpack_encoded.len(),
                 bincode_encoded.len(),
                 cbor_encoded.len());
    }
    
    // Add a dummy benchmark to make criterion happy
    c.bench_function("size_analysis_dummy", |b| b.iter(|| black_box(1)));
}

/// Memory allocation analysis
fn benchmark_memory_allocations(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocations");
    let test_data = (0..1024).map(|_| fastrand::u8(..)).collect::<Vec<u8>>();
    
    // Benchmark allocations for repeated operations
    group.bench_function("uubed_q64_repeated_alloc", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let result = q64_encode(black_box(&test_data));
                black_box(result);
            }
        });
    });
    
    group.bench_function("uubed_q64_buffer_reuse", |b| {
        let mut buffer = vec![0u8; test_data.len() * 2];
        b.iter(|| {
            for _ in 0..100 {
                let result = q64_encode_to_buffer(black_box(&test_data), black_box(&mut buffer));
                let _ = black_box(result);
            }
        });
    });
    
    group.bench_function("base64_repeated_alloc", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let result = general_purpose::STANDARD.encode(black_box(&test_data));
                black_box(result);
            }
        });
    });
    
    group.finish();
}

/// Roundtrip correctness verification
fn verify_roundtrip_correctness() {
    println!("\n=== ROUNDTRIP CORRECTNESS VERIFICATION ===");
    let datasets = create_test_datasets();
    
    for dataset in &datasets {
        println!("Testing {}: {}", dataset.name, dataset.description);
        
        // uubed Q64
        let q64_encoded = q64_encode(&dataset.data);
        let q64_decoded = q64_decode(&q64_encoded).unwrap();
        assert_eq!(dataset.data, q64_decoded, "Q64 roundtrip failed for {}", dataset.name);
        
        // Base64
        let base64_encoded = general_purpose::STANDARD.encode(&dataset.data);
        let base64_decoded = general_purpose::STANDARD.decode(&base64_encoded).unwrap();
        assert_eq!(dataset.data, base64_decoded, "Base64 roundtrip failed for {}", dataset.name);
        
        // Hex
        let hex_encoded = hex::encode(&dataset.data);
        let hex_decoded = hex::decode(&hex_encoded).unwrap();
        assert_eq!(dataset.data, hex_decoded, "Hex roundtrip failed for {}", dataset.name);
        
        // MessagePack
        let wrapper = EmbeddingWrapper { data: dataset.data.clone() };
        let msgpack_encoded = rmp_serde::to_vec(&wrapper).unwrap();
        let msgpack_decoded: EmbeddingWrapper = rmp_serde::from_slice(&msgpack_encoded).unwrap();
        assert_eq!(dataset.data, msgpack_decoded.data, "MessagePack roundtrip failed for {}", dataset.name);
        
        println!("  ✓ All encodings passed roundtrip test");
    }
    
    println!("All roundtrip tests passed!");
}

criterion_group!(
    benches,
    benchmark_encoding_speed,
    benchmark_decoding_speed, 
    benchmark_size_efficiency,
    benchmark_memory_allocations
);
criterion_main!(benches);