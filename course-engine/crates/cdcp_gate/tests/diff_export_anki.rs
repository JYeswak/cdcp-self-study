//! Differential harness: `cdcp_gate export-anki` vs `scripts/export_anki.py`
//! (bd-substrate-rust-migration-jhd.13).
//!
//! `dist/` is untracked, so ground truth is NOT the live tree: oracle and
//! rust each run in their own temp copy of the same fixture. `compare`
//! asserts stdout, stderr, exit, the FILE SET under `dist/anki/`, and the
//! bytes of every file. The one permitted normalisation is `Norm::RootPrefix`
//! (each side's root -> `<ROOT>`), and it fails if it does not fire.
//!
//! `.apkg` is absent: two oracle runs on identical inputs differ (pinned
//! by `the_oracle_apkg_is_not_byte_reproducible_against_itself`).
//! Empty decks (missing/empty/no-id bank, emptying filter, empty format)
//! are exit 1 and write nothing on BOTH sides. Missing python3 is FAILURE.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const BIN: &str = env!("CARGO_BIN_EXE_cdcp_gate");
const ORACLE: &str = "scripts/export_anki.py";
const ORACLE_BASENAME: &str = "export_anki.py";
const GATE: &str = "export-anki";
/// Every file this gate may produce lands under here.
const OUT_DIR_REL: &str = "dist/anki";
/// The bank directory both loaders read.
const ITEMS_DIR_REL: &str = "bank/items";

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
    /// copied in: it resolves its own root from `__file__`, so a script left
    /// outside the fixture would read the LIVE bank and write the LIVE
    /// `dist/anki/`. Rule 5 — the oracle gains no `--root` flag to make this
    /// convenient; the fixture moves to the oracle instead.
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
        std::fs::copy(&script, dst.join(ORACLE_BASENAME)).unwrap();
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

    /// Byte-copy the live item bank — the input the default invocation reads.
    fn seed_live_bank(&self) {
        copy_tree(&engine_root().join(ITEMS_DIR_REL), &self.at(ITEMS_DIR_REL));
        let n = std::fs::read_dir(self.at(ITEMS_DIR_REL)).unwrap().count();
        assert!(
            n > 0,
            "copied zero bank items — a vacuous fixture is an ERROR, not a pass"
        );
    }

    /// Byte-copy the live web packs, for `--source seed42` and `--source keys`.
    fn seed_live_packs(&self) {
        let root = engine_root();
        for rel in [
            "web/data/bank_items_seed42.json",
            "web/data/mock40_seed42.json",
            "web/data/keys_seed42.json",
        ] {
            let src = root.join(rel);
            if src.is_file() {
                let dst = self.at(rel);
                std::fs::create_dir_all(dst.parent().unwrap()).unwrap();
                std::fs::copy(&src, &dst).unwrap();
            }
        }
    }

    fn seed_live_goldens_fixture(&self) {
        let rel = "goldens/fixtures/mock40_seed42.json";
        let src = engine_root().join(rel);
        if src.is_file() {
            let dst = self.at(rel);
            std::fs::create_dir_all(dst.parent().unwrap()).unwrap();
            std::fs::copy(&src, &dst).unwrap();
        }
    }
}

/// One implementation's complete observable behaviour.
struct Run {
    code: i32,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    /// Every file under `dist/anki/`, keyed by path relative to the fixture root.
    files: BTreeMap<String, Vec<u8>>,
}

impl Run {
    fn out(&self) -> String {
        String::from_utf8_lossy(&self.stdout).into_owned()
    }
    fn err(&self) -> String {
        String::from_utf8_lossy(&self.stderr).into_owned()
    }
    fn names(&self) -> Vec<String> {
        self.files.keys().cloned().collect()
    }
    fn text(&self, rel: &str) -> String {
        String::from_utf8_lossy(
            self.files
                .get(rel)
                .unwrap_or_else(|| panic!("no {rel} among {:?}", self.names())),
        )
        .into_owned()
    }
}

/// Enumerate the produced file SET, not just the files we expected. A port that
/// writes an extra file is as wrong as one that omits a file, and only a walk
/// catches the extra.
fn collect(root: &Path, rel_dir: &str) -> BTreeMap<String, Vec<u8>> {
    let mut out = BTreeMap::new();
    fn walk(base: &Path, dir: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(base, &p, out);
            } else {
                let rel = p
                    .strip_prefix(base)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                out.insert(rel, std::fs::read(&p).unwrap());
            }
        }
    }
    walk(root, &root.join(rel_dir), &mut out);
    out
}

fn python(root: &Path, args: &[&str]) -> Run {
    let out = Command::new("python3")
        .current_dir(root)
        .arg(format!("scripts/{ORACLE_BASENAME}"))
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
        files: collect(root, OUT_DIR_REL),
    }
}

fn rust(root: &Path, args: &[&str]) -> Run {
    // The BUILT binary, never `cargo run`: cargo writes build diagnostics to
    // stderr, and a sibling's warning would read here as a false divergence.
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
        files: collect(root, OUT_DIR_REL),
    }
}

/// How stdout/stderr are compared. See the header — `RootPrefix` is the ONE
/// permitted normalisation and it fails if it does not fire.
#[derive(Clone, Copy, PartialEq)]
enum Norm {
    Raw,
    RootPrefix,
}

fn canon_stream(bytes: &[u8], root: &Path, norm: Norm) -> (Vec<u8>, usize) {
    match norm {
        Norm::Raw => (bytes.to_vec(), 0),
        Norm::RootPrefix => {
            let s = String::from_utf8_lossy(bytes).into_owned();
            let r = root.to_string_lossy().into_owned();
            let hits = s.matches(&r).count();
            (s.replace(&r, "<ROOT>").into_bytes(), hits)
        }
    }
}

