# Q18 — assembled module coverage

## Verdict

No public official EPI domain-weight vector is available to compare against.
`docs/OQ_REGISTER.md` marks OQ-06 (official EPI domain weights) **FORBIDDEN
unknown** and says never to claim an official blueprint. This receipt therefore
measures the implemented sampler against an equal-share diagnostic only; it does
not certify blueprint fidelity.

## Measurement

- tree: `worktree`
- bank: 931 approved / 957 loaded; bank hash `b5e419ade0b93e14bd3e465d767a5325b1759a247dbd0e47bb9f230222551fcc`
- command: `target/debug/cdcp assemble --seed 0..99`, default `n_items=40`,
  `max_per_module=8`, `min_modules=8`
- attempted: 100 seeds
- successful: 100/100
- selected items: 4,000
- equal-share diagnostic: 266.667 items/module (6.667%)
- per-form shape: every module appeared in every form, with 2 or 3 items per
  form; this is the observed result of the current stratified sampler, not an
  official exam rule.

| module | selected | share | delta from equal |
|---:|---:|---:|---:|
| 01 | 267 | 6.675% | +0.333 pp |
| 02 | 268 | 6.700% | +0.533 pp |
| 03 | 273 | 6.825% | +1.158 pp |
| 04 | 268 | 6.700% | +0.533 pp |
| 05 | 266 | 6.650% | -0.017 pp |
| 06 | 269 | 6.725% | +0.058 pp |
| 07 | 266 | 6.650% | -0.017 pp |
| 08 | 260 | 6.500% | -0.167 pp |
| 09 | 261 | 6.525% | -0.142 pp |
| 10 | 267 | 6.675% | +0.333 pp |
| 11 | 267 | 6.675% | +0.333 pp |
| 12 | 270 | 6.750% | +0.083 pp |
| 13 | 262 | 6.550% | -0.117 pp |
| 14 | 268 | 6.700% | +0.533 pp |
| 15 | 268 | 6.700% | +0.533 pp |
| **total** | **4,000** | **100.000%** | |

The sampler is therefore close to equal module frequency despite the approved
pool being uneven (`m06=140`, `m09=122`, `m15=106`, versus `m05=32` and
`m08=37`). That is an implementation measurement, not evidence that equal
weight is the real exam blueprint.

## Boundary

This run cannot decide what the real exam weights are, whether the public exam
uses these 15 modules, or whether equal-share practice is pedagogically right.
It also does not test item quality, proposition distinctness, or correctness.
An externally sourced weighting may be compared in a future, explicitly scoped
tick; no weighting was invented here.
