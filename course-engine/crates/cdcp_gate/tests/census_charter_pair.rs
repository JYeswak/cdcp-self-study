//! CHARTER mutate/delete pairs for the differential census.
//!
//! Bead: `bd-census-mutation-pair-unproven-pi0v`.
//!
//! `differential_verdict_census.rs` can compile and go green while the thing
//! that makes it a gate — `assert!(total <= budget)` and
//! `assert!(violations.is_empty())` — has never been shown to be load-bearing.
//! A comment that says "we mutated once" is not a pair. This file mutates,
//! runs the census suite, deletes the assertion, runs it again, and restores
//! through `cdcp_restore_safe`.
//!
//! Two pairs, four legs. A driver that only runs leg (a) is the defect.
//!
//!   1. RATCHET. Tighten `agreement_only_total_budget` to 0 and plant one
//!      bare comparator call in a registered harness → suite non-zero and
//!      names `THE RATCHET SLIPPED`. Mutation still in place, delete
//!      `assert!(total <= budget)` → suite returns to zero.
//!   2. VERDICT-SHAPE. Register a wrong success token for
//!      `verify-knowledge-paths` → suite non-zero and names that gate.
//!      Mutation still in place, delete `assert!(violations.is_empty())`
//!      → suite returns to zero.
//!
//! `#[ignore]` is load-bearing: this binary mutates the live tree, so
//! `cargo test --workspace` must not run it next to other cdcp_gate tests.
//! The needle test below stays live so a deleted pair is a RED, not a skip.
//!
//! Prove:
//!   cargo test -p cdcp_gate --offline --test census_charter_pair \
//!     -- --ignored --nocapture --test-threads=1

use std::path::{Path, PathBuf};
use std::process::Command;

const CENSUS_TEST: &str = "differential_verdict_census";

const RATCHET_ASSERT: &str = r#"    assert!(
        total <= reg.census.agreement_only_total_budget,
        "THE RATCHET SLIPPED. {total} agreement-only cases across the tree, above \
         the budget of {}. Agreement is necessary for a correct port and is NOT \
         sufficient for a correct gate: every case in this count is one that no \
         shared defect can make fail, and one that evaporates entirely when its \
         oracle is retired. The budget may fall; it may not rise. Per harness: \
         {per:?}",
        reg.census.agreement_only_total_budget
    );"#;

const RATCHET_ASSERT_DELETED: &str = r#"    // CHARTER pair leg (b): assert!(total <= budget) deleted
    let _ = (total, &per);"#;

const SHAPE_ASSERT: &str = r#"    assert!(
        violations.is_empty(),
        "VERDICT-SHAPE VIOLATIONS ({}). `exit == 0` must be EQUIVALENT to \
         \"stdout carries this gate's success token\". A success token printed \
         on a path that returns non-zero is the class bd-lt7, \
         bd-builder-verdict-shape-qm65 and bd-verify-coverage-verdict-before-write-rk9n \
         each patched by hand; at three instances the fix is a detector, not a \
         fourth patch.\n\n{}",
        violations.len(),
        violations.join("\n---\n")
    );"#;

const SHAPE_ASSERT_DELETED: &str = r#"    // CHARTER pair leg (b): assert!(violations.is_empty()) deleted
    let _ = &violations;"#;

const TOKEN_NEEDLE: &str = "name = \"verify-knowledge-paths\"\nsuccess_token = \"PASS\"";
const TOKEN_WRONG: &str =
    "name = \"verify-knowledge-paths\"\nsuccess_token = \"CENSUS_PAIR_WRONG_TOKEN\"";

const PLANT: &str = r#"

#[test]
fn census_charter_pair_bare_comparator_plant() {
    assert_identical(&std::path::PathBuf::new(), "census-charter-pair-plant");
}
"#;

fn engine_root() -> PathBuf {
    cdcp_gate::root::resolve(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
}

fn cargo_bin() -> String {
    option_env!("CARGO").unwrap_or("cargo").to_string()
}

fn census_src() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/differential_verdict_census.rs")
}

fn registry_src() -> PathBuf {
    engine_root().join("registries/differential_harnesses.toml")
}

fn plant_src() -> PathBuf {
    // The injection-count differential harness was retired with its Python
    // oracle. Keep this mutate/delete pair anchored to a live registered
    // harness so the census proof remains executable.
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/diff_verify_coverage.rs")
}

