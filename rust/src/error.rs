// this_file: rust/src/error.rs
/// Comprehensive error handling for uubed-rs

use std::error::Error;
use std::fmt;

/// Main error type for all uubed operations
#[derive(Debug, Clone, PartialEq)]
pub enum UubedError {
    /// Q64 encoding/decoding errors
    Q64Error(Q64ErrorKind),
    /// SimHash computation errors
    SimHashError(SimHashErrorKind),
    /// Top-k selection errors
    TopKError(TopKErrorKind),
    /// Z-order encoding errors
    ZOrderError(ZOrderErrorKind),
    /// Input validation errors
    ValidationError(ValidationErrorKind),
    /// Memory allocation or capacity errors
    MemoryError(String),
    /// Internal computation errors
    ComputationError(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Q64ErrorKind {
    /// Input string has odd length (Q64 requires even length)
    OddLength { length: usize },
    /// Invalid character found in Q64 string
    InvalidCharacter { character: char, position: usize },
    /// Character found at wrong position for Q64 scheme
    WrongPosition { character: char, position: usize, expected_alphabet: u8 },
    /// Output buffer capacity exceeded
    BufferOverflow { required: usize, available: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub enum SimHashErrorKind {
    /// Number of planes is invalid (must be > 0)
    InvalidPlanes { planes: usize },
    /// Embedding dimensions too large for matrix generation
    DimensionsTooLarge { dimensions: usize, max_supported: usize },
    /// Matrix generation failed
    MatrixGenerationFailed { planes: usize, dimensions: usize },
    /// Random number generation failed
    RngFailure { source: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum TopKErrorKind {
    /// k value is invalid (must be > 0)
    InvalidK { k: usize },
    /// k value exceeds maximum supported
    KTooLarge { k: usize, max_supported: usize },
    /// Embedding too large for index representation
    EmbeddingTooLarge { size: usize, max_supported: usize },
    /// Parallel processing failed
    ParallelProcessingFailed { source: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ZOrderErrorKind {
    /// Embedding dimensions not suitable for Z-order encoding
    UnsuitableDimensions { dimensions: usize, reason: String },
    /// Bit manipulation overflow
    BitOverflow { value: u64, max_bits: u32 },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationErrorKind {
    /// Input is empty when non-empty input is required
    EmptyInput { operation: String },
    /// Input size exceeds maximum allowed
    InputTooLarge { size: usize, max_size: usize, operation: String },
    /// Input contains invalid values
    InvalidInputValues { details: String },
    /// Parameters are incompatible
    IncompatibleParameters { details: String },
}

impl fmt::Display for UubedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UubedError::Q64Error(kind) => write!(f, "Q64 error: {}", kind),
            UubedError::SimHashError(kind) => write!(f, "SimHash error: {}", kind),
            UubedError::TopKError(kind) => write!(f, "Top-k error: {}", kind),
            UubedError::ZOrderError(kind) => write!(f, "Z-order error: {}", kind),
            UubedError::ValidationError(kind) => write!(f, "Validation error: {}", kind),
            UubedError::MemoryError(msg) => write!(f, "Memory error: {}", msg),
            UubedError::ComputationError(msg) => write!(f, "Computation error: {}", msg),
        }
    }
}

impl fmt::Display for Q64ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Q64ErrorKind::OddLength { length } => {
                write!(f, "Input has odd length {}, Q64 requires even length", length)
            }
            Q64ErrorKind::InvalidCharacter { character, position } => {
                write!(f, "Invalid character '{}' at position {}", character, position)
            }
            Q64ErrorKind::WrongPosition { character, position, expected_alphabet } => {
                write!(f, "Character '{}' at position {} belongs to alphabet {}, not expected alphabet", 
                       character, position, expected_alphabet)
            }
            Q64ErrorKind::BufferOverflow { required, available } => {
                write!(f, "Buffer overflow: need {} bytes, only {} available", required, available)
            }
        }
    }
}

impl fmt::Display for SimHashErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SimHashErrorKind::InvalidPlanes { planes } => {
                write!(f, "Invalid number of planes: {}, must be > 0", planes)
            }
            SimHashErrorKind::DimensionsTooLarge { dimensions, max_supported } => {
                write!(f, "Dimensions {} exceed maximum supported {}", dimensions, max_supported)
            }
            SimHashErrorKind::MatrixGenerationFailed { planes, dimensions } => {
                write!(f, "Failed to generate matrix for {} planes × {} dimensions", planes, dimensions)
            }
            SimHashErrorKind::RngFailure { source } => {
                write!(f, "Random number generation failed: {}", source)
            }
        }
    }
}

