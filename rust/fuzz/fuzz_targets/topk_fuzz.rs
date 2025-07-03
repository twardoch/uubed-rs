#![no_main]

use libfuzzer_sys::fuzz_target;
use uubed_native::encoders::{top_k_q64, topk_optimized};
use arbitrary::{Arbitrary, Unstructured};

#[derive(Debug)]
struct TopkInput {
    embedding: Vec<u8>,
    k: usize,
}

impl<'a> Arbitrary<'a> for TopkInput {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let max_size = 10000; // Limit size for performance
        let embedding_size = u.int_in_range(1..=max_size)?;
        let mut embedding = Vec::with_capacity(embedding_size);
        
        for _ in 0..embedding_size {
            embedding.push(u.arbitrary()?);
        }
        
        let k = u.int_in_range(1..=(embedding_size + 10))?; // Allow k > embedding_size
        
        Ok(TopkInput { embedding, k })
    }
}

fuzz_target!(|input: TopkInput| {
    let TopkInput { embedding, k } = input;
    
    // Test original top-k implementation
    let original_result = std::panic::catch_unwind(|| {
        top_k_q64(&embedding, k)
    });
    
    // Test optimized top-k implementation
    let optimized_result = std::panic::catch_unwind(|| {
        topk_optimized::top_k_q64_optimized(&embedding, k)
    });
    
    match (original_result, optimized_result) {
        (Ok(orig), Ok(opt)) => {
            // Both succeeded - they should produce the same result
            assert_eq!(
                orig, opt,
                "Top-k implementations disagree for embedding size {} and k {}",
                embedding.len(),
                k
            );
            
            // Verify output format
            let expected_len = k * 2; // k indices encoded as Q64 (2 chars per index)
            assert_eq!(
                orig.len(),
                expected_len,
                "Unexpected output length for k={}, got {} chars",
                k,
                orig.len()
            );
            
            // Verify all characters are valid Q64
            for ch in orig.chars() {
                assert!(
                    ch.is_ascii_alphanumeric() || ch == '_' || ch == '-',
                    "Invalid character '{}' in top-k output",
                    ch
                );
            }
        }
        (Err(_), Err(_)) => {
            // Both panicked - this should not happen for any valid input
            panic!(
                "Both top-k implementations panicked for embedding size {} and k {}",
                embedding.len(),
                k
            );
        }
        (Ok(_), Err(_)) => {
            panic!(
                "Optimized top-k panicked while original succeeded for size {} and k {}",
                embedding.len(),
                k
            );
        }
        (Err(_), Ok(_)) => {
            panic!(
                "Original top-k panicked while optimized succeeded for size {} and k {}",
                embedding.len(),
                k
            );
        }
    }
});