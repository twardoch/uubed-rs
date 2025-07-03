#![no_main]

use libfuzzer_sys::fuzz_target;
use uubed_native::encoders::simhash_q64;
use arbitrary::{Arbitrary, Unstructured};

#[derive(Debug)]
struct SimhashInput {
    embedding: Vec<u8>,
    planes: usize,
}

impl<'a> Arbitrary<'a> for SimhashInput {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let max_embedding_size = 5000; // Reasonable limit for fuzzing
        let max_planes = 512; // Reasonable limit for planes
        
        let embedding_size = u.int_in_range(1..=max_embedding_size)?;
        let mut embedding = Vec::with_capacity(embedding_size);
        
        for _ in 0..embedding_size {
            embedding.push(u.arbitrary()?);
        }
        
        let planes = u.int_in_range(1..=max_planes)?;
        
        Ok(SimhashInput { embedding, planes })
    }
}

fuzz_target!(|input: SimhashInput| {
    let SimhashInput { embedding, planes } = input;
    
    // Test SimHash implementation
    let result = std::panic::catch_unwind(|| {
        simhash_q64(&embedding, planes)
    });
    
    match result {
        Ok(hash) => {
            // Verify output format
            let expected_bytes = (planes + 7) / 8; // Ceil division
            let expected_chars = expected_bytes * 2; // Q64 encoding: 1 byte = 2 chars
            
            assert_eq!(
                hash.len(),
                expected_chars,
                "SimHash output length mismatch: expected {} chars for {} planes, got {}",
                expected_chars,
                planes,
                hash.len()
            );
            
            // Verify all characters are valid Q64
            for ch in hash.chars() {
                assert!(
                    ch.is_ascii_alphanumeric() || ch == '_' || ch == '-',
                    "Invalid character '{}' in SimHash output",
                    ch
                );
            }
            
            // Test determinism - same input should produce same output
            let hash2 = simhash_q64(&embedding, planes);
            assert_eq!(
                hash, hash2,
                "SimHash is not deterministic for embedding size {} and {} planes",
                embedding.len(),
                planes
            );
            
            // Test that different embeddings (usually) produce different hashes
            if embedding.len() > 1 {
                let mut different_embedding = embedding.clone();
                // Flip one bit
                different_embedding[0] = different_embedding[0].wrapping_add(1);
                
                let different_hash = simhash_q64(&different_embedding, planes);
                // They might occasionally be the same due to hash collisions, but usually different
                // We just ensure this doesn't panic
                let _ = different_hash;
            }
        }
        Err(_) => {
            // SimHash should not panic for any reasonable input
            panic!(
                "SimHash panicked for embedding size {} and {} planes",
                embedding.len(),
                planes
            );
        }
    }
});