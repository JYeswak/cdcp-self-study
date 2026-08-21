//! Thin dispatcher for the substrate policy.
//!
//! The assertion logic and its unit tests live in `cdcp_registry_check`, where
//! the registry and shell-walk product logic already live. This file only
//! adapts the shared gate context and error convention.
#![forbid(unsafe_code)]

pub use cdcp_registry_check::substrate_guard::*;

use crate::registry::{GateCtx as DispatchCtx, GateError as DispatchError};

pub fn run(ctx: &DispatchCtx) -> Result<(), DispatchError> {
    let product_ctx = cdcp_registry_check::substrate_guard::GateCtx {
        root: ctx.root.clone(),
        args: ctx.args.clone(),
    };
    cdcp_registry_check::substrate_guard::evaluate(&product_ctx).map_err(|error| match error {
        cdcp_registry_check::substrate_guard::GateError::Violation(items) => {
            DispatchError::Violation(items)
        }
        cdcp_registry_check::substrate_guard::GateError::Usage(message) => {
            DispatchError::Usage(message)
        }
        cdcp_registry_check::substrate_guard::GateError::Error(message) => {
            DispatchError::Error(message)
        }
    })
}
