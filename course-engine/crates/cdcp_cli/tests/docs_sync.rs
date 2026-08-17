//! bd-readme-public-rigor-8y0r.1 — advertised content counts vs the ledger.
//!
//! In-tree TEMP fixtures. No patch/worktree harness. `--check` must RED on a
//! decremented site and name file:line + advertised + actual. `--write` must
//! refuse an unsound ledger and leave the prose file byte-unchanged.

use assert_cmd::Command;
use cdcp_cli::docs::{self, LedgerKey, Mode, MIN_ADVERTISEMENT_SITES};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

fn cdcp() -> Command {
    Command::cargo_bin("cdcp").expect("cdcp bin")
}

fn engine_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("engine root")
}

fn workspace_root() -> PathBuf {
    engine_root().parent().expect("repo root").to_path_buf()
}

static SEQ: AtomicU64 = AtomicU64::new(0);

fn scratch() -> PathBuf {
    let n = SEQ.fetch_add(1, Ordering::SeqCst);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!(
        "cdcp-docs-sync-{}-{}-{}",
        std::process::id(),
        nanos,
        n
    ));
    fs::create_dir_all(&dir).expect("scratch");
    dir
}

fn write(path: &Path, body: &str) {
    if let Some(p) = path.parent() {
        fs::create_dir_all(p).expect("mkdir");
    }
    fs::write(path, body).expect("write");
}

fn plant_tree(units_json: &str) -> PathBuf {
    let top = scratch();
    let engine = top.join("engine");
    fs::create_dir_all(engine.join("registries")).unwrap();
    fs::write(
        engine.join("registries/claims.toml"),
        "schema_version = 1\n",
    )
    .unwrap();
    write(
        &top.join("README.md"),
        &fs::read_to_string(workspace_root().join("README.md")).unwrap(),
    );
    write(
        &top.join("CHARTER.md"),
        &fs::read_to_string(workspace_root().join("CHARTER.md")).unwrap(),
    );
    write(
        &engine.join("README.md"),
        &fs::read_to_string(engine_root().join("README.md")).unwrap(),
    );
    write(&engine.join("web/data/units_index.json"), units_json);
    let wasm_src = engine_root().join("web/assets/wasm/cdcp_wasm.wasm");
    let wasm_dst = engine.join("web/assets/wasm/cdcp_wasm.wasm");
    fs::create_dir_all(wasm_dst.parent().unwrap()).unwrap();
    fs::copy(&wasm_src, &wasm_dst).unwrap();
    engine
}

fn live_units() -> String {
    fs::read_to_string(engine_root().join("web/data/units_index.json")).unwrap()
}

#[test]
fn decremented_readme_count_is_red_and_names_file_line_advertised_actual() {
    let engine = plant_tree(&live_units());
    let ledger = docs::load_ledger(&engine).expect("sound ledger");
    let readme = engine.join("../README.md");
    let original = fs::read_to_string(&readme).unwrap();
    let site = docs::scan_text(&original)
        .into_iter()
        .find(|h| h.key == LedgerKey::BankItemCount)
        .expect("README must advertise bank_item_count");
    let planted = site.value.saturating_sub(1);
    let mut patched = original.clone();
    patched.replace_range(site.start..site.end, &planted.to_string());
    assert_ne!(patched, original, "plant must change the file");
    fs::write(&readme, &patched).unwrap();

    let err = docs::sync_tree(&engine, Mode::Check).expect_err("decremented count must RED");
    let msg = err.to_string();
    let expected = format!(
        "advertised bank_item_count={planted} actual={}",
        ledger.bank_item_count
    );
    assert!(
        msg.contains(&expected),
        "must name advertised + actual ({expected}):\n{msg}"
    );
    assert!(
        msg.contains(&format!("README.md:{}", site.line)),
        "must name file:line (README.md:{}):\n{msg}",
        site.line
    );
}

#[test]
fn write_refuses_missing_bank_item_count_and_writes_nothing() {
    let mut idx: serde_json::Value = serde_json::from_str(&live_units()).unwrap();
    idx.as_object_mut().unwrap().remove("bank_item_count");
    let engine = plant_tree(&idx.to_string());
    let readme = engine.join("../README.md");
    let before = fs::read(&readme).unwrap();
    let err = docs::sync_tree(&engine, Mode::Write).expect_err("missing key must refuse write");
    let msg = err.to_string();
    assert!(msg.contains("missing key bank_item_count"), "{msg}");
    let after = fs::read(&readme).unwrap();
    assert_eq!(
        before, after,
        "--write must not touch prose when the ledger is unsound"
    );
}

#[test]
fn write_refuses_zero_bank_item_count_and_writes_nothing() {
    let mut idx: serde_json::Value = serde_json::from_str(&live_units()).unwrap();
    idx["bank_item_count"] = serde_json::json!(0);
    let engine = plant_tree(&idx.to_string());
    let readme = engine.join("../README.md");
    let before = fs::read(&readme).unwrap();
    let err = docs::sync_tree(&engine, Mode::Write).expect_err("zero must refuse write");
    let msg = err.to_string();
    assert!(msg.contains("bank_item_count is 0"), "{msg}");
    let after = fs::read(&readme).unwrap();
    assert_eq!(before, after);
}

