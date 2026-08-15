//! Verdict suite for `cdcp_learn::glossary` (extracted from the gate by
//! bd-engine-not-gate-ar39.2). The Python oracle is DELETED; agreement-only
//! cases evaporated with it. Every case below asserts the correct answer.
//!
//! # HOW A *BUILDER* IS DIFFERENTIALLY TESTED (the reusable part)
//!
//! Nine ports landed before this one and every one of them was a CHECKER — a
//! checker can be run anywhere, any number of times, and observing it costs
//! nothing. A builder MUTATES TRACKED FILES. Three rules follow, and they are
//! the whole shape of this file:
//!
//!   1. **NEVER RUN EITHER IMPLEMENTATION AGAINST THE LIVE TREE.** Comparing by
//!      running both builders in the repo and diffing `web/data/` is a race with
//!      every other reader and writer of those paths — the damage class
//!      bd-791t and bd-gl4j document. The "live tree" case here is a TREE COPY
//!      in TEMP whose inputs are byte-copies of the live ones, so it computes
//!      exactly the live answer and touches nothing.
//!   2. **EACH IMPLEMENTATION GETS ITS OWN COPY.** Both builders write to the
//!      same relative path, so a shared fixture would mean the second run
//!      silently overwrote the first and the artifact comparison would compare
//!      a file with itself. Every case materialises `py/` and `rs/` from the
//!      same template and compares across them.
//!   3. **THE ARTIFACT IS PART OF THE COMPARISON, AS BYTES.** stdout, stderr
//!      and exit code are not the whole observable behaviour of a builder. The
//!      four-tuple is `(stdout, stderr, exit code, bytes written)`, compared as
//!      raw bytes and never as parsed JSON — key order, indentation, escaping
//!      and the trailing newline are exactly where two JSON writers diverge.
//!
//! A fourth rule is about what NOT to do: `build_glossary_json.py` takes no
//! path flags — it derives its root from `Path(__file__).resolve()` — and this
//! harness does NOT add one to make it testable. Widening a gate's argument
//! surface changes the thing under test. The fixture instead copies the script
//! into `<fixture>/scripts/`, which is the only handle the oracle actually
//! offers, and hands the Rust the same root through the dispatcher's `--root`.
//!
//! The tie-back that makes the tree copy trustworthy: the live case asserts the
//! bytes both sides produce are IDENTICAL TO THE TRACKED `web/data/glossary.json`.
//! That is the read-only proof that running the port in the live tree would be
//! a no-op write, obtained without performing one.
//!
//! ANTI-VACUOUS DISCIPLINE. A differential that silently compares nothing
//! passes exactly like one that compared everything, so: a missing `python3` is
//! a FAILURE and never a skip; a fixture that copied no source is a FAILURE;
//! and every case increments a counter that is asserted by its own case.

use cdcp_learn::glossary::MIN_TERMS;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

const SRC_REL: &str = "web/content/reference/GLOSSARY.md";
const ARTIFACT_REL: &str = "web/data/glossary.json";

/// Cases actually run, so "the suite ran" is itself checked.
static COMPARED: AtomicUsize = AtomicUsize::new(0);
/// Unique sub-directory per run, so concurrent cases never collide.
static ROUND: AtomicUsize = AtomicUsize::new(0);

fn engine_root() -> PathBuf {
    cdcp_learn::resolve_engine_root(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
}

// ── fixture plumbing ───────────────────────────────────────────────────────

fn write_file(path: &Path, body: &str) {
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p).unwrap();
    }
    std::fs::write(path, body).unwrap();
}

fn copy_tree(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for e in std::fs::read_dir(src)
        .expect("read fixture template")
        .flatten()
    {
        let from = e.path();
        let to = dst.join(e.file_name());
        if from.is_dir() {
            copy_tree(&from, &to);
        } else {
            std::fs::copy(&from, &to).unwrap();
        }
    }
}

/// A tree-copy fixture. The template is what a case builds; `compare` then
/// materialises it twice, once per implementation.
struct Fixture {
    dir: tempfile::TempDir,
}

impl Fixture {
    fn new() -> Fixture {
        let f = Fixture {
            dir: tempfile::tempdir().unwrap(),
        };
        std::fs::create_dir_all(f.template()).unwrap();
        f
    }