/// The whole acceptance bar in one function: stdout, stderr, exit code, the
/// produced FILE SET, and the BYTES of every file in it — each compared across
/// two independent copies of the fixture.
fn compare(label: &str, f: &Fixture, args: &[&str], norm: Norm) -> Run {
    let n = ROUND.fetch_add(1, Ordering::SeqCst);
    let base = f.dir.path().join(format!("round{n}"));
    let py_root = base.join("py");
    let rs_root = base.join("rs");
    copy_tree(&f.template(), &py_root);
    copy_tree(&f.template(), &rs_root);
    // The oracle resolves its own root, so its printed paths are symlink-free.
    // Canonicalise both sides or the `<ROOT>` substitution cannot fire on macOS,
    // where a temp dir is reached through /var -> /private/var.
    let py_root = py_root.canonicalize().unwrap();
    let rs_root = rs_root.canonicalize().unwrap();

    let py = python(&py_root, args);
    let rs = rust(&rs_root, args);

    let (py_out, py_hits) = canon_stream(&py.stdout, &py_root, norm);
    let (rs_out, rs_hits) = canon_stream(&rs.stdout, &rs_root, norm);
    if norm == Norm::RootPrefix {
        assert!(
            py_hits > 0 && rs_hits > 0,
            "[{label}] the <ROOT> substitution matched nothing (python {py_hits}, rust {rs_hits}). \
             A normalisation that never fires hides divergence instead of describing it.\n\
             --- python ---\n{}\n--- rust ---\n{}",
            py.out(),
            rs.out()
        );
        for (side, b) in [("python", &py_out), ("rust", &rs_out)] {
            let s = String::from_utf8_lossy(b);
            assert!(
                !s.contains("/round") && !s.contains(&*base.to_string_lossy()),
                "[{label}] a fixture path survived the {side} substitution: {s}"
            );
        }
    }

    assert_eq!(
        py_out,
        rs_out,
        "[{label}] STDOUT differs.\n--- python ---\n{}\n--- rust ---\n{}",
        py.out(),
        rs.out()
    );
    let (py_err, _) = canon_stream(&py.stderr, &py_root, norm);
    let (rs_err, _) = canon_stream(&rs.stderr, &rs_root, norm);
    assert_eq!(
        py_err,
        rs_err,
        "[{label}] STDERR differs.\n--- python ---\n{}\n--- rust ---\n{}",
        py.err(),
        rs.err()
    );
    assert_eq!(
        py.code,
        rs.code,
        "[{label}] EXIT CODE differs: python {} vs rust {}\npy stderr: {}\nrs stderr: {}",
        py.code,
        rs.code,
        py.err(),
        rs.err()
    );

    // ── the FILE SET, before any byte comparison ───────────────────────────
    assert_eq!(
        py.names(),
        rs.names(),
        "[{label}] the produced FILE SET differs. A deck written under the wrong name, or one \
         file short, is a broken deck even when every byte written matches.\npython: {:?}\nrust:   {:?}",
        py.names(),
        rs.names()
    );
    for name in py.names() {
        let a = &py.files[&name];
        let b = &rs.files[&name];
        assert_eq!(
            a.len(),
            b.len(),
            "[{label}] {name} LENGTH differs: python {} bytes, rust {} bytes",
            a.len(),
            b.len()
        );
        assert!(
            a == b,
            "[{label}] {name} BYTES differ at offset {:?}",
            a.iter().zip(b.iter()).position(|(x, y)| x != y)
        );
    }

    // ── VERDICT SHAPE and WRITE-AFTER-VERDICT, per side, on EVERY case ─────
    // Asserted per side rather than only across the two, because a differential
    // only catches a regression that lands on ONE side — two implementations
    // that both regress agree with each other perfectly.
    for (side, r) in [("python", &py), ("rust", &rs)] {
        if r.code != 0 {
            let all = format!("{}{}", r.out(), r.err());
            for token in ["export_anki ok", "PASS"] {
                assert!(
                    !all.contains(token),
                    "[{label}] {side} exited {} carrying the success token {token:?}. A reader \
                     skimming output would see success while CI saw non-zero, and which one wins \
                     depends on whether anyone looked:\n{all}",
                    r.code
                );
            }
            assert!(
                r.files.is_empty(),
                "[{label}] {side} exited {} but left {:?} behind. A builder writes before it \
                 judges unless it is stopped from doing so, and a later reader cannot tell a \
                 shipped deck from the residue of a failed run.",
                r.code,
                r.names()
            );
        } else {
            assert!(
                !r.files.is_empty(),
                "[{label}] {side} exited 0 without producing a single file; a green export that \
                 exported nothing is not an export"
            );
        }
    }

    COMPARED.fetch_add(1, Ordering::SeqCst);
    rs
}

// ── synthetic content ──────────────────────────────────────────────────────

/// One bank item as TOML. `extra` carries whatever the case is actually testing.
fn item_toml(id: &str, module: i64, stem: &str, correct: &str, extra: &str) -> String {
    format!(
        "id = \"{id}\"\nmodule = {module}\nstatus = \"approved\"\n\
         stem = \"{stem}\"\nchoices = [\"alpha\", \"beta\", \"gamma\", \"delta\"]\n\
         correct = \"{correct}\"\nexplanation = \"Because {id} says so.\"\n{extra}"
    )
}

