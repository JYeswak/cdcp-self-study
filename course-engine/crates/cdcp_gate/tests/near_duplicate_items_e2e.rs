//! End-to-end for `cdcp_gate near-duplicate-items` (bd-near-duplicate-item-gate-i5v).
//!
//! Two of these tests run against THIS repo's live 804-item bank rather than a
//! fixture, because the thing under test is a calibration: a threshold that
//! catches the known-true pair on a synthetic two-item fixture but drowns on the
//! real bank would pass a fixture-only suite and still be worthless.

mod support;

use cdcp_gate::gates::near_duplicate_items;
use std::path::{Path, PathBuf};

fn engine_root() -> PathBuf {
    cdcp_gate::root::resolve(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
}

fn run_here(args: &[&str]) -> (i32, String) {
    support::run_gate(&engine_root(), args)
}

/// A minimal engine-shaped tree: just the root anchor plus a bank directory.
struct Bank {
    dir: tempfile::TempDir,
    root: PathBuf,
}

impl Bank {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().canonicalize().expect("canonicalize");
        std::fs::create_dir_all(root.join("registries")).unwrap();
        std::fs::write(root.join("registries/claims.toml"), "schema_version = 1\n").unwrap();
        Bank { dir, root }
    }

    fn with_items_dir() -> Self {
        let b = Bank::new();
        std::fs::create_dir_all(b.root.join("bank/items")).unwrap();
        b
    }

    fn item(
        &self,
        name: &str,
        id: &str,
        stem: &str,
        correct: &str,
        choices: &[&str],
        status: &str,
    ) {
        let list = choices
            .iter()
            .map(|c| format!("  {:?},", c))
            .collect::<Vec<_>>()
            .join("\n");
        let body = format!(
            "id = {id:?}\nmodule = 14\nstem = {stem:?}\nchoices = [\n{list}\n]\ncorrect = {correct:?}\nexplanation = \"long enough explanation\"\ntopic_ids = [\"t\"]\nbloom = \"apply\"\nsource_class = \"original\"\nquantity_evidence = \"qualitative_only\"\nstatus = {status:?}\n"
        );
        std::fs::write(self.root.join("bank/items").join(name), body).unwrap();
    }

    fn raw(&self, name: &str, body: &str) {
        std::fs::write(self.root.join("bank/items").join(name), body).unwrap();
    }

    fn gate(&self, args: &[&str]) -> (i32, String) {
        let _ = &self.dir;
        support::run_gate(&self.root, args)
    }
}

const IST_KEY: &str =
    "Validates power, cooling, and controls failovers as a combined system under planned scenarios";

// ── calibration against the live bank ───────────────────────────────────────

/// Read one real item file off disk.
fn live_item(name: &str) -> near_duplicate_items::Item {
    let path = engine_root()
        .join(near_duplicate_items::BANK_REL)
        .join(name);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    near_duplicate_items::parse_item(&text, name).expect("parses")
}

/// THE calibration target, read from the live item files rather than from a
/// hand-copied fixture. `m14-q040` and `m14-q121` are one item entered twice
/// with the key moved from B to A. Exact-stem hashing over the same 804 items
/// returns zero groups; this detector must return this pair.
///
/// This asserts against the TEXT, not against the gate's verdict, on purpose:
/// the pair was resolved by retiring one member (below), and a calibration that
/// evaporated the moment the defect was fixed would leave the threshold
/// unguarded for every future edit.
#[test]
fn the_known_true_pair_is_detected_from_the_live_item_files() {
    let a = live_item("m14-q040.toml");
    let b = live_item("m14-q121.toml");
    assert_eq!(a.id, "mock40-q40");
    assert_eq!(b.id, "bank-m14-q121");

    let (findings, comparisons) =
        near_duplicate_items::find_near_duplicates(&[a, b]).expect("two items compare");
    assert_eq!(comparisons, 1, "one pair means exactly one comparison");
    assert_eq!(
        findings.len(),
        1,
        "the known-true pair must be detected on its live text"
    );
    assert_eq!(
        findings[0].key.percent(),
        100,
        "the key text is identical verbatim: {}",
        findings[0].line()
    );
    assert_eq!(findings[0].rule, near_duplicate_items::Rule::SharedKeyText);
}

/// The pair was resolved by RETIRING one member, not by moving a threshold
/// until the finding disappeared. Exactly one of the two must be out of the
/// assembly pool, and the live gate must consequently stop naming them
/// together.
#[test]
fn the_calibration_pair_is_resolved_by_retirement_not_by_a_threshold() {
    let a = live_item("m14-q040.toml");
    let b = live_item("m14-q121.toml");
    assert_eq!(
        [&a, &b].iter().filter(|i| i.is_approved()).count(),
        1,
        "exactly one of m14-q040 / m14-q121 may be approved; got a={:?} b={:?}",
        a.status,
        b.status
    );
    assert_eq!(
        near_duplicate_items::KEY_SIMILARITY_PCT,
        60,
        "the calibrated threshold moved; re-run the false-positive review before changing it"
    );

    let (_, out) = run_here(&["near-duplicate-items"]);
    for line in out.lines() {
        assert!(
            !(line.contains("mock40-q40") && line.contains("bank-m14-q121")),
            "the retired item is still in the assembly pool: {line}"
        );
    }
}

