//! `cdcp publishability` — doctor JSON parse + corpus-rights scan.
//!
//! EXTRACT-THEN-DELETE (`bd-extract-publishability-bar-python-9tji`).
//! `tests/publishability-bar.sh` used to spawn an interpreter to (a) parse
//! the fleet doctor's `--json` body and print sorted error codes, and (b)
//! refuse a corpus manifest whose `sources` are empty or missing `rights`.
//! Those jobs live here so the script body has no live interpreter. Not a
//! gate: a printed code list is not a publishability score, and a present
//! `rights` field is not proof the rights are correct.
//!
//! Fail-closed vs the retired one-liners:
//! - unreadable / non-JSON doctor output prints [`DOCTOR_UNPARSEABLE`]
//!   (the shell treats that token as unexpected);
//! - an empty `sources` list is RED (never vacuously green);
//! - `rights` must be a non-empty string. A number, bool, or object is
//!   not a rights field (the retired `if not s.get("rights")` treated
//!   `1` as present — that fallback is deleted).
//!
//! `doctor-errors` keeps the retired stdout contract so the shell
//! comparison stays a string match: exit 0, one line, codes or the
//! unparseable token. `set -e` must not abort before the comparison.

use serde_json::Value;
use std::fs;
use std::path::Path;

/// Token the retired python printed on any parse failure.
pub(crate) const DOCTOR_UNPARSEABLE: &str = "DOCTOR_UNPARSEABLE";
/// Error token for a 0-byte / whitespace-only manifest.
pub(crate) const EMPTY_DOCUMENT: &str = "empty document";
/// Error token when `sources` is missing, not an array, or length 0.
pub(crate) const EMPTY_SOURCES: &str = "empty sources";
/// Error token prefix when a source lacks a non-empty string `rights`.
pub(crate) const MISSING_RIGHTS: &str = "missing rights";

/// `cdcp publishability doctor-errors --json <path>`.
///
/// Always prints one line and returns `Ok`. A missing file is the
/// unparseable token, not a CLI abort — matching `except Exception`.
pub(crate) fn emit_doctor_errors(path: &Path) -> Result<(), String> {
    if path.as_os_str().is_empty() {
        return Err("publishability doctor-errors: --json is empty".into());
    }
    let raw = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => {
            println!("{DOCTOR_UNPARSEABLE}");
            return Ok(());
        }
    };
    println!("{}", doctor_error_codes(&raw));
    Ok(())
}

/// `cdcp publishability corpus-rights --file <path>`.
pub(crate) fn emit_corpus_rights(path: &Path) -> Result<(), String> {
    if path.as_os_str().is_empty() {
        return Err("publishability corpus-rights: --file is empty".into());
    }
    let raw = fs::read_to_string(path)
        .map_err(|e| format!("publishability corpus-rights: read {}: {e}", path.display()))?;
    let n = check_corpus_rights(&raw)?;
    println!("corpus-rights: ok ({n} sources)");
    Ok(())
}

/// Sorted comma-joined `errors[].code`, or [`DOCTOR_UNPARSEABLE`].
///
/// Retired python:
/// `",".join(sorted(e.get("code", "?") for e in d.get("errors", [])))`
/// wrapped in `except Exception: print("DOCTOR_UNPARSEABLE")`.
pub(crate) fn doctor_error_codes(text: &str) -> String {
    let Ok(val) = serde_json::from_str::<Value>(text) else {
        return DOCTOR_UNPARSEABLE.to_string();
    };
    let Some(obj) = val.as_object() else {
        return DOCTOR_UNPARSEABLE.to_string();
    };
    let errors = match obj.get("errors") {
        None => return String::new(),
        Some(Value::Array(a)) => a,
        Some(_) => return DOCTOR_UNPARSEABLE.to_string(),
    };
    let mut codes = Vec::with_capacity(errors.len());
    for e in errors {
        let Some(map) = e.as_object() else {
            return DOCTOR_UNPARSEABLE.to_string();
        };
        let code = match map.get("code") {
            None => "?".to_string(),
            Some(Value::String(s)) => s.clone(),
            Some(_) => return DOCTOR_UNPARSEABLE.to_string(),
        };
        codes.push(code);
    }
    codes.sort();
    codes.join(",")
}

/// Every `sources[]` row must carry a non-empty string `rights`.
///
/// Returns the source count so a scan that looked at nothing cannot
/// hide behind a bare `Ok`.
pub(crate) fn check_corpus_rights(text: &str) -> Result<usize, String> {
    if text.trim().is_empty() {
        return Err(format!(
            "{EMPTY_DOCUMENT} — a 0-byte manifest pins no sources"
        ));
    }
    let doc: Value =
        serde_json::from_str(text).map_err(|e| format!("corpus manifest is not JSON: {e}"))?;
    let sources = match doc.get("sources") {
        Some(Value::Array(a)) => a,
        _ => {
            return Err(format!(
                "{EMPTY_SOURCES} — a manifest that names no sources certifies nothing"
            ));
        }
    };
    if sources.is_empty() {
        return Err(format!(
            "{EMPTY_SOURCES} — a manifest that names no sources certifies nothing"
        ));
    }
    let mut missing = Vec::new();
    for s in sources {
        let Some(map) = s.as_object() else {
            missing.push("?".to_string());
            continue;
        };
        let url = match map.get("url") {
            Some(Value::String(u)) if !u.is_empty() => u.clone(),
            _ => "?".to_string(),
        };
        if !rights_present(map.get("rights")) {
            missing.push(url);
        }
    }
    if !missing.is_empty() {
        return Err(format!("{MISSING_RIGHTS}: {}", missing.join(", ")));
    }
    Ok(sources.len())
}

