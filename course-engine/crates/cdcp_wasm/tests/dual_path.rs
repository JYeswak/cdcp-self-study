//! L4 dual-path: native digest == wasm32 digest via wasmtime.
//!
//! Fixtures: `goldens/fixtures/mock40_seed42` all-correct / all-wrong against the
//! full on-disk bank (same surface as `cdcp goldens check`).
//!
//! Requires:
//! - `wasm32-unknown-unknown` rustup target
//! - buildable `cdcp_wasm` as wasm32 cdylib
//!
//! If the wasm toolchain/build is missing and `CDCP_REQUIRE_WASM` is unset, the
//! dual-path assertion returns early (check.sh records SKIP-honest, not full L4 green).
//! Set `CDCP_REQUIRE_WASM=1` to hard-fail when wasm is unavailable.

use cdcp_bank::Bank;
use cdcp_grade::{all_correct_attempt, all_wrong_attempt, grade_digest};
use cdcp_wasm::{
    engine_identities, grade_digest_json, ENGINE_IDENTITY_ORACLE, ENGINE_IDENTITY_SUBJECT,
};
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn bank_path() -> PathBuf {
    repo_root().join("bank/items")
}

fn fixture_path() -> PathBuf {
    repo_root().join("goldens/fixtures/mock40_seed42.json")
}

fn golden_path(name: &str) -> PathBuf {
    repo_root().join("goldens").join(name)
}

#[derive(serde::Deserialize)]
struct SampleFixture {
    exam_id: String,
    seed: u64,
    item_ids: Vec<String>,
}

fn load_fixture_bank() -> (Bank, SampleFixture) {
    let bank = Bank::load_dir(&bank_path()).expect("load bank");
    let fix: SampleFixture =
        serde_json::from_str(&std::fs::read_to_string(fixture_path()).unwrap()).unwrap();
    (bank, fix)
}

fn ensure_wasm_built() -> Result<PathBuf, String> {
    let root = repo_root();
    let candidates = [
        root.join("target/wasm32-unknown-unknown/debug/cdcp_wasm.wasm"),
        root.join("target/wasm32-unknown-unknown/release/cdcp_wasm.wasm"),
    ];
    for c in &candidates {
        if c.is_file() {
            return Ok(c.clone());
        }
    }
    let status = Command::new("cargo")
        .args([
            "build",
            "-p",
            "cdcp_wasm",
            "--target",
            "wasm32-unknown-unknown",
            "--manifest-path",
        ])
        .arg(root.join("Cargo.toml"))
        .current_dir(&root)
        .status()
        .map_err(|e| format!("spawn cargo: {e}"))?;
    if !status.success() {
        return Err(format!(
            "cargo build -p cdcp_wasm --target wasm32-unknown-unknown failed: {status}"
        ));
    }
    let built = root.join("target/wasm32-unknown-unknown/debug/cdcp_wasm.wasm");
    if built.is_file() {
        Ok(built)
    } else {
        Err(format!(
            "wasm artifact missing after build: {}",
            built.display()
        ))
    }
}

/// Call guest `cdcp_grade_digest` via wasmtime linear memory.
fn wasm_grade_digest(
    wasm_path: &Path,
    bank_json: &str,
    attempt_json: &str,
) -> Result<String, String> {
    use wasmtime::*;

    let engine = Engine::default();
    let module = Module::from_file(&engine, wasm_path).map_err(|e| format!("module: {e}"))?;
    let mut store = Store::new(&engine, ());
    let instance =
        Instance::new(&mut store, &module, &[]).map_err(|e| format!("instantiate: {e}"))?;

    let memory = instance
        .get_memory(&mut store, "memory")
        .ok_or_else(|| "wasm export `memory` missing".to_string())?;

    let alloc = instance
        .get_typed_func::<u32, u32>(&mut store, "cdcp_alloc")
        .map_err(|e| format!("cdcp_alloc: {e}"))?;
    let free = instance
        .get_typed_func::<(u32, u32), ()>(&mut store, "cdcp_free")
        .map_err(|e| format!("cdcp_free: {e}"))?;
    let grade = instance
        .get_typed_func::<(u32, u32, u32, u32), i32>(&mut store, "cdcp_grade_digest")
        .map_err(|e| format!("cdcp_grade_digest: {e}"))?;
    let last_ptr = instance
        .get_typed_func::<(), u32>(&mut store, "cdcp_last_ptr")
        .map_err(|e| format!("cdcp_last_ptr: {e}"))?;
    let last_len = instance
        .get_typed_func::<(), u32>(&mut store, "cdcp_last_len")
        .map_err(|e| format!("cdcp_last_len: {e}"))?;

    let write_str = |store: &mut Store<()>,
                     alloc: &TypedFunc<u32, u32>,
                     memory: &Memory,
                     s: &str|
     -> Result<(u32, u32), String> {
        let bytes = s.as_bytes();
        let len = bytes.len() as u32;
        let ptr = alloc
            .call(&mut *store, len)
            .map_err(|e| format!("alloc: {e}"))?;
        memory
            .write(&mut *store, ptr as usize, bytes)
            .map_err(|e| format!("mem write: {e}"))?;
        Ok((ptr, len))
    };

    let (bank_ptr, bank_len) = write_str(&mut store, &alloc, &memory, bank_json)?;
    let (att_ptr, att_len) = write_str(&mut store, &alloc, &memory, attempt_json)?;

    let rc = grade
        .call(&mut store, (bank_ptr, bank_len, att_ptr, att_len))
        .map_err(|e| format!("grade call: {e}"))?;

    let out_ptr = last_ptr
        .call(&mut store, ())
        .map_err(|e| format!("last_ptr: {e}"))? as usize;
    let out_len = last_len
        .call(&mut store, ())
        .map_err(|e| format!("last_len: {e}"))? as usize;
    let mut buf = vec![0u8; out_len];
    memory
        .read(&store, out_ptr, &mut buf)
        .map_err(|e| format!("mem read: {e}"))?;

    let _ = free.call(&mut store, (bank_ptr, bank_len));
    let _ = free.call(&mut store, (att_ptr, att_len));

    let text = String::from_utf8(buf).map_err(|e| format!("utf8 out: {e}"))?;
    if rc < 0 {
        Err(format!("wasm grade error: {text}"))
    } else {
        Ok(text)
    }
}

