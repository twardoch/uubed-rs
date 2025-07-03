// this_file: rust/src/parallel.rs
/// Parallel batch encoding operations for high-throughput scenarios.

use rayon::prelude::*;
use crate::encoders::*;
use crate::error::UubedError;

/// Parallel Q64 encoding for multiple embeddings
///
/// # Arguments
/// * `embeddings` - Vector of embedding byte slices
/// * `num_threads` - Optional number of threads (defaults to system cores)
///
/// # Returns
/// * `Vec<String>` - Encoded strings in same order as input
///
/// # Performance
/// - Scales linearly up to available CPU cores
/// - Optimal for embeddings >1KB and batch sizes >100
pub fn parallel_q64_encode(embeddings: &[&[u8]], num_threads: Option<usize>) -> Vec<String> {
    // Set thread pool size if specified
    if let Some(threads) = num_threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap()
            .install(|| {
                embeddings.par_iter()
                    .map(|embedding| q64_encode(embedding))
                    .collect()
            })
    } else {
        embeddings.par_iter()
            .map(|embedding| q64_encode(embedding))
            .collect()
    }
}

/// Parallel SimHash encoding for multiple embeddings
///
/// # Arguments
/// * `embeddings` - Vector of embedding byte slices
/// * `planes` - Number of hyperplanes for SimHash
/// * `num_threads` - Optional number of threads
///
/// # Returns
/// * `Vec<String>` - Encoded SimHash strings
pub fn parallel_simhash_encode(
    embeddings: &[&[u8]], 
    planes: usize, 
    num_threads: Option<usize>
) -> Vec<String> {
    if let Some(threads) = num_threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap()
            .install(|| {
                embeddings.par_iter()
                    .map(|embedding| simhash_q64(embedding, planes))
                    .collect()
            })
    } else {
        embeddings.par_iter()
            .map(|embedding| simhash_q64(embedding, planes))
            .collect()
    }
}

/// Parallel Top-K encoding for multiple embeddings
///
/// # Arguments
/// * `embeddings` - Vector of embedding byte slices
/// * `k` - Number of top indices to select
/// * `num_threads` - Optional number of threads
///
/// # Returns
/// * `Vec<String>` - Encoded Top-K strings
pub fn parallel_topk_encode(
    embeddings: &[&[u8]], 
    k: usize, 
    num_threads: Option<usize>
) -> Vec<String> {
    if let Some(threads) = num_threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap()
            .install(|| {
                embeddings.par_iter()
                    .map(|embedding| top_k_q64_optimized(embedding, k))
                    .collect()
            })
    } else {
        embeddings.par_iter()
            .map(|embedding| top_k_q64_optimized(embedding, k))
            .collect()
    }
}

/// Parallel Z-order encoding for multiple embeddings
///
/// # Arguments
/// * `embeddings` - Vector of embedding byte slices
/// * `num_threads` - Optional number of threads
///
/// # Returns
/// * `Vec<String>` - Encoded Z-order strings
pub fn parallel_zorder_encode(embeddings: &[&[u8]], num_threads: Option<usize>) -> Vec<String> {
    if let Some(threads) = num_threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap()
            .install(|| {
                embeddings.par_iter()
                    .map(|embedding| z_order_q64(embedding))
                    .collect()
            })
    } else {
        embeddings.par_iter()
            .map(|embedding| z_order_q64(embedding))
            .collect()
    }
}

/// High-performance batch processor with work-stealing and adaptive load balancing
///
/// This struct provides advanced parallel processing capabilities for large batches
/// of embeddings with automatic load balancing and optimal work distribution.
pub struct BatchProcessor {
    thread_pool: rayon::ThreadPool,
    chunk_size: usize,
}