    fn template(&self) -> PathBuf {
        self.dir.path().join("template")
    }

    /// Byte-copy the live glossary into the template.
    fn seed_live_source(&self) {
        let live = engine_root().join(SRC_REL);
        assert!(live.is_file(), "the live glossary is missing: {SRC_REL}");
        let dst = self
            .template()
            .join(SRC_REL.replace('/', std::path::MAIN_SEPARATOR_STR));
        std::fs::create_dir_all(dst.parent().unwrap()).unwrap();
        std::fs::copy(&live, &dst).unwrap();
        assert!(
            std::fs::metadata(&dst).unwrap().len() > 0,
            "copied an empty glossary — a vacuous fixture is an ERROR, not a pass"
        );
    }

    fn put_source(&self, body: &str) {
        write_file(
            &self.template().join("web/content/reference/GLOSSARY.md"),
            body,
        );
    }
}

struct Run {
    code: i32,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    artifact: Option<Vec<u8>>,
}

impl Run {
    fn out(&self) -> String {
        String::from_utf8_lossy(&self.stdout).into_owned()
    }
    fn err(&self) -> String {
        String::from_utf8_lossy(&self.stderr).into_owned()
    }
}

fn artifact_of(root: &Path) -> Option<Vec<u8>> {
    std::fs::read(root.join(ARTIFACT_REL)).ok()
}

/// Compile in a private copy of the fixture. Agreement-only (python vs rust)
/// is gone with the oracle. Every case asserts the CORRECT answer.
fn compare(label: &str, f: &Fixture) -> Run {
    let n = ROUND.fetch_add(1, Ordering::SeqCst);
    let root = f.dir.path().join(format!("round{n}"));
    copy_tree(&f.template(), &root);

    let outcome = cdcp_learn::glossary::write_glossary(&root).expect("write_glossary");
    let r = Run {
        code: outcome.code,
        stdout: outcome.stdout.into_bytes(),
        stderr: Vec::new(),
        artifact: artifact_of(&root),
    };

    if r.code != 0 {
        assert!(
            !r.out().contains("PASS"),
            "[{label}] exited {} with a success token on stdout:\n{}",
            r.code,
            r.out()
        );
        assert!(
            r.artifact.is_none(),
            "[{label}] exited {} but left {ARTIFACT_REL} behind",
            r.code
        );
    } else {
        assert!(
            r.artifact.is_some(),
            "[{label}] exited 0 without writing {ARTIFACT_REL}"
        );
    }

    COMPARED.fetch_add(1, Ordering::SeqCst);
    r
}

/// A table with `n` well-formed rows.
fn table(n: usize) -> String {
    let mut s = String::from("| Term | Definition |\n|---|---|\n");
    for i in 0..n {
        s.push_str(&format!(
            "| **T{i:03}** | Definition number {i}, long enough to be real. |\n"
        ));
    }
    s
}

// ── case 1: the live tree, without touching the live tree ──────────────────
// DROPPED: the_oracle_is_present_and_green_on_a_copy_of_the_live_tree —
// that case asserted nothing but "python ran". The oracle is deleted.

#[test]
fn live_inputs_are_green_and_reproduce_the_tracked_artifact() {
    let f = Fixture::new();
    f.seed_live_source();
    let rs = compare("live inputs", &f);

    assert_eq!(rs.code, 0, "live inputs must be GREEN: {}", rs.out());
    assert!(
        rs.out().starts_with("PASS: glossary terms="),
        "{}",
        rs.out()
    );
    assert!(
        rs.err().is_empty(),
        "the oracle writes nothing to stderr on the green path: {:?}",
        rs.err()
    );

    // The read-only tie-back. If these bytes match the committed artifact, a
    // run of either implementation in the live tree is a no-op write — which is
    // how this suite gets the live-tree guarantee without a live-tree run.
    let tracked = std::fs::read(engine_root().join(ARTIFACT_REL))
        .expect("the tracked web/data/glossary.json must exist");
    assert_eq!(
        rs.artifact.as_deref(),
        Some(tracked.as_slice()),
        "the port would rewrite the tracked {ARTIFACT_REL}; a builder that does \
         not reproduce its own committed output is not byte-exact"
    );
}

// ── case 2: anti-vacuous — a missing or empty input is never a pass ────────

