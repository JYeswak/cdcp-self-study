//! Differential harness: `cdcp_gate verify-coverage` against
//! `scripts/verify_coverage.py` (bd-substrate-rust-migration-jhd.6).
//!
//! The Python script is the oracle for this port and stays in the tree for
//! exactly that reason. Every case below runs BOTH implementations on the same
//! inputs and asserts stdout, stderr, and exit code match byte for byte. A
//! disagreement on any byte fails the port, not the oracle.
//!
//! The case list starts from the enumeration of `scripts/selftest_l6_coverage.sh`,
//! which is the L4 known-bad suite `check.sh` already runs —
//!
//!   a) empty bank dir                       -> ERROR (anti-vacuous)
//!   b) bank holding only module-1 items     -> RED (every other module SHORT)
//!   c) live bank                            -> GREEN
//!
//! and adds the legs the rebase (bd-lt7) introduced, which that shell suite
//! predates and does not reach —
//!
//!   d) a declared module with zero items    -> RED, naming the module
//!   e) an exemption with a blank reason     -> schema ERROR, and the module
//!                                              STAYS REQUIRED so the shortfall
//!                                              still reports
//!   f) `[[domain_min]]` for an undeclared module -> ERROR (cross-source drift)
//!   g) empty registry / missing registry / every module exempted -> ERROR
//!   h) emission ORDER: required modules, then recorded exemptions, then
//!      undeclared extras, then the failure list, each numerically ascending
//!   i) `--write-json`, whose summary bytes are compared as well as its stdout
//!
//! and the legs the two 2026-08-14 beads added —
//!
//!   j) retiring approved items in a copy of the live bank until one module is
//!      under its floor -> RED naming module, count and floor. NO FILE IS ADDED
//!      OR REMOVED, so a file-counting gate stays GREEN on this fixture by
//!      construction; the case asserts the scanned count is unchanged and still
//!      clears the floor, which is the defect made observable
//!      (bd-coverage-counts-retired-items-49jh)
//!   k) a bank whose items are all retired or draft -> ERROR, not a pass, even
//!      though the file count is healthy
//!   l) a `status` outside the C1 lifecycle -> ERROR naming the item, never a
//!      silent drop into "not approved"
//!   m) a `--write-json` that CANNOT succeed, two ways: a parent that is a file
//!      (the mkdir refuses) and a target that is a directory (the atomic rename
//!      refuses after the temp write). Both sides must print NOTHING, exit
//!      non-zero, leave no artifact and no `.tmp` residue
//!      (bd-verify-coverage-verdict-before-write-rk9n)
//!   n) the ledger written from a COPY of the live tree is byte-identical
//!      across the two AND equal to the tracked `web/data/coverage.json`
//!   o) omitted `--policy` on an isolated `--bank`/`--domains` fixture does
//!      NOT read the shipped `knowledge/bank_policy.toml` (bd-conu). The
//!      leak named live modules 3–15. After bd-j98g the no-file path is
//!      RED (missing policy), still without those live-floor findings.
//!   p) a missing policy file is ERROR, not a lowered N=1 floor (bd-j98g).
//!      A present file with empty `[[domain_min]]` stays N=1 and must not
//!      report like the missing-file path.
//!
//! # THE VERDICT-SHAPE DETECTOR
//!
//! [`assert_no_success_token_on_a_failing_path`] runs on EVERY case, PER SIDE.
//! It is the leg proven by a mutation pair on 2026-08-14: make both
//! implementations print the success token regardless of the verdict and this
//! suite exits 101 on 12 cases; delete that one assertion with the mutation
//! still in place and it returns to 0. Nothing else in the suite catches it,
//! which is what makes it the detector rather than a coincidence.
//!
//! It is asserted PER SIDE and not merely across the two because a differential
//! only catches a regression that lands on ONE side. Two implementations that
//! both regress agree with each other perfectly — and that is exactly what
//! happened here, where the Python printed its verdict above an unguarded write
//! and the Rust buffered the same verdict into `Outcome.stdout`, which `Halt`
//! preserves.
//!
//! ANTI-VACUOUS DISCIPLINE. A differential that silently compares nothing passes
//! exactly like one that compared everything, so: a missing `python3` is a
//! FAILURE and never a skip; a specimen bank that copied zero files is a
//! FAILURE; a fixture registry that declares zero modules when it meant to
//! declare several is a FAILURE; and every case increments a counter that is
//! asserted at the end.
//!
//! NOTHING HERE HARDCODES A MODULE COUNT. The live-tree expectations are
//! derived from `knowledge/domains.toml` at run time, because a test that
//! writes today's registry size down as a constant is the same defect bd-lt7
//! was opened for, one level up.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const BIN: &str = env!("CARGO_BIN_EXE_cdcp_gate");
const ORACLE: &str = "scripts/verify_coverage.py";
const GATE: &str = "verify-coverage";

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

fn python(root: &Path, args: &[&str]) -> Run {
    let out = Command::new("python3")
        .current_dir(root)
        .arg(ORACLE)
        .args(args)
        .output()
        .unwrap_or_else(|e| {
            panic!(
                "python3 {ORACLE} could not run ({e}). The oracle is REQUIRED: a differential \
                 that cannot run its reference is a failure, never a skip."
            )
        });
    Run {
        code: out.status.code().unwrap_or(-1),
        stdout: out.stdout,
        stderr: out.stderr,
    }
}

/// The BUILT binary, never `cargo run`: cargo writes build diagnostics to
/// stderr, and a sibling gate's warning would land in the captured stream and
/// read as a divergence.
fn rust(root: &Path, args: &[&str]) -> Run {
    let out = Command::new(BIN)
        .current_dir(root)
        .arg("--root")
        .arg(root)
        .arg(GATE)
        .args(args)
        .output()
        .expect("run cdcp_gate");
    Run {
        code: out.status.code().unwrap_or(-1),
        stdout: out.stdout,
        stderr: out.stderr,
    }
}

/// Every string this gate emits that a human would read as "it passed".
///
/// Keyed by TOKEN rather than by line number, and checked on EVERY case rather
/// than on the handful that happen to be RED, because the defect this guards
/// is not "the wrong line was printed" — it is "a success line was printed on a
/// path that had not finished deciding yet".
const SUCCESS_TOKENS: &[&str] = &["PASS", "coverage GREEN"];

/// VERDICT SHAPE: no success token on a path that returned non-zero
/// (bd-verify-coverage-verdict-before-write-rk9n, the third instance of the
/// class bd-lt7 and bd-builder-verdict-shape-qm65 fixed).
///
/// Asserted PER SIDE rather than only across the two. A differential only
/// catches a regression that lands on ONE side; two implementations that both
/// regress agree with each other perfectly and the byte comparison above stays
/// green while both of them lie. That is not hypothetical here: the Python
/// printed its verdict before an unguarded write and the Rust buffered the same
/// verdict into `Outcome.stdout`, which `Halt` preserves — the two agreed, byte
/// for byte, on a PASS above exit 1.
fn assert_no_success_token_on_a_failing_path(label: &str, side: &str, r: &Run) {
    if r.code == 0 {
        return;
    }
    for token in SUCCESS_TOKENS {
        assert!(
            !r.out().contains(token),
            "[{label}] {side} exited {} with the success token {token:?} on stdout. This is \
             the defect itself: a reader skimming stdout sees success while CI sees \
             non-zero, and which one wins depends on whether anyone looked:\n{}",
            r.code,
            r.out()
        );
    }
}

/// The whole acceptance bar in one function. Returns the (identical) run so a
/// case can additionally assert *what* the shared output says.
fn assert_byte_identical(label: &str, root: &Path, args: &[&str]) -> Run {
    let py = python(root, args);
    let rs = rust(root, args);

    assert_no_success_token_on_a_failing_path(label, "python", &py);
    assert_no_success_token_on_a_failing_path(label, "rust", &rs);

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

fn write(path: &Path, body: &str) {
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p).unwrap();
    }
    std::fs::write(path, body).unwrap();
}

/// A present policy with no `[[domain_min]]` rows. This is the honest N=1
/// default — distinguishable from a missing file (bd-j98g).
fn write_empty_policy(path: &Path) {
    write(path, "schema_version = 1\n");
}

/// A synthetic domain registry declaring exactly `orders`, in the order given —
/// which is deliberately NOT always ascending, so the report's own ordering is
/// under test rather than inherited from the fixture.
fn domains_registry(orders: &[i64]) -> String {
    let mut s = String::from("schema_version = 1\n");
    for o in orders {
        s.push_str(&format!(
            "\n[[domain]]\nid = \"d{o:02}-fixture\"\norder = {o}\n"
        ));
    }
    s
}

/// One bank item, in the single-table `id = ...` form the oracle accepts.
///
/// The status is EXPLICIT and `approved`, because the floors count the approved
/// pool (bd-coverage-counts-retired-items-49jh) and a fixture that meant "this
/// module is stocked" must plant items a learner could actually be drawn. A
/// status-less item is `draft` by C1's default and stocks nothing — see
/// [`item_with_status`] for the fixtures that mean exactly that.
fn item(id: &str, module: i64) -> String {
    item_with_status(id, module, Some("approved"))
}

/// One bank item with an explicit C1 status, or none at all.
fn item_with_status(id: &str, module: i64, status: Option<&str>) -> String {
    let mut s = format!("id = {id:?}\nmodule = {module}\n");
    if let Some(st) = status {
        s.push_str(&format!("status = {st:?}\n"));
    }
    s
}

/// A bank directory holding approved `(id, module)` items, and the count planted.
fn plant_bank(dir: &Path, items: &[(&str, i64)]) -> usize {
    std::fs::create_dir_all(dir).unwrap();
    for (id, m) in items {
        write(&dir.join(format!("{id}.toml")), &item(id, *m));
    }
    items.len()
}

/// A bank directory holding `(id, module, status)` items — the mixed-pool
/// fixture the floor rebase needs, where `None` means no `status` key at all.
fn plant_bank_with_status(dir: &Path, items: &[(&str, i64, Option<&str>)]) -> usize {
    std::fs::create_dir_all(dir).unwrap();
    for (id, m, st) in items {
        write(
            &dir.join(format!("{id}.toml")),
            &item_with_status(id, *m, *st),
        );
    }
    items.len()
}

/// Copy the live bank into TEMP so a fixture never mutates `bank/items`.
fn specimen_bank(root: &Path, into: &Path) -> usize {
    std::fs::create_dir_all(into).unwrap();
    let mut n = 0usize;
    for e in std::fs::read_dir(root.join("bank/items"))
        .expect("live bank/items")
        .flatten()
    {
        let name = e.file_name().to_string_lossy().into_owned();
        if name.ends_with(".toml") {
            std::fs::copy(e.path(), into.join(&name)).unwrap();
            n += 1;
        }
    }
    assert!(
        n > 0,
        "copied zero bank items into TEMP — a vacuous specimen is an ERROR, not a pass"
    );
    n
}

/// How many modules `knowledge/domains.toml` declares, read from the registry
/// rather than written down here. See the module header.
fn declared_module_count(root: &Path) -> usize {
    let text = std::fs::read_to_string(root.join("knowledge/domains.toml")).expect("domains.toml");
    let n = text
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("order ="))
        .count();
    assert!(
        n > 1,
        "the live domain registry parsed to {n} module(s) — a vacuous derivation is an ERROR"
    );
    n
}

