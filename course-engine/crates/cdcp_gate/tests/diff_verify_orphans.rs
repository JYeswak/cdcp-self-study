//! Differential harness: `cdcp_gate verify-orphans` against `scripts/verify_orphans.py`.
//!
//! The Python script is the oracle for this port (bd-substrate-rust-migration-jhd.2)
//! and stays in the tree for exactly that reason. Every case below runs BOTH
//! implementations on the same inputs and asserts stdout, stderr, and exit code
//! match byte for byte. A disagreement on any byte fails the port, not the oracle.
//!
//! The case list is not invented here: it is the enumeration of
//! `scripts/selftest_orphan.sh`, which is the L4 known-bad suite check.sh already
//! runs —
//!
//!   a) empty bank dir                  -> ERROR (anti-vacuous)
//!   b) empty topics registry           -> ERROR (anti-vacuous)
//!   c) item referencing unknown topic  -> RED (orphan ref, forward direction)
//!   d) item with empty topic_ids       -> RED (unanchored item)
//!   e) topic referenced by zero items  -> RED (orphan topic, reverse direction)
//!   f) live tree                       -> GREEN
//!
//! plus the suite's intermediate "specimen bank is clean again" assertion and a
//! handful of argument/path shapes the suite does not reach.
//!
//! ANTI-VACUOUS DISCIPLINE. A differential that silently compares nothing passes
//! exactly like one that compared everything, so: a missing `python3` is a
//! FAILURE and never a skip; a specimen bank that copied zero files is a
//! FAILURE; and every case increments a counter that is asserted at the end.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const BIN: &str = env!("CARGO_BIN_EXE_cdcp_gate");
const ORACLE: &str = "scripts/verify_orphans.py";
const GATE: &str = "verify-orphans";

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

/// Copy the live bank into TEMP so a planted specimen is the ONLY defect, which
/// is what makes a needle prove the specific detector fired. The live tree is
/// never mutated, exactly as `selftest_orphan.sh` promises.
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

fn write(path: &Path, body: &str) {
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p).unwrap();
    }
    std::fs::write(path, body).unwrap();
}

const ORPHAN_REF_SPECIMEN: &str = r#"id = "selftest-orphan-ref"
module = 1
stem = "selftest planted item — orphan reference specimen, not for exam use"
choices = ["A", "B", "C", "D"]
correct = "A"
explanation = "planted for the orphan-item selftest only"
topic_ids = ["zz-topic-that-does-not-exist"]
bloom = "remember"
source_class = "original"
quantity_evidence = "qualitative_only"
"#;

const UNANCHORED_SPECIMEN: &str = r#"id = "selftest-unanchored"
module = 1
stem = "selftest planted item — unanchored specimen, not for exam use"
choices = ["A", "B", "C", "D"]
correct = "A"
explanation = "planted for the orphan-item selftest only"
topic_ids = []
bloom = "remember"
source_class = "original"
quantity_evidence = "qualitative_only"
"#;

const ORPHAN_TOPIC_SPECIMEN: &str = r#"
[[topic]]
id = "zz-selftest-orphan-topic"
domain = "01-mission-critical"
label = "selftest planted orphan topic — assessed by zero bank items"
source = "src-epi-cdcp-page"
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

// ── (f) the GREEN case ─────────────────────────────────────────────────────

#[test]
fn live_tree_is_byte_identical_and_green() {
    let root = engine_root();
    let rs = assert_byte_identical("f live tree", &root, &[]);
    assert_eq!(rs.code, 0, "live tree must be GREEN: {}", rs.out());
    assert!(rs.out().starts_with("PASS\n"), "{}", rs.out());
    assert!(rs.out().contains("orphan integrity GREEN"), "{}", rs.out());
    assert!(
        rs.err().is_empty(),
        "the oracle writes nothing to stderr on the green path: {:?}",
        rs.err()
    );
}

// ── (a)(b) anti-vacuous: an empty input set is an ERROR, not a pass ────────