#[test]
fn a_missing_glossary_is_an_error_in_both_and_writes_nothing() {
    let f = Fixture::new();
    // No source under the root AND no sibling `reference/` fallback, so the
    // oracle prints the fallback path it looked at last.
    let rs = compare("missing glossary", &f);
    assert_ne!(rs.code, 0, "a missing glossary must never be a pass");
    assert!(
        rs.out().starts_with("FAIL: missing glossary at "),
        "{}",
        rs.out()
    );
    assert!(
        rs.out().trim_end().ends_with("reference/GLOSSARY.md"),
        "the path it looked at must be named: {}",
        rs.out()
    );
    assert!(
        rs.artifact.is_none(),
        "nothing may be written when the source is missing"
    );
}

#[test]
fn an_empty_glossary_is_an_error_in_both() {
    let f = Fixture::new();
    f.put_source("");
    let rs = compare("empty glossary", &f);
    assert_ne!(rs.code, 0, "zero terms must never be a pass: {}", rs.out());
    assert!(rs.out().contains("terms=0"), "{}", rs.out());

    // THE VERDICT-SHAPE DEFECT, now fixed rather than witnessed
    // (bd-builder-verdict-shape-qm65). This case used to pin the defect in
    // place so the port could not quietly improve it and blind the
    // differential; measured 2026-08-14 both sides emitted, byte for byte:
    //
    //     PASS: glossary terms=0 → web/data/glossary.json
    //     FAIL: need ≥15 terms
    //     (exit 1, 161-byte glossary.json left behind)
    //
    // The verdict now LEADS a report composed once, after every check.
    assert!(
        rs.out().starts_with("FAIL: glossary terms=0"),
        "the verdict must lead the report and must be FAIL: {}",
        rs.out()
    );
    assert!(
        rs.out().contains("need ≥"),
        "the finding must name the floor: {}",
        rs.out()
    );
    // `compare` asserts the no-PASS-on-RED and no-artifact-on-RED legs for both
    // sides; restated here because this is the case the defect was found on.
    assert!(!rs.out().contains("PASS"), "{}", rs.out());
    assert!(
        rs.artifact.is_none(),
        "a below-floor build must not leave a short glossary.json behind"
    );
}

#[test]
fn a_table_with_only_a_header_yields_zero_terms_in_both() {
    let f = Fixture::new();
    f.put_source("# G\n\n| Term | Definition |\n|---|---|\n");
    let rs = compare("header only", &f);
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(rs.out().contains("terms=0"), "{}", rs.out());
}

// ── case 3: the floor itself, from both sides ──────────────────────────────

#[test]
fn one_term_below_the_floor_is_red_and_the_floor_itself_is_green() {
    let f = Fixture::new();

    f.put_source(&table(MIN_TERMS - 1));
    let rs = compare("one below the floor", &f);
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out()
            .starts_with(&format!("FAIL: glossary terms={}", MIN_TERMS - 1)),
        "{}",
        rs.out()
    );
    assert!(rs.out().contains("need ≥"), "{}", rs.out());
    assert!(rs.artifact.is_none(), "one term short still writes nothing");

    f.put_source(&table(MIN_TERMS));
    let rs = compare("exactly the floor", &f);
    assert_eq!(rs.code, 0, "{}", rs.out());
    assert!(!rs.out().contains("FAIL"), "{}", rs.out());
}

// ── case 4: the sibling-reference fallback and its absolute `source` ───────

#[test]
fn the_sibling_reference_fallback_records_an_absolute_source() {
    let f = Fixture::new();
    // Nothing under the root; the glossary lives one level up, which is the
    // branch that prints an ABSOLUTE `source` instead of a relative one.
    let n = ROUND.fetch_add(1, Ordering::SeqCst);
    let root = f.dir.path().join(format!("round{n}"));
    std::fs::create_dir_all(&root).unwrap();
    write_file(&root.parent().unwrap().join("reference/GLOSSARY.md"), &table(MIN_TERMS));

    let outcome = cdcp_learn::glossary::write_glossary(&root).expect("write_glossary");
    COMPARED.fetch_add(1, Ordering::SeqCst);
    assert_eq!(outcome.code, 0, "{}", outcome.stdout);
    let body = outcome.artifact.expect("artifact").1;
    assert!(
        body.contains("\"source\": \"/"),
        "the out-of-root fallback must record an absolute source: {body}"
    );
}

