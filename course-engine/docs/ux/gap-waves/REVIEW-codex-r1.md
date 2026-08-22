# Codex R1 review — fidelity and buildability of PLAN-LEARNER-SURFACE

This is a fidelity/buildability review, not a contrarian review of whether the
plan chose the right problem. I compared the plan at
`course-engine/docs/PLAN-LEARNER-SURFACE.md` with all 40 evidence findings in
`w1-pane3.md`, `w2-pane3.md`, and `w3-pane3.md`, then checked the plan's cited
ranges against the current working tree. The plan contains 13 `S-` story IDs:
11 product stories in P3–P6 and two P0 constitution stories. The “11 stories”
language is therefore accurate only if P0 is excluded.

## 1. FIDELITY — all 40 findings classified

Verdicts mean: **CARRIED** = a named story implements the finding;
**DEFERRED** = the plan names it and consciously postpones it;
**REJECTED** = §4.4 names it with a killing constraint; **LOST** = the plan
does not name or implement it. The plan has no explicit deferred list, so an
omitted finding is LOST rather than being generously called deferred.

**Counts: CARRIED 17 / DEFERRED 0 / REJECTED 8 / LOST 15.**

| G-id | Verdict | Plan story or omission | Evidence and plan citation |
|---|---|---|---|
| G-31-01 | CARRIED | S-P6-1 | `w1-pane3.md:7-15`; `PLAN-LEARNER-SURFACE.md:474-490` |
| G-31-02 | CARRIED | S-P6-2 | `w1-pane3.md:17-25`; `PLAN-LEARNER-SURFACE.md:492-506` |
| G-31-03 | CARRIED | S-P6-2 | `w1-pane3.md:27-35`; `PLAN-LEARNER-SURFACE.md:492-506` |
| G-31-04 | LOST | No story names before/after scenarios | `w1-pane3.md:37-45`; the complete story inventory is `PLAN-LEARNER-SURFACE.md:247-521` and does not name this G-id |
| G-31-05 | LOST | No story names a persona/context route rail | `w1-pane3.md:47-55`; `PLAN-LEARNER-SURFACE.md:167-200` names workflows but no persona rail, and `:247-521` has no implementing story |
| G-31-06 | LOST | No story names a representative inventory or expansion route | `w1-pane3.md:57-65`; `PLAN-LEARNER-SURFACE.md:474-506` keeps the grid/landmark concern but not the compact-inventory decision |
| G-31-07 | LOST | No story names authored audience/provenance context | `w1-pane3.md:67-75`; the plan's evidence inventory and stories `PLAN-LEARNER-SURFACE.md:103-161,247-521` do not carry the audience-context decision |
| G-31-08 | REJECTED | §4.4 rejects Three.js/lazy 3D hero | `w1-pane3.md:77-85`; `PLAN-LEARNER-SURFACE.md:137-145` names offline/no-build as the killer |
| G-31-09 | REJECTED | §4.4 rejects remote OG-image preview/proxy fallback | `w1-pane3.md:87-95`; `PLAN-LEARNER-SURFACE.md:137-146` names no network at runtime |
| G-31-10 | CARRIED | S-P6-1 preserves “No cards due” for returners | `w1-pane3.md:97-105`; `PLAN-LEARNER-SURFACE.md:474-488` explicitly preserves the KEEP branch |
| G-31-11 | CARRIED | S-P6-2 plus invariant honesty boundary | `w1-pane3.md:107-115`; `PLAN-LEARNER-SURFACE.md:78-100,492-506` preserves the claim-not-credential treatment |
| G-32-01 | CARRIED | S-P6-1 | `w2-pane3.md:13-21`; `PLAN-LEARNER-SURFACE.md:474-485` |
| G-32-02 | CARRIED | S-P3-1 | `w2-pane3.md:23-31`; `PLAN-LEARNER-SURFACE.md:315-336` |
| G-32-03 | CARRIED | S-P3-2 | `w2-pane3.md:33-41`; `PLAN-LEARNER-SURFACE.md:338-357` |
| G-32-04 | LOST | No story creates a bounded repetition round/review boundary | `w2-pane3.md:43-51`; P3 stories `PLAN-LEARNER-SURFACE.md:313-376` cover retry, selection, and interaction breadth but not a round boundary |
| G-32-05 | LOST | No story carries analogy/familiar-ground-before-formalism | `w2-pane3.md:53-61`; no corresponding story appears in `PLAN-LEARNER-SURFACE.md:247-521` |
| G-32-06 | CARRIED | S-P3-3 | `w2-pane3.md:63-71`; `PLAN-LEARNER-SURFACE.md:359-376` |
| G-32-07 | LOST | No story carries learner-authored result/hearing feedback | `w2-pane3.md:73-81`; no corresponding story appears in `PLAN-LEARNER-SURFACE.md:313-376` |
| G-32-08 | CARRIED | S-P6-3 | `w2-pane3.md:83-91`; `PLAN-LEARNER-SURFACE.md:508-521` |
| G-32-09 | LOST | No story carries formal/English/intuition repetition | `w2-pane3.md:93-101`; no corresponding story appears in `PLAN-LEARNER-SURFACE.md:313-376,508-521` |
| G-32-10 | LOST | No story carries staged analogy/table/subproblem comparison | `w2-pane3.md:103-111`; no corresponding story appears in `PLAN-LEARNER-SURFACE.md:313-376` |
| G-32-11 | REJECTED | §4.4 rejects Colab handoff | `w2-pane3.md:113-121`; `PLAN-LEARNER-SURFACE.md:137-147` names offline runtime |
| G-32-12 | LOST | No story ties visual progress to narrative/learning progress | `w2-pane3.md:123-131`; `PLAN-LEARNER-SURFACE.md:313-376` does not implement a progress/narrative label |
| G-32-13 | REJECTED | §4.4 rejects unlock/achievement framing | `w2-pane3.md:133-141`; `PLAN-LEARNER-SURFACE.md:137-150` names the credential-boundary constraint |
| G-32-14 | REJECTED | §4.4 rejects CDN-loaded interaction stack | `w2-pane3.md:143-151`; `PLAN-LEARNER-SURFACE.md:137-151` names no CDN and preserves our stronger contract |
| G-33-01 | CARRIED | S-P4-2 | `w3-pane3.md:13-21`; `PLAN-LEARNER-SURFACE.md:401-415` |
| G-33-02 | CARRIED | S-P4-3 | `w3-pane3.md:23-31`; `PLAN-LEARNER-SURFACE.md:417-431` |
| G-33-03 | CARRIED | S-P5-2 | `w3-pane3.md:33-41`; `PLAN-LEARNER-SURFACE.md:455-468` |
| G-33-04 | CARRIED | S-P5-1 | `w3-pane3.md:43-51`; `PLAN-LEARNER-SURFACE.md:437-453` |
| G-33-05 | CARRIED | S-P4-1 | `w3-pane3.md:53-61`; `PLAN-LEARNER-SURFACE.md:382-399` |
| G-33-06 | LOST | No story addresses retry/diagnostics when WASM grading itself fails | `w3-pane3.md:63-71`; S-P5-2 concerns repeated learner misses, not the operational failure described here (`PLAN-LEARNER-SURFACE.md:455-468`) |
| G-33-07 | LOST | No story carries separate compact/detail/history view state | `w3-pane3.md:73-81`; S-P5-1 adds omission/expand-all but not the browser/dashboard view split (`PLAN-LEARNER-SURFACE.md:437-453`) |
| G-33-08 | CARRIED | S-P4-1 | `w3-pane3.md:83-91`; `PLAN-LEARNER-SURFACE.md:382-399` |
| G-33-09 | LOST | No story adds mastery filters/result counts | `w3-pane3.md:93-101`; no hub-filter story appears in `PLAN-LEARNER-SURFACE.md:417-506` |
| G-33-10 | REJECTED | §4.4 rejects React/Three.js mastery simulation | `w3-pane3.md:103-111`; `PLAN-LEARNER-SURFACE.md:137-150` names static HTML/vanilla/one-CSS as the killer |
| G-33-11 | LOST | No story adds results position/active-section navigation | `w3-pane3.md:113-121`; S-P5-1 has height and omission requirements but no active-section contract (`PLAN-LEARNER-SURFACE.md:437-453`) |
| G-33-12 | REJECTED | §4.4 rejects anime.js continuous loops | `w3-pane3.md:123-131`; `PLAN-LEARNER-SURFACE.md:137-151` names vanilla/offline and motion constraints |
| G-33-13 | REJECTED | §4.4 rejects periodic GitHub fetch | `w3-pane3.md:133-141`; `PLAN-LEARNER-SURFACE.md:137-151` names fully offline/no-network |
| G-33-14 | LOST | The plan preserves generic WASM/honesty constraints but never carries the finding’s explicit distinction between mastery semantics and learner-visible recency | `w3-pane3.md:143-151`; no G-33-14 or schedule-threshold story appears in `PLAN-LEARNER-SURFACE.md:247-521` |
| G-33-15 | CARRIED | S-P5-1 plus W4 bad-result workflow | `w3-pane3.md:153-161`; `PLAN-LEARNER-SURFACE.md:188-200,437-468` preserves factual score, recovery, and non-credential language |

