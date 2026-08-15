//! The bank-hash domain tag lives in three places. They move together or not at all.
//!
//! bd-6ycw: C2 changed what `bank_hash` covers while the tag stayed `cdcp-bank-v1`,
//! so a hash computed under the OLD definition and one under the NEW were both
//! labelled v1. The values differ, so nothing silently matched — the cost was that a
//! mismatch read as DRIFT rather than as a DEFINITION CHANGE, and a reader could not
//! tell those apart.
//!
//! THE THREE SITES:
//!   1. `cdcp_core::BANK_HASH_DOMAIN`            — the constant that actually salts the hash
//!   2. `content.lock` `canonical`               — the ecosystem pin's label for it
//!   3. `crates/cdcp_data/src/gen_lock.rs` CANONICAL — the writer of site 2
//!
//! Site 1 is AUTHORITATIVE. Sites 2 and 3 are labels that claim to name it.
//!
//! WHY THIS TEST KEYS ON THE CONSTANT AND NEVER ON A GREP FOR THE LITERAL:
//! a test that greps all three files for `"cdcp-bank-v2"` passes when all three are
//! wrong together, which is precisely the partial-bump state it is meant to catch —
//! it would certify a tree where someone search-and-replaced the string everywhere
//! EXCEPT the constant, leaving the label lying about the hash. So the expected value
//! is DERIVED from `bank_hash_domain_label()` at runtime, and the two text sites are
//! compared against that. There is no literal in this file to search-and-replace.

use cdcp_core::bank_hash_domain_label;
use std::path::{Path, PathBuf};

/// `crates/cdcp_core` -> repo root (`course-engine/`).
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/cdcp_core has two ancestors")
        .to_path_buf()
}

fn read(rel: &str) -> String {
    let p = root().join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

/// Pull `key = "value"` out of a line-oriented file (TOML, Python, or a
/// Rust `pub const KEY: &str = "value"`).
///
/// Anti-vacuous: zero matches is an ERROR, not a pass. A renamed or deleted key
/// must not read like agreement. Two matches is also an ERROR — an ambiguous
/// site cannot be said to agree with anything.
fn scalar_assignment(text: &str, key: &str, whence: &str) -> String {
    let hits: Vec<String> = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.starts_with('#') && !l.starts_with("//"))
        .filter_map(|l| {
            let rest = if let Some(r) = l.strip_prefix(key) {
                r.trim_start()
            } else if let Some(r) = l
                .strip_prefix("pub const ")
                .or_else(|| l.strip_prefix("const "))
            {
                r.strip_prefix(key)?.trim_start()
            } else {
                return None;
            };
            let rest = rest
                .strip_prefix(':')
                .map(|s| {
                    s.trim_start()
                        .strip_prefix("&str")
                        .unwrap_or(s)
                        .trim_start()
                })
                .unwrap_or(rest);
            let rest = rest.strip_prefix('=')?.trim();
            let inner = rest.strip_prefix('"')?;
            let end = inner.find('"')?;
            Some(inner[..end].to_string())
        })
        .collect();

    match hits.len() {
        1 => hits.into_iter().next().expect("len checked"),
        0 => panic!(
            "{whence}: no `{key} = \"...\"` assignment found. \
             An absent site is an ERROR, not agreement — the domain tag is \
             unpinned there and nothing is checking it."
        ),
        n => panic!(
            "{whence}: found {n} `{key} = \"...\"` assignments {hits:?}. \
             An ambiguous site cannot agree with the constant; leave exactly one."
        ),
    }
}

/// Anti-vacuous floor: the constant itself must parse to a non-empty label.
/// An empty or unparseable domain is an ERROR — see `bank_hash_domain_label`.
#[test]
fn domain_constant_is_parseable_and_non_empty() {
    let label = bank_hash_domain_label().expect("BANK_HASH_DOMAIN must parse");
    assert!(
        !label.trim().is_empty(),
        "BANK_HASH_DOMAIN parsed to an empty/whitespace label — every hash would \
         then be comparable to every other hash, which is what the tag exists to prevent"
    );
    assert!(
        label.starts_with("cdcp-bank-v"),
        "BANK_HASH_DOMAIN label {label:?} does not name the cdcp bank hash definition"
    );
}

/// Site 2 must name site 1.
#[test]
fn content_lock_canonical_matches_the_constant() {
    let expected = bank_hash_domain_label().expect("BANK_HASH_DOMAIN must parse");
    let found = scalar_assignment(&read("content.lock"), "canonical", "content.lock");
    assert_eq!(
        found, expected,
        "\nBANK HASH DOMAIN DISAGREEMENT — a partial bump is worse than none.\n  \
         cdcp_core::BANK_HASH_DOMAIN (AUTHORITATIVE) = {expected:?}\n  \
         content.lock `canonical`                    = {found:?}\n\
         content.lock is labelling hashes with a definition the code does not compute.\n\
         Move all three sites in ONE commit, then re-run the block in goldens/PROVENANCE.md."
    );
}

/// Site 3 writes site 2, so it must name site 1 too — otherwise the next
/// `UPDATE_CONTENT_LOCK=1` run silently re-introduces the disagreement.
#[test]
fn gen_content_lock_canonical_matches_the_constant() {
    let expected = bank_hash_domain_label().expect("BANK_HASH_DOMAIN must parse");
    let found = scalar_assignment(
        &read("crates/cdcp_data/src/gen_lock.rs"),
        "CANONICAL",
        "crates/cdcp_data/src/gen_lock.rs",
    );
    assert_eq!(
        found, expected,
        "\nBANK HASH DOMAIN DISAGREEMENT — a partial bump is worse than none.\n  \
         cdcp_core::BANK_HASH_DOMAIN (AUTHORITATIVE) = {expected:?}\n  \
         crates/cdcp_data/src/gen_lock.rs CANONICAL  = {found:?}\n\
         This module WRITES content.lock, so leaving it stale re-introduces the\n\
         disagreement on the next `cdcp content-lock` run even if content.lock\n\
         is hand-repaired. Move all three sites in ONE commit."
    );
}

/// The two label sites must also agree with EACH OTHER. Sites 2 and 3 are
/// compared to site 1 above; this pins the remaining edge of the triangle so a
/// future fourth reader cannot find two different labels both "passing".
#[test]
fn the_two_label_sites_agree_with_each_other() {
    let lock = scalar_assignment(&read("content.lock"), "canonical", "content.lock");
    let script = scalar_assignment(
        &read("crates/cdcp_data/src/gen_lock.rs"),
        "CANONICAL",
        "crates/cdcp_data/src/gen_lock.rs",
    );
    assert_eq!(
        lock, script,
        "content.lock `canonical` = {lock:?} but crates/cdcp_data/src/gen_lock.rs \
         CANONICAL = {script:?} — the writer and the written disagree."
    );
}
