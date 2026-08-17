//! Thin dispatcher. Implementation lives in `cdcp_bank::orphans`
//! (EXTRACT-THEN-DELETE, bd-engine-not-gate-ar39.9). `check.sh` still runs
//! `cdcp_gate verify-orphans`; this file must stay so the globbed registry
//! keeps the subcommand. Dual-path oracle `scripts/verify_orphans.py` stays.

use crate::registry::{GateCtx, GateError};
use cdcp_bank::orphans::{self, Outcome};
use std::io::Write;

pub const NAME: &str = orphans::NAME;
pub const SUMMARY: &str = orphans::SUMMARY;

pub use orphans::{
    evaluate, find_topic_ids, is_absolute_posix, join_posix, norm_posix, parse_args, py_repr,
    py_space, py_str, py_strip, py_truthy, DEFAULT_BANK, DEFAULT_TOPICS, MAX_REPORT,
};

pub fn run(ctx: &GateCtx) -> Result<(), GateError> {
    let (bank, topics) = orphans::parse_args(&ctx.args).map_err(GateError::usage)?;

    // The Python resolves its own location (`Path(__file__).resolve()`), so the
    // printed default paths are symlink-free. Do the same to the engine root —
    // and only to the root: an absolute `--bank`/`--topics` is printed exactly
    // as `PurePosixPath` normalises it, never canonicalised.
    let root = ctx.root.canonicalize().unwrap_or_else(|_| ctx.root.clone());
    let root_str = orphans::norm_posix(&root.to_string_lossy());

    let Outcome { stdout, code } = orphans::evaluate(&root_str, &bank, &topics);
    print!("{stdout}");
    let _ = std::io::stdout().flush();

    if code != 0 {
        // See cdcp_bank::orphans: the oracle exits 1 with an empty stderr, and
        // this port's acceptance bar is byte-identical output. Routing through
        // `GateError` would write to stderr and exit 2/4 instead.
        std::process::exit(code);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_apply_when_no_flags_are_given() {
        let (b, t) = parse_args(&[]).unwrap();
        assert_eq!(b, DEFAULT_BANK);
        assert_eq!(t, DEFAULT_TOPICS);
    }

    #[test]
    fn an_unknown_flag_is_usage_never_a_silent_pass() {
        let args = vec!["--bnak".to_string(), "/x".to_string()];
        assert_eq!(
            parse_args(&args)
                .err()
                .map(|e| GateError::usage(e).code())
                .unwrap(),
            crate::exit::USAGE,
            "a typo must not read as a default-path run"
        );
    }
}
