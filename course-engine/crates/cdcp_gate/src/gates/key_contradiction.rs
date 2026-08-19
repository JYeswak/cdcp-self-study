use crate::registry::{GateCtx, GateError};
use cdcp_bank::key_contradiction::{self, Eval};
pub const NAME: &str = key_contradiction::NAME;
pub const SUMMARY: &str = key_contradiction::SUMMARY;
#[rustfmt::skip] pub fn run(ctx: &GateCtx) -> Result<(), GateError> { ctx.reject_unknown_flags(&[])?; match key_contradiction::evaluate(&ctx.root) { Eval::Ok(text) => { println!("{text}"); Ok(()) }, Eval::Violation(items) => Err(GateError::violation(items)), Eval::Error(message) => Err(GateError::error(message)) } }
