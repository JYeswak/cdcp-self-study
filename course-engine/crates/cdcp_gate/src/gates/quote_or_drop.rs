//! Thin dispatcher for the periodic citation truth receipt.
//!
//! Normal invocation is hermetic. `--refresh` is an explicit authoring sweep
//! that performs network I/O and writes the committed receipt; it is not part
//! of the check chain.

use crate::registry::{GateCtx, GateError};
use cdcp_bank::quote_or_drop::{self, Eval};

pub const NAME: &str = quote_or_drop::NAME;
pub const SUMMARY: &str = quote_or_drop::SUMMARY;

pub fn run(ctx: &GateCtx) -> Result<(), GateError> {
    ctx.reject_unknown_flags(&["--refresh"])?;
    if ctx.has_flag("--refresh") {
        let (receipt, counts) = quote_or_drop::refresh(&ctx.root).map_err(GateError::error)?;
        println!(
            "quote_or_drop refresh: cited={} http_resolved={} resolved_for_grounding={} dead={} bot_blocked={} supporting={} non_supporting={} unverifiable={} item_files={} receipt={}",
            counts.cited,
            counts.http_resolved,
            counts.resolved,
            counts.dead,
            counts.bot_blocked,
            counts.supporting,
            counts.non_supporting,
            counts.unverifiable,
            receipt.item_file_denominator,
            quote_or_drop::RECEIPT
        );
        return Ok(());
    }
    match quote_or_drop::evaluate(&ctx.root) {
        Eval::Ok(text) => {
            println!("{text}");
            Ok(())
        }
        Eval::Violation(items) => Err(GateError::violation(items)),
        Eval::Error(message) => Err(GateError::error(message)),
    }
}
