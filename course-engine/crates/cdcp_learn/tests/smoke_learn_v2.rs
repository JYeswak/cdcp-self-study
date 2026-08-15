//! Verdict suite for `cdcp_learn::learn_v2` (extracted by
//! bd-substrate-rust-migration-jhd.20). The Python is DELETED. Every case
//! asserts the correct answer, not agreement with a retired script.
//!
//! This smoke is a READER: it reads the Learn-v2 surface and writes nothing.
//! Cases run against TEMP fixtures (or the live tree read-only).
//!
//! ANTI-VACUOUS: empty `units` is RED (bd-learnv2-vacuous-coverage-dad8);
//! empty M01 is RED; a missing / malformed / non-object / non-UTF-8 input
//! is a named FAIL + verdict, not a panic (bd-learnv2-unguarded-reads-0f7g
//! closed). Check-item floors count APPROVED ids only (a20t): a unit whose
//! only checks are retired is not covered. Zero files / empty scan is RED.
//! A suite that ran no case is RED.

use cdcp_learn::learn_v2::{run, JS_ASSETS, M01_NEEDLES};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static RAN: AtomicUsize = AtomicUsize::new(0);
static ROUND: AtomicUsize = AtomicUsize::new(0);

/// Raise when you add a `#[test]`. A DROP means a case was deleted.
const EXPECTED_CASES: usize = 25;

fn engine_root() -> PathBuf {
    cdcp_learn::resolve_engine_root(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
}

fn tick() {
    RAN.fetch_add(1, Ordering::SeqCst);
}

struct Fixture {
    dir: tempfile::TempDir,
    root: PathBuf,
}

impl Fixture {
    fn new() -> Fixture {
        let dir = tempfile::tempdir().unwrap();
        let n = ROUND.fetch_add(1, Ordering::SeqCst);
        let root = dir.path().join(format!("r{n}"));
        std::fs::create_dir_all(&root).unwrap();
        Fixture { dir, root }
    }

    fn at(&self, rel: &str) -> PathBuf {
        cdcp_learn::join_rel(&self.root, rel)
    }

    fn put(&self, rel: &str, body: &str) {
        let p = self.at(rel);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(p, body).unwrap();
    }

    fn put_bytes(&self, rel: &str, body: &[u8]) {
        let p = self.at(rel);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(p, body).unwrap();
    }

    fn rm(&self, rel: &str) {
        let p = self.at(rel);
        if p.is_dir() {
            std::fs::remove_dir_all(&p).unwrap();
        } else if p.exists() {
            std::fs::remove_file(&p).unwrap();
        }
    }
}

fn unit(uid: &str, checks: usize) -> Value {
    let ids: Vec<String> = (0..checks).map(|k| format!("{uid}-c{k}")).collect();
    json!({"id": uid, "check_item_ids": ids})
}

fn good_payload() -> Value {
    let m01: Vec<Value> = (0..4).map(|k| unit(&format!("u01-{k}"), 2)).collect();
    let m06: Vec<Value> = (0..3).map(|k| unit(&format!("u06-{k}"), 2)).collect();
    let mut units = m01.clone();
    units.extend(m06.clone());
    json!({
        "unit_count": 60,
        "approved_item_count": 14,
        "bank_item_count": 14,
        "by_module": {
            "01-mission-critical": m01,
            "06-power": m06,
        },
        "units": units,
    })
}

fn collect_check_ids(payload: &Value) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    let mut take = |rows: &[Value]| {
        for u in rows {
            if let Some(ids) = u.get("check_item_ids").and_then(|a| a.as_array()) {
                for id in ids {
                    if let Some(s) = id.as_str() {
                        if seen.insert(s.to_string()) {
                            out.push(s.to_string());
                        }
                    }
                }
            }
        }
    };
    if let Some(units) = payload.get("units").and_then(|u| u.as_array()) {
        take(units);
    }
    if let Some(by) = payload.get("by_module").and_then(|o| o.as_object()) {
        for v in by.values() {
            if let Some(arr) = v.as_array() {
                take(arr);
            }
        }
    }
    out
}

