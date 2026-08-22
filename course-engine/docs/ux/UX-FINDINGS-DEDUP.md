# UX findings — de-duplicated

**Source:** `docs/ux/UX-FINDINGS-LOG.md`, frozen at **117 raw findings** (controller 11, pane 2 ~62,
pane 3 ~44) after both panes were told to hold so the file would stop moving mid-read.

**Denominators, predeclared:** 117 raw → **59 distinct findings**. 0 blocker · 34 major → 23 after
merge · 5 minor · 5 question · 34 keep (kept whole; a keep is a measurement, not an opinion, and
merging them would destroy the evidence).

---

## The headline: one defect accounts for 31 of 117 findings

**F-01 · The key is the only plausible option.** `severity: major` · **31 raw findings merged**
· surfaces: mock, quiz · item ids logged: 30+

Both panes, independently, kept writing the same sentence in different words:

> `m06-q231` makes A, B, and C absolute claims (`zero need`, `immunity from all`, `automatic 2N`) while D is the only…
> `m10-q217` gives A, C, and D universal wording (`regardless of site`, `every closed loop`, `every data centre`)…
> `m03-q235` makes A and B deny any distinction while C invents a mandatory assessor signature; D is the only…
> `m06-q090` … A, B, and D are impossible consequences…
> `bank-m14-q124` … A, B, and C are visibly absurd operator-log parodies…

**A candidate answers these by eliminating the three that sound wrong.** No data-centre knowledge
required — the distractors are absolutes, impossibilities, or off-topic, and the key is the only
sentence a professional would ever say.

**This cue survives every gate we own.** We measured and drove to chance: length rank, key
position, hedged-option, avoid-hedge, stem overlap, all-of-the-above, grammatical agreement. We
never measured *"the key is the only non-absolute option"* — and it is plausibly the strongest cue
of all, because it needs no counting, no comparison, and no test-wiseness beyond ordinary reading.

It is also the same defect as the cartoon-distractor work from 2026-08-20, which we believed
finished. It was measured by proxy (option length) and the proxy went to chance while the defect
stayed. **This is the first time anyone read the items in the surface a learner uses.**

> Suggested change: this is a content programme, not a CSS fix. Before any rewriting, build the
> detector — an option-set where exactly one option lacks an absolute/universal/impossible marker
> is the signal — and measure the real rate across all 957 items through the assembler, so we get
> a predeclared denominator instead of two panes' sampling.

---

## Consolidated findings

### Content (the exam itself)

| id | merged | sev | finding | change |
|---|---|---|---|---|
| F-01 | 31 | major | Key is the only plausible option; distractors are absolutes/absurdities | Build the detector, measure the real rate, then a content wave |
| F-02 | 5 | major | Option lengths wildly uneven within an item — `m11-q237` keys 37 words against 15/15/15; `m06-q255` keys 37 against 6–9 | Length parity is a *separate* axis from plausibility; both must hold |
| F-03 | 1 | major | `m01-q046` (edge/micro DC) — distractors are `diesel storage yard`, `replace…` — instance of F-01 but in **quiz**, confirming it is not mock-specific | Include quiz in whatever wave F-01 produces |
| F-04 | 2 | major | Teaching/test mismatch: `m10-q300` demands an applied leak sequence the m10 lesson never teaches; `m15-q385` tests OSHA 1904.39 24-hour amputation reporting absent from the m15 lesson | **A learner cannot pass by studying the material we shipped.** Highest-value class in the campaign |

### The exam loop

| id | merged | sev | finding | change |
|---|---|---|---|---|
| F-05 | 2 | major | Submit fires with **no confirmation** — mock navigated to Results in 98.9 ms; quiz graded directly | Confirm, and state the unanswered count before the point of no return |
| F-06 | 1 | major | Submit stays **disabled** at 36/40 with only "Submit unlocks when every…" — a learner who wants to submit early cannot, and is not told which four are missing | Allow submit with a warning; name the gaps |
| F-07 | 1 | major | **No flag / review-later control** at any question | Every real exam has one; without it the jump grid is the only recovery and it does not mark uncertainty |
| F-08 | 1 | major | Q37 header shows only `37 / 40` and `36 of 40 answered` — no sense of remaining, skipped, or nearly-done | Progress that answers "how much is left and what did I skip" |
| F-09 | 1 | major | Tab order at 40/40 walks Next **and all 40 jump buttons** before reaching Submit | Move Submit into reach; it is the primary action of the screen |
| F-10 | 1 | major | Results at 6/40 renders a **15,233 px** page; "Open Drill" sits at y=15,152 after a 13,515 px review section | The recovery action is unreachable at exactly the moment a struggling learner needs it |
| F-11 | 1 | major | Weak modules lists bare `M01`–`M15` with no miss count, rate, or priority | The one screen that should say *what to study next* says nothing |
| F-12 | 1 | major | Mock reopened after submit shows all 40 answers and Submit still enabled | Ambiguous whether a graded attempt can be resubmitted |
| F-13 | 1 | major | Quiz pre-copy says "approved pool 140 of 146 in bank"; the quiz serves 12 | Internal pool arithmetic shown to a learner, and it does not describe what they are about to take |
| F-14 | 1 | major | 1/12 quiz result says "Missed items sent to Drill" without stating the missed count | Say the number |

