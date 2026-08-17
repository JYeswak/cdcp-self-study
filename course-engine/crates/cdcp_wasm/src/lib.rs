//! WASM dual-path surface (ORACLE-GAUNTLET L4 + schedule law + typed assess).
//!
//! **Oracle** = native `cdcp_grade` / `cdcp_schedule` / `cdcp_assess` on the host.
//! **Subject** = this crate compiled to `wasm32-unknown-unknown`.
//! **Comparator** = hex digest equality for grade and typed assess; numeric
//! equality for schedule.
//!
//! Fixed-bank grade path only (no assemble / shuffle) — valid L4 for frozen fixtures.
//! Typed assess extends that contract past four letters: item JSON + response
//! JSON → `ScoreReport` digest, integer/rational scoring only.
//! Schedule exports the short-interval ladder and mastery thresholds so the
//! browser cannot keep a second, unpinned implementation of those laws.
//!
//! Unsafe is confined to the wasm32 C ABI (`abi` module). Native builds forbid it.
#![cfg_attr(not(target_arch = "wasm32"), forbid(unsafe_code))]

use cdcp_bank::Bank;
use cdcp_core::ExamAttempt;
use cdcp_grade::grade_digest;
use cdcp_schedule::{self, ReviewAttempt};

pub use cdcp_schedule::{
    cap_days, first_step_days, is_mastered, is_practiced_milli, is_practiced_ratio, migrate_card,
    migrate_state_version, next_interval_days, ratio_to_milli, validate_schedule, validate_steps,
    validate_thresholds, DAY_MS, INTERVAL_STEPS, MASTERED_MILLI, MASTERED_MIN_GAP_MS,
    PRACTICED_MILLI, STATE_VERSION, STATE_VERSION_UNVERSIONED,
};

/// Engine identity labels asserted at the dual-path comparator (docs/ORACLE-GAUNTLET.md).
pub const ENGINE_IDENTITY_ORACLE: &str = "cdcp_grade-native";
pub const ENGINE_IDENTITY_SUBJECT: &str = "cdcp_wasm-wasm32";

/// Pure grade-digest from JSON payloads (works on native and wasm).
///
/// * `bank_json` — JSON array of `BankItem`, or `{"items":[...]}`.
/// * `attempt_json` — `ExamAttempt` (`exam_id`, `seed`, `bank_hash`, `answers`).
///
/// Returns lowercase hex SHA-256 of `canonical_json(GradeReport)`.
pub fn grade_digest_json(bank_json: &str, attempt_json: &str) -> Result<String, String> {
    let bank = Bank::from_json_str(bank_json).map_err(|e| e.to_string())?;
    let attempt: ExamAttempt =
        serde_json::from_str(attempt_json).map_err(|e| format!("attempt json: {e}"))?;
    grade_digest(&bank, &attempt).map_err(|e| e.to_string())
}

/// Typed-assess digest from JSON payloads (works on native and wasm).
///
/// * `item_json` — `cdcp_assess::Item` (tagged `kind`).
/// * `response_json` — `cdcp_assess::Response` (tagged `kind`, must match).
///
/// Returns lowercase hex SHA-256 of canonical `ScoreReport`. Scoring is
/// integer/rational only — this wrapper does not introduce f32/f64 compares.
pub fn score_digest_json(item_json: &str, response_json: &str) -> Result<String, String> {
    cdcp_assess::score_digest_json(item_json, response_json).map_err(|e| e.to_string())
}

/// Subject/oracle identity pair for comparator entry checks.
pub fn engine_identities() -> (&'static str, &'static str) {
    (ENGINE_IDENTITY_ORACLE, ENGINE_IDENTITY_SUBJECT)
}

/// JSON mastery payload: `[{"ratio":0.9,"at_ms":…}, …]` or `ratio_milli`.
#[derive(serde::Deserialize)]
struct AttemptIn {
    #[serde(default)]
    ratio: Option<f64>,
    #[serde(default)]
    ratio_milli: Option<u32>,
    #[serde(default)]
    at_ms: i64,
}

