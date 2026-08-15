//! Differential harness: `scripts/verify_knowledge_paths.py` vs the
//! `verify-knowledge-paths` gate (bd-substrate-rust-migration-jhd.5).
//!
//! Every case here runs BOTH binaries over the SAME tree and asserts stdout,
//! stderr, and exit code are identical byte for byte. A port that merely agrees on
//! the verdict is not a port: `check.sh` logs, goldens, and any operator reading
//! the output all consume the bytes.
//!
//! Anti-vacuous (L4): the harness asserts its own case count and asserts the
//! oracle is present. A differential suite that silently ran zero comparisons
//! reports exactly like one where both sides agreed everywhere.

use std::path::{Path, PathBuf};
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_cdcp_gate");
const ORACLE_REL: &str = "scripts/verify_knowledge_paths.py";

/// Number of `#[test]` cases this file must carry. Raise it when you add one; a
/// DROP means a case was deleted, and a differential suite that quietly shrank
/// reports exactly like one where both sides agreed everywhere. `>=`, so adding is
/// free and removing is loud.
const EXPECTED_CASES: usize = 28;

fn engine_root() -> PathBuf {
    cdcp_gate::root::resolve(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
}

/// The oracle's source, read from the live repo. Missing is an ERROR, not a skip.
fn oracle_source() -> String {
    let p = engine_root().join(ORACLE_REL);
    std::fs::read_to_string(&p).unwrap_or_else(|e| {
        panic!(
            "{}: the differential oracle must stay in the tree ({e})",
            p.display()
        )
    })
}

/// One synthetic study tree: `<tmp>/study/engine` is the engine ROOT and
/// `<tmp>/study/modules` is the parent corpus, matching the real layout the gate
/// resolves `../modules/...` against.
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
        std::fs::create_dir_all(root.join("knowledge")).unwrap();
        std::fs::create_dir_all(root.join("scripts")).unwrap();
        std::fs::create_dir_all(root.join("registries")).unwrap();
        std::fs::create_dir_all(&corpus).unwrap();
        std::fs::write(root.join("registries/claims.toml"), "schema_version = 1\n").unwrap();
        // The oracle derives ROOT from its own location, so it has to live in the
        // fixture rather than be pointed at it.
        std::fs::write(root.join(ORACLE_REL), oracle_source()).unwrap();
        Tree {
            _dir: dir,
            root,
            corpus,
        }
    }

    fn domains(self, body: &str) -> Self {
        std::fs::write(self.root.join("knowledge/domains.toml"), body).unwrap();
        self
    }

    fn knowledge(self, name: &str, body: &str) -> Self {
        std::fs::write(self.root.join("knowledge").join(name), body).unwrap();
        self
    }

    fn note(self, name: &str) -> Self {
        let p = self.corpus.join(name);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, "# notes\n").unwrap();
        self
    }

    fn engine_note(self, rel: &str) -> Self {
        let p = self.root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, "# stray copy\n").unwrap();
        self
    }
}

/// A tree whose two domains both resolve — the baseline the RED cases perturb.
fn green_tree() -> Tree {
    Tree::new().note("01-a.md").note("02-b.md").domains(
        "[[domain]]\nid = \"01-a\"\nprimary_notes = \"../modules/01-a.md\"\n\n\
             [[domain]]\nid = \"02-b\"\nprimary_notes = \"../modules/02-b.md\"\n",
    )
}

