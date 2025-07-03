use uubed_native::encoders::{
    q64_encode, q64_decode,
    simhash_q64,
    top_k_q64,
    z_order_q64,
    topk_optimized::top_k_q64_optimized,
};

#[test]
fn test_full_pipeline() {
    let embedding = vec![10, 50, 30, 80, 20, 90, 40, 70, 15, 25];
    
    // Test Q64 roundtrip
    let encoded = q64_encode(&embedding);
    let decoded = q64_decode(&encoded).unwrap();
    assert_eq!(embedding, decoded);
    
    // Test all encoders produce valid output
    let simhash = simhash_q64(&embedding, 64);
    let topk = top_k_q64(&embedding, 4);
    let topk_opt = top_k_q64_optimized(&embedding, 4);
    let zorder = z_order_q64(&embedding);
    
    // Verify outputs are non-empty and have expected characteristics
    assert!(!simhash.is_empty());
    assert!(!topk.is_empty());
    assert!(!topk_opt.is_empty());
    assert!(!zorder.is_empty());
    
    // Verify consistency between top-k implementations
    assert_eq!(topk, topk_opt);
}

#[test]
fn test_performance_characteristics() {
    // Test with different sizes to ensure scaling
    let sizes = vec![100, 1000, 10000];
    
    for size in sizes {
        let embedding: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        
        // All operations should complete in reasonable time
        let start = std::time::Instant::now();
        
        let _q64 = q64_encode(&embedding[..256.min(embedding.len())]);
        let _simhash = simhash_q64(&embedding, 64);
        let _topk = top_k_q64_optimized(&embedding, 32);
        let _zorder = z_order_q64(&embedding[..128.min(embedding.len())]);
        
        let duration = start.elapsed();
        assert!(duration.as_millis() < 1000, "Operations took too long for size {}", size);
    }
}

#[test]
fn test_edge_cases() {
    // Empty input
    let empty: Vec<u8> = vec![];
    assert_eq!(q64_encode(&empty), "");
    
    // Single element
    let single = vec![42];
    let encoded = q64_encode(&single);
    let decoded = q64_decode(&encoded).unwrap();
    assert_eq!(single, decoded);
    
    // Large k values
    let small_embedding = vec![1, 2, 3];
    let topk = top_k_q64_optimized(&small_embedding, 10); // k > length
    // When k=10 and embedding.len()=3:
    // - top_k_indices_optimized returns [0, 1, 2, 255, 255, 255, 255, 255, 255, 255] (10 indices)
    // - Q64 encoding produces 2 characters per byte
    // - So 10 bytes → 20 characters
    assert_eq!(topk.len(), 20);
}

#[test]
fn test_correctness_properties() {
    let embedding = (0..256u16).map(|i| (i % 256) as u8).collect::<Vec<_>>();
    
    // Q64 should be deterministic
    let encoded1 = q64_encode(&embedding);
    let encoded2 = q64_encode(&embedding);
    assert_eq!(encoded1, encoded2);
    
    // SimHash should be deterministic
    let simhash1 = simhash_q64(&embedding, 128);
    let simhash2 = simhash_q64(&embedding, 128);
    assert_eq!(simhash1, simhash2);
    
    // Top-k should be deterministic
    let topk1 = top_k_q64_optimized(&embedding, 16);
    let topk2 = top_k_q64_optimized(&embedding, 16);
    assert_eq!(topk1, topk2);
    
    // Z-order should be deterministic
    let zorder1 = z_order_q64(&embedding);
    let zorder2 = z_order_q64(&embedding);
    assert_eq!(zorder1, zorder2);
}