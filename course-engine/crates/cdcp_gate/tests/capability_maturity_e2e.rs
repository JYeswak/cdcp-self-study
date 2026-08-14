//! End-to-end legs for the `capability-maturity` gate (bd-hardening-b-ledgers-gvm.1).
//!
//! # CLAIM: FLOOR-RAISE
//!
//! Three things are held here and nothing more:
//!
//!   1. **Known-bad.** Each way a capability claim can be unbacked — an expired
//!      review, a test reference naming a function nothing defines, a contract
//!      reference naming a file that is not here, a blank field, an empty
//!      `evidence[]`, a level whose evidence obligation is unmet, a quoted
//!      CHARTER cell that has drifted, and a published "wired" over a level that
//!      cannot carry it — is planted and asserted to reach a non-zero exit with
//!      the row named.
//!   2. **Known-GOOD.** A well-formed, in-date row whose evidence resolves
//!      passes. An attack-only suite ships an over-strict gate, and over-strict
//!      gates get routed around instead of fixed.
//!   3. **The live ledger.** `registries/capability-maturity.toml` is schema
//!      clean, nothing in it has expired, and every reference in it resolves.
//!      Its outstanding findings are enumerated in `KNOWN_DEBTS` below with a
//!      reason each, so a debt cannot appear silently and a debt that is paid
//!      off fails this test until it is struck from the list.
//!
//! # WHAT THIS SUITE CANNOT DECIDE
//!
//! It cannot decide that a cited test asserts anything — `defines_fn` reads a
//! name, never a body — nor that a level is honest, nor that a `last_review`
//! date reflects a review rather than a keystroke. It runs the gate binary, so
//! it says nothing about whether `scripts/check.sh` calls it; BUILT != WIRED is
//! settled by the check.sh step, not by a test.

use std::path::{Path, PathBuf};
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_cdcp_gate");

/// Exit codes, mirrored from `cdcp_gate::exit` so a change there shows up here.
const OK: i32 = 0;
const VIOLATION: i32 = 2;
const ERROR: i32 = 4;

