# bd-std-rendered-output-5moj — rendered-output contract

## Inventory denominator

The inventory was built by reading the learner HTML entry points and tracing
every `textContent`, `innerHTML`, label, progress-bar attribute, option badge,
score, denominator, module identifier, and status string in the production
renderers. Repeated item rows are one renderer site; independently drifting
labels and numeric dimensions are separate sites.

| renderer | sites | named sites |
|---|---:|---|
| Mock (`mock.html` / `mock.js`) | 10 | exam form meta, seed menu, progress, timer, submit label, unanswered hint, question meta, option letters, jump labels, pack identity |
| Results (`results.html` / `results.js`) | 13 | exam, seed, bank hash, answer count, score, digest, engine, study signal, weak heading, weak chip, item status, chosen/correct letters, Learn link |
| Module quiz (`quiz.html` / `quiz.js`) | 10 | module picker, progress, status, question meta, option letters, unanswered hint, score, digest, mode, item review |
| Learn units / progress | 5 | unit status, “You are here” bar, quick-check heading/options, check completion, visited summary |
| Drill | 4 | mode heading, missed count, item module, correct label |
| Mastery hub | 3 | module row, badges, recommendation |
| **Total** | **45** | **predeclared denominator** |

The inventory is explicit in `web/assets/js/smoke_rendered_output.js`; its
anti-vacuous count fails if any named assertion is deleted.

## Production-path evidence

`node web/assets/js/smoke_rendered_output.js` drives the actual `mock.js` and
`quiz.js` auto-start paths and the actual `results.js` auto-run path. Results
loads the shipped bank/key files and grades both attempts through the shipped
WASM bridge, not a computed-value substitute.

```text
rendered-output inventory: 45 named sites
known-good: exit=0; mock/results/quiz production renderers and WASM path passed
mock: {"progress":"1 / 40","meta":"Item 1 of 40 · m01-q201","letters":"ABCD","timer":"60:00"}
results all-correct: {"score":"40 / 40", ... "engine":"cdcp_wasm-wasm32"}
results all-wrong: {"score":"0 / 40", ... "engine":"cdcp_wasm-wasm32"}
quiz: {"progress":"1 / 8", ... "meta":"Module 06 · Item 1 of 8 · m06-q066","letters":"ABCD"}
```

The complete stable result strings included:

- `40 / 40 correct meets the practice bar of 27`;
- `0 / 40 is below the practice bar of 27`;
- `Study only — not a credential`;
- `ABCD` option badges and `60:00` timer;
- `Module 06 · Item 1 of 8 · m06-q066`.

## Known-bad and anti-vacuous legs

The smoke makes a temporary copy of the real `mock.js`, mutates the named
production component `badge.textContent = letter` so only the A badge renders
as B, and runs the same production entry point. The result was:

```text
known-bad option-letter: exit=2; only mock.option-letters RED:
mock option labels rendered "BBCD"
```

The other mock sites remained green in that run. Running with
`--delete-assertion` removed the `results.score` assertion from the inventory
and failed closed:

```text
rendered-output inventory incomplete: 44/45 assertions
exit=2
```

The clean inventory was 45/45, and the production runtime legs were green.

## Boundary

This proves stable DOM strings and the WASM-backed result path at the 45 sites
enumerated here. It does not prove CSS/layout/pixel rendering, browser-specific
font metrics, or that an unenumerated presentation site does not exist. A real
browser review remains necessary for those dimensions. The contract also proves
that the pack renders and grades as expected; it does not prove the content or
keys are pedagogically correct.

## Read-only drift audit (2026-08-20)

The 45-site denominator above is the denominator for the current rendered-output
assertion inventory. `E` means a complete exact runtime string is asserted, `P`
means only part of the rendered string is asserted, and `S` means the smoke only
checks that a source marker exists. A source-marker check is not rendered-output
coverage.

