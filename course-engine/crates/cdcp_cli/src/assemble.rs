//! Product assemble entry (`cdcp assemble`).
//!
//! Presentation is [`cdcp_assemble::assemble_with`] → [`cdcp_assemble::assemble_input`].
//! A planted non-letter assess kind is [`cdcp_assemble::AssembleError::NotLetterMcq`]
//! — never four shuffled A–D strings.

use cdcp_assemble::{assemble, assemble_with, AssembleConfig, AssembleInput};
use cdcp_assess::Item;
use cdcp_bank::Bank;
use serde::Deserialize;
use std::fs;
use std::path::Path;

/// One typed row in a `--assess` file: `[{id, module, stem, item}, ...]`.
#[derive(Debug, Deserialize)]
struct AssessRow {
    id: String,
    module: u32,
    stem: String,
    item: Item,
}

pub(crate) fn run(
    bank: &Path,
    seed: u64,
    assess: Option<&Path>,
    out: Option<&Path>,
) -> Result<(), String> {
    let b = Bank::load_dir(bank).map_err(|e| e.to_string())?;
    let cfg = AssembleConfig::default();

    let exam = match assess {
        None => assemble(&b, seed, cfg).map_err(|e| e.to_string())?,
        Some(path) => {
            let rows = load_assess(path)?;
            assemble_owned(&b, seed, cfg, &rows)?
        }
    };

    let mut payload = serde_json::to_string_pretty(&exam).map_err(|e| e.to_string())?;
    payload.push('\n');
    match out {
        Some(path) => {
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
            }
            fs::write(path, payload.as_bytes()).map_err(|e| e.to_string())?;
            println!(
                "assemble: seed={seed} n_items={} -> {}",
                exam.n_items,
                path.display()
            );
        }
        None => print!("{payload}"),
    }
    Ok(())
}

fn assemble_owned(
    bank: &Bank,
    seed: u64,
    cfg: AssembleConfig,
    rows: &[AssessRow],
) -> Result<cdcp_assemble::AssembledExam, String> {
    for row in rows {
        if row.id.trim().is_empty() {
            return Err("assess row missing id".into());
        }
        row.item
            .validate()
            .map_err(|e| format!("item {}: {e}", row.id))?;
    }
    let extra: Vec<AssembleInput<'_>> = rows
        .iter()
        .map(|r| AssembleInput::Assess {
            id: r.id.as_str(),
            module: r.module,
            stem: r.stem.as_str(),
            item: &r.item,
        })
        .collect();
    assemble_with(bank, seed, cfg, &extra).map_err(|e| e.to_string())
}

fn load_assess(path: &Path) -> Result<Vec<AssessRow>, String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let rows: Vec<AssessRow> =
        serde_json::from_str(&raw).map_err(|e| format!("parse {}: {e}", path.display()))?;
    if rows.is_empty() {
        return Err(format!(
            "{}: assess list is empty — an empty input is an ERROR, not an empty exam",
            path.display()
        ));
    }
    Ok(rows)
}
