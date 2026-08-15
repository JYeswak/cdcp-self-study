//! Guards for the two stated limits that are not the build_learn sweep
//! (bd-zhnd). The sweep's CHARTER pair lives in `build_learn_charter_pair.rs`.
//!
//! These tests live in `cdcp_learn`, not `cdcp_gate`: growing the gate crate
//! is the ratchet `gate_shrink.toml` exists to stop. The live compilers
//! (this crate / `cdcp_cli`) own the product-side pins.
//!
//! Anti-vacuous: an empty scan of README, of the gate source, or of the
//! generator source is an ERROR, never a pass.

use std::path::{Path, PathBuf};

fn engine_root() -> PathBuf {
    cdcp_learn::resolve_engine_root(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
}

fn repo_root() -> PathBuf {
    engine_root()
        .parent()
        .expect("engine root has a parent")
        .to_path_buf()
}

fn read(path: &Path) -> String {
    assert!(
        path.is_file(),
        "required file missing: {} — an empty scan is an ERROR, not a pass",
        path.display()
    );
    let text =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    assert!(
        !text.trim().is_empty(),
        "{} is empty — an empty scan is an ERROR, not a pass",
        path.display()
    );
    text
}

// ── 1. MIN_ADVERTISEMENT_SITES ────────────────────────────────────────────
//
// Why a literal 5 is right, and cannot be derived from this-run parse
// count: if the floor were `advertised.len()` of the README being
// scanned, losing a site would drop the floor and stay GREEN — the
// exact partial-coverage hole the floor exists to close. It also cannot
// be derived from REGISTERED_SUITES: that is a different population
// (suites, not advertisement sites).
//
// It IS the number of sites the shipped parent README is contracted to
// carry. The badge markup is two of them. Adding a site is free (the
// floor is not an equality). Removing or obscuring one is a deliberate
// edit that must move this inventory and the gate constant together.

/// Shapes the shipped README is contracted to carry. Length is the
/// recorded argument for `MIN_ADVERTISEMENT_SITES = 5`.
const SHIPPED_ADVERTISEMENT_SHAPES: &[&str] = &[
    "badge_alt: known-bad (shell selftest suites): N injections",
    "badge_url: known--bad_(...)N_injections",
    "tldr_gate: N known-bad injections (shell selftest suites)",
    "prose: N known-bad faults",
    "l4_row: N injections (shell selftest suites",
];

/// Digit-branch of the gate's advertisement scanner: a number followed
/// by optional `known-bad` and then `injection(s)` / `faults`. Enough
/// to count the shipped README (all five sites use digits). A
/// word-spelled rewrite of a site would drop the count and fail closed.
fn count_digit_advertisement_sites(text: &str) -> usize {
    let lower = text.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut i = 0usize;
    let mut n = 0usize;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            let mut j = i;
            let mut sep = 0usize;
            while j < bytes.len() && (bytes[j].is_ascii_whitespace() || bytes[j] == b'_') {
                sep += 1;
                j += 1;
            }
            if sep == 0 {
                continue;
            }
            let after_sep = &lower[j..];
            let tail = if after_sep.starts_with("known-bad") {
                let k = j + "known-bad".len();
                let mut k2 = k;
                let mut sep2 = 0usize;
                while k2 < bytes.len() && (bytes[k2].is_ascii_whitespace() || bytes[k2] == b'_') {
                    sep2 += 1;
                    k2 += 1;
                }
                if sep2 == 0 {
                    after_sep
                } else {
                    &lower[k2..]
                }
            } else {
                after_sep
            };
            if tail.starts_with("injection") || tail.starts_with("faults") {
                n += 1;
                i = j + 1;
                continue;
            }
        } else {
            i += 1;
        }
    }
    n
}

fn live_shape_hits(readme: &str) -> Vec<&'static str> {
    let mut hits = Vec::new();
    if readme.contains("known-bad (shell selftest suites):") {
        hits.push(SHIPPED_ADVERTISEMENT_SHAPES[0]);
    }
    if readme.contains("known--bad") && readme.contains("_injections") {
        hits.push(SHIPPED_ADVERTISEMENT_SHAPES[1]);
    }
    if readme.contains("known-bad injections (shell selftest suites)") {
        hits.push(SHIPPED_ADVERTISEMENT_SHAPES[2]);
    }
    if readme.contains("known-bad faults") {
        hits.push(SHIPPED_ADVERTISEMENT_SHAPES[3]);
    }
    if readme.contains("injections (shell selftest suites") {
        hits.push(SHIPPED_ADVERTISEMENT_SHAPES[4]);
    }
    hits
}