/// Seed-42 pack for a fixture. `check_status` is written on every id drawn
/// from the payload. A sentinel approved row is added when that would leave
/// the approved pool empty, so a retired-only known-bad measures coverage
/// rather than collapsing into the empty-pool error.
fn bank_pack(ids: &[String], check_status: &str) -> String {
    let mut items: Vec<Value> = ids
        .iter()
        .map(|id| json!({"id": id, "status": check_status, "module": 1}))
        .collect();
    if items.is_empty() || check_status != "approved" {
        items.push(json!({"id": "sentinel-approved", "status": "approved", "module": 1}));
    }
    Value::Array(items).to_string()
}

const GOOD_GLOSSARY: &str = "{\"term_count\": 40}\n";
const GOOD_M01_HTML: &str = concat!(
    "<div class=\"learn-unit-shell\"></div>",
    "<script src=\"learn_units.js\"></script>",
    "<script src=\"learn_glossary.js\"></script>\n",
);
const GOOD_DRILL: &str = "<script src=\"concept_card.js\"></script>\n";

fn plant(payload: &Value) -> Fixture {
    plant_with_bank(payload, "approved")
}

fn plant_with_bank(payload: &Value, check_status: &str) -> Fixture {
    let f = Fixture::new();
    f.put("web/data/units_index.json", &payload.to_string());
    f.put(
        "web/data/bank_items_seed42.json",
        &bank_pack(&collect_check_ids(payload), check_status),
    );
    f.put("web/data/glossary.json", GOOD_GLOSSARY);
    for name in ["learn_units.js", "learn_glossary.js", "concept_card.js"] {
        f.put(&format!("web/assets/js/{name}"), "// script\n");
    }
    f.put("web/learn/01-mission-critical.html", GOOD_M01_HTML);
    f.put("web/drill.html", GOOD_DRILL);
    f
}

fn tree_digest(dir: &Path) -> Vec<(String, u64)> {
    fn walk(base: &Path, dir: &Path, out: &mut Vec<(String, u64)>) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(base, &p, out);
            } else if let Ok(bytes) = std::fs::read(&p) {
                let rel = p.strip_prefix(base).unwrap().to_string_lossy().into_owned();
                let mut h: u64 = 0xcbf2_9ce4_8422_2325;
                for b in &bytes {
                    h ^= u64::from(*b);
                    h = h.wrapping_mul(0x1000_0000_01b3);
                }
                out.push((rel, h));
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, dir, &mut out);
    out.sort();
    out
}

fn assert_fail_row(o: &cdcp_learn::BuildOutcome, needle: &str) {
    assert_ne!(o.code, 0, "known-bad stayed GREEN:\n{}", o.stdout);
    assert!(
        o.stdout.contains(&format!("  FAIL: {needle}\n")),
        "missing FAIL row {needle:?}:\n{}",
        o.stdout
    );
    assert!(
        o.stdout.contains("smoke_learn_v2: FAIL"),
        "known-bad must reach the verdict line, not panic:\n{}",
        o.stdout
    );
    assert!(
        !o.stdout.contains("smoke_learn_v2: PASS"),
        "known-bad reached PASS:\n{}",
        o.stdout
    );
}

#[test]
fn live_tree_passes() {
    tick();
    let root = engine_root();
    assert!(
        root.join("web").is_dir(),
        "live engine has no web/ — a missing product tree is an ERROR"
    );
    let before = tree_digest(&root.join("web"));
    let o = run(&root);
    assert_eq!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout.starts_with("==> smoke_learn_v2 (M8 B/D assets)\n"),
        "{}",
        o.stdout
    );
    assert!(o.stdout.contains("smoke_learn_v2: PASS"), "{}", o.stdout);
    assert!(o.stdout.contains("ok: unit_count="), "{}", o.stdout);
    assert!(
        o.stdout
            .contains("ok: M01 every unit has ≥2 approved check items"),
        "{}",
        o.stdout
    );
    assert!(
        o.stdout.contains("ok: approved check_item_ids coverage"),
        "live floors must name the approved population:\n{}",
        o.stdout
    );
    assert!(
        o.stdout
            .contains("ok: populations named: bank_item_count + approved_item_count"),
        "live units_index must pair the file-set count with the approved pool:\n{}",
        o.stdout
    );
    assert!(o.artifact.is_none(), "smoke is a reader");
    assert_eq!(
        before,
        tree_digest(&root.join("web")),
        "the live tree was modified by a reader"
    );
}

