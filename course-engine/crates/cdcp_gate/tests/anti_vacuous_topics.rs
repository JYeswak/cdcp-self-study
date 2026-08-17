//! bd-9nyt — the anti-vacuous legs of `verify-objectives`, asserted as VERDICTS
//! rather than as agreement between two implementations.
//!
//! `tests/diff_verify_objectives.rs` compares the Rust port against
//! `scripts/verify_objectives.py` byte for byte, and its own header records what
//! that cannot decide: *whether the oracle is right*. It already ran
//! `--min-items-per-topic 0`, found the two sides identical, and passed — while
//! both of them printed `covered=106 shortfalls=0 mode=strict` and exited 0
//! having compared not one topic. A faithful port of a hole is a hole.
//!
//! So this file asserts what the answer must BE, on BOTH implementations, for
//! the three shapes the bd-9nyt sweep closed:
//!
//!   1. **The anti-vacuous topic leg could not fire in its own case.** It was
//!      spelled `elif not primary_topics and topics_path.is_file() and declared`
//!      — the loudest way to end up with zero topics is the file not existing,
//!      and the `is_file()` conjunct locked the defence out of exactly that. It
//!      was harmless only because a neighbouring load independently reports
//!      `missing topics registry`, i.e. the defence was standing down in favour
//!      of a check it does not name and would not notice the removal of.
//!      `the_anti_vacuous_leg_holds_the_line_without_its_neighbour` is the
//!      proof: on an absent topics file BOTH lines must now appear, and the
//!      anti-vacuous one is no longer conditional on the other.
//!   2. **`--min-items-per-topic 0` disabled the whole comparison in silence.**
//!   3. **A present-but-unreadable `topic_ids`/`objective_ids`** read as an
//!      absent field — quieter than an untagged item.
//!
//! Plus the KNOWN-GOOD half, which matters as much: the live tree stays green,
//! an absent `bank_policy.toml` stays legal, and a floor turned off with
//! `--skip-topic-coverage` stays green. An over-strict gate gets routed around,
//! which is a slower death than a leaky one.
//!
//! ## What this file cannot decide
//!
//! It says nothing about whether a topic tag is the RIGHT tag, nor about exam
//! outcomes. `unreasoned_path_guards_are_a_defect` is a REGRESSION tripwire over
//! the live `crates/cdcp_gate/src/gates/*.rs` tree and the remaining
//! `scripts/verify_*.py` / `scripts/validate_*.py` oracles, not a proof that
//! every helper follows `?` — see its own doc.

#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const BIN: &str = env!("CARGO_BIN_EXE_cdcp_gate");
const ORACLE: &str = "scripts/verify_objectives.py";
const GATE: &str = "verify-objectives";

/// Verdicts actually asserted, so "the suite ran" is itself checked.
static ASSERTED: AtomicUsize = AtomicUsize::new(0);

