//! this_file: crates/uubed-python/src/tm.rs
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::path::Path;
use uubed_tm::{Pair, Store};

fn error(e: impl std::fmt::Display) -> PyErr {
    PyValueError::new_err(e.to_string())
}

#[pyclass]
pub struct NativeTm {
    store: Option<Store>,
}

impl NativeTm {
    fn read(&self) -> PyResult<&Store> {
        self.store
            .as_ref()
            .ok_or_else(|| error("translation memory is closed"))
    }
    fn write(&mut self) -> PyResult<&mut Store> {
        self.store
            .as_mut()
            .ok_or_else(|| error("translation memory is closed"))
    }
}

#[pymethods]
impl NativeTm {
    #[new]
    fn new(path: &str) -> PyResult<Self> {
        Ok(Self {
            store: Some(Store::open(Path::new(path)).map_err(error)?),
        })
    }

    #[staticmethod]
    fn create(path: &str, config: &str, dimensions: usize) -> PyResult<Self> {
        Ok(Self {
            store: Some(Store::create(Path::new(path), config, dimensions).map_err(error)?),
        })
    }

    fn config(&self) -> PyResult<String> {
        self.read()?.config().map_err(error)
    }
    fn stats(&self) -> PyResult<(usize, usize, usize)> {
        self.read()?.stats().map_err(error)
    }
    fn add_pairs(&mut self, pairs_json: &str) -> PyResult<()> {
        let pairs: Vec<Pair> = serde_json::from_str(pairs_json).map_err(error)?;
        self.write()?.add_pairs(&pairs).map_err(error)
    }
    fn pending(&self, after: i64, limit: usize) -> PyResult<Vec<(i64, String)>> {
        self.read()?.pending(after, limit).map_err(error)
    }
    fn set_vectors(&mut self, rows: Vec<(i64, Vec<u8>, f64)>) -> PyResult<()> {
        self.write()?.set_vectors(&rows).map_err(error)
    }
    fn exact(&self, source: &str, language: &str) -> PyResult<String> {
        serde_json::to_string(&self.read()?.exact(source, language).map_err(error)?).map_err(error)
    }
    fn search(
        &self,
        query: Vec<f32>,
        language: &str,
        limit: usize,
        minimum: f64,
    ) -> PyResult<String> {
        serde_json::to_string(
            &self
                .read()?
                .search(&query, language, limit, minimum)
                .map_err(error)?,
        )
        .map_err(error)
    }
    fn close(&mut self) {
        self.store = None;
    }
}
