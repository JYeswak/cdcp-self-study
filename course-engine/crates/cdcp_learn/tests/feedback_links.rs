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
    evaluate_topic_anchors, extract_heading_ids, run, slugify_heading, write_topic_anchors,
    BANK_JSON_REL, CONTENT_DIR_REL, KEYS_JSON_REL, LEARN_DIR_REL, RESULTS_JS_REL, SLUGS_JS_REL,
    TOPICS_TOML_REL, TOPIC_ANCHORS_JSON_REL,
};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Cases actually run, so "the suite ran" is itself checked.
static RAN: AtomicUsize = AtomicUsize::new(0);
static ROUND: AtomicUsize = AtomicUsize::new(0);

/// Raise when you add a `#[test]`. A DROP means a case was deleted.
const EXPECTED_CASES: usize = 31;

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

fn slugs_js(pairs: &[(&str, &str)]) -> String {
    let body: String = pairs
        .iter()
        .map(|(n, slug)| format!("  {n}: \"{slug}\",\n"))
        .collect();
    format!("export const MODULE_LEARN_SLUGS = Object.freeze({{\n{body}}});\n")
}

fn results_js() -> String {
    "import { MODULE_LEARN_SLUGS } from \"../../data/module_learn_slugs.js\";\n\
     export { MODULE_LEARN_SLUGS };\n\
     export function itemLearnHref() {}\n\
     const learn_href = true;\n\
     const copy = \"Review section in Learn\";\n\
     fetch(\"data/topic_anchors.json\");\n"
        .to_string()
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
    f.put(RESULTS_JS_REL, &results_js());
    f.put(SLUGS_JS_REL, &slugs_js(&[("1", "01-mission-critical")]));
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
    assert!(
        !o.stdout.contains("regen failed") && !o.stdout.contains("FileNotFoundError"),
        "the python import-failure note is deleted:\n{}",
        o.stdout
    );
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
fn missing_generated_slugs_file_is_red() {
    tick();
    let f = green();
    f.rm(SLUGS_JS_REL);
    let o = run(&f.root);
    assert_ne!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout.contains("missing web/data/module_learn_slugs.js"),
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
        SLUGS_JS_REL,
        "export const MODULE_LEARN_SLUGS = Object.freeze({});\n",
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
        SLUGS_JS_REL,
        &slugs_js(&[("1", "01-mission-critical"), ("2", "02-standards")]),
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
        SLUGS_JS_REL,
        &slugs_js(&[("1", "01-mission-critical"), ("14", "14-auxiliary")]),
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
    f.put(SLUGS_JS_REL, &slugs_js(&[("1", "01-mission-critical")]));
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
        "import { MODULE_LEARN_SLUGS } from \"../../data/module_learn_slugs.js\";\n\
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

/// Converted from the retired differential
/// `an_absent_builder_module_falls_back_to_the_existing_artifact`.
/// The smoke is a reader: python-absent + committed artifact is GREEN,
/// and there is no import-failure note.
#[test]
fn an_absent_python_builder_with_existing_artifact_is_green() {
    tick();
    let f = green();
    assert!(
        !f.at("scripts/build_learn.py").exists(),
        "this case is the python-absent tree"
    );
    let o = run(&f.root);
    assert_eq!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout.starts_with("PASS: smoke_feedback_links"),
        "{}",
        o.stdout
    );
    assert!(
        !o.stdout.contains("regen failed")
            && !o.stdout.contains("FileNotFoundError")
            && !o.stdout.contains("note: using existing")
            && !o.stdout.contains("scripts/build_learn.py"),
        "the import-failure note path is deleted:\n{}",
        o.stdout
    );
    let _ = &f.dir;
}

/// Converted from the retired differential
/// `an_absent_builder_module_with_no_artifact_is_red`.
#[test]
fn an_absent_python_builder_with_no_artifact_is_red() {
    tick();
    let f = green();
    f.rm(TOPIC_ANCHORS_JSON_REL);
    assert!(
        !f.at("scripts/build_learn.py").exists(),
        "this case is the python-absent tree"
    );
    let o = run(&f.root);
    assert_ne!(o.code, 0, "missing artifact must be RED:\n{}", o.stdout);
    assert!(o.stdout.contains("topic_anchors missing"), "{}", o.stdout);
    assert!(
        !o.stdout.contains("scripts/build_learn.py") && !o.stdout.contains("FileNotFoundError"),
        "the missing-artifact path must not stat or name the python builder:\n{}",
        o.stdout
    );
    let _ = &f.dir;
}

/// The rust builder produces the topic map without `scripts/build_learn.py`.
#[test]
fn rust_builder_maps_label_to_heading_without_python() {
    tick();
    let f = green();
    f.rm(TOPIC_ANCHORS_JSON_REL);
    assert!(!f.at("scripts/build_learn.py").exists());
    let o = evaluate_topic_anchors(&f.root);
    assert_eq!(o.code, 0, "{}", o.stdout);
    let (_path, body) = o.artifact.expect("green builder carries bytes");
    let v: serde_json::Value = serde_json::from_str(&body).expect("builder json");
    assert_eq!(v["generated_by"], "cdcp_learn");
    assert_eq!(
        v["topics"]["m01-dc-types"]["anchor"],
        "types-of-data-centres"
    );
    assert_eq!(v["topics_with_anchor"], 1);
    // The smoke remains a reader: no committed file is still RED.
    let smoke = run(&f.root);
    assert_ne!(smoke.code, 0, "{}", smoke.stdout);
    assert!(
        smoke.stdout.contains("topic_anchors missing"),
        "{}",
        smoke.stdout
    );
    let _ = &f.dir;
}

/// Should-fail: empty topics.toml is RED and writes nothing.
#[test]
fn rust_builder_empty_topics_is_red_and_writes_nothing() {
    tick();
    let f = green();
    f.put(TOPICS_TOML_REL, "schema_version = 1\n");
    let before = std::fs::read(f.at(TOPIC_ANCHORS_JSON_REL)).expect("fixture artifact");
    let o = write_topic_anchors(&f.root).expect("write call");
    assert_ne!(o.code, 0, "empty topics must be RED:\n{}", o.stdout);
    assert!(o.artifact.is_none(), "RED must not carry an artifact");
    assert!(o.stdout.contains("zero topics"), "{}", o.stdout);
    let after = std::fs::read(f.at(TOPIC_ANCHORS_JSON_REL)).expect("artifact after RED write");
    assert_eq!(before, after, "a RED compile wrote topic_anchors.json");
    let _ = &f.dir;
}

/// Live committed anchors (still python-generated) must agree with the rust
/// builder's topic→anchor map. Disagreement means the port is not ready to
/// drop the python writer.
#[test]
fn rust_builder_live_tree_agrees_with_committed_anchors() {
    tick();
    let root = engine_root();
    let o = evaluate_topic_anchors(&root);
    assert_eq!(o.code, 0, "{}", o.stdout);
    let (_path, body) = o.artifact.expect("green builder carries bytes");
    let built: serde_json::Value = serde_json::from_str(&body).expect("built json");
    let committed: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join(TOPIC_ANCHORS_JSON_REL))
            .expect("committed topic_anchors.json"),
    )
    .expect("committed json");
    let built_topics = built["topics"].as_object().expect("built topics");
    let committed_topics = committed["topics"].as_object().expect("committed topics");
    assert_eq!(
        built_topics.len(),
        committed_topics.len(),
        "topic set size drifted (built={} committed={})",
        built_topics.len(),
        committed_topics.len()
    );
    let mut diffs = Vec::new();
    for (tid, row) in committed_topics {
        let Some(got) = built_topics.get(tid) else {
            diffs.push(format!("{tid}: rust dropped the topic"));
            continue;
        };
        if got.get("anchor") != row.get("anchor") {
            diffs.push(format!(
                "{tid}: anchor rust={:?} committed={:?}",
                got.get("anchor"),
                row.get("anchor")
            ));
        }
        if got.get("slug") != row.get("slug") {
            diffs.push(format!(
                "{tid}: slug rust={:?} committed={:?}",
                got.get("slug"),
                row.get("slug")
            ));
        }
    }
    for tid in built_topics.keys() {
        if !committed_topics.contains_key(tid) {
            diffs.push(format!("{tid}: rust added a topic"));
        }
    }
    assert!(
        diffs.is_empty(),
        "rust builder disagrees with committed topic_anchors.json ({} diffs):\n{}",
        diffs.len(),
        diffs.join("\n")
    );
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