The 15 LOST findings are **G-31-04, G-31-05, G-31-06, G-31-07, G-32-04,
G-32-05, G-32-07, G-32-09, G-32-10, G-32-12, G-33-06, G-33-07, G-33-09,
G-33-11, and G-33-14**. They are not consciously deferred: neither the
workflow section nor the story section names a future phase, owner, or reason
for postponement.

## 2. DISTORTION — carried does not mean faithfully carried

The carried set is real, but several stories compress the source observation
more aggressively than the plan admits.

| Finding | Story | Fidelity of the story to the finding |
|---|---|---|
| G-31-01 | S-P6-1 | **Weaker but faithful.** The source combines a promise, a primary/secondary action hierarchy, and a destination; the story keeps the named module action but does not require the promise or secondary-action distinction (`w1-pane3.md:7-15`, `PLAN-LEARNER-SURFACE.md:474-490`). |
| G-31-02 | S-P6-2 | **Faithful.** A group landmark/kicker is the exact transferable decision; the plan correctly adds a density guard (`w1-pane3.md:17-25`, `PLAN-LEARNER-SURFACE.md:492-506`). |
| G-31-03 | S-P6-2 | **Faithful but under-specified.** The source requires named metrics with scope/context; the plan adds denominators and real measurements, but does not name which metric a learner needs (`w1-pane3.md:27-35`, `PLAN-LEARNER-SURFACE.md:492-506`). |
| G-31-10 | S-P6-1 | **Faithful.** It preserves the empty-state strength while correctly separating first-run from return-state use (`w1-pane3.md:97-105`, `PLAN-LEARNER-SURFACE.md:474-488`). |
| G-31-11 | S-P6-2 / §3 | **Faithful and appropriately stronger.** The plan keeps the claim boundary as an invariant rather than treating marketing confidence as a copy goal (`w1-pane3.md:107-115`, `PLAN-LEARNER-SURFACE.md:78-100`). |
| G-32-01 | S-P6-1 | **Weaker.** The source orients the beginner to activity, attention, reward, and round loop before the first prompt; the story asks for a named module action but not the concept-to-attempt handoff or effort expectation (`w2-pane3.md:13-21`, `PLAN-LEARNER-SURFACE.md:474-485`). |
| G-32-02 | S-P3-1 | **Faithful, with a useful extra constraint.** Same-concept retry, explanation, and corrected-after-miss are preserved; the plan should still specify which distinction copy appears (`w2-pane3.md:23-31`, `PLAN-LEARNER-SURFACE.md:315-336`). |
| G-32-03 | S-P3-2 | **Weaker.** The source uses due state, errors, success, recency, and confusable peers; the story requires only different miss profiles and leaves the weighting function unspecified (`w2-pane3.md:33-41`, `PLAN-LEARNER-SURFACE.md:338-357`). |
| G-32-06 | S-P3-3 | **Weaker.** The finding is manipulation → changed model state → causal next step; the story mainly requires propagating placement/reveal/retry and accessible controls. It does not require the new interaction to expose a causal state transition (`w2-pane3.md:63-71`, `PLAN-LEARNER-SURFACE.md:359-376`). |
| G-32-08 | S-P6-3 | **Faithful.** Explicit prerequisite plus permitted skip and preserved skip-link order match the finding (`w2-pane3.md:83-91`, `PLAN-LEARNER-SURFACE.md:508-521`). |
| G-33-01 | S-P4-2 | **Faithful.** Prior comparison, last-seen time, and delta are the handback decision; the first-visit no-baseline guard is a good clarification (`w3-pane3.md:13-21`, `PLAN-LEARNER-SURFACE.md:401-415`). |
| G-33-02 | S-P4-3 | **Weaker and narrower.** The source computes one primary move from weighted open-work impact and gives a reason/target/contract. The story only orders weak modules by severity and keeps an existing reason string; it does not define the primary move contract (`w3-pane3.md:23-31`, `PLAN-LEARNER-SURFACE.md:417-431`). |
| G-33-03 | S-P5-2 | **Weaker.** The source keeps omitted work, dead ends, and retry predicates; the story keeps repeat-missed/resolved learner items but does not specify a dead-end/ruled-out state or retry predicate schema (`w3-pane3.md:33-41`, `PLAN-LEARNER-SURFACE.md:455-468`). |
| G-33-04 | S-P5-1 | **Faithful at the local scale.** Missed-first, omission notice, count, and expand-all preserve the source’s budget/omission decision without importing its network projection machinery (`w3-pane3.md:43-51`, `PLAN-LEARNER-SURFACE.md:437-453`). |
| G-33-05 | S-P4-1 | **Weaker but correctly adapted.** Local measured-at and pack identity preserve provenance; stable citable URLs, access date, collaborative review time, and handback metadata are intentionally not carried (`w3-pane3.md:53-61`, `PLAN-LEARNER-SURFACE.md:382-399`). |
| G-33-08 | S-P4-1 | **Weaker.** The source makes cache/current/refresh behavior visible; the story only asks for a stale label/date. It does not define a learner action for refresh or how a stale local result becomes current (`w3-pane3.md:83-91`, `PLAN-LEARNER-SURFACE.md:382-399`). |
| G-33-15 | S-P5-1 / S-P5-2 | **Mostly faithful.** The factual, non-credential recovery treatment is preserved, but the plan does not explicitly guard against competition/streak language in the bad-result acceptance (`w3-pane3.md:153-161`, `PLAN-LEARNER-SURFACE.md:437-468`). |

