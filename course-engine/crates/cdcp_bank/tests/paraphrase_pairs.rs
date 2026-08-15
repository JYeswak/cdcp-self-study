//! Live-tree + on-disk known-bad plants for `cdcp_bank::paraphrase`.
//!
//! Replaces `scripts/verify_paraphrase_pairs.py --selftest` as the L4
//! tripwire (bd-substrate-rust-migration-jhd.21). The Python is retired
//! from check.sh; rust is the grader-of-record.

use cdcp_bank::paraphrase::{
    check_ledger, parse_ledger, run, Request, EXIT_OK, REQUIRED_DISTINCT_IDS, REQUIRED_PAIR_IDS,
};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static SEQ: AtomicU64 = AtomicU64::new(1);

fn engine_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("course-engine root")
}

fn tmp(tag: &str) -> PathBuf {
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("cdcp-pp-{}-{}-{}", tag, std::process::id(), n))
}

fn live_req() -> Request {
    let root = engine_root();
    Request {
        ledger: root.join("registries/paraphrase_pairs.toml"),
        bank: root.join("bank/items"),
        selftest_only: false,
    }
}

#[test]
fn live_tree_ledger_is_green() {
    let out = run(&live_req());
    assert_eq!(
        out.code, EXIT_OK,
        "stderr={} stdout={}",
        out.stderr, out.stdout
    );
    assert!(out.stdout.contains("ok — ledger intact"), "{}", out.stdout);
    assert!(
        out.stdout.contains("NOT a grader-of-record"),
        "report must print and must not be a verdict: {}",
        out.stdout
    );
    assert!(
        out.stdout.contains(REQUIRED_DISTINCT_IDS[0]) || out.stdout.contains("known-distinct"),
        "known-distinct pair must remain visible: {}",
        out.stdout
    );
    assert!(
        out.stdout.contains("selftest RED"),
        "in-process plants must trip on the live run: {}",
        out.stdout
    );
}

#[test]
fn missing_required_pair_row_is_red() {
    let root = engine_root();
    let live = fs::read_to_string(root.join("registries/paraphrase_pairs.toml")).unwrap();
    let raw: toml::Value = toml::from_str(&live).unwrap();
    let (mut ledger, load_errs) = parse_ledger(&raw, "plant-missing-pair");
    assert!(load_errs.is_empty(), "{load_errs:?}");
    ledger.pairs.retain(|row| {
        row.as_table()
            .and_then(|t| t.get("id"))
            .and_then(|v| v.as_str())
            != Some(REQUIRED_PAIR_IDS[0])
    });
    let bank = cdcp_bank::Bank::load_dir(&root.join("bank/items")).expect("live bank");
    let items = cdcp_bank::paraphrase::items_from_bank(&bank);
    let errs = check_ledger(&ledger, &items);
    assert!(
        errs.iter().any(|e| e.contains(REQUIRED_PAIR_IDS[0])),
        "deleting a required pair without adjudication must RED: {errs:?}"
    );
}

#[test]
fn empty_ledger_file_is_red() {
    let root = engine_root();
    let dir = tmp("empty-ledger");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let ledger = dir.join("empty.toml");
    fs::write(
        &ledger,
        "schema_version = 1\n\n[registry]\nname = \"x\"\nmin_pairs = 4\n",
    )
    .unwrap();
    let out = run(&Request {
        ledger,
        bank: root.join("bank/items"),
        selftest_only: false,
    });
    let _ = fs::remove_dir_all(&dir);
    assert_ne!(
        out.code, EXIT_OK,
        "empty [[pair]] must not pass: {}",
        out.stderr
    );
    assert!(
        out.stderr.contains("zero [[pair]]") || out.stderr.contains("empty [[pair]]"),
        "{}",
        out.stderr
    );
}

#[test]
fn empty_bank_dir_is_error() {
    let root = engine_root();
    let dir = tmp("empty-bank");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let out = run(&Request {
        ledger: root.join("registries/paraphrase_pairs.toml"),
        bank: dir.clone(),
        selftest_only: false,
    });
    let _ = fs::remove_dir_all(&dir);
    assert_ne!(
        out.code, EXIT_OK,
        "zero item files must not pass: {}",
        out.stderr
    );
    assert!(
        out.stderr.contains("zero item") || out.stderr.contains("vacuous"),
        "{}",
        out.stderr
    );
}

#[test]
fn missing_bank_dir_is_error() {
    let root = engine_root();
    let out = run(&Request {
        ledger: root.join("registries/paraphrase_pairs.toml"),
        bank: PathBuf::from("/tmp/cdcp-pp-no-such-bank-dir"),
        selftest_only: false,
    });
    assert_ne!(out.code, EXIT_OK, "{}", out.stderr);
    assert!(out.stderr.contains("bank dir missing"), "{}", out.stderr);
}

#[test]
fn missing_ledger_file_is_error() {
    let root = engine_root();
    let out = run(&Request {
        ledger: PathBuf::from("/tmp/cdcp-pp-no-such-ledger.toml"),
        bank: root.join("bank/items"),
        selftest_only: false,
    });
    assert_ne!(out.code, EXIT_OK, "{}", out.stderr);
    assert!(out.stderr.contains("ledger missing"), "{}", out.stderr);
}

#[test]
fn python_oracle_is_gone() {
    // EXTRACT-THEN-DELETE: rust selftests replace --selftest. Putting the
    // first-level python3 script back is a Substrate Law regression.
    let py = engine_root().join("scripts/verify_paraphrase_pairs.py");
    assert!(
        !py.is_file(),
        "scripts/verify_paraphrase_pairs.py must stay deleted (rust is the gate): {}",
        py.display()
    );
}

/// Anti-vacuous: a scan of a directory that exists but holds no item files
/// is ERROR. The helper is named so a future "skip empty" refactor trips it.
#[test]
fn zero_toml_files_is_an_error_not_a_pass() {
    use cdcp_bank::paraphrase::load_scan_items;
    let dir = tmp("zero-toml");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("README.txt"), "not an item\n").unwrap();
    let err = load_scan_items(&dir).expect_err("zero toml files must ERROR");
    let _ = fs::remove_dir_all(&dir);
    assert!(
        err.contains("zero item files") || err.contains("vacuous"),
        "{err}"
    );
}
