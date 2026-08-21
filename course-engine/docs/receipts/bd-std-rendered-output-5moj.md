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
| Results (`results.html` / `results.js`) | 12 | exam, seed, bank hash, answer count, score, digest, engine, study signal, weak heading, weak chip, item status, chosen/correct letters, Learn link |
| Module quiz (`quiz.html` / `quiz.js`) | 10 | module picker, progress, status, question meta, option letters, unanswered hint, score, digest, mode, item review |
| Learn units / progress | 5 | unit status, “You are here” bar, quick-check heading/options, check completion, visited summary |
| Drill | 4 | mode heading, missed count, item module, correct label |
| Mastery hub | 4 | module row, badges, recommendation |
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
