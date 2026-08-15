//! Verdict suite for `cdcp_learn::diagrams` (bd-substrate-rust-migration-jhd.14).
//!
//! EXTRACT-THEN-DELETE: this is NOT a differential against
//! `scripts/smoke_diagrams.py`. The Python is deleted in the same commit.
//! Every case asserts WHAT THE CORRECT ANSWER IS against the Rust alone.
//!
//! The parked wave-8 transcription stays parked. Empty input is an ERROR.
//! An unclosed honesty-banner is a FAIL (bd-smoke-diagrams-unclosed-banner-swallows-page-61v0
//! closed, not reproduced — a footer disclaimer must not save a hollow banner).
//!
//! This smoke is a READER. Fixtures live under $TMPDIR. The live-tree claim
//! is bought by copying the tracked surface and asserting the copy matches
//! before the smoke runs.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static RAN: AtomicUsize = AtomicUsize::new(0);

/// Raise when you add a `#[test]`. A DROP means a case was deleted.
const EXPECTED_CASES: usize = 32;

const LIVE_IDS: &[&str] = &[
    "power-path",
    "site-stack",
    "heat-path",
    "fire-sequence",
    "standards-map",
    "floor-airflow",
    "dual-cord-spof",
];

const IDS: &[&str] = &["a1", "b2", "c3", "d4", "e5", "f6", "g7"];

fn engine_root() -> PathBuf {
    cdcp_learn::resolve_engine_root(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
}

fn tick() {
    RAN.fetch_add(1, Ordering::SeqCst);
}

fn run(root: &Path) -> cdcp_learn::BuildOutcome {
    cdcp_learn::diagrams::run(root)
}

struct Tree {
    _dir: tempfile::TempDir,
    root: PathBuf,
}

impl Tree {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir
            .path()
            .canonicalize()
            .expect("canonicalize")
            .join("engine");
        for d in ["docs", "web/diagrams", "registries"] {
            std::fs::create_dir_all(root.join(d)).unwrap();
        }
        std::fs::write(root.join("registries/claims.toml"), "schema_version = 1\n").unwrap();
        Tree { _dir: dir, root }
    }

    fn write(&self, rel: &str, body: &str) -> &Self {
        let p = self.root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
        self
    }

    fn remove(&self, rel: &str) -> &Self {
        let p = self.root.join(rel);
        if p.is_file() {
            std::fs::remove_file(p).unwrap();
        }
        self
    }
}

fn page(id: &str) -> String {
    format!(
        "<!doctype html><html><body>\n\
         <div class=\"honesty-banner\">This tool does <strong>not</strong> grant EPI/EXIN certification.</div>\n\
         <div data-diagram=\"{id}\"></div>\n\
         </body></html>\n"
    )
}

fn present_row(id: &str) -> String {
    format!("| `{id}` | T | 01 | P0 | **present** | `web/diagrams/{id}.html` |")
}

fn registry_md(rows: &[String]) -> String {
    let mut s = String::from(
        "## Inventory\n\n\
         | ID | Title | Modules | Priority | Status | Path |\n\
         |----|-------|---------|----------|--------|------|\n",
    );
    for r in rows {
        s.push_str(r);
        s.push('\n');
    }
    s
}

fn good_rows() -> Vec<String> {
    IDS.iter().map(|id| present_row(id)).collect()
}

fn green_tree() -> Tree {
    let t = Tree::new();
    t.write("docs/DIAGRAM-REGISTRY.md", &registry_md(&good_rows()));
    for id in IDS {
        t.write(&format!("web/diagrams/{id}.html"), &page(id));
    }
    t
}

