//! Differential harness: `cdcp_gate verify-objectives` against
//! `scripts/verify_objectives.py`. Byte-identical stdout/stderr/exit on every
//! case. Floors measure the approved pool (bd-f996). Missing python3 is a
//! FAILURE. Live-tree module counts are derived from `domains.toml` at run
//! time.
//!
//! Agreement is necessary and not sufficient (bd-differential-shared-blindspot-4qje).
//! Floor and anti-vacuous cases also pin the VERDICT — the named finding, the
//! mode word, and that `covered=` is not a number no comparison produced. A
//! defect present in both implementations used to pass here (`m min topic 0`
//! printed `covered=106 mode=strict` EXIT 0). Parse-error prose and CPython
//! traceback text are the documented deviations.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const BIN: &str = env!("CARGO_BIN_EXE_cdcp_gate");
const ORACLE: &str = "scripts/verify_objectives.py";
const GATE: &str = "verify-objectives";

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

/// The whole acceptance bar in one function. Returns the (identical) run so a
/// case can additionally assert *what* the shared output says.
fn assert_byte_identical(label: &str, root: &Path, args: &[&str]) -> Run {
    let py = python(root, args);
    let rs = rust(root, args);

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

/// `write_domains` from the shell suite: a registry declaring exactly `orders`,
/// in the order given — which is deliberately NOT always ascending, so the
/// report's own ordering is under test rather than inherited from the fixture.
fn domains_registry(orders: &[i64]) -> String {
    let mut s = String::from("schema_version = 1\n");
    for o in orders {
        s.push_str(&format!(
            "\n[[domain]]\nid = \"{o:02}-fixture\"\norder = {o}\nepi_heading = \"Fixture domain {o}\"\n"
        ));
    }
    s
}

/// `write_topics` from the shell suite: one topic per domain id, in order.
fn topics_registry(domains: &[&str]) -> String {
    let mut s = String::from("schema_version = 1\n");
    for (i, d) in domains.iter().enumerate() {
        let n = i + 1;
        s.push_str(&format!(
            "\n[[topic]]\nid = \"t-fixture-{n}\"\ndomain = \"{d}\"\nlabel = \"fixture topic {n}\"\n"
        ));
    }
    s
}

/// `write_bank` from the shell suite: one item per module.
fn plant_bank(dir: &Path, modules: &[i64]) -> usize {
    std::fs::create_dir_all(dir).unwrap();
    for m in modules {
        write(
            &dir.join(format!("m{m}.toml")),
            &format!("id = \"sel-m{m:02}\"\nmodule = {m}\nstatus = \"approved\"\ntopic_ids = [\"t-fixture-1\"]\n"),
        );
    }
    assert!(
        !modules.is_empty(),
        "a vacuous fixture bank is an ERROR, not a pass"
    );
    modules.len()
}

/// `run_objectives` from the shell suite: the LIVE objectives and claims
/// registries (these cases are about the MODULE SET, and a fixture objectives
/// registry would only add a second thing that could be wrong), a fixture
/// domains/topics/bank/policy, and the topic floor off.
fn suite_args<'a>(
    domains: &'a str,
    topics: &'a str,
    bank: &'a str,
    policy: &'a str,
) -> Vec<&'a str> {
    vec![
        "--objectives",
        "registries/objectives.toml",
        "--claims",
        "registries/claims.toml",
        "--domains",
        domains,
        "--topics",
        topics,
        "--bank",
        bank,
        "--policy",
        policy,
        "--skip-topic-coverage",
    ]
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

// ── (d) the GREEN case, and the rebase's own signals ───────────────────────

#[test]
fn live_tree_is_byte_identical_and_green() {
    let root = engine_root();
    let declared = declared_module_count(&root);

    let rs = assert_byte_identical("d live tree", &root, &[]);
    assert_eq!(rs.code, 0, "live tree must be GREEN: {}", rs.out());
    assert!(rs.out().starts_with("PASS\n"), "{}", rs.out());
    assert!(
        rs.out().contains("objective coverage GREEN"),
        "{}",
        rs.out()
    );
    assert!(
        rs.err().is_empty(),
        "the oracle writes nothing to stderr on the green path: {:?}",
        rs.err()
    );

    // The module set is DERIVED, and the report says so with the registry's own
    // count. Both numbers come from domains.toml, never from a literal here.
    assert!(
        rs.out().contains(&format!("declares={declared}")),
        "the report must name the registry it derived from: {}",
        rs.out()
    );
    assert!(
        rs.out().contains(&format!(
            "modules ({declared} required, derived from domains.toml; min 1 approved item each)"
        )),
        "every declared module must be required on the live tree: {}",
        rs.out()
    );
    // …and the last-declared module is inside the required set, which is the
    // property the rebase existed to restore. Derived, not named.
    assert!(
        rs.out().contains(&format!("    m{declared:02}: ")),
        "the highest declared module is missing from the report: {}",
        rs.out()
    );
    assert!(
        rs.out()
            .contains(" approved (floors count the approved pool only)"),
        "every count must name its population: {}",
        rs.out()
    );

    // The exact spelling selftest_l7_objectives.sh case (d) uses.
    let rs = assert_byte_identical(
        "d live tree, explicit registries",
        &root,
        &[
            "--objectives",
            "registries/objectives.toml",
            "--claims",
            "registries/claims.toml",
            "--bank",
            "bank/items",
        ],
    );
    assert_eq!(rs.code, 0, "{}", rs.out());
}

// ── (a)(b)(b2)(c) the objectives/claims/bank legs of the shell suite ──────

#[test]
fn the_shell_selftest_registry_cases_are_byte_identical() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();

    // (a) an objectives registry with no [[objective]] rows
    let empty_obj = td.path().join("objectives_empty.toml");
    write(
        &empty_obj,
        "schema_version = 1\n\n[registry]\nname = \"objectives\"\n\
         description = \"selftest empty — must RED\"\n",
    );
    let rs = assert_byte_identical(
        "a empty objectives",
        &root,
        &[
            "--objectives",
            empty_obj.to_str().unwrap(),
            "--claims",
            "registries/claims.toml",
            "--bank",
            "bank/items",
            "--skip-topic-coverage",
        ],
    );
    assert_ne!(
        rs.code,
        0,
        "an empty registry must never pass: {}",
        rs.out()
    );
    assert!(
        rs.out().contains("zero [[objective]]"),
        "the suite's needle must survive the port: {}",
        rs.out()
    );

    // (b) an objective citing a claim_id that claims.toml does not define
    let bad_obj = td.path().join("objectives_bad_claim.toml");
    write(
        &bad_obj,
        "schema_version = 1\n\n[registry]\nname = \"objectives\"\n\n\
         [[objective]]\nid = \"obj-selftest-unresolved\"\n\
         text = \"planted objective with missing claim ref\"\n\
         claim_ids = [\"claim-does-not-exist-selftest-only\"]\n",
    );
    let rs = assert_byte_identical(
        "b unresolved claim_id",
        &root,
        &[
            "--objectives",
            bad_obj.to_str().unwrap(),
            "--claims",
            "registries/claims.toml",
            "--bank",
            "bank/items",
            "--skip-topic-coverage",
        ],
    );
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out().contains("unresolved claim_id"),
        "the suite's needle must survive the port: {}",
        rs.out()
    );

    // (b2) an objective citing nothing at all
    let no_claims = td.path().join("objectives_empty_claims.toml");
    write(
        &no_claims,
        "schema_version = 1\n\n[[objective]]\nid = \"obj-selftest-no-claims\"\n\
         text = \"planted objective with empty claim_ids\"\nclaim_ids = []\n",
    );
    let rs = assert_byte_identical(
        "b2 empty claim_ids",
        &root,
        &[
            "--objectives",
            no_claims.to_str().unwrap(),
            "--claims",
            "registries/claims.toml",
            "--bank",
            "bank/items",
            "--skip-topic-coverage",
        ],
    );
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(rs.out().contains("claim_ids empty"), "{}", rs.out());

    // (c) an empty bank directory
    let empty_bank = td.path().join("empty_bank");
    std::fs::create_dir_all(&empty_bank).unwrap();
    let rs = assert_byte_identical(
        "c empty bank",
        &root,
        &[
            "--objectives",
            "registries/objectives.toml",
            "--claims",
            "registries/claims.toml",
            "--bank",
            empty_bank.to_str().unwrap(),
            "--skip-topic-coverage",
        ],
    );
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(rs.out().contains("empty bank"), "{}", rs.out());

    // nothing planted may leak into the live tree
    assert!(
        !root.join("registries/objectives_empty.toml").exists(),
        "specimen leaked into the live registries"
    );
}

