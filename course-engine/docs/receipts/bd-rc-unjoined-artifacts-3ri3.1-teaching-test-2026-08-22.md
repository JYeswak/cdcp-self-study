# W1b teaching/test mismatch measurement — 2026-08-22

## Consumer and decision boundary

The consumer is the operator deciding whether W4 should fund a content wave. The
feature gated is the decision that a learner can prepare from the shipped module
page before answering that module's item. The observed defect was F-04: two
known items appeared to demand knowledge absent from their module pages, but the
rate across the bank had never been measured.

This receipt may be deleted when a full adjudicated census replaces this sample
as the W1b evidence, or when the same-module teaching/test acceptance is retired.

## Denominator and method

- 957 total item files: 931 `approved`, 26 `retired`.
- Predeclared sample: eight approved items per module, 120 items total.
- Within each module, approved IDs were lexically sorted and selected at
  `floor(i*n/8)` for `i = 0..7`.
- Retired items were excluded because they are not shipped to learners.
- The sample is systematic, not a census; its rate is not extrapolated to all
  931 approved items.
- The rendered module Markdown was assessed, including self-checks and
  interview drills.

## Result

| Module | Approved | Sample | Taught | Shallow | Absent | Contradicted |
|---|---:|---:|---:|---:|---:|---:|
| m01 | 43 | 8 | 8 | 0 | 0 | 0 |
| m02 | 52 | 8 | 8 | 0 | 0 | 0 |
| m03 | 55 | 8 | 8 | 0 | 0 | 0 |
| m04 | 39 | 8 | 8 | 0 | 0 | 0 |
| m05 | 32 | 8 | 8 | 0 | 0 | 0 |
| m06 | 140 | 8 | 8 | 0 | 0 | 0 |
| m07 | 34 | 8 | 8 | 0 | 0 | 0 |
| m08 | 37 | 8 | 8 | 0 | 0 | 0 |
| m09 | 122 | 8 | 8 | 0 | 0 | 0 |
| m10 | 35 | 8 | 8 | 0 | 0 | 0 |
| m11 | 78 | 8 | 8 | 0 | 0 | 0 |
| m12 | 64 | 8 | 8 | 0 | 0 | 0 |
| m13 | 48 | 8 | 8 | 0 | 0 | 0 |
| m14 | 46 | 8 | 8 | 0 | 0 | 0 |
| m15 | 106 | 8 | 5 | 2 | 1 | 0 |
| **sample total** | **931** | **120** | **117** | **2** | **1** | **0** |

The sample mismatch rate (`SHALLOW + ABSENT + CONTRADICTED`) is **3/120 =
2.5%**. Hard absence/contradiction is **1/120 = 0.8%**. These are sample
measurements, not bank-wide estimates.

The non-TAUGHT sample items are:

- `m15-q350` — **ABSENT**: the page does not teach sanitization or recording a
  reused device's destination.
- `m15-q363` — **SHALLOW**: the page teaches role/competence matrices and
  training records, but not job-based IDPs or DOE O 360.1D.
- `m15-q376` — **SHALLOW**: the page teaches service-level reporting and
  capacity planning, but not the specific connection from ticket totals to
  support-demand decisions.

The two known F-04 cases were separately targeted and are not folded into the
sample rate: `m10-q300` is **ABSENT** (the applied leak-response sequence), and
`m15-q385` is **ABSENT** (OSHA 1904.39 amputation reporting within 24 hours).

## Should-fail and judgement limits

`m03-q217` was first judged ABSENT after a search for “energization story” and
“no grid risk.” A closer read found the dedicated BTM section: it requires land,
fuel or gas path, emissions permits, and treats inability to host the plant as a
walk-away condition. It is **TAUGHT**. This guards against a term-search-only
absence verdict.

The closest calls were `m15-q363` and `m15-q376`; either could be called ABSENT
if the exact external artifact or sentence were required. They are SHALLOW here
because the underlying operational concept is present. Static reading cannot
decide whether a learner will infer, remember, or apply the knowledge, whether a
cross-module pointer is pedagogically sufficient, or whether an item is a good
discriminator. No CONTRADICTED pair appeared in this sample; that does not rule
one out across the bank.

## Evidence scope

Read-only inputs were `bank/items/m01-q*.toml` through `m15-q*.toml` and
`web/content/modules/01-*.md` through `15-*.md`; no bank item or module page was
modified.
