# bd-d97q inverted-hedge close

The residual exact-one population was measured before editing at 62 items,
with the lone hedged option keyed 0 times. Fourteen initially selected items
were joined by `m06-q090` as a fifteenth honest conditional rewrite. For each
of these 15, the key was qualified without changing its proposition and the
old distractor hedge was removed without making that distractor true.

Afterward the bank remains 62 exactly-one items, with 15 keyed lone hedges:
15/62 = 24.2%.

Edited ids:

`m01-q202`, `m01-q205`, `m03-q203`, `m04-q213`, `m05-q208`, `m06-q054`,
`m06-q090`, `m06-q098`, `m06-q201`, `m07-q053`, `m07-q202`, `m09-q126`,
`m10-q207`, `m14-q208`, `m15-q214`.

## Product measurement

The existing real-assembly harness used the unchanged seed denominator 0..99.

| strategy | mean / 40 | min | max | pass >=27 |
|---|---:|---:|---:|---:|
| hedged | 10.98 | 5 | 18 | 0/100 |
| avoid-lone-hedge | 10.25 | 5 | 18 | 0/100 |
| uniform-random | 9.94 | 5 | 20 | 0/100 |

On exactly-one assembled instances, the hedged strategy was correct 61/289
(21.1%), and the fixed first non-hedged choice was correct 83/289 (28.7%).
The prior inverted cue is therefore no longer clean. Multi-hedge instances
are reported separately as a diagnostic and are not used for the applicable
assertion.

## Verification

`construction-faults` live verdict PASS with zero absolute-language
distractors and length-rank-uniformity PASS at 27.4/25.0/23.4/24.2%.
`answer-key-skew`, `verify-bank`, `key-contradiction`, `verify-orphans`,
`validate-grounding`, `near-duplicate-items`, formatting, clippy, and the
assembly tests remain green. No `correct` field changed.

This closes the small inverted 62-item residual; it does not claim item truth
or candidate discrimination.