fn snapshot(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(base: &Path, cur: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        let Ok(rd) = std::fs::read_dir(cur) else {
            return;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(base, &p, out);
            } else if p.is_file() {
                let rel = p.strip_prefix(base).unwrap().to_string_lossy().into_owned();
                out.insert(rel, std::fs::read(&p).unwrap_or_default());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(dir, dir, &mut out);
    out
}

fn live_copy() -> Tree {
    let t = Tree::new();
    let src = engine_root();
    let reg = src.join("docs/DIAGRAM-REGISTRY.md");
    t.write(
        "docs/DIAGRAM-REGISTRY.md",
        &std::fs::read_to_string(&reg).expect("live registry"),
    );
    for id in LIVE_IDS {
        let rel = format!("web/diagrams/{id}.html");
        let body =
            std::fs::read_to_string(src.join(&rel)).unwrap_or_else(|e| panic!("live {rel}: {e}"));
        t.write(&rel, &body);
    }
    t
}

// ── live tree ──────────────────────────────────────────────────────────────

#[test]
fn live_copy_is_green_and_checked_seven() {
    tick();
    let t = live_copy();
    let o = run(&t.root);
    assert_eq!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout
            .contains("smoke_diagrams: PASS (7 present diagrams from the registry)"),
        "{}",
        o.stdout
    );
    for id in LIVE_IDS {
        assert!(
            o.stdout.contains(&format!("  ok: {id}\n")),
            "live {id} unreported:\n{}",
            o.stdout
        );
    }
    assert!(o.artifact.is_none(), "smoke is a reader");
}

#[test]
fn smoke_writes_nothing() {
    tick();
    let t = live_copy();
    let before = snapshot(&t.root);
    let _ = run(&t.root);
    let after = snapshot(&t.root);
    assert_eq!(before, after, "smoke_diagrams wrote to the tree");
    assert!(
        before.len() >= LIVE_IDS.len(),
        "an empty snapshot is an ERROR ({} files)",
        before.len()
    );
}

#[test]
fn synthetic_green_tree_passes() {
    tick();
    let t = green_tree();
    let o = run(&t.root);
    assert_eq!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout
            .contains("smoke_diagrams: PASS (7 present diagrams from the registry)"),
        "{}",
        o.stdout
    );
}

// ── registry legs ──────────────────────────────────────────────────────────

#[test]
fn missing_registry_is_error_two() {
    tick();
    let t = Tree::new();
    let o = run(&t.root);
    assert_eq!(o.code, 2, "{}", o.stdout);
    assert!(
        o.stdout
            .contains("smoke_diagrams: ERROR: missing registry "),
        "{}",
        o.stdout
    );
}

#[test]
fn empty_registry_is_error_two() {
    tick();
    let t = green_tree();
    t.write("docs/DIAGRAM-REGISTRY.md", "");
    let o = run(&t.root);
    assert_eq!(o.code, 2, "{}", o.stdout);
    assert!(
        o.stdout.contains("no '## Inventory' heading"),
        "{}",
        o.stdout
    );
}

#[test]
fn heading_without_table_is_error() {
    tick();
    let t = green_tree();
    t.write(
        "docs/DIAGRAM-REGISTRY.md",
        "## Inventory\n\nprose, no pipes\n",
    );
    let o = run(&t.root);
    assert_eq!(o.code, 2, "{}", o.stdout);
    assert!(
        o.stdout.contains("no table under '## Inventory'"),
        "{}",
        o.stdout
    );
}

#[test]
fn columns_changed_is_error() {
    tick();
    let t = green_tree();
    t.write(
        "docs/DIAGRAM-REGISTRY.md",
        "## Inventory\n\n| Id | Title | Modules | Priority | Status | Path |\n|---|---|---|---|---|---|\n",
    );
    let o = run(&t.root);
    assert_eq!(o.code, 2, "{}", o.stdout);
    assert!(
        o.stdout.contains("inventory columns changed"),
        "{}",
        o.stdout
    );
}

#[test]
fn no_separator_row_is_error() {
    tick();
    let t = green_tree();
    t.write(
        "docs/DIAGRAM-REGISTRY.md",
        &format!(
            "## Inventory\n\n| ID | Title | Modules | Priority | Status | Path |\n{}\n",
            present_row("a1")
        ),
    );
    let o = run(&t.root);
    assert_eq!(o.code, 2, "{}", o.stdout);
    assert!(
        o.stdout
            .contains("inventory header not followed by a separator row"),
        "{}",
        o.stdout
    );
}

#[test]
fn zero_present_rows_is_error_never_a_pass() {
    tick();
    let cases = [
        ("no rows", registry_md(&[])),
        (
            "all planned",
            registry_md(&["| `only` | T | 01 | P2 | planned | \u{2014} |".into()]),
        ),
    ];
    for (label, md) in cases {
        let t = Tree::new();
        t.write("docs/DIAGRAM-REGISTRY.md", &md);
        let o = run(&t.root);
        assert_eq!(o.code, 2, "[{label}] {}", o.stdout);
        assert!(
            o.stdout
                .contains("zero present diagrams parsed from the registry"),
            "[{label}] {}",
            o.stdout
        );
        assert!(
            !o.stdout.contains("smoke_diagrams: PASS"),
            "[{label}] PASS must not appear: {}",
            o.stdout
        );
    }
}

