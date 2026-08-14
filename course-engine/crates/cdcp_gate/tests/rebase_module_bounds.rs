//! bd-lt7 — gates must not encode a known defect as an invariant.
//!
//! # CLAIM: FLOOR-RAISE
//!
//! Three things are held here, and nothing more:
//!
//!   1. `scripts/verify_coverage.py` and `scripts/build_units.py` derive their
//!      module set from a registry (`knowledge/domains.toml` and
//!      `web/data/modules_index.json` respectively) rather than from a numeric
//!      bound, and each still trips on a real defect after the rebase.
//!   2. Neither refuses legitimate work — every rebased gate has a known-GOOD
//!      leg here, because an attack-only suite ships an over-strict gate, and
//!      over-strict gates get routed around.
//!   3. Every numeric module bound left in `scripts/` and `crates/` is
//!      INVENTORIED below with a verdict. A bound that is not in the inventory
//!      fails this test with its file, line and text. The inventory is the
//!      "justified in place" half of bd-lt7's acceptance: a bound may stay, but
//!      only if someone wrote down why.
//!
//! # THE CLASS THIS EXISTS FOR
//!
//! Module 15 was assessed but never taught. Three gates had, over time, written
//! that defect down as a RULE — the hub must NOT link to module 15, module > 14
//! needs no Learn page, module 15 keys are "unexpected" — so the correct fix
//! failed three gates for being correct. The defect was never "someone
//! hardcoded 14"; it was that the assertion came from OBSERVED STATE instead of
//! from a stated contract. A gate rebased onto a registry can still be wrong,
//! but it is wrong in a way the registry can correct.
//!
//! # WHAT THIS TEST CANNOT DECIDE
//!
//! The inventory is a text scan. It cannot tell a live bound from one quoted in
//! a docstring — it only insists that every match was looked at once and given
//! a verdict — and it will not see a module bound spelled some way the patterns
//! below do not match (a named constant, a value read from config, arithmetic).
//! It says nothing about whether the registries themselves are right: if
//! `domains.toml` omits a module the course teaches, every gate downstream of it
//! is confidently wrong together, and no assertion here would notice.

use std::path::{Path, PathBuf};
use std::process::Command;

