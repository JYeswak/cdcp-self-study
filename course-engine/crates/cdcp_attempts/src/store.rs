//! Local-first attempt log: SQLite + append-only JSONL.
//!
//! Export is a separate operation and requires [`ExportPolicy::opt_in`].
//! The local JSONL file is not an export.

use crate::error::AttemptError;
use crate::event::{AttemptEvent, AttemptMode, SCHEMA_VERSION};
use crate::{minimum_n_warning, ExportPolicy, ExportReceipt};
use rusqlite::{params, Connection};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

/// SQLite file name inside the store directory.
pub const SQLITE_NAME: &str = "attempts.sqlite";
/// Append-only JSONL mirror inside the store directory.
pub const JSONL_NAME: &str = "attempts.jsonl";

const SCHEMA_SQL: &str = "
CREATE TABLE IF NOT EXISTS attempt_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    schema_version INTEGER NOT NULL,
    item_version TEXT NOT NULL,
    bank_hash TEXT NOT NULL,
    learner_pseudonym TEXT NOT NULL,
    mode TEXT NOT NULL,
    exposure_count INTEGER NOT NULL,
    chosen_option TEXT NOT NULL,
    correctness INTEGER NOT NULL,
    latency_ms INTEGER NOT NULL,
    timestamp_unix_ms INTEGER NOT NULL,
    prior_attempts INTEGER NOT NULL
);
";

/// On-disk attempt log. Creates the directory and both files on open.
pub struct AttemptLog {
    dir: PathBuf,
    conn: Connection,
    jsonl_path: PathBuf,
}

