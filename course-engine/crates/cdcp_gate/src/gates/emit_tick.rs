use crate::registry::{GateCtx, GateError}; use cdcp_bank::tick_emitter::{self, TickError};
pub const NAME: &str = tick_emitter::NAME; pub const SUMMARY: &str = tick_emitter::SUMMARY;
#[rustfmt::skip] pub fn run(ctx: &GateCtx) -> Result<(), GateError> { tick_emitter::run(&ctx.root, &ctx.args).map(|text| println!("{text}")).map_err(|e| match e { TickError::Usage(m) => GateError::usage(m), TickError::Error(m) => GateError::error(m) }) }