// ── (e) a declared module with zero items ─────────────────────────────────

#[test]
fn a_declared_module_with_zero_items_is_red_and_named() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();

    // THE bd-lt7 regression, in the shape the shell suite plants it: a registry
    // declaring two modules, a bank stocking only the first.
    let reg = td.path().join("domains.toml");
    write(&reg, &domains_registry(&[1, 15]));
    let topics = td.path().join("topics_ok.toml");
    write(&topics, &topics_registry(&["01-fixture"]));
    let bank = td.path().join("bank_m1");
    let planted = plant_bank(&bank, &[1]);
    assert!(planted > 0);
    // An "empty" policy is a real file with no rows, so the exemption ledger is
    // READ and found to hold nothing — not absent, which is a different path.
    let policy = td.path().join("policy_empty.toml");
    write(&policy, "# fixture policy: no rows\n");

    let args = suite_args(
        reg.to_str().unwrap(),
        topics.to_str().unwrap(),
        bank.to_str().unwrap(),
        policy.to_str().unwrap(),
    );
    let rs = assert_byte_identical("e declared module starved", &root, &args);
    assert_ne!(
        rs.code,
        0,
        "a starved declared module passed:\n{}",
        rs.out()
    );
    assert!(
        rs.out().contains("domain module 15: 0 approved < min 1"),
        "the finding must NAME the module: {}",
        rs.out()
    );
    assert!(
        rs.out().contains("    m15: 0 approved of 0 scanned (min 1) [SHORT]"),
        "the per-module line must flag it too: {}",
        rs.out()
    );
    // The known-GOOD leg: the stocked module is not dragged red with it.
    assert!(
        rs.out()
            .contains("    m01: 1 approved of 1 scanned (min 1) [ok]"),
        "{}",
        rs.out()
    );
    assert!(
        !rs.out().contains("PASS"),
        "no PASS may appear anywhere on a failing run: {}",
        rs.out()
    );
}

// ── (f)(i) an exemption without a reason, and the control that it works ───

#[test]
fn a_reasonless_exemption_is_an_error_and_leaves_the_module_required() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();

    let reg = td.path().join("domains.toml");
    write(&reg, &domains_registry(&[1, 15]));
    let topics = td.path().join("topics_ok.toml");
    write(&topics, &topics_registry(&["01-fixture"]));
    let bank = td.path().join("bank_m1");
    plant_bank(&bank, &[1]);

    for (label, body) in [
        ("absent reason", "[[coverage_exempt]]\nmodule = 15\n"),
        (
            "blank reason",
            "[[coverage_exempt]]\nmodule = 15\nreason = \"\"\n",
        ),
        (
            "whitespace reason",
            "[[coverage_exempt]]\nmodule = 15\nreason = \"   \"\n",
        ),
    ] {
        let policy = td
            .path()
            .join(format!("policy_{}.toml", label.replace(' ', "_")));
        write(&policy, body);
        let args = suite_args(
            reg.to_str().unwrap(),
            topics.to_str().unwrap(),
            bank.to_str().unwrap(),
            policy.to_str().unwrap(),
        );
        let rs = assert_byte_identical(&format!("f exemption, {label}"), &root, &args);
        assert_ne!(rs.code, 0, "[{label}] {}", rs.out());
        assert!(
            rs.out().contains(
                "bank_policy.toml: coverage_exempt module 15 has no reason \
                 (an exemption without a reason is a schema error)"
            ),
            "[{label}] the schema error must be stated: {}",
            rs.out()
        );
        // THE POINT OF THIS CASE. A rejected exemption must not quietly exempt:
        // module 15 stays required, so its shortfall still reports and it still
        // appears in the per-module block.
        assert!(
            rs.out().contains("domain module 15: 0 approved < min 1"),
            "[{label}] the rejected exemption suppressed the shortfall: {}",
            rs.out()
        );
        assert!(
            rs.out().contains("    m15: 0 approved of 0 scanned (min 1) [SHORT]"),
            "[{label}] the module left the required set: {}",
            rs.out()
        );
        assert!(
            !rs.out().contains("recorded exemptions"),
            "[{label}] a malformed row must not be recorded as an exemption: {}",
            rs.out()
        );
    }

    // (i) the known-GOOD control the schema check must not break: a row WITH a
    // reason does exempt, is PRINTED so the hole is visible, and the run is
    // GREEN. An attack-only suite ships an over-strict gate.
    let good = td.path().join("policy_with_reason.toml");
    write(
        &good,
        "[[coverage_exempt]]\nmodule = 15\nreason = \"fixture: module not yet authored\"\n",
    );
    let args = suite_args(
        reg.to_str().unwrap(),
        topics.to_str().unwrap(),
        bank.to_str().unwrap(),
        good.to_str().unwrap(),
    );
    let rs = assert_byte_identical("i recorded exemption honoured", &root, &args);
    assert_eq!(
        rs.code,
        0,
        "a well-formed exemption must still be honoured: {}",
        rs.out()
    );
    assert!(
        rs.out().contains("objective coverage GREEN"),
        "{}",
        rs.out()
    );
    assert!(
        rs.out()
            .contains("    m15: 0 approved of 0 scanned — exempt: fixture: module not yet authored"),
        "an exemption must be printed, not silent: {}",
        rs.out()
    );

    // …and an exemption may not ALSO carry a floor. Picking one is the rule.
    let both = td.path().join("policy_both.toml");
    write(
        &both,
        "[[domain_min]]\nmodule = 15\nmin_items = 3\n\
         [[coverage_exempt]]\nmodule = 15\nreason = \"conflicting\"\n",
    );
    let args = suite_args(
        reg.to_str().unwrap(),
        topics.to_str().unwrap(),
        bank.to_str().unwrap(),
        both.to_str().unwrap(),
    );
    let rs = assert_byte_identical("f exemption conflicting with a floor", &root, &args);
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out().contains(
            "bank_policy.toml: module 15 is both coverage_exempt and has a \
             [[domain_min]] floor — pick one"
        ),
        "{}",
        rs.out()
    );
    assert!(
        rs.out().contains("domain module 15: 0 approved"),
        "the conflicted module must stay required: {}",
        rs.out()
    );

    // An exemption naming a module the registry never declared is drift too.
    let undeclared = td.path().join("policy_undeclared.toml");
    write(
        &undeclared,
        "[[coverage_exempt]]\nmodule = 9\nreason = \"never declared\"\n",
    );
    let args = suite_args(
        reg.to_str().unwrap(),
        topics.to_str().unwrap(),
        bank.to_str().unwrap(),
        undeclared.to_str().unwrap(),
    );
    let rs = assert_byte_identical("f exemption for an undeclared module", &root, &args);
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out()
            .contains("bank_policy.toml: coverage_exempt module 9 is not in the domain registry"),
        "{}",
        rs.out()
    );
}

