# REVIEW — grok r1 (contrarian)

**Target:** `docs/PLAN-LEARNER-SURFACE.md` v1 (pre-review).
**Role:** find where the plan is *wrong*, not where it is incomplete.
**Date:** 2026-08-21. Read-only against the plan; this file is the only write.

Citations below are to the plan, to the files it claimed a fresh agent would not need, and to receipts that already exist in this tree. The self-containment claim was tested by reading those files. It failed.

---

## Verdict — is this plan worth executing as written? (one paragraph, lead with the answer)

No. Execute the in-flight exam loop (P1) and the five-form bead (P2), then stop draining this document into beads until §9 is rewritten against the measurements that already exist. The plan's own thesis is that this project drives a proxy to target and leaves the defect; it then registers four *rendered-string* properties as `invariant(6)` — the same lattice rank as `claim-not-epi-certified` and `claim-grade-byte-exact` — and sequences a constitution phase that ships no learner-visible bytes in front of the only stories that could. Worse: §9 defers F-01 and F-04 because they are "gated behind W1a/W1b" and F-04 is "still the most important unmeasured number," but `docs/receipts/plausibility-detector-2026-08-21.md` already reports 126/135 = 93.3% bank-wide (91.4% through assembler seeds 0..99) and `docs/receipts/bd-rc-unjoined-artifacts-3ri3.1-teaching-test-census-2026-08-22.md` already reports a 105/931 = 11.3% lexical teaching-floor plus named human ABSENT/SHALLOW rows. A plan that parks the two defects it admits outrank it, on a rationale the tree has already falsified, is not a surface plan. It is a permission slip to polish the frame.

---

## Fatal — things that make the plan wrong, not incomplete

### F.1 §9 is factually false in this checkout

The plan's permission to exist as a chrome programme is this sentence:

> Two defects outrank everything in this document. Neither is in scope here and neither is unblocked by any story above.
> F-01 … gated behind the W1a/W1b measurements in the RC-A lane.
> F-04 … Still the most important unmeasured number in the project.

Both halves are wrong *now*.

- W1a is measured. The live gate `cdcp_gate plausibility-detector` exits 2 on the named branch `absolute-universal-lone-plausible`. Bank-wide 126/135 applicable rows key the lone unmarked option (93.3%). Assembler seeds 0..99: 498/545 = 91.4% versus a 25% chance floor. That is not a gate. That is a green light for a content wave, and a preflight intersection of 14 items already exists at `docs/receipts/bd-rc-proxy-measurement-n52x.2-preflight-2026-08-21.md`.
- W1b is measured. 931 approved rows, lexical floor 105/931 = 11.3% SHALLOW+ABSENT, with the original human cases (`m10-q300`, `m15-q385`, plus `m15-q350` ABSENT; `m15-q363`, `m15-q376` SHALLOW) named. The plan's "unknown rate" is a stale controller memory, not a fact.

A document that claims "every claim carries its evidence inline" and then asserts an unmeasured number that has a receipt is not incomplete. It is wrong. Materializing beads from §6–§7 while §9 still says "gated / unmeasured" will dispatch a chrome swarm against a content defect the constitution already knows how to count.

Proposed revision (the only §9 that is admissible):

```diff
--- a/docs/PLAN-LEARNER-SURFACE.md
+++ b/docs/PLAN-LEARNER-SURFACE.md
@@ -9 What this plan does not do
-- **F-01 — the key is the only plausible option.** … gated behind the W1a/W1b
-  measurements in the RC-A lane.
-- **F-04 — the teaching/test mismatch rate.** Still the most important
-  unmeasured number in the project.
+F-01 is measured: 126/135 = 93.3% bank-wide, 91.4% on assembler seeds 0..99
+(receipt: plausibility-detector-2026-08-21.md). The content wave is unblocked.
+F-04 is measured as a lexical review floor of 105/931 = 11.3% plus five named
+human rows (receipt: …-teaching-test-census-2026-08-22.md). It is no longer
+an unknown. This plan does not rewrite items; that is RC-A / n52x.2. What this
+plan MUST do is refuse any story whose green gate can be read as "the course
+now teaches," and it MUST refuse teaching-loop retry (S-P3-1) on items the
+detector already names as lone-unmarked-key hits — retrying those trains the
+cue the content wave is supposed to kill.
```

Until that lands, every later phase is mis-aimed.

### F.2 The highest-ranked surface story amplifies the highest-ranked content defect