/// A small bank that is GREEN, so the structural cases have files to inspect.
fn synthetic_bank(f: &Fixture) {
    f.put(
        "bank/items/m01-q001.toml",
        &item_toml(
            "m01-q001",
            1,
            "Which of these is a mission-critical facility?",
            "A",
            "topic_ids = [\"m01-importance\"]\ntags = [\"basics\", \"intro\"]\n",
        ),
    );
    f.put(
        "bank/items/m02-q001.toml",
        &item_toml(
            "m02-q001",
            2,
            "Which body publishes the standard?",
            "c",
            "topic_ids = [\"m02-standards\"]\ntags = [\"standards\"]\n",
        ),
    );
    f.put(
        "bank/items/m02-q002.toml",
        &item_toml(
            "m02-q002",
            2,
            "Second standards question",
            "B",
            "topic_ids = [\"m02-standards\"]\n",
        ),
    );
    f.put(
        "bank/items/m06-q001.toml",
        &item_toml(
            "m06-q001",
            6,
            "Which UPS topology is double conversion?",
            "D",
            "topic_ids = [\"m06-ups\"]\ntags = \"power\"\n",
        ),
    );
}

// ── the oracle must exist and be RUNNABLE at all ───────────────────────────

#[test]
fn the_oracle_is_present_and_green_on_a_copy_of_the_live_bank() {
    let f = Fixture::new();
    f.seed_live_bank();
    let base = f.dir.path().join("oracle-check");
    copy_tree(&f.template(), &base);
    let py = python(&base, &["--format", "tsv", "--out", OUT_DIR_REL]);
    assert_eq!(
        py.code,
        0,
        "the oracle is RED on the live bank, so no differential below can be trusted:\n{}\n{}",
        py.out(),
        py.err()
    );
    assert!(
        !py.files.is_empty(),
        "the oracle produced no files on the live bank; the differential would compare nothing"
    );
}

// ── case 1: the live bank, without touching the live tree ──────────────────

#[test]
fn live_bank_tsv_is_byte_identical() {
    let f = Fixture::new();
    f.seed_live_bank();
    let mut approved = 0usize;
    let mut scanned = 0usize;
    for e in std::fs::read_dir(f.at(ITEMS_DIR_REL)).unwrap().flatten() {
        scanned += 1;
        if std::fs::read_to_string(e.path())
            .map(|b| b.contains("status = \"approved\""))
            .unwrap_or(false)
        {
            approved += 1;
        }
    }
    let rs = compare(
        "live bank tsv",
        &f,
        &["--format", "tsv", "--out", OUT_DIR_REL],
        Norm::Raw,
    );
    assert_eq!(rs.code, 0, "{}\n{}", rs.out(), rs.err());
    assert!(rs.out().starts_with("export_anki ok\n"), "{}", rs.out());
    assert!(
        rs.out().contains(&format!("  cards={approved}\n")),
        "the card count must be the approved pool ({approved}): {}",
        rs.out()
    );
    assert!(
        rs.out()
            .contains(&format!("  {scanned} scanned, {approved} exported\n")),
        "receipt must name both populations: {}",
        rs.out()
    );
    assert!(rs.err().is_empty(), "stderr must be empty: {:?}", rs.err());
    assert_eq!(
        rs.names(),
        vec![
            "dist/anki/README.txt".to_string(),
            "dist/anki/cdcp_bank.tsv".to_string()
        ]
    );
    // The TSV opens with the two operator comment lines and NO header row —
    // an added header would silently become a card on import.
    let tsv = rs.text("dist/anki/cdcp_bank.tsv");
    assert!(
        tsv.starts_with("# CDCP Study Anki export"),
        "{}",
        &tsv[..80]
    );
    assert_eq!(
        tsv.lines().count(),
        approved + 2,
        "one row per card plus exactly two comment lines"
    );
    assert!(
        tsv.ends_with('\n'),
        "the trailing newline is part of the deck"
    );
}

#[test]
fn the_default_invocation_is_byte_identical_including_its_absolute_paths() {
    // The invocation `check.sh` actually uses, minus `apkg` — which is why this
    // case needs the <ROOT> substitution and every other case does not.
    let f = Fixture::new();
    f.seed_live_bank();
    let rs = compare(
        "default out dir",
        &f,
        &["--format", "tsv"],
        Norm::RootPrefix,
    );
    assert_eq!(rs.code, 0, "{}\n{}", rs.out(), rs.err());
    assert!(
        rs.out().contains("/dist/anki/cdcp_bank.tsv"),
        "the default --out must land in dist/anki: {}",
        rs.out()
    );
    assert_eq!(
        rs.names(),
        vec![
            "dist/anki/README.txt".to_string(),
            "dist/anki/cdcp_bank.tsv".to_string()
        ]
    );
}

#[test]
fn csv_and_tsv_together_are_byte_identical_and_differ_from_each_other() {
    let f = Fixture::new();
    f.seed_live_bank();
    let rs = compare(
        "tsv+csv",
        &f,
        &["--format", "tsv,csv", "--out", OUT_DIR_REL],
        Norm::Raw,
    );
    assert_eq!(
        rs.names(),
        vec![
            "dist/anki/README.txt".to_string(),
            "dist/anki/cdcp_bank.csv".to_string(),
            "dist/anki/cdcp_bank.tsv".to_string()
        ]
    );
    // The CSV carries a header row; the TSV deliberately does not. A port that
    // conflated the two writers would still be byte-identical on one of them.
    assert!(
        rs.text("dist/anki/cdcp_bank.csv")
            .starts_with("stem,answer,explanation,module\n"),
        "the csv form leads with a header row"
    );
    assert!(!rs
        .text("dist/anki/cdcp_bank.tsv")
        .contains("stem\tanswer\t"));
}

// ── case 2: ANTI-VACUOUS — every road to a shipped empty deck ──────────────

