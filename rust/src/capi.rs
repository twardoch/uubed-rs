// this_file: rust/src/capi.rs
//! C API for uubed-rs
//! 
//! This module provides a C-compatible interface to the uubed encoding library,
//! enabling usage from C, C++, and other languages that support C FFI.
//! 
//! # Memory Management
//! 
//! The API follows RAII principles:
//! - Strings returned by encoding functions must be freed with `uubed_free_string`
//! - Error messages must be freed with `uubed_free_error_message`
//! - Contexts should be properly disposed with their respective free functions
//! 
//! # Error Handling
//! 
//! All functions return error codes. Use `uubed_get_last_error_message` to get
//! human-readable error descriptions.
//! 
//! # Thread Safety
//! 
//! All functions are thread-safe. Context objects can be used concurrently
//! from multiple threads.

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uchar, c_uint};
use libc::size_t;
use std::ptr;
use std::slice;
use std::sync::Mutex;

use crate::encoders::{q64_encode, q64_decode, q64_encode_to_buffer};
use crate::encoders::{simhash_q64, top_k_q64, top_k_q64_optimized, z_order_q64};
use crate::encoders::q64::Q64Error;
use crate::error::{UubedError, UubedResult};

/// Error codes for C API
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UubedErrorCode {
    /// Operation succeeded
    Success = 0,
    /// Q64 encoding/decoding error
    Q64Error = 1,
    /// SimHash computation error
    SimHashError = 2,
    /// Top-k selection error
    TopKError = 3,
    /// Z-order encoding error
    ZOrderError = 4,
    /// Input validation error
    ValidationError = 5,
    /// Memory allocation error
    MemoryError = 6,
    /// Internal computation error
    ComputationError = 7,
    /// Invalid parameter passed to function
    InvalidParameter = 8,
    /// Buffer too small for operation
    BufferTooSmall = 9,
    /// Unknown/unexpected error
    UnknownError = 10,
}

/// Thread-local storage for error messages
thread_local! {
    static LAST_ERROR: Mutex<Option<CString>> = Mutex::new(None);
}

/// Set the last error message for the current thread
fn set_last_error(error: &UubedError) {
    let error_msg = CString::new(error.to_string()).unwrap_or_else(|_| {
        CString::new("Failed to create error message").unwrap()
    });
    
    LAST_ERROR.with(|e| {
        *e.lock().unwrap() = Some(error_msg);
    });
}

/// Convert UubedError to error code
fn error_to_code(error: &UubedError) -> UubedErrorCode {
    match error {
        UubedError::Q64Error(_) => UubedErrorCode::Q64Error,
        UubedError::SimHashError(_) => UubedErrorCode::SimHashError,
        UubedError::TopKError(_) => UubedErrorCode::TopKError,
        UubedError::ZOrderError(_) => UubedErrorCode::ZOrderError,
        UubedError::ValidationError(_) => UubedErrorCode::ValidationError,
        UubedError::MemoryError(_) => UubedErrorCode::MemoryError,
        UubedError::ComputationError(_) => UubedErrorCode::ComputationError,
    }
}

/// Handle result and set error if needed
fn handle_result<T>(result: UubedResult<T>) -> Result<T, UubedErrorCode> {
    match result {
        Ok(value) => Ok(value),
        Err(error) => {
            let code = error_to_code(&error);
            set_last_error(&error);
            Err(code)
        }
    }
}

/// Handle Q64Error specifically
fn handle_q64_result<T>(result: Result<T, Q64Error>) -> Result<T, UubedErrorCode> {
    match result {
        Ok(value) => Ok(value),
        Err(error) => {
            let error_msg = CString::new(error.to_string()).unwrap_or_else(|_| {
                CString::new("Failed to create error message").unwrap()
            });
            
            LAST_ERROR.with(|e| {
                *e.lock().unwrap() = Some(error_msg);
            });
            Err(UubedErrorCode::Q64Error)
        }
    }
}

/// Get the last error message for the current thread
/// 
/// # Returns
/// 
/// Pointer to error message string, or NULL if no error.
/// The caller must NOT free this pointer - it's managed internally.
/// 
/// # Thread Safety
/// 
/// Thread-safe. Each thread maintains its own error state.
#[no_mangle]
pub extern "C" fn uubed_get_last_error_message() -> *const c_char {
    LAST_ERROR.with(|e| {
        match e.lock().unwrap().as_ref() {
            Some(msg) => msg.as_ptr(),
            None => ptr::null(),
        }
    })
}

/// Clear the last error message for the current thread
#[no_mangle]
pub extern "C" fn uubed_clear_last_error() {
    LAST_ERROR.with(|e| {
        *e.lock().unwrap() = None;
    });
}

