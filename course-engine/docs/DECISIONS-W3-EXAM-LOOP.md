# Decision record — W3 exam-loop, the four blocked questions

**Bead:** `bd-rc-surface-undesigned-0ns7.2` (W3: exam-loop experience — needs a design decision, not
just a fix)
**Date:** 2026-08-21
**Decided by:** controller (cockpit), on delegated authority. Josh handed the call over explicitly
rather than answering item by item. Recorded here because an unrecorded fork gets re-litigated by
the next agent.
**Status of these four before this record:** `HUMAN DECISION REQUIRED` in
`docs/receipts/bd-rc-surface-undesigned-0ns7.2-exam-loop-2026-08-21.md`, which correctly refused to
invent them. That refusal was right. This document is the answer, not a bypass of it.

Each decision below states the call, the reasoning, what gets registered so it cannot silently
drift, and a should-fail so the change cannot be vacuously green.

---

## D-1 · Seed breadth (F-25) — ship five named forms, stop exposing the integer

**Call:** `#seed-select` ships **five** deterministic forms, labelled **Form A–E** in the learner
UI, backed by seeds `42, 137, 251, 389, 503`. Seed 42 stays Form A and keeps its existing golden
pair, so nothing currently pinned moves.

**Reasoning.**

The constraint everyone assumed was a data constraint is not one. `web/data/bank_items_seed42.json`
is **973 KB and contains the entire 957-item bank** — it already ships. A form pack
(`mock40_seed42.json`) is **20 KB**: forty item references and a shuffle remap. The marginal cost of
a fifth form is ~33 KB on a 1.3 MB pack. The one-option selector was never a size or engineering
limit. It is an absence, which is exactly the RC-C diagnosis — *verified, never designed*.

Five, specifically, because it is the smallest number that satisfies two independent bounds:

- **Study need.** `STUDY-PLAN-14-DAY.md` is a fourteen-day schedule. Five forms let a learner sit a
  fresh exam on days 1, 4, 7, 10 and 14 with no repetition. Four leaves the capstone reusing a form;
  six buys nothing the plan asks for.
- **What we can afford to prove.** Every shipped form gets a **golden pair** (all-correct and
  all-wrong digest), asserted Rust == WASM on every gate run. Five forms is ten digests. That is the
  ceiling at which exhaustive verification stays cheap. **A form without a golden pair is a build
  error, not a shipped form.** Shipping twenty forms and goldening one would manufacture nineteen
  unverified surfaces — the precise failure this project exists to refuse.

**Stop exposing the integer.** `web/mock.html:57` renders a raw `u64` in a learner-facing control,
and `web/mock.html:84` prints "This copy includes seed 42". That is engineering internals on a study
surface — the same defect class as F-26 (item ids in the header) and F-23 (`CDCP_FILE_ORIGIN` in the
hub hero). The learner picks *Form C*. The seed integer moves into a "reproduce this exam"
affordance, which pairs naturally with the F-26 fix that turns the item id into a report-this-question
channel. Reproducibility is preserved and stops being the default reading.

**Registered so it cannot drift.**
- `claim-form-count-5` in `registries/claims.toml`, enforced by `claims-lint` across README, hub and
  `mock.html`. This prospectively kills the F-22 class of defect — the "14 modules" printed directly
  above fifteen rows — for form counts.
- A **diversity contract** as an SLO row, not an invariant: cross-form item overlap bounded, and
  per-module strata consistent across all five forms. It belongs in `slo.toml` because it is a
  statistical property. Per the strength lattice a statistical claim may not justify an invariant,
  and stating it as one would be a build error.

**Should-fail.** Plant two forms with identical seeds; the diversity gate must go RED and name the
colliding pair. Plant a sixth form with no golden pair; the build must fail. Neither may pass.

**Green does not prove.** Five verified forms prove five forms assemble deterministically and grade
identically in both engines. They do not prove the five are *pedagogically* distinct — that is what
the diversity SLO measures, and it is a proxy for distinctness, not distinctness itself. Say so in
the gate output, per RC-A.

---

## D-2 · Drill hierarchy (F-28) — one surface, three modes, one stateful hub entry

**Call:** `drill.html` is the single Drill surface with three modes. The hub shows **exactly one**
Drill entry, and its label and destination are computed from learner state.

**Reasoning.**

There are currently **five routes to Drill** on the hub — `web/index.html:28` (nav),
`:51` (`?mode=due`), `:61` (`?mode=due` again), `:74` (bare), `:79` (`?mode=miss`) — for **three**
distinct behaviours. Drill-due, Drill and Miss-review are not three products. They are three modes
of one object: *items you should practise now*. Five names for three modes of one thing is what
produced eight equal-weight cards, and the eight equal-weight cards are why the hub answers no
question.

The modes:

| mode | means |
|---|---|
| `?mode=due` | short-interval ladder, cards due now |
| `?mode=missed` | misses from the last graded attempt |
| `?mode=module&m=NN` | one module's items |

Nav keeps **one** "Drill" item. Mode selection lives inside the surface, where a learner can see all
three and pick — rather than being scattered across the hub, where they read as unrelated features.

**The hub card becomes stateful:**