#[test]
fn present_count_pin_trips_on_six_and_eight() {
    tick();
    let six: Vec<String> = good_rows()[..6].to_vec();
    let t = green_tree();
    t.write("docs/DIAGRAM-REGISTRY.md", &registry_md(&six));
    let o = run(&t.root);
    assert_eq!(o.code, 2, "{}", o.stdout);
    assert!(
        o.stdout.contains("present count 6 != pinned 7"),
        "{}",
        o.stdout
    );

    let mut eight = good_rows();
    eight.push(present_row("h8"));
    t.write("docs/DIAGRAM-REGISTRY.md", &registry_md(&eight));
    t.write("web/diagrams/h8.html", &page("h8"));
    let o = run(&t.root);
    assert_eq!(o.code, 2, "{}", o.stdout);
    assert!(
        o.stdout.contains("present count 8 != pinned 7"),
        "{}",
        o.stdout
    );
}

#[test]
fn unbackticked_id_is_error_not_a_silent_skip() {
    tick();
    let t = green_tree();
    let mut rows = good_rows();
    rows[0] = "| a1 | T | 01 | P0 | **present** | `web/diagrams/a1.html` |".into();
    t.write("docs/DIAGRAM-REGISTRY.md", &registry_md(&rows));
    let o = run(&t.root);
    assert_eq!(o.code, 2, "{}", o.stdout);
    assert!(o.stdout.contains("malformed ID cell"), "{}", o.stdout);
}

#[test]
fn unrecognised_status_is_error_never_a_silent_exclusion() {
    tick();
    let t = green_tree();
    let mut rows = good_rows();
    rows[3] = "| `d4` | T | 01 | P0 | shipped | `web/diagrams/d4.html` |".into();
    t.write("docs/DIAGRAM-REGISTRY.md", &registry_md(&rows));
    let o = run(&t.root);
    assert_eq!(o.code, 2, "{}", o.stdout);
    assert!(o.stdout.contains("unrecognised status"), "{}", o.stdout);
}

#[test]
fn path_redirect_at_the_registry_is_error() {
    tick();
    let t = green_tree();
    let mut rows = good_rows();
    rows[3] = "| `d4` | T | 01 | P0 | **present** | `docs/DIAGRAM-REGISTRY.md` |".into();
    t.write("docs/DIAGRAM-REGISTRY.md", &registry_md(&rows));
    let o = run(&t.root);
    assert_eq!(o.code, 2, "{}", o.stdout);
    assert!(
        o.stdout.contains("docs/DIAGRAM-REGISTRY.md")
            && o.stdout.contains("is not")
            && o.stdout.contains("web/diagrams/d4.html"),
        "{}",
        o.stdout
    );
}

#[test]
fn planned_row_naming_a_path_is_error() {
    tick();
    let t = green_tree();
    let mut rows = good_rows();
    rows.push("| `h8` | T | 01 | P2 | planned | `web/diagrams/h8.html` |".into());
    t.write("docs/DIAGRAM-REGISTRY.md", &registry_md(&rows));
    let o = run(&t.root);
    assert_eq!(o.code, 2, "{}", o.stdout);
    assert!(
        o.stdout.contains("is planned but names path"),
        "{}",
        o.stdout
    );
}

#[test]
fn duplicate_id_is_error() {
    tick();
    let t = green_tree();
    let mut rows = good_rows();
    rows[6] = present_row("a1");
    t.write("docs/DIAGRAM-REGISTRY.md", &registry_md(&rows));
    let o = run(&t.root);
    assert_eq!(o.code, 2, "{}", o.stdout);
    assert!(o.stdout.contains("duplicate ID `a1`"), "{}", o.stdout);
}