#[test]
fn write_restores_a_drifted_token() {
    let engine = plant_tree(&live_units());
    let ledger = docs::load_ledger(&engine).expect("sound ledger");
    let readme = engine.join("../README.md");
    let original = fs::read_to_string(&readme).unwrap();
    let site = docs::scan_text(&original)
        .into_iter()
        .find(|h| h.key == LedgerKey::BankItemCount)
        .expect("README must advertise bank_item_count");
    let planted = site.value.saturating_sub(1);
    let mut patched = original.clone();
    patched.replace_range(site.start..site.end, &planted.to_string());
    fs::write(&readme, &patched).unwrap();
    docs::sync_tree(&engine, Mode::Write).expect("write on sound ledger");
    let after = fs::read_to_string(&readme).unwrap();
    let restored = docs::scan_text(&after);
    assert!(
        restored
            .iter()
            .filter(|h| h.key == LedgerKey::BankItemCount)
            .all(|h| h.value == ledger.bank_item_count),
        "every bank site must equal ledger {}",
        ledger.bank_item_count
    );
    assert!(
        !after.contains(&format!("{planted} original item files")),
        "drifted token must be gone"
    );
}

#[test]
fn zero_advertisement_sites_is_error() {
    let engine = plant_tree(&live_units());
    fs::write(engine.join("../README.md"), "# empty\n").unwrap();
    fs::write(engine.join("../CHARTER.md"), "# empty\n").unwrap();
    fs::write(engine.join("README.md"), "# empty\n").unwrap();
    let err = docs::sync_tree(&engine, Mode::Check).expect_err("empty coverage is ERROR");
    let msg = err.to_string();
    assert!(
        msg.contains("advertisement site") && msg.contains("ERROR"),
        "anti-vacuous empty scan:\n{msg}"
    );
}

#[test]
fn live_tree_meets_ratchet_and_has_no_decoration() {
    let root = engine_root();
    let n = docs::sync_tree(&root, Mode::Check).expect("live tree must be in sync after (d)");
    assert!(
        n >= MIN_ADVERTISEMENT_SITES,
        "live site count {n} below floor {MIN_ADVERTISEMENT_SITES}"
    );
}

#[test]
fn grep_decoration_tier_is_gone_from_public_readme() {
    let readme = fs::read_to_string(workspace_root().join("README.md")).unwrap();
    assert!(
        !regex_naive(&readme, r"[0-9]+ scripts"),
        "bare script count survived"
    );
    assert!(
        !regex_naive(&readme, r"[0-9]+ Rust crates"),
        "bare crate count survived"
    );
    assert!(!readme.contains("k lines"), "~Nk lines decoration survived");
}

fn regex_naive(text: &str, _pat: &str) -> bool {
    for needle in [" scripts", " Rust crates"] {
        for (i, _) in text.match_indices(needle) {
            let before = &text[..i];
            if before
                .chars()
                .rev()
                .take_while(|c| c.is_ascii_digit())
                .count()
                > 0
            {
                return true;
            }
        }
    }
    false
}

#[test]
fn cli_docs_sync_check_on_live_tree_is_green() {
    cdcp()
        .env("CDCP_DEV", "1")
        .current_dir(engine_root())
        .args(["docs", "sync", "--check"])
        .assert()
        .success();
}

#[test]
fn cli_docs_is_hidden_from_learners_and_listed_for_dev() {
    let learner = cdcp()
        .env_remove("CDCP_DEV")
        .arg("--help")
        .assert()
        .success();
    let learner_out = String::from_utf8_lossy(&learner.get_output().stdout);
    assert!(
        !learner_out
            .lines()
            .any(|l| l.trim_start().starts_with("docs")),
        "docs leaked into learner help:\n{learner_out}"
    );

    let dev = cdcp().env("CDCP_DEV", "1").arg("--help").assert().success();
    let dev_out = String::from_utf8_lossy(&dev.get_output().stdout);
    assert!(
        dev_out.contains("docs"),
        "CDCP_DEV=1 --help must list docs:\n{dev_out}"
    );
}

#[test]
fn corpus_replay_seed_inputs_do_not_panic() {
    let dir = engine_root().join("fuzz/seed_corpus/docs_sync_scan");
    assert!(dir.is_dir(), "missing seed corpus {}", dir.display());
    let mut n = 0usize;
    for ent in fs::read_dir(&dir).unwrap() {
        let path = ent.unwrap().path();
        if path.is_file() {
            let bytes = fs::read(&path).unwrap();
            let _ = docs::scan_document(&bytes);
            n += 1;
        }
    }
    assert!(n >= 8, "seed corpus too small ({n}) — not a floor");
}

#[test]
fn unused_ledger_key_enum_is_named() {
    // Keeps LedgerKey::as_str in the public surface the fuzz target uses.
    assert_eq!(LedgerKey::WasmKib.as_str(), "wasm_kib");
}
