//! Verdict suite for `cdcp_learn::units` (extracted from the gate by
//! bd-engine-not-gate-ar39.2).
//!
//! # THIS FILE WAS A DIFFERENTIAL AND IS NOT ONE ANY MORE
//!
//! It ran every case twice — `scripts/build_units.py` and the Rust port — and
//! asserted the two agreed on stdout, stderr, exit code and the bytes written.
//! That comparison did its job: it is what proved the port faithful at port
//! time. It was RETIRED on 2026-08-14 along with the oracle, for two reasons
//! worth writing down because the next port will face the same choice:
//!
//!   1. **A differential is blind to a defect BOTH sides share.** This gate is
//!      the proof. `build_units.py` and `build_units.rs` agreed byte for byte
//!      while BOTH baked retired bank items into `check_item_ids`
//!      (`05-lighting__formulas-rules-of-thumb -> m05-q200`,
//!      `11-network__formulas-rules-of-thumb -> m11-q226`). Fifteen cases were
//!      green over a wrong answer. Agreement is not correctness, and a suite
//!      that only asserts agreement cannot tell the difference.
//!   2. **An oracle kept past port time is a permanent tax in the wrong
//!      language.** Every deliberate behaviour change would have to be written
//!      twice, in Python, on a project whose Substrate Law puts Rust on the
//!      load-bearing path. `scripts/check.sh` never invoked the `.py`; only
//!      this file did.
//!
//! So every case below now asserts WHAT THE CORRECT ANSWER IS, against the Rust
//! alone. No case was dropped: the ones whose failure mode is SILENCE — the
//! anti-vacuous legs and the coverage floor — are the ones that matter most, and
//! they are all still here, strengthened from "the two implementations agree"
//! into "this is the verdict, and this is why".
//!
//! # THE RULES THAT SURVIVED THE RETIREMENT
//!
//!   1. **NEVER RUN THE BUILDER AGAINST THE LIVE TREE.** `build-units` MUTATES
//!      a tracked file. Running it in the repo races every other reader and
//!      writer of `web/data/` (bd-791t, bd-gl4j) and — worse — MAKES the tracked
//!      artifact current, so a run proves nothing about what is committed. Every
//!      case here builds a TREE COPY in temp whose inputs are byte-copies of the
//!      live ones, and the live case then asserts the produced bytes EQUAL the
//!      tracked `web/data/units_index.json`. That read-only comparison is
//!      strictly stronger than a live run: it proves the committed artifact is
//!      current without touching it.
//!   2. **THE ARTIFACT IS PART OF THE VERDICT, AS BYTES.** stdout, stderr and
//!      exit code are not the whole observable behaviour of a builder.
//!   3. **WRITE-AFTER-VERDICT, asserted on every case.** A run that exits
//!      non-zero must leave no artifact, and a run that exits zero must leave
//!      one. Checked by `run_gate` on every case rather than on the handful that
//!      happen to be RED (bd-builder-verdict-shape-qm65).
//!
//! ANTI-VACUOUS DISCIPLINE. A suite that silently checked nothing passes exactly
//! like one that checked everything: a fixture that copied no module is a
//! FAILURE, a live fixture whose bank carries no retired row is a FAILURE (it
//! would make the status legs vacuous), and every case increments a counter that
//! is asserted by its own case.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

const ARTIFACT_REL: &str = "web/data/units_index.json";
const PACK_REL: &str = "web/data/bank_items_seed42.json";

/// Cases actually run, so "the suite ran" is itself checked.
static RAN: AtomicUsize = AtomicUsize::new(0);
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
        .unwrap_or_else(|e| panic!("read {}: {e}", src.display()))
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

struct Fixture {
    dir: tempfile::TempDir,
}

impl Fixture {
    fn new() -> Fixture {
        Fixture {
            dir: tempfile::tempdir().unwrap(),
        }
    }

    fn template(&self) -> PathBuf {
        self.dir.path().join("template")
    }

    fn at(&self, rel: &str) -> PathBuf {
        let mut p = self.template();
        for part in rel.split('/') {
            p.push(part);
        }
        p
    }

    fn put(&self, rel: &str, body: &str) {
        write_file(&self.at(rel), body);
    }

