//! Thin dispatcher for the product-owned roadmap/doc consistency evaluator.
//!
//! The implementation lives in cdcp_registry_check::verify_doc_consistency;
//! this file preserves the generated cdcp_gate subcommand.

use crate::registry::{GateCtx, GateError};

pub const NAME: &str = "verify-doc-consistency";
pub const SUMMARY: &str = "roadmap milestone status agrees and publication truth holds";

pub fn run(ctx: &GateCtx) -> Result<(), GateError> {
    match cdcp_registry_check::verify_doc_consistency::evaluate_root(&ctx.root, &ctx.args) {
        cdcp_registry_check::verify_doc_consistency::DocConsistencyOutcome::Output {
            text,
            code,
        } => {
            print!("{text}");
            if code != 0 {
                std::process::exit(code.into());
            }
            Ok(())
        }
        cdcp_registry_check::verify_doc_consistency::DocConsistencyOutcome::Usage(message) => {
            Err(GateError::usage(message))
        }
    }
}
