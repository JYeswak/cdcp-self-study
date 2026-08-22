//! bd-smvb — contracts that prose asserted after the code moved.
//!
//! Two decisions, both gated:
//!
//! 1. Hub copy names the complete Learn catalog and its split: 15 pages made
//!    up of 14 public facility domains plus one operations module. A changed
//!    domain registry or vanished ops row should make this contract RED rather
//!    than silently changing the learner-facing count.
//! 2. `web/data/coverage.json` is DELETED. It had no product consumer and
//!    drifted. Restoring it without a reader turns this RED.
//!
//! Anti-vacuous: an empty domain registry is ERROR; a suite that ran no
//! case is ERROR.

use std::path::{Path, PathBuf};

/// Raise when you add a test function. A DROP means a case was deleted.
const EXPECTED_CASES: usize = 5;

fn engine_root() -> PathBuf {
    cdcp_learn::resolve_engine_root(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
}

fn expected_card(epi_n: usize, ops_n: usize) -> String {
    format!(
        "{} Learn modules ({} public facility domains + {} ops module)",
        epi_n + ops_n,
        epi_n,
        ops_n
    )
}

/// `(epi_count, ops_count)` from a domains.toml body.
///
/// `exam_weight_unknown = true` is the ops-expansion mark. Missing or
/// false is an EPI syllabus domain. Zero declared rows is ERROR.
fn epi_ops_counts(text: &str) -> Result<(usize, usize), String> {
    let parsed: toml::Value = text
        .parse()
        .map_err(|e| format!("domains.toml parse error: {e}"))?;
    let rows = match parsed.get("domain") {
        Some(toml::Value::Array(rows)) => rows.as_slice(),
        None => return Err("domain registry declares zero modules".into()),
        Some(_) => return Err("domains.toml `domain` is not an array of tables".into()),
    };
    if rows.is_empty() {
        return Err("domain registry declares zero modules".into());
    }
    let mut epi = 0usize;
    let mut ops = 0usize;
    for row in rows {
        let Some(table) = row.as_table() else {
            return Err("domains.toml: [[domain]] row is not a table".into());
        };
        let unknown = table
            .get("exam_weight_unknown")
            .and_then(toml::Value::as_bool)
            .unwrap_or(false);
        if unknown {
            ops += 1;
        } else {
            epi += 1;
        }
    }
    if epi == 0 {
        return Err("zero EPI domains (no row without exam_weight_unknown)".into());
    }
    if ops == 0 {
        return Err("zero ops-expansion domains (no exam_weight_unknown row)".into());
    }
    Ok((epi, ops))
}

#[test]
fn empty_registry_is_an_error() {
    let err = epi_ops_counts("schema_version = 1\n").expect_err("empty must be ERROR");
    assert!(
        err.contains("zero modules"),
        "empty registry must name the vacuous case: {err}"
    );
}

#[test]
fn a_wrong_numeral_does_not_match_the_derived_card() {
    let (epi, ops) = epi_ops_counts(
        r#"
[[domain]]
id = "01-a"
order = 1
[[domain]]
id = "15-ops"
order = 15
exam_weight_unknown = true
"#,
    )
    .expect("planted pair");
    assert_eq!((epi, ops), (1, 1));
    let want = expected_card(epi, ops);
    assert!(
        !"14 Learn modules (13 public facility domains + 1 ops module)".contains(&want),
        "a stale numeral must not satisfy the derived card {want:?}"
    );
    assert_eq!(want, "2 Learn modules (1 public facility domains + 1 ops module)");
}

#[test]
fn hub_card_names_the_full_catalog_split() {
    let root = engine_root();
    let domains = std::fs::read_to_string(root.join("knowledge/domains.toml"))
        .expect("knowledge/domains.toml readable");
    let (epi, ops) = epi_ops_counts(&domains).expect("live registry must parse");
    assert!(epi > 0 && ops > 0, "vacuous split: epi={epi} ops={ops}");

    let html = std::fs::read_to_string(root.join("web/index.html")).expect("web/index.html");
    let want = expected_card(epi, ops);
    assert!(
        html.contains(&want),
        "hub copy must carry the derived catalog split (EPI={epi}, ops={ops}); \
         missing {want:?} in web/index.html"
    );
    assert!(
        html.contains("bd-smvb"),
        "the catalog-split comment must stay next to the numeral so the next \
         editor does not silently change the learner-facing count"
    );
}

#[test]
fn coverage_json_is_not_a_shipped_unread_ledger() {
    let p = engine_root().join("web/data/coverage.json");
    assert!(
        !p.exists(),
        "web/data/coverage.json was deleted (bd-smvb): it had no product \
         consumer and drifted. Do not restore it without a reader. The live \
         report is `cdcp_gate verify-coverage` stdout."
    );
}

#[test]
fn the_suite_declares_its_cases() {
    let src = include_str!("stale_contract_prose.rs");
    let n = src.lines().filter(|l| l.trim() == "#[test]").count();
    assert_eq!(
        n, EXPECTED_CASES,
        "stale_contract_prose declares {n} test functions, expected {EXPECTED_CASES} — a deleted case is not a pass"
    );
}
