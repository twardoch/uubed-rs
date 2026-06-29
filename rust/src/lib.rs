// this_file: rust/src/lib.rs
//! uubed-core: High-performance encoding library

pub mod encoders;
pub mod error;
pub mod parallel;
pub mod simd;

// Python bindings are optional and only compiled when PyO3 is available
#[cfg(feature = "python")]
pub mod bindings;

// C API is optional and only compiled when specifically enabled
#[cfg(feature = "capi")]
pub mod capi;

// Re-export main functions
pub use encoders::{mq64_decode, mq64_encode, q64_decode, q64_encode};
pub use error::{UubedError, UubedResult};
