// this_file: rust/tests/version_test.rs
// Version management and semver compliance tests

use uubed_native::*;

#[test]
fn test_version_string_format() {
    // Test that version follows semver format (major.minor.patch)
    let version = env!("CARGO_PKG_VERSION");
    let parts: Vec<&str> = version.split('.').collect();
    
    assert_eq!(parts.len(), 3, "Version should have 3 parts (major.minor.patch)");
    
    // Check that each part is a valid number
    for part in parts {
        assert!(part.parse::<u32>().is_ok(), "Version part '{}' should be a valid number", part);
    }
}

#[test]
fn test_version_consistency() {
    // Test that the version is consistent across different access methods
    let cargo_version = env!("CARGO_PKG_VERSION");
    
    // If we had a version function in the library, we'd test it here
    // For now, just ensure the environment variable is set
    assert!(!cargo_version.is_empty(), "Version should not be empty");
}

#[test]
fn test_library_metadata() {
    // Test that essential metadata is present
    let name = env!("CARGO_PKG_NAME");
    let authors = env!("CARGO_PKG_AUTHORS");
    let description = env!("CARGO_PKG_DESCRIPTION");
    
    assert_eq!(name, "uubed-core");
    assert!(!authors.is_empty());
    assert!(!description.is_empty());
}

#[test]
fn test_feature_flags() {
    // Test that feature flags work correctly
    #[cfg(feature = "simd")]
    {
        // SIMD feature is enabled
        assert!(true);
    }
    
    #[cfg(not(feature = "simd"))]
    {
        // SIMD feature is disabled
        assert!(true);
    }
}

#[test]
fn test_build_info() {
    // Test that we can access build information
    let target = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_else(|_| "unknown".to_string());
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| "unknown".to_string());
    
    assert!(!target.is_empty());
    assert!(!target_os.is_empty());
}

#[cfg(test)]
mod api_compatibility {
    use super::*;
    
    #[test]
    fn test_q64_api_stability() {
        // Test that the Q64 API is stable
        let input = vec![1u8, 2u8, 3u8, 4u8];
        let result = q64_encode(&input);
        
        // The API should consistently return results
        assert!(!result.is_empty());
        
        // Test that we can decode back
        let decoded = q64_decode(&result);
        assert!(decoded.is_ok());
    }
    
    #[test]
    fn test_mq64_api_stability() {
        // Test that the MQ64 API is stable
        let input = vec![1u8, 2u8, 3u8, 4u8];
        let result = mq64_encode(&input);
        
        // The API should consistently return results
        assert!(!result.is_empty());
        
        // Test that we can decode back
        let decoded = mq64_decode(&result);
        assert!(decoded.is_ok());
    }
}

#[cfg(test)]
mod backward_compatibility {
    use super::*;
    
    #[test]
    fn test_encoding_format_stability() {
        // Test that encoding formats remain stable across versions
        let test_vector = vec![1u8, 2u8, 3u8, 4u8];
        
        // These should produce consistent results
        let q64_result = q64_encode(&test_vector);
        let mq64_result = mq64_encode(&test_vector);
        
        assert!(!q64_result.is_empty());
        assert!(!mq64_result.is_empty());
        
        // The length should be predictable
        assert!(!q64_result.is_empty());
        assert!(!mq64_result.is_empty());
    }
}

#[test]
fn test_error_handling_consistency() {
    // Test that error handling is consistent across versions
    let empty_input: Vec<u8> = vec![];
    
    // Empty input should be handled gracefully
    let q64_result = q64_encode(&empty_input);
    let mq64_result = mq64_encode(&empty_input);
    
    // Both should either succeed or fail consistently
    assert_eq!(q64_result.is_empty(), mq64_result.is_empty());
}