// this_file: rust/src/encoders/topk.rs
//! Top-k indices encoder for sparse representation.

use rayon::prelude::*;

/// Find top k indices with highest values
///
/// Uses parallel partial sorting for efficiency on large embeddings
pub fn top_k_indices(embedding: &[u8], k: usize) -> Vec<u8> {
    if embedding.len() <= 256 {
        // Fast path for small embeddings
        top_k_indices_small(embedding, k)
    } else {
        // Parallel path for large embeddings
        top_k_indices_parallel(embedding, k)
    }
}

/// Fast implementation for embeddings that fit in a u8 index
fn top_k_indices_small(embedding: &[u8], k: usize) -> Vec<u8> {
    let mut indexed: Vec<(u8, u8)> = embedding
        .iter()
        .enumerate()
        .map(|(idx, &val)| (val, idx as u8))
        .collect();

    // Partial sort to get top k
    let k_clamped = k.min(indexed.len());
    if k_clamped > 0 {
        indexed.select_nth_unstable_by(k_clamped - 1, |a, b| b.0.cmp(&a.0));
    }

    // Extract indices and sort them
    let mut indices: Vec<u8> = indexed[..k_clamped]
        .iter()
        .map(|(_, idx)| *idx)
        .collect();
    indices.sort_unstable();

    // Pad with 255 if needed
    indices.resize(k, 255);
    indices
}

/// Parallel implementation for large embeddings
fn top_k_indices_parallel(embedding: &[u8], k: usize) -> Vec<u8> {
    // Split into chunks for parallel processing
    let chunk_size = 256;
    let chunks: Vec<_> = embedding
        .chunks(chunk_size)
        .enumerate()
        .collect();

    // Find top candidates from each chunk in parallel
    let candidates: Vec<(u8, usize)> = chunks
        .par_iter()
        .flat_map(|(chunk_idx, chunk)| {
            let mut local_top: Vec<(u8, usize)> = chunk
                .iter()
                .enumerate()
                .map(|(idx, &val)| (val, chunk_idx * chunk_size + idx))
                .collect();

            // Keep top k from each chunk
            let local_k = k.min(local_top.len());
            if local_k > 0 {
                local_top.select_nth_unstable_by(local_k - 1, |a, b| b.0.cmp(&a.0));
                local_top.truncate(local_k);
            }
            local_top
        })
        .collect();

    // Final selection from candidates
    let mut final_candidates = candidates;
    let final_k = k.min(final_candidates.len());
    if final_k > 0 {
        final_candidates.select_nth_unstable_by(final_k - 1, |a, b| b.0.cmp(&a.0));
    }

    // Extract indices, handle large indices
    let mut indices: Vec<u8> = final_candidates[..final_k]
        .iter()
        .map(|(_, idx)| (*idx).min(255) as u8)
        .collect();
    indices.sort_unstable();

    // Pad with 255 if needed
    indices.resize(k, 255);
    indices
}

/// Generate top-k encoding with Q64
pub fn top_k_q64(embedding: &[u8], k: usize) -> String {
    let indices = top_k_indices(embedding, k);
    super::q64::q64_encode(&indices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_top_k_basic() {
        let data = vec![10, 50, 30, 80, 20, 90, 40, 70];
        let top3 = top_k_indices(&data, 3);
        assert_eq!(top3, vec![3, 5, 7]);  // Indices of 80, 90, 70
    }

    #[test]
    fn test_top_k_padding() {
        let data = vec![10, 20, 30];
        let top5 = top_k_indices(&data, 5);
        assert_eq!(top5, vec![0, 1, 2, 255, 255]);  // Padded with 255
    }

    #[test]
    fn test_top_k_empty() {
        let data = vec![];
        let top3 = top_k_indices(&data, 3);
        assert_eq!(top3, vec![255, 255, 255]);
    }

    #[test]
    fn test_top_k_large() {
        let mut data = vec![0; 300];
        data[100] = 255;
        data[200] = 200;
        data[299] = 150;

        let top3 = top_k_indices(&data, 3);
        assert!(top3.contains(&100));
        assert!(top3.contains(&200));
        assert!(top3.contains(&255)); // 299 clamped to 255
    }

    #[test]
    fn test_top_k_q64_length() {
        let data = vec![10, 50, 30, 80, 20, 90, 40, 70];
        let encoded = top_k_q64(&data, 8);
        assert_eq!(encoded.len(), 16); // 8 indices = 16 q64 chars
    }
}