§4.2 ranks `G-32-02` (wrong-answer retry) first across all three waves. S-P3-1 generalizes `power-path.html`'s retry into `learn_units.js:400-418`. That would be the right move *if the micro-check items were questions*. A large fraction of them are not. They are F-01 specimens: three absolute/impossible distractors and a lone unmarked key. Form A item 1 is the canonical case (`UX-FINDINGS-DEDUP.md`).

On those items a retry is not a teachable moment. It is a second chance to apply the lexical elimination the first click already revealed. The explanation appends, every other choice is marked, and "try again" means "now pick the sentence that does not contain *always/never/regardless*." That is test-wiseness drill, scored as `corrected-after-miss` and fed toward whatever mastery path S-P3-1 invents.

The plan never intersects S-P3-1 with F-01. It cannot: §9 has parked F-01. The intersection is the load-bearing collision, and it is fatal to "a wrong answer is a teachable moment" as an invariant.

```diff
--- a/docs/PLAN-LEARNER-SURFACE.md
+++ b/docs/PLAN-LEARNER-SURFACE.md
@@ S-P3-1 ACCEPTANCE
-a wrong answer offers retry; the explanation still appears; a correct answer
-on retry is recorded as *corrected-after-miss*
+Retry is offered only on items that are NOT in the live
+`absolute-universal-lone-plausible` finding set. Detector-hit items stay
+terminal on first click (current behaviour) until the content wave removes
+them from that set. A retry that lands on a detector-hit item is a SHOULD-FAIL
+for this story, not an ACCEPTANCE.
```

If you will not take that restriction, S-P3-1 must wait on n52x.2. Ranking it first while F-01 is live is how a pedagogy bar becomes a cartoon-distractor tutor.

### F.3 Lattice inflation: string tests at `invariant(6)`

S-P0-1:

> four rows registered at the correct lattice strength — these are `invariant`(6) class properties about behaviour, not performance numbers

The live lattice (`registries/claims.toml`) puts `claim-not-epi-certified`, `claim-forbidden-dump-bank`, `claim-grade-byte-exact`, and `claim-complete-attempt-mastery` at invariant. Those are properties that must hold on every execution or the product is a lie. "A zero-state profile renders a first action naming a specific module" is not in that class. It is a UI contract on a synthetic fixture. The registry already has the correct weaker bins: `slo` ("empirical operational or coverage target — never an invariant") and `bounded_model` ("checked only within declared bounds"). S-P0-2's own GREEN-DOES-NOT-PROVE admits the harness is a proxy for learners. A proxy cannot justify an invariant. `rank(justifier) >= rank(claim)` is already law in this repo. S-P0-1 proposes to break it on arrival.

This is not a paperwork nit. It is the disease in §1, performed by the constitution the plan wants to write. Once `claim-cold-start-names-action` sits at rank 6, every later agent will treat a hardcoded "Start Module 01" string as an honesty-class fact, the same way `gate_shrink` treated line count as logic moved.

```diff
--- a/docs/PLAN-LEARNER-SURFACE.md
+++ b/docs/PLAN-LEARNER-SURFACE.md
@@ S-P0-1 ACCEPTANCE
-these are `invariant`(6) class properties about behaviour
+these are `bounded_model`(4) properties: they hold on the four planted
+profiles and nowhere else. Stating them as invariant is a build error.
+`claim-not-epi-certified` remains the only learner-facing invariant this
+surface is allowed to share a rank with.
```

### F.4 "Self-contained" is a false claim, and it is load-bearing

§0: a fresh agent must implement any story without asking a human, because every claim carries file/line/number inline and decisions are restated rather than cited.

P1 then says: "Full spec and reasoning: `docs/DECISIONS-W3-EXAM-LOOP.md`." That is a citation, not a restatement. D-1 through D-4 — five named forms, one Drill surface, submit-always-enabled, drop the footer route — are the only decided product law in the exam loop, and they live in another file. A fresh agent implementing P1 from the plan alone does not have the seeds `42, 137, 251, 389, 503`, the "Form A–E" copy, the "partials never feed mastery" const-assert, or the should-fails.

The line-number citations are already a known moving target (§7 P1 note: `hub_mastery.js:339-378` has moved). `learn_units.js:400-418` / `:544-551` still match HEAD as of this read. `results.html:46-58` still matches. That is luck, not a contract.

The self-containment claim is how the plan proposes to survive bead materialization. It does not survive a read of P1. That is wrong, not incomplete: either paste D-1..D-4 into the plan (one author, one voice, as the operator asked) or drop the sentence in §0.

