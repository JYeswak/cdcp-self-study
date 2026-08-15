//! Artifact schema + approved-only + known-bad retirement.
//!
//! The product contract after bd-engine-not-gate-ar39.2: declared keys, correct
//! counts, check_item_ids approved-only, bytes stable across runs. A replica of
//! `json.dumps(indent=2)` is not a requirement.

use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

const PACK: &str = "web/data/bank_items_seed42.json";

static ROUND: AtomicUsize = AtomicUsize::new(0);

fn engine_root() -> PathBuf {
    cdcp_learn::resolve_engine_root(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
}

fn copy_tree(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for e in std::fs::read_dir(src)
        .unwrap_or_else(|err| panic!("read {}: {err}", src.display()))
        .flatten()
    {
        let from = e.path();
        let to = dst.join(e.file_name());
        if from.is_dir() {
            copy_tree(&from, &to);
        } else {
            std::fs::copy(&from, &to).unwrap();
        }
    }
}

fn seed_live(dst: &Path) {
    let root = engine_root();
    copy_tree(
        &root.join("web/content/modules"),
        &dst.join("web/content/modules"),
    );
    for rel in ["knowledge/topics.toml", "web/data/modules_index.json", PACK] {
        let p = dst.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::copy(root.join(rel), &p).unwrap();
    }
}

fn live_compile() -> (tempfile::TempDir, cdcp_learn::BuildOutcome) {
    let td = tempfile::tempdir().unwrap();
    let n = ROUND.fetch_add(1, Ordering::SeqCst);
    let root = td.path().join(format!("r{n}"));
    seed_live(&root);
    let outcome = cdcp_learn::units::write_units(&root).expect("write_units");
    assert_eq!(outcome.code, 0, "{}", outcome.stdout);
    (td, outcome)
}

fn parse_artifact(outcome: &cdcp_learn::BuildOutcome) -> Value {
    let body = &outcome.artifact.as_ref().expect("artifact").1;
    serde_json::from_str(body).expect("units_index is JSON")
}

fn collect_check_ids(v: &Value) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(units) = v.get("units").and_then(|u| u.as_array()) {
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

#[test]
fn units_index_carries_the_declared_keys() {
    let (_td, outcome) = live_compile();
    let v = parse_artifact(&outcome);
    let obj = v.as_object().expect("object");
    for key in cdcp_learn::UNITS_INDEX_KEYS {
        assert!(
            obj.contains_key(*key),
            "units_index missing declared key {key}; have {:?}",
            obj.keys().collect::<Vec<_>>()
        );
    }
    let units = v["units"].as_array().expect("units array");
    assert!(!units.is_empty(), "zero units is an ERROR, not a schema");
    for (i, unit) in units.iter().enumerate() {
        let u = unit.as_object().expect("unit object");
        for key in cdcp_learn::UNIT_ROW_KEYS {
            assert!(
                u.contains_key(*key),
                "units[{i}] missing declared key {key}"
            );
        }
    }
    assert_eq!(v["generated_by"], cdcp_learn::GENERATED_BY);
}

#[test]
fn unit_count_and_module_count_match_the_collections() {
    let (_td, outcome) = live_compile();
    let v = parse_artifact(&outcome);
    let units = v["units"].as_array().unwrap();
    let by = v["by_module"].as_object().unwrap();
    assert_eq!(
        v["unit_count"].as_u64().unwrap() as usize,
        units.len(),
        "unit_count disagrees with units.len()"
    );
    assert_eq!(
        v["module_count"].as_u64().unwrap() as usize,
        by.len(),
        "module_count disagrees with by_module.len()"
    );
    assert!(
        !units.is_empty() && !by.is_empty(),
        "zero modules or zero units is an ERROR"
    );
}

#[test]
fn check_item_ids_are_approved_only_zero_retired() {
    let root = engine_root();
    let bank = cdcp_learn::units::load_bank(&root).expect("load bank");
    assert!(!bank.is_empty(), "empty bank is an ERROR, not a pass");
    let withheld: BTreeSet<&str> = bank
        .iter()
        .filter(|it| !it.is_approved())
        .map(|it| it.id.as_str())
        .collect();
    assert!(
        !withheld.is_empty(),
        "the live pack carries NO non-approved row, so this case proves nothing"
    );

    let (_td, outcome) = live_compile();
    let v = parse_artifact(&outcome);
    let ids = collect_check_ids(&v);
    assert!(!ids.is_empty(), "no check_item_ids drawn");
    let leaked: Vec<&str> = ids
        .iter()
        .filter(|id| withheld.contains(id.as_str()))
        .map(String::as_str)
        .collect();
    assert!(
        leaked.is_empty(),
        "{} retired/withheld item(s) reached unit checks: {leaked:?}",
        leaked.len()
    );
}

#[test]
fn bytes_are_stable_across_runs() {
    let (_td1, a) = live_compile();
    let (_td2, b) = live_compile();
    assert_eq!(
        a.artifact.as_ref().map(|(_, body)| body.as_str()),
        b.artifact.as_ref().map(|(_, body)| body.as_str()),
        "two compiles of the same inputs must emit identical bytes"
    );
}

#[test]
fn known_bad_retiring_one_more_item_leaves_the_unit_checks() {
    // KNOWN-BAD: retire one more item, regenerate, assert it leaves the unit
    // checks. An absence assertion on a compiler that draws nothing would pass
    // vacuously — the plant is an id the compiler actually drew.
    let td = tempfile::tempdir().unwrap();
    let n = ROUND.fetch_add(1, Ordering::SeqCst);
    let root = td.path().join(format!("r{n}"));
    seed_live(&root);

    let green = cdcp_learn::units::write_units(&root).expect("baseline");
    assert_eq!(green.code, 0, "{}", green.stdout);
    let green_v = parse_artifact(&green);
    let drawn = collect_check_ids(&green_v);
    let plant = drawn
        .first()
        .cloned()
        .expect("baseline must draw at least one check id");

    // Flip that id to retired in the pack and recompile.
    let pack_path = root.join(PACK);
    let raw = std::fs::read_to_string(&pack_path).unwrap();
    let mut pack: Value = serde_json::from_str(&raw).unwrap();
    let items = pack.as_array_mut().expect("pack is an array");
    let mut flipped = 0usize;
    for it in items.iter_mut() {
        if it.get("id").and_then(|v| v.as_str()) == Some(plant.as_str()) {
            it["status"] = Value::String("retired".into());
            flipped += 1;
        }
    }
    assert_eq!(flipped, 1, "plant {plant} must exist exactly once in the pack");
    std::fs::write(&pack_path, serde_json::to_string(&pack).unwrap()).unwrap();

    let red = cdcp_learn::units::write_units(&root).expect("after retire");
    assert_eq!(
        red.code, 0,
        "retiring one drawn item must not break the floor: {}",
        red.stdout
    );
    let after = parse_artifact(&red);
    let after_ids = collect_check_ids(&after);
    assert!(
        !after_ids.iter().any(|id| id == &plant),
        "retired item {plant} must leave the unit checks; still present in {after_ids:?}"
    );
    assert!(
        !after_ids.is_empty(),
        "the compiler must BACKFILL, not emit an empty check list"
    );
}

#[test]
fn glossary_carries_the_declared_keys_and_counts() {
    let td = tempfile::tempdir().unwrap();
    let n = ROUND.fetch_add(1, Ordering::SeqCst);
    let root = td.path().join(format!("g{n}"));
    let src = engine_root().join("web/content/reference/GLOSSARY.md");
    let dst = root.join("web/content/reference/GLOSSARY.md");
    std::fs::create_dir_all(dst.parent().unwrap()).unwrap();
    std::fs::copy(&src, &dst).unwrap();

    let outcome = cdcp_learn::glossary::write_glossary(&root).expect("write_glossary");
    assert_eq!(outcome.code, 0, "{}", outcome.stdout);
    let body = &outcome.artifact.as_ref().expect("artifact").1;
    let v: Value = serde_json::from_str(body).expect("glossary is JSON");
    let obj = v.as_object().unwrap();
    for key in cdcp_learn::GLOSSARY_KEYS {
        assert!(obj.contains_key(*key), "glossary missing {key}");
    }
    let terms = v["terms"].as_object().unwrap();
    assert_eq!(
        v["term_count"].as_u64().unwrap() as usize,
        terms.len(),
        "term_count disagrees with terms.len()"
    );
    assert!(
        terms.len() >= cdcp_learn::glossary::MIN_TERMS,
        "live glossary is below the term floor"
    );
    assert_eq!(v["generated_by"], cdcp_learn::GENERATED_BY);
}

#[test]
fn zero_modules_or_zero_units_is_an_error() {
    let td = tempfile::tempdir().unwrap();
    let root = td.path().join("empty");
    std::fs::create_dir_all(root.join("web/content/modules")).unwrap();
    std::fs::write(
        {
            let p = root.join("knowledge/topics.toml");
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            p
        },
        "[[topic]]\nid = \"t\"\ndomain = \"01-x\"\nlabel = \"T\"\n",
    )
    .unwrap();
    std::fs::write(
        {
            let p = root.join("web/data/modules_index.json");
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            p
        },
        r#"{"modules":[{"id":"01-mission-critical","empty":false}]}"#,
    )
    .unwrap();
    let outcome = cdcp_learn::units::evaluate(&root).expect("evaluate");
    assert_ne!(outcome.code, 0, "zero units must be RED");
    assert!(outcome.artifact.is_none(), "RED must write nothing");
    assert!(
        outcome.stdout.contains("zero modules discovered")
            || outcome.stdout.contains("zero units discovered"),
        "{}",
        outcome.stdout
    );
}
