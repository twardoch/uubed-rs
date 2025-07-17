use uubed_native::encoders::{q64_decode, mq64_decode};
use uubed_native::encoders::q64::Q64Error;
use uubed_native::error::{UubedError, UubedResult};

#[test]
fn test_q64_decode_invalid_characters() {
    // Test various invalid characters
    let invalid_inputs = [
        "invalid!",
        "hello@world",
        "test#123",
        "abc$def",
        "xyz%uvw",
        "contains^symbols",
        "has&ampersand",
        "with*asterisk",
        "plus+sign",
        "equals=sign",
        "question?mark",
        "exclamation!",
        "period.dot",
        "comma,here",
        "semicolon;here",
        "colon:here",
        "quotes'here",
        "double\"quotes",
        "brackets[here]",
        "braces{here}",
        "parens(here)",
        "less<than",
        "greater>than",
        "pipe|here",
        "backslash\\here",
        "forward/slash",
        "tilde~here",
        "backtick`here",
        "spaces here",
        "tab\there",
        "newline\nhere",
        "carriage\rreturn",
    ];
    
    for invalid in &invalid_inputs {
        let result = q64_decode(invalid);
        assert!(result.is_err(), "Should fail for input: {}", invalid);
        // Just check that it's an error - the specific error type depends on the input
    }
}

#[test]
fn test_q64_decode_odd_length_input() {
    let odd_length_inputs = [
        "a", "abc", "abcde", "abcdefg", "abcdefghi",
    ];
    
    for input in &odd_length_inputs {
        let result = q64_decode(input);
        assert!(result.is_err(), "Should fail for odd length input: {}", input);
    }
}

#[test]
fn test_q64_decode_empty_string() {
    let result = q64_decode("");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Vec::<u8>::new());
}

#[test]
fn test_q64_decode_valid_edge_cases() {
    // Test with valid Q64 character pairs (position-dependent)
    // pos 0: ABCDEFGHIJKLMNOP, pos 1: QRSTUVWXYZabcdef
    let valid_pairs = ["AQ", "AR", "AS", "BQ", "CQ"]; // Position 0 + Position 1 characters
    
    for pair in &valid_pairs {
        let result = q64_decode(pair);
        assert!(result.is_ok(), "Should succeed for valid pair: {}", pair);
        assert_eq!(result.unwrap().len(), 1);
    }
}

#[test]
fn test_mq64_decode_invalid_characters() {
    let invalid_inputs = [
        "invalid!@#",
        "contains spaces",
        "has\ttabs",
        "with\nnewlines",
        "special$chars",
        "unicode测试",
        "emoji🚀test",
    ];
    
    for invalid in &invalid_inputs {
        let result = mq64_decode(invalid);
        assert!(result.is_err(), "Should fail for input: {}", invalid);
    }
}

#[test]
fn test_mq64_decode_malformed_input() {
    let malformed_inputs = [
        "a", "abc", "abcde", // Odd lengths
        "ZZZZ", // Invalid Q64 sequences
        "----", // All same character
        "____", // All underscore
    ];
    
    for input in &malformed_inputs {
        let result = mq64_decode(input);
        assert!(result.is_err(), "Should fail for malformed input: {}", input);
    }
}

#[test]
fn test_error_display() {
    let error = Q64Error { message: "test error".to_string() };
    let display = format!("{}", error);
    assert!(display.contains("test error"));
}

#[test]
fn test_error_debug() {
    let error = Q64Error { message: "debug test".to_string() };
    let debug = format!("{:?}", error);
    assert!(debug.contains("Q64Error"));
    assert!(debug.contains("debug test"));
}

#[test]
fn test_ubbed_result_type() {
    // Test that UubedResult works as expected
    let success: UubedResult<Vec<u8>> = Ok(vec![1, 2, 3]);
    assert!(success.is_ok());
    
    let failure: UubedResult<Vec<u8>> = Err(UubedError::ValidationError(
        uubed_native::error::ValidationErrorKind::EmptyInput { operation: "test".to_string() }
    ));
    assert!(failure.is_err());
}

#[test]
fn test_large_invalid_input() {
    // Test with large invalid input to ensure no buffer overflows
    let large_invalid = "invalid_chars!@#$%^&*()".repeat(1000);
    let result = q64_decode(&large_invalid);
    assert!(result.is_err());
}

#[test]
fn test_boundary_characters() {
    // Test characters just outside the valid range
    let boundary_tests = [
        "@", // ASCII 64, just before 'A' (65)
        "[", // ASCII 91, just after 'Z' (90)
        "`", // ASCII 96, just before 'a' (97)
        "{", // ASCII 123, just after 'z' (122)
        "/", // ASCII 47, just before '0' (48)
        ":", // ASCII 58, just after '9' (57)
        ".", // ASCII 46, just before '-' (45)
        "^", // ASCII 94, just before '_' (95)
    ];
    
    for char in &boundary_tests {
        let test_input = format!("{}A", char); // Make it even length
        let result = q64_decode(&test_input);
        assert!(result.is_err(), "Should fail for boundary char: {}", char);
    }
}

#[test]
fn test_mixed_valid_invalid() {
    // Test strings that mix valid and invalid characters
    let mixed_tests = [
        "AB!C", // Valid-Invalid-Valid
        "A@BC", // Valid-Invalid-Valid-Valid
        "AB CD", // Valid-Valid-Space-Valid-Valid
        "AB\nCD", // Valid-Valid-Newline-Valid-Valid
    ];
    
    for test in &mixed_tests {
        let result = q64_decode(test);
        assert!(result.is_err(), "Should fail for mixed input: {}", test);
    }
}

#[test]
fn test_null_and_control_characters() {
    // Test with null and control characters
    let control_tests = [
        "AB\0CD", // Null character
        "AB\x01CD", // Control character
        "AB\x1FCD", // Control character
        "AB\x7FCD", // DEL character
    ];
    
    for test in &control_tests {
        let result = q64_decode(test);
        assert!(result.is_err(), "Should fail for control chars: {:?}", test);
    }
}

#[test]
fn test_high_ascii_characters() {
    // Test with high ASCII characters
    let high_ascii = "AB\u{0080}CD"; // High bit set
    let result = q64_decode(high_ascii);
    assert!(result.is_err(), "Should fail for high ASCII");
}

#[test]
fn test_case_sensitivity() {
    // Q64 is case-sensitive and position-dependent
    let valid1 = "AQ"; // Position 0 (A) + Position 1 (Q)
    let valid2 = "BR"; // Position 0 (B) + Position 1 (R)
    
    let result1 = q64_decode(valid1);
    let result2 = q64_decode(valid2);
    
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert_ne!(result1.unwrap(), result2.unwrap());
}

#[test]
fn test_error_propagation() {
    // Test that errors are properly propagated through the system
    fn test_function() -> Result<Vec<u8>, Q64Error> {
        q64_decode("invalid!@#")
    }
    
    let result = test_function();
    assert!(result.is_err());
}

#[test]
fn test_consistent_error_messages() {
    // Test that similar errors produce consistent messages
    let errors = [
        q64_decode("invalid!").unwrap_err(),
        q64_decode("bad@char").unwrap_err(),
        q64_decode("wrong#symbol").unwrap_err(),
    ];
    
    for error in &errors {
        let message = format!("{}", error);
        assert!(message.contains("error") || message.contains("Q64"));
    }
}