### F.5 The exam-loop "target" silently drops F-07

W3 today: "there is no flag control." W3 target: submit always available, gaps named, confirmation, Submit in tab order. F-07 (`UX-FINDINGS-DEDUP.md`, major, "Every real exam has one") is named in the diagnosis and absent from the target. D-1..D-4 do not decide it either.

That is not an omitted story. It is a completeness lie: the plan presents the exam loop as specified and in flight, while the one control that holds *uncertainty* — the thing a 60-minute exam is for — remains undesigned. Jump-grid-as-recovery was already rejected in the finding. Confirmation-on-submit does not replace a flag. If the call is "defer F-07," say so with a reason. If the call is "P1 covers the exam loop," the call is false.

---

## Structural — sequencing, dependency, scope errors

### S.1 P0-before-everything is the wrong order even on the plan's own terms

The DAG:

```
P0 constitution ──> P3, P4, P5, P6
P1 exam loop in flight, does not need P0
P2 five forms, independent
```

Two facts the DAG cannot survive:

1. **P1 is already shipping learner-visible bytes without P0.** Submit policy, Drill hierarchy, and `MASTERY_REQUIRES_COMPLETE_ATTEMPT` are in `cdcp_schedule` (`const` + `const` assert) and `claim-complete-attempt-mastery` is already an invariant row. The exam loop did not wait for a learner-surface constitution. The plan's "without a registry row, none of those properties can be asserted" is empirically false for the highest-traffic path.
2. **S-P4-1 is estimated S (< 1h) and the data already persists.** `review.js:405-420` writes `saved_at`; `mastery.js` writes `at_ms`; `drill.js:319-320` already renders a timestamp in one place. Blocking "surface the date that already exists" on a constitution phase is how a one-hour product tick becomes a ceremony queue.

Cheaper path that gets a learner-visible improvement sooner:

```
P1 (finish in-flight; do not restack)
P2 (independent; assembler/goldens)
S-P4-1 (date + pack identity; same commit registers claim-state-is-dated at bounded_model)
S-P6-1 cold branch of the D-2 hub card (zero-state names a module; KEEP copy stays for returners)
S-P3-1 retry ∩ not-F-01
content wave n52x.2  ← not this plan, but MUST be concurrent, not "after the frame is pretty"
P4-2 / P5 / P3-2 / P6-2..3 later, serialized on file ownership
```

P6-last is the second sequencing error. The justification is "routing into an undesigned teaching loop is a signpost pointing at the same hole." That is true of a *persona rail* that promises a designed Learn. It is not true of the zero-state hub card. Every new learner hits W1 once; they hit W2 only if they survive W1. A first-run card that says "Start Module 01 — Mission-critical" does not require retry or evidence-driven next. D-2 already designed the mechanism. S-P6-1 is the missing branch of a card pane 3 is implementing *today*. Putting it last guarantees a merge against a hub that has already moved (see C.1).

### S.2 Two "next" brains will disagree

S-P3-2 replaces `idx++` inside Learn with a miss-weighted unit selector.
S-P4-3 reorders the hub's weak → unpracticed → unmastered pick by miss severity.

Both are "Next is computed from what the learner demonstrated." They do not share a function, a store, or a test. A learner can finish a micro-check, get sent to unit 7 of module 06 by Learn, then return to a hub that sends them to module 10 because severity ranked a leak-response miss higher. That is not two stories. That is one product law ("what is the next action?") implemented twice, with no specified winner.

The corpus finding `G-32-03` (letter game: one weighted selector) and `G-33-02` (asimposium: one primary move with a reason) are the same decision at two altitudes. Forking them is how you get eight equal-weight cards again, only now they are two computed cards that argue.

Revision: one selector, two renderers. Put the law in `cdcp_schedule` (WASM already owns mastery and due-at; JS already must not add days — `review.js` comment at line 26). Hub and Learn both call it. S-P3-2 and S-P4-3 collapse to one story with two surfaces. If you will not do that, delete S-P3-2 and keep sequential Learn plus a hub recommendation. Two recommenders is the defect F-28 named for Drill, reproduced for "Next."

### S.3 Pedagogy state in JS vs "WASM owns the law"