| # | site | production string (representative state) | coverage | unprotected component |
|---:|---|---|:---:|---|
| 1 | `mock.exam-form-meta` (`mock.html`) | `40Q · 60:00 · study bar 27/40 · study signal / not a pass mark` | S | Any label, unit, denominator, or pass-bar text can drift. |
| 2 | `mock.seed-menu` (`mock.html`) | `Seed`; option `42`; custom option `N (custom)` | S | Seed labels and option text can drift. |
| 3 | `mock.progress` (`mock.js`) | `1 / 40` | E | None in the exercised state. |
| 4 | `mock.timer` (`mock.js`) | `60:00` | E | None in the exercised state. |
| 5 | `mock.submit-label` (`mock.html`) | `Submit attempt` | S | Action wording can drift. |
| 6 | `mock.unanswered` (`mock.js`) | `0 of 40 answered. Submit unlocks when every item has a choice.` | S | Count, denominator, and unlock wording can drift. |
| 7 | `mock.question-meta` (`mock.js`) | `Item 1 of 40 · m01-q201` | P | The regex does not protect the item id or separator. |
| 8 | `mock.option-letters` (`mock.js`) | `ABCD` | E | None in the exercised state. |
| 9 | `mock.jump-labels` (`mock.js`) | Buttons `1`–`40`; ARIA `Question N, answered/unanswered` | S | Visible numbers and accessibility labels can drift. |
| 10 | `mock.pack-identity` (`mock.js`) | `mock40_seed42.json` | S | Pack and seed identity can drift. |
| 11 | `results.exam` (`results.js`) | `mock40` | S | Exam identity can drift. |
| 12 | `results.seed` (`results.js`) | `42` | S | Displayed seed can drift. |
| 13 | `results.bank-hash` (`results.js`) | `26003203...` (full hash in the title) | S | Truncation and hash display can drift. |
| 14 | `results.answer-count` (`results.js`) | `40 item(s) recorded` | S | Count and wording can drift. |
| 15 | `results.score` (`results.js`) | `40 / 40` or `0 / 40` | E | Endpoint score formatting is covered. |
| 16 | `results.digest` (`results.js`) | 64-character lowercase digest | S | Displayed digest can drift. |
| 17 | `results.engine` (`results.js`) | `cdcp_wasm-wasm32` | E | Engine identity is covered. |
| 18 | `results.study-signal` (`results.js`) | `Study signal: 40 / 40 correct meets the practice bar of 27. This is not EPI/EXIN certification and is never a CDCP credential. Treat it as readiness practice only.` | P | Score, bar, disclaimer, and wording are not exact-checked. |
| 19 | `results.weak-module-heading` (`results.js`) | `Weak modules` plus none/CTA text | P | Only the heading substring is checked. |
| 20 | `results.weak-module-chip` (`results.js`) | `M06` | S | Module-number formatting can drift. |
| 21 | `results.item-status` (`results.js`) | `Correct` or `Incorrect` | P | Only presence is checked. |
| 22 | `results.item-letters` (`results.js`) | `chosen A · correct B` | P | Letters and separators can drift. |
| 23 | `results.learn-link` (`results.js`) | `Review section in Learn →` or `Review module in Learn →` | S | Destination wording and suffix can drift. |
| 24 | `quiz.module-picker` (`quiz.js`) | `Module 06`; `Start quiz` | S | Module and action labels can drift. |
| 25 | `quiz.progress` (`quiz.js`) | `1 / 8` | E | None in the exercised state. |
| 26 | `quiz.status` (`quiz.js`) | `Module 06 quiz: 8 items (approved pool 140 of 146 in bank). Study only — not a credential.` | P | Count, pool figures, and disclaimer are not exact-checked. |
| 27 | `quiz.question-meta` (`quiz.js`) | `Module 06 · Item 1 of 8 · m06-q066` | P | Only a prefix regex is checked. |
| 28 | `quiz.option-letters` (`quiz.js`) | `ABCD` | E | None in the exercised state. |
| 29 | `quiz.unanswered` (`quiz.js`) | `Answer all items to enable grade.` or `N of T answered. Grade unlocks when every item has a choice.` | S | Counts and gating wording can drift. |
| 30 | `quiz.score` (`quiz.js`) | `8 / 8` | S | Quiz score formatting can drift. |
| 31 | `quiz.digest` (`quiz.js`) | Full digest or `— (no WASM digest; key-compare only)` | S | Digest and fallback wording can drift. |
| 32 | `quiz.mode` (`quiz.js`) | `Graded via WASM (cdcp_wasm-wasm32)...` or the WASM-unavailable fallback | S | Grading mode and disclaimer can drift. |
| 33 | `quiz.item-review` (`quiz.js`) | Per-item `Correct/Incorrect`, id, `chosen A · correct B`, explanation | S | The complete review can drift. |
| 34 | `learn.unit-status` (`learn_units.js`) | `Unit 1 / N · ~M min · 5–8 min target` | S | Unit, count, and time labels can drift. |
| 35 | `learn.here-bar` (`learn_units.js`) | `You are here · unit 1 of N · ~M min (5–8 min target)` | S | Unit/count/time text and progress can drift. |
| 36 | `learn.quick-check` (`learn_units.js`) | `Quick check (study only)` or its empty-state text | S | Study-only labeling can drift. |
| 37 | `learn.check-completion` (`learn_units.js`) | `Check complete · c/n (study signal only — not a credential)` | S | Completion count and disclaimer can drift. |
| 38 | `learn.visited-summary` (`learn_progress.js`) | `Visited X of Y modules (this browser only).` | S | Counts and scope wording can drift. |
| 39 | `drill.mode-heading` (`drill.js`) | `Drill / short-interval review`, or due/missed variants | S | Mode and interval labels can drift. |
| 40 | `drill.missed-count` (`drill.js`) | `Drill ready · missed N · due M. Study only — not a credential.` | S | Counts and disclaimer can drift. |
| 41 | `drill.item-module` (`drill.js`) | `M06`, with `ivl 1d · due now`-style text | S | Module and scheduling labels can drift. |
| 42 | `drill.correct-label` (`drill.js`) | `correct B` | S | Rendered key letter can drift. |
| 43 | `hub.module-row` (`hub_mastery.js`) | `01`–`15`, module title, and `Quiz` | S | Ordering, module labels, and action text can drift. |
| 44 | `hub.badges` (`hub_mastery.js`) | `Open`, `Practiced`, or `Mastered`, with threshold tooltip | S | Status and threshold text can drift. |
| 45 | `hub.recommendation` (`hub_mastery.js`) | `Next up · Quiz M06 · <title> · <reason>` or `All modules practiced` | S | Recommendation, module, and reason can drift. |

