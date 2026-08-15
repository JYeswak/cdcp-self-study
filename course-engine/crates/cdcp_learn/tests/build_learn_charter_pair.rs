//! Named assertion for the build_learn orphan-sweep CHARTER pair (bd-zhnd).
//!
//! `.flywheel/CHARTER.md` pair:
//!
//!   1. MUTATE `cdcp_learn::content::should_unlink_content_copy` so
//!      `README.md` is unlinked → this suite goes non-zero.
//!   2. Mutation still in place, delete this assertion → suite returns
//!      to zero for this concern.
//!
//! The in-process meta-test is the mutate-leg proof that does not dirty
//! the live tree: an unguarded predicate (keep = navigable names only)
//! deletes the planted doc, and that deletion is asserted. A sweep that
//! never deletes anything makes the meta-test RED. A sweep that deletes
//! README.md makes the survival test RED.
//!
//! Live compiler: rust `cdcp_learn::build` (EXTRACT-THEN-DELETE of
//! `scripts/build_learn.py`, bd-substrate-rust-migration-jhd.28). The
//! sweep predicate is also asserted in-process.

use cdcp_learn::content::{
    should_unlink_content_copy, sweep_content_copies, PROTECTED_CONTENT_DOCS,
};
use std::collections::BTreeSet;
use std::path::Path;

const PLANTED_DOC: &str = "README.md";
const PLANTED_NOTES: &str = "NOTES.md";
const STALE_COPY: &str = "99-stale.md";
const LIVE_COPY: &str = "01-mission-critical.md";

fn nav(ids: &[&str]) -> BTreeSet<String> {
    ids.iter().map(|s| (*s).to_string()).collect()
}

fn write(dir: &Path, rel: &str, body: &str) {
    let p = dir.join(rel);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&p, body).unwrap();
}

// ── CHARTER assertion ─────────────────────────────────────────────────────

#[test]
fn planted_non_module_md_survives_the_orphan_sweep() {
    let td = tempfile::tempdir().unwrap();
    let dir = td.path();
    write(dir, PLANTED_DOC, "tracked documentation\n");
    write(dir, PLANTED_NOTES, "another non-module doc\n");
    write(dir, LIVE_COPY, "# live module\n");
    write(dir, STALE_COPY, "leftover generated copy\n");

    assert!(
        dir.join(PLANTED_DOC).is_file(),
        "the plant must exist BEFORE the sweep — otherwise survival is vacuous"
    );
    assert!(
        dir.join(STALE_COPY).is_file(),
        "the stale copy must exist BEFORE the sweep — otherwise deletion is vacuous"
    );

    let report = sweep_content_copies(dir, &nav(&[LIVE_COPY])).expect("sweep");
    assert!(
        report.scanned >= 4,
        "scanned {} files — an empty scan is an ERROR: {report:?}",
        report.scanned
    );

    assert!(
        dir.join(PLANTED_DOC).is_file(),
        "FAIL: planted {PLANTED_DOC} was deleted — deleting a tracked doc is RED"
    );
    assert!(
        dir.join(PLANTED_NOTES).is_file(),
        "FAIL: planted {PLANTED_NOTES} was deleted — a non-module .md must survive"
    );
    assert!(
        dir.join(LIVE_COPY).is_file(),
        "the live module copy must survive with the planted docs"
    );
    assert!(
        !dir.join(STALE_COPY).exists(),
        "the sweep must still delete leftover generated copies; \
         a no-op sweep would keep {PLANTED_DOC} by never deleting anything"
    );
    assert!(
        report.kept.iter().any(|n| n == PLANTED_DOC),
        "receipt must name the surviving planted doc: {report:?}"
    );
}

// ── meta-test (CHARTER mutate leg, in-process) ────────────────────────────