#[test]
fn a_missing_bank_is_an_error_in_both_and_writes_nothing() {
    let f = Fixture::new();
    let rs = compare(
        "no bank dir",
        &f,
        &["--format", "tsv", "--out", OUT_DIR_REL],
        Norm::Raw,
    );
    assert_ne!(rs.code, 0, "a missing bank must never ship a deck");
    assert_eq!(rs.err(), "FAIL: zero items to export\n");
    assert!(rs.out().is_empty(), "nothing on stdout: {:?}", rs.out());
    assert!(rs.files.is_empty());
}

#[test]
fn an_empty_bank_directory_is_an_error_in_both_and_writes_nothing() {
    let f = Fixture::new();
    std::fs::create_dir_all(f.at(ITEMS_DIR_REL)).unwrap();
    let rs = compare(
        "empty bank dir",
        &f,
        &["--format", "tsv", "--out", OUT_DIR_REL],
        Norm::Raw,
    );
    assert_ne!(rs.code, 0, "an empty bank must never ship a deck");
    assert_eq!(rs.err(), "FAIL: zero items to export\n");
    assert!(rs.files.is_empty());
}

#[test]
fn a_bank_of_files_without_ids_is_an_error_in_both() {
    // The nastiest empty shape: 40 files present, every one of them skipped.
    // A gate that counted FILES instead of ITEMS would be green here.
    let f = Fixture::new();
    for i in 0..40 {
        f.put(
            &format!("bank/items/no-id-{i:02}.toml"),
            "stem = \"orphan\"\nmodule = 1\n",
        );
    }
    let rs = compare(
        "bank with no ids",
        &f,
        &["--format", "tsv", "--out", OUT_DIR_REL],
        Norm::Raw,
    );
    assert_ne!(rs.code, 0, "40 unusable files must not read as 40 cards");
    assert_eq!(rs.err(), "FAIL: zero items to export\n");
    assert!(rs.files.is_empty());
}

#[test]
fn a_module_filter_that_removes_everything_is_an_error_in_both() {
    let f = Fixture::new();
    synthetic_bank(&f);
    let rs = compare(
        "module filter empties",
        &f,
        &["--module", "99", "--format", "tsv", "--out", OUT_DIR_REL],
        Norm::Raw,
    );
    assert_ne!(rs.code, 0);
    assert_eq!(rs.err(), "FAIL: filter removed all items\n");
    assert!(rs.files.is_empty());
}

#[test]
fn a_tag_filter_that_removes_everything_is_an_error_in_both() {
    let f = Fixture::new();
    synthetic_bank(&f);
    let rs = compare(
        "tag filter empties",
        &f,
        &[
            "--tag",
            "nosuchtag",
            "--format",
            "tsv",
            "--out",
            OUT_DIR_REL,
        ],
        Norm::Raw,
    );
    assert_ne!(rs.code, 0);
    assert_eq!(rs.err(), "FAIL: filter removed all items\n");
    assert!(rs.files.is_empty());
}

#[test]
fn an_empty_format_list_is_an_error_in_both() {
    let f = Fixture::new();
    synthetic_bank(&f);
    let rs = compare(
        "no formats",
        &f,
        &["--format", ",", "--out", OUT_DIR_REL],
        Norm::Raw,
    );
    assert_ne!(rs.code, 0, "requesting no format must not be a silent pass");
    assert_eq!(rs.err(), "FAIL: no formats requested\n");
    assert!(rs.files.is_empty());
}

#[test]
fn an_unknown_format_is_an_error_in_both_and_is_named() {
    let f = Fixture::new();
    synthetic_bank(&f);
    let rs = compare(
        "unknown format",
        &f,
        &["--format", "tsv,xyz,abc", "--out", OUT_DIR_REL],
        Norm::Raw,
    );
    assert_ne!(rs.code, 0);
    // A Python `sorted()` list repr, single-quoted, comma-space separated.
    assert_eq!(rs.err(), "FAIL: unknown format(s): ['abc', 'xyz']\n");
    assert!(
        rs.files.is_empty(),
        "the tsv leg must not be written before the format list is judged"
    );
}

#[test]
fn a_missing_keys_pack_is_an_error_in_both() {
    let f = Fixture::new();
    synthetic_bank(&f);
    // No web/data packs and no goldens fixture: the source cannot be built.
    let rs = compare(
        "keys pack absent",
        &f,
        &["--source", "keys", "--format", "tsv", "--out", OUT_DIR_REL],
        Norm::Raw,
    );
    assert_ne!(rs.code, 0);
    assert_eq!(rs.err(), "FAIL: keys/seed42 packs not found\n");
    assert!(rs.files.is_empty());
}

// ── case 3: the alternate sources ──────────────────────────────────────────

#[test]
fn the_seed42_source_is_byte_identical_on_the_live_pack() {
    let f = Fixture::new();
    f.seed_live_bank();
    f.seed_live_packs();
    if !f.at("web/data/bank_items_seed42.json").is_file() {
        panic!(
            "web/data/bank_items_seed42.json is absent from the live tree, so this case would \
             test the FALLBACK instead of the source it names. A case that silently tests \
             something else is worse than a missing case."
        );
    }
    let rs = compare(
        "seed42 source",
        &f,
        &[
            "--source",
            "seed42",
            "--format",
            "tsv",
            "--out",
            OUT_DIR_REL,
        ],
        Norm::Raw,
    );
    assert_eq!(rs.code, 0, "{}\n{}", rs.out(), rs.err());
    assert!(rs.out().contains("  source=seed42\n"), "{}", rs.out());
    let approved = std::fs::read_dir(f.at(ITEMS_DIR_REL))
        .unwrap()
        .flatten()
        .filter(|e| {
            std::fs::read_to_string(e.path())
                .map(|b| b.contains("status = \"approved\""))
                .unwrap_or(false)
        })
        .count();
    assert!(
        rs.out().contains(&format!("  cards={approved}\n")),
        "seed42 must export the approved pool ({approved}), not the file count: {}",
        rs.out()
    );
    assert_eq!(
        rs.names(),
        vec![
            "dist/anki/README.txt".to_string(),
            "dist/anki/cdcp_seed42_bank.tsv".to_string()
        ],
        "the stem must switch with the source, or the deck lands under the wrong name"
    );
}