G-33-14 (wave 3): "WASM owns mastery laws." D-3 put completeness in `cdcp_schedule` as a const-assert, not a `results.js` if. S-P3-1 then proposes `corrected-after-miss` as a Learn-local recording so it does not inflate first-pass mastery. If that record lives only in `learn_units.js` / localStorage, the dual-path contract is already split: native `cdcp_schedule` will not know a miss was corrected, and a future `cdcp grade` path cannot reproduce the study signal. The plan's §3.3 ("Nothing in this plan may put state into the grading path") is being used to keep pedagogy *out* of the crate that already decides practiced/mastered. That is the wrong side of the seam. Retry outcome is mastery evidence. It belongs next to `MASTERY_REQUIRES_COMPLETE_ATTEMPT`, or it is a second, ungraded, JS-only grade.

### S.4 P4 blocks P5 is a merge tactic pretending to be a product dependency

"Both live in `results.js`/`review.js`. Serialize them or take the merge conflict." That is an Agent Mail reservation, not a DAG edge. S-P5-1 (budget the review wall) does not need dated state to be true. A missed-first list with an omission count is valid on a first visit. Making P5 wait on P4 so two stories do not touch one file is how this repo produces claim-churn instead of product ticks. Reserve the files. Do not invent a "state legibility blocks density" law.

### S.5 The whole plan is mis-prioritized, strongest case

The buyer in `CHARTER.md` / README is interview fluency: walk a white-space tour and explain trade-offs. The study bar is 27/40 as a *signal*. F-01 means Form A item 1, and 126 other authored rows, can be answered by reading for absolutes. F-04's human rows mean at least `m10-q300` and `m15-q385` test decisions the lesson does not teach. W1a says this is systemic (14 of 15 modules have applicable rows; 11 of those at 100%). W1b says the lexical floor is 11.3% and m10 is 37.1%.

Against that, this plan's ranked work is: retry chrome, idx++ chrome, a handback, a shorter results page, a date label, a hub first-run string.

A learner who receives all of P0–P6 and none of n52x.2 will:

- be told to start Module 01 (P6),
- miss a cartoon item, retry it, and learn the cue (P3),
- be routed to another cartoon item because that miss is now "demonstrated weakness" (P3-2 / P4-3),
- sit Form A, eliminate three absolutes, submit with a lovely confirmation (P1),
- see a dated 6/40 with missed-first review (P4/P5),
- and still not know why a CRAC fails closed.

That is the frame on a broken picture. The plan *says this* in the last paragraph of §9, then schedules six phases as if saying it were the mitigation. Saying it is not the mitigation. Concurrent content work is. A surface plan that does not yield the right of way to n52x.2, and that ranks S-P3-1 above "do not retry detector-hit items," is a chrome epic wearing RC-C's name.

I actually believe this. P1 and P2 should finish because they are decided and in motion. P0 as a phase should not start. P3–P6 should be re-scoped after one content increment has moved the 93.3% rate, or should be limited to S-P4-1 and S-P6-1 (data already there; D-2 already designed).

---

## Proxy audit — acceptance criteria that could go green with the defect intact

§1's standard, applied to §2.

### P1 "A cold learner is told what to do first"

**Costume:** a first action naming a specific module.

**What can go green with the defect intact:**

- Hardcode `"Start Module 01 — Mission-critical"` for every zero-state profile. The string names a module. The SHOULD-FAIL (null panel is not the primary route) passes. The defect — eight equal-weight cards and operating instructions in the hero — can remain below the fold. F-23 instruction density can rise if the new card is additive.
- Name a module the learner should *not* start with. Module 07 has zero exact-three F-01 rows in the W1a table; Module 06 has 25/25 at 100%. A selector that always picks `m01` is not "told what to do first." It is `idx++` at hub scale.
- Name "Take a mock" as the first action. It names an action. It is the current empty-state advice. It dumps a cold learner into Form A item 1, the live F-01 specimen.

**What would have to be measured instead:** a cold learner who follows the named first action, without other hub cards, reaches a Learn unit that (a) is a real starting module under a declared rule (syllabus order, or "first module with a non-detector-hit micro-check"), and (b) completes one concept→check cycle. The planted profile must include the *destination page*, not only the hub string. A hub test is a signpost test.

### P2 "A wrong answer is a teachable moment, not a terminal event"

**Costume:** a retry path within the same concept.

**What can go green with the defect intact:**

- A "Try again" button that re-enables the four choices *after* the explanation and the correct letter are already painted. That is a retry of a revealed item. The teachable moment is gone; what remains is compliance.
- Retry on an F-01 item (F.2). The "concept" being taught is "pick the option without *always*."
- Record `corrected-after-miss` in a JS flag that nothing in `cdcp_schedule` reads. Mastery still inflates; the SHOULD-FAIL about silent upgrade is satisfied because a different *key* was written, even if the practiced bar moves.