fn attempt_milli(a: &AttemptIn) -> u32 {
    if let Some(m) = a.ratio_milli {
        return m;
    }
    a.ratio.map(ratio_to_milli).unwrap_or(0)
}

/// Mastered verdict from JSON attempts. Empty array is not mastered (not an error).
pub fn is_mastered_json(json: &str) -> Result<bool, String> {
    let raw: Vec<AttemptIn> =
        serde_json::from_str(json).map_err(|e| format!("mastered json: {e}"))?;
    let attempts: Vec<ReviewAttempt> = raw
        .iter()
        .map(|a| ReviewAttempt {
            ratio_milli: attempt_milli(a),
            at_ms: a.at_ms,
        })
        .collect();
    Ok(is_mastered(&attempts))
}

#[cfg(target_arch = "wasm32")]
mod abi {
    //! Minimal linear-memory ABI for host runtimes (wasmtime/wasmi).
    //!
    //! Protocol:
    //! 1. Host calls `cdcp_alloc(n)` for bank and attempt UTF-8 buffers, writes bytes.
    //! 2. Host calls `cdcp_grade_digest(bank_ptr, bank_len, att_ptr, att_len)`.
    //! 3. On success (return ≥ 0): return value is length of hex at `cdcp_last_ptr()`.
    //! 4. On failure (return < 0): error UTF-8 length is `-rc`, bytes at `cdcp_last_ptr()`.
    //! 5. Host calls `cdcp_free` on alloc'd input buffers when done.

    use super::{grade_digest_json, score_digest_json};
    use std::cell::RefCell;

    thread_local! {
        static LAST: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    }

    #[no_mangle]
    pub extern "C" fn cdcp_alloc(len: usize) -> *mut u8 {
        let mut v = Vec::<u8>::with_capacity(len);
        let ptr = v.as_mut_ptr();
        // Leak to guest; host must call cdcp_free.
        std::mem::forget(v);
        ptr
    }

    /// # Safety
    /// `ptr` must come from `cdcp_alloc(capacity)` with `capacity >= len`.
    #[no_mangle]
    pub unsafe extern "C" fn cdcp_free(ptr: *mut u8, len: usize) {
        if ptr.is_null() || len == 0 {
            return;
        }
        let _ = Vec::from_raw_parts(ptr, len, len);
    }

    #[no_mangle]
    pub extern "C" fn cdcp_last_ptr() -> *const u8 {
        LAST.with(|c| c.borrow().as_ptr())
    }

    #[no_mangle]
    pub extern "C" fn cdcp_last_len() -> usize {
        LAST.with(|c| c.borrow().len())
    }

    /// Returns hex length (≥0) on success, or `-err_len` on failure.
    ///
    /// # Safety
    /// Pointers must refer to valid UTF-8 buffers of the given lengths in wasm linear memory.
    #[no_mangle]
    pub unsafe extern "C" fn cdcp_score_digest(
        item_ptr: *const u8,
        item_len: usize,
        response_ptr: *const u8,
        response_len: usize,
    ) -> i32 {
        let item = std::slice::from_raw_parts(item_ptr, item_len);
        let response = std::slice::from_raw_parts(response_ptr, response_len);
        let item_s = match std::str::from_utf8(item) {
            Ok(s) => s,
            Err(e) => return store_err(format!("item utf8: {e}")),
        };
        let response_s = match std::str::from_utf8(response) {
            Ok(s) => s,
            Err(e) => return store_err(format!("response utf8: {e}")),
        };
        match score_digest_json(item_s, response_s) {
            Ok(hex) => {
                let bytes = hex.into_bytes();
                let n = bytes.len() as i32;
                LAST.with(|c| *c.borrow_mut() = bytes);
                n
            }
            Err(e) => store_err(e),
        }
    }