The main distortion risk is that “evidence-driven” is being reduced to
“different output for two profiles.” A selector can satisfy that assertion by
reading an arbitrary state bit while ignoring misses. Similarly, “handback” is
being reduced to a date/delta without defining which prior attempt is the
baseline.

## 3. STALE CITATIONS — 5 of the 13 plan ranges are stale or misaligned

I checked every range named in the dispatch. Ten still point at the behavior
the plan describes. Five moved under pane 3 and now mislead an implementer:

| Plan citation | Current tree | Verdict and corrected location |
|---|---|---|
| `course-engine/web/assets/js/hub_mastery.js:339-378` | `:339-354` is `saveLastWeak`; recommendation begins at `:376` and the weak/unpracticed/unmastered selection is `:390-432` | **STALE.** The plan’s claim is now at `:376-432`; the old range starts in the wrong function. |
| `course-engine/web/assets/js/hub_mastery.js:467-483` | `:461-468` is `moduleBadgeState`; `badgeHtml` is `:479-497`; `paintRecommend` is `:499-546` | **STALE.** The displayed reason/Next up strings are now `:518-544`, not `:467-483`. |
| `course-engine/web/assets/js/results.js:437-508` | `:437-451` finishes weak-module rendering, `:454-477` renders recovery, and `renderItemList` begins at `:479` and continues through `:559` | **STALE/MISALIGNED.** The range still overlaps the item loop, but it no longer identifies the list function as claimed. Correct citation: `:479-559`; recovery is `:454-477`. |
| `course-engine/web/assets/js/results.js:561-568` | `:561-568` is `escapeHtml`; attempt identity is now populated at `:613-620` | **STALE.** Correct citation: `:613-620`. |
| `course-engine/web/index.html:60-104` | `:60-93` is six hub cards; `:95-108` is the Specialist tracks section, not an eight-card grid | **STALE.** The plan’s “eight-card grid” premise no longer matches the current six-card block; cite `:59-93` for the grid and `:95-108` for tracks. |

