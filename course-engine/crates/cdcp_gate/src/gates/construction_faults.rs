//! Thin dispatcher. Implementation lives in cdcp_bank::construction_faults.

use crate::registry::{GateCtx, GateError};
use cdcp_bank::construction_faults::{self, Eval};

pub const NAME: &str = construction_faults::NAME;
pub const SUMMARY: &str = construction_faults::SUMMARY;

pub fn run(ctx: &GateCtx) -> Result<(), GateError> {
    ctx.reject_unknown_flags(&[])?;
    match construction_faults::evaluate(&ctx.root) {
        Eval::Ok(text) => {
            println!("{text}");
            Ok(())
        }
        Eval::Violation(items) => Err(GateError::violation(items)),
        Eval::Error(message) => Err(GateError::error(message)),
    }
}
