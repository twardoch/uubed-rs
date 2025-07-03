// this_file: rust/src/encoders/simhash_safe.rs
/// Thread-safe SimHash implementation with lock-free caching.

use rayon::prelude::*;
use once_cell::sync::Lazy;
use std::sync::Arc;

// Using Arc<DashMap> for lock-free concurrent access
// In production, we'd add dashmap dependency. For now, using thread_local caching
thread_local! {
    static MATRIX_CACHE: std::cell::RefCell<std::collections::HashMap<(usize, usize), Arc<ProjectionMatrix>>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

/// Random projection matrix for SimHash
#[derive(Clone)]
struct ProjectionMatrix {
    data: Vec<f32>,
    planes: usize,
    dims: usize,
}

// SAFETY: ProjectionMatrix contains only primitive types and is immutable after creation
unsafe impl Send for ProjectionMatrix {}
unsafe impl Sync for ProjectionMatrix {}

impl ProjectionMatrix {
    /// Generate matrix with fixed seed for reproducibility
    fn new(planes: usize, dims: usize) -> Self {
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;
        use rand_distr::{Distribution, Normal};

        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let normal = Normal::new(0.0, 1.0).unwrap();

        let mut data = Vec::with_capacity(planes * dims);

        // Generate normal distribution for better random projections
        for _ in 0..(planes * dims) {
            data.push(normal.sample(&mut rng));
        }

        Self { data, planes, dims }
    }

    /// Get cached matrix or create new one - thread-safe version
    fn get_or_create(planes: usize, dims: usize) -> Arc<ProjectionMatrix> {
        MATRIX_CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            cache.entry((planes, dims))
                .or_insert_with(|| Arc::new(ProjectionMatrix::new(planes, dims)))
                .clone()
        })
    }

    /// Project vector onto hyperplanes (parallel)
    fn project(&self, embedding: &[u8]) -> Vec<bool> {
        let min_len = embedding.len().min(self.dims);

        // Parallel projection onto each hyperplane
        (0..self.planes)
            .into_par_iter()
            .map(|plane| {
                let offset = plane * self.dims;
                let mut dot_product = 0.0f32;

                // Compute dot product with this hyperplane
                for i in 0..min_len {
                    dot_product += embedding[i] as f32 * self.data[offset + i];
                }

                // Return sign of projection
                dot_product > 0.0
            })
            .collect()
    }
}

/// Generate SimHash from embedding
pub fn simhash(embedding: &[u8], planes: usize) -> Vec<u8> {
    let matrix = ProjectionMatrix::get_or_create(planes, embedding.len());
    let bits = matrix.project(embedding);

    // Pack bits into bytes
    let mut hash = vec![0u8; (planes + 7) / 8];
    for (i, &bit) in bits.iter().enumerate() {
        if bit {
            hash[i / 8] |= 1 << (i % 8);
        }
    }

    hash
}

/// Generate SimHash with Q64 encoding
pub fn simhash_q64_safe(embedding: &[u8], planes: usize) -> String {
    let hash = simhash(embedding, planes);
    super::q64::q64_encode(&hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_safety() {
        use std::thread;
        
        let embedding = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let emb = embedding.clone();
                thread::spawn(move || {
                    simhash(&emb, 64)
                })
            })
            .collect();
        
        let results: Vec<_> = handles.into_iter()
            .map(|h| h.join().unwrap())
            .collect();
        
        // All threads should produce the same result
        for i in 1..results.len() {
            assert_eq!(results[0], results[i]);
        }
    }

    #[test]
    fn test_deterministic() {
        let data = vec![10, 20, 30, 40, 50];
        let hash1 = simhash(&data, 64);
        let hash2 = simhash(&data, 64);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_different_embeddings() {
        let data1 = vec![10, 20, 30];
        let data2 = vec![40, 50, 60];
        let hash1 = simhash(&data1, 64);
        let hash2 = simhash(&data2, 64);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_concurrent_access() {
        use rayon::prelude::*;
        
        // Test parallel access with different embedding sizes
        let results: Vec<_> = (0..100)
            .into_par_iter()
            .map(|i| {
                let size = 10 + (i % 20);
                let embedding: Vec<u8> = (0..size).map(|j| (i + j) as u8).collect();
                simhash(&embedding, 64)
            })
            .collect();
        
        // Verify we got results for all
        assert_eq!(results.len(), 100);
    }
}