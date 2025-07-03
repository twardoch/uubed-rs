// Standalone performance test for topk optimizations
// Run with: rustc test_topk_perf.rs -O -C target-cpu=native && ./test_topk_perf

use std::time::Instant;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

// Original implementation
fn top_k_indices(embedding: &[u8], k: usize) -> Vec<u8> {
    if embedding.len() <= 256 {
        top_k_indices_small(embedding, k)
    } else {
        top_k_indices_parallel(embedding, k)
    }
}

fn top_k_indices_small(embedding: &[u8], k: usize) -> Vec<u8> {
    let mut indexed: Vec<(u8, u8)> = embedding
        .iter()
        .enumerate()
        .map(|(idx, &val)| (val, idx as u8))
        .collect();

    let k_clamped = k.min(indexed.len());
    if k_clamped > 0 {
        indexed.select_nth_unstable_by(k_clamped - 1, |a, b| b.0.cmp(&a.0));
    }

    let mut indices: Vec<u8> = indexed[..k_clamped]
        .iter()
        .map(|(_, idx)| *idx)
        .collect();
    indices.sort_unstable();
    indices.resize(k, 255);
    indices
}

fn top_k_indices_parallel(embedding: &[u8], k: usize) -> Vec<u8> {
    let chunk_size = 256;
    let mut candidates: Vec<(u8, usize)> = Vec::new();
    
    for (chunk_idx, chunk) in embedding.chunks(chunk_size).enumerate() {
        let mut local_top: Vec<(u8, usize)> = chunk
            .iter()
            .enumerate()
            .map(|(idx, &val)| (val, chunk_idx * chunk_size + idx))
            .collect();

        let local_k = k.min(local_top.len());
        if local_k > 0 {
            local_top.select_nth_unstable_by(local_k - 1, |a, b| b.0.cmp(&a.0));
            local_top.truncate(local_k);
        }
        candidates.extend(local_top);
    }

    let final_k = k.min(candidates.len());
    if final_k > 0 {
        candidates.select_nth_unstable_by(final_k - 1, |a, b| b.0.cmp(&a.0));
    }

    let mut indices: Vec<u8> = candidates[..final_k]
        .iter()
        .map(|(_, idx)| (*idx).min(255) as u8)
        .collect();
    indices.sort_unstable();
    indices.resize(k, 255);
    indices
}

// Optimized implementation
fn top_k_indices_optimized(embedding: &[u8], k: usize) -> Vec<u8> {
    if k == 0 || embedding.is_empty() {
        return vec![255; k];
    }

    let len = embedding.len();
    
    if len <= 256 {
        top_k_indices_small_optimized(embedding, k)
    } else if k <= 16 {
        top_k_indices_heap(embedding, k)
    } else {
        // For this test, just use the heap approach
        top_k_indices_heap(embedding, k)
    }
}

fn top_k_indices_small_optimized(embedding: &[u8], k: usize) -> Vec<u8> {
    let k_clamped = k.min(embedding.len());
    
    let mut heap = BinaryHeap::with_capacity(k_clamped + 1);
    
    for (idx, &val) in embedding.iter().enumerate() {
        heap.push(Reverse((val, idx as u8)));
        if heap.len() > k_clamped {
            heap.pop();
        }
    }
    
    let mut indices: Vec<u8> = heap
        .into_sorted_vec()
        .into_iter()
        .map(|Reverse((_, idx))| idx)
        .collect();
    
    indices.sort_unstable();
    indices.resize(k, 255);
    indices
}

fn top_k_indices_heap(embedding: &[u8], k: usize) -> Vec<u8> {
    let k_clamped = k.min(embedding.len());
    
    let mut heap = BinaryHeap::with_capacity(k_clamped + 1);
    
    for (idx, &val) in embedding.iter().enumerate() {
        if heap.len() < k_clamped {
            heap.push(Reverse((val, idx)));
        } else if let Some(&Reverse((min_val, _))) = heap.peek() {
            if val > min_val {
                heap.pop();
                heap.push(Reverse((val, idx)));
            }
        }
    }
    
    let mut indices: Vec<u8> = heap
        .into_sorted_vec()
        .into_iter()
        .map(|Reverse((_, idx))| idx.min(255) as u8)
        .collect();
    
    indices.sort_unstable();
    indices.resize(k, 255);
    indices
}

// Simple random number generator
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    
    fn next_u8(&mut self) -> u8 {
        self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.state >> 32) as u8
    }
}

fn benchmark_topk(name: &str, embedding: &[u8], k: usize, iterations: usize, f: fn(&[u8], usize) -> Vec<u8>) {
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
    
    let mut rng = SimpleRng::new(12345);
    
    let sizes = vec![256, 1024, 4096, 16384];
    let k_values = vec![8, 16, 32, 64];
    
    for size in &sizes {
        println!("\nEmbedding size: {}", size);
        let embedding: Vec<u8> = (0..*size).map(|_| rng.next_u8()).collect();
        
        for k in &k_values {
            if *k <= *size {
                println!("\n  k = {}", k);
                
                let iterations = match size {
                    256 => 10000,
                    1024 => 5000,
                    4096 => 1000,
                    16384 => 500,
                    _ => 100,
                };
                
                benchmark_topk("    Original", &embedding, *k, iterations, top_k_indices);
                benchmark_topk("    Optimized", &embedding, *k, iterations, top_k_indices_optimized);
            }
        }
    }
}