fn engine_root() -> PathBuf {
    cdcp_gate::root::resolve(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
}

#[derive(Debug)]
struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

fn capture(cmd: &mut Command) -> Run {
    let out = cmd.output().unwrap_or_else(|e| panic!("spawn failed: {e}"));
    Run {
        code: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

// ── 1. the inventory of numeric module bounds ──────────────────────────────

/// Why a matched bound is allowed to stay in the tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Verdict {
    /// Not a bound in effect: prose, a docstring, or a comment recording the
    /// history of a bound that has already been removed.
    Prose,
    /// A live comparison that is not a module bound at all — a report
    /// truncation, a loop counter, a term count, a float exponent.
    NotAModuleBound,
    /// A live module bound that is deliberately kept, with the reason.
    Justified,
    /// A live module bound that is a genuine bd-lt7 instance in a file this
    /// bead's owner does not own. Recorded so the sweep is honest about what it
    /// found rather than reporting a clean tree.
    OpenInstanceNotMine,
}

/// `(file, exact trimmed source line, verdict, reason)`.
///
/// Keyed by text and not by line number so that moving code does not fail this
/// test, while introducing a NEW bound anywhere does.
const INVENTORY: &[(&str, &str, Verdict, &str)] = &[
    // ── the two gates rebased by bd-lt7: only history remains ──────────────
    (
        "scripts/verify_coverage.py",
        "## Why the derivation, and not `range(1, 15)` (bd-lt7)",
        Verdict::Prose,
        "docstring heading recording the bound this gate no longer has",
    ),
    (
        "scripts/verify_coverage.py",
        "Until 2026-08-14 this gate read `PRIMARY_MODULES = range(1, 15)` and module 15",
        Verdict::Prose,
        "docstring recording the removed bound",
    ),
    (
        "scripts/build_units.py",
        "## Why the derivation, and not `int(m[:2]) <= 14` (bd-lt7)",
        Verdict::Prose,
        "docstring heading recording the bound this gate no longer has",
    ),
    (
        "scripts/build_units.py",
        "<= 14]`, which held module 15 out of the floor. Module 15 was, at that time,",
        Verdict::Prose,
        "docstring recording the removed bound",
    ),
    (
        "scripts/build_units.py",
        "# `int(m[:2]) <= 14` here silently exempted module 15 from this floor.",
        Verdict::Prose,
        "comment at the rebased site, naming what used to be there",
    ),
    // ── open bd-lt7 instances in files this bead's owner does not own ──────
    (
        "scripts/verify_objectives.py",
        "PRIMARY_MODULES = range(1, 15)  # 1–14 inclusive",
        Verdict::OpenInstanceNotMine,
        "same defect, same shape: the L7 objectives gate excludes module 15 from \
         its primary-domain set by literal. Needs the domains.toml derivation.",
    ),
    (
        "scripts/verify_objectives.py",
        "if domains_listed < 14:",
        Verdict::OpenInstanceNotMine,
        "asserts domains.toml lists 14 primary domains — a count observed before \
         module 15 shipped, now a soft warning that can never be satisfied \
         honestly. Derive the expected count from the registry.",
    ),
    (
        "scripts/smoke_hub_mastery.mjs",
        "assert(MODULE_CATALOG.length === 14, \"MODULE_CATALOG has 14 modules\");",
        Verdict::OpenInstanceNotMine,
        "the web hub's module catalog is pinned at 14, so module 15 cannot be \
         surfaced by the hub at all. This is the product-facing leg of the same \
         defect and is the most consequential remaining instance.",
    ),
    (
        "scripts/smoke_hub_mastery.mjs",
        "for (let m = 1; m <= 14; m++) {",
        Verdict::OpenInstanceNotMine,
        "mastery sweep skips module 15; paired with the clamp on the line below.",
    ),
    (
        "scripts/smoke_feedback_links.py",
        "for n in range(1, 15):",
        Verdict::OpenInstanceNotMine,
        "report-only: prints M01..M14 Learn links, silently omitting module 15. \
         Cosmetic, but a leftover in a file otherwise rebased on 2026-08-14.",
    ),
    // ── live comparisons that are not module bounds ────────────────────────
    (
        "scripts/build_glossary_json.py",
        "if len(terms) < 15:",
        Verdict::NotAModuleBound,
        "glossary term-count floor",
    ),
    (
        "scripts/smoke_learn_v2.py",
        "if (g.get(\"term_count\") or 0) < 15:",
        Verdict::NotAModuleBound,
        "glossary term-count floor",
    ),
    (
        "scripts/smoke_learn_v2.py",
        "fail(\"glossary term_count < 15\")",
        Verdict::NotAModuleBound,
        "message text for the term-count floor",
    ),
    (
        "scripts/smoke_srs.mjs",
        "for (let i = 0; i < 15; i++) {",
        Verdict::NotAModuleBound,
        "loop counter over a synthetic SRS queue",
    ),
    (
        "scripts/smoke_feedback_links.py",
        "if len(missing_module) > 15:",
        Verdict::NotAModuleBound,
        "report truncation after 15 failure lines",
    ),
    (
        "scripts/smoke_feedback_links.py",
        "if len(unmapped_modules) > 15:",
        Verdict::NotAModuleBound,
        "report truncation after 15 failure lines",
    ),
    (
        "crates/cdcp_gate/src/gates/verify_bank.rs",
        "let body = if decpt <= -4 || decpt > 16 {",
        Verdict::NotAModuleBound,
        "CPython float repr: the exponent-notation cutoff",
    ),
    (
        "crates/cdcp_gate/tests/diff_verify_content_lock.rs",
        "assert!(removed >= 15, \"emptied only {removed} files\");",
        Verdict::NotAModuleBound,
        "fixture file count in an unrelated differential case",
    ),
    // ── live module bounds deliberately kept ──────────────────────────────
    (
        "crates/cdcp_gate/tests/diff_verify_knowledge_paths.rs",
        "for bound in [\"range(1, 15)\", \"range(1,15)\", \"<= 14\", \"< 15\"] {",
        Verdict::Justified,
        "a sibling gate's own detector for these bounds — the literals are the \
         thing being searched for, not a bound in effect",
    ),
    (
        "crates/cdcp_gate/src/gates/verify_knowledge_paths.rs",
        "//! bd-lt7 tracks gates that hardcode a module bound (`range(1, 15)`, `<= 14`) and",
        Verdict::Prose,
        "module header cross-referencing this bead",
    ),
    (
        "crates/cdcp_assemble/tests/learn_surface_coverage.rs",
        "compared >= 14,",
        Verdict::Justified,
        "a FLOOR, not an exclusion: ≥14 modules must be compared, which 15 \
         satisfies. It cannot hold a module out; it can only notice a collapse.",
    ),
    (
        "crates/cdcp_assemble/tests/learn_surface_coverage.rs",
        "let m15 = rows.iter().find(|r| r.order == 15);",
        Verdict::Justified,
        "the C5 decision stated as an assertion: module 15 specifically must be \
         taught because it is assessed. Naming the module IS the point here.",
    ),
];

/// The bound shapes this sweep can see. Deliberately narrow: comparisons and
/// ranges against 13–16, which is where a "modules 1..14" assumption lands.
fn bound_hits(line: &str) -> bool {
    let l = line.replace(' ', "");
    for n in ["13", "14", "15", "16"] {
        for op in ["<=", ">=", "===", "!==", "==", "!=", "<", ">"] {
            if let Some(rest) = find_after(&l, &format!("{op}{n}")) {
                if !rest.starts_with(|c: char| c.is_ascii_digit()) {
                    return true;
                }
            }
        }
        if l.contains(&format!("range(1,{n})")) {
            return true;
        }
    }
    false
}

fn find_after<'a>(hay: &'a str, needle: &str) -> Option<&'a str> {
    hay.find(needle).map(|i| &hay[i + needle.len()..])
}

