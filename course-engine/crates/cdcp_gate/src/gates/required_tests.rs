use crate::registry::{GateCtx, GateError};
use cdcp_bank::required_tests::{self, Eval};
pub const NAME: &str = required_tests::NAME;
pub const SUMMARY: &str = required_tests::SUMMARY;
#[rustfmt::skip] pub fn run(ctx: &GateCtx) -> Result<(), GateError> { ctx.reject_unknown_flags(&[])?; match required_tests::evaluate(&ctx.root) { Eval::Ok(text) => { println!("{text}"); Ok(()) }, Eval::Violation(items) => Err(GateError::violation(items)), Eval::Error(message) => Err(GateError::error(message)) } }