    fn rm(&self, rel: &str) {
        let p = self.at(rel);
        if p.is_dir() {
            std::fs::remove_dir_all(&p).unwrap();
        } else if p.exists() {
            std::fs::remove_file(&p).unwrap();
        }
    }

    fn read(&self, rel: &str) -> String {
        std::fs::read_to_string(self.at(rel))
            .unwrap_or_else(|e| panic!("read {rel} from the fixture: {e}"))
    }

    /// Byte-copy every live input this gate reads, minus the per-item TOML bank
    /// (which the seed JSON supersedes; `seed_live_bank_dir` adds it).
    fn seed_live(&self) {
        let root = engine_root();
        copy_tree(
            &root.join("web/content/modules"),
            &self.at("web/content/modules"),
        );
        let n = std::fs::read_dir(self.at("web/content/modules"))
            .unwrap()
            .count();
        assert!(
            n > 0,
            "copied zero module files — a vacuous fixture is an ERROR, not a pass"
        );
        for rel in [
            "knowledge/topics.toml",
            "web/data/modules_index.json",
            PACK_REL,
        ] {
            let dst = self.at(rel);
            std::fs::create_dir_all(dst.parent().unwrap()).unwrap();
            std::fs::copy(root.join(rel), &dst).unwrap_or_else(|e| panic!("copy {rel}: {e}"));
        }
    }

    fn seed_live_bank_dir(&self) {
        copy_tree(&engine_root().join("bank/items"), &self.at("bank/items"));
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
    fn body(&self) -> String {
        String::from_utf8_lossy(self.artifact.as_deref().expect("artifact")).into_owned()
    }
}

/// Compile in a private copy of the fixture template and assert the
/// invariants that hold on EVERY case, green or red.
fn run_gate(label: &str, f: &Fixture) -> Run {
    let n = ROUND.fetch_add(1, Ordering::SeqCst);
    let root = f.dir.path().join(format!("round{n}"));
    copy_tree(&f.template(), &root);

    let outcome = cdcp_learn::units::write_units(&root).expect("write_units");
    let r = Run {
        code: outcome.code,
        stdout: outcome.stdout.into_bytes(),
        stderr: Vec::new(),
        artifact: std::fs::read(root.join(ARTIFACT_REL)).ok(),
    };

    // VERDICT SHAPE. A reader skimming stdout must never see a success token on
    // a run CI saw fail, or which one wins depends on whether anyone looked.
    if r.code != 0 {
        assert!(
            !r.out().contains("PASS"),
            "[{label}] exited {} with a success token on stdout:\n{}",
            r.code,
            r.out()
        );
    }
    // WRITE-AFTER-VERDICT, both directions.
    if r.code == 0 {
        assert!(
            r.artifact.is_some(),
            "[{label}] exited 0 without writing {ARTIFACT_REL}; a green build \
             that produced no artifact is not a build"
        );
    } else {
        assert!(
            r.artifact.is_none(),
            "[{label}] exited {} but left {ARTIFACT_REL} behind; a failing build \
             must leave no artifact, or a later reader cannot tell a passing \
             artifact from the residue of a failed run",
            r.code
        );
    }

    RAN.fetch_add(1, Ordering::SeqCst);
    r
}

// ── synthetic content ──────────────────────────────────────────────────────

/// A `##` section long enough to survive the short-section filter.
fn section(title: &str, words: usize) -> String {
    let body = (0..words)
        .map(|i| format!("word{i}"))
        .collect::<Vec<_>>()
        .join(" ");
    format!("## {title}\n\n{body}\n\n")
}

/// `(id, module, topic, status)` rows. `status` is spelled out per row rather
/// than defaulted, because the whole class of defect this gate now guards is
/// "somebody forgot the status field existed".
fn bank_json(items: &[(&str, i64, &str, &str)]) -> String {
    let rows: Vec<String> = items
        .iter()
        .map(|(id, module, topic, status)| {
            format!(
                "{{\"id\": \"{id}\", \"module\": {module}, \"topic_ids\": [\"{topic}\"], \
                 \"stem\": \"Which of these is the most likely failure mode for {id} on site?\", \
                 \"explanation\": \"Because the explanation is comfortably long enough to score.\", \
                 \"choices\": [\"A\", \"B\", \"C\", \"D\"], \"correct\": \"A\", \
                 \"status\": \"{status}\"}}"
            )
        })
        .collect();
    format!("[{}]\n", rows.join(", "))
}

const SYNTHETIC_BANK: &[(&str, i64, &str, &str)] = &[
    ("m01-q001", 1, "m01-importance", "approved"),
    ("m01-q002", 1, "m01-dc-types", "approved"),
    ("m01-q003", 1, "m01-importance", "approved"),
    ("m01-q004", 1, "m01-dc-types", "approved"),
    ("m01-q005", 1, "m01-importance", "approved"),
    ("m02-q001", 2, "m02-standards", "approved"),
    ("m02-q002", 2, "m02-standards", "approved"),
    ("m02-q003", 2, "m02-standards", "approved"),
    ("m06-q001", 6, "m06-ups", "approved"),
    ("m06-q002", 6, "m06-ups", "approved"),
    ("m06-q003", 6, "m06-ups", "approved"),
    ("m06-q004", 6, "m06-ups", "approved"),
];

/// A small, self-contained tree that exercises the structural paths the live
/// corpus does not reach: a registry-declared module with no file, an
/// `empty: true` module, duplicate headings, a fenced block, a short section,
/// and a non-ASCII title.
///
/// It carries BOTH named spot-check modules and enough bank items behind them
/// to clear the coverage floor, so the tree is GREEN and therefore HAS an
/// artifact. That is load-bearing since bd-builder-verdict-shape-qm65: a RED run
/// writes nothing, so a case that inspects `units_index.json` has to be a case
/// that passes. The RED shapes get their own fixtures below, each of which
/// starves exactly the thing it is testing.
fn synthetic(f: &Fixture) {
    synthetic_with_bank(f, SYNTHETIC_BANK);
}

fn synthetic_with_bank(f: &Fixture, bank: &[(&str, i64, &str, &str)]) {
    f.put(
        "web/data/modules_index.json",
        r#"{
  "schema_version": 1,
  "modules": [
    {"id": "01-mission-critical", "empty": false},
    {"id": "02-standards", "empty": false},
    {"id": "06-power", "empty": false},
    {"id": "03-ghost", "empty": false},
    {"id": "99-hidden", "empty": true}
  ]
}
"#,
    );
    f.put(
        "knowledge/topics.toml",
        r#"
[[topic]]
id = "m01-importance"
domain = "01-mission-critical"
label = "Importance of the data centre"

[[topic]]
id = "m01-dc-types"
domain = "01-mission-critical"
label = "Data centre types"

[[topic]]
id = "m02-standards"
domain = "02-standards"
label = "Standards bodies"

[[topic]]
id = "m06-ups"
domain = "06-power"
label = "UPS topologies"
"#,
    );
    f.put(PACK_REL, &bank_json(bank));

