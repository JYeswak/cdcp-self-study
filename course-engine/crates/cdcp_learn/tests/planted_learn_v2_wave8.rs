//! Independent verdicts for the five wave-8 learn-v2 agreement-only cases
//! (`bd-wave8-ports-agreement-only-debt-idns`).
//!
//! EXTRACT-THEN-DELETE (jhd.20) retired `scripts/smoke_learn_v2.py` and the
//! `cdcp_gate` port. Product is `cdcp_learn::learn_v2`. Moved out of
//! `cdcp_gate/tests` by bd-engine-not-gate-ar39.7 so the leftover plants
//! live with the product caller.
//!
//! Each case is now pointed at a PLANTED fixture and asserts the named
//! FAIL row, a fixture-specific COUNT / marker (so a silent fallback is
//! visible), and the verdict line. Known-bad plants the same fallback
//! defect in both implementations.

use serde_json::{json, Value};
use std::path::{Path, PathBuf};

struct Run {
    code: i32,
    stdout: String,
}

fn engine_root() -> PathBuf {
    cdcp_learn::resolve_engine_root(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
}

fn cmp(label: &str, root: &Path) -> Run {
    run_at(label, root)
}

/// Census name for the five crash-class sites. The product no longer
/// raises: a malformed / non-UTF-8 / non-object input is a FAIL row.
fn compare_crash(label: &str, root: &Path) -> Run {
    run_at(label, root)
}

fn run_at(label: &str, root: &Path) -> Run {
    let o = cdcp_learn::learn_v2::run(root);
    assert!(
        o.artifact.is_none(),
        "[{label}] a reader must not propose a write"
    );
    Run {
        code: o.code,
        stdout: o.stdout,
    }
}

fn unit(uid: &str, checks: usize) -> Value {
    let ids: Vec<String> = (0..checks).map(|k| format!("{uid}-c{k}")).collect();
    json!({"id": uid, "check_item_ids": ids})
}

/// Fixture marker: declared unit_count is 77, glossary term_count is 41.
/// Live tree is neither. A silent fallback cannot print both markers.
fn good_payload() -> Value {
    let m01: Vec<Value> = (0..4).map(|k| unit(&format!("u01-{k}"), 2)).collect();
    let m06: Vec<Value> = (0..3).map(|k| unit(&format!("u06-{k}"), 2)).collect();
    let mut units = m01.clone();
    units.extend(m06.clone());
    json!({
        "unit_count": 77,
        "approved_item_count": 14,
        "bank_item_count": 14,
        "by_module": {
            "01-mission-critical": m01,
            "06-power": m06,
        },
        "units": units,
    })
}

fn collect_ids(payload: &Value) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(units) = payload.get("units").and_then(|u| u.as_array()) {
        for u in units {
            if let Some(ids) = u.get("check_item_ids").and_then(|a| a.as_array()) {
                for id in ids {
                    if let Some(s) = id.as_str() {
                        out.push(s.to_string());
                    }
                }
            }
        }
    }
    out
}

fn bank_pack(ids: &[String]) -> String {
    let items: Vec<Value> = ids
        .iter()
        .map(|id| json!({"id": id, "status": "approved", "module": 1}))
        .collect();
    Value::Array(items).to_string()
}

struct Fixture {
    dir: tempfile::TempDir,
    root: PathBuf,
}

impl Fixture {
    fn planted() -> Fixture {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("r");
        let payload = good_payload();
        put(&root, "web/data/units_index.json", &payload.to_string());
        put(
            &root,
            "web/data/bank_items_seed42.json",
            &bank_pack(&collect_ids(&payload)),
        );
        put(&root, "web/data/glossary.json", "{\"term_count\": 41}\n");
        for name in ["learn_units.js", "learn_glossary.js", "concept_card.js"] {
            put(&root, &format!("web/assets/js/{name}"), "// script\n");
        }
        put(
            &root,
            "web/learn/01-mission-critical.html",
            "<div class=\"learn-unit-shell\"></div>\
             <script src=\"learn_units.js\"></script>\
             <script src=\"learn_glossary.js\"></script>\n",
        );
        put(
            &root,
            "web/drill.html",
            "<script src=\"concept_card.js\"></script>\n",
        );
        Fixture { dir, root }
    }

    fn put(&self, rel: &str, body: &str) {
        put(&self.root, rel, body);
    }

    fn put_bytes(&self, rel: &str, body: &[u8]) {
        let p = self.root.join(rel);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(p, body).unwrap();
    }
}

fn put(root: &Path, rel: &str, body: &str) {
    let p = root.join(rel);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(p, body).unwrap();
}

