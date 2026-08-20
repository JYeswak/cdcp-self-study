//! Thin dispatcher. Implementation lives in `cdcp_bank::validate_grounding`
//! (EXTRACT-THEN-DELETE, bd-engine-not-gate-ar39.13). `check.sh` still runs
//! `cdcp_gate validate-grounding`; this file must stay so the globbed
//! registry keeps the subcommand. The former Python differential oracle is
//! retired; the product module's unit suite retains the known-bad legs.

use crate::registry::{GateCtx, GateError};
use cdcp_bank::validate_grounding;
use std::io::Write;

pub const NAME: &str = validate_grounding::NAME;
pub const SUMMARY: &str = validate_grounding::SUMMARY;

pub use validate_grounding::{
    corpus_root_errors, corpus_roots, evaluate, format_help, format_usage, load_corpus_text,
    overlap_score, parse_args, py_digit, py_float, py_float_repr, py_int, py_repr, py_space,
    py_str, py_truthy, py_word, search, tokenize, topic_labels, universal_newlines, word_runs,
    Args, Node, Outcome, CORPUS_PUBLIC_MARKER, CORPUS_PUBLIC_REL, CORPUS_SUFFIXES, FREE_EVIDENCE,
    ITEMS_REL, KNOWLEDGE_REL, MAX_REPORT, MIN_CORPUS_CHARS, MIN_SCANNED_ITEMS, PROG,
    SIBLING_CORPUS_DIRS, STOP, TOPICS_REL,
};

pub fn run(ctx: &GateCtx) -> Result<(), GateError> {
    let outcome = match validate_grounding::parse_args(&ctx.args) {
        Ok(a) => {
            // The Python resolves its own location, so the corpus walk starts
            // from a symlink-free root. Do the same.
            let root = ctx.root.canonicalize().unwrap_or_else(|_| ctx.root.clone());
            validate_grounding::evaluate(&root, &a)
        }
        Err(o) => o,
    };
    print!("{}", outcome.stdout);
    let _ = std::io::stdout().flush();
    eprint!("{}", outcome.stderr);
    let _ = std::io::stderr().flush();

    if outcome.code != 0 {
        // See cdcp_bank::validate_grounding: the oracle exits 1 (or argparse's
        // 2) with the report on stdout, and this port's acceptance bar is
        // byte-identical output. Routing through `GateError` would write to
        // stderr and exit 2/4 instead.
        std::process::exit(outcome.code);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_gate_is_registered_under_a_kebab_case_name() {
        assert_eq!(NAME, "validate-grounding");
        assert!(crate::registry::find(NAME).is_some());
    }

    #[test]
    fn an_unknown_flag_is_usage_never_a_silent_pass() {
        let err = parse_args(&["--bnak".into()]).unwrap_err();
        assert_eq!(err.code, 2);
        assert!(err.stdout.is_empty());
        assert!(err.stderr.starts_with("usage: validate_grounding.py"));
    }
}
