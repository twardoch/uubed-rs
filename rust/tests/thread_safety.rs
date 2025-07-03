use std::sync::Arc;
use std::thread;
use uubed_native::encoders::{q64_encode, simhash_q64, top_k_q64, z_order_q64};

#[test]
fn test_concurrent_q64_encoding() {
    let data = Arc::new(vec![1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    let mut handles = vec![];
    
    for _ in 0..100 {
        let data = Arc::clone(&data);
        let handle = thread::spawn(move || {
            q64_encode(&data)
        });
        handles.push(handle);
    }
    
    let results: Vec<_> = handles.into_iter()
        .map(|h| h.join().unwrap())
        .collect();
    
    // All threads should produce the same result
    let expected = &results[0];
    for result in &results[1..] {
        assert_eq!(expected, result);
    }
}

#[test]
fn test_concurrent_simhash() {
    let embedding = Arc::new(vec![10u8; 256]);
    let mut handles = vec![];
    
    for _ in 0..50 {
        let emb = Arc::clone(&embedding);
        let handle = thread::spawn(move || {
            simhash_q64(&emb, 64)
        });
        handles.push(handle);
    }
    
    let results: Vec<_> = handles.into_iter()
        .map(|h| h.join().unwrap())
        .collect();
    
    // All results should be identical for the same input
    let expected = &results[0];
    for result in &results[1..] {
        assert_eq!(expected, result);
    }
}

#[test]
fn test_concurrent_topk() {
    let embedding = Arc::new((0..1000u16).map(|i| (i % 256) as u8).collect::<Vec<_>>());
    let mut handles = vec![];
    
    for _ in 0..20 {
        let emb = Arc::clone(&embedding);
        let handle = thread::spawn(move || {
            top_k_q64(&emb, 16)
        });
        handles.push(handle);
    }
    
    let results: Vec<_> = handles.into_iter()
        .map(|h| h.join().unwrap())
        .collect();
    
    // All results should be identical
    let expected = &results[0];
    for result in &results[1..] {
        assert_eq!(expected, result);
    }
}

#[test]
fn test_concurrent_zorder() {
    let embedding = Arc::new(vec![100u8; 128]);
    let mut handles = vec![];
    
    for _ in 0..30 {
        let emb = Arc::clone(&embedding);
        let handle = thread::spawn(move || {
            z_order_q64(&emb)
        });
        handles.push(handle);
    }
    
    let results: Vec<_> = handles.into_iter()
        .map(|h| h.join().unwrap())
        .collect();
    
    // All results should be identical
    let expected = &results[0];
    for result in &results[1..] {
        assert_eq!(expected, result);
    }
}

#[test]
fn test_mixed_concurrent_operations() {
    use rayon::prelude::*;
    
    // Test all encoders running concurrently on different data
    let results: Vec<(String, String, String, String)> = (0..100)
        .into_par_iter()
        .map(|i| {
            let data = vec![(i % 256) as u8; 256];
            let q64 = q64_encode(&data[..16]);
            let simhash = simhash_q64(&data, 64);
            let topk = top_k_q64(&data, 8);
            let zorder = z_order_q64(&data[..64]);
            (q64, simhash, topk, zorder)
        })
        .collect();
    
    // Verify we got results for all operations
    assert_eq!(results.len(), 100);
    
    // Verify deterministic results for same input
    for i in 0..100 {
        for j in 0..100 {
            if i % 256 == j % 256 {
                assert_eq!(results[i], results[j]);
            }
        }
    }
}