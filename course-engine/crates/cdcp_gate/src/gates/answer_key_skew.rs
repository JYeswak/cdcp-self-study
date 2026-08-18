//! Thin dispatcher. Implementation lives in `cdcp_bank::answer_key_skew`
//! (EXTRACT-THEN-DELETE, bd-engine-not-gate-ar39.16). `check.sh` still runs
//! `cdcp_gate answer-key-skew`; this file keeps that public subcommand stable.

use crate::registry::{GateCtx, GateError};
use cdcp_bank::answer_key_skew::{self, Eval};

pub const NAME: &str = answer_key_skew::NAME;
pub const SUMMARY: &str = answer_key_skew::SUMMARY;

pub fn run(ctx: &GateCtx) -> Result<(), GateError> {
    ctx.reject_unknown_flags(&[])?;
    match answer_key_skew::evaluate(&ctx.root) {
        Eval::Ok(text) => {
            println!("{text}");
            Ok(())
        }
        Eval::Violation(items) => Err(GateError::violation(items)),
        Eval::Error(message) => Err(GateError::error(message)),
    }
}
