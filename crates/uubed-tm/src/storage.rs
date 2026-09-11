//! this_file: crates/uubed-tm/src/storage.rs
use crate::{Error, Pair, Result, dimensions};
use rusqlite::{Connection, OpenFlags, params};
use std::path::Path;

pub struct Store {
    pub(crate) conn: Connection,
    pub(crate) dimensions: usize,
}

impl Store {
    pub fn create(path: &Path, config: &str, width: usize) -> Result<Self> {
        if path.exists() {
            return Err(Error::Invalid("index already exists"));
        }
        if !(1..=65536).contains(&width) {
            return Err(Error::Invalid("invalid index dimensions"));
        }
        // Reserve the filename atomically, including concurrent native callers.
        std::fs::File::create_new(path)?;
        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_WRITE)?;
        conn.execute_batch(
            "PRAGMA foreign_keys=ON;
            PRAGMA user_version=1;
            CREATE TABLE metadata(config TEXT NOT NULL, dimensions INTEGER NOT NULL);
            CREATE TABLE sources(id INTEGER PRIMARY KEY, text TEXT NOT NULL UNIQUE COLLATE BINARY,
                vector BLOB, scale REAL, norm REAL);
            CREATE TABLE pairs(source_id INTEGER NOT NULL REFERENCES sources(id),
                language TEXT NOT NULL, target TEXT NOT NULL COLLATE BINARY,
                origin TEXT NOT NULL, unit TEXT NOT NULL,
                UNIQUE(source_id, language, target, origin, unit));
            CREATE INDEX target_pairs ON pairs(language, source_id);",
        )?;
        conn.execute(
            "INSERT INTO metadata VALUES(?1, ?2)",
            params![config, width],
        )?;
        Ok(Self {
            conn,
            dimensions: width,
        })
    }

    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        let dimensions = dimensions(&conn)?;
        Ok(Self { conn, dimensions })
    }

    pub fn config(&self) -> Result<String> {
        Ok(self
            .conn
            .query_row("SELECT config FROM metadata", [], |r| r.get(0))?)
    }

    pub fn add_pairs(&mut self, pairs: &[Pair]) -> Result<()> {
        let tx = self.conn.transaction()?;
        for p in pairs {
            if p.source.trim().is_empty() || p.target.trim().is_empty() || p.language.is_empty() {
                return Err(Error::Invalid(
                    "translation pairs must have source, target and language",
                ));
            }
            tx.execute(
                "INSERT OR IGNORE INTO sources(text) VALUES(?1)",
                [&p.source],
            )?;
            tx.execute(
                "INSERT OR IGNORE INTO pairs SELECT id, ?2, ?3, ?4, ?5 FROM sources WHERE text=?1",
                params![p.source, p.language, p.target, p.origin, p.unit],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn pending(&self, after: i64, limit: usize) -> Result<Vec<(i64, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, text FROM sources WHERE vector IS NULL AND id>?1 ORDER BY id LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![after, limit], |r| Ok((r.get(0)?, r.get(1)?)))?;
        Ok(rows.collect::<std::result::Result<_, _>>()?)
    }

    pub fn set_vectors(&mut self, vectors: &[(i64, Vec<u8>, f64)]) -> Result<()> {
        let tx = self.conn.transaction()?;
        for (id, payload, scale) in vectors {
            let norm = payload
                .iter()
                .map(|v| f64::from(*v as i8).powi(2))
                .sum::<f64>()
                .sqrt();
            if payload.len() != self.dimensions
                || payload.contains(&128)
                || norm == 0.0
                || !scale.is_finite()
                || *scale <= 0.0
            {
                return Err(Error::Invalid("invalid compact int8 vector"));
            }
            let changed = tx.execute(
                "UPDATE sources SET vector=?2, scale=?3, norm=?4 WHERE id=?1",
                params![id, payload, scale, norm],
            )?;
            if changed != 1 {
                return Err(Error::Invalid("unknown embedding source id"));
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn stats(&self) -> Result<(usize, usize, usize)> {
        Ok(self.conn.query_row(
            "SELECT (SELECT count(*) FROM sources),
            (SELECT count(*) FROM pairs), (SELECT count(*) FROM sources WHERE vector IS NULL)",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?)
    }
}
