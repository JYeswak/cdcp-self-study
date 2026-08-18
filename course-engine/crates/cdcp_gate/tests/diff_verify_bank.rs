//! Differential harness: `cdcp_gate verify-bank` vs `scripts/verify_bank.py`.
//!
//! Every case here runs BOTH implementations against the SAME tree and asserts
//! stdout, stderr, and exit code are identical byte for byte. The Python stays
//! in the tree as the oracle for exactly this purpose; if the two disagree on a
//! single byte, the Rust is wrong.
//!
//! Fixture trees are built in a tempdir with a copy of the oracle at
//! `<tmp>/scripts/verify_bank.py`, because the oracle derives its own root from
//! `Path(__file__).resolve().parents[1]` and ignores the working directory. The
//! Rust side is pointed at the same tree with `--root`.
//!
//! Anti-vacuous: a missing `python3` is a hard FAILURE here, never a skip. A
//! differential suite that silently ran zero comparisons reports exactly like
//! one that ran them all and found no difference — which is the whole defect
//! this harness exists to rule out.

use std::path::{Path, PathBuf};
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_cdcp_gate");

fn engine_root() -> PathBuf {
    cdcp_gate::root::resolve(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

/// The oracle, run against `root`'s copy of itself.
fn python(root: &Path) -> Run {
    let script = root.join("scripts/verify_bank.py");
    assert!(script.is_file(), "fixture is missing the oracle copy");
    capture(Command::new("python3").arg(&script))
}

/// The port, run against the same tree.
fn rust(root: &Path) -> Run {
    capture(Command::new(BIN).arg("--root").arg(root).arg("verify-bank"))
}

/// The whole acceptance bar, in one place.
fn assert_byte_identical(label: &str, root: &Path) -> Run {
    let py = python(root);
    let rs = rust(root);
    assert_eq!(
        py.stdout, rs.stdout,
        "[{label}] STDOUT differs\n--- python ---\n{}\n--- rust ---\n{}",
        py.stdout, rs.stdout
    );
    assert_eq!(
        py.stderr, rs.stderr,
        "[{label}] STDERR differs\n--- python ---\n{}\n--- rust ---\n{}",
        py.stderr, rs.stderr
    );
    assert_eq!(
        py.code, rs.code,
        "[{label}] EXIT CODE differs: python {} vs rust {}",
        py.code, rs.code
    );
    py
}

// ── fixture builder ────────────────────────────────────────────────────────

struct Fixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
}

impl Fixture {
    /// A tree the oracle can run in: its own copy of the script, an empty
    /// `bank/items/`, a one-topic registry, and a policy whose floors a tiny
    /// fixture pool can actually clear.
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_path_buf();
        let f = Fixture { _dir: dir, root };
        std::fs::create_dir_all(f.root.join("bank/items")).unwrap();
        std::fs::create_dir_all(f.root.join("scripts")).unwrap();
        std::fs::copy(
            engine_root().join("scripts/verify_bank.py"),
            f.root.join("scripts/verify_bank.py"),
        )
        .expect("copy the oracle into the fixture");
        f.write(
            "knowledge/topics.toml",
            "[[topic]]\nid = \"t-one\"\n\n[[topic]]\nid = \"t-two\"\n",
        );
        f.write(
            "knowledge/bank_policy.toml",
            "exam_n_items = 1\npool_min_items = 2\n\
             [[domain_min]]\nmodule = 1\nmin_items = 1\n",
        );
        f
    }

    fn write(&self, rel: &str, body: &str) {
        let p = self.root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    }

    fn remove_dir(&self, rel: &str) {
        std::fs::remove_dir_all(self.root.join(rel)).unwrap();
    }

    fn check(&self, label: &str) -> Run {
        assert_byte_identical(label, &self.root)
    }
}

/// A schema-clean, APPROVED item, as one `[[items]]` table.
///
/// The explicit `status` is load-bearing (bd-8exw): every floor in this gate is
/// measured against `status == "approved"`, and an absent status is `draft`, so
/// a fixture that omitted it would be counted out of every floor it means to
/// exercise.
fn good_item(id: &str, module: i64, correct: &str, topic: &str) -> String {
    item_with_status(id, module, correct, topic, "approved")
}

/// The same item at an arbitrary lifecycle status. `retired` here is the
/// known-bad injection: it changes the drawable pool WITHOUT deleting a file.
fn item_with_status(id: &str, module: i64, correct: &str, topic: &str, status: &str) -> String {
    format!(
        "[[items]]\nid = {id:?}\nmodule = {module}\nstem = \"stem for {id}\"\n\
         choices = [\"alpha\", \"beta\", \"gamma\", \"delta\"]\ncorrect = {correct:?}\n\
         explanation = \"an explanation of sufficient length\"\ntopic_ids = [{topic:?}]\n\
         bloom = \"apply\"\nsource_class = \"original\"\n\
         quantity_evidence = \"qualitative_only\"\nstatus = {status:?}\n\n"
    )
}

/// `n` clean items whose `correct` letters cycle through `letters`.
fn pool(n: usize, letters: &[&str]) -> String {
    let mut s = String::new();
    for i in 0..n {
        s.push_str(&good_item(
            &format!("i-{i:03}"),
            1,
            letters[i % letters.len()],
            "t-one",
        ));
    }
    s
}

// ── the suite ──────────────────────────────────────────────────────────────

#[test]
fn python3_is_present_because_a_skipped_differential_is_a_fooled_certificate() {
    let out = Command::new("python3")
        .arg("--version")
        .output()
        .expect("python3 must be installed: the oracle cannot be skipped");
    assert!(out.status.success(), "python3 --version failed");
}

/// Case 1 — the live repo tree, currently GREEN.
#[test]
fn live_repo_tree_is_byte_identical() {
    let root = engine_root();
    let py = python(&root);
    let rs = rust(&root);
    assert_eq!(py.stdout, rs.stdout, "live tree STDOUT differs");
    assert_eq!(py.stderr, rs.stderr, "live tree STDERR differs");
    assert_eq!(py.code, rs.code, "live tree EXIT CODE differs");
    assert_eq!(
        py.code, 0,
        "the live tree is expected GREEN:\n{}",
        py.stdout
    );
    assert!(py.stdout.starts_with("PASS\n"), "{}", py.stdout);
    // The distribution and the module map are where a HashMap port would break.
    assert!(
        py.stdout.contains("  correct_dist(approved)={'A': "),
        "{}",
        py.stdout
    );
    // BOTH populations, and they differ: 804 files, 779 drawable. Asserting
    // only the file count is exactly what let bd-8exw hide for a day.
    assert!(
        py.stdout
            .contains("  items=804 scanned, 779 approved (floors count the approved pool only)\n"),
        "{}",
        py.stdout
    );
    // m14 carries 44 files and 42 approved — the pair the old single map
    // collapsed into one number.
    assert!(py.stdout.contains("14: 42, 15: 39}"), "{}", py.stdout);
    assert!(py.stdout.contains("14: 44, 15: 39}"), "{}", py.stdout);
    assert!(
        py.stdout
            .contains("  domain_floors=15 checked (approved pool)\n"),
        "live bank_policy.toml carries 15 [[domain_min]] rows:\n{}",
        py.stdout
    );
}