// ── (g)(h) cross-source drift ─────────────────────────────────────────────

#[test]
fn the_two_drift_shapes_are_byte_identical() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();

    let reg = td.path().join("domains_1.toml");
    write(&reg, &domains_registry(&[1]));
    let topics_ok = td.path().join("topics_ok.toml");
    write(&topics_ok, &topics_registry(&["01-fixture"]));
    let bank = td.path().join("bank_m1");
    plant_bank(&bank, &[1]);
    let empty_policy = td.path().join("policy_empty.toml");
    write(&empty_policy, "# fixture policy: no rows\n");

    // (g) a floor keyed to a module the registry never declared
    let stray = td.path().join("policy_stray_min.toml");
    write(
        &stray,
        "[[domain_min]]\nmodule = 1\nmin_items = 1\n\n\
         [[domain_min]]\nmodule = 15\nmin_items = 16\n",
    );
    let args = suite_args(
        reg.to_str().unwrap(),
        topics_ok.to_str().unwrap(),
        bank.to_str().unwrap(),
        stray.to_str().unwrap(),
    );
    let rs = assert_byte_identical("g domain_min for an undeclared module", &root, &args);
    assert_ne!(
        rs.code,
        0,
        "two sources of truth disagreeing must never be a pass: {}",
        rs.out()
    );
    assert!(
        rs.out().contains(
            "bank_policy.toml: [[domain_min]] module 15 is not declared in \
             the domain registry"
        ),
        "{}",
        rs.out()
    );
    // The known-GOOD leg: a floor keyed to a DECLARED module is not drift. This
    // gate applies its own floor of one item, so an aligned row changes nothing.
    let aligned = td.path().join("policy_aligned.toml");
    write(&aligned, "[[domain_min]]\nmodule = 1\nmin_items = 4\n");
    let args = suite_args(
        reg.to_str().unwrap(),
        topics_ok.to_str().unwrap(),
        bank.to_str().unwrap(),
        aligned.to_str().unwrap(),
    );
    let rs = assert_byte_identical("g aligned domain_min is not drift", &root, &args);
    assert_eq!(rs.code, 0, "{}", rs.out());
    assert!(
        !rs.out().contains("is not declared in the domain registry"),
        "an aligned floor must not report as drift: {}",
        rs.out()
    );

    // A malformed floor row is named rather than skipped.
    let unusable = td.path().join("policy_unusable.toml");
    write(&unusable, "[[domain_min]]\nmodule = 1\n[[domain_min]]\n");
    let args = suite_args(
        reg.to_str().unwrap(),
        topics_ok.to_str().unwrap(),
        bank.to_str().unwrap(),
        unusable.to_str().unwrap(),
    );
    let rs = assert_byte_identical("g unusable domain_min row", &root, &args);
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out()
            .contains("bank_policy.toml: unusable [[domain_min]] row {}"),
        "{}",
        rs.out()
    );

    // (h) a topic sitting in a domain the registry never declared. The fixture
    // carries one GOOD topic as well, so the finding proves the drift detector
    // fired rather than the "zero topics in a required domain" floor.
    let topics_drift = td.path().join("topics_drift.toml");
    write(
        &topics_drift,
        &topics_registry(&["01-fixture", "99-never-declared"]),
    );
    let args = suite_args(
        reg.to_str().unwrap(),
        topics_drift.to_str().unwrap(),
        bank.to_str().unwrap(),
        empty_policy.to_str().unwrap(),
    );
    let rs = assert_byte_identical("h topic in an undeclared domain", &root, &args);
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out().contains(
            "topics.toml: topic in an undeclared domain: t-fixture-2 \
             (domain='99-never-declared')"
        ),
        "the finding must name the topic AND the domain: {}",
        rs.out()
    );

    // …and the drift list truncates identically once it overruns the slice.
    let mut many = topics_registry(&["01-fixture"]);
    for i in 0..30 {
        many.push_str(&format!(
            "\n[[topic]]\nid = \"tx{i}\"\ndomain = \"nowhere-{i}\"\n"
        ));
    }
    let topics_many = td.path().join("topics_many.toml");
    write(&topics_many, &many);
    let args = suite_args(
        reg.to_str().unwrap(),
        topics_many.to_str().unwrap(),
        bank.to_str().unwrap(),
        empty_policy.to_str().unwrap(),
    );
    let rs = assert_byte_identical("h drift truncation footer", &root, &args);
    assert!(
        rs.out().contains(" more topics in undeclared domains"),
        "the truncation footer must be reached: {}",
        rs.out()
    );
}

// ── (j) anti-vacuous: an empty input set is an ERROR, never a pass ────────

#[test]
fn empty_and_missing_input_sets_are_errors_in_both() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();
    let bank = td.path().join("bank");
    plant_bank(&bank, &[1]);
    let bank_arg = bank.to_str().unwrap().to_string();
    let topics = td.path().join("topics_ok.toml");
    write(&topics, &topics_registry(&["01-fixture"]));
    let topics_arg = topics.to_str().unwrap().to_string();
    let no_policy = td.path().join("absent_policy.toml");
    let no_policy_arg = no_policy.to_str().unwrap().to_string();

    // a registry that parses but declares nothing
    let empty_reg = td.path().join("empty_domains.toml");
    write(&empty_reg, "schema_version = 1\n");
    let rs = assert_byte_identical(
        "j registry declaring zero modules",
        &root,
        &suite_args(
            empty_reg.to_str().unwrap(),
            &topics_arg,
            &bank_arg,
            &no_policy_arg,
        ),
    );
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out()
            .contains("domain registry declares zero modules (vacuous coverage is ERROR)"),
        "{}",
        rs.out()
    );
    // …and this oracle does NOT also say "zero required modules", because that
    // leg is guarded by `declared`. Reproduced, not corrected.
    assert!(
        !rs.out().contains("zero required modules after exemptions"),
        "the empty-registry leg must not double-report: {}",
        rs.out()
    );

    // a registry that is not there at all — reported once, and NOT also as
    // "declares zero modules", because that leg returns early
    let missing_reg = td.path().join("no_such_domains.toml");
    let rs = assert_byte_identical(
        "j missing registry",
        &root,
        &suite_args(
            missing_reg.to_str().unwrap(),
            &topics_arg,
            &bank_arg,
            &no_policy_arg,
        ),
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
    let reg = td.path().join("two.toml");
    write(&reg, &domains_registry(&[1, 2]));
    let rs = assert_byte_identical(
        "j missing bank dir",
        &root,
        &suite_args(
            reg.to_str().unwrap(),
            &topics_arg,
            missing_bank.to_str().unwrap(),
            &no_policy_arg,
        ),
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
    let all_exempt = td.path().join("policy_all_exempt.toml");
    write(
        &all_exempt,
        "[[coverage_exempt]]\nmodule = 1\nreason = \"a\"\n\
         [[coverage_exempt]]\nmodule = 2\nreason = \"b\"\n",
    );
    let rs = assert_byte_identical(
        "j every module exempted",
        &root,
        &suite_args(
            reg.to_str().unwrap(),
            &topics_arg,
            &bank_arg,
            all_exempt.to_str().unwrap(),
        ),
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

    // a claims registry with no [[claim]] rows
    let no_claims = td.path().join("claims_empty.toml");
    write(&no_claims, "schema_version = 1\n");
    let rs = assert_byte_identical(
        "j zero claim rows",
        &root,
        &[
            "--objectives",
            "registries/objectives.toml",
            "--claims",
            no_claims.to_str().unwrap(),
            "--domains",
            reg.to_str().unwrap(),
            "--topics",
            &topics_arg,
            "--bank",
            &bank_arg,
            "--policy",
            &no_policy_arg,
            "--skip-topic-coverage",
        ],
    );
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out()
            .contains("registries/claims.toml has zero [[claim]] rows (empty = ERROR)"),
        "{}",
        rs.out()
    );

    // a topics registry present but holding no topic in any required domain
    let topics_empty = td.path().join("topics_empty.toml");
    write(&topics_empty, "schema_version = 1\n");
    let rs = assert_byte_identical(
        "j topics with zero required-domain topics",
        &root,
        &suite_args(
            reg.to_str().unwrap(),
            topics_empty.to_str().unwrap(),
            &bank_arg,
            &no_policy_arg,
        ),
    );
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out()
            .contains("topics.toml has zero topics in a required domain"),
        "{}",
        rs.out()
    );

    // a topics registry that is not there at all
    let rs = assert_byte_identical(
        "j missing topics registry",
        &root,
        &suite_args(
            reg.to_str().unwrap(),
            td.path().join("no_such_topics.toml").to_str().unwrap(),
            &bank_arg,
            &no_policy_arg,
        ),
    );
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out().contains("missing topics registry:"),
        "{}",
        rs.out()
    );
    // …and the anti-vacuous leg fires too, on its own account. Until bd-9nyt it
    // could not: it carried `topics_path.is_file()`, so the one case that leaves
    // zero topics most loudly was the one case its condition excluded it from,
    // and the run was RED only because the line above happens to exist.
    assert!(
        rs.out()
            .contains("topics.toml has zero topics in a required domain"),
        "the anti-vacuous leg must not depend on a neighbouring check: {}",
        rs.out()
    );

    // the two registries this gate cannot run without, absent — the SHORT
    // report shape, with no body and no verdict block
    let rs = assert_byte_identical(
        "j missing objectives and claims",
        &root,
        &[
            "--objectives",
            td.path().join("no_obj.toml").to_str().unwrap(),
            "--claims",
            td.path().join("no_claims.toml").to_str().unwrap(),
            "--bank",
            &bank_arg,
        ],
    );
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(rs.out().starts_with("FAIL\n"), "{}", rs.out());
    assert!(
        rs.out().contains("missing objectives registry:")
            && rs.out().contains("missing claims registry:"),
        "{}",
        rs.out()
    );
    assert!(
        !rs.out().contains("gate=l7-objective-coverage"),
        "the early exit prints no body: {}",
        rs.out()
    );
}

