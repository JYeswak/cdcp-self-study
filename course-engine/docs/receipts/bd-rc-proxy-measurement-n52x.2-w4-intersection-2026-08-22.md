# W4 intersection increment — 2026-08-22

Bead: `bd-rc-proxy-measurement-n52x.2`
Preflight: `docs/receipts/bd-rc-proxy-measurement-n52x.2-preflight-2026-08-21.md`

This is a **content** increment, not a detector change. 13 of the 14-item
W1a∩W1b intersection had cartoon absolute/universal distractors rewritten.
Key letters are unchanged. `m03-q240` was deliberately left alone.

## Should-fail row (left alone)

`m03-q240` (module 3, key D). The wrong beliefs the item tests *are* the
absolute claims: queue position is irrelevant "regardless of" upgrade scope;
behind-the-meter generation "bypasses every queue … universally and
guarantees energization"; a PPA "guarantees" interconnection. Those sentences
are how operators actually over-claim. Softening `regardless` / `every` /
`universally` / `guarantees` would hide the misconception the LBNL *Queued Up*
corpus documents. The detector hit is a lexical cost of stating those beliefs
honestly. Corpus cannot support a better distractor without lying about the
failure mode.

After this increment it remains the lone intersection row still on the
`absolute-universal-lone-plausible` branch.

## Rewritten (13)

| id | old cue | what changed |
|---|---|---|
| m02-q206 | never / always / any | same-domain Tier misuse, no markers |
| m03-q205 | guaranteed / automatic / zero | transport-adjacency misconceptions, no markers |
| m06-q082 | every+no / permanently / only+always | isolated chassis, disabled SPD, water-pipe electrode |
| m06-q086 | zero+no / regardless / only | VRLA inspection / lithium / freezing, no markers |
| m06-q252 | Only ×3 off-topic | double-conversion scope errors (days-long plant, CRAH filter, pair SPD) |
| m09-q145 | always / all / always | leftover discharge, choked tiles, dropped containment |
| m10-q210 | no+any / any / all+permanently | shoulder wait, ditch source (keeps `may`), silenced alarms |
| m10-q211 | entirely+only / every / only+no | WUE as meter-replacement / dry plant / brochure term |
| m11-q107 | Only ×3 off-topic | MDF / computer-room MMR / generator switchgear |
| m11-q122 | Only ×3 off-topic | single lateral / cooling-count failover / shared entrance |
| m11-q206 | Only ×3 off-topic | RU count / tray vs underfloor / PoE-as-frequency |
| m11-q235 | any+no / no / any+only | UFC over-application kept; markers dropped; `can` restored on A |
| m14-q209 | always / never / entirely | default blame / split clocks / overlay replaces mechanics |

## Plausibility (same commands as W1a)

```text
cargo run --locked -q -p cdcp_assemble --example plausibility_detector
```

| surface | before (2026-08-21) | after |
|---|---|---|
| bank-wide hits/applicable | 126/135 = 93.3% | 113/122 = 92.6% |
| bank-wide zero-marker | 263 | 276 |
| assembler seeds 0..99 hits/applicable | 498/545 = 91.4% | 439/486 = 90.3% |
| form mean key-is-lone-plausible | 4.98 | 4.39 |
| chance floor | 25.0% | 25.0% |

Honest reading of the rate: the 13 rows left the exact-three population
(marker_count 3 → 0). Hits and applicable both fell by 13, so the *rate among
remaining cue-shaped items* barely moved. That is not the length-rank failure
mode (proxy to chance, defect intact). The cartoon "Only <unrelated>" shape is
gone from these 13. 113 authored lone-unmarked-key rows remain. This bead stays
open.

`cdcp_gate plausibility-detector` is expected to exit 2 with
`key_is_lone_plausible=113` `rate=92.6%`. The e2e pin in
`crates/cdcp_gate/tests/plausibility_detector_e2e.rs` was updated to that
measurement. Updating the pin records the new live defect; it does not
regenerate a golden to hide a detector bug.

## Coupled properties (guessing_strategies example, seeds 0..99)

HEAD bank vs this increment. Control bands unchanged (not widened).

| metric | HEAD | after | band |
|---|---|---|---|
| longest mean | 10.35 | 10.31 | (table only) |
| always-A/B/C/D mean | 10.20 / 9.91 / 9.61 / 10.28 | unchanged | |
| hedged mean | 10.98 | 10.92 | 9.0..=11.0 |
| avoid-hedged mean | 10.22 | 10.23 | |
| stem-overlap mean | 11.07 | 11.14 | 9.0..=11.5 |
| HEDGED_ANY | 793, 279, 35.2% | 806, 279, 34.6% | diagnostic |
| HEDGED_EXACTLY_ONE | 271, 61, 22.5% | 293, 61, 20.8% | 20–30% |
| AVOID_EXACTLY_ONE | 271, 78, 28.8% | 293, 86, 29.4% | 20–30% |
| STEM_OVERLAP_APPLICABLE | 1845, 503, 27.3% | 1867, 512, 27.4% | 20–30% |
| pass_count (all strategies) | 0 | 0 | must stay 0 |

A first rewrite draft tripped AVOID_EXACTLY_ONE above 30% by dropping `may` /
`can` from distractors. Restored: m10-q210 B keeps `may`; m11-q235 A keeps `can`.
The example then exits 0. The band was not moved.

answer-key letters unchanged → `answer-key-skew` population is the same 931
approved rows with the same A/B/C/D keys.

## Teaching lexical floor (teaching_mismatch example)

Before (census receipt): 826 TAUGHT / 101 SHALLOW / 4 ABSENT = 105/931 = 11.3%.
After: 831 TAUGHT / 96 SHALLOW / 4 ABSENT = 100/931 = 10.7%.

Intersection rows still SHALLOW (lexical floor, not a human ABSENT finding):
m02-q206, m03-q240 (left alone), m06-q082, m06-q086, m09-q145, m10-q210,
m10-q211, m11-q235, m14-q209.

Left the FINDING list (now TAUGHT under the declared lexical floor):
m03-q205, m06-q252, m11-q107, m11-q122, m11-q206.

This is the lexical scanner, not a claim that the lesson teaches the decision.

## Pins re-frozen (official writers)

Bank hash `4320c3fb…` → `c2e058784c7ec5c314beac421de2317a31b5772e3a31748e0735e77a904e082c`.

- `cdcp goldens generate` (`UPDATE_GOLDENS=1`) — bank_hash.txt + all-correct + all-wrong
- `cdcp export-web --seed 42` — `web/data/{bank_items,keys,mock40}_seed42.json`
- `cdcp content-lock` — `content.lock` bank_hash

Seed-42 `item_ids` are unchanged (none of the 14 sit on Form A). Digest movement
is the bank_hash in the golden preimage, not a sampler redraw.

## Green does not prove

13 rewritten rows do not make the bank teach. 92.6% of remaining exact-three
items still key the unmarked option. No green gate from this increment may be
read as F-01 closed.