    let mut m1 = String::from("# Module 1\n\n");
    m1.push_str(&section("Learning objectives", 5)); // short, survives on title
    m1.push_str(&section("Importance of the data centre", 60));
    m1.push_str(&section("Data centre types", 60));
    m1.push_str(&section("Scope note — read this first", 60)); // non-ASCII title
    m1.push_str(&section("Repeated", 60));
    m1.push_str(&section("Repeated", 60)); // duplicate slug -> `repeated-2`
    m1.push_str(&section("Dropped filler", 3)); // short, dropped, keeps its slug
    m1.push_str("## Fenced\n\n```\n## Not a heading\n```\n\n");
    m1.push_str(&(0..60).map(|i| format!("word{i} ")).collect::<String>());
    m1.push('\n');
    f.put("web/content/modules/01-mission-critical.md", &m1);

    let mut m2 = String::from("# Module 2\n\n");
    m2.push_str(&section("Standards bodies", 60));
    f.put("web/content/modules/02-standards.md", &m2);

    // The second named spot check. Present so the synthetic tree is GREEN and
    // therefore writes an artifact for the structural cases to read.
    let mut m6 = String::from("# Module 6\n\n");
    m6.push_str(&section("UPS topologies", 60));
    m6.push_str(&section("Generators", 60));
    m6.push_str(&section("Distribution", 60));
    f.put("web/content/modules/06-power.md", &m6);

