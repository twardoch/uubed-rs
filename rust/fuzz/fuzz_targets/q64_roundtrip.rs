#![no_main]

use libfuzzer_sys::fuzz_target;
use uubed_native::encoders::{q64_encode, q64_decode};

fuzz_target!(|data: Vec<u8>| {
    // Test Q64 roundtrip encoding/decoding
    let encoded = q64_encode(&data);
    
    // Verify the encoded string has the expected length
    assert_eq!(encoded.len(), data.len() * 2);
    
    // Verify roundtrip consistency
    match q64_decode(&encoded) {
        Ok(decoded) => {
            assert_eq!(decoded, data, "Q64 roundtrip failed for input: {:?}", data);
        }
        Err(e) => {
            panic!("Q64 decode failed for valid encoded data: {:?}, error: {:?}", encoded, e);
        }
    }
    
    // Verify the encoded string contains only valid characters
    for ch in encoded.chars() {
        assert!(
            ch.is_ascii_alphanumeric() || ch == '_' || ch == '-',
            "Invalid character '{}' in Q64 encoded string: {}",
            ch,
            encoded
        );
    }
});