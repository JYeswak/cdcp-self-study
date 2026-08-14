//! Differential harness: `cdcp_gate validate-grounding` against
//! `scripts/validate_grounding.py`.
//!
//! The Python script is the oracle for this port
//! (bd-substrate-rust-migration-jhd.9) and stays in the tree for exactly that
//! reason. Every case below runs BOTH implementations on the same inputs and
//! asserts stdout, stderr, and exit code match byte for byte. A disagreement on
//! any byte fails the port, not the oracle.
//!
//! # The case list
//!
//!   a) the live tree                         -> GREEN, `corpus_chars=659149`
//!   b) the live tree under a raised bar       -> the sample list and the
//!      `--strict-overlap` FAIL path, including the 60-finding truncation
//!   c) a synthetic uncited numeric setpoint   -> RED
//!   d) the same item with free evidence       -> the finding disappears
//!   e) a synthetic item with zero overlap     -> WARN, and RED under strict
//!   f) each clause pattern, each dump phrase, the fake multi-level cite
//!   g) anti-vacuous: zero items, a bank one item short of the floor, zero
//!      corpus characters, a corpus of seven characters, and each missing corpus
//!      root — every one of them RED and NAMED, plus the known-GOOD leg where
//!      the smallest legitimate tree is still green
//!   h) ordering and formatting: the sample list's stable sort, the
//!      `--sample-report` slice with a negative bound, `%.3f` on real scores
//!   i) argparse: `--help` at two widths, abbreviation, ambiguity, bad values,
//!      a missing value, `--`, and a token containing a space
//!
//! # ANTI-VACUOUS IS SATISFIED (bd-yje7), NOT PINNED
//!
//! Until 2026-08-14 the oracle exited 0 with a `PASS` banner on a bank holding
//! zero items, on a corpus of zero characters, and on a missing
//! `knowledge/corpus/public`; three cases here pinned that defect so fixing it
//! would have to be a deliberate edit on the oracle, the port and this ledger at
//! once. It was fixed in that order — oracle first, this harness red, then the
//! port — and the pins are gone, replaced by cases that assert the ERROR and the
//! text that names it.
//!
//! Two consequences for every fixture below. First, `Fixture::grounded()` now
//! carries `MIN_SCANNED_ITEMS` filler items and a corpus over `MIN_CORPUS_CHARS`
//! — a fixture under the floors is RED for a reason that has nothing to do with
//! the case it is testing. The filler items are built from the padding corpus's
//! own invented vocabulary, so they score 1.000 and never enter a low-overlap
//! sample list. Second, the floors themselves need a known-GOOD leg, because an
//! attack-only suite would pass just as well with the floors set absurdly high:
//! `the_smallest_legitimate_tree_is_still_green` is that leg.
//!
//! # HARNESS DISCIPLINE
//!
//! A differential that silently compares nothing passes exactly like one that
//! compared everything, so: a missing `python3` is a FAILURE and never a skip; a
//! copied oracle that is empty or differs from the repo's is a FAILURE; and
//! every case increments a counter that `the_harness_compared_something`
//! re-establishes for itself.
//!
//! # THE CORPUS IS NOT REPRODUCED HERE
//!
//! `knowledge/corpus/**` holds third-party captures, some `redistribution:
//! not-licensed`. No fixture in this file copies, quotes, or paraphrases any of
//! that text; the synthetic corpora below are invented strings, and the only
//! live-tree facts asserted are integers the oracle itself prints.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const BIN: &str = env!("CARGO_BIN_EXE_cdcp_gate");
const ORACLE_REL: &str = "scripts/validate_grounding.py";
const GATE: &str = "validate-grounding";

use cdcp_gate::gates::validate_grounding::{MIN_CORPUS_CHARS, MIN_SCANNED_ITEMS};

/// Invented vocabulary for the padding corpus and the padding items. Nothing
/// here comes from `knowledge/corpus/**`, and nothing here collides with the
/// invented tokens `unmoored_item` relies on scoring zero.
const PAD_WORDS: &str = "chiller plenum containment aisle rack economiser humidity \
                         envelope inlet redundancy topology maintainability concurrent";

/// Cases actually compared, so "the harness ran" is itself checked.
static COMPARED: AtomicUsize = AtomicUsize::new(0);

