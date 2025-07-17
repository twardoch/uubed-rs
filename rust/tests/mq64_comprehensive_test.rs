use uubed_native::encoders::{mq64_encode, mq64_encode_with_levels, mq64_decode};

#[test]
fn test_mq64_basic_encoding() {
    let embedding = vec![10, 50, 30, 80, 20, 90, 40, 70];
    
    let encoded = mq64_encode(&embedding);
    assert!(!encoded.is_empty());
    
    // Verify deterministic behavior
    let encoded2 = mq64_encode(&embedding);
    assert_eq!(encoded, encoded2);
}

#[test]
fn test_mq64_encode_with_levels_function() {
    let embedding = vec![100, 200, 150, 75, 25, 250, 125, 175];
    let levels = vec![4, 8];
    
    let encoded = mq64_encode_with_levels(&embedding, &levels);
    assert!(!encoded.is_empty());
    
    // Should be different from regular mq64_encode (which uses default levels)
    let encoded_regular = mq64_encode(&embedding);
    assert_ne!(encoded, encoded_regular);
}

#[test]
fn test_mq64_roundtrip() {
    let embedding = vec![42, 100, 200, 150, 75, 25, 250, 125];
    
    let encoded = mq64_encode(&embedding);
    let decoded = mq64_decode(&encoded);
    
    assert!(decoded.is_ok());
    let decoded_embedding = decoded.unwrap();
    assert_eq!(embedding.len(), decoded_embedding.len());
    
    // MQ64 encoding is hierarchical and not lossy in the same way as other encoders
    // The decoded result should match the original for this test
    assert_eq!(embedding, decoded_embedding);
}

#[test]
fn test_mq64_different_level_counts() {
    let embedding = vec![0, 50, 100, 150, 200, 255];
    
    for levels in [vec![2], vec![4], vec![6], vec![10], vec![32]] {
        let encoded = mq64_encode_with_levels(&embedding, &levels);
        // Only check non-empty if level <= embedding.len()
        if levels.iter().any(|&l| l <= embedding.len()) {
            assert!(!encoded.is_empty(), "Empty encoding for {:?} levels", levels);
        }
        
        // Verify deterministic
        let encoded2 = mq64_encode_with_levels(&embedding, &levels);
        assert_eq!(encoded, encoded2, "Non-deterministic for {:?} levels", levels);
        
        // Verify decodable
        let decoded = mq64_decode(&encoded);
        assert!(decoded.is_ok(), "Failed to decode with {:?} levels", levels);
        
        // The decoded length should be the max level that fits the embedding
        let max_level = levels.iter().filter(|&&l| l <= embedding.len()).max().unwrap_or(&0);
        let expected_len = embedding.len().min(*max_level);
        assert_eq!(decoded.unwrap().len(), expected_len, "Length mismatch for {:?} levels", levels);
    }
}

#[test]
fn test_mq64_empty_input() {
    let empty: Vec<u8> = vec![];
    let encoded = mq64_encode(&empty);
    assert_eq!(encoded, "");
    
    let decoded = mq64_decode(&encoded);
    assert!(decoded.is_ok());
    assert_eq!(decoded.unwrap(), empty);
}

#[test]
fn test_mq64_single_element() {
    let single = vec![128];
    let encoded = mq64_encode(&single);
    assert!(!encoded.is_empty());
    
    let decoded = mq64_decode(&encoded);
    assert!(decoded.is_ok());
    let decoded_vec = decoded.unwrap();
    assert_eq!(decoded_vec.len(), 1);
    
    // Should be exact match for MQ64
    assert_eq!(single, decoded_vec);
}

#[test]
fn test_mq64_extreme_values() {
    let extreme = vec![0, 255, 1, 254, 2, 253];
    let encoded = mq64_encode(&extreme);
    assert!(!encoded.is_empty());
    
    let decoded = mq64_decode(&encoded);
    assert!(decoded.is_ok());
    assert_eq!(decoded.unwrap(), extreme);
}