#[test]
fn engine_identities_distinct_at_comparator() {
    let (oracle, subject) = engine_identities();
    assert_eq!(oracle, ENGINE_IDENTITY_ORACLE);
    assert_eq!(subject, ENGINE_IDENTITY_SUBJECT);
    assert_ne!(
        oracle, subject,
        "comparator must see distinct EngineIdentity labels"
    );
}

#[test]
fn native_json_path_matches_goldens() {
    let (bank, fix) = load_fixture_bank();
    let bank_json = bank.to_json_items().unwrap();
    let cases = [
        (
            "all-correct",
            all_correct_attempt(&bank, &fix.exam_id, fix.seed, &fix.item_ids).unwrap(),
            "mock40_seed42_all_correct.sha256",
        ),
        (
            "all-wrong",
            all_wrong_attempt(&bank, &fix.exam_id, fix.seed, &fix.item_ids).unwrap(),
            "mock40_seed42_all_wrong.sha256",
        ),
    ];
    for (label, attempt, golden_name) in cases {
        let native = grade_digest(&bank, &attempt).unwrap();
        let via = grade_digest_json(&bank_json, &serde_json::to_string(&attempt).unwrap()).unwrap();
        assert_eq!(native, via, "{label} json path");
        let expected = std::fs::read_to_string(golden_path(golden_name))
            .unwrap()
            .trim()
            .to_string();
        assert_eq!(native, expected, "{label} golden pin");
    }
}

#[test]
fn native_equals_wasm_mock40_seed42() {
    let wasm_path = match ensure_wasm_built() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("SKIP wasm dual-path: {e}");
            if std::env::var("CDCP_REQUIRE_WASM").ok().as_deref() == Some("1") {
                panic!("CDCP_REQUIRE_WASM=1 but wasm unavailable: {e}");
            }
            return;
        }
    };

    eprintln!("using wasm subject: {}", wasm_path.display());
    let (oracle, subject) = engine_identities();
    assert_ne!(oracle, subject);

    let (bank, fix) = load_fixture_bank();
    let bank_json = bank.to_json_items().unwrap();

    let cases = [
        (
            "all-correct",
            all_correct_attempt(&bank, &fix.exam_id, fix.seed, &fix.item_ids).unwrap(),
            "mock40_seed42_all_correct.sha256",
        ),
        (
            "all-wrong",
            all_wrong_attempt(&bank, &fix.exam_id, fix.seed, &fix.item_ids).unwrap(),
            "mock40_seed42_all_wrong.sha256",
        ),
    ];

    for (label, attempt, golden_name) in cases {
        let native = grade_digest(&bank, &attempt).expect("native grade");
        let attempt_json = serde_json::to_string(&attempt).unwrap();
        let oracle_json = grade_digest_json(&bank_json, &attempt_json).unwrap();
        assert_eq!(native, oracle_json, "{label}: native vs json oracle");

        let expected = std::fs::read_to_string(golden_path(golden_name))
            .unwrap()
            .trim()
            .to_string();
        assert_eq!(native, expected, "{label}: golden pin");

        let wasm_hex = wasm_grade_digest(&wasm_path, &bank_json, &attempt_json)
            .unwrap_or_else(|e| panic!("{label}: wasm subject failed: {e}"));
        assert_eq!(
            native, wasm_hex,
            "{label}: dual-path mismatch oracle={oracle} subject={subject}\n native={native}\n wasm  ={wasm_hex}"
        );
        assert_eq!(native.len(), 64, "{label}: digest width");
        println!("ok dual-path {label}: {native}");
    }
}