/// Case 3 — anti-vacuous. An empty `bank/items/` is an ERROR in both, never a
/// pass: a bank that was never populated must not report like a clean one.
#[test]
fn empty_items_directory_is_an_error_in_both_never_a_pass() {
    let f = Fixture::new();
    let run = f.check("empty-items-dir");
    assert_ne!(run.code, 0, "an empty bank passed:\n{}", run.stdout);
    assert!(run.stdout.starts_with("FAIL\n"), "{}", run.stdout);
    assert!(
        run.stdout.contains("  - zero items loaded\n"),
        "{}",
        run.stdout
    );
    assert!(
        run.stdout.contains(
            "  - pool too small: 0 approved < pool_min_items 2 (0 scanned, 0 not approved; need ≥2× exam size 1)\n"
        ),
        "{}",
        run.stdout
    );
}

/// ANTI-VACUOUS (bd-bank-zero-domain-floors-vacuous-o80a). A policy that lost
/// its `[[domain_min]]` table on an otherwise-green bank is RED naming the
/// condition. Both sides must agree on the bytes.
#[test]
fn zero_domain_min_rows_on_a_non_empty_bank_is_red_in_both() {
    let f = Fixture::new();
    f.write(
        "knowledge/bank_policy.toml",
        "exam_n_items = 1\npool_min_items = 2\n",
    );
    f.write(
        "bank/items/pool.toml",
        &format!(
            "{}{}",
            good_item("i-a", 1, "A", "t-one"),
            good_item("i-b", 1, "B", "t-one"),
        ),
    );
    let run = f.check("zero-domain-min-non-empty-bank");
    assert_ne!(run.code, 0, "{}", run.stdout);
    assert_eq!(
        run.stdout,
        "FAIL\n  - zero [[domain_min]] floors while bank/items is non-empty \
         (2 scanned; vacuous domain floors are ERROR)\n"
    );
}

/// Complementary empty-bank path: zero domain floors + zero items is the
/// existing empty-bank RED, not this finding.
#[test]
fn zero_domain_min_rows_on_an_empty_bank_stays_the_empty_bank_path_in_both() {
    let f = Fixture::new();
    f.write(
        "knowledge/bank_policy.toml",
        "exam_n_items = 1\npool_min_items = 2\n",
    );
    let run = f.check("zero-domain-min-empty-bank");
    assert_ne!(run.code, 0, "{}", run.stdout);
    assert!(
        run.stdout.contains("  - zero items loaded\n"),
        "{}",
        run.stdout
    );
    assert!(
        !run.stdout.contains("vacuous domain floors"),
        "empty-bank path must not also name the domain-floor rule:\n{}",
        run.stdout
    );
}

#[test]
fn missing_items_directory_is_byte_identical() {
    let f = Fixture::new();
    f.remove_dir("bank/items");
    let run = f.check("missing-items-dir");
    assert_eq!(run.stdout, "FAIL: bank/items/ missing\n");
    assert_ne!(run.code, 0);
}

#[test]
fn missing_topics_registry_is_byte_identical() {
    let f = Fixture::new();
    std::fs::remove_file(f.root.join("knowledge/topics.toml")).unwrap();
    f.write("bank/items/pool.toml", &pool(4, &["A", "B"]));
    let run = f.check("missing-topics-registry");
    assert_eq!(run.stdout, "FAIL: knowledge/topics.toml missing\n");
    assert_ne!(run.code, 0);
}

#[test]
fn topics_registry_with_zero_ids_is_byte_identical() {
    let f = Fixture::new();
    f.write("knowledge/topics.toml", "# no ids at all\n[[topic]]\n");
    f.write("bank/items/pool.toml", &pool(4, &["A", "B"]));
    let run = f.check("zero-topic-ids");
    assert!(
        run.stdout.contains("  - topics.toml has zero topic ids\n"),
        "{}",
        run.stdout
    );
    assert_ne!(run.code, 0);
}

/// Case 4a — a clean pool. Proves the PASS block, including the `correct_dist`
/// and `modules` dict reprs, matches character for character.
#[test]
fn a_clean_fixture_pool_passes_identically() {
    let f = Fixture::new();
    f.write(
        "bank/items/pool.toml",
        &format!(
            "{}{}{}{}",
            good_item("i-a", 1, "A", "t-one"),
            good_item("i-b", 2, "B", "t-two"),
            good_item("i-c", 2, "C", "t-one"),
            good_item("i-d", 3, "D", "t-two"),
        ),
    );
    let run = f.check("clean-pool");
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    assert!(
        run.stdout
            .contains("  domain_floors=1 checked (approved pool)\n"),
        "{}",
        run.stdout
    );
    assert!(
        run.stdout
            .contains("  correct_dist(approved)={'A': 1, 'B': 1, 'C': 1, 'D': 1}\n"),
        "{}",
        run.stdout
    );
    assert!(
        run.stdout
            .contains("  modules(approved)={1: 1, 2: 2, 3: 1}\n"),
        "{}",
        run.stdout
    );
    assert!(
        run.stdout
            .contains("  modules(scanned)={1: 1, 2: 2, 3: 1}\n"),
        "{}",
        run.stdout
    );
}

/// Case 4b — a malformed/missing field of every kind the oracle checks, in one
/// pool, so the ORDER of the findings is compared too.
#[test]
fn every_malformed_field_reports_identically() {
    let f = Fixture::new();
    let body = concat!(
        // blank stem, three choices, correct out of range, short explanation
        "[[items]]\nid = \"bad-one\"\nmodule = 1\nstem = \"   \"\n",
        "choices = [\"a\", \"b\", \"c\"]\ncorrect = \"E\"\nexplanation = \"short\"\n",
        "topic_ids = [\"t-one\"]\nbloom = \"apply\"\nsource_class = \"original\"\n",
        "quantity_evidence = \"qualitative_only\"\nstatus = \"approved\"\n\n",
        // missing topic_ids, unknown topic, bad source_class/qe/bloom/module
        "[[items]]\nid = \"bad-two\"\nstem = \"a stem\"\n",
        "choices = [\"a\", \"b\", \"\", \"d\"]\ncorrect = \"B\"\n",
        "explanation = \"an explanation of sufficient length\"\n",
        "bloom = \"memorise\"\nsource_class = \"derived\"\n",
        "quantity_evidence = \"vibes\"\nmodule = \"not-a-number\"\nstatus = \"approved\"\n\n",
        // unknown topic id, absent optional fields entirely
        "[[items]]\nid = \"bad-three\"\nmodule = 2\nstem = \"another stem\"\n",
        "choices = [\"a\", \"b\", \"c\", \"d\"]\ncorrect = \"C\"\n",
        "explanation = \"an explanation of sufficient length\"\n",
        "topic_ids = [\"t-nope\", \"t-one\"]\nbloom = \"apply\"\n",
        "source_class = \"original\"\nquantity_evidence = \"qualitative_only\"\nstatus = \"approved\"\n\n",
    );
    f.write("bank/items/pool.toml", body);
    let run = f.check("malformed-fields");
    assert_ne!(run.code, 0);
    for expected in [
        "  - bad-one: empty stem\n",
        "  - bad-one: choices must be length 4\n",
        "  - bad-one: correct must be A-D, got 'E'\n",
        "  - bad-one: explanation too short\n",
        "  - bad-two: empty choice text\n",
        "  - bad-two: topic_ids required\n",
        "  - bad-two: source_class must be original, got 'derived'\n",
        "  - bad-two: bad quantity_evidence 'vibes'\n",
        "  - bad-two: bad bloom 'memorise'\n",
        "  - bad-two: bad module 'not-a-number'\n",
        "  - bad-three: unknown topic_id 't-nope'\n",
    ] {
        assert!(
            run.stdout.contains(expected),
            "missing {expected:?} in:\n{}",
            run.stdout
        );
    }
}

