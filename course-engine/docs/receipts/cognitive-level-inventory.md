# Cognitive-level inventory (Q17)

Measured at commit `c6d515db58fc75ab91a3847a83d5d07f7c83b390` on 2026-08-20.
The source set was `bank/items/*.toml`: 957 files, of which 931 were
`status = "approved"` and 26 were `status = "retired"`. `git status --
bank/items` was clean; this inventory changed no bank item.

## First measurement

The requested three buckets were derived from the authored `bloom` field, not
from a new semantic reading of each stem:

| authored `bloom` | inventory bucket | reason |
|---|---|---|
| `remember`, `understand` | recall | lower-order knowledge/interpretation tags; neither asserts application or trade-off analysis |
| `apply` | apply | direct application tag |
| `analyze`, `evaluate` | analyse | the available higher-order tags; no item currently uses `create` |

This makes the result reproducible and exposes the existing metadata before any
rewriting. It is a mechanical proxy, not an independent judgement of the
question. The raw-tag counts are: all items `remember=150`, `understand=384`,
`apply=318`, `analyze=86`, `evaluate=19`; approved items `remember=139`,
`understand=373`, `apply=314`, `analyze=86`, `evaluate=19`.

Across all 957 files the result is:

| bucket | count | share |
|---|---:|---:|
| recall | 534 | 55.8% |
| apply | 318 | 33.2% |
| analyse | 105 | 11.0% |
| total | 957 | 100.0% |

The approved product pool alone is recall `512/931` (55.0%), apply `314/931`
(33.7%), and analyse `105/931` (11.3%). Retired items account for the
remaining 26: recall 22 and apply 4.

## Distribution by module

The primary denominator below is all 957 files, as requested. The approved
total is included so the product-pool denominator cannot be mistaken for the
full tree.

| module | recall | apply | analyse | all total | approved total | retired |
|---|---:|---:|---:|---:|---:|---:|
| m01 | 30 | 10 | 3 | 43 | 43 | 0 |
| m02 | 35 | 15 | 2 | 52 | 52 | 0 |
| m03 | 31 | 17 | 7 | 55 | 55 | 0 |
| m04 | 23 | 13 | 3 | 39 | 39 | 0 |
| m05 | 20 | 11 | 2 | 33 | 32 | 1 |
| m06 | 82 | 54 | 10 | 146 | 140 | 6 |
| m07 | 21 | 8 | 6 | 35 | 34 | 1 |
| m08 | 25 | 11 | 3 | 39 | 37 | 2 |
| m09 | 75 | 46 | 6 | 127 | 122 | 5 |
| m10 | 20 | 11 | 6 | 37 | 35 | 2 |
| m11 | 42 | 31 | 8 | 81 | 78 | 3 |
| m12 | 45 | 18 | 3 | 66 | 64 | 2 |
| m13 | 29 | 15 | 6 | 50 | 48 | 2 |
| m14 | 26 | 14 | 8 | 48 | 46 | 2 |
| m15 | 30 | 44 | 32 | 106 | 106 | 0 |
| **total** | **534** | **318** | **105** | **957** | **931** | **26** |

## Proposed target, not yet a gate

A useful starting hypothesis for the approved pool is approximately **45%
recall / 35% apply / 20% analyse**, with a review band of roughly 40--50%,
30--40%, and 15--25% respectively. This is a proposal for curriculum review,
not a ratified requirement and not permission to rewrite items to hit numbers.

The classifier cannot decide whether a purported application really requires
using a rule, whether an analysis really weighs a trade-off, or whether an
`understand` item is meaningful understanding rather than disguised recall.
It also cannot establish truth, grounding, discrimination, or learner
performance. Marginal cases require a human reading of the question and its
options; this inventory is the measured proxy that makes that review
targetable.