#[test]
fn green_fixture_passes() {
    tick();
    let f = plant(&good_payload());
    let o = run(&f.root);
    assert_eq!(o.code, 0, "{}", o.stdout);
    assert!(o.stdout.contains("smoke_learn_v2: PASS"), "{}", o.stdout);
    assert!(o.stdout.contains("ok: unit_count=60"), "{}", o.stdout);
    assert!(
        o.stdout
            .contains("ok: approved check_item_ids coverage 7/7"),
        "{}",
        o.stdout
    );
    assert!(
        o.stdout
            .contains("ok: M01 every unit has ≥2 approved check items"),
        "{}",
        o.stdout
    );
    assert!(o.stdout.contains("ok: glossary terms=40"), "{}", o.stdout);
    assert!(o.artifact.is_none());
    let _ = &f.dir;
}

#[test]
fn empty_tree_is_an_error() {
    tick();
    let f = Fixture::new();
    let o = run(&f.root);
    assert_ne!(o.code, 0, "empty tree must be RED, got PASS:\n{}", o.stdout);
    assert!(o.stdout.contains("smoke_learn_v2: FAIL"), "{}", o.stdout);
    assert!(
        o.stdout
            .contains("FAIL: missing units_index.json — run `cdcp build-units`"),
        "{}",
        o.stdout
    );
    assert!(
        o.stdout.contains("FAIL: missing glossary.json"),
        "{}",
        o.stdout
    );
    assert!(
        o.stdout.contains("FAIL: missing drill.html"),
        "{}",
        o.stdout
    );
    assert!(
        !o.stdout.contains("smoke_learn_v2: PASS"),
        "PASS must not appear on a failing run:\n{}",
        o.stdout
    );
    let _ = &f.dir;
}

/// Known-bad (dad8): by_module populated, units=[], must be RED — never ok 0/0.
#[test]
fn empty_units_is_red() {
    tick();
    let m01: Vec<Value> = (0..4).map(|k| unit(&format!("u01-{k}"), 2)).collect();
    let m06: Vec<Value> = (0..3).map(|k| unit(&format!("u06-{k}"), 2)).collect();
    let payload = json!({
        "unit_count": 60,
        "by_module": {"01-mission-critical": m01, "06-power": m06},
        "units": [],
    });
    let f = plant(&payload);
    let o = run(&f.root);
    assert_fail_row(
        &o,
        "zero units in units_index.json (vacuous coverage is ERROR)",
    );
    assert!(
        !o.stdout
            .contains("ok: approved check_item_ids coverage 0/0"),
        "vacuous ok 0/0 still printed:\n{}",
        o.stdout
    );
    assert!(
        !o.stdout.contains("ok: check_item_ids coverage 0/0"),
        "pre-a20t vacuous ok 0/0 still printed:\n{}",
        o.stdout
    );
    let _ = &f.dir;
}

/// Known-bad (dad8): M01 list empty, units otherwise populated — floor must not mask.
#[test]
fn empty_m01_is_red() {
    tick();
    let m06: Vec<Value> = (0..3).map(|k| unit(&format!("u06-{k}"), 2)).collect();
    let payload = json!({
        "unit_count": 60,
        "by_module": {"01-mission-critical": [], "06-power": m06},
        "units": m06,
    });
    let f = plant(&payload);
    let o = run(&f.root);
    assert_fail_row(
        &o,
        "zero M01 units in units_index.json (vacuous M01 check-item floor is ERROR)",
    );
    assert!(
        !o.stdout
            .contains("ok: M01 every unit has ≥2 approved check items"),
        "vacuous M01 ok still printed:\n{}",
        o.stdout
    );
    assert!(
        !o.stdout.contains("ok: M01 every unit has ≥2 check items"),
        "pre-a20t vacuous M01 ok still printed:\n{}",
        o.stdout
    );
    let _ = &f.dir;
}

