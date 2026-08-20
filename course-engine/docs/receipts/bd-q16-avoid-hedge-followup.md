# Q16 / bd-d97q follow-up: inverse hedge cue

Date: 2026-08-20

## Denominator reconciliation

The dispatch reported `21/71 = 29.6%`, but that denominator is not reproducible
from this tree with the declared hedge vocabulary (`can`, `could`, `generally`,
`may`, `might`, `often`, `usually`, `typically`, `tends`, `sometimes`). Before
the repair, the approved bank contained 63 exact-one items, with 15 key hits
and 19 avoid hits. The real assembler, over the fixed seeds 0..99, measured
283 assembled occurrences: 61 key hits (21.6%) and 83 avoid hits (29.3%).
The assembler is the product measurement; repeated item occurrences are
intentional because the learner sits assembled mocks, not the directory.

## Repair

Four avoid-hit items received one additional hedge on a distractor. Each added
claim remains unambiguously false; no key, stem, status, citation, or
explanation was changed:

| item | changed distractor | why it remains false |
|---|---|---|
| m02-q076 | `International standards may replace local law without an adoption decision` | A standard cannot replace adopted law without the jurisdiction/project adoption decision. |
| m02-q210 | `Protection Class may be the converter between Rated and Tier, because protection and topology are one rating lattice` | Protection Class does not translate the distinct TIA Rated, Uptime Tier, and Availability Class frameworks. |
| bank-m12-q074 | `Can improve ASD sampling automatically when the door is propped during operations near the protected zone` | Propping a rated door defeats compartmentation; it does not automatically improve ASD sampling. |
| m12-q200 | `Low-temperature LED lighting may cause most technical-space fire incidents when electrical protection is properly maintained` | Low-temperature LED lighting is not the dominant fire-cause pattern asserted by the item. |

Existing explanations still address the revised distractors: 4 fit, 0 stale
clauses, 0 citation or scope caveats lost. The raw approved-bank population is
now 59 exact-one items, 15 key hits (25.4%), and 15 avoid hits (25.4%).

## Product measurement

The same predeclared assembled seeds 0..99 were rerun through
`cdcp_assemble::assemble`:

| metric | before | after |
|---|---:|---:|
| exact-one assembled occurrences | 283 | 271 |
| lone hedge is key | 61/283 = 21.6% | 61/271 = 22.5% |
| avoid lone hedge | 83/283 = 29.3% | 78/271 = 28.8% |
| hedged strategy mean | 10.98/40 | 10.98/40 |
| avoid-hedged strategy mean | 10.22/40 | 10.22/40 |
| pass-bar seeds, all strategies | 0/100 | 0/100 |

The product rates remain inside the harness's declared 20–30% band. This
establishes that the two named hedge routes are near chance on the assembled
population; it does not establish item truth or learner discrimination.

## Coupled properties

- `verify-bank`: rc=0; 957 scanned, 931 approved; keys A/B/C/D = 235/243/229/224.
- `answer-key-skew`: PASS; 25.2/26.1/24.6/24.1%, unchanged.
- `construction-faults`: rc=0; live verdict PASS; length ranks 27.3/24.0/22.9/25.9% PASS.
- `near-duplicate-items`: rc=0; 0 pairs.
- `verify-orphans`: rc=0; 0 orphan topics, refs, or unanchored items.
- `validate-grounding`: rc=0; 0 high-severity findings.
- `check-osha`: rc=0; 1002 scanned, 0 faults.
- Explanations: all four revised distractors remain covered by their existing clauses; no option letters or citation exclusions were introduced.
- Independent WASM key cross-check: BLOCKED by stale shipped artifacts. The ignored test was forced with `--include-ignored` and failed at the golden pin (`left 26003203…`, `right 89ec9854…`). This bank-only change must be repacked before that artifact-level check can pass; `web/data/`, WASM, and goldens were not touched here.

The repair addresses the measured product cue without claiming that a clean
construction statistic proves teaching quality or discrimination.
