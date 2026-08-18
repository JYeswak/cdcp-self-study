//! Thin dispatcher. Implementation lives in `cdcp_bank::verify_coverage`
//! (EXTRACT-THEN-DELETE, bd-engine-not-gate-ar39.14). `check.sh` still runs
//! `cdcp_gate verify-coverage`; this file must stay so the globbed
//! registry keeps the subcommand. Dual-path oracle `scripts/verify_coverage.py`
//! stays.

use crate::registry::{GateCtx, GateError};
use cdcp_bank::verify_coverage;
use std::io::Write;

pub const NAME: &str = verify_coverage::NAME;
pub const SUMMARY: &str = verify_coverage::SUMMARY;

pub use verify_coverage::{
    evaluate, join_posix, json_dumps, json_str, norm_posix, parse_args, py_float_repr, py_int,
    py_int_from_str, py_isdigit_ascii, py_iter, py_repr, py_repr_value, py_resolve, py_space,
    py_str_value, py_strip, py_truthy, py_type_name, Args, Halt, IntErr, Outcome, APPROVED,
    DEFAULT_BANK, DEFAULT_DOMAINS, DEFAULT_N, DEFAULT_POLICY, KNOWN_STATUSES, MAX_REPORT,
};

pub fn run(ctx: &GateCtx) -> Result<(), GateError> {
    let args = verify_coverage::parse_args(&ctx.args).map_err(GateError::usage)?;

    // The Python resolves its own location (`Path(__file__).resolve()`), so the
    // printed default paths are symlink-free. Do the same to the engine root —
    // and only to the root: an absolute `--bank`/`--policy`/`--domains` is
    // printed exactly as `PurePosixPath` normalises it, never canonicalised.
    let root = ctx.root.canonicalize().unwrap_or_else(|_| ctx.root.clone());
    let root_str = verify_coverage::norm_posix(&root.to_string_lossy());

    let outcome = verify_coverage::evaluate(&root_str, &args);
    print!("{}", outcome.stdout);
    let _ = std::io::stdout().flush();
    if !outcome.stderr.is_empty() {
        eprint!("{}", outcome.stderr);
        let _ = std::io::stderr().flush();
    }
    if outcome.code != 0 {
        // See cdcp_bank::verify_coverage: the oracle exits 1 with this report on
        // stdout, and byte-identical output is the acceptance bar. Routing
        // through `GateError` would write to stderr and exit 2/4 instead.
        std::process::exit(outcome.code);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn the_gate_is_registered_under_a_kebab_case_name() {
        assert_eq!(NAME, "verify-coverage");
        assert!(crate::registry::find(NAME).is_some());
    }

    #[test]
    fn an_unknown_flag_is_usage_never_a_silent_pass() {
        let ctx = GateCtx::new(PathBuf::from("/tmp"), vec!["--bnak".into(), "/x".into()]);
        let err = run(&ctx).unwrap_err();
        assert_eq!(err.code(), crate::exit::USAGE);
    }
}
