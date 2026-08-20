# Loop #3 — External validation protocol (hybrid)

**Status:** VERIFIED (Josh 2026-08-12) · bead `bd-1tx`  
**Product:** CDCP course-engine study tool  
**Hard rule:** No log, README, or scorecard may claim EPI/EXIN certification or official exam pass.

Loop #3 is the signal **outside** `./scripts/check.sh` that the product changed interview-ready fluency [[claim:claim-interview-ready]]. Internal green gates are Loop #1 only.

For the pending real-person mock, use [`EXTERNAL-HUMAN-MOCK.md`](EXTERNAL-HUMAN-MOCK.md).
It is deliberately separate from the T1 self-log: an agent or maintainer may
produce a T1 diagnostic, but neither may be reported as Q14's external-human
signal.

## Hybrid design

| Tier | What | When | Loop #3 weight |
|------|------|------|----------------|
| **T1 floor** | Structured **self-log** after a real mock/drill session in this product | Soon (first run required) | Satisfies v1 Loop #3 |
| **T2 strong** | Peer whiteboard **or** real/mock interview using bank scenarios | When calendar allows | Upgrades signal |
| **T3 optional** | External learner report after public (L88) | After publicize | Nice-to-have |

T1 alone is allowed for v1 so the graph is not blocked on hiring a peer. T2 is the honesty upgrade, not a silent requirement to close forever.

---

## Tier 1 — Self-log (floor)

### Preconditions

1. Product opens offline (hub → mock or quiz/drill).  
2. Complete **one** graded attempt (mock seed any, or Learn-15 / Drill-10).  
3. Note weak modules from results (or manual list if UI unavailable).

### Log template

Create `docs/loop3/runs/YYYY-MM-DD.md` (copy below):

```markdown
# Loop3 run — YYYY-MM-DD

- **tier:** T1
- **product:** course-engine
- **seed / mode:** (e.g. mock seed 42 / learn15 m06)
- **bank_hash:** (from results or `cdcp bank-hash`)
- **score:** N/40 (or quiz n/m)
- **study_signal:** pass | fail  (threshold 27 for mock — study only, NOT cert)
- **weak_modules:** (list)
- **whiteboard_3:** 
  1. …
  2. …
  3. …
- **still_open_gap:** …
- **product_helped:** yes | partial | no — one sentence
- **epi_claim:** none
```

### Pass criteria for T1

- File exists under `docs/loop3/runs/` with date + non-empty whiteboard_3 and gap.  
- `epi_claim: none` explicit.  
- Attempt used **this** product (not a random PDF dump).

---

## Tier 2 — Peer or interview (strong)

Same log file fields, plus:

- **tier:** T2  
- **external_human:** name or role (no need for full PII)  
- **format:** peer whiteboard 30–45m | mock interview | real interview  
- **topics_hit:** power / cooling / fire / network / …  
- **outcome:** fluent | mixed | stuck — their words preferred  

Scenarios: pull from bank runbook tags, power-path diagram, or weak modules from last T1.

---

## Tier 3 — Post-public (optional)

After L88 publicize: one issue/comment/PR from someone who ran hub → mock without your oral help. Log as T3. Stars alone do **not** count.

---

## What never counts as Loop #3

- `./scripts/check.sh` green  
- Closing beads / scorecards  
- “I feel ready” without a dated log  
- Any certificate language  

---

## Graph

| Bead | Role |
|------|------|
| `bd-1tx` | Protocol decision (this doc) |
| First T1 run | Child / follow-up bead — first dated file in `docs/loop3/runs/` |
| `bd-1z2` HUMAN epic | Closes when protocol + first T1 run (or WAIVE by Josh) done |

## Related

- Charter Loop #3 / RULE ZERO  
- OQ-09 free corpus · OQ-10 defer spend · H-PUB L88 publicize  
