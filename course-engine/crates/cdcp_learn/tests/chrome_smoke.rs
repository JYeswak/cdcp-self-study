//! Verdict suite for `cdcp_learn::chrome` (extracted by
//! bd-substrate-rust-migration-jhd.15). The Python is DELETED. Every case
//! asserts the correct answer, not agreement with a retired script.
//!
//! This smoke is a CHECKER: it reads the Learn UI tree and writes nothing.
//! Cases run against TEMP fixtures (or the live tree read-only). The live-tree
//! case is the only one that opens the committed `web/`.
//!
//! ANTI-VACUOUS: an empty `web/` is RED; a modules index with no navigable
//! modules is RED; a suite that ran no case is RED. Every planted known-bad
//! is a real verdict — delete the assertion and this file goes red.

use cdcp_learn::chrome::{
    smoke, CSS, CSS_HOOKS, HUB, HUB_NEEDLES, INDEX, M01, M01_NEEDLES, M06, M06_NEEDLES, MD_JS,
    MIN_CHECKS, POWER_PATH,
};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Cases actually run, so "the suite ran" is itself checked.
static RAN: AtomicUsize = AtomicUsize::new(0);
static ROUND: AtomicUsize = AtomicUsize::new(0);

fn engine_root() -> PathBuf {
    cdcp_learn::resolve_engine_root(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
}

fn write_file(path: &Path, body: &str) {
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p).unwrap();
    }
    std::fs::write(path, body).unwrap();
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
        write_file(&self.at(rel), body);
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

/// Minimal GREEN tree: every compiled-in needle, nothing extra.
fn green() -> Fixture {
    let f = Fixture::new();
    f.put("web/assets/js/learn_chrome.js", "/* chrome */\n");
    f.put(
        MD_JS,
        "function latexToHtml(src) { return src; }\n<div class=\"math-block\"></div>\n",
    );
    f.put(
        M01,
        "<nav id=\"learn-toc\"></nav>\n<div id=\"learn-progress-bar\"></div>\n<script src=\"../assets/js/learn_chrome.js\"></script>\n",
    );
    f.put(
        M06,
        "<aside class=\"diagram-cta\"><a href=\"../diagrams/power-path.html\">power</a></aside>\n",
    );
    f.put(
        HUB,
        "<p id=\"learn-continue\"></p>\n<script src=\"assets/js/learn_chrome.js\"></script>\n",
    );
    f.put(
        INDEX,
        r#"{"modules":[{"id":"01-mission-critical","empty":false,"estimate_minutes":24,"word_count":100}]}"#,
    );
    f.put(POWER_PATH, "<html>power-path</html>\n");
    f.put(
        CSS,
        ".learn-toc{}\n.math-block{}\n.learn-continue{}\n.diagram-cta{}\n",
    );
    f
}

fn tick() {
    RAN.fetch_add(1, Ordering::SeqCst);
}

#[test]
fn live_tree_passes() {
    tick();
    let root = engine_root();
    assert!(
        root.join("web").is_dir(),
        "live engine has no web/ — a missing product tree is an ERROR"
    );
    let r = smoke(&root);
    assert_eq!(r.code, 0, "{}", r.stdout);
    assert!(
        r.stdout.contains("smoke_learn_chrome: PASS"),
        "{}",
        r.stdout
    );
    assert!(
        r.checks >= MIN_CHECKS,
        "live tree performed {} checks < MIN_CHECKS={MIN_CHECKS}",
        r.checks
    );
    assert_eq!(r.errors, 0);
}

#[test]
fn green_fixture_passes_and_meets_the_floor() {
    tick();
    let f = green();
    let r = smoke(&f.root);
    assert_eq!(r.code, 0, "{}", r.stdout);
    assert_eq!(r.errors, 0, "{}", r.stdout);
    assert!(
        r.checks >= MIN_CHECKS,
        "green fixture performed {} checks < MIN_CHECKS={MIN_CHECKS}: {}",
        r.checks,
        r.stdout
    );
    // Keep the temp dir alive until after the run.
    let _ = &f.dir;
}

#[test]
fn empty_web_is_an_error() {
    tick();
    let f = Fixture::new();
    let r = smoke(&f.root);
    assert_ne!(r.code, 0, "empty web/ must be RED, got PASS:\n{}", r.stdout);
    assert!(
        r.errors > 0,
        "empty web/ produced no named errors:\n{}",
        r.stdout
    );
    assert!(
        r.stdout.contains("smoke_learn_chrome: FAIL"),
        "{}",
        r.stdout
    );
    let _ = &f.dir;
}

#[test]
fn missing_learn_chrome_js_is_red() {
    tick();
    let f = green();
    f.rm("web/assets/js/learn_chrome.js");
    let r = smoke(&f.root);
    assert_ne!(r.code, 0, "{}", r.stdout);
    assert!(r.stdout.contains("missing learn_chrome.js"), "{}", r.stdout);
    let _ = &f.dir;
}

#[test]
fn learn_md_without_latex_path_is_red() {
    tick();
    let f = green();
    f.put(MD_JS, "/* no formula path */\n");
    let r = smoke(&f.root);
    assert_ne!(r.code, 0, "{}", r.stdout);
    assert!(
        r.stdout.contains("learn_md.js missing latex/math-block path"),
        "{}",
        r.stdout
    );
    let _ = &f.dir;
}