fn engine_root() -> PathBuf {
    cdcp_gate::root::resolve(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
}

struct Run {
    code: i32,
    out: String,
}

/// Run the BUILT binary, never `cargo run`: cargo writes build diagnostics to
/// stderr and a sibling gate's warning would read as this gate's output.
fn rust(root: &Path, args: &[&str]) -> Run {
    let o = Command::new(BIN)
        .current_dir(root)
        .arg("--root")
        .arg(root)
        .arg(GATE)
        .args(args)
        .output()
        .expect("run cdcp_gate");
    Run {
        code: o.status.code().unwrap_or(-1),
        out: String::from_utf8_lossy(&o.stdout).into_owned(),
    }
}

/// The oracle. A missing `python3` is a FAILURE and never a skip: a suite that
/// silently checked one implementation passes exactly like one that checked two.
fn python(root: &Path, args: &[&str]) -> Run {
    let o = Command::new("python3")
        .current_dir(root)
        .arg(ORACLE)
        .args(args)
        .output()
        .unwrap_or_else(|e| {
            panic!("python3 {ORACLE} could not run ({e}); the oracle is REQUIRED, never skipped")
        });
    Run {
        code: o.status.code().unwrap_or(-1),
        out: String::from_utf8_lossy(&o.stdout).into_owned(),
    }
}

/// Assert the same verdict on BOTH implementations. `needles` must all appear.
fn both_red(label: &str, root: &Path, args: &[&str], needles: &[&str]) {
    for (who, r) in [("rust", rust(root, args)), ("python", python(root, args))] {
        assert_ne!(
            r.code, 0,
            "[{label}/{who}] expected RED, exited 0:\n{}",
            r.out
        );
        for n in needles {
            assert!(
                r.out.contains(n),
                "[{label}/{who}] exited {} but never said {n:?}:\n{}",
                r.code,
                r.out
            );
        }
        assert!(
            !r.out.starts_with("PASS"),
            "[{label}/{who}] a RED run must not open with PASS:\n{}",
            r.out
        );
    }
    ASSERTED.fetch_add(1, Ordering::SeqCst);
}

/// The known-GOOD half. A gate that only has attack legs ships over-strict.
fn both_green(label: &str, root: &Path, args: &[&str], needles: &[&str]) {
    for (who, r) in [("rust", rust(root, args)), ("python", python(root, args))] {
        assert_eq!(
            r.code, 0,
            "[{label}/{who}] a legitimate input must NOT fail:\n{}",
            r.out
        );
        assert!(
            r.out.contains("objective coverage GREEN"),
            "[{label}/{who}] exited 0 without the GREEN receipt:\n{}",
            r.out
        );
        for n in needles {
            assert!(
                r.out.contains(n),
                "[{label}/{who}] missing {n:?}:\n{}",
                r.out
            );
        }
    }
    ASSERTED.fetch_add(1, Ordering::SeqCst);
}

fn write(path: &Path, body: &str) {
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p).unwrap();
    }
    std::fs::write(path, body).unwrap();
}

/// A TEMP tree carrying the live registries, knowledge and bank. Nothing here
/// ever touches the working tree: the known-bad cases delete inputs, and
/// deleting a live input to test a case is how a suite eats its own repo.
struct Tree {
    _td: tempfile::TempDir,
    root: PathBuf,
}

impl Tree {
    fn new() -> Self {
        let src = engine_root();
        let td = tempfile::tempdir().unwrap();
        let root = td.path().canonicalize().unwrap();
        let mut copied = 0usize;
        for dir in ["registries", "knowledge", "bank/items"] {
            let from = src.join(dir);
            let to = root.join(dir);
            std::fs::create_dir_all(&to).unwrap();
            for e in std::fs::read_dir(&from)
                .unwrap_or_else(|e| panic!("live {dir} unreadable: {e}"))
                .flatten()
            {
                if e.path().is_file() {
                    std::fs::copy(e.path(), to.join(e.file_name())).unwrap();
                    copied += 1;
                }
            }
        }
        // Anti-vacuous on the FIXTURE: a specimen that copied nothing would make
        // every case below pass for the wrong reason.
        assert!(
            copied > 100,
            "the specimen tree copied {copied} files — a vacuous fixture is an ERROR, not a pass"
        );
        Tree { _td: td, root }
    }

    fn p(&self, rel: &str) -> PathBuf {
        self.root.join(rel)
    }

    /// The gate is invoked with `--root` at the specimen, so every default path
    /// resolves inside it and the live tree is never read.
    fn args(&self) -> Vec<String> {
        vec![
            "--objectives".into(),
            self.p("registries/objectives.toml").display().to_string(),
            "--claims".into(),
            self.p("registries/claims.toml").display().to_string(),
            "--domains".into(),
            self.p("knowledge/domains.toml").display().to_string(),
            "--topics".into(),
            self.p("knowledge/topics.toml").display().to_string(),
            "--policy".into(),
            self.p("knowledge/bank_policy.toml").display().to_string(),
            "--bank".into(),
            self.p("bank/items").display().to_string(),
        ]
    }
}

fn as_refs(v: &[String], extra: &[&str]) -> Vec<String> {
    let mut o = v.to_vec();
    o.extend(extra.iter().map(|s| (*s).to_string()));
    o
}

fn strs(v: &[String]) -> Vec<&str> {
    v.iter().map(String::as_str).collect()
}

// ── 1) the headline: the anti-vacuous leg, in the case it defends ──────────