Still current and checked: `learn_units.js:400-418`, `learn_units.js:544-551`,
`learn_units.js:317-323`, `results.html:46-58`, `results.html:92-94`,
`review.js:405-420`, `power-path.html:236-258`, and
`power-path.html:431-452`. The current results and hub changes are not merely
line shifts: the hub recommendation and results recovery code have materially
changed shape, so the stale ranges must be repaired before materialization.

## 4. BUILDABILITY — the fresh-agent bar fails for every product story

The plan says a fresh agent must not ask a human a question, but the 11 P3–P6
stories each leave a product decision open. P0 also has unresolved wiring
questions. These are not requests for more evidence; they are missing
implementation choices.

| Story | Human question a fresh implementer would have to ask |
|---|---|
| S-P3-1 | Which distinction/correction copy should appear for each micro-check, and what exact state/schema records `corrected-after-miss` without changing the existing study signal? `PLAN-LEARNER-SURFACE.md:315-336` names the behavior but not the data contract. |
| S-P3-2 | What is the deterministic weighting formula, tie-break order, and next-action vocabulary? “Weights outcomes” and “different profiles differ” are not enough to implement the selector (`:338-357`). |
| S-P3-3 | Which modules are approved for interaction, and what state transition does each expose? “Identify the modules” explicitly delegates content/design judgment to the implementer (`:359-376`). |
| S-P4-1 | What timestamp is authoritative for Results versus mastery, what makes it stale, and what exact pack identity is displayed? `review.js`’s `saved_at` is a missed-feed timestamp, not automatically the mock attempt timestamp (`:382-399`). |
| S-P4-2 | Which prior attempt is the baseline, and what delta is meaningful: score, weak-module set, mastered modules, or all three? The story says “delta” without a schema or selection rule (`:401-415`). |
| S-P4-3 | How is severity calculated, where are miss counts/rates persisted, and how are ties resolved? Current `hub_mastery.js` accepts weak module IDs, not a specified severity payload (`:417-431`). |
| S-P5-1 | What is the numeric height/item budget, which items are initially omitted, and does expand-all preserve the same order? “Budget” and “missed first” do not define the interaction contract (`:437-453`). |
| S-P5-2 | What counts as “resolved,” how long does history live, and what exact retry predicate is shown for a repeat miss? The story has no retention or scheduling rule (`:455-468`). |
| S-P6-1 | Which specific module/action is the zero-state recommendation, and how is first-run distinguished from returning-with-nothing-due when local storage is empty or partial? The plan says tied to a module but names no module or priority (`:474-490`). |
| S-P6-2 | Which learner-actionable facts replace the footer, and what are their fixed denominators/source measurements? The acceptance prohibits invented numbers but does not choose any (`:492-506`). |
| S-P6-3 | What prerequisite applies to which Learn module, and what does “skip” skip to? The story names only `01-mission-critical.html`, not the 15-module content decisions (`:508-521`). |
| S-P0-1 | Which exact registry files, claim row syntax, expected IDs, and lattice justification are wired into the checker? The plan names `registries/claims.toml` and a scan set but not the concrete test/fixture contract (`:249-269`). |
| S-P0-2 | Is this a browser-rendered harness, a string fixture harness, or both; how are four storage/state profiles created; and how is “bypass” detected? The story names profiles and rendered strings but no runner or fixture format (`:271-290`). |