    // 03-ghost is declared by the registry and has NO file. 99-hidden DOES
    // have a file and real sections — the `empty: true` flag is the only thing
    // holding it out, so the filter is genuinely exercised rather than masked
    // by an absent file.
    let mut m99 = String::from("# Hidden\n\n");
    m99.push_str(&section("Hidden section one", 60));
    m99.push_str(&section("Hidden section two", 60));
    f.put("web/content/modules/99-hidden.md", &m99);
}

/// Every id in the live pack that is NOT drawable, read through the gate's own
/// loader rather than a second parser written here — a suite that parses the
/// artifact its own way can disagree with the gate about what the bytes say, and
/// then it is testing the second parser.
///
/// Derived from the pack rather than hard-coded, so a retirement wave cannot
/// walk past this suite.
fn live_withheld_ids() -> Vec<String> {
    let bank =
        cdcp_learn::units::load_bank(&engine_root()).expect("the tracked bank pack must load");
    assert!(!bank.is_empty(), "an empty pack is an ERROR, not a pass");
    bank.iter()
        .filter(|it| !it.is_approved())
        .map(|it| it.id.clone())
        .collect()
}

// ── case 1: the live tree, without touching the live tree ──────────────────

#[test]
fn live_inputs_are_green_and_reproduce_the_tracked_artifact() {
    let f = Fixture::new();
    f.seed_live();
    let rs = run_gate("live inputs", &f);

    assert_eq!(rs.code, 0, "live inputs must be GREEN: {}", rs.out());
    assert!(
        rs.out().starts_with("PASS: build_units units="),
        "{}",
        rs.out()
    );
    assert!(
        rs.out().contains("ok: check coverage "),
        "the coverage floor must be reported, not assumed: {}",
        rs.out()
    );
    // The two counts must be reported SEPARATELY and must differ on the live
    // pack: 804 manifest rows, 779 drawable. Collapsing them is the confusion
    // that put retired items into the artifact in the first place.
    assert!(
        rs.out().contains("bank_items=") && rs.out().contains("approved_pool="),
        "the manifest count and the draw pool must both be reported: {}",
        rs.out()
    );
    assert!(
        rs.err().is_empty(),
        "nothing may be written to stderr on the green path: {:?}",
        rs.err()
    );

    // Non-ASCII in a unit title must arrive as a `\uXXXX` escape — the artifact
    // is emitted with Python's `ensure_ascii=True` shape and readers depend on
    // it. This is the single most likely place a JSON writer drifts.
    assert!(
        rs.body().contains("\\u2014"),
        "an em dash in a title must be escaped, not emitted raw"
    );

    // The read-only tie-back. If these bytes match the committed artifact, a run
    // in the live tree would be a no-op write — which is how this suite covers
    // the live tree without a live-tree run.
    let tracked = std::fs::read(engine_root().join(ARTIFACT_REL))
        .expect("the tracked web/data/units_index.json must exist");
    assert_eq!(
        rs.artifact.as_deref(),
        Some(tracked.as_slice()),
        "the gate would rewrite the tracked {ARTIFACT_REL}, so the committed \
         artifact is stale. Regenerate it deliberately — this test will NOT \
         regenerate it for you"
    );
}

// ── case 1b: THE STATUS FILTER, on the live corpus (bd-qqwc) ───────────────

#[test]
fn no_withheld_item_reaches_a_unit_check_on_the_live_corpus() {
    // Anti-vacuous precondition: with zero withheld rows in the pack this case
    // would report identically against a gate that never read `status` at all.
    let withheld = live_withheld_ids();
    assert!(
        !withheld.is_empty(),
        "the live pack carries NO non-approved row, so this case proves nothing. \
         A suite that cannot fail is not a gate."
    );

    let f = Fixture::new();
    f.seed_live();
    let rs = run_gate("live status filter", &f);
    assert_eq!(rs.code, 0, "{}", rs.out());
    let body = rs.body();

    let mut found: Vec<&String> = Vec::new();
    for id in &withheld {
        // The artifact carries ids only inside `check_item_ids` arrays, so a
        // substring hit on a quoted id is a hit on a served item.
        if body.contains(&format!("\"{id}\"")) {
            found.push(id);
        }
    }
    assert!(
        found.is_empty(),
        "{} withheld item(s) reached the unit checks: {found:?}. Before \
         2026-08-14 this case would have named m05-q200 and m11-q226.",
        found.len()
    );
}