/// The item `selftest_l6_coverage.sh` plants for its case (b), byte for byte.
const SELFTEST_M01_ITEM: &str = r#"id = "selftest-m01-only"
module = 1
stem = "selftest planted item — not for exam use"
choices = ["A", "B", "C", "D"]
correct = "A"
explanation = "planted for coverage selftest only"
topic_ids = ["m01-importance"]
bloom = "remember"
source_class = "original"
quantity_evidence = "qualitative_only"
"#;

// ── the oracle must exist at all ───────────────────────────────────────────

#[test]
fn the_oracle_is_present_and_runnable() {
    let root = engine_root();
    assert!(
        root.join(ORACLE).is_file(),
        "{ORACLE} is the differential oracle for this port; without it the port is unverified"
    );
    // Not `--help`: run the real thing on the real tree, so a Python that
    // imports but cannot execute is caught here rather than read as agreement.
    let py = python(&root, &[]);
    assert_eq!(
        py.code,
        0,
        "the oracle is RED on the live tree, so no differential below can be trusted:\n{}\n{}",
        py.out(),
        py.err()
    );
}

// ── (c) the GREEN case, and the rebase's own signals ───────────────────────

#[test]
fn live_tree_is_byte_identical_and_green() {
    let root = engine_root();
    let declared = declared_module_count(&root);

    let rs = assert_byte_identical("c live tree", &root, &[]);
    assert_eq!(rs.code, 0, "live tree must be GREEN: {}", rs.out());
    assert!(rs.out().starts_with("PASS\n"), "{}", rs.out());
    assert!(rs.out().contains("coverage GREEN"), "{}", rs.out());
    assert!(
        rs.err().is_empty(),
        "the oracle writes nothing to stderr on the green path: {:?}",
        rs.err()
    );

    // The module set is DERIVED, and the report says so with the registry's own
    // count. Both numbers come from domains.toml, never from a literal here.
    assert!(
        rs.out()
            .contains(&format!("registry=domains.toml declares={declared}")),
        "the report must name the registry it derived from: {}",
        rs.out()
    );
    assert!(
        rs.out().contains(&format!(
            "modules ({declared} required, derived from the domain registry)"
        )),
        "every declared module must be required on the live tree: {}",
        rs.out()
    );
    // …and the last-declared module is inside the required set, which is the
    // property the rebase existed to restore. Derived, not named.
    let last = format!("m{declared:02}: ");
    assert!(
        rs.out().contains(&last),
        "the highest declared module is missing from the report: {}",
        rs.out()
    );

    // The explicit `--bank bank/items` spelling selftest_l6_coverage.sh case (c)
    // uses is a distinct code path (relative argument, resolved) and must agree
    // too.
    let rs = assert_byte_identical(
        "c live tree, explicit bank",
        &root,
        &["--bank", "bank/items"],
    );
    assert_eq!(rs.code, 0, "{}", rs.out());
}

// ── (a)(b) exactly what selftest_l6_coverage.sh exercises ─────────────────

#[test]
fn the_shell_selftest_cases_are_byte_identical() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();

    // (a) empty bank directory → vacuous ERROR
    let empty_bank = td.path().join("empty_bank");
    std::fs::create_dir_all(&empty_bank).unwrap();
    let rs = assert_byte_identical(
        "a empty bank",
        &root,
        &["--bank", empty_bank.to_str().unwrap()],
    );
    assert_ne!(rs.code, 0, "an empty bank must never pass: {}", rs.out());
    assert!(
        rs.out().contains("empty bank"),
        "the suite's needle must survive the port: {}",
        rs.out()
    );

    // (b) a bank holding only the planted module-1 item → every other declared
    // module is SHORT. The suite's needle is the module-2 shortfall line.
    let filt = td.path().join("m01_only");
    std::fs::create_dir_all(&filt).unwrap();
    write(&filt.join("planted-m01.toml"), SELFTEST_M01_ITEM);
    let rs = assert_byte_identical(
        "b m01-only bank",
        &root,
        &["--bank", filt.to_str().unwrap()],
    );
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out().contains("module 2:"),
        "the suite's needle must survive the port: {}",
        rs.out()
    );

    // nothing planted may leak into the live tree
    assert!(
        !root.join("bank/items/planted-m01.toml").exists(),
        "specimen leaked into the live bank"
    );
}

// ── (o) bd-conu: omitted --policy must not read the shipped policy ─────────
//
// THE LEAK: argparse defaulted --policy to the live knowledge/bank_policy.toml.
// A fixture that passed isolated --bank/--domains and no --policy therefore
// graded the shipped [[domain_min]] rows. Measured 2026-08-14: policy=present
// and 14 findings `[[domain_min]] module N is not a required module` for
// N=2..15, none of which the fixture declared. A case can go RED for a reason
// it did not inject, or GREEN because the live policy supplied something the
// fixture forgot.
//
// THE DETECTOR: one declared module, one approved item, NO --policy. The leak
// makes this name live floors + undeclared-module drift for modules 2–15.
// After conu, omitted --policy does not read the shipped file. After bd-j98g
// the same no-file path is RED naming the missing local policy — still
// without those live-floor findings. Do not reopen conu: a sibling policy
// beside the fixture domains is still the one that is read.
// Should-fail: two declared modules, only module 1 stocked — RED naming
// module 2 only (plus the missing-policy line). Anti-vacuous: empty bank
// stays an error, still without live-policy findings.

#[test]
fn omitted_policy_does_not_grade_the_live_bank_policy() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();
    let bank = td.path().join("bank");
    let planted = plant_bank(&bank, &[("only", 1)]);
    assert!(planted > 0, "a vacuous fixture bank is an ERROR");
    let bank_s = bank.to_str().unwrap();

    // Isolated one-module registry, no --policy. Must NOT read the shipped
    // policy (bd-conu) and must RED for the missing local file (bd-j98g).
    let one = td.path().join("one.toml");
    write(&one, &domains_registry(&[1]));
    let one_s = one.to_str().unwrap();
    let rs = assert_byte_identical(
        "bd-conu isolated no-policy does not read the live file",
        &root,
        &["--bank", bank_s, "--domains", one_s],
    );
    assert_ne!(
        rs.code,
        0,
        "a missing local policy must be RED, not a lowered N=1 pass:\n{}",
        rs.out()
    );
    assert!(
        rs.out().contains("policy=absent"),
        "omitted --policy must mean no policy, not the shipped file:\n{}",
        rs.out()
    );
    assert!(
        !rs.out().contains("policy=absent (N=1 OQ-05)"),
        "N=1 must not be claimed as a fallback for a missing file:\n{}",
        rs.out()
    );
    assert!(
        rs.out().contains("bank_policy.toml missing:"),
        "the missing local policy must be named:\n{}",
        rs.out()
    );
    assert!(
        !rs.out().contains("[[domain_min]] module"),
        "live domain_min rows leaked into an isolated fixture:\n{}",
        rs.out()
    );
    assert!(
        rs.out()
            .contains("m01: 1 approved of 1 scanned (min 1) [ok]"),
        "{}",
        rs.out()
    );
    for n in 2..=15 {
        assert!(
            !rs.out()
                .contains(&format!("module {n} is not a required module")),
            "live policy named module {n} the fixture never declared:\n{}",
            rs.out()
        );
    }

    // Known-bad: two declared modules, only module 1 stocked, still no --policy.
    // RED, and the findings name ONLY the fixture's modules.
    let two = td.path().join("two.toml");
    write(&two, &domains_registry(&[1, 2]));
    let two_s = two.to_str().unwrap();
    let rs = assert_byte_identical(
        "bd-conu isolated no-policy names only fixture modules",
        &root,
        &["--bank", bank_s, "--domains", two_s],
    );
    assert_ne!(rs.code, 0, "the planted shortfall passed:\n{}", rs.out());
    assert!(
        rs.out()
            .contains("module 2: 0 approved < min 1 (0 scanned, 0 not approved)"),
        "the planted shortfall must be named: {}",
        rs.out()
    );
    assert!(
        !rs.out().contains("[[domain_min]] module"),
        "live floors leaked onto a known-bad:\n{}",
        rs.out()
    );
    for n in 3..=15 {
        assert!(
            !rs.out().contains(&format!("module {n}")),
            "finding mentioned module {n} the fixture never declared:\n{}",
            rs.out()
        );
    }

    // Anti-vacuous: empty bank + isolated registry + no --policy is still ERROR.
    let empty = td.path().join("empty");
    std::fs::create_dir_all(&empty).unwrap();
    let rs = assert_byte_identical(
        "bd-conu isolated empty bank is still an error",
        &root,
        &["--bank", empty.to_str().unwrap(), "--domains", one_s],
    );
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out().contains("empty bank"),
        "the empty-bank needle vanished: {}",
        rs.out()
    );
    assert!(
        !rs.out().contains("[[domain_min]] module"),
        "live policy leaked onto the empty-bank path:\n{}",
        rs.out()
    );

    // Same-root: a policy sitting BESIDE the fixture domains IS used. This is
    // the other half of the AC — we did not "ignore policy whenever omitted".
    write(
        &td.path().join("bank_policy.toml"),
        "[[domain_min]]\nmodule = 1\nmin_items = 5\n",
    );
    let rs = assert_byte_identical(
        "bd-conu sibling policy is the fixture's",
        &root,
        &["--bank", bank_s, "--domains", one_s],
    );
    assert_ne!(
        rs.code,
        0,
        "the fixture sibling policy was ignored:\n{}",
        rs.out()
    );
    assert!(
        rs.out()
            .contains("module 1: 1 approved < min 5 (1 scanned, 0 not approved)"),
        "the fixture sibling policy must raise the floor:\n{}",
        rs.out()
    );
    assert!(
        rs.out().contains("policy=present"),
        "a sibling policy file must count as present:\n{}",
        rs.out()
    );
}

// ── (p) bd-j98g: missing policy is ERROR, not a lowered N=1 floor ─────────
//
// THE HOLE: load_domain_mins returned {m: DEFAULT_N} with no error when the
// resolved policy file was not a file. Absence therefore LOWERED every sized
// [[domain_min]] floor to 1. A one-item fixture PASSed; the live bank PASSed
// with `--policy` pointed at a nonexistent file (`policy=absent (N=1 OQ-05)`).
// That is the opposite of verify_objectives, where absence removes exemptions
// and makes the gate stricter.
//
// THE RULE: missing file = ERROR naming the path. A present file with empty
// [[domain_min]] keeps N=1. Those two must not report alike.
//
// THE DETECTOR: isolated one-module bank stocked at 1, `--policy` at a path
// that does not exist. Used to be GREEN with `policy=absent (N=1 OQ-05)`.
// Now RED, names the missing file, and does not claim N=1 as a fallback.
// Should-fail (distinguishable): the same fixture with an empty-but-present
// policy is GREEN at min 1. A sized floor of 20 on the same stock is RED
// naming the floor. The live bank with `--policy` at a missing path is RED
// (it used to PASS because every module has >> 1 approved item).