/// Known-bad (0f7g): missing Learn page is a FAIL row, not a panic.
#[test]
fn missing_m01_page_is_red() {
    tick();
    let f = plant(&good_payload());
    f.rm("web/learn/01-mission-critical.html");
    let o = run(&f.root);
    assert_fail_row(&o, "missing learn/01-mission-critical.html");
    assert!(
        o.stdout.contains("ok: drill concept_card"),
        "grading after the defect was skipped:\n{}",
        o.stdout
    );
    for token in [
        "ok: M01 learn-unit-shell",
        "ok: M01 learn_units.js",
        "ok: M01 learn_glossary.js",
    ] {
        assert!(
            !o.stdout.contains(token),
            "M01 needle graded despite missing page:\n{}",
            o.stdout
        );
    }
    let _ = &f.dir;
}

/// Known-bad (0f7g): missing drill.html is a FAIL row, not a panic.
#[test]
fn missing_drill_is_red() {
    tick();
    let f = plant(&good_payload());
    f.rm("web/drill.html");
    let o = run(&f.root);
    assert_fail_row(&o, "missing drill.html");
    assert!(
        o.stdout.contains("ok: M01 learn_glossary.js"),
        "grading after the defect was skipped:\n{}",
        o.stdout
    );
    assert!(
        !o.stdout.contains("ok: drill concept_card"),
        "drill graded despite missing page:\n{}",
        o.stdout
    );
    let _ = &f.dir;
}

/// Known-bad (0f7g): malformed units JSON is a FAIL row, not a panic.
#[test]
fn malformed_units_json_is_red() {
    tick();
    let f = plant(&good_payload());
    f.put("web/data/units_index.json", "{\"unit_count\": }\n");
    let o = run(&f.root);
    assert_fail_row(&o, "units_index.json is not valid JSON");
    assert!(
        o.stdout.contains("ok: glossary terms=40"),
        "grading after the defect was skipped:\n{}",
        o.stdout
    );
    assert!(
        !o.stdout.contains("ok: unit_count="),
        "malformed units still graded:\n{}",
        o.stdout
    );
    let _ = &f.dir;
}

/// Known-bad (0f7g): malformed glossary JSON is a FAIL row, not a panic.
#[test]
fn malformed_glossary_json_is_red() {
    tick();
    let f = plant(&good_payload());
    f.put("web/data/glossary.json", "not json at all\n");
    let o = run(&f.root);
    assert_fail_row(&o, "glossary.json is not valid JSON");
    assert!(
        o.stdout.contains("ok: unit_count=60"),
        "grading after the defect was skipped:\n{}",
        o.stdout
    );
    assert!(
        !o.stdout.contains("ok: glossary terms="),
        "malformed glossary still graded:\n{}",
        o.stdout
    );
    let _ = &f.dir;
}

/// Known-bad (0f7g): a JSON array has no object keys — grade it, do not panic.
#[test]
fn units_json_array_is_red() {
    tick();
    let f = plant(&good_payload());
    f.put("web/data/units_index.json", "[1, 2, 3]\n");
    let o = run(&f.root);
    assert_fail_row(&o, "units_index.json is not a JSON object");
    assert!(
        o.stdout.contains("ok: glossary terms=40"),
        "grading after the defect was skipped:\n{}",
        o.stdout
    );
    let _ = &f.dir;
}

