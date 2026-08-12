//! Crash-only fuzz: canonical_json on arbitrary JSON Values must never panic.
//!
//! Does NOT assert byte-idempotency for all JSON (serde_json may re-emit
//! out-of-range integers as floats with precision loss). GradeReport digests
//! only carry bounded integers; that property is covered by unit/proptest tests.
#![no_main]

use cdcp_core::canonical_json;
use libfuzzer_sys::fuzz_target;
use serde_json::Value;

fuzz_target!(|data: &[u8]| {
    let Ok(v) = serde_json::from_slice::<Value>(data) else {
        return;
    };
    let _ = canonical_json(&v);
});