The plan is therefore not self-contained enough to materialize. At minimum, it
needs a per-story file list, state schema, fixed examples, and exact expected
strings/ordering rules before a fresh agent can implement without asking.

## 5. DEPENDENCY GRAPH — phase edges are too coarse

The plan’s phase DAG is acyclic as drawn: P0→P3/P4, P4→P5, P0/P1/P3/P4→P6,
and P2 independent (`PLAN-LEARNER-SURFACE.md:204-225`). There is no cycle in
that graph. The problem is that file ownership is not represented at story
level.

### Missing collision edges

These are the definite edges the plan needs if stories can run concurrently:

| Missing edge | Shared file or surface |
|---|---|
| S-P0-2 ↔ P1 | `web/assets/js/smoke_rendered_output.js`; P1 is explicitly in flight and the P0 harness edits the same file (`PLAN-LEARNER-SURFACE.md:271-290,294-302`). |
| S-P3-1 ↔ S-P3-2 | `web/assets/js/learn_units.js`; both stories edit the same state/advance machinery (`:315-342`). |
| S-P3-3 ↔ S-P6-3 | Learn module HTML/interaction surfaces; P3-3 says to choose module pages but P6-3 directly changes `learn/01-mission-critical.html`, so the plan must reserve the page set (`:359-376,508-521`). |
| S-P4-1 ↔ S-P4-2 | Results summary/attempt state and the persisted review timestamp; both define the return-state contract (`:382-415`). |
| S-P4-1 ↔ S-P4-3 | `index.html` mastery block and `hub_mastery.js` state/recommendation output (`:382-431`). |
| S-P4-2 ↔ S-P5-1 | Results state and result-page structure; handback and budgeted review both change the result presentation (`:401-415,437-453`). |
| S-P4-2 ↔ S-P5-2 | Results/review history and the dashboard’s retry state (`:401-415,455-468`). |
| S-P5-1 ↔ S-P5-2 | Results/review disclosure and history/retry presentation; both alter the recovery surface (`:437-468`). |
| S-P6-1 ↔ S-P6-2 | `web/index.html` card grid and hub state/CTA; both change the first screen and its action hierarchy (`:474-506`). |
| P1 ↔ S-P4-1, S-P4-2, S-P4-3 | P1’s stated moving baseline includes `results.js`, `review.js`, `hub_mastery.js`, `mastery.js`, and hub HTML; each P4 story touches at least one of those surfaces (`:294-302,382-431`). |
| P1 ↔ S-P5-1, S-P5-2 | P1’s moving Results/Review surfaces overlap both density and history stories (`:294-302,437-468`). |
| P1 ↔ S-P6-1, S-P6-2 | P1 changes the stateful hub card and `index.html`; both cold-start stories change that same screen (`:294-302,474-506`). |

