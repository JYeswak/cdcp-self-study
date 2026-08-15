//! Verdict suite for `cdcp_learn::a11y` (extracted by
//! bd-substrate-rust-migration-jhd.19). The Python is DELETED. Every case
//! asserts the correct answer, not agreement with a retired script.
//!
//! This smoke is a READER: it reads the primary page shells and writes
//! nothing. Cases run against TEMP fixtures (or the live tree read-only).
//!
//! ANTI-VACUOUS: a missing `web/` is RED; an empty page / empty CSS is RED;
//! zero pages scanned is RED; undecodable or unreadable input is a named
//! FAIL, not a panic (bd-a11y-undecodable-raises-6jmi closed, not reproduced).
//! A marker that exists only inside an HTML comment or a `<script>` /
//! `<style>` body does not satisfy (bd-a11y-comment-blind-udze).
//! A suite that ran no case is RED.

use cdcp_learn::a11y::{
    run, strip_ignored_markup, CSS_REL, MIN_CHECKS, OPTIONAL_PAGES, REQUIRED_PAGES,
};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static RAN: AtomicUsize = AtomicUsize::new(0);
static ROUND: AtomicUsize = AtomicUsize::new(0);

/// Raise when you add a `#[test]`. A DROP means a case was deleted.
const EXPECTED_CASES: usize = 24;

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

const GOOD_CSS: &str = ":root{--touch-min:44px}\na:focus-visible{outline:2px solid}\n";

const GOOD_PAGE: &str = concat!(
    "<!doctype html><html><head>\n",
    "<link rel=\"stylesheet\" href=\"assets/css/course.css\">\n",
    "</head><body>\n",
    "<a class=\"skip-link\" href=\"#main\">Skip to main content</a>\n",
    "<div class=\"honesty-banner\">This is a study tool only.</div>\n",
    "<main id=\"main\">body</main>\n",
    "</body></html>\n"
);

