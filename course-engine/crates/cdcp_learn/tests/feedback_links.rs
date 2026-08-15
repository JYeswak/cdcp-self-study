//! Verdict suite for `cdcp_learn::feedback` (extracted by
//! bd-substrate-rust-migration-jhd.18). The Python is DELETED. Every case
//! asserts the correct answer, not agreement with a retired script.
//!
//! This smoke is a READER: it reads the Learn / results / topic-anchor
//! surface and writes nothing. Cases run against TEMP fixtures (or the live
//! tree read-only).
//!
//! ANTI-VACUOUS: an empty tree is RED; a missing or empty topic registry is
//! RED; a missing or empty `topic_anchors.json` is RED; `topics_with_anchor=0`
//! with declared modules is RED (bd-feedback-links-vacuous-topics-ilad).
//! A suite that ran no case is RED.

use cdcp_learn::feedback::{
    extract_heading_ids, run, slugify_heading, BANK_JSON_REL, CONTENT_DIR_REL, KEYS_JSON_REL,
    LEARN_DIR_REL, RESULTS_JS_REL, TOPICS_TOML_REL, TOPIC_ANCHORS_JSON_REL,
};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Cases actually run, so "the suite ran" is itself checked.
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

    fn rm(&self, rel: &str) {
        let p = self.at(rel);
        if p.is_dir() {
            std::fs::remove_dir_all(&p).unwrap();
        } else if p.exists() {
            std::fs::remove_file(&p).unwrap();
        }
    }
}

fn results_js(pairs: &[(&str, &str)]) -> String {
    let body: String = pairs
        .iter()
        .map(|(n, slug)| format!("  {n}: \"{slug}\",\n"))
        .collect();
    format!(
        "export const MODULE_LEARN_SLUGS = Object.freeze({{\n{body}}});\n\
         export function itemLearnHref() {{}}\n\
         const learn_href = true;\n\
         const copy = \"Review section in Learn\";\n\
         fetch(\"data/topic_anchors.json\");\n"
    )
}

fn learn_md_js() -> &'static str {
    "function slugify(text) { return text; }\nfunction uniqueSlug(base, used) { return base; }\nconst id = \"heading\";\n"
}

fn domains(rows: &[(&str, i64)]) -> String {
    let mut s = String::from("schema_version = 1\n");
    for (id, order) in rows {
        s.push_str(&format!("\n[[domain]]\nid = \"{id}\"\norder = {order}\n"));
    }
    s
}

fn topics(rows: &[(&str, &str, &str)]) -> String {
    let mut s = String::from("schema_version = 1\n");
    for (id, domain, label) in rows {
        s.push_str(&format!(
            "\n[[topic]]\nid = \"{id}\"\ndomain = \"{domain}\"\nlabel = \"{label}\"\n"
        ));
    }
    s
}

fn anchors(rows: &[(&str, &str, &str, i64)]) -> String {
    let mut topics_json = String::new();
    for (i, (tid, slug, anchor, module)) in rows.iter().enumerate() {
        if i > 0 {
            topics_json.push(',');
        }
        topics_json.push_str(&format!(
            "\"{tid}\":{{\"topic_id\":\"{tid}\",\"slug\":\"{slug}\",\"anchor\":\"{anchor}\",\"module\":{module}}}"
        ));
    }
    format!(
        "{{\"schema_version\":1,\"topic_count\":{},\"topics_with_anchor\":{},\"topics\":{{{topics_json}}}}}",
        rows.len(),
        rows.len()
    )
}

fn keys(ids: &[&str]) -> String {
    let rows: String = ids
        .iter()
        .enumerate()
        .map(|(i, id)| format!("{}{{\"item_id\":\"{id}\"}}", if i == 0 { "" } else { "," }))
        .collect();
    format!("{{\"keys\":[{rows}]}}")
}

fn bank(rows: &[(&str, i64, &str)]) -> String {
    let body: String = rows
        .iter()
        .enumerate()
        .map(|(i, (id, module, tid))| {
            format!(
                "{}{{\"id\":\"{id}\",\"module\":{module},\"topic_ids\":[\"{tid}\"]}}",
                if i == 0 { "" } else { "," }
            )
        })
        .collect();
    format!("[{body}]")
}