fn engine_root() -> PathBuf {
    cdcp_gate::root::resolve(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
}

struct Run {
    code: i32,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl Run {
    fn out(&self) -> String {
        String::from_utf8_lossy(&self.stdout).into_owned()
    }
    fn err(&self) -> String {
        String::from_utf8_lossy(&self.stderr).into_owned()
    }
}

/// Run the oracle that lives under `engine/scripts/`.
fn python(engine: &Path, args: &[&str], columns: Option<&str>) -> Run {
    let script = engine.join(ORACLE_REL);
    let mut cmd = Command::new("python3");
    cmd.current_dir(engine).arg(&script).args(args);
    match columns {
        Some(c) => cmd.env("COLUMNS", c),
        None => cmd.env_remove("COLUMNS"),
    };
    let out = cmd.output().unwrap_or_else(|e| {
        panic!(
            "python3 {} could not run ({e}). The oracle is REQUIRED: a differential \
             that cannot run its reference is a failure, never a skip.",
            script.display()
        )
    });
    Run {
        code: out.status.code().unwrap_or(-1),
        stdout: out.stdout,
        stderr: out.stderr,
    }
}

/// Run the port against the same engine root.
///
/// The built binary is invoked directly, never through `cargo run`: cargo writes
/// build diagnostics to stderr, and a sibling gate's warning landing in the
/// captured stream would read as a divergence that is not one.
fn rust(engine: &Path, args: &[&str], columns: Option<&str>) -> Run {
    let mut cmd = Command::new(BIN);
    cmd.current_dir(engine)
        .arg("--root")
        .arg(engine)
        .arg(GATE)
        .args(args);
    match columns {
        Some(c) => cmd.env("COLUMNS", c),
        None => cmd.env_remove("COLUMNS"),
    };
    let out = cmd.output().expect("run cdcp_gate");
    Run {
        code: out.status.code().unwrap_or(-1),
        stdout: out.stdout,
        stderr: out.stderr,
    }
}

/// The whole acceptance bar in one function. Returns the (identical) run so a
/// case can additionally assert *what* the shared output says.
fn compare_env(label: &str, engine: &Path, args: &[&str], columns: Option<&str>) -> Run {
    let py = python(engine, args, columns);
    let rs = rust(engine, args, columns);

    assert_eq!(
        py.stdout,
        rs.stdout,
        "[{label}] STDOUT differs.\n--- python ---\n{}\n--- rust ---\n{}",
        py.out(),
        rs.out()
    );
    assert_eq!(
        py.stderr,
        rs.stderr,
        "[{label}] STDERR differs.\n--- python ---\n{}\n--- rust ---\n{}",
        py.err(),
        rs.err()
    );
    assert_eq!(
        py.code, rs.code,
        "[{label}] EXIT CODE differs: python {} vs rust {}",
        py.code, rs.code
    );

    COMPARED.fetch_add(1, Ordering::SeqCst);
    rs
}

fn compare(label: &str, engine: &Path, args: &[&str]) -> Run {
    compare_env(label, engine, args, None)
}

// ── synthetic trees ────────────────────────────────────────────────────────

/// A throwaway engine root with its own copy of the oracle.
///
/// The oracle derives every path from `Path(__file__).resolve()`, so pointing it
/// at a fixture means copying the script beside a fixture `bank/`. The live tree
/// is never mutated. The copy is verified byte-for-byte against the repo's, so a
/// truncated copy cannot make both sides fail identically and read as green.
struct Fixture {
    dir: tempfile::TempDir,
}

impl Fixture {
    /// A tree with `bank/items/` and `knowledge/` present and empty.
    fn bare() -> Self {
        let f = Fixture {
            dir: tempfile::tempdir().expect("tempdir"),
        };
        std::fs::create_dir_all(f.engine().join("scripts")).unwrap();
        std::fs::create_dir_all(f.engine().join("bank/items")).unwrap();
        std::fs::create_dir_all(f.engine().join("knowledge")).unwrap();

        let src = engine_root().join(ORACLE_REL);
        let dst = f.engine().join(ORACLE_REL);
        std::fs::copy(&src, &dst).expect("copy the oracle into the fixture");
        let a = std::fs::read(&src).unwrap();
        let b = std::fs::read(&dst).unwrap();
        assert!(!a.is_empty(), "the repo's oracle is empty");
        assert_eq!(a, b, "the fixture's oracle copy is not the repo's oracle");
        f
    }

    /// `bare()` plus a small invented corpus, a topic registry, and enough
    /// padding to clear the anti-vacuous floors.
    fn grounded() -> Self {
        let f = Self::bare();
        f.write(
            "modules/m01-cooling.md",
            "cooling airflow containment rack aisle plenum chiller economiser\n",
        );
        f.write(
            "reference/glossary.md",
            "inlet temperature humidity envelope allowable recommended\n",
        );
        f.write(
            "engine/knowledge/topics.toml",
            "schema_version = 1\n\n[[topic]]\nid = \"t-one\"\nlabel = \"cooling airflow containment\"\n",
        );
        f.write(
            "engine/knowledge/corpus/public/src-invented.txt",
            "redundancy topology maintainability concurrent sustained\n",
        );
        f.pad_to_floor();
        f
    }

    /// The four corpus roots present, a corpus over `MIN_CORPUS_CHARS`, and
    /// `MIN_SCANNED_ITEMS` filler items — the smallest tree the gate will call
    /// non-vacuous, and nothing more.
    ///
    /// Every filler item's whole text is drawn from `PAD_WORDS`, which is also
    /// the padding corpus, so each scores exactly 1.000: the padding cannot add
    /// a low-overlap warn, a sample-list row, or a finding to any case below.
    fn pad_to_floor(&self) {
        let line = format!("{PAD_WORDS}\n");
        let reps = MIN_CORPUS_CHARS / line.len() + 2;
        self.write("modules/pad-corpus.md", &line.repeat(reps));
        self.write("reference/pad-glossary.md", &line);
        self.write("engine/knowledge/corpus/public/pad-invented.txt", &line);
        for i in 0..MIN_SCANNED_ITEMS {
            self.item(
                &format!("pad-{i:03}.toml"),
                &format!(
                    "id = \"pad-{i:03}\"\nstem = \"{PAD_WORDS}\"\nchoices = []\n\
                     explanation = \"\"\ntopic_ids = []\n"
                ),
            );
        }
    }

    fn engine(&self) -> PathBuf {
        self.dir.path().join("engine")
    }

    /// Write a file relative to the fixture root, so `modules/` and
    /// `reference/` (siblings of the engine root) are reachable.
    fn write(&self, rel: &str, body: &str) {
        let p = self.dir.path().join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    }

    fn item(&self, name: &str, body: &str) {
        self.write(&format!("engine/bank/items/{name}"), body);
    }
}

/// A well-formed, well-grounded item, used as the control in every needle case.
fn grounded_item(id: &str) -> String {
    format!(
        "id = \"{id}\"\n\
         stem = \"Which cooling arrangement uses containment at the rack?\"\n\
         choices = [\"Hot aisle containment\", \"Plenum chiller bypass\"]\n\
         explanation = \"Containment separates the cooling airflow from the aisle.\"\n\
         topic_ids = [\"t-one\"]\n"
    )
}

/// An item whose whole text is invented tokens, so nothing overlaps the corpus.
fn unmoored_item(id: &str) -> String {
    format!(
        "id = \"{id}\"\n\
         stem = \"Zzzzqq wwwwqq vvvvqq uuuuqq.\"\n\
         choices = [\"ttttqq\", \"ssssqq\"]\n\
         explanation = \"rrrrqq qqqqzz ppppzz\"\n\
         topic_ids = [\"t-absent\"]\n"
    )
}

// ── a) the live tree ───────────────────────────────────────────────────────

#[test]
fn the_live_tree_is_green_and_identical() {
    let root = engine_root();
    let rs = compare("live tree", &root, &[]);
    assert_eq!(rs.code, 0, "{}", rs.out());
    assert!(rs.out().starts_with("scanned_items="), "{}", rs.out());
    assert!(rs.out().contains("\nPASS\n"), "{}", rs.out());
    assert!(
        rs.out()
            .contains("  no high-severity hallucination heuristics\n"),
        "{}",
        rs.out()
    );
    assert!(rs.out().contains("\n  corpus_chars="), "{}", rs.out());
    // Anti-vacuous on the harness itself: a tree that scanned nothing must not
    // be mistaken for a tree that scanned everything and came back clean.
    assert!(
        !rs.out().contains("scanned_items=0\n"),
        "the live bank scanned zero items:\n{}",
        rs.out()
    );
    assert!(
        !rs.out().contains("corpus_chars=0\n"),
        "the live corpus is empty:\n{}",
        rs.out()
    );
}

// ── b) the live tree under a raised bar ────────────────────────────────────

#[test]
fn the_live_tree_under_a_raised_bar_matches_including_scores_and_truncation() {
    let root = engine_root();

    // The sample list: `%.3f` over real scores, stably sorted, then sliced.
    let rs = compare(
        "live tree, warn at 0.9",
        &root,
        &["--min-overlap", "0.9", "--sample-report", "5"],
    );
    assert_eq!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out().contains("lowest_overlap_samples:\n"),
        "{}",
        rs.out()
    );
    assert_eq!(
        rs.out().lines().filter(|l| l.starts_with("  0.")).count(),
        5,
        "--sample-report must cap the sample list:\n{}",
        rs.out()
    );
    assert!(
        rs.out().contains(" (use --strict-overlap to fail)\n"),
        "{}",
        rs.out()
    );

    // The FAIL path, past the 60-finding cap.
    let rs = compare(
        "live tree, strict at 0.95",
        &root,
        &["--strict-overlap", "--min-overlap", "0.95"],
    );
    assert_eq!(rs.code, 1, "{}", rs.out());
    assert!(rs.out().contains("\nFAIL\n"), "{}", rs.out());
    assert!(
        rs.out().contains(" more\n"),
        "this case must exercise the report truncation:\n{}",
        rs.out()
    );
}

// ── c/d) the numeric-setpoint heuristic ────────────────────────────────────

#[test]
fn an_uncited_numeric_setpoint_is_red_and_free_evidence_excuses_it() {
    let f = Fixture::grounded();
    f.item("a01.toml", &grounded_item("syn-ok"));
    f.item(
        "a02.toml",
        "id = \"syn-numeric\"\n\
         stem = \"Inlet air must be 22 C at the rack.\"\n\
         choices = [\"cooling\", \"airflow\"]\n\
         explanation = \"containment plenum chiller aisle\"\n\
         topic_ids = [\"t-one\"]\n\
         quantity_evidence = \"qualitative_only\"\n",
    );
    let rs = compare("uncited setpoint", &f.engine(), &[]);
    assert_eq!(rs.code, 1, "{}", rs.out());
    assert!(
        rs.out()
            .contains("  - syn-numeric: numeric setpoint without free/licensed evidence\n"),
        "{}",
        rs.out()
    );

    // Same text, evidence named: the finding goes away, so the detector is
    // reading `quantity_evidence` and not merely the digits.
    f.item(
        "a02.toml",
        "id = \"syn-numeric\"\n\
         stem = \"Inlet air must be 22 C at the rack.\"\n\
         choices = [\"cooling\", \"airflow\"]\n\
         explanation = \"containment plenum chiller aisle\"\n\
         topic_ids = [\"t-one\"]\n\
         quantity_evidence = \"free_url\"\n",
    );
    let rs = compare("setpoint with free evidence", &f.engine(), &[]);
    assert_eq!(rs.code, 0, "{}", rs.out());
    assert!(rs.out().contains("high_severity=0\n"), "{}", rs.out());

    // The recommended/required branch, which needs no trigger verb.
    f.item(
        "a02.toml",
        "id = \"syn-degrees\"\n\
         stem = \"Inlet 27 \u{b0}C recommended for the aisle.\"\n\
         choices = [\"cooling\", \"airflow\"]\n\
         explanation = \"containment plenum chiller\"\n\
         topic_ids = [\"t-one\"]\n",
    );
    let rs = compare("recommended setpoint", &f.engine(), &[]);
    assert_eq!(rs.code, 1, "{}", rs.out());
    assert!(
        rs.out()
            .contains("  - syn-degrees: numeric setpoint without free/licensed evidence\n"),
        "{}",
        rs.out()
    );
}

// ── e) the overlap heuristic ───────────────────────────────────────────────

#[test]
fn an_item_with_no_corpus_overlap_warns_and_fails_only_under_strict() {
    let f = Fixture::grounded();
    f.item("a01.toml", &grounded_item("syn-ok"));
    f.item("a02.toml", &unmoored_item("syn-unmoored"));

    let rs = compare("unmoored item, default", &f.engine(), &[]);
    assert_eq!(rs.code, 0, "{}", rs.out());
    assert!(rs.out().contains("low_overlap_warns=1\n"), "{}", rs.out());
    assert!(rs.out().contains("  0.000  syn-unmoored\n"), "{}", rs.out());
    assert!(
        rs.out()
            .contains("  warns=1 (use --strict-overlap to fail)\n"),
        "{}",
        rs.out()
    );

    let rs = compare("unmoored item, strict", &f.engine(), &["--strict-overlap"]);
    assert_eq!(rs.code, 1, "{}", rs.out());
    assert!(
        rs.out()
            .contains("  - syn-unmoored: low corpus overlap 0.000 < 0.08\n"),
        "{}",
        rs.out()
    );

    // `repr(float)` of a non-default bar reaches the message.
    let rs = compare(
        "unmoored item, strict at 0.5",
        &f.engine(),
        &["--strict-overlap", "--min-overlap", "0.5"],
    );
    assert!(
        rs.out()
            .contains("  - syn-unmoored: low corpus overlap 0.000 < 0.5\n"),
        "{}",
        rs.out()
    );
}

// ── f) the pattern heuristics ──────────────────────────────────────────────

#[test]
fn every_clause_dump_and_multilevel_pattern_matches_byte_for_byte() {
    let f = Fixture::grounded();
    let needles: &[(&str, &str, &str)] = &[
        (
            "family clause",
            "per ISO 22237 clause 4.1 the aisle is contained",
            "hallucinated-clause pattern: \\b(?:ISO|IEC|EN|ANSI|TIA|NFPA|IEEE)\\s*[\\...",
        ),
        (
            "bare clause",
            "see clause 5.2.1 for the containment rule",
            "hallucinated-clause pattern: \\bclause\\s+\\d+\\.\\d+(?:\\.\\d+)*\\b...",
        ),
        (
            "section sign",
            "annex\u{a7}3.4 covers the cooling plant",
            "hallucinated-clause pattern: \\b\u{a7}\\s*\\d+\\.\\d+...",
        ),
        (
            "dump actual",
            "this is an actual exam question about cooling",
            "dump-language: actual exam question",
        ),
        (
            "dump brain",
            "from a BRAIN DUMP of the cooling syllabus",
            "dump-language: brain\\s*dump",
        ),
        (
            "dump real",
            "taken from a real EPI exam on cooling",
            "dump-language: real EPI exam",
        ),
        (
            "dump pass",
            "a guaranteed pass for the cooling module",
            "dump-language: guaranteed pass",
        ),
        (
            "multi-level cite",
            "ISO/IEC 22237 requires 3.5.2 containment",
            "looks like fake multi-level clause cite",
        ),
    ];

    for (label, stem, expected) in needles {
        f.item("a01.toml", &grounded_item("syn-ok"));
        f.item(
            "a02.toml",
            &format!(
                "id = \"needle\"\nstem = \"{stem}\"\n\
                 choices = [\"cooling\", \"airflow\"]\n\
                 explanation = \"containment plenum chiller aisle\"\n\
                 topic_ids = [\"t-one\"]\n\
                 quantity_evidence = \"free_url\"\n"
            ),
        );
        let rs = compare(label, &f.engine(), &[]);
        assert_eq!(rs.code, 1, "[{label}] must go RED:\n{}", rs.out());
        assert!(
            rs.out().contains(&format!("  - needle: {expected}\n")),
            "[{label}] expected {expected:?} in:\n{}",
            rs.out()
        );
    }

    // KNOWN-GOOD leg: an attack-only suite ships an over-strict gate, and
    // over-strict gates get routed around. The control item alone stays green.
    let f2 = Fixture::grounded();
    f2.item("a01.toml", &grounded_item("syn-ok"));
    let rs = compare("control stays green", &f2.engine(), &[]);
    assert_eq!(rs.code, 0, "{}", rs.out());
    assert!(rs.out().contains("high_severity=0\n"), "{}", rs.out());
}

// ── g) anti-vacuous: each condition is RED and NAMES ITSELF ────────────────

/// The known-GOOD leg for the floors. Without it, the floors could be set to a
/// million and every RED case below would still pass — and an over-strict gate
/// gets routed around, which is a slower death than no gate.
#[test]
fn the_smallest_legitimate_tree_is_still_green() {
    let f = Fixture::bare();
    f.pad_to_floor();
    let rs = compare("smallest legitimate tree", &f.engine(), &[]);
    assert_eq!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out()
            .starts_with(&format!("scanned_items={MIN_SCANNED_ITEMS}\n")),
        "{}",
        rs.out()
    );
    assert!(rs.out().contains("\nPASS\n"), "{}", rs.out());
    assert!(rs.out().contains("low_overlap_warns=0\n"), "{}", rs.out());
}