/// THE REGRESSION. Before bd-9nyt the anti-vacuous leg carried
/// `topics_path.is_file()`, so on an absent topics file it did not run at all —
/// the run was RED only because a DIFFERENT check, in a different block, happens
/// to report the missing file. Remove that neighbour and the gate went quiet.
///
/// The assertion is deliberately about BOTH lines. `missing topics registry`
/// alone would pass on the old code; `topics.toml has zero topics in a required
/// domain` is the leg proving it fired on its own account, in the one case its
/// own condition used to exclude it from.
#[test]
fn the_anti_vacuous_leg_holds_the_line_without_its_neighbour() {
    let t = Tree::new();
    let root = engine_root();
    std::fs::remove_file(t.p("knowledge/topics.toml")).unwrap();
    let a = t.args();
    both_red(
        "absent topics registry",
        &root,
        &strs(&a),
        &[
            "missing topics registry:",
            "topics.toml has zero topics in a required domain",
        ],
    );
}

/// The case the old spelling COULD see: the file is there and holds nothing.
/// Kept because removing the `is_file()` conjunct must not cost this leg.
#[test]
fn an_empty_topics_file_is_an_error() {
    let t = Tree::new();
    let root = engine_root();
    write(&t.p("knowledge/topics.toml"), "");
    both_red(
        "zero-byte topics registry",
        &root,
        &strs(&t.args()),
        &["topics.toml has zero topics in a required domain"],
    );

    // …and a 0-byte file is not the only shape of empty. A header with no rows
    // reads identically to `is_file()` and differently to a check that counts.
    write(&t.p("knowledge/topics.toml"), "schema_version = 1\n");
    both_red(
        "header-only topics registry",
        &root,
        &strs(&t.args()),
        &["topics.toml has zero topics in a required domain"],
    );
}

// ── 2) the floor is on, or it is off out loud ──────────────────────────────

/// `--min-items-per-topic 0` used to fall straight through the comparison and
/// still print a coverage number. Both halves are asserted: the RED verdict, and
/// the report no longer naming a mode it is not in.
#[test]
fn a_silently_disabled_topic_floor_is_an_error() {
    let t = Tree::new();
    let root = engine_root();
    let a = as_refs(&t.args(), &["--min-items-per-topic", "0"]);
    both_red(
        "min-items 0",
        &root,
        &strs(&a),
        &[
            "--min-items-per-topic 0 turns the primary-topic floor off without saying so",
            "mode=off",
            "covered=n/a",
        ],
    );

    // The most misleading string this gate could print: a strict mode with zero
    // comparisons behind it. It must not be printable.
    let a = as_refs(
        &t.args(),
        &["--min-items-per-topic", "0", "--strict-topics"],
    );
    both_red(
        "min-items 0 under --strict-topics",
        &root,
        &strs(&a),
        &["mode=off", "covered=n/a"],
    );
    for (who, r) in [
        ("rust", rust(&root, &strs(&a))),
        ("python", python(&root, &strs(&a))),
    ] {
        assert!(
            !r.out.contains("mode=strict"),
            "[{who}] a floor that compared nothing must not report mode=strict:\n{}",
            r.out
        );
    }
}

/// KNOWN-GOOD. Turning the floor off with the flag that says so stays green —
/// and still refuses to print a coverage number it did not compute.
#[test]
fn a_floor_turned_off_on_purpose_is_still_green() {
    let t = Tree::new();
    let root = engine_root();
    let a = as_refs(
        &t.args(),
        &["--min-items-per-topic", "0", "--skip-topic-coverage"],
    );
    both_green(
        "min-items 0 with --skip-topic-coverage",
        &root,
        &strs(&a),
        &["mode=skipped", "covered=n/a"],
    );
}

