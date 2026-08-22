# Wave 3, pane 3 — accumulated state and the dashboard bar

This is a differential read of six assigned artifacts against the current
Results, Review, and hub-mastery surfaces. The corpus contains both stateful
interfaces and negative controls: some artifacts make accumulated state,
recency, provenance, or recovery explicit; `llm-docs` is documentation rather
than a learner dashboard. These decisions are evidence of what each source
chose. They do not prove that a particular dashboard improves CDCP exam
outcomes.

## Findings

### G-33-01 · A returning visitor sees a handback, not just a current snapshot

- **Seen in:** `asimposium.org/site/design.html:638-644` — the orientation pack carries the live hypothesis/claim counts, synthesis staleness, the last five material events, the latest handback, dead-end headlines, and warnings.
- **What it decides:** A return visit should answer “what changed since I was here?” before asking the visitor to choose another action. Accumulated state is a handback and delta, not merely a current total.
- **What ours does:** `course-engine/web/results.html:46-58` shows the current attempt’s exam, seed, bank hash, answer count, engine, and storage key, but no prior-attempt comparison, last-seen time, or changed-since-last-visit summary. `course-engine/web/assets/js/results.js:561-568` populates those identity fields immediately, so this is a real current snapshot with a missing return delta.
- **Class:** STATE
- **Transferable under our constraints?** YES — a local prior-attempt summary and timestamp can be read from existing storage without a service.
- **Cost:** M (< 1d)
- **Regression risk:** KEEP: honesty banner remains claim-not-credential; KEEP: dark-theme contrast measured; KEEP: 375 px has no horizontal overflow.

### G-33-02 · The next action is weighted by impact, not merely the first weak row

- **Seen in:** `asimposium.org/site/design.html:665-693` — an arriving fellow gets one primary move with a reason, target, and contract; the problems index sorts open moves by aggregated weight and can trigger moves such as closing a gap or returning to an object.
- **What it decides:** “What next?” should be a computed priority whose reason is legible, not an arbitrary first item in a catalog.
- **What ours does:** `course-engine/web/assets/js/hub_mastery.js:339-378` chooses a weak module, then the first unpracticed module, then the first unmastered module in registry order; `course-engine/web/assets/js/hub_mastery.js:467-483` displays the label and reason. The route is now computed and honest, but a weak module’s miss count/rate severity does not change the priority within the weak list.
- **Class:** STATE
- **Transferable under our constraints?** YES — local module statistics can provide a deterministic priority and a short reason.
- **Cost:** L (needs a design decision a human must make)
- **Regression risk:** KEEP: “No cards due” names the next action for returners; KEEP: mastery thresholds are measured by the Rust/WASM law; KEEP: honesty banner is load-bearing.

### G-33-03 · Bad or dead work remains visible as a recoverable state

- **Seen in:** `asimposium.org/site/design.html:625-644` — the pack includes omitted items and a graveyard of dead ends/killed hypotheses with retry predicates, rather than silently dropping work that failed or was ruled out.
- **What it decides:** A bad state should preserve what failed and the condition for trying again. It should not become an empty dashboard after the learner has invested effort.
- **What ours does:** `course-engine/web/results.html:80-90` gives a scored attempt a missed-item and weak-module route, and `course-engine/web/assets/js/results.js:415-435` computes those links. The Results path does not retain a visible “previously tried / retry when” history for resolved or repeatedly missed work; that state lives in the Drill storage rather than the dashboard.
- **Class:** RECOVERY
- **Transferable under our constraints?** YES — a small local history or retry reason can be rendered from existing missed-item and mastery records.
- **Cost:** M (< 1d)
- **Regression risk:** KEEP: score failure withholds invented numbers; KEEP: honesty banner remains claim-not-credential; KEEP: 375 px has no horizontal overflow.

### G-33-04 · A large state set is budgeted before it becomes a wall