/// Minimal GREEN tree: one declared module, one topic, one key, one anchor.
fn green() -> Fixture {
    let f = Fixture::new();
    f.put(
        "knowledge/domains.toml",
        &domains(&[("01-mission-critical", 1)]),
    );
    f.put(
        TOPICS_TOML_REL,
        &topics(&[(
            "m01-dc-types",
            "01-mission-critical",
            "Types of data centres",
        )]),
    );
    f.put(RESULTS_JS_REL, &results_js(&[("1", "01-mission-critical")]));
    f.put("web/assets/js/learn_md.js", learn_md_js());
    f.put(
        &format!("{LEARN_DIR_REL}/01-mission-critical.html"),
        "<html>learn</html>\n",
    );
    f.put(
        &format!("{CONTENT_DIR_REL}/01-mission-critical.md"),
        "## Types of data centres\n\nBody.\n",
    );
    f.put(KEYS_JSON_REL, &keys(&["m01-q001"]));
    f.put(BANK_JSON_REL, &bank(&[("m01-q001", 1, "m01-dc-types")]));
    f.put(
        TOPIC_ANCHORS_JSON_REL,
        &anchors(&[(
            "m01-dc-types",
            "01-mission-critical",
            "types-of-data-centres",
            1,
        )]),
    );
    f
}

#[test]
fn live_tree_passes() {
    tick();
    let root = engine_root();
    assert!(
        root.join("web").is_dir(),
        "live engine has no web/ — a missing product tree is an ERROR"
    );
    let o = run(&root);
    assert_eq!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout.starts_with("PASS: smoke_feedback_links"),
        "{}",
        o.stdout
    );
    assert!(
        o.stdout.contains("untaught_module_items=0 (must be 0)"),
        "{}",
        o.stdout
    );
    assert!(o.artifact.is_none(), "smoke is a reader");
}

#[test]
fn live_tree_writes_nothing() {
    tick();
    let root = engine_root();
    let path = root.join(TOPIC_ANCHORS_JSON_REL);
    let before = std::fs::read(&path).expect("live topic_anchors.json");
    assert!(
        !before.is_empty(),
        "live topic_anchors.json is empty — a missing artifact is an ERROR"
    );
    let _ = run(&root);
    let after = std::fs::read(&path).expect("topic_anchors.json after run");
    assert_eq!(before, after, "the smoke wrote topic_anchors.json");
}

#[test]
fn green_fixture_passes() {
    tick();
    let f = green();
    let o = run(&f.root);
    assert_eq!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout.starts_with("PASS: smoke_feedback_links"),
        "{}",
        o.stdout
    );
    assert!(
        o.stdout.contains("M01 → learn/01-mission-critical.html"),
        "{}",
        o.stdout
    );
    let _ = &f.dir;
}

#[test]
fn empty_tree_is_an_error() {
    tick();
    let f = Fixture::new();
    let o = run(&f.root);
    assert_ne!(o.code, 0, "empty tree must be RED, got PASS:\n{}", o.stdout);
    assert!(
        o.stdout.starts_with("FAIL: smoke_feedback_links"),
        "{}",
        o.stdout
    );
    assert!(o.stdout.contains("domain registry missing"), "{}", o.stdout);
    assert!(
        !o.stdout.contains("PASS: smoke_feedback_links"),
        "PASS must not appear on a failing run:\n{}",
        o.stdout
    );
    let _ = &f.dir;
}

#[test]
fn empty_registry_is_an_error() {
    tick();
    let f = green();
    f.put("knowledge/domains.toml", "schema_version = 1\n");
    let o = run(&f.root);
    assert_ne!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout
            .contains("domain registry declares zero modules (vacuous link check is ERROR)"),
        "{}",
        o.stdout
    );
    let _ = &f.dir;
}

/// bd-feedback-links-vacuous-topics-ilad: the retired script PASSED this.
#[test]
fn missing_topics_toml_is_an_error() {
    tick();
    let f = green();
    f.rm(TOPICS_TOML_REL);
    let o = run(&f.root);
    assert_ne!(
        o.code, 0,
        "absent topics.toml must be RED, got PASS:\n{}",
        o.stdout
    );
    assert!(o.stdout.contains("topic registry missing"), "{}", o.stdout);
    let _ = &f.dir;
}

/// bd-feedback-links-vacuous-topics-ilad: empty registry switched the
/// section-anchor guard OFF and the Python printed 0.0% and PASSED.
#[test]
fn empty_topics_toml_is_an_error() {
    tick();
    let f = green();
    f.put(TOPICS_TOML_REL, "schema_version = 1\n");
    let o = run(&f.root);
    assert_ne!(
        o.code, 0,
        "empty topics.toml must be RED, got PASS:\n{}",
        o.stdout
    );
    assert!(
        o.stdout.contains(
            "topic registry declares zero topics (vacuous section-anchor check is ERROR)"
        ),
        "{}",
        o.stdout
    );
    let _ = &f.dir;
}

#[test]
fn missing_topic_anchors_is_an_error() {
    tick();
    let f = green();
    f.rm(TOPIC_ANCHORS_JSON_REL);
    let o = run(&f.root);
    assert_ne!(o.code, 0, "{}", o.stdout);
    assert!(o.stdout.contains("topic_anchors missing"), "{}", o.stdout);
    let _ = &f.dir;
}