    /// Returns hex length (≥0) on success, or `-err_len` on failure.
    ///
    /// # Safety
    /// Pointers must refer to valid UTF-8 buffers of the given lengths in wasm linear memory.
    #[no_mangle]
    pub unsafe extern "C" fn cdcp_grade_digest(
        bank_ptr: *const u8,
        bank_len: usize,
        attempt_ptr: *const u8,
        attempt_len: usize,
    ) -> i32 {
        let bank = std::slice::from_raw_parts(bank_ptr, bank_len);
        let attempt = std::slice::from_raw_parts(attempt_ptr, attempt_len);
        let bank_s = match std::str::from_utf8(bank) {
            Ok(s) => s,
            Err(e) => return store_err(format!("bank utf8: {e}")),
        };
        let attempt_s = match std::str::from_utf8(attempt) {
            Ok(s) => s,
            Err(e) => return store_err(format!("attempt utf8: {e}")),
        };
        match grade_digest_json(bank_s, attempt_s) {
            Ok(hex) => {
                let bytes = hex.into_bytes();
                let n = bytes.len() as i32;
                LAST.with(|c| *c.borrow_mut() = bytes);
                n
            }
            Err(e) => store_err(e),
        }
    }

    fn store_err(msg: String) -> i32 {
        let bytes = msg.into_bytes();
        let n = bytes.len() as i32;
        LAST.with(|c| *c.borrow_mut() = bytes);
        -n
    }

    /// 1 if the compiled schedule is valid; 0 if empty/zero (cannot happen
    /// unless the crate was built against a broken constant).
    #[no_mangle]
    pub extern "C" fn cdcp_schedule_ok() -> i32 {
        i32::from(cdcp_schedule::validate_schedule().is_ok())
    }

    #[no_mangle]
    pub extern "C" fn cdcp_interval_step_count() -> i32 {
        match cdcp_schedule::validate_steps(&cdcp_schedule::INTERVAL_STEPS) {
            Ok(()) => cdcp_schedule::INTERVAL_STEPS.len() as i32,
            Err(_) => -1,
        }
    }

    /// Step at `index`, or -1 if out of range / invalid ladder.
    #[no_mangle]
    pub extern "C" fn cdcp_interval_step(index: i32) -> i32 {
        if index < 0 {
            return -1;
        }
        cdcp_schedule::INTERVAL_STEPS
            .get(index as usize)
            .copied()
            .map(|s| s as i32)
            .unwrap_or(-1)
    }

    #[no_mangle]
    pub extern "C" fn cdcp_next_interval_days(current: i32, correct: i32) -> i32 {
        cdcp_schedule::next_interval_days(current, correct != 0) as i32
    }

    #[no_mangle]
    pub extern "C" fn cdcp_day_ms() -> i32 {
        cdcp_schedule::DAY_MS as i32
    }

    #[no_mangle]
    pub extern "C" fn cdcp_practiced_milli() -> i32 {
        match cdcp_schedule::validate_thresholds(
            cdcp_schedule::PRACTICED_MILLI,
            cdcp_schedule::MASTERED_MILLI,
        ) {
            Ok(()) => cdcp_schedule::PRACTICED_MILLI as i32,
            Err(_) => -1,
        }
    }

    #[no_mangle]
    pub extern "C" fn cdcp_mastered_milli() -> i32 {
        match cdcp_schedule::validate_thresholds(
            cdcp_schedule::PRACTICED_MILLI,
            cdcp_schedule::MASTERED_MILLI,
        ) {
            Ok(()) => cdcp_schedule::MASTERED_MILLI as i32,
            Err(_) => -1,
        }
    }

    #[no_mangle]
    pub extern "C" fn cdcp_mastered_min_gap_ms() -> i32 {
        cdcp_schedule::MASTERED_MIN_GAP_MS as i32
    }

    /// 1 if `ratio_milli` meets practiced; 0 otherwise.
    #[no_mangle]
    pub extern "C" fn cdcp_is_practiced(ratio_milli: i32) -> i32 {
        if ratio_milli < 0 {
            return 0;
        }
        i32::from(cdcp_schedule::is_practiced_milli(ratio_milli as u32))
    }

    /// Current persisted schedule-state version.
    #[no_mangle]
    pub extern "C" fn cdcp_state_version() -> i32 {
        cdcp_schedule::STATE_VERSION as i32
    }