/// The `covered=` field is a coverage computation, and the rule is one-way: a
/// number appears only when a comparison produced it.
#[test]
fn the_report_never_prints_a_coverage_number_the_floor_did_not_compute() {
    let t = Tree::new();
    let root = engine_root();

    // floor ran -> a real number, and it is not zero on the live registries
    let r = rust(&root, &strs(&t.args()));
    assert_eq!(r.code, 0, "{}", r.out);
    let line = r
        .out
        .lines()
        .find(|l| l.trim_start().starts_with("primary_topics="))
        .unwrap_or_else(|| panic!("no primary_topics line:\n{}", r.out))
        .to_string();
    let covered = line
        .split("covered=")
        .nth(1)
        .and_then(|s| s.split_whitespace().next())
        .unwrap_or("");
    let n: usize = covered
        .parse()
        .unwrap_or_else(|_| panic!("the floor RAN, so covered= must be a number: {line}"));
    assert!(
        n > 0,
        "a live tree covering zero topics is a vacuous pass: {line}"
    );

    // floor did not run -> never a number
    for extra in [
        vec!["--skip-topic-coverage"],
        vec!["--min-items-per-topic", "0", "--skip-topic-coverage"],
    ] {
        let a = as_refs(&t.args(), &extra);
        let r = rust(&root, &strs(&a));
        assert!(
            r.out.contains("covered=n/a"),
            "the floor did not run, so covered= must be n/a ({extra:?}):\n{}",
            r.out
        );
    }
    ASSERTED.fetch_add(1, Ordering::SeqCst);
}

// ── 3) present-but-unreadable is not absent ────────────────────────────────

/// A bank item spelling `topic_ids = "t-01"` instead of `["t-01"]` used to
/// contribute zero to every tally and say nothing — quieter than an untagged
/// item, because the topics it meant to cover then only soft-warn.
#[test]
fn a_present_but_unreadable_tag_field_is_an_error() {
    let t = Tree::new();
    let root = engine_root();
    write(
        &t.p("bank/items/zz-bd9nyt-probe.toml"),
        "id = \"bd9nyt-probe\"\nmodule = 1\ntopic_ids = \"t-not-a-list\"\n",
    );
    both_red(
        "topic_ids is a string",
        &root,
        &strs(&t.args()),
        &["bd9nyt-probe: topic_ids is not a list: 't-not-a-list'"],
    );

    write(
        &t.p("bank/items/zz-bd9nyt-probe.toml"),
        "id = \"bd9nyt-probe\"\nmodule = 1\ntopic_ids = []\nobjective_ids = \"obj-not-a-list\"\n",
    );
    both_red(
        "objective_ids is a string",
        &root,
        &strs(&t.args()),
        &["bd9nyt-probe: objective_ids is not a list: 'obj-not-a-list'"],
    );
}

// ── 4) KNOWN-GOOD: the live tree, and the legitimately-optional input ──────

/// If this ever goes red, the sweep over-tightened and the gate will be routed
/// around. It is the control every attack leg above is measured against.
#[test]
fn the_live_tree_is_still_green_and_the_optional_input_is_still_optional() {
    let root = engine_root();
    both_green("live tree", &root, &[], &["mode=soft-warn"]);

    // `bank_policy.toml` is the one input this gate reads whose ABSENCE is
    // legitimate: an empty ledger holds nothing out, so every declared module
    // stays required and the gate gets STRICTER. Absence must stay legal — the
    // written reason in `load_exemptions` is only worth something if the
    // behaviour it describes is asserted.
    let td = tempfile::tempdir().unwrap();
    let absent = td.path().join("no_such_policy.toml");
    both_green(
        "absent policy ledger",
        &root,
        &["--policy", absent.to_str().unwrap()],
        &["policy=absent"],
    );
}

// ── 5) the general detector, scoped and argued ────────────────────────────