#[test]
fn min_advertisement_sites_is_the_shipped_readme_inventory() {
    assert!(
        !SHIPPED_ADVERTISEMENT_SHAPES.is_empty(),
        "SHIPPED_ADVERTISEMENT_SHAPES is empty — a floor of nothing is not a floor"
    );
    assert_eq!(
        SHIPPED_ADVERTISEMENT_SHAPES.len(),
        5,
        "the recorded argument for the literal is five shipped sites; \
         if the inventory grew or shrank, edit MIN_ADVERTISEMENT_SITES \
         in the same commit"
    );

    let readme = read(&repo_root().join("README.md"));
    let sites = count_digit_advertisement_sites(&readme);
    assert!(
        sites > 0,
        "README.md advertises no parseable injection count — empty scan is an ERROR, not a pass"
    );
    assert!(
        sites >= SHIPPED_ADVERTISEMENT_SHAPES.len(),
        "README.md advertises the count at {sites} site(s); the floor is {}. \
         Losing a site is the partial-coverage hole. Adding a site is free.",
        SHIPPED_ADVERTISEMENT_SHAPES.len()
    );

    let hits = live_shape_hits(&readme);
    assert_eq!(
        hits.len(),
        SHIPPED_ADVERTISEMENT_SHAPES.len(),
        "shipped README is missing a contracted advertisement shape. \
         hits={hits:?} inventory={SHIPPED_ADVERTISEMENT_SHAPES:?}"
    );

    // Pin the gate constant to this inventory without depending on the
    // crate (product must not take a gate dep).
    let gate = read(&engine_root().join("crates/cdcp_gate/src/gates/verify_injection_count.rs"));
    assert!(
        gate.contains("pub const MIN_ADVERTISEMENT_SITES: usize = 5;"),
        "MIN_ADVERTISEMENT_SITES drifted from the shipped inventory of 5. \
         Derive it from SHIPPED_ADVERTISEMENT_SHAPES.len() or record the new argument."
    );
}

// ── 2. content.lock tree walk is one level deep ───────────────────────────
//
// Recursive walk would pin knowledge/corpus/*.toml (external blobs the
// lock header excludes). The honest close is: assert the depth limit so
// it cannot silently deepen or shallow. Generator and verifier must stay
// on the same glob.

const DEPTH_PIN: usize = 1;

#[test]
fn content_lock_walk_is_one_level_and_the_limit_is_pinned() {
    assert_eq!(
        DEPTH_PIN, 1,
        "the contracted walk depth is one; a different number is a mechanism change"
    );

    let gate = read(&engine_root().join("crates/cdcp_gate/src/gates/verify_content_lock.rs"));
    let gen = read(&engine_root().join("crates/cdcp_data/src/gen_lock.rs"));

    // Generator: list_one_level, not a recursive walk. Anti-vacuous: the
    // needles must be present (a file that never mentioned the walk would
    // pass a negative check).
    assert!(
        gen.contains("fn list_one_level"),
        "gen_lock.rs lost the one-level walk"
    );
    assert!(
        gen.contains("KNOWLEDGE_DIR_REL") && gen.contains("KNOWLEDGE_SUFFIX"),
        "gen_lock.rs lost the one-level knowledge glob"
    );
    assert!(
        gen.contains("WEB_MODULES_REL") && gen.contains("MODULE_SUFFIX"),
        "gen_lock.rs lost the one-level web-modules glob"
    );
    assert!(
        gen.contains("PARENT_MODULES_REL"),
        "gen_lock.rs lost the one-level parent-modules glob"
    );
    assert!(
        !gen.contains("WalkDir") && !gen.contains("rglob"),
        "gen_lock.rs grew a recursive walk — that silently deepens the walk"
    );

    // Verifier: discover is read_dir of one directory. The in-gate test
    // plants knowledge/corpus/deep.toml and asserts it is not found.
    assert!(
        gate.contains("Files matching `*<suffix>` DIRECTLY under `dir`"),
        "verify_content_lock.rs lost the one-level discover contract"
    );
    assert!(
        gate.contains("fn the_walk_does_not_recurse_and_does_not_widen_the_suffix"),
        "the in-gate depth-pin test is gone — the walk can silently deepen"
    );
    assert!(
        gate.contains("knowledge/corpus/deep.toml"),
        "the nested plant is gone — the no-recurse test is vacuous"
    );
    assert!(
        gate.contains("assert_eq!(found, vec![\"knowledge/a.toml\".to_string()])"),
        "the no-recurse assertion no longer pins the one-level result"
    );
    // Cannot silently shallow: LOCKED_ROOTS still name the three globs.
    for needle in [
        "label: \"knowledge/*.toml\"",
        "label: \"web/content/modules/*.md\"",
        "label: \"../modules/*.md\"",
    ] {
        assert!(
            gate.contains(needle),
            "LOCKED_ROOTS lost {needle} — the walk can silently shallow"
        );
    }
    // The honest stdout still names what the walk does not cover.
    assert!(
        gate.contains("does not look inside subdirectories of a locked root")
            || gate.contains("cannot see a file smuggled one level down"),
        "the gate no longer names the depth limit on the GREEN path"
    );
}
