#![no_main]

use libfuzzer_sys::fuzz_target;
use uubed_native::encoders::q64_decode;

fuzz_target!(|data: Vec<u8>| {
    // Convert arbitrary bytes to a string for Q64 decoding
    if let Ok(input_str) = std::str::from_utf8(&data) {
        // Test Q64 decoding with arbitrary string input
        match q64_decode(input_str) {
            Ok(decoded) => {
                // If decoding succeeds, verify the relationship between input and output
                // Q64 should decode 2 characters to 1 byte
                if input_str.len() % 2 == 0 {
                    assert_eq!(decoded.len(), input_str.len() / 2);
                }
            }
            Err(_) => {
                // Decoding can fail for invalid input - this is expected
                // Just ensure it doesn't panic
            }
        }
    }
    
    // Also test with raw string conversion (may contain invalid UTF-8)
    let raw_string = String::from_utf8_lossy(&data);
    match q64_decode(&raw_string) {
        Ok(_) | Err(_) => {
            // Either result is fine, just ensure no panic
        }
    }
});