/// A REGRESSION TRIPWIRE for the absent-input-reads-as-success class, over the
/// live gate tree and the remaining Python oracles (bd-absent-ok-widen-m0j2).
///
/// # The rule
///
/// Every existence test that opens a BLOCK (`if …is_file()`, `if not …is_file():`,
/// `} else if …exists()`) must have, inside the if/else chain it opens, either
/// an error-recording call — or, in that chain or the twelve lines above it, the
/// marker `ABSENT-OK:` followed by the sentence saying why absence is fine. A
/// guard with neither is the defect: it is a check that can be skipped by a file
/// not being there, and nobody wrote down that they meant it.
///
/// The marker is structural on purpose. "Add a comment" cannot be satisfied by
/// accident, and it puts the sentence at the site rather than in a bead nobody
/// reads next to the code.
///
/// # Scope (widened from two files)
///
/// bd-9nyt invented the convention on `verify_objectives`. This wave globs
/// `crates/cdcp_gate/src/gates/*.rs` and the remaining `scripts/verify_*.py` /
/// `scripts/validate_*.py` oracles. `build_units.rs` is gone (extracted); it is
/// not scanned. An empty glob is ERROR.
///
/// # What it cannot decide
///
/// It does not read what the guarded block DOES, so it cannot tell a skipped
/// verdict from a skipped log line; it treats both as needing a sentence. It
/// does not follow the `?` operator or a helper that raises elsewhere. And it
/// cannot tell a true `ABSENT-OK:` from a lazy one — that is a review's job, and
/// the per-file reasoned cap below is what keeps the lazy ones visible.
#[test]
fn unreasoned_path_guards_are_a_defect() {
    let root = engine_root();
    let targets = scan_targets(&root);

    let mut total = 0usize;
    let mut reasoned = 0usize;
    let mut flagged: Vec<String> = Vec::new();
    for (path, rel, python) in &targets {
        let src = std::fs::read_to_string(path).unwrap_or_else(|e| {
            panic!("{rel} unreadable: {e} — a scan that read nothing is a FAILURE")
        });
        let sites = guard_sites(&src, *python);
        let min = min_sites(rel);
        // Anti-vacuous, per file: a scanner that matched nothing reports exactly
        // like one that matched everything and found it clean. Floors are
        // retuned per-file (bd-absent-ok-widen-m0j2); a 0-site file is only
        // legal where the file is known to have no block-opening existence test.
        assert!(
            sites.len() >= min,
            "{rel}: found {} path guards, floor is {min} — a near-empty scan \
             means the scanner broke or a guard vanished, not that the file is clean",
            sites.len()
        );
        let mut file_reasoned = 0usize;
        for s in &sites {
            total += 1;
            match classify(s) {
                Verdict::RecordsError => {}
                Verdict::Reasoned => {
                    reasoned += 1;
                    file_reasoned += 1;
                }
                Verdict::Unreasoned => flagged.push(format!("{rel}:{}: {}", s.line, s.head)),
            }
        }
        let max = max_reasoned(rel, sites.len());
        assert!(
            file_reasoned <= max,
            "{rel}: {file_reasoned} of {} guards are ABSENT-OK (cap {max}) — \
             the exemption must stay rare per file, not just in aggregate",
            sites.len()
        );
    }

    assert!(
        flagged.is_empty(),
        "path guards that neither record an error nor carry an `ABSENT-OK:` reason:\n  {}",
        flagged.join("\n  ")
    );
    assert!(
        total >= 50,
        "the scan covered only {total} guards — empty (or near-empty) scan set is ERROR"
    );
    // Global: the marker must stay exercised. The "exemption stays rare"
    // direction is enforced per-file above (`reasoned * 2 < total` retuned
    // as `file_reasoned <= max_reasoned`), because a 1-site search helper
    // cannot express "less than half".
    assert!(
        reasoned > 0,
        "{reasoned} of {total} guards are ABSENT-OK — the marker must stay exercised"
    );

    // PLANTED KNOWN-BAD: the scanner is itself checked. A detector nobody proved
    // can trip is a detector that reports clean because it looks at nothing.
    let bad = "fn f() {\n    if Path::new(&p).is_file() {\n        do_the_check();\n    }\n}\n";
    assert_eq!(
        guard_sites(bad, false).len(),
        1,
        "the scanner missed a bare guard"
    );
    assert!(
        matches!(classify(&guard_sites(bad, false)[0]), Verdict::Unreasoned),
        "a bare guard around a check must be FLAGGED"
    );
    // …and its known-GOOD twin, so the scanner is not simply flagging everything.
    let ok = "fn f() {\n    // ABSENT-OK: absence removes nothing this gate checks.\n    \
              if Path::new(&p).is_file() {\n        do_the_check();\n    }\n}\n";
    assert!(
        matches!(classify(&guard_sites(ok, false)[0]), Verdict::Reasoned),
        "a guard carrying a written reason must NOT be flagged"
    );
    let errs =
        "fn f() {\n    if Path::new(&p).is_file() {\n        do_it();\n    } else {\n        \
                errors.push(format!(\"missing: {p}\"));\n    }\n}\n";
    assert!(
        matches!(
            classify(&guard_sites(errs, false)[0]),
            Verdict::RecordsError
        ),
        "a guard whose else records an error must NOT be flagged"
    );
    ASSERTED.fetch_add(1, Ordering::SeqCst);
}