#[test]
fn absent_policy_is_an_error_not_a_lowered_floor() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();
    let bank = td.path().join("bank");
    let planted = plant_bank(&bank, &[("only", 1)]);
    assert!(planted > 0, "a vacuous fixture bank is an ERROR");
    let bank_s = bank.to_str().unwrap();
    let reg = td.path().join("domains.toml");
    write(&reg, &domains_registry(&[1]));
    let reg_s = reg.to_str().unwrap();

    let missing = td.path().join("no-such-bank_policy.toml");
    let missing_s = missing.to_str().unwrap();
    assert!(
        !missing.exists(),
        "the known-bad must not accidentally plant the file it claims is missing"
    );

    let rs = assert_byte_identical(
        "bd-j98g isolated missing policy is red",
        &root,
        &["--bank", bank_s, "--domains", reg_s, "--policy", missing_s],
    );
    assert_ne!(
        rs.code,
        0,
        "a fixture stocked below every sized floor must not PASS just because \
         the policy file is absent:\n{}",
        rs.out()
    );
    assert!(
        rs.out().contains("policy=absent"),
        "missing file must be reported as absent:\n{}",
        rs.out()
    );
    assert!(
        !rs.out().contains("policy=absent (N=1 OQ-05)"),
        "N=1 must not be claimed as the applied floor when the file is gone:\n{}",
        rs.out()
    );
    assert!(
        rs.out().contains(&format!("bank_policy.toml missing: {missing_s}")),
        "the missing policy path must be named:\n{}",
        rs.out()
    );
    assert!(
        rs.out()
            .contains("(absence would lower sized [[domain_min]] floors to N=1)"),
        "the failure must say WHY absence is unsafe:\n{}",
        rs.out()
    );
    assert!(
        !rs.out().contains("coverage GREEN"),
        "a success token on the missing-policy path is the defect:\n{}",
        rs.out()
    );

    // Anti-vacuous: a present file with empty [[domain_min]] is NOT the
    // missing-file path. Same bank, same registry, N=1, GREEN.
    let empty = td.path().join("empty_policy.toml");
    write_empty_policy(&empty);
    let empty_s = empty.to_str().unwrap();
    let rs = assert_byte_identical(
        "bd-j98g present-empty [[domain_min]] is n=1, not missing",
        &root,
        &["--bank", bank_s, "--domains", reg_s, "--policy", empty_s],
    );
    assert_eq!(
        rs.code,
        0,
        "a present policy with no [[domain_min]] rows is the honest N=1 default:\n{}",
        rs.out()
    );
    assert!(
        rs.out().contains("policy=present"),
        "empty [[domain_min]] must not report as absent:\n{}",
        rs.out()
    );
    assert!(
        !rs.out().contains("bank_policy.toml missing:"),
        "a present file must not be named as missing:\n{}",
        rs.out()
    );
    assert!(
        rs.out()
            .contains("m01: 1 approved of 1 scanned (min 1) [ok]"),
        "{}",
        rs.out()
    );

    // Same stock under a sized floor: RED naming the floor, policy=present.
    let sized = td.path().join("sized_policy.toml");
    write(
        &sized,
        "[[domain_min]]\nmodule = 1\nmin_items = 20\n",
    );
    let rs = assert_byte_identical(
        "bd-j98g present sized floor still raises",
        &root,
        &[
            "--bank",
            bank_s,
            "--domains",
            reg_s,
            "--policy",
            sized.to_str().unwrap(),
        ],
    );
    assert_ne!(rs.code, 0, "the sized floor must still trip:\n{}", rs.out());
    assert!(
        rs.out().contains("policy=present"),
        "{}",
        rs.out()
    );
    assert!(
        rs.out()
            .contains("module 1: 1 approved < min 20 (1 scanned, 0 not approved)"),
        "the sized floor must be named:\n{}",
        rs.out()
    );

    // The live bank with `--policy` pointed at a missing file used to PASS
    // because every module has >> 1 approved item. That is the fail-open
    // that would hide a deleted knowledge/bank_policy.toml.
    let rs = assert_byte_identical(
        "bd-j98g live bank + missing --policy is red",
        &root,
        &["--policy", missing_s],
    );
    assert_ne!(
        rs.code,
        0,
        "the live bank must not PASS with floors lowered to N=1:\n{}",
        rs.out()
    );
    assert!(
        rs.out().contains(&format!("bank_policy.toml missing: {missing_s}")),
        "{}",
        rs.out()
    );
    assert!(
        !rs.out().contains("policy=absent (N=1 OQ-05)"),
        "{}",
        rs.out()
    );
}

// ── (d) a declared module with zero items ─────────────────────────────────

#[test]
fn a_declared_module_with_zero_items_is_red_and_named() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();

    let reg = td.path().join("domains.toml");
    write(&reg, &domains_registry(&[1, 2, 3]));
    let bank = td.path().join("bank");
    let planted = plant_bank(&bank, &[("a", 1), ("b", 1), ("c", 3)]);
    assert!(planted > 0, "a vacuous fixture bank is an ERROR");
    let empty_policy = td.path().join("empty_policy.toml");
    write_empty_policy(&empty_policy);

    let rs = assert_byte_identical(
        "d declared module with zero items",
        &root,
        &[
            "--domains",
            reg.to_str().unwrap(),
            "--bank",
            bank.to_str().unwrap(),
            "--policy",
            empty_policy.to_str().unwrap(),
        ],
    );
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out()
            .contains("module 2: 0 approved < min 1 (0 scanned, 0 not approved)"),
        "the starved module must be NAMED, not merely counted: {}",
        rs.out()
    );
    assert!(
        rs.out()
            .contains("    m02: 0 approved of 0 scanned (min 1) [SHORT]"),
        "the per-module line must flag it too: {}",
        rs.out()
    );
    // The known-GOOD leg: the stocked modules are not dragged red with it.
    assert!(
        rs.out()
            .contains("m01: 2 approved of 2 scanned (min 1) [ok]"),
        "{}",
        rs.out()
    );
    assert!(
        rs.out()
            .contains("m03: 1 approved of 1 scanned (min 1) [ok]"),
        "{}",
        rs.out()
    );
}

// ── (e) an exemption without a reason is a SCHEMA ERROR, and does not exempt ─

#[test]
fn a_reasonless_exemption_is_an_error_and_leaves_the_module_required() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();

    let reg = td.path().join("domains.toml");
    write(&reg, &domains_registry(&[1, 2]));
    let bank = td.path().join("bank");
    plant_bank(&bank, &[("a", 1)]);

    for (label, body) in [
        (
            "blank reason",
            "[[coverage_exempt]]\nmodule = 2\nreason = \"\"\n",
        ),
        (
            "whitespace reason",
            "[[coverage_exempt]]\nmodule = 2\nreason = \"   \"\n",
        ),
        ("absent reason", "[[coverage_exempt]]\nmodule = 2\n"),
    ] {
        let policy = td
            .path()
            .join(format!("policy_{}.toml", label.replace(' ', "_")));
        write(&policy, body);
        let rs = assert_byte_identical(
            &format!("e exemption, {label}"),
            &root,
            &[
                "--domains",
                reg.to_str().unwrap(),
                "--bank",
                bank.to_str().unwrap(),
                "--policy",
                policy.to_str().unwrap(),
            ],
        );
        assert_ne!(rs.code, 0, "[{label}] {}", rs.out());
        assert!(
            rs.out().contains(
                "bank_policy.toml: coverage_exempt module 2 has no reason \
                 (an exemption without a reason is a schema error)"
            ),
            "[{label}] the schema error must be stated: {}",
            rs.out()
        );
        // THE POINT OF THIS CASE. A rejected exemption must not quietly
        // exempt: module 2 stays in the required set, so its shortfall still
        // reports and the module still appears in the per-module block.
        assert!(
            rs.out()
                .contains("module 2: 0 approved < min 1 (0 scanned, 0 not approved)"),
            "[{label}] the rejected exemption suppressed the shortfall: {}",
            rs.out()
        );
        assert!(
            rs.out()
                .contains("    m02: 0 approved of 0 scanned (min 1) [SHORT]"),
            "[{label}] the module left the required set: {}",
            rs.out()
        );
        assert!(
            !rs.out().contains("recorded exemptions"),
            "[{label}] a malformed row must not be recorded as an exemption: {}",
            rs.out()
        );
    }

    // The known-GOOD leg the schema check must not break: a row WITH a reason
    // does exempt, is recorded in the report, and the run is GREEN.
    let good = td.path().join("policy_good.toml");
    write(
        &good,
        "[[coverage_exempt]]\nmodule = 2\nreason = \"assessed elsewhere, see bd-fixture\"\n",
    );
    let rs = assert_byte_identical(
        "e exemption, recorded with a reason",
        &root,
        &[
            "--domains",
            reg.to_str().unwrap(),
            "--bank",
            bank.to_str().unwrap(),
            "--policy",
            good.to_str().unwrap(),
        ],
    );
    assert_eq!(
        rs.code,
        0,
        "a well-formed exemption must still be honoured: {}",
        rs.out()
    );
    assert!(
        rs.out().contains(
            "    m02: 0 approved of 0 scanned — exempt: assessed elsewhere, see bd-fixture"
        ),
        "{}",
        rs.out()
    );

    // …and an exemption may not ALSO carry a floor. Picking one is the rule.
    let both = td.path().join("policy_both.toml");
    write(
        &both,
        "[[domain_min]]\nmodule = 2\nmin_items = 3\n\
         [[coverage_exempt]]\nmodule = 2\nreason = \"conflicting\"\n",
    );
    let rs = assert_byte_identical(
        "e exemption conflicting with a floor",
        &root,
        &[
            "--domains",
            reg.to_str().unwrap(),
            "--bank",
            bank.to_str().unwrap(),
            "--policy",
            both.to_str().unwrap(),
        ],
    );
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out().contains(
            "bank_policy.toml: module 2 is both coverage_exempt and has a \
             [[domain_min]] floor — pick one"
        ),
        "{}",
        rs.out()
    );
    assert!(
        rs.out()
            .contains("module 2: 0 approved < min 3 (0 scanned, 0 not approved)"),
        "the conflicted module must stay required: {}",
        rs.out()
    );

    // An exemption naming a module the registry never declared is drift too.
    let undeclared = td.path().join("policy_undeclared.toml");
    write(
        &undeclared,
        "[[coverage_exempt]]\nmodule = 9\nreason = \"never declared\"\n",
    );
    let rs = assert_byte_identical(
        "e exemption for an undeclared module",
        &root,
        &[
            "--domains",
            reg.to_str().unwrap(),
            "--bank",
            bank.to_str().unwrap(),
            "--policy",
            undeclared.to_str().unwrap(),
        ],
    );
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out()
            .contains("bank_policy.toml: coverage_exempt module 9 is not in the domain registry"),
        "{}",
        rs.out()
    );
}

// ── (f) cross-source drift ────────────────────────────────────────────────

