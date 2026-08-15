//! Verdict suite for `cdcp_learn::build` (bd-substrate-rust-migration-jhd.28).
//!
//! EXTRACT-THEN-DELETE: this is NOT a differential against
//! `scripts/build_learn.py`. The Python is deleted in the same commit.
//! Every case asserts WHAT THE CORRECT ANSWER IS against the Rust alone.
//!
//! WRITE-AFTER-VERDICT: a RED run writes nothing and unlinks nothing.
//! ANTI-VACUOUS: missing/empty domains, empty notes without
//! exam_weight_unknown, a missing notes file, or an empty content sweep
//! is RED.

use std::path::{Path, PathBuf};

fn engine_root() -> PathBuf {
    cdcp_learn::resolve_engine_root(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
}

struct Tree {
    _dir: tempfile::TempDir,
    root: PathBuf,
    corpus: PathBuf,
}

impl Tree {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let base = dir.path().canonicalize().expect("canonicalize");
        let root = base.join("study/engine");
        let corpus = base.join("study/modules");
        for d in [
            "knowledge",
            "web/data",
            "web/learn",
            "web/content/modules",
            "registries",
        ] {
            std::fs::create_dir_all(root.join(d)).unwrap();
        }
        std::fs::create_dir_all(&corpus).unwrap();
        std::fs::write(root.join("registries/claims.toml"), "schema_version = 1\n").unwrap();
        Tree {
            _dir: dir,
            root,
            corpus,
        }
    }

    fn write(&self, rel: &str, body: &str) -> &Self {
        let p = self.root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
        self
    }

    fn note(&self, name: &str, body: &str) -> &Self {
        let p = self.corpus.join(name);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
        self
    }

    fn exists(&self, rel: &str) -> bool {
        self.root.join(rel).is_file()
    }

    fn read(&self, rel: &str) -> String {
        std::fs::read_to_string(self.root.join(rel)).unwrap()
    }
}

fn one_domain(extra: &str) -> String {
    format!(
        "schema_version = 1\n\n\
         [[domain]]\n\
         id = \"01-mission-critical\"\n\
         order = 1\n\
         epi_heading = \"The Mission Critical Site\"\n\
         {extra}\n"
    )
}

fn green_tree() -> Tree {
    let t = Tree::new();
    t.write(
        "knowledge/domains.toml",
        &one_domain("primary_notes = \"../modules/01-mission-critical.md\""),
    );
    t.note(
        "01-mission-critical.md",
        "# Mission critical\n\n## Core concepts\n\nEnough words here to count.\n",
    );
    t.write("web/content/modules/README.md", "tracked\n");
    t
}

// ── known-bad plants ──────────────────────────────────────────────────────

#[test]
fn missing_domains_toml_is_red_and_writes_nothing() {
    let t = Tree::new();
    t.write("web/content/modules/README.md", "tracked\n");
    let plan = cdcp_learn::build::evaluate(&t.root).expect("evaluate");
    assert_eq!(plan.code, 1);
    assert!(plan.stdout.contains("FAIL: build_learn"), "{}", plan.stdout);
    assert!(
        plan.writes.is_empty() && plan.unlinks.is_empty(),
        "RED must not carry writes: {plan:?}"
    );
}

#[test]
fn zero_domain_rows_is_red() {
    let t = Tree::new();
    t.write("knowledge/domains.toml", "schema_version = 1\n");
    let plan = cdcp_learn::build::evaluate(&t.root).expect("evaluate");
    assert_eq!(plan.code, 1);
    assert!(
        plan.stdout.contains("zero [[domain]] rows"),
        "{}",
        plan.stdout
    );
    assert!(plan.writes.is_empty());
}

#[test]
fn empty_primary_notes_without_exam_unknown_is_red() {
    let t = Tree::new();
    t.write(
        "knowledge/domains.toml",
        &one_domain("primary_notes = \"\"\n"),
    );
    let plan = cdcp_learn::build::evaluate(&t.root).expect("evaluate");
    assert_eq!(plan.code, 1);
    assert!(
        plan.stdout
            .contains("empty primary_notes without exam_weight_unknown=true"),
        "{}",
        plan.stdout
    );
    assert!(plan.writes.is_empty());
}

#[test]
fn missing_notes_file_is_red_and_names_the_path() {
    let t = Tree::new();
    t.write(
        "knowledge/domains.toml",
        &one_domain("primary_notes = \"../modules/nope.md\""),
    );
    let plan = cdcp_learn::build::evaluate(&t.root).expect("evaluate");
    assert_eq!(plan.code, 1);
    assert!(
        plan.stdout
            .contains("primary_notes missing: ../modules/nope.md"),
        "{}",
        plan.stdout
    );
    assert!(plan.writes.is_empty());
}

#[test]
fn red_does_not_unlink_a_planted_stale_copy() {
    let t = Tree::new();
    t.write(
        "knowledge/domains.toml",
        &one_domain("primary_notes = \"\"\n"),
    );
    t.write("web/content/modules/99-stale.md", "stale\n");
    let plan = cdcp_learn::build::evaluate(&t.root).expect("evaluate");
    assert_eq!(plan.code, 1);
    assert!(
        plan.unlinks.is_empty(),
        "RED must not schedule unlinks: {:?}",
        plan.unlinks
    );
    assert!(t.exists("web/content/modules/99-stale.md"));
}