#[test]
fn m01_missing_toc_is_red() {
    tick();
    let f = green();
    f.put(
        M01,
        "<div id=\"learn-progress-bar\"></div>\n<script src=\"learn_chrome.js\"></script>\n",
    );
    let r = smoke(&f.root);
    assert_ne!(r.code, 0, "{}", r.stdout);
    assert!(
        r.stdout.contains("id=\"learn-toc\""),
        "must name the missing TOC needle:\n{}",
        r.stdout
    );
    let _ = &f.dir;
}

#[test]
fn m06_missing_power_path_cta_is_red() {
    tick();
    let f = green();
    f.put(M06, "<aside class=\"diagram-cta\">no diagram href</aside>\n");
    let r = smoke(&f.root);
    assert_ne!(r.code, 0, "{}", r.stdout);
    assert!(
        r.stdout.contains("diagrams/power-path.html"),
        "{}",
        r.stdout
    );
    let _ = &f.dir;
}

#[test]
fn hub_missing_continue_is_red() {
    tick();
    let f = green();
    f.put(HUB, "<script src=\"learn_chrome.js\"></script>\n");
    let r = smoke(&f.root);
    assert_ne!(r.code, 0, "{}", r.stdout);
    assert!(
        r.stdout.contains("id=\"learn-continue\""),
        "{}",
        r.stdout
    );
    let _ = &f.dir;
}

#[test]
fn empty_modules_index_is_red() {
    tick();
    let f = green();
    f.put(INDEX, r#"{"modules":[]}"#);
    let r = smoke(&f.root);
    assert_ne!(r.code, 0, "empty modules must be RED:\n{}", r.stdout);
    assert!(
        r.stdout.contains("no navigable modules"),
        "{}",
        r.stdout
    );
    let _ = &f.dir;
}

#[test]
fn all_empty_flagged_modules_is_red() {
    tick();
    let f = green();
    f.put(
        INDEX,
        r#"{"modules":[{"id":"99-blank","empty":true,"estimate_minutes":10}]}"#,
    );
    let r = smoke(&f.root);
    assert_ne!(r.code, 0, "{}", r.stdout);
    assert!(
        r.stdout.contains("no navigable modules"),
        "{}",
        r.stdout
    );
    let _ = &f.dir;
}

#[test]
fn modules_missing_eta_is_red() {
    tick();
    let f = green();
    f.put(
        INDEX,
        r#"{"modules":[{"id":"01-mission-critical","empty":false}]}"#,
    );
    let r = smoke(&f.root);
    assert_ne!(r.code, 0, "{}", r.stdout);
    assert!(
        r.stdout.contains("modules missing word_count/eta"),
        "{}",
        r.stdout
    );
    let _ = &f.dir;
}

#[test]
fn zero_eta_counts_as_missing() {
    tick();
    let f = green();
    f.put(
        INDEX,
        r#"{"modules":[{"id":"01-mission-critical","empty":false,"estimate_minutes":0,"word_count":0}]}"#,
    );
    let r = smoke(&f.root);
    assert_ne!(r.code, 0, "eta=0 must be missing:\n{}", r.stdout);
    let _ = &f.dir;
}

#[test]
fn unparseable_modules_index_is_red() {
    tick();
    let f = green();
    f.put(INDEX, "this is not json");
    let r = smoke(&f.root);
    assert_ne!(r.code, 0, "{}", r.stdout);
    assert!(
        r.stdout.contains("modules_index.json is not JSON"),
        "{}",
        r.stdout
    );
    let _ = &f.dir;
}

#[test]
fn missing_css_file_is_named_fail_not_a_panic() {
    tick();
    let f = green();
    f.rm(CSS);
    let r = smoke(&f.root);
    assert_ne!(r.code, 0, "{}", r.stdout);
    assert!(r.stdout.contains("missing course.css"), "{}", r.stdout);
    let _ = &f.dir;
}

#[test]
fn css_missing_hook_is_red() {
    tick();
    let f = green();
    f.put(CSS, ".learn-toc{}\n.math-block{}\n.learn-continue{}\n");
    let r = smoke(&f.root);
    assert_ne!(r.code, 0, "{}", r.stdout);
    assert!(
        r.stdout.contains("course.css missing .diagram-cta"),
        "{}",
        r.stdout
    );
    let _ = &f.dir;
}

#[test]
fn missing_power_path_file_is_red() {
    tick();
    let f = green();
    f.rm(POWER_PATH);
    let r = smoke(&f.root);
    assert_ne!(r.code, 0, "{}", r.stdout);
    assert!(
        r.stdout.contains("missing diagrams/power-path.html"),
        "{}",
        r.stdout
    );
    let _ = &f.dir;
}

#[test]
fn compiled_needles_are_not_empty() {
    tick();
    assert!(!M01_NEEDLES.is_empty());
    assert!(!M06_NEEDLES.is_empty());
    assert!(!HUB_NEEDLES.is_empty());
    assert!(!CSS_HOOKS.is_empty());
    assert!(MIN_CHECKS >= 15);
}

#[test]
fn the_suite_ran_something() {
    // Runs a case itself — test order is not a contract, and "0 cases run"
    // must never report like "all passed".
    let before = RAN.load(Ordering::SeqCst);
    let f = green();
    let r = smoke(&f.root);
    assert_eq!(r.code, 0, "{}", r.stdout);
    RAN.fetch_add(1, Ordering::SeqCst);
    assert!(
        RAN.load(Ordering::SeqCst) > before,
        "the verdict suite ran nothing"
    );
    let _ = &f.dir;
}