/// An absent `module` renders as Python's `None`, not as an empty string.
#[test]
fn a_missing_module_renders_as_python_none() {
    let f = Fixture::new();
    let mut body = pool(2, &["A", "B"]);
    body.push_str(
        "[[items]]\nid = \"no-mod\"\nstem = \"a stem\"\n\
         choices = [\"a\", \"b\", \"c\", \"d\"]\ncorrect = \"C\"\n\
         explanation = \"an explanation of sufficient length\"\n\
         topic_ids = [\"t-one\"]\nbloom = \"apply\"\nsource_class = \"original\"\n\
         quantity_evidence = \"qualitative_only\"\nstatus = \"approved\"\n\n",
    );
    f.write("bank/items/pool.toml", &body);
    let run = f.check("missing-module");
    assert!(
        run.stdout.contains("  - no-mod: bad module None\n"),
        "{}",
        run.stdout
    );
}

/// Case 4c — a missing `id` is reported against the FILE NAME, not an item id.
#[test]
fn a_missing_id_is_reported_by_filename_identically() {
    let f = Fixture::new();
    f.write("bank/items/pool.toml", &pool(2, &["A", "B"]));
    f.write(
        "bank/items/nameless.toml",
        "[[items]]\nmodule = 1\nstem = \"orphaned\"\n\
         choices = [\"a\", \"b\", \"c\", \"d\"]\ncorrect = \"A\"\n\
         explanation = \"an explanation of sufficient length\"\n\
         topic_ids = [\"t-one\"]\nbloom = \"apply\"\nsource_class = \"original\"\n\
         quantity_evidence = \"qualitative_only\"\nstatus = \"approved\"\n",
    );
    let run = f.check("missing-id");
    assert!(
        run.stdout.contains("  - nameless.toml: missing id\n"),
        "{}",
        run.stdout
    );
}

/// A file that is neither a single item nor an `items[]` array.
#[test]
fn a_file_with_neither_id_nor_items_reports_identically() {
    let f = Fixture::new();
    f.write("bank/items/pool.toml", &pool(2, &["A", "B"]));
    f.write("bank/items/junk.toml", "note = \"nothing to see\"\n");
    let run = f.check("no-id-or-items");
    assert!(
        run.stdout.contains("  - junk.toml: no id or items[]\n"),
        "{}",
        run.stdout
    );
}

/// Anti-vacuous at FILE granularity (bd-0czh, the class sweep of bd-2kr).
///
/// `items = []` takes the `isinstance(data["items"], list)` branch and adds
/// nothing, so it can never reach the `no id or items[]` leg above — Python's
/// `elif` cannot run once the `if` has. Before the fix this file was scanned,
/// contributed nothing, and was never named, on BOTH sides, at exit 0.
///
/// Note what the aggregate does here: `pool.toml` carries two items, so `n = 2`
/// clears `pool_min_items = 2` and the whole-bank `zero items loaded` check can
/// never fire. The healthy total is exactly what hid a file that was never
/// checked, which is why this asserts on the NAME and not on a count.
#[test]
fn a_file_whose_items_yield_nothing_is_named_and_red_in_both() {
    let f = Fixture::new();
    f.write("bank/items/pool.toml", &pool(2, &["A", "B"]));
    f.write("bank/items/zz-silently-empty.toml", "items = []\n");
    let run = f.check("items[] yielding zero items");
    assert_ne!(
        run.code, 0,
        "a bank file that contributed nothing must never be a pass:\n{}",
        run.stdout
    );
    assert!(
        run.stdout.contains(
            "  - zz-silently-empty.toml: items[] yielded zero items (vacuous file scan is ERROR)\n"
        ),
        "the file that yielded nothing must be named:\n{}",
        run.stdout
    );
    assert!(
        !run.stdout.contains("zero items loaded"),
        "the aggregate check is satisfied by the other file — if it fired, this \
         case would be testing the whole-bank rule instead of the file rule:\n{}",
        run.stdout
    );
}

/// The known-GOOD leg the bd-0czh fix must not break, stated separately from the
/// `id = …` shape test below because it is the leg a too-wide fix would eat: a
/// file with NO `items` key never takes the list branch, so it is never named.
#[test]
fn a_single_item_id_file_is_untouched_by_the_zero_yield_rule() {
    let f = Fixture::new();
    for (name, id) in [("a-first.toml", "i-one"), ("b-second.toml", "i-two")] {
        f.write(
            &format!("bank/items/{name}"),
            &good_item(id, 1, "A", "t-one").replace("[[items]]\n", ""),
        );
    }
    let run = f.check("single-item `id =` files survive the fix");
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    assert!(
        !run.stdout.contains("yielded zero items"),
        "a file with no items key never took the list branch:\n{}",
        run.stdout
    );
}

/// A single-item file (the live bank's shape) loads the same way as an array.
#[test]
fn single_item_files_load_identically_and_sort_by_name() {
    let f = Fixture::new();
    for (name, id) in [("b-second.toml", "i-two"), ("a-first.toml", "i-one")] {
        f.write(
            &format!("bank/items/{name}"),
            &good_item(id, 1, "A", "t-one").replace("[[items]]\n", ""),
        );
    }
    let run = f.check("single-item-files");
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    assert!(
        run.stdout
            .contains("  items=2 scanned, 2 approved (floors count the approved pool only)\n"),
        "{}",
        run.stdout
    );
}

/// Case 4d — a duplicate id.
#[test]
fn duplicate_ids_report_identically() {
    let f = Fixture::new();
    let mut body = good_item("twin", 1, "A", "t-one");
    // Same id, different stem, so the duplicate-stem rule stays out of the way.
    body.push_str(&good_item("twin", 1, "B", "t-one").replace("stem for twin", "a different stem"));
    body.push_str(&good_item("i-other", 1, "C", "t-one"));
    f.write("bank/items/pool.toml", &body);
    let run = f.check("duplicate-ids");
    assert!(
        run.stdout.contains("  - duplicate ids: ['twin']\n"),
        "{}",
        run.stdout
    );
    assert_ne!(run.code, 0);
}

