use uubed_native::encoders::simhash_safe::{simhash, simhash_q64_safe};
use std::collections::HashSet;

#[test]
fn test_simhash_safe_basic() {
    let embedding = vec![10, 20, 30, 40, 50];
    let hash = simhash(&embedding, 64);
    
    assert!(!hash.is_empty());
    assert_eq!(hash.len(), 8); // 64 bits = 8 bytes
    
    // Test Q64 variant
    let q64_hash = simhash_q64_safe(&embedding, 64);
    assert!(!q64_hash.is_empty());
    assert_eq!(q64_hash.len(), 16); // 8 bytes * 2 = 16 characters
}

#[test]
fn test_simhash_safe_deterministic() {
    let embedding = vec![100, 200, 150, 75, 25];
    
    // Multiple calls should produce identical results
    let hash1 = simhash(&embedding, 32);
    let hash2 = simhash(&embedding, 32);
    let hash3 = simhash(&embedding, 32);
    
    assert_eq!(hash1, hash2);
    assert_eq!(hash2, hash3);
    
    // Same for Q64 variant
    let q64_1 = simhash_q64_safe(&embedding, 32);
    let q64_2 = simhash_q64_safe(&embedding, 32);
    assert_eq!(q64_1, q64_2);
}

#[test]
fn test_simhash_safe_different_inputs() {
    let embedding1 = vec![1, 2, 3, 4, 5];
    let embedding2 = vec![10, 20, 30, 40, 50];
    let embedding3 = vec![100, 200, 150, 75, 25];
    
    let hash1 = simhash(&embedding1, 64);
    let hash2 = simhash(&embedding2, 64);
    let hash3 = simhash(&embedding3, 64);
    
    // Different inputs should produce different hashes
    assert_ne!(hash1, hash2);
    assert_ne!(hash2, hash3);
    assert_ne!(hash1, hash3);
}

#[test]
fn test_simhash_safe_different_plane_counts() {
    let embedding = vec![42, 84, 126, 168, 210];
    
    let planes = [8, 16, 32, 64, 128];
    let mut hashes = Vec::new();
    
    for &plane_count in &planes {
        let hash = simhash(&embedding, plane_count);
        let expected_bytes = (plane_count + 7) / 8;
        assert_eq!(hash.len(), expected_bytes);
        hashes.push(hash);
    }
    
    // Different plane counts should produce different hash lengths
    let mut lengths = HashSet::new();
    for hash in &hashes {
        lengths.insert(hash.len());
    }
    assert_eq!(lengths.len(), planes.len());
}

#[test]
fn test_simhash_safe_empty_input() {
    let empty: Vec<u8> = vec![];
    let hash = simhash(&empty, 64);
    assert_eq!(hash.len(), 8);
    
    // Should be deterministic even with empty input
    let hash2 = simhash(&empty, 64);
    assert_eq!(hash, hash2);
}

#[test]
fn test_simhash_safe_single_element() {
    let single = vec![123];
    let hash = simhash(&single, 64);
    assert_eq!(hash.len(), 8);
    
    // Should be deterministic
    let hash2 = simhash(&single, 64);
    assert_eq!(hash, hash2);
}

#[test]
fn test_simhash_safe_large_embedding() {
    let large: Vec<u8> = (0..1000).map(|i| (i % 256) as u8).collect();
    let hash = simhash(&large, 128);
    assert_eq!(hash.len(), 16); // 128 bits = 16 bytes
    
    // Should be deterministic even with large input
    let hash2 = simhash(&large, 128);
    assert_eq!(hash, hash2);
}

#[test]
fn test_simhash_safe_bit_distribution() {
    let embedding = vec![128, 64, 32, 16, 8, 4, 2, 1];
    let hash = simhash(&embedding, 64);
    
    // Count set bits
    let mut bit_count = 0;
    for &byte in &hash {
        bit_count += byte.count_ones();
    }
    
    // With random projections, we should have roughly half bits set
    // Allow some variance
    assert!(bit_count > 20 && bit_count < 44, "Bit count: {}", bit_count);
}

#[test]
fn test_simhash_safe_consistency_across_sizes() {
    let embedding = vec![50, 100, 150, 200, 250];
    
    // Test that same embedding produces consistent results with different plane counts
    let hash32 = simhash(&embedding, 32);
    let hash64 = simhash(&embedding, 64);
    let hash128 = simhash(&embedding, 128);
    
    assert_eq!(hash32.len(), 4);
    assert_eq!(hash64.len(), 8);
    assert_eq!(hash128.len(), 16);
    
    // Each should be deterministic
    assert_eq!(hash32, simhash(&embedding, 32));
    assert_eq!(hash64, simhash(&embedding, 64));
    assert_eq!(hash128, simhash(&embedding, 128));
}

