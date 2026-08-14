# STATE — cdcp-self-study loop

**Updated:** 2026-08-14 · ticks T1 (ec5e3f6) GREEN · T2 (27a0c2d) GREEN

## Where the loop is

`scripts/check.sh` exit **0** on the full chain. Milestone A done; C1 (item status +
approved-only assembly) and C5 (module 15 taught) landed; S0 substrate floor wired and
observed denying.

## In flight

Four port agents on `bd-substrate-rust-migration-jhd`: verify_bank (.7),
verify_content_lock (.8), verify_knowledge_paths (.5), verify_doc_consistency (.4).
Landed: verify_orphans (.2), verify_injection_count (.3) — awaiting check.sh wiring.

Codex adversarial review of 27a0c2d running (read-only sandbox).

## Verified this session, controller-side, not agent-reported

- substrate-guard denies: unlisted `.py`/`.sh` -> 2 naming the file; new `.rs` anywhere
  incl. `src/gates/` -> 0; editing an allowlisted file -> 0; zero files scanned -> 4.
  A real `git commit` was refused by the installed shim.
- `build.rs` glob registration holds under SIX concurrent agents: 8 gates auto-registered,
  zero edits to any shared file, `cdcp_gate list` exit 0. This was S0's central claim and
  it is what makes the remaining ports parallelisable.
- `export-web --seed 42` reports `golden_pinned=false` — the fixture no longer
  short-circuits the Rust sampler.
- `cdcp_registry_check` exit 0 after the DIAGRAM-REGISTRY reword.
- `clippy --locked --workspace --all-targets -D warnings` exit 0.

## Open decisions recorded, not re-litigable

- **Exit codes (bd-2m9).** Ported gates reproduce the oracle's exit code (1), NOT
  cdcp_gate's 0/2/3/4, for as long as an oracle exists to be exact against. check.sh reads
  only zero vs non-zero, verified. One commit flips the whole set after the oracles die.
  Mixed state is never committed.
- **Corpus rights class is NINE, not ten (bd-corpus-...-kej).** `src-curriculum-map-local`
  is our own file, swept in on an inherited default. Corrected.
- **Module 15: TEACH, not exclude (C5).** Recorded in CHARTER §11 row 8.
- **Meta-test definition corrected in the CHARTER in place.** "Delete the assertion ->
  non-zero" is incoherent for a differential test and two agents refused it rather than
  fake a result. The correct form is the mutate-then-delete PAIR.

## The defect class this session kept finding

A list that should have been DERIVED was hardcoded, so the gate reported green while
certifying nothing about anything new:

- `smoke_diagrams.py` PRESENT hardcoded to 3 P0 diagrams -> FIXED, derives from the
  registry, 3 -> 7, proven to trip.
- Migration epic's "17" from a remembered count -> `.19`/`.20` filed.
- `mock40_seed42.json` preferred over live sampling -> FIXED.
- Three gates encoded "module 15 is untaught" as a RULE, so the correct fix failed three
  gates FOR BEING CORRECT -> bd-lt7. This is the most dangerous instance: the gate layer
  became an immune system attacking the repair.

## Next

1. Wire the landed ports into check.sh in one batch; commit.
2. Wave 3: `.6` verify_coverage, `.9` validate_grounding, `.10` verify_objectives.
   NOTE `.6` carries the bd-lt7 hardcoded bound — port it byte-exact WITH the defect.
3. Relay codex findings; file beads for anything it confirms.