/// Duplicate stems: the group list, its ordering, and the 100-char clip.
#[test]
fn duplicate_stems_report_identically_including_group_order() {
    let f = Fixture::new();
    let shared = "a stem shared by several items";
    let mut body = String::new();
    for (id, correct) in [("i-c", "A"), ("i-a", "B"), ("i-b", "C")] {
        body.push_str(
            &good_item(id, 1, correct, "t-one").replace(&format!("stem for {id}"), shared),
        );
    }
    // A second, smaller duplicate group sorts after the bigger one.
    for id in ["j-b", "j-a"] {
        body.push_str(
            &good_item(id, 1, "D", "t-one").replace(&format!("stem for {id}"), "another shared"),
        );
    }
    f.write("bank/items/pool.toml", &body);
    let run = f.check("duplicate-stems");
    assert!(
        run.stdout.contains(
            "  - duplicate stem (3 items ['i-c', 'i-a', 'i-b']): 'a stem shared by several items'\n"
        ),
        "{}",
        run.stdout
    );
    let big = run.stdout.find("(3 items").expect("3-item group");
    let small = run.stdout.find("(2 items").expect("2-item group");
    assert!(
        big < small,
        "heaviest group must sort first:\n{}",
        run.stdout
    );
}

/// A stem over 100 characters is clipped with `stem[:100]` before `repr`.
#[test]
fn a_long_duplicate_stem_is_clipped_at_one_hundred_characters() {
    let f = Fixture::new();
    let long: String = "x".repeat(140);
    let mut body = String::new();
    for id in ["k-a", "k-b"] {
        body.push_str(&good_item(id, 1, "A", "t-one").replace(&format!("stem for {id}"), &long));
    }
    f.write("bank/items/pool.toml", &body);
    let run = f.check("long-duplicate-stem");
    assert!(
        run.stdout.contains(&format!("): '{}'\n", "x".repeat(100))),
        "{}",
        run.stdout
    );
}

/// Case 4e — the pool floor, with the non-ASCII `≥` and `×` in the message.
#[test]
fn pool_too_small_reports_identically() {
    let f = Fixture::new();
    f.write(
        "knowledge/bank_policy.toml",
        "exam_n_items = 4\npool_min_items = 40\n\
         [[domain_min]]\nmodule = 1\nmin_items = 1\n",
    );
    f.write("bank/items/pool.toml", &pool(3, &["A", "B", "C"]));
    let run = f.check("pool-too-small");
    assert!(
        run.stdout.contains(
            "  - pool too small: 3 approved < pool_min_items 40 (3 scanned, 0 not approved; need ≥10× exam size 4)\n"
        ),
        "{}",
        run.stdout
    );
}

/// Case 4f — a starved domain.
#[test]
fn domain_minimum_shortfall_reports_identically() {
    let f = Fixture::new();
    f.write(
        "knowledge/bank_policy.toml",
        "exam_n_items = 1\npool_min_items = 2\n\
         [[domain_min]]\nmodule = 9\nmin_items = 5\n\
         [[domain_min]]\nmodule = 2\nmin_items = 1\n",
    );
    f.write(
        "bank/items/pool.toml",
        &format!(
            "{}{}",
            good_item("i-a", 2, "A", "t-one"),
            good_item("i-b", 2, "B", "t-one")
        ),
    );
    let run = f.check("domain-min-shortfall");
    assert!(
        run.stdout.contains(
            "  - module 9: 0 approved items < domain_min 5 (0 scanned, 0 not approved)\n"
        ),
        "{}",
        run.stdout
    );
}

/// Case 4g — letter diversity. 29 of 41 is 70.7%, which must render as `71%`;
/// this is where a naive `{:.0}` of an un-multiplied fraction would break.
#[test]
fn letter_diversity_percentage_rounds_identically() {
    let f = Fixture::new();
    let mut body = String::new();
    for i in 0..29 {
        body.push_str(&good_item(&format!("b-{i:03}"), 1, "B", "t-one"));
    }
    for i in 0..6 {
        body.push_str(&good_item(&format!("a-{i:03}"), 1, "A", "t-one"));
    }
    for i in 0..6 {
        body.push_str(&good_item(&format!("c-{i:03}"), 1, "C", "t-one"));
    }
    f.write("bank/items/pool.toml", &body);
    let run = f.check("letter-diversity-71pct");
    assert!(
        run.stdout
            .contains("  - correct=B is 71% of approved pool (max 70% for diversity)\n"),
        "{}",
        run.stdout
    );
}

/// Both diversity rules at once, on an all-one-letter pool of exactly 40.
#[test]
fn a_monoculture_pool_trips_both_diversity_rules_identically() {
    let f = Fixture::new();
    f.write("bank/items/pool.toml", &pool(40, &["B"]));
    let run = f.check("letter-monoculture");
    assert!(
        run.stdout
            .contains("  - correct=B is 100% of approved pool (max 70% for diversity)\n"),
        "{}",
        run.stdout
    );
    assert!(
        run.stdout
            .contains("  - need at least 3 distinct correct letters in the approved pool\n"),
        "{}",
        run.stdout
    );
}

/// One item under the 40-item threshold, the diversity rules must not fire.
#[test]
fn a_pool_of_thirty_nine_skips_the_diversity_rules_identically() {
    let f = Fixture::new();
    f.write("bank/items/pool.toml", &pool(39, &["B"]));
    let run = f.check("below-diversity-threshold");
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    assert!(!run.stdout.contains("diversity"), "{}", run.stdout);
}

/// Case 4h — MANIFEST drift.
#[test]
fn manifest_item_count_drift_reports_identically() {
    let f = Fixture::new();
    f.write("bank/items/pool.toml", &pool(4, &["A", "B"]));
    f.write("bank/MANIFEST.toml", "item_count = 99\n");
    let run = f.check("manifest-drift");
    assert!(
        run.stdout
            .contains("  - MANIFEST item_count 99 != loaded 4\n"),
        "{}",
        run.stdout
    );
}

/// An `item_count` given as a string prints through `str()`, not `repr()`.
#[test]
fn a_string_manifest_count_renders_without_quotes() {
    let f = Fixture::new();
    f.write("bank/items/pool.toml", &pool(4, &["A", "B"]));
    f.write("bank/MANIFEST.toml", "item_count = \"99\"\n");
    let run = f.check("manifest-string-count");
    assert!(
        run.stdout
            .contains("  - MANIFEST item_count 99 != loaded 4\n"),
        "{}",
        run.stdout
    );
}

/// Case 4i — the 80-line truncation and its `... +N more` tail.
#[test]
fn the_eighty_finding_truncation_is_identical() {
    let f = Fixture::new();
    let mut body = String::new();
    for i in 0..90 {
        body.push_str(
            &good_item(&format!("i-{i:03}"), 1, "A", "t-one").replace("\"apply\"", "\"memorise\""),
        );
    }
    f.write("bank/items/pool.toml", &body);
    let run = f.check("eighty-line-truncation");
    let shown = run.stdout.lines().filter(|l| l.starts_with("  - ")).count();
    assert_eq!(shown, 80, "{}", run.stdout);
    assert!(run.stdout.contains("  ... +"), "{}", run.stdout);
}