fn scan_files(dir: &Path, exts: &[&str], out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        let name = p.file_name().map(|n| n.to_string_lossy().into_owned());
        if p.is_dir() {
            if matches!(name.as_deref(), Some("target") | Some("__pycache__")) {
                continue;
            }
            scan_files(&p, exts, out);
        } else if p
            .extension()
            .and_then(|s| s.to_str())
            .map(|e| exts.contains(&e))
            .unwrap_or(false)
        {
            out.push(p);
        }
    }
}

#[test]
fn every_numeric_module_bound_in_the_tree_is_inventoried() {
    let root = engine_root();
    let mut files = Vec::new();
    scan_files(
        &root.join("scripts"),
        &["py", "mjs", "js", "sh"],
        &mut files,
    );
    scan_files(&root.join("crates"), &["rs"], &mut files);
    // Anti-vacuous: an empty scan set is an ERROR. A sweep that read nothing
    // reports exactly like one that read everything and found it clean.
    assert!(
        files.len() >= 40,
        "scanned only {} files — a vacuous bound sweep is an ERROR",
        files.len()
    );
    // This test file's own INVENTORY quotes every bound in the tree, so it
    // would match itself on every row. It is the ledger, not a subject.
    files.retain(|p| !p.ends_with("rebase_module_bounds.rs"));

    let mut unexpected: Vec<String> = Vec::new();
    let mut matched: Vec<(String, String)> = Vec::new();
    for f in &files {
        let rel = f
            .strip_prefix(&root)
            .unwrap_or(f)
            .to_string_lossy()
            .into_owned();
        let Ok(text) = std::fs::read_to_string(f) else {
            continue;
        };
        for (i, line) in text.lines().enumerate() {
            if !bound_hits(line) {
                continue;
            }
            let trimmed = line.trim().to_string();
            if INVENTORY
                .iter()
                .any(|(file, src, _, _)| *file == rel && *src == trimmed)
            {
                matched.push((rel.clone(), trimmed));
            } else {
                unexpected.push(format!("{rel}:{}: {trimmed}", i + 1));
            }
        }
    }

    assert!(
        unexpected.is_empty(),
        "{} numeric module bound(s) are not in the INVENTORY in this file. Add a \
         row with a verdict and a reason, or derive the value from a registry:\n  {}",
        unexpected.len(),
        unexpected.join("\n  ")
    );
    // Anti-vacuous again, from the other side: if the detector stopped matching
    // anything, this test would pass while checking nothing.
    assert!(
        matched.len() >= INVENTORY.len(),
        "the detector found {} bounds but the inventory lists {} — a row whose \
         line no longer exists means the detector or the inventory has drifted",
        matched.len(),
        INVENTORY.len()
    );
}