#[test]
fn a_domain_min_row_for_an_undeclared_module_is_an_error() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();

    let reg = td.path().join("domains.toml");
    write(&reg, &domains_registry(&[1, 2]));
    let bank = td.path().join("bank");
    plant_bank(&bank, &[("a", 1), ("b", 2)]);

    let policy = td.path().join("policy_drift.toml");
    write(&policy, "[[domain_min]]\nmodule = 9\nmin_items = 3\n");
    let rs = assert_byte_identical(
        "f domain_min for an undeclared module",
        &root,
        &[
            "--domains",
            reg.to_str().unwrap(),
            "--bank",
            bank.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
        ],
    );
    assert_ne!(
        rs.code,
        0,
        "two sources of truth disagreeing must never be a pass: {}",
        rs.out()
    );
    assert!(
        rs.out().contains(
            "bank_policy.toml: [[domain_min]] module 9 is not a required \
             module in the domain registry"
        ),
        "{}",
        rs.out()
    );

    // The known-GOOD leg: a floor keyed to a DECLARED module raises the bar and
    // is not itself a drift error.
    let aligned = td.path().join("policy_aligned.toml");
    write(&aligned, "[[domain_min]]\nmodule = 2\nmin_items = 4\n");
    let rs = assert_byte_identical(
        "f aligned domain_min raises the floor",
        &root,
        &[
            "--domains",
            reg.to_str().unwrap(),
            "--bank",
            bank.to_str().unwrap(),
            "--policy",
            aligned.to_str().unwrap(),
        ],
    );
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out()
            .contains("m02: 1 approved of 1 scanned (min 4) [SHORT]"),
        "{}",
        rs.out()
    );
    assert!(
        !rs.out().contains("is not a required module"),
        "an aligned floor must not report as drift: {}",
        rs.out()
    );

    // A malformed floor row is named rather than skipped.
    let unusable = td.path().join("policy_unusable.toml");
    write(&unusable, "[[domain_min]]\nmodule = 1\n");
    let rs = assert_byte_identical(
        "f unusable domain_min row",
        &root,
        &[
            "--domains",
            reg.to_str().unwrap(),
            "--bank",
            bank.to_str().unwrap(),
            "--policy",
            unusable.to_str().unwrap(),
        ],
    );
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out()
            .contains("bank_policy.toml: unusable [[domain_min]] row {'module': 1}"),
        "{}",
        rs.out()
    );
}

// ── (g) anti-vacuous: an empty input set is an ERROR, never a pass ────────

#[test]
fn empty_and_missing_input_sets_are_errors_in_both() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();
    let bank = td.path().join("bank");
    plant_bank(&bank, &[("a", 1)]);
    let bank_arg = bank.to_str().unwrap().to_string();

    // a registry that parses but declares nothing
    let empty_reg = td.path().join("empty_domains.toml");
    write(&empty_reg, "schema_version = 1\n");
    let rs = assert_byte_identical(
        "g registry declaring zero modules",
        &root,
        &[
            "--domains",
            empty_reg.to_str().unwrap(),
            "--bank",
            &bank_arg,
        ],
    );
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out()
            .contains("domain registry declares zero modules (vacuous coverage is ERROR)"),
        "{}",
        rs.out()
    );
    assert!(
        rs.out()
            .contains("zero required modules after exemptions (vacuous coverage is ERROR)"),
        "an emptied required set must be named too: {}",
        rs.out()
    );

    // a registry that is not there at all — reported once, and NOT also as
    // "declares zero modules", because that leg returns early
    let missing_reg = td.path().join("no_such_domains.toml");
    let rs = assert_byte_identical(
        "g missing registry",
        &root,
        &[
            "--domains",
            missing_reg.to_str().unwrap(),
            "--bank",
            &bank_arg,
        ],
    );
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out().contains("domain registry missing:"),
        "{}",
        rs.out()
    );
    assert!(
        !rs.out().contains("declares zero modules"),
        "the missing leg must not double-report: {}",
        rs.out()
    );

    // a bank directory that is not there at all
    let missing_bank = td.path().join("no_such_bank");
    let rs = assert_byte_identical(
        "g missing bank dir",
        &root,
        &["--bank", missing_bank.to_str().unwrap()],
    );
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(rs.out().contains("bank dir missing:"), "{}", rs.out());
    assert!(
        rs.out().contains("empty bank: zero items loaded"),
        "{}",
        rs.out()
    );

    // every declared module exempted → zero required → ERROR, never a green
    // report of a check that had nothing left to check
    let reg = td.path().join("two.toml");
    write(&reg, &domains_registry(&[1, 2]));
    let all_exempt = td.path().join("policy_all_exempt.toml");
    write(
        &all_exempt,
        "[[coverage_exempt]]\nmodule = 1\nreason = \"a\"\n\
         [[coverage_exempt]]\nmodule = 2\nreason = \"b\"\n",
    );
    let rs = assert_byte_identical(
        "g every module exempted",
        &root,
        &[
            "--domains",
            reg.to_str().unwrap(),
            "--policy",
            all_exempt.to_str().unwrap(),
            "--bank",
            &bank_arg,
        ],
    );
    assert_ne!(
        rs.code,
        0,
        "a required set emptied by exemptions must never be a pass: {}",
        rs.out()
    );
    assert!(
        rs.out()
            .contains("zero required modules after exemptions (vacuous coverage is ERROR)"),
        "{}",
        rs.out()
    );

    // an empty bank AND an empty registry at once — still an ERROR, still identical
    let empty_bank = td.path().join("empty_bank");
    std::fs::create_dir_all(&empty_bank).unwrap();
    let rs = assert_byte_identical(
        "g empty bank and empty registry",
        &root,
        &[
            "--bank",
            empty_bank.to_str().unwrap(),
            "--domains",
            empty_reg.to_str().unwrap(),
        ],
    );
    assert_ne!(rs.code, 0, "{}", rs.out());
}

// ── (h) emission ORDER ────────────────────────────────────────────────────

#[test]
fn emission_order_is_reproduced_exactly() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();

    // The registry declares its modules OUT of numeric order on purpose, and
    // the bank carries two modules the registry never declared. Python dict
    // order and Rust map order differ, so this pins the sort rather than luck.
    let reg = td.path().join("domains.toml");
    write(&reg, &domains_registry(&[3, 1, 22, 2]));
    let bank = td.path().join("bank");
    plant_bank(
        &bank,
        &[("a", 1), ("b", 3), ("c", 22), ("z9", 9), ("z7", 7)],
    );
    let policy = td.path().join("policy.toml");
    write(
        &policy,
        "[[coverage_exempt]]\nmodule = 22\nreason = \"later\"\n\
         [[coverage_exempt]]\nmodule = 3\nreason = \"earlier\"\n",
    );

    let rs = assert_byte_identical(
        "h emission order",
        &root,
        &[
            "--domains",
            reg.to_str().unwrap(),
            "--bank",
            bank.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
        ],
    );
    let out = rs.out();
    let at = |needle: &str| {
        out.find(needle)
            .unwrap_or_else(|| panic!("missing {needle:?} in:\n{out}"))
    };

    // header block, in the oracle's fixed order
    let (bank_at, items_at) = (at("  bank="), at("  items="));
    let (policy_at, registry_at) = (at("  policy="), at("  registry="));
    let modules_at = at("  modules (");
    assert!(
        bank_at < items_at && items_at < policy_at && policy_at < registry_at,
        "header order drifted:\n{out}"
    );
    assert!(registry_at < modules_at, "header order drifted:\n{out}");

    // required modules ascending, then exemptions ascending, then extras ascending
    let m01 = at("    m01: ");
    let m02 = at("    m02: ");
    let exempt_hdr = at("  recorded exemptions");
    let e03 = at("    m03: 1 approved of 1 scanned — exempt: earlier");
    let e22 = at("    m22: 1 approved of 1 scanned — exempt: later");
    let extras_hdr = at("  undeclared modules present in the bank");
    let x07 = at("    m07: 1 scanned (not in the domain registry)");
    let x09 = at("    m09: 1 scanned (not in the domain registry)");
    assert!(
        modules_at < m01 && m01 < m02,
        "required order drifted:\n{out}"
    );
    assert!(m02 < exempt_hdr, "block order drifted:\n{out}");
    assert!(
        exempt_hdr < e03 && e03 < e22,
        "exemption order drifted:\n{out}"
    );
    assert!(e22 < extras_hdr, "block order drifted:\n{out}");
    assert!(
        extras_hdr < x07 && x07 < x09,
        "extras order drifted:\n{out}"
    );

    // the failure list comes last, and its own order is
    // registry → exemption → floor → bank → shortfalls
    let reg2 = td.path().join("messy.toml");
    write(
        &reg2,
        &format!(
            "{}\n[[domain]]\nid = \"\"\norder = \"nope\"\n",
            domains_registry(&[1, 2])
        ),
    );
    let policy2 = td.path().join("messy_policy.toml");
    write(
        &policy2,
        "[[coverage_exempt]]\nmodule = 2\nreason = \"\"\n\
         [[domain_min]]\nmodule = 8\nmin_items = 2\n",
    );
    let bank2 = td.path().join("messy_bank");
    plant_bank(&bank2, &[("ok", 1)]);
    write(&bank2.join("zz-junk.toml"), "label = \"nothing useful\"\n");
    write(
        &bank2.join("zz-badmod.toml"),
        "id = \"zz-badmod\"\nmodule = \"nope\"\n",
    );

    let rs = assert_byte_identical(
        "h failure-list order",
        &root,
        &[
            "--domains",
            reg2.to_str().unwrap(),
            "--bank",
            bank2.to_str().unwrap(),
            "--policy",
            policy2.to_str().unwrap(),
        ],
    );
    let out = rs.out();
    let at = |needle: &str| {
        out.find(needle)
            .unwrap_or_else(|| panic!("missing {needle:?} in:\n{out}"))
    };
    let f_registry = at("    - domains.toml: {'id': '', 'order': 'nope'} has no usable order");
    let f_exempt = at("    - bank_policy.toml: coverage_exempt module 2 has no reason");
    let f_floor = at("    - bank_policy.toml: [[domain_min]] module 8 is not a required");
    let f_load = at("    - zz-junk.toml: no id or items[]");
    let f_badmod = at("    - zz-badmod: bad module 'nope'");
    let f_short = at("    - module 2: 0 approved < min 1 (0 scanned, 0 not approved)");
    assert!(
        f_registry < f_exempt
            && f_exempt < f_floor
            && f_floor < f_load
            && f_load < f_badmod
            && f_badmod < f_short,
        "failure-list order drifted:\n{out}"
    );
}

#[test]
fn the_failure_list_truncates_identically() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();

    // Enough declared-but-unstocked modules to overrun the oracle's report
    // slice, so the truncation footer is compared too.
    let orders: Vec<i64> = (1..=60).collect();
    let reg = td.path().join("many.toml");
    write(&reg, &domains_registry(&orders));
    let bank = td.path().join("bank");
    plant_bank(&bank, &[("a", 1)]);
    let empty_policy = td.path().join("empty_policy.toml");
    write_empty_policy(&empty_policy);

    let rs = assert_byte_identical(
        "h truncation footer",
        &root,
        &[
            "--domains",
            reg.to_str().unwrap(),
            "--bank",
            bank.to_str().unwrap(),
            "--policy",
            empty_policy.to_str().unwrap(),
        ],
    );
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out().contains(" more\n"),
        "the truncation footer must be reached: {}",
        rs.out()
    );
}

