use std::time::Instant;
use rand::prelude::*;

// Import the modules directly
mod encoders {
    pub mod q64 {
        include!("../src/encoders/q64.rs");
    }
    pub mod topk {
        include!("../src/encoders/topk.rs");
    }
    pub mod topk_optimized {
        include!("../src/encoders/topk_optimized.rs");
    }
}

use encoders::{topk, topk_optimized};

fn generate_random_embedding(size: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..size).map(|_| rng.gen()).collect()
}

fn benchmark_topk(name: &str, embedding: &[u8], k: usize, iterations: usize, f: impl Fn(&[u8], usize) -> Vec<u8>) {
    // Warm up
    for _ in 0..10 {
        let _ = f(embedding, k);
    }
    
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = f(embedding, k);
    }
    let duration = start.elapsed();
    
    let avg_micros = duration.as_micros() / iterations as u128;
    println!("{}: {} iterations in {:?} (avg: {} µs/op)", name, iterations, duration, avg_micros);
}

fn main() {
    println!("Top-k Performance Comparison\n");
    
    let sizes = vec![256, 1024, 4096, 16384, 65536];
    let k_values = vec![8, 16, 32, 64];
    
    for size in &sizes {
        println!("\nEmbedding size: {}", size);
        let embedding = generate_random_embedding(*size);
        
        for k in &k_values {
            if *k <= *size {
                println!("\n  k = {}", k);
                
                let iterations = match size {
                    256 => 10000,
                    1024 => 5000,
                    4096 => 1000,
                    16384 => 500,
                    65536 => 100,
                    _ => 100,
                };
                
                benchmark_topk("    Original", &embedding, *k, iterations, |e, k| {
                    topk::top_k_indices(e, k)
                });
                
                benchmark_topk("    Optimized", &embedding, *k, iterations, |e, k| {
                    topk_optimized::top_k_indices_optimized(e, k)
                });
            }
        }
    }
    
    // Test sparse embeddings
    println!("\n\nSparse Embeddings Test (size=16384, k=32)");
    for sparsity in &[0.5, 0.8, 0.9, 0.95] {
        println!("\n  Sparsity: {}", sparsity);
        
        let mut rng = rand::thread_rng();
        let embedding: Vec<u8> = (0..16384)
            .map(|_| {
                if rng.gen::<f32>() < *sparsity {
                    0
                } else {
                    rng.gen()
                }
            })
            .collect();
        
        benchmark_topk("    Original", &embedding, 32, 500, |e, k| {
            topk::top_k_indices(e, k)
        });
        
        benchmark_topk("    Optimized", &embedding, 32, 500, |e, k| {
            topk_optimized::top_k_indices_optimized(e, k)
        });
    }
}