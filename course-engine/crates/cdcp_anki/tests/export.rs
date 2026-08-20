//! Product tests for the learner Anki export.
//!
//! These assert the CONTENT contract (approved-only, pinned clock, two-run
//! identity, empty = ERROR). They do not claim byte-identity with CPython's
//! sqlite3 header.

use cdcp_anki::{
    approved_only, deck_clock, evaluate, expected_approved_live, load_live_bank,
    note_count_in_apkg, peek_apkg, planted_clock_leak_trips, resolve_engine_root,
    retired_ids_in_apkg, run, write_apkg, zip_date_time, Request, Source, ITEMS_DIR_REL,
    PINNED_EPOCH, WEB_DATA_REL,
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
fn live_export_is_approved_and_ships_no_retired() {
    let root = engine_root();
    let expected = expected_approved_live(&root).expect("approved count pin");
    let bank = load_live_bank(&root).expect("live bank");
    assert!(
        !bank.is_empty(),
        "empty live bank is an ERROR, not a 0-card deck"
    );
    let approved = approved_only(&bank);
    assert_eq!(
        approved.len(),
        expected,
        "count-pin gate should report approved-count drift before this assertion"
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
    assert_eq!(o.cards, expected);
    assert!(o.stdout.contains("export_anki ok"), "{}", o.stdout);
    assert!(o.stdout.contains(&format!("cards={expected}")));
    assert!(
        o.stdout.contains("unresolvable=0"),
        "green path must print the count: {}",
        o.stdout
    );

    let apkg_path = td.path().join("dist/anki/cdcp_bank.apkg");
    let apkg = fs::read(&apkg_path).expect("wrote apkg");
    assert_eq!(note_count_in_apkg(&apkg).unwrap(), expected);
    let leaked = retired_ids_in_apkg(&apkg, &bank).unwrap();
    assert!(
        leaked.is_empty(),
        "retired/draft items leaked into the deck: {leaked:?}"
    );
}

#[test]
fn check_plants_clock_leak_then_asserts_live_identity() {
    let root = engine_root();
    let expected = expected_approved_live(&root).expect("approved count pin");
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
        o.stdout.contains(&format!("ok cards={expected}")),
        "{}",
        o.stdout
    );
    assert!(
        o.stdout.contains("unresolvable=0"),
        "green --check must print the count: {}",
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

/// Was `an_answer_letter_outside_the_choice_list_falls_through_to_the_letter`
/// in the retired `diff_export_anki.rs`. That case PINNED exit 0 and a bare
/// `'D'`/`'E'` Back. urz3 inverts it: the export is RED and names the id.
#[test]
fn an_answer_letter_outside_the_choice_list_is_red_and_names_the_id() {
    let td = tempfile::tempdir().unwrap();
    fs::create_dir_all(td.path().join(ITEMS_DIR_REL)).unwrap();
    fs::write(
        td.path().join(ITEMS_DIR_REL).join("q01.toml"),
        "id = \"q01\"\nmodule = 1\nstem = \"s\"\nchoices = [\"only\"]\ncorrect = \"D\"\nexplanation = \"e\"\n",
    )
    .unwrap();
    let mut req = Request::default_for(td.path());
    req.format = "tsv".into();
    req.out = td.path().join("dist/anki");
    let o = run(&req);
    assert_eq!(o.code, 1, "known-bad must be RED: {}", o.stderr);
    assert!(
        o.stderr.contains("unresolvable"),
        "named failure, not a silent pass: {}",
        o.stderr
    );
    assert!(
        o.stderr.contains("q01"),
        "must name the item id: {}",
        o.stderr
    );
    assert!(o.files.is_empty(), "RED must carry no files: {:?}", o.files);
    assert!(
        !td.path().join("dist/anki/cdcp_bank.tsv").exists(),
        "must not write the TSV"
    );
}

#[test]
fn every_card_unresolvable_is_error() {
    let td = tempfile::tempdir().unwrap();
    fs::create_dir_all(td.path().join(ITEMS_DIR_REL)).unwrap();
    fs::write(
        td.path().join(ITEMS_DIR_REL).join("q01.toml"),
        "id = \"q01\"\nmodule = 1\nstem = \"s\"\nchoices = [\"only\"]\ncorrect = \"D\"\nexplanation = \"e\"\n",
    )
    .unwrap();
    fs::write(
        td.path().join(ITEMS_DIR_REL).join("q02.toml"),
        "id = \"q02\"\nmodule = 1\nstem = \"s\"\nchoices = [\"only\"]\ncorrect = \"E\"\nexplanation = \"e\"\n",
    )
    .unwrap();
    let mut req = Request::default_for(td.path());
    req.format = "tsv".into();
    req.out = td.path().join("dist/anki");
    let o = run(&req);
    assert_eq!(o.code, 1, "{}", o.stderr);
    assert!(
        o.stderr.contains("every card unresolvable"),
        "anti-vacuous: all-bad must be a named ERROR, not a skip: {}",
        o.stderr
    );
    assert!(o.stderr.contains("q01"), "{}", o.stderr);
    assert!(o.stderr.contains("q02"), "{}", o.stderr);
    assert!(o.files.is_empty());
    assert!(!td.path().join("dist/anki/cdcp_bank.tsv").exists());
}

#[test]
fn mixed_pool_with_one_unresolvable_writes_nothing() {
    let td = tempfile::tempdir().unwrap();
    fs::create_dir_all(td.path().join(ITEMS_DIR_REL)).unwrap();
    fs::write(
        td.path().join(ITEMS_DIR_REL).join("good.toml"),
        "id = \"good\"\nmodule = 1\nstem = \"s\"\nchoices = [\"alpha\"]\ncorrect = \"A\"\nexplanation = \"e\"\n",
    )
    .unwrap();
    fs::write(
        td.path().join(ITEMS_DIR_REL).join("q01.toml"),
        "id = \"q01\"\nmodule = 1\nstem = \"s\"\nchoices = [\"only\"]\ncorrect = \"D\"\nexplanation = \"e\"\n",
    )
    .unwrap();
    let mut req = Request::default_for(td.path());
    req.format = "tsv".into();
    req.out = td.path().join("dist/anki");
    let o = run(&req);
    assert_eq!(o.code, 1, "{}", o.stderr);
    assert!(o.stderr.contains("q01"), "{}", o.stderr);
    assert!(
        !o.stderr.contains("every card unresolvable"),
        "mixed pool is not the all-bad case: {}",
        o.stderr
    );
    assert!(o.files.is_empty());
    assert!(
        !td.path().join("dist/anki/cdcp_bank.tsv").exists(),
        "must not ship the good card while a bad one sits in the pool"
    );
}

#[test]
fn planted_green_export_prints_unresolvable_zero() {
    let td = tempfile::tempdir().unwrap();
    fs::create_dir_all(td.path().join(ITEMS_DIR_REL)).unwrap();
    fs::write(
        td.path().join(ITEMS_DIR_REL).join("good.toml"),
        "id = \"good\"\nmodule = 1\nstem = \"s\"\nchoices = [\"alpha\"]\ncorrect = \"A\"\nexplanation = \"e\"\n",
    )
    .unwrap();
    let mut req = Request::default_for(td.path());
    req.format = "tsv".into();
    req.out = td.path().join("dist/anki");
    let o = run(&req);
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert!(
        o.stdout.contains("unresolvable=0"),
        "green path must print the count: {}",
        o.stdout
    );
    assert!(td.path().join("dist/anki/cdcp_bank.tsv").exists());
    let tsv = fs::read_to_string(td.path().join("dist/anki/cdcp_bank.tsv")).unwrap();
    assert!(tsv.contains("A) alpha"), "{tsv}");
    assert!(!tsv.contains("\tD\t"), "{tsv}");
}

/// Empty `correct` used to take the Python `'' in "ABCD"` branch and crash
/// in `ord()`. urz3 made it a named FAIL; this pins the empty case.
#[test]
fn empty_correct_is_red_and_names_the_id() {
    let td = tempfile::tempdir().unwrap();
    fs::create_dir_all(td.path().join(ITEMS_DIR_REL)).unwrap();
    fs::write(
        td.path().join(ITEMS_DIR_REL).join("empty.toml"),
        "id = \"empty\"\nmodule = 1\nstem = \"s\"\nchoices = [\"alpha\", \"beta\", \"gamma\", \"delta\"]\ncorrect = \"\"\nexplanation = \"e\"\n",
    )
    .unwrap();
    let mut req = Request::default_for(td.path());
    req.format = "tsv".into();
    req.out = td.path().join("dist/anki");
    let o = run(&req);
    assert_eq!(o.code, 1, "empty correct must be RED: {}", o.stderr);
    assert!(
        o.stderr.contains("FAIL:"),
        "named FAIL, not a traceback or unlabeled exit: {}",
        o.stderr
    );
    assert!(
        o.stderr.contains("unresolvable"),
        "named failure, not GateError::Error theater: {}",
        o.stderr
    );
    assert!(
        o.stderr.contains("empty"),
        "must name the item id: {}",
        o.stderr
    );
    assert!(o.files.is_empty(), "RED must carry no files: {:?}", o.files);
    assert!(!td.path().join("dist/anki/cdcp_bank.tsv").exists());
}

/// Two-letter `correct` (`"AB"`) is a substring of `"ABCD"` — the retired
/// Python membership test accepted it and then `ord()` crashed. Must be a
/// named FAIL, never a shipped card and never a traceback.
#[test]
fn two_letter_correct_is_red_and_names_the_id() {
    let td = tempfile::tempdir().unwrap();
    fs::create_dir_all(td.path().join(ITEMS_DIR_REL)).unwrap();
    fs::write(
        td.path().join(ITEMS_DIR_REL).join("ab.toml"),
        "id = \"ab\"\nmodule = 1\nstem = \"s\"\nchoices = [\"alpha\", \"beta\", \"gamma\", \"delta\"]\ncorrect = \"AB\"\nexplanation = \"e\"\n",
    )
    .unwrap();
    let mut req = Request::default_for(td.path());
    req.format = "tsv".into();
    req.out = td.path().join("dist/anki");
    let o = run(&req);
    assert_eq!(o.code, 1, "multi-char correct must be RED: {}", o.stderr);
    assert!(
        o.stderr.contains("FAIL:"),
        "named FAIL, not a traceback or unlabeled exit: {}",
        o.stderr
    );
    assert!(
        o.stderr.contains("unresolvable"),
        "named failure: {}",
        o.stderr
    );
    assert!(
        o.stderr.contains("ab"),
        "must name the item id: {}",
        o.stderr
    );
    assert!(o.files.is_empty(), "RED must carry no files: {:?}", o.files);
    assert!(!td.path().join("dist/anki/cdcp_bank.tsv").exists());
}

/// `--source keys`: a mock40 item whose id is absent from keys_seed42.json
/// used to become `correct=''` and crash the Python exporter. Same named
/// FAIL as the empty-bank-item case; must identify the item id.
#[test]
fn missing_keys_seed42_key_is_red_and_names_the_id() {
    let td = tempfile::tempdir().unwrap();
    let data = td.path().join(WEB_DATA_REL);
    fs::create_dir_all(&data).unwrap();
    fs::write(
        data.join("mock40_seed42.json"),
        r#"{"items":[{"id":"q-missing","stem":"s","choices":["alpha","beta","gamma","delta"],"module":1}]}"#,
    )
    .unwrap();
    fs::write(data.join("keys_seed42.json"), r#"{"keys":[]}"#).unwrap();
    let mut req = Request::default_for(td.path());
    req.source = Source::Keys;
    req.format = "tsv".into();
    req.out = td.path().join("dist/anki");
    let o = run(&req);
    assert_eq!(
        o.code, 1,
        "missing keys_seed42 key must be RED: {}",
        o.stderr
    );
    assert!(
        o.stderr.contains("FAIL:"),
        "named FAIL, not a traceback: {}",
        o.stderr
    );
    assert!(
        o.stderr.contains("unresolvable"),
        "named failure: {}",
        o.stderr
    );
    assert!(
        o.stderr.contains("q-missing"),
        "must name the item id: {}",
        o.stderr
    );
    assert!(o.files.is_empty(), "RED must carry no files: {:?}", o.files);
    assert!(
        !td.path().join("dist/anki/cdcp_seed42_mock40.tsv").exists(),
        "must not write the keys-source TSV"
    );
}