impl AttemptLog {
    /// Open (or create) a store at `dir`. A newly created store has zero
    /// events; [`events`](Self::events) and [`export_jsonl`](Self::export_jsonl)
    /// then return [`AttemptError::EmptyStore`].
    pub fn open(dir: impl AsRef<Path>) -> Result<Self, AttemptError> {
        let dir = dir.as_ref().to_path_buf();
        fs::create_dir_all(&dir)?;
        let sqlite_path = dir.join(SQLITE_NAME);
        let jsonl_path = dir.join(JSONL_NAME);
        let conn = Connection::open(&sqlite_path)?;
        conn.execute_batch(SCHEMA_SQL)?;
        if !jsonl_path.exists() {
            fs::File::create(&jsonl_path)?;
        }
        Ok(Self {
            dir,
            conn,
            jsonl_path,
        })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Number of events in SQLite. Zero is a valid count (introspection);
    /// it is not a successful analysis.
    pub fn count(&self) -> Result<u64, AttemptError> {
        let n: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM attempt_events", [], |row| row.get(0))?;
        u64::try_from(n).map_err(|e| AttemptError::StoredInteger(e.to_string()))
    }

    /// Append one event to SQLite and the JSONL mirror in the same step.
    pub fn record(&mut self, event: &AttemptEvent) -> Result<(), AttemptError> {
        event.validate()?;
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO attempt_events (
                schema_version, item_version, bank_hash, learner_pseudonym, mode,
                exposure_count, chosen_option, correctness, latency_ms,
                timestamp_unix_ms, prior_attempts
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                i64::from(event.schema_version),
                event.item_version.as_str(),
                event.bank_hash.as_str(),
                event.learner_pseudonym.as_str(),
                event.mode.as_str(),
                i64::from(event.exposure_count),
                event.chosen_option.as_str(),
                i64::from(event.correctness),
                i64_from_u64(event.latency_ms, "latency_ms")?,
                i64_from_u64(event.timestamp_unix_ms, "timestamp_unix_ms")?,
                i64::from(event.prior_attempts),
            ],
        )?;
        {
            let mut f = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.jsonl_path)?;
            serde_json::to_writer(&mut f, event)?;
            f.write_all(b"\n")?;
            f.flush()?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Load every event. An empty store is an ERROR, not an empty `Vec`.
    pub fn events(&self) -> Result<Vec<AttemptEvent>, AttemptError> {
        let sqlite_n = self.count()?;
        let jsonl_n = self.jsonl_line_count()?;
        if sqlite_n != jsonl_n {
            return Err(AttemptError::LogDiverged {
                sqlite: sqlite_n,
                jsonl: jsonl_n,
            });
        }
        if sqlite_n == 0 {
            return Err(AttemptError::EmptyStore);
        }
        let mut stmt = self.conn.prepare(
            "SELECT schema_version, item_version, bank_hash, learner_pseudonym, mode,
                    exposure_count, chosen_option, correctness, latency_ms,
                    timestamp_unix_ms, prior_attempts
             FROM attempt_events
             ORDER BY id ASC",
        )?;
        let mut rows = stmt.query([])?;
        let mut out = Vec::with_capacity(sqlite_n as usize);
        while let Some(row) = rows.next()? {
            out.push(row_to_event(row)?);
        }
        Ok(out)
    }

    /// Write events as JSONL to `w`. Requires an explicit opt-in. Empty
    /// store is an ERROR even when opted in.
    pub fn export_jsonl<W: Write>(
        &self,
        policy: &ExportPolicy,
        mut w: W,
    ) -> Result<ExportReceipt, AttemptError> {
        if !policy.is_opted_in() {
            return Err(AttemptError::ExportNotOptedIn);
        }
        let events = self.events()?;
        for event in &events {
            serde_json::to_writer(&mut w, event)?;
            w.write_all(b"\n")?;
        }
        let event_count = events.len() as u64;
        Ok(ExportReceipt {
            event_count,
            minimum_n: minimum_n_warning(event_count),
        })
    }

    fn jsonl_line_count(&self) -> Result<u64, AttemptError> {
        let f = fs::File::open(&self.jsonl_path)?;
        let reader = BufReader::new(f);
        let mut n = 0u64;
        for line in reader.lines() {
            let line = line?;
            if !line.is_empty() {
                n += 1;
            }
        }
        Ok(n)
    }
}

fn i64_from_u64(n: u64, field: &str) -> Result<i64, AttemptError> {
    i64::try_from(n).map_err(|_| AttemptError::StoredInteger(format!("{field} exceeds i64")))
}

fn u64_from_i64(n: i64, field: &str) -> Result<u64, AttemptError> {
    u64::try_from(n).map_err(|_| AttemptError::StoredInteger(format!("{field} is negative")))
}

fn u32_from_i64(n: i64, field: &str) -> Result<u32, AttemptError> {
    u32::try_from(n).map_err(|_| AttemptError::StoredInteger(format!("{field} out of u32")))
}

fn row_to_event(row: &rusqlite::Row<'_>) -> Result<AttemptEvent, AttemptError> {
    let schema_version = u32_from_i64(row.get(0)?, "schema_version")?;
    if schema_version != SCHEMA_VERSION {
        return Err(AttemptError::StoredInteger(format!(
            "unsupported schema_version {schema_version}"
        )));
    }
    let correctness = match row.get::<_, i64>(7)? {
        0 => false,
        1 => true,
        other => {
            return Err(AttemptError::StoredInteger(format!(
                "correctness must be 0 or 1, got {other}"
            )))
        }
    };
    let event = AttemptEvent {
        schema_version,
        item_version: row.get(1)?,
        bank_hash: row.get(2)?,
        learner_pseudonym: row.get(3)?,
        mode: AttemptMode::parse(&row.get::<_, String>(4)?)?,
        exposure_count: u32_from_i64(row.get(5)?, "exposure_count")?,
        chosen_option: row.get(6)?,
        correctness,
        latency_ms: u64_from_i64(row.get(8)?, "latency_ms")?,
        timestamp_unix_ms: u64_from_i64(row.get(9)?, "timestamp_unix_ms")?,
        prior_attempts: u32_from_i64(row.get(10)?, "prior_attempts")?,
    };
    event.validate()?;
    Ok(event)
}