- due > 0 → **"Drill · N cards due"** → `?mode=due`
- due == 0 and last attempt has misses → **"Review N missed items"** → `?mode=missed`
- neither → **"Drill · nothing due"**, naming the next action

That last branch is the one worth being careful about, because it is where two honest readings
collided. `w1-pane3.md` G-31-10 argues our "No cards due" empty state is a *strength* — it names the
next action instead of showing a blank, which several corpus sites do not manage. That is correct
and it is a measured KEEP. My reading was that it is the worst thing on the hub, because on a first
visit the only wayfinding element on the page is a null state telling a brand-new learner to go
somewhere else and come back.

**Both are true, and the state-computed card resolves it without discarding either.** A good empty
state stays a good empty state. It simply stops being the *first-run* experience, because on first
run the card renders the due/missed branch or names the starting module instead. The KEEP survives;
the cold-start hole closes. Neither reading had to lose.

Net effect: five Drill routes collapse to one, the hub drops from eight equal cards toward five or
six, and one of them finally carries state — which is the `F` grade on state representation starting
to move.

**Registered.** `smoke_rendered_output.js` must assert the **rendered card label** for all three
branches. A correct internal mode value proves nothing about what reached the learner — that is the
`rendered-output-contracts` standard both RC epics already cite. Assert the string a human sees.

**Should-fail.** Plant a state where due == 0 and misses == 0 and assert the card is not blank and
names an action. Plant due == 3 and assert the label contains "3". A card that renders identically
across all three states must go RED.

---

## D-3 · Early submit (F-06) — always enabled, gaps named, result marked partial

**Call:** Submit is **always enabled**. Submitting with unanswered items requires a confirmation
that names the gaps. A partial attempt is scored, labelled partial, and **never feeds mastery**.

**Reasoning.**

Blocking submit at 36/40 is paternalistic and, worse, it is a dead end: the learner is told
"Submit unlocks when every…" and given no way to find the four missing items. The control refuses
the action and withholds the information needed to satisfy it.

But an incomplete attempt cannot be presented as comparable to a complete one, because this
project's entire identity is *a score is a study signal, not a pass mark*
(`claim-study-signal-27`). So: allow the action, and be exact about what the result means.

- Submit label carries state: **"Submit · 40 of 40"** / **"Submit · 36 of 40 — 4 unanswered"**.
- Clicking with gaps opens a confirmation naming the count **and listing the item numbers as jump
  links**: *"Items 12, 19, 27, 33 are unanswered. Review them, or submit as-is."*
- **This one control also fixes F-05** — the 98.9 ms no-confirmation submit. A sixty-minute exam
  should not end by accident. One change, two findings, and the confirmation is *informative*
  rather than a speed bump, because it carries the answer to "which four?".
- **F-09 rides along:** Submit moves into reach in tab order. Today it sits behind Next and all
  forty jump buttons at 40/40 — the primary action of the screen is last.
- Results for a partial attempt are marked partial: **"27/40 · 4 unanswered"**. Unanswered items
  score as incorrect but are counted separately, because "got it wrong" and "never saw it" are
  different facts and merging them destroys the one signal the learner needs.
- **A partial attempt does not move module mastery.** `PRACTICED_MILLI = 800`
  (`crates/cdcp_schedule/src/lib.rs:27`) means ≥80% on a module quiz. An attempt with unanswered
  items cannot support that claim, so it must not be admissible evidence for it.

**Registered.** Mastery's input contract becomes a claim row: *only complete attempts are
admissible evidence for practiced/mastered*. `cdcp_schedule` already guards its thresholds with
const asserts (`:71-75`); the completeness precondition belongs at the same level, so no later
change can quietly let partials count.

**Should-fail.** Feed a planted partial attempt scoring 90% into the mastery path; the bar must not
move and the gate must assert it did not. Bypass the completeness check and the fixture must fail.

---

## D-4 · The legacy footer Drill link — remove it

**Call:** Remove. Results ships **exactly one** Drill route.

**Reasoning.**

The receipt asks whether it "should remain as a secondary route." It was never a route. It is where
the content happened to end. Pane 2 measured the situation precisely: the new recovery CTA sits at
**y = 1,008** at 1280×900 and the legacy footer link at **y = 10,453** — two routes to the same
destination, 9,400 px apart, one of which nobody chose.

It existed as compensation for recovery being unreachable, and recovery is now reachable. Keeping it
re-creates in miniature the exact defect D-2 removes from the hub: several links, one behaviour, no
hierarchy. Removing it is the same consolidation.

**Should-fail.** `smoke_rendered_output.js` asserts exactly one Drill route on `results.html`. Plant
a second and it goes RED.

---

## What these four do not decide

They resolve the exam loop's blocked questions. They do **not** touch the two findings that outrank
everything here:

- **F-01** — the key is the only plausible option, 31 of 117 findings, live in Form A item 1. That
  is a content programme gated behind the W1a/W1b measurements, and nothing in this record unblocks
  it.
- **F-04** — the teaching/test mismatch rate, still the most important unmeasured number in the
  project.

A perfect exam loop delivering items a candidate can answer by eliminating three absolutes is still
a tool that does not teach. These decisions make the surface honest about what it is doing. They do
not make the questions good.