**What would have to be measured instead:** (1) detector-hit items are not retried; (2) on a non-hit miss, the distinction is named *before* the correct option is marked, and a second attempt is scored separately in the same store WASM uses for practiced/mastered; (3) a planted item whose distractors are plausible is actually retried, and a planted F-01 item is not. Without (1) and (3) this is a button.

### P3 "`Next` is computed from what the learner demonstrated"

**Costume:** two planted miss profiles produce different next actions, asserted on the rendered string.

**What can go green with the defect intact:**

```js
next = (hash(missIds) % 2 === 0) ? units[idx + 1] : units[idx + 2];
```

Two profiles, two strings, SHOULD-FAIL ("identical next action") is red only for a constant function. The plan *names* this risk in GREEN-DOES-NOT-PROVE ("they do not prove the weighting is pedagogically correct") and then makes the non-constant function the ACCEPTANCE. That is how option *length* became the cartoon-distractor gate.

**What would have to be measured instead:** a named rule, tested as the rule. Example of a rule that is not a proxy: "next unit is the earliest unread-or-missed unit in this module; if none, the hub recommendation from the single selector in S.2." Plant (a) miss on unit 2 vs miss on unit 5, assert next is unit 2 vs unit 5; (b) two misses on unit 2 with different *severity* must not be required to diverge if the rule does not say they should. The current SHOULD-FAIL demands divergence even when the pedagogy should be stable. That will force jitter into the selector to keep the test green.

### P4 "Accumulated state is legible and dated"

**Costume:** measured-at and a delta render; a stale profile "says so."

**What can go green with the defect intact:**

- Render `saved_at` as a locale string and a delta of "score 6/40 → 6/40 (no change)" forever. Recency of an unchanged, cartoon-driven score. The learner can see what changed (nothing) and when it was measured (a clock). The defect — a mastery signal they cannot *decide to trust* because the items are F-01 — is untouched. G-33-05 asked "when should I trust this as current?" Trust has two axes: freshness *and* whether the underlying attempt was a study signal or a cue. The plan measures the first and costumes it as the second.
- Define "stale" as `Date.now() - saved_at > N` in JS, while `cdcp_schedule` uses `DAY_MS = 86_400_000` with **no DST** (`lib.rs:23-24`). A DST-crossing laptop, or a machine with a wrong clock, produces a "stale" label that disagrees with the WASM mastered-gap law. The planted stale profile uses the JS clock. Green, split-brain.
- First-visit SHOULD-FAIL ("no invented improvement") can pass while the returning-visitor profile fabricates a baseline from a *partial* attempt that D-3 forbade as mastery evidence. Dated garbage.

**What would have to be measured instead:** (1) the timestamp shown is the same `at_ms` / `saved_at` the schedule crate already persisted, formatted, not recomputed; (2) "stale" is defined against `DAY_MS` via the WASM bridge, not `Date.now()` ad hoc; (3) a delta that includes a detector-hit item is labelled as including items the plausibility gate still fails, or the delta omits them and says so. Otherwise you have a clock on a proxy.

### Cross-cutting: S-P0-2's four profiles are RC-A in miniature

The plan knows this (`GREEN-DOES-NOT-PROVE`: "profiles are a **proxy** for learners, and per RC-A the output must name that gap"). Then it makes those profiles the thing P0 wires to the new invariants. A known proxy is being installed as the enforcement mechanism for properties the plan just called invariant. Naming the gap in stdout does not stop the next agent from closing a bead when the four strings match.

---

## The case against P0

**Strongest case that P0 should be deleted.**

P0's deliverable is four registry rows and a widening of `smoke_rendered_output.js`. No HTML a learner sees changes. No bank item changes. No grade digest changes. The only consumer of P0, on the day it lands, is the next agent, who now has vocabulary with which to write `[[claim:claim-cold-start-names-action]]` next to a string they have not made true. That is the 2026-08-20 pattern this repo already measured: 65 of 91 commits process, 43 of those claim/release churn, `bank/items` absorbing 8 lines. AGENTS.md banned committing queue claims for this reason. P0 is a queue claim with a lattice.

The plan's defence: "the 98-step gate cannot fail on a design defect because the constitution has no vocabulary." True. Incomplete. Vocabulary without a bound surface is how `claims-lint` becomes a spellchecker for slogans. The live counterexample is already in the registry: `claim-complete-attempt-mastery` was added *with* the `cdcp_schedule` const and the JS precondition, not in a prior phase that shipped nothing. That is L1 used correctly. S-P0-1 inverts it: register first, wire later (S-P0-2), implement even later (P3–P6). GREEN-DOES-NOT-PROVE on S-P0-1 says the rows "prove nothing about the surface until S-P0-2." So the phase that "must precede everything" admits it proves nothing. A phase whose success is "the vocabulary exists" has no product consumer.