### Structure and a11y

| id | merged | sev | finding | change |
|---|---|---|---|---|
| F-15 | 3+1 | major | Shell `h2` "Learning objectives" precedes article `h1` on **all 15** module pages (pane 3 generalised it from 3 instances) | Systemic, one fix |
| F-16 | 2 | major | "On this page" TOC clips long entries at 375 px (`learn/06`, `learn/07`) | Determine systemic vs long-label; constrain the mobile TOC |
| F-17 | 1 | major | `reference` exposes **two `h1`s**: `Reference` then `CDCP Self-Study Glossary` | One h1 per document |
| F-18 | 1 | major | Power-path diagram nodes are `?` buttons whose accessible name is only "?" | Real labels; a screen-reader user gets nothing |
| F-19 | 3 | major | Narrow-viewport overflow: glossary Definition column off-screen at 375; runbooks link rail and vignette clipped | One overflow strategy, applied |
| F-20 | 1 | major | Runbooks: six scenarios are list items with **no headings** | A learner cannot navigate or link to a scenario |
| F-21 | 1 | major | `reference` "How to use" links `../practice/DRILL-CARDS.md` → **404** | Dead link on a shipped page |

### Copy and framing

| id | merged | sev | finding | change |
|---|---|---|---|---|
| F-22 | 2 | question | Module count stated as **14** in hub card and glossary; **15** module pages ship | Establish the true number and say it once |
| F-23 | 2 | major | Instruction density: hub leads with `cdcp serve` operating instructions incl. `CDCP_FILE_ORIGIN`; mock front-loads six facts before a 60-min timer | Move operational text to the error states that fire it; progressive disclosure before the exam |
| F-24 | 1 | minor | Mock header is four middot-separated facts; the timer has no more weight than the disclaimer | Give the timer visual primacy |
| F-25 | 1 | major | Seed `<select>` has exactly one option, `42` | Populate or render as text — and note it caps how many distinct exams anyone can practise on |
| F-26 | 1 | minor | Item id `m01-q201` shown in the learner header | Demote into a "report this question" affordance, which also buys us a defect channel |
| F-27 | 1 | minor | `favicon.ico` 404s on every page load | Ship one |
| F-28 | 1 | question | "Drill" appears as three hub cards plus a nav item | Decide: one surface with modes, or three surfaces with distinct names |

---

## What we keep — 34 measured, unmerged

The redesign must not regress these. Highlights:

- **Dark-theme contrast measured**, not assumed: 13.35:1 body, 7.73:1 secondary, 5.18:1 faint
  caption, 11.87:1 amber, 10.31:1 accent against `#060b09`.
- **Every advertised key binding works** — A–D, 1–4, ArrowLeft/Right, P/N, verified in a fresh mock.
- **375 px has no horizontal overflow** on mock: answer cards, the 1–40 jump grid, and submit all fit.
- **Reduced-motion is honoured** — durations collapse, smooth scrolling off, skip-link transition removed.
- **Prose column stays constrained at 1280 and 1920** instead of running edge-to-edge.
- **Skip-to-main-content link** present and first in tab order.
- **Diagram self-checks use fieldset/legend with text-bearing labels.**
- **"No cards due" empty state names the next action** rather than showing a blank panel.

---

## Honest coverage gaps

The corpus is **uneven and I am not going to present it as even**:

| surface | findings |
|---|---|
| mock | 45 |
| quiz | 23 |
| results | 9 |
| hub | 8 |
| reference | 5 |
| runbooks | 3 |
| **all 15 learn modules combined** | **~12** |

A learner spends more time in the learn modules than in the exam, and they are the least observed
surface. F-04 (teaching/test mismatch) came out of that thin sample — **two mismatches from a
partial pass across a handful of modules.** The true rate is unknown and is probably the most
important unmeasured number we have.

Also unobserved: dark mode on the exam surfaces specifically, tablet (768) on learn, and any
session longer than a single sitting.