/// Known-bad (0f7g): every UTF-8 read site FAIL-closes, never a panic.
#[test]
fn undecodable_utf8_is_red() {
    tick();
    let plants = [
        (
            "web/data/units_index.json",
            "units_index.json is not valid UTF-8",
            "ok: glossary terms=40",
        ),
        (
            "web/data/glossary.json",
            "glossary.json is not valid UTF-8",
            "ok: unit_count=60",
        ),
        (
            "web/learn/01-mission-critical.html",
            "learn/01-mission-critical.html is not valid UTF-8",
            "ok: drill concept_card",
        ),
        (
            "web/drill.html",
            "drill.html is not valid UTF-8",
            "ok: M01 learn_glossary.js",
        ),
    ];
    let bad = [b'{', 0x80, b'}'];
    for (rel, needle, continued) in plants {
        let f = plant(&good_payload());
        f.put_bytes(rel, &bad);
        let o = run(&f.root);
        assert_fail_row(&o, needle);
        assert!(
            o.stdout.contains(continued),
            "grading after the defect was skipped ({rel}):\n{}",
            o.stdout
        );
        let _ = &f.dir;
    }
}

#[test]
fn unit_count_too_low_is_red() {
    tick();
    let mut payload = good_payload();
    payload["unit_count"] = json!(10);
    let f = plant(&payload);
    let o = run(&f.root);
    assert_fail_row(&o, "unit_count too low");
    let _ = &f.dir;
}

#[test]
fn glossary_term_floor_is_red() {
    tick();
    let f = plant(&good_payload());
    f.put("web/data/glossary.json", "{\"term_count\": 3}\n");
    let o = run(&f.root);
    assert_ne!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout.contains("FAIL: glossary term_count"),
        "{}",
        o.stdout
    );
    assert!(o.stdout.contains("smoke_learn_v2: FAIL"), "{}", o.stdout);
    let _ = &f.dir;
}

#[test]
fn coverage_below_eighty_is_red() {
    tick();
    let m01: Vec<Value> = (0..4).map(|k| unit(&format!("u01-{k}"), 0)).collect();
    let m06: Vec<Value> = (0..3).map(|k| unit(&format!("u06-{k}"), 0)).collect();
    let mut units = m01.clone();
    units.extend(m06.clone());
    let payload = json!({
        "unit_count": 60,
        "by_module": {"01-mission-critical": m01, "06-power": m06},
        "units": units,
    });
    let f = plant(&payload);
    let o = run(&f.root);
    assert_ne!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout
            .contains("approved check_item_ids coverage 0/7 < 80%"),
        "{}",
        o.stdout
    );
    assert!(o.stdout.contains("smoke_learn_v2: FAIL"), "{}", o.stdout);
    let _ = &f.dir;
}

#[test]
fn missing_js_asset_is_red() {
    tick();
    let f = plant(&good_payload());
    f.rm("web/assets/js/concept_card.js");
    let o = run(&f.root);
    assert_fail_row(&o, "missing assets/js/concept_card.js");
    let _ = &f.dir;
}

#[test]
fn m01_missing_needle_is_red() {
    tick();
    let f = plant(&good_payload());
    f.put(
        "web/learn/01-mission-critical.html",
        "<script src=\"learn_units.js\"></script><script src=\"learn_glossary.js\"></script>\n",
    );
    let o = run(&f.root);
    assert_fail_row(&o, "M01 missing learn-unit-shell");
    let _ = &f.dir;
}

#[test]
fn drill_missing_concept_card_is_red() {
    tick();
    let f = plant(&good_payload());
    f.put("web/drill.html", "<html>no card</html>\n");
    let o = run(&f.root);
    assert_fail_row(&o, "drill.html missing concept_card.js");
    let _ = &f.dir;
}

/// Stricter than the retired script: a type mismatch is a FAIL, not a raise.
#[test]
fn type_mismatch_units_not_array_is_red_not_a_panic() {
    tick();
    let f = plant(&good_payload());
    f.put(
        "web/data/units_index.json",
        r#"{"unit_count":60,"by_module":{"01-mission-critical":[{},{},{},{}],"06-power":[{},{},{}]},"units":"nope"}"#,
    );
    let o = run(&f.root);
    assert_fail_row(&o, "units_index.json units is not a JSON array");
    let _ = &f.dir;
}

