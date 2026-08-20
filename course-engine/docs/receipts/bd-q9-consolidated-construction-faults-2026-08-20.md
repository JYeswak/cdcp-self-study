# Q9 consolidated construction-fault receipt — 2026-08-20

This is the consolidated Q9 measurement of option-set construction cues. It
does not measure answer-key truth, currency, grounding, cognitive level,
redundancy, or candidate discrimination. Discrimination requires response data
that this repository does not have.

## Fixed denominator and commands

The denominator was declared before reading the scores: assembler seeds `0`
through `99`, inclusive (`100` seeds), each through the real
`cdcp_assemble::assemble` path. All 100 assembled a 40-question mock; no seed
was dropped or replaced. The product harness uses longest-option ties → first
occurrence, hedged no-match → A, stem-overlap ties → first occurrence, and
uniform random `ChaCha12(seed ^ 0xBADC0FFE)`.

Commands run individually (no full `scripts/check.sh` probe):

```text
cargo run -q -p cdcp_assemble --example guessing_strategies   rc=0
cargo run -q -p cdcp_gate -- construction-faults               rc=0
cargo run -q -p cdcp_gate -- answer-key-skew                   rc=0
cargo run -q -p cdcp_gate -- verify-bank                       rc=0
```

## Product-level guessing routes

All scores are out of 40; `pass` means `>=27/40`.

| strategy | mean | min | max | pass count/share |
|---|---:|---:|---:|---:|
| strictly-longest | 10.35 | 3 | 19 | 0/100 (0.0%) |
| always-A | 10.20 | 5 | 19 | 0/100 (0.0%) |
| always-B | 9.91 | 5 | 18 | 0/100 (0.0%) |
| always-C | 9.61 | 4 | 16 | 0/100 (0.0%) |
| always-D | 10.28 | 4 | 18 | 0/100 (0.0%) |
| hedged | 10.98 | 5 | 18 | 0/100 (0.0%) |
| avoid-hedged | 10.22 | 5 | 18 | 0/100 (0.0%) |
| stem-overlap | 11.07 | 6 | 19 | 0/100 (0.0%) |
| uniform random | 9.94 | 5 | 20 | 0/100 (0.0%) |

No named guessing route reached the study bar on any predeclared seed.

## Consolidated cue table

| cue | population/denominator | result | verdict or interpretation |
|---|---:|---:|---|
| length rank of keyed option | live approved, 931 | rank counts `254/223/214/240` = `27.3/24.0/23.0/25.8%` | PASS; expected `25% ± 10pp` |
| longest option is keyed | live approved, 931 | `176/931` = `18.9%` | observational only; not a verdict threshold |
| grammatical disagreement | live approved, 931 | `0/931` = `0.0%` | no detected cue |
| absolute-language distractor | live approved, 931 | `0/931` = `0.0%` | no detected cue |
| all/none-of-the-above | live approved, 931 | `0/931` = `0.0%` | no detected cue |
| hedged option, exactly one applicable | assembled items, 283 | key hit `61/283` = `21.6%`; avoid-hedged hit `83/283` = `29.3%` | both within the declared `20–30%` band |
| hedged strategy, whole mock | 100 assembled mocks | mean `10.98/40`; random control `9.94/40` | inside the harness control band; no pass-bar seed |
| unique nonzero stem-overlap option | assembled items, 1,849 | key hit `503/1849` = `27.2%` | within the declared `20–30%` band |
| stem-overlap strategy, whole mock | 100 assembled mocks | mean `11.07/40`; random control `9.94/40` | inside the harness control band; no pass-bar seed |

The construction-fault output also measured the preserved damaged corpus as a
known-bad control: longest-key `210/448` (46.9%), absolute-language distractor
`16/448` (3.6%), grammatical disagreement `0/448`, all/none `0/448`, and
length-rank `303/35/25/85` (67.6/7.8/5.6/19.0%, expected uniformity FAIL).
The live verdict was PASS; the damaged control was EXPECTED-RED.

## Answer-key skew by approved module

The aggregate approved pool is `931` items: `A=235 (25.2%), B=243 (26.1%),
C=229 (24.6%), D=224 (24.1%)`, inside the `15–35%` registry band. Every
module is also inside that band. The longest same-key run in sorted item-file
order is four.

| module | n | A | B | C | D | longest run |
|---:|---:|---:|---:|---:|---:|---:|
| 01 | 43 | 11 | 11 | 11 | 10 | 3 |
| 02 | 52 | 13 | 13 | 13 | 13 | 3 |
| 03 | 55 | 14 | 13 | 14 | 14 | 4 |
| 04 | 39 | 9 | 10 | 10 | 10 | 4 |
| 05 | 32 | 8 | 8 | 8 | 8 | 4 |
| 06 | 140 | 38 | 37 | 34 | 31 | 3 |
| 07 | 34 | 10 | 8 | 8 | 8 | 3 |
| 08 | 37 | 9 | 9 | 10 | 9 | 1 |
| 09 | 122 | 32 | 31 | 29 | 30 | 2 |
| 10 | 35 | 8 | 11 | 8 | 8 | 3 |
| 11 | 78 | 18 | 20 | 22 | 18 | 2 |
| 12 | 64 | 14 | 20 | 14 | 16 | 2 |
| 13 | 48 | 11 | 13 | 12 | 12 | 2 |
| 14 | 46 | 13 | 12 | 10 | 11 | 2 |
| 15 | 106 | 27 | 27 | 26 | 26 | 4 |

## Boundary of the result

These measurements close the named option-set guessing routes at the declared
measurement surface. They do not establish that the questions are true,
well-grounded, instructionally sufficient, or discriminating for prepared
versus unprepared candidates. Only candidate response data can establish the
last property.