Of the 45 named sites, 7 have complete exact runtime assertions, 7 are partial,
and 31 are source-marker only. Therefore **38/45 are not fully protected from
rendered drift**. The known-bad option-letter mutation and the 44/45 deleted-
assertion leg prove that the smoke can fail; they do not upgrade partial or
source-marker coverage to exact coverage.

## Additional surface found outside the 45-site denominator

The read-only walk also found dynamic sites not separately represented in the
current inventory: full-article progress and TOC labels, glossary term popovers,
quick-check option labels, Drill card actions (`Show answer`, `Good`, `Again`),
the Drill SRS table and interval/due values, Hub due-card messages, the Hub
`Continue unit` state, and Learn-unit progress-bar percentages. These repeat by
unit, module, and state, so they require an explicit decision about whether the
denominator counts renderer sites, state variants, or instances. They are not
silently folded into 45; **45 is the current predeclared smoke denominator, not
a claim that the whole learner surface has been exhaustively enumerated**.

## Grading-boundary audit

`web/assets/js/grade_bridge.js` sends bank and attempt JSON through WASM and
returns only a 64-character GradeExact digest or an error. It does not return a
score or module breakdown. `results.js` independently builds the displayed
score, study signal, weak modules, per-item status, and chosen/correct letters
from the JS keys pack and bank. `quiz.js` does the same through
`gradeByKeys(...)`, while WASM supplies only the digest.

Consequently, a correct WASM grade can still be rendered incorrectly if the JS
presentation path, key-pack ordering, module mapping, threshold, or formatter
drifts. The current endpoint tests prove WASM loading and `40 / 40` / `0 / 40`,
but do not cross-check an arbitrary displayed score, weak-module breakdown,
per-item letter, or pass-bar string against a decoded WASM report. The boundary
proves the grade digest and the tested display endpoints, not the full
`WASM grade → JavaScript presentation → learner-visible string` mapping.

Exact-string assertions would prove correctness only at the sites and states
enumerated here. They can never prove that this enumeration is complete; a
browser review and deliberate expansion of the denominator are still needed
for the additional sites above, CSS/layout, and browser-specific rendering.
