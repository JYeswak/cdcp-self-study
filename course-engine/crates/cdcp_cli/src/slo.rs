//! `cdcp slo` — wall budgets and epoch-ms for `scripts/smoke_slo.sh`.
//!
//! EXTRACT-THEN-DELETE (`bd-extract-smoke-slo-python-l5ke`). The smoke
//! driver used to parse slo.toml and read the wall clock from an
//! interpreter one-liner. This module is a typed rewrite, not a
//! line-for-line copy:
//!
//! - `[budgets]` is required. Root-level keys are not a fallback.
//! - each wall is a non-negative integer. A float or a negative is RED.
//! - an empty file or an empty `[budgets]` table is RED (a document
//!   that names no walls certifies nothing).
//!
//! stdout of `slo budgets` is exactly three lines of integers, in
//! [`REQUIRED_BUDGET_KEYS`] order. stdout of `slo now-ms` is one
//! integer. The shell times product walls against those numbers.

use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Error token when the document has no `[budgets]` table.
pub(crate) const MISSING_BUDGETS: &str = "missing [budgets]";
/// Error token for a 0-byte / whitespace-only document.
pub(crate) const EMPTY_DOCUMENT: &str = "empty document";
/// Error token for `[budgets]` with zero rows.
pub(crate) const EMPTY_BUDGETS: &str = "empty [budgets]";
/// Error token prefix when a required wall key is absent.
pub(crate) const MISSING_KEYS: &str = "missing budget key";

/// Walls `smoke_slo.sh` times, in print order.
///
/// Emptying this list is a RED run, not a silently vacuous parse.
pub(crate) const REQUIRED_BUDGET_KEYS: &[&str] = &["grade_ms", "export_ms", "bank_verify_ms"];

/// `cdcp slo budgets --file <path>`.
pub(crate) fn emit_budgets(path: &Path) -> Result<(), String> {
    if path.as_os_str().is_empty() {
        return Err("slo budgets: --file is empty".into());
    }
    let raw = fs::read_to_string(path)
        .map_err(|e| format!("slo budgets: read {}: {e}", path.display()))?;
    let [grade, export, verify] = budgets_from_text(&raw)?;
    println!("{grade}");
    println!("{export}");
    println!("{verify}");
    Ok(())
}

/// `cdcp slo now-ms`.
pub(crate) fn emit_now_ms() -> Result<(), String> {
    println!("{}", epoch_ms()?);
    Ok(())
}

/// Parse the three wall ceilings from a slo.toml body.
pub(crate) fn budgets_from_text(text: &str) -> Result<[u64; 3], String> {
    if REQUIRED_BUDGET_KEYS.is_empty() {
        return Err(
            "REQUIRED_BUDGET_KEYS is empty — a parse that requires no walls certifies nothing"
                .into(),
        );
    }
    if text.trim().is_empty() {
        return Err(format!(
            "{EMPTY_DOCUMENT} — a 0-byte slo file pins no budget"
        ));
    }
    let doc: toml::Value =
        toml::from_str(text).map_err(|e| format!("slo.toml is not TOML: {e}"))?;
    let Some(toml::Value::Table(table)) = doc.get("budgets") else {
        return Err(format!(
            "{MISSING_BUDGETS} table — root-level keys are not a fallback"
        ));
    };
    if table.is_empty() {
        return Err(format!(
            "{EMPTY_BUDGETS} table — a table that names no walls certifies nothing"
        ));
    }
    let mut missing = Vec::new();
    let mut values = [0u64; 3];
    for (i, key) in REQUIRED_BUDGET_KEYS.iter().enumerate() {
        match table.get(*key) {
            None => missing.push(*key),
            Some(v) => values[i] = wall_ms(key, v)?,
        }
    }
    if !missing.is_empty() {
        return Err(format!("{MISSING_KEYS}: {}", missing.join(", ")));
    }
    Ok(values)
}

/// Unix-epoch milliseconds. A clock before 1970 is a named error.
pub(crate) fn epoch_ms() -> Result<u128, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .map_err(|e| format!("clock is before unix epoch: {e}"))
}

fn wall_ms(key: &str, value: &toml::Value) -> Result<u64, String> {
    match value.as_integer() {
        Some(n) if n >= 0 => Ok(n as u64),
        Some(n) => Err(format!("{key} must be >= 0, got {n}")),
        None => Err(format!("{key} must be a non-negative integer, got {value}")),
    }
}