/// Encode binary data using Q64 algorithm
/// 
/// # Parameters
/// 
/// * `data` - Input binary data
/// * `data_len` - Length of input data in bytes
/// * `output` - Pointer to store the encoded string (caller must free with uubed_free_string)
/// 
/// # Returns
/// 
/// Error code indicating success or failure
/// 
/// # Memory Management
/// 
/// The output string must be freed using `uubed_free_string`.
/// 
/// # Example
/// 
/// ```c
/// const uint8_t data[] = {0x12, 0x34, 0x56};
/// char* encoded = NULL;
/// UubedErrorCode result = uubed_q64_encode(data, 3, &encoded);
/// if (result == UUBED_SUCCESS) {
///     printf("Encoded: %s\n", encoded);
///     uubed_free_string(encoded);
/// }
/// ```
#[no_mangle]
pub extern "C" fn uubed_q64_encode(
    data: *const c_uchar,
    data_len: size_t,
    output: *mut *mut c_char,
) -> UubedErrorCode {
    if data.is_null() || output.is_null() {
        return UubedErrorCode::InvalidParameter;
    }
    
    if data_len == 0 {
        // Handle empty input
        match CString::new("") {
            Ok(empty_str) => {
                unsafe { *output = empty_str.into_raw(); }
                return UubedErrorCode::Success;
            }
            Err(_) => return UubedErrorCode::MemoryError,
        }
    }
    
    let input_slice = unsafe { slice::from_raw_parts(data, data_len) };
    
    let encoded = q64_encode(input_slice);
    
    match CString::new(encoded) {
        Ok(c_string) => {
            unsafe { *output = c_string.into_raw(); }
            UubedErrorCode::Success
        }
        Err(_) => UubedErrorCode::MemoryError,
    }
}

/// Decode Q64-encoded string back to binary data
/// 
/// # Parameters
/// 
/// * `encoded` - Q64-encoded string (null-terminated)
/// * `output` - Pointer to store decoded data (caller must free with uubed_free_bytes)
/// * `output_len` - Pointer to store length of decoded data
/// 
/// # Returns
/// 
/// Error code indicating success or failure
#[no_mangle]
pub extern "C" fn uubed_q64_decode(
    encoded: *const c_char,
    output: *mut *mut c_uchar,
    output_len: *mut size_t,
) -> UubedErrorCode {
    if encoded.is_null() || output.is_null() || output_len.is_null() {
        return UubedErrorCode::InvalidParameter;
    }
    
    let encoded_str = unsafe {
        match CStr::from_ptr(encoded).to_str() {
            Ok(s) => s,
            Err(_) => return UubedErrorCode::InvalidParameter,
        }
    };
    
    match handle_q64_result(q64_decode(encoded_str)) {
        Ok(decoded) => {
            let len = decoded.len();
            let boxed = decoded.into_boxed_slice();
            let ptr = Box::into_raw(boxed) as *mut c_uchar;
            
            unsafe {
                *output = ptr;
                *output_len = len;
            }
            UubedErrorCode::Success
        }
        Err(code) => code,
    }
}

/// Zero-copy Q64 encoding into pre-allocated buffer
/// 
/// # Parameters
/// 
/// * `data` - Input binary data
/// * `data_len` - Length of input data
/// * `output_buffer` - Pre-allocated output buffer
/// * `buffer_len` - Size of output buffer (must be at least data_len * 2)
/// * `bytes_written` - Pointer to store number of bytes written
/// 
/// # Returns
/// 
/// Error code indicating success or failure
#[no_mangle]
pub extern "C" fn uubed_q64_encode_to_buffer(
    data: *const c_uchar,
    data_len: size_t,
    output_buffer: *mut c_uchar,
    buffer_len: size_t,
    bytes_written: *mut size_t,
) -> UubedErrorCode {
    if data.is_null() || output_buffer.is_null() || bytes_written.is_null() {
        return UubedErrorCode::InvalidParameter;
    }
    
    let input_slice = unsafe { slice::from_raw_parts(data, data_len) };
    let output_slice = unsafe { slice::from_raw_parts_mut(output_buffer, buffer_len) };
    
    match handle_q64_result(q64_encode_to_buffer(input_slice, output_slice)) {
        Ok(written) => {
            unsafe { *bytes_written = written; }
            UubedErrorCode::Success
        }
        Err(code) => code,
    }
}

