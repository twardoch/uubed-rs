//! this_file: crates/uubed-embed/src/lib.rs
//! Resident CPU inference using FastEmbed's ONNX and tokenizer implementation.
//! Callers supply a pinned model snapshot; this crate never downloads weights.

use fastembed::{
    InitOptionsUserDefined, OutputKey, Pooling, TextEmbedding, TokenizerFiles,
    UserDefinedEmbeddingModel,
};
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unsupported FastEmbed model: {0}")]
    Unsupported(String),
    #[error("{0}")]
    Invalid(&'static str),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Inference(#[from] fastembed::Error),
}

fn contract(model: &str) -> Result<(usize, Pooling, OutputKey), Error> {
    match model {
        "minilm" => Ok((384, Pooling::Mean, OutputKey::ByName("last_hidden_state"))),
        "gemma" => Ok((768, Pooling::Mean, OutputKey::ByName("sentence_embedding"))),
        "granite" => Ok((768, Pooling::Cls, OutputKey::ByName("last_hidden_state"))),
        "jina" => Ok((768, Pooling::Mean, OutputKey::ByName("sentence_embedding"))),
        "nomic" => Ok((768, Pooling::Mean, OutputKey::ByName("sentence_embedding"))),
        "voyage-nano" => Ok((2048, Pooling::Mean, OutputKey::ByName("pooler_output"))),
        _ => Err(Error::Unsupported(model.to_owned())),
    }
}

pub struct Embedder {
    model: TextEmbedding,
    dimensions: usize,
}

impl Embedder {
    pub fn load(
        alias: &str,
        directory: &Path,
        model_file: &str,
        external_file: Option<&str>,
        max_tokens: usize,
    ) -> Result<Self, Error> {
        let (dimensions, pooling, output) = contract(alias)?;
        if max_tokens < 16 {
            return Err(Error::Invalid("max_tokens must be at least 16"));
        }
        let tokenizer_files = TokenizerFiles {
            tokenizer_file: std::fs::read(directory.join("tokenizer.json"))?,
            config_file: std::fs::read(directory.join("config.json"))?,
            tokenizer_config_file: std::fs::read(directory.join("tokenizer_config.json"))?,
            // Tokenizer JSON already contains special tokens. Some vendors omit this optional map.
            special_tokens_map_file: match std::fs::read(directory.join("special_tokens_map.json"))
            {
                Ok(bytes) => bytes,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => b"{}".to_vec(),
                Err(e) => return Err(e.into()),
            },
        };
        let mut model = UserDefinedEmbeddingModel::new(
            std::fs::read(directory.join(model_file))?,
            tokenizer_files,
        )
        .with_pooling(pooling);
        model.output_key = Some(output);
        if let Some(file) = external_file {
            let name = Path::new(file)
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or(Error::Invalid("invalid external model filename"))?;
            model = model
                .with_external_initializer(name.to_owned(), std::fs::read(directory.join(file))?);
        }
        let model = TextEmbedding::try_new_from_user_defined(
            model,
            InitOptionsUserDefined::new()
                .with_max_length(max_tokens)
                .with_intra_threads(4),
        )?;
        Ok(Self { model, dimensions })
    }

    pub fn embed(&mut self, texts: Vec<String>, batch_size: usize) -> Result<Vec<Vec<f32>>, Error> {
        if batch_size == 0 || texts.iter().any(|t| t.trim().is_empty()) {
            return Err(Error::Invalid(
                "provide nonempty texts and a positive batch_size",
            ));
        }
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        let count = texts.len();
        let rows = self.model.embed(texts, Some(batch_size))?;
        if rows.len() != count
            || rows.iter().any(|v| {
                v.len() != self.dimensions
                    || v.iter().any(|x| !x.is_finite())
                    || v.iter().all(|x| *x == 0.0)
            })
        {
            return Err(Error::Invalid("FastEmbed returned invalid vectors"));
        }
        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn contracts_reject_substitution_and_loading_requires_files() {
        for (alias, dims) in [
            ("minilm", 384),
            ("gemma", 768),
            ("granite", 768),
            ("jina", 768),
            ("nomic", 768),
            ("voyage-nano", 2048),
        ] {
            assert_eq!(contract(alias).unwrap().0, dims);
            assert!(Embedder::load(
                alias,
                Path::new("/nonexistent-uubed-model"),
                "model.onnx",
                None,
                512
            )
            .is_err());
        }
        for alias in ["voyage-lite", "unknown"] {
            assert!(matches!(contract(alias), Err(Error::Unsupported(_))));
        }
    }

    #[test]
    fn invalid_token_budget_fails_before_reading_weights() {
        for max_tokens in [0, 1, 15] {
            assert!(matches!(
                Embedder::load(
                    "gemma",
                    Path::new("/nonexistent-uubed-model"),
                    "model.onnx",
                    None,
                    max_tokens
                ),
                Err(Error::Invalid(_))
            ));
        }
    }
}