/// The sweep found open instances, and says so out loud rather than reporting a
/// clean tree. Reducing this list is the follow-on work; it may never grow
/// silently.
#[test]
fn the_open_instances_are_named_and_counted() {
    let open: Vec<&str> = INVENTORY
        .iter()
        .filter(|(_, _, v, _)| *v == Verdict::OpenInstanceNotMine)
        .map(|(f, _, _, _)| *f)
        .collect();
    assert_eq!(
        open.len(),
        5,
        "open bd-lt7 instances changed: {open:?} — update this count deliberately"
    );
}

// ── 2. verify_coverage.py: known-bad and known-GOOD ───────────────────────

struct Fixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_path_buf();
        Fixture { _dir: dir, root }
    }
    fn write(&self, rel: &str, body: &str) {
        let p = self.root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    }
    fn path(&self, rel: &str) -> PathBuf {
        self.root.join(rel)
    }
}

/// One bank item, as a single-item TOML file.
fn item(id: &str, module: u32, topic: &str) -> String {
    format!(
        "id = {id:?}\nmodule = {module}\nstem = \"stem for {id}\"\n\
         choices = [\"alpha\", \"beta\", \"gamma\", \"delta\"]\ncorrect = \"A\"\n\
         explanation = \"an explanation of sufficient length\"\ntopic_ids = [{topic:?}]\n\
         bloom = \"apply\"\nsource_class = \"original\"\n\
         quantity_evidence = \"qualitative_only\"\n"
    )
}

fn domains_registry(orders: &[u32]) -> String {
    let mut s = String::from("schema_version = 1\n");
    for o in orders {
        s.push_str(&format!(
            "\n[[domain]]\nid = \"{o:02}-fixture\"\norder = {o}\n\
             epi_heading = \"Fixture domain {o}\"\n"
        ));
    }
    s
}

fn coverage(f: &Fixture, bank: &str, domains: &str, policy: Option<&str>) -> Run {
    let root = engine_root();
    let mut cmd = Command::new("python3");
    cmd.arg(root.join("scripts/verify_coverage.py"))
        .arg("--bank")
        .arg(f.path(bank))
        .arg("--domains")
        .arg(f.path(domains));
    if let Some(p) = policy {
        cmd.arg("--policy").arg(f.path(p));
    }
    capture(&mut cmd)
}

#[test]
fn python3_is_present_because_a_skipped_leg_is_a_fooled_certificate() {
    let out = Command::new("python3")
        .arg("--version")
        .output()
        .expect("python3 must be installed: these legs cannot be skipped");
    assert!(out.status.success());
}