/// The known-GOOD leg, on real items. `m09-q206` and `m09-q207` share a stem
/// shape ("<X>-aisle containment primarily aims to:") and are the two halves of
/// a genuine distinction — hot-aisle vs cold-aisle containment. A gate that
/// flags these has learned stem shape instead of duplication.
#[test]
fn a_real_pair_that_only_shares_a_stem_shape_is_not_flagged() {
    let (_, out) = run_here(&["near-duplicate-items"]);
    for line in out.lines() {
        assert!(
            !(line.contains("m09-q206") && line.contains("m09-q207")),
            "known-GOOD pair (hot-aisle vs cold-aisle containment) was flagged: {line}"
        );
        assert!(
            !(line.contains("m02-q067") && line.contains("m02-q203")),
            "known-GOOD pair (ISO/IEC 22237 vs EN 50600) was flagged: {line}"
        );
        assert!(
            !(line.contains("m08-q062") && line.contains("m09-q154")),
            "known-GOOD pair (rack orientation vs HAC/CAC comparison) was flagged: {line}"
        );
    }
}

/// The live run must not be vacuous: it has to say how much it compared.
#[test]
fn the_live_run_reports_a_nonzero_comparison_count_or_findings() {
    let (code, out) = run_here(&["near-duplicate-items"]);
    if code == 0 {
        assert!(
            out.contains("pair comparison(s)"),
            "a green verdict must state what was compared:\n{out}"
        );
        assert!(
            !out.contains("0 pair comparison(s)"),
            "zero comparisons must never report green:\n{out}"
        );
    } else {
        assert!(
            out.contains("violation(s)"),
            "a red verdict must count its findings:\n{out}"
        );
    }
}

// ── L4: the gate is shown to trip ───────────────────────────────────────────

#[test]
fn the_injected_known_bad_trips_the_detector() {
    let b = Bank::with_items_dir();
    b.item(
        "one.toml",
        "seed-1",
        "Integrated Systems Testing (IST) is valuable because it:",
        "B",
        &[
            "Only tests a single breaker nameplate",
            IST_KEY,
            "Replaces daily backups of VMs",
            "Is optional marketing text",
        ],
        "approved",
    );
    b.item(
        "two.toml",
        "seed-2",
        "What colour is a fire extinguisher body under local code?",
        "A",
        &["Red", "Chartreuse", "Ultraviolet", "Transparent"],
        "approved",
    );
    // Clean first: the fixture itself must not already be red, or the selftest
    // below would be indistinguishable from a pre-existing failure.
    let (code, out) = b.gate(&["near-duplicate-items"]);
    assert_eq!(code, 0, "fixture should be clean:\n{out}");

    let out = std::process::Command::new(support::BIN)
        .current_dir(&b.root)
        .arg("--root")
        .arg(&b.root)
        .arg("near-duplicate-items")
        .env("CDCP_NEAR_DUPLICATE_SELFTEST", "1")
        .output()
        .expect("run gate");
    let text =
        String::from_utf8_lossy(&out.stdout).into_owned() + &String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(0),
        "the selftest exits 0 only when the planted known-bad was caught:\n{text}"
    );
    assert!(
        text.contains("selftest reached RED"),
        "selftest must say it reached RED:\n{text}"
    );
    assert!(text.contains("seed-1-selftest-clone"), "{text}");
}

/// The known-bad, planted in the TREE rather than in memory: a cosmetically
/// reworded copy of an existing item must turn the gate RED naming both ids.
#[test]
fn a_reworded_copy_planted_in_the_tree_goes_red_naming_both_ids() {
    let b = Bank::with_items_dir();
    b.item(
        "orig.toml",
        "plant-orig",
        "Integrated Systems Testing (IST) is valuable because it:",
        "B",
        &[
            "Only tests a single breaker nameplate",
            IST_KEY,
            "Replaces daily backups of VMs",
            "Is optional marketing text",
        ],
        "approved",
    );
    let (code, out) = b.gate(&["near-duplicate-items"]);
    assert_ne!(code, 0, "one item is zero comparisons, an ERROR:\n{out}");

    b.item(
        "copy.toml",
        "plant-copy",
        "Integrated Systems Testing (IST) is valuable primarily because it:",
        "A",
        &[
            IST_KEY,
            "Only tests a single breaker nameplate in isolation forever",
            "Replaces daily VM backups",
            "Is optional marketing text",
        ],
        "approved",
    );
    b.item(
        "filler.toml",
        "plant-filler",
        "Which document records a planned maintenance sequence?",
        "C",
        &[
            "A purchase order",
            "A lease",
            "A method of procedure",
            "A floor plan",
        ],
        "approved",
    );
    let (code, out) = b.gate(&["near-duplicate-items"]);
    assert_eq!(code, 2, "planted duplicate must be a VIOLATION:\n{out}");
    assert!(
        out.contains("plant-orig") && out.contains("plant-copy"),
        "{out}"
    );
}

