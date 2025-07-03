use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use rand::prelude::*;
use uubed_native::encoders::{q64_encode, simhash_q64, top_k_q64, topk_optimized, z_order_q64};

/// Generate a very large embedding simulating real-world data
fn generate_large_embedding(size: usize, pattern: &str) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    
    match pattern {
        "random" => {
            // Fully random data
            (0..size).map(|_| rng.gen()).collect()
        }
        "sparse" => {
            // 90% sparse data (common in embeddings)
            (0..size).map(|_| {
                if rng.gen::<f32>() < 0.9 {
                    0
                } else {
                    rng.gen_range(1..=255)
                }
            }).collect()
        }
        "clustered" => {
            // Data with clusters of high values
            (0..size).map(|i| {
                let cluster = i / 1000;
                if cluster % 10 == 0 {
                    rng.gen_range(200..=255)
                } else {
                    rng.gen_range(0..=50)
                }
            }).collect()
        }
        "gradient" => {
            // Gradually changing values
            (0..size).map(|i| {
                ((i as f64 / size as f64) * 255.0) as u8
            }).collect()
        }
        _ => panic!("Unknown pattern"),
    }
}

fn bench_very_large_q64(c: &mut Criterion) {
    let mut group = c.benchmark_group("large-q64");
    group.sample_size(10); // Reduce sample size for very large data
    
    // Test embeddings from 1MB to 16MB
    for mb in [1, 4, 8, 16].iter() {
        let size = mb * 1024 * 1024;
        let data = generate_large_embedding(size, "random");
        
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_function(BenchmarkId::new("encode", format!("{}MB", mb)), |b| {
            b.iter(|| {
                // Only encode first 1MB to keep reasonable time
                let chunk_size = 1024 * 1024;
                let encoded = black_box(q64_encode(&data[..chunk_size.min(data.len())]));
                encoded
            })
        });
    }
    
    group.finish();
}

fn bench_very_large_topk(c: &mut Criterion) {
    let mut group = c.benchmark_group("large-topk");
    group.sample_size(10);
    
    // Different embedding patterns
    let patterns = ["random", "sparse", "clustered", "gradient"];
    
    // Test sizes: 1M, 10M, 50M elements
    for &size in [1_000_000, 10_000_000, 50_000_000].iter() {
        for pattern in &patterns {
            let embedding = generate_large_embedding(size, pattern);
            
            // Different k values
            for &k in [64, 256, 1024].iter() {
                group.throughput(Throughput::Elements(size as u64));
                
                // Benchmark original
                group.bench_function(
                    BenchmarkId::new(
                        "original",
                        format!("size={}_k={}_pattern={}", size / 1_000_000, k, pattern)
                    ),
                    |b| {
                        b.iter(|| {
                            black_box(top_k_q64(&embedding, k))
                        })
                    },
                );
                
                // Benchmark optimized
                group.bench_function(
                    BenchmarkId::new(
                        "optimized",
                        format!("size={}_k={}_pattern={}", size / 1_000_000, k, pattern)
                    ),
                    |b| {
                        b.iter(|| {
                            black_box(topk_optimized::top_k_q64_optimized(&embedding, k))
                        })
                    },
                );
            }
        }
    }
    
    group.finish();
}

fn bench_very_large_simhash(c: &mut Criterion) {
    let mut group = c.benchmark_group("large-simhash");
    group.sample_size(10);
    
    // Test with different embedding sizes
    for &size in [1_000_000, 5_000_000, 10_000_000].iter() {
        let embedding = generate_large_embedding(size, "sparse");
        
        for &planes in [64, 128, 256].iter() {
            group.throughput(Throughput::Elements(size as u64));
            group.bench_function(
                BenchmarkId::new("simhash", format!("size={}_planes={}", size / 1_000_000, planes)),
                |b| {
                    b.iter(|| {
                        black_box(simhash_q64(&embedding, planes))
                    })
                },
            );
        }
    }
    
    group.finish();
}

fn bench_very_large_zorder(c: &mut Criterion) {
    let mut group = c.benchmark_group("large-zorder");
    group.sample_size(10);
    
    // Z-order typically used for smaller dimensional data
    for &dims in [128, 256, 512, 1024].iter() {
        let count = 100_000; // 100k vectors
        
        for pattern in ["random", "clustered"].iter() {
            group.throughput(Throughput::Elements((count * dims) as u64));
            group.bench_function(
                BenchmarkId::new("zorder", format!("dims={}_count={}_pattern={}", dims, count, pattern)),
                |b| {
                    b.iter(|| {
                        // Process vectors in batches
                        let mut results = Vec::with_capacity(count);
                        for _i in 0..count {
                            let embedding = generate_large_embedding(dims, pattern);
                            results.push(black_box(z_order_q64(&embedding)));
                        }
                        results
                    })
                },
            );
        }
    }
    
    group.finish();
}

fn bench_scaling_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling-analysis");
    group.sample_size(10);
    
    // Test how performance scales with size
    let sizes = vec![
        100_000,
        500_000,
        1_000_000,
        5_000_000,
        10_000_000,
        20_000_000,
    ];
    
    for &size in &sizes {
        let embedding = generate_large_embedding(size, "sparse");
        let k = 128;
        
        group.throughput(Throughput::Elements(size as u64));
        
        // Measure time complexity
        group.bench_function(
            BenchmarkId::new("topk-scaling", format!("{}M", size / 1_000_000)),
            |b| {
                b.iter(|| {
                    black_box(topk_optimized::top_k_q64_optimized(&embedding, k))
                })
            },
        );
    }
    
    group.finish();
}

fn bench_memory_pressure(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory-pressure");
    group.sample_size(10);
    
    // Test performance under memory pressure with concurrent operations
    let embedding_size = 10_000_000; // 10M elements
    let num_threads = 8;
    
    group.bench_function("concurrent-large-operations", |b| {
        b.iter(|| {
            use std::sync::Arc;
            use std::thread;
            
            let embedding = Arc::new(generate_large_embedding(embedding_size, "sparse"));
            let handles: Vec<_> = (0..num_threads)
                .map(|i| {
                    let emb = Arc::clone(&embedding);
                    thread::spawn(move || {
                        match i % 4 {
                            0 => q64_encode(&emb[..1024]),
                            1 => simhash_q64(&emb, 128),
                            2 => topk_optimized::top_k_q64_optimized(&emb, 256),
                            _ => z_order_q64(&emb[..256]),
                        }
                    })
                })
                .collect();
            
            for handle in handles {
                let _ = handle.join().unwrap();
            }
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_very_large_q64,
    bench_very_large_topk,
    bench_very_large_simhash,
    bench_very_large_zorder,
    bench_scaling_analysis,
    bench_memory_pressure
);
criterion_main!(benches);