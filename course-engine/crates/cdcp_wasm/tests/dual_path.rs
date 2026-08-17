//! L4 dual-path: native digest == wasm32 digest via wasmtime.
//!
//! Fixtures: `goldens/fixtures/mock40_seed42` all-correct / all-wrong against the
//! full on-disk bank (same surface as `cdcp goldens check`).
//!
//! Requires:
//! - the shipped blob `web/assets/wasm/cdcp_wasm.wasm` (what the browser loads)
//! - wasmtime (dev-dep) to instantiate that blob
//!
//! The subject is the committed artifact, not `target/.../debug/cdcp_wasm.wasm`.
//! check.sh asserts that blob byte-equals a `--release --locked` rebuild.
//!
//! Skip policy (TESTING.md): the native==wasm comparison is `#[ignore]`. A missing
//! artifact must not score as PASS — `cargo test` prints `ignored`, not `ok`.
//! check.sh L4 runs `--include-ignored` with `CDCP_REQUIRE_WASM=1` when the wasm32
//! target is installed. Running the ignored test without an artifact panics.
//! `CDCP_FORCE_WASM_MISSING=1` forces that panic (anti-vacuous plant).
//! `CDCP_WASM_SUBJECT` overrides the path (selftest plants only).

use cdcp_assess::{Item, Quantity, Ratio, Response, SetCredit, Tolerance, ToleranceKind};
use cdcp_bank::Bank;
use cdcp_grade::{all_correct_attempt, all_wrong_attempt, grade_digest};
use cdcp_wasm::{
    engine_identities, grade_digest_json, score_digest_json, ENGINE_IDENTITY_ORACLE,
    ENGINE_IDENTITY_SUBJECT,
};
use std::path::{Path, PathBuf};

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

/// Relative path of the blob the learner runtime actually fetches.
const SHIPPED_WASM_REL: &str = "web/assets/wasm/cdcp_wasm.wasm";

/// Load the shipped wasm — never `target/wasm32-unknown-unknown/debug`.
///
/// Override with `CDCP_WASM_SUBJECT` (absolute or repo-relative) for plants.
/// `CDCP_FORCE_WASM_MISSING=1` is the anti-vacuous missing-artifact plant.
fn shipped_wasm() -> Result<PathBuf, String> {
    if std::env::var("CDCP_FORCE_WASM_MISSING").ok().as_deref() == Some("1") {
        return Err("CDCP_FORCE_WASM_MISSING=1 (anti-vacuous: no artifact)".into());
    }
    let root = repo_root();
    if let Ok(raw) = std::env::var("CDCP_WASM_SUBJECT") {
        let override_path = {
            let p = PathBuf::from(&raw);
            if p.is_absolute() {
                p
            } else {
                root.join(p)
            }
        };
        if override_path.is_file() {
            return Ok(override_path);
        }
        return Err(format!(
            "CDCP_WASM_SUBJECT is not a file: {}",
            override_path.display()
        ));
    }
    let shipped = root.join(SHIPPED_WASM_REL);
    if shipped.is_file() {
        Ok(shipped)
    } else {
        Err(format!(
            "shipped wasm missing: {} (dual-path loads the browser artifact, not target/.../debug)",
            shipped.display()
        ))
    }
}

/// Missing wasm is a failed comparison, never a passed one.
fn wasm_artifact_or_fail(result: Result<PathBuf, String>) -> PathBuf {
    result.unwrap_or_else(|e| {
        panic!(
            "native==wasm unproven: no wasm artifact ({e}). \
             This test is #[ignore] so `cargo test` reports ignored, not ok. \
             Run with --include-ignored (check.sh L4 sets CDCP_REQUIRE_WASM=1)."
        );
    })
}

/// Call a two-buffer guest digest export via wasmtime linear memory.
fn wasm_two_json_digest(
    wasm_path: &Path,
    export: &str,
    left: &str,
    right: &str,
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
    let digest = instance
        .get_typed_func::<(u32, u32, u32, u32), i32>(&mut store, export)
        .map_err(|e| format!("{export}: {e}"))?;
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

    let (left_ptr, left_len) = write_str(&mut store, &alloc, &memory, left)?;
    let (right_ptr, right_len) = write_str(&mut store, &alloc, &memory, right)?;

    let rc = digest
        .call(&mut store, (left_ptr, left_len, right_ptr, right_len))
        .map_err(|e| format!("{export} call: {e}"))?;

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

    let _ = free.call(&mut store, (left_ptr, left_len));
    let _ = free.call(&mut store, (right_ptr, right_len));

    let text = String::from_utf8(buf).map_err(|e| format!("utf8 out: {e}"))?;
    if rc < 0 {
        Err(format!("wasm {export} error: {text}"))
    } else {
        Ok(text)
    }
}

