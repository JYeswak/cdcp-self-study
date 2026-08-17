//! Thin dispatcher. Implementation lives in `cdcp_learn::knowledge_paths`
//! (EXTRACT-THEN-DELETE, bd-engine-not-gate-ar39.11). `check.sh` still runs
//! `cdcp_gate verify-knowledge-paths`; this file must stay so the globbed
//! registry keeps the subcommand. Dual-path `scripts/verify_knowledge_paths.py`
//! was already retired (jhd.33); this extract does not resurrect it.

use crate::registry::{GateCtx, GateError};
use cdcp_learn::knowledge_paths;
use std::io::Write;

pub const NAME: &str = knowledge_paths::NAME;
pub const SUMMARY: &str = knowledge_paths::SUMMARY;

pub use knowledge_paths::{
    evaluate, is_absolute_posix, is_relative_to, join_posix, norm_posix, py_repr, py_resolve,
    py_space, py_splitlines, py_str, py_strip, py_truthy, DOMAINS_REL, KNOWLEDGE_REL,
};

pub fn run(ctx: &GateCtx) -> Result<(), GateError> {
    // The oracle ignores sys.argv; this gate does not (module header). A typo'd
    // flag must not read as a clean full-tree run.
    ctx.reject_unknown_flags(&[])?;

    // The oracle resolves its own location, so the printed root is symlink-free.
    let root = ctx.root.canonicalize().unwrap_or_else(|_| ctx.root.clone());
    let root_str = knowledge_paths::norm_posix(&root.to_string_lossy());

    let outcome = knowledge_paths::evaluate(&root_str).map_err(GateError::error)?;
    print!("{}", outcome.stdout);
    let _ = std::io::stdout().flush();

    if outcome.code != 0 {
        // See cdcp_learn::knowledge_paths: the retired oracle exited 1 with an
        // empty stderr, and this port's acceptance bar is that report on
        // stdout. Routing through `GateError` would write to stderr and exit
        // 2/4 instead.
        std::process::exit(outcome.code);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    #[test]
    fn any_argument_is_usage_never_a_silent_full_tree_run() {
        let ctx = GateCtx::new(PathBuf::from("/"), vec!["--staged".into()]);
        assert_eq!(run(&ctx).unwrap_err().code(), crate::exit::USAGE);
    }

    #[test]
    fn the_live_repo_tree_is_green() {
        let root = crate::root::resolve(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        let root_str =
            knowledge_paths::norm_posix(&root.canonicalize().unwrap_or(root).to_string_lossy());
        let out = evaluate(&root_str).expect("evaluate");
        assert_eq!(out.code, 0, "{}", out.stdout);
        assert!(out.stdout.starts_with("PASS\n"), "{}", out.stdout);
        assert!(
            out.stdout.contains("  primary_notes_checked="),
            "{}",
            out.stdout
        );
    }
}
