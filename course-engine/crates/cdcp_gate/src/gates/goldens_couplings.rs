//! Thin dispatcher for the golden-coupling policy.
//!
//! The ledger schema, artifact discovery, pin extraction and tests live in
//! `cdcp_registry_check`; this file preserves the established gate command.
#![forbid(unsafe_code)]

pub use cdcp_registry_check::goldens_couplings::*;

use crate::registry::{GateCtx as DispatchCtx, GateError as DispatchError};

pub fn run(ctx: &DispatchCtx) -> Result<(), DispatchError> {
    let product_ctx = cdcp_registry_check::goldens_couplings::GateCtx {
        root: ctx.root.clone(),
        args: ctx.args.clone(),
    };
    cdcp_registry_check::goldens_couplings::run_gate(&product_ctx).map_err(|error| match error {
        cdcp_registry_check::goldens_couplings::GateError::Violation(items) => {
            DispatchError::Violation(items)
        }
        cdcp_registry_check::goldens_couplings::GateError::Usage(message) => {
            DispatchError::Usage(message)
        }
        cdcp_registry_check::goldens_couplings::GateError::Error(message) => {
            DispatchError::Error(message)
        }
    })
}
