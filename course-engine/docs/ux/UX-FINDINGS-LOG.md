# UX findings log — learner surface

**Consumer:** the de-duplication pass, then the fix beads it produces.
**Gate:** operator will not move on until 100–500 observed runs are logged and de-duplicated.
**Deletion condition:** retires when every surviving finding is either a closed bead or an
explicitly declined entry with a reason. This file is not a permanent artifact.

**Scope note.** `ui-ux-pro-max` is written for app UI (iOS/Android/React Native). This is a
local-first static HTML + WASM surface. Contrast, spacing rhythm, hit-target size, motion
timing, focus order and a11y transfer directly. Safe-area insets, bottom-nav limits, Phosphor/RN
component guidance and Dynamic Type do **not** — do not log findings that only exist because a
phone rule was applied to a desktop page.

## Entry format — one line per observation, append only

```
| id | surface | viewport | mode | severity | finding | why it matters | suggested change |
```

- **id** — `<pane>-<nnn>`, e.g. `p2-014`. Unique, never reused.
- **surface** — `hub` `mock` `quiz` `drill` `results` `learn` `learn/<module>` `reference` `runbooks` `diagram`
- **viewport** — `375` `768` `1280` `1920`, or `n/a`
- **mode** — `mouse` `keyboard` `dark` `light` `reduced-motion` `closed-notes` `first-run` `returning` `post-fail`
- **severity** — `blocker` (cannot complete the task) · `major` (completes but wrong/painful) ·
  `minor` (polish) · `question` (needs a human decision, not a defect)
- **finding** — what you observed, concretely. Not "improve the header".
- **why it matters** — the learner consequence. If you cannot name one, do not log it.
- **suggested change** — one specific change. "Make it nicer" is not a suggestion.

## Rules

1. **Observe, do not speculate.** Log what the page did, at a URL, at a viewport. If you did not
   load it, do not log it.
2. **One finding per line.** A line containing "and also" is two findings.
3. **No duplicates within your own pane** — check your existing ids before appending.
4. Cross-pane duplicates are expected and get merged in the de-dup pass. Do not coordinate to
   avoid them; independent rediscovery is signal about severity.
5. **Content defects count.** If an item reads badly, is ambiguous, or has an obviously-wrong
   option, log it — the exam experience includes the questions.
6. **Log what is GOOD too**, marked `severity=keep`. A redesign that discards a working thing is
   a regression, and we will not remember what worked.

## Seeded findings (controller, 2026-08-21)

| id | surface | viewport | mode | severity | finding | why it matters | suggested change |
|---|---|---|---|---|---|---|---|
| c-001 | hub | 1280 | first-run | minor | `GET /favicon.ico` 404s on every page load; no favicon is set | The browser tab is how a learner finds this among a dozen tabs during a study session; a blank tab icon reads as unfinished | Ship a favicon; it is one file and one link tag |
| c-002 | hub | 1280 | first-run | major | The h1 lead paragraph is operating instructions: "Start with cdcp serve and open http://127.0.0.1:8766/. Double-clicking this file (file://) is not supported and fails as CDCP_FILE_ORIGIN" | The learner is already served and already here — they are reading instructions for reaching a place they have reached. `CDCP_FILE_ORIGIN` is an internal error code shown to a learner | Move the file:// guidance to the error state that actually fires it; lead with what to do next |
| c-003 | mock | 1280 | first-run | major | Pre-exam instruction paragraph carries six distinct facts in one block: key bindings, timer semantics, submit behaviour, tab persistence, seed identity, closed-notes | Highest cognitive load lands immediately before a 60-minute timed task; a first-timer cannot absorb it and a returning learner does not need it | Split: keys into a dismissible shortcut hint, timer/submit semantics into the timer tooltip, seed into the seed control |
| c-004 | mock | 1280 | first-run | minor | Header line is `40Q · 60:00 · study bar 27/40 · study signal / not a pass mark` — four facts middot-separated | Dense middot lists do not scan; the one number that matters during the exam (time left) has equal weight to a disclaimer | Give the timer its own visual weight; demote the disclaimer to the footer where it already appears |
| c-005 | mock | 1280 | first-run | major | The Seed control is a `<select>` containing exactly one option, "42" | A control that looks interactive and offers no choice is a dead end; a learner who wants a different exam finds a dropdown that cannot do it | Either populate real seed choices or render it as static text, not a combobox |
| c-006 | mock | 1280 | first-run | minor | Item header exposes the internal id: "Item 1 of 40 · m01-q201" | Developer data in the learner surface; it means nothing to a candidate | Keep the id but demote it into a "report this question" affordance, which also gives us a defect channel |
| c-007 | mock | 1280 | first-run | major | Option lengths within one item are wildly uneven — A is 5 words, C is 15 ("Lightning strike with no process involvement, no equipment configuration error, and no organizational contribution") | Uneven options do not scan as a set, and an over-qualified distractor reads as wrong on sight regardless of knowledge | Content fix, not CSS: this is the construction-fault work reaching the surface. Log the item ids |
| c-008 | hub | 1280 | first-run | question | "Drill" appears as three separate cards — Session · Drill-10, Practice · Drill, Session · Miss-review — plus a Drill nav item | A learner cannot tell which drill they want without reading three descriptions | Decide whether these are one surface with modes or three surfaces, and name them accordingly |
| c-009 | hub | 1280 | first-run | question | Card copy says "14 curriculum modules + ops expansions"; the corpus has 15 module files | Either the count is stale or m15 is deliberately excluded as an "ops expansion" — a learner counting modules will find the mismatch | Verify and state the real number |
| c-010 | hub | 1280 | first-run | keep | "Skip to main content" link is present and first in tab order | This is correct a11y that a redesign would plausibly discard | Keep it; verify it survives any header rework |
| c-011 | hub | 1280 | first-run | keep | Empty state for "Now · 90 seconds" reads "No cards due. Take a mock or quiz, then come back for a 90-second loop." | Names the next action instead of showing a blank panel — a genuinely good empty state | Keep this pattern and reuse it for other empty surfaces |
