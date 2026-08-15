//! L5 learner-pack shape — `n_items==40`, `items` len 40, no leaked `correct`.
//!
//! Extracted from the `python3 -c` block in `scripts/check.sh` by
//! `bd-substrate-rust-migration-jhd.35`. The pack is learner-facing
//! (`web/data/mock40_seed42.json`): stems + choices only. Keys live in
//! `keys_seed42.json`. A leaked `correct` letter on an item is a product
//! defect, not a gate concern — this crate is the product path.
//!
//! # Contract (the retired python asserts, fail-closed)
//!
//! * the pack file is present, readable, and JSON
//! * the root is an object
//! * `n_items` is the number 40
//! * `items` is an array of length 40
//! * no item object carries a `correct` key (presence is the leak; value
//!   is irrelevant)
//!
//! Empty / missing / unparseable is an ERROR. A scan that found no items
//! cannot PASS (`items` length 0 ≠ 40).
//!
//! # What this cannot decide
//!
//! It does not grade, does not compare bank_hash, and does not prove the
//! forty stems are the published draw. L6 `export-web` byte-stability and
//! the goldens coupling ledger own those questions.

#![forbid(unsafe_code)]

use crate::{join_rel, BuildOutcome};
use serde_json::Value;
use std::path::Path;

pub const NAME: &str = "check-learner-pack";
pub const SUMMARY: &str = "L5 learner pack: n_items==40, items len 40, no leaked correct letters";

/// Default pack relative to the course-engine root.
pub const DEFAULT_REL: &str = "web/data/mock40_seed42.json";

/// Compiled-in floor. Emptying this to 0 would make an empty pack green.
pub const EXPECTED_N_ITEMS: u64 = 40;

/// Check `<root>/web/data/mock40_seed42.json`.
pub fn run(root: &Path) -> BuildOutcome {
    check_path(&join_rel(root, DEFAULT_REL))
}

/// Check an explicit pack path (temp plants, `--pack`).
pub fn check_path(path: &Path) -> BuildOutcome {
    let display = path.display().to_string();
    if !path.is_file() {
        return fail(vec![format!("{display}: missing (L5 learner pack)")]);
    }
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => return fail(vec![format!("{display}: unreadable: {e}")]),
    };
    if text.trim().is_empty() {
        return fail(vec![format!("{display}: empty file")]);
    }
    let parsed: Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => return fail(vec![format!("{display}: invalid JSON: {e}")]),
    };
    check_value(&parsed, &display)
}

fn check_value(d: &Value, display: &str) -> BuildOutcome {
    let Some(obj) = d.as_object() else {
        return fail(vec![format!("{display}: root is not a JSON object")]);
    };

    let mut errors: Vec<String> = Vec::new();

    match obj.get("n_items") {
        None => errors.push(format!("{display}: n_items missing")),
        Some(v) if is_expected_n(v) => {}
        Some(v) => errors.push(format!("{display}: n_items={v} (want {EXPECTED_N_ITEMS})")),
    }

    match obj.get("items") {
        None => errors.push(format!("{display}: items missing")),
        Some(Value::Array(items)) => {
            if items.len() as u64 != EXPECTED_N_ITEMS {
                errors.push(format!(
                    "{display}: items={} (want {EXPECTED_N_ITEMS})",
                    items.len()
                ));
            }
            let mut leaked: Vec<String> = Vec::new();
            for (i, item) in items.iter().enumerate() {
                let Some(row) = item.as_object() else {
                    errors.push(format!("{display}: items[{i}] is not an object"));
                    continue;
                };
                if row.contains_key("correct") {
                    let who = row
                        .get("id")
                        .and_then(Value::as_str)
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| format!("items[{i}]"));
                    leaked.push(who);
                }
            }
            if !leaked.is_empty() {
                errors.push(format!(
                    "learner pack leaks correct letters ({})",
                    leaked.join(", ")
                ));
            }
        }
        Some(_) => errors.push(format!("{display}: items is not an array")),
    }

    if errors.is_empty() {
        outcome(
            0,
            format!(
                "check_learner_pack: PASS n_items={EXPECTED_N_ITEMS} items={EXPECTED_N_ITEMS} ({display})\n"
            ),
        )
    } else {
        fail(errors)
    }
}

fn is_expected_n(v: &Value) -> bool {
    match v {
        Value::Number(n) => {
            n.as_u64() == Some(EXPECTED_N_ITEMS)
                || n.as_i64() == Some(EXPECTED_N_ITEMS as i64)
                || n.as_f64() == Some(EXPECTED_N_ITEMS as f64)
        }
        _ => false,
    }
}

fn fail(errors: Vec<String>) -> BuildOutcome {
    let mut out = String::from("FAIL: check_learner_pack\n");
    for e in errors {
        out.push_str("  - ");
        out.push_str(&e);
        out.push('\n');
    }
    outcome(1, out)
}