/// Call guest `cdcp_grade_digest` via wasmtime linear memory.
fn wasm_grade_digest(
    wasm_path: &Path,
    bank_json: &str,
    attempt_json: &str,
) -> Result<String, String> {
    wasm_two_json_digest(wasm_path, "cdcp_grade_digest", bank_json, attempt_json)
}

/// Call guest `cdcp_score_digest` via wasmtime linear memory.
fn wasm_score_digest(
    wasm_path: &Path,
    item_json: &str,
    response_json: &str,
) -> Result<String, String> {
    wasm_two_json_digest(wasm_path, "cdcp_score_digest", item_json, response_json)
}

/// Same as [`shipped_wasm`]: typed-assess dual-path also grades the blob that ships.
/// Freshness of that blob is check.sh's sha256 compare against a --release --locked rebuild.
fn shipped_wasm_for_assess() -> Result<PathBuf, String> {
    shipped_wasm()
}

struct AssessFixture {
    label: &'static str,
    item_json: String,
    response_json: String,
    pin: &'static str,
}

/// Three kinds named in the leftover. Digests are over ScoreReport, not the
/// item body. Single-select pin is the 64t.1 crate pin (still valid).
fn assess_fixtures() -> Vec<AssessFixture> {
    let single = Item::single_select(["utility", "genset", "both", "neither"], "genset").unwrap();
    let single_ok = Response::single_select("genset").unwrap();

    let multi = Item::multi_select(
        ["A-side", "B-side", "tie", "spare"],
        ["A-side", "B-side"],
        SetCredit::Jaccard,
    )
    .unwrap();
    let multi_subset = Response::multi_select(["A-side"]).unwrap();

    let numeric = Item::numeric_range(
        Quantity::new(Ratio::from_int(72), "kW").unwrap(),
        Tolerance::new(ToleranceKind::Absolute, Ratio::from_int(1)).unwrap(),
    )
    .unwrap();
    let numeric_ok =
        Response::numeric_range(Quantity::new(Ratio::from_int(72), "kW").unwrap()).unwrap();

    vec![
        AssessFixture {
            label: "single-select",
            item_json: serde_json::to_string(&single).unwrap(),
            response_json: serde_json::to_string(&single_ok).unwrap(),
            // {"earned":1,"full_credit":true,"kind":"single-select","out_of":1}
            pin: "b86064f06cabce71277297df37e985b36da1546566618b22e0a3ef628bfa9dba",
        },
        AssessFixture {
            label: "multi-select",
            item_json: serde_json::to_string(&multi).unwrap(),
            response_json: serde_json::to_string(&multi_subset).unwrap(),
            // Jaccard 1/2 → {"earned":1,"full_credit":false,"kind":"multi-select","out_of":2}
            pin: "5069e0aeca632a42ef29973ee9055437a94876934f61e12c9448d80dde714b30",
        },
        AssessFixture {
            label: "numeric-range",
            item_json: serde_json::to_string(&numeric).unwrap(),
            response_json: serde_json::to_string(&numeric_ok).unwrap(),
            // {"earned":1,"full_credit":true,"kind":"numeric-range","out_of":1}
            pin: "610b51a19742bf708672567fe7d251cdf522db4736a624af52ab139ca84dcf0e",
        },
    ]
}