/// `fact_policy.toml` replaces the quantity_evidence allowlist wholesale.
#[test]
fn fact_policy_overrides_the_quantity_evidence_allowlist_identically() {
    let f = Fixture::new();
    f.write(
        "knowledge/fact_policy.toml",
        "allowed_quantity_evidence = [\"only_this\"]\n",
    );
    f.write("bank/items/pool.toml", &pool(2, &["A", "B"]));
    let run = f.check("fact-policy-override");
    // The default value is no longer allowed once the policy names its own set.
    assert!(
        run.stdout
            .contains("  - i-000: bad quantity_evidence 'qualitative_only'\n"),
        "{}",
        run.stdout
    );
}

/// `int()` coercion of `module`: a numeric string, a float, and a bool all fold
/// into integer keys in the `modules` map.
#[test]
fn module_coercion_matches_python_int_identically() {
    let f = Fixture::new();
    let body = format!(
        "{}{}{}{}",
        good_item("i-a", 2, "A", "t-one"),
        good_item("i-b", 0, "B", "t-one").replace("module = 0", "module = \"١٢\""),
        good_item("i-c", 0, "C", "t-one").replace("module = 0", "module = 3.9"),
        good_item("i-d", 0, "D", "t-one").replace("module = 0", "module = true"),
    );
    f.write("bank/items/pool.toml", &body);
    let run = f.check("module-int-coercion");
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    assert!(
        run.stdout
            .contains("  modules(approved)={1: 1, 2: 1, 3: 1, 12: 1}\n"),
        "{}",
        run.stdout
    );
}

/// Python's `int(str)` accepts Unicode `Nd` digits. The known-bad mutant
/// removes Unicode support from `unicode_decimal_digit`, while Python still
/// passes this mixed-script fixture; the differential test then REDs instead
/// of certifying the narrowing.
#[test]
fn unicode_nd_blocks_are_byte_identical_and_known_bad_is_red() {
    let starts = [
        0x0660, 0x06f0, 0x0966, 0x09e6, 0x0a66, 0x0ae6, 0x0b66, 0x0be6, 0x0c66,
    ];
    let digits: String = starts
        .into_iter()
        .enumerate()
        .map(|(n, start)| char::from_u32(start + n as u32 + 1).unwrap())
        .collect();
    let f = Fixture::new();
    f.write(
        "knowledge/bank_policy.toml",
        &format!(
            "exam_n_items = \"{digits}\"\npool_min_items = 1\n\
             [[domain_min]]\nmodule = 1\nmin_items = 1\n"
        ),
    );
    f.write("bank/items/pool.toml", &pool(2, &["A", "B"]));
    let run = f.check("unicode-nd-blocks");
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    assert!(
        run.stdout.contains("pool_min=1 exam_n=123456789"),
        "{}",
        run.stdout
    );
}

// The test that used to sit here — `a_falsy_pool_minimum_falls_back_to_the_
// default_identically` — pinned the bd-hw3 defect: `pool_min_items = 0` and
// `exam_n_items = 0` were FALSY, so `int(bp.get(key) or default)` substituted
// 400/40 and the gate reported on floors nobody had configured. The pin existed
// to make repairing that deliberate, not permanent, so it is DELETED rather
// than amended, and the five cases below take its place.

/// bd-hw3, known-bad #1. Both spellings of a zero exam size are now the SAME
/// finding on both sides. `= 0` used to default silently to 40; `= "0"` used to
/// print three lines of a PASS report and then die on `n / exam_n`.
#[test]
fn both_spellings_of_a_zero_exam_size_report_identically_and_fail_closed() {
    for (label, spelling) in [("int-zero", "0"), ("str-zero", "\"0\"")] {
        let f = Fixture::new();
        f.write(
            "knowledge/bank_policy.toml",
            &format!(
                "exam_n_items = {spelling}\npool_min_items = 2\n\
                 [[domain_min]]\nmodule = 1\nmin_items = 1\n"
            ),
        );
        f.write("bank/items/pool.toml", &pool(4, &["A", "B"]));
        let run = f.check(label);
        assert_ne!(run.code, 0, "[{label}] {}", run.stdout);
        assert_eq!(
            run.stdout, "FAIL\n  - bank_policy.toml: exam_n_items must be > 0, got 0\n",
            "[{label}] the two spellings must produce one message"
        );
        assert!(
            run.stderr.is_empty(),
            "[{label}] nothing may raise: {}",
            run.stderr
        );
    }
}

/// bd-hw3, known-bad #2. A negative floor used to DISABLE the pool check —
/// truthy, so it survived `or`, and `n < -1` is never true — and report PASS.
#[test]
fn a_negative_pool_floor_reports_identically_and_does_not_disable_the_check() {
    let f = Fixture::new();
    f.write(
        "knowledge/bank_policy.toml",
        "exam_n_items = 4\npool_min_items = -1\n\
         [[domain_min]]\nmodule = 1\nmin_items = 1\n",
    );
    f.write("bank/items/pool.toml", &pool(4, &["A", "B"]));
    let run = f.check("negative-pool-floor");
    assert_ne!(run.code, 0, "{}", run.stdout);
    assert_eq!(
        run.stdout,
        "FAIL\n  - bank_policy.toml: pool_min_items must be > 0, got -1\n"
    );
}

/// bd-hw3, known-bad #3. Junk in a floor is a finding, not an uncaught
/// `ValueError` whose only trace is a traceback on stderr.
#[test]
fn non_numeric_policy_floors_report_identically_and_do_not_raise() {
    let f = Fixture::new();
    f.write(
        "knowledge/bank_policy.toml",
        "exam_n_items = \"forty\"\npool_min_items = [1]\n\
         [[domain_min]]\nmodule = 1\nmin_items = 1\n",
    );
    f.write("bank/items/pool.toml", &pool(4, &["A", "B"]));
    let run = f.check("non-numeric-policy-floors");
    assert_ne!(run.code, 0, "{}", run.stdout);
    // Source order: pool_min_items is read first, so it reports first.
    assert_eq!(
        run.stdout,
        concat!(
            "FAIL\n",
            "  - bank_policy.toml: pool_min_items must be an integer, got [1]\n",
            "  - bank_policy.toml: exam_n_items must be an integer, got 'forty'\n",
        )
    );
    assert!(run.stderr.is_empty(), "{}", run.stderr);
}