// ── (i) --write-json, compared as bytes ───────────────────────────────────

#[test]
fn the_written_summary_is_byte_identical() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();
    let target = td.path().join("out/coverage.json");
    let target_arg = target.to_str().unwrap().to_string();

    // Both sides write to the SAME path, so the `wrote <path>` line is directly
    // comparable; the python bytes are copied aside before the rust run.
    let py = python(&root, &["--write-json", &target_arg]);
    let py_json = std::fs::read(&target).expect("the oracle wrote no summary");
    assert!(!py_json.is_empty(), "an empty summary is a vacuous compare");
    std::fs::remove_file(&target).unwrap();

    let rs = rust(&root, &["--write-json", &target_arg]);
    let rs_json = std::fs::read(&target).expect("the port wrote no summary");

    assert_eq!(py.stdout, rs.stdout, "STDOUT differs:\n{}", py.out());
    assert_eq!(py.stderr, rs.stderr, "STDERR differs:\n{}", py.err());
    assert_eq!(py.code, rs.code, "EXIT CODE differs");
    assert_eq!(
        String::from_utf8_lossy(&py_json),
        String::from_utf8_lossy(&rs_json),
        "the written coverage summary differs"
    );
    COMPARED.fetch_add(1, Ordering::SeqCst);
    assert!(
        rs.out().contains("  wrote "),
        "the write must be announced: {}",
        rs.out()
    );

    // …and on a RED run, where the summary is still written and the `wrote`
    // line still precedes the failure list.
    let reg = td.path().join("domains.toml");
    write(&reg, &domains_registry(&[1, 2]));
    let bank = td.path().join("bank");
    plant_bank(&bank, &[("a", 1)]);
    let policy = td.path().join("policy.toml");
    write(
        &policy,
        "[[domain_min]]\nmodule = 1\nmin_items = 9\n\
         [[coverage_exempt]]\nmodule = 2\nreason = \"held out\"\n",
    );
    let red_target = td.path().join("out/red.json");
    let red_arg = red_target.to_str().unwrap().to_string();
    let args = [
        "--domains",
        reg.to_str().unwrap(),
        "--bank",
        bank.to_str().unwrap(),
        "--policy",
        policy.to_str().unwrap(),
        "--write-json",
        &red_arg,
    ];
    let py = python(&root, &args);
    let py_json = std::fs::read(&red_target).expect("the oracle wrote no summary on the red path");
    std::fs::remove_file(&red_target).unwrap();
    let rs = rust(&root, &args);
    let rs_json = std::fs::read(&red_target).expect("the port wrote no summary on the red path");

    assert_eq!(py.stdout, rs.stdout, "STDOUT differs:\n{}", py.out());
    assert_eq!(py.stderr, rs.stderr, "STDERR differs:\n{}", py.err());
    assert_eq!(py.code, rs.code, "EXIT CODE differs");
    assert_eq!(
        String::from_utf8_lossy(&py_json),
        String::from_utf8_lossy(&rs_json),
        "the written summary differs on the red path"
    );
    COMPARED.fetch_add(1, Ordering::SeqCst);
    assert_ne!(rs.code, 0, "{}", rs.out());
    let out = rs.out();
    assert!(
        out.find("  wrote ").unwrap() < out.find("  failures:").unwrap(),
        "the summary must be announced before the failure list:\n{out}"
    );
    // the summary is a MACHINE ledger: the shortfall it records must be there
    let text = String::from_utf8_lossy(&rs_json);
    assert!(text.contains("\"shortfalls\""), "{text}");
    assert!(text.contains("\"exemptions\""), "{text}");
    assert!(
        text.contains("\"status\": \"fail\""),
        "the ledger must record the verdict: {text}"
    );
}

// ── shapes the shell suite never reaches ──────────────────────────────────

#[test]
fn path_and_option_shapes_are_byte_identical() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();

    // engine-root-relative arguments, including untidy spellings the printed
    // header must normalise the same way on both sides
    assert_byte_identical("relative bank", &root, &["--bank", "bank/items"]);
    assert_byte_identical("untidy relative bank", &root, &["--bank", "./bank//items/"]);
    assert_byte_identical(
        "relative registry",
        &root,
        &["--domains", "knowledge/domains.toml"],
    );
    assert_byte_identical(
        "relative policy",
        &root,
        &["--policy", "knowledge/bank_policy.toml"],
    );

    // `--opt=value` and argparse's unambiguous prefixes
    assert_byte_identical("equals form", &root, &["--bank=bank/items"]);
    assert_byte_identical("abbreviated option", &root, &["--ban", "bank/items"]);
    assert_byte_identical("shortest prefix", &root, &["--d", "knowledge/domains.toml"]);

    // a missing policy IS an error (bd-j98g) — it must not lower sized floors
    // to N=1. The live bank used to PASS here because every module has >> 1 item.
    let absent = td.path().join("absent_policy.toml");
    let rs = assert_byte_identical(
        "absent policy is an error, not an N=1 fallback",
        &root,
        &["--policy", absent.to_str().unwrap()],
    );
    assert_ne!(rs.code, 0, "missing policy must be RED:\n{}", rs.out());
    assert!(
        rs.out().contains("policy=absent"),
        "{}",
        rs.out()
    );
    assert!(
        !rs.out().contains("policy=absent (N=1 OQ-05)"),
        "N=1 must not be claimed as a fallback:\n{}",
        rs.out()
    );
    assert!(
        rs.out().contains("bank_policy.toml missing:"),
        "{}",
        rs.out()
    );

    // all four options at once, absolute and relative mixed
    let bank = td.path().join("bank");
    plant_bank(&bank, &[("a", 1), ("b", 2)]);
    let reg = td.path().join("domains.toml");
    write(&reg, &domains_registry(&[1, 2]));
    let json = td.path().join("summary.json");
    assert_byte_identical(
        "all options at once",
        &root,
        &[
            "--bank",
            bank.to_str().unwrap(),
            "--domains",
            reg.to_str().unwrap(),
            "--policy",
            "knowledge/bank_policy.toml",
            "--write-json",
            json.to_str().unwrap(),
        ],
    );
}

#[test]
fn malformed_registry_rows_and_bank_files_are_byte_identical() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();
    let bank = td.path().join("bank");
    plant_bank(&bank, &[("a", 1)]);
    let bank_arg = bank.to_str().unwrap().to_string();
    let empty_policy = td.path().join("empty_policy.toml");
    write_empty_policy(&empty_policy);
    let empty_policy_arg = empty_policy.to_str().unwrap().to_string();

    // a [[domain]] row whose order cannot be coerced
    let bad_order = td.path().join("bad_order.toml");
    write(
        &bad_order,
        "[[domain]]\nid = \"01-alpha\"\norder = \"x\"\n\n[[domain]]\nid = \"02-beta\"\norder = 2\n",
    );
    let rs = assert_byte_identical(
        "domain row with no usable order",
        &root,
        &[
            "--domains",
            bad_order.to_str().unwrap(),
            "--bank",
            &bank_arg,
            "--policy",
            &empty_policy_arg,
        ],
    );
    assert!(
        rs.out()
            .contains("domains.toml: '01-alpha' has no usable order"),
        "the id is printed through repr(), quotes and all: {}",
        rs.out()
    );

    // two rows claiming one order
    let dup = td.path().join("dup.toml");
    write(
        &dup,
        "[[domain]]\nid = \"01-alpha\"\norder = 1\n\n[[domain]]\nid = \"01-again\"\norder = 1\n",
    );
    let rs = assert_byte_identical(
        "duplicate order",
        &root,
        &[
            "--domains",
            dup.to_str().unwrap(),
            "--bank",
            &bank_arg,
            "--policy",
            &empty_policy_arg,
        ],
    );
    assert!(
        rs.out()
            .contains("domains.toml: duplicate order 1 (01-alpha and 01-again)"),
        "{}",
        rs.out()
    );

    // a [[domain]] key that is a list of scalars rather than a table array
    let not_table = td.path().join("not_table.toml");
    write(&not_table, "domain = [\"justastring\"]\n");
    let rs = assert_byte_identical(
        "domain row is not a table",
        &root,
        &[
            "--domains",
            not_table.to_str().unwrap(),
            "--bank",
            &bank_arg,
            "--policy",
            &empty_policy_arg,
        ],
    );
    assert!(
        rs.out()
            .contains("domains.toml: [[domain]] row is not a table: 'justastring'"),
        "{}",
        rs.out()
    );

    // bank files: junk, an uncoercible module, and a missing module key
    let reg = td.path().join("two.toml");
    write(&reg, &domains_registry(&[1, 2]));
    let messy = td.path().join("messy_bank");
    plant_bank(&messy, &[("good", 1), ("also", 2)]);
    write(&messy.join("zz-junk.toml"), "label = \"nothing useful\"\n");
    write(
        &messy.join("zz-badmod.toml"),
        "id = \"zz-badmod\"\nmodule = \"nope\"\n",
    );
    write(&messy.join("zz-nomod.toml"), "id = \"zz-nomod\"\n");
    let rs = assert_byte_identical(
        "malformed bank files",
        &root,
        &[
            "--domains",
            reg.to_str().unwrap(),
            "--bank",
            messy.to_str().unwrap(),
            "--policy",
            &empty_policy_arg,
        ],
    );
    assert_ne!(rs.code, 0, "{}", rs.out());
    for needle in [
        "zz-junk.toml: no id or items[]",
        "zz-badmod: bad module 'nope'",
        "zz-nomod: bad module None",
    ] {
        assert!(
            rs.out().contains(needle),
            "missing {needle:?}: {}",
            rs.out()
        );
    }

    // ── anti-vacuous at FILE granularity (bd-0czh) ─────────────────────────
    // `items = []` takes the `isinstance(data["items"], list)` branch and adds
    // nothing, so it can never reach the `no id or items[]` leg — Python's
    // `elif` cannot run once the `if` has. Both sides must now name the file and
    // go RED. Note what the surrounding numbers do: `items=` stays at the count
    // the other files carry, and every module still clears its floor, so the
    // aggregate `empty bank` check can never fire on this. Only the named
    // failure line distinguishes a file that was never really checked from one
    // that passed, which is why the assertion is on the name and not on a count.
    let quiet = td.path().join("quiet_bank");
    plant_bank(&quiet, &[("good", 1), ("also", 2)]);
    write(&quiet.join("zz-silently-empty.toml"), "items = []\n");
    let rs = assert_byte_identical(
        "file yielding zero items",
        &root,
        &[
            "--domains",
            reg.to_str().unwrap(),
            "--bank",
            quiet.to_str().unwrap(),
            "--policy",
            &empty_policy_arg,
        ],
    );
    assert_ne!(
        rs.code,
        0,
        "a bank file that contributed nothing must never be a pass: {}",
        rs.out()
    );
    assert!(
        rs.out().contains("  items=2 scanned, 2 approved "),
        "the healthy aggregate is exactly what hides this defect; it must survive \
         the fix so the case keeps testing what it claims to: {}",
        rs.out()
    );
    assert!(
        rs.out().contains(
            "zz-silently-empty.toml: items[] yielded zero items (vacuous file scan is ERROR)"
        ),
        "the file that yielded nothing must be named: {}",
        rs.out()
    );

    // The known-GOOD leg the fix must not break: a legitimate single-item file
    // in `id = ...` form, with no `items` key at all, is still counted and still
    // passes. A fix that turned every zero-yield file RED by widening the
    // else-branch would take this file with it.
    let single = td.path().join("single_item_bank");
    std::fs::create_dir_all(&single).unwrap();
    write(
        &single.join("solo-one.toml"),
        "id = \"solo-one\"\nmodule = 1\nstatus = \"approved\"\n",
    );
    write(
        &single.join("solo-two.toml"),
        "id = \"solo-two\"\nmodule = 2\nstatus = \"approved\"\n",
    );
    let rs = assert_byte_identical(
        "single-item `id =` files, no items key, still pass",
        &root,
        &[
            "--domains",
            reg.to_str().unwrap(),
            "--bank",
            single.to_str().unwrap(),
            "--policy",
            &empty_policy_arg,
        ],
    );
    assert_eq!(
        rs.code,
        0,
        "the elif branch must survive the fix: {}",
        rs.out()
    );
    assert!(
        !rs.out().contains("yielded zero items"),
        "a file with no items key never took the list branch: {}",
        rs.out()
    );

    // the `items[]` table-array form, which IS the shape most of the live bank
    // would use, must still be counted
    let nested = td.path().join("nested_bank");
    std::fs::create_dir_all(&nested).unwrap();
    write(
        &nested.join("multi.toml"),
        "[[items]]\nid = \"n1\"\nmodule = 1\nstatus = \"approved\"\n\n\
         [[items]]\nid = \"n2\"\nmodule = 2\nstatus = \"approved\"\n",
    );
    let rs = assert_byte_identical(
        "items[] table array is counted",
        &root,
        &[
            "--domains",
            reg.to_str().unwrap(),
            "--bank",
            nested.to_str().unwrap(),
            "--policy",
            &empty_policy_arg,
        ],
    );
    assert_eq!(rs.code, 0, "{}", rs.out());
    assert!(rs.out().contains("items=2"), "{}", rs.out());
}