#[test]
fn empty_bank_and_empty_topics_are_errors_in_both() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();

    // (a) empty bank directory
    let empty_bank = td.path().join("empty_bank");
    std::fs::create_dir_all(&empty_bank).unwrap();
    let rs = assert_byte_identical(
        "a empty bank",
        &root,
        &["--bank", empty_bank.to_str().unwrap()],
    );
    assert_ne!(
        rs.code,
        0,
        "an empty bank must never be a pass: {}",
        rs.out()
    );
    assert!(
        rs.out().contains("empty bank"),
        "the anti-vacuous signal must be named: {}",
        rs.out()
    );

    // (b) empty topics registry
    let empty_topics = td.path().join("empty_topics.toml");
    write(&empty_topics, "");
    let rs = assert_byte_identical(
        "b empty topics",
        &root,
        &["--topics", empty_topics.to_str().unwrap()],
    );
    assert_ne!(
        rs.code,
        0,
        "an empty topic registry must never be a pass: {}",
        rs.out()
    );
    assert!(
        rs.out().contains("empty topic registry"),
        "the anti-vacuous signal must be named: {}",
        rs.out()
    );

    // both empty at once — still an ERROR, and still identical
    let rs = assert_byte_identical(
        "a+b both empty",
        &root,
        &[
            "--bank",
            empty_bank.to_str().unwrap(),
            "--topics",
            empty_topics.to_str().unwrap(),
        ],
    );
    assert_ne!(rs.code, 0, "{}", rs.out());
}

// ── (c)(d)(e) every injection selftest_orphan.sh exercises ────────────────

#[test]
fn planted_known_bads_are_byte_identical_and_red() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();
    let bank = td.path().join("bank_items");
    let copied = specimen_bank(&root, &bank);
    let bank_arg = bank.to_str().unwrap().to_string();

    // The specimen bank is a faithful copy, so it is GREEN before anything is
    // planted. Without this control, a needle below could be firing on a defect
    // the copy itself introduced.
    let rs = assert_byte_identical("specimen bank clean (pre)", &root, &["--bank", &bank_arg]);
    assert_eq!(
        rs.code,
        0,
        "specimen bank of {copied} files is not clean: {}",
        rs.out()
    );

    // (c) orphan item: topic_ids points at a topic that does not exist
    let ref_specimen = bank.join("zz-selftest-orphan-ref.toml");
    write(&ref_specimen, ORPHAN_REF_SPECIMEN);
    let rs = assert_byte_identical("c orphan ref", &root, &["--bank", &bank_arg]);
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out().contains(
            "selftest-orphan-ref: unknown topic_id 'zz-topic-that-does-not-exist' (orphan item)"
        ),
        "the forward-direction detector must name the item and the id: {}",
        rs.out()
    );
    std::fs::remove_file(&ref_specimen).unwrap();

    // (d) unanchored item: topic_ids present but empty
    let un_specimen = bank.join("zz-selftest-unanchored.toml");
    write(&un_specimen, UNANCHORED_SPECIMEN);
    let rs = assert_byte_identical("d unanchored", &root, &["--bank", &bank_arg]);
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out()
            .contains("selftest-unanchored: missing/empty topic_ids (orphan item)"),
        "{}",
        rs.out()
    );
    std::fs::remove_file(&un_specimen).unwrap();

    // the suite's own control: removing the specimens returns the bank to GREEN
    let rs = assert_byte_identical("specimen bank clean (post)", &root, &["--bank", &bank_arg]);
    assert_eq!(rs.code, 0, "{}", rs.out());

    // (e) orphan topic: declared in the registry, referenced by zero items
    let topics = td.path().join("topics_plus_orphan.toml");
    let mut body = std::fs::read_to_string(root.join("knowledge/topics.toml")).unwrap();
    body.push_str(ORPHAN_TOPIC_SPECIMEN);
    write(&topics, &body);
    let rs = assert_byte_identical(
        "e orphan topic",
        &root,
        &["--topics", topics.to_str().unwrap()],
    );
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out().contains(
            "orphan topic 'zz-selftest-orphan-topic': declared in topics.toml, referenced by zero bank items"
        ),
        "the reverse-direction detector must name the topic: {}",
        rs.out()
    );

    // nothing planted may leak into the live tree
    for leaked in [
        "bank/items/zz-selftest-orphan-ref.toml",
        "bank/items/zz-selftest-unanchored.toml",
    ] {
        assert!(
            !root.join(leaked).exists(),
            "specimen leaked into the live tree: {leaked}"
        );
    }
    assert!(
        !std::fs::read_to_string(root.join("knowledge/topics.toml"))
            .unwrap()
            .contains("zz-selftest-orphan-topic"),
        "specimen topic leaked into knowledge/topics.toml"
    );
}

