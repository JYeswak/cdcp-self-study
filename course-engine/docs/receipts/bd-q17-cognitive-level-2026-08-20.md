# Q17 cognitive-demand measurement — 2026-08-20

## Result

All 957 `bank/items/*.toml` files were classified without changing any bank
file. The learner-relevant approved pool is 931 items; 26 retired items remain
in the 957-item denominator and are reported separately below.

This is a metadata proxy, not a human judgement of cognition:

| `bloom` value | Reported bin |
|---|---|
| `remember` | recall a fact |
| `understand` | recall proxy, conservatively |
| `apply` | apply a rule to a situation |
| `analyze`, `evaluate` | analyse a trade-off |

The `understand` collapse is deliberate and visible. It prevents a headline
claim of high judgement demand from being inferred from a label that does not
establish it. It also means the recall headline is an upper-bound proxy, not a
semantic adjudication.

## Per-module distribution

Counts are `recall / apply / analyse`; percentages are in the same order.
`all` includes retired files. `approved` is the learner pool.

| Module | All n | All counts | All % | Approved n | Approved counts | Approved % |
|---:|---:|---:|---:|---:|---:|---:|
| 01 | 43 | 30 / 10 / 3 | 69.8 / 23.3 / 7.0 | 43 | 30 / 10 / 3 | 69.8 / 23.3 / 7.0 |
| 02 | 52 | 35 / 15 / 2 | 67.3 / 28.8 / 3.8 | 52 | 35 / 15 / 2 | 67.3 / 28.8 / 3.8 |
| 03 | 55 | 31 / 17 / 7 | 56.4 / 30.9 / 12.7 | 55 | 31 / 17 / 7 | 56.4 / 30.9 / 12.7 |
| 04 | 39 | 23 / 13 / 3 | 59.0 / 33.3 / 7.7 | 39 | 23 / 13 / 3 | 59.0 / 33.3 / 7.7 |
| 05 | 33 | 20 / 11 / 2 | 60.6 / 33.3 / 6.1 | 32 | 19 / 11 / 2 | 59.4 / 34.4 / 6.2 |
| 06 | 146 | 82 / 54 / 10 | 56.2 / 37.0 / 6.8 | 140 | 76 / 54 / 10 | 54.3 / 38.6 / 7.1 |
| 07 | 35 | 21 / 8 / 6 | 60.0 / 22.9 / 17.1 | 34 | 20 / 8 / 6 | 58.8 / 23.5 / 17.6 |
| 08 | 39 | 25 / 11 / 3 | 64.1 / 28.2 / 7.7 | 37 | 23 / 11 / 3 | 62.2 / 29.7 / 8.1 |
| 09 | 127 | 75 / 46 / 6 | 59.1 / 36.2 / 4.7 | 122 | 70 / 46 / 6 | 57.4 / 37.7 / 4.9 |
| 10 | 37 | 20 / 11 / 6 | 54.1 / 29.7 / 16.2 | 35 | 18 / 11 / 6 | 51.4 / 31.4 / 17.1 |
| 11 | 81 | 42 / 31 / 8 | 51.9 / 38.3 / 9.9 | 78 | 40 / 30 / 8 | 51.3 / 38.5 / 10.3 |
| 12 | 66 | 45 / 18 / 3 | 68.2 / 27.3 / 4.5 | 64 | 44 / 17 / 3 | 68.8 / 26.6 / 4.7 |
| 13 | 50 | 29 / 15 / 6 | 58.0 / 30.0 / 12.0 | 48 | 28 / 14 / 6 | 58.3 / 29.2 / 12.5 |
| 14 | 48 | 26 / 14 / 8 | 54.2 / 29.2 / 16.7 | 46 | 25 / 13 / 8 | 54.3 / 28.3 / 17.4 |
| 15 | 106 | 30 / 44 / 32 | 28.3 / 41.5 / 30.2 | 106 | 30 / 44 / 32 | 28.3 / 41.5 / 30.2 |
| **Total** | **957** | **534 / 318 / 105** | **55.8 / 33.2 / 11.0** | **931** | **512 / 314 / 105** | **55.0 / 33.7 / 11.3** |

## Finding and provisional target

On the approved-pool proxy, recall dominates at 55.0%, analyse is only 11.3%,
and Module 15 is a clear outlier at 28.3 / 41.5 / 30.2. Modules 01, 02 and
12 are the highest-recall modules at 69.8%, 67.3% and 68.8% respectively.

A provisional review target is **50% recall / 35% apply / 15% analyse** for
the approved pool, with no module above 60% recall after human adjudication.
This is a review target, not permission to rewrite items or a certification
claim. It should be checked against the real exam blueprint before becoming a
gate; content coverage and cognitive demand are not interchangeable.

## What this cannot decide

`bloom` is an author-supplied label. It cannot determine whether a learner can
answer an `apply` or `analyse` item by recognizing a familiar phrase, whether a
scenario actually requires a decision, or whether an `understand` item is
genuine comprehension rather than fact recall. Mapping all 384 `understand`
labels to recall makes the headline conservative but does not resolve them.
Only a human reread can adjudicate those margins, and discrimination still
requires candidate response data that this repository does not have.

Reproduction (read-only):

```text
python3 + tomllib over bank/items/*.toml
mapping = remember,understand -> recall; apply -> apply; analyze,evaluate -> analyse
```