#[test]
fn the_reader_writes_nothing() {
    tick();
    let f = plant(&good_payload());
    let before = tree_digest(&f.root);
    let o = run(&f.root);
    assert_eq!(o.code, 0, "{}", o.stdout);
    assert_eq!(o.artifact, None);
    assert_eq!(
        before,
        tree_digest(&f.root),
        "a reader wrote into the fixture"
    );
    let _ = &f.dir;
}

/// Known-bad (a20t): a unit whose only checks are retired is not covered.
/// The pack still has an approved sentinel so this is a coverage fail, not
/// the empty-pool fail.
#[test]
fn retired_only_checks_do_not_count_as_covered() {
    tick();
    let f = plant_with_bank(&good_payload(), "retired");
    let o = run(&f.root);
    assert_fail_row(
        &o,
        "approved check_item_ids coverage 0/7 < 80% (sample thin: [\"u01-0\", \"u01-1\", \"u01-2\", \"u01-3\", \"u06-0\"])",
    );
    assert!(
        o.stdout
            .contains("FAIL: M01 unit missing ≥2 approved check_item_ids"),
        "M01 check floor must measure approved ids:\n{}",
        o.stdout
    );
    assert!(
        !o.stdout.contains("ok: approved check_item_ids coverage"),
        "retired-only units must not count as covered:\n{}",
        o.stdout
    );
    assert!(
        !o.stdout.contains("NONE are status='approved'"),
        "the plant keeps an approved sentinel so this names coverage, not an empty pool:\n{}",
        o.stdout
    );
    let _ = &f.dir;
}

/// Known-bad (a20t): file-set `bank_item_count` without the approved pair.
#[test]
fn file_set_count_unpaired_is_red() {
    tick();
    let mut payload = good_payload();
    payload
        .as_object_mut()
        .unwrap()
        .remove("approved_item_count");
    payload["bank_item_count"] = json!(804);
    let f = plant(&payload);
    let o = run(&f.root);
    assert_fail_row(
        &o,
        "units_index.json bank_item_count is a file-set count and is not paired with approved_item_count",
    );
    let _ = &f.dir;
}

/// Known-bad (a20t): cannot measure approved checks without the pack.
#[test]
fn missing_bank_pack_cannot_measure_approved() {
    tick();
    let f = plant(&good_payload());
    f.rm("web/data/bank_items_seed42.json");
    let o = run(&f.root);
    assert_fail_row(
        &o,
        "missing web/data/bank_items_seed42.json — cannot measure approved check_item_ids",
    );
    assert!(
        !o.stdout.contains("ok: approved check_item_ids coverage"),
        "coverage must not run without a pack:\n{}",
        o.stdout
    );
    let _ = &f.dir;
}

/// A status filter that removes the whole pool is ERROR, not 0/N content.
#[test]
fn empty_approved_pool_is_red() {
    tick();
    let f = plant(&good_payload());
    f.put(
        "web/data/bank_items_seed42.json",
        r#"[{"id":"x","status":"retired","module":1}]"#,
    );
    let o = run(&f.root);
    assert_fail_row(
        &o,
        "bank loaded 1 rows and NONE are status='approved' (vacuous approved-check floor is ERROR)",
    );
    let _ = &f.dir;
}

#[test]
fn compiled_lists_are_not_empty() {
    tick();
    assert!(!JS_ASSETS.is_empty());
    assert_eq!(JS_ASSETS.len(), 3);
    assert!(!M01_NEEDLES.is_empty());
    assert_eq!(M01_NEEDLES.len(), 3);
}

#[test]
fn this_suite_has_not_shrunk() {
    tick();
    let this = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/smoke_learn_v2.rs"),
    )
    .expect("this test file");
    let cases = this.matches("#[test]").count();
    assert!(
        cases >= EXPECTED_CASES,
        "case count fell to {cases}; EXPECTED_CASES is {EXPECTED_CASES}. \
         A suite that quietly shrank reports exactly like one that passed."
    );
    assert!(
        RAN.load(Ordering::SeqCst) > 0,
        "the verdict suite ran nothing"
    );
}