impl fmt::Display for TopKErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TopKErrorKind::InvalidK { k } => {
                write!(f, "Invalid k value: {}, must be > 0", k)
            }
            TopKErrorKind::KTooLarge { k, max_supported } => {
                write!(f, "k value {} exceeds maximum supported {}", k, max_supported)
            }
            TopKErrorKind::EmbeddingTooLarge { size, max_supported } => {
                write!(f, "Embedding size {} exceeds maximum supported {}", size, max_supported)
            }
            TopKErrorKind::ParallelProcessingFailed { source } => {
                write!(f, "Parallel processing failed: {}", source)
            }
        }
    }
}

impl fmt::Display for ZOrderErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ZOrderErrorKind::UnsuitableDimensions { dimensions, reason } => {
                write!(f, "Unsuitable dimensions {}: {}", dimensions, reason)
            }
            ZOrderErrorKind::BitOverflow { value, max_bits } => {
                write!(f, "Bit overflow: value {} exceeds {} bits", value, max_bits)
            }
        }
    }
}

impl fmt::Display for ValidationErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationErrorKind::EmptyInput { operation } => {
                write!(f, "Empty input not allowed for operation: {}", operation)
            }
            ValidationErrorKind::InputTooLarge { size, max_size, operation } => {
                write!(f, "Input size {} exceeds maximum {} for operation: {}", size, max_size, operation)
            }
            ValidationErrorKind::InvalidInputValues { details } => {
                write!(f, "Invalid input values: {}", details)
            }
            ValidationErrorKind::IncompatibleParameters { details } => {
                write!(f, "Incompatible parameters: {}", details)
            }
        }
    }
}

impl Error for UubedError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl Error for Q64ErrorKind {}
impl Error for SimHashErrorKind {}
impl Error for TopKErrorKind {}
impl Error for ZOrderErrorKind {}
impl Error for ValidationErrorKind {}

/// Result type for uubed operations
pub type UubedResult<T> = Result<T, UubedError>;

/// Input validation utilities
pub mod validation {
    use super::*;
    
    /// Maximum supported embedding size (16MB)
    pub const MAX_EMBEDDING_SIZE: usize = 16777216; // 16 * 1024 * 1024
    
    /// Maximum supported k value for top-k operations
    pub const MAX_K_VALUE: usize = 100_000;
    
    /// Maximum supported planes for SimHash
    pub const MAX_SIMHASH_PLANES: usize = 8192;
    
    /// Maximum supported dimensions for SimHash
    pub const MAX_SIMHASH_DIMENSIONS: usize = 1_000_000;
    
    /// Validate embedding input
    pub fn validate_embedding(embedding: &[u8], operation: &str) -> UubedResult<()> {
        if embedding.is_empty() {
            return Err(UubedError::ValidationError(ValidationErrorKind::EmptyInput {
                operation: operation.to_string(),
            }));
        }
        
        if embedding.len() > MAX_EMBEDDING_SIZE {
            return Err(UubedError::ValidationError(ValidationErrorKind::InputTooLarge {
                size: embedding.len(),
                max_size: MAX_EMBEDDING_SIZE,
                operation: operation.to_string(),
            }));
        }
        
        Ok(())
    }
    
    /// Validate k parameter for top-k operations
    pub fn validate_k(k: usize) -> UubedResult<()> {
        if k == 0 {
            return Err(UubedError::TopKError(TopKErrorKind::InvalidK { k }));
        }
        
        if k > MAX_K_VALUE {
            return Err(UubedError::TopKError(TopKErrorKind::KTooLarge {
                k,
                max_supported: MAX_K_VALUE,
            }));
        }
        
        Ok(())
    }
    
