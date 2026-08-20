//! Thin dispatcher for the bank-to-learner-pack freshness coupling.
//!
//! The product logic lives in `cdcp_data::pack_freshness`; this file keeps the
//! `cdcp_gate pack-freshness` command discoverable without putting policy in the
//! gate crate.

use cdcp_data::evaluate_pack_freshness;

use crate::registry::{GateCtx, GateError};

pub const NAME: &str = "pack-freshness";
pub const SUMMARY: &str = "fail when the learner pack predates the authored bank";

pub fn run(ctx: &GateCtx) -> Result<(), GateError> {
    ctx.reject_unknown_flags(&[])?;
    let report = evaluate_pack_freshness(&ctx.root).map_err(GateError::error)?;
    println!(
        "pack_freshness: bank_files={} pack_files={} bank_commit={} bank_epoch={} pack_commit={} pack_epoch={}",
        report.bank_files,
        report.pack_files,
        report.bank_commit,
        report.bank_epoch,
        report.pack_commit,
        report.pack_epoch
    );
    if !report.is_fresh() {
        return Err(GateError::violation(vec![format!(
            "learner pack is stale: bank {} ({}) is newer than pack {} ({})",
            report.bank_commit, report.bank_epoch, report.pack_commit, report.pack_epoch
        )]));
    }
    println!(
        "pack_freshness: PASS (freshness only; content correctness remains covered by pack/golden checks)"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn unknown_arguments_are_usage_errors() {
        let ctx = GateCtx::new(PathBuf::from("/tmp"), vec!["--staged".into()]);
        assert_eq!(run(&ctx).unwrap_err().code(), crate::exit::USAGE);
    }

}