P1 did not wait. P2 does not need it (assembler, goldens, packs). S-P4-1 does not need it. The DAG's "P0 first, non-negotiable" is therefore not a dependency. It is a ritual that makes the chrome programme feel like the rest of the 98-step gate.

Should-fail theatre: "delete one claim row and the build must fail." That proves the linter reads the file. It is the same class of proof as an empty-scan ERROR. It does not prove a cold learner was told what to do.

**Do I actually believe P0 should be deleted?** Yes as a *phase*. No as a *habit*. Each story that first makes a property true should add its bounded_model row and one rendered-string assertion in the same commit, the way D-3 already did for completeness. That is constitution as a side-effect of shipping, not a preamble. Demote S-P0-1/S-P0-2 to a checklist line inside S-P6-1, S-P3-1, S-P3-2, S-P4-1. If a story cannot name the claim it registers, it is not a story yet. If it can, it does not need a prior phase.

```diff
--- a/docs/PLAN-LEARNER-SURFACE.md
+++ b/docs/PLAN-LEARNER-SURFACE.md
@@ ## 6. Phases
-| **P0** | Constitution — register the learner surface | P3, P4, P5, P6 | — | not started |
+| **P0** | deleted as a phase. Claim rows land in the first story
+  that satisfies them (S-P6-1, S-P3-1, S-P3-2, S-P4-1), at
+  bounded_model, never invariant. An empty learner-claim set
+  after those four stories is an ERROR; an empty set before
+  them is the starting state, not a defect.
```

---

## What the methodology structurally excluded

Not "more findings." The UX campaign looked at screens. The corpus waves looked at other people's screens. Both are *first-sitting chrome*. A learner surface has failure classes those instruments cannot see.

### M.1 False mastery from test-wiseness

The only way to see F-01 is to *read the items as a candidate*. The UX audit actually did this, and it produced 31 of 117 findings. The website differential would never have found it (no corpus homepage contains a four-option MCQ with three absolutes). The plan then *excluded* the one class the UX method uniquely produced. So the methodology of the plan is: take a chrome-and-corpus pipeline, drop the only non-chrome result, and design chrome.

What remains invisible even to the original audit: a learner who *uses* the cue across a 40-item form, clears 27, and is told they have a study signal. That is a longitudinal property of the bank-plus-surface together. No screenshot of results.html, and no frankensim hero, can show it. You need to sit Form A as someone who does not know the domain. The plan has no story whose ACCEPTANCE is "a domain-naive planted solver that only eliminates absolute markers cannot reach 27/40 on Form A." That is the missing learner-surface property. It is also an RC-A measurement, and it is more honest than P1–P4.

### M.2 Pack identity vs localStorage identity

Offline, the bank hash is the integrity identifier. Mastery, misses, and `saved_at` live in localStorage keyed by schema version, not by `bank_hash`. A `git pull` that rebuilds the bank leaves a learner with dated state about items that no longer exist, or that changed their key. Wave 3 asked "which pack produced this?" and the plan answers by *displaying* the current pack hash next to possibly-stale attempt state. Neither a UX pass nor a website read produces "what happens when the artifact under the hash moves." The SHOULD-FAIL for S-P4-1 (stale profile says so) uses a clock, not a hash mismatch. The excluded class is **content-addressed state**. The product already content-addresses the bank. The surface does not content-address the learner's history.

### M.3 The actual cold start is `cargo build`, not `index.html`

Every wave and the UX log start at the hub. README's honesty constitution and the artifact-rigor posture start at `git clone` / `cdcp study`. The reason the hub hero leads with `CDCP_FILE_ORIGIN` is that the real first-run failure is opening `file://`. Moving that copy to "the error states that fire it" (W1 target, S-P6-1) is correct *for people who already reached HTTP*. The methodology never asked: what does a non-Rust learner see when `cargo` is not installed? There is no GitHub Release tarball (README). The designed hub is the second screen of a toolchain product. A 212-website differential of shipped marketing sites will always over-weight the hero and under-weight the install. This plan inherits that bias.

### M.4 Time as a learner law, sitting next to a grader that forbids clocks

