// this_file: rust/src/lib.rs
//! uubed-core: High-performance encoding library

pub mod encoders;
pub mod bindings;

// Re-export main functions
pub use encoders::{q64_encode, q64_decode};