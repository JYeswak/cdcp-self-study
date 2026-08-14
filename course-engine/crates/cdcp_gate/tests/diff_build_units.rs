//! Differential harness: `cdcp_gate build-units` against `scripts/build_units.py`
//! (bd-substrate-rust-migration-jhd.12).
//!
//! # HOW A *BUILDER* IS DIFFERENTIALLY TESTED (the reusable part)
//!
//! Nine ports landed before this pair and every one of them was a CHECKER — a
//! checker can be run anywhere, any number of times, and observing it costs
//! nothing. A builder MUTATES TRACKED FILES. Three rules follow, and they are
//! the whole shape of this file:
//!
//!   1. **NEVER RUN EITHER IMPLEMENTATION AGAINST THE LIVE TREE.** Comparing by
//!      running both builders in the repo and diffing `web/data/` is a race with
//!      every other reader and writer of those paths — the damage class
//!      bd-791t and bd-gl4j document. The "live tree" case here is a TREE COPY
//!      in TEMP whose inputs are byte-copies of the live ones, so it computes
//!      exactly the live answer and touches nothing.
//!   2. **EACH IMPLEMENTATION GETS ITS OWN COPY.** Both builders write to the
//!      same relative path, so a shared fixture would mean the second run
//!      silently overwrote the first and the artifact comparison would compare
//!      a file with itself. Every case materialises `py/` and `rs/` from the
//!      same template and compares across them.
//!   3. **THE ARTIFACT IS PART OF THE COMPARISON, AS BYTES.** stdout, stderr
//!      and exit code are not the whole observable behaviour of a builder. The
//!      four-tuple is `(stdout, stderr, exit code, bytes written)`, compared as
//!      raw bytes and never as parsed JSON — key order, indentation, `\uXXXX`
//!      escaping and the trailing newline are exactly where two JSON writers
//!      diverge while agreeing on every value.
//!
//! A fourth rule is about what NOT to do: `build_units.py` takes no path flags
//! — it derives its root from `Path(__file__).resolve()` — and this harness does
//! NOT add one to make it testable. Widening a gate's argument surface changes
//! the thing under test. The fixture instead copies the script into
//! `<fixture>/scripts/`, which is the only handle the oracle actually offers,
//! and hands the Rust the same root through the dispatcher's `--root`.
//!
//! The tie-back that makes the tree copy trustworthy: the live case asserts the
//! bytes both sides produce are IDENTICAL TO THE TRACKED
//! `web/data/units_index.json`. That is the read-only proof that running the
//! port in the live tree would be a no-op write, obtained without one.
//!
//! ANTI-VACUOUS DISCIPLINE. A differential that silently compares nothing
//! passes exactly like one that compared everything, so: a missing `python3` is
//! a FAILURE and never a skip; a fixture that copied no module is a FAILURE;
//! and every case increments a counter that is asserted by its own case.
//!
//! Two of the cases below exist to WITNESS a defect rather than to hold a
//! floor — a missing `knowledge/topics.toml` and a missing
//! `web/data/modules_index.json` are each degraded silently by the oracle and
//! the build stays GREEN. Those are findings on bd-.12, reported and NOT fixed:
//! adding the check here would make the Rust stricter than its own oracle and
//! blind every other case in this file.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const BIN: &str = env!("CARGO_BIN_EXE_cdcp_gate");
const ORACLE: &str = "scripts/build_units.py";
const GATE: &str = "build-units";
const ARTIFACT_REL: &str = "web/data/units_index.json";

/// Cases actually compared, so "the harness ran" is itself checked.
static COMPARED: AtomicUsize = AtomicUsize::new(0);
/// Unique sub-directory per comparison, so `py/` and `rs/` never collide.
static ROUND: AtomicUsize = AtomicUsize::new(0);

