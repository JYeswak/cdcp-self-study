//! bd-lt7 — gates must not encode a known defect as an invariant.
//!
//! # CLAIM: FLOOR-RAISE
//!
//! Three things are held here, and nothing more:
//!
//!   1. `scripts/verify_coverage.py`, `scripts/build_units.py` and
//!      `scripts/smoke_feedback_links.py` derive their module set from a
//!      registry (`knowledge/domains.toml`, `web/data/modules_index.json`, and
//!      `knowledge/domains.toml` again) rather than from a numeric bound, and
//!      each still trips on a real defect after the rebase.
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
    // ── bd-lt7: ZERO open instances as of 2026-08-14 ──────────────────────
    //
    // All five OpenInstanceNotMine rows that lived here are gone: two agents
    // closed verify_objectives.py, smoke_feedback_links.py and
    // smoke_hub_mastery.mjs in the same wave. Every bound below is now PROSE in
    // a docstring recording what was removed and why — which is exactly the
    // shape this inventory is for, because a grep for the old literal still
    // hits and a reader must be able to tell documentation from live code.
    // (The controller grepped these five and briefly read them as unfixed.)
    (
        "scripts/verify_objectives.py",
        "## Why the derivation, and not `range(1, 15)` (bd-lt7)",
        Verdict::Prose,
        "docstring heading recording the removed PRIMARY_MODULES literal",
    ),
    (
        "scripts/verify_objectives.py",
        "Until 2026-08-14 this gate read `PRIMARY_MODULES = range(1, 15)` and skipped",
        Verdict::Prose,
        "docstring recording the removed literal and the domains.toml derivation",
    ),
    (
        "scripts/verify_objectives.py",
        "The old `domains_listed < 14` soft warning went with it. It was a FLOOR, so it",
        Verdict::Prose,
        "docstring recording the removed soft warning and WHY removing it was safe: \
         it was a floor whose comparand was the same observed count, so once the \
         module set derives from domains.toml the check compares the registry \
         against itself",
    ),
    (
        "scripts/smoke_feedback_links.py",
        "`for n in range(1, 15)` report loop. The table happened to be right; the loop",
        Verdict::Prose,
        "docstring recording the removed report-loop bound",
    ),
    // The suite that now ASSERTS the verify_objectives rebase has to name the
    // literal it protects against, so the sweep sees it too. Both rows are
    // header prose in the selftest, not a bound in effect anywhere.
    (
        "scripts/selftest_l7_objectives.sh",
        "# knowledge/domains.toml instead of `range(1, 15)`. (e) is the regression",
        Verdict::Prose,
        "selftest header naming the removed literal its case (e) guards against",
    ),
    (
        "scripts/selftest_l7_objectives.sh",
        "# THE bd-lt7 regression. Under `PRIMARY_MODULES = range(1, 15)` this exact",
        Verdict::Prose,
        "comment at case (e), recording the bound under which that exact fixture \
         tree was GREEN",
    ),
    (
        "scripts/smoke_hub_mastery.mjs",
        "* Until 2026-08-14 this gate asserted `MODULE_CATALOG.length === 14` and swept",
        Verdict::Prose,
        "docstring recording the removed catalog-length bound",
    ),
    (
        "scripts/smoke_hub_mastery.mjs",
        "* modules with two `m <= 14` loops. Module 15 is assessed AND taught, so those",
        Verdict::Prose,
        "docstring recording the two removed sweep bounds",
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
        0,
        "open bd-lt7 instances changed: {open:?} — update this count deliberately. \
         It reached 0 on 2026-08-14; a NEW hardcoded module bound anywhere in the \
         tree must be inventoried with a verdict, not silently added here."
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

// ── 4. smoke_feedback_links.py: known-bad and known-GOOD ──────────────────
//
// This gate takes NO path flags — every input is `Path(__file__).parents[1] /
// …` — so the only way to inject a known-bad into it is to give it a whole
// tree of its own. That is what `feedback_fixture` builds. The alternative
// considered and rejected was adding `--domains` / `--root` to the script:
// the gate's Python is correct, and widening its argument surface to make it
// testable would be changing the thing under test in order to test it.
//
// The fixture also has to exist for a second reason. The script WRITES
// `web/data/topic_anchors.json` on every run; pointed at the live tree it
// would dirty the working copy, and a leg that dirties the tree cannot be a
// CI leg. Inside the fixture the write lands on the copy.

/// A tree `smoke_feedback_links.py` can run in: a copy of the script and of
/// every input it resolves off its own location. Copied, not synthesised —
/// results.js, the Learn pages and the seed42 packs are the real product
/// surfaces, and a hand-written stand-in would let this leg pass while the
/// shipped ones diverged.
fn feedback_fixture() -> Fixture {
    let root = engine_root();
    let f = Fixture::new();

    let copy = |rel: &str| {
        let dst = f.path(rel);
        std::fs::create_dir_all(dst.parent().unwrap()).unwrap();
        std::fs::copy(root.join(rel), &dst)
            .unwrap_or_else(|e| panic!("copy {rel} into the fixture: {e}"));
    };
    // The gate, and the module it imports at runtime to rebuild topic anchors.
    copy("scripts/smoke_feedback_links.py");
    copy("scripts/build_learn.py");
    // The registry under test, and the topic registry the anchor builder reads.
    copy("knowledge/domains.toml");
    copy("knowledge/topics.toml");
    // The product surfaces the gate checks the registry against.
    copy("web/assets/js/results.js");
    copy("web/assets/js/learn_md.js");
    copy("web/data/keys_seed42.json");
    copy("web/data/bank_items_seed42.json");
    // A pre-existing anchor map, so a fixture in which the rebuild cannot run
    // degrades to the gate's documented fallback instead of to a spurious
    // failure that would look like the injection firing.
    copy("web/data/topic_anchors.json");

    let mut copied = 0usize;
    for (dir, ext) in [("web/learn", "html"), ("web/content/modules", "md")] {
        std::fs::create_dir_all(f.path(dir)).unwrap();
        for e in std::fs::read_dir(root.join(dir)).unwrap().flatten() {
            let p = e.path();
            if p.extension().and_then(|s| s.to_str()) == Some(ext) {
                let name = p.file_name().unwrap().to_string_lossy().into_owned();
                std::fs::copy(&p, f.path(&format!("{dir}/{name}"))).unwrap();
                copied += 1;
            }
        }
    }
    // Anti-vacuous: a fixture that copied no Learn surface would make every
    // "missing learn page" finding below fire for the wrong reason.
    assert!(
        copied >= 30,
        "fixture copied only {copied} Learn/content files — a vacuous fixture \
         would make every injection below fire for the wrong reason"
    );
    f
}

fn feedback_links(f: &Fixture) -> Run {
    capture(Command::new("python3").arg(f.path("scripts/smoke_feedback_links.py")))
}

/// Rewrite the fixture's domain registry, dropping every `[[domain]]` block
/// whose `order` is in `drop`. Text surgery rather than a TOML round-trip
/// because the point is to change ONE fact and leave the file otherwise as
/// shipped.
fn drop_domains(f: &Fixture, drop: &[u32]) {
    let path = f.path("knowledge/domains.toml");
    let text = std::fs::read_to_string(&path).unwrap();
    let mut out = String::new();
    let mut removed = 0usize;
    for (i, chunk) in text.split("[[domain]]").enumerate() {
        if i == 0 {
            out.push_str(chunk);
            continue;
        }
        if drop
            .iter()
            .any(|o| chunk.lines().any(|l| l.trim() == format!("order = {o}")))
        {
            removed += 1;
            continue;
        }
        out.push_str("[[domain]]");
        out.push_str(chunk);
    }
    assert_eq!(
        removed,
        drop.len(),
        "the fixture registry did not contain every module this case removes — \
         the injection would not have applied"
    );
    std::fs::write(path, out).unwrap();
}

/// Known-GOOD. The shipped tree passes, and module 15 is INSIDE the swept set:
/// under `for n in range(1, 15)` the report printed M01–M14 and module 15 was
/// simply absent from the surface this gate describes.
#[test]
fn feedback_links_known_good_the_shipped_tree_passes_with_module_15_reported() {
    let f = feedback_fixture();
    let run = feedback_links(&f);
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    assert!(
        run.stdout.starts_with("PASS: smoke_feedback_links"),
        "{}",
        run.stdout
    );
    assert!(
        run.stdout
            .contains("modules=15 (derived from knowledge/domains.toml)"),
        "the module count must be derived, and must be 15:\n{}",
        run.stdout
    );
    assert!(
        run.stdout.contains("M15 → learn/15-ops-adjacent.html"),
        "module 15 must appear in the report loop, not stop at M14:\n{}",
        run.stdout
    );
    assert!(
        run.stdout.contains("untaught_module_items=0 (must be 0)"),
        "{}",
        run.stdout
    );
}

/// Known-BAD, the bd-lt7 regression from the product side. Retire module 15
/// from the registry while `results.js` still links it and the bank still
/// assesses it: the gate must report the drift in BOTH directions — a Learn
/// link for a module nobody declares, and items on a real form whose module has
/// no Learn surface.
#[test]
fn feedback_links_known_bad_a_module_the_registry_stops_declaring_is_drift() {
    let f = feedback_fixture();
    drop_domains(&f, &[15]);
    let run = feedback_links(&f);
    assert_ne!(
        run.code, 0,
        "a Learn link for an undeclared module passed:\n{}",
        run.stdout
    );
    assert!(
        run.stdout.contains(
            "module 15: results.js maps '15-ops-adjacent' but knowledge/domains.toml \
             does not declare that module"
        ),
        "the product→registry direction must name the module:\n{}",
        run.stdout
    );
    assert!(
        run.stdout.contains("assessed but untaught: ")
            && run
                .stdout
                .contains("module 15 is not declared in knowledge/domains.toml"),
        "an item on a real form with no Learn surface is the C5 defect and must \
         be named, never a silently skipped row:\n{}",
        run.stdout
    );
    assert!(
        run.stdout.starts_with("FAIL: smoke_feedback_links"),
        "the verdict must lead the report:\n{}",
        run.stdout
    );
    assert!(
        !run.stdout.contains("PASS"),
        "no PASS may appear anywhere on a failing run:\n{}",
        run.stdout
    );
}

/// Known-BAD, the registry→product direction. A module the registry declares
/// with no Learn surface behind it must go RED naming the module. `range(1,
/// 15)` could not have seen this for module 15 at all.
#[test]
fn feedback_links_known_bad_a_declared_module_with_no_learn_surface_trips() {
    let f = feedback_fixture();
    let path = f.path("knowledge/domains.toml");
    let mut text = std::fs::read_to_string(&path).unwrap();
    text.push_str(
        "\n[[domain]]\nid = \"16-fixture-only\"\norder = 16\n\
         epi_heading = \"Fixture domain with no Learn surface\"\n\
         exam_weight_unknown = true\n",
    );
    std::fs::write(&path, text).unwrap();

    let run = feedback_links(&f);
    assert_ne!(run.code, 0, "{}", run.stdout);
    assert!(
        run.stdout.contains(
            "module 16: results.js slug map None != '16-fixture-only' \
             (knowledge/domains.toml)"
        ),
        "the slug-map gap must name the module:\n{}",
        run.stdout
    );
    assert!(
        run.stdout
            .contains("missing learn page web/learn/16-fixture-only.html"),
        "the missing Learn page must be named:\n{}",
        run.stdout
    );
    assert!(
        run.stdout
            .contains("missing content web/content/modules/16-fixture-only.md"),
        "the missing content file must be named:\n{}",
        run.stdout
    );
}

/// Known-BAD. Anti-vacuous: a registry that declares nothing is an ERROR, not a
/// green run over an empty module set. This is the failure that reports exactly
/// like a clean one if nobody writes the check.
#[test]
fn feedback_links_known_bad_an_empty_registry_is_an_error() {
    let f = feedback_fixture();
    std::fs::write(f.path("knowledge/domains.toml"), "schema_version = 1\n").unwrap();
    let run = feedback_links(&f);
    assert_ne!(run.code, 0, "{}", run.stdout);
    assert!(
        run.stdout
            .contains("domain registry declares zero modules (vacuous link check is ERROR)"),
        "{}",
        run.stdout
    );
    assert!(
        run.stdout.starts_with("FAIL: smoke_feedback_links"),
        "{}",
        run.stdout
    );
}

/// Known-GOOD, the other half of the drift rule. Retiring a module from BOTH
/// sources at once is a legitimate edit, not drift, and must stay green — an
/// attack-only suite would make the registry uneditable.
#[test]
fn feedback_links_known_good_retiring_a_module_from_both_sources_is_not_drift() {
    let f = feedback_fixture();
    // 14 is chosen over 15 only because the bank has items in both; the point
    // is that the two sources agree after the edit, whichever module it is.
    drop_domains(&f, &[14]);
    let js_path = f.path("web/assets/js/results.js");
    let js = std::fs::read_to_string(&js_path).unwrap();
    let stripped: String = js
        .lines()
        .filter(|l| !l.contains("14: '14-auxiliary'") && !l.contains("14: \"14-auxiliary\""))
        .collect::<Vec<_>>()
        .join("\n");
    assert_ne!(
        stripped.len(),
        js.len(),
        "the fixture's slug map did not contain module 14 on its own line — this \
         case would otherwise assert nothing"
    );
    std::fs::write(&js_path, stripped + "\n").unwrap();

    let run = feedback_links(&f);
    // The two sources now agree about module 14, so neither drift direction
    // fires. Items still assessed in module 14 are the C5 defect and are
    // REPORTED — this leg asserts only that the drift rules stayed quiet.
    assert!(
        !run.stdout.contains("module 14: results.js maps"),
        "an agreed retirement must not be reported as product→registry drift:\n{}",
        run.stdout
    );
    assert!(
        !run.stdout.contains("module 14: results.js slug map"),
        "an agreed retirement must not be reported as registry→product drift:\n{}",
        run.stdout
    );
}