#[test]
fn the_seed42_fallback_warns_on_stderr_and_stays_green_in_both() {
    // The pack is absent, so the oracle warns and falls back to the bank. The
    // WARN goes to STDERR while the report goes to STDOUT — a port that merged
    // the two streams would be caught here and nowhere else.
    let f = Fixture::new();
    f.seed_live_bank();
    let rs = compare(
        "seed42 fallback",
        &f,
        &[
            "--source",
            "seed42",
            "--format",
            "tsv",
            "--out",
            OUT_DIR_REL,
        ],
        Norm::Raw,
    );
    assert_eq!(rs.code, 0, "{}\n{}", rs.out(), rs.err());
    assert_eq!(
        rs.err(),
        "WARN: bank_items_seed42.json missing — falling back to bank\n"
    );
    // …and the FALLBACK changes the stem back to the bank one while `source=`
    // still reports what was ASKED for. Reproduced, not tidied.
    assert!(rs.out().contains("  source=seed42\n"), "{}", rs.out());
    assert_eq!(
        rs.names(),
        vec![
            "dist/anki/README.txt".to_string(),
            "dist/anki/cdcp_bank.tsv".to_string()
        ]
    );
    assert!(
        rs.text("dist/anki/README.txt").contains("Source: seed42\n"),
        "the README records the requested source, not the one actually used"
    );
}

#[test]
fn the_keys_source_is_byte_identical_on_the_live_packs() {
    let f = Fixture::new();
    f.seed_live_bank();
    f.seed_live_packs();
    let have_packs = f.at("web/data/mock40_seed42.json").is_file()
        && f.at("web/data/keys_seed42.json").is_file();
    if !have_packs {
        // Fall back to the goldens fixture branch, which is the other half of
        // the same loader. Either way a real branch is exercised — never a skip.
        f.seed_live_goldens_fixture();
        assert!(
            f.at("goldens/fixtures/mock40_seed42.json").is_file(),
            "neither the live packs nor the goldens fixture exist, so `--source keys` cannot be \
             differentially tested at all. That is a FAILURE, not a skip."
        );
    }
    let rs = compare(
        "keys source",
        &f,
        &["--source", "keys", "--format", "tsv", "--out", OUT_DIR_REL],
        Norm::Raw,
    );
    assert_eq!(rs.code, 0, "{}\n{}", rs.out(), rs.err());
    assert!(rs.out().contains("  source=keys\n"), "{}", rs.out());
    assert_eq!(
        rs.names(),
        vec![
            "dist/anki/README.txt".to_string(),
            "dist/anki/cdcp_seed42_mock40.tsv".to_string()
        ]
    );
}

// ── case 4: field formatting, where two writers agree on values and differ
//    on bytes ───────────────────────────────────────────────────────────────

#[test]
fn quoting_newlines_tabs_and_unicode_are_byte_identical() {
    let f = Fixture::new();
    // Every escape hatch of the csv dialect in one bank:
    //   q01 a double quote  -> field quoted, quote doubled
    //   q02 a literal tab   -> flattened to a space in the TSV, quoted in the CSV
    //   q03 a newline       -> flattened in the TSV, QUOTED AND KEPT in the CSV
    //   q04 a bare CR       -> QUOTED (measured, not inferred), NOT flattened
    //   q05 a comma         -> inert in the TSV, quoted in the CSV
    //   q06 non-ASCII       -> raw UTF-8 in both, never escaped
    //   q07 leading/trailing whitespace -> stripped
    f.put(
        "bank/items/q01.toml",
        "id = \"q01\"\nmodule = 1\nstem = \"He said \\\"yes\\\" firmly\"\nchoices = [\"a\", \"b\"]\ncorrect = \"A\"\nexplanation = \"quote \\\"inside\\\"\"\n",
    );
    f.put(
        "bank/items/q02.toml",
        "id = \"q02\"\nmodule = 1\nstem = \"before\\tafter\"\nchoices = [\"a\"]\ncorrect = \"A\"\nexplanation = \"tab\\there\"\n",
    );
    f.put(
        "bank/items/q03.toml",
        "id = \"q03\"\nmodule = 1\nstem = \"line one\\nline two\"\nchoices = [\"a\"]\ncorrect = \"A\"\nexplanation = \"para\\n\\npara\"\n",
    );
    f.put(
        "bank/items/q04.toml",
        "id = \"q04\"\nmodule = 1\nstem = \"cr\\rhere\"\nchoices = [\"a\"]\ncorrect = \"A\"\nexplanation = \"\"\n",
    );
    f.put(
        "bank/items/q05.toml",
        "id = \"q05\"\nmodule = 1\nstem = \"a, b, and c\"\nchoices = [\"a\"]\ncorrect = \"A\"\nexplanation = \"x, y\"\n",
    );
    f.put(
        "bank/items/q06.toml",
        "id = \"q06\"\nmodule = 1\nstem = \"Résumé — naïve ✓ 温度\"\nchoices = [\"±5 °C\"]\ncorrect = \"A\"\nexplanation = \"em — dash\"\n",
    );
    f.put(
        "bank/items/q07.toml",
        "id = \"q07\"\nmodule = 1\nstem = \"   padded   \"\nchoices = [\"  spaced  \"]\ncorrect = \"  a  \"\nexplanation = \"\\n  wrapped  \\n\"\n",
    );
    let rs = compare(
        "field formatting",
        &f,
        &["--format", "tsv,csv", "--out", OUT_DIR_REL],
        Norm::Raw,
    );
    assert_eq!(rs.code, 0, "{}\n{}", rs.out(), rs.err());
    let tsv = rs.text("dist/anki/cdcp_bank.tsv");
    let csv = rs.text("dist/anki/cdcp_bank.csv");

    // The TSV is one line per card no matter what the field held.
    assert_eq!(
        tsv.lines().count(),
        7 + 2,
        "a newline inside a field must not open a second TSV row: {tsv}"
    );
    // …while the CSV keeps the newline inside a quoted field, so it has MORE
    // lines than cards. Two writers, two rules, both reproduced.
    assert!(
        csv.lines().count() > 7 + 1,
        "the csv form keeps embedded newlines inside quotes: {csv}"
    );
    // A bare CR survives the TSV path untouched — the flatten list is
    // {tab, newline} and nothing else.
    // A bare CR is NOT flattened (the flatten list is {tab, newline}) but IS
    // quoted. Both halves matter and the port originally got the second one
    // wrong; the byte comparison is what found it.
    assert!(
        tsv.contains("\"cr\rhere\""),
        "a bare CR is quoted and kept verbatim: {tsv:?}"
    );
    // Non-ASCII is raw UTF-8, never a \u escape.
    assert!(tsv.contains("Résumé — naïve ✓ 温度"), "{tsv}");
    // Whitespace is stripped from every field before it is written.
    assert!(
        tsv.contains("padded\t"),
        "leading/trailing space is stripped"
    );
}