// ── (k) emission ORDER ────────────────────────────────────────────────────

#[test]
fn emission_order_is_reproduced_exactly() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();

    // The registry declares its modules OUT of numeric order on purpose, the
    // bank carries two modules the registry never declared, and two modules are
    // exempted out of declaration order. Python dict order and Rust map order
    // differ, so this pins the sort rather than luck.
    let reg = td.path().join("domains.toml");
    write(&reg, &domains_registry(&[3, 1, 22, 2]));
    let bank = td.path().join("bank");
    plant_bank(&bank, &[1, 3, 22, 9, 7]);
    let policy = td.path().join("policy.toml");
    write(
        &policy,
        "[[coverage_exempt]]\nmodule = 22\nreason = \"later\"\n\
         [[coverage_exempt]]\nmodule = 3\nreason = \"earlier\"\n",
    );
    // Two topics in required domains with no items, so the WARNINGS block is
    // emitted and its position in the body is under test too.
    let topics = td.path().join("topics.toml");
    write(&topics, &topics_registry(&["01-fixture", "02-fixture"]));

    let rs = assert_byte_identical(
        "k emission order",
        &root,
        &[
            "--objectives",
            "registries/objectives.toml",
            "--claims",
            "registries/claims.toml",
            "--domains",
            reg.to_str().unwrap(),
            "--topics",
            topics.to_str().unwrap(),
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
    let gate_at = at("  gate=l7-objective-coverage");
    let objectives_at = at("  objectives=");
    let claims_at = at("  claims=");
    let registry_at = at("  registry=");
    let policy_at = at("  policy=");
    let bank_at = at("  bank=");
    let items_at = at("  items=");
    let regobj_at = at("  registry_objectives=");
    let knownclaims_at = at("  known_claims=");
    let modules_at = at("  modules (");
    assert!(
        gate_at < objectives_at
            && objectives_at < claims_at
            && claims_at < registry_at
            && registry_at < policy_at
            && policy_at < bank_at
            && bank_at < items_at
            && items_at < regobj_at
            && regobj_at < knownclaims_at
            && knownclaims_at < modules_at,
        "header order drifted:\n{out}"
    );

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

    // then the five fixed trailer lines, then warnings, then the failure list
    let topics_line = at("  primary_topics=");
    let gap_line = at("  gap: no full LO×item matrix");
    let note_line = at("  note: coverage ≠ exam pass probability");
    let warnings_hdr = at("  warnings:");
    let failures_hdr = at("  failures:");
    assert!(x09 < topics_line, "trailer order drifted:\n{out}");
    assert!(
        topics_line < gap_line && gap_line < note_line && note_line < warnings_hdr,
        "trailer order drifted:\n{out}"
    );
    assert!(
        warnings_hdr < failures_hdr,
        "warnings precede failures:\n{out}"
    );

    // the failure list's own order is
    // registry → exemption → floor → bank → item → shortfall → topics
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
    plant_bank(&bank2, &[1]);
    write(&bank2.join("zz-junk.toml"), "label = \"nothing useful\"\n");
    write(
        &bank2.join("zz-badmod.toml"),
        "id = \"zz-badmod\"\nmodule = \"nope\"\n",
    );
    let topics2 = td.path().join("messy_topics.toml");
    write(&topics2, &topics_registry(&["01-fixture", "77-nowhere"]));

    let rs = assert_byte_identical(
        "k failure-list order",
        &root,
        &suite_args(
            reg2.to_str().unwrap(),
            topics2.to_str().unwrap(),
            bank2.to_str().unwrap(),
            policy2.to_str().unwrap(),
        ),
    );
    let out = rs.out();
    let at = |needle: &str| {
        out.find(needle)
            .unwrap_or_else(|| panic!("missing {needle:?} in:\n{out}"))
    };
    let f_registry = at("    - domains.toml: {'id': '', 'order': 'nope'} has no usable order");
    let f_exempt = at("    - bank_policy.toml: coverage_exempt module 2 has no reason");
    let f_floor = at("    - bank_policy.toml: [[domain_min]] module 8 is not declared");
    let f_load = at("    - zz-junk.toml: no id or items[]");
    let f_badmod = at("    - zz-badmod: bad module 'nope'");
    let f_short = at("    - domain module 2: 0 approved < min 1");
    let f_topic = at("    - topics.toml: topic in an undeclared domain:");
    assert!(
        f_registry < f_exempt
            && f_exempt < f_floor
            && f_floor < f_load
            && f_load < f_badmod
            && f_badmod < f_short
            && f_short < f_topic,
        "failure-list order drifted:\n{out}"
    );
}

#[test]
fn the_failure_and_warning_lists_truncate_identically() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();

    // Enough declared-but-unstocked modules to overrun the oracle's report
    // slice, so the truncation footer is compared too.
    let orders: Vec<i64> = (1..=60).collect();
    let reg = td.path().join("many.toml");
    write(&reg, &domains_registry(&orders));
    let bank = td.path().join("bank");
    plant_bank(&bank, &[1]);
    let topics = td.path().join("topics.toml");
    write(&topics, &topics_registry(&["01-fixture"]));
    let no_policy = td.path().join("absent.toml");

    let rs = assert_byte_identical(
        "k failure truncation footer",
        &root,
        &suite_args(
            reg.to_str().unwrap(),
            topics.to_str().unwrap(),
            bank.to_str().unwrap(),
            no_policy.to_str().unwrap(),
        ),
    );
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out().contains(" more\n"),
        "the truncation footer must be reached: {}",
        rs.out()
    );

    // …and the WARNING list, which has its own slice and its own footer. Every
    // topic sits in a required domain and none is cited by an item, so the soft
    // floor produces one warning apiece.
    let mut warn_topics = String::from("schema_version = 1\n");
    for i in 0..30 {
        warn_topics.push_str(&format!(
            "\n[[topic]]\nid = \"tw{i}\"\ndomain = \"01-fixture\"\n"
        ));
    }
    let wt = td.path().join("warn_topics.toml");
    write(&wt, &warn_topics);
    let reg2 = td.path().join("one.toml");
    write(&reg2, &domains_registry(&[1]));
    let rs = assert_byte_identical(
        "k warning truncation footer",
        &root,
        &[
            "--objectives",
            "registries/objectives.toml",
            "--claims",
            "registries/claims.toml",
            "--domains",
            reg2.to_str().unwrap(),
            "--topics",
            wt.to_str().unwrap(),
            "--bank",
            bank.to_str().unwrap(),
            "--policy",
            no_policy.to_str().unwrap(),
        ],
    );
    // Soft by default: shortfalls WARN and the run is still GREEN. That is the
    // oracle's documented honesty, reproduced rather than tightened.
    assert_eq!(
        rs.code,
        0,
        "primary-topic shortfalls must soft-warn by default: {}",
        rs.out()
    );
    assert!(rs.out().contains("  warnings:"), "{}", rs.out());
    assert!(
        rs.out().contains("topic shortfalls soft-warn)"),
        "the GREEN line must say the shortfalls were soft: {}",
        rs.out()
    );

    // …and --strict-topics turns the same input RED, on both sides.
    let rs = assert_byte_identical(
        "k strict topics turns the same input red",
        &root,
        &[
            "--objectives",
            "registries/objectives.toml",
            "--claims",
            "registries/claims.toml",
            "--domains",
            reg2.to_str().unwrap(),
            "--topics",
            wt.to_str().unwrap(),
            "--bank",
            bank.to_str().unwrap(),
            "--policy",
            no_policy.to_str().unwrap(),
            "--strict-topics",
        ],
    );
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(rs.out().contains("mode=strict"), "{}", rs.out());
    assert!(!rs.out().contains("PASS"), "{}", rs.out());
}