fn engine_root() -> PathBuf {
    cdcp_gate::root::resolve(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
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
    /// A fixture with the oracle in place and nothing else. The oracle MUST be
    /// copied in: it resolves its own root from `__file__`, so a script outside
    /// the fixture would build the live tree instead.
    fn new() -> Fixture {
        let f = Fixture {
            dir: tempfile::tempdir().unwrap(),
        };
        let script = engine_root().join(ORACLE);
        assert!(
            script.is_file(),
            "{ORACLE} is the differential oracle for this port; without it the port is unverified"
        );
        let dst = f.template().join("scripts");
        std::fs::create_dir_all(&dst).unwrap();
        std::fs::copy(&script, dst.join("build_units.py")).unwrap();
        f
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
            "web/data/bank_items_seed42.json",
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

fn artifact_of(root: &Path) -> Option<Vec<u8>> {
    std::fs::read(root.join(ARTIFACT_REL)).ok()
}

fn python(root: &Path) -> Run {
    let out = Command::new("python3")
        .current_dir(root)
        .arg("scripts/build_units.py")
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
        artifact: artifact_of(root),
    }
}

fn rust(root: &Path) -> Run {
    // The BUILT binary, never `cargo run`: cargo writes build diagnostics to
    // stderr, and a sibling's warning would read here as a false divergence.
    let out = Command::new(BIN)
        .current_dir(root)
        .arg("--root")
        .arg(root)
        .arg(GATE)
        .output()
        .expect("run cdcp_gate");
    Run {
        code: out.status.code().unwrap_or(-1),
        stdout: out.stdout,
        stderr: out.stderr,
        artifact: artifact_of(root),
    }
}

/// The whole acceptance bar in one function: stdout, stderr, exit code AND the
/// bytes written, each compared across two independent copies of the fixture.
fn compare(label: &str, f: &Fixture) -> Run {
    let n = ROUND.fetch_add(1, Ordering::SeqCst);
    let base = f.dir.path().join(format!("round{n}"));
    let py_root = base.join("py");
    let rs_root = base.join("rs");
    copy_tree(&f.template(), &py_root);
    copy_tree(&f.template(), &rs_root);

    let py = python(&py_root);
    let rs = rust(&rs_root);

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
    assert_eq!(
        py.artifact.is_some(),
        rs.artifact.is_some(),
        "[{label}] one side wrote {ARTIFACT_REL} and the other did not \
         (python wrote: {}, rust wrote: {})",
        py.artifact.is_some(),
        rs.artifact.is_some()
    );
    if let (Some(a), Some(b)) = (&py.artifact, &rs.artifact) {
        assert_eq!(
            a.len(),
            b.len(),
            "[{label}] ARTIFACT LENGTH differs: python {} bytes, rust {} bytes",
            a.len(),
            b.len()
        );
        assert!(
            a == b,
            "[{label}] ARTIFACT BYTES differ at offset {:?}",
            a.iter().zip(b.iter()).position(|(x, y)| x != y)
        );
    }

    COMPARED.fetch_add(1, Ordering::SeqCst);
    rs
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

fn bank_json(items: &[(&str, i64, &str)]) -> String {
    let rows: Vec<String> = items
        .iter()
        .map(|(id, module, topic)| {
            format!(
                "{{\"id\": \"{id}\", \"module\": {module}, \"topic_ids\": [\"{topic}\"], \
                 \"stem\": \"Which of these is the most likely failure mode for {id} on site?\", \
                 \"explanation\": \"Because the explanation is comfortably long enough to score.\", \
                 \"choices\": [\"A\", \"B\", \"C\", \"D\"], \"correct\": \"A\"}}"
            )
        })
        .collect();
    format!("[{}]\n", rows.join(", "))
}

/// A small, self-contained tree that exercises the structural paths the live
/// corpus does not reach: a registry-declared module with no file, an
/// `empty: true` module, duplicate headings, a fenced block, a short section,
/// and a non-ASCII title.
fn synthetic(f: &Fixture) {
    f.put(
        "web/data/modules_index.json",
        r#"{
  "schema_version": 1,
  "modules": [
    {"id": "01-mission-critical", "empty": false},
    {"id": "02-standards", "empty": false},
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
"#,
    );
    f.put(
        "web/data/bank_items_seed42.json",
        &bank_json(&[
            ("m01-q001", 1, "m01-importance"),
            ("m01-q002", 1, "m01-dc-types"),
            ("m01-q003", 1, "m01-importance"),
            ("m01-q004", 1, "m01-dc-types"),
            ("m01-q005", 1, "m01-importance"),
            ("m02-q001", 2, "m02-standards"),
            ("m02-q002", 2, "m02-standards"),
            ("m02-q003", 2, "m02-standards"),
        ]),
    );

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

    // 03-ghost is declared by the registry and has NO file. 99-hidden DOES
    // have a file and real sections — the `empty: true` flag is the only thing
    // holding it out, so the filter is genuinely exercised rather than masked
    // by an absent file.
    let mut m99 = String::from("# Hidden\n\n");
    m99.push_str(&section("Hidden section one", 60));
    m99.push_str(&section("Hidden section two", 60));
    f.put("web/content/modules/99-hidden.md", &m99);
}

// ── the oracle must exist and be RUNNABLE at all ───────────────────────────

#[test]
fn the_oracle_is_present_and_green_on_a_copy_of_the_live_tree() {
    let f = Fixture::new();
    f.seed_live();
    let base = f.dir.path().join("oracle-check");
    copy_tree(&f.template(), &base);
    let py = python(&base);
    assert_eq!(
        py.code,
        0,
        "the oracle is RED on the live inputs, so no differential below can be trusted:\n{}\n{}",
        py.out(),
        py.err()
    );
}

// ── case 1: the live tree, without touching the live tree ──────────────────

#[test]
fn live_inputs_are_byte_identical_green_and_reproduce_the_tracked_artifact() {
    let f = Fixture::new();
    f.seed_live();
    let rs = compare("live inputs", &f);

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
    assert!(
        rs.err().is_empty(),
        "the oracle writes nothing to stderr on the green path: {:?}",
        rs.err()
    );

    // Non-ASCII in a unit title must arrive as a `\uXXXX` escape, because the
    // oracle dumps with the default `ensure_ascii=True`. This is the single
    // most likely place two JSON writers agree on values and differ on bytes.
    assert!(
        rs.body().contains("\\u2014"),
        "an em dash in a title must be escaped, not emitted raw"
    );

    // The read-only tie-back. If these bytes match the committed artifact, a
    // run of either implementation in the live tree is a no-op write — which is
    // how this suite covers the live tree without a live-tree run.
    let tracked = std::fs::read(engine_root().join(ARTIFACT_REL))
        .expect("the tracked web/data/units_index.json must exist");
    assert_eq!(
        rs.artifact.as_deref(),
        Some(tracked.as_slice()),
        "the port would rewrite the tracked {ARTIFACT_REL}; a builder that does \
         not reproduce its own committed output is not byte-exact"
    );
}

// ── case 2: anti-vacuous — a missing or empty input is never a pass ────────

#[test]
fn a_missing_content_directory_is_an_error_in_both_and_writes_nothing() {
    let f = Fixture::new();
    f.seed_live();
    f.rm("web/content/modules");
    let rs = compare("missing content dir", &f);
    assert_ne!(rs.code, 0, "a missing content tree must never be a pass");
    assert_eq!(
        rs.out(),
        "FAIL: missing web/content/modules — run build_learn.py first\n"
    );
    assert!(
        rs.artifact.is_none(),
        "nothing may be written when the content tree is missing"
    );
}

#[test]
fn zero_modules_and_zero_units_are_errors_in_both() {
    let f = Fixture::new();
    f.seed_live();
    f.rm("web/content/modules");
    // The directory exists but holds nothing: the registry still names modules,
    // every one of them is skipped for want of a file, and the build discovers
    // nothing at all.
    std::fs::create_dir_all(f.at("web/content/modules")).unwrap();
    let rs = compare("empty content dir", &f);
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
    // WRITE-BEFORE-VERDICT, reproduced: the empty artifact lands anyway.
    assert!(
        rs.artifact.is_some(),
        "the oracle writes the artifact before it evaluates any failure"
    );
    assert!(rs.body().contains("\"unit_count\": 0"), "{}", rs.body());
}

#[test]
fn modules_that_carry_no_units_between_them_are_an_error_in_both() {
    let f = Fixture::new();
    synthetic(&f);
    // Every section is now too short to survive, so the modules exist and hold
    // nothing — a different vacuous shape from "no modules at all".
    f.put(
        "web/content/modules/01-mission-critical.md",
        "## A\n\nshort\n",
    );
    f.put("web/content/modules/02-standards.md", "## B\n\nshort\n");
    let rs = compare("modules with zero units", &f);
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out()
            .contains("modules carry zero units between them (vacuous check floor is ERROR)"),
        "{}",
        rs.out()
    );
}

#[test]
fn a_bank_less_tree_is_red_in_both() {
    let f = Fixture::new();
    f.seed_live();
    f.rm("web/data/bank_items_seed42.json");
    // and no bank/items either, so nothing can be attached to any unit
    let rs = compare("no bank at all", &f);
    assert_ne!(rs.code, 0, "zero attachable items must never be a pass");
    assert!(rs.out().starts_with("FAIL: build_units "), "{}", rs.out());
    assert!(rs.out().contains("bank_items=0"), "{}", rs.out());
    assert!(
        rs.out().contains(" units across ") && rs.out().contains("have ≥2 checks"),
        "the coverage floor must name the shortfall: {}",
        rs.out()
    );
}

// ── case 3: a module the registry declares that carries no units ──────────

#[test]
fn a_declared_module_with_no_file_and_a_declared_empty_module_are_byte_identical() {
    let f = Fixture::new();
    synthetic(&f);
    let rs = compare("ghost and empty modules", &f);

    // `03-ghost` is declared and has no file: the oracle `continue`s, so it
    // never reaches by_module at all and is invisible in the report.
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
    assert!(rs.out().contains("modules=2"), "{}", rs.out());

    // Whatever the oracle does with them, it does byte for byte — the two
    // assertions above only describe what `compare` already proved identical.
    assert_ne!(
        rs.code, 0,
        "the synthetic tree has no 06-power, so the spot check must fire"
    );
    assert!(
        rs.out().contains("06-power has 0 units, need ≥3"),
        "{}",
        rs.out()
    );
}

#[test]
fn a_module_file_with_no_surviving_sections_is_byte_identical() {
    let f = Fixture::new();
    synthetic(&f);
    // 02-standards exists but every section is too short to survive.
    f.put(
        "web/content/modules/02-standards.md",
        "## Only\n\nshort body\n",
    );
    let rs = compare("module with zero units", &f);
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
fn structural_parsing_edges_are_byte_identical() {
    let f = Fixture::new();
    synthetic(&f);
    let rs = compare("structural edges", &f);
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

// ── case 5: the two inputs whose absence the oracle SURVIVES (findings) ───

#[test]
fn a_missing_topic_registry_is_not_an_error_in_either_side() {
    let f = Fixture::new();
    f.seed_live();
    f.rm("knowledge/topics.toml");
    let rs = compare("no topics.toml", &f);
    // FINDING, witnessed rather than repaired: an absent registry silently
    // yields empty `topic_ids`, the picker treats that as "no preference"
    // rather than "nothing to match", and the build stays GREEN. Adding the
    // check here would make the Rust stricter than its oracle.
    assert_eq!(
        rs.code,
        0,
        "the oracle's behaviour changed; re-open the finding on bd-.12: {}",
        rs.out()
    );
    assert!(rs.body().contains("\"topic_ids\": []"), "{}", rs.out());
}

#[test]
fn a_missing_learn_index_falls_back_to_globbing_in_both() {
    let f = Fixture::new();
    f.seed_live();
    f.rm("web/data/modules_index.json");
    let rs = compare("no modules_index.json", &f);
    // FINDING, witnessed rather than repaired: the module set the header calls
    // DERIVED can be absent without anyone hearing about it.
    assert_eq!(
        rs.code,
        0,
        "the oracle's behaviour changed; re-open the finding on bd-.12: {}",
        rs.out()
    );
    // The glob picks up README.md too, which the Learn index excludes — the
    // observable difference the fallback introduces.
    assert!(rs.out().contains("modules="), "{}", rs.out());
}

// ── case 6: the per-item TOML bank fallback ───────────────────────────────

#[test]
fn the_toml_bank_fallback_is_byte_identical() {
    let f = Fixture::new();
    f.seed_live();
    f.rm("web/data/bank_items_seed42.json");
    f.seed_live_bank_dir();
    let n = std::fs::read_dir(f.at("bank/items")).unwrap().count();
    assert!(
        n > 0,
        "copied zero bank TOMLs — a vacuous fallback case is an ERROR"
    );
    let rs = compare("toml bank fallback", &f);
    assert!(
        rs.out().contains(&format!("bank_items={n}")),
        "the fallback must load every TOML: {}",
        rs.out()
    );
    assert_eq!(rs.code, 0, "{}", rs.out());
}

// ── case 7: the named spot checks ─────────────────────────────────────────

#[test]
fn a_thinned_spot_check_module_is_red_in_both() {
    let f = Fixture::new();
    f.seed_live();
    let mut md = String::from("# M1\n\n");
    md.push_str(&section("Learning objectives", 60));
    md.push_str(&section("Only one more", 60));
    f.put("web/content/modules/01-mission-critical.md", &md);
    let rs = compare("thinned spot-check module", &f);
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

// ── the harness must not be vacuously green ───────────────────────────────

#[test]
fn the_harness_compared_something() {
    // Runs a case itself rather than reading a counter another test may or may
    // not have incremented — test order and parallelism are not a contract,
    // and "0 cases compared" must never report like "all passed".
    let before = COMPARED.load(Ordering::SeqCst);
    let f = Fixture::new();
    synthetic(&f);
    compare("harness self-check", &f);
    assert!(
        COMPARED.load(Ordering::SeqCst) > before,
        "the differential harness compared nothing"
    );
}
