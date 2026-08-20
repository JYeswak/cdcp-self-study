# Answer-length batch 2: M01 and M03

Date: 2026-08-20
Bead: bd-2qvn

This batch audited all 43 approved M01 items and all 55 approved M03 items.
Keys, stems, topic IDs, and citation/source comment lines were preserved. The
73 changed item bodies changed choices only; 25 items already occupied a rank
needed for the target distribution and were left unchanged.

## Keyed length rank

Rank 1 is longest and rank 4 is shortest, using the gate's character-length
metric and deterministic A-D tie break.

| module | before counts | before shares | after counts | after shares |
| --- | --- | --- | --- | --- |
| M01 (43) | 42 / 0 / 1 / 0 | 97.7% / 0.0% / 2.3% / 0.0% | 11 / 11 / 11 / 10 | 25.6% / 25.6% / 25.6% / 23.3% |
| M03 (55) | 54 / 1 / 0 / 0 | 98.2% / 1.8% / 0.0% / 0.0% | 14 / 14 / 13 / 14 | 25.5% / 25.5% / 23.6% / 25.5% |

The full approved bank moved from 813 / 48 / 31 / 39 (87.3% / 5.2% / 3.3%
/ 4.2%) to 742 / 72 / 54 / 63 (79.7% / 7.7% / 5.8% / 6.8%). The remaining
bank-wide skew is expected because M02, M04-M09, and M11-M15 have not yet been
adjudicated.

## Verification

- `answer-key-skew`: GREEN — A=274, B=246, C=209, D=202; unchanged.
- `verify-bank`: GREEN — 957 scanned, 931 approved.
- `construction-faults`: RED, with the rank report below. This is a real
  residual finding, not a bypass: the live population is still dominated by
  untouched modules.

```text
live-approved: items=931; length-rank-uniformity=FAIL counts=[742,72,54,63] shares=[79.7%,7.7%,5.8%,6.8%]
damaged-corpus: items=448; length-rank-uniformity=FAIL counts=[303,35,25,85] shares=[67.6%,7.8%,5.6%,19.0%]
```

Uniformity within M01 and M03 is not a discrimination certificate. It removes
one mechanical length cue; distractor plausibility and item discrimination
still require response data, which this project does not have.