#[cfg(test)]
mod unit {
    use super::*;

    fn production_src() -> &'static str {
        include_str!("slo.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source precedes tests")
    }

    #[test]
    fn required_keys_are_the_three_smoke_walls() {
        assert_eq!(
            REQUIRED_BUDGET_KEYS,
            &["grade_ms", "export_ms", "bank_verify_ms"]
        );
        assert!(!REQUIRED_BUDGET_KEYS.is_empty());
    }

    #[test]
    fn well_formed_budgets_table_yields_three_integers() {
        let text = "\
schema_version = 1
[budgets]
grade_ms = 5000
export_ms = 15000
bank_verify_ms = 10000
";
        assert_eq!(budgets_from_text(text).unwrap(), [5000, 15000, 10000]);
    }

    #[test]
    fn extra_budget_keys_are_ignored() {
        let text = "\
[budgets]
grade_ms = 1
export_ms = 2
bank_verify_ms = 3
unused_ms = 99
";
        assert_eq!(budgets_from_text(text).unwrap(), [1, 2, 3]);
    }

    #[test]
    fn empty_document_is_red() {
        for raw in ["", "   \n\t  "] {
            let err = budgets_from_text(raw).unwrap_err();
            assert!(err.contains(EMPTY_DOCUMENT), "{err}");
        }
    }

    #[test]
    fn missing_budgets_table_is_red() {
        let err = budgets_from_text("schema_version = 1\n").unwrap_err();
        assert!(err.contains(MISSING_BUDGETS), "{err}");
    }

    #[test]
    fn root_level_keys_are_not_a_fallback() {
        // The retired python did `data.get("budgets") or data`. That
        // fallback is deleted: a document with only root-level walls is RED.
        let err = budgets_from_text(
            "\
grade_ms = 5000
export_ms = 15000
bank_verify_ms = 10000
",
        )
        .unwrap_err();
        assert!(err.contains(MISSING_BUDGETS), "{err}");
    }

    #[test]
    fn empty_budgets_table_is_red() {
        let err = budgets_from_text("[budgets]\n").unwrap_err();
        assert!(err.contains(EMPTY_BUDGETS), "{err}");
    }

    #[test]
    fn missing_one_key_is_red_and_names_it() {
        let err = budgets_from_text(
            "\
[budgets]
grade_ms = 1
export_ms = 2
",
        )
        .unwrap_err();
        assert!(err.contains(MISSING_KEYS), "{err}");
        assert!(err.contains("bank_verify_ms"), "{err}");
    }

    #[test]
    fn negative_budget_is_red() {
        let err = budgets_from_text(
            "\
[budgets]
grade_ms = -1
export_ms = 2
bank_verify_ms = 3
",
        )
        .unwrap_err();
        assert!(err.contains("grade_ms"), "{err}");
        assert!(err.contains(">= 0"), "{err}");
    }

    #[test]
    fn float_or_string_budget_is_red() {
        for body in [
            "[budgets]\ngrade_ms = 1.5\nexport_ms = 2\nbank_verify_ms = 3\n",
            "[budgets]\ngrade_ms = \"5000\"\nexport_ms = 2\nbank_verify_ms = 3\n",
        ] {
            let err = budgets_from_text(body).unwrap_err();
            assert!(err.contains("grade_ms"), "{err}");
            assert!(err.contains("non-negative integer"), "{err}");
        }
    }

    #[test]
    fn epoch_ms_is_a_recent_unix_millis() {
        let before = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let got = epoch_ms().unwrap();
        let after = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        assert!(got >= before, "got {got} < before {before}");
        assert!(got <= after, "got {got} > after {after}");
        // 2020-01-01T00:00:00Z in ms. A clock that far back is not "now".
        assert!(got > 1_577_836_800_000, "epoch_ms too old: {got}");
    }

    #[test]
    fn production_has_no_python_and_no_network() {
        let src = production_src();
        for needle in [
            "python3",
            "tomllib",
            "tomli",
            "time.time",
            "TcpStream",
            "UdpSocket",
            "TcpListener",
            "std::net",
            "reqwest",
            "ureq",
        ] {
            assert!(!src.contains(needle), "production mentions {needle}");
        }
        assert!(
            src.contains("doc.get(\"budgets\")"),
            "delete the [budgets] lookup → selftest non-zero"
        );
    }
}
