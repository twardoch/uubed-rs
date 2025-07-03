// this_file: rust/src/bindings.rs
//! Python bindings for uubed-core using PyO3.

use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use pyo3::buffer::PyBuffer;
use pyo3::types::PyBytes;
use std::collections::HashMap;

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

/// Zero-copy Q64 encoding using PyBuffer (supports numpy arrays, bytearrays)
#[pyfunction]
#[pyo3(signature = (data))]
fn q64_encode_buffer_native(py: Python<'_>, data: PyBuffer<u8>) -> PyResult<Bound<'_, PyBytes>> {
    // Get read-only view of input data
    let input_slice = match data.as_slice(py) {
        Some(slice) => {
            // Convert ReadOnlyCell to regular slice
            let bytes: &[u8] = unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const u8, slice.len()) };
            bytes
        },
        None => return Err(PyValueError::new_err("Failed to access input buffer")),
    };
    
    // Allocate new buffer and encode
    let encoded = crate::encoders::q64_encode(input_slice);
    Ok(PyBytes::new_bound(py, encoded.as_bytes()))
}

/// Batch Q64 encoding for multiple embeddings with buffer pooling
#[pyfunction]
#[pyo3(signature = (embeddings, reuse_buffers=true))]
fn q64_encode_batch_native(
    py: Python<'_>,
    embeddings: Vec<PyBuffer<u8>>,
    reuse_buffers: bool,
) -> PyResult<Vec<Bound<'_, PyBytes>>> {
    let mut results = Vec::with_capacity(embeddings.len());
    let mut buffer_pool: Option<Vec<u8>> = if reuse_buffers { Some(Vec::new()) } else { None };
    
    for data_buffer in embeddings {
        let input_slice = match data_buffer.as_slice(py) {
            Some(slice) => {
                unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const u8, slice.len()) }
            },
            None => return Err(PyValueError::new_err("Failed to access input buffer")),
        };
        
        let required_len = input_slice.len() * 2;
        
        if let Some(ref mut pool_buffer) = buffer_pool {
            // Reuse buffer from pool
            if pool_buffer.len() < required_len {
                pool_buffer.resize(required_len, 0);
            }
            
            crate::encoders::q64_encode_to_buffer(input_slice, &mut pool_buffer[..required_len])
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
                
            results.push(PyBytes::new_bound(py, &pool_buffer[..required_len]));
        } else {
            // Allocate new buffer for each
            let encoded = crate::encoders::q64_encode(input_slice);
            results.push(PyBytes::new_bound(py, encoded.as_bytes()));
        }
    }
    
    Ok(results)
}

/// Memory-efficient streaming Q64 encoder for very large data
#[pyclass]
struct Q64StreamEncoder {
    buffer: Vec<u8>,
    chunk_size: usize,
}

#[pymethods]
impl Q64StreamEncoder {
    #[new]
    #[pyo3(signature = (chunk_size=65536))]
    fn new(chunk_size: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(chunk_size * 2),
            chunk_size,
        }
    }
    
    /// Encode a chunk of data, yielding results as available
    fn encode_chunk<'a>(&mut self, py: Python<'a>, data: PyBuffer<u8>) -> PyResult<Bound<'a, PyBytes>> {
        let input_slice = match data.as_slice(py) {
            Some(slice) => {
                unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const u8, slice.len()) }
            },
            None => return Err(PyValueError::new_err("Failed to access input buffer")),
        };
        
        let required_len = input_slice.len() * 2;
        if self.buffer.len() < required_len {
            self.buffer.resize(required_len, 0);
        }
        
        crate::encoders::q64_encode_to_buffer(input_slice, &mut self.buffer[..required_len])
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
            
        Ok(PyBytes::new_bound(py, &self.buffer[..required_len]))
    }
    
    fn get_chunk_size(&self) -> usize {
        self.chunk_size
    }
}

