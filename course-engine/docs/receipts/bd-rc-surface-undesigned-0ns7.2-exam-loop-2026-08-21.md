# W3 exam-loop verification — 2026-08-21

Bead: `bd-rc-surface-undesigned-0ns7.2` (still `in_progress`)

This is a verification and decision packet, not a close receipt.  The browser
was tested against `http://127.0.0.1:8766/` after commit `2ff0d5f2`
(`feat(cdcp-course): make exam recovery actionable`).  No product decision was
invented for the one-seed limit (F-25) or the multiple Drill entry points (F-28),
and `docs/ux/UX-FINDINGS-LOG.md` remained frozen.

## Protocol

- Fresh browser pages, seed `42`, the shipped 40-item pack, and the live server;
  viewport measurements were taken at `1280×900` and `375×800`.
- The bad-result run answered the first six items with their shipped keys and the
  remaining 34 with a wrong option.  It produced `6 / 40` through the real WASM
  results route.
- Q37 was tested after answering Q1–Q36, then the flag control was toggled.
  Focus order was measured after all 40 answers were present while viewing Q1.

## Findings against the W3 decision list

| finding | live result after `2ff0d5f2` | decision/status |
|---|---|---|
| F-05 submit point of no return | `web/assets/js/mock.js:654-680` now calls `window.confirm(...)` before writing the attempt and navigating. An instrumented browser run that returned `true` reached Results and produced `6 / 40`. | The accidental-submit path now has a confirmation; copy and the human choice of warning semantics remain review items. |
| F-06 incomplete submit | At Q37 with 36/40 answered: progress was `37 / 40 · 36 answered · 4 left`; the submit button remained disabled. The new review action read `Review unanswered (Q37, Q38, Q39, Q40)` and its accessible name named all four gaps. | The missing-question information is actionable, but early submission is still a product decision, not silently resolved. |
| F-07 review later | Q37 exposed `Flag for review`; toggling it produced `aria-pressed="true"`, status `Flagged for review — use the question map to return here.`, and a jump-chip label containing `flagged for review`. | Browser leg passes. |
| F-08 tired/progress state | The Q37 accessible progress was `Question 37 of 40, 36 answered, 4 remaining`; the visible line included `4 left`; four jump chips were unanswered. | Browser leg passes. |
| F-09 submit reachability | With all answers and Q1 current, Submit was focus index 15 and the first jump button index 16, so Submit precedes the 40-button map. However the hidden review button was still focus index 14. | The main ordering change works, but the hidden-control regression below prevents a clean close. |
| F-10 recovery after 6/40 | At `1280×900`: document `10,534px`, item review `8,848px`, new recovery CTA top `1,008.3px`, old footer Drill link `10,453.7px`. At `375×800`: document `15,982px`, item review `13,513px`, recovery CTA top `1,277.0px`, footer link `15,901.8px`. Frozen pre-W2 baseline was `15,233px` / footer y `15,152px` in `UX-FINDINGS-DEDUP.md`. | The new “Review 34 missed item(s)” route is near the score; the legacy footer remains far below the review. No before/after improvement is claimed beyond these measured states. |
| F-11 weak modules | The bad result now showed detail such as `M02 0/3 · 0% · 3 missed`, ordered by correctness/missed count, rather than bare module IDs. | Browser leg passes. |
| F-12 returning state | Returning to `mock.html` after a submitted attempt restored `40 / 40 · 40 answered · 0 left`; the draft and attempt keys remained in `sessionStorage`, and Submit was enabled. The new confirmation now protects a second submission. | Persistence is confirmed; whether a graded attempt should reopen as an editable draft remains a product-copy/state decision. |
| F-25 seed breadth | `#seed-select` still contains exactly one option: `42`. | Human decision required; do not invent more seeds. |
| F-28 Drill hierarchy | The hub currently exposes five Drill links, including the nav, card/link variants, and `drill.html?mode=due`. | Human decision required: one surface with modes or distinct named surfaces. |

## New close blocker found during verification

The new review control was initially set to `hidden = true` when all 40 items
were answered (`web/assets/js/mock.js:497-510`), but the browser computed
`display: flex`, `visibility: visible`, a `228×44px` rectangle, and `tabIndex=0`
because the author stylesheet's `.btn { display: inline-flex; }` at
`web/assets/css/course.css:325-337` overrides the user-agent `[hidden]` rule.
It remained visible/focusable after `40 / 40 · 40 answered · 0 left` and retained
the stale text `Review unanswered (Q40)`.  This was a real regression in the W3
patch, not a reason to weaken a test.  Follow-up commit `88fc5086` now sets an
explicit inline hidden-state display and clears the stale label; a fresh browser
run measured `display:none`, a zero-sized rectangle, and no focus-order entry.
The same explicit pre-grade hiding was applied to the Results recovery panel.

The existing rendered-output smoke also reds before it can certify the patch:

```
node web/assets/js/smoke_rendered_output.js
smoke_rendered_output: ERROR: mock.unanswered: source marker missing
(web/assets/js/mock.js: answered + " of " + total)
```

That fixture still asserts the pre-W3 copy and must be updated by its owner in
the same scoped product change.  The hidden-control defect is now resolved, but
this red and the human decisions below still prevent an honest W3 close.

## Decisions still requiring the operator

1. Whether to ship additional deterministic seed packs, and what diversity
   contract makes the seed selector meaningful (F-25).
2. Whether Drill is one learner surface with modes or several named surfaces
   (F-28).
3. Whether incomplete submissions are allowed after a confirmation warning or
   remain blocked with the now-actionable unanswered review route (F-06).
4. Whether the old footer Drill link should remain as a secondary route after
   the near-score recovery CTA (F-10).

No item above is a license to fabricate a seed, erase a route, or claim the
KEEP list stayed green.  The W3 bead remains open pending the owner fix,
rendered-output smoke update, and human decisions.