// ── (l) --write-json, compared as bytes ───────────────────────────────────

#[test]
fn the_written_summary_is_byte_identical() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();
    let target = td.path().join("out/objectives.json");
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
        "the written objective summary differs"
    );
    COMPARED.fetch_add(1, Ordering::SeqCst);
    assert!(
        rs.out().contains("  wrote "),
        "the write must be announced: {}",
        rs.out()
    );

    // …and on a RED run, where the summary is still written and the `wrote` line
    // still precedes the failure list.
    let reg = td.path().join("domains.toml");
    write(&reg, &domains_registry(&[1, 2]));
    let bank = td.path().join("bank");
    plant_bank(&bank, &[1]);
    let topics = td.path().join("topics.toml");
    write(&topics, &topics_registry(&["01-fixture", "02-fixture"]));
    let policy = td.path().join("policy.toml");
    write(
        &policy,
        "[[coverage_exempt]]\nmodule = 9\nreason = \"never declared\"\n",
    );
    let red_target = td.path().join("out/red.json");
    let red_arg = red_target.to_str().unwrap().to_string();
    let mut args = suite_args(
        reg.to_str().unwrap(),
        topics.to_str().unwrap(),
        bank.to_str().unwrap(),
        policy.to_str().unwrap(),
    );
    args.pop(); // drop --skip-topic-coverage: exercise the soft-warn summary
    args.push("--write-json");
    args.push(&red_arg);

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
    // the summary is a MACHINE ledger: what it records must be there
    let text = String::from_utf8_lossy(&rs_json);
    for needle in [
        "\"domain_shortfalls\"",
        "\"required_modules\"",
        "\"registry_objectives\"",
        "\"primary_topic_shortfalls\"",
        "\"status\": \"fail\"",
    ] {
        assert!(text.contains(needle), "missing {needle} in ledger: {text}");
    }
}

/// The write happens BEFORE the verdict, so a write that CANNOT happen is a
/// failure of the gate rather than a traceback under a PASS someone already
/// read. The fixture makes the target's parent a regular FILE, which is
/// deterministic and does not depend on the euid the suite runs as.
#[test]
fn an_unwritable_summary_target_fails_first_with_no_pass_on_stdout() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();
    let blocker = td.path().join("afile");
    write(&blocker, "this is a regular file, not a directory\n");

    for (label, rel) in [
        ("direct parent", "afile/summary.json"),
        ("nested parent", "afile/sub/summary.json"),
    ] {
        let target = td.path().join(rel);
        let target_arg = target.to_str().unwrap().to_string();
        let rs = assert_byte_identical(
            &format!("l unwritable summary, {label}"),
            &root,
            &["--write-json", &target_arg],
        );
        assert_ne!(
            rs.code,
            0,
            "[{label}] an unwritable summary must be RED: {}",
            rs.out()
        );
        assert!(
            rs.out().contains("could not write summary to "),
            "[{label}] the failure must name the write: {}",
            rs.out()
        );
        // THE POINT OF THIS CASE. The live tree is otherwise GREEN, so a gate
        // that printed its verdict before writing would say PASS and then die.
        assert_eq!(
            rs.out().matches("PASS").count(),
            0,
            "[{label}] a verdict was printed that the exit code then contradicted:\n{}",
            rs.out()
        );
        assert!(
            rs.out().starts_with("FAIL\n") || rs.out().starts_with("FAIL"),
            "[{label}] the verdict must lead the report: {}",
            rs.out()
        );
        assert!(
            !rs.out().contains("  wrote "),
            "[{label}] a failed write must not be announced as one: {}",
            rs.out()
        );
        assert!(
            !target.exists(),
            "[{label}] the summary was written after all"
        );
    }
}

// ── (m) shapes the shell suite never reaches ──────────────────────────────

