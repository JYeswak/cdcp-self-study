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

    fn count_pin(root: &Path, id: &str) -> u64 {
        cdcp_registry_check::count_pins::expected_value(root, id)
            .unwrap_or_else(|error| panic!("count pin {id:?}: {error}"))
    }

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
        let file_count = count_pin(&root, "bank.file-count");
        let approved_count = count_pin(&root, "bank.approved-count");
        assert!(
            pass.contains(&format!(
                "  items={file_count} scanned, {approved_count} approved (floors count the approved pool only)\n"
            )),
            "{pass}"
        );
        let m14_approved = count_pin(&root, "bank.module-14.approved-count");
        let m15_approved = count_pin(&root, "bank.module-15.approved-count");
        let m14_files = count_pin(&root, "bank.module-14.file-count");
        let m15_files = count_pin(&root, "bank.module-15.file-count");
        assert!(
            pass.contains(&format!("14: {m14_approved}, 15: {m15_approved}}}"))
                && pass.contains(&format!("14: {m14_files}, 15: {m15_files}}}")),
            "{pass}"
        );
        assert!(
            pass.contains("  domain_floors=15 checked (approved pool)\n"),
            "live bank_policy.toml carries 15 [[domain_min]] rows:\n{pass}"
        );
    }
}
