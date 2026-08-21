//! Thin dispatcher for the advertised known-bad injection-count gate.
//!
//! The implementation and its tests live in `cdcp_registry_check`; this file
//! preserves the established `cdcp_gate verify-injection-count` command while
//! keeping assertion logic out of the dispatcher crate.
#![forbid(unsafe_code)]

pub use cdcp_registry_check::verify_injection_count::{cardinals, NAME, SUMMARY};

use crate::registry::{GateCtx as DispatchCtx, GateError as DispatchError};

pub fn run(ctx: &DispatchCtx) -> Result<(), DispatchError> {
    let product_ctx = cdcp_registry_check::verify_injection_count::GateCtx {
        root: ctx.root.clone(),
        args: ctx.args.clone(),
    };
    cdcp_registry_check::verify_injection_count::run_gate(&product_ctx).map_err(|error| match error {
        cdcp_registry_check::verify_injection_count::GateError::Violation(items) => {
            DispatchError::Violation(items)
        }
        cdcp_registry_check::verify_injection_count::GateError::Usage(message) => {
            DispatchError::Usage(message)
        }
        cdcp_registry_check::verify_injection_count::GateError::Error(message) => {
            DispatchError::Error(message)
        }
    })
}