impl BatchProcessor {
    /// Create a new batch processor
    ///
    /// # Arguments
    /// * `num_threads` - Number of worker threads (defaults to CPU cores)
    /// * `chunk_size` - Work chunk size for load balancing (defaults to adaptive)
    pub fn new(num_threads: Option<usize>, chunk_size: Option<usize>) -> Result<Self, UubedError> {
        let pool = match num_threads {
            Some(threads) => rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .map_err(|e| UubedError::ComputationError(format!("Failed to create thread pool: {}", e)))?,
            None => rayon::ThreadPoolBuilder::new()
                .build()
                .map_err(|e| UubedError::ComputationError(format!("Failed to create thread pool: {}", e)))?,
        };
        
        // Adaptive chunk size based on number of threads
        let adaptive_chunk_size = chunk_size.unwrap_or_else(|| {
            std::cmp::max(1, 10000 / pool.current_num_threads())
        });
        
        Ok(Self {
            thread_pool: pool,
            chunk_size: adaptive_chunk_size,
        })
    }
    
    /// Process a large batch with optimal work distribution
    ///
    /// # Arguments
    /// * `embeddings` - Vector of embeddings to process
    /// * `method` - Encoding method to apply
    /// 
    /// # Returns
    /// * `Vec<String>` - Encoded results in original order
    pub fn process_batch<F>(&self, embeddings: &[&[u8]], method: F) -> Vec<String>
    where
        F: Fn(&[u8]) -> String + Send + Sync,
    {
        self.thread_pool.install(|| {
            embeddings
                .par_chunks(self.chunk_size)
                .flat_map(|chunk| {
                    chunk.par_iter()
                        .map(|embedding| method(embedding))
                        .collect::<Vec<_>>()
                })
                .collect()
        })
    }
    
    /// Get number of worker threads
    pub fn thread_count(&self) -> usize {
        self.thread_pool.current_num_threads()
    }
    
    /// Get current chunk size
    pub fn chunk_size(&self) -> usize {
        self.chunk_size
    }
}

impl Default for BatchProcessor {
    fn default() -> Self {
        Self::new(None, None).expect("Failed to create default batch processor")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_q64_encode() {
        let embeddings = vec![
            vec![1, 2, 3, 4],
            vec![5, 6, 7, 8], 
            vec![9, 10, 11, 12],
        ];
        let embedding_refs: Vec<&[u8]> = embeddings.iter().map(|e| e.as_slice()).collect();
        
        let results = parallel_q64_encode(&embedding_refs, Some(2));
        assert_eq!(results.len(), 3);
        
        // Verify results match sequential encoding
        for (i, embedding) in embeddings.iter().enumerate() {
            let expected = q64_encode(embedding);
            assert_eq!(results[i], expected);
        }
    }
    
    #[test]
    fn test_batch_processor() {
        let processor = BatchProcessor::new(Some(2), Some(2)).unwrap();
        assert_eq!(processor.thread_count(), 2);
        assert_eq!(processor.chunk_size(), 2);
        
        let embeddings = vec![
            vec![1, 2, 3, 4],
            vec![5, 6, 7, 8],
            vec![9, 10, 11, 12],
            vec![13, 14, 15, 16],
        ];
        let embedding_refs: Vec<&[u8]> = embeddings.iter().map(|e| e.as_slice()).collect();
        
        let results = processor.process_batch(&embedding_refs, |emb| q64_encode(emb));
        assert_eq!(results.len(), 4);
        
        // Verify order preservation
        for (i, embedding) in embeddings.iter().enumerate() {
            let expected = q64_encode(embedding);
            assert_eq!(results[i], expected);
        }
    }
    
    #[test]
    fn test_parallel_performance_scaling() {
        // Test with larger dataset to verify scaling
        let large_embedding = vec![42u8; 1024]; // 1KB embedding
        let embeddings: Vec<&[u8]> = (0..100).map(|_| large_embedding.as_slice()).collect();
        
        // Test single-threaded
        let start = std::time::Instant::now();
        let results_single = parallel_q64_encode(&embeddings, Some(1));
        let time_single = start.elapsed();
        
        // Test multi-threaded
        let start = std::time::Instant::now();
        let results_multi = parallel_q64_encode(&embeddings, None);
        let time_multi = start.elapsed();
        
        // Results should be identical
        assert_eq!(results_single, results_multi);
        
        // Multi-threaded should be faster (or at least not much slower)
        // Allow for some overhead in small tests
        assert!(time_multi <= time_single * 2);
    }
}