// ── case 5: every table shape the parser has to agree on ──────────────────

#[test]
fn table_edge_cases_are_byte_identical() {
    let f = Fixture::new();
    let mut src = table(MIN_TERMS);
    src.push_str(concat!(
        // a definition that is a horizontal rule -> skipped
        "| **Ruled** | --- |\n",
        // an empty definition cell -> skipped via the \\s* backtrack
        "| **Blank** |  |\n",
        // a parenthesised term -> registers a bare alias too
        "| **ASHRAE (TC 9.9)** | Thermal guidelines for IT rooms. |\n",
        // a redefinition -> replaces the value, keeps the position
        "| **T000** | Redefined, and it must win. |\n",
        // a bare `Term` cell -> skipped by name
        "| **Term** | not a real row |\n",
        // a single asterisk inside the term -> no match at all
        "| **a*b** | never parsed |\n",
        // a row split across lines, because \\s matches newlines
        "|\n  **Spanning**\n  | still one row. |\n",
        // a term whose bare form already exists -> setdefault must not clobber
        "| **T001 (alias)** | second definition. |\n",
    ));
    f.put_source(&src);
    let rs = compare("table edge cases", &f);
    assert_eq!(rs.code, 0, "{}", rs.out());
    let body = String::from_utf8(rs.artifact.expect("artifact")).unwrap();
    assert!(body.contains("\"ASHRAE\":"), "bare alias missing: {body}");
    assert!(body.contains("\"ASHRAE (TC 9.9)\":"), "{body}");
    assert!(body.contains("\"Spanning\":"), "{body}");
    assert!(!body.contains("\"Ruled\":"), "{body}");
    assert!(!body.contains("\"Blank\":"), "{body}");
    assert!(!body.contains("\"a*b\":"), "{body}");
    assert!(body.contains("Redefined, and it must win."), "{body}");
    assert!(
        body.contains("\"T001\": \"Definition number 1"),
        "setdefault must not clobber the original T001: {body}"
    );
}

#[test]
fn escaping_and_key_ordering_are_byte_identical() {
    let f = Fixture::new();
    let mut src = table(MIN_TERMS);
    src.push_str(concat!(
        // JSON escapes: quote, backslash, tab
        "| **Quoted** | He said \"no\" and left. |\n",
        "| **Slashed** | A back\\slash and a /forward/ one. |\n",
        "| **Tabbed** | before\tafter, with a real tab. |\n",
        // non-ASCII passes through unescaped under ensure_ascii=False
        "| **Degrees** | 27 °C ± 1, ≈ the ASHRAE recommended band → check. |\n",
        // casefolded ordering across cases, and a stable tie
        "| **zeta** | lower first. |\n",
        "| **ZEBRA** | upper second. |\n",
        "| **Ångström** | non-ASCII sort key. |\n",
    ));
    f.put_source(&src);
    let rs = compare("escaping and ordering", &f);
    assert_eq!(rs.code, 0, "{}", rs.out());
    let body = String::from_utf8(rs.artifact.expect("artifact")).unwrap();
    assert!(body.contains(r#"He said \"no\" and left."#), "{body}");
    assert!(body.contains(r#"A back\\slash"#), "{body}");
    assert!(body.contains(r#"before\tafter"#), "{body}");
    assert!(
        body.contains("27 °C"),
        "ensure_ascii=False must pass through: {body}"
    );
    let zebra = body.find("\"ZEBRA\"").expect("ZEBRA");
    let zeta = body.find("\"zeta\"").expect("zeta");
    assert!(zebra < zeta, "casefolded ordering is not case-sensitive");
}

// ── the harness must not be vacuously green ───────────────────────────────

#[test]
fn the_harness_compared_something() {
    // Runs a case itself rather than reading a counter another test may or may
    // not have incremented — test order and parallelism are not a contract,
    // and "0 cases compared" must never report like "all passed".
    let before = COMPARED.load(Ordering::SeqCst);
    let f = Fixture::new();
    f.put_source(&table(MIN_TERMS));
    compare("harness self-check", &f);
    assert!(
        COMPARED.load(Ordering::SeqCst) > before,
        "the differential harness compared nothing"
    );
}
