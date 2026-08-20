# bd-2qvn product-level guessing measurement

Measured 2026-08-20 through the shipped `cdcp_assemble::assemble` path. The
harness loads `bank/items`, calls the real approved-only stratified sampler and
choice-shuffle/remap path, and scores the resulting `AssembledExam`. It does
not resample or reimplement selection.

## Fixed denominator

The seed list was predeclared as the inclusive range `0..99` (100 seeds),
including the pinned seed 42. One hundred seeds gives one-percent resolution
for the pass share while remaining large enough to show the spread across
independent forms. All 100 assembled successfully as 40-question mocks, so
there were zero denominator failures.

Rules:

- longest: character length; ties select the first presented option;
- A/B/C/D: fixed presented position;
- hedged: first option containing one of `can`, `could`, `generally`, `may`,
  `might`, `often`, `usually`, `typically`, `tends`, or `sometimes`; if none,
  select A;
- stem overlap: most unique content-word overlap with the stem; ties select
  the first presented option;
- uniform random: independent ChaCha12 draw seeded with
  `seed ^ 0xBADC_0FFE`.

## Results

Scores are out of 40. Pass share uses all 100 predeclared seeds.

| Strategy | Mean | Min | Max | >=27 | Pass share |
| --- | ---: | ---: | ---: | ---: | ---: |
| always pick longest | 10.40 | 3 | 19 | 0/100 | 0.0% |
| always A | 10.20 | 5 | 19 | 0/100 | 0.0% |
| always B | 9.91 | 5 | 18 | 0/100 | 0.0% |
| always C | 9.61 | 4 | 16 | 0/100 | 0.0% |
| always D | 10.28 | 4 | 18 | 0/100 | 0.0% |
| hedged option | 12.27 | 6 | 21 | 0/100 | 0.0% |
| stem overlap | 11.99 | 6 | 18 | 0/100 | 0.0% |
| uniform random | 9.94 | 5 | 20 | 0/100 | 0.0% |

No residual seed reached 27/40, so there are no item-right dumps to attach.
The random control is close to the four-option chance mean of 10/40.

## Before/after

- bd-2qvn original assembled-mock claim: always-longest `36/40`.
  Product measurement: mean `10.40/40`, maximum `19/40`, `0/100` passes.
- bd-opyi original assembled-mock claim: always-A `21.9/40`.
  Product measurement: mean `10.20/40`, maximum `19/40`, `0/100` passes.

This closes the four named guessing routes at the product level for this fixed
100-seed sample. It does not show that the items teach anything or that they
discriminate between prepared and unprepared candidates. Those properties
require real candidate responses, which this project does not have.