fn assert_planted_fail(label: &str, rs: &Run, fail_row: &str, marker: &str) {
    let out = &rs.stdout;
    assert_ne!(rs.code, 0, "[{label}] planted input stayed GREEN:\n{out}");
    assert!(
        out.contains(&format!("  FAIL: {fail_row}\n")),
        "[{label}] missing named FAIL row {fail_row:?}:\n{out}"
    );
    assert!(
        out.contains(marker),
        "[{label}] fixture marker {marker:?} missing — the run did not scan \
         the planted tree (a silent fallback would print the live tree):\n{out}"
    );
    assert!(
        out.contains("smoke_learn_v2: FAIL"),
        "[{label}] must reach the verdict line, not panic:\n{out}"
    );
    assert!(
        !out.contains("smoke_learn_v2: PASS"),
        "[{label}] PASS on a planted fail:\n{out}"
    );
}

#[test]
fn planted_malformed_units_index_is_red_and_still_grades_the_fixture_glossary() {
    let f = Fixture::planted();
    f.put("web/data/units_index.json", "{\"unit_count\": }\n");
    let rs = compare_crash("malformed units index", &f.root);
    // Glossary still grades, so term_count=41 proves THIS tree was scanned.
    assert_planted_fail(
        "malformed units index",
        &rs,
        "units_index.json is not valid JSON",
        "ok: glossary terms=41",
    );
    assert!(
        !rs.stdout.contains("ok: unit_count="),
        "malformed units must not be graded: {}",
        rs.stdout
    );
    let _ = &f.dir;
}

#[test]
fn planted_malformed_glossary_is_red_and_still_grades_the_fixture_units() {
    let f = Fixture::planted();
    f.put("web/data/glossary.json", "not json at all\n");
    let rs = compare_crash("malformed glossary", &f.root);
    assert_planted_fail(
        "malformed glossary",
        &rs,
        "glossary.json is not valid JSON",
        "ok: unit_count=77",
    );
    let _ = &f.dir;
}

#[test]
fn planted_trailing_comma_glossary_is_red_and_names_the_row() {
    let f = Fixture::planted();
    f.put("web/data/glossary.json", "{\"term_count\": 41,}\n");
    let rs = compare_crash("trailing comma is not json", &f.root);
    assert_planted_fail(
        "trailing comma is not json",
        &rs,
        "glossary.json is not valid JSON",
        "ok: unit_count=77",
    );
    let _ = &f.dir;
}

#[test]
fn planted_undecodable_units_index_is_red_and_still_grades_the_fixture_glossary() {
    let f = Fixture::planted();
    f.put_bytes("web/data/units_index.json", &[b'{', 0x80, b'}']);
    let rs = compare_crash("units index is not utf-8", &f.root);
    assert_planted_fail(
        "units index is not utf-8",
        &rs,
        "units_index.json is not valid UTF-8",
        "ok: glossary terms=41",
    );
    let _ = &f.dir;
}

#[test]
fn planted_units_index_array_is_red_and_still_grades_the_fixture_glossary() {
    let f = Fixture::planted();
    f.put("web/data/units_index.json", "[1, 2, 3]\n");
    let rs = compare_crash("units index is a json array", &f.root);
    assert_planted_fail(
        "units index is a json array",
        &rs,
        "units_index.json is not a JSON object",
        "ok: glossary terms=41",
    );
    let _ = &f.dir;
}

/// The honest green control: the fixture markers exist so a misspelled
/// marker in the five cases above cannot pass by matching nothing.
#[test]
fn planted_good_fixture_is_green_and_prints_the_markers() {
    let f = Fixture::planted();
    let rs = cmp("synthetic good", &f.root);
    assert_eq!(rs.code, 0, "{}", rs.stdout);
    assert!(rs.stdout.contains("ok: unit_count=77"), "{}", rs.stdout);
    assert!(rs.stdout.contains("ok: glossary terms=41"), "{}", rs.stdout);
    assert!(rs.stdout.contains("smoke_learn_v2: PASS"), "{}", rs.stdout);
    let _ = &f.dir;
}

/// Known-bad: both implementations ignore the named root (shared fallback).
/// They agree, and they are GREEN. The converted verdict does not trip —
/// which is the point: under the old assertion this passed.
#[test]
fn known_bad_shared_fallback_passes_agreement_and_fails_the_converted_verdict() {
    let f = Fixture::planted();
    f.put("web/data/units_index.json", "{\"unit_count\": }\n");
    let py = cdcp_learn::learn_v2::run(&engine_root());
    let rs = cdcp_learn::learn_v2::run(&engine_root());
    assert_eq!(py.code, rs.code);
    assert_eq!(py.stdout, rs.stdout);
    assert_eq!(
        py.code, 0,
        "live fallback is GREEN — the old assertion would pass:\n{}",
        py.stdout
    );
    let converted_trips = rs.code != 0
        && rs
            .stdout
            .contains("  FAIL: units_index.json is not valid JSON\n")
        && rs.stdout.contains("ok: glossary terms=41");
    assert!(
        !converted_trips,
        "shared fallback must fail the converted verdict; got:\n{}",
        rs.stdout
    );
    let honest = compare_crash("control: honest scan of the plant", &f.root);
    assert_planted_fail(
        "control",
        &honest,
        "units_index.json is not valid JSON",
        "ok: glossary terms=41",
    );
    let _ = &f.dir;
}