#[test]
fn the_answer_letter_is_resolved_against_the_choice_list() {
    let f = Fixture::new();
    synthetic_bank(&f);
    let rs = compare(
        "answer resolution",
        &f,
        &["--format", "tsv", "--out", OUT_DIR_REL],
        Norm::Raw,
    );
    let tsv = rs.text("dist/anki/cdcp_bank.tsv");
    // Upper and lower case both resolve, and the letter is re-emitted uppercased.
    assert!(tsv.contains("\tA) alpha\t"), "{tsv}");
    assert!(
        tsv.contains("\tC) gamma\t"),
        "lowercase 'c' must uppercase: {tsv}"
    );
    assert!(tsv.contains("\tD) delta\t"), "{tsv}");
}

#[test]
fn an_answer_letter_outside_the_choice_list_falls_through_to_the_letter() {
    let f = Fixture::new();
    f.put(
        "bank/items/q01.toml",
        "id = \"q01\"\nmodule = 1\nstem = \"s\"\nchoices = [\"only\"]\ncorrect = \"D\"\nexplanation = \"e\"\n",
    );
    f.put(
        "bank/items/q02.toml",
        "id = \"q02\"\nmodule = 1\nstem = \"s\"\nchoices = [\"only\"]\ncorrect = \"E\"\nexplanation = \"e\"\n",
    );
    let rs = compare(
        "answer out of range",
        &f,
        &["--format", "tsv", "--out", OUT_DIR_REL],
        Norm::Raw,
    );
    let tsv = rs.text("dist/anki/cdcp_bank.tsv");
    // Both degrade to the bare letter rather than to the wrong choice — and
    // BOTH SHIP. An unanswerable card is not an error to this gate, which is
    // one of the things it cannot decide. bd-anki-answerless-card-ships-urz3.
    assert!(tsv.contains("\tD\t"), "{tsv}");
    assert!(tsv.contains("\tE\t"), "{tsv}");
    assert_eq!(
        rs.code,
        0,
        "the oracle ships an unresolved answer: {}",
        rs.out()
    );
}

// ── case 5: the filters ────────────────────────────────────────────────────

#[test]
fn the_module_filter_is_byte_identical() {
    let f = Fixture::new();
    synthetic_bank(&f);
    let rs = compare(
        "module filter",
        &f,
        &["--module", "2", "--format", "tsv", "--out", OUT_DIR_REL],
        Norm::Raw,
    );
    assert_eq!(rs.code, 0, "{}\n{}", rs.out(), rs.err());
    assert!(rs.out().contains("  cards=2\n"), "{}", rs.out());
}

#[test]
fn the_tag_filter_matches_tags_and_topic_ids_and_a_string_tag_field() {
    let f = Fixture::new();
    synthetic_bank(&f);
    // `tags` as a bare string is normalised to a one-element list by the oracle.
    let by_tag = compare(
        "tag filter (string tags)",
        &f,
        &["--tag", "power", "--format", "tsv", "--out", OUT_DIR_REL],
        Norm::Raw,
    );
    assert!(by_tag.out().contains("  cards=1\n"), "{}", by_tag.out());
    // …and a topic id substring matches too, on a different axis.
    let by_topic = compare(
        "tag filter (topic ids)",
        &f,
        &[
            "--tag",
            "standards",
            "--format",
            "tsv",
            "--out",
            OUT_DIR_REL,
        ],
        Norm::Raw,
    );
    assert!(by_topic.out().contains("  cards=2\n"), "{}", by_topic.out());
}

#[test]
fn an_unseeded_limit_takes_the_alphabetically_first_n_in_both() {
    let f = Fixture::new();
    synthetic_bank(&f);
    let rs = compare(
        "unseeded limit",
        &f,
        &["--limit", "2", "--format", "tsv", "--out", OUT_DIR_REL],
        Norm::Raw,
    );
    assert!(rs.out().contains("  cards=2\n"), "{}", rs.out());
    let tsv = rs.text("dist/anki/cdcp_bank.tsv");
    // Sorted by id, then truncated — so it is the first two ids, not a sample.
    // That is the documented behaviour of `--limit` without `--seed`.
    assert!(tsv.contains("mission-critical"), "{tsv}");
    assert!(!tsv.contains("UPS topology"), "{tsv}");
}

