// this_file: rust/src/bindings.rs
//! Python bindings for uubed-core using PyO3.

use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;

/// Encode bytes using Q64 algorithm
#[pyfunction]
#[pyo3(signature = (data))]
fn q64_encode_native(data: &[u8]) -> String {
    crate::encoders::q64_encode(data)
}

/// Decode Q64 string to bytes
#[pyfunction]
#[pyo3(signature = (encoded))]
fn q64_decode_native(encoded: &str) -> PyResult<Vec<u8>> {
    crate::encoders::q64_decode(encoded)
        .map_err(|e| PyValueError::new_err(e.to_string()))
}

/// Generate SimHash with Q64 encoding
#[pyfunction]
#[pyo3(signature = (embedding, planes=64))]
fn simhash_q64_native(embedding: &[u8], planes: usize) -> String {
    crate::encoders::simhash_q64(embedding, planes)
}

/// Generate top-k indices with Q64 encoding
#[pyfunction]
#[pyo3(signature = (embedding, k=8))]
fn top_k_q64_native(embedding: &[u8], k: usize) -> String {
    crate::encoders::top_k_q64(embedding, k)
}

/// Generate Z-order with Q64 encoding
#[pyfunction]
#[pyo3(signature = (embedding))]
fn z_order_q64_native(embedding: &[u8]) -> String {
    crate::encoders::z_order_q64(embedding)
}

/// Python module initialization
#[pymodule]
fn uubed_native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(q64_encode_native, m)?)?;
    m.add_function(wrap_pyfunction!(q64_decode_native, m)?)?;
    m.add_function(wrap_pyfunction!(simhash_q64_native, m)?)?;
    m.add_function(wrap_pyfunction!(top_k_q64_native, m)?)?;
    m.add_function(wrap_pyfunction!(z_order_q64_native, m)?)?;

    // Add version info
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    Ok(())
}