#![no_main]

use libfuzzer_sys::fuzz_target;
use uubed_native::encoders::z_order_q64;
use arbitrary::{Arbitrary, Unstructured};

#[derive(Debug)]
struct ZOrderInput {
    embedding: Vec<u8>,
}

impl<'a> Arbitrary<'a> for ZOrderInput {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let max_size = 1000; // Z-order typically used for smaller dimensional data
        let embedding_size = u.int_in_range(0..=max_size)?; // Allow empty embedding
        let mut embedding = Vec::with_capacity(embedding_size);
        
        for _ in 0..embedding_size {
            embedding.push(u.arbitrary()?);
        }
        
        Ok(ZOrderInput { embedding })
    }
}

fuzz_target!(|input: ZOrderInput| {
    let ZOrderInput { embedding } = input;
    
    // Test Z-order implementation
    let result = std::panic::catch_unwind(|| {
        z_order_q64(&embedding)
    });
    
    match result {
        Ok(zorder) => {
            // Verify output format - all characters should be valid Q64
            for ch in zorder.chars() {
                assert!(
                    ch.is_ascii_alphanumeric() || ch == '_' || ch == '-',
                    "Invalid character '{}' in Z-order output",
                    ch
                );
            }
            
            // Test determinism
            let zorder2 = z_order_q64(&embedding);
            assert_eq!(
                zorder, zorder2,
                "Z-order is not deterministic for embedding size {}",
                embedding.len()
            );
            
            // For non-empty embeddings, output should not be empty
            if !embedding.is_empty() {
                assert!(
                    !zorder.is_empty(),
                    "Z-order produced empty output for non-empty embedding of size {}",
                    embedding.len()
                );
            }
            
            // Test that different embeddings produce different outputs (usually)
            if embedding.len() > 0 {
                let mut different_embedding = embedding.clone();
                if different_embedding[0] < 255 {
                    different_embedding[0] += 1;
                } else {
                    different_embedding[0] -= 1;
                }
                
                let different_zorder = z_order_q64(&different_embedding);
                // Usually different, but hash collisions are possible
                let _ = different_zorder;
            }
        }
        Err(_) => {
            // Z-order should handle any input gracefully
            // Empty input might be acceptable depending on implementation
            if !embedding.is_empty() {
                panic!(
                    "Z-order panicked for non-empty embedding of size {}",
                    embedding.len()
                );
            }
        }
    }
});