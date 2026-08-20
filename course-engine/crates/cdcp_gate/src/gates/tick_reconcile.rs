use crate::registry::{GateCtx, GateError};
use cdcp_bank::tick_reconcile::{self, ReconcileError};
pub const NAME: &str = tick_reconcile::NAME;
pub const SUMMARY: &str = tick_reconcile::SUMMARY;
#[rustfmt::skip] pub fn run(ctx: &GateCtx) -> Result<(), GateError> { tick_reconcile::run(&ctx.root).map(|text| println!("{text}")).map_err(|e| match e { ReconcileError::Unreconciled(v) => GateError::violation(v), ReconcileError::Error(m) => GateError::error(m) }) }