    /// Migrate a persisted version. Returns the current version, or `-1`
    /// when the input is unknown / negative (ERROR — never a default).
    #[no_mangle]
    pub extern "C" fn cdcp_migrate_state_version(from: i32) -> i32 {
        if from < 0 {
            return -1;
        }
        match cdcp_schedule::migrate_state_version(from as u32) {
            Ok(v) => v as i32,
            Err(_) => -1,
        }
    }

    /// 1 = mastered, 0 = not, <0 = parse error (`-err_len`, bytes at last_ptr).
    ///
    /// # Safety
    /// `ptr` must refer to `len` bytes of UTF-8 JSON in wasm linear memory.
    #[no_mangle]
    pub unsafe extern "C" fn cdcp_is_mastered(ptr: *const u8, len: usize) -> i32 {
        let bytes = std::slice::from_raw_parts(ptr, len);
        let json = match std::str::from_utf8(bytes) {
            Ok(s) => s,
            Err(e) => return store_err(format!("mastered utf8: {e}")),
        };
        match super::is_mastered_json(json) {
            Ok(true) => 1,
            Ok(false) => 0,
            Err(e) => store_err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cdcp_bank::Bank;
    use cdcp_grade::{all_correct_attempt, all_wrong_attempt, grade_digest};
    use std::path::PathBuf;

    fn bank_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../bank/items")
    }

    fn fixture_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../goldens/fixtures/mock40_seed42.json")
    }

    #[derive(serde::Deserialize)]
    struct SampleFixture {
        exam_id: String,
        seed: u64,
        item_ids: Vec<String>,
    }

    /// Subset bank containing only fixture item_ids (smaller wasm payloads).
    fn fixture_subset_bank() -> (Bank, SampleFixture) {
        let full = Bank::load_dir(&bank_path()).expect("load bank");
        let fix: SampleFixture =
            serde_json::from_str(&std::fs::read_to_string(fixture_path()).unwrap()).unwrap();
        let items: Vec<_> = fix
            .item_ids
            .iter()
            .map(|id| {
                full.get(id)
                    .unwrap_or_else(|| panic!("missing fixture item {id}"))
                    .clone()
            })
            .collect();
        let bank = Bank::from_items(items).expect("subset bank");
        (bank, fix)
    }

    #[test]
    fn identities_are_distinct() {
        let (oracle, subject) = engine_identities();
        assert_ne!(oracle, subject);
        assert!(oracle.contains("native"));
        assert!(subject.contains("wasm"));
    }

    #[test]
    fn assess_json_path_matches_pinned_single_select_digest() {
        let item =
            cdcp_assess::Item::single_select(["utility", "genset", "both", "neither"], "genset")
                .unwrap();
        let ok = cdcp_assess::Response::single_select("genset").unwrap();
        let item_json = serde_json::to_string(&item).unwrap();
        let resp_json = serde_json::to_string(&ok).unwrap();
        let via = score_digest_json(&item_json, &resp_json).unwrap();
        assert_eq!(via, cdcp_assess::score_digest(&item, &ok).unwrap());
        // Same pin as cdcp_assess::tests::digest_is_idempotent_and_64_hex / 64t.1.
        assert_eq!(
            via,
            "b86064f06cabce71277297df37e985b36da1546566618b22e0a3ef628bfa9dba"
        );
    }

    #[test]
    fn json_path_matches_native_grade_digest() {
        let (bank, fix) = fixture_subset_bank();
        let bank_json = bank.to_json_items().unwrap();
        for (label, attempt) in [
            (
                "all-correct",
                all_correct_attempt(&bank, &fix.exam_id, fix.seed, &fix.item_ids).unwrap(),
            ),
            (
                "all-wrong",
                all_wrong_attempt(&bank, &fix.exam_id, fix.seed, &fix.item_ids).unwrap(),
            ),
        ] {
            let native = grade_digest(&bank, &attempt).unwrap();
            let attempt_json = serde_json::to_string(&attempt).unwrap();
            let via_json = grade_digest_json(&bank_json, &attempt_json).unwrap();
            assert_eq!(native, via_json, "{label} json path mismatch");
            assert_eq!(native.len(), 64);
        }
    }
}
