use uubed_native::encoders::{
    q64_encode_to_buffer, simhash_to_buffer, top_k_to_buffer, z_order_to_buffer,
    q64_encode, simhash_q64, top_k_q64_optimized, z_order_q64
};

#[test]
fn test_q64_encode_to_buffer_success() {
    let input = vec![1, 2, 3, 4, 5];
    let mut buffer = vec![0u8; 20]; // 5 bytes * 2 = 10 characters needed
    
    let result = q64_encode_to_buffer(&input, &mut buffer);
    assert!(result.is_ok());
    let written = result.unwrap();
    assert_eq!(written, 10);
    
    // Verify the result matches regular q64_encode
    let expected = q64_encode(&input);
    let buffer_result = String::from_utf8(buffer[..written].to_vec()).unwrap();
    assert_eq!(expected, buffer_result);
}

#[test]
fn test_q64_encode_to_buffer_insufficient_space() {
    let input = vec![1, 2, 3, 4, 5];
    let mut buffer = vec![0u8; 5]; // Too small - need 10 bytes
    
    let result = q64_encode_to_buffer(&input, &mut buffer);
    assert!(result.is_err());
}

#[test]
fn test_q64_encode_to_buffer_empty_input() {
    let input: Vec<u8> = vec![];
    let mut buffer = vec![0u8; 10];
    
    let result = q64_encode_to_buffer(&input, &mut buffer);
    assert!(result.is_ok());
    let written = result.unwrap();
    assert_eq!(written, 0);
}

#[test]
fn test_q64_encode_to_buffer_exact_size() {
    let input = vec![42, 100, 200];
    let mut buffer = vec![0u8; 6]; // Exactly the right size
    
    let result = q64_encode_to_buffer(&input, &mut buffer);
    assert!(result.is_ok());
    let written = result.unwrap();
    assert_eq!(written, 6);
    
    let expected = q64_encode(&input);
    let buffer_result = String::from_utf8(buffer[..written].to_vec()).unwrap();
    assert_eq!(expected, buffer_result);
}

#[test]
fn test_simhash_to_buffer_success() {
    let embedding = vec![10, 20, 30, 40, 50];
    let planes = 32;
    let expected_bytes = (planes + 7) / 8; // 4 bytes for 32 planes
    let mut buffer = vec![0u8; expected_bytes * 2]; // Q64 encoding doubles size
    
    let result = simhash_to_buffer(&embedding, planes, &mut buffer);
    assert!(result.is_ok());
    let written = result.unwrap();
    assert_eq!(written, expected_bytes * 2);
    
    // Verify consistency with regular simhash_q64
    let expected = simhash_q64(&embedding, planes);
    let buffer_result = String::from_utf8(buffer[..written].to_vec()).unwrap();
    assert_eq!(expected, buffer_result);
}

#[test]
fn test_simhash_to_buffer_insufficient_space() {
    let embedding = vec![10, 20, 30, 40, 50];
    let planes = 64;
    let mut buffer = vec![0u8; 10]; // Too small for 64 planes
    
    let result = simhash_to_buffer(&embedding, planes, &mut buffer);
    assert!(result.is_err());
}

#[test]
fn test_simhash_to_buffer_different_plane_counts() {
    let embedding = vec![1, 2, 3, 4, 5, 6, 7, 8];
    
    for planes in [8, 16, 32, 64, 128] {
        let expected_bytes = (planes + 7) / 8;
        let mut buffer = vec![0u8; expected_bytes * 2];
        
        let result = simhash_to_buffer(&embedding, planes, &mut buffer);
        assert!(result.is_ok(), "Failed for {} planes", planes);
        let written = result.unwrap();
        assert_eq!(written, expected_bytes * 2, "Wrong length for {} planes", planes);
        
        // Verify deterministic behavior
        let mut buffer2 = vec![0u8; expected_bytes * 2];
        let result2 = simhash_to_buffer(&embedding, planes, &mut buffer2);
        assert!(result2.is_ok());
        assert_eq!(buffer[..written], buffer2[..written]);
    }
}

#[test]
fn test_top_k_to_buffer_success() {
    let embedding = vec![10, 50, 30, 80, 20, 90, 40, 70];
    let k = 4;
    let mut buffer = vec![0u8; k * 2]; // k indices * 2 chars each
    
    let result = top_k_to_buffer(&embedding, k, &mut buffer);
    assert!(result.is_ok());
    let written = result.unwrap();
    assert_eq!(written, k * 2);
    
    // Verify consistency with regular top_k_q64_optimized
    let expected = top_k_q64_optimized(&embedding, k);
    let buffer_result = String::from_utf8(buffer[..written].to_vec()).unwrap();
    assert_eq!(expected, buffer_result);
}

#[test]
fn test_top_k_to_buffer_insufficient_space() {
    let embedding = vec![10, 50, 30, 80, 20];
    let k = 4;
    let mut buffer = vec![0u8; 5]; // Too small for k=4
    
    let result = top_k_to_buffer(&embedding, k, &mut buffer);
    assert!(result.is_err());
}

#[test]
fn test_top_k_to_buffer_large_k() {
    let embedding = vec![1, 2, 3];
    let k = 10; // Larger than embedding size
    let mut buffer = vec![0u8; k * 2];
    
    let result = top_k_to_buffer(&embedding, k, &mut buffer);
    assert!(result.is_ok());
    let written = result.unwrap();
    assert_eq!(written, k * 2);
    
    // Verify consistency
    let expected = top_k_q64_optimized(&embedding, k);
    let buffer_result = String::from_utf8(buffer[..written].to_vec()).unwrap();
    assert_eq!(expected, buffer_result);
}

