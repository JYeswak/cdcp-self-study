//! Thin dispatcher for the product-owned doc-facts evaluator.
//!
//! The implementation lives in cdcp_registry_check::doc_facts; this file
//! remains so the generated cdcp_gate doc-facts subcommand is unchanged.

use crate::registry::{GateCtx, GateError};

pub use cdcp_registry_check::doc_facts::{
    corpus_root, evaluate, evaluate_root, parse_registry, schema_errors, strip_markers,
    walk_markdown, Artifact, Doc, Exclude, Fact, GateOutcome, Marker, Probe, Registry, Report,
    ARTIFACT_KINDS, ITEM_KEYWORDS, MARKER_CLOSE, MARKER_OPEN, MAX_DEPTH, MAX_MARKDOWN_FILES,
    MIN_FACTS, MIN_MARKER_SITES, MIN_NEGATIVE_SITES, MIN_QUESTION_LEN, MIN_REASON_LEN,
    NEVER_EXCLUDABLE, PROBE_KINDS, REGISTRY_PATH, SKIP_DIRS,
};

pub const NAME: &str = "doc-facts";
pub const SUMMARY: &str =
    "present-tense prose claims about code carry a yes/no the tree recomputes";

pub fn run(ctx: &GateCtx) -> Result<(), GateError> {
    match evaluate_root(&ctx.root, &ctx.args) {
        GateOutcome::Ok(text) => {
            print!("{text}");
            Ok(())
        }
        GateOutcome::Violation(items) => Err(GateError::violation(items)),
        GateOutcome::Error(message) => Err(GateError::error(message)),
        GateOutcome::Usage(message) => Err(GateError::usage(message)),
    }
}