#[test]
fn a_seeded_limit_samples_identically() {
    // THE case that proves the hand-rolled MT19937 is CPython's. If the port's
    // Mersenne Twister, its init_by_array seeding, its getrandbits or its
    // rejection-sampling _randbelow drifted by one draw, this deck would hold a
    // different set of cards while every byte it did write still matched.
    let f = Fixture::new();
    f.seed_live_bank();
    for seed in ["0", "1", "42", "1337"] {
        let rs = compare(
            "seeded limit",
            &f,
            &[
                "--limit",
                "17",
                "--seed",
                seed,
                "--format",
                "tsv",
                "--out",
                OUT_DIR_REL,
            ],
            Norm::Raw,
        );
        assert!(rs.out().contains("  cards=17\n"), "{}", rs.out());
    }
}

#[test]
fn a_limit_larger_than_the_bank_is_a_no_op_in_both() {
    let f = Fixture::new();
    synthetic_bank(&f);
    let rs = compare(
        "limit above size",
        &f,
        &[
            "--limit",
            "1000",
            "--seed",
            "42",
            "--format",
            "tsv",
            "--out",
            OUT_DIR_REL,
        ],
        Norm::Raw,
    );
    // `len(out) > limit` is false, so the shuffle never runs and the order is
    // the plain id sort. A port that shuffled anyway would diverge here.
    assert!(rs.out().contains("  cards=4\n"), "{}", rs.out());
}

// ── case 6: THE RETIRED-ITEM FINDING, measured rather than asserted ────────

#[test]
fn the_shipped_deck_excludes_every_retired_item() {
    // bd-anki-ships-retired-bbdr / bd-fqpp. Closed 2026-08-14: bank and seed42
    // sources keep only status==approved. Measured by subtraction: default
    // export and an export after the retired TOMLs are deleted must be the
    // same TSV. A regression that ships withdrawn cards makes the line counts
    // differ again.
    let f = Fixture::new();
    f.seed_live_bank();

    let mut approved = 0usize;
    let mut retired: Vec<PathBuf> = Vec::new();
    for e in std::fs::read_dir(f.at(ITEMS_DIR_REL)).unwrap().flatten() {
        let body = std::fs::read_to_string(e.path()).unwrap();
        if body.contains("status = \"retired\"") {
            retired.push(e.path());
        } else if body.contains("status = \"approved\"") {
            approved += 1;
        }
    }
    assert!(
        !retired.is_empty(),
        "the live bank holds no retired items, so this case would prove nothing. A finding test \
         that cannot fire is not a finding test."
    );
    assert!(approved > 0, "the live bank holds no approved items either");
    let n_retired = retired.len();

    // Run 1: the whole bank (filter must drop retired).
    let all = compare(
        "retired items (whole bank, filter on)",
        &f,
        &["--format", "tsv", "--out", OUT_DIR_REL],
        Norm::Raw,
    );
    assert!(
        all.out().contains(&format!("  cards={approved}\n")),
        "the deck must carry exactly the {approved} approved items: {}",
        all.out()
    );
    let all_tsv = all.text("dist/anki/cdcp_bank.tsv");

    // Run 2: retired TOMLs removed — must be identical, or the filter leaked.
    for p in &retired {
        std::fs::remove_file(p).unwrap();
    }
    let only_approved = compare(
        "retired items (approved only)",
        &f,
        &["--format", "tsv", "--out", OUT_DIR_REL],
        Norm::Raw,
    );
    assert!(
        only_approved
            .out()
            .contains(&format!("  cards={approved}\n")),
        "the approved-only deck must carry exactly the {approved} approved items: {}",
        only_approved.out()
    );
    let approved_tsv = only_approved.text("dist/anki/cdcp_bank.tsv");

    // THE MEASUREMENT. Default export and approved-only-on-disk must be the
    // same TSV. A leak of even one retired card moves the line count.
    assert_eq!(
        all_tsv, approved_tsv,
        "PINNED FIX (bd-anki-ships-retired-bbdr): default deck must equal the \
         deck built after deleting {n_retired} retired TOMLs. If this fails, \
         a withdrawn item is shipping again."
    );
    assert!(
        all.out().contains(&format!(
            "  {} scanned, {approved} exported\n",
            approved + n_retired
        )),
        "receipt must name both populations: {}",
        all.out()
    );
}

#[test]
fn an_all_retired_bank_is_error_in_both_and_writes_nothing() {
    // p45d / bbdr.1: zero approved is DISTINCT from an empty bank, and must
    // not write a retired deck.
    let f = Fixture::new();
    for i in 0..3 {
        f.put(
            &format!("bank/items/r{i}.toml"),
            &format!(
                "id = \"r{i}\"\nstatus = \"retired\"\nmodule = 1\n\
                 stem = \"retired-only-{i}\"\nchoices = [\"a\"]\ncorrect = \"A\"\n"
            ),
        );
    }
    let rs = compare(
        "all retired bank",
        &f,
        &["--format", "tsv", "--out", OUT_DIR_REL],
        Norm::Raw,
    );
    assert_ne!(rs.code, 0, "all-retired must never ship a deck");
    assert_eq!(rs.err(), "FAIL: zero approved items to export\n");
    assert!(rs.out().is_empty(), "nothing on stdout: {:?}", rs.out());
    assert!(rs.files.is_empty(), "must not write a retired deck");
}

// ── case 7: THE `.apkg` LEG — pinned as unsatisfiable, not skipped ─────────

#[test]
fn the_oracle_apkg_is_not_byte_reproducible_against_itself() {
    // bd-anki-apkg-not-reproducible-e13a.
    //
    // This is the measurement that decides the shape of this whole file. Two
    // runs of THE SAME implementation on THE SAME inputs produce different
    // decks, because the deck embeds int(time.time()) and the zip is
    // DEFLATE-compressed so a one-second shift avalanches. A byte-exact
    // differential on the .apkg is therefore UNSATISFIABLE — not hard, not
    // deferred: impossible against this oracle.
    //
    // Written as a PINNED DEFECT: the day `export_anki.py` is made
    // reproducible, this test goes RED and forces the .apkg leg to be ported
    // and byte-compared like everything else.
    let f = Fixture::new();
    // Two cards is enough; the point is the timestamp, not the volume.
    f.put(
        "bank/items/q01.toml",
        "id = \"q01\"\nmodule = 1\nstem = \"s1\"\nchoices = [\"a\"]\ncorrect = \"A\"\nexplanation = \"e\"\n",
    );
    f.put(
        "bank/items/q02.toml",
        "id = \"q02\"\nmodule = 1\nstem = \"s2\"\nchoices = [\"a\"]\ncorrect = \"A\"\nexplanation = \"e\"\n",
    );

    let mut decks: Vec<Vec<u8>> = Vec::new();
    let mut tsvs: Vec<Vec<u8>> = Vec::new();
    for i in 0..2 {
        let base = f.dir.path().join(format!("apkg-run-{i}"));
        copy_tree(&f.template(), &base);
        let py = python(&base, &["--format", "tsv,apkg", "--out", OUT_DIR_REL]);
        assert_eq!(py.code, 0, "{}\n{}", py.out(), py.err());
        decks.push(py.files["dist/anki/cdcp_bank.apkg"].clone());
        tsvs.push(py.files["dist/anki/cdcp_bank.tsv"].clone());
        if i == 0 {
            // Cross a whole-second boundary so the embedded clock has moved.
            std::thread::sleep(std::time::Duration::from_millis(1_100));
        }
    }

    // The DETERMINISTIC surface is reproducible…
    assert_eq!(
        tsvs[0], tsvs[1],
        "the TSV must be reproducible; if it is not, the port has a second, worse problem"
    );
    // …and the deck is NOT.
    assert_ne!(
        decks[0], decks[1],
        "PINNED DEFECT bd-anki-apkg-not-reproducible-e13a has been FIXED: two runs of the oracle \
         now produce identical .apkg bytes. That is good news and this test is now the thing \
         standing in the way — port the .apkg leg in export_anki.rs, add it to `compare`, and \
         delete this test."
    );
}

#[test]
fn the_rust_refuses_the_apkg_leg_loudly_and_writes_nothing() {
    // The port's ONE deliberate divergence from its oracle, made loud rather
    // than silent: `--format apkg` is ERROR (exit 4) here and success there.
    // A silent stub that wrote a plausible-looking deck would be the worst
    // possible outcome for a shipped learner artifact.
    let f = Fixture::new();
    f.seed_live_bank();
    let base = f.dir.path().join("apkg-refusal");
    copy_tree(&f.template(), &base);
    let base = base.canonicalize().unwrap();
    let rs = rust(&base, &["--format", "tsv,apkg", "--out", OUT_DIR_REL]);
    assert_eq!(
        rs.code,
        4,
        "the refusal must be an ERROR exit, never a pass: {}\n{}",
        rs.out(),
        rs.err()
    );
    assert!(
        rs.err().contains("bd-anki-apkg-not-reproducible-e13a"),
        "the refusal must name the blocking bead: {}",
        rs.err()
    );
    assert!(
        rs.files.is_empty(),
        "the refusal must leave nothing behind, including the TSV leg it could have written: {:?}",
        rs.names()
    );
    assert!(
        !rs.out().contains("export_anki ok"),
        "no success token on the refusal path: {}",
        rs.out()
    );
}

// ── case 8: the README the oracle drops next to the deck ──────────────────

#[test]
fn the_readme_records_the_card_count_and_the_source() {
    let f = Fixture::new();
    synthetic_bank(&f);
    let rs = compare(
        "readme",
        &f,
        &["--format", "tsv", "--out", OUT_DIR_REL],
        Norm::Raw,
    );
    let readme = rs.text("dist/anki/README.txt");
    assert!(readme.contains("Cards: 4\n"), "{readme}");
    assert!(readme.contains("Source: bank\n"), "{readme}");
    assert!(
        readme.contains("Does NOT grant EPI/EXIN certification."),
        "the honesty line is part of the shipped bytes: {readme}"
    );
    assert!(readme.ends_with('\n'));
}

// ── the harness must not be vacuously green ───────────────────────────────

#[test]
fn the_harness_compared_something() {
    // Runs a case itself rather than reading a counter another test may or may
    // not have incremented — test order and parallelism are not a contract, and
    // "0 cases compared" must never report like "all passed".
    let before = COMPARED.load(Ordering::SeqCst);
    let f = Fixture::new();
    synthetic_bank(&f);
    let run = compare(
        "harness self-check",
        &f,
        &["--format", "tsv,csv", "--out", OUT_DIR_REL],
        Norm::Raw,
    );
    assert!(
        COMPARED.load(Ordering::SeqCst) > before,
        "the differential harness compared nothing"
    );
    // INDEPENDENT VERDICT, not merely agreement (bd-differential-shared-blindspot-4qje).
    // The counter proves a comparison HAPPENED; it says nothing about whether
    // the shared answer is right, and a defect present in both sides agrees
    // with itself. This states what the answer IS: a synthetic bank exports
    // cleanly and says so.
    assert_eq!(run.code, 0, "{}", run.out());
    assert!(
        run.out().contains("export_anki ok"),
        "the self-check case must carry the success token, not just agree: {}",
        run.out()
    );
}
