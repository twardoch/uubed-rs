use quickcheck::{Arbitrary, Gen, TestResult};
use quickcheck_macros::quickcheck;
use uubed_native::encoders::{q64_encode, q64_decode, simhash_q64, top_k_q64, z_order_q64, topk_optimized};

// Custom types for property testing
#[derive(Clone, Debug)]
struct ValidEmbedding(Vec<u8>);

impl Arbitrary for ValidEmbedding {
    fn arbitrary(g: &mut Gen) -> Self {
        let size = usize::arbitrary(g) % 10000 + 1; // 1 to 10k elements
        let embedding = (0..size).map(|_| u8::arbitrary(g)).collect();
        ValidEmbedding(embedding)
    }
    
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let vec = self.0.clone();
        Box::new(vec.shrink().map(ValidEmbedding))
    }
}

#[derive(Clone, Debug)]
struct SmallEmbedding(Vec<u8>);

impl Arbitrary for SmallEmbedding {
    fn arbitrary(g: &mut Gen) -> Self {
        let size = usize::arbitrary(g) % 256 + 1; // 1 to 256 elements
        let embedding = (0..size).map(|_| u8::arbitrary(g)).collect();
        SmallEmbedding(embedding)
    }
}

#[derive(Clone, Debug)]
struct ValidK(usize);

impl Arbitrary for ValidK {
    fn arbitrary(g: &mut Gen) -> Self {
        let k = usize::arbitrary(g) % 1024 + 1; // 1 to 1024
        ValidK(k)
    }
}

// Property tests for Q64 encoding
#[quickcheck]
fn prop_q64_roundtrip(data: Vec<u8>) -> bool {
    let encoded = q64_encode(&data);
    match q64_decode(&encoded) {
        Ok(decoded) => decoded == data,
        Err(_) => false,
    }
}

#[quickcheck]
fn prop_q64_deterministic(data: Vec<u8>) -> bool {
    let encoded1 = q64_encode(&data);
    let encoded2 = q64_encode(&data);
    encoded1 == encoded2
}

#[quickcheck]
fn prop_q64_length_relationship(data: Vec<u8>) -> bool {
    let encoded = q64_encode(&data);
    // Q64 encoding should produce 2 characters per byte
    encoded.len() == data.len() * 2
}

#[quickcheck]
fn prop_q64_empty_input() -> bool {
    let empty: Vec<u8> = vec![];
    let encoded = q64_encode(&empty);
    encoded.is_empty() && q64_decode(&encoded).unwrap().is_empty()
}

// Property tests for Top-k encoding
#[quickcheck]
fn prop_topk_deterministic(embedding: ValidEmbedding, k: ValidK) -> bool {
    let emb = &embedding.0;
    let k = k.0.min(emb.len());
    
    let result1 = top_k_q64(emb, k);
    let result2 = top_k_q64(emb, k);
    result1 == result2
}

#[quickcheck]
fn prop_topk_optimized_matches_original(embedding: ValidEmbedding, k: ValidK) -> bool {
    let emb = &embedding.0;
    let k = k.0.min(emb.len());
    
    let original = top_k_q64(emb, k);
    let optimized = topk_optimized::top_k_q64_optimized(emb, k);
    original == optimized
}

#[quickcheck]
fn prop_topk_length_consistency(embedding: ValidEmbedding, k: ValidK) -> bool {
    let emb = &embedding.0;
    let k = k.0.min(emb.len());
    
    let result = top_k_q64(emb, k);
    // Top-k should produce consistent length output (k indices encoded as Q64)
    result.len() == k * 2 // Each index becomes 2 Q64 characters
}

#[quickcheck]
fn prop_topk_handles_large_k(embedding: SmallEmbedding) -> bool {
    let emb = &embedding.0;
    let large_k = emb.len() + 100; // k much larger than embedding size
    
    let result = top_k_q64(emb, large_k);
    !result.is_empty() // Should handle gracefully
}

// Property tests for SimHash
#[quickcheck]
fn prop_simhash_deterministic(embedding: ValidEmbedding, planes: u8) -> TestResult {
    let emb = &embedding.0;
    let planes = (planes as usize % 512) + 1; // 1 to 512 planes
    
    if emb.is_empty() {
        return TestResult::discard();
    }
    
    let hash1 = simhash_q64(emb, planes);
    let hash2 = simhash_q64(emb, planes);
    TestResult::from_bool(hash1 == hash2)
}

#[quickcheck]
fn prop_simhash_different_inputs_different_outputs(emb1: ValidEmbedding, emb2: ValidEmbedding) -> TestResult {
    let e1 = &emb1.0;
    let e2 = &emb2.0;
    
    if e1.is_empty() || e2.is_empty() || e1 == e2 {
        return TestResult::discard();
    }
    
    let hash1 = simhash_q64(e1, 64);
    let hash2 = simhash_q64(e2, 64);
    
    // Different inputs should (usually) produce different hashes
    // This is probabilistic, so we'll accept some collisions
    TestResult::from_bool(hash1 != hash2)
}

