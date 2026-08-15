# `.parked-wave8/` — EXTRACT-THEN-DELETE quarantine

**Date parked:** 2026-08-14  
**Bead:** `bd-parked-wave8-unannounced-saz0` (this directory)  
**Related:** `bd-wave8-ports-agreement-only-debt-idns`, `bd-wave8-agent-deaths-redispatch-qb01`, `bd-engine-not-gate-ar39.3`  
**Owner:** course-engine integrator (not an individual port author)

## Why these files are here

`crates/cdcp_gate/src/gates/` is globbed by `build.rs`. One incomplete `smoke_*.rs` takes the **whole crate** down and every sibling with it. On 2026-08-14 the integrator moved unverified wave-8 gate ports out of that glob so the crate could stay green.

That quarantine is still the right *shape*. What was missing was a note, a bead, and a return criterion (saz0). This file is that note.

## Disposition (binding — ar39.3, 2026-08-15)

**STAY PARKED.** After the gate-shrink ratchet (`bd-engine-not-gate-ar39.1`) exists, files here are **EXTRACT-THEN-DELETE into product crates** (`cdcp_learn`, `cdcp_anki`, …). They are **not** a byte-exact `cdcp_gate` transcription and must not be revived in-place into `src/gates/`.

A port that only grows `cdcp_gate` fails the ratchet even if it is correct, tested, and compiling.

## What is in here (2026-08-15)

| File | Origin | Notes |
|---|---|---|
| `smoke_diagrams.rs` + `diff_smoke_diagrams.rs` | jhd.14 (qb01 dead agent) | Never compiled or tested in this tree. Brace-balanced only. |
| `smoke_learn_chrome.rs` + `diff_smoke_learn_chrome.rs` | jhd.15 (qb01 dead agent) | Same. |
| `smoke_learn.rs` + `diff_smoke_learn.rs` | wave-8 sibling | Parked with the wave, not because it failed a compile. |
| `smoke_feedback_links.rs` + `diff_smoke_feedback_links.rs` | wave-8 sibling | Same. |

`smoke_a11y` / `smoke_learn_v2` were restored to the crate by their owner after this park (saz0 comment). They are **not** in this directory. Their agreement-only debt is `bd-wave8-ports-agreement-only-debt-idns` and is paid by planting independent verdicts, **not** by raising the census budget.

## Criterion for leaving this directory

A file leaves `.parked-wave8/` only when **all** of these are true:

1. The logic is extracted into a **product crate** (not `cdcp_gate/src/gates/`).
2. `cdcp_gate` line count does not rise (`gate_shrink` ceiling 49422). The parked file is deleted, not copied back.
3. Any differential case asserts an **independent** verdict (exit + named finding), not agreement-only. `idns` budget must not be edited upward.
4. The owning bead is claimed, reserved, and closed with a VERDICT naming the new paths.

Until then this directory is quarantine, not abandonment.

## What this is not

- Not a compile-failure pile. Some of these compiled when parked.
- Not permission to `cp` back into `src/gates/` to “finish the port.”
- Not a syspolicyd incident and not a reason to swarm `syspolicyd-*` beads.

Joshua ACK 2026-08-15: keep moving; this README is the saz0 note that was missing.