`cdcp_grade` is pure: no I/O, no clock, no randomness. `cdcp_schedule` *does* use a clock (`DAY_MS`, `MASTERED_MIN_GAP_MS`, `cdcp_due_at_ms`). The seam is already there and already bridged. A single-sitting UX audit cannot see: the 24h mastery gap, laptop sleep, two timezones, a user who takes Form A at 23:50 and Form B at 00:10, DST (explicitly ignored by `DAY_MS`). P4's "dated" property will be implemented with `Date.now()` in JS because that is what `saved_at` already is. The excluded class is **schedule time vs wall time vs displayed time**. Wave 3 saw recency as a label. It is a law.

### M.5 Multi-session, multi-tab, multi-device last-write-wins

DEDUP already confessed: "any session longer than a single sitting" was unobserved. Corpus sites with accounts do not have this problem; we have one origin's localStorage. Two tabs, a half-submitted mock, a Drill session in the other, last writer wins. No finding in F-01..F-28 or G-31..G-33 is a storage-isolation finding except F-12 (reopened mock still submittable), which is the single-tab shadow of this class. The plan's P1 treats F-12 as exam-loop chrome. The class is **local persistence under concurrency**.

### M.6 Authoring: the writer cannot see the cue

A learner surface is downstream of a bank. Nobody who writes `bank/items/*.toml` has a view that says "this option set is a lone-unmarked-key hit." The UX auditors saw it because they sat the exam. The next author will reintroduce F-01 during n52x.2 unless a gate already fails the item. That gate exists (`plausibility-detector` exits 2). It is not a learner-surface story, and a website differential would never ask for it. The excluded class is **author-facing consequences of learner-facing defects**. A plan that wants "designed" and not "verified" still has to say who is supposed to stop writing cartoon distractors. Today the only consumer of the detector is an agent reading a receipt.

### M.7 Fatigue, a clock, and a flag — exam cognition

Sixty minutes, forty items, working memory. F-07 (flag) is the affordance that class requires. Corpus explainers are untimed. UX findings were produced by agents who were not under the timer. The methodology structurally prefers layout and copy over *exam-as-endurance*. Dropping F-07 from the W3 target (F.5) is the predictable output of a method that never sat a timed form as a tired human.

### M.8 WASM failure as a learner event

G-33-06: grader failure withholds scores (honest) and points at a build command (not a retry). Transferable: DEGRADED. The plan dropped it. Marketing websites do not have a WASM instantiate path. This product does. A designed learner surface that cannot recover from a missing `cdcp_wasm.wasm` without reading a compiler error is still undesigned on the only path that makes the byte-exact claim *real in the browser*.

---

## Inadmissible ideas I considered and rejected (with the killing constraint named)

| Idea | Why it looked useful | Killing constraint |
|---|---|---|
| LLM rewrite of the 126 F-01 distractors inside the product path, or an in-app "explain why this is wrong" model call | would actually attack F-01 | **No LLM in the product path.** An LLM cannot be pinned to a golden. |
| Ship a hosted miss-rate dashboard so "evidence-driven next" uses real cohort weights | would make P3 not a single-user proxy | **Fully offline, no network, no fetch.** |
| React virtual list / CSS-in-JS for the 13,515 px review wall | would budget the wall cheaply | **No build step; static HTML + vanilla JS + one CSS file.** |
| Three.js / canvas "place the CRAC" as the S-P3-3 propagation | G-32-06 wants manipulable models | **No npm, no CDN, reduced-motion must not be load-bearing (G-33-12 / G-31-08 already rejected).** |
| Expanding-interval SRS to make "Next" actually about forgetting | letter-game weights are a real pedagogy | Charter / `cdcp_schedule`: **not SRS; `[1, 3]` cap.** Calling it SRS is an overclaim. |
| Soften or hide the honesty banner on first-run so the hero can "convert" | G-31-01 wants a promise then a CTA | **`claim-not-epi-certified` is never softened.** G-32-13 already killed achievement framing. |
| A/B two hub first-actions and keep the winner | would stop hardcoding Module 01 | Offline, no telemetry; and the grader forbids randomness. A JS coin-flip on the hub is survivable technically and still **implies we optimize a conversion metric**, which is how a study tool starts performing. |
| Pixel-perfect visual regression (Percy/Chromatic) as the P0 harness | would catch KEEP contrast/overflow | **Network + third-party.** Also a proxy for "designed." The KEEP floor is already measured numbers, not screenshots. |
| Fetch new forms / bank updates at runtime so five forms stay fresh | D-1's diversity SLO over time | **No network at runtime.** Forms ship in the pack. |
| Put retry / next-unit selection into `cdcp_grade` so it is byte-exact | would unify the WASM law | **Grader stays pure: no clock, no state, no I/O.** Schedule, not grade. The admissible home is `cdcp_schedule`, which already has a clock. |