#[test]
fn zero_items_is_an_error_that_names_itself() {
    let f = Fixture::grounded();
    for e in std::fs::read_dir(f.engine().join("bank/items"))
        .unwrap()
        .flatten()
    {
        std::fs::remove_file(e.path()).unwrap();
    }
    let rs = compare("empty bank", &f.engine(), &[]);
    assert_eq!(rs.code, 1, "a bank that was never scanned is not a pass");
    assert!(rs.out().starts_with("scanned_items=0\n"), "{}", rs.out());
    assert!(!rs.out().contains("PASS"), "{}", rs.out());
    assert!(
        rs.out().contains(&format!(
            "FAIL: vacuous grounding check\n  - scanned_items=0 < floor {MIN_SCANNED_ITEMS} "
        )),
        "{}",
        rs.out()
    );
}

/// A DELIBERATE floor, not `> 0`: one item short is still RED, and the message
/// prints both the count and the threshold.
#[test]
fn a_bank_one_item_short_of_the_floor_is_still_an_error() {
    let f = Fixture::grounded();
    let last = MIN_SCANNED_ITEMS - 1;
    std::fs::remove_file(f.engine().join(format!("bank/items/pad-{last:03}.toml"))).unwrap();
    let rs = compare("one item short", &f.engine(), &[]);
    assert_eq!(rs.code, 1, "{}", rs.out());
    assert!(
        rs.out().contains(&format!(
            "  - scanned_items={last} < floor {MIN_SCANNED_ITEMS} "
        )),
        "{}",
        rs.out()
    );
}