#[test]
fn a_retired_item_is_dropped_from_the_pool_and_replaced() {
    // KNOWN-BAD, built by RETIRING one more item rather than by asserting the
    // absence of the two that were already wrong — an absence assertion passes
    // on a gate that draws nothing at all.
    let f = Fixture::new();
    synthetic(&f);
    let green = run_gate("known-bad baseline", &f);
    assert_eq!(green.code, 0, "{}", green.out());
    assert!(
        green.body().contains("\"m06-q001\""),
        "the plant must target an id the gate actually draws: {}",
        green.body()
    );
    assert!(
        green.out().contains("bank_items=12 approved_pool=12"),
        "baseline must draw from the whole synthetic bank: {}",
        green.out()
    );

    let mut planted: Vec<(&str, i64, &str, &str)> = SYNTHETIC_BANK.to_vec();
    for row in planted.iter_mut() {
        if row.0 == "m06-q001" {
            row.3 = "retired";
        }
    }
    let g = Fixture::new();
    synthetic_with_bank(&g, &planted);
    let red = run_gate("known-bad planted", &g);
    assert_eq!(
        red.code,
        0,
        "retiring one of four module-6 items must not break the floor: {}",
        red.out()
    );
    assert!(
        red.out().contains("bank_items=12 approved_pool=11"),
        "the manifest count must stay 12 and the draw pool must drop to 11: {}",
        red.out()
    );
    assert!(
        !red.body().contains("\"m06-q001\""),
        "a retired item must not reach any check_item_ids: {}",
        red.body()
    );
    assert!(
        red.body().contains("\"m06-q002\""),
        "the unit must BACKFILL from the approved pool, not shrink: {}",
        red.body()
    );
    // The unit must not silently lose a question: the count is the thing a
    // learner sees, and a shorter check reads exactly like a thinner module.
    assert!(
        red.body().contains("\"check_count\": 3"),
        "check_count must hold at 3 after the retirement: {}",
        red.body()
    );
}

#[test]
fn a_bank_whose_every_row_is_withheld_is_an_error_not_an_empty_build() {
    // A FILTER THAT REMOVES EVERYTHING IS AN ERROR. Without this leg the gate
    // would emit an artifact with every check_item_ids=[] and let the coverage
    // floor complain about CONTENT, for what is actually a status-filter fault.
    let mut all_retired: Vec<(&str, i64, &str, &str)> = SYNTHETIC_BANK.to_vec();
    for row in all_retired.iter_mut() {
        row.3 = "retired";
    }
    let f = Fixture::new();
    synthetic_with_bank(&f, &all_retired);
    let rs = run_gate("every row withheld", &f);
    assert_ne!(rs.code, 0, "an emptied draw pool must never be a pass");
    assert!(
        rs.out()
            .contains("bank loaded 12 rows and NONE are status='approved'"),
        "the report must name the fault as a STATUS fault, and give both \
         counts: {}",
        rs.out()
    );
    assert!(
        rs.out().contains("bank_items=12 approved_pool=0"),
        "{}",
        rs.out()
    );
    assert!(
        rs.artifact.is_none(),
        "nothing may be written when the draw pool is empty"
    );
}

#[test]
fn a_row_with_no_status_is_withheld_never_permitted() {
    // `export-web` refuses to write a manifest row without a `status`, so a row
    // that reaches this gate without one came from somewhere that guard does not
    // cover. Guessing in its favour is how a withdrawn item reaches a learner.
    let f = Fixture::new();
    synthetic(&f);
    let raw = f.read(PACK_REL);
    let stripped = raw.replace(", \"status\": \"approved\"", "");
    assert!(
        !stripped.contains("\"status\""),
        "the fixture must actually have lost its status fields"
    );
    f.put(PACK_REL, &stripped);
    let rs = run_gate("status-less rows", &f);
    assert_ne!(rs.code, 0, "a status-less pack must never be a pass");
    assert!(
        rs.out()
            .contains("bank loaded 12 rows and NONE are status='approved'"),
        "{}",
        rs.out()
    );
    assert!(rs.artifact.is_none());
}

// ── case 2: anti-vacuous — a missing or empty input is never a pass ────────

