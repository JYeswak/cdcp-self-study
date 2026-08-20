# bd-6pxr: exact-one correctness reread of m05, m07, and m14

Reviewed 2026-08-20. The question for every approved item was whether exactly
one option is defensible, independent of whether the keyed option was the
intended one. The review covered all 112 approved items:

| module | approved items read | item files changed |
|---|---:|---:|
| m05 | 32 | 1 |
| m07 | 34 | 0 |
| m14 | 46 | 1 |
| **total** | **112** | **2** |

The changed-surface priority set was 31 m05 items, 33 m07 items, and 40 m14
items whose stems, choices, or keys had changed in the recent plausibility
passes. The other 8, 1, and 6 items were also read rather than assumed clean.

## Findings and repairs

### (a) Arguably true distractor: 1, repaired

* `m05-q139`: the key is “Life-safety provision for egress when normal
  lighting fails.” Its changed distractor “A supplementary source that raises
  average illuminance during maintenance windows” could be defended as a
  situational use of emergency lighting. It was changed to “A general
  task-lighting source that replaces normal lighting during maintenance
  windows.” That remains a plausible maintenance confusion but is false for
  emergency lighting’s life-safety purpose. The correct key stayed `C`.

### (b) Ambiguous distractor: 1, repaired

* `bank-m14-q105`: EMS is intentionally used for environmental monitoring in
  this module, but the acronym has legitimate energy-management meanings
  outside that taxonomy. The stem was changed from “In data-centre
  auxiliary-system usage” to “In this course's data-centre
  auxiliary-monitoring taxonomy” so the question tests the taught term rather
  than an unresolved acronym collision. The correct key stayed `A`; the
  explanation was not changed.

### (d) Several distractors instantiate the keyed concept: 0

No item in these modules required “the intended answer is the best of these.”
In particular, no distractor set contained multiple examples of the concept
named by its key.

After the two repairs, no unresolved (a), (b), or (d) finding remains. m05,
m07, and m14 are clean on this human reread, with the limitation that this is
an adjudication pass, not a proof oracle.

## Scope and verification

The parsed diff against the pre-review tree contains exactly two changes:
`m05-q139.choices` and `bank-m14-q105.stem`. No `correct`, explanation,
citation comment, or other item field changed. `web/data/` was not edited or
staged.

Focused verification after the repairs:

* `verify-bank`: rc 0; 957 scanned, 931 approved.
* `answer-key-skew`: rc 0; A/B/C/D `235/243/229/224` and unchanged by this
  review.
* `near-duplicate-items`: rc 0; 0 near-duplicate pairs.
* `construction-faults`: rc 0; live verdict PASS; embedded
  `length-rank-uniformity=PASS` with counts `[253, 231, 220, 227]`.