#[test]
fn zero_corpus_characters_is_an_error_that_names_itself() {
    // No `modules/`, no `reference/`, and a `knowledge/` with nothing readable:
    // three missing roots AND an empty corpus, every one of them named.
    let f = Fixture::bare();
    f.item("a01.toml", &grounded_item("syn-ok"));
    let rs = compare("empty corpus", &f.engine(), &[]);
    assert_eq!(rs.code, 1, "an empty corpus can contradict nothing");
    assert!(!rs.out().contains("PASS"), "{}", rs.out());
    assert!(
        rs.out()
            .contains(&format!("  - corpus_chars=0 < floor {MIN_CORPUS_CHARS} ")),
        "{}",
        rs.out()
    );
    for label in ["../modules", "../reference", "knowledge/corpus/public"] {
        assert!(
            rs.out()
                .contains(&format!("  - corpus root missing: {label}\n")),
            "[{label}] {}",
            rs.out()
        );
    }
}

/// Seven characters clear "non-empty" and are still RED. This is the case that
/// separates a recorded floor from a hole moved one byte to the left.
#[test]
fn a_corpus_of_seven_characters_does_not_satisfy_the_floor() {
    let f = Fixture::grounded();
    for (rel, body) in [
        ("modules/m01-cooling.md", "a"),
        ("modules/pad-corpus.md", "b"),
        ("reference/glossary.md", "c"),
        ("reference/pad-glossary.md", "d"),
    ] {
        f.write(rel, body);
    }
    std::fs::remove_file(f.engine().join("knowledge/topics.toml")).unwrap();
    std::fs::remove_file(f.engine().join("knowledge/corpus/public/src-invented.txt")).unwrap();
    std::fs::remove_file(f.engine().join("knowledge/corpus/public/pad-invented.txt")).unwrap();
    let rs = compare("seven-character corpus", &f.engine(), &[]);
    assert_eq!(rs.code, 1, "{}", rs.out());
    // Four one-character chunks joined by "\n".
    assert!(
        rs.out()
            .contains(&format!("  - corpus_chars=7 < floor {MIN_CORPUS_CHARS} ")),
        "{}",
        rs.out()
    );
}