#[test]
fn status_spellings_normalise_to_present() {
    tick();
    let t = green_tree();
    let mut rows = good_rows();
    rows[0] = "| `a1` | T | 01 | P0 | PRESENT | `web/diagrams/a1.html` |".into();
    rows[1] = "| `b2` | T | 01 | P0 | `**Present**` | `web/diagrams/b2.html` |".into();
    rows[2] = "| `c3` | T | 01 | P0 | ***present*** | `web/diagrams/c3.html` |".into();
    t.write("docs/DIAGRAM-REGISTRY.md", &registry_md(&rows));
    let o = run(&t.root);
    assert_eq!(o.code, 0, "{}", o.stdout);
}

// ── artifact legs ──────────────────────────────────────────────────────────

#[test]
fn missing_diagram_file_is_fail() {
    tick();
    let t = green_tree();
    t.remove("web/diagrams/a1.html");
    let o = run(&t.root);
    assert_eq!(o.code, 1, "{}", o.stdout);
    assert!(
        o.stdout.contains("a1: missing web/diagrams/a1.html"),
        "{}",
        o.stdout
    );
}

#[test]
fn plaintext_file_is_not_html() {
    tick();
    let t = green_tree();
    t.write(
        "web/diagrams/a1.html",
        "not certif a1 data-diagram honesty-banner",
    );
    let o = run(&t.root);
    assert_eq!(o.code, 1, "{}", o.stdout);
    assert!(
        o.stdout.contains("a1: not HTML (zero elements parsed)"),
        "{}",
        o.stdout
    );
}

#[test]
fn empty_file_is_not_html() {
    tick();
    let t = green_tree();
    t.write("web/diagrams/a1.html", "");
    let o = run(&t.root);
    assert_eq!(o.code, 1, "{}", o.stdout);
    assert!(
        o.stdout.contains("a1: not HTML (zero elements parsed)"),
        "{}",
        o.stdout
    );
}

#[test]
fn missing_honesty_banner_is_fail() {
    tick();
    let t = green_tree();
    t.write(
        "web/diagrams/a1.html",
        "<div data-diagram=\"a1\">not certified</div>",
    );
    let o = run(&t.root);
    assert_eq!(o.code, 1, "{}", o.stdout);
    assert!(
        o.stdout
            .contains("no element with class=\"honesty-banner\""),
        "{}",
        o.stdout
    );
}

#[test]
fn empty_banner_is_fail() {
    tick();
    let t = green_tree();
    t.write(
        "web/diagrams/a1.html",
        "<div class=\"honesty-banner\"></div><i data-diagram=\"a1\"></i>",
    );
    let o = run(&t.root);
    assert_eq!(o.code, 1, "{}", o.stdout);
    assert!(
        o.stdout.contains("honesty-banner element is empty"),
        "{}",
        o.stdout
    );
}

#[test]
fn banner_without_not_word_is_fail() {
    tick();
    let t = green_tree();
    t.write(
        "web/diagrams/a1.html",
        "<div class=\"honesty-banner\">no certification here</div><i data-diagram=\"a1\"></i>",
    );
    let o = run(&t.root);
    assert_eq!(o.code, 1, "{}", o.stdout);
    assert!(
        o.stdout
            .contains("honesty-banner does not disclaim certification"),
        "{}",
        o.stdout
    );
}

#[test]
fn notice_is_not_the_word_not() {
    tick();
    let t = green_tree();
    t.write(
        "web/diagrams/a1.html",
        "<div class=\"honesty-banner\">notice: certification</div><i data-diagram=\"a1\"></i>",
    );
    let o = run(&t.root);
    assert_eq!(o.code, 1, "{}", o.stdout);
    assert!(
        o.stdout
            .contains("honesty-banner does not disclaim certification"),
        "{}",
        o.stdout
    );
}

#[test]
fn text_outside_the_banner_does_not_count() {
    tick();
    let t = green_tree();
    t.write(
        "web/diagrams/a1.html",
        "<div class=\"honesty-banner\">welcome</div>does not grant certification<i data-diagram=\"a1\"></i>",
    );
    let o = run(&t.root);
    assert_eq!(o.code, 1, "{}", o.stdout);
    assert!(
        o.stdout
            .contains("honesty-banner does not disclaim certification"),
        "{}",
        o.stdout
    );
}