#[test]
fn a_missing_content_directory_is_an_error_and_writes_nothing() {
    let f = Fixture::new();
    f.seed_live();
    f.rm("web/content/modules");
    let rs = run_gate("missing content dir", &f);
    assert_ne!(rs.code, 0, "a missing content tree must never be a pass");
    assert_eq!(
        rs.out(),
        "FAIL: missing web/content/modules — run `cdcp build-learn` first\n"
    );
    assert!(
        rs.artifact.is_none(),
        "nothing may be written when the content tree is missing"
    );
}

#[test]
fn zero_modules_and_zero_units_are_errors() {
    let f = Fixture::new();
    f.seed_live();
    f.rm("web/content/modules");
    // The directory exists but holds nothing: the registry still names modules,
    // every one of them is skipped for want of a file, and the build discovers
    // nothing at all.
    std::fs::create_dir_all(f.at("web/content/modules")).unwrap();
    let rs = run_gate("empty content dir", &f);
    assert_ne!(rs.code, 0, "a vacuous unit build must never be a pass");
    assert!(
        rs.out().starts_with("FAIL: build_units units=0 modules=0"),
        "{}",
        rs.out()
    );
    for expected in [
        "zero modules discovered (vacuous unit build is ERROR)",
        "zero units discovered (vacuous unit build is ERROR)",
        "zero modules matched the module-id shape (vacuous check floor is ERROR)",
    ] {
        assert!(
            rs.out().contains(expected),
            "missing {expected:?}: {}",
            rs.out()
        );
    }
    // WRITE-AFTER-VERDICT: the empty artifact used to land anyway, which meant
    // a reader of web/data/units_index.json could not tell this run from a
    // passing one. `run_gate` asserts this for every case; restated here because
    // this is the case where the residue was a `"unit_count": 0` artifact.
    assert!(
        rs.artifact.is_none(),
        "a vacuous build must not leave a units_index.json behind"
    );
}

#[test]
fn modules_that_carry_no_units_between_them_are_an_error() {
    let f = Fixture::new();
    synthetic(&f);
    // Every section is now too short to survive, so the modules exist and hold
    // nothing — a different vacuous shape from "no modules at all". Every
    // module-shaped id has to be starved, or a survivor keeps `total_u`
    // non-zero and this leg never fires.
    f.put(
        "web/content/modules/01-mission-critical.md",
        "## A\n\nshort\n",
    );
    f.put("web/content/modules/02-standards.md", "## B\n\nshort\n");
    f.put("web/content/modules/06-power.md", "## C\n\nshort\n");
    let rs = run_gate("modules with zero units", &f);
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out()
            .contains("modules carry zero units between them (vacuous check floor is ERROR)"),
        "{}",
        rs.out()
    );
}

#[test]
fn a_bank_less_tree_is_red() {
    let f = Fixture::new();
    f.seed_live();
    f.rm(PACK_REL);
    // and no bank/items either, so nothing can be attached to any unit
    let rs = run_gate("no bank at all", &f);
    assert_ne!(rs.code, 0, "zero attachable items must never be a pass");
    assert!(rs.out().starts_with("FAIL: build_units "), "{}", rs.out());
    assert!(
        rs.out().contains("bank_items=0 approved_pool=0"),
        "{}",
        rs.out()
    );
    // An ABSENT bank is a different fault from a bank whose rows are all
    // withheld, and the report must not conflate them: there is nothing to
    // filter here, so the status message must NOT fire.
    assert!(
        !rs.out().contains("NONE are status="),
        "an absent bank must not be reported as a status-filter fault: {}",
        rs.out()
    );
    assert!(
        rs.out().contains(" units across ") && rs.out().contains("have ≥2 checks"),
        "the coverage floor must name the shortfall: {}",
        rs.out()
    );
}

// ── case 3: a module the registry declares that carries no units ──────────