#[test]
fn every_missing_corpus_root_is_named() {
    for (rel, label, sibling) in [
        ("modules", "../modules", true),
        ("reference", "../reference", true),
        ("knowledge/corpus/public", "knowledge/corpus/public", false),
    ] {
        let f = Fixture::grounded();
        let dir = if sibling {
            f.dir.path().join(rel)
        } else {
            f.engine().join(rel)
        };
        for e in std::fs::read_dir(&dir).unwrap().flatten() {
            std::fs::remove_file(e.path()).unwrap();
        }
        std::fs::remove_dir(&dir).unwrap();
        let rs = compare(&format!("missing {label}"), &f.engine(), &[]);
        assert_eq!(rs.code, 1, "[{label}] {}", rs.out());
        assert!(
            rs.out()
                .contains(&format!("  - corpus root missing: {label}\n")),
            "[{label}] {}",
            rs.out()
        );
    }
}

/// A root that exists but cannot be listed contributes zero characters in
/// silence, exactly like a missing one. Both sides must call it unreadable.
#[cfg(unix)]
#[test]
fn an_unlistable_corpus_root_is_named_too() {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::grounded();
    let dir = f.engine().join("knowledge/corpus/public");
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o000)).unwrap();
    let enforced = std::fs::read_dir(&dir).is_err();
    let rs = compare("unlistable corpus/public", &f.engine(), &[]);
    // The byte-for-byte comparison above is the acceptance bar and holds either
    // way; the assertion below only applies where the mode bits actually bite
    // (never for a process running as root).
    if enforced {
        assert_eq!(rs.code, 1, "{}", rs.out());
        assert!(
            rs.out()
                .contains("  - corpus root unreadable: knowledge/corpus/public\n"),
            "{}",
            rs.out()
        );
    }
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn a_missing_bank_directory_short_circuits_before_every_other_check() {
    let f = Fixture::grounded();
    let items = f.engine().join("bank/items");
    for e in std::fs::read_dir(&items).unwrap().flatten() {
        std::fs::remove_file(e.path()).unwrap();
    }
    std::fs::remove_dir(&items).unwrap();
    let rs = compare("missing bank dir", &f.engine(), &[]);
    assert_eq!(rs.code, 1, "{}", rs.out());
    // Still the FIRST thing checked: no counters, no vacuity block, one line.
    assert_eq!(rs.out(), "FAIL: bank/items missing\n");
}