- **Seen in:** `asimposium.org/site/design.html:625-649` — the projection carries a budget, cursor/ETag/hash, and an omitted list; the interface explicitly distinguishes what is present from what was left out.
- **What it decides:** When accumulated state is large, the interface must preserve navigability and explain omission instead of dumping every record into one vertical surface.
- **What ours does:** `course-engine/web/assets/js/results.js:437-508` builds one ordered item-review list for every graded row, while `course-engine/web/results.html:92-94` provides a single results section with no pagination, collapse boundary, or count-based grouping. The local review surface therefore makes the 40-item result wall possible even though its weak-module summary is compact.
- **Class:** DENSITY
- **Transferable under our constraints?** YES — a local “missed first / expand all / item count” presentation can be implemented with the existing DOM and CSS.
- **Cost:** M (< 1d)
- **Regression risk:** KEEP: 375 px has no horizontal overflow; KEEP: prose column stays constrained; KEEP: reduced-motion is honoured.

### G-33-05 · Provenance includes version and access recency

- **Seen in:** `asimposium.org/site/design.html:945-967` — stable citable URLs carry version/access date, and orientation exposes claim position, review time, notices, and the latest handback.
- **What it decides:** A state display should answer “which pack produced this?” and “when should I trust this as current?” Provenance is part of the result, not metadata hidden from the learner.
- **What ours does:** `course-engine/web/results.html:46-58` exposes exam, seed, bank hash, answers, and engine, but no attempt timestamp or freshness label. `course-engine/web/assets/js/review.js:405-420` already persists `saved_at` for missed items, yet the learner-facing Results summary does not surface it.
- **Class:** STATE
- **Transferable under our constraints?** YES — local timestamps, seed, and bank hash are already available; the source’s collaborative review metadata is not required.
- **Cost:** S (< 1h)
- **Regression risk:** KEEP: bank hash is exposed for integrity; KEEP: dark-theme contrast measured; KEEP: honesty banner remains claim-not-credential.

### G-33-06 · An operationally bad state offers retry and diagnostics at the failure point

- **Seen in:** `markdown_web_browser/docs/BROWSER_UI.md:54-73` — loading and capture progress are visible, and a failure gives a clear error, a “Try Again” action, and status details rather than leaving the user at a dead screen.
- **What it decides:** A bad state is actionable when the user can see what failed, retry the relevant operation, and inspect enough detail to choose the next move.
- **What ours does:** `course-engine/web/assets/js/results.js:646-663` correctly fails closed when WASM grading fails and withholds scores, but its diagnostic points to a build command/CLI oracle and returns without a Results retry action. This is honest safety behavior, but weak recovery for a learner who already submitted.
- **Class:** RECOVERY
- **Transferable under our constraints?** DEGRADED — local retry and concise diagnostics transfer; the source’s asynchronous capture job and backend status machine do not fit the shipped static/offline path.
- **Cost:** M (< 1d)
- **Regression risk:** KEEP: no invented scores on grader failure; KEEP: fully offline/no-network runtime; KEEP: honesty banner remains load-bearing.

### G-33-07 · History, cache age, and view mode keep a long state surface navigable

- **Seen in:** `markdown_web_browser/docs/BROWSER_UI.md:122-145` — the state model names history stack/current index, cache entries with timestamps, and view state; the UI deliberately separates simple browser status from a detailed state-machine/dashboard view.
- **What it decides:** A large state surface needs a compact current view plus a deliberate place for history and diagnostics; those states should not all compete in the primary reading column.
- **What ours does:** `course-engine/web/results.html:92-100` places Weak modules, Item review, and navigation links in one linear page, while `course-engine/web/assets/js/results.js:437-508` emits every item into the same ordered list. There is no separate compact/current versus detailed/history mode.
- **Class:** DENSITY
- **Transferable under our constraints?** YES — two disclosure levels can be static HTML plus vanilla JS; no browser backend is needed.
- **Cost:** L (needs a design decision a human must make)
- **Regression risk:** KEEP: 375 px has no horizontal overflow; KEEP: skip link remains first in tab order; KEEP: reduced-motion is honoured.

### G-33-08 · Recency is useful only when it is learner-visible