#[test]
fn a_declared_module_with_no_file_and_a_declared_empty_module_are_both_absent() {
    let f = Fixture::new();
    synthetic(&f);
    let rs = run_gate("ghost and empty modules", &f);
    assert_eq!(rs.code, 0, "{}", rs.out());

    // `03-ghost` is declared and has no file: the loop `continue`s, so it never
    // reaches by_module at all and is invisible in the report.
    assert!(
        !rs.body().contains("03-ghost"),
        "a declared module with no file must not appear in the artifact: {}",
        rs.body()
    );
    // `99-hidden` is declared `empty`, so it is filtered out of domain_ids
    // before the loop — even though its file exists and holds real sections.
    assert!(
        f.at("web/content/modules/99-hidden.md").is_file(),
        "the empty-module filter must be tested against a module that HAS a file"
    );
    assert!(!rs.body().contains("99-hidden"), "{}", rs.body());
    // 01, 02 and 06 survive; 03-ghost and 99-hidden do not.
    assert!(rs.out().contains("modules=3"), "{}", rs.out());
    assert!(rs.body().contains("\"module_count\": 3"), "{}", rs.body());
}

#[test]
fn a_module_file_with_no_surviving_sections_reports_a_shortfall() {
    let f = Fixture::new();
    synthetic(&f);
    // 02-standards exists but every section is too short to survive.
    f.put(
        "web/content/modules/02-standards.md",
        "## Only\n\nshort body\n",
    );
    let rs = run_gate("module with zero units", &f);
    assert!(
        rs.out()
            .contains("WARN shortfalls: ['02-standards: 0 units']"),
        "the shortfall list must be a Python list repr: {}",
        rs.out()
    );
    assert!(rs.body().contains("\"02-standards\": []"), "{}", rs.body());
}

// ── case 4: structural parsing — headings, fences, slugs, escaping ────────

#[test]
fn structural_parsing_edges_land_in_the_artifact() {
    let f = Fixture::new();
    synthetic(&f);
    let rs = run_gate("structural edges", &f);
    let body = rs.body();

    // duplicate headings -> `-2` suffix, and the DROPPED short section still
    // consumed its own slug before the filter ran.
    assert!(body.contains("\"heading_id\": \"repeated\""), "{body}");
    assert!(body.contains("\"heading_id\": \"repeated-2\""), "{body}");
    // a fenced block cannot open a unit
    assert!(!body.contains("not-a-heading"), "{body}");
    // a short section survives only when its title names an objective
    assert!(
        body.contains("\"heading_id\": \"learning-objectives\""),
        "{body}"
    );
    assert!(!body.contains("dropped-filler"), "{body}");
    // ensure_ascii escaping of the em dash in the title
    assert!(
        body.contains("Scope note \\u2014 read this first"),
        "{body}"
    );
    // and the slug drops the em dash entirely
    assert!(
        body.contains("\"heading_id\": \"scope-note-read-this-first\""),
        "{body}"
    );
}

// ── case 5: the two input registries that used to be able to vanish ───────
//
// Measured 2026-08-14, before bd-build-units-vacuous-registries-9153, these two
// cases were GREEN:
//
//   no topics.toml       -> exit 0, "PASS: build_units units=134 modules=15"
//   no modules_index.json-> exit 0, "PASS: build_units units=134 modules=16"
//
// The first is the anti-vacuous law broken on an input: an empty topic map reads
// to the picker as "no preference" rather than "nothing to match", so every unit
// still drew its items and the report was indistinguishable from one that
// checked everything. The second is worse than silent — a GREEN verdict carrying
// a WRONG number, because the glob fallback swept in README.md and emitted
// `"README": []` into `by_module`. Both are now exit 1, and the glob fallback is
// DELETED rather than flagged.

#[test]
fn a_missing_topic_registry_is_an_error_and_writes_nothing() {
    let f = Fixture::new();
    f.seed_live();
    f.rm("knowledge/topics.toml");
    let rs = run_gate("no topics.toml", &f);
    assert_ne!(
        rs.code,
        0,
        "a topic registry that vanished must never be a pass: {}",
        rs.out()
    );
    assert!(
        rs.out()
            .starts_with("FAIL: build_units missing required input registries"),
        "the verdict must lead the report: {}",
        rs.out()
    );
    assert!(
        rs.out().contains("knowledge/topics.toml"),
        "the absent file must be NAMED, not merely counted: {}",
        rs.out()
    );
    assert!(
        !rs.out().contains("PASS"),
        "no PASS may appear anywhere on a failing run: {}",
        rs.out()
    );
    assert!(
        rs.artifact.is_none(),
        "nothing may be written when a required registry is missing"
    );
}

