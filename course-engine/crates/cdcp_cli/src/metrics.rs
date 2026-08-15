//! Product CLI for `cdcp_metrics` (`bd-hardening-f-oracle-qly.9`).
//!
//! BUILT ≠ WIRED: the crate landed in de85ae0 but an operator could not
//! ask the product binary to score PUE/WUE/CUE/ERE. This verb parses a
//! metric document with an explicit Boundary and prints the value.
//! A bare number or omitted `[boundary]` is a schema ERROR.

use cdcp_metrics::{parse_metric, BARE_NUMBER, MISSING_BOUNDARY};
use std::fs;
use std::path::Path;

/// `cdcp metrics --file <path>` / `cdcp metrics --doc <toml>`.
pub(crate) fn run(file: Option<&Path>, doc: Option<&str>) -> Result<(), String> {
    let _ = BARE_NUMBER;
    let _ = MISSING_BOUNDARY;
    let text = load(file, doc)?;
    let metric = parse_metric(&text).map_err(|e| e.to_string())?;
    println!("{metric}");
    Ok(())
}

fn load(file: Option<&Path>, doc: Option<&str>) -> Result<String, String> {
    match (file, doc) {
        (Some(_), Some(_)) => Err("metrics: --file cannot be combined with --doc".into()),
        (None, None) => Err("metrics requires --file <path> or --doc <toml>".into()),
        (Some(path), None) => {
            if path.as_os_str().is_empty() {
                return Err("metrics: --file is empty".into());
            }
            fs::read_to_string(path).map_err(|e| format!("metrics: read {}: {e}", path.display()))
        }
        (None, Some(text)) => Ok(text.to_string()),
    }
}