// ── h) ordering, slicing, and formatting ───────────────────────────────────

#[test]
fn the_sample_list_ordering_and_slicing_match() {
    let f = Fixture::grounded();
    // Three tiers of overlap plus ties inside a tier, so the stable sort's
    // fallback to filename order is actually exercised.
    f.item("a01.toml", &unmoored_item("z-tie-a"));
    f.item("a02.toml", &unmoored_item("z-tie-b"));
    f.item("a03.toml", &unmoored_item("z-tie-c"));
    f.item(
        "a04.toml",
        "id = \"partial-one\"\n\
         stem = \"cooling wwwwqq vvvvqq uuuuqq\"\nchoices = []\n\
         explanation = \"\"\ntopic_ids = []\n",
    );
    f.item(
        "a05.toml",
        "id = \"partial-two\"\n\
         stem = \"cooling airflow vvvvqq uuuuqq\"\nchoices = []\n\
         explanation = \"\"\ntopic_ids = []\n",
    );
    f.item("a06.toml", &grounded_item("full-overlap"));

    for args in [
        vec!["--min-overlap", "1.0"],
        vec!["--min-overlap", "1.0", "--sample-report", "2"],
        vec!["--min-overlap", "1.0", "--sample-report", "0"],
        vec!["--min-overlap", "1.0", "--sample-report", "-2"],
        vec!["--min-overlap", "1.0", "--sample-report", "999"],
    ] {
        let label = format!("sample slice {args:?}");
        let rs = compare(&label, &f.engine(), &args);
        assert_eq!(rs.code, 0, "[{label}]:\n{}", rs.out());
    }

    // `--sample-report 0` slices to nothing while the header still reports the
    // full count, which is exactly the shape a vacuous report takes.
    let rs = compare(
        "sample slice zero",
        &f.engine(),
        &["--min-overlap", "1.0", "--sample-report", "0"],
    );
    assert!(rs.out().contains("low_overlap_warns=6\n"), "{}", rs.out());
    assert_eq!(
        rs.out().lines().filter(|l| l.starts_with("  0.")).count(),
        0,
        "{}",
        rs.out()
    );
}