#[test]
fn test_mq64_large_input() {
    let large: Vec<u8> = (0..1000).map(|i| (i % 256) as u8).collect();
    let encoded = mq64_encode(&large);
    assert!(!encoded.is_empty());
    
    let decoded = mq64_decode(&encoded);
    assert!(decoded.is_ok());
    assert_eq!(decoded.unwrap(), large);
}

#[test]
fn test_mq64_decode_invalid_input() {
    // Test with invalid characters not in Q64 alphabet
    let invalid = "invalid!@#$%^&*()";
    let result = mq64_decode(invalid);
    assert!(result.is_err());
}

#[test]
fn test_mq64_decode_empty_string() {
    let result = mq64_decode("");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Vec::<u8>::new());
}

#[test]
fn test_mq64_decode_malformed_input() {
    // Test with odd length string (should be even for Q64)
    let odd_length = "abc"; // 3 characters
    let result = mq64_decode(odd_length);
    assert!(result.is_err());
}

#[test]
fn test_mq64_hierarchical_behavior() {
    let embedding = vec![0, 32, 64, 96, 128, 160, 192, 224, 255];
    
    // Test different hierarchical levels
    for levels in [vec![2], vec![4], vec![8], vec![16]] {
        let encoded = mq64_encode_with_levels(&embedding, &levels);
        let decoded = mq64_decode(&encoded).unwrap();
        
        // MQ64 decode returns the last level, which is the largest level that fits
        let max_level = levels.iter().filter(|&&l| l <= embedding.len()).max().unwrap_or(&0);
        let expected_len = embedding.len().min(*max_level);
        assert_eq!(decoded.len(), expected_len);
        for i in 0..decoded.len() {
            assert_eq!(decoded[i], embedding[i]);
        }
    }
}

#[test]
fn test_mq64_preserves_relative_order() {
    let embedding = vec![10, 50, 30, 80, 20, 90, 40, 70];
    let encoded = mq64_encode(&embedding);
    let decoded = mq64_decode(&encoded).unwrap();
    
    // Check that values are preserved exactly
    assert_eq!(decoded, embedding);
}

#[test]
fn test_mq64_consistency_across_calls() {
    let embedding = vec![123, 45, 67, 89, 12, 34, 56, 78];
    
    // Run multiple times to ensure consistency
    let mut results = Vec::new();
    for _ in 0..10 {
        let encoded = mq64_encode(&embedding);
        let decoded = mq64_decode(&encoded).unwrap();
        results.push(decoded);
    }
    
    // All results should be identical
    for result in &results[1..] {
        assert_eq!(results[0], *result);
    }
}

#[test]
fn test_mq64_performance_regression() {
    let embedding: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();
    
    let start = std::time::Instant::now();
    let encoded = mq64_encode(&embedding);
    let encode_time = start.elapsed();
    
    let start = std::time::Instant::now();
    let decoded = mq64_decode(&encoded).unwrap();
    let decode_time = start.elapsed();
    
    // Reasonable performance expectations (debug build)
    assert!(encode_time.as_millis() < 1000, "Encoding took {} ms", encode_time.as_millis());
    assert!(decode_time.as_millis() < 1000, "Decoding took {} ms", decode_time.as_millis());
    assert_eq!(decoded, embedding);
}

#[test]
fn test_mq64_with_levels_parameter_validation() {
    let embedding = vec![100, 200, 150];
    
    // Test with very small levels (should not panic)
    let result = mq64_encode_with_levels(&embedding, &[1]);
    assert!(!result.is_empty());
    
    // Test with very large levels (should not panic, but might be empty if level > data.len())
    let result = mq64_encode_with_levels(&embedding, &[256]);
    // Since embedding has length 3, level 256 > 3, so no parts will be encoded
    assert!(result.is_empty());
}

#[test]
fn test_mq64_thread_safety() {
    use std::sync::Arc;
    use std::thread;
    
    let embedding = Arc::new(vec![10, 20, 30, 40, 50]);
    let mut handles = vec![];
    
    for _ in 0..10 {
        let emb = Arc::clone(&embedding);
        let handle = thread::spawn(move || {
            let encoded = mq64_encode(&emb);
            mq64_decode(&encoded).unwrap()
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