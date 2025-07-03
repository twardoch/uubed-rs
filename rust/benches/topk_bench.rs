use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rand::prelude::*;
use uubed_native::encoders::{topk, topk_optimized};

fn generate_random_embedding(size: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..size).map(|_| rng.gen()).collect()
}

fn generate_sparse_embedding(size: usize, sparsity: f32) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..size)
        .map(|_| {
            if rng.gen::<f32>() < sparsity {
                0
            } else {
                rng.gen()
            }
        })
        .collect()
}

fn bench_topk_implementations(c: &mut Criterion) {
    let mut group = c.benchmark_group("topk-comparison");
    
    // Test different embedding sizes
    for size in [256, 1024, 4096, 16384, 65536].iter() {
        let embedding = generate_random_embedding(*size);
        
        // Test different k values
        for k in [8, 16, 32, 64, 128].iter() {
            if *k <= *size {
                group.bench_with_input(
                    BenchmarkId::new("original", format!("size={}_k={}", size, k)),
                    &(&embedding, *k),
                    |b, (emb, k)| {
                        b.iter(|| {
                            topk::top_k_indices(black_box(emb), black_box(*k))
                        })
                    },
                );
                
                group.bench_with_input(
                    BenchmarkId::new("optimized", format!("size={}_k={}", size, k)),
                    &(&embedding, *k),
                    |b, (emb, k)| {
                        b.iter(|| {
                            topk_optimized::top_k_indices_optimized(black_box(emb), black_box(*k))
                        })
                    },
                );
            }
        }
    }
    
    group.finish();
}

fn bench_sparse_embeddings(c: &mut Criterion) {
    let mut group = c.benchmark_group("topk-sparse");
    
    let size = 16384;
    let k = 32;
    
    for sparsity in [0.5, 0.8, 0.9, 0.95].iter() {
        let embedding = generate_sparse_embedding(size, *sparsity);
        
        group.bench_with_input(
            BenchmarkId::new("original", format!("sparsity={}", sparsity)),
            &(&embedding, k),
            |b, (emb, k)| {
                b.iter(|| {
                    topk::top_k_indices(black_box(emb), black_box(*k))
                })
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("optimized", format!("sparsity={}", sparsity)),
            &(&embedding, k),
            |b, (emb, k)| {
                b.iter(|| {
                    topk_optimized::top_k_indices_optimized(black_box(emb), black_box(*k))
                })
            },
        );
    }
    
    group.finish();
}

fn bench_edge_cases(c: &mut Criterion) {
    let mut group = c.benchmark_group("topk-edge-cases");
    
    // Very small embeddings
    let small = generate_random_embedding(16);
    group.bench_function("small-original", |b| {
        b.iter(|| topk::top_k_indices(black_box(&small), black_box(8)))
    });
    group.bench_function("small-optimized", |b| {
        b.iter(|| topk_optimized::top_k_indices_optimized(black_box(&small), black_box(8)))
    });
    
    // k = 1 (finding maximum)
    let large = generate_random_embedding(65536);
    group.bench_function("max-original", |b| {
        b.iter(|| topk::top_k_indices(black_box(&large), black_box(1)))
    });
    group.bench_function("max-optimized", |b| {
        b.iter(|| topk_optimized::top_k_indices_optimized(black_box(&large), black_box(1)))
    });
    
    // k = n (full sort)
    let medium = generate_random_embedding(1024);
    group.bench_function("fullsort-original", |b| {
        b.iter(|| topk::top_k_indices(black_box(&medium), black_box(1024)))
    });
    group.bench_function("fullsort-optimized", |b| {
        b.iter(|| topk_optimized::top_k_indices_optimized(black_box(&medium), black_box(1024)))
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_topk_implementations,
    bench_sparse_embeddings,
    bench_edge_cases
);
criterion_main!(benches);