/// bd-hw3, known-GOOD leg. A policy that omits both keys still takes the
/// built-in 400/40 defaults, and a policy that sets them legitimately is
/// honoured. The rebase must not refuse valid configuration — an over-strict
/// gate gets routed around, which is a slower death than no gate.
#[test]
fn legitimate_and_absent_policy_floors_are_still_honoured_identically() {
    let f = Fixture::new();
    f.write(
        "knowledge/bank_policy.toml",
        "# no pool_min/exam_n declared here\n\
         [[domain_min]]\nmodule = 1\nmin_items = 1\n",
    );
    f.write("bank/items/pool.toml", &pool(4, &["A", "B"]));
    let absent = f.check("policy-floors-absent");
    assert!(
        absent
            .stdout
            .contains("  - pool too small: 4 approved < pool_min_items 400 (4 scanned, 0 not approved; need ≥10× exam size 40)\n"),
        "the built-in defaults must still apply:\n{}",
        absent.stdout
    );

    let g = Fixture::new();
    g.write(
        "knowledge/bank_policy.toml",
        "exam_n_items = 2\npool_min_items = 4\n\
         [[domain_min]]\nmodule = 1\nmin_items = 1\n",
    );
    g.write("bank/items/pool.toml", &pool(4, &["A", "B"]));
    let ok = g.check("policy-floors-legitimate");
    assert_eq!(ok.code, 0, "{}{}", ok.stdout, ok.stderr);
    assert!(
        ok.stdout
            .contains("  pool_min=4 exam_n=2 multiplier≈2.0x (approved pool)\n"),
        "{}",
        ok.stdout
    );
}

/// bd-hw3, the latent nondeterminism. `ALLOWED_CORRECT` was a frozenset and the
/// diversity loop iterated it, so emission order rode on PYTHONHASHSEED. Two
/// pins, because either alone is weak: the oracle's stdout must be byte-equal
/// across seeds, AND the ordered tuple must exist in the source and be the thing
/// the loops iterate. The behavioural pin is currently satisfiable by accident
/// (at most one line is reachable at a 70% threshold); the structural pin is
/// what survives someone lowering that threshold.
#[test]
fn the_correct_letter_order_is_pinned_and_hash_seed_independent() {
    let f = Fixture::new();
    // 29/41 = 71% of B, which is the one diversity line reachable today.
    let mut body = String::new();
    for i in 0..29 {
        body.push_str(&good_item(&format!("b-{i:03}"), 1, "B", "t-one"));
    }
    for i in 0..6 {
        body.push_str(&good_item(&format!("a-{i:03}"), 1, "A", "t-one"));
    }
    for i in 0..6 {
        body.push_str(&good_item(&format!("c-{i:03}"), 1, "C", "t-one"));
    }
    f.write("bank/items/pool.toml", &body);

    let script = f.root.join("scripts/verify_bank.py");
    let mut seen: Option<String> = None;
    for seed in ["0", "1", "42", "12345"] {
        let run = capture(
            Command::new("python3")
                .env("PYTHONHASHSEED", seed)
                .arg(&script),
        );
        assert!(
            run.stdout
                .contains("  - correct=B is 71% of approved pool (max 70% for diversity)\n"),
            "seed {seed}:\n{}",
            run.stdout
        );
        match &seen {
            None => seen = Some(run.stdout),
            Some(first) => assert_eq!(
                *first, run.stdout,
                "oracle stdout varies with PYTHONHASHSEED={seed}"
            ),
        }
    }

    // Structural pin: the ordered tuple exists and is what the loops iterate.
    let src = std::fs::read_to_string(engine_root().join("scripts/verify_bank.py")).unwrap();
    assert!(
        src.contains("CORRECT_LETTERS = (\"A\", \"B\", \"C\", \"D\")"),
        "the ordered tuple must be declared verbatim"
    );
    assert!(
        src.contains("for L in CORRECT_LETTERS:"),
        "the diversity loop must iterate the ordered tuple"
    );
    assert!(
        !src.contains("for L in ALLOWED_CORRECT"),
        "no loop may iterate the frozenset: its order is PYTHONHASHSEED-dependent"
    );
    // ALLOWED_CORRECT must survive as a frozenset — membership on an unhashable
    // value has to keep raising, which a tuple would silently answer False.
    assert!(
        src.contains("ALLOWED_CORRECT = frozenset(CORRECT_LETTERS)"),
        "membership must keep set semantics"
    );
}

/// bd-hw3 acceptance, stated as its own case: no verdict may be written ahead of
/// a path that can still raise. Every RED fixture in this suite is checked for a
/// stdout that begins with a verdict it then contradicts.
#[test]
fn no_red_case_writes_pass_before_dying() {
    // A pool that passes every structural rule but has an unusable exam size —
    // the exact shape that used to print "PASS\n  items=1\n  unique_ids=1\n"
    // and then raise.
    let f = Fixture::new();
    f.write(
        "knowledge/bank_policy.toml",
        "exam_n_items = \"0\"\npool_min_items = 1\n\
         [[domain_min]]\nmodule = 1\nmin_items = 1\n",
    );
    f.write("bank/items/pool.toml", &pool(4, &["A", "B"]));
    let run = f.check("verdict-last");
    assert_ne!(run.code, 0);
    assert!(
        !run.stdout.contains("PASS"),
        "a PASS reached stdout on a failing run:\n{}",
        run.stdout
    );

    // And the raise path that remains (a non-string stem) still writes nothing.
    let g = Fixture::new();
    g.write(
        "bank/items/pool.toml",
        "[[items]]\nid = \"x\"\nmodule = 1\nstem = 5\n",
    );
    let py = python(&g.root);
    assert_eq!(
        py.stdout, "",
        "the oracle must write no verdict before a raise"
    );
    assert_eq!(py.code, 1);
}

/// The oracle's uncaught-exception path. CPython flushes the stdout written so
/// far, prints a traceback, and exits 1. stdout and the exit code must match;
/// the traceback text is the one surface this port does not reproduce, so the
/// assertion is "both non-empty" rather than equality — stated here rather than
/// hidden by omitting the case.
#[test]
fn an_oracle_raise_matches_on_stdout_and_exit_code_but_not_traceback_text() {
    let f = Fixture::new();
    f.write(
        "bank/items/pool.toml",
        "[[items]]\nid = \"x\"\nmodule = 1\nstem = 5\n",
    );
    let py = python(&f.root);
    let rs = rust(&f.root);
    assert_eq!(py.stdout, rs.stdout, "raise-path STDOUT differs");
    assert_eq!(py.stdout, "", "the oracle prints nothing before this raise");
    assert_eq!(py.code, rs.code, "raise-path EXIT CODE differs");
    assert_eq!(py.code, 1, "an uncaught exception exits 1");
    assert!(!py.stderr.is_empty(), "the oracle must have said something");
    assert!(!rs.stderr.is_empty(), "the port must not fail silently");
    assert!(
        py.stderr.contains("AttributeError") && rs.stderr.contains("AttributeError"),
        "both sides must name the same exception\npython: {}\nrust: {}",
        py.stderr,
        rs.stderr
    );
}

// ── bd-8exw: the floors measure the APPROVED pool ──────────────────────────
//
// Prior art copied rather than re-derived: `diff_verify_coverage.rs` cases
// (j)(k)(l) — `module_lines` / `retire_in_place` — do the same injection for
// the sibling gate.