#[derive(Debug)]
struct Run {
    code: i32,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

fn run_python(root: &Path) -> Run {
    let out = Command::new("python3")
        .arg(root.join(ORACLE_REL))
        .output()
        .expect("python3 must be available for the differential oracle");
    Run {
        code: out.status.code().unwrap_or(-1),
        stdout: out.stdout,
        stderr: out.stderr,
    }
}

fn run_rust(root: &Path) -> Run {
    let out = Command::new(BIN)
        .arg("--root")
        .arg(root)
        .arg("verify-knowledge-paths")
        .output()
        .expect("gate binary");
    Run {
        code: out.status.code().unwrap_or(-1),
        stdout: out.stdout,
        stderr: out.stderr,
    }
}

fn show(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

/// The whole acceptance bar, in one assertion: same bytes out, same bytes on
/// stderr, same status. Returns the (agreed) run so a case can additionally pin
/// what that agreement WAS — two sides agreeing on the wrong thing is still wrong.
#[must_use]
fn assert_identical(root: &Path, case: &str) -> Run {
    let py = run_python(root);
    let rs = run_rust(root);
    assert_eq!(
        show(&py.stdout),
        show(&rs.stdout),
        "[{case}] stdout differs\n--- python ---\n{}\n--- rust ---\n{}",
        show(&py.stdout),
        show(&rs.stdout)
    );
    assert_eq!(
        py.stdout, rs.stdout,
        "[{case}] stdout differs in raw bytes despite matching as text"
    );
    assert_eq!(
        show(&py.stderr),
        show(&rs.stderr),
        "[{case}] stderr differs"
    );
    assert_eq!(py.stderr, rs.stderr, "[{case}] stderr differs in raw bytes");
    assert_eq!(
        py.code,
        rs.code,
        "[{case}] exit code differs (python {} vs rust {})\npython stdout:\n{}\nrust stdout:\n{}",
        py.code,
        rs.code,
        show(&py.stdout),
        show(&rs.stdout)
    );
    py
}

// ── case 1: the live repo ──────────────────────────────────────────────────

#[test]
fn live_repo_tree_agrees_and_is_green() {
    let root = engine_root();
    let py = run_python(&root);
    let rs = run_rust(&root);
    assert_eq!(
        show(&py.stdout),
        show(&rs.stdout),
        "live tree stdout differs"
    );
    assert_eq!(py.stderr, rs.stderr, "live tree stderr differs");
    assert_eq!(py.code, rs.code, "live tree exit code differs");
    assert_eq!(
        py.code,
        0,
        "the live tree is expected GREEN:\n{}",
        show(&py.stdout)
    );
    assert!(
        show(&py.stdout).starts_with("PASS\n"),
        "{}",
        show(&py.stdout)
    );
    // Module 15 is checked like every other row: no hardcoded bound here (bd-lt7).
    assert!(
        show(&py.stdout).contains("primary_notes_checked=15"),
        "a module-count change is a real event, not a test to relax:\n{}",
        show(&py.stdout)
    );
}

// ── case 2: a primary_notes pointing at a file that does not exist ─────────

#[test]
fn a_nonexistent_target_is_red_in_both() {
    let t = Tree::new().note("01-a.md").domains(
        "[[domain]]\nid = \"01-a\"\nprimary_notes = \"../modules/01-a.md\"\n\n\
             [[domain]]\nid = \"02-b\"\nprimary_notes = \"../modules/does-not-exist.md\"\n",
    );
    let r = assert_identical(&t.root, "nonexistent target");
    assert_eq!(r.code, 1);
    let s = show(&r.stdout);
    assert!(s.starts_with("FAIL\n"), "{s}");
    assert!(
        s.contains("02-b: primary_notes does not resolve to a file: '../modules/does-not-exist.md' (resolved "),
        "{s}"
    );
    // The `(resolved ...)` form is realpath output: no `..` survives.
    assert!(
        !s.contains("/../"),
        "resolve() emulation leaked a `..`:\n{s}"
    );
}

#[test]
fn a_target_that_is_a_directory_is_not_a_file() {
    let t = green_tree();
    std::fs::create_dir_all(t.corpus.join("03-dir.md")).unwrap();
    let t = t.domains("[[domain]]\nid = \"03-d\"\nprimary_notes = \"../modules/03-dir.md\"\n");
    let r = assert_identical(&t.root, "directory target");
    assert_eq!(r.code, 1);
    assert!(
        show(&r.stdout).contains("does not resolve to a file"),
        "{}",
        show(&r.stdout)
    );
}

#[test]
fn a_symlink_to_a_real_note_resolves_through() {
    let t = green_tree().note("real.md");
    std::os::unix::fs::symlink(t.corpus.join("real.md"), t.corpus.join("link.md")).unwrap();
    let t = t.domains("[[domain]]\nid = \"01-a\"\nprimary_notes = \"../modules/link.md\"\n");
    let r = assert_identical(&t.root, "symlinked note");
    assert_eq!(r.code, 0, "{}", show(&r.stdout));
    let s = show(&r.stdout);
    assert!(s.starts_with("PASS\n"), "{s}");
    assert!(
        s.contains("primary_notes_checked=1"),
        "the symlink must be followed and counted:\n{s}"
    );
}

#[test]
fn a_dangling_symlink_reports_the_link_target_path() {
    let t = green_tree();
    std::os::unix::fs::symlink(t.corpus.join("gone.md"), t.corpus.join("dangle.md")).unwrap();
    let t = t.domains("[[domain]]\nid = \"01-a\"\nprimary_notes = \"../modules/dangle.md\"\n");
    let r = assert_identical(&t.root, "dangling symlink");
    assert_eq!(r.code, 1);
    assert!(
        show(&r.stdout).contains("gone.md)"),
        "resolve() must follow the link before reporting:\n{}",
        show(&r.stdout)
    );
}

// ── case 3: empty / malformed primary_notes ───────────────────────────────

#[test]
fn an_empty_primary_notes_without_the_licence_is_red_in_both() {
    let t = green_tree().domains(
        "[[domain]]\nid = \"01-a\"\nprimary_notes = \"\"\n\n\
         [[domain]]\nid = \"02-b\"\nprimary_notes = \"   \"\n",
    );
    let r = assert_identical(&t.root, "empty primary_notes");
    assert_eq!(r.code, 1);
    let s = show(&r.stdout);
    assert!(
        s.contains("  - 01-a: empty primary_notes without exam_weight_unknown=true\n"),
        "{s}"
    );
    assert!(
        s.contains("  - 02-b: empty primary_notes without exam_weight_unknown=true\n"),
        "whitespace-only must strip to empty:\n{s}"
    );
}

#[test]
fn an_empty_primary_notes_with_the_licence_is_allowed_in_both() {
    let t = green_tree().domains(
        "[[domain]]\nid = \"01-a\"\nprimary_notes = \"../modules/01-a.md\"\n\n\
         [[domain]]\nid = \"15-ops\"\nprimary_notes = \"\"\nexam_weight_unknown = true\n",
    );
    let r = assert_identical(&t.root, "licensed empty");
    assert_eq!(r.code, 0, "{}", show(&r.stdout));
    assert!(
        show(&r.stdout).contains("empty_allowed=1"),
        "{}",
        show(&r.stdout)
    );
}

#[test]
fn a_truthy_but_non_boolean_licence_does_not_count_in_either() {
    let t = green_tree()
        .domains("[[domain]]\nid = \"01-a\"\nprimary_notes = \"\"\nexam_weight_unknown = 1\n");
    let r = assert_identical(&t.root, "non-boolean licence");
    assert_eq!(r.code, 1, "`is True` rejects 1:\n{}", show(&r.stdout));
    assert!(
        show(&r.stdout).contains("01-a: empty primary_notes without exam_weight_unknown=true"),
        "`exam_weight_unknown = 1` must not license an empty pointer:\n{}",
        show(&r.stdout)
    );
}

#[test]
fn a_missing_primary_notes_key_is_red_in_both() {
    let t = green_tree().domains("[[domain]]\nid = \"01-a\"\norder = 1\n");
    let r = assert_identical(&t.root, "missing key");
    assert_eq!(r.code, 1);
    assert!(
        show(&r.stdout).contains("  - 01-a: primary_notes field missing\n"),
        "{}",
        show(&r.stdout)
    );
}

#[test]
fn a_non_string_primary_notes_is_stringified_identically() {
    let t = green_tree().domains(
        "[[domain]]\nid = \"n-int\"\nprimary_notes = 123\n\n\
         [[domain]]\nid = \"n-bool\"\nprimary_notes = true\n\n\
         [[domain]]\nid = \"n-list\"\nprimary_notes = [\"a\", \"b\"]\n",
    );
    let r = assert_identical(&t.root, "non-string primary_notes");
    assert_eq!(r.code, 1);
    let s = show(&r.stdout);
    assert!(
        s.contains("n-int: primary_notes does not resolve to a file: '123' "),
        "{s}"
    );
    assert!(
        s.contains("n-bool: primary_notes does not resolve to a file: 'True' "),
        "{s}"
    );
    assert!(
        s.contains("n-list: primary_notes does not resolve to a file: \"['a', 'b']\" "),
        "{s}"
    );
}

#[test]
fn a_row_with_no_id_falls_back_to_missing_id_in_both() {
    let t = green_tree().domains("[[domain]]\nprimary_notes = \"../modules/gone.md\"\n");
    let r = assert_identical(&t.root, "no id");
    assert_eq!(r.code, 1);
    assert!(
        show(&r.stdout).contains("  - <missing-id>: "),
        "{}",
        show(&r.stdout)
    );
}

#[test]
fn a_value_needing_python_repr_escapes_identically() {
    let t =
        green_tree().domains("[[domain]]\nid = \"q\"\nprimary_notes = \"../mod's/it\\\"s.md\"\n");
    let r = assert_identical(&t.root, "repr escaping");
    assert_eq!(r.code, 1);
    assert!(
        show(&r.stdout).contains("does not resolve to a file: '../mod\\'s/it\"s.md'"),
        "{}",
        show(&r.stdout)
    );
}

// ── the registry-level early exits ────────────────────────────────────────

#[test]
fn a_missing_domains_toml_prints_the_same_single_line() {
    let t = Tree::new();
    let r = assert_identical(&t.root, "missing domains.toml");
    assert_eq!(r.code, 1);
    assert_eq!(show(&r.stdout), "FAIL: knowledge/domains.toml missing\n");
}

#[test]
fn zero_domain_rows_is_an_error_in_both_never_a_pass() {
    let t = Tree::new().domains("schema_version = 1\n");
    let r = assert_identical(&t.root, "zero rows");
    assert_eq!(r.code, 1);
    assert_eq!(
        show(&r.stdout),
        "FAIL: domains.toml has zero [[domain]] rows\n"
    );
}

#[test]
fn an_empty_domain_array_is_also_zero_rows_in_both() {
    let t = Tree::new().domains("domain = []\n");
    let r = assert_identical(&t.root, "empty domain array");
    assert_eq!(r.code, 1);
    assert_eq!(
        show(&r.stdout),
        "FAIL: domains.toml has zero [[domain]] rows\n"
    );
}

// ── the wrong-tree guard ──────────────────────────────────────────────────

#[test]
fn a_note_under_the_engines_own_modules_dir_is_red_in_both() {
    let t = green_tree()
        .engine_note("modules/01-a.md")
        .domains("[[domain]]\nid = \"01-a\"\nprimary_notes = \"modules/01-a.md\"\n");
    let r = assert_identical(&t.root, "engine-local modules/");
    assert_eq!(r.code, 1);
    let s = show(&r.stdout);
    assert!(
        s.contains("01-a: primary_notes resolves under course-engine/modules/ ("),
        "{s}"
    );
    assert!(
        s.contains("); parent corpus is ../modules/ relative to ROOT\n"),
        "{s}"
    );
}

#[test]
fn an_absolute_primary_notes_is_used_verbatim_in_both() {
    let t = green_tree();
    let abs = t.corpus.join("01-a.md");
    let missing = t.corpus.join("nope.md");
    let t = t.domains(&format!(
        "[[domain]]\nid = \"ok\"\nprimary_notes = \"{}\"\n\n\
         [[domain]]\nid = \"bad\"\nprimary_notes = \"{}\"\n",
        abs.display(),
        missing.display()
    ));
    let r = assert_identical(&t.root, "absolute paths");
    assert_eq!(r.code, 1);
    assert!(
        show(&r.stdout).contains("bad: primary_notes does not resolve"),
        "{}",
        show(&r.stdout)
    );
    assert!(!show(&r.stdout).contains("ok: "), "{}", show(&r.stdout));
}

// ── the second leg: the line scan over other knowledge/*.toml ─────────────

#[test]
fn other_knowledge_files_are_line_scanned_identically() {
    let t = green_tree().knowledge(
        "topics.toml",
        "# header\n\n[[topic]]\nprimary_notes = \"../modules/nope.md\"\n",
    );
    let r = assert_identical(&t.root, "second leg line numbers");
    assert_eq!(r.code, 1);
    assert!(
        show(&r.stdout)
            .contains("  - topics.toml:4: primary_notes '../modules/nope.md' does not resolve\n"),
        "{}",
        show(&r.stdout)
    );
}

#[test]
fn second_leg_quoting_prefixes_and_comments_match_byte_for_byte() {
    let t = green_tree().knowledge(
        "a.toml",
        "primary_notes_extra = \"../modules/nope.md\"\n\
         primary_notes = '../modules/01-a.md'\n\
         primary_notes\n\
         primary_notes = \"../modules/01-a.md\"  # trailing\n\
           primary_notes   =   \"../modules/02-b.md\"\n\
         primary_notes = \"\"\n",
    );
    let r = assert_identical(&t.root, "second leg quoting");
    assert_eq!(r.code, 1);
    let s = show(&r.stdout);
    assert!(
        s.contains("a.toml:1: primary_notes '../modules/nope.md' does not resolve"),
        "{s}"
    );
    assert!(
        s.contains("a.toml:4: primary_notes '../modules/01-a.md\"  # trailing' does not resolve"),
        "only one quote layer is stripped, so the comment stays in the value:\n{s}"
    );
    assert_eq!(
        s.lines().filter(|l| l.starts_with("  - ")).count(),
        2,
        "lines 2, 3, 5 and 6 must produce nothing:\n{s}"
    );
}

#[test]
fn second_leg_emission_order_follows_sorted_glob_then_line_number() {
    let t = green_tree()
        .knowledge(
            "zz.toml",
            "primary_notes = \"../modules/z1.md\"\nprimary_notes = \"../modules/z2.md\"\n",
        )
        .knowledge("aa.toml", "primary_notes = \"../modules/a1.md\"\n")
        .knowledge("mm.toml", "primary_notes = \"../modules/m1.md\"\n");
    let r = assert_identical(&t.root, "second leg ordering");
    assert_eq!(r.code, 1);
    let s = show(&r.stdout);
    let order: Vec<&str> = s
        .lines()
        .filter(|l| l.starts_with("  - "))
        .map(|l| l.trim_start_matches("  - "))
        .collect();
    assert_eq!(order.len(), 4, "{s}");
    assert!(order[0].starts_with("aa.toml:1"), "{s}");
    assert!(order[1].starts_with("mm.toml:1"), "{s}");
    assert!(order[2].starts_with("zz.toml:1"), "{s}");
    assert!(order[3].starts_with("zz.toml:2"), "{s}");
}

#[test]
fn second_leg_line_numbering_survives_exotic_line_breaks() {
    // `str.splitlines()` breaks on CR, CRLF and FF too; a `lines()` stand-in would
    // number the finding differently.
    let t = green_tree().knowledge(
        "b.toml",
        "one\rtwo\r\nthree\u{0c}primary_notes = \"../modules/nope.md\"\n",
    );
    let r = assert_identical(&t.root, "exotic line breaks");
    assert_eq!(r.code, 1);
    assert!(
        show(&r.stdout).contains("b.toml:4: primary_notes"),
        "{}",
        show(&r.stdout)
    );
}

#[test]
fn domains_toml_is_skipped_by_the_second_leg_in_both() {
    // Every domains.toml line starts with `primary_notes`; if the skip were lost
    // the findings would double.
    let t = green_tree();
    let r = assert_identical(&t.root, "domains.toml skipped");
    assert_eq!(r.code, 0, "{}", show(&r.stdout));
    let s = show(&r.stdout);
    assert!(s.starts_with("PASS\n"), "{s}");
    assert!(
        s.contains("primary_notes_checked=2"),
        "the first-leg pointers must still be counted:\n{s}"
    );
    assert!(
        !s.contains("  - "),
        "a lost skip would double-report domains.toml as second-leg findings:\n{s}"
    );
}

#[test]
fn non_toml_neighbours_and_case_variants_are_not_scanned_in_either() {
    let t = green_tree()
        .knowledge("notes.md", "primary_notes = \"../modules/nope.md\"\n")
        .knowledge("case.TOML", "primary_notes = \"../modules/nope.md\"\n")
        .knowledge(
            ".hidden.toml",
            "primary_notes = \"../modules/hidden-nope.md\"\n",
        );
    let r = assert_identical(&t.root, "glob selectivity");
    // The dotfile IS matched by pathlib's glob — that is the point of the case.
    assert_eq!(r.code, 1, "{}", show(&r.stdout));
    let s = show(&r.stdout);
    assert!(
        s.contains(".hidden.toml:1: "),
        "pathlib glob matches dotfiles:\n{s}"
    );
    assert_eq!(
        s.lines().filter(|l| l.starts_with("  - ")).count(),
        1,
        "{s}"
    );
}

// ── ordering of the two legs against each other ───────────────────────────

#[test]
fn first_leg_findings_precede_second_leg_findings_in_both() {
    let t = green_tree()
        .domains("[[domain]]\nid = \"01-a\"\nprimary_notes = \"../modules/gone.md\"\n")
        .knowledge("aa.toml", "primary_notes = \"../modules/also-gone.md\"\n");
    let r = assert_identical(&t.root, "leg ordering");
    assert_eq!(r.code, 1);
    let s = show(&r.stdout);
    let first = s.find("01-a: ").expect("domain finding");
    let second = s.find("aa.toml:1").expect("line-scan finding");
    assert!(first < second, "{s}");
}

// ── anti-vacuous: what the oracle does NOT check ──────────────────────────

/// FINDING (reported, deliberately NOT fixed): the oracle's only anti-vacuous
/// check is "zero `[[domain]]` rows". A registry whose every row is a licensed
/// empty passes with `primary_notes_checked=0` — zero pointers verified reports
/// exactly like every pointer landing. The port reproduces that hole; this case
/// pins it so closing it later is a visible, reviewed change on both sides.
#[test]
fn zero_primary_notes_checked_still_passes_in_both_which_is_the_hole() {
    let t = Tree::new().domains(
        "[[domain]]\nid = \"a\"\nprimary_notes = \"\"\nexam_weight_unknown = true\n\n\
         [[domain]]\nid = \"b\"\nprimary_notes = \"\"\nexam_weight_unknown = true\n",
    );
    let r = assert_identical(&t.root, "vacuous pass");
    assert_eq!(
        r.code, 0,
        "the oracle passes here; a port that failed would be a behaviour change"
    );
    assert!(
        show(&r.stdout).contains("primary_notes_checked=0"),
        "{}",
        show(&r.stdout)
    );
}

/// FINDING (reported, deliberately NOT fixed): the second leg is silent when it
/// scans zero files, so a `knowledge/` holding only `domains.toml` produces the
/// same output as one whose every neighbour resolved.
#[test]
fn zero_scanned_neighbours_is_silent_in_both_which_is_the_other_hole() {
    let t = green_tree();
    let r = assert_identical(&t.root, "zero neighbours");
    assert_eq!(r.code, 0, "{}", show(&r.stdout));
    assert!(
        show(&r.stdout).contains("primary_notes_checked=2"),
        "{}",
        show(&r.stdout)
    );
}

// ── the harness's own honesty ─────────────────────────────────────────────

#[test]
fn the_oracle_is_still_in_the_tree() {
    let src = oracle_source();
    assert!(
        src.contains("primary_notes"),
        "the file at {ORACLE_REL} is not the oracle this suite differentiates against"
    );
    // bd-lt7: the sibling defect is a hardcoded module bound. This script has none;
    // if one ever appears, the port must copy it and this assertion must be the
    // place that argument starts.
    for bound in ["range(1, 15)", "range(1,15)", "<= 14", "< 15"] {
        assert!(
            !src.contains(bound),
            "{ORACLE_REL} grew a hardcoded module bound {bound:?} (bd-lt7): port it byte-exact, do not fix it here"
        );
    }
}

#[test]
fn the_harness_actually_compared_something() {
    let src = include_str!("diff_verify_knowledge_paths.rs");
    // Whole lines only: a substring count would also match this very literal and
    // the doc comment above, inflating the total by two and letting a deleted case
    // slip under the bar.
    let cases = src.lines().filter(|l| l.trim() == "#[test]").count();
    assert!(
        cases >= EXPECTED_CASES,
        "{cases} differential cases, expected at least {EXPECTED_CASES} — a suite that shrank silently is an ERROR, not a pass"
    );

    // The three-way comparison must live in `assert_identical` ITSELF. Scanning the
    // whole file would let a deleted assertion hide behind an identical-looking
    // comparison in some other case, which is exactly how a harness rots into
    // stdout-only while still reporting green.
    let body = src
        .split_once("fn assert_identical")
        .expect("assert_identical must exist")
        .1
        .split_once("\n}\n")
        .expect("assert_identical must be a complete fn")
        .0;
    for probe in [
        "assert_eq!(\n        show(&py.stdout),\n        show(&rs.stdout),",
        "assert_eq!(\n        py.stdout, rs.stdout,",
        "assert_eq!(\n        show(&py.stderr),\n        show(&rs.stderr),",
        "assert_eq!(py.stderr, rs.stderr,",
        "assert_eq!(\n        py.code,\n        rs.code,",
    ] {
        assert!(
            body.contains(probe),
            "assert_identical stopped comparing — missing {probe:?}"
        );
    }
}