#[test]
fn test_simhash_safe_q64_encoding() {
    let embedding = vec![10, 20, 30, 40, 50];
    let planes = 64;
    
    let raw_hash = simhash(&embedding, planes);
    let q64_hash = simhash_q64_safe(&embedding, planes);
    
    // Q64 encoding should double the length
    assert_eq!(q64_hash.len(), raw_hash.len() * 2);
    
    // Should only contain valid Q64 characters
    let valid_chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    for ch in q64_hash.chars() {
        assert!(valid_chars.contains(ch), "Invalid character: {}", ch);
    }
}

#[test]
fn test_simhash_safe_extreme_plane_counts() {
    let embedding = vec![1, 2, 3, 4, 5];
    
    // Test with very small plane count
    let hash1 = simhash(&embedding, 1);
    assert_eq!(hash1.len(), 1);
    
    // Test with large plane count
    let hash512 = simhash(&embedding, 512);
    assert_eq!(hash512.len(), 64); // 512 bits = 64 bytes
    
    // Both should be deterministic
    assert_eq!(hash1, simhash(&embedding, 1));
    assert_eq!(hash512, simhash(&embedding, 512));
}

#[test]
fn test_simhash_safe_parallel_consistency() {
    use rayon::prelude::*;
    
    let embedding = vec![42, 84, 126, 168, 210];
    
    // Generate hashes in parallel
    let results: Vec<_> = (0..100)
        .into_par_iter()
        .map(|_| simhash(&embedding, 64))
        .collect();
    
    // All results should be identical
    for result in &results[1..] {
        assert_eq!(results[0], *result);
    }
}

#[test]
fn test_simhash_safe_thread_safety() {
    use std::sync::Arc;
    use std::thread;
    
    let embedding = Arc::new(vec![10, 20, 30, 40, 50]);
    let mut handles = vec![];
    
    for _ in 0..20 {
        let emb = Arc::clone(&embedding);
        let handle = thread::spawn(move || {
            simhash(&emb, 64)
        });
        handles.push(handle);
    }
    
    let results: Vec<_> = handles.into_iter()
        .map(|h| h.join().unwrap())
        .collect();
    
    // All results should be identical
    for result in &results[1..] {
        assert_eq!(results[0], *result);
    }
}

#[test]
fn test_simhash_safe_caching_behavior() {
    let embedding1 = vec![1, 2, 3, 4, 5];
    let embedding2 = vec![10, 20, 30, 40, 50];
    
    // First calls - should create cache entries
    let hash1a = simhash(&embedding1, 64);
    let hash2a = simhash(&embedding2, 64);
    
    // Second calls - should use cached matrices
    let hash1b = simhash(&embedding1, 64);
    let hash2b = simhash(&embedding2, 64);
    
    // Results should be identical
    assert_eq!(hash1a, hash1b);
    assert_eq!(hash2a, hash2b);
}

#[test]
fn test_simhash_safe_different_embedding_sizes() {
    let sizes = [1, 5, 10, 50, 100, 500];
    
    for &size in &sizes {
        let embedding: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        let hash = simhash(&embedding, 64);
        
        assert_eq!(hash.len(), 8);
        
        // Should be deterministic
        let hash2 = simhash(&embedding, 64);
        assert_eq!(hash, hash2);
    }
}

#[test]
fn test_simhash_safe_performance_regression() {
    let embedding: Vec<u8> = (0..1000).map(|i| (i % 256) as u8).collect();
    
    let start = std::time::Instant::now();
    let _hash = simhash(&embedding, 128);
    let duration = start.elapsed();
    
    // Should complete in reasonable time (debug build)
    assert!(duration.as_millis() < 500, "SimHash took {} ms", duration.as_millis());
}

#[test]
fn test_simhash_safe_stability_across_calls() {
    let embedding = vec![123, 45, 67, 89, 12, 34, 56, 78];
    
    // Generate many hashes to ensure stability
    let mut hashes = Vec::new();
    for _ in 0..100 {
        hashes.push(simhash(&embedding, 64));
    }
    
    // All should be identical
    for hash in &hashes[1..] {
        assert_eq!(hashes[0], *hash);
    }
}

#[test]
fn test_simhash_safe_hamming_distance() {
    let embedding1 = vec![10, 20, 30, 40, 50];
    let embedding2 = vec![15, 25, 35, 45, 55]; // Similar to embedding1
    let embedding3 = vec![200, 150, 100, 50, 25]; // Different from embedding1
    
    let hash1 = simhash(&embedding1, 64);
    let hash2 = simhash(&embedding2, 64);
    let hash3 = simhash(&embedding3, 64);
    
    // Calculate Hamming distances
    fn hamming_distance(a: &[u8], b: &[u8]) -> u32 {
        a.iter().zip(b.iter()).map(|(x, y)| (x ^ y).count_ones()).sum()
    }
    
    let dist_1_2 = hamming_distance(&hash1, &hash2);
    let dist_1_3 = hamming_distance(&hash1, &hash3);
    
    // Similar embeddings should have smaller Hamming distance
    assert!(dist_1_2 < dist_1_3, "dist_1_2={}, dist_1_3={}", dist_1_2, dist_1_3);
}