//! Verdict suite for `cdcp_learn::reference` (bd-substrate-rust-migration-jhd.29).
//!
//! EXTRACT-THEN-DELETE: this is NOT a differential against
//! `scripts/build_reference.py`. The Python is deleted in the same commit.

use std::path::{Path, PathBuf};

fn engine_root() -> PathBuf {
    cdcp_learn::resolve_engine_root(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
}

struct Tree {
    _dir: tempfile::TempDir,
    root: PathBuf,
    parent_ref: PathBuf,
}

impl Tree {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let base = dir.path().canonicalize().expect("canonicalize");
        let root = base.join("study/engine");
        let parent_ref = base.join("study/reference");
        for d in ["web/content/reference", "web", "registries"] {
            std::fs::create_dir_all(root.join(d)).unwrap();
        }
        std::fs::create_dir_all(&parent_ref).unwrap();
        std::fs::write(root.join("registries/claims.toml"), "schema_version = 1\n").unwrap();
        Tree {
            _dir: dir,
            root,
            parent_ref,
        }
    }

    fn seed_docs(&self) {
        std::fs::write(
            self.parent_ref.join("GLOSSARY.md"),
            "# Glossary\n\nSee [power](POWER-AND-REDUNDANCY-CHEATSHEET.md).\n",
        )
        .unwrap();
        std::fs::write(
            self.parent_ref.join("POWER-AND-REDUNDANCY-CHEATSHEET.md"),
            "# Power\n\nSee [M06](../modules/06-power.md).\n",
        )
        .unwrap();
    }

    fn exists(&self, rel: &str) -> bool {
        self.root.join(rel).is_file()
    }

    fn read(&self, rel: &str) -> String {
        std::fs::read_to_string(self.root.join(rel)).unwrap()
    }
}

#[test]
fn missing_parent_dir_is_red_and_writes_nothing() {
    let t = Tree::new();
    std::fs::remove_dir_all(&t.parent_ref).unwrap();
    let plan = cdcp_learn::reference::evaluate(&t.root).expect("evaluate");
    assert_eq!(plan.code, 1);
    assert!(
        plan.stdout.contains("missing parent reference dir"),
        "{}",
        plan.stdout
    );
    assert!(plan.writes.is_empty() && plan.unlinks.is_empty());
}

#[test]
fn missing_source_file_is_red() {
    let t = Tree::new();
    std::fs::write(t.parent_ref.join("GLOSSARY.md"), "# g\n").unwrap();
    let plan = cdcp_learn::reference::evaluate(&t.root).expect("evaluate");
    assert_eq!(plan.code, 1);
    assert!(
        plan.stdout.contains("POWER-AND-REDUNDANCY-CHEATSHEET.md"),
        "{}",
        plan.stdout
    );
    assert!(plan.writes.is_empty());
}

#[test]
fn green_plant_writes_page_and_rewritten_copies() {
    let t = Tree::new();
    t.seed_docs();
    std::fs::write(t.root.join("web/content/reference/STALE.md"), "old\n").unwrap();
    let outcome = cdcp_learn::reference::write_reference(&t.root).expect("write");
    assert_eq!(outcome.code, 0, "{}", outcome.stdout);
    assert!(outcome.stdout.contains("PASS: build_reference"));
    assert!(t.exists("web/reference.html"));
    assert!(t.exists("web/content/reference/GLOSSARY.md"));
    assert!(t.exists("web/content/reference/POWER-AND-REDUNDANCY-CHEATSHEET.md"));
    assert!(!t.exists("web/content/reference/STALE.md"));
    let page = t.read("web/reference.html");
    assert!(page.contains("does <strong>not</strong> grant EPI/EXIN certification"));
    assert!(page.contains("content/reference/GLOSSARY.md"));
    let gloss = t.read("web/content/reference/GLOSSARY.md");
    assert!(gloss.contains("](#power)"), "{gloss}");
    let power = t.read("web/content/reference/POWER-AND-REDUNDANCY-CHEATSHEET.md");
    assert!(power.contains("](learn/06-power.html)"), "{power}");
}

#[test]
fn live_tree_compiles() {
    let root = engine_root();
    let plan = cdcp_learn::reference::evaluate(&root).expect("evaluate live");
    assert_eq!(plan.code, 0, "live tree must compile:\n{}", plan.stdout);
    assert!(!plan.writes.is_empty());
}

#[test]
fn python_builder_is_gone() {
    let root = engine_root();
    assert!(
        !root.join("scripts/build_reference.py").exists(),
        "EXTRACT-THEN-DELETE: scripts/build_reference.py must stay gone"
    );
}
