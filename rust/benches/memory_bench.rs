use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use rand::prelude::*;
use uubed_native::encoders::{q64_encode, simhash_q64, top_k_q64, topk_optimized, z_order_q64};

/// Custom allocator to track memory usage
struct TrackingAllocator {
    allocated: AtomicUsize,
    peak: AtomicUsize,
}

impl TrackingAllocator {
    const fn new() -> Self {
        Self {
            allocated: AtomicUsize::new(0),
            peak: AtomicUsize::new(0),
        }
    }
    
    fn reset(&self) {
        self.allocated.store(0, Ordering::SeqCst);
        self.peak.store(0, Ordering::SeqCst);
    }
    
    #[allow(dead_code)]
    fn current(&self) -> usize {
        self.allocated.load(Ordering::SeqCst)
    }
    
    fn peak_usage(&self) -> usize {
        self.peak.load(Ordering::SeqCst)
    }
}

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = System.alloc(layout);
        if !ret.is_null() {
            let old_size = self.allocated.fetch_add(layout.size(), Ordering::SeqCst);
            let new_size = old_size + layout.size();
            let mut peak = self.peak.load(Ordering::Relaxed);
            while new_size > peak {
                match self.peak.compare_exchange_weak(
                    peak,
                    new_size,
                    Ordering::SeqCst,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(p) => peak = p,
                }
            }
        }
        ret
    }
    
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        self.allocated.fetch_sub(layout.size(), Ordering::SeqCst);
    }
}

#[global_allocator]
static ALLOCATOR: TrackingAllocator = TrackingAllocator::new();

fn generate_embedding(size: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..size).map(|_| rng.gen()).collect()
}

#[allow(dead_code)]
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

fn bench_memory_q64(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory-q64");
    
    for size in [128, 1024, 8192, 65536].iter() {
        let data = generate_embedding(*size);
        
        group.bench_function(BenchmarkId::new("encode", size), |b| {
            b.iter_custom(|iters| {
                let mut total_time = std::time::Duration::new(0, 0);
                let mut peak_memory = 0;
                
                for _ in 0..iters {
                    ALLOCATOR.reset();
                    let start = std::time::Instant::now();
                    let _ = black_box(q64_encode(&data));
                    total_time += start.elapsed();
                    peak_memory = peak_memory.max(ALLOCATOR.peak_usage());
                }
                
                println!("Q64 encode size={}: peak memory = {} bytes", size, peak_memory);
                total_time
            })
        });
    }
    
    group.finish();
}

fn bench_memory_topk(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory-topk");
    
    for size in [1024, 8192, 65536, 262144].iter() {
        let embedding = generate_embedding(*size);
        
        for k in [16, 64, 256].iter() {
            if *k <= *size {
                group.bench_function(
                    BenchmarkId::new("original", format!("size={}_k={}", size, k)),
                    |b| {
                        b.iter_custom(|iters| {
                            let mut total_time = std::time::Duration::new(0, 0);
                            let mut peak_memory = 0;
                            
                            for _ in 0..iters {
                                ALLOCATOR.reset();
                                let start = std::time::Instant::now();
                                let _ = black_box(top_k_q64(&embedding, *k));
                                total_time += start.elapsed();
                                peak_memory = peak_memory.max(ALLOCATOR.peak_usage());
                            }
                            
                            println!("Top-k original size={} k={}: peak memory = {} bytes", 
                                    size, k, peak_memory);
                            total_time
                        })
                    },
                );
                
                group.bench_function(
                    BenchmarkId::new("optimized", format!("size={}_k={}", size, k)),
                    |b| {
                        b.iter_custom(|iters| {
                            let mut total_time = std::time::Duration::new(0, 0);
                            let mut peak_memory = 0;
                            
                            for _ in 0..iters {
                                ALLOCATOR.reset();
                                let start = std::time::Instant::now();
                                let _ = black_box(topk_optimized::top_k_q64_optimized(&embedding, *k));
                                total_time += start.elapsed();
                                peak_memory = peak_memory.max(ALLOCATOR.peak_usage());
                            }
                            
                            println!("Top-k optimized size={} k={}: peak memory = {} bytes", 
                                    size, k, peak_memory);
                            total_time
                        })
                    },
                );
            }
        }
    }
    
    group.finish();
}

fn bench_memory_simhash(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory-simhash");
    
    for size in [1024, 8192, 65536].iter() {
        let embedding = generate_embedding(*size);
        
        for planes in [64, 128, 256].iter() {
            group.bench_function(
                BenchmarkId::new("simhash", format!("size={}_planes={}", size, planes)),
                |b| {
                    b.iter_custom(|iters| {
                        let mut total_time = std::time::Duration::new(0, 0);
                        let mut peak_memory = 0;
                        
                        for _ in 0..iters {
                            ALLOCATOR.reset();
                            let start = std::time::Instant::now();
                            let _ = black_box(simhash_q64(&embedding, *planes));
                            total_time += start.elapsed();
                            peak_memory = peak_memory.max(ALLOCATOR.peak_usage());
                        }
                        
                        println!("SimHash size={} planes={}: peak memory = {} bytes", 
                                size, planes, peak_memory);
                        total_time
                    })
                },
            );
        }
    }
    
    group.finish();
}

fn bench_memory_concurrent(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory-concurrent");
    
    let embedding = Arc::new(generate_embedding(65536));
    
    for num_threads in [2, 4, 8, 16].iter() {
        group.bench_function(
            BenchmarkId::new("concurrent-load", num_threads),
            |b| {
                b.iter_custom(|iters| {
                    let mut total_time = std::time::Duration::new(0, 0);
                    let mut peak_memory = 0;
                    
                    for _ in 0..iters {
                        ALLOCATOR.reset();
                        let start = std::time::Instant::now();
                        
                        let handles: Vec<_> = (0..*num_threads)
                            .map(|_| {
                                let emb = Arc::clone(&embedding);
                                std::thread::spawn(move || {
                                    // Mix of operations
                                    let _ = q64_encode(&emb[..1024]);
                                    let _ = simhash_q64(&emb, 64);
                                    let _ = top_k_q64(&emb, 32);
                                    let _ = z_order_q64(&emb[..128]);
                                })
                            })
                            .collect();
                        
                        for handle in handles {
                            handle.join().unwrap();
                        }
                        
                        total_time += start.elapsed();
                        peak_memory = peak_memory.max(ALLOCATOR.peak_usage());
                    }
                    
                    println!("Concurrent {} threads: peak memory = {} bytes", 
                            num_threads, peak_memory);
                    total_time
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_memory_q64,
    bench_memory_topk,
    bench_memory_simhash,
    bench_memory_concurrent
);
criterion_main!(benches);