/// THE KNOWN-BAD. Retire items IN PLACE until the drawable pool is under
/// `pool_min_items`, WITHOUT deleting a file.
///
/// This fixture is chosen so the pre-fix gate stayed GREEN on it *by
/// construction*: the file count never moves, and the file count was the only
/// thing the floor could see. That is the proof the population changed, not
/// merely the wording.
#[test]
fn retiring_in_place_trips_the_pool_floor_without_deleting_a_file() {
    let f = Fixture::new();
    f.write(
        "knowledge/bank_policy.toml",
        "exam_n_items = 1\npool_min_items = 4\n\
         [[domain_min]]\nmodule = 1\nmin_items = 1\n",
    );
    // Six FILES. Green while all six are approved.
    let mut body = String::new();
    for i in 0..6 {
        body.push_str(&good_item(
            &format!("i-{i:03}"),
            1,
            ["A", "B", "C", "D"][i % 4],
            "t-one",
        ));
    }
    f.write("bank/items/pool.toml", &body);
    let before = f.check("retire-in-place: before");
    assert_eq!(before.code, 0, "{}{}", before.stdout, before.stderr);
    assert!(
        before
            .stdout
            .contains("  items=6 scanned, 6 approved (floors count the approved pool only)\n"),
        "{}",
        before.stdout
    );

    // The SAME six files, three retired in place. Not one file removed.
    let mut retired = String::new();
    for i in 0..3 {
        retired.push_str(&good_item(
            &format!("i-{i:03}"),
            1,
            ["A", "B", "C", "D"][i % 4],
            "t-one",
        ));
    }
    for i in 3..6 {
        retired.push_str(&item_with_status(
            &format!("i-{i:03}"),
            1,
            ["A", "B", "C", "D"][i % 4],
            "t-one",
            "retired",
        ));
    }
    f.write("bank/items/pool.toml", &retired);
    let after = f.check("retire-in-place: after");
    assert_ne!(
        after.code, 0,
        "3 drawable items under a floor of 4 must be RED:\n{}",
        after.stdout
    );
    assert!(
        after.stdout.contains(
            "  - pool too small: 3 approved < pool_min_items 4 \
             (6 scanned, 3 not approved; need ≥4× exam size 1)\n"
        ),
        "the finding must name the module-free pool in BOTH populations:\n{}",
        after.stdout
    );
    // The file set is untouched, which is why a file-counting floor could not
    // have seen this.
    assert!(
        after.stdout.contains("6 scanned"),
        "the file count must be unchanged at 6:\n{}",
        after.stdout
    );
}

/// The same injection against a `[[domain_min]]` floor: RED naming the module,
/// the approved count, and the floor.
#[test]
fn retiring_in_place_trips_a_domain_floor_naming_module_count_and_floor() {
    let f = Fixture::new();
    f.write(
        "knowledge/bank_policy.toml",
        "exam_n_items = 1\npool_min_items = 1\n\
         [[domain_min]]\nmodule = 6\nmin_items = 4\n",
    );
    let mut body = String::new();
    for i in 0..2 {
        body.push_str(&good_item(&format!("m6-{i}"), 6, "A", "t-one"));
    }
    for i in 2..5 {
        body.push_str(&item_with_status(
            &format!("m6-{i}"),
            6,
            "B",
            "t-one",
            "retired",
        ));
    }
    f.write("bank/items/pool.toml", &body);
    let run = f.check("retire-in-place: domain floor");
    assert_ne!(run.code, 0, "{}", run.stdout);
    assert!(
        run.stdout.contains(
            "  - module 6: 2 approved items < domain_min 4 (5 scanned, 3 not approved)\n"
        ),
        "the shortfall must name module, approved count, floor, and the file \
         count that hid it:\n{}",
        run.stdout
    );
}

/// ANTI-VACUOUS, the leg the empty-bank case cannot reach: a bank FULL of files
/// and EMPTY of drawable items. Zero approved is an ERROR naming the condition,
/// distinct from `zero items loaded`, which counts files and stays silent here.
#[test]
fn a_bank_of_only_retired_items_is_an_error_in_both_never_a_pass() {
    let f = Fixture::new();
    f.write(
        "knowledge/bank_policy.toml",
        "exam_n_items = 1\npool_min_items = 1\n\
         [[domain_min]]\nmodule = 1\nmin_items = 1\n",
    );
    let mut body = String::new();
    for i in 0..4 {
        body.push_str(&item_with_status(
            &format!("dead-{i}"),
            1,
            "A",
            "t-one",
            "retired",
        ));
    }
    f.write("bank/items/pool.toml", &body);
    let run = f.check("zero-approved");
    assert_ne!(
        run.code, 0,
        "a bank nobody can be assessed from passed:\n{}",
        run.stdout
    );
    assert!(
        run.stdout.contains(
            "  - zero approved items (4 scanned): the floors measure a pool no \
             learner can be assessed from (vacuous scan is ERROR)\n"
        ),
        "{}",
        run.stdout
    );
    assert!(
        !run.stdout.contains("zero items loaded"),
        "four files loaded — the empty-bank leg is a DIFFERENT failure and must \
         stay silent, or this case would be testing that one instead:\n{}",
        run.stdout
    );
}

/// An unmodelled status is a finding naming the item, never a silent drop into
/// "not approved" — a bucket decided by guess is the same defect one level down.
/// The U+200B suffix also proves the Python `repr(str)` escaping leg.
#[test]
fn an_unmodelled_status_is_a_named_finding_in_both() {
    let f = Fixture::new();
    let mut body = pool(2, &["A", "B"]);
    let odd = item_with_status("i-odd", 1, "C", "t-one", "published")
        .replace("status = \"published\"", "status = \"published\\u200b\"");
    body.push_str(&odd);
    f.write("bank/items/pool.toml", &body);
    let run = f.check("unknown-status");
    assert_ne!(run.code, 0, "{}", run.stdout);
    assert!(
        run.stdout
            .contains("  - i-odd: unknown status 'published\\u200b'\n"),
        "{}",
        run.stdout
    );
}

/// Python 3.11 classifies the unassigned U+0378 as non-printable, so its
/// `repr(str)` emits a `\\u` escape. The known-bad mutant leaves that code point
/// printable in Rust; this fixture must RED on that mutant rather than silently
/// accepting a literal character.
#[test]
fn unassigned_unicode_status_repr_is_byte_identical_and_known_bad_is_red() {
    let f = Fixture::new();
    let mut body = pool(2, &["A", "B"]);
    let odd = item_with_status("i-unassigned", 1, "C", "t-one", "published")
        .replace("status = \"published\"", "status = \"published\\u0378\"");
    body.push_str(&odd);
    f.write("bank/items/pool.toml", &body);
    let run = f.check("unknown-status-unassigned-unicode");
    assert_ne!(run.code, 0, "{}", run.stdout);
    assert!(
        run.stdout
            .contains("  - i-unassigned: unknown status 'published\\u0378'\n"),
        "{}",
        run.stdout
    );
}