fn restore_safe(dest: &Path, original: &[u8]) {
    let bak = std::env::temp_dir().join(format!(
        "cdcp-census-pair-{}-{}",
        std::process::id(),
        dest.file_name().unwrap().to_string_lossy()
    ));
    std::fs::write(&bak, original).unwrap_or_else(|e| panic!("write bak {}: {e}", bak.display()));
    let helper = engine_root().join("scripts/restore_safe.inc.sh");
    let status = Command::new("/bin/sh")
        .arg("-c")
        .arg(r#". "$1" && cdcp_restore_safe "$2" "$3""#)
        .arg("restore_safe")
        .arg(&helper)
        .arg(dest)
        .arg(&bak)
        .status()
        .unwrap_or_else(|e| panic!("spawn cdcp_restore_safe: {e}"));
    let _ = std::fs::remove_file(&bak);
    assert!(
        status.success(),
        "cdcp_restore_safe failed for {} (rc={:?})",
        dest.display(),
        status.code()
    );
    let now = std::fs::read(dest).unwrap_or_else(|e| panic!("re-read {}: {e}", dest.display()));
    assert_eq!(
        now,
        original,
        "cdcp_restore_safe left {} different from the captured bytes",
        dest.display()
    );
}

struct RestoreOnDrop {
    files: Vec<(PathBuf, Vec<u8>)>,
}

impl RestoreOnDrop {
    fn capture(paths: &[&Path]) -> Self {
        let files = paths
            .iter()
            .map(|p| {
                let b = std::fs::read(p).unwrap_or_else(|e| panic!("capture {}: {e}", p.display()));
                ((*p).to_path_buf(), b)
            })
            .collect();
        RestoreOnDrop { files }
    }

    fn restore_now(&self) {
        for (p, b) in &self.files {
            restore_safe(p, b);
        }
    }
}

impl Drop for RestoreOnDrop {
    fn drop(&mut self) {
        for (p, b) in &self.files {
            let cur = std::fs::read(p).unwrap_or_default();
            if cur != *b {
                restore_safe(p, b);
            }
        }
    }
}

fn replace_once(path: &Path, old: &str, new: &str) {
    let text =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let n = text.matches(old).count();
    assert_eq!(
        n,
        1,
        "CHARTER pair: needle count {n} in {} (want 1)",
        path.display()
    );
    std::fs::write(path, text.replacen(old, new, 1))
        .unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
}

fn tighten_budget_to_zero(path: &Path) {
    let text =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let key = "agreement_only_total_budget = ";
    let start = text
        .find(key)
        .unwrap_or_else(|| panic!("{}: missing {key}", path.display()));
    assert!(
        text[start + key.len()..].find(key).is_none(),
        "{}: {key} is not unique",
        path.display()
    );
    let after = start + key.len();
    let end = text[after..]
        .find('\n')
        .map(|i| after + i)
        .unwrap_or(text.len());
    let digits = &text[after..end];
    assert!(
        !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()),
        "{}: budget is not digits ({digits:?})",
        path.display()
    );
    let mut new = String::with_capacity(text.len());
    new.push_str(&text[..after]);
    new.push('0');
    new.push_str(&text[end..]);
    std::fs::write(path, new).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
}

fn plant_bare_comparator(path: &Path) {
    let text =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    assert!(
        !text.contains("census-charter-pair-plant"),
        "{} already holds the plant",
        path.display()
    );
    let mut new = text;
    if !new.ends_with('\n') {
        new.push('\n');
    }
    new.push_str(PLANT);
    std::fs::write(path, new).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
}

