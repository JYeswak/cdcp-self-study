//! Thin dispatcher. Implementation lives in `cdcp_bank::near_duplicate`
//! (EXTRACT-THEN-DELETE, bd-engine-not-gate-ar39.7). `check.sh` still runs
//! `cdcp_gate near-duplicate-items`; this file must stay so the globbed
//! registry keeps the subcommand.

use crate::registry::{GateCtx, GateError};
use cdcp_bank::near_duplicate::{self, Eval};

pub const NAME: &str = near_duplicate::NAME;
pub const SUMMARY: &str = near_duplicate::SUMMARY;

pub fn run(ctx: &GateCtx) -> Result<(), GateError> {
    ctx.reject_unknown_flags(&[])?;
    match near_duplicate::evaluate(&ctx.root) {
        Eval::Ok(s) => {
            print!("{s}");
            Ok(())
        }
        Eval::Violation(items) => Err(GateError::violation(items)),
        Eval::Error(m) => Err(GateError::error(m)),
    }
}