- **Seen in:** `markdown_web_browser/docs/BROWSER_UI.md:173-176` — the first visit captures fully, repeat visits use cached output immediately, and forced refresh explicitly clears and recaptures; the state change is temporal and user-visible.
- **What it decides:** “Current” and “stale” should be an explicit choice for the returning reader, not an implementation detail.
- **What ours does:** `NOTHING — no code exists for this question` on the learner-visible Results and hub surfaces: `course-engine/web/assets/js/review.js:405-420` stores `saved_at`, and `course-engine/web/assets/js/mastery.js:258-264` stores `at_ms`, but no current UI line renders “measured at” or “stale since.” The owning surface is Results plus Module mastery.
- **Class:** STATE
- **Transferable under our constraints?** YES — a local timestamp and “current pack” label fit the existing offline storage model.
- **Cost:** S (< 1h)
- **Regression risk:** KEEP: local storage remains offline; KEEP: bank hash and seed remain visible; KEEP: honesty banner remains claim-not-credential.

### G-33-09 · A catalog gives the reader filters and a result count before the wall

- **Seen in:** `classic-patents.com/src/app/page.tsx:107-145` — the catalog names its total, offers category and search controls, reports the filtered result count, and gives a specific recovery instruction when no items match.
- **What it decides:** Before showing many accumulated items, let the reader narrow the set and explain an empty filter result.
- **What ours does:** `course-engine/web/assets/js/hub_mastery.js:517-562` emits all curriculum modules into one mastery grid with badges and Open/Quiz links; it has no filter, compact status count, or “show only weak/unmastered” view. `course-engine/web/index.html:121-129` names the thresholds but not a current filtered count.
- **Class:** HIERARCHY
- **Transferable under our constraints?** YES — module filters and counts can be computed from local mastery state with vanilla DOM controls.
- **Cost:** M (< 1d)
- **Regression risk:** KEEP: 375 px has no horizontal overflow; KEEP: dark-theme contrast measured; KEEP: keyboard/focus affordances remain visible.

### G-33-10 · Rich simulation is not an admissible substitute for compact mastery state

- **Seen in:** `classic-patents.com/src/app/page.tsx:149-161` — the product promises preserved source material alongside equations, real-time 3D WebGL physical simulations, and interactive parameter dials.
- **What it decides:** A catalog item can expose a manipulable model rather than only a static description, but this source chooses a framework-heavy simulation surface.
- **What ours does:** `course-engine/web/index.html:121-129` uses a static mastery shell with local threshold copy, and `course-engine/web/assets/js/hub_mastery.js:522-562` renders deterministic text badges and links rather than a 3D state canvas.
- **Class:** AFFORDANCE
- **Transferable under our constraints?** NO — importing this implementation would violate the static HTML + vanilla JS + one CSS file constraint and introduce the source’s React/Next/Three.js-style application surface; the shipped runtime must remain fully offline with no npm/build dependency.
- **Cost:** L (needs a design decision a human must make)
- **Regression risk:** KEEP: fully offline/no CDN/no build step; KEEP: reduced-motion is honoured; KEEP: dark-theme contrast measured.

### G-33-11 · A long document exposes both position and the current section

- **Seen in:** `eidetic-engine-website-project/src/EideticEngineWebsite.tsx:23-39` and `:82-91` — the page defines named section navigation and computes a persistent scroll-progress fraction; `:121-147` updates the active section as the reader moves.
- **What it decides:** Long content is less wall-like when the reader can see both how far they are through it and which section currently owns their attention.
- **What ours does:** `course-engine/web/results.html:92-100` has section headings and end navigation but no results-progress or active-section indicator; `course-engine/web/assets/js/results.js:437-508` renders the complete item sequence without a current-position signal.
- **Class:** HIERARCHY
- **Transferable under our constraints?** YES — a compact results progress label and active section can use existing headings and vanilla scroll observers.
- **Cost:** M (< 1d)
- **Regression risk:** KEEP: reduced-motion is honoured; KEEP: prose column stays constrained; KEEP: 375 px has no horizontal overflow.

### G-33-12 · Decorative motion must not become a dependency of state comprehension

