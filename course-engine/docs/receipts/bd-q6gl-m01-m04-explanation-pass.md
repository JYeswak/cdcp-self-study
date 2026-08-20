# bd-q6gl: explanation pass for m01–m04

Reviewed on 2026-08-20 against the approved bank. This pass changed the
`explanation` field only. Stems, choices, correct keys, statuses, topic and
objective fields, and citation comments were preserved.

## Scope

| module | approved reviewed | explanations rewritten | already met the shape |
|---|---:|---:|---:|
| m01 | 43 | 35 | 8 |
| m02 | 52 | 42 | 10 |
| m03 | 55 | 46 | 9 |
| m04 | 39 | 33 | 6 |
| **total** | **189** | **156** | **33** |

Each rewritten explanation states why the keyed proposition is correct and
names the learner's confusion for each genuinely attractive distractor. No
rewritten explanation names an option by letter.

## Escalations

* **(a) arguably true: 1** — `mock40-q02` remains unchanged. Its distractor
  “Five-nines identifies an availability target, not a utility-feed
  arrangement” is independently true, so the item has a second defensible
  statement. This is a correctness defect, not an explanation problem.
* **(b) ambiguous: 0** — no additional item was found where a distractor is
  false under one reading and true under another.
* **(c) weak: 67 item IDs** — this is a conservative content-review count,
  counting an ID once when it contains at least one clearly off-domain or
  low-attractiveness distractor. There is no learner response data, so this is
  not a claim that nobody would select one. Weak options were documented in
  the explanation and left for later construction work.
* **(d) several distractors instantiate the keyed concept: 0** — no item in
  these four modules required the “best of these” qualification.

The weak-finding IDs are:

* **m01 (13):** `mock40-q04`, `m01-q044`, `m01-q045`, `m01-q046`,
  `m01-q048`, `m01-q049`, `m01-q052`, `m01-q053`, `m01-q062`, `m01-q200`,
  `m01-q203`, `m01-q204`, `m01-q209`.
* **m02 (10):** `m02-q064`, `m02-q066`, `m02-q070`, `m02-q078`,
  `m02-q079`, `m02-q082`, `m02-q089`, `m02-q201`, `m02-q203`, `m02-q209`.
* **m03 (26):** `mock40-q07`, `mock40-q08`, `m03-q093`, `m03-q094`,
  `m03-q096`, `m03-q097`, `m03-q098`, `m03-q100`, `m03-q101`, `m03-q102`,
  `m03-q104`, `m03-q105`, `m03-q107`, `m03-q108`, `m03-q110`, `m03-q111`,
  `m03-q113`, `m03-q200`, `m03-q201`, `m03-q202`, `m03-q204`, `m03-q208`,
  `m03-q209`, `m03-q210`, `m03-q211`, `m03-q213`.
* **m04 (18):** `mock40-q10`, `mock40-q11`, `m04-q116`, `m04-q117`,
  `m04-q118`, `m04-q119`, `m04-q120`, `m04-q124`, `m04-q125`, `m04-q126`,
  `m04-q129`, `m04-q130`, `m04-q131`, `m04-q132`, `m04-q200`, `m04-q201`,
  `m04-q205`, `m04-q207`.

## Invariants and gates

The parsed before/after comparison for all 189 items found no field changes
outside `explanation`; the changed-file diff also contains only explanation
lines. The four module commits were kept separate:

* m01 `b3a602f`
* m02 `061af53`
* m03 `a409c7d`
* m04 `525c7d2`

Focused verification after the batch:

* `construction-faults`: rc 0; live verdict PASS; embedded
  `length-rank-uniformity=PASS`, counts `[254, 231, 220, 226]`, shares
  `[27.3%, 24.8%, 23.6%, 24.3%]`.
* `answer-key-skew`: rc 0; A/B/C/D `235/243/229/224` (`25.2%/26.1%/24.6%/24.1%`),
  within the 15–35% band.
* `near-duplicate-items`: rc 0; 0 near-duplicate pairs.
* `verify-bank`: rc 0; 957 scanned, 931 approved.

The standalone `length-rank-uniformity` subcommand does not exist; its
verdict is the embedded construction-faults result above. The dirty
`web/data/` files were neither edited nor staged by these commits. The
goldens coupling remains intentionally RED pending `bd-q6gl`'s later pack
regeneration.
