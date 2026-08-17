//! Independent verdicts for the wave-8 a11y agreement-only case
//! (`bd-wave8-ports-agreement-only-debt-idns`).
//!
//! The Python oracle and the `cdcp_gate` port are gone (EXTRACT-THEN-DELETE,
//! jhd.19). Product is `cdcp_learn::a11y` / `cdcp smoke-a11y`. Moved out of
//! `cdcp_gate/tests` by bd-engine-not-gate-ar39.7 so the leftover plant
//! lives with the product caller.
//!
//! A case that only said "the two sides agree" evaporated with the second
//! side. This file keeps the one leftover call site and points it at a
//! PLANTED finding: resolved path (the named page), item COUNT
//! (finding-lines — a silent fallback to the live tree reports zero), and
//! the named finding.
//!
//! Known-bad: the same defect is planted in BOTH "implementations" (they
//! ignore the named root and scan the live tree). Agreement still holds.
//! The converted verdict does not.

use std::path::{Path, PathBuf};

const CSS: &str = "web/assets/css/course.css";
const REQUIRED: &[&str] = &[
    "index.html",
    "mock.html",
    "results.html",
    "learn.html",
    "drill.html",
    "quiz.html",
];

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
/// Honesty + landmark + stylesheet, no skip-link. Unique to this plant.
const PLANTED_QUIZ: &str = concat!(
    "<link href=\"assets/css/course.css\">",
    "<div class=\"honesty-banner\">study tool only</div>",
    "<main>y</main>",
);
const PLANTED_FINDING: &str = "web/quiz.html: missing skip link";

struct Run {
    code: i32,
    stdout: String,
}

fn engine_root() -> PathBuf {
    cdcp_learn::resolve_engine_root(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
}

/// Census comparator. One implementation remains; the verdict lives at the
/// call site, not in the agreement of two copies of the same function.
fn cmp(label: &str, root: &Path) -> Run {
    let o = cdcp_learn::a11y::run(root);
    assert!(
        o.artifact.is_none(),
        "[{label}] a reader must not propose a write"
    );
    Run {
        code: o.code,
        stdout: o.stdout,
    }
}

fn put(root: &Path, rel: &str, body: &str) {
    let p = root.join(rel);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(p, body).unwrap();
}

/// Six required shells + optional reference + CSS. quiz.html is the plant.
fn planted_quiz_skip_link() -> (tempfile::TempDir, PathBuf) {
    let td = tempfile::tempdir().unwrap();
    let root = td.path().join("r");
    put(&root, CSS, GOOD_CSS);
    for name in REQUIRED.iter().chain(["reference.html"].iter()) {
        let body = if *name == "quiz.html" {
            PLANTED_QUIZ
        } else {
            GOOD_PAGE
        };
        put(&root, &format!("web/{name}"), body);
    }
    (td, root)
}

fn finding_lines(out: &str) -> usize {
    out.lines().filter(|l| l.starts_with("  - ")).count()
}

/// THE converted case. Under the old assertion this was a bare `cmp` against
/// a green tree: both sides agreed, neither was asked whether the bytes were
/// right, and a shared fallback to the live shells would have stayed green.
#[test]
fn planted_quiz_skip_link_is_red_and_names_the_page() {
    let (_td, root) = planted_quiz_skip_link();
    let rs = cmp("planted quiz skip-link", &root);
    let out = &rs.stdout;

    assert_ne!(rs.code, 0, "planted skip-link must be RED:\n{out}");
    // 1. resolved path — the detector names THIS page, not a live-tree page
    assert!(
        out.contains(PLANTED_FINDING),
        "[{}] the detector did not name the planted page: {out}",
        root.display()
    );
    // 2. item COUNT — one finding-line. A silent fallback to the live tree
    //    is GREEN (zero `  - ` lines) and would pass an exit-only check.
    assert_eq!(
        finding_lines(out),
        1,
        "expected exactly the one planted finding, not a different tree: {out}"
    );
    // 3. named finding + no success token
    assert!(
        out.starts_with("FAIL: smoke_a11y\n"),
        "the fail token must open the report: {out}"
    );
    assert!(
        !out.contains("PASS: smoke_a11y"),
        "PASS must not appear on the planted path: {out}"
    );
}

/// Known-bad: plant the fallback defect in BOTH implementations. They agree
/// (old assertion: pass). The converted verdict does not (new: fail).
#[test]
fn known_bad_shared_fallback_passes_agreement_and_fails_the_converted_verdict() {
    let (_td, root) = planted_quiz_skip_link();
    // BOTH sides ignore the named root and scan the live tree — the shared
    // defect an agreement-only case cannot see.
    let py = cdcp_learn::a11y::run(&engine_root());
    let rs = cdcp_learn::a11y::run(&engine_root());

    assert_eq!(py.code, rs.code);
    assert_eq!(py.stdout, rs.stdout);
    assert_eq!(
        py.code, 0,
        "live fallback is GREEN — the old assertion would pass:\n{}",
        py.stdout
    );

    let converted_trips =
        rs.code != 0 && rs.stdout.contains(PLANTED_FINDING) && finding_lines(&rs.stdout) == 1;
    assert!(
        !converted_trips,
        "shared fallback must fail the converted verdict; got:\n{}",
        rs.stdout
    );
    // Honesty: the named root still has the plant. The defect is the scan
    // target, not a missing fixture.
    let honest = cmp("control: honest scan of the plant", &root);
    assert_ne!(honest.code, 0, "{}", honest.stdout);
    assert!(honest.stdout.contains(PLANTED_FINDING), "{}", honest.stdout);
}