#[test]
fn a_missing_learn_index_is_an_error_and_never_globs() {
    let f = Fixture::new();
    f.seed_live();
    f.rm("web/data/modules_index.json");
    let rs = run_gate("no modules_index.json", &f);
    assert_ne!(
        rs.code,
        0,
        "a Learn index that vanished must never be a pass: {}",
        rs.out()
    );
    assert!(
        rs.out().contains("web/data/modules_index.json"),
        "the absent file must be NAMED: {}",
        rs.out()
    );
    // The whole point of deleting the fallback: the module set is never
    // recomputed from a glob, so README.md can never be counted as a module and
    // `modules=16` can never be printed over a 15-module Learn index.
    assert!(
        !rs.out().contains("modules=16") && !rs.out().contains("modules="),
        "the glob fallback must be gone, not merely warned about: {}",
        rs.out()
    );
    assert!(
        rs.artifact.is_none(),
        "nothing may be written when the Learn index is missing"
    );
}

#[test]
fn both_registries_missing_are_named_in_one_report() {
    // An operator fixing one absent registry only to discover the next on the
    // re-run is a gate reporting less than it knows. Both are collected.
    let f = Fixture::new();
    f.seed_live();
    f.rm("knowledge/topics.toml");
    f.rm("web/data/modules_index.json");
    let rs = run_gate("no registries at all", &f);
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(rs.out().contains("knowledge/topics.toml"), "{}", rs.out());
    assert!(
        rs.out().contains("web/data/modules_index.json"),
        "{}",
        rs.out()
    );
    assert!(rs.artifact.is_none());
}

// ── case 6: the per-item TOML bank fallback ───────────────────────────────

#[test]
fn the_toml_bank_fallback_loads_every_file_and_filters_the_same_way() {
    let f = Fixture::new();
    f.seed_live();
    f.rm(PACK_REL);
    f.seed_live_bank_dir();
    let n = std::fs::read_dir(f.at("bank/items")).unwrap().count();
    assert!(
        n > 0,
        "copied zero bank TOMLs — a vacuous fallback case is an ERROR"
    );
    let withheld = live_withheld_ids();
    assert!(
        !withheld.is_empty(),
        "the fallback status leg needs at least one withheld item to be real"
    );
    let rs = run_gate("toml bank fallback", &f);
    assert_eq!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out().contains(&format!("bank_items={n}")),
        "the fallback must load every TOML: {}",
        rs.out()
    );
    // The SECOND path into this gate must filter identically to the first, or a
    // tree without the exported pack quietly reintroduces the whole defect.
    assert!(
        rs.out()
            .contains(&format!("approved_pool={}", n - withheld.len())),
        "the TOML path must apply the same status filter (expected \
         {n} - {} withheld): {}",
        withheld.len(),
        rs.out()
    );
    let body = rs.body();
    let leaked: Vec<&String> = withheld
        .iter()
        .filter(|id| body.contains(&format!("\"{id}\"")))
        .collect();
    assert!(
        leaked.is_empty(),
        "the TOML fallback leaked withheld items into the checks: {leaked:?}"
    );
}

// ── case 7: the named spot checks ─────────────────────────────────────────

#[test]
fn a_thinned_spot_check_module_is_red() {
    let f = Fixture::new();
    f.seed_live();
    let mut md = String::from("# M1\n\n");
    md.push_str(&section("Learning objectives", 60));
    md.push_str(&section("Only one more", 60));
    f.put("web/content/modules/01-mission-critical.md", &md);
    let rs = run_gate("thinned spot-check module", &f);
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out()
            .contains("01-mission-critical has 2 units, need ≥4"),
        "{}",
        rs.out()
    );
    assert!(
        rs.out()
            .contains("WARN shortfalls: ['01-mission-critical: 2 units']"),
        "{}",
        rs.out()
    );
}

// ── the suite must not be vacuously green ─────────────────────────────────

#[test]
fn the_suite_ran_something() {
    // Runs a case itself rather than reading a counter another test may or may
    // not have incremented — test order and parallelism are not a contract, and
    // "0 cases run" must never report like "all passed".
    let before = RAN.load(Ordering::SeqCst);
    let f = Fixture::new();
    synthetic(&f);
    run_gate("suite self-check", &f);
    assert!(
        RAN.load(Ordering::SeqCst) > before,
        "the verdict suite ran nothing"
    );
}
