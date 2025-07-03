// this_file: rust/src/encoders/simhash.rs
//! SimHash implementation with parallel matrix multiplication.

use rayon::prelude::*;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::collections::HashMap;
use super::q64::Q64Error;

/// Cache for projection matrices of different sizes
static MATRIX_CACHE: Lazy<Mutex<HashMap<(usize, usize), ProjectionMatrix>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Random projection matrix for SimHash
#[derive(Clone)]
struct ProjectionMatrix {
    data: Vec<f32>,
    planes: usize,
    dims: usize,
}

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

    /// Get cached matrix or create new one
    fn get_or_create(planes: usize, dims: usize) -> ProjectionMatrix {
        let mut cache = MATRIX_CACHE.lock().unwrap();
        cache.entry((planes, dims))
            .or_insert_with(|| ProjectionMatrix::new(planes, dims))
            .clone()
    }

    /// Project vector onto hyperplanes (parallel)
    fn project(&self, embedding: &[u8]) -> Vec<bool> {
        // Convert bytes to centered floats
        let centered: Vec<f32> = embedding
            .iter()
            .map(|&b| (b as f32 - 128.0) / 128.0)
            .collect();

        // Parallel matrix multiplication
        (0..self.planes)
            .into_par_iter()
            .map(|plane_idx| {
                let row_start = plane_idx * self.dims;
                let row_end = row_start + self.dims.min(centered.len());

                let dot_product: f32 = self.data[row_start..row_end]
                    .iter()
                    .zip(&centered)
                    .map(|(a, b)| a * b)
                    .sum();

                dot_product > 0.0
            })
            .collect()
    }
}

/// Generate SimHash with Q64 encoding
///
/// # Algorithm
/// 1. Project embedding onto random hyperplanes
/// 2. Take sign of each projection as a bit
/// 3. Pack bits into bytes
/// 4. Encode with position-safe Q64
pub fn simhash_q64(embedding: &[u8], planes: usize) -> String {
    // Get cached projection matrix for efficiency
    let matrix = ProjectionMatrix::get_or_create(planes, embedding.len());

    // Project and get bits
    let bits = matrix.project(embedding);

    // Pack bits into bytes
    let mut bytes = Vec::with_capacity((bits.len() + 7) / 8);
    for chunk in bits.chunks(8) {
        let mut byte = 0u8;
        for (i, &bit) in chunk.iter().enumerate() {
            if bit {
                byte |= 1 << (7 - i);
            }
        }
        bytes.push(byte);
    }

    // Encode with Q64
    super::q64::q64_encode(&bytes)
}

/// Zero-copy version: Generate SimHash with Q64 encoding into pre-allocated buffer
///
/// # Arguments
/// * `embedding` - Input embedding vector
/// * `planes` - Number of random hyperplanes (determines hash length)
/// * `output` - Pre-allocated buffer (must be at least `(planes / 4)` bytes)
///
/// # Returns
/// * `Ok(bytes_written)` - Number of bytes written to buffer
/// * `Err(String)` - Error message if buffer is too small
///
/// # Performance
/// - Zero allocation encoding for maximum performance
/// - Reuses cached projection matrices
pub fn simhash_to_buffer(embedding: &[u8], planes: usize, output: &mut [u8]) -> Result<usize, Q64Error> {
    // Get or generate projection matrix
    let matrix = ProjectionMatrix::get_or_create(planes, embedding.len());
    
    // Calculate hash
    let mut bits = vec![false; planes];
    
    // Compute dot products
    for plane_idx in 0..planes {
        let offset = plane_idx * embedding.len();
        let plane = &matrix.data[offset..offset + embedding.len()];
        
        let dot: f32 = embedding.iter()
            .zip(plane.iter())
            .map(|(&e, &p)| (e as f32 - 128.0) * p)
            .sum();
        
        bits[plane_idx] = dot >= 0.0;
    }
    
    // Pack bits directly into output buffer
    let bytes_needed = (bits.len() + 7) / 8;
    let q64_needed = bytes_needed * 2;  // Q64 doubles the size
    
    if output.len() < q64_needed {
        return Err(Q64Error {
            message: format!("Output buffer too small: need {} bytes, got {}", q64_needed, output.len())
        });
    }
    
    // Pack bits into temporary bytes
    let mut temp_bytes = [0u8; 128]; // Max 1024 planes = 128 bytes
    for (byte_idx, chunk) in bits.chunks(8).enumerate() {
        let mut byte = 0u8;
        for (i, &bit) in chunk.iter().enumerate() {
            if bit {
                byte |= 1 << (7 - i);
            }
        }
        temp_bytes[byte_idx] = byte;
    }
    
    // Encode directly to output buffer
    super::q64::q64_encode_to_buffer(&temp_bytes[..bytes_needed], output)?;
    
    Ok(q64_needed)
}
/// - Directly writes to output buffer
pub fn simhash_q64_to_buffer(
    embedding: &[u8], 
    planes: usize, 
    output: &mut [u8]
) -> Result<usize, String> {
    // Calculate required output size
    let hash_bytes = (planes + 7) / 8;  // Number of bytes for hash
    let required_len = hash_bytes * 2;  // Q64 encoding doubles the size
    
    if output.len() < required_len {
        return Err(format!(
            "Output buffer too small: need {} bytes, got {}",
            required_len, output.len()
        ));
    }

    // Get cached projection matrix for efficiency
    let matrix = ProjectionMatrix::get_or_create(planes, embedding.len());

    // Project and get bits
    let bits = matrix.project(embedding);

    // Pack bits into bytes directly in a temporary buffer
    let mut hash_bytes_buf = vec![0u8; hash_bytes];
    for (chunk_idx, chunk) in bits.chunks(8).enumerate() {
        let mut byte = 0u8;
        for (i, &bit) in chunk.iter().enumerate() {
            if bit {
                byte |= 1 << (7 - i);
            }
        }
        hash_bytes_buf[chunk_idx] = byte;
    }

    // Encode with Q64 directly to output buffer
    super::q64::q64_encode_to_buffer(&hash_bytes_buf, output)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simhash_deterministic() {
        let embedding = vec![100; 32];
        let hash1 = simhash_q64(&embedding, 64);
        let hash2 = simhash_q64(&embedding, 64);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_simhash_locality() {
        let base = vec![100; 32];
        let mut similar = base.clone();
        similar[0] = 101;  // Small change

        let different: Vec<u8> = base.iter().map(|&x| 255 - x).collect();

        let hash1 = simhash_q64(&base, 64);
        let hash2 = simhash_q64(&similar, 64);
        let hash3 = simhash_q64(&different, 64);

        // Count differences
        let diff_similar = hash1.chars()
            .zip(hash2.chars())
            .filter(|(a, b)| a != b)
            .count();

        let diff_different = hash1.chars()
            .zip(hash3.chars())
            .filter(|(a, b)| a != b)
            .count();

        assert!(diff_similar < diff_different);
    }

    #[test]
    fn test_simhash_length() {
        let embedding = vec![0; 256];
        let hash = simhash_q64(&embedding, 64);
        assert_eq!(hash.len(), 16); // 64 bits = 8 bytes = 16 q64 chars
    }
}