/// The oracle does not guard either of its `bank_policy.toml` loads, so a
/// malformed policy raises: CPython flushes what it printed (nothing — the
/// raise happens before the first `print`), writes a traceback, and exits 1.
/// The port reproduces stdout and the exit code exactly; the traceback text is
/// the single surface it does not reproduce, which is asserted here rather than
/// left implicit.
#[test]
fn a_malformed_policy_raises_identically_except_for_the_traceback() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();
    let bad = td.path().join("bad_policy.toml");
    write(&bad, "this is not toml =\n");

    let args = ["--policy", bad.to_str().unwrap()];
    let py = python(&root, &args);
    let rs = rust(&root, &args);

    assert_eq!(
        py.stdout,
        rs.stdout,
        "STDOUT must still match byte for byte on the raise path:\n--- python ---\n{}\n--- rust ---\n{}",
        py.out(),
        rs.out()
    );
    assert!(
        py.stdout.is_empty(),
        "the raise precedes the first print: {:?}",
        py.out()
    );
    assert_eq!(py.code, rs.code, "EXIT CODE differs on the raise path");
    assert_eq!(py.code, 1, "the oracle exits 1 on an uncaught exception");
    assert!(
        !py.stderr.is_empty() && !rs.stderr.is_empty(),
        "both sides must say something on stderr: python {:?} rust {:?}",
        py.err(),
        rs.err()
    );
    COMPARED.fetch_add(1, Ordering::SeqCst);
}

/// A full copy of the live bank, checked against the live registry, is GREEN —
/// the control that keeps every planted needle above provably attributable to
/// the thing planted rather than to the copy.
#[test]
fn a_specimen_copy_of_the_live_bank_is_clean() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();
    let bank = td.path().join("bank_items");
    let copied = specimen_bank(&root, &bank);
    let rs = assert_byte_identical(
        "specimen bank clean",
        &root,
        &["--bank", bank.to_str().unwrap()],
    );
    assert_eq!(
        rs.code,
        0,
        "specimen bank of {copied} files is not clean: {}",
        rs.out()
    );
}

// ══ WHICH POOL THE FLOOR MEASURES ═════════════════════════════════════════
//
// bd-coverage-counts-retired-items-49jh. Both implementations counted FILES
// against the per-module floors. C1 restricts assembly to `approved`, so the
// floor guaranteed the stock of a pool no learner is ever drawn from, and it
// failed in the OPEN direction because the file count is guaranteed to be >=
// the approved count.
//
// The known-bad below is the one the OLD code could not possibly trip: it
// retires approved items in a copy of the live bank WITHOUT deleting a single
// file, so the file count is unchanged and only the drawable pool moves. A
// file-counting gate stays GREEN on it by construction.

/// One parsed `    mNN: A approved of S scanned (min M) [flag]` line.
#[derive(Debug, Clone, Copy)]
struct ModuleLine {
    module: i64,
    approved: i64,
    scanned: i64,
    floor: i64,
}

/// Parse the per-module block. Keyed on the line SHAPE, so a report that
/// stopped naming both populations fails here rather than being reinterpreted.
fn module_lines(out: &str) -> Vec<ModuleLine> {
    let mut v = Vec::new();
    for line in out.lines() {
        let t = line.trim();
        let Some(rest) = t.strip_prefix('m') else {
            continue;
        };
        let Some((num, rest)) = rest.split_once(": ") else {
            continue;
        };
        let Ok(module) = num.parse::<i64>() else {
            continue;
        };
        let Some((approved, rest)) = rest.split_once(" approved of ") else {
            continue;
        };
        let Some((scanned, rest)) = rest.split_once(" scanned (min ") else {
            continue;
        };
        let Some((floor, _)) = rest.split_once(") [") else {
            continue;
        };
        let (Ok(approved), Ok(scanned), Ok(floor)) = (
            approved.parse::<i64>(),
            scanned.parse::<i64>(),
            floor.parse::<i64>(),
        ) else {
            continue;
        };
        v.push(ModuleLine {
            module,
            approved,
            scanned,
            floor,
        });
    }
    v
}

/// Retire `n` currently-approved items of `module` in a bank directory, IN
/// PLACE, by flipping their status line. No file is added or removed, which is
/// the whole point: only the drawable pool moves.
fn retire_in_place(bank: &Path, module: i64, n: usize) -> usize {
    let mut done = 0usize;
    let mut names: Vec<String> = std::fs::read_dir(bank)
        .expect("bank dir")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|s| s.ends_with(".toml"))
        .collect();
    names.sort();
    for name in names {
        if done == n {
            break;
        }
        let p = bank.join(&name);
        let text = std::fs::read_to_string(&p).unwrap();
        let is_module = text
            .lines()
            .any(|l| l.trim() == format!("module = {module}"));
        let is_approved = text.lines().any(|l| l.trim() == "status = \"approved\"");
        if !(is_module && is_approved) {
            continue;
        }
        let flipped = text.replace("status = \"approved\"", "status = \"retired\"");
        assert_ne!(flipped, text, "the status flip did not change {name}");
        std::fs::write(&p, flipped).unwrap();
        done += 1;
    }
    assert_eq!(
        done, n,
        "wanted to retire {n} approved items of module {module} and only found {done} — \
         an injection that could not be planted is a FAILURE, never a skip"
    );
    done
}

#[test]
fn retiring_a_module_under_its_floor_is_red_and_names_it() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();
    let bank = td.path().join("bank_items");
    specimen_bank(&root, &bank);

    // The control: the untouched specimen is GREEN, so every needle below is
    // attributable to the retirement rather than to the copy.
    let green = assert_byte_identical(
        "retire-under-floor control",
        &root,
        &["--bank", bank.to_str().unwrap()],
    );
    assert_eq!(green.code, 0, "control must be GREEN: {}", green.out());

    // Pick the module with the THINNEST margin, derived from the report rather
    // than written down here — a test that hardcodes today's counts is the same
    // defect one level up.
    let lines = module_lines(&green.out());
    assert!(
        lines.len() > 1,
        "parsed {} module lines from the control report — a vacuous parse is an ERROR:\n{}",
        lines.len(),
        green.out()
    );
    let target = lines
        .iter()
        .copied()
        .min_by_key(|m| m.approved - m.floor)
        .expect("a module to starve");
    assert!(
        target.approved >= target.floor,
        "the control was already breaching: {target:?}"
    );
    let to_retire = (target.approved - target.floor + 1) as usize;
    retire_in_place(&bank, target.module, to_retire);

    let rs = assert_byte_identical(
        "retire-under-floor injection",
        &root,
        &["--bank", bank.to_str().unwrap()],
    );
    assert_ne!(
        rs.code,
        0,
        "retiring module {} below its floor of {} must be RED:\n{}",
        target.module,
        target.floor,
        rs.out()
    );
    let want_approved = target.approved - to_retire as i64;
    assert!(
        rs.out().contains(&format!(
            "module {}: {want_approved} approved < min {}",
            target.module, target.floor
        )),
        "the starved module, its approved count and its floor must all be NAMED:\n{}",
        rs.out()
    );

    // THE PROOF THAT THE POPULATION CHANGED, not merely the verdict. Not one
    // file was added or removed, so the SCANNED count is untouched and still
    // clears the floor. A gate counting files sees this fixture as `[ok]`; the
    // fixed gate sees it as SHORT. That gap is the defect, made observable.
    let after = module_lines(&rs.out());
    let hit = after
        .iter()
        .find(|m| m.module == target.module)
        .unwrap_or_else(|| panic!("module {} vanished from the report", target.module));
    assert_eq!(
        hit.scanned, target.scanned,
        "the injection must not change the FILE count — otherwise it would also \
         have tripped the old file-counting gate and would prove nothing"
    );
    assert!(
        hit.scanned >= hit.floor,
        "the file count must still clear the floor ({} scanned vs min {}), which is \
         exactly what a file-counting gate would have reported [ok] on",
        hit.scanned,
        hit.floor
    );
    assert_eq!(hit.approved, want_approved, "approved count drifted");
    assert!(hit.approved < hit.floor, "the approved pool must be SHORT");
}

#[test]
fn a_bank_with_nothing_approved_is_an_error_not_a_pass() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();
    let reg = td.path().join("domains.toml");
    write(&reg, &domains_registry(&[1, 2]));
    let empty_policy = td.path().join("empty_policy.toml");
    write_empty_policy(&empty_policy);

    // Every declared module is STOCKED — with items no learner can be drawn.
    // The aggregate file count is healthy, which is precisely the state a
    // file-counting floor reported green on.
    let bank = td.path().join("all_retired");
    let planted = plant_bank_with_status(
        &bank,
        &[
            ("r1", 1, Some("retired")),
            ("r2", 1, Some("retired")),
            ("r3", 2, Some("retired")),
            ("d1", 2, None),
        ],
    );
    assert_eq!(planted, 4, "a fixture that planted nothing is an ERROR");

    let rs = assert_byte_identical(
        "bank with nothing approved",
        &root,
        &[
            "--domains",
            reg.to_str().unwrap(),
            "--bank",
            bank.to_str().unwrap(),
            "--policy",
            empty_policy.to_str().unwrap(),
        ],
    );
    assert_ne!(rs.code, 0, "a bank with no drawable item must never pass");
    assert!(
        rs.out().contains("zero approved items (4 scanned)"),
        "the vacuous approved pool must be named, with both numbers: {}",
        rs.out()
    );
    // A MISSING status is `draft` by C1's default — silence never publishes —
    // and is not itself an error.
    assert!(
        !rs.out().contains("unknown status"),
        "an absent status is the draft default, not an unknown one: {}",
        rs.out()
    );
    // …and every module is individually named as starved, not merely the total.
    assert!(
        rs.out().contains("module 1: 0 approved < min 1"),
        "{}",
        rs.out()
    );
    assert!(
        rs.out().contains("module 2: 0 approved < min 1"),
        "{}",
        rs.out()
    );
}