// ── anti-vacuous ────────────────────────────────────────────────────────────

#[test]
fn a_missing_bank_directory_is_an_error_not_a_pass() {
    let b = Bank::new();
    let (code, out) = b.gate(&["near-duplicate-items"]);
    assert_eq!(code, 4, "{out}");
    assert!(out.contains("no bank directory"), "{out}");
}

#[test]
fn zero_item_files_is_an_error_not_a_pass() {
    let b = Bank::with_items_dir();
    let (code, out) = b.gate(&["near-duplicate-items"]);
    assert_eq!(code, 4, "{out}");
    assert!(out.contains("zero .toml item files"), "{out}");
}

#[test]
fn zero_approved_items_is_an_error_not_a_pass() {
    let b = Bank::with_items_dir();
    b.item(
        "a.toml",
        "d-1",
        "stem one here",
        "A",
        &["w", "x", "y", "z"],
        "draft",
    );
    b.item(
        "b.toml",
        "d-2",
        "stem two here",
        "A",
        &["w", "x", "y", "z"],
        "retired",
    );
    let (code, out) = b.gate(&["near-duplicate-items"]);
    assert_eq!(code, 4, "an unassessed pool must not report green:\n{out}");
    assert!(out.contains("ZERO are status"), "{out}");
}

#[test]
fn one_approved_item_is_zero_comparisons_and_therefore_an_error() {
    let b = Bank::with_items_dir();
    b.item(
        "a.toml",
        "a-1",
        "stem one here",
        "A",
        &["w", "x", "y", "z"],
        "approved",
    );
    b.item(
        "b.toml",
        "b-2",
        "stem two here",
        "A",
        &["w", "x", "y", "z"],
        "draft",
    );
    let (code, out) = b.gate(&["near-duplicate-items"]);
    assert_eq!(code, 4, "{out}");
    assert!(out.contains("ZERO comparisons"), "{out}");
}

#[test]
fn an_unparseable_item_is_an_error_never_a_silent_skip() {
    let b = Bank::with_items_dir();
    b.item(
        "a.toml",
        "a-1",
        "stem one here",
        "A",
        &["w", "x", "y", "z"],
        "approved",
    );
    b.item(
        "b.toml",
        "b-2",
        "stem two here",
        "A",
        &["w", "x", "y", "z"],
        "approved",
    );
    b.raw("broken.toml", "id = \"nope\"\nthis is not toml [[[\n");
    let (code, out) = b.gate(&["near-duplicate-items"]);
    assert_eq!(
        code, 4,
        "a file that could not be read must not pass:\n{out}"
    );
    assert!(out.contains("broken.toml"), "{out}");
}

// ── scope and hygiene ───────────────────────────────────────────────────────

#[test]
fn retiring_one_of_a_pair_resolves_the_finding() {
    let b = Bank::with_items_dir();
    b.item(
        "orig.toml",
        "r-orig",
        "Integrated Systems Testing (IST) is valuable because it:",
        "B",
        &[
            "Only tests a single breaker nameplate",
            IST_KEY,
            "Replaces daily backups of VMs",
            "Is optional marketing text",
        ],
        "approved",
    );
    b.item(
        "copy.toml",
        "r-copy",
        "Integrated Systems Testing (IST) is valuable primarily because it:",
        "A",
        &[
            IST_KEY,
            "Only tests a single breaker nameplate in isolation forever",
            "Replaces daily VM backups",
            "Is optional marketing text",
        ],
        "approved",
    );
    b.item(
        "filler.toml",
        "r-filler",
        "Which document records a planned maintenance sequence?",
        "C",
        &[
            "A purchase order",
            "A lease",
            "A method of procedure",
            "A floor plan",
        ],
        "approved",
    );
    assert_eq!(b.gate(&["near-duplicate-items"]).0, 2);

    b.item(
        "copy.toml",
        "r-copy",
        "Integrated Systems Testing (IST) is valuable primarily because it:",
        "A",
        &[
            IST_KEY,
            "Only tests a single breaker nameplate in isolation forever",
            "Replaces daily VM backups",
            "Is optional marketing text",
        ],
        "retired",
    );
    let (code, out) = b.gate(&["near-duplicate-items"]);
    assert_eq!(code, 0, "retiring one of the pair must clear it:\n{out}");
    assert!(out.contains("2 approved"), "{out}");
}

#[test]
fn an_unknown_flag_is_usage_not_a_silent_pass() {
    let (code, out) = run_here(&["near-duplicate-items", "--threshold=0"]);
    assert_eq!(code, 3, "{out}");
}

#[test]
fn the_gate_is_registered_in_the_dispatcher() {
    let (code, out) = run_here(&["list"]);
    assert_eq!(code, 0);
    assert!(out.contains("near-duplicate-items"), "{out}");
}
