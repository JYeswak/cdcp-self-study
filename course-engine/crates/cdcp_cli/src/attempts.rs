//! Product CLI for `cdcp_attempts` (`bd-hardening-l-attempts-bg2.2`).
//!
//! BUILT ≠ WIRED: the crate landed in c51b3ba but no product binary called
//! it. This verb is a thin wrapper: record / list / export. Export stays
//! OFF unless `--opt-in` is passed. An empty store is the crate's schema
//! ERROR, never `[]`. There is no IRT, difficulty, or discrimination
//! command — those functions stay uncallable from this surface.

use cdcp_attempts::{
    AttemptEvent, AttemptLog, AttemptMode, ExportPolicy, EMPTY_STORE, EXPORT_NOT_OPTED_IN,
    JSONL_NAME, SQLITE_NAME,
};
use clap::Subcommand;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Conventional probe path under the engine root. Doctor/health mention
/// this location. Record still requires an explicit `--store` — this path
/// is not a silent default write target.
pub(crate) const STORE_REL: &str = "var/attempts";

/// `cdcp attempts <record|list|export>`.
#[derive(Subcommand)]
pub(crate) enum AttemptsCmd {
    /// Append one event. Requires an explicit `--store` (opt-in path).
    Record {
        /// Directory for the local sqlite + jsonl log. Created if missing.
        #[arg(long)]
        store: PathBuf,
        #[arg(long)]
        item_version: String,
        #[arg(long)]
        bank_hash: String,
        #[arg(long)]
        learner_pseudonym: String,
        /// `learn`, `quiz`, `drill`, or `mock`.
        #[arg(long)]
        mode: String,
        #[arg(long)]
        exposure_count: u32,
        #[arg(long)]
        chosen_option: String,
        /// Explicit `true` or `false`. Omission is a clap ERROR, not false.
        #[arg(long, value_parser = ["true", "false"])]
        correctness: String,
        #[arg(long)]
        latency_ms: u64,
        /// Required. `0` is the crate's unset-clock ERROR.
        #[arg(long)]
        timestamp_unix_ms: u64,
        #[arg(long)]
        prior_attempts: u32,
    },
    /// Print stored events as JSONL. Empty store is an ERROR, not `[]`.
    List {
        #[arg(long)]
        store: PathBuf,
    },
    /// Write JSONL to stdout or `--out`. Requires `--opt-in`. Default OFF.
    Export {
        #[arg(long)]
        store: PathBuf,
        /// Explicit export opt-in. Absent = policy OFF = non-zero.
        #[arg(long)]
        opt_in: bool,
        /// Destination file. Default: stdout.
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

/// Status doctor/health print. Absence is the default, not a defect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StoreMention {
    pub state: &'static str,
    pub rel_path: &'static str,
    pub n: u64,
    pub export_policy: &'static str,
}

pub(crate) fn run(cmd: AttemptsCmd) -> Result<(), String> {
    // Touch the tokens so deleting them from this wrapper is a compile error
    // only after the known-bad CLI test is also deleted — keep them load-bearing.
    let _ = EMPTY_STORE;
    let _ = EXPORT_NOT_OPTED_IN;
    match cmd {
        AttemptsCmd::Record {
            store,
            item_version,
            bank_hash,
            learner_pseudonym,
            mode,
            exposure_count,
            chosen_option,
            correctness,
            latency_ms,
            timestamp_unix_ms,
            prior_attempts,
        } => record(
            &store,
            &item_version,
            &bank_hash,
            &learner_pseudonym,
            &mode,
            exposure_count,
            &chosen_option,
            correctness == "true",
            latency_ms,
            timestamp_unix_ms,
            prior_attempts,
        ),
        AttemptsCmd::List { store } => list(&store),
        AttemptsCmd::Export { store, opt_in, out } => export(&store, opt_in, out.as_deref()),
    }
}

#[allow(clippy::too_many_arguments)]
fn record(
    store: &Path,
    item_version: &str,
    bank_hash: &str,
    learner_pseudonym: &str,
    mode: &str,
    exposure_count: u32,
    chosen_option: &str,
    correctness: bool,
    latency_ms: u64,
    timestamp_unix_ms: u64,
    prior_attempts: u32,
) -> Result<(), String> {
    require_store_flag(store)?;
    let mode = AttemptMode::parse(mode).map_err(|e| e.to_string())?;
    let event = AttemptEvent::new(
        item_version,
        bank_hash,
        learner_pseudonym,
        mode,
        exposure_count,
        chosen_option,
        correctness,
        latency_ms,
        timestamp_unix_ms,
        prior_attempts,
    )
    .map_err(|e| e.to_string())?;
    let mut log = AttemptLog::open(store).map_err(|e| e.to_string())?;
    log.record(&event).map_err(|e| e.to_string())?;
    println!(
        "attempts: recorded item_version={} mode={} store={}",
        event.item_version,
        event.mode.as_str(),
        store.display()
    );
    Ok(())
}

fn list(store: &Path) -> Result<(), String> {
    let log = open_for_read(store)?;
    let events = log.events().map_err(|e| e.to_string())?;
    let mut out = io::stdout().lock();
    for event in &events {
        serde_json::to_writer(&mut out, event).map_err(|e| e.to_string())?;
        out.write_all(b"\n").map_err(|e| e.to_string())?;
    }
    out.flush().map_err(|e| e.to_string())?;
    Ok(())
}

fn export(store: &Path, opt_in: bool, dest: Option<&Path>) -> Result<(), String> {
    let log = open_for_read(store)?;
    let policy = if opt_in {
        ExportPolicy::opt_in()
    } else {
        ExportPolicy::off()
    };
    match dest {
        None => {
            let mut stdout = io::stdout().lock();
            let receipt = log
                .export_jsonl(&policy, &mut stdout)
                .map_err(|e| e.to_string())?;
            stdout.flush().map_err(|e| e.to_string())?;
            emit_receipt(&receipt);
            Ok(())
        }
        Some(path) => {
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("attempts: mkdir {}: {e}", parent.display()))?;
                }
            }
            let mut file = fs::File::create(path)
                .map_err(|e| format!("attempts: write {}: {e}", path.display()))?;
            let receipt = log
                .export_jsonl(&policy, &mut file)
                .map_err(|e| e.to_string())?;
            file.flush().map_err(|e| e.to_string())?;
            emit_receipt(&receipt);
            eprintln!("attempts: export -> {}", path.display());
            Ok(())
        }
    }
}

