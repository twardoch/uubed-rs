//! this_file: crates/uubed-python/src/embed.rs
//! Python ownership of a resident FastEmbed session; inference releases the GIL.

use pyo3::{exceptions::PyValueError, prelude::*};
use std::path::Path;

#[pyclass]
pub struct NativeEmbedder {
    inner: Option<uubed_embed::Embedder>,
}

#[pymethods]
impl NativeEmbedder {
    #[new]
    #[pyo3(signature = (alias, directory, model_file, external_file=None, max_tokens=512))]
    fn new(
        py: Python<'_>,
        alias: String,
        directory: String,
        model_file: String,
        external_file: Option<String>,
        max_tokens: usize,
    ) -> PyResult<Self> {
        let model = py
            .allow_threads(move || {
                uubed_embed::Embedder::load(
                    &alias,
                    Path::new(&directory),
                    &model_file,
                    external_file.as_deref(),
                    max_tokens,
                )
            })
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(Self { inner: Some(model) })
    }

    fn embed(
        &mut self,
        py: Python<'_>,
        texts: Vec<String>,
        batch_size: usize,
    ) -> PyResult<Vec<Vec<f32>>> {
        let model = self
            .inner
            .as_mut()
            .ok_or_else(|| PyValueError::new_err("FastEmbed session is closed"))?;
        py.allow_threads(move || model.embed(texts, batch_size))
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    fn close(&mut self) {
        self.inner = None;
    }
}