/// bd-smoke-diagrams-unclosed-banner-swallows-page-61v0.
/// The retired script scored this GREEN (the banner swallowed the footer).
/// EXTRACT-THEN-DELETE closes the hole: unclosed is RED.
#[test]
fn unclosed_banner_plus_footer_disclaimer_is_red() {
    tick();
    let t = green_tree();
    t.write(
        "web/diagrams/a1.html",
        "<div class=\"honesty-banner\">Welcome<i data-diagram=\"a1\"></i>\
         <footer>This tool does not grant EPI/EXIN certification.</footer>",
    );
    let o = run(&t.root);
    assert_ne!(
        o.code, 0,
        "unclosed banner must be RED, got PASS:\n{}",
        o.stdout
    );
    assert!(
        o.stdout.contains("honesty-banner is never closed"),
        "must name the unclosed banner even if a footer would disclaim:\n{}",
        o.stdout
    );
    assert!(
        !o.stdout.contains("smoke_diagrams: PASS"),
        "PASS must not appear: {}",
        o.stdout
    );
}

#[test]
fn closed_hollow_banner_plus_footer_is_red() {
    tick();
    let t = green_tree();
    t.write(
        "web/diagrams/a1.html",
        "<div class=\"honesty-banner\">Welcome</div><i data-diagram=\"a1\"></i>\
         <footer>This tool does not grant EPI/EXIN certification.</footer>",
    );
    let o = run(&t.root);
    assert_eq!(o.code, 1, "{}", o.stdout);
    assert!(
        o.stdout
            .contains("honesty-banner does not disclaim certification"),
        "{}",
        o.stdout
    );
}

#[test]
fn banner_inside_a_comment_does_not_count() {
    tick();
    let t = green_tree();
    t.write(
        "web/diagrams/a1.html",
        "<!--<div class=\"honesty-banner\">does not grant certification</div>-->\
         <i data-diagram=\"a1\"></i>",
    );
    let o = run(&t.root);
    assert_eq!(o.code, 1, "{}", o.stdout);
    assert!(
        o.stdout
            .contains("no element with class=\"honesty-banner\""),
        "{}",
        o.stdout
    );
}

#[test]
fn missing_data_diagram_is_fail() {
    tick();
    let t = green_tree();
    t.write(
        "web/diagrams/a1.html",
        "<div class=\"honesty-banner\">does not grant certification</div>",
    );
    let o = run(&t.root);
    assert_eq!(o.code, 1, "{}", o.stdout);
    assert!(
        o.stdout.contains("no element with data-diagram=\"a1\""),
        "{}",
        o.stdout
    );
}

#[test]
fn class_substring_is_not_the_token() {
    tick();
    let t = green_tree();
    t.write(
        "web/diagrams/a1.html",
        "<div class=\"my-honesty-banner\">does not grant certification</div>\
         <i data-diagram=\"a1\"></i>",
    );
    let o = run(&t.root);
    assert_eq!(o.code, 1, "{}", o.stdout);
    assert!(
        o.stdout
            .contains("no element with class=\"honesty-banner\""),
        "{}",
        o.stdout
    );
}

#[test]
fn nested_markup_in_banner_is_green() {
    tick();
    let t = green_tree();
    t.write(
        "web/diagrams/a1.html",
        "<div class=\"honesty-banner\"><p>does <strong>not</strong> grant</p> \
         <em>certification</em></div><i data-diagram=\"a1\"></i>",
    );
    let o = run(&t.root);
    assert_eq!(o.code, 0, "{}", o.stdout);
}

#[test]
fn entity_that_spells_not_does_not_disclaim() {
    tick();
    let t = green_tree();
    t.write(
        "web/diagrams/a1.html",
        "<div class=\"honesty-banner\">&not; certification</div><i data-diagram=\"a1\"></i>",
    );
    let o = run(&t.root);
    assert_eq!(o.code, 1, "{}", o.stdout);
    assert!(
        o.stdout
            .contains("honesty-banner does not disclaim certification"),
        "{}",
        o.stdout
    );
}

// ── anti-vacuous meta ──────────────────────────────────────────────────────

#[test]
fn this_suite_has_not_shrunk() {
    tick();
    let this = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/smoke_diagrams.rs"),
    )
    .expect("this test file");
    let cases = this.matches("#[test]").count();
    assert!(
        cases >= EXPECTED_CASES,
        "case count fell to {cases}; EXPECTED_CASES is {EXPECTED_CASES}. \
         A suite that quietly shrank reports exactly like one that passed."
    );
    assert!(
        RAN.load(Ordering::SeqCst) >= 1,
        "this file's own tick must have fired"
    );
}
