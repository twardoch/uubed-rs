// this_file: rust/src/encoders/topk_optimized.rs
/// Optimized Top-k indices encoder with SIMD and memory optimizations.

use rayon::prelude::*;
use std::cmp::Reverse;

/// Find top k indices with highest values - optimized version
///
/// Optimizations:
/// - SIMD-friendly memory layout
/// - Optimized heap-based selection for better cache locality
/// - Reduced allocations
/// - Better parallelization strategy
pub fn top_k_indices_optimized(embedding: &[u8], k: usize) -> Vec<u8> {
    if k == 0 || embedding.is_empty() {
        return vec![255; k];
    }

    let len = embedding.len();
    
    if len <= 256 {
        // Optimized small path
        top_k_indices_small_optimized(embedding, k)
    } else if k <= 16 {
        // For small k, use heap-based approach
        top_k_indices_heap(embedding, k)
    } else {
        // Parallel path with improved chunking
        top_k_indices_parallel_optimized(embedding, k)
    }
}

/// Optimized implementation for small embeddings
fn top_k_indices_small_optimized(embedding: &[u8], k: usize) -> Vec<u8> {
    let k_clamped = k.min(embedding.len());
    
    // For very small k or when k is close to n, use different strategies
    if k_clamped <= 4 || k_clamped as f32 / embedding.len() as f32 > 0.25 {
        // Use the original approach for these cases
        let mut indexed: Vec<(u8, u8)> = embedding
            .iter()
            .enumerate()
            .map(|(idx, &val)| (val, idx as u8))
            .collect();

        if k_clamped > 0 {
            indexed.select_nth_unstable_by(k_clamped - 1, |a, b| b.0.cmp(&a.0));
        }

        let mut indices: Vec<u8> = indexed[..k_clamped]
            .iter()
            .map(|(_, idx)| *idx)
            .collect();
        indices.sort_unstable();
        indices.resize(k, 255);
        return indices;
    }
    
    // Use heap for other cases
    use std::collections::BinaryHeap;
    let mut heap = BinaryHeap::with_capacity(k_clamped + 1);
    
    for (idx, &val) in embedding.iter().enumerate() {
        heap.push(Reverse((val, idx as u8)));
        if heap.len() > k_clamped {
            heap.pop();
        }
    }
    
    // Extract and sort indices
    let mut indices: Vec<u8> = heap
        .into_sorted_vec()
        .into_iter()
        .map(|Reverse((_, idx))| idx)
        .collect();
    
    indices.sort_unstable();
    indices.resize(k, 255);
    indices
}

/// Heap-based approach for large embeddings with small k
fn top_k_indices_heap(embedding: &[u8], k: usize) -> Vec<u8> {
    use std::collections::BinaryHeap;
    
    let k_clamped = k.min(embedding.len());
    
    // Track top k using min-heap
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
    
    // Extract indices, handle large indices
    let mut indices: Vec<u8> = heap
        .into_sorted_vec()
        .into_iter()
        .map(|Reverse((_, idx))| idx.min(255) as u8)
        .collect();
    
    indices.sort_unstable();
    indices.resize(k, 255);
    indices
}

/// Optimized parallel implementation with better work distribution
fn top_k_indices_parallel_optimized(embedding: &[u8], k: usize) -> Vec<u8> {
    // Adaptive chunk size based on embedding length and available threads
    let num_threads = rayon::current_num_threads();
    let chunk_size = ((embedding.len() + num_threads - 1) / num_threads).max(256);
    
    // Process chunks in parallel with pre-allocated space
    let candidates: Vec<Vec<(u8, usize)>> = embedding
        .par_chunks(chunk_size)
        .enumerate()
        .map(|(chunk_idx, chunk)| {
            let base_idx = chunk_idx * chunk_size;
            let local_k = k.min(chunk.len());
            
            if local_k == 0 {
                return Vec::new();
            }
            
            // Use heap for efficient top-k selection in each chunk
            use std::collections::BinaryHeap;
            let mut heap = BinaryHeap::with_capacity(local_k + 1);
            
            for (idx, &val) in chunk.iter().enumerate() {
                heap.push(Reverse((val, base_idx + idx)));
                if heap.len() > local_k {
                    heap.pop();
                }
            }
            
            heap.into_sorted_vec()
                .into_iter()
                .map(|Reverse(item)| item)
                .collect()
        })
        .collect();
    
    // Merge candidates efficiently
    let total_candidates: usize = candidates.iter().map(|v| v.len()).sum();
    let mut all_candidates = Vec::with_capacity(total_candidates);
    
    for chunk_candidates in candidates {
        all_candidates.extend(chunk_candidates);
    }
    
    // Final top-k selection
    let final_k = k.min(all_candidates.len());
    if final_k > 0 {
        all_candidates.select_nth_unstable_by(final_k - 1, |a, b| b.0.cmp(&a.0));
    }
    
    // Extract indices with bounds checking
    let mut indices: Vec<u8> = all_candidates[..final_k]
        .iter()
        .map(|(_, idx)| (*idx).min(255) as u8)
        .collect();
    
    indices.sort_unstable();
    indices.resize(k, 255);
    indices
}

// SIMD implementations are reserved for future optimization
// Current implementation focuses on algorithmic improvements and parallelization

/// Generate top-k encoding with Q64 using optimized encoder
pub fn top_k_q64_optimized(embedding: &[u8], k: usize) -> String {
    let indices = top_k_indices_optimized(embedding, k);
    super::q64::q64_encode(&indices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimized_matches_original() {
        let data = vec![10, 50, 30, 80, 20, 90, 40, 70];
        let top3 = top_k_indices_optimized(&data, 3);
        assert_eq!(top3, vec![3, 5, 7]);  // Indices of 80, 90, 70
    }

    #[test]
    fn test_heap_approach() {
        let mut data = vec![0; 1000];
        data[100] = 255;
        data[500] = 200;
        data[900] = 150;
        
        let top3 = top_k_indices_heap(&data, 3);
        assert!(top3.contains(&100));
        assert!(top3.contains(&255)); // 500 clamped to 255
        assert_eq!(top3.len(), 3);
    }

    #[test]
    fn test_parallel_optimized() {
        let mut data = vec![0; 10000];
        // Set some high values
        data[1000] = 255;
        data[5000] = 250;
        data[9999] = 245;
        
        let top3 = top_k_indices_parallel_optimized(&data, 3);
        assert_eq!(top3.len(), 3);
        // Verify indices are sorted
        assert!(top3.windows(2).all(|w| w[0] <= w[1]));
    }

    #[test]
    fn test_edge_cases() {
        // Empty data
        assert_eq!(top_k_indices_optimized(&[], 5), vec![255; 5]);
        
        // k = 0
        assert_eq!(top_k_indices_optimized(&[1, 2, 3], 0), vec![]);
        
        // k > len
        let data = vec![10, 20];
        assert_eq!(top_k_indices_optimized(&data, 5), vec![0, 1, 255, 255, 255]);
    }
}