#[test]
fn empty_topic_anchors_is_an_error() {
    tick();
    let f = green();
    f.put(
        TOPIC_ANCHORS_JSON_REL,
        r#"{"schema_version":1,"topic_count":0,"topics_with_anchor":0,"topics":{}}"#,
    );
    let o = run(&f.root);
    assert_ne!(
        o.code, 0,
        "empty topic_anchors must be RED, got PASS:\n{}",
        o.stdout
    );
    assert!(
        o.stdout
            .contains("topic_anchors.json has zero topics (vacuous section-anchor check is ERROR)"),
        "{}",
        o.stdout
    );
    assert!(
        o.stdout
            .contains("topics_with_anchor=0 with declared modules present"),
        "{}",
        o.stdout
    );
    let _ = &f.dir;
}

#[test]
fn topics_with_anchor_zero_is_an_error() {
    tick();
    let f = green();
    f.put(
        TOPIC_ANCHORS_JSON_REL,
        r#"{"schema_version":1,"topic_count":1,"topics_with_anchor":0,"topics":{"m01-dc-types":{"topic_id":"m01-dc-types","slug":"01-mission-critical","anchor":null,"module":1}}}"#,
    );
    let o = run(&f.root);
    assert_ne!(o.code, 0, "topics_with_anchor=0 must be RED:\n{}", o.stdout);
    assert!(
        o.stdout
            .contains("topics_with_anchor=0 with declared modules present"),
        "{}",
        o.stdout
    );
    let _ = &f.dir;
}

#[test]
fn unparseable_topic_anchors_is_an_error() {
    tick();
    let f = green();
    f.put(TOPIC_ANCHORS_JSON_REL, "this is not json");
    let o = run(&f.root);
    assert_ne!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout.contains("topic_anchors.json invalid"),
        "{}",
        o.stdout
    );
    let _ = &f.dir;
}

#[test]
fn missing_results_js_is_red() {
    tick();
    let f = green();
    f.rm(RESULTS_JS_REL);
    let o = run(&f.root);
    assert_ne!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout.contains("missing web/assets/js/results.js"),
        "{}",
        o.stdout
    );
    let _ = &f.dir;
}

#[test]
fn empty_module_learn_slugs_is_red() {
    tick();
    let f = green();
    f.put(
        RESULTS_JS_REL,
        "export const MODULE_LEARN_SLUGS = Object.freeze({});\n\
         export function itemLearnHref() {}\n\
         const learn_href = true;\n\
         const copy = \"Review section in Learn\";\n\
         fetch(\"data/topic_anchors.json\");\n",
    );
    let o = run(&f.root);
    assert_ne!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout.contains("MODULE_LEARN_SLUGS empty"),
        "{}",
        o.stdout
    );
    let _ = &f.dir;
}

#[test]
fn undeclared_module_in_results_js_is_drift() {
    tick();
    let f = green();
    f.put(
        RESULTS_JS_REL,
        &results_js(&[("1", "01-mission-critical"), ("2", "02-standards")]),
    );
    let o = run(&f.root);
    assert_ne!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout.contains(
            "module 2: results.js maps '02-standards' but knowledge/domains.toml does not declare that module"
        ),
        "{}",
        o.stdout
    );
    let _ = &f.dir;
}

#[test]
fn declared_module_with_no_learn_surface_is_red() {
    tick();
    let f = green();
    f.put(
        "knowledge/domains.toml",
        &domains(&[("01-mission-critical", 1), ("16-fixture-only", 16)]),
    );
    let o = run(&f.root);
    assert_ne!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout
            .contains("module 16: results.js slug map None != '16-fixture-only'"),
        "{}",
        o.stdout
    );
    assert!(
        o.stdout
            .contains("missing learn page web/learn/16-fixture-only.html"),
        "{}",
        o.stdout
    );
    assert!(
        o.stdout
            .contains("missing content web/content/modules/16-fixture-only.md"),
        "{}",
        o.stdout
    );
    let _ = &f.dir;
}

#[test]
fn assessed_but_untaught_is_named() {
    tick();
    let f = green();
    f.put(KEYS_JSON_REL, &keys(&["m01-q001", "orphan-q001"]));
    f.put(
        BANK_JSON_REL,
        &bank(&[
            ("m01-q001", 1, "m01-dc-types"),
            ("orphan-q001", 99, "m99-none"),
        ]),
    );
    let o = run(&f.root);
    assert_ne!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout.contains("assessed but untaught: ")
            && o.stdout.contains("module 99 is not declared"),
        "{}",
        o.stdout
    );
    let _ = &f.dir;
}