#[test]
fn corpus_character_counting_matches_on_the_awkward_inputs() {
    // CRLF, a lone CR, a BOM, invalid UTF-8, a hidden file, a nested directory,
    // a wrong-suffix file, and the `corpus/public` split — all at once, because
    // every one of them moves `corpus_chars` and none of them moves the verdict.
    let f = Fixture::bare();
    f.pad_to_floor();
    f.item("a01.toml", &grounded_item("syn-ok"));
    f.write("modules/crlf.md", "ab\r\ncd\re\n");
    f.write("modules/nested/deep.txt", "deeper corpus text\n");
    f.write("modules/.hidden.md", "hidden corpus text\n");
    f.write("modules/ignored.rst", "this suffix is not read\n");
    f.write("modules/UPPER.MD", "suffix matching is case-folded\n");
    f.write("reference/bom.md", "\u{feff}after the byte order mark\n");
    f.write(
        "engine/knowledge/corpus/public/taken.txt",
        "captures are read from the txt leg only\n",
    );
    f.write(
        "engine/knowledge/corpus/public/skipped.md",
        "and never from the md leg\n",
    );
    std::fs::write(
        f.dir.path().join("modules/invalid.txt"),
        [b'o', b'k', 0xff, 0xfe, b'\n'],
    )
    .unwrap();

    let rs = compare("awkward corpus", &f.engine(), &[]);
    assert_eq!(rs.code, 0, "{}", rs.out());
    assert!(
        !rs.out().contains("corpus_chars=0\n"),
        "the fixture corpus must not be empty:\n{}",
        rs.out()
    );
}

#[cfg(unix)]
#[test]
fn symlink_handling_in_the_corpus_walk_matches() {
    use std::os::unix::fs::symlink;
    let f = Fixture::bare();
    f.pad_to_floor();
    f.item("a01.toml", &grounded_item("syn-ok"));
    f.write("outside/linked.md", "text reached through a link\n");
    f.write("modules/real.md", "text reached directly\n");
    // A symlinked FILE is read; a symlinked DIRECTORY is yielded but not
    // recursed into. Both are pathlib behaviours the port reproduces.
    symlink(
        f.dir.path().join("outside/linked.md"),
        f.dir.path().join("modules/aliased.md"),
    )
    .unwrap();
    symlink(
        f.dir.path().join("outside"),
        f.dir.path().join("modules/aliasdir"),
    )
    .unwrap();
    let rs = compare("symlinked corpus entries", &f.engine(), &[]);
    assert_eq!(rs.code, 0, "{}", rs.out());
}

#[test]
fn dotfile_bank_items_are_scanned_the_way_pathlib_globs_them() {
    let f = Fixture::grounded();
    f.item("a01.toml", &grounded_item("visible"));
    f.item(".hidden.toml", &grounded_item("hidden"));
    f.item("not-an-item.txt", "id = \"ignored\"\n");
    let rs = compare("dotfile bank item", &f.engine(), &[]);
    // The two written here plus the padding; the `.txt` is not an item.
    assert!(
        rs.out()
            .starts_with(&format!("scanned_items={}\n", MIN_SCANNED_ITEMS + 2)),
        "{}",
        rs.out()
    );
}

/// `id = ""` is present-but-empty. It used to prefix every finding for the item
/// with nothing at all; both sides now fall back to the filename, the shape
/// `verify_orphans` already had (bd-yje7).
#[test]
fn an_empty_id_falls_back_to_the_filename_on_both_sides() {
    let f = Fixture::grounded();
    f.item(
        "blank-id.toml",
        "id = \"\"\nstem = \"see clause 5.2.1 for the containment rule\"\n\
         choices = [\"cooling\", \"airflow\"]\nexplanation = \"\"\ntopic_ids = []\n",
    );
    let rs = compare("empty item id", &f.engine(), &[]);
    assert_eq!(rs.code, 1, "{}", rs.out());
    assert!(
        rs.out().contains("  - blank-id.toml: hallucinated-clause"),
        "{}",
        rs.out()
    );
}

/// A `topics.toml` whose very FIRST byte opens a block used to lose that label
/// silently — the raw id was tokenised in its place and the score degraded with
/// no finding printed. Both sides now read it (bd-yje7).
#[test]
fn a_first_line_topic_block_is_read_on_both_sides() {
    let f = Fixture::grounded();
    // No header line, and a label whose words appear nowhere in the corpus, so
    // the item can only clear the bar if the LABEL was read.
    f.write(
        "engine/knowledge/topics.toml",
        "[[topic]]\nid = \"t-first\"\nlabel = \"qqqqzz wwwwzz\"\n",
    );
    f.item(
        "a01.toml",
        "id = \"first-block\"\nstem = \"qqqqzz wwwwzz\"\nchoices = []\n\
         explanation = \"\"\ntopic_ids = [\"t-first\"]\n",
    );
    let rs = compare(
        "first-line topic block",
        &f.engine(),
        &["--min-overlap", "0.9"],
    );
    assert_eq!(rs.code, 0, "{}", rs.out());
    assert!(rs.out().contains("low_overlap_warns=0\n"), "{}", rs.out());
}