    /// Validate SimHash parameters
    pub fn validate_simhash_params(planes: usize, dimensions: usize) -> UubedResult<()> {
        if planes == 0 {
            return Err(UubedError::SimHashError(SimHashErrorKind::InvalidPlanes { planes }));
        }
        
        if planes > MAX_SIMHASH_PLANES {
            return Err(UubedError::SimHashError(SimHashErrorKind::InvalidPlanes { planes }));
        }
        
        if dimensions > MAX_SIMHASH_DIMENSIONS {
            return Err(UubedError::SimHashError(SimHashErrorKind::DimensionsTooLarge {
                dimensions,
                max_supported: MAX_SIMHASH_DIMENSIONS,
            }));
        }
        
        Ok(())
    }
    
    /// Validate Q64 string format
    pub fn validate_q64_string(s: &str) -> UubedResult<()> {
        if s.len() % 2 != 0 {
            return Err(UubedError::Q64Error(Q64ErrorKind::OddLength {
                length: s.len(),
            }));
        }
        
        for (pos, ch) in s.chars().enumerate() {
            if !is_q64_character(ch, pos) {
                return Err(UubedError::Q64Error(Q64ErrorKind::InvalidCharacter {
                    character: ch,
                    position: pos,
                }));
            }
        }
        
        Ok(())
    }
    
    fn is_q64_character(ch: char, _position: usize) -> bool {
        // Q64 uses different alphabets based on position
        // This is a simplified check - actual validation would need the alphabet tables
        ch.is_ascii_alphanumeric() || ch == '_' || ch == '-'
    }
}

/// Error recovery utilities
pub mod recovery {
    use super::*;
    
    /// Attempt to recover from Q64 decoding errors by cleaning the input
    pub fn recover_q64_decode(input: &str) -> UubedResult<String> {
        let mut cleaned = String::new();
        
        for ch in input.chars() {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                cleaned.push(ch);
            }
        }
        
        // Ensure even length
        if cleaned.len() % 2 != 0 {
            cleaned.pop(); // Remove last character to make even
        }
        
        if cleaned.is_empty() {
            return Err(UubedError::ValidationError(ValidationErrorKind::EmptyInput {
                operation: "Q64 decode recovery".to_string(),
            }));
        }
        
        Ok(cleaned)
    }
    
    /// Clamp k value to valid range for embedding size
    pub fn clamp_k_value(k: usize, embedding_size: usize) -> usize {
        k.min(embedding_size).min(validation::MAX_K_VALUE).max(1)
    }
    
    /// Clamp planes value to valid range
    pub fn clamp_planes_value(planes: usize) -> usize {
        planes.min(validation::MAX_SIMHASH_PLANES).max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_display() {
        let err = UubedError::Q64Error(Q64ErrorKind::OddLength { length: 5 });
        assert!(err.to_string().contains("odd length"));
        
        let err = UubedError::TopKError(TopKErrorKind::InvalidK { k: 0 });
        assert!(err.to_string().contains("Invalid k value"));
    }
    
    #[test]
    fn test_validation() {
        // Test empty input validation
        let empty: Vec<u8> = vec![];
        assert!(validation::validate_embedding(&empty, "test").is_err());
        
        // Test valid input
        let valid = vec![1, 2, 3, 4, 5];
        assert!(validation::validate_embedding(&valid, "test").is_ok());
        
        // Test k validation
        assert!(validation::validate_k(0).is_err());
        assert!(validation::validate_k(100).is_ok());
        
        // Test SimHash validation
        assert!(validation::validate_simhash_params(0, 100).is_err());
        assert!(validation::validate_simhash_params(64, 1000).is_ok());
    }
    
    #[test]
    fn test_recovery() {
        // Test Q64 recovery
        let dirty = "abc!@#def$%^";
        let cleaned = recovery::recover_q64_decode(dirty).unwrap();
        assert_eq!(cleaned, "abcdef");
        
        // Test k clamping
        assert_eq!(recovery::clamp_k_value(0, 100), 1);
        assert_eq!(recovery::clamp_k_value(1000, 500), 500);
        assert_eq!(recovery::clamp_k_value(200_000, 100), 100);
    }
}