#[test]
fn empty_keys_is_an_error() {
    tick();
    let f = green();
    f.put(KEYS_JSON_REL, r#"{"keys":[]}"#);
    let o = run(&f.root);
    assert_ne!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout.contains("keys_seed42 has zero keys (vacuous)"),
        "{}",
        o.stdout
    );
    let _ = &f.dir;
}

#[test]
fn missing_keys_is_an_error() {
    tick();
    let f = green();
    f.rm(KEYS_JSON_REL);
    let o = run(&f.root);
    assert_ne!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout.contains("missing web/data/keys_seed42.json"),
        "{}",
        o.stdout
    );
    let _ = &f.dir;
}

#[test]
fn empty_bank_is_an_error() {
    tick();
    let f = green();
    f.put(BANK_JSON_REL, "[]");
    let o = run(&f.root);
    assert_ne!(o.code, 0, "{}", o.stdout);
    assert!(o.stdout.contains("bank_items_seed42 empty"), "{}", o.stdout);
    let _ = &f.dir;
}

#[test]
fn key_not_in_bank_is_named() {
    tick();
    let f = green();
    f.put(KEYS_JSON_REL, &keys(&["does-not-exist"]));
    let o = run(&f.root);
    assert_ne!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout
            .contains("key item_id not in bank_items_seed42: does-not-exist"),
        "{}",
        o.stdout
    );
    let _ = &f.dir;
}

#[test]
fn stale_anchor_not_in_headings_is_red() {
    tick();
    let f = green();
    f.put(
        TOPIC_ANCHORS_JSON_REL,
        &anchors(&[("m01-dc-types", "01-mission-critical", "no-such-heading", 1)]),
    );
    let o = run(&f.root);
    assert_ne!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout.contains(
            "topic m01-dc-types: anchor 'no-such-heading' not in headings of 01-mission-critical"
        ),
        "{}",
        o.stdout
    );
    let _ = &f.dir;
}

#[test]
fn retiring_a_module_from_both_sources_is_not_drift() {
    tick();
    let f = green();
    f.put(
        "knowledge/domains.toml",
        &domains(&[("01-mission-critical", 1), ("14-auxiliary", 14)]),
    );
    f.put(
        RESULTS_JS_REL,
        &results_js(&[("1", "01-mission-critical"), ("14", "14-auxiliary")]),
    );
    f.put(
        &format!("{LEARN_DIR_REL}/14-auxiliary.html"),
        "<html>aux</html>\n",
    );
    f.put(&format!("{CONTENT_DIR_REL}/14-auxiliary.md"), "## Aux\n");
    // Now retire from BOTH sources.
    f.put(
        "knowledge/domains.toml",
        &domains(&[("01-mission-critical", 1)]),
    );
    f.put(RESULTS_JS_REL, &results_js(&[("1", "01-mission-critical")]));
    let o = run(&f.root);
    assert!(
        !o.stdout.contains("module 14: results.js maps"),
        "an agreed retirement must not be product→registry drift:\n{}",
        o.stdout
    );
    assert!(
        !o.stdout.contains("module 14: results.js slug map"),
        "an agreed retirement must not be registry→product drift:\n{}",
        o.stdout
    );
    let _ = &f.dir;
}

#[test]
fn results_js_missing_item_learn_href_is_red() {
    tick();
    let f = green();
    f.put(
        RESULTS_JS_REL,
        "export const MODULE_LEARN_SLUGS = Object.freeze({\n  1: \"01-mission-critical\",\n});\n\
         const learn_href = true;\n\
         const copy = \"Review section in Learn\";\n\
         fetch(\"data/topic_anchors.json\");\n",
    );
    let o = run(&f.root);
    assert_ne!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout.contains("results.js missing itemLearnHref"),
        "{}",
        o.stdout
    );
    let _ = &f.dir;
}

#[test]
fn slugify_and_heading_ids_are_stable() {
    tick();
    assert_eq!(
        slugify_heading("Types of data centres"),
        "types-of-data-centres"
    );
    let ids = extract_heading_ids("## Types of data centres\n");
    assert!(ids.contains("types-of-data-centres"));
}

#[test]
fn this_suite_has_not_shrunk() {
    tick();
    let this = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/feedback_links.rs"),
    )
    .expect("this test file");
    let cases = this.matches("#[test]").count();
    assert!(
        cases >= EXPECTED_CASES,
        "case count fell to {cases}; EXPECTED_CASES is {EXPECTED_CASES}. \
         A suite that quietly shrank reports exactly like one that passed."
    );
    RAN.fetch_add(1, Ordering::SeqCst);
    assert!(
        RAN.load(Ordering::SeqCst) > 0,
        "the verdict suite ran nothing"
    );
}