/// Encode embedding using SimHash algorithm
/// 
/// # Parameters
/// 
/// * `embedding` - Input embedding data
/// * `embedding_len` - Length of embedding
/// * `planes` - Number of hash planes (must be > 0)
/// * `output` - Pointer to store encoded string
/// 
/// # Returns
/// 
/// Error code indicating success or failure
#[no_mangle]
pub extern "C" fn uubed_simhash_encode(
    embedding: *const c_uchar,
    embedding_len: size_t,
    planes: c_uint,
    output: *mut *mut c_char,
) -> UubedErrorCode {
    if embedding.is_null() || output.is_null() || planes == 0 {
        return UubedErrorCode::InvalidParameter;
    }
    
    let input_slice = unsafe { slice::from_raw_parts(embedding, embedding_len) };
    
    let encoded = simhash_q64(input_slice, planes as usize);
    
    match CString::new(encoded) {
        Ok(c_string) => {
            unsafe { *output = c_string.into_raw(); }
            UubedErrorCode::Success
        }
        Err(_) => UubedErrorCode::MemoryError,
    }
}

/// Encode embedding using Top-K algorithm
/// 
/// # Parameters
/// 
/// * `embedding` - Input embedding data
/// * `embedding_len` - Length of embedding
/// * `k` - Number of top elements to select (must be > 0)
/// * `output` - Pointer to store encoded string
/// 
/// # Returns
/// 
/// Error code indicating success or failure
#[no_mangle]
pub extern "C" fn uubed_topk_encode(
    embedding: *const c_uchar,
    embedding_len: size_t,
    k: c_uint,
    output: *mut *mut c_char,
) -> UubedErrorCode {
    if embedding.is_null() || output.is_null() || k == 0 {
        return UubedErrorCode::InvalidParameter;
    }
    
    let input_slice = unsafe { slice::from_raw_parts(embedding, embedding_len) };
    
    let encoded = top_k_q64(input_slice, k as usize);
    
    match CString::new(encoded) {
        Ok(c_string) => {
            unsafe { *output = c_string.into_raw(); }
            UubedErrorCode::Success
        }
        Err(_) => UubedErrorCode::MemoryError,
    }
}

/// Encode embedding using optimized Top-K algorithm
/// 
/// # Parameters
/// 
/// * `embedding` - Input embedding data
/// * `embedding_len` - Length of embedding
/// * `k` - Number of top elements to select (must be > 0)
/// * `output` - Pointer to store encoded string
/// 
/// # Returns
/// 
/// Error code indicating success or failure
#[no_mangle]
pub extern "C" fn uubed_topk_encode_optimized(
    embedding: *const c_uchar,
    embedding_len: size_t,
    k: c_uint,
    output: *mut *mut c_char,
) -> UubedErrorCode {
    if embedding.is_null() || output.is_null() || k == 0 {
        return UubedErrorCode::InvalidParameter;
    }
    
    let input_slice = unsafe { slice::from_raw_parts(embedding, embedding_len) };
    
    let encoded = top_k_q64_optimized(input_slice, k as usize);
    
    match CString::new(encoded) {
        Ok(c_string) => {
            unsafe { *output = c_string.into_raw(); }
            UubedErrorCode::Success
        }
        Err(_) => UubedErrorCode::MemoryError,
    }
}

/// Encode embedding using Z-order (Morton) algorithm
/// 
/// # Parameters
/// 
/// * `embedding` - Input embedding data
/// * `embedding_len` - Length of embedding
/// * `output` - Pointer to store encoded string
/// 
/// # Returns
/// 
/// Error code indicating success or failure
#[no_mangle]
pub extern "C" fn uubed_zorder_encode(
    embedding: *const c_uchar,
    embedding_len: size_t,
    output: *mut *mut c_char,
) -> UubedErrorCode {
    if embedding.is_null() || output.is_null() {
        return UubedErrorCode::InvalidParameter;
    }
    
    let input_slice = unsafe { slice::from_raw_parts(embedding, embedding_len) };
    
    let encoded = z_order_q64(input_slice);
    
    match CString::new(encoded) {
        Ok(c_string) => {
            unsafe { *output = c_string.into_raw(); }
            UubedErrorCode::Success
        }
        Err(_) => UubedErrorCode::MemoryError,
    }
}

/// Free a string allocated by uubed encoding functions
/// 
/// # Parameters
/// 
/// * `s` - String to free (must be allocated by uubed functions)
/// 
/// # Safety
/// 
/// The pointer must have been returned by an uubed encoding function.
/// After calling this function, the pointer is invalid and must not be used.
#[no_mangle]
pub extern "C" fn uubed_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}

/// Free bytes allocated by uubed decoding functions
/// 
/// # Parameters
/// 
/// * `bytes` - Bytes to free
/// * `len` - Length of the byte array
/// 
/// # Safety
/// 
/// The pointer must have been returned by an uubed decoding function.
#[no_mangle]
pub extern "C" fn uubed_free_bytes(bytes: *mut c_uchar, len: size_t) {
    if !bytes.is_null() && len > 0 {
        unsafe {
            let _ = Vec::from_raw_parts(bytes, len, len);
        }
    }
}