/// Known-GOOD. The live tree passes, and module 15 is now INSIDE the required
/// set rather than listed as an optional extra.
#[test]
fn verify_coverage_known_good_the_live_tree_passes_with_module_15_required() {
    let root = engine_root();
    let run = capture(Command::new("python3").arg(root.join("scripts/verify_coverage.py")));
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    assert!(
        run.stdout
            .contains("modules (15 required, derived from the domain registry)"),
        "the required set must be derived, and must include module 15:\n{}",
        run.stdout
    );
    assert!(
        run.stdout.contains("    m15: "),
        "module 15 must be listed among the required modules:\n{}",
        run.stdout
    );
    assert!(
        !run.stdout.contains("(optional)"),
        "module 15 must no longer be reported as an optional extra:\n{}",
        run.stdout
    );
}

/// Known-BAD, the bd-lt7 regression itself. A registry that declares module 15
/// while the bank holds nothing for it must go RED. Under `range(1, 15)` this
/// tree was GREEN.
#[test]
fn verify_coverage_known_bad_a_declared_module_with_no_items_trips() {
    let f = Fixture::new();
    f.write("domains.toml", &domains_registry(&[1, 2, 15]));
    for m in [1u32, 2] {
        f.write(
            &format!("bank/m{m:02}.toml"),
            &item(&format!("i-{m}"), m, "t"),
        );
    }
    let run = coverage(&f, "bank", "domains.toml", None);
    assert_ne!(
        run.code, 0,
        "a starved declared module passed:\n{}",
        run.stdout
    );
    assert!(
        run.stdout.contains("module 15: 0 items < min 1"),
        "the finding must name the module:\n{}",
        run.stdout
    );
}

/// Known-BAD. Anti-vacuous: a registry that declares nothing is an ERROR, not a
/// green run over an empty required set.
#[test]
fn verify_coverage_known_bad_an_empty_registry_is_an_error() {
    let f = Fixture::new();
    f.write("domains.toml", "schema_version = 1\n");
    f.write("bank/m01.toml", &item("i-1", 1, "t"));
    let run = coverage(&f, "bank", "domains.toml", None);
    assert_ne!(run.code, 0, "{}", run.stdout);
    assert!(
        run.stdout
            .contains("domain registry declares zero modules (vacuous coverage is ERROR)"),
        "{}",
        run.stdout
    );
    assert!(
        run.stdout
            .contains("zero required modules after exemptions"),
        "{}",
        run.stdout
    );
}

/// Known-BAD. A missing registry is an ERROR, not a silent skip.
#[test]
fn verify_coverage_known_bad_a_missing_registry_is_an_error() {
    let f = Fixture::new();
    f.write("bank/m01.toml", &item("i-1", 1, "t"));
    let run = coverage(&f, "bank", "domains.toml", None);
    assert_ne!(run.code, 0, "{}", run.stdout);
    assert!(
        run.stdout.contains("domain registry missing"),
        "{}",
        run.stdout
    );
}

/// Known-BAD. The escape hatch may not be quieter than the rule: an exemption
/// without a reason is a schema error, not an exemption.
#[test]
fn verify_coverage_known_bad_an_exemption_without_a_reason_is_an_error() {
    let f = Fixture::new();
    f.write("domains.toml", &domains_registry(&[1, 15]));
    f.write("policy.toml", "[[coverage_exempt]]\nmodule = 15\n");
    f.write("bank/m01.toml", &item("i-1", 1, "t"));
    let run = coverage(&f, "bank", "domains.toml", Some("policy.toml"));
    assert_ne!(run.code, 0, "{}", run.stdout);
    assert!(
        run.stdout.contains(
            "coverage_exempt module 15 has no reason (an exemption without a reason is a \
             schema error)"
        ),
        "{}",
        run.stdout
    );
    // And the module stays required, so the shortfall is still reported.
    assert!(
        run.stdout.contains("module 15: 0 items < min 1"),
        "a rejected exemption must not silently hold the module out:\n{}",
        run.stdout
    );
}