#[test]
fn empty_ok_without_any_md_is_red_empty_sweep() {
    let t = Tree::new();
    t.write(
        "knowledge/domains.toml",
        &one_domain("primary_notes = \"\"\nexam_weight_unknown = true\n"),
    );
    let plan = cdcp_learn::build::evaluate(&t.root).expect("evaluate");
    assert_eq!(plan.code, 1, "{}", plan.stdout);
    assert!(
        plan.stdout.contains("empty sweep is ERROR"),
        "{}",
        plan.stdout
    );
    assert!(plan.writes.is_empty());
}

// ── green path ────────────────────────────────────────────────────────────

#[test]
fn green_plant_writes_index_page_copy_and_sweeps_stale() {
    let t = green_tree();
    t.write("web/content/modules/99-stale.md", "leftover\n");
    t.write("web/learn/99-stale.html", "<html></html>\n");
    let outcome = cdcp_learn::build::write_learn(&t.root).expect("write");
    assert_eq!(outcome.code, 0, "{}", outcome.stdout);
    assert!(
        outcome.stdout.contains("PASS: build_learn"),
        "{}",
        outcome.stdout
    );
    assert!(t.exists("web/data/modules_index.json"));
    assert!(t.exists("web/data/topic_anchors.json"));
    assert!(t.exists("web/learn.html"));
    assert!(t.exists("web/learn/01-mission-critical.html"));
    assert!(t.exists("web/content/modules/01-mission-critical.md"));
    assert!(t.exists("web/content/modules/README.md"));
    assert!(!t.exists("web/content/modules/99-stale.md"));
    assert!(!t.exists("web/learn/99-stale.html"));

    let index = t.read("web/data/modules_index.json");
    assert!(index.contains("\"generated_by\": \"cdcp_learn\""));
    assert!(index.contains("01-mission-critical"));
    let page = t.read("web/learn/01-mission-critical.html");
    assert!(page.contains("does <strong>not</strong> grant EPI/EXIN certification"));
    assert!(page.contains("../assets/css/course.css"));
    assert!(page.contains("learn_reader.js"));
    assert!(page.contains("diagrams/site-stack.html"));
}

#[test]
fn empty_ok_module_has_no_page_and_no_href() {
    let t = Tree::new();
    t.write(
        "knowledge/domains.toml",
        "schema_version = 1\n\n\
         [[domain]]\n\
         id = \"01-mission-critical\"\n\
         order = 1\n\
         epi_heading = \"The Mission Critical Site\"\n\
         primary_notes = \"../modules/01-mission-critical.md\"\n\n\
         [[domain]]\n\
         id = \"15-ops-adjacent\"\n\
         order = 15\n\
         epi_heading = \"Ops\"\n\
         primary_notes = \"\"\n\
         exam_weight_unknown = true\n",
    );
    t.note("01-mission-critical.md", "# M\n\nnotes\n");
    t.write("web/content/modules/README.md", "tracked\n");
    let outcome = cdcp_learn::build::write_learn(&t.root).expect("write");
    assert_eq!(outcome.code, 0, "{}", outcome.stdout);
    assert!(t.exists("web/learn/01-mission-critical.html"));
    assert!(!t.exists("web/learn/15-ops-adjacent.html"));
    let index: serde_json::Value =
        serde_json::from_str(&t.read("web/data/modules_index.json")).unwrap();
    assert_eq!(index["empty_ok_count"], 1);
    let ops = index["modules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["id"] == "15-ops-adjacent")
        .unwrap();
    assert_eq!(ops["empty"], true);
    assert!(ops["href"].is_null());
}

#[test]
fn live_tree_compiles_and_matches_word_count_pin() {
    let root = engine_root();
    let plan = cdcp_learn::build::evaluate(&root).expect("evaluate live");
    assert_eq!(plan.code, 0, "live tree must compile:\n{}", plan.stdout);
    assert!(
        !plan.writes.is_empty(),
        "a green live compile must carry writes"
    );
    let index = plan
        .writes
        .iter()
        .find(|(p, _)| p.ends_with("modules_index.json"))
        .expect("index write");
    let v: serde_json::Value = serde_json::from_slice(&index.1).unwrap();
    assert_eq!(v["generated_by"], "cdcp_learn");
    assert_eq!(v["module_count"], 15);
    assert_eq!(v["navigable_count"], 15);
    let m01 = v["modules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["id"] == "01-mission-critical")
        .unwrap();
    assert_eq!(m01["word_count"], 3484);
    assert_eq!(m01["estimate_minutes"], 24);
    let m13 = plan
        .writes
        .iter()
        .find(|(p, _)| p.ends_with("13-security.html"))
        .expect("M13 page write");
    let html = String::from_utf8_lossy(&m13.1);
    assert!(
        html.contains("diagrams/security-layers.html"),
        "M13 must keep the shipped security-layers CTA"
    );
}

#[test]
fn python_builder_is_gone() {
    let root = engine_root();
    assert!(
        !root.join("scripts/build_learn.py").exists(),
        "EXTRACT-THEN-DELETE: scripts/build_learn.py must stay gone"
    );
}