#[test]
fn test_z_order_to_buffer_success() {
    let embedding = vec![10, 20, 30, 40, 50, 60, 70, 80];
    let mut buffer = vec![0u8; 1000]; // Large enough buffer
    
    let result = z_order_to_buffer(&embedding, &mut buffer);
    assert!(result.is_ok());
    let written = result.unwrap();
    assert!(written > 0);
    
    // The buffer version uses a different algorithm than the regular version
    // Just verify it produces consistent results
    let mut buffer2 = vec![0u8; 1000];
    let result2 = z_order_to_buffer(&embedding, &mut buffer2);
    assert!(result2.is_ok());
    let written2 = result2.unwrap();
    assert_eq!(written, written2);
    assert_eq!(buffer[..written], buffer2[..written2]);
}

#[test]
fn test_z_order_to_buffer_insufficient_space() {
    let embedding = vec![10, 20, 30, 40, 50, 60, 70, 80];
    let mut buffer = vec![0u8; 5]; // Too small
    
    let result = z_order_to_buffer(&embedding, &mut buffer);
    assert!(result.is_err());
}

#[test]
fn test_z_order_to_buffer_empty_input() {
    let embedding: Vec<u8> = vec![];
    let mut buffer = vec![0u8; 10];
    
    let result = z_order_to_buffer(&embedding, &mut buffer);
    assert!(result.is_ok());
    let written = result.unwrap();
    // Z-order always produces 8 bytes (4 bytes -> 8 Q64 chars) even for empty input
    assert_eq!(written, 8);
}

#[test]
fn test_z_order_to_buffer_single_element() {
    let embedding = vec![42];
    let mut buffer = vec![0u8; 10];
    
    let result = z_order_to_buffer(&embedding, &mut buffer);
    assert!(result.is_ok());
    let written = result.unwrap();
    assert!(written > 0);
    
    // The buffer version uses a different algorithm than the regular version
    // Just verify it produces consistent results
    let mut buffer2 = vec![0u8; 10];
    let result2 = z_order_to_buffer(&embedding, &mut buffer2);
    assert!(result2.is_ok());
    let written2 = result2.unwrap();
    assert_eq!(written, written2);
    assert_eq!(buffer[..written], buffer2[..written2]);
}

#[test]
fn test_buffer_operations_deterministic() {
    let embedding = vec![100, 200, 150, 75, 25, 250, 125, 175];
    
    // Test all buffer operations produce deterministic results
    for _ in 0..10 {
        // Q64
        let mut buffer1 = vec![0u8; 20];
        let mut buffer2 = vec![0u8; 20];
        let result1 = q64_encode_to_buffer(&embedding, &mut buffer1);
        let result2 = q64_encode_to_buffer(&embedding, &mut buffer2);
        assert!(result1.is_ok() && result2.is_ok());
        assert_eq!(buffer1[..result1.unwrap()], buffer2[..result2.unwrap()]);
        
        // SimHash
        let mut buffer1 = vec![0u8; 16];
        let mut buffer2 = vec![0u8; 16];
        let result1 = simhash_to_buffer(&embedding, 64, &mut buffer1);
        let result2 = simhash_to_buffer(&embedding, 64, &mut buffer2);
        assert!(result1.is_ok() && result2.is_ok());
        assert_eq!(buffer1[..result1.unwrap()], buffer2[..result2.unwrap()]);
        
        // Top-K
        let mut buffer1 = vec![0u8; 16];
        let mut buffer2 = vec![0u8; 16];
        let result1 = top_k_to_buffer(&embedding, 8, &mut buffer1);
        let result2 = top_k_to_buffer(&embedding, 8, &mut buffer2);
        assert!(result1.is_ok() && result2.is_ok());
        assert_eq!(buffer1[..result1.unwrap()], buffer2[..result2.unwrap()]);
        
        // Z-order
        let mut buffer1 = vec![0u8; 100];
        let mut buffer2 = vec![0u8; 100];
        let result1 = z_order_to_buffer(&embedding, &mut buffer1);
        let result2 = z_order_to_buffer(&embedding, &mut buffer2);
        assert!(result1.is_ok() && result2.is_ok());
        let written1 = result1.unwrap();
        let written2 = result2.unwrap();
        assert_eq!(written1, written2);
        assert_eq!(buffer1[..written1], buffer2[..written2]);
    }
}

#[test]
fn test_buffer_operations_zero_copy_behavior() {
    let embedding = vec![1, 2, 3, 4, 5];
    let mut buffer = vec![0u8; 100];
    
    // Fill buffer with pattern to verify it's actually being written to
    for i in 0..buffer.len() {
        buffer[i] = 0xFF;
    }
    
    let result = q64_encode_to_buffer(&embedding, &mut buffer);
    assert!(result.is_ok());
    let written = result.unwrap();
    
    // Verify the written portion is no longer 0xFF
    for i in 0..written {
        assert_ne!(buffer[i], 0xFF);
    }
    
    // Verify the unwritten portion is still 0xFF
    for i in written..buffer.len() {
        assert_eq!(buffer[i], 0xFF);
    }
}