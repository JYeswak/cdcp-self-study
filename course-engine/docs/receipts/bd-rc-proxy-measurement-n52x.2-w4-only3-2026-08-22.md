# W4 Only-unrelated increment — 2026-08-22

Bead: `bd-rc-proxy-measurement-n52x.2`
Prior increment: `c4d23a8a` / `docs/receipts/bd-rc-proxy-measurement-n52x.2-w4-intersection-2026-08-22.md`

This batch rewrites the remaining **three-option `only` cartoons**: every distractor
was `Only <unrelated domain>` and the key was the unmarked sentence. Same
defect class as `m06-q252` / `m11-q107` in the first increment. Key letters
unchanged. None of these 15 sit on seed-42 Form A.

`m03-q240` remains the should-fail leave-alone from the first increment.

## Rewritten (15)

m01-q045, m01-q203, m03-q111, m03-q202, m06-q254, m09-q125, m09-q139,
m09-q244, m10-q106, m11-q115, m11-q129, m11-q217, bank-m12-q069 (file
`m12-q069.toml`), m13-q200, m14-q204.

Distractors are now same-domain or nearby-wrong, without absolute/universal
markers. No hedge words (`can`/`may`/…) were added, so the avoid-hedged
control band would not be the accidental lever.

## Plausibility

```text
cargo run --locked -q -p cdcp_assemble --example plausibility_detector
```

| surface | after increment 1 (`c4d23a8a`) | after this increment |
|---|---|---|
| bank-wide hits/applicable | 113/122 = 92.6% | 98/107 = 91.6% |
| bank-wide zero-marker | 276 | 291 |
| assembler seeds 0..99 | 439/486 = 90.3% | 382/429 = 89.0% |
| form mean lone-plausible | 4.39 | 3.82 |
| chance floor | 25.0% | 25.0% |

Hits and applicable both fell by 15 (the batch left the exact-three
population). Rate among remaining cue-shaped rows: 91.6%. 98 hits remain.
Bead stays open.

`cdcp_gate plausibility-detector` expected exit 2 with
`key_is_lone_plausible=98` `rate=91.6%`. E2E pin updated to that
measurement.

## Coupled properties (guessing_strategies, seeds 0..99)

| metric | increment 1 | this increment | band |
|---|---|---|---|
| longest mean | 10.31 | 10.37 | table |
| always-A/B/C/D mean | 10.20 / 9.91 / 9.61 / 10.28 | unchanged | |
| hedged mean | 10.92 | 10.92 | 9.0..=11.0 |
| stem-overlap mean | 11.14 | 11.12 | 9.0..=11.5 |
| HEDGED_ANY | 806, 279, 34.6% | 806, 279, 34.6% | diagnostic |
| HEDGED_EXACTLY_ONE | 293, 61, 20.8% | 293, 61, 20.8% | 20–30% |
| AVOID_EXACTLY_ONE | 293, 86, 29.4% | 293, 86, 29.4% | 20–30% |
| STEM_OVERLAP_APPLICABLE | 1867, 512, 27.4% | 1870, 512, 27.4% | 20–30% |
| pass_count | 0 | 0 | must stay 0 |

answer-key letters unchanged.

## Pins re-frozen

Bank hash `c2e05878…` → `a328bcd3a4ef7f6d915b9ce66f727eae1528b62c390d771db1b528c31d19251f`.

- `UPDATE_GOLDENS=1 cdcp goldens generate`
- `cdcp export-web --seed 42`
- `cdcp content-lock`

Seed-42 `item_ids` unchanged.

## Green does not prove

The `Only <unrelated>` shape is gone from this 15. 98 authored
lone-unmarked-key rows remain, still ~91.6% of the exact-three population.
F-01 is not closed.