/// Known-GOOD, the escape hatch working. A RECORDED exemption with a reason
/// holds a module out of the floor and is printed, so the hole is visible.
#[test]
fn verify_coverage_known_good_a_recorded_exemption_with_a_reason_is_honoured() {
    let f = Fixture::new();
    f.write("domains.toml", &domains_registry(&[1, 15]));
    f.write(
        "policy.toml",
        "[[coverage_exempt]]\nmodule = 15\nreason = \"fixture: not yet authored\"\n",
    );
    f.write("bank/m01.toml", &item("i-1", 1, "t"));
    let run = coverage(&f, "bank", "domains.toml", Some("policy.toml"));
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    assert!(
        run.stdout
            .contains("m15: 0 — exempt: fixture: not yet authored"),
        "an exemption must be printed, not silent:\n{}",
        run.stdout
    );
}

/// Known-BAD. The two registries disagreeing about which modules exist is the
/// drift that produced this bead in the first place.
#[test]
fn verify_coverage_known_bad_a_floor_for_an_undeclared_module_is_drift() {
    let f = Fixture::new();
    f.write("domains.toml", &domains_registry(&[1]));
    f.write(
        "policy.toml",
        "[[domain_min]]\nmodule = 1\nmin_items = 1\n\
         [[domain_min]]\nmodule = 15\nmin_items = 16\n",
    );
    f.write("bank/m01.toml", &item("i-1", 1, "t"));
    let run = coverage(&f, "bank", "domains.toml", Some("policy.toml"));
    assert_ne!(run.code, 0, "{}", run.stdout);
    assert!(
        run.stdout
            .contains("[[domain_min]] module 15 is not a required module"),
        "{}",
        run.stdout
    );
}

// ── 3. build_units.py: known-bad and known-GOOD ───────────────────────────

/// A markdown module with `n` `##` units, each long enough to survive the
/// 40-word floor in `split_h2_units`.
fn module_markdown(title: &str, n: usize) -> String {
    // `split_h2_units` drops any section under forty words, so the filler has
    // to clear that floor or the fixture silently builds zero units and every
    // leg below would be measuring an empty tree instead of the rule.
    let filler = "This unit describes the fixture content in enough words that the \
                  unit survives the forty word floor applied by the splitter, which \
                  drops any section shorter than that unless it names objectives. \
                  The prose here carries no meaning of its own and exists only to \
                  push the counted word total past the threshold that the splitter \
                  applies to every section it emits, so that the fixture measures \
                  the check floor rather than the word floor."
        .to_string();
    let mut s = format!("# {title}\n\n");
    for i in 1..=n {
        s.push_str(&format!("## Unit {i} of {title}\n\n{filler}\n\n"));
    }
    s
}

/// A tree `build_units.py` can run in: its own copy of the script, a Learn
/// index, module markdown, and a bank.
fn units_fixture(modules: &[(&str, usize)], bank_modules: &[u32]) -> Fixture {
    let f = Fixture::new();
    std::fs::create_dir_all(f.path("scripts")).unwrap();
    std::fs::copy(
        engine_root().join("scripts/build_units.py"),
        f.path("scripts/build_units.py"),
    )
    .expect("copy build_units.py into the fixture");
    f.write("knowledge/topics.toml", "# no topics in this fixture\n");

    let rows: Vec<String> = modules
        .iter()
        .enumerate()
        .map(|(i, (id, _))| format!("{{\"id\": {id:?}, \"order\": {}, \"empty\": false}}", i + 1))
        .collect();
    f.write(
        "web/data/modules_index.json",
        &format!("{{\"modules\": [{}]}}\n", rows.join(", ")),
    );
    for (id, units) in modules {
        f.write(
            &format!("web/content/modules/{id}.md"),
            &module_markdown(id, *units),
        );
    }
    for m in bank_modules {
        for i in 0..6 {
            f.write(
                &format!("bank/items/m{m:02}-{i}.toml"),
                &item(&format!("i-{m:02}-{i}"), *m, "t"),
            );
        }
    }
    f
}