I did not reject "run the content wave." That is admissible, in another lane, and it outranks this plan. I rejected folding it *into* this plan's stories, because a surface plan that starts rewriting TOML will collide with n52x.2 and with `claim-forbidden-dump-bank` / near-duplicate admission gates it does not name.

---

## What I would keep exactly as written and why

A review that finds everything wrong is as useless as one that finds nothing. These parts are right. Do not "improve" them in review rounds.

**§3, the invariant box, in full.** Offline, no-build, pure grader, honesty banner, KEEP floor, reduced-motion, no LLM. This is the only reason the corpus waves are usable. The six `Transferable: NO` rows in §4.4 are the proof the box is live. Several of my own first impulses died against it (previous section). A plan that can reject Three.js, Colab, CDN stacks, and "unlock challenges" is a plan that still knows what this product is.

**§4.3, the power-path correction.** An earlier controller reading said Learn had no interactivity. That was wrong; `power-path.html:431-452` already has wrong-slot retry. The plan records the correction and changes the work from "add interactivity" to "generalize a pattern we have." That is the opposite of the house failure mode. Keep the sentence, keep the cheaper scope, do not let S-P3-3 quietly become "invent a widget per module." The "leave at least one module alone and say why" clause is the only anti-vacuous rule in P3 that I trust.

**§4.4 rejection table and §4.5 "already better than the corpus."** The always-visible claim boundary, the offline + reduced-motion contract, and the stateful empty state as a *returner* strength are real. D-2's resolution of the G-31-10 / "single worst thing" disagreement (KEEP copy for returners, different branch for zero-state) is the correct product move. S-P6-1's ACCEPTANCE already encodes both branches. Keep that dual assertion; it is the one place the plan actually preserves a KEEP while closing a hole.

**Every story's GREEN-DOES-NOT-PROVE line, as a habit.** The plan knows it is using proxies. The failure is using them as phase gates and as `invariant(6)` anyway — not the habit of writing the gap down. Keep the field. Make it binding: a story whose GREEN-DOES-NOT-PROVE names a proxy may not register an invariant.

**D-3's substance, even though it lives in another file.** Partials never feed mastery; unanswered ≠ wrong; confirmation names the gaps as jump links. That is product law, already partly compiled (`MASTERY_REQUIRES_COMPLETE_ATTEMPT`, `claim-complete-attempt-mastery`). It is not chrome. It is the same family as the honesty banner. Finish P1. Do not redesign it in this review.

**D-1's "a form without a golden pair is a build error."** Five forms, ten digests, seed integers demoted out of the default reading. The diversity SLO is correctly *not* an invariant. This is how the lattice is supposed to work. P2 should ship. It is independent of this plan's chrome DAG and it is the only story here that adds a learner-visible study affordance the 14-day plan actually uses.

**§8 risk table, especially proxy substitution, KEEP regression, honesty drift, and shared-file collision.** The diagnosis is accurate. The mitigation for proxy substitution ("name property, proxy, and gap in output") is necessary and insufficient — see the proxy audit — but it is the right warning to print. The KEEP-as-floor rule is the only reason a hub rewrite will not trade 13.35:1 contrast for a prettier card. Shared-file collision is real; the wrong mitigation is a fake DAG edge (S.4), the right one is reservations. Keep the risk; change the mitigation.

**The last paragraph of §9, as a sentence, not as a policy.**

> A perfectly designed surface delivering items answerable by eliminating three absolutes is still a tool that does not teach. This plan makes the surface honest about what it is doing. It does not make the questions good, and no green gate produced by this plan may be read as if it did.

That is the true north. It is also the sentence that makes the rest of the plan indefensible as a *standalone* execution queue. Keep it. Then obey it: do not materialize P0–P6 as the next epic while 93.3% of applicable items still key the lone unmarked option, and do not let any green smoke test from this file be cited as interview-readiness.

**Should-fail as an admission test for stories.** "A story without a planted known-bad is not admissible; an empty scan set is an ERROR." Keep this as a gate on *materialization*. It is one of the few rules in the plan that would have killed S-P3-2's "two profiles, two strings" if anyone had demanded the known-bad to name the *pedagogical rule* rather than non-constancy.

That is the set. Everything else in v1 should be treated as a draft that failed its own §1 audit.