- **Seen in:** `eidetic-engine-website-project/src/MemoryGraph.tsx:1-3` imports React and anime.js, and `:64-83` starts a timed animation timeline; `:126-136` adds continuous looping pulses.
- **What it decides:** The artifact uses animation to make relationships feel alive, including continuous motion around a state graph.
- **What ours does:** `course-engine/web/assets/css/course.css:1141-1155` collapses animation and transitions under `prefers-reduced-motion: reduce`; the mastery/results surfaces use textual state and links, so comprehension does not depend on an animation completing.
- **Class:** MOTION
- **Transferable under our constraints?** NO — the source’s React/anime.js/npm implementation violates our vanilla/offline/no-build constraint, and its continuous loops are not a safe motion baseline for our measured reduced-motion contract. The transferable lesson is to make state legible without motion.
- **Cost:** S (< 1h)
- **Regression risk:** KEEP: reduced-motion is honoured; KEEP: dark-theme contrast measured; KEEP: no CDN at runtime.

### G-33-13 · Metadata can make content recency explicit, but remote auto-update is not our state model

- **Seen in:** `nextjs-github-markdown-blog/README.md:125-140` — each post carries date, category, tags, author, and source metadata; `:150-158` says the repository is version-controlled and the system periodically checks for new Markdown files.
- **What it decides:** A reader can orient to what an item is and how current its source is when date and authorship are part of the item presentation.
- **What ours does:** `course-engine/web/results.html:46-58` identifies the exam and bank hash but does not display a measured-at date, content revision, or author/source provenance beside the learner’s result.
- **Class:** COPY
- **Transferable under our constraints?** NO — the source’s periodic GitHub fetch and remote content model violate fully offline/no-network runtime; only the local date/category/provenance fields are candidates for a degraded extraction.
- **Cost:** S (< 1h)
- **Regression risk:** KEEP: fully offline/no network; KEEP: bank hash remains the integrity identifier; KEEP: honesty banner remains claim-not-credential.

### G-33-14 · Documentation scope and versioning are not the same as learner mastery

- **Seen in:** `llm-docs/README.md:51-68` — the repository makes its current implementation and source/condensed-document relationship explicit, while `:106-115` lists version control for timely updates as future work. This is a useful negative control: it has document provenance language, not a learner-state dashboard.
- **What it decides:** Naming the current artifact and its update policy is valuable, but it does not by itself tell a returning learner what changed in their performance.
- **What ours does:** `course-engine/web/index.html:121-129` now exposes the practiced/mastered definitions in the module-mastery shell, and `course-engine/crates/cdcp_schedule/src/lib.rs:266-289` makes the laws precise: practiced is ≥800 milli and mastered requires two ≥900-milli attempts at least one day apart. Our state semantics are stronger than this corpus negative control, but the learner still cannot see when a qualifying attempt occurred.
- **Class:** STATE
- **Transferable under our constraints?** YES — explicit local state semantics and a measured-at field are compatible with the existing WASM/localStorage architecture; importing a documentation-serving website is unnecessary.
- **Cost:** S (< 1h)
- **Regression risk:** KEEP: WASM owns mastery laws; KEEP: honesty banner never claims certification; KEEP: dark-theme contrast measured.

### G-33-15 · A bad 6/40 result should preserve honesty while making recovery concrete

- **Seen in:** `asimposium.org/site/index.html:251-261` — the ledger rejects writable status, treats standing as computed from independent reviews, and puts dead ends/checked nulls first rather than using leaderboards, streaks, or activity meters.
- **What it decides:** A poor state should be factual, non-performative, and oriented toward the next corrective action; a status display must not turn a number into a credential or a competition.
- **What ours does:** `course-engine/web/assets/js/results.js:320-331` labels a 6/40-style outcome as below the practice bar without credential language; `:343-412` orders weak modules with correct/attempted, rate, and missed count; and `:415-435` places missed review and weakest-module actions beside the score. This is a case where our current Results surface does better than the corpus for this specific learner state, although it still lacks the recency/history additions above.
- **Class:** FEEDBACK
- **Transferable under our constraints?** YES — the factual bad-state treatment is already local, deterministic, and compatible with the honesty boundary.
- **Cost:** S (< 1h)
- **Regression risk:** KEEP: honesty banner is load-bearing; KEEP: study signal is never a credential; KEEP: no invented score on grader failure.

