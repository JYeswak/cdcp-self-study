//! Typed assessment beyond four letters.
//!
//! Kinds: single-select, multi-select, ordering, numeric-range,
//! topology-selection, procedural-sequence. Scoring is a pure function of
//! `(item, response)` using integer or rational arithmetic only. Identical
//! fixtures produce identical [`ScoreReport`] bytes on any host that can
//! compile this crate (including `wasm32-unknown-unknown`).
//!
//! Sequence kinds name their partial-credit policy. `all-or-nothing` refuses
//! partial credit. There is no default policy.
//!
//! A numeric-range item or response without units, or without a declared
//! tolerance, is a schema ERROR. A bare JSON number is not a quantity.
//!
//! This crate does not flatten new kinds into A–D. [`lift_letter_mcq`] is a
//! one-way lift of an existing four-letter item so its grade can be checked
//! here unchanged. Bank migration of the live 804 items is not this crate.
#![forbid(unsafe_code)]

mod error;
mod ratio;
mod score;
mod types;

pub use error::AssessError;
pub use ratio::Ratio;
pub use score::score;
pub use types::{
    lift_letter_mcq, Id, Item, Quantity, Response, Score, ScoreReport, SequenceCredit, SetCredit,
    Tolerance, ToleranceKind, Units, KINDS,
};

use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// Score plus the item kind, for a host-independent digest.
pub fn score_report(item: &Item, response: &Response) -> Result<ScoreReport, AssessError> {
    let s = score(item, response)?;
    Ok(ScoreReport {
        kind: item.kind_name().to_string(),
        earned: s.earned(),
        out_of: s.out_of(),
        full_credit: s.is_full(),
    })
}

/// SHA-256 hex of [`canonical_json`] over [`ScoreReport`].
pub fn score_digest(item: &Item, response: &Response) -> Result<String, AssessError> {
    let report = score_report(item, response)?;
    let bytes = canonical_json(&report)?;
    Ok(sha256_hex(&bytes))
}

/// Dual-path surface: JSON item + JSON response → digest. Same contract
/// `cdcp_grade` / `cdcp_wasm` use for the letter bank.
pub fn score_digest_json(item_json: &str, response_json: &str) -> Result<String, AssessError> {
    let item = Item::from_json(item_json)?;
    let response = Response::from_json(response_json)?;
    score_digest(&item, &response)
}

/// Compact JSON with object keys sorted. Same idea as `cdcp_core::canonical_json`,
/// kept here so this crate does not depend on the four-letter core.
pub fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, AssessError> {
    let v = serde_json::to_value(value).map_err(|e| AssessError::Json(e.to_string()))?;
    serde_json::to_vec(&sort_value(v)).map_err(|e| AssessError::Json(e.to_string()))
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

fn sort_value(v: Value) -> Value {
    match v {
        Value::Object(map) => {
            let mut bt = BTreeMap::new();
            for (k, val) in map {
                bt.insert(k, sort_value(val));
            }
            let mut out = serde_json::Map::new();
            for (k, val) in bt {
                out.insert(k, val);
            }
            Value::Object(out)
        }
        Value::Array(arr) => Value::Array(arr.into_iter().map(sort_value).collect()),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_are_the_six_named_in_the_bead() {
        assert_eq!(
            KINDS,
            [
                "single-select",
                "multi-select",
                "ordering",
                "numeric-range",
                "topology-selection",
                "procedural-sequence",
            ]
        );
        assert_eq!(KINDS.len(), 6);
    }

    #[test]
    fn digest_is_idempotent_and_64_hex() {
        let item = Item::single_select(["utility", "genset", "both", "neither"], "genset").unwrap();
        let ok = Response::single_select("genset").unwrap();
        let d1 = score_digest(&item, &ok).unwrap();
        let d2 = score_digest(&item, &ok).unwrap();
        assert_eq!(d1, d2);
        assert_eq!(d1.len(), 64);
        assert!(d1.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')));
        // Pinned payload: {"earned":1,"full_credit":true,"kind":"single-select","out_of":1}
        assert_eq!(
            d1,
            "b86064f06cabce71277297df37e985b36da1546566618b22e0a3ef628bfa9dba"
        );
    }

    #[test]
    fn json_path_matches_native_digest() {
        let item = Item::single_select(["utility", "genset", "both", "neither"], "genset").unwrap();
        let ok = Response::single_select("genset").unwrap();
        let item_json = serde_json::to_string(&item).unwrap();
        let resp_json = serde_json::to_string(&ok).unwrap();
        assert_eq!(
            score_digest(&item, &ok).unwrap(),
            score_digest_json(&item_json, &resp_json).unwrap()
        );
    }
}