#[test]
fn path_and_option_shapes_are_byte_identical() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();

    // engine-root-relative arguments, including untidy spellings the printed
    // header must normalise the same way on both sides
    assert_byte_identical("m relative bank", &root, &["--bank", "bank/items"]);
    assert_byte_identical(
        "m untidy relative bank",
        &root,
        &["--bank", "./bank//items/"],
    );
    assert_byte_identical(
        "m relative registries",
        &root,
        &[
            "--domains",
            "knowledge/domains.toml",
            "--topics",
            "knowledge/topics.toml",
            "--policy",
            "knowledge/bank_policy.toml",
        ],
    );

    // `--opt=value` and argparse's unambiguous prefixes
    assert_byte_identical("m equals form", &root, &["--bank=bank/items"]);
    assert_byte_identical("m abbreviated option", &root, &["--ban", "bank/items"]);
    assert_byte_identical(
        "m shortest prefixes",
        &root,
        &[
            "--o",
            "registries/objectives.toml",
            "--c",
            "registries/claims.toml",
            "--t",
            "knowledge/topics.toml",
            "--d",
            "knowledge/domains.toml",
            "--p",
            "knowledge/bank_policy.toml",
            "--b",
            "bank/items",
        ],
    );

    // the two store_true flags, together and apart
    assert_byte_identical("m skip flag", &root, &["--skip-topic-coverage"]);
    assert_byte_identical("m strict flag", &root, &["--strict-topics"]);
    assert_byte_identical(
        "m both flags",
        &root,
        &["--skip-topic-coverage", "--strict-topics"],
    );

    // The topic floor, raised and lowered.
    //
    // `m min topic 0` USED TO STOP AT "the two sides agree", and it passed for
    // weeks while both of them compared not one topic and printed
    // `covered=106 shortfalls=0 mode=strict` under exit 0 — the header's own
    // "a defect faithfully ported is still a defect", caught in this
    // harness's blind spot (bd-differential-shared-blindspot-4qje). Agreement
    // is necessary and not sufficient. The case now pins the ERROR and the
    // named finding; a shared defect that still printed PASS / covered=N /
    // mode=strict would fail here even if both sides agreed.
    const FLOOR_OFF_FINDING: &str =
        "--min-items-per-topic 0 turns the primary-topic floor off without saying so";
    let rs = assert_byte_identical("m min topic 0", &root, &["--min-items-per-topic", "0"]);
    assert_ne!(
        rs.code,
        0,
        "a floor of 0 disables every topic comparison; that must not be a pass: {}",
        rs.out()
    );
    assert!(
        rs.out().starts_with("FAIL"),
        "min=0 must lead with FAIL, not PASS: {}",
        rs.out()
    );
    assert!(
        rs.out().contains(FLOOR_OFF_FINDING),
        "min=0 must name the finding, not just go non-zero: {}",
        rs.out()
    );
    assert!(
        rs.out().contains("mode=off") && rs.out().contains("covered=n/a"),
        "the report must not name a mode it is not in, nor a coverage number no \
         comparison produced: {}",
        rs.out()
    );

    // THE MEASURED DEFECT. `--min-items-per-topic 0 --strict-topics` is what
    // printed `mode=strict covered=106 EXIT 0` with zero comparisons. Pinning
    // only the unflagged spelling would let that string become printable again.
    let rs = assert_byte_identical(
        "m min topic 0 under --strict-topics",
        &root,
        &["--min-items-per-topic", "0", "--strict-topics"],
    );
    assert_ne!(
        rs.code,
        0,
        "min=0 under --strict-topics is the measured vacuous PASS: {}",
        rs.out()
    );
    assert!(
        rs.out().starts_with("FAIL") && rs.out().contains(FLOOR_OFF_FINDING),
        "the measured spelling must stay FAIL and name the finding: {}",
        rs.out()
    );
    assert!(
        rs.out().contains("mode=off") && rs.out().contains("covered=n/a"),
        "zero comparisons must not report a coverage number: {}",
        rs.out()
    );
    assert!(
        !rs.out().contains("mode=strict"),
        "a floor that compared nothing must not report mode=strict: {}",
        rs.out()
    );
    assert!(
        !rs.out().contains("covered=106"),
        "a floor that compared nothing must not report 106/106 covered: {}",
        rs.out()
    );

    let rs = assert_byte_identical("m min topic 5", &root, &["--min-items-per-topic", "5"]);
    assert!(rs.out().contains("min_per_topic=5"), "{}", rs.out());
    assert!(
        !rs.out().contains("mode=off") && !rs.out().contains("covered=n/a"),
        "a raised floor must actually compare: {}",
        rs.out()
    );

    // an absent policy is NOT an error — it is simply an empty ledger
    let absent = td.path().join("absent_policy.toml");
    let rs = assert_byte_identical(
        "m absent policy",
        &root,
        &["--policy", absent.to_str().unwrap()],
    );
    assert!(rs.out().contains("policy=absent"), "{}", rs.out());
    assert_eq!(rs.code, 0, "{}", rs.out());
}

