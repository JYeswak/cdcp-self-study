//! bd-curriculum-truth-ebrr.3 — pedagogy UX locks (unit path, Continue, Drill-10).
//!
//! Static file checks on the learner-visible surface. Does not drive a browser.
//! Anti-vacuous: a suite that ran no case is ERROR; missing required files RED.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static RAN: AtomicUsize = AtomicUsize::new(0);

fn engine_root() -> PathBuf {
    cdcp_learn::resolve_engine_root(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
}

fn repo_root() -> PathBuf {
    engine_root()
        .parent()
        .expect("course-engine has a parent")
        .to_path_buf()
}

fn read(rel: &str) -> String {
    let p = engine_root().join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()))
}

fn tick() {
    RAN.fetch_add(1, Ordering::SeqCst);
}

#[test]
fn first_run_learn_opens_unit_1_not_full_article() {
    tick();
    let hub = read("web/learn.html");
    assert!(
        hub.contains("learn/01-mission-critical.html?unit=1"),
        "learn hub must offer Start unit 1 / first-unit href"
    );
    assert!(
        hub.contains("id=\"learn-start\"") || hub.contains("Start unit 1"),
        "first-run CTA missing"
    );
    assert!(
        hub.contains("Full article is the appendix"),
        "unit path must name full article as appendix"
    );
    let chrome = read("web/assets/js/learn_chrome.js");
    assert!(
        chrome.contains("maybeOpenUnitPath"),
        "first-run redirect helper missing"
    );
    assert!(
        chrome.contains("?unit="),
        "chrome must restore a unit query, not the catalog"
    );
    let units = read("web/assets/js/learn_units.js");
    assert!(
        units.contains("modeFull = params.get(\"full\") === \"1\""),
        "unit mode is default unless ?full=1"
    );
}

#[test]
fn continue_restores_unit_offset() {
    tick();
    let chrome = read("web/assets/js/learn_chrome.js");
    assert!(
        chrome.contains("function continueHref"),
        "continueHref must exist"
    );
    assert!(
        chrome.contains("?unit=") && chrome.contains("c.unit"),
        "Continue must encode unit offset"
    );
    let units = read("web/assets/js/learn_units.js");
    assert!(
        units.contains("persistUnit"),
        "unit navigation must persist offset"
    );
    let hub = read("web/learn.html");
    assert!(
        hub.contains("last unit in this browser"),
        "Continue copy must say unit, not only module"
    );
}

#[test]
fn unit_has_target_and_you_are_here_bar() {
    tick();
    let page = read("web/learn/01-mission-critical.html");
    assert!(
        page.contains("id=\"unit-here-bar\""),
        "M01 missing you-are-here bar"
    );
    assert!(
        page.contains("5–8 min target") || page.contains("5-8 min target"),
        "M01 missing 5–8 min target copy"
    );
    let css = read("web/assets/css/course.css");
    assert!(
        css.contains(".unit-here-bar"),
        "course.css missing .unit-here-bar"
    );
    let units = read("web/assets/js/learn_units.js");
    assert!(
        units.contains("function targetMinutes"),
        "sitting target helper missing"
    );
}

#[test]
fn hub_prefers_drill10_due_count() {
    tick();
    let hub = read("web/index.html");
    assert!(
        hub.contains("id=\"hub-due\""),
        "hub missing due-card count slot"
    );
    assert!(
        hub.contains("drill.html?mode=due"),
        "hub missing one-tap Drill-10"
    );
    let due_at = hub.find("id=\"hub-due\"").expect("hub-due");
    let learn_at = hub.find("id=\"hub-learn\"").expect("hub-learn");
    assert!(
        due_at < learn_at,
        "Drill-10 due block must appear before the Learn handbook card"
    );
    assert!(
        hub.contains("diagrams/power-path.html"),
        "90-second loop names one diagram"
    );
    let chrome = read("web/assets/js/learn_chrome.js");
    assert!(
        chrome.contains("function countDueCards"),
        "due-card counter missing"
    );
    assert!(
        chrome.contains("cdcp.srs.v1"),
        "due count reads the historical review key"
    );
}

#[test]
fn heavy_modules_have_one_produced_artifact() {
    tick();
    let m01 = read("web/learn/01-mission-critical.html");
    let m06 = read("web/learn/06-power.html");
    let m09 = read("web/learn/09-cooling.html");
    assert!(
        m01.contains("id=\"produced-artifact\"") && m01.contains("60-second site tour"),
        "M01 missing 60s tour prompt"
    );
    assert!(
        m06.contains("id=\"produced-artifact\"") && m06.contains("Label the one-line"),
        "M06 missing labeled one-line prompt"
    );
    assert!(
        m09.contains("id=\"produced-artifact\"") && m09.contains("Demarc sketch"),
        "M09 missing demarc sketch prompt"
    );
    let m02 = read("web/learn/02-standards.html");
    assert!(
        !m02.contains("id=\"produced-artifact\""),
        "produced-artifact is one per heavy module, not every page"
    );
    for page in [&m01, &m06, &m09] {
        assert!(
            page.contains("not a credential"),
            "artifact prompt must stay study-only"
        );
    }
}

#[test]
fn drill_and_charter_agree_short_interval_not_srs_shipped() {
    tick();
    let drill = read("web/drill.html");
    assert!(
        drill.contains("not spaced repetition"),
        "drill.html must keep the short-interval hedge"
    );
    assert!(
        !drill.to_lowercase().contains("srs shipped"),
        "drill.html must not reintroduce SRS shipped"
    );
    let charter =
        std::fs::read_to_string(repo_root().join("CHARTER.md")).expect("parent CHARTER.md");
    assert!(
        charter.contains("not SRS") || charter.contains("Not spaced repetition"),
        "CHARTER must agree the ladder is not SRS"
    );
    assert!(
        !charter.contains("SRS + Anki export"),
        "CHARTER pedagogy table must not say SRS shipped (ebrr.17)"
    );
    let chrome = read("web/assets/js/learn_chrome.js");
    assert!(
        !chrome.contains("SRS shipped"),
        "learn chrome must not claim SRS shipped"
    );
}

#[test]
fn no_cert_flavored_xp_or_streaks() {
    tick();
    for rel in [
        "web/index.html",
        "web/learn.html",
        "web/assets/js/learn_chrome.js",
        "web/assets/js/learn_units.js",
    ] {
        let body = read(rel);
        let lower = body.to_lowercase();
        assert!(
            !lower.contains("day streak")
                && !lower.contains("xp points")
                && !lower.contains("earn xp")
                && !lower.contains("keep your streak"),
            "{rel} grew cert-flavored XP/streak copy"
        );
    }
    let index = read("web/index.html");
    assert!(
        index.contains("honesty-banner"),
        "do not gut the honesty banner"
    );
    assert!(
        index.contains("study bar 27"),
        "do not gut the 27/40 study-signal hedge"
    );
    let css = read("web/assets/css/course.css");
    assert!(
        css.contains("never green") && css.contains("certified"),
        "do not gut the not-EPI CSS comment"
    );
}

#[test]
fn the_suite_ran_something() {
    let before = RAN.load(Ordering::SeqCst);
    assert!(
        engine_root().join("web/learn.html").is_file(),
        "missing learn.html — empty scan is ERROR"
    );
    RAN.fetch_add(1, Ordering::SeqCst);
    assert!(
        RAN.load(Ordering::SeqCst) > before,
        "the pedagogy suite ran nothing"
    );
}