fn outcome(code: i32, stdout: impl Into<String>) -> BuildOutcome {
    BuildOutcome {
        stdout: stdout.into(),
        code,
        artifact: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use std::path::Path;

    fn engine_root() -> std::path::PathBuf {
        crate::resolve_engine_root(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
    }

    fn write_pack(body: &Value) -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("mock40_seed42.json");
        std::fs::write(&path, serde_json::to_string(body).unwrap()).unwrap();
        (dir, path)
    }

    fn minimal_items(n: usize) -> Vec<Value> {
        (0..n)
            .map(|i| {
                json!({
                    "id": format!("t-{i:02}"),
                    "stem": format!("stem {i}"),
                    "choices": ["a", "b", "c", "d"],
                })
            })
            .collect()
    }

    fn good_pack() -> Value {
        json!({
            "n_items": EXPECTED_N_ITEMS,
            "items": minimal_items(EXPECTED_N_ITEMS as usize),
        })
    }

    #[test]
    fn expected_n_is_forty() {
        assert_eq!(EXPECTED_N_ITEMS, 40);
        assert_eq!(DEFAULT_REL, "web/data/mock40_seed42.json");
    }

    #[test]
    fn live_committed_pack_is_green() {
        let out = run(&engine_root());
        assert_eq!(out.code, 0, "{}", out.stdout);
        assert!(
            out.stdout.contains("check_learner_pack: PASS"),
            "{}",
            out.stdout
        );
        assert!(out.artifact.is_none());
    }

    #[test]
    fn planted_correct_letter_is_red() {
        let mut pack = good_pack();
        pack["items"][0]["correct"] = json!("A");
        let (_dir, path) = write_pack(&pack);
        let out = check_path(&path);
        assert_ne!(out.code, 0, "leaked correct must not PASS: {}", out.stdout);
        assert!(
            out.stdout.contains("learner pack leaks correct letters"),
            "{}",
            out.stdout
        );
        assert!(
            out.stdout.contains("t-00"),
            "must name the leaking item: {}",
            out.stdout
        );
    }

    #[test]
    fn n_items_drift_is_red() {
        let mut pack = good_pack();
        pack["n_items"] = json!(39);
        let (_dir, path) = write_pack(&pack);
        let out = check_path(&path);
        assert_ne!(out.code, 0, "{}", out.stdout);
        assert!(out.stdout.contains("n_items=39"), "{}", out.stdout);
    }

    #[test]
    fn items_len_mismatch_is_red() {
        let pack = json!({
            "n_items": 40,
            "items": minimal_items(39),
        });
        let (_dir, path) = write_pack(&pack);
        let out = check_path(&path);
        assert_ne!(out.code, 0, "{}", out.stdout);
        assert!(out.stdout.contains("items=39"), "{}", out.stdout);
    }

    #[test]
    fn missing_file_is_red() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nope.json");
        let out = check_path(&path);
        assert_ne!(out.code, 0, "{}", out.stdout);
        assert!(out.stdout.contains("missing"), "{}", out.stdout);
    }

    #[test]
    fn empty_file_is_red() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.json");
        std::fs::write(&path, "").unwrap();
        let out = check_path(&path);
        assert_ne!(out.code, 0, "{}", out.stdout);
        assert!(out.stdout.contains("empty"), "{}", out.stdout);
    }

    #[test]
    fn invalid_json_is_red() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.json");
        std::fs::write(&path, "{not json").unwrap();
        let out = check_path(&path);
        assert_ne!(out.code, 0, "{}", out.stdout);
        assert!(out.stdout.contains("invalid JSON"), "{}", out.stdout);
    }

    #[test]
    fn missing_n_items_is_red() {
        let pack = json!({ "items": minimal_items(40) });
        let (_dir, path) = write_pack(&pack);
        let out = check_path(&path);
        assert_ne!(out.code, 0, "{}", out.stdout);
        assert!(out.stdout.contains("n_items missing"), "{}", out.stdout);
    }

    #[test]
    fn missing_items_is_red() {
        let pack = json!({ "n_items": 40 });
        let (_dir, path) = write_pack(&pack);
        let out = check_path(&path);
        assert_ne!(out.code, 0, "{}", out.stdout);
        assert!(out.stdout.contains("items missing"), "{}", out.stdout);
    }

    #[test]
    fn empty_items_cannot_pass() {
        let pack = json!({ "n_items": 40, "items": [] });
        let (_dir, path) = write_pack(&pack);
        let out = check_path(&path);
        assert_ne!(out.code, 0, "empty items must not PASS: {}", out.stdout);
        assert!(out.stdout.contains("items=0"), "{}", out.stdout);
    }

    #[test]
    fn correct_key_null_still_leaks() {
        let mut pack = good_pack();
        pack["items"][3]["correct"] = Value::Null;
        let (_dir, path) = write_pack(&pack);
        let out = check_path(&path);
        assert_ne!(out.code, 0, "{}", out.stdout);
        assert!(
            out.stdout.contains("learner pack leaks correct letters"),
            "{}",
            out.stdout
        );
    }

    #[test]
    fn non_object_item_is_red() {
        let mut pack = good_pack();
        pack["items"][1] = json!("not-an-object");
        let (_dir, path) = write_pack(&pack);
        let out = check_path(&path);
        assert_ne!(out.code, 0, "{}", out.stdout);
        assert!(
            out.stdout.contains("items[1] is not an object"),
            "{}",
            out.stdout
        );
    }

    #[test]
    fn good_temp_pack_is_green() {
        let (_dir, path) = write_pack(&good_pack());
        let out = check_path(&path);
        assert_eq!(out.code, 0, "{}", out.stdout);
    }
}
