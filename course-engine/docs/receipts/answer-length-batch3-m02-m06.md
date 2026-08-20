# Answer-length batch 3: M02 completed, M06 stopped

Date: 2026-08-20  
Bead: bd-2qvn

## Scope and disposition

M02 covered 52 approved items. Thirty-seven item files needed choice-text
changes; 15 already occupied the desired keyed length ranks and were left
alone. The changes preserve each stem, correct key, status, module, topic IDs,
and citation/source comment lines. No key moved.

M06 was audited as the requested 140-item module, but its exploratory
rebalancing was discarded. It reached 35/35/35/35 only by appending generic
clauses to distractors and shortening some answers mechanically. That violates
the no-filler rule, so the M06 files were restored to their pre-batch state and
no M06 change is part of this commit. M06 remains an open substantive-review
finding rather than a claimed uniformity win.

## Keyed length rank

Rank 1 is longest and rank 4 is shortest, using the construction-faults gate's
character-length metric and deterministic A-D tie break.

| module | before counts | before shares | after counts | after shares |
| --- | --- | --- | --- | --- |
| M02 (52) | 50 / 2 / 0 / 0 | 96.2% / 3.8% / 0.0% / 0.0% | 14 / 13 / 14 / 11 | 26.9% / 25.0% / 26.9% / 21.2% |
| M06 (140) | 137 / 2 / 0 / 1 | 97.9% / 1.4% / 0.0% / 0.7% | 137 / 2 / 0 / 1 | 97.9% / 1.4% / 0.0% / 0.7% |

The M02 distribution is within the requested 20–30% band at every rank. M06
did not meet the band without disallowed padding, so this batch does not claim
M06 completion.

## Verification

- `answer-key-skew`: PASS — approved single-select=931; A=274 (29.4%), B=246
  (26.4%), C=209 (22.4%), D=202 (21.7%). The distribution is unchanged.
- `verify-bank`: PASS — 957 scanned, 931 approved; correct distribution is
  A=274, B=246, C=209, D=202.
- `construction-faults`: RED, as expected for the remaining bank-wide skew:

```text
live-approved: items=931; longest-option-correct=574 (61.7%); grammatical-disagreement=0 (0.0%); absolute-language-distractor=113 (12.1%); all-none-of-the-above=0 (0.0%); detector-hits=687; length-rank-uniformity=FAIL counts=[706,83,68,74] shares=[75.8%,8.9%,7.3%,7.9%] expected=25.0%±10pp
damaged-corpus: items=448; longest-option-correct=210 (46.9%); grammatical-disagreement=0 (0.0%); absolute-language-distractor=16 (3.6%); all-none-of-the-above=0 (0.0%); detector-hits=226; length-rank-uniformity=FAIL counts=[303,35,25,85] shares=[67.6%,7.8%,5.6%,19.0%] expected=25.0%±10pp
delta damaged-minus-live: longest-option-correct rate_delta=-14.8pp; absolute-language-distractor rate_delta=-8.6pp; length-rank-uniformity count_delta=[-403,-48,-43,+11]
```

The construction-fault gate remains RED because most modules are untouched; it
does not certify discrimination, answer-key correctness, or distractor
plausibility. No target/probe machinery was changed.
