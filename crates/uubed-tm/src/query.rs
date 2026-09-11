//! this_file: crates/uubed-tm/src/query.rs
use crate::{Error, Hit, Result, Store};
use rusqlite::params;
use std::collections::BTreeMap;

impl Store {
    fn pairs(&self, id: i64, source: &str, language: &str, score: f64) -> Result<Vec<Hit>> {
        let mut stmt = self.conn.prepare("SELECT target, origin, unit FROM pairs WHERE source_id=?1 AND language=?2 ORDER BY target, origin, unit")?;
        let rows = stmt.query_map(params![id, language], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })?;
        let mut hits: BTreeMap<String, Hit> = BTreeMap::new();
        for row in rows {
            let (target, origin, unit) = row?;
            hits.entry(target.clone())
                .or_insert_with(|| Hit {
                    source: source.into(),
                    target,
                    language: language.into(),
                    score,
                    provenance: Vec::new(),
                })
                .provenance
                .push((origin, unit));
        }
        Ok(hits.into_values().collect())
    }

    pub fn exact(&self, source: &str, language: &str) -> Result<Vec<Hit>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM sources WHERE text=?1 COLLATE BINARY")?;
        let mut rows = stmt.query([source])?;
        match rows.next()? {
            Some(row) => self.pairs(row.get(0)?, source, language, 1.0),
            None => Ok(Vec::new()),
        }
    }

    /// Exhaustive cosine over compact int8 rows, filtered by target availability.
    /// Memory is bounded by top-k candidates; no extension or resident float matrix.
    pub fn search(
        &self,
        query: &[f32],
        language: &str,
        limit: usize,
        minimum: f64,
    ) -> Result<Vec<Hit>> {
        let norm = query
            .iter()
            .map(|v| f64::from(*v).powi(2))
            .sum::<f64>()
            .sqrt();
        if query.len() != self.dimensions || !norm.is_finite() || norm == 0.0 {
            return Err(Error::Invalid(
                "query must have matching dimensions and a finite nonzero norm",
            ));
        }
        if !(1..=100).contains(&limit) || !minimum.is_finite() || !(-1.0..=1.0).contains(&minimum) {
            return Err(Error::Invalid("invalid result limit or minimum cosine"));
        }
        let mut stmt = self.conn.prepare(
            "SELECT id, text, vector, norm FROM sources s WHERE vector IS NOT NULL
            AND EXISTS(SELECT 1 FROM pairs p WHERE p.source_id=s.id AND p.language=?1) ORDER BY id",
        )?;
        let mut rows = stmt.query([language])?;
        let mut best: Vec<(f64, i64, String)> = Vec::new();
        while let Some(row) = rows.next()? {
            let bytes: Vec<u8> = row.get(2)?;
            let stored_norm: f64 = row.get(3)?;
            if bytes.len() != self.dimensions
                || bytes.contains(&128)
                || !stored_norm.is_finite()
                || stored_norm <= 0.0
            {
                return Err(Error::Invalid("corrupt stored vector"));
            }
            let dot: f64 = bytes
                .iter()
                .zip(query)
                .map(|(a, b)| f64::from(*a as i8) * f64::from(*b))
                .sum();
            let score = (dot / (stored_norm * norm)).clamp(-1.0, 1.0);
            if score < minimum
                || (best.len() == limit && best.last().is_some_and(|last| score <= last.0))
            {
                continue;
            }
            best.push((score, row.get(0)?, row.get(1)?));
            best.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
            best.truncate(limit);
        }
        let mut hits = Vec::new();
        for (score, id, source) in best {
            hits.extend(self.pairs(id, &source, language, score)?);
        }
        Ok(hits)
    }
}