fn build_units(f: &Fixture) -> Run {
    capture(Command::new("python3").arg(f.path("scripts/build_units.py")))
}

/// Known-GOOD. The live tree passes, and its check floor now covers all 15
/// modules rather than the 14 that `int(m[:2]) <= 14` admitted.
#[test]
fn build_units_known_good_the_live_tree_passes_over_all_declared_modules() {
    let root = engine_root();
    let run = capture(Command::new("python3").arg(root.join("scripts/build_units.py")));
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    assert!(
        run.stdout.starts_with("PASS: build_units"),
        "{}",
        run.stdout
    );
    assert!(
        run.stdout.contains("15-ops-adjacent"),
        "module 15 must be inside the check floor, not exempt from it:\n{}",
        run.stdout
    );
}

/// Known-GOOD, on a fixture: a tree where every module has bank items passes.
#[test]
fn build_units_known_good_a_well_stocked_fixture_passes() {
    let f = units_fixture(
        &[("01-mission-critical", 4), ("06-power", 3), ("15-ops", 3)],
        &[1, 6, 15],
    );
    let run = build_units(&f);
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    assert!(
        run.stdout.starts_with("PASS: build_units"),
        "{}",
        run.stdout
    );
}

/// Known-BAD, the bd-lt7 regression itself. Module 15 in the Learn index with
/// no bank items behind it must go RED. Under `int(m[:2]) <= 14` its units were
/// excluded from the denominator and this tree was GREEN.
#[test]
fn build_units_known_bad_a_starved_module_15_now_trips_the_check_floor() {
    let f = units_fixture(
        &[("01-mission-critical", 4), ("06-power", 3), ("15-ops", 4)],
        &[1, 6], // nothing for module 15
    );
    let run = build_units(&f);
    assert_ne!(
        run.code, 0,
        "a starved module 15 passed the check floor:\n{}",
        run.stdout
    );
    assert!(
        run.stdout.contains("have ≥2 checks"),
        "the finding must name the check floor:\n{}",
        run.stdout
    );
    // The verdict is the head of the report, and it is not a PASS.
    assert!(
        run.stdout.starts_with("FAIL: build_units"),
        "verdict must lead the report and must be FAIL:\n{}",
        run.stdout
    );
    assert!(
        !run.stdout.contains("PASS"),
        "no PASS may appear anywhere on a failing run:\n{}",
        run.stdout
    );
}

/// Known-BAD. Anti-vacuous: no modules at all is an ERROR, not a green run over
/// an empty set.
#[test]
fn build_units_known_bad_zero_modules_is_an_error() {
    let f = units_fixture(&[], &[1]);
    std::fs::create_dir_all(f.path("web/content/modules")).unwrap();
    let run = build_units(&f);
    assert_ne!(run.code, 0, "{}", run.stdout);
    assert!(
        run.stdout
            .contains("zero modules discovered (vacuous unit build is ERROR)"),
        "{}",
        run.stdout
    );
    assert!(
        run.stdout.starts_with("FAIL: build_units"),
        "{}",
        run.stdout
    );
}

/// bd-hw3's shape rule applied to this gate: it used to print
/// "PASS: build_units …" and then emit "FAIL: …" underneath on its way to
/// returning 1. stdout and CI must never disagree.
#[test]
fn build_units_never_writes_a_verdict_it_then_contradicts() {
    let f = units_fixture(&[("01-mission-critical", 2), ("06-power", 3)], &[1, 6]);
    let run = build_units(&f);
    assert_ne!(run.code, 0, "{}", run.stdout);
    assert!(
        run.stdout
            .contains("01-mission-critical has 2 units, need ≥4"),
        "{}",
        run.stdout
    );
    let first = run.stdout.lines().next().unwrap_or("");
    assert!(
        first.starts_with("FAIL: build_units"),
        "the first line written was {first:?}"
    );
    assert!(run.stderr.is_empty(), "{}", run.stderr);
}
