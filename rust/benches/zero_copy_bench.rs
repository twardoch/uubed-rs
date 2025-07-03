// this_file: rust/benches/zero_copy_bench.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use uubed_native::encoders::{q64_encode, q64_encode_to_buffer};

fn benchmark_zero_copy_vs_string(c: &mut Criterion) {
    let mut group = c.benchmark_group("q64_zero_copy");
    
    // Test different sizes
    let sizes = vec![100, 1000, 10000, 100000];
    
    for size in sizes {
        let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        
        // String allocation version
        group.bench_with_input(
            BenchmarkId::new("string_allocation", size),
            &data,
            |b, data| {
                b.iter(|| {
                    let result = q64_encode(black_box(data));
                    black_box(result);
                });
            },
        );
        
        // Zero-copy version
        group.bench_with_input(
            BenchmarkId::new("zero_copy", size),
            &data,
            |b, data| {
                let mut buffer = vec![0u8; data.len() * 2];
                b.iter(|| {
                    let result = q64_encode_to_buffer(black_box(data), black_box(&mut buffer));
                    black_box(result);
                });
            },
        );
        
        // Zero-copy with reused buffer (more realistic scenario)
        group.bench_with_input(
            BenchmarkId::new("zero_copy_reused", size),
            &data,
            |b, data| {
                let mut buffer = vec![0u8; data.len() * 2];
                b.iter(|| {
                    buffer.fill(0); // Clear buffer
                    let result = q64_encode_to_buffer(black_box(data), black_box(&mut buffer));
                    black_box(result);
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_memory_allocation_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");
    
    let data: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();
    
    // Multiple allocations (worst case)
    group.bench_function("multiple_allocations", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let result = q64_encode(black_box(&data));
                black_box(result);
            }
        });
    });
    
    // Single buffer reuse (best case)
    group.bench_function("reused_buffer", |b| {
        let mut buffer = vec![0u8; data.len() * 2];
        b.iter(|| {
            for _ in 0..100 {
                buffer.fill(0);
                let result = q64_encode_to_buffer(black_box(&data), black_box(&mut buffer));
                black_box(result);
            }
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_zero_copy_vs_string,
    benchmark_memory_allocation_patterns
);
criterion_main!(benches);