#[test]
fn an_unrecognised_status_is_named_rather_than_bucketed_by_guess() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();
    let reg = td.path().join("domains.toml");
    write(&reg, &domains_registry(&[1]));
    let empty_policy = td.path().join("empty_policy.toml");
    write_empty_policy(&empty_policy);

    let bank = td.path().join("odd_status");
    plant_bank_with_status(
        &bank,
        &[("good", 1, Some("approved")), ("odd", 1, Some("published"))],
    );

    let rs = assert_byte_identical(
        "unrecognised status",
        &root,
        &[
            "--domains",
            reg.to_str().unwrap(),
            "--bank",
            bank.to_str().unwrap(),
            "--policy",
            empty_policy.to_str().unwrap(),
        ],
    );
    assert_ne!(
        rs.code,
        0,
        "a status nobody modelled must not be silently bucketed: {}",
        rs.out()
    );
    assert!(
        rs.out().contains("odd: unknown status 'published'"),
        "the ITEM and the value must both be named: {}",
        rs.out()
    );
    // Fail-CLOSED: the unmodelled item did not count toward the floor either.
    assert!(
        rs.out().contains("m01: 1 approved of 2 scanned"),
        "an unmodelled status must not count as approved: {}",
        rs.out()
    );
}

// ══ THE VERDICT MAY NOT PRECEDE THE SIDE EFFECT ═══════════════════════════
//
// bd-verify-coverage-verdict-before-write-rk9n. Both implementations emitted
// the verdict and then ran an unguarded `--write-json` write. The Python's
// OSError propagated out of `main()` under a stdout that already said PASS;
// the Rust's `Halt` PRESERVED the buffered PASS into `Outcome.stdout` and
// exited 1. The two agreed, byte for byte, on a lie — which is why the
// detector in `assert_byte_identical` is asserted PER SIDE.

/// The four-tuple for a run that writes: stdout, stderr, exit code, and the
/// bytes at the target — plus whether the atomic write's temp file survived.
struct WriteRun {
    run: Run,
    artifact: Option<Vec<u8>>,
    temp_residue: bool,
}

fn write_run(f: fn(&Path, &[&str]) -> Run, root: &Path, args: &[&str], target: &Path) -> WriteRun {
    let run = f(root, args);
    let mut tmp = target.as_os_str().to_os_string();
    tmp.push(".tmp");
    WriteRun {
        run,
        artifact: std::fs::read(target).ok(),
        temp_residue: Path::new(&tmp).exists(),
    }
}

/// Both sides, on a `--write-json` target that CANNOT be written, and the
/// assertions that make a failed write unable to hide under a verdict.
fn assert_failed_write_is_silent_and_leaves_nothing(label: &str, root: &Path, target: &Path) {
    let target_arg = target.to_str().unwrap().to_string();
    let args = ["--write-json", target_arg.as_str()];
    let py = write_run(python, root, &args, target);
    let rs = write_run(rust, root, &args, target);

    for (side, w) in [("python", &py), ("rust", &rs)] {
        assert_ne!(
            w.run.code, 0,
            "[{label}] {side} reported success for a write that did not happen"
        );
        for token in SUCCESS_TOKENS {
            assert!(
                !w.run.out().contains(token),
                "[{label}] {side} exited {} with {token:?} on stdout above a FAILED WRITE:\n{}",
                w.run.code,
                w.run.out()
            );
        }
        assert!(
            w.artifact.is_none(),
            "[{label}] {side} left an artifact behind after a failed write; a later reader \
             cannot tell a passing ledger from the residue of a failed run"
        );
        assert!(
            !w.temp_residue,
            "[{label}] {side} left its atomic-write temp file behind"
        );
        assert!(
            !w.run.err().is_empty(),
            "[{label}] {side} failed the write and said nothing on stderr"
        );
    }

    // stdout and the exit code stay byte-identical across the two; the
    // traceback text is the one surface this port does not reproduce.
    assert_eq!(
        py.run.stdout,
        rs.run.stdout,
        "[{label}] STDOUT differs on the failed-write path:\n--- python ---\n{}\n--- rust ---\n{}",
        py.run.out(),
        rs.run.out()
    );
    assert!(
        py.run.stdout.is_empty(),
        "[{label}] the report must not have started: {:?}",
        py.run.out()
    );
    assert_eq!(
        py.run.code, rs.run.code,
        "[{label}] EXIT CODE differs on the failed-write path"
    );
    COMPARED.fetch_add(1, Ordering::SeqCst);
}

#[test]
fn a_write_json_whose_parent_is_a_file_fails_silently_in_both() {
    let td = tempfile::tempdir().unwrap();
    // A TREE COPY, never the live tree: this case makes a write FAIL, and a
    // gate whose write is being made to fail is the last thing that should be
    // pointed at the repo.
    let root = td.path().join("tree");
    live_tree_copy(&engine_root(), &root);
    let root = root.as_path();

    // The parent is a regular FILE, so `mkdir(parents=True, exist_ok=True)` and
    // `create_dir_all` both refuse. Chosen over a chmod because it is
    // deterministic and cannot be defeated by running the suite as root — an
    // injection that silently stops injecting is the failure this file exists
    // to prevent.
    let blocker = td.path().join("blocker");
    write(&blocker, "not a directory\n");
    let target = blocker.join("coverage.json");
    assert_failed_write_is_silent_and_leaves_nothing("parent is a file", root, &target);
}

#[test]
fn a_write_json_onto_a_directory_fails_after_the_temp_write_in_both() {
    let td = tempfile::tempdir().unwrap();
    let root = td.path().join("tree");
    live_tree_copy(&engine_root(), &root);
    let root = root.as_path();

    // Here `mkdir` SUCCEEDS and the temp file is written; the atomic rename is
    // what fails. This is the leg that proves the cleanup: the temp file must
    // not survive, or a failed write would leave `coverage.json.tmp` sitting
    // next to a stale ledger.
    let target = td.path().join("out/coverage.json");
    std::fs::create_dir_all(&target).unwrap();
    assert_failed_write_is_silent_and_leaves_nothing("target is a directory", root, &target);
}

// ══ THE TRACKED LEDGER, WITHOUT TOUCHING THE LIVE TREE ════════════════════

const TRACKED_ARTIFACT: &str = "web/data/coverage.json";

/// Materialise every input this gate reads into TEMP, plus the oracle itself.
///
/// The oracle resolves its own root from `Path(__file__).resolve().parents[1]`,
/// so it MUST be copied in — a script left outside the fixture would read the
/// live tree instead. No `--path` flag is added to make it testable: widening a
/// gate's argument surface changes the thing under test.
///
/// Both implementations share ONE copy here, which is the deliberate difference
/// from the builder harnesses. This gate PRINTS its own root (`bank=`, `wrote`),
/// so two roots would make stdout differ by construction and the byte
/// comparison would have to be weakened to survive the fixture. Weakening the
/// comparison to accommodate the fixture is worse than sharing a scratch root
/// neither run reads the other's output from: the runs are sequential and the
/// artifact is snapshotted and removed between them.
fn live_tree_copy(root: &Path, into: &Path) {
    for rel in [
        "scripts/verify_coverage.py",
        "knowledge/domains.toml",
        "knowledge/bank_policy.toml",
    ] {
        let from = root.join(rel);
        assert!(from.is_file(), "the live tree is missing {rel}");
        let to = into.join(rel);
        std::fs::create_dir_all(to.parent().unwrap()).unwrap();
        std::fs::copy(&from, &to).unwrap();
    }
    let n = specimen_bank(root, &into.join("bank/items"));
    assert!(
        n > 0,
        "a fixture that copied no bank is an ERROR, not a pass"
    );
}

#[test]
fn the_written_ledger_is_byte_identical_and_reproduces_the_tracked_artifact() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();
    let copy = td.path().join("tree");
    live_tree_copy(&root, &copy);

    let target = copy.join(TRACKED_ARTIFACT);
    let args = ["--write-json", TRACKED_ARTIFACT];

    let py = python(&copy, &args);
    let py_json = std::fs::read(&target).expect("the oracle wrote no ledger");
    assert!(!py_json.is_empty(), "an empty ledger is a vacuous compare");
    std::fs::remove_file(&target).unwrap();

    let rs = rust(&copy, &args);
    let rs_json = std::fs::read(&target).expect("the port wrote no ledger");

    assert_eq!(py.stdout, rs.stdout, "STDOUT differs:\n{}", py.out());
    assert_eq!(py.stderr, rs.stderr, "STDERR differs:\n{}", py.err());
    assert_eq!(py.code, rs.code, "EXIT CODE differs");
    assert_eq!(py.code, 0, "the live inputs must be GREEN:\n{}", py.out());
    // RAW BYTES, never parsed JSON: key order, indent, \uXXXX escaping and the
    // trailing newline are exactly where two writers agree on every value and
    // disagree on every byte.
    assert_eq!(py_json, rs_json, "the written ledger differs");
    COMPARED.fetch_add(1, Ordering::SeqCst);

    // THE TIE-BACK THAT BUYS THE LIVE-TREE CLAIM. The bytes both sides produce
    // are identical to the TRACKED artifact, so running either implementation
    // in the live tree would be a no-op write — established read-only, without
    // performing one.
    let tracked = std::fs::read(root.join(TRACKED_ARTIFACT))
        .unwrap_or_else(|e| panic!("the tracked ledger {TRACKED_ARTIFACT} is unreadable: {e}"));
    assert_eq!(
        String::from_utf8_lossy(&rs_json),
        String::from_utf8_lossy(&tracked),
        "the tracked {TRACKED_ARTIFACT} is not what this gate produces from the live \
         inputs. A machine ledger that has drifted from its generator is a ledger \
         nobody can trust; regenerate it in a tree copy and commit the bytes."
    );

    // And the ledger carries BOTH populations, so a later reader cannot mistake
    // one for the other — the confusion this bead exists to end.
    let text = String::from_utf8_lossy(&rs_json);
    for key in [
        "\"schema_version\": 3",
        "\"approved_count\"",
        "\"scanned_counts\"",
        "\"item_count\"",
        "\"counts\"",
    ] {
        assert!(text.contains(key), "the ledger is missing {key}: {text}");
    }
}

// ── the harness must not be vacuously green ───────────────────────────────