// ── i) argparse ────────────────────────────────────────────────────────────

#[test]
fn the_argument_parser_matches_byte_for_byte() {
    let f = Fixture::grounded();
    f.item("a01.toml", &grounded_item("syn-ok"));
    let e = f.engine();

    // Accepted spellings, all of which run the gate.
    for args in [
        vec!["--strict-overlap"],
        vec!["--strict"],
        vec!["--min", "0.5"],
        vec!["--min-overlap=0.5"],
        vec!["--min-overlap", "-0.5"],
        vec!["--sample-report", "-3"],
        vec!["--sample", "4"],
        vec!["--min-overlap", "0.5", "--min-overlap", "0.9"],
        vec!["--min-overlap", "1_0.5"],
        vec!["--min-overlap", " 0.5 "],
    ] {
        let label = format!("accepted {args:?}");
        let rs = compare(&label, &e, &args);
        assert_eq!(rs.code, 0, "[{label}]:\n{}\n{}", rs.out(), rs.err());
    }

    // Rejected spellings, all of which must be argparse's status 2 with the
    // usage block on stderr and nothing at all on stdout.
    for args in [
        vec!["--bogus"],
        vec!["-x"],
        vec!["-"],
        vec!["--"],
        vec!["--", "--strict-overlap"],
        vec!["--strict-overlap", "--"],
        vec!["extra"],
        vec!["has space"],
        vec!["--s"],
        vec!["--min-overlap"],
        vec!["--min-overlap", "x"],
        vec!["--min-overlap="],
        vec!["--sample-report", "1.5"],
        vec!["--sample-report", "0x10"],
        vec!["--strict-overlap=1"],
        vec!["--min-overlap", "--strict-overlap"],
    ] {
        let label = format!("rejected {args:?}");
        let rs = compare(&label, &e, &args);
        assert_eq!(rs.code, 2, "[{label}]:\n{}\n{}", rs.out(), rs.err());
        assert!(
            rs.out().is_empty(),
            "[{label}] wrote to stdout: {}",
            rs.out()
        );
        assert!(
            rs.err().starts_with("usage: validate_grounding.py"),
            "[{label}]: {}",
            rs.err()
        );
    }

    // Values Python's `float()`/`int()` accept and Rust's `parse` does not.
    let rs = compare("infinite bar", &e, &["--min-overlap", "inf"]);
    assert_eq!(rs.code, 0, "{}", rs.out());
    let rs = compare("nan bar", &e, &["--min-overlap", "nan"]);
    assert_eq!(rs.code, 0, "{}", rs.out());
}

#[test]
fn help_output_matches_at_several_widths() {
    let f = Fixture::grounded();
    let e = f.engine();
    for flag in ["--help", "-h"] {
        let rs = compare_env(&format!("help {flag}"), &e, &[flag], None);
        assert_eq!(rs.code, 0, "{}", rs.err());
        assert!(rs.out().starts_with("usage: validate_grounding.py"));
        assert!(rs.out().contains("\noptions:\n"), "{}", rs.out());
    }
    // The oracle wraps to the terminal width; both sides read COLUMNS.
    for cols in ["40", "60", "100", "200"] {
        let rs = compare_env(
            &format!("help at COLUMNS={cols}"),
            &e,
            &["--help"],
            Some(cols),
        );
        assert_eq!(rs.code, 0, "{}", rs.err());
    }
    // An unusable COLUMNS falls back the same way on both sides.
    for cols in ["", "0", "-5", "notanumber"] {
        compare_env(
            &format!("help at COLUMNS={cols:?}"),
            &e,
            &["--help"],
            Some(cols),
        );
    }
    // And the usage block inside an ERROR follows the same width.
    compare_env("error usage at COLUMNS=40", &e, &["--bogus"], Some("40"));
}

// ── the harness must not be vacuously green ────────────────────────────────

#[test]
fn the_harness_compared_something() {
    // Runs a case itself rather than reading a counter another test may or may
    // not have incremented — test order and parallelism are not a contract, and
    // "0 cases compared" must never report like "all passed".
    let f = Fixture::grounded();
    f.item("a01.toml", &grounded_item("self-check"));
    let before = COMPARED.load(Ordering::SeqCst);
    let rs = compare("harness self-check", &f.engine(), &[]);
    assert_eq!(rs.code, 0, "{}", rs.out());
    assert!(
        COMPARED.load(Ordering::SeqCst) > before,
        "the differential harness compared nothing"
    );
}