struct Site {
    line: usize,
    head: String,
    /// The whole if/else chain the guard opens.
    chain: String,
    /// The twelve lines above it, where a reason naturally sits.
    preamble: String,
}

enum Verdict {
    RecordsError,
    Reasoned,
    Unreasoned,
}

/// Live `src/gates/*.rs` plus remaining `scripts/{verify,validate}_*.py`.
/// An empty glob is ERROR. `build_units.rs` is extracted and not scanned.
fn scan_targets(root: &Path) -> Vec<(PathBuf, String, bool)> {
    let mut out = Vec::new();

    let gates_dir = root.join("crates/cdcp_gate/src/gates");
    let mut gate_files = 0usize;
    let rd = std::fs::read_dir(&gates_dir)
        .unwrap_or_else(|e| panic!("gates dir unreadable: {e} — an empty scan set is an ERROR"));
    for e in rd.flatten() {
        let p = e.path();
        if p.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let rel = format!(
            "crates/cdcp_gate/src/gates/{}",
            e.file_name().to_string_lossy()
        );
        out.push((p, rel, false));
        gate_files += 1;
    }
    assert!(
        gate_files >= 10,
        "gates glob found {gate_files} .rs files — empty (or near-empty) scan set is ERROR"
    );

    let scripts = root.join("scripts");
    let mut oracle_files = 0usize;
    let rd = std::fs::read_dir(&scripts)
        .unwrap_or_else(|e| panic!("scripts/ unreadable: {e} — an empty scan set is an ERROR"));
    for e in rd.flatten() {
        let name = e.file_name();
        let name = name.to_string_lossy();
        let is_oracle =
            (name.starts_with("verify_") || name.starts_with("validate_")) && name.ends_with(".py");
        if !is_oracle {
            continue;
        }
        out.push((e.path(), format!("scripts/{name}"), true));
        oracle_files += 1;
    }
    assert!(
        oracle_files >= 5,
        "oracle glob found {oracle_files} verify_/validate_ .py files — empty scan set is ERROR"
    );

    out.sort_by(|a, b| a.1.cmp(&b.1));
    out
}

/// Measured floors (2026-08-15). A drop means a guard vanished or the scanner
/// broke. 0 is reserved for files known to have no block-opening existence test.
fn min_sites(rel: &str) -> usize {
    match rel.rsplit('/').next().unwrap_or(rel) {
        // Dispatchers: the walks live in product crates (near_duplicate,
        // orphans, verify_bank). A 0-site file is legal only here.
        "mod.rs"
        | "install_hooks.rs"
        | "near_duplicate_items.rs"
        | "verify_orphans.rs"
        | "verify_bank.rs" => 0,
        "capability_maturity.rs"
        | "goldens_couplings.rs"
        | "verify_injection_count.rs"
        | "verify_step_count.rs"
        | "verify_doc_consistency.py" => 1,
        "doc_facts.rs" | "verify_orphans.py" | "verify_paraphrase_pairs.py" => 2,
        "corpus_redistribution.rs"
        | "verify_doc_consistency.rs"
        | "verify_knowledge_paths.rs"
        | "verify_knowledge_paths.py"
        | "verify_injection_count.py" => 3,
        "substrate_guard.rs" | "verify_coverage.py" => 4,
        "verify_bank.py" | "verify_coverage.rs" | "validate_grounding.py" => 5,
        "validate_grounding.rs" | "verify_objectives.py" => 7,
        "verify_objectives.rs" => 8,
        "verify_content_lock.rs" => 9,
        // A newly added target is scanned; give it a floor once measured.
        _ => 0,
    }
}

/// Per-file retune of `reasoned * 2 < total`. A 1- or 2-site search/walk
/// helper cannot express "less than half"; those files pin an explicit cap
/// instead of dropping the assertion.
fn max_reasoned(rel: &str, sites: usize) -> usize {
    match rel.rsplit('/').next().unwrap_or(rel) {
        "capability_maturity.rs"
        | "goldens_couplings.rs"
        | "verify_injection_count.rs"
        | "verify_step_count.rs" => 1,
        "doc_facts.rs" | "corpus_redistribution.rs" | "verify_doc_consistency.rs" => 2,
        "substrate_guard.rs" | "validate_grounding.py" => 3,
        "validate_grounding.rs" => 5,
        _ if sites < 3 => sites,
        _ => sites.saturating_sub(1) / 2,
    }
}