/// A non-`str` status can never equal a member of the tuple, so it is unknown
/// too, and renders through `repr()`.
#[test]
fn a_non_string_status_is_unknown_and_reprs_identically() {
    let f = Fixture::new();
    let mut body = pool(2, &["A", "B"]);
    body.push_str(
        &item_with_status("i-num", 1, "C", "t-one", "x").replace("status = \"x\"", "status = 7"),
    );
    f.write("bank/items/pool.toml", &body);
    let run = f.check("non-string-status");
    assert!(
        run.stdout.contains("  - i-num: unknown status 7\n"),
        "{}",
        run.stdout
    );
}

/// Silence is not approval. An item with no `status` line is `draft`, matching
/// `cdcp_bank::ItemStatus`'s serde default — and `draft` is a KNOWN status, so
/// it is counted out of the floors without also being reported as junk.
#[test]
fn an_absent_status_is_draft_in_both_and_counts_out_of_the_floors() {
    let f = Fixture::new();
    f.write(
        "knowledge/bank_policy.toml",
        "exam_n_items = 1\npool_min_items = 2\n\
         [[domain_min]]\nmodule = 1\nmin_items = 1\n",
    );
    let bare = good_item("i-bare", 1, "B", "t-one").replace("\nstatus = \"approved\"", "");
    assert!(!bare.contains("status"), "the fixture must carry no status");
    f.write(
        "bank/items/pool.toml",
        &format!("{}{}", good_item("i-ok", 1, "A", "t-one"), bare),
    );
    let run = f.check("absent-status-is-draft");
    assert_ne!(run.code, 0, "{}", run.stdout);
    assert!(
        run.stdout.contains(
            "  - pool too small: 1 approved < pool_min_items 2 \
             (2 scanned, 1 not approved; need ≥2× exam size 1)\n"
        ),
        "{}",
        run.stdout
    );
    assert!(
        !run.stdout.contains("unknown status"),
        "draft is modelled; an absent status is not junk:\n{}",
        run.stdout
    );
}

/// Letter diversity is a claim about the pool a mock is drawn from. Retired
/// items must neither dilute the fraction nor lift the pool over the 40-item
/// gate.
#[test]
fn letter_diversity_is_gated_and_measured_on_the_approved_pool_in_both() {
    // 45 files: 40 approved and all B, plus 5 retired A. 40/45 is 89% and
    // 40/40 is 100% — either trips, but only the approved reading is right,
    // and the SECOND rule (three distinct letters) separates them: the file
    // set uses two letters, the approved pool uses one.
    let f = Fixture::new();
    let mut body = String::new();
    for i in 0..40 {
        body.push_str(&good_item(&format!("b-{i:03}"), 1, "B", "t-one"));
    }
    for i in 0..5 {
        body.push_str(&item_with_status(
            &format!("a-{i:03}"),
            1,
            "A",
            "t-one",
            "retired",
        ));
    }
    f.write("bank/items/pool.toml", &body);
    let run = f.check("diversity-on-approved-pool");
    assert!(
        run.stdout
            .contains("  - correct=B is 100% of approved pool (max 70% for diversity)\n"),
        "100%, not 89%: the retired A items are not in the pool:\n{}",
        run.stdout
    );

    // And the 40-item GATE is on the drawable count: 39 approved among 45
    // files skips the rules, where a file-set gate of 45 would have applied
    // them.
    let g = Fixture::new();
    let mut body = String::new();
    for i in 0..39 {
        body.push_str(&good_item(&format!("b-{i:03}"), 1, "B", "t-one"));
    }
    for i in 0..6 {
        body.push_str(&item_with_status(
            &format!("a-{i:03}"),
            1,
            "A",
            "t-one",
            "retired",
        ));
    }
    g.write("bank/items/pool.toml", &body);
    let below = g.check("diversity-gate-on-approved-count");
    assert!(
        !below.stdout.contains("diversity"),
        "39 approved is under the threshold however many files there are:\n{}",
        below.stdout
    );
}

/// MANIFEST drift stays on the FILE SET, deliberately. A retirement that never
/// reached the manifest is exactly what this catches; counting it on the
/// approved pool would hide it. Stated as a test so a later "consistency" pass
/// cannot quietly rebase it.
#[test]
fn manifest_drift_is_measured_against_the_file_set_on_purpose_in_both() {
    let f = Fixture::new();
    let mut body = pool(3, &["A", "B", "C"]);
    body.push_str(&item_with_status("i-dead", 1, "D", "t-one", "retired"));
    f.write("bank/items/pool.toml", &body);
    f.write("bank/MANIFEST.toml", "item_count = 4\n");
    let ok = f.check("manifest-tracks-files");
    assert_eq!(
        ok.code, 0,
        "4 files and a manifest of 4 is in sync even though 1 is retired:\n{}{}",
        ok.stdout, ok.stderr
    );
    assert!(ok.stdout.starts_with("PASS\n"), "{}", ok.stdout);
    assert!(
        !ok.stdout.contains("MANIFEST item_count"),
        "an in-sync file-set manifest must not report drift:\n{}",
        ok.stdout
    );

    // A manifest that tracked the APPROVED pool would read 3 and be green;
    // here it is drift, which is the reading this test pins.
    f.write("bank/MANIFEST.toml", "item_count = 3\n");
    let drift = f.check("manifest-is-not-the-approved-pool");
    assert_ne!(drift.code, 0, "{}", drift.stdout);
    assert!(
        drift
            .stdout
            .contains("  - MANIFEST item_count 3 != loaded 4\n"),
        "{}",
        drift.stdout
    );
}

/// bd-8exw moved a raise point. The approved-pool tally is a PRE-PASS over
/// `loaded`, so `it.get(...)` on a non-mapping item now raises one loop earlier
/// than it used to. stdout is empty at that point either way, but "either way"
/// is a claim, so it is asserted rather than assumed.
#[test]
fn a_non_mapping_item_raises_identically_from_the_approved_pool_pre_pass() {
    let f = Fixture::new();
    f.write("bank/items/pool.toml", "items = [1, 2]\n");
    let py = python(&f.root);
    let rs = rust(&f.root);
    assert_eq!(py.stdout, rs.stdout, "pre-pass raise STDOUT differs");
    assert_eq!(py.stdout, "", "nothing may be written before this raise");
    assert_eq!(py.code, rs.code, "pre-pass raise EXIT CODE differs");
    assert_eq!(py.code, 1);
    assert!(
        py.stderr.contains("AttributeError") && rs.stderr.contains("AttributeError"),
        "both sides must name the same exception\npython: {}\nrust: {}",
        py.stderr,
        rs.stderr
    );
}

/// The one deliberate divergence outside the verdict path: the oracle ignores
/// argv, this port rejects it, because a typo'd flag must not read as a pass.
#[test]
fn the_port_rejects_arguments_the_oracle_would_have_ignored() {
    let root = engine_root();
    let rs = capture(
        Command::new(BIN)
            .arg("--root")
            .arg(&root)
            .arg("verify-bank")
            .arg("--bank"),
    );
    assert_eq!(rs.code, cdcp_gate::exit::USAGE as i32, "{}", rs.stderr);
    assert!(rs.stderr.contains("takes no arguments"), "{}", rs.stderr);
}