fn emit_receipt(receipt: &cdcp_attempts::ExportReceipt) {
    eprint!("attempts: export event_count={}", receipt.event_count);
    if let Some(w) = &receipt.minimum_n {
        eprint!(
            " minimum_n=warning observed_n={} required_n={}",
            w.observed_n, w.required_n
        );
    }
    eprintln!();
}

fn require_store_flag(store: &Path) -> Result<(), String> {
    if store.as_os_str().is_empty() {
        return Err("attempts: --store is empty (store path is opt-in)".into());
    }
    Ok(())
}

/// List/export must not create an empty store as a side effect of a read.
fn open_for_read(store: &Path) -> Result<AttemptLog, String> {
    require_store_flag(store)?;
    if !store.exists() {
        return Err(cdcp_attempts::AttemptError::EmptyStore.to_string());
    }
    AttemptLog::open(store).map_err(|e| e.to_string())
}

pub(crate) fn mention(root: &Path) -> StoreMention {
    let dir = root.join(STORE_REL);
    let sqlite = dir.join(SQLITE_NAME);
    let jsonl = dir.join(JSONL_NAME);
    if !dir.is_dir() || (!sqlite.is_file() && !jsonl.is_file()) {
        return StoreMention {
            state: "absent",
            rel_path: STORE_REL,
            n: 0,
            export_policy: "off",
        };
    }
    match AttemptLog::open(&dir) {
        Ok(log) => match log.events() {
            Ok(events) => StoreMention {
                state: "ready",
                rel_path: STORE_REL,
                n: events.len() as u64,
                export_policy: "off",
            },
            Err(cdcp_attempts::AttemptError::EmptyStore) => StoreMention {
                state: "empty",
                rel_path: STORE_REL,
                n: 0,
                export_policy: "off",
            },
            Err(cdcp_attempts::AttemptError::LogDiverged { .. }) => StoreMention {
                state: "diverged",
                rel_path: STORE_REL,
                n: 0,
                export_policy: "off",
            },
            Err(_) => StoreMention {
                state: "unreadable",
                rel_path: STORE_REL,
                n: 0,
                export_policy: "off",
            },
        },
        Err(_) => StoreMention {
            state: "unreadable",
            rel_path: STORE_REL,
            n: 0,
            export_policy: "off",
        },
    }
}

pub(crate) fn doctor_line(root: &Path) -> String {
    let m = mention(root);
    format!(
        "attempts-store: state={} path={} n={} export={}",
        m.state, m.rel_path, m.n, m.export_policy
    )
}
