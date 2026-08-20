//! Thin dispatcher. Implementation lives in `cdcp_bank::verify_bank`
//! (EXTRACT-THEN-DELETE, bd-engine-not-gate-ar39.10). `check.sh` still runs
//! `cdcp_gate verify-bank`; this file must stay so the globbed registry
//! keeps the subcommand. Dual-path oracle `scripts/verify_bank.py` stays.

use crate::registry::{GateCtx, GateError};
use cdcp_bank::verify_bank::{self, Outcome};
use std::io::Write;

pub const NAME: &str = verify_bank::NAME;
pub const SUMMARY: &str = verify_bank::SUMMARY;

pub use verify_bank::{evaluate, find_topic_ids, py_repr, py_space, py_strip};

pub fn run(ctx: &GateCtx) -> Result<(), GateError> {
    // The oracle takes no arguments and silently ignores any it is given. This
    // port rejects them instead: a typo'd flag must not read as "the gate
    // passed". That is the single deliberate divergence outside the verdict
    // path, and it cannot change the bytes of any argument-free invocation.
    if let Some(a) = ctx.args.first() {
        return Err(GateError::usage(format!(
            "verify-bank takes no arguments; got {a:?}"
        )));
    }

    let Outcome {
        stdout,
        stderr,
        code,
    } = verify_bank::evaluate(&ctx.root);
    print!("{stdout}");
    let _ = std::io::stdout().flush();
    if !stderr.is_empty() {
        eprint!("{stderr}");
        let _ = std::io::stderr().flush();
    }
    if code != 0 {
        // See cdcp_bank::verify_bank: the oracle exits 1 with this report on
        // stdout, and byte-identical output is the acceptance bar. Routing
        // through `GateError` would write to stderr and exit 2 instead.
        std::process::exit(code);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    #[test]
    fn the_gate_takes_no_arguments() {
        let ctx = GateCtx::new(PathBuf::from("/tmp"), vec!["--bank".into()]);
        let err = run(&ctx).unwrap_err();
        assert_eq!(err.code(), crate::exit::USAGE);
    }

    #[test]
    fn the_live_repo_tree_is_green() {
        let root = crate::root::resolve(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        let out = evaluate(&root);
        assert_eq!(out.code, 0, "{}{}", out.stdout, out.stderr);
        let pass = out.stdout.as_str();
        assert!(pass.starts_with("PASS\n"), "{pass}");
        assert!(pass.contains("  source_class=original\n"));
        assert!(
            pass.contains(
                // 904/879 -> 957/931: the MANIFEST was corrected to the real file set in
                // fd4d6d8; the old pin predated that and had been red since.
                "  items=957 scanned, 931 approved (floors count the approved pool only)\n"
            ),
            "{pass}"
        );
        assert!(
            // m15 56 -> 106: module 15 grew with the CDFOS/CDFOM track.
            pass.contains("14: 46, 15: 106}") && pass.contains("14: 48, 15: 106}"),
            "{pass}"
        );
        assert!(
            pass.contains("  domain_floors=15 checked (approved pool)\n"),
            "live bank_policy.toml carries 15 [[domain_min]] rows:\n{pass}"
        );
    }
}
