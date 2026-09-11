//! this_file: crates/uubed-tm/src/lib.rs
//! Portable translation pairs and compact English embeddings in ordinary SQLite.
mod query;
mod storage;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
pub use storage::Store;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Invalid(&'static str),
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Pair {
    pub source: String,
    pub target: String,
    pub language: String,
    pub origin: String,
    pub unit: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Hit {
    pub source: String,
    pub target: String,
    pub language: String,
    pub score: f64,
    pub provenance: Vec<(String, String)>,
}

fn dimensions(conn: &Connection) -> Result<usize> {
    let version: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if version != 1 {
        return Err(Error::Invalid("unsupported translation-memory schema"));
    }
    let n: i64 = conn.query_row("SELECT dimensions FROM metadata", [], |r| r.get(0))?;
    if !(1..=65536).contains(&n) {
        return Err(Error::Invalid("invalid index dimensions"));
    }
    Ok(n as usize)
}
