# W1b teaching/test mismatch census — 2026-08-22

## Decision boundary

This measures whether the shipped bank is plausibly taught by the module page
the learner is directed to. It is a floor for deciding whether a content wave
is warranted; it is not a claim that lexical similarity proves a learner can
make the decision. No bank item or module page was changed.

## Denominator (predeclared)

- The repository contains **957** item files.
- The shipped population is the **931 `approved`** rows.
- The remaining **26 retired/non-shipped** rows are excluded because a learner
  cannot receive them. They remain in the inventory count so a silent status
  change cannot move the denominator.
- All 15 module Markdown pages are required and read. A missing or tiny page is
  an error, not an empty teaching result.

## Predeclared rubric

The reproducible census uses the topic labels attached to each item and the
module's rendered Markdown. Tokens are lower-cased, punctuation-separated,
and common function words are removed. `topic_support` is the fraction of the
item's topic-label tokens present in the module. `evidence_support` is the
fraction of the correct choice plus explanation tokens present in the module.

- **TAUGHT**: `topic_support >= 0.50` and `evidence_support >= 0.55`.
- **SHALLOW**: `topic_support >= 0.40` and `evidence_support >= 0.30`, but
  below the TAUGHT floor.
- **ABSENT**: below both review floors, or a prior human read explicitly found
  that the required decision is not taught.
- **CONTRADICTED**: reserved for a human-confirmed conflict that names both
  the item id and the conflicting module/source artifact id. The lexical
  scanner never invents this state.

The five prior human adjudications are fixed inputs, not tuned after seeing the
census: `m10-q300`, `m15-q350`, and `m15-q385` are ABSENT; `m15-q363` and
`m15-q376` are SHALLOW. The earlier 120-row sample remains at
`3/120 = 2.5%` mismatch and is not silently substituted for this census.

## Result

| Population | TAUGHT | SHALLOW | ABSENT | CONTRADICTED | mismatch floor |
|---|---:|---:|---:|---:|---:|
| 931 approved items | 826 | 101 | 4 | 0 | **105/931 = 11.3%** |

This **11.3% is a lexical review-floor rate, not the semantic teaching rate**.
The confirmed prior human cases are a known lower bound of 5 rows, not an
extrapolation. The measurement therefore says that at least 105 rows require
review under the declared floor, while the actual learner-teaching mismatch
rate remains a human-adjudication question.

| Module | Approved | TAUGHT | SHALLOW | ABSENT | CONTRADICTED | mismatch | rate |
|---|---:|---:|---:|---:|---:|---:|---:|
| m01 | 43 | 39 | 4 | 0 | 0 | 4 | 9.3% |
| m02 | 52 | 46 | 6 | 0 | 0 | 6 | 11.5% |
| m03 | 55 | 43 | 11 | 1 | 0 | 12 | 21.8% |
| m04 | 39 | 37 | 2 | 0 | 0 | 2 | 5.1% |
| m05 | 32 | 28 | 4 | 0 | 0 | 4 | 12.5% |
| m06 | 140 | 131 | 9 | 0 | 0 | 9 | 6.4% |
| m07 | 34 | 28 | 6 | 0 | 0 | 6 | 17.6% |
| m08 | 37 | 34 | 3 | 0 | 0 | 3 | 8.1% |
| m09 | 122 | 112 | 10 | 0 | 0 | 10 | 8.2% |
| m10 | 35 | 22 | 12 | 1 | 0 | 13 | 37.1% |
| m11 | 78 | 61 | 17 | 0 | 0 | 17 | 21.8% |
| m12 | 64 | 61 | 3 | 0 | 0 | 3 | 4.7% |
| m13 | 48 | 45 | 3 | 0 | 0 | 3 | 6.2% |
| m14 | 46 | 40 | 6 | 0 | 0 | 6 | 13.0% |
| m15 | 106 | 99 | 5 | 2 | 0 | 7 | 6.6% |

The full list of every non-TAUGHT row, with its module, scores, and reason, is
emitted by the measurement command below. The receipt names the prior human
cases explicitly because those are the rows currently safe to call content
findings rather than lexical review candidates.

## Known human-read cases

- `m10-q300` — **ABSENT**: the module does not teach the applied leak-response
  sequence (contain/isolate/protect/notify/document).
- `m15-q350` — **ABSENT**: the module does not teach sanitization or recording
  a reused device's destination.
- `m15-q385` — **ABSENT**: the module does not teach the OSHA 1904.39
  amputation-reporting timeline.
- `m15-q363` — **SHALLOW**: role/competence matrices and training are taught,
  but not job-based IDPs or DOE O 360.1D.
- `m15-q376` — **SHALLOW**: service-level reporting and capacity planning are
  taught, but not the specific support-demand decision.

No CONTRADICTED row is claimed. A contradiction requires two named artifacts;
the lexical scanner cannot establish one.

## Should-fail and limits

`m03-q217` was initially judged ABSENT after a search for “energization story”
and “no grid risk.” A closer read found the dedicated BTM section: land, fuel
or gas path, permits, and walk-away conditions. It remains **TAUGHT**. This is
why a token absence cannot be promoted directly to a content rewrite.

The scanner cannot decide whether a learner will infer, remember, or apply a
decision; whether a cross-module pointer is pedagogically sufficient; or
whether an item is a good discriminator. “The page mentions the term” is not
“the page teaches the decision.” The next honest step for a semantic rate is
human adjudication of the 105 review-floor rows, with CONTRADICTED requiring
both artifact ids.

## Reproduction

```text
cargo run -q -p cdcp_assemble --example teaching_mismatch
```

Implementation: `crates/cdcp_assemble/examples/teaching_mismatch.rs`.