/// The unguarded sweep: keep = navigable names only. That is the
/// pre-fix behaviour that deleted README.md. If this test goes green
/// without deleting the plant, the known-bad is gone and the survival
/// test is vacuous.
#[test]
fn unguarded_sweep_deletes_the_planted_doc_and_that_is_the_known_bad() {
    let navigable = nav(&[LIVE_COPY]);
    // Control: the unguarded predicate WOULD unlink README.md.
    let unguarded = |name: &str| !navigable.contains(name);
    assert!(
        unguarded(PLANTED_DOC),
        "control failed: unguarded keep set would not delete {PLANTED_DOC}"
    );
    // The live predicate must disagree with that control on README.md
    // and agree with it on a stale generated copy.
    assert!(
        !should_unlink_content_copy(PLANTED_DOC, &navigable),
        "live predicate unlinks {PLANTED_DOC} — mutate-leg of the pair"
    );
    assert!(
        should_unlink_content_copy(STALE_COPY, &navigable),
        "live predicate no longer unlinks stale generated copies — \
         the sweep is a no-op and the survival test is vacuous"
    );
    assert!(
        PROTECTED_CONTENT_DOCS.contains(&PLANTED_DOC),
        "PROTECTED_CONTENT_DOCS lost {PLANTED_DOC}"
    );

    // Behavioural: apply the unguarded rule and observe the deletion.
    let td = tempfile::tempdir().unwrap();
    write(td.path(), PLANTED_DOC, "tracked\n");
    write(td.path(), LIVE_COPY, "# m\n");
    for ent in std::fs::read_dir(td.path()).unwrap().flatten() {
        let name = ent.file_name();
        let name = name.to_string_lossy();
        if name.ends_with(".md") && unguarded(&name) {
            std::fs::remove_file(ent.path()).unwrap();
        }
    }
    assert!(
        !td.path().join(PLANTED_DOC).exists(),
        "the known-bad must actually delete {PLANTED_DOC} — otherwise \
         the survival assertion cannot go RED under a missing guard"
    );
    assert!(
        td.path().join(LIVE_COPY).is_file(),
        "control must not also delete the live module copy"
    );
}

// ── live rust builder ─────────────────────────────────────────────────────

#[test]
fn live_builder_leaves_a_planted_non_module_md_in_place() {
    let td = tempfile::tempdir().unwrap();
    let root = td.path();

    write(
        root,
        "knowledge/domains.toml",
        "schema_version = 1\n\n\
         [[domain]]\n\
         id = \"01-mission-critical\"\n\
         order = 1\n\
         epi_heading = \"The Mission Critical Site\"\n\
         primary_notes = \"notes/01.md\"\n",
    );
    write(
        root,
        "notes/01.md",
        "# Mission critical\n\nA planted note.\n",
    );
    write(
        root,
        "web/content/modules/README.md",
        "tracked documentation\n",
    );
    write(
        root,
        "web/content/modules/NOTES.md",
        "another non-module doc\n",
    );
    write(
        root,
        "web/content/modules/99-stale.md",
        "leftover generated copy\n",
    );
    std::fs::create_dir_all(root.join("web/data")).unwrap();
    std::fs::create_dir_all(root.join("web/learn")).unwrap();

    assert!(
        root.join("web/content/modules/README.md").is_file(),
        "plant must exist before the builder runs"
    );
    assert!(
        !root.join("scripts/build_learn.py").exists(),
        "EXTRACT-THEN-DELETE: scripts/build_learn.py must stay gone"
    );

    let outcome = cdcp_learn::build::write_learn(root).expect("build");
    assert_eq!(
        outcome.code, 0,
        "live rust builder failed:\n{}",
        outcome.stdout
    );
    assert!(
        outcome.stdout.contains("PASS: build_learn"),
        "builder printed no success token:\n{}",
        outcome.stdout
    );

    assert!(
        root.join("web/content/modules/README.md").is_file(),
        "FAIL: live builder deleted web/content/modules/README.md — \
         deleting a tracked doc is RED"
    );
    assert!(
        root.join("web/content/modules/NOTES.md").is_file(),
        "FAIL: live builder deleted a planted non-module NOTES.md"
    );
    assert!(
        !root.join("web/content/modules/99-stale.md").exists(),
        "live builder must still delete leftover generated copies; \
         a no-op sweep would keep README.md by never deleting anything"
    );
    assert!(
        root.join("web/content/modules/01-mission-critical.md")
            .is_file(),
        "live builder must copy the navigable module notes"
    );
}
