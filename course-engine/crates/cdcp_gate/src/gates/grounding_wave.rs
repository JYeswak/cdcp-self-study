use crate::registry::{GateCtx, GateError};
use cdcp_bank::grounding_wave::{self, Eval};
pub const NAME: &str = grounding_wave::NAME;
pub const SUMMARY: &str = grounding_wave::SUMMARY;
#[rustfmt::skip] pub fn run(ctx: &GateCtx) -> Result<(), GateError> { ctx.reject_unknown_flags(&[])?; match grounding_wave::evaluate(&ctx.root) { Eval::Ok(text) => { println!("{text}"); Ok(()) }, Eval::Violation(items) => Err(GateError::violation(items)), Eval::Error(message) => Err(GateError::error(message)) } }