/// The findings the LIVE ledger carries, with the reason each is outstanding.
///
/// EMPTY as of 2026-08-14, and the reason matters more than the emptiness.
///
/// This list held `l2.slo-as-code` and `l5.fuzz-crash-floor` when the gate was
/// authored. Both were findings about the repo: CHARTER §5 published
/// "YES · wired" over capabilities no named test asserted. The controller
/// corrected those two cells in place — the same patch commit 467b429 applied to
/// L3 — and aligned the ledger's `charter_claim.status`, so the published claim
/// and the evidenced level now agree and this gate has nothing to refuse.
///
/// THE UNDERLYING DEBTS ARE NOT PAID. No test asserts an SLO budget (bd-kog9),
/// and nothing runs the fuzz targets (bd-p228). What changed is that the CHARTER
/// stopped overstating them. This gate detects CLAIM-VERSUS-EVIDENCE DRIFT, not
/// capability gaps: correcting a claim removes the finding without closing the
/// gap, and that is the correct division of labour — the gap is a bead, the lie
/// is a build failure.
///
/// An empty list is therefore a real state, not a vacuous one: it asserts the
/// published claims and the evidenced levels agree today. A NEW finding is a
/// finding about the repo — file it, then add it here with a reason.
const KNOWN_DEBTS: &[(&str, &str)] = &[];
fn engine_root() -> PathBuf {
    cdcp_gate::root::resolve(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
}

fn run_gate(root: &Path, args: &[&str]) -> (i32, String) {
    let out = Command::new(BIN)
        .current_dir(root)
        .arg("--root")
        .arg(root)
        .args(args)
        .output()
        .expect("run cdcp_gate");
    let mut s = String::from_utf8_lossy(&out.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.code().unwrap_or(-1), s)
}

// ── fixture ────────────────────────────────────────────────────────────────

struct Repo {
    _dir: tempfile::TempDir,
    root: PathBuf,
}

/// A markdown table shaped like CHARTER §5.
const CHARTER_TABLE: &str = "\
# fixture charter

## 5. Artifact rigor

| Layer | Applies? | How |
|-------|----------|-----|
| **L3 External oracle (factual content)** | **NO** | nothing outside this project checks the keys |
| **L9 Something wired** | **YES · wired** | a step in check.sh |
";

impl Repo {
    /// A repo whose ledger is clean: the compiled-in REQUIRED row, one
    /// `gate-wired` row whose published cell it can carry, and every reference
    /// resolving.
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().canonicalize().expect("canonicalize");
        let r = Repo { _dir: dir, root };
        r.write("registries/claims.toml", "schema_version = 1\n");
        r.write("CHARTER.md", CHARTER_TABLE);
        r.write("scripts/check.sh", "#!/bin/sh\nexit 0\n");
        r.write(
            "crates/demo/tests/a.rs",
            "#[test]\nfn alpha_is_green() {}\n#[test]\nfn beta_is_red() {}\n",
        );
        r.set_ledger(&(header() + &absent_row() + &wired_row()));
        r
    }

    fn path(&self, rel: &str) -> PathBuf {
        self.root.join(rel)
    }

    fn write(&self, rel: &str, body: &str) {
        let p = self.path(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, body).unwrap();
    }

    fn remove(&self, rel: &str) {
        std::fs::remove_file(self.path(rel)).unwrap();
    }

    fn set_ledger(&self, body: &str) {
        self.write("registries/capability-maturity.toml", body);
    }

    fn gate(&self, args: &[&str]) -> (i32, String) {
        run_gate(&self.root, args)
    }

    fn git(&self, args: &[&str]) {
        let out = Command::new("git")
            .current_dir(&self.root)
            .args(args)
            .output()
            .expect("git");
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    /// Initialise git and commit everything; returns the new HEAD sha.
    fn commit_all(&self) -> String {
        self.git(&["init", "-q"]);
        self.git(&["add", "-A"]);
        self.git(&[
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "user.name=fixture",
            "commit",
            "-q",
            "--no-verify",
            "-m",
            "base",
        ]);
        let out = Command::new("git")
            .current_dir(&self.root)
            .args(["rev-parse", "HEAD"])
            .output()
            .expect("git");
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }
}

fn header() -> String {
    "schema_version = 1\n\n[policy]\nstaleness_days = 90\n".to_string()
}

const LONG_CLAIM: &str =
    "a claim sentence long enough that a reviewer has something concrete to disagree with";

/// The compiled-in REQUIRED row, in a shape that passes.
fn absent_row() -> String {
    format!(
        "\n[[capability]]\n\
         id = \"l3.external-oracle-factual\"\n\
         title = \"External oracle for factual content\"\n\
         level = \"absent\"\n\
         owner = \"fixture\"\n\
         last_review = \"2099-01-01\"\n\
         claim = \"{LONG_CLAIM}\"\n\
         charter_claim = {{ file = \"CHARTER.md\", row = \"L3 External oracle (factual content)\", status = \"NO\" }}\n\
         evidence = [ {{ kind = \"contract\", ref = \"CHARTER.md\" }} ]\n"
    )
}

/// A `gate-wired` row that carries its published cell. This is the known-GOOD.
fn wired_row() -> String {
    format!(
        "\n[[capability]]\n\
         id = \"demo.wired\"\n\
         title = \"A capability whose evidence carries its published cell\"\n\
         level = \"gate-wired\"\n\
         owner = \"fixture\"\n\
         last_review = \"2099-01-01\"\n\
         claim = \"{LONG_CLAIM}\"\n\
         charter_claim = {{ file = \"CHARTER.md\", row = \"L9 Something wired\", status = \"YES · wired\" }}\n\
         evidence = [\n\
         \x20 {{ kind = \"test\", ref = \"crates/demo/tests/a.rs::alpha_is_green\" }},\n\
         \x20 {{ kind = \"contract\", ref = \"scripts/check.sh\" }},\n\
         ]\n"
    )
}

/// `wired_row` with one line rewritten.
fn wired_row_with(from: &str, to: &str) -> String {
    let w = wired_row();
    assert!(w.contains(from), "fixture edit target {from:?} not present");
    w.replace(from, to)
}

// ── 1. known-GOOD ─────────────────────────────────────────────────────────

#[test]
fn good_a_well_formed_in_date_ledger_passes() {
    let r = Repo::new();
    let (code, out) = r.gate(&["capability-maturity"]);
    assert_eq!(code, OK, "{out}");
    assert!(out.contains("rows=2"), "{out}");
    assert!(out.contains("charter_checked=2"), "{out}");
}

#[test]
fn good_extra_evidence_beyond_the_obligation_is_not_refused() {
    let r = Repo::new();
    r.set_ledger(
        &(header()
            + &absent_row()
            + &wired_row_with(
                "{ kind = \"contract\", ref = \"scripts/check.sh\" },",
                "{ kind = \"contract\", ref = \"scripts/check.sh\" },\n  { kind = \"known_bad\", ref = \"crates/demo/tests/a.rs::beta_is_red\" },",
            )),
    );
    let (code, out) = r.gate(&["capability-maturity"]);
    assert_eq!(code, OK, "{out}");
}

#[test]
fn good_a_published_absent_cell_needs_no_test() {
    // `absent` obliges a contract and nothing else: an over-strict gate that
    // demanded a test for a capability we do not have would be unsatisfiable.
    let r = Repo::new();
    r.set_ledger(&(header() + &absent_row()));
    let (code, out) = r.gate(&["capability-maturity"]);
    assert_eq!(code, OK, "{out}");
}

// ── 2. known-bad ──────────────────────────────────────────────────────────

#[test]
fn bad_expired_row_is_red_and_names_the_row_and_its_age() {
    let r = Repo::new();
    r.set_ledger(
        &(header()
            + &absent_row()
            + &wired_row_with(
                "last_review = \"2099-01-01\"",
                "last_review = \"2020-01-01\"",
            )),
    );
    let (code, out) = r.gate(&["capability-maturity"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("demo.wired"), "{out}");
    assert!(out.contains("EXPIRED"), "{out}");
    assert!(out.contains("days old"), "{out}");
}

#[test]
fn bad_dangling_test_ref_is_red_and_names_row_and_ref() {
    let r = Repo::new();
    r.set_ledger(
        &(header()
            + &absent_row()
            + &wired_row_with("a.rs::alpha_is_green", "a.rs::a_function_nobody_wrote")),
    );
    let (code, out) = r.gate(&["capability-maturity"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("demo.wired"), "{out}");
    assert!(out.contains("a_function_nobody_wrote"), "{out}");
    assert!(out.contains("crates/demo/tests/a.rs"), "{out}");
}

#[test]
fn bad_test_ref_into_a_file_that_is_gone_is_red() {
    let r = Repo::new();
    r.remove("crates/demo/tests/a.rs");
    let (code, out) = r.gate(&["capability-maturity"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("not a readable file"), "{out}");
}

#[test]
fn bad_contract_ref_naming_nothing_is_red() {
    let r = Repo::new();
    r.set_ledger(
        &(header()
            + &absent_row()
            + &wired_row_with(
                "ref = \"scripts/check.sh\"",
                "ref = \"scripts/not_here.sh\"",
            )),
    );
    let (code, out) = r.gate(&["capability-maturity"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("names nothing readable"), "{out}");
}

#[test]
fn bad_blank_field_is_a_schema_error_never_permission() {
    for (from, to, needle) in [
        ("owner = \"fixture\"", "owner = \"\"", "`owner`"),
        ("level = \"gate-wired\"", "level = \"\"", "`level`"),
        (
            "last_review = \"2099-01-01\"",
            "last_review = \"\"",
            "`last_review`",
        ),
        ("claim = \"a claim sentence", "claim = \"\" #", "`claim`"),
    ] {
        let r = Repo::new();
        let row = if from.starts_with("claim") {
            wired_row().replace(&format!("claim = \"{LONG_CLAIM}\""), "claim = \"\"")
        } else {
            wired_row_with(from, to)
        };
        r.set_ledger(&(header() + &absent_row() + &row));
        let (code, out) = r.gate(&["capability-maturity"]);
        assert_eq!(code, ERROR, "{from:?} -> {out}");
        assert!(out.contains(needle), "{from:?} -> {out}");
    }
}

#[test]
fn bad_empty_evidence_list_is_a_schema_error() {
    let r = Repo::new();
    let row = wired_row();
    let start = row.find("evidence = [").unwrap();
    r.set_ledger(&(header() + &absent_row() + &row[..start] + "evidence = []\n"));
    let (code, out) = r.gate(&["capability-maturity"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("empty `evidence`"), "{out}");
}

#[test]
fn bad_level_outside_the_lattice_is_a_schema_error() {
    let r = Repo::new();
    r.set_ledger(
        &(header()
            + &absent_row()
            + &wired_row_with("level = \"gate-wired\"", "level = \"shipped\"")),
    );
    let (code, out) = r.gate(&["capability-maturity"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("not in the lattice"), "{out}");
}

#[test]
fn bad_unmet_level_obligation_is_red() {
    let r = Repo::new();
    // gate-wired with the test reference removed: a level claiming a wired gate
    // that cannot name the function which would go red.
    let row = wired_row().replace(
        "  { kind = \"test\", ref = \"crates/demo/tests/a.rs::alpha_is_green\" },\n",
        "",
    );
    r.set_ledger(&(header() + &absent_row() + &row));
    let (code, out) = r.gate(&["capability-maturity"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("`test` reference"), "{out}");
    assert!(out.contains("demo.wired"), "{out}");
}

#[test]
fn bad_proven_to_trip_without_a_known_bad_is_red() {
    let r = Repo::new();
    r.set_ledger(
        &(header()
            + &absent_row()
            + &wired_row_with("level = \"gate-wired\"", "level = \"proven-to-trip\"")),
    );
    let (code, out) = r.gate(&["capability-maturity"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("`known_bad` reference"), "{out}");
}

#[test]
fn bad_published_wired_cell_over_a_level_that_cannot_carry_it_is_red() {
    let r = Repo::new();
    // The evidence is trimmed to what `smoke-checked` obliges, and the level is
    // lowered to match it — while the published cell still says wired. This is
    // the L3 failure's exact shape.
    let row = wired_row()
        .replace("level = \"gate-wired\"", "level = \"smoke-checked\"")
        .replace(
            "  { kind = \"test\", ref = \"crates/demo/tests/a.rs::alpha_is_green\" },\n",
            "",
        );
    r.set_ledger(&(header() + &absent_row() + &row));
    let (code, out) = r.gate(&["capability-maturity"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("publishes"), "{out}");
    assert!(out.contains("gate-wired"), "{out}");
}

#[test]
fn bad_drifted_charter_cell_is_red_and_names_both_sides() {
    let r = Repo::new();
    r.write(
        "CHARTER.md",
        &CHARTER_TABLE.replace("**YES · wired**", "**NO**"),
    );
    let (code, out) = r.gate(&["capability-maturity"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("DRIFT"), "{out}");
    assert!(out.contains("YES · wired"), "{out}");
}

#[test]
fn bad_charter_row_that_no_longer_exists_is_red() {
    let r = Repo::new();
    r.write(
        "CHARTER.md",
        &CHARTER_TABLE.replace("**L9 Something wired**", "**L9 Renamed**"),
    );
    let (code, out) = r.gate(&["capability-maturity"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("no table row labelled"), "{out}");
}

#[test]
fn bad_missing_required_row_is_a_schema_error() {
    let r = Repo::new();
    r.set_ledger(&(header() + &wired_row()));
    let (code, out) = r.gate(&["capability-maturity"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("l3.external-oracle-factual"), "{out}");
}

#[test]
fn bad_duplicate_ids_are_a_schema_error() {
    let r = Repo::new();
    r.set_ledger(&(header() + &absent_row() + &wired_row() + &wired_row()));
    let (code, out) = r.gate(&["capability-maturity"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("duplicate `id`"), "{out}");
}

#[test]
fn bad_widened_staleness_window_is_a_schema_error() {
    let r = Repo::new();
    r.set_ledger(
        &(header().replace("staleness_days = 90", "staleness_days = 3650")
            + &absent_row()
            + &wired_row()),
    );
    let (code, out) = r.gate(&["capability-maturity"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("never widen it"), "{out}");
}

#[test]
fn bad_dropping_every_charter_cross_check_is_a_schema_error() {
    let r = Repo::new();
    let stripped = absent_row()
        .lines()
        .filter(|l| !l.starts_with("charter_claim"))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    let stripped_wired = wired_row()
        .lines()
        .filter(|l| !l.starts_with("charter_claim"))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    r.set_ledger(&(header() + &stripped + &stripped_wired));
    let (code, out) = r.gate(&["capability-maturity"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("quote a published CHARTER cell"), "{out}");
}

// ── 3. anti-vacuous ───────────────────────────────────────────────────────

#[test]
fn anti_vacuous_zero_rows_is_an_error_not_a_pass() {
    let r = Repo::new();
    r.set_ledger(&header());
    let (code, out) = r.gate(&["capability-maturity"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("vacuous ledger"), "{out}");
}

#[test]
fn anti_vacuous_a_charter_file_with_no_table_is_an_error() {
    let r = Repo::new();
    r.write("CHARTER.md", "# charter\n\nno tables in this document\n");
    let (code, out) = r.gate(&["capability-maturity"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("zero markdown table rows"), "{out}");
}

#[test]
fn anti_vacuous_a_missing_ledger_is_an_error_not_a_pass() {
    let r = Repo::new();
    r.remove("registries/capability-maturity.toml");
    let (code, out) = r.gate(&["capability-maturity"]);
    assert_eq!(code, ERROR, "{out}");
}

#[test]
fn an_unknown_flag_is_usage_not_a_silent_pass() {
    let r = Repo::new();
    let (code, out) = r.gate(&["capability-maturity", "--quite"]);
    assert_eq!(code, 3, "{out}");
}

// ── 4. commit evidence ────────────────────────────────────────────────────

#[test]
fn a_commit_reference_resolves_when_git_can_name_it_and_is_red_when_it_cannot() {
    let r = Repo::new();
    let head = r.commit_all();
    let short = &head[..8];

    r.set_ledger(
        &(header()
            + &absent_row().replace(
                "evidence = [ { kind = \"contract\", ref = \"CHARTER.md\" } ]",
                &format!(
                    "evidence = [ {{ kind = \"contract\", ref = \"CHARTER.md\" }}, {{ kind = \"commit\", ref = \"{short}\" }} ]"
                ),
            )
            + &wired_row()),
    );
    let (code, out) = r.gate(&["capability-maturity"]);
    assert_eq!(code, OK, "{out}");

    r.set_ledger(
        &(header()
            + &absent_row().replace(
                "evidence = [ { kind = \"contract\", ref = \"CHARTER.md\" } ]",
                "evidence = [ { kind = \"contract\", ref = \"CHARTER.md\" }, { kind = \"commit\", ref = \"0123456789abcdef0123\" } ]",
            )
            + &wired_row()),
    );
    let (code, out) = r.gate(&["capability-maturity"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("not a commit"), "{out}");
}

// ── 5. the live ledger ────────────────────────────────────────────────────

/// Row ids named by the gate's findings on the live tree.
fn live_finding_ids(out: &str) -> Vec<String> {
    let mut ids: Vec<String> = Vec::new();
    for line in out.lines() {
        let Some(rest) = line.split("[[capability]] ").nth(1) else {
            continue;
        };
        let id = rest.split(':').next().unwrap_or("").trim().to_string();
        if !id.is_empty() && !ids.contains(&id) {
            ids.push(id);
        }
    }
    ids.sort();
    ids
}

#[test]
fn the_live_ledger_is_schema_clean_and_its_findings_are_the_named_debts() {
    let root = engine_root();
    let (code, out) = run_gate(&root, &["capability-maturity"]);
    assert_ne!(
        code, ERROR,
        "the shipped ledger must be schema clean and every reference must be evaluable:\n{out}"
    );
    assert_ne!(code, 3, "the gate was invoked wrongly:\n{out}");

    let found = live_finding_ids(&out);
    let mut expected: Vec<String> = KNOWN_DEBTS.iter().map(|(id, _)| (*id).into()).collect();
    expected.sort();
    assert_eq!(
        found, expected,
        "the live ledger's outstanding findings changed.\n\
         If a debt was PAID OFF, strike its row from KNOWN_DEBTS in this file.\n\
         If a NEW one appeared, it is a finding about the repo — file it, then add it here \
         with a reason.\ngate output:\n{out}"
    );
    if expected.is_empty() {
        assert_eq!(code, OK, "{out}");
    } else {
        assert_eq!(code, VIOLATION, "{out}");
    }
}

#[test]
fn the_live_ledger_has_no_expired_row_and_no_dangling_reference() {
    let root = engine_root();
    let (_, out) = run_gate(&root, &["capability-maturity"]);
    for needle in [
        "EXPIRED",
        "names nothing readable",
        "not a readable file",
        "does not define",
        "not a commit",
        "no table row labelled",
        "DRIFT",
    ] {
        assert!(
            !out.contains(needle),
            "the shipped ledger carries a {needle:?} finding, which is never an accepted debt:\n{out}"
        );
    }
}

#[test]
fn every_known_debt_carries_a_reason() {
    assert!(
        !KNOWN_DEBTS.is_empty() || cfg!(debug_assertions),
        "an empty debt list is allowed only once the debts are really paid"
    );
    for (id, reason) in KNOWN_DEBTS {
        assert!(!id.is_empty(), "a debt with no row id is not tracked");
        assert!(
            reason.len() >= 40,
            "{id}: a debt without a reason a reviewer can disagree with is not recorded, it is hidden"
        );
    }
}

#[test]
fn the_gate_is_registered_and_listed() {
    let root = engine_root();
    let (code, out) = run_gate(&root, &["list"]);
    assert_eq!(code, OK);
    assert!(out.contains("capability-maturity"), "{out}");
}

/// Anti-vacuous, from the other side: if the fixture stopped producing verdicts
/// this suite would pass while checking nothing.
#[test]
fn the_suite_reached_both_verdicts() {
    let r = Repo::new();
    let (green, _) = r.gate(&["capability-maturity"]);
    r.set_ledger(
        &(header()
            + &absent_row()
            + &wired_row_with(
                "last_review = \"2099-01-01\"",
                "last_review = \"2020-01-01\"",
            )),
    );
    let (red, _) = r.gate(&["capability-maturity"]);
    assert_eq!(green, OK);
    assert_eq!(red, VIOLATION);
    assert_ne!(
        green, red,
        "the fixture produced one verdict for both trees"
    );
}