P0-1 ↔ P2 is also a likely missing edge: P0 edits claims/claims-lint while
P2 says it touches registries (`PLAN-LEARNER-SURFACE.md:249-266,306-309`).
The plan must name P2’s exact registry files; until then, “independent” is not
safe.

### Drawn edges not justified by file overlap

- **P0→P3/P4/P5/P6** is a logical gate/constitution dependency, not a file
  collision. It is defensible as policy, but §6 should say so; the P0 registry
  files do not justify serializing product edits by themselves.
- **P3→P6** is only partially file-justified. S-P3-3 and S-P6-3 may collide
  on selected Learn pages, but S-P3-1/S-P3-2 edit `learn_units.js` while
  S-P6-1/S-P6-2 edit hub files. The plan needs story-level reservations rather
  than treating all of P3 as one lock.
- **P4→P6** has a real overlap through hub mastery/index, but not every P4
  story blocks every P6 story. It is a safe coarse serialization edge, not an
  accurate dependency graph.
- **P2 independent** is not established until its “registries” are enumerated;
  if it includes `claims.toml` or the same checker fixtures, the edge is false.

Adding the missing edges with a consistent order does not create a cycle. The
plan’s current graph is acyclic; it is simply under-specified and unsafe for
parallel materialization.

## 6. SHOULD-FAIL QUALITY