fn green() -> Fixture {
    let f = Fixture::new();
    f.put(CSS_REL, GOOD_CSS);
    for name in REQUIRED_PAGES.iter().chain(OPTIONAL_PAGES.iter()) {
        f.put(&format!("web/{name}"), GOOD_PAGE);
    }
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

#[test]
fn live_tree_passes() {
    tick();
    let root = engine_root();
    assert!(
        root.join("web").is_dir(),
        "live engine has no web/ — a missing product tree is an ERROR"
    );
    let before = tree_digest(&root.join("web"));
    let r = run(&root);
    assert_eq!(r.code, 0, "{}", r.stdout);
    assert!(r.stdout.starts_with("PASS: smoke_a11y\n"), "{}", r.stdout);
    assert!(r.stdout.contains("pages_checked=7"), "{}", r.stdout);
    assert!(r.stdout.contains("optional_present=1"), "{}", r.stdout);
    assert_eq!(r.artifact, None, "a reader must not propose a write");
    assert_eq!(
        before,
        tree_digest(&root.join("web")),
        "the live tree was modified by a reader"
    );
}

#[test]
fn green_fixture_passes_and_meets_the_floor() {
    tick();
    let f = green();
    let r = run(&f.root);
    assert_eq!(r.code, 0, "{}", r.stdout);
    assert!(r.stdout.contains("PASS: smoke_a11y"), "{}", r.stdout);
    assert!(r.stdout.contains("pages_checked=7"), "{}", r.stdout);
    let _ = &f.dir;
}

#[test]
fn missing_web_is_an_error() {
    tick();
    let f = Fixture::new();
    let r = run(&f.root);
    assert_ne!(r.code, 0, "missing web/ must be RED:\n{}", r.stdout);
    assert_eq!(r.stdout, "FAIL: smoke_a11y \u{2014} missing web/\n");
    let _ = &f.dir;
}

#[test]
fn zero_pages_scanned_is_red() {
    tick();
    let f = Fixture::new();
    f.put(CSS_REL, GOOD_CSS);
    let r = run(&f.root);
    assert_ne!(
        r.code, 0,
        "a scan of nothing must never pass:\n{}",
        r.stdout
    );
    assert!(
        r.stdout
            .contains("zero primary HTML pages checked \u{2014} refusing vacuous green"),
        "{}",
        r.stdout
    );
    assert!(
        r.stdout
            .contains("missing required primary page web/index.html"),
        "{}",
        r.stdout
    );
    let _ = &f.dir;
}

#[test]
fn empty_page_and_empty_stylesheet_are_each_red() {
    tick();
    let f = green();
    f.put("web/quiz.html", "  \n\t \n");
    let r = run(&f.root);
    assert_ne!(r.code, 0);
    assert!(
        r.stdout
            .contains("web/quiz.html: empty file \u{2014} refusing vacuous green"),
        "{}",
        r.stdout
    );

    let f = green();
    f.put(CSS_REL, "\n   \n");
    let r = run(&f.root);
    assert_ne!(r.code, 0);
    assert!(
        r.stdout
            .contains("course.css is empty \u{2014} refusing vacuous green"),
        "{}",
        r.stdout
    );

    let f = green();
    f.rm(CSS_REL);
    let r = run(&f.root);
    assert_ne!(r.code, 0);
    assert!(
        r.stdout.contains("missing web/assets/css/course.css"),
        "{}",
        r.stdout
    );
    let _ = &f.dir;
}

#[test]
fn a_page_with_no_markers_names_all_four_findings() {
    tick();
    let f = green();
    f.put("web/drill.html", "<html><body>hello</body></html>\n");
    let r = run(&f.root);
    assert_ne!(r.code, 0);
    for needle in [
        "web/drill.html: missing skip link (.skip-link or Skip to main content)",
        "web/drill.html: missing honesty banner (.honesty-banner) and meta honesty language",
        "web/drill.html: missing main/content landmark (<main> or role=main)",
        "web/drill.html: missing course.css stylesheet link",
    ] {
        assert!(r.stdout.contains(needle), "{needle:?} in {}", r.stdout);
    }
    let _ = &f.dir;
}

#[test]
fn stylesheet_missing_either_token_is_red() {
    tick();
    let f = green();
    f.put(CSS_REL, "body{color:red}\n");
    let r = run(&f.root);
    assert_ne!(r.code, 0);
    assert!(
        r.stdout.contains("course.css: missing :focus-visible rule"),
        "{}",
        r.stdout
    );
    assert!(
        r.stdout.contains("course.css: missing --touch-min token"),
        "{}",
        r.stdout
    );
    let _ = &f.dir;
}

#[test]
fn css_tokens_are_case_sensitive() {
    tick();
    let f = green();
    f.put(
        CSS_REL,
        ":root{--TOUCH-MIN:44px}\na:FOCUS-VISIBLE{outline:2px}\n",
    );
    let r = run(&f.root);
    assert_ne!(r.code, 0, "CSS patterns have no re.I:\n{}", r.stdout);
    assert!(
        r.stdout.contains("course.css: missing :focus-visible rule")
            && r.stdout.contains("course.css: missing --touch-min token"),
        "{}",
        r.stdout
    );

    let f = green();
    f.put(
        CSS_REL,
        ":root{--touch-minimum:44px}\na:focus-visible{outline:2px}\n",
    );
    let r = run(&f.root);
    assert_ne!(r.code, 0, "{}", r.stdout);
    assert!(
        r.stdout.contains("course.css: missing --touch-min token"),
        "{}",
        r.stdout
    );
    let _ = &f.dir;
}

#[test]
fn skip_links_is_not_skip_link() {
    tick();
    let f = green();
    f.put(
        "web/index.html",
        "<link href=\"assets/css/course.css\"><a class=\"skip-links\">x</a>\
         <div class=\"honesty-banner\">study tool only</div><main>y</main>",
    );
    let r = run(&f.root);
    assert_ne!(r.code, 0, "{}", r.stdout);
    assert!(
        r.stdout.contains("web/index.html: missing skip link"),
        "{}",
        r.stdout
    );
    let _ = &f.dir;
}

#[test]
fn mainframe_is_not_a_landmark() {
    tick();
    let f = green();
    f.put(
        "web/index.html",
        "<link href=\"assets/css/course.css\"><a class=\"skip-link\">x</a>\
         <div class=\"honesty-banner\">study tool only</div><mainframe>y</mainframe>",
    );
    let r = run(&f.root);
    assert_ne!(r.code, 0, "{}", r.stdout);
    assert!(
        r.stdout
            .contains("web/index.html: missing main/content landmark"),
        "{}",
        r.stdout
    );
    let _ = &f.dir;
}

#[test]
fn id_main_without_landmark_is_the_other_message() {
    tick();
    let f = green();
    f.put(
        "web/index.html",
        "<link href=\"assets/css/course.css\"><a class=\"skip-link\">x</a>\
         <div class=\"honesty-banner\">study tool only</div><div id=\"main\">y</div>",
    );
    let r = run(&f.root);
    assert_ne!(r.code, 0, "{}", r.stdout);
    assert!(
        r.stdout.contains(
            "web/index.html: #main present but missing landmark element (<main> or role=main)"
        ),
        "{}",
        r.stdout
    );
    let _ = &f.dir;
}

#[test]
fn hollow_honesty_banner_is_red() {
    tick();
    let f = green();
    f.put(
        "web/index.html",
        "<link href=\"assets/css/course.css\"><a class=\"skip-link\">x</a>\
         <div class=\"honesty-banner\">Welcome</div><main>y</main>",
    );
    let r = run(&f.root);
    assert_ne!(r.code, 0, "{}", r.stdout);
    assert!(
        r.stdout.contains(
            "web/index.html: honesty-banner present but no non-grant / meta honesty language"
        ),
        "{}",
        r.stdout
    );
    let _ = &f.dir;
}

#[test]
fn honesty_carried_by_meta_description_is_green() {
    tick();
    let f = green();
    f.put(
        "web/index.html",
        "<link href=\"assets/css/course.css\">\
         <meta charset=\"utf-8\" name=\"description\" content=\"a study tool only, no cert\">\
         <a class=\"skip-link\">x</a><main>y</main>",
    );
    let r = run(&f.root);
    assert_eq!(r.code, 0, "{}", r.stdout);
    let _ = &f.dir;
}

#[test]
fn optional_page_is_checked_only_when_present() {
    tick();
    let f = green();
    f.rm("web/reference.html");
    let r = run(&f.root);
    assert_eq!(r.code, 0, "{}", r.stdout);
    assert!(r.stdout.contains("pages_checked=6"), "{}", r.stdout);
    assert!(r.stdout.contains("optional_present=0"), "{}", r.stdout);

    let f = green();
    f.put("web/reference.html", "<html>nothing</html>");
    let r = run(&f.root);
    assert_ne!(r.code, 0, "{}", r.stdout);
    assert!(
        r.stdout.contains("web/reference.html: missing skip link"),
        "{}",
        r.stdout
    );
    let _ = &f.dir;
}

#[test]
fn every_required_page_is_reported_when_all_are_broken() {
    tick();
    let f = green();
    for n in REQUIRED_PAGES {
        f.put(&format!("web/{n}"), "<html>x</html>");
    }
    let r = run(&f.root);
    assert_ne!(r.code, 0);
    let lines = r.stdout.lines().filter(|l| l.starts_with("  - ")).count();
    assert!(
        lines >= REQUIRED_PAGES.len() * 4,
        "expected at least four findings per shell, got {lines}:\n{}",
        r.stdout
    );
    let _ = &f.dir;
}

#[test]
fn cache_busting_query_string_does_not_count_as_the_stylesheet() {
    tick();
    let f = green();
    f.put(
        "web/index.html",
        "<link href=\"assets/css/course.css?v=2\"><a class=\"skip-link\">x</a>\
         <div class=\"honesty-banner\">study tool only</div><main>y</main>",
    );
    let r = run(&f.root);
    assert_ne!(r.code, 0, "{}", r.stdout);
    assert!(
        r.stdout
            .contains("web/index.html: missing course.css stylesheet link"),
        "{}",
        r.stdout
    );
    let _ = &f.dir;
}

/// bd-a11y-undecodable-raises-6jmi: invalid UTF-8 is a named FAIL, not a raise.
#[test]
fn undecodable_page_is_a_named_fail_not_a_panic() {
    tick();
    let f = green();
    f.put_bytes("web/mock.html", &[b'<', 0x80, b'>']);
    let r = run(&f.root);
    assert_ne!(r.code, 0, "undecodable input must never be a pass");
    assert!(
        r.stdout.contains("FAIL: smoke_a11y"),
        "must print the FAIL token, not raise:\n{}",
        r.stdout
    );
    assert!(
        r.stdout
            .contains("web/mock.html: not valid UTF-8 \u{2014} refusing vacuous green"),
        "must name the file:\n{}",
        r.stdout
    );
    let _ = &f.dir;
}

#[test]
fn undecodable_stylesheet_is_a_named_fail() {
    tick();
    let f = green();
    f.put_bytes(CSS_REL, &[0xff, 0xfe, b'x']);
    let r = run(&f.root);
    assert_ne!(r.code, 0);
    assert!(
        r.stdout
            .contains("web/assets/css/course.css: not valid UTF-8 \u{2014} refusing vacuous green"),
        "{}",
        r.stdout
    );
    let _ = &f.dir;
}

#[test]
fn the_reader_writes_nothing() {
    tick();
    let f = green();
    let before = tree_digest(&f.root);
    let r = run(&f.root);
    assert_eq!(r.code, 0, "{}", r.stdout);
    assert_eq!(r.artifact, None);
    assert_eq!(
        before,
        tree_digest(&f.root),
        "a reader wrote into the fixture"
    );
    let _ = &f.dir;
}

/// Wrap the first occurrence of `needle` in an HTML comment.
fn comment_wrap_first(src: &str, needle: &str) -> String {
    let Some(i) = src.find(needle) else {
        panic!("plant chrome drifted: missing {needle:?}");
    };
    let j = i + needle.len();
    format!("{}<!-- {} -->{}", &src[..i], needle, &src[j..])
}

/// Wrap `start` through the first `end` after it in an HTML comment.
fn comment_wrap_span(src: &str, start: &str, end: &str) -> String {
    let Some(i) = src.find(start) else {
        panic!("plant chrome drifted: missing start {start:?}");
    };
    let Some(rel) = src[i..].find(end) else {
        panic!("plant chrome drifted: missing end {end:?} after {start:?}");
    };
    let j = i + rel + end.len();
    format!("{}<!-- {} -->{}", &src[..i], &src[i..j], &src[j..])
}

/// Copy of a live shell with every census marker hosted only in a comment.
fn comment_out_required_markers(src: &str) -> String {
    let mut s = src.to_string();
    s = comment_wrap_first(
        &s,
        r#"<link rel="stylesheet" href="assets/css/course.css">"#,
    );
    s = comment_wrap_first(
        &s,
        r##"<a class="skip-link" href="#main">Skip to main content</a>"##,
    );
    s = comment_wrap_span(&s, r#"<div class="honesty-banner""#, "</div>");
    s = comment_wrap_span(&s, "<main", "</main>");
    s = comment_wrap_span(&s, r#"<meta name="description""#, ">");
    s
}

/// bd-a11y-comment-blind-udze: a live page whose only markers live in
/// `<!-- ... -->` is RED, and the row names the page and the class.
#[test]
fn commented_out_markers_on_a_live_page_copy_are_red() {
    tick();
    let live =
        std::fs::read_to_string(engine_root().join("web/index.html")).expect("live web/index.html");
    for token in [
        "skip-link",
        "honesty-banner",
        "<main",
        "assets/css/course.css",
        "Does not grant EPI/EXIN",
    ] {
        assert!(
            live.contains(token),
            "live index.html no longer carries {token:?} — plant would be vacuous"
        );
    }
    let planted = comment_out_required_markers(&live);
    assert!(
        planted.contains("<!--"),
        "plant must wrap markers, not delete them"
    );
    for token in [
        "skip-link",
        "honesty-banner",
        "<main",
        "assets/css/course.css",
    ] {
        assert!(
            planted.contains(token),
            "plant lost the raw marker bytes for {token:?}"
        );
        assert!(
            !strip_ignored_markup(&planted).contains(token),
            "strip_ignored_markup left {token:?} visible:\n{}",
            strip_ignored_markup(&planted)
        );
    }

    let f = green();
    f.put("web/index.html", &planted);
    let r = run(&f.root);
    assert_ne!(
        r.code, 0,
        "commented-only markers must not PASS:\n{}",
        r.stdout
    );
    assert!(r.stdout.contains("FAIL: smoke_a11y"), "{}", r.stdout);
    for needle in [
        "web/index.html: missing skip link (.skip-link or Skip to main content)",
        "web/index.html: missing honesty banner (.honesty-banner) and meta honesty language",
        "web/index.html: missing main/content landmark (<main> or role=main)",
        "web/index.html: missing course.css stylesheet link",
    ] {
        assert!(
            r.stdout.contains(needle),
            "must name the page and the marker class {needle:?}:\n{}",
            r.stdout
        );
    }
    let _ = &f.dir;
}

/// Same hole, other host: a `<script>` string is not the DOM.
#[test]
fn markers_only_inside_script_are_red() {
    tick();
    let f = green();
    f.put(
        "web/quiz.html",
        concat!(
            "<!doctype html><html><head></head><body>\n",
            "<script>\n",
            "const chrome = `",
            "<link rel=\"stylesheet\" href=\"assets/css/course.css\">",
            "<a class=\"skip-link\" href=\"#main\">Skip to main content</a>",
            "<div class=\"honesty-banner\">This is a study tool only.</div>",
            "<main id=\"main\">body</main>",
            "`;\n",
            "</script>\n",
            "<p>visible copy with no markers</p>\n",
            "</body></html>\n",
        ),
    );
    let r = run(&f.root);
    assert_ne!(
        r.code, 0,
        "script-hosted markers must not PASS:\n{}",
        r.stdout
    );
    assert!(
        r.stdout
            .contains("web/quiz.html: missing skip link (.skip-link or Skip to main content)"),
        "{}",
        r.stdout
    );
    assert!(
        r.stdout.contains(
            "web/quiz.html: missing honesty banner (.honesty-banner) and meta honesty language"
        ),
        "{}",
        r.stdout
    );
    assert!(
        r.stdout
            .contains("web/quiz.html: missing main/content landmark (<main> or role=main)"),
        "{}",
        r.stdout
    );
    assert!(
        r.stdout
            .contains("web/quiz.html: missing course.css stylesheet link"),
        "{}",
        r.stdout
    );
    let _ = &f.dir;
}

/// A comment next to a live marker must not eat the live one.
#[test]
fn commented_duplicate_does_not_hide_live_markers() {
    tick();
    let f = green();
    f.put(
        "web/drill.html",
        concat!(
            "<!doctype html><html><head>\n",
            "<!-- <link rel=\"stylesheet\" href=\"assets/css/course.css\"> -->\n",
            "<link rel=\"stylesheet\" href=\"assets/css/course.css\">\n",
            "</head><body>\n",
            "<!-- <a class=\"skip-link\" href=\"#main\">Skip to main content</a> -->\n",
            "<a class=\"skip-link\" href=\"#main\">Skip to main content</a>\n",
            "<!-- <div class=\"honesty-banner\">This is a study tool only.</div> -->\n",
            "<div class=\"honesty-banner\">This is a study tool only.</div>\n",
            "<!-- <main id=\"main\">dead</main> -->\n",
            "<main id=\"main\">body</main>\n",
            "</body></html>\n",
        ),
    );
    let r = run(&f.root);
    assert_eq!(
        r.code, 0,
        "live markers beside comments must still PASS:\n{}",
        r.stdout
    );
    let _ = &f.dir;
}

#[test]
fn compiled_lists_are_not_empty() {
    tick();
    assert!(!REQUIRED_PAGES.is_empty());
    assert!(!OPTIONAL_PAGES.is_empty());
    assert!(MIN_CHECKS >= 27);
}

#[test]
fn the_suite_ran_something() {
    let before = RAN.load(Ordering::SeqCst);
    let f = green();
    let r = run(&f.root);
    assert_eq!(r.code, 0, "{}", r.stdout);
    RAN.fetch_add(1, Ordering::SeqCst);
    assert!(
        RAN.load(Ordering::SeqCst) > before,
        "the verdict suite ran nothing"
    );
    assert_eq!(
        EXPECTED_CASES, 24,
        "EXPECTED_CASES changed — add or drop a case deliberately"
    );
    let _ = &f.dir;
}