fn rebuild_gate_bin() {
    // The census test embeds CARGO_BIN_EXE_cdcp_gate. A sibling can delete a
    // gate file after this target dir last linked the bin; without a rebuild
    // the roster and the registry disagree and the wrong assertion fires.
    let out = Command::new(cargo_bin())
        .current_dir(engine_root())
        .args([
            "build",
            "-p",
            "cdcp_gate",
            "--offline",
            "--bin",
            "cdcp_gate",
        ])
        .output()
        .unwrap_or_else(|e| panic!("spawn cargo build -p cdcp_gate: {e}"));
    assert!(
        out.status.success(),
        "cargo build -p cdcp_gate --bin cdcp_gate failed\n{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

fn run_census_test(exact: &str) -> (i32, String) {
    let out = Command::new(cargo_bin())
        .current_dir(engine_root())
        .args([
            "test",
            "-p",
            "cdcp_gate",
            "--offline",
            "--test",
            CENSUS_TEST,
            "--",
            "--exact",
            exact,
            "--nocapture",
        ])
        .output()
        .unwrap_or_else(|e| panic!("spawn cargo test --test {CENSUS_TEST} --exact {exact}: {e}"));
    let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.code().unwrap_or(-1), text)
}

/// Always-on. A pair that can be deleted without this going red is a comment.
#[test]
fn charter_pair_needles_are_present() {
    let census = std::fs::read_to_string(census_src()).expect("census src");
    assert!(
        census.contains("total <= reg.census.agreement_only_total_budget"),
        "ratchet assert missing from {CENSUS_TEST}"
    );
    assert!(
        census.contains("THE RATCHET SLIPPED"),
        "ratchet must name itself"
    );
    assert!(
        census.contains("violations.is_empty()"),
        "verdict-shape assert missing from {CENSUS_TEST}"
    );
    assert!(
        census.contains("VERDICT-SHAPE VIOLATIONS"),
        "verdict-shape must name itself"
    );

    let me = include_str!("census_charter_pair.rs");
    assert!(
        me.contains("fn the_census_charter_pairs_are_load_bearing"),
        "the mutate/delete driver was deleted"
    );
    assert!(
        me.contains("census-charter-pair-plant"),
        "the plant payload was deleted"
    );
    assert!(
        me.contains("cdcp_restore_safe"),
        "restore must go through cdcp_restore_safe"
    );

    let reg = std::fs::read_to_string(registry_src()).expect("registry");
    assert!(
        reg.contains("agreement_only_total_budget = "),
        "registry lost the ratchet key"
    );
    assert!(
        reg.contains(TOKEN_NEEDLE),
        "verify-knowledge-paths success_token needle drifted"
    );
}

/// THE PAIR. Isolated: do not run next to other cdcp_gate tests.
#[ignore = "mutates the live census sources; run as --test census_charter_pair -- --ignored"]
#[test]
fn the_census_charter_pairs_are_load_bearing() {
    let census = census_src();
    let registry = registry_src();
    let plant = plant_src();
    rebuild_gate_bin();

    const RATCHET: &str = "the_agreement_only_ratchet_holds";
    const SHAPE: &str = "every_dispatched_gate_is_registered_and_holds_the_verdict_shape";

    // Baseline: an unperturbed tree must be GREEN or the pair cannot discriminate.
    let (base_r, base_r_out) = run_census_test(RATCHET);
    assert_eq!(
        base_r, 0,
        "fixture is vacuous: unperturbed {RATCHET} is not green\n{base_r_out}"
    );
    let (base_s, base_s_out) = run_census_test(SHAPE);
    assert_eq!(
        base_s, 0,
        "fixture is vacuous: unperturbed {SHAPE} is not green\n{base_s_out}"
    );

    // ── pair 1: the agreement-only ratchet ──────────────────────────────
    let guard1 = RestoreOnDrop::capture(&[&registry, &plant, &census]);
    tighten_budget_to_zero(&registry);
    plant_bare_comparator(&plant);

    let (ratchet_mutate, out_a) = run_census_test(RATCHET);
    assert_ne!(
        ratchet_mutate, 0,
        "RATCHET mutate stayed GREEN (want non-zero)\n{out_a}"
    );
    assert!(
        out_a.contains("THE RATCHET SLIPPED"),
        "RATCHET mutate did not name the ratchet\n{out_a}"
    );
    let pair = 1;

    replace_once(&census, RATCHET_ASSERT, RATCHET_ASSERT_DELETED);
    let (ratchet_delete, out_b) = run_census_test(RATCHET);
    assert_eq!(
        ratchet_delete, 0,
        "RATCHET delete-assert stayed RED (rc={ratchet_delete}; want 0)\n{out_b}"
    );
    let pair = pair + 1;
    guard1.restore_now();
    drop(guard1);

    // ── pair 2: the verdict-shape equivalence ───────────────────────────
    let guard2 = RestoreOnDrop::capture(&[&registry, &census]);
    replace_once(&registry, TOKEN_NEEDLE, TOKEN_WRONG);

    let (shape_mutate, out_c) = run_census_test(SHAPE);
    assert_ne!(
        shape_mutate, 0,
        "SHAPE mutate stayed GREEN (want non-zero)\n{out_c}"
    );
    assert!(
        out_c.contains("VERDICT-SHAPE VIOLATIONS"),
        "SHAPE mutate did not name the verdict-shape leg\n{out_c}"
    );
    assert!(
        out_c.contains("verify-knowledge-paths"),
        "SHAPE mutate did not name the mutated gate\n{out_c}"
    );
    let pair = pair + 1;

    replace_once(&census, SHAPE_ASSERT, SHAPE_ASSERT_DELETED);
    let (shape_delete, out_d) = run_census_test(SHAPE);
    assert_eq!(
        shape_delete, 0,
        "SHAPE delete-assert stayed RED (rc={shape_delete}; want 0)\n{out_d}"
    );
    let pair = pair + 1;
    guard2.restore_now();
    drop(guard2);

    assert_eq!(
        pair, 4,
        "ANTI-VACUOUS: CHARTER pair ran {pair} legs, want 4 (a suite that only runs leg 1 is the defect)"
    );
    println!(
        "CENSUS_PAIR_LEGS={pair} RATCHET_MUTATE={ratchet_mutate} \
         RATCHET_DELETE={ratchet_delete} SHAPE_MUTATE={shape_mutate} \
         SHAPE_DELETE={shape_delete}"
    );
}