/// A rights field is a non-empty string. Missing / null / empty / non-string
/// is absent. Whitespace-only is absent (the retired `if not s.get("rights")`
/// treated `"   "` as present — that is not recording rights).
fn rights_present(value: Option<&Value>) -> bool {
    match value {
        Some(Value::String(s)) => !s.trim().is_empty(),
        _ => false,
    }
}

#[cfg(test)]
mod unit {
    use super::*;

    fn production_src() -> &'static str {
        include_str!("publishability.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source precedes tests")
    }

    #[test]
    fn two_codes_print_sorted_and_comma_joined() {
        let json = r#"{"errors":[{"code":"z_last"},{"code":"a_first"}]}"#;
        assert_eq!(doctor_error_codes(json), "a_first,z_last");
    }

    #[test]
    fn last_code_is_not_first_when_two_exist() {
        let json = r#"{"errors":[{"code":"brand_voice_composite_low"},{"code":"aaa"}]}"#;
        assert_ne!(doctor_error_codes(json), "brand_voice_composite_low");
        assert_eq!(doctor_error_codes(json), "aaa,brand_voice_composite_low");
    }

    #[test]
    fn missing_errors_key_is_empty_not_unparseable() {
        // Retired `.get("errors", [])`.
        assert_eq!(doctor_error_codes(r#"{"status":"pass"}"#), "");
    }

    #[test]
    fn empty_errors_array_is_empty() {
        assert_eq!(doctor_error_codes(r#"{"errors":[]}"#), "");
    }

    #[test]
    fn missing_code_field_is_question_mark() {
        assert_eq!(doctor_error_codes(r#"{"errors":[{}]}"#), "?");
    }

    #[test]
    fn duplicate_codes_are_kept() {
        let json = r#"{"errors":[{"code":"a"},{"code":"a"}]}"#;
        assert_eq!(doctor_error_codes(json), "a,a");
    }

    #[test]
    fn unparseable_bodies_emit_the_token() {
        for raw in ["", "   \n", "not-json", "[]", "null", r#"{"errors":{}}"#] {
            assert_eq!(doctor_error_codes(raw), DOCTOR_UNPARSEABLE, "body {raw:?}");
        }
    }

    #[test]
    fn non_object_error_item_is_unparseable() {
        assert_eq!(
            doctor_error_codes(r#"{"errors":["brand_voice_composite_low"]}"#),
            DOCTOR_UNPARSEABLE
        );
    }

    #[test]
    fn non_string_code_is_unparseable() {
        assert_eq!(
            doctor_error_codes(r#"{"errors":[{"code":1}]}"#),
            DOCTOR_UNPARSEABLE
        );
    }

    #[test]
    fn well_formed_sources_count() {
        let text = r#"{
          "sources": [
            {"url":"https://a.example","rights":"publisher-retains-copyright"},
            {"url":"https://b.example","rights":"cc-by"}
          ]
        }"#;
        assert_eq!(check_corpus_rights(text).unwrap(), 2);
    }

    #[test]
    fn empty_document_is_red() {
        for raw in ["", "   \n\t  "] {
            let err = check_corpus_rights(raw).unwrap_err();
            assert!(err.contains(EMPTY_DOCUMENT), "{err}");
        }
    }

    #[test]
    fn empty_sources_is_red() {
        let err = check_corpus_rights(r#"{"sources":[]}"#).unwrap_err();
        assert!(err.contains(EMPTY_SOURCES), "{err}");
    }

    #[test]
    fn missing_sources_key_is_red() {
        let err = check_corpus_rights(r#"{"schema":"x"}"#).unwrap_err();
        assert!(err.contains(EMPTY_SOURCES), "{err}");
    }

    #[test]
    fn missing_rights_is_red_and_names_the_url() {
        let err =
            check_corpus_rights(r#"{"sources":[{"url":"https://no-rights.example","title":"x"}]}"#)
                .unwrap_err();
        assert!(err.contains(MISSING_RIGHTS), "{err}");
        assert!(err.contains("https://no-rights.example"), "{err}");
    }

    #[test]
    fn empty_string_rights_is_red() {
        let err =
            check_corpus_rights(r#"{"sources":[{"url":"https://empty.example","rights":""}]}"#)
                .unwrap_err();
        assert!(err.contains(MISSING_RIGHTS), "{err}");
    }

    #[test]
    fn whitespace_only_rights_is_red() {
        let err =
            check_corpus_rights(r#"{"sources":[{"url":"https://ws.example","rights":"   "}]}"#)
                .unwrap_err();
        assert!(err.contains(MISSING_RIGHTS), "{err}");
    }

    #[test]
    fn numeric_rights_is_not_a_fallback() {
        // Retired python: `if not s.get("rights")` treats `1` as present.
        let err = check_corpus_rights(r#"{"sources":[{"url":"https://num.example","rights":1}]}"#)
            .unwrap_err();
        assert!(err.contains(MISSING_RIGHTS), "{err}");
    }

    #[test]
    fn production_has_no_python_and_no_network() {
        let src = production_src();
        for needle in [
            "python3",
            "tomllib",
            "tomli",
            "TcpStream",
            "UdpSocket",
            "TcpListener",
            "std::net",
            "reqwest",
            "ureq",
            "cdcp_gate",
            "Command::new",
        ] {
            assert!(!src.contains(needle), "production mentions {needle}");
        }
        assert!(
            src.contains("d.get(\"errors\""),
            "delete the retired-errors contract → selftest non-zero"
        );
        assert!(
            src.contains("rights_present"),
            "delete the rights predicate → selftest non-zero"
        );
    }
}
