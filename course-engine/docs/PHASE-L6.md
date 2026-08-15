# PHASE L6 — Pedagogy mastery + coverage oracle (charter §3–4)

**North star:** Joshua can follow a mastery path — module practiced bars, weak-module → learn links, short-interval due drills, and multi-seed mocks — without any EPI/cert claim. [[claim:claim-not-epi-certified]] [[claim:claim-study-signal-27]] [[claim:claim-domain-covered]] [[claim:claim-interview-ready]]

**Constitution:** parent `CHARTER.md` §3–4 session shapes · `ORACLE-GAUNTLET` pillar (c) · OQ-05 ASSUMED min_items≥1 (ratchet later).

**Why now:** Oracle L0–L5 product surface + GradeExact are GREEN. [[claim:claim-grade-byte-exact]] Buyer still needs **charter pedagogy** (practiced 80%, mastery 90%×2 spaced, next-module recommend, coverage gate) — that is the next RULE ZERO product tick.

**Loop value bar:** GREEN only if (1) mastery state changes learner navigation, (2) coverage gate is wired in `check.sh`, (3) multi-seed mock grades still match oracle for seed42 goldens. [[claim:claim-grade-byte-exact]]

## Preconditions (done)

W0–L5 green · `scorecards/L5.json` · hub/learn/quiz/drill/mock/results · WASM digests

## Out of scope

- Anki `.apkg` export (v1.1 stretch)
- Official domain weights (OQ-06 FORBIDDEN)
- Paid SDO full text (OQ-10)
- L6 formal Lean (charter Artifact L6 = NO)

## Stories (bead DAG)

| Story | Title | Deps |
|-------|-------|------|
| S1 | Domain coverage oracle gate (≥N items / domain 1–14) | — |
| S2 | Mastery state: practiced 80% · mastery 90%×2≥24h | — |
| S3 | Results weak_modules → Learn deep links | — |
| S4 | Hub mastery dashboard + next-module recommend | S2, S3 |
| S5 | Multi-seed assemble export + mock seed UI (`export-web --seed N` always samples; no implicit fixture) [[fact:fact-export-web-implicit-fixture-at-seed-42=no]] | — |
| S6 | Session shapes: Drill-10 due-only + Miss-review entry | S2 |
| S7 | check.sh L6 + scorecards/L6.json + README truth | S1, S4, S5, S6 |

## Explicit non-claims

- Mastery / practiced ≠ EPI certified [[claim:claim-not-epi-certified]]  
- Domain coverage ≠ exam pass probability [[claim:claim-domain-covered]]  
- Study signal 27 remains mock-only [[claim:claim-study-signal-27]]  

## Gate commands (target)

```bash
./scripts/check.sh   # includes L6 coverage + mastery smoke
# seed42 digests still match goldens after multi-seed work
```