The plan has a planted bad for each story, but several plants can pass while
the real property remains broken. The strongest replacements assert the
observable state transition, not merely the presence of a word or a differing
fixture output.

| Story | Can the current plant pass without fixing the defect? | Stronger plant |
|---|---|---|
| S-P0-1 | **Possibly.** A checker can assert a hand-authored expected row while the real scan set does not detect deletion. | Delete each expected claim row in turn, run the real registry gate, and assert the missing ID is named; also plant an unreferenced claim and require the checker to reject the orphan. |
| S-P0-2 | **Yes.** A fixture/string assertion can pass while the browser path never renders it, and “bypass” is not an executable condition. | Launch the shipped static page with seeded localStorage/sessionStorage profiles, capture visible DOM text, and require the real harness to fail when the profile fixture is removed or the assertion module is not invoked. |
| S-P3-1 | **Yes.** A retry button can exist while every choice remains disabled or while a retry silently records a clean first-pass correct. | Click wrong in the real module: assert same concept/unit remains, at least one choice is enabled after retry, `data-done` does not terminally close the card, then click correct and assert a distinct corrected-after-miss record plus unchanged first-pass score. |
| S-P3-2 | **Yes.** Two profiles can differ because of an arbitrary flag while miss evidence is ignored. | Keep every field identical except item miss counts/rates; assert the highest-severity item is selected, its reason contains the computed evidence, ties use a fixed order, and repeated runs are identical. |
| S-P3-3 | **Yes.** An accessible name can be fixed with `aria-label` while the interaction is a no-op or has no causal state change. | For every selected interaction, drive mouse and keyboard input, assert model state before/after, wrong feedback, retry, and a text-bearing accessible name; plant a no-op handler and a reveal-only interaction that never exposes changed state. |
| S-P4-1 | **Yes.** A hidden or unrelated “stale” string can satisfy the scan while the visible result shows the wrong timestamp/pack. | Seed fresh and stale profiles with different hashes/timestamps; assert visible measured-at, visible pack identity, stale/fresh branch, and correct association with the attempt that produced the score. |
| S-P4-2 | **Yes.** A constant “0% change” can pass first-visit and returning-profile presence tests without computing a delta. | Use two prior/current attempts with known score, weak-set, and mastery changes; assert the exact delta, no baseline on first visit, and “no change” only for identical attempts. |
| S-P4-3 | **Yes.** An arbitrary state bit can reorder modules without using miss severity. | Keep module set and all state fixed except miss count/rate; assert lowest rate/highest miss count wins, exact tie-breaks, and reason numbers match the selected module. |
| S-P5-1 | **Yes.** A notice can appear while all 40 items remain rendered, or the implementation can hide everything and report “40 omitted.” | Assert initial DOM visibility is bounded, missed items precede correct items, omitted count equals hidden rows, expand-all reveals exactly the omitted rows, and 375px height/overflow stays within the measured target. |
| S-P5-2 | **Yes.** The no-history plant can pass while repeat misses never render history. | Pair a no-history profile with a repeat-miss profile; assert absence in the former and visible item identity, attempt time, and actionable retry predicate in the latter. |
| S-P6-1 | **Yes.** The null panel can be hidden while the zero-state CTA is generic or points to no specific module. | Seed an empty store and assert the visible primary link has a known module route/title; seed returner/no-due and assert the KEEP copy; seed partial/corrupt storage and assert no fabricated progress. |
| S-P6-2 | **Yes.** Dropping every number makes “every displayed number has a denominator” vacuously pass. | Require the named landmark and at least one approved, fixture-backed fact; enumerate every rendered number and assert its registered denominator/source, with an explicit zero-metric branch if no metric is approved. |
| S-P6-3 | **Yes.** A hidden prerequisite or a link that does not skip the intended material can satisfy a text scan. | Tab through the real page, assert skip-to-main remains first, prerequisite is visible, skip target is the correct section, and the module’s rendered content differs between take-prerequisite and skip paths. |