#[test]
fn the_harness_compared_something() {
    // Runs a case itself rather than reading a counter another test may or may
    // not have incremented — test order and parallelism are not a contract, and
    // "0 cases compared" must never report like "all passed".
    let root = engine_root();
    let before = COMPARED.load(Ordering::SeqCst);
    assert_byte_identical("harness self-check", &root, &[]);
    assert!(
        COMPARED.load(Ordering::SeqCst) > before,
        "the differential harness compared nothing"
    );
}

// ── no fourth instance: the class scan (bd-0czh) ───────────────────────────
//
// bd-2kr fixed ONE `items = []` fail-open. Three more were sitting in the tree,
// byte-identical, for a week. The reason they survived an audit is recorded in
// the bead and is the reason this scan exists at all: every one of those files
// ALREADY had an anti-vacuous check at whole-bank granularity, so grepping any
// of them for "zero items" returned a hit and read as guarded. The controller
// made exactly that call on verify_coverage.py — GUARDED off a grep — then read
// the branch and found the hole, with the matching message sitting eighty lines
// away in a different scope. A healthy total is what hides a file that was never
// checked, and a message-keyed test would be fooled by the same string that
// fooled the grep.
//
// So this keys on the BRANCH. It finds every Python branch that iterates an
// `items` collection into an accumulator, and requires each one to carry a
// zero-yield guard in its own body. A NEW loader written tomorrow in the same
// shape fails here before it can reach the bank.
//
// It lives in this file because this is where the deliberately-quiet pin lived.

/// A branch that iterates an `items` collection, and whether its own body
/// carries the zero-yield guard.
#[derive(Debug)]
struct ItemsBranch {
    file: String,
    line: usize,
    guarded: bool,
}

/// Leading indent, or `None` if the line is indented with tabs. A scanner that
/// guessed at mixed indentation would be a scanner that can be fooled, so a tab
/// is an unsupported shape and fails the scan rather than being skipped.
fn leading_indent(line: &str) -> Option<usize> {
    let ws: String = line.chars().take_while(|c| c.is_whitespace()).collect();
    if ws.contains('\t') {
        return None;
    }
    Some(ws.len())
}

/// The lines of the suite a header at `start` (0-based) opens: everything more
/// deeply indented, blank lines included, up to the first dedent.
fn block_of(lines: &[&str], start: usize, header_indent: usize) -> Vec<(usize, String)> {
    let mut body = Vec::new();
    for (i, raw) in lines.iter().enumerate().skip(start + 1) {
        if raw.trim().is_empty() {
            continue;
        }
        match leading_indent(raw) {
            None => break,
            Some(ind) if ind <= header_indent => break,
            Some(_) => body.push((i, (*raw).to_string())),
        }
    }
    body
}

/// `NAME = len(COLL)` -> `(NAME, COLL)`.
fn parse_snapshot(t: &str) -> Option<(String, String)> {
    let (lhs, rhs) = t.split_once('=')?;
    let (lhs, rhs) = (lhs.trim(), rhs.trim());
    if lhs.is_empty() || !lhs.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return None;
    }
    let inner = rhs.strip_prefix("len(")?.strip_suffix(')')?;
    Some((lhs.to_string(), inner.trim().to_string()))
}

/// `if len(COLL) == NAME:` -> `(COLL, NAME)`.
fn parse_zero_yield_guard(t: &str) -> Option<(String, String)> {
    let cond = t.strip_prefix("if ")?.strip_suffix(':')?;
    let (lhs, rhs) = cond.split_once("==")?;
    let inner = lhs.trim().strip_prefix("len(")?.strip_suffix(')')?;
    Some((inner.trim().to_string(), rhs.trim().to_string()))
}

/// Every `items`-iterating branch in one Python source, with its guard verdict.
///
/// The shape it keys on, and the only shape it accepts as guarded:
///
///   if <cond mentioning items>:      <- branch header
///       before = len(loaded)         <- snapshot of the accumulator
///       for it in data["items"]:     <- the iteration
///           loaded.append(...)
///       if len(loaded) == before:    <- the zero-yield guard, same accumulator
///           errors.append(...)       <- and it must RECORD, not just branch
///
/// A loader that guards differently still fails here. That is deliberate: the
/// four bank loaders read identically on purpose, and "guarded, but in a shape
/// this scan cannot verify" must not be indistinguishable from "guarded".
fn items_branches(name: &str, src: &str) -> Vec<ItemsBranch> {
    let lines: Vec<&str> = src.lines().collect();
    let mut found = Vec::new();

    for (i, raw) in lines.iter().enumerate() {
        let t = raw.trim_start();
        let is_branch_header = t.starts_with("if ") || t.starts_with("elif ");
        if !is_branch_header || !t.contains("items") {
            continue;
        }
        assert!(
            t.ends_with(':'),
            "{name}:{}: a multi-line `items` condition is a shape this scan \
             cannot follow. Fix the scanner, never the loader — an unreadable \
             branch must not read as a guarded one.",
            i + 1
        );
        let Some(header_indent) = leading_indent(raw) else {
            panic!("{name}:{}: tab-indented branch, unsupported shape", i + 1);
        };
        let body = block_of(&lines, i, header_indent);

        // Is this an ITEMS-ITERATING branch — the defect shape — or just some
        // other conditional that happens to say "items"?
        let iterates = body.iter().any(|(_, l)| {
            let bt = l.trim_start();
            match (bt.starts_with("for "), bt.find(" in ")) {
                (true, Some(at)) => bt[at + 4..].contains("items"),
                _ => false,
            }
        });
        if !iterates {
            continue;
        }

        // The guard, in this branch's own body, on the accumulator the loop fed.
        let mut snapshots: Vec<(String, String)> = Vec::new();
        let mut guarded = false;
        for (j, l) in &body {
            let bt = l.trim_start();
            if let Some(pair) = parse_snapshot(bt) {
                snapshots.push(pair);
                continue;
            }
            if let Some((coll, cmp)) = parse_zero_yield_guard(bt) {
                let matches_snapshot = snapshots.iter().any(|(nm, cl)| *nm == cmp && *cl == coll);
                if !matches_snapshot {
                    continue;
                }
                // A guard that branches but records nothing is a guard that
                // still lets the file report like one that passed.
                let g_indent = leading_indent(l).expect("guard indent");
                let records = block_of(&lines, *j, g_indent)
                    .iter()
                    .any(|(_, gl)| gl.contains("errors.append"));
                if records {
                    guarded = true;
                }
            }
        }

        found.push(ItemsBranch {
            file: name.to_string(),
            line: i + 1,
            guarded,
        });
    }
    found
}

#[test]
fn no_bank_loader_iterates_items_without_a_zero_yield_guard() {
    let scripts = engine_root().join("scripts");
    let mut sources: Vec<(String, String)> = std::fs::read_dir(&scripts)
        .expect("scripts/ must be readable")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("py"))
        .map(|p| {
            let name = p.file_name().unwrap().to_string_lossy().into_owned();
            let body = std::fs::read_to_string(&p)
                .unwrap_or_else(|e| panic!("unreadable {}: {e}", p.display()));
            (name, body)
        })
        .collect();
    sources.sort();

    // Anti-vacuous, twice over. A scan that read no files, or found no branches
    // to check, reports exactly like one that checked everything and found it
    // sound — which is the defect this whole bead is about, one level up.
    assert!(
        sources.len() >= 8,
        "scanned only {} python sources under {}; a scan that read almost \
         nothing must not report like one that read everything",
        sources.len(),
        scripts.display()
    );

    let branches: Vec<ItemsBranch> = sources
        .iter()
        .flat_map(|(n, s)| items_branches(n, s))
        .collect();

    for expected in [
        "verify_bank.py",
        "verify_coverage.py",
        "verify_objectives.py",
        "verify_orphans.py",
    ] {
        assert!(
            branches.iter().any(|b| b.file == expected),
            "the scan found no items-iterating branch in {expected}. Either the \
             loader was rewritten into a shape this scan cannot see — fix the \
             scan — or the file moved. Silence here is not evidence."
        );
    }
    assert!(
        branches.len() >= 4,
        "expected at least the four bank loaders, found {}",
        branches.len()
    );

    let unguarded: Vec<String> = branches
        .iter()
        .filter(|b| !b.guarded)
        .map(|b| format!("scripts/{}:{}", b.file, b.line))
        .collect();
    assert!(
        unguarded.is_empty(),
        "these branches iterate an `items` collection with no zero-yield guard \
         in the branch body, so a bank file yielding zero items is scanned, \
         contributes nothing, and is never named (bd-2kr, bd-0czh):\n  {}\n\
         Add, inside the branch:\n    before = len(loaded)\n    ...\n    \
         if len(loaded) == before:\n        errors.append(\n            \
         f\"{{path.name}}: items[] yielded zero items \"\n            \
         \"(vacuous file scan is ERROR)\"\n        )",
        unguarded.join("\n  ")
    );
}

/// L4: the scan above is proven to trip, on fixtures, without touching the tree.
/// A detector that has never returned "unguarded" is indistinguishable from one
/// that cannot.
#[test]
fn the_zero_yield_scan_fires_on_the_known_bad_and_stays_quiet_on_the_known_good() {
    let known_bad = r#"
def load_items(bank_dir):
    for path in sorted(bank_dir.glob("*.toml")):
        data = load_toml(path)
        if "items" in data and isinstance(data["items"], list):
            for it in data["items"]:
                loaded.append((path.name, it))
        elif "id" in data:
            loaded.append((path.name, data))
        else:
            errors.append(f"{path.name}: no id or items[]")
"#;
    let bad = items_branches("known_bad.py", known_bad);
    assert_eq!(bad.len(), 1, "the branch must be seen at all: {bad:?}");
    assert!(!bad[0].guarded, "the known-bad must read as UNGUARDED");

    let known_good = r#"
def load_items(bank_dir):
    for path in sorted(bank_dir.glob("*.toml")):
        data = load_toml(path)
        if "items" in data and isinstance(data["items"], list):
            before = len(loaded)
            for it in data["items"]:
                loaded.append((path.name, it))
            if len(loaded) == before:
                errors.append(f"{path.name}: items[] yielded zero items")
        elif "id" in data:
            loaded.append((path.name, data))
        else:
            errors.append(f"{path.name}: no id or items[]")
"#;
    let good = items_branches("known_good.py", known_good);
    assert_eq!(good.len(), 1, "the branch must be seen at all: {good:?}");
    assert!(good[0].guarded, "the known-good must read as GUARDED");

    // A guard that branches on the right thing but records nothing still lets
    // the file report like one that passed, so it must NOT read as guarded.
    let silent_guard = known_good.replace(
        "errors.append(f\"{path.name}: items[] yielded zero items\")",
        "pass",
    );
    let silent = items_branches("silent_guard.py", &silent_guard);
    assert_eq!(silent.len(), 1);
    assert!(
        !silent[0].guarded,
        "a guard that records nothing must not count as a guard"
    );

    // And the elif leg must not be mistaken for an items-iterating branch: it
    // mentions no items collection and iterates nothing.
    let single_item_only = r#"
def load_items(bank_dir):
    for path in sorted(bank_dir.glob("*.toml")):
        data = load_toml(path)
        if "id" in data:
            loaded.append((path.name, data))
"#;
    assert!(
        items_branches("single.py", single_item_only).is_empty(),
        "a plain `id =` loader has no items branch to guard"
    );
}