/// Zero-copy Q64 encoding with caller-provided output buffer (highest performance)
#[pyfunction]
#[pyo3(signature = (input_data, output_buffer))]
fn q64_encode_inplace_native(
    py: Python<'_>,
    input_data: PyBuffer<u8>,
    output_buffer: PyBuffer<u8>
) -> PyResult<usize> {
    // Get read-only view of input data
    let input_slice = match input_data.as_slice(py) {
        Some(slice) => {
            unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const u8, slice.len()) }
        },
        None => return Err(PyValueError::new_err("Failed to access input buffer")),
    };
    
    // Get mutable view of output buffer  
    let output_slice = match output_buffer.as_mut_slice(py) {
        Some(slice) => slice,
        None => return Err(PyValueError::new_err("Failed to access output buffer as mutable")),
    };
    
    // Check if output buffer is large enough (Q64 encoding doubles the size)
    let required_output_size = input_slice.len() * 2;
    if output_slice.len() < required_output_size {
        return Err(PyValueError::new_err(format!(
            "Output buffer too small: need {} bytes, got {}",
            required_output_size,
            output_slice.len()
        )));
    }
    
    // Encode directly into the provided output buffer
    let encoded_str = crate::encoders::q64_encode(input_slice);
    let encoded_bytes = encoded_str.as_bytes();
    
    // Copy encoded data to output buffer using Cell-compatible method
    for (i, &byte) in encoded_bytes.iter().enumerate() {
        output_slice[i].set(byte);
    }
    
    // Return the number of bytes written
    Ok(encoded_bytes.len())
}

/// Performance monitoring and statistics
#[pyclass]
struct Q64Stats {
    total_bytes_encoded: u64,
    total_operations: u64,
    buffer_reuses: u64,
    allocations: u64,
}

#[pymethods]
impl Q64Stats {
    #[new]
    fn new() -> Self {
        Self {
            total_bytes_encoded: 0,
            total_operations: 0,
            buffer_reuses: 0,
            allocations: 0,
        }
    }
    
    fn reset(&mut self) {
        *self = Self::new();
    }
    
    fn get_stats(&self) -> HashMap<String, u64> {
        let mut stats = HashMap::new();
        stats.insert("total_bytes_encoded".to_string(), self.total_bytes_encoded);
        stats.insert("total_operations".to_string(), self.total_operations);
        stats.insert("buffer_reuses".to_string(), self.buffer_reuses);
        stats.insert("allocations".to_string(), self.allocations);
        stats.insert("avg_bytes_per_op".to_string(), 
                    if self.total_operations > 0 { 
                        self.total_bytes_encoded / self.total_operations 
                    } else { 0 });
        stats
    }
}

/// Simplified batch processor for large datasets
#[pyclass]
struct SimpleBatchProcessor {
    chunk_size: usize,
}

#[pymethods]
impl SimpleBatchProcessor {
    #[new]
    #[pyo3(signature = (chunk_size=10000))]
    fn new(chunk_size: usize) -> Self {
        Self { chunk_size }
    }
    
    /// Process large batch with chunking to manage memory
    fn process_batch<'a>(
        &self,
        py: Python<'a>,
        embeddings: Vec<PyBuffer<u8>>,
    ) -> PyResult<Vec<Bound<'a, PyBytes>>> {
        let mut results = Vec::with_capacity(embeddings.len());
        
        // Process in chunks to manage memory
        for chunk in embeddings.chunks(self.chunk_size) {
            for data_buffer in chunk {
                let input_slice = match data_buffer.as_slice(py) {
                    Some(slice) => {
                        unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const u8, slice.len()) }
                    },
                    None => return Err(PyValueError::new_err("Failed to access input buffer")),
                };
                
                let encoded = crate::encoders::q64_encode(input_slice);
                results.push(PyBytes::new_bound(py, encoded.as_bytes()));
            }
            
            // Allow Python to handle interrupts
            py.check_signals()?;
        }
        
        Ok(results)
    }
    
    fn get_chunk_size(&self) -> usize {
        self.chunk_size
    }
}

/// Memory pool for efficient buffer reuse
#[pyclass]
struct BufferPool {
    pools: HashMap<usize, Vec<Vec<u8>>>,
    max_pool_size: usize,
    allocations: u64,
    reuses: u64,
}