fn classify(s: &Site) -> Verdict {
    const RECORDS: [&str; 6] = [
        "errors.push(",
        "errors.append(",
        "errors.extend(",
        "return Err(",
        "missing",
        "parse error",
    ];
    if RECORDS.iter().any(|t| s.chain.contains(t)) {
        return Verdict::RecordsError;
    }
    if s.chain.contains("ABSENT-OK:") || s.preamble.contains("ABSENT-OK:") {
        return Verdict::Reasoned;
    }
    Verdict::Unreasoned
}

/// Every block-opening existence test, with the chain it opens. Deliberately
/// ignores existence tests that are not branches — `format!("policy={}", if
/// p.is_file() …)` selects a REPORT STRING and no code path, and is commented as
/// such at its site rather than parsed for here.
fn guard_sites(src: &str, python: bool) -> Vec<Site> {
    const TESTS: [&str; 3] = ["is_file()", ".exists()", "is_dir()"];
    let lines: Vec<&str> = src.lines().collect();
    let mut out = Vec::new();
    for (i, raw) in lines.iter().enumerate() {
        let t = raw.trim_start();
        let opens = if python {
            t.starts_with("if ") || t.starts_with("elif ")
        } else {
            t.starts_with("if ") || t.starts_with("} else if ") || t.starts_with("else if ")
        };
        if !opens || !TESTS.iter().any(|x| t.contains(x)) {
            continue;
        }
        let chain = if python {
            py_chain(&lines, i)
        } else {
            rs_chain(&lines, i)
        };
        let from = i.saturating_sub(12);
        out.push(Site {
            line: i + 1,
            head: t.to_string(),
            chain,
            preamble: lines[from..i].join("\n"),
        });
    }
    out
}

/// Brace-match the chain, with string literals stripped so a `{}` in a
/// `format!` cannot unbalance the count.
fn rs_chain(lines: &[&str], start: usize) -> String {
    let mut depth = 0i32;
    let mut seen = false;
    let mut out = String::new();
    for l in &lines[start..] {
        out.push_str(l);
        out.push('\n');
        for c in strip_str_lits(l).chars() {
            match c {
                '{' => {
                    depth += 1;
                    seen = true;
                }
                '}' => depth -= 1,
                _ => {}
            }
        }
        if seen && depth <= 0 {
            break;
        }
    }
    out
}

fn strip_str_lits(l: &str) -> String {
    let mut o = String::with_capacity(l.len());
    let mut in_s = false;
    let mut esc = false;
    for c in l.chars() {
        if in_s {
            if esc {
                esc = false;
            } else if c == '\\' {
                esc = true;
            } else if c == '"' {
                in_s = false;
            }
            continue;
        }
        if c == '"' {
            in_s = true;
            continue;
        }
        o.push(c);
    }
    o
}

/// The indented suite, plus any `else`/`elif` continuation at the same column.
fn py_chain(lines: &[&str], start: usize) -> String {
    let indent = |s: &str| s.len() - s.trim_start().len();
    let base = indent(lines[start]);
    let mut out = String::from(lines[start]);
    out.push('\n');
    for l in &lines[start + 1..] {
        if l.trim().is_empty() {
            out.push('\n');
            continue;
        }
        let ind = indent(l);
        if ind > base {
            out.push_str(l);
            out.push('\n');
            continue;
        }
        let t = l.trim_start();
        if ind == base && (t.starts_with("else") || t.starts_with("elif ")) {
            out.push_str(l);
            out.push('\n');
            continue;
        }
        break;
    }
    out
}

// ── the suite must have asserted something ────────────────────────────────

#[test]
fn the_suite_asserted_something() {
    // Cargo runs tests in parallel, so this cannot read a final total. It
    // asserts its own contribution instead: a run in which nothing incremented
    // the counter is a suite that compiled and checked nothing.
    let before = ASSERTED.load(Ordering::SeqCst);
    let root = engine_root();
    both_green("counter control", &root, &["--skip-topic-coverage"], &[]);
    assert!(
        ASSERTED.load(Ordering::SeqCst) > before,
        "no verdict was asserted — a vacuous suite is an ERROR, not a pass"
    );
}