The weakest current should-fail is **S-P6-2**: its unregistered-number plant
can be defeated by deleting all numbers, while the real defect is unsupported
or misleading proof. The stronger replacement above makes the test
non-vacuous by requiring one approved fixture-backed fact and auditing every
number that actually renders.

## 7. ESTIMATES

The plan does not actually assign S/M/L to the 11 product stories. It labels
S-P4-1 “S” in prose (`PLAN-LEARNER-SURFACE.md:382-390`) and calls P0 cheap
(`:227-230`), but the other story headings have no cost field. That omission
itself fails the fresh-agent bar.

My estimates after reading the current code are:

| Story | My estimate | Disagreement / reason |
|---|---:|---|
| S-P0-1 | M | Registry lattice, scan-set wiring, and checker tests are more than a row edit; the plan’s “cheap” claim hides the test contract. |
| S-P0-2 | L | Four real browser/storage profiles plus rendered-output assertions require a runner/fixture design, not just assertions. |
| S-P3-1 | L | Corrected-after-miss requires a state schema and mastery semantics, not only a retry button; the plan’s lack of exact copy adds design work. |
| S-P3-2 | L | Weight formula, persistence shape, deterministic ties, and rendered contract are an algorithm/design task. |
| S-P3-3 | L | Content audit across 15 modules, accessible interaction design, and reduced-motion/keyboard verification are substantial. |
| S-P4-1 | M | `saved_at` is not automatically the attempt timestamp, and Results plus mastery need a shared provenance schema; S is too low. |
| S-P4-2 | L | Baseline selection and delta semantics require a new state model and multiple returning profiles. |
| S-P4-3 | L | Current weak storage is module IDs; severity requires changing the payload producer, consumer, ordering, and tests. |
| S-P5-1 | L | The 40-row surface needs disclosure behavior, omission accounting, responsive height/overflow measurement, and item-order tests. |
| S-P5-2 | L | Retry predicates, retention, and repeat-miss history are a product/state decision, not a small rendering addition. |
| S-P6-1 | M | D-2 provides a route mechanism, but first-run/partial-storage branching and a specific module priority remain to be chosen. |
| S-P6-2 | M | The landmark is small; choosing real learner-actionable evidence and wiring denominator checks is the larger part. |
| S-P6-3 | M | One page is cited, but the prerequisite decision must be authored against the 15-module curriculum and checked against focus order. |

The plan’s ranking is directionally sound, but its cost language is not ready
for bead materialization. In particular, S-P4-1 is not an S while the source
of the authoritative attempt timestamp is unresolved, and S-P4-3 is not a
small sort change while the persisted weak payload contains only module IDs.

## Return summary

- **Fidelity:** CARRIED 17 / DEFERRED 0 / REJECTED 8 / LOST 15.
- **Stale citations:** 5 of 13; stale ranges are both `hub_mastery.js` ranges,
  both `results.js` ranges, and `index.html:60-104`.
- **Fresh-agent failures:** all 11 product stories fail without human
  questions; P0-1 and P0-2 also fail without registry/harness wiring detail.
- **Missing dependency edges:** P0-2↔P1; P3-1↔P3-2; P3-3↔P6-3;
  P4-1↔P4-2; P4-1↔P4-3; P4-2↔P5-1; P4-2↔P5-2; P5-1↔P5-2;
  P6-1↔P6-2; P1↔all overlapping P4/P5/P6 stories; and likely P0-1↔P2
  pending exact registry ownership.
- **Weakest should-fail:** S-P6-2, because deleting all numbers makes its
  denominator assertion pass; require one approved metric and audit every
  rendered number instead.

This review establishes faithfulness and buildability risks only. It does not
establish that the plan is aimed at the correct problem; that is the parallel
contrarian lane. HOLD after this receipt; do not start implementation.