#[test]
fn malformed_registry_rows_and_bank_files_are_byte_identical() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();
    let bank = td.path().join("bank");
    plant_bank(&bank, &[1]);
    let bank_arg = bank.to_str().unwrap().to_string();
    let topics = td.path().join("topics.toml");
    write(&topics, &topics_registry(&["01-fixture"]));
    let topics_arg = topics.to_str().unwrap().to_string();
    let no_policy = td.path().join("absent.toml");
    let no_policy_arg = no_policy.to_str().unwrap().to_string();

    // a [[domain]] row whose order cannot be coerced
    let bad_order = td.path().join("bad_order.toml");
    write(
        &bad_order,
        "[[domain]]\nid = \"01-fixture\"\norder = \"x\"\n\n[[domain]]\nid = \"02-beta\"\norder = 2\n",
    );
    let rs = assert_byte_identical(
        "m domain row with no usable order",
        &root,
        &suite_args(
            bad_order.to_str().unwrap(),
            &topics_arg,
            &bank_arg,
            &no_policy_arg,
        ),
    );
    assert!(
        rs.out()
            .contains("domains.toml: '01-fixture' has no usable order"),
        "the id is printed through repr(), quotes and all: {}",
        rs.out()
    );

    // two rows claiming one order
    let dup = td.path().join("dup.toml");
    write(
        &dup,
        "[[domain]]\nid = \"01-a\"\norder = 1\n\n[[domain]]\nid = \"01-again\"\norder = 1\n",
    );
    let rs = assert_byte_identical(
        "m duplicate order",
        &root,
        &suite_args(
            dup.to_str().unwrap(),
            &topics_arg,
            &bank_arg,
            &no_policy_arg,
        ),
    );
    assert!(
        rs.out()
            .contains("domains.toml: duplicate order 1 (01-a and 01-again)"),
        "{}",
        rs.out()
    );

    // a [[domain]] key that is a list of scalars rather than a table array
    let not_table = td.path().join("not_table.toml");
    write(&not_table, "domain = [\"justastring\"]\n");
    let rs = assert_byte_identical(
        "m domain row is not a table",
        &root,
        &suite_args(
            not_table.to_str().unwrap(),
            &topics_arg,
            &bank_arg,
            &no_policy_arg,
        ),
    );
    assert!(
        rs.out()
            .contains("domains.toml: [[domain]] row is not a table: 'justastring'"),
        "{}",
        rs.out()
    );

    // a [[coverage_exempt]] key that is not a table array, and one with no module
    let reg = td.path().join("two.toml");
    write(&reg, &domains_registry(&[1, 2]));
    for (label, body, needle) in [
        (
            "exempt row is not a table",
            "coverage_exempt = [\"x\"]\n",
            "bank_policy.toml: coverage_exempt row is not a table: 'x'",
        ),
        (
            "exempt row has no module",
            "[[coverage_exempt]]\nreason = \"why\"\n",
            "bank_policy.toml: coverage_exempt row has no usable module: {'reason': 'why'}",
        ),
    ] {
        let policy = td.path().join(format!("{}.toml", label.replace(' ', "_")));
        write(&policy, body);
        let rs = assert_byte_identical(
            &format!("m {label}"),
            &root,
            &suite_args(
                reg.to_str().unwrap(),
                &topics_arg,
                &bank_arg,
                policy.to_str().unwrap(),
            ),
        );
        assert!(rs.out().contains(needle), "[{label}] {}", rs.out());
    }

    // bank files: junk, an uncoercible module, a missing module key, and an
    // objective_ids entry that resolves to nothing
    let messy = td.path().join("messy_bank");
    plant_bank(&messy, &[1, 2]);
    write(&messy.join("zz-junk.toml"), "label = \"nothing useful\"\n");
    write(
        &messy.join("zz-badmod.toml"),
        "id = \"zz-badmod\"\nmodule = \"nope\"\n",
    );
    write(&messy.join("zz-nomod.toml"), "id = \"zz-nomod\"\n");
    write(
        &messy.join("zz-obj.toml"),
        "id = \"zz-obj\"\nmodule = 1\nobjective_ids = [\"obj-does-not-exist\", \"\"]\n",
    );
    let rs = assert_byte_identical(
        "m malformed bank files",
        &root,
        &suite_args(
            reg.to_str().unwrap(),
            &topics_arg,
            messy.to_str().unwrap(),
            &no_policy_arg,
        ),
    );
    assert_ne!(rs.code, 0, "{}", rs.out());
    for needle in [
        "zz-junk.toml: no id or items[]",
        "zz-badmod: bad module 'nope'",
        "zz-nomod: bad module None",
        "zz-obj: unknown objective_id 'obj-does-not-exist'",
        "zz-obj: empty objective_ids entry",
    ] {
        assert!(
            rs.out().contains(needle),
            "missing {needle:?}: {}",
            rs.out()
        );
    }

    // The file-granular anti-vacuous rule bd-0czh added to the oracle ON THE DAY
    // this port was written: an `items = []` file used to contribute nothing and
    // say nothing, so a file that was never really checked reported exactly like
    // one that passed. The port was RE-BASELINED onto the fix rather than kept
    // on the fail-open behaviour, and this leg is what pins that decision.
    let quiet = td.path().join("quiet_bank");
    plant_bank(&quiet, &[1, 2]);
    write(&quiet.join("zz-silently-empty.toml"), "items = []\n");
    write(
        &quiet.join("zz-nontable-items.toml"),
        "items = [\"nope\"]\n",
    );
    let rs = assert_byte_identical(
        "m items[] yielding zero items is named in both",
        &root,
        &suite_args(
            reg.to_str().unwrap(),
            &topics_arg,
            quiet.to_str().unwrap(),
            &no_policy_arg,
        ),
    );
    assert_ne!(
        rs.code,
        0,
        "a file that contributed nothing must not report like one that passed: {}",
        rs.out()
    );
    for needle in [
        "zz-silently-empty.toml: items[] yielded zero items (vacuous file scan is ERROR)",
        "zz-nontable-items.toml: items[] yielded zero items (vacuous file scan is ERROR)",
    ] {
        assert!(
            rs.out().contains(needle),
            "missing {needle:?}: {}",
            rs.out()
        );
    }

    // the `items[]` table-array form must still be counted
    let nested = td.path().join("nested_bank");
    std::fs::create_dir_all(&nested).unwrap();
    write(
        &nested.join("multi.toml"),
        "[[items]]\nid = \"n1\"\nmodule = 1\nstatus = \"approved\"\n\n\
         [[items]]\nid = \"n2\"\nmodule = 2\nstatus = \"approved\"\n",
    );
    let rs = assert_byte_identical(
        "m items[] table array is counted",
        &root,
        &suite_args(
            reg.to_str().unwrap(),
            &topics_arg,
            nested.to_str().unwrap(),
            &no_policy_arg,
        ),
    );
    assert_eq!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out().contains("items=2 scanned, 2 approved"),
        "{}",
        rs.out()
    );

    // duplicate objective ids, an id-less objective, and a blank claim_id entry
    let obj = td.path().join("objectives_messy.toml");
    write(
        &obj,
        "schema_version = 1\n\
         [[objective]]\nid = \"obj-a\"\nclaim_ids = [\"c1\"]\n\
         [[objective]]\nid = \"obj-a\"\nclaim_ids = [\"c1\"]\n\
         [[objective]]\nid = \"\"\nclaim_ids = [\"c1\"]\n\
         [[objective]]\nid = \"obj-b\"\nclaim_ids = [\"\", \"c1\"]\n",
    );
    let claims = td.path().join("claims_c1.toml");
    write(&claims, "schema_version = 1\n[[claim]]\nid = \"c1\"\n");
    let rs = assert_byte_identical(
        "m messy objectives registry",
        &root,
        &[
            "--objectives",
            obj.to_str().unwrap(),
            "--claims",
            claims.to_str().unwrap(),
            "--domains",
            reg.to_str().unwrap(),
            "--topics",
            &topics_arg,
            "--bank",
            &bank_arg,
            "--policy",
            &no_policy_arg,
            "--skip-topic-coverage",
        ],
    );
    assert_ne!(rs.code, 0, "{}", rs.out());
    for needle in [
        "objective with empty/missing id",
        "objective obj-b: empty claim_id entry",
        "duplicate objective ids: ['obj-a']",
    ] {
        assert!(
            rs.out().contains(needle),
            "missing {needle:?}: {}",
            rs.out()
        );
    }
    assert!(
        // Three ids survive (the blank-id row is skipped before it is counted)
        // and two of them resolve every claim they cite — obj-b's blank entry
        // costs it the tick, the duplicate obj-a rows each earn one.
        rs.out()
            .contains("registry_objectives=3 claim_resolve_ok=2"),
        "the counters must agree too: {}",
        rs.out()
    );

    // a topic row with no usable id
    let topics_bad = td.path().join("topics_bad.toml");
    write(
        &topics_bad,
        "schema_version = 1\n[[topic]]\ndomain = \"01-fixture\"\n\
         [[topic]]\nid = \"t-fixture-1\"\ndomain = \"01-fixture\"\n",
    );
    let rs = assert_byte_identical(
        "m topic with empty/missing id",
        &root,
        &suite_args(
            reg.to_str().unwrap(),
            topics_bad.to_str().unwrap(),
            &bank_arg,
            &no_policy_arg,
        ),
    );
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out().contains("topic with empty/missing id"),
        "{}",
        rs.out()
    );

    // a `topic` key that is not iterable at all — CAUGHT by the oracle's try,
    // and so reported rather than raised
    let topics_int = td.path().join("topics_int.toml");
    write(&topics_int, "schema_version = 1\ntopic = 5\n");
    let rs = assert_byte_identical(
        "m topic key is an int",
        &root,
        &suite_args(
            reg.to_str().unwrap(),
            topics_int.to_str().unwrap(),
            &bank_arg,
            &no_policy_arg,
        ),
    );
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out()
            .contains("parse topics: 'int' object is not iterable"),
        "{}",
        rs.out()
    );
}

// ── (n) the oracle's uncaught exceptions ──────────────────────────────────

