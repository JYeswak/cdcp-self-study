//! WASM dual-path grade surface (ORACLE-GAUNTLET L4).
//!
//! **Oracle** = native `cdcp_grade` on the host.  
//! **Subject** = this crate compiled to `wasm32-unknown-unknown`.  
//! **Comparator** = hex digest equality for the same `(bank_json, attempt_json)`.
//!
//! Fixed-bank grade path only (no assemble / shuffle) — valid L4 for frozen fixtures.
//!
//! Unsafe is confined to the wasm32 C ABI (`abi` module). Native builds forbid it.
#![cfg_attr(not(target_arch = "wasm32"), forbid(unsafe_code))]

use cdcp_bank::Bank;
use cdcp_core::ExamAttempt;
use cdcp_grade::grade_digest;

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

/// Subject/oracle identity pair for comparator entry checks.
pub fn engine_identities() -> (&'static str, &'static str) {
    (ENGINE_IDENTITY_ORACLE, ENGINE_IDENTITY_SUBJECT)
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

    use super::grade_digest_json;
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
