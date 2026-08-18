//! Thin dispatcher. Implementation lives in `cdcp_bank::verify_content_lock`
//! (EXTRACT-THEN-DELETE, bd-engine-not-gate-ar39.13). `check.sh` still runs
//! `cdcp_gate verify-content-lock`; this file must stay so the globbed
//! registry keeps the subcommand. Dual-path oracle `scripts/verify_content_lock.py`
//! stays retired (gna0); this extract does not resurrect it.

use crate::registry::{GateCtx, GateError};
use cdcp_bank::verify_content_lock;
use std::io::Write;

pub const NAME: &str = verify_content_lock::NAME;
pub const SUMMARY: &str = verify_content_lock::SUMMARY;

/// Literal pin for `registries/goldens-couplings.toml` `bank.hash-pin`.
/// Must stay a string literal assigned to this name in THIS file so the
/// couplings extractor can still read `::GOLDEN_REL`.
pub const GOLDEN_REL: &str = "goldens/bank_hash.txt";

pub use verify_content_lock::{
    bank_hash_timeout, evaluate, live_bank_hash, py_eq_one, py_repr, py_repr_opt, py_slice, py_str,
    py_strip, py_truthy, resolve_pinned, selftest_mutate, sha256_file, sha256_hex_bytes, verify,
    LockedRoot, Outcome, Sha256, Verdict, BANK_ARG, BANK_HASH_TIMEOUT_S, LOCKED_ROOTS, LOCK_REL,
    SELFTEST_ENV, TIMEOUT_ENV,
};

pub fn run(ctx: &GateCtx) -> Result<(), GateError> {
    ctx.reject_unknown_flags(&[])?;

    // The oracle resolves its own location (`Path(__file__).resolve()`), so its
    // ROOT is symlink-free and the `missing content.lock at <path>` message
    // prints a real path. Do the same to the engine root.
    let root = ctx.root.canonicalize().unwrap_or_else(|_| ctx.root.clone());

    let selftest = std::env::var(SELFTEST_ENV).unwrap_or_default() == "1";
    let outcome = if selftest {
        verify_content_lock::selftest_mutate(&root)
    } else {
        verify_content_lock::evaluate(&root)
    };

    print!("{}", outcome.stdout);
    let _ = std::io::stdout().flush();
    eprint!("{}", outcome.stderr);
    let _ = std::io::stderr().flush();

    if outcome.code != 0 {
        // See cdcp_bank::verify_content_lock: the oracle exits 1 with this exact
        // stderr, and this port's acceptance bar is byte-identical output.
        // Routing through `GateError` would rewrite the text and change the code.
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
        assert_eq!(NAME, "verify-content-lock");
        assert!(crate::registry::find(NAME).is_some());
    }

    #[test]
    fn an_unknown_flag_is_usage_never_a_silent_pass() {
        let ctx = GateCtx::new(PathBuf::from("/tmp"), vec!["--bnak".into()]);
        let err = run(&ctx).unwrap_err();
        assert_eq!(err.code(), crate::exit::USAGE);
    }

    #[test]
    fn golden_rel_stays_the_literal_the_couplings_ledger_pins() {
        assert_eq!(GOLDEN_REL, "goldens/bank_hash.txt");
        assert_eq!(GOLDEN_REL, verify_content_lock::GOLDEN_REL);
    }

    /// Live-repo pin. Fixture-based green path lives with the impl
    /// (`cdcp_bank::verify_content_lock::tests::a_fully_pinned_fixture_is_green_and_the_receipt_names_the_counts`).
    #[test]
    fn the_live_tree_is_green_and_the_receipt_names_the_counts() {
        let root = crate::root::resolve(std::path::Path::new(env!("CARGO_MANIFEST_DIR")))
            .expect("engine root")
            .canonicalize()
            .expect("canonical root");
        let out = evaluate(&root);
        assert_eq!(out.code, 0, "stderr: {}", out.stderr);
        assert!(out.stderr.is_empty(), "{}", out.stderr);
        assert!(
            out.stdout
                .starts_with("verify_content_lock: PASS bank_hash="),
            "{}",
            out.stdout
        );
        assert!(out.stdout.contains('\u{2026}'), "{}", out.stdout);
        assert!(
            out.stdout.contains(
                "verify_content_lock: covered roots (every file found under these is pinned \
                 and matched): knowledge/*.toml=9 web/content/modules/*.md=16 ../modules/*.md=15\n"
            ),
            "{}",
            out.stdout
        );
        assert!(
            out.stdout
                .contains("verify_content_lock: NOT covered: anything outside those roots"),
            "{}",
            out.stdout
        );
    }
}
