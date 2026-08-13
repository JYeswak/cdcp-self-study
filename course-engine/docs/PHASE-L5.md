# PHASE L5 — Browser product surface + UI e2e digest match

**North star:** Offline static HTML/CSS/JS (+ WASM grade) so Joshua can open the course, take a mock exam, and see a score whose digest matches the Rust oracle — without claiming EPI certification. [[claim:claim-not-epi-certified]] [[claim:claim-grade-byte-exact]]

**Constitution:** `CHARTER.md` §1 shipped means (1)+(3) · `ORACLE-GAUNTLET.md` L5 · `VISUAL.md` · honesty lattice.

**Loop value bar:** GREEN only if product UI path changed **and** e2e digest gate is wired in `check.sh` **and** known-bad “certified” UI string trips RED.

## Preconditions (done)

- W0 knowledge · L1 registries · L2 bank+assemble · L3 GradeExact · L4 native==wasm (`cdcp_wasm`) [[claim:claim-grade-byte-exact]]

## Out of scope this phase

- Multi-tenant SaaS · Next/React · LLM grade-of-record · official EPI materials · full Anki export

## Stories (bead DAG)

North-star create (WHAT/WHY/ACCEPTANCE + labels): epic `bd-y7v`. Map: `beads_compliance_audit/wave-grades/L5-BEADS.json`.

| Story | Bead | Title | Deps (blocks) | Labels |
|-------|------|-------|---------------|--------|
| Epic | `bd-y7v` | browser course + UI e2e digest match (M3–M5) | S8 | epic,l5,oracle,ui |
| S1 | `bd-2d9` | Design tokens + honesty shell | — (ready) | css,honesty,l5,ui,visual |
| S2 | `bd-174` | Browser exam pack export (JSON + wasm glue) | — (ready) | bank,cli,export,l5,wasm |
| S3 | `bd-38o` | Hub + mock exam take UI | S1, S2 | html,js,l5,mock,ui |
| S4 | `bd-1an` | Results + browser grade via WASM dual-path | S3 | grade,l5,results,ui,wasm |
| S5 | `bd-38i` | UI e2e digest match + known-bad UI honesty | S4 | e2e,gate,l5,oracle,testing |
| S6 | `bd-3ro` | Learn: render 14 modules | S1 | content,l5,learn,ui |
| S7 | `bd-ca8` | Module quiz + minimal drill/SRS | S6, S2 | drill,l5,quiz,srs,ui |
| S8 | `bd-39j` | check.sh L5 wire + scorecards/L5.json | S5, S7 | check,gate,l5,scorecard |

**Close rule (HANDLE):** every close carries a `VERDICT:` comment with measured proof (digest match / check.sh stage / selftest RED evidence). Status is not fact.

## Explicit non-claims

- L5 complete ≠ EPI certified [[claim:claim-not-epi-certified]]  
- Study signal 27 ≠ credential [[claim:claim-study-signal-27]]  
- Browser grade must never disagree with `cdcp goldens check` for frozen fixtures [[claim:claim-grade-byte-exact]]  


## Gate commands (target)

```bash
./scripts/check.sh                    # includes L5 stage when wired
# L5 known-bad: planted "certified" UI string → RED
# L5 e2e: seed42 all-correct digest == goldens/mock40_seed42_all_correct.sha256
```