#[quickcheck]
fn prop_simhash_length_proportional_to_planes(embedding: ValidEmbedding, planes: u8) -> TestResult {
    let emb = &embedding.0;
    let planes = (planes as usize % 256) + 1;
    
    if emb.is_empty() {
        return TestResult::discard();
    }
    
    let hash = simhash_q64(emb, planes);
    let expected_bytes = (planes + 7) / 8; // Ceil division
    let expected_chars = expected_bytes * 2; // Q64 encoding
    
    TestResult::from_bool(hash.len() == expected_chars)
}

// Property tests for Z-order encoding
#[quickcheck]
fn prop_zorder_deterministic(embedding: SmallEmbedding) -> bool {
    let emb = &embedding.0;
    
    let result1 = z_order_q64(emb);
    let result2 = z_order_q64(emb);
    result1 == result2
}

#[quickcheck]
fn prop_zorder_non_empty_output(embedding: SmallEmbedding) -> TestResult {
    let emb = &embedding.0;
    
    if emb.is_empty() {
        return TestResult::discard();
    }
    
    let result = z_order_q64(emb);
    TestResult::from_bool(!result.is_empty())
}

// Invariant tests - properties that should always hold
#[quickcheck]
fn prop_all_encoders_handle_empty_input() -> bool {
    let empty: Vec<u8> = vec![];
    
    // Q64 with empty input
    let q64_result = q64_encode(&empty);
    let q64_ok = q64_result.is_empty();
    
    // SimHash with empty should not crash (though result may vary)
    let simhash_result = std::panic::catch_unwind(|| simhash_q64(&empty, 64));
    let simhash_ok = simhash_result.is_ok();
    
    // Top-k with empty should not crash
    let topk_result = std::panic::catch_unwind(|| top_k_q64(&empty, 5));
    let topk_ok = topk_result.is_ok();
    
    // Z-order with empty should not crash
    let zorder_result = std::panic::catch_unwind(|| z_order_q64(&empty));
    let zorder_ok = zorder_result.is_ok();
    
    q64_ok && simhash_ok && topk_ok && zorder_ok
}

#[quickcheck]
fn prop_all_encoders_handle_single_byte(byte: u8) -> bool {
    let single = vec![byte];
    
    // All encoders should handle single byte input without crashing
    let q64_ok = std::panic::catch_unwind(|| q64_encode(&single)).is_ok();
    let simhash_ok = std::panic::catch_unwind(|| simhash_q64(&single, 32)).is_ok();
    let topk_ok = std::panic::catch_unwind(|| top_k_q64(&single, 1)).is_ok();
    let zorder_ok = std::panic::catch_unwind(|| z_order_q64(&single)).is_ok();
    
    q64_ok && simhash_ok && topk_ok && zorder_ok
}

#[quickcheck]
fn prop_no_encoder_produces_invalid_utf8(embedding: ValidEmbedding) -> TestResult {
    let emb = &embedding.0;
    
    if emb.is_empty() {
        return TestResult::discard();
    }
    
    // Ensure all encoders produce valid UTF-8 strings
    let q64_result = q64_encode(&emb[..emb.len().min(1000)]); // Limit size for performance
    let simhash_result = simhash_q64(emb, 64);
    let topk_result = top_k_q64(emb, emb.len().min(32));
    let zorder_result = z_order_q64(&emb[..emb.len().min(128)]);
    
    // All results should be valid UTF-8 (which they are since they're Strings)
    // and should contain only printable characters
    let q64_valid = q64_result.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    let simhash_valid = simhash_result.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    let topk_valid = topk_result.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    let zorder_valid = zorder_result.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    
    TestResult::from_bool(q64_valid && simhash_valid && topk_valid && zorder_valid)
}

// Performance invariants
#[quickcheck]
fn prop_linear_scaling_behavior(size_factor: u8) -> TestResult {
    let size_factor = (size_factor as usize % 10) + 1; // 1 to 10
    let base_size = 1000;
    let large_size = base_size * size_factor;
    
    if large_size > 50000 {
        return TestResult::discard(); // Avoid very slow tests
    }
    
    let small_emb: Vec<u8> = (0..base_size).map(|i| (i % 256) as u8).collect();
    let large_emb: Vec<u8> = (0..large_size).map(|i| (i % 256) as u8).collect();
    
    let k = 32;
    
    // Measure time for small embedding
    let start = std::time::Instant::now();
    let _ = top_k_q64(&small_emb, k);
    let small_time = start.elapsed();
    
    // Measure time for large embedding
    let start = std::time::Instant::now();
    let _ = topk_optimized::top_k_q64_optimized(&large_emb, k);
    let large_time = start.elapsed();
    
    // The optimized version should scale better than quadratically
    let time_ratio = large_time.as_nanos() as f64 / small_time.as_nanos() as f64;
    let size_ratio = size_factor as f64;
    
    // Allow some variance but expect roughly linear scaling for optimized version
    TestResult::from_bool(time_ratio <= size_ratio * 3.0)
}