/// Get version information
/// 
/// # Returns
/// 
/// Version string (do not free - statically allocated)
#[no_mangle]
pub extern "C" fn uubed_get_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const c_char
}

/// Check if SIMD optimizations are available
/// 
/// # Returns
/// 
/// 1 if SIMD is available, 0 otherwise
#[no_mangle]
pub extern "C" fn uubed_has_simd_support() -> c_int {
    #[cfg(all(target_arch = "x86_64", feature = "simd"))]
    {
        1
    }
    #[cfg(not(all(target_arch = "x86_64", feature = "simd")))]
    {
        0
    }
}

/// Get maximum supported embedding size
/// 
/// # Returns
/// 
/// Maximum embedding size in bytes
#[no_mangle]
pub extern "C" fn uubed_max_embedding_size() -> size_t {
    crate::error::validation::MAX_EMBEDDING_SIZE
}

/// Get maximum supported k value for top-k operations
/// 
/// # Returns
/// 
/// Maximum k value
#[no_mangle]
pub extern "C" fn uubed_max_k_value() -> size_t {
    crate::error::validation::MAX_K_VALUE
}

/// Get maximum supported planes for SimHash operations
/// 
/// # Returns
/// 
/// Maximum planes value
#[no_mangle]
pub extern "C" fn uubed_max_simhash_planes() -> size_t {
    crate::error::validation::MAX_SIMHASH_PLANES
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;
    
    #[test]
    fn test_c_api_q64_roundtrip() {
        let test_data = vec![0x12, 0x34, 0x56, 0x78];
        let mut encoded_ptr: *mut c_char = ptr::null_mut();
        
        // Test encoding
        let result = uubed_q64_encode(
            test_data.as_ptr(),
            test_data.len(),
            &mut encoded_ptr,
        );
        assert_eq!(result, UubedErrorCode::Success);
        assert!(!encoded_ptr.is_null());
        
        // Test decoding
        let mut decoded_ptr: *mut c_uchar = ptr::null_mut();
        let mut decoded_len: size_t = 0;
        
        let result = uubed_q64_decode(
            encoded_ptr,
            &mut decoded_ptr,
            &mut decoded_len,
        );
        assert_eq!(result, UubedErrorCode::Success);
        assert!(!decoded_ptr.is_null());
        assert_eq!(decoded_len, test_data.len());
        
        // Verify data
        let decoded_slice = unsafe {
            slice::from_raw_parts(decoded_ptr, decoded_len)
        };
        assert_eq!(decoded_slice, test_data.as_slice());
        
        // Cleanup
        uubed_free_string(encoded_ptr);
        uubed_free_bytes(decoded_ptr, decoded_len);
    }
    
    #[test]
    fn test_c_api_error_handling() {
        let mut output: *mut c_char = ptr::null_mut();
        
        // Test null pointer
        let result = uubed_q64_encode(ptr::null(), 0, &mut output);
        assert_eq!(result, UubedErrorCode::InvalidParameter);
        
        // Test invalid Q64 string
        let invalid_q64 = CString::new("invalid!").unwrap();
        let mut decoded_ptr: *mut c_uchar = ptr::null_mut();
        let mut decoded_len: size_t = 0;
        
        let result = uubed_q64_decode(
            invalid_q64.as_ptr(),
            &mut decoded_ptr,
            &mut decoded_len,
        );
        assert_ne!(result, UubedErrorCode::Success);
        
        // Check error message is available
        let error_msg = uubed_get_last_error_message();
        assert!(!error_msg.is_null());
    }
    
    #[test]
    fn test_c_api_zero_copy() {
        let test_data = vec![0xAB, 0xCD, 0xEF];
        let mut buffer = vec![0u8; test_data.len() * 2];
        let mut bytes_written: size_t = 0;
        
        let result = uubed_q64_encode_to_buffer(
            test_data.as_ptr(),
            test_data.len(),
            buffer.as_mut_ptr(),
            buffer.len(),
            &mut bytes_written,
        );
        
        assert_eq!(result, UubedErrorCode::Success);
        assert_eq!(bytes_written, test_data.len() * 2);
    }
    
    #[test]
    fn test_c_api_version_info() {
        let version = uubed_get_version();
        assert!(!version.is_null());
        
        let simd_support = uubed_has_simd_support();
        assert!(simd_support == 0 || simd_support == 1);
        
        let max_size = uubed_max_embedding_size();
        assert!(max_size > 0);
    }
}