/// Several exotic inputs raise rather than report. CPython flushes what it
/// printed, writes a traceback and exits 1. The port reproduces stdout and the
/// exit code exactly; the traceback text is the single surface it does not
/// reproduce, which is asserted here rather than left implicit.
#[test]
fn the_raise_paths_match_except_for_the_traceback() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();
    let bank = td.path().join("bank");
    plant_bank(&bank, &[1]);
    let bank_arg = bank.to_str().unwrap().to_string();
    let topics = td.path().join("topics.toml");
    write(&topics, &topics_registry(&["01-fixture"]));
    let topics_arg = topics.to_str().unwrap().to_string();
    let no_policy = td.path().join("absent.toml");
    let no_policy_arg = no_policy.to_str().unwrap().to_string();
    let reg = td.path().join("one.toml");
    write(&reg, &domains_registry(&[1]));

    // a [[claim]] array holding something other than tables: `row.get` on a str
    let bad_claims = td.path().join("claims_bad.toml");
    write(
        &bad_claims,
        "schema_version = 1\nclaim = [\"justastring\"]\n",
    );
    // a `domain` key that is not iterable
    let dom_int = td.path().join("domains_int.toml");
    write(&dom_int, "schema_version = 1\ndomain = 7\n");
    // an infinite float where an integer is expected
    let dom_inf = td.path().join("domains_inf.toml");
    write(&dom_inf, "[[domain]]\nid = \"x\"\norder = inf\n");
    // a [[domain_min]] module that passes the isdigit screen but not int()
    let pol_dd = td.path().join("policy_double_dash.toml");
    write(&pol_dd, "[[domain_min]]\nmodule = \"--5\"\nmin_items = 1\n");
    // a bank item whose module is an infinite float
    let bank_inf = td.path().join("bank_inf");
    std::fs::create_dir_all(&bank_inf).unwrap();
    write(&bank_inf.join("z.toml"), "id = \"z\"\nmodule = inf\n");

    let cases: Vec<(&str, Vec<String>)> = vec![
        (
            "n claim row is not a table",
            vec![
                "--objectives".into(),
                "registries/objectives.toml".into(),
                "--claims".into(),
                bad_claims.to_str().unwrap().into(),
                "--domains".into(),
                reg.to_str().unwrap().into(),
                "--topics".into(),
                topics_arg.clone(),
                "--bank".into(),
                bank_arg.clone(),
                "--policy".into(),
                no_policy_arg.clone(),
            ],
        ),
        (
            "n domain key is not iterable",
            suite_args(
                dom_int.to_str().unwrap(),
                &topics_arg,
                &bank_arg,
                &no_policy_arg,
            )
            .into_iter()
            .map(String::from)
            .collect(),
        ),
        (
            "n order is an infinite float",
            suite_args(
                dom_inf.to_str().unwrap(),
                &topics_arg,
                &bank_arg,
                &no_policy_arg,
            )
            .into_iter()
            .map(String::from)
            .collect(),
        ),
        (
            "n domain_min module passes isdigit but not int()",
            suite_args(
                reg.to_str().unwrap(),
                &topics_arg,
                &bank_arg,
                pol_dd.to_str().unwrap(),
            )
            .into_iter()
            .map(String::from)
            .collect(),
        ),
        (
            "n item module is an infinite float",
            suite_args(
                reg.to_str().unwrap(),
                &topics_arg,
                bank_inf.to_str().unwrap(),
                &no_policy_arg,
            )
            .into_iter()
            .map(String::from)
            .collect(),
        ),
    ];

    for (label, argv) in cases {
        let args: Vec<&str> = argv.iter().map(String::as_str).collect();
        let py = python(&root, &args);
        let rs = rust(&root, &args);
        assert_eq!(
            py.stdout,
            rs.stdout,
            "[{label}] STDOUT must still match byte for byte on the raise path:\n\
             --- python ---\n{}\n--- rust ---\n{}",
            py.out(),
            rs.out()
        );
        assert_eq!(
            py.code, rs.code,
            "[{label}] EXIT CODE differs on the raise path"
        );
        assert_eq!(
            py.code, 1,
            "[{label}] the oracle exits 1 on an uncaught exception"
        );
        assert!(
            !py.stderr.is_empty() && !rs.stderr.is_empty(),
            "[{label}] both sides must say something on stderr: python {:?} rust {:?}",
            py.err(),
            rs.err()
        );
        COMPARED.fetch_add(1, Ordering::SeqCst);
    }
}

// ── floors measure the approved pool, not the file set (bd-f996) ──────────

fn plant_item(dir: &Path, id: &str, module: i64, status: Option<&str>, topic: &str) {
    std::fs::create_dir_all(dir).unwrap();
    let st = status.map(|s| format!("status = {s:?}\n")).unwrap_or_default();
    write(
        &dir.join(format!("{id}.toml")),
        &format!("id = {id:?}\nmodule = {module}\n{st}topic_ids = [{topic:?}]\n"),
    );
}

fn f996_tree(td: &tempfile::TempDir, orders: &[i64], topic_doms: &[&str]) -> (String, String, String) {
    let reg = td.path().join("domains.toml");
    write(&reg, &domains_registry(orders));
    let topics = td.path().join("topics.toml");
    write(&topics, &topics_registry(topic_doms));
    let policy = td.path().join("policy.toml");
    write(&policy, "# fixture policy: no rows\n");
    (
        reg.to_str().unwrap().to_string(),
        topics.to_str().unwrap().to_string(),
        policy.to_str().unwrap().to_string(),
    )
}

/// Retire every item under one topic WITHOUT deleting a file → RED naming the
/// topic, approved count, and floor. File count stays >= 1 (old gate GREEN).
#[test]
fn retiring_every_item_under_one_topic_is_red_on_the_approved_floor() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();
    let (reg, topics, policy) = f996_tree(&td, &[1, 2], &["01-fixture", "02-fixture"]);
    let bank = td.path().join("bank");
    plant_item(&bank, "a1", 1, Some("approved"), "t-fixture-1");
    plant_item(&bank, "a2", 1, Some("approved"), "t-fixture-1");
    plant_item(&bank, "b1", 2, Some("approved"), "t-fixture-2");
    let bank_s = bank.to_str().unwrap().to_string();
    let args = vec![
        "--objectives", "registries/objectives.toml",
        "--claims", "registries/claims.toml",
        "--domains", reg.as_str(), "--topics", topics.as_str(),
        "--bank", bank_s.as_str(), "--policy", policy.as_str(),
        "--strict-topics",
    ];
    let green = assert_byte_identical("retire-topic control", &root, &args);
    assert_eq!(green.code, 0, "control must be GREEN: {}", green.out());
    assert!(green.out().contains("items=3 scanned, 3 approved"), "{}", green.out());
    for name in ["a1.toml", "a2.toml"] {
        let p = bank.join(name);
        let text = std::fs::read_to_string(&p).unwrap();
        let flipped = text.replace("status = \"approved\"", "status = \"retired\"");
        assert_ne!(flipped, text);
        std::fs::write(&p, flipped).unwrap();
    }
    assert_eq!(std::fs::read_dir(&bank).unwrap().count(), 3, "no file deleted");
    let rs = assert_byte_identical("retire-topic injection", &root, &args);
    assert_ne!(rs.code, 0, "approved-pool shortfall must be RED: {}", rs.out());
    let out = rs.out();
    assert!(out.contains("topic t-fixture-1: 0 approved < min 1 (2 scanned, 2 not approved)"), "{out}");
    assert!(out.contains("items=3 scanned, 1 approved"), "{out}");
    assert!(out.contains("domain module 1: 0 approved < min 1 (2 scanned, 2 not approved)"), "{out}");
}

#[test]
fn a_bank_of_only_retired_items_is_an_error_distinct_from_empty_bank() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();
    let (reg, topics, policy) = f996_tree(&td, &[1, 2], &["01-fixture"]);
    let bank = td.path().join("bank");
    plant_item(&bank, "r1", 1, Some("retired"), "t-fixture-1");
    plant_item(&bank, "r2", 1, Some("retired"), "t-fixture-1");
    plant_item(&bank, "d1", 2, None, "t-fixture-1");
    let rs = assert_byte_identical(
        "zero approved items",
        &root,
        &suite_args(&reg, &topics, bank.to_str().unwrap(), &policy),
    );
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(rs.out().contains("zero approved items (3 scanned)"), "{}", rs.out());
    assert!(!rs.out().contains("empty bank: zero items loaded"), "{}", rs.out());
    assert!(!rs.out().contains("unknown status"), "{}", rs.out());
}

#[test]
fn an_unrecognised_status_is_named_rather_than_bucketed() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();
    let (reg, topics, policy) = f996_tree(&td, &[1], &["01-fixture"]);
    let bank = td.path().join("bank");
    plant_item(&bank, "good", 1, Some("approved"), "t-fixture-1");
    plant_item(&bank, "odd", 1, Some("published"), "t-fixture-1");
    let rs = assert_byte_identical(
        "unknown status",
        &root,
        &suite_args(&reg, &topics, bank.to_str().unwrap(), &policy),
    );
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(rs.out().contains("odd: unknown status 'published'"), "{}", rs.out());
    assert!(rs.out().contains("m01: 1 approved of 2 scanned (min 1) [ok]"), "{}", rs.out());
}

// ── the control: a full copy of the live bank ─────────────────────────────

/// A full copy of the live bank, checked against the live registries, is GREEN —
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
