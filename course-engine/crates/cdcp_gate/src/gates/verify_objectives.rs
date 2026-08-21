//! Thin dispatcher. Implementation lives in `cdcp_learn::objectives`
//! (EXTRACT-THEN-DELETE, bd-engine-not-gate-ar39.12). `check.sh` still runs
//! `cdcp_gate verify-objectives`; this file must stay so the globbed
//! registry keeps the subcommand. The Rust implementation is the sole gate-path
//! implementation; known-bad legs live in `scripts/selftest_l7_objectives.sh`.

use crate::registry::{GateCtx, GateError};
use cdcp_learn::objectives;
use std::io::Write;

pub const NAME: &str = objectives::NAME;
pub const SUMMARY: &str = objectives::SUMMARY;

pub use objectives::{
    evaluate, join_posix, json_dumps, json_str, norm_posix, parse_args, py_float_repr, py_int,
    py_int_from_str, py_isdigit_ascii, py_iter, py_repr, py_repr_value, py_space, py_str_value,
    py_strip, py_truthy, py_type_name, Args, Halt, IntErr, Outcome, APPROVED, DEFAULT_BANK,
    DEFAULT_CLAIMS, DEFAULT_DOMAINS, DEFAULT_OBJECTIVES, DEFAULT_POLICY, DEFAULT_TOPICS,
    KNOWN_STATUSES, MIN_ITEMS_PER_MODULE,
};

pub fn run(ctx: &GateCtx) -> Result<(), GateError> {
    let args = objectives::parse_args(&ctx.args).map_err(GateError::usage)?;

    // The Python resolves its own location (`Path(__file__).resolve()`), so the
    // printed default paths are symlink-free. Do the same to the engine root —
    // and only to the root: an absolute option value is printed exactly as
    // `PurePosixPath` normalises it, never canonicalised.
    let root = ctx.root.canonicalize().unwrap_or_else(|_| ctx.root.clone());
    let root_str = objectives::norm_posix(&root.to_string_lossy());

    let outcome = objectives::evaluate(&root_str, &args);
    print!("{}", outcome.stdout);
    let _ = std::io::stdout().flush();
    if !outcome.stderr.is_empty() {
        eprint!("{}", outcome.stderr);
        let _ = std::io::stderr().flush();
    }
    if outcome.code != 0 {
        // See cdcp_learn::objectives: the oracle exits 1 with this report on
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
        assert_eq!(NAME, "verify-objectives");
        assert!(crate::registry::find(NAME).is_some());
    }

    #[test]
    fn an_unknown_flag_is_usage_never_a_silent_pass() {
        let ctx = GateCtx::new(PathBuf::from("/tmp"), vec!["--bnak".into(), "/x".into()]);
        let err = run(&ctx).unwrap_err();
        assert_eq!(err.code(), crate::exit::USAGE);
    }
}