#[pymethods]
impl BufferPool {
    #[new]
    #[pyo3(signature = (max_pool_size=100))]
    fn new(max_pool_size: usize) -> Self {
        Self {
            pools: HashMap::new(),
            max_pool_size,
            allocations: 0,
            reuses: 0,
        }
    }
    
    fn get_buffer(&mut self, size: usize) -> Vec<u8> {
        if let Some(pool) = self.pools.get_mut(&size) {
            if let Some(mut buffer) = pool.pop() {
                buffer.clear();
                buffer.resize(size, 0);
                self.reuses += 1;
                return buffer;
            }
        }
        
        self.allocations += 1;
        vec![0u8; size]
    }
    
    fn return_buffer(&mut self, buffer: Vec<u8>) {
        let size = buffer.capacity();
        let pool = self.pools.entry(size).or_insert_with(Vec::new);
        
        if pool.len() < self.max_pool_size {
            pool.push(buffer);
        }
    }
    
    fn get_stats(&self) -> HashMap<String, u64> {
        let mut stats = HashMap::new();
        stats.insert("allocations".to_string(), self.allocations);
        stats.insert("reuses".to_string(), self.reuses);
        stats.insert("pool_count".to_string(), self.pools.len() as u64);
        
        let total_pooled: usize = self.pools.values().map(|v| v.len()).sum();
        stats.insert("total_pooled_buffers".to_string(), total_pooled as u64);
        
        stats
    }
    
    fn clear_pools(&mut self) {
        self.pools.clear();
    }
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

/// Generate top-k indices with Q64 encoding (optimized version)
#[pyfunction]
#[pyo3(signature = (embedding, k=8))]
fn top_k_q64_optimized_native(embedding: &[u8], k: usize) -> String {
    crate::encoders::top_k_q64_optimized(embedding, k)
}

#[pyfunction]
#[pyo3(signature = (embedding))]
fn z_order_q64_native(embedding: &[u8]) -> String {
    crate::encoders::z_order_q64(embedding)
}
/// Encode data using Mq64 (hierarchical QuadB64) encoding
#[pyfunction]
#[pyo3(signature = (data, levels=None))]
fn mq64_encode_native(data: &[u8], levels: Option<Vec<usize>>) -> PyResult<String> {
    let s = match levels {
        Some(lvls) => crate::encoders::mq64_encode_with_levels(data, &lvls),
        None => crate::encoders::mq64_encode(data),
    };
    Ok(s)
}

/// Decode Mq64 string to full data bytes
#[pyfunction]
#[pyo3(signature = (encoded))]
fn mq64_decode_native(encoded: &str) -> PyResult<Vec<u8>> {
    crate::encoders::mq64_decode(encoded)
        .map_err(|e| PyValueError::new_err(e.to_string()))
}

/// Python module initialization
#[pymodule]
fn uubed_native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Basic encoding functions
    m.add_function(wrap_pyfunction!(q64_encode_native, m)?)?;
    m.add_function(wrap_pyfunction!(q64_decode_native, m)?)?;
    
    // Advanced PyO3 optimized functions
    m.add_function(wrap_pyfunction!(q64_encode_buffer_native, m)?)?;
    m.add_function(wrap_pyfunction!(q64_encode_batch_native, m)?)?;
    m.add_function(wrap_pyfunction!(q64_encode_inplace_native, m)?)?;
    
    // Other encoder functions
    m.add_function(wrap_pyfunction!(simhash_q64_native, m)?)?;
    m.add_function(wrap_pyfunction!(top_k_q64_native, m)?)?;
    m.add_function(wrap_pyfunction!(top_k_q64_optimized_native, m)?)?;
    m.add_function(wrap_pyfunction!(z_order_q64_native, m)?)?;
    // Mq64 hierarchical encoding
    m.add_function(wrap_pyfunction!(mq64_encode_native, m)?)?;
    m.add_function(wrap_pyfunction!(mq64_decode_native, m)?)?;

    // Advanced PyO3 classes
    m.add_class::<Q64StreamEncoder>()?;
    m.add_class::<Q64Stats>()?;
    m.add_class::<SimpleBatchProcessor>()?;
    m.add_class::<BufferPool>()?;

    // Add version info
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    Ok(())
}