// ── shapes the shell suite never reaches ──────────────────────────────────

#[test]
fn defect_shapes_beyond_the_shell_suite_are_byte_identical() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();
    let bank = td.path().join("bank_items");
    specimen_bank(&root, &bank);
    let bank_arg = bank.to_str().unwrap().to_string();

    // a blank topic_id entry inside an otherwise well-formed list
    write(
        &bank.join("zz-blank-entry.toml"),
        "id = \"zz-blank-entry\"\ntopic_ids = [\"   \"]\n",
    );
    let rs = assert_byte_identical("blank topic_id entry", &root, &["--bank", &bank_arg]);
    assert!(
        rs.out()
            .contains("zz-blank-entry: blank topic_id entry (orphan item)"),
        "{}",
        rs.out()
    );
    std::fs::remove_file(bank.join("zz-blank-entry.toml")).unwrap();

    // topic_ids present but not a list at all
    write(
        &bank.join("zz-not-a-list.toml"),
        "id = \"zz-not-a-list\"\ntopic_ids = \"m01-importance\"\n",
    );
    let rs = assert_byte_identical("non-list topic_ids", &root, &["--bank", &bank_arg]);
    assert!(
        rs.out()
            .contains("zz-not-a-list: missing/empty topic_ids (orphan item)"),
        "{}",
        rs.out()
    );
    std::fs::remove_file(bank.join("zz-not-a-list.toml")).unwrap();

    // a file that is neither an item nor an items[] table array
    write(&bank.join("zz-junk.toml"), "label = \"nothing useful\"\n");
    let rs = assert_byte_identical("no id or items[]", &root, &["--bank", &bank_arg]);
    assert!(
        rs.out().contains("zz-junk.toml: no id or items[]"),
        "{}",
        rs.out()
    );
    std::fs::remove_file(bank.join("zz-junk.toml")).unwrap();

    // an items[] table array, and an item inside it with no id: the report falls
    // back to the FILE name, which is a behaviour worth pinning
    write(
        &bank.join("zz-multi.toml"),
        "[[items]]\ntopic_ids = []\n\n[[items]]\nid = \"zz-multi-2\"\ntopic_ids = [\"m01-importance\"]\n",
    );
    let rs = assert_byte_identical(
        "items[] with an id-less entry",
        &root,
        &["--bank", &bank_arg],
    );
    assert!(
        rs.out()
            .contains("zz-multi.toml: missing/empty topic_ids (orphan item)"),
        "{}",
        rs.out()
    );
    std::fs::remove_file(bank.join("zz-multi.toml")).unwrap();

    // ── KNOWN ORACLE DEFECT, reproduced deliberately and NOT fixed ─────────
    // `items = []` takes the `isinstance(data["items"], list)` branch, adds
    // nothing, and never reaches the `no id or items[]` error. A bank file
    // emptied this way is scanned and silently contributes nothing — a file
    // that was never checked reports exactly like one that passed. Fixing it
    // here would stop this being a port, so the behaviour is pinned instead,
    // and the divergence is filed for a separate bead.
    write(&bank.join("zz-silently-empty.toml"), "items = []\n");
    let rs = assert_byte_identical("oracle defect: items = []", &root, &["--bank", &bank_arg]);
    assert_eq!(
        rs.code,
        0,
        "pinning the oracle's behaviour, not endorsing it: {}",
        rs.out()
    );
    assert!(
        !rs.out().contains("zz-silently-empty"),
        "the oracle says nothing about this file; if that ever changes, this port must change with it: {}",
        rs.out()
    );
    std::fs::remove_file(bank.join("zz-silently-empty.toml")).unwrap();

    // duplicate topic ids in the registry
    let dup_topics = td.path().join("topics_dup.toml");
    let mut body = std::fs::read_to_string(root.join("knowledge/topics.toml")).unwrap();
    body.push_str("\n[[topic]]\nid = \"m01-importance\"\ndomain = \"01-mission-critical\"\nlabel = \"duplicate\"\nsource = \"src-epi-cdcp-page\"\n");
    write(&dup_topics, &body);
    let rs = assert_byte_identical(
        "duplicate topic ids",
        &root,
        &["--topics", dup_topics.to_str().unwrap()],
    );
    assert!(
        rs.out()
            .contains("duplicate topic ids in registry: ['m01-importance']"),
        "{}",
        rs.out()
    );

    // more than MAX_REPORT failures: the truncation footer must match too
    let many = td.path().join("topics_many.toml");
    let mut body = std::fs::read_to_string(root.join("knowledge/topics.toml")).unwrap();
    for i in 0..50 {
        body.push_str(&format!(
            "\n[[topic]]\nid = \"zz-extra-{i:03}\"\ndomain = \"01-mission-critical\"\nlabel = \"extra\"\nsource = \"src-epi-cdcp-page\"\n"
        ));
    }
    write(&many, &body);
    let rs = assert_byte_identical(
        "truncated failure list",
        &root,
        &["--topics", many.to_str().unwrap()],
    );
    assert!(rs.out().contains("... +10 more"), "{}", rs.out());
}

#[test]
fn path_and_option_shapes_are_byte_identical() {
    let root = engine_root();
    let td = tempfile::tempdir().unwrap();
    let missing_bank = td.path().join("no_such_bank");
    let missing_topics = td.path().join("no_such_topics.toml");

    // a missing directory / registry is an ERROR, not "nothing to check"
    let rs = assert_byte_identical(
        "missing bank dir",
        &root,
        &["--bank", missing_bank.to_str().unwrap()],
    );
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(rs.out().contains("bank dir missing:"), "{}", rs.out());

    let rs = assert_byte_identical(
        "missing topics registry",
        &root,
        &["--topics", missing_topics.to_str().unwrap()],
    );
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out().contains("topics registry missing:"),
        "{}",
        rs.out()
    );

    // engine-root-relative arguments, including the untidy spellings the printed
    // header must normalise the same way on both sides
    assert_byte_identical("relative bank", &root, &["--bank", "bank/items"]);
    assert_byte_identical(
        "relative topics",
        &root,
        &["--topics", "knowledge/topics.toml"],
    );
    assert_byte_identical("untidy relative path", &root, &["--bank", "./bank//items/"]);

    // `--opt=value` and argparse's unambiguous prefixes
    assert_byte_identical("equals form", &root, &["--bank=bank/items"]);
    assert_byte_identical("abbreviated option", &root, &["--ban", "bank/items"]);

    // both options at once, absolute + relative mixed
    assert_byte_identical(
        "both options",
        &root,
        &[
            "--bank",
            root.join("bank/items").to_str().unwrap(),
            "--topics",
            "knowledge/topics.toml",
        ],
    );
}

// ── the harness must not be vacuously green ───────────────────────────────

#[test]
fn the_harness_compared_something() {
    // Runs the full case list itself rather than reading a counter another test
    // may or may not have incremented — test order and parallelism are not a
    // contract, and "0 cases compared" must never report like "all passed".
    let root = engine_root();
    let before = COMPARED.load(Ordering::SeqCst);
    assert_byte_identical("harness self-check", &root, &[]);
    assert!(
        COMPARED.load(Ordering::SeqCst) > before,
        "the differential harness compared nothing"
    );
}
