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
| p3-001 | learn/06 | 375 | keyboard | major | The generated "On this page" TOC clips the long entry "Interconnect queue and behind-the-meter as a primary path" at the right edge. | A learner cannot read the full section name on the narrowest supported device. | Constrain the mobile TOC and make long entries wrap or scroll with an obvious affordance. |
| p3-002 | learn/07 | 375 | keyboard | major | The generated "On this page" TOC clips the long "Further free resources (public standards names, vendor ...)" entry at the right edge. | A learner cannot read the full destination before activating the link. | Make every mobile TOC label fully reachable and verify long labels at 375px. |
| p3-003 | learn/06 | 375 | keyboard | major | The visible shell h2 "Learning objectives" precedes the article h1 "Power Infrastructure" in the DOM heading order. | Screen-reader heading navigation reaches a subsection before the module identity. | Render the module h1 before the shell h2 or make the shell title a non-heading status element. |
| p3-004 | learn/07 | 375 | keyboard | major | The visible shell h2 "Learning objectives" precedes the article h1 "Electro Magnetic Fields (EMF)" in the DOM heading order. | Screen-reader heading navigation reaches a subsection before the module identity. | Render the module h1 before the shell h2 or make the shell title a non-heading status element. |
| p3-005 | learn/11 | 375 | keyboard | major | The visible shell h2 "Learning objectives" precedes the article h1 "Scalable Network and Cabling Infrastructure" in the DOM heading order. | Screen-reader heading navigation reaches a subsection before the module identity. | Render the module h1 before the shell h2 or make the shell title a non-heading status element. |
| p3-006 | reference | 375 | keyboard | major | The rendered "How to use" link points to `../practice/DRILL-CARDS.md`, which returns HTTP 404 from the running learner server. | A learner following the study instruction reaches a dead end before practice. | Replace it with a served learner URL or remove the link and check rendered reference links. |
| p3-007 | reference | 375 | keyboard | major | The glossary table extends beyond the 375px reading width; its Definition column is off-screen without a visible horizontal-scroll cue. | Key explanations require panning and a learner may miss the definition column. | Stack term and definition on mobile or provide a labelled scroll affordance. |
| p3-008 | diagrams/power-path | 375 | keyboard | major | The interactive power-path nodes render as `?` buttons with no aria-label or title; their accessible name is only "?". | A keyboard or screen-reader learner cannot identify a focused node before activating it. | Give each node an accessible name derived from its target, such as "Reveal Utility feed node". |
| p3-009 | runbooks | 375 | keyboard | major | The cross-page link rail under the introduction runs past the narrow viewport, ending visibly at `... Module quiz · M...`. | The learner may not be able to reach the later destinations on mobile. | Convert the rail to a wrapping link list with each destination label fully visible. |
| p3-010 | runbooks | 375 | keyboard | major | The long-form vignette introduction and first stem are clipped at the right edge instead of showing the complete sentence in the reading column. | Scenario context can be missing before the learner chooses where to practice. | Enforce text containment and overflow wrapping on vignette content and verify at 375px. |
| p3-011 | reference | 375 | content | major | The rendered glossary says "this program's 14 modules" while the learner hub exposes 15 module pages from 01 through 15. | The reference teaches a stale corpus count and undermines trust in the study map. | Derive the count from the module index or remove the hard-coded number. |
| p3-012 | cross-cutting | 375/768/1280/1920 | dark | question | The learner surface declares `color-scheme: dark` and has no light-theme override. | Learners who need a light palette cannot choose one, and the light-mode contrast case cannot be established. | Decide whether dark-only is intentional; otherwise add a tested light theme and contrast contract. |
| p3-013 | cross-cutting | 375/768/1280/1920 | reduced-motion | keep | The reduced-motion rule collapses animation and transition durations, disables smooth scrolling, and removes skip-link transition. | Motion-sensitive and keyboard learners are not forced through the default animated transitions. | Preserve this rule and add a regression check when new motion is introduced. |
| p3-014 | cross-cutting | 375/768/1280/1920 | dark | keep | The observed dark-theme token ratios against `#060b09` are 13.35:1 body, 7.73:1 secondary, 5.18:1 faint-caption, 11.87:1 amber, and 10.31:1 accent. | The shipped dark palette clears WCAG AA contrast for the observed text hierarchy. | Keep these ratios when palette tokens change. |
| p3-015 | diagrams/power-path | 375/1280 | keyboard | keep | The power-path self-check radios use fieldsets with legends and each radio is wrapped in a text-bearing label. | A keyboard learner can identify each question and option without relying on layout. | Preserve the fieldset, legend, and label structure. |
| p3-016 | learn | 1280/1920 | dark | keep | At 1280 and 1920 the long-form modules keep a separate TOC and a constrained prose column instead of expanding across the full viewport. | Reading line length stays usable on wide monitors. | Preserve the max-width and two-column reading layout. |