#[test]
fn dual_path_loads_shipped_artifact_not_target_debug() {
    let path = shipped_wasm().expect("committed web/assets/wasm/cdcp_wasm.wasm must exist");
    let display = path.to_string_lossy();
    assert!(
        display.contains("web/assets/wasm/cdcp_wasm.wasm")
            || std::env::var("CDCP_WASM_SUBJECT").is_ok(),
        "dual-path subject must be the shipped blob, got {display}"
    );
    assert!(
        !display.contains("target/wasm32-unknown-unknown/debug"),
        "dual-path must not load the debug target wasm, got {display}"
    );
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
#[should_panic(expected = "native==wasm unproven")]
fn absent_wasm_artifact_is_not_a_passing_comparison() {
    let _ = wasm_artifact_or_fail(Err("no artifact".into()));
}

#[test]
#[ignore = "requires wasm32 artifact; cargo test -- --include-ignored (check.sh L4)"]
fn native_equals_wasm_mock40_seed42() {
    let wasm_path = wasm_artifact_or_fail(shipped_wasm());

    eprintln!("using shipped wasm subject: {}", wasm_path.display());
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

/// Native vs shipped wasm only — no golden pin.
///
/// The wasm-freshness plant changes a grade-affecting constant and rebuilds
/// *native only*. A golden pin would go RED even if this test still rebuilt
/// wasm from the mutated source. This test is the plant's needle: it stays
/// GREEN if the subject is rebuilt from the same tree, and RED when the
/// subject is the committed blob.
#[test]
#[ignore = "requires shipped wasm artifact; cargo test -- --include-ignored (check.sh L4)"]
fn shipped_wasm_matches_native_grade() {
    let wasm_path = wasm_artifact_or_fail(shipped_wasm());
    eprintln!("using shipped wasm subject: {}", wasm_path.display());
    let (oracle, subject) = engine_identities();
    assert_ne!(oracle, subject);

    let (bank, fix) = load_fixture_bank();
    let bank_json = bank.to_json_items().unwrap();
    let attempt = all_correct_attempt(&bank, &fix.exam_id, fix.seed, &fix.item_ids).unwrap();
    let native = grade_digest(&bank, &attempt).expect("native grade");
    let attempt_json = serde_json::to_string(&attempt).unwrap();
    let wasm_hex = wasm_grade_digest(&wasm_path, &bank_json, &attempt_json)
        .unwrap_or_else(|e| panic!("all-correct: wasm subject failed: {e}"));
    assert_eq!(
        native, wasm_hex,
        "all-correct: dual-path mismatch oracle={oracle} subject={subject}\n native={native}\n wasm  ={wasm_hex}"
    );
}

#[test]
fn native_assess_json_path_matches_pins() {
    for fx in assess_fixtures() {
        let native = score_digest_json(&fx.item_json, &fx.response_json)
            .unwrap_or_else(|e| panic!("{} native digest: {e}", fx.label));
        assert_eq!(
            native, fx.pin,
            "{} pin drifted — recompute and record",
            fx.label
        );
        assert_eq!(native.len(), 64, "{} digest width", fx.label);
        assert!(
            native.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')),
            "{} digest not lowercase hex",
            fx.label
        );
    }
}

/// Should-fail: a bare JSON number is not a quantity. Green here would mean
/// the schema floor is gone.
#[test]
fn assess_bare_number_is_error_not_digest() {
    let item = r#"{"kind":"numeric-range","expected":72,"tolerance":{"kind":"absolute","magnitude":{"num":1,"den":1}}}"#;
    let response =
        r#"{"kind":"numeric-range","submitted":{"value":{"num":72,"den":1},"units":"kW"}}"#;
    let err = score_digest_json(item, response).expect_err("bare number must not score");
    assert!(!err.is_empty(), "empty error string is not a typed failure");
}

#[test]
#[ignore = "requires wasm32 artifact; cargo test -- --include-ignored (check.sh L4)"]
fn native_equals_wasm_typed_assess() {
    let wasm_path = wasm_artifact_or_fail(shipped_wasm_for_assess());
    eprintln!("using shipped wasm subject: {}", wasm_path.display());
    let (oracle, subject) = engine_identities();
    assert_ne!(oracle, subject);

    for fx in assess_fixtures() {
        let native = score_digest_json(&fx.item_json, &fx.response_json)
            .unwrap_or_else(|e| panic!("{}: native: {e}", fx.label));
        assert_eq!(native, fx.pin, "{}: pin", fx.label);

        let wasm_hex = wasm_score_digest(&wasm_path, &fx.item_json, &fx.response_json)
            .unwrap_or_else(|e| panic!("{}: wasm subject failed: {e}", fx.label));
        assert_eq!(
            native, wasm_hex,
            "{}: dual-path mismatch oracle={oracle} subject={subject}\n native={native}\n wasm  ={wasm_hex}",
            fx.label
        );
        println!("ok assess dual-path {}: {native}", fx.label);
    }

    // Should-fail on the subject too: a crash that returned a digest would
    // still match a crashed oracle. The plant must be Err on both sides.
    let bad_item = r#"{"kind":"numeric-range","expected":72,"tolerance":{"kind":"absolute","magnitude":{"num":1,"den":1}}}"#;
    let bad_resp =
        r#"{"kind":"numeric-range","submitted":{"value":{"num":72,"den":1},"units":"kW"}}"#;
    assert!(
        score_digest_json(bad_item, bad_resp).is_err(),
        "native plant must stay red"
    );
    let wasm_err = wasm_score_digest(&wasm_path, bad_item, bad_resp);
    assert!(
        wasm_err.is_err(),
        "wasm plant must be Err, got {wasm_err:?}"
    );
}
