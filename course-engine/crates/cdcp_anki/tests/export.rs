//! Product tests for the learner Anki export.
//!
//! These assert the CONTENT contract (approved-only, pinned clock, two-run
//! identity, empty = ERROR). They do not claim byte-identity with CPython's
//! sqlite3 header.

use cdcp_anki::{
    approved_only, deck_clock, evaluate, load_live_bank, note_count_in_apkg, peek_apkg,
    planted_clock_leak_trips, resolve_engine_root, retired_ids_in_apkg, run, write_apkg,
    zip_date_time, Request, EXPECTED_APPROVED_LIVE, ITEMS_DIR_REL, PINNED_EPOCH,
};
use std::fs;
use std::path::PathBuf;

fn engine_root() -> PathBuf {
    resolve_engine_root(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
}

#[test]
fn empty_bank_is_error() {
    let td = tempfile::tempdir().unwrap();
    fs::create_dir_all(td.path().join(ITEMS_DIR_REL)).unwrap();
    let mut req = Request::default_for(td.path());
    req.format = "tsv".into();
    let o = evaluate(&req);
    assert_eq!(o.code, 1);
    assert!(
        o.stderr.contains("FAIL: zero items to export"),
        "{}",
        o.stderr
    );
    assert!(o.files.is_empty());
}

#[test]
fn all_retired_is_error_and_writes_nothing() {
    let td = tempfile::tempdir().unwrap();
    fs::create_dir_all(td.path().join(ITEMS_DIR_REL)).unwrap();
    fs::write(
        td.path().join(ITEMS_DIR_REL).join("r1.toml"),
        "id = \"r1\"\nstatus = \"retired\"\nmodule = 1\n\
         stem = \"retired-only-planted-stem\"\nchoices = [\"a\"]\ncorrect = \"A\"\n",
    )
    .unwrap();
    let mut req = Request::default_for(td.path());
    req.format = "tsv,apkg".into();
    req.out = td.path().join("dist/anki");
    let o = run(&req);
    assert_eq!(o.code, 1);
    assert_eq!(o.stderr, "FAIL: zero approved items to export\n");
    assert!(o.files.is_empty());
    assert!(!td.path().join("dist/anki/cdcp_bank.tsv").exists());
    assert!(!td.path().join("dist/anki/cdcp_bank.apkg").exists());
}

#[test]
fn planted_clock_leak_trips_red() {
    planted_clock_leak_trips().expect("two clocks must change the bytes");
}

#[test]
fn two_rust_exports_are_byte_identical() {
    let items = {
        let mut v = load_live_bank(&engine_root()).expect("live bank");
        v.retain(|c| {
            let s = c.status.trim().to_ascii_lowercase();
            s != "retired" && s != "draft"
        });
        v
    };
    assert!(
        !items.is_empty(),
        "live approved pool is empty — anti-vacuous"
    );
    let clock = deck_clock().unwrap();
    let a = write_apkg(&items, "CDCP Study", clock).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(1_100));
    let b = write_apkg(&items, "CDCP Study", clock).unwrap();
    assert_eq!(a, b, "two Rust .apkg runs 1.1s apart must be identical");
    let (crt, mod_, scm, dt) = peek_apkg(&a).unwrap();
    assert_eq!(crt, clock);
    assert_eq!(mod_, clock * 1000);
    assert_eq!(scm, clock * 1000);
    let expect = zip_date_time(clock);
    let expect_dos = (
        expect.0,
        expect.1,
        expect.2,
        expect.3,
        expect.4,
        expect.5 - (expect.5 % 2),
    );
    assert_eq!(dt, expect_dos);
}

#[test]
fn live_export_is_779_approved_and_ships_no_retired() {
    let root = engine_root();
    let bank = load_live_bank(&root).expect("live bank");
    assert!(
        !bank.is_empty(),
        "empty live bank is an ERROR, not a 0-card deck"
    );
    let approved = approved_only(&bank);
    assert_eq!(
        approved.len(),
        EXPECTED_APPROVED_LIVE,
        "approved-only pin drifted (bank decision, not a test fix)"
    );
    let retired_n = bank.len() - approved.len();
    assert!(
        retired_n > 0,
        "the pin is vacuous if the bank has no retired items"
    );

    let td = tempfile::tempdir().unwrap();
    let mut req = Request::default_for(&root);
    req.out = td.path().join("dist/anki");
    req.format = "tsv,apkg".into();
    let o = run(&req);
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert_eq!(o.cards, EXPECTED_APPROVED_LIVE);
    assert!(o.stdout.contains("export_anki ok"), "{}", o.stdout);
    assert!(o
        .stdout
        .contains(&format!("cards={EXPECTED_APPROVED_LIVE}")));

    let apkg_path = td.path().join("dist/anki/cdcp_bank.apkg");
    let apkg = fs::read(&apkg_path).expect("wrote apkg");
    assert_eq!(note_count_in_apkg(&apkg).unwrap(), EXPECTED_APPROVED_LIVE);
    let leaked = retired_ids_in_apkg(&apkg, &bank).unwrap();
    assert!(
        leaked.is_empty(),
        "retired/draft items leaked into the deck: {leaked:?}"
    );
}

#[test]
fn check_plants_clock_leak_then_asserts_live_identity() {
    let root = engine_root();
    let mut req = Request::default_for(&root);
    req.check = true;
    let o = run(&req);
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert!(
        o.stdout.contains("planted clock leak trips"),
        "{}",
        o.stdout
    );
    assert!(
        o.stdout.contains("two runs identical sha256="),
        "{}",
        o.stdout
    );
    assert!(
        o.stdout
            .contains(&format!("ok cards={EXPECTED_APPROVED_LIVE}")),
        "{}",
        o.stdout
    );
    assert!(o.files.is_empty(), "--check must not write --out");
}

#[test]
fn default_clock_is_the_pinned_epoch_without_source_date() {
    // The test process may inherit SOURCE_DATE_EPOCH; only assert the fallback
    // when the env is unset/empty.
    match std::env::var("SOURCE_DATE_EPOCH") {
        Ok(v) if !v.trim().is_empty() => {
            let c = deck_clock().unwrap();
            assert_ne!(c, 0);
        }
        _ => assert_eq!(deck_clock().unwrap(), PINNED_EPOCH),
    }
}