## The five required questions

1. **How does the artifact render accumulated state so a returning visitor sees what changed?** `asimposium.org` answers this with a handback, recent material events, warnings, and synthesis staleness (G-33-01); the browser artifact answers it with cache/reload state (G-33-08). Our Results shows the current attempt identity and bank hash, while mastery persists timestamps without rendering them (G-33-01, G-33-05, G-33-08). The dashboard bar therefore remains a current-state snapshot, not a change report.

2. **How is “what to do next” computed and displayed?** The corpus gives one primary move with a reason and weighted priority (G-33-02). Our hub now computes weak → unpracticed → unmastered and displays its reason (G-33-02), which is a meaningful D-2 improvement; what it still lacks is severity-weighted ordering inside the weak set and a visible count of the evidence behind the recommendation.

3. **What does a bad state look like?** For a 6/40 result, ours now shows a below-practice study signal, weak modules ordered by rate/misses, item-level explanations, and a nearby missed-review/weakest-module route (G-33-15). That is materially better than a bare score and preserves the honesty banner. The corpus adds two useful tests: do not erase dead ends (G-33-03), and provide retry diagnostics at the failure point (G-33-06). A bad learner state is not the same as a grader failure; the latter still lacks a local retry action.

4. **How does the artifact handle a lot of items without becoming a wall?** The corpus uses budgets, cursors, omission explanations, filters, and separate compact/detail views (G-33-04, G-33-07, G-33-09). Ours has a compact weak-module summary, but it still emits the full item list and a single linear review section (G-33-04, G-33-07, G-33-11). The measured 13,515px review surface is therefore a density/state-navigation defect, not merely a styling preference.

5. **Does anything show provenance or recency of state?** Yes: stable version/access dates and handbacks in `asimposium.org` (G-33-05), cache timestamps in `markdown_web_browser` (G-33-08), and dated source metadata in the GitHub blog (G-33-13). Our bank hash, seed, engine, `saved_at`, and `at_ms` create the raw ingredients, but no learner-facing current/stale label exists (G-33-05, G-33-08). The schedule’s thresholds define what mastery means; they do not tell the learner when it was measured (G-33-14).

## What this wave did better / did not prove

We do better on the concrete bad-result contract: a 6/40 outcome remains a study signal, exposes weak-module evidence, and routes recovery without green certification treatment (G-33-15). We also do better than the motion-heavy graph source on the shipped accessibility boundary: state remains textual and reduced motion is explicitly honoured (G-33-12). These are strengths to preserve while adding recency, density controls, and history.

Green-does-not-prove: reading six artifacts proves that these interfaces chose handbacks, weighted next moves, filters, provenance, recovery, or motion patterns; it does not establish that those choices improve CDCP exam performance, that a marketing/catalog/documentation surface is a valid study model, or that every missing dashboard feature is worth its implementation and cognitive cost. It also does not establish that the current 6/40 recovery treatment is optimal for every learner.

## Cross-wave: the five things we are most missing

1. **G-32-02 — wrong-answer retry inside the same concept.** It outranks every visual dashboard refinement because the learner currently loses the most valuable correction moment exactly when the misconception is visible.
2. **G-32-03 — evidence-driven next concept.** It outranks broad interactivity because fixed `idx++` makes every later practice decision blind to what the learner just demonstrated.
3. **G-33-01 — a returning-state handback.** It outranks a richer badge vocabulary because a learner cannot act on mastery state they cannot see changing since the last attempt.
4. **G-33-04 — a bounded, non-wall results view.** It outranks additional metrics because a 13,515px review surface hides the corrective evidence before the learner can use it.
5. **G-33-05 — learner-visible provenance and recency.** It outranks decorative polish because without “measured when / against which pack,” a mastery or weak-module signal cannot tell the learner whether it is current enough to trust.
