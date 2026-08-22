# PLAN — The Learner Surface: from verified to designed

**Version:** v1 (pre-review). Revise **in place**. Never mint a successor document.
**Owner:** controller (cockpit), session `cdcp`.
**Parent epic:** `bd-rc-surface-undesigned-0ns7` — *EPIC RC-C: the learner surface was verified,
never designed.*
**Status:** plan-space. Not converged. Not yet materialized into beads.

---

## 0. How to read this, and why it is self-contained

A fresh agent who has never seen this repository must be able to implement any story here without
asking a human a question. That means every claim in this plan carries its evidence inline —
file, line, and measured number — rather than pointing at another document. Where a decision was
already made, the decision and its reasoning are restated here rather than cited.

This plan covers **one** thing: the surface a learner touches. It deliberately does **not** cover
the two defects that outrank everything in it. See §9.

---

## 1. The problem, stated in measurements

The engineering under this surface is genuinely good. The design above it does not exist. Those are
different conditions and they have different remedies: broken gets patched, unconsidered gets
designed.

**What was measured and is good** (34 KEEP findings, `docs/ux/UX-FINDINGS-DEDUP.md`):

- dark-theme contrast **13.35:1** body / **7.73:1** secondary / **5.18:1** faint caption against
  `#060b09` — measured, not assumed
- every advertised key binding works: A–D, 1–4, ArrowLeft/Right, P/N, verified in a fresh mock
- **375 px has no horizontal overflow** on mock — answer cards, the 1–40 jump grid, submit all fit
- reduced-motion honoured: durations collapse, smooth scrolling off, skip-link transition removed
- prose column stays constrained at 1280 and 1920
- skip-to-main-content link present and first in tab order
- diagram self-checks use `fieldset`/`legend` with text-bearing labels
- the "No cards due" empty state names its next action instead of showing a blank panel

**What was measured and is not** — the campaign produced **117 raw findings → 59 distinct, 0
blocker, 64 major**. Zero blockers with sixty-four majors is the signature of a surface that is
correct and unconsidered.

**What the differential read against an external corpus added.** Three waves against Jeffrey
Emanuel's shipped web work (212 repos, Studio mirror) produced **40 further findings** in
`docs/ux/gap-waves/w{1,2,3}-pane3.md`, each carrying a line citation on both sides and a
transferability verdict against our constraints. Six were rejected outright as inadmissible here.

**The structural cause.** This project's rigor model is that a claim is not real until it is a row
in a registry that `claims-lint` enforces. There are **18 registries** — `claims.toml`,
`slo.toml`, `objectives.toml`, `construction_faults.toml`, `differential_harnesses.toml` and more.
**Not one row mentions `ux`, `design`, `journey`, `learner`, `progress`, or `recover`.**

The 98-step gate therefore *cannot fail on a design defect*. It is not that someone forgot to add a
check. It is that the constitution has no vocabulary in which such a check could be expressed. This
is why §2 comes before every other phase.

---

## 2. What "designed" means here, and how we will know

We are not adopting a visual style. The corpus is an oracle, not a superior — several of its
signature moves are inadmissible here and were rejected on the record (§4.4). "Designed" means
exactly four properties, each of which becomes a registered claim:

| # | Property | Today | The one-line test |
|---|---|---|---|
| P1 | **A cold learner is told what to do first** | The hub's only wayfinding element renders "No cards due" on first visit | A zero-state profile renders a first action naming a specific module |
| P2 | **A wrong answer is a teachable moment, not a terminal event** | `learn_units.js:400-418` disables every choice on first click; no retry | A wrong micro-check answer offers a retry path within the same concept |
| P3 | **"Next" is computed from what the learner demonstrated** | `learn_units.js:544-551` advances by `idx++` | Two learners with different miss profiles get different next actions |
| P4 | **Accumulated state is legible and dated** | `results.html:46-58` shows exam, seed, bank hash — no timestamp, no delta | The learner can see what changed since last visit and when it was measured |

Each is falsifiable, each gets a planted known-bad, and none of them is a matter of taste.

---

## 3. Constraints — the invariant box

Every story in this plan is bounded by these. A proposal that violates one is rejected, not
negotiated. These are why six corpus findings were marked `Transferable: NO`.

1. **Fully offline at runtime.** No CDN, no network, no fetch. After `git clone`, nothing in this
   project talks to the network again.
2. **No build step in the shipped path.** Static HTML + vanilla JS + one CSS file
   (`assets/css/course.css`). No React, no Next, no Framer Motion, no npm.
3. **The grader stays pure.** `cdcp_grade` is a pure function with no I/O, no clock, no randomness,
   `#![forbid(unsafe_code)]`. Rust and WASM must produce byte-identical digests. Nothing in this
   plan may put state into the grading path.
4. **The honesty banner is load-bearing.** Registered as `claim-not-epi-certified`. It is never
   softened, never hidden on scroll, never given a "certified" green treatment. Progress and
   mastery language must never imply a credential.
5. **The 34 KEEP measurements are a regression floor.** A change that improves a major while
   regressing a measured keep is a **net loss** and must be caught, not traded.
6. **Reduced-motion is honoured and measured.** State must be legible without motion. Motion may
   never be load-bearing for comprehension (`G-33-12`).
7. **No LLM in the product path.** Ever. An LLM grader cannot be pinned to a golden.

---

## 4. The evidence base

### 4.1 Where the findings live

| Source | Contents |
|---|---|
| `docs/ux/UX-FINDINGS-DEDUP.md` | 59 distinct findings (F-01…F-28) + the 34 KEEP measurements |
| `docs/ux/gap-waves/w1-pane3.md` | 11 findings vs flagship product sites — hierarchy, proof, routing |
| `docs/ux/gap-waves/w2-pane3.md` | 14 findings vs interactive explainers — **the pedagogy bar** |
| `docs/ux/gap-waves/w3-pane3.md` | 15 findings vs data/state surfaces — **the dashboard bar** |
| `docs/DECISIONS-W3-EXAM-LOOP.md` | D-1…D-4, decided, with registry rows and should-fails |

### 4.2 The five highest-ranked gaps

Ranked by the agent that read all three corpora, and independently consistent with the controller's
ordering:

1. **`G-32-02` — wrong-answer retry inside the same concept.** Outranks every dashboard refinement
   because the learner loses the correction moment exactly when the misconception is visible.
2. **`G-32-03` — evidence-driven next concept.** Fixed `idx++` makes every later practice decision
   blind to what the learner just demonstrated.
3. **`G-33-01` — a returning-state handback.** A learner cannot act on mastery state they cannot
   see changing.
4. **`G-33-04` — a bounded, non-wall results view.** A 13,515 px review surface hides the
   corrective evidence before the learner can use it.
5. **`G-33-05` — learner-visible provenance and recency.** Without "measured when, against which
   pack," a mastery signal cannot tell the learner whether to trust it.

### 4.3 One correction to the record

An earlier controller reading claimed the Learn surface has essentially no interactivity. That was
wrong and the correction changes the work. `web/diagrams/power-path.html:236-258` and `:431-452`
already implement placement, wrong-slot feedback **and retry**. The capability exists; it was built
once and never propagated. The fix in Phase 3 is therefore *generalize the pattern we already
solved*, not *add interactivity* — cheaper, better grounded, and consistent with the substrate law.

### 4.4 What we rejected, and why

Recording rejections matters as much as recording gaps; a wave that finds everything transferable
has stopped evaluating.

| Rejected | Killing constraint |
|---|---|
| Three.js / lazy-mounted 3D hero (`G-31-08`) | offline, no build step |
| Remote OG-image previews with proxy fallback (`G-31-09`) | no network at runtime |
| Colab "run it yourself" handoff (`G-32-11`) | offline runtime |
| CDN-loaded interaction stack — Tailwind, GSAP, TF.js (`G-32-14`) | no CDN; our offline + reduced-motion contract is **better** |
| "Unlock challenges" achievement framing (`G-32-13`) | would imply a credential; violates `claim-not-epi-certified` |
| React/Three.js simulation surface for mastery (`G-33-10`) | static HTML + vanilla JS + one CSS file |
| anime.js continuous decorative loops (`G-33-12`) | vanilla/offline; motion may not carry meaning |
| Periodic GitHub content fetch (`G-33-13`) | fully offline |

### 4.5 Where we are already better than the corpus

Not flattery — these are constraints any later change must preserve.

- The always-visible claim boundary (`learn/01-mission-critical.html:19-24`) has no equivalent
  anywhere in the corpus.
- The offline + reduced-motion contract (`course.css:1-3`, `:1141-1155`) versus a corpus artifact
  that loads seven remote dependencies to teach letters.
- A stateful empty state that names its next action, where corpus heroes offer generic CTAs.

---

## 5. User workflows — the plan is derived from these, not the reverse

### W1 · The cold start (today: broken)

Someone lands on the hub having never used this. **Today:** the hero explains `cdcp serve`,
`file://` and `CDCP_FILE_ORIGIN`; the only wayfinding panel says *"No cards due. Take a mock or
quiz, then come back."* Eight equal-weight cards follow. **Target:** the first screen names one
action tied to a specific module, and operating text moves to the error states that fire it.

### W2 · The learning loop (today: teaches, then closes the door)

A learner opens module 06, reads a unit, hits a micro-check, answers wrong. **Today:** every choice
disables, correct/incorrect marks appear, an explanation appends, and Next advances by `idx++`.
**Target:** the miss offers a retry in the same concept, and what the learner just demonstrated
changes what comes next.

### W3 · The exam (in flight — D-2/D-3/D-4)

Forty questions, sixty minutes. **Today:** submit fires in 98.9 ms with no confirmation; there is no
flag control; submit is *disabled* at 36/40 without naming which four are missing. **Target:**
submit always available with gaps named as jump links, one confirmation before the point of no
return, Submit reachable in tab order.

### W4 · The bad result (today: actively hostile)

A learner scores 6/40. **Today:** the results page rendered 15,233 px with the recovery link at
y=15,152 — the worse you do, the further recovery moves. Partially fixed: the new CTA measures
y=1,008 at 1280×900. **Target:** the review surface is budgeted, missed items come first, and the
page states what it left out rather than dumping every row.

### W5 · The return visit (today: no such concept)

A learner comes back three days later. **Today:** `results.html:46-58` shows the *current* attempt's
exam, seed, bank hash, answer count, engine. There is no prior-attempt comparison, no last-seen
time, no changed-since summary — despite `review.js:405-420` **already persisting `saved_at`**.
**Target:** a handback — what changed, when it was measured, which pack produced it.

---

## 6. Phases and the dependency DAG

```
P0 constitution ──┬──> P3 teaching loop ──┐
                  │                        ├──> P6 cold start
                  ├──> P4 state legibility ┤
                  │         │              │
                  │         └──> P5 density/recovery
                  │
P1 exam loop ─────┴──> (in flight, pane 3)
P2 five forms (bd-111m) ──> independent of P3-P6
```

| Phase | Name | Blocks | Blocked by | Status |
|---|---|---|---|---|
| **P0** | Constitution — register the learner surface | P3, P4, P5, P6 | — | not started |
| **P1** | Exam loop (D-2/D-3/D-4) | P6 (hub card) | — | **in flight** |
| **P2** | Five named forms (D-1, `bd-111m`) | — | — | filed |
| **P3** | The teaching loop | P6 | P0 | not started |
| **P4** | State legibility | P5, P6 | P0 | not started |
| **P5** | Density and recovery | — | P0, P4 | not started |
| **P6** | Cold start and hub journey | — | P0, P1, P3, P4 | not started |

**Why P0 first, non-negotiable.** Every other phase produces a learner-facing property. Without a
registry row, none of those properties can be asserted by the gate, which means every one of them is
one refactor away from silently regressing — and we would have rebuilt the exact failure mode the
project exists to refuse. P0 is also cheap: it is registry rows and gate wiring, no product code.

**Why P4 blocks P5.** Both live in `results.js`/`review.js`. Serialize them or take the merge
conflict. Same-file stories get a dependency edge, not a hope.

**Why P6 is last.** The hub's job is to route into surfaces. Routing into an undesigned teaching
loop is a redesign of a signpost pointing at the same hole.

---

## 7. The stories

Every story below carries WHAT / WHY / ACCEPTANCE / SHOULD-FAIL / GREEN-DOES-NOT-PROVE. A story
without a planted known-bad is not admissible: an empty scan set is an ERROR, not a pass.

---

### P0 · Constitution

#### S-P0-1 · Register the four learner-surface properties as claims

**WHAT:** add rows to `registries/claims.toml` for the four properties in §2 —
`claim-cold-start-names-action`, `claim-wrong-answer-retryable`,
`claim-next-is-evidence-driven`, `claim-state-is-dated`. Extend `claims_lint.toml`'s scan set to
include the learner-facing HTML and the JS that renders learner-visible strings.

**WHY:** 18 registries and zero learner rows is the structural cause of the whole RC-C epic. The
gate cannot fail on a property the constitution cannot express. This row set is what makes every
later phase enforceable rather than aspirational.

**ACCEPTANCE:** four rows registered at the correct lattice strength — these are `invariant`(6)
class properties about behaviour, not performance numbers, so a perf figure stated here is a build
error. `cdcp_registry_check` validates them and its own test suite covers the new rows: the checker
is itself checked. An empty learner-claim set is an ERROR, not a pass.

**SHOULD-FAIL:** delete one claim row and the build must fail naming it. Register a claim justified
by a weaker claim class and the lattice check must reject it (`rank(justifier) >= rank(claim)`).

**GREEN-DOES-NOT-PROVE:** registered claims prove the vocabulary exists. They prove nothing about
the surface until S-P0-2 wires assertions to them.

#### S-P0-2 · Extend `smoke_rendered_output` into a learner-surface contract harness

**WHAT:** extend `web/assets/js/smoke_rendered_output.js` (473 lines today) so each claim in S-P0-1
has at least one assertion on the **rendered string a human sees**, driven by planted state
profiles: zero-state, mid-progress, bad-result, returning-visitor.

**WHY:** `rendered-output-contracts` — a correct internal value proves nothing about what survived
to the surface. Both RC epics already cite this standard; this applies it to design properties. The
state profiles matter because every finding in this plan is a *state-dependent* defect, and a
harness that only ever sees one state cannot find them.

**ACCEPTANCE:** four profiles are fixtures. Each claim has ≥1 rendered-string assertion. The
harness fails if the profile set is empty.

**SHOULD-FAIL:** plant a zero-state profile whose hub renders no action → RED. Plant a results view
with no measured-at field → RED. Bypass the harness and the fixture must fail.

**GREEN-DOES-NOT-PROVE:** the harness proves the strings render for four synthetic profiles. It does
not prove a real learner's state produces sensible output — profiles are a **proxy** for learners,
and per RC-A the output must name that gap.

---

### P1 · Exam loop — in flight

D-2 (one Drill surface, three modes, one stateful hub card), D-3 (submit always enabled, gaps named,
partials never feed mastery), D-4 (remove the legacy footer route). Full spec and reasoning:
`docs/DECISIONS-W3-EXAM-LOOP.md`. Currently being implemented on pane 3.

**Note for later phases: the baseline is moving.** `hub_mastery.js:339-378` already selects weak →
unpracticed → unmastered with a displayed reason (`:467-483`). Any story touching hub state must
re-read those lines before citing them.

---

### P2 · Five named forms — `bd-111m`, filed

Ships as decided in D-1. Independent of P3–P6; it touches the assembler, goldens, packs and
registries, not the learner-surface JS.

---

### P3 · The teaching loop — the highest-value phase

#### S-P3-1 · A wrong micro-check answer is retryable within the same concept

**WHAT:** `learn_units.js:400-418` currently sets `data-done` on first click, disables every choice,
marks selected and correct, and appends an explanation. Add a retry path: on a wrong answer, name
the distinction, keep the concept open, and offer another attempt before advancing.

**WHY:** `G-32-02`, ranked first across all three waves. The corpus treats wrong as a teachable
state; we treat it as a terminal score event. The correction moment is lost exactly when the
misconception is visible and cheapest to fix. **The pattern already exists in this repository** —
`power-path.html:431-452` supports wrong-slot feedback and retry. This generalizes it.

**ACCEPTANCE:** a wrong answer offers retry; the explanation still appears; a correct answer on
retry is recorded as *corrected-after-miss*, not as a clean first-pass correct, because those are
different facts and merging them destroys the mastery signal. Verified in the browser at 375 px
and 1280 px.

**SHOULD-FAIL:** plant a retry that silently upgrades a miss to a clean correct in the mastery
path → RED. Plant a wrong answer that disables all choices with no retry → RED.

**GREEN-DOES-NOT-PROVE:** a retry affordance proves the control exists. It does not prove learners
use it or that corrected-after-miss is the right mastery treatment — that needs W-series evidence
we do not have.

#### S-P3-2 · Next unit is selected from demonstrated weakness

**WHAT:** `learn_units.js:544-551` advances by `idx++`. Replace with a selection that weights the
learner's own check outcomes. `learn_units.js:317-323` already persists unit position; extend that
state rather than inventing a parallel store.

**WHY:** `G-32-03`, ranked second. "Next" is currently the next row. The corpus weights by due
state, error count, success rate, recency, and confusable peers. Every later practice decision is
blind while this is an increment.

**ACCEPTANCE:** two planted profiles with different miss patterns produce **different** next
actions, asserted on the rendered string. Selection is deterministic given state — no randomness,
consistent with the purity discipline. localStorage only; no service.

**SHOULD-FAIL:** plant two distinct miss profiles that yield an identical next action → RED. That
single assertion is what makes this claim non-vacuous.

**GREEN-DOES-NOT-PROVE:** differing outputs prove the selector reads state. They do not prove the
weighting is pedagogically correct — the weights are a **proxy** for learning need and the gate must
say so.

#### S-P3-3 · Propagate the manipulable-model pattern beyond one diagram

**WHAT:** `power-path.html` is the only manipulable model across fifteen modules. Identify the
modules where a placement/reveal/retry interaction is genuinely supported by existing content, and
propagate the pattern. **Do not** invent interactions where the corpus of content cannot support
one.

**WHY:** `G-32-06`. Breadth and causal sequencing are the gap, not the absence of capability (§4.3).

**ACCEPTANCE:** each new interaction uses `fieldset`/`legend` with text-bearing labels (a KEEP), is
keyboard-complete, and honours reduced-motion. **Include at least one module deliberately left
alone, and say why** — a wave that improves everything it touches has stopped evaluating.

**SHOULD-FAIL:** plant an interaction whose accessible name is only `?` → RED. That is F-18,
already found once on the power-path nodes.

**GREEN-DOES-NOT-PROVE:** more interactions do not prove better learning. This is a breadth claim
only.

---

### P4 · State legibility

#### S-P4-1 · Surface the measured-at date and pack identity

**WHAT:** `results.html:46-58` shows exam, seed, bank hash, answers, engine — no timestamp, no
freshness label. `review.js:405-420` **already persists `saved_at`**. Surface it, plus which pack
produced the result.

**WHY:** `G-33-05` and `G-33-08`. The data exists and the surface does not show it. Cheapest story
in the plan (**S**, < 1h) with real value: a mastery signal a learner cannot date is a signal they
cannot decide to trust.

**ACCEPTANCE:** measured-at and pack identity render on results and on the mastery block. Asserted
as rendered strings.

**SHOULD-FAIL:** plant a stale profile and assert the surface says so rather than presenting stale
state as current.

**GREEN-DOES-NOT-PROVE:** a date proves recency is displayed, not that the underlying state is
correct.

#### S-P4-2 · The return handback

**WHAT:** on return, answer "what changed since I was here?" before offering another action — prior
attempt comparison, last-seen time, delta.

**WHY:** `G-33-01`, ranked third. We render a current snapshot with a missing return delta.

**ACCEPTANCE:** a returning-visitor profile renders a delta; a first-visit profile renders **no**
delta and no fabricated baseline. Both asserted.

**SHOULD-FAIL:** plant a first visit and assert no invented "improvement" appears. Fabricating a
baseline is the failure mode here and it must be impossible.

**GREEN-DOES-NOT-PROVE:** a delta proves change is displayed, not that it is the change that
matters.

#### S-P4-3 · Weight the next action by severity, not registry order

**WHAT:** `hub_mastery.js:339-378` picks weak → first unpracticed → first unmastered in **registry
order**. Within the weak list, miss count and rate do not change priority. Make severity change the
order and keep the reason legible (`:467-483` already displays one).

**WHY:** `G-33-02`. The route is computed and honest; the ordering inside it is arbitrary.

**ACCEPTANCE:** two profiles differing only in miss severity produce different orderings, with the
reason string naming the severity. Deterministic.

**SHOULD-FAIL:** plant two profiles with sharply different severity that produce identical ordering
→ RED.

**GREEN-DOES-NOT-PROVE:** severity ordering is a **proxy** for study value. Name the gap in output.

---

### P5 · Density and recovery

#### S-P5-1 · Budget the results review surface

**WHAT:** `results.js:437-508` builds one ordered list for every graded row; `results.html:92-94`
has no pagination, collapse boundary, or grouping. Introduce a budget: missed items first, an
expand-all, an item count, and an explicit statement of what was omitted.

**WHY:** `G-33-04`, ranked fourth. The 40-item wall measured 13,515 px at 375 px. **State what was
left out** — an interface that silently truncates reads exactly like one that showed everything.

**ACCEPTANCE:** measured page height before and after at 375 px and 1280 px. Missed items first.
Omission is stated, never silent. 375 px overflow KEEP re-verified.

**SHOULD-FAIL:** plant a 40-miss profile and assert the omission notice appears with the correct
count. Plant truncation with no notice → RED.

**GREEN-DOES-NOT-PROVE:** a shorter page is not a more useful one. Height is a **proxy** for
navigability.

#### S-P5-2 · Failed and ruled-out work stays visible with a retry condition

**WHAT:** repeatedly-missed and resolved items live in Drill storage and never surface on the
dashboard. Render a small local history with the condition for trying again.

**WHY:** `G-33-03`. A bad state should preserve what failed and when to retry, not become an empty
dashboard after real effort.

**ACCEPTANCE:** a profile with repeat misses renders history and a retry condition. Withholds
invented numbers when data is absent — an existing KEEP.

**SHOULD-FAIL:** plant a profile with no history and assert nothing is fabricated.

**GREEN-DOES-NOT-PROVE:** visible history is not acted-upon history.

---

### P6 · Cold start and hub journey

#### S-P6-1 · A first-run route that names one action

**WHAT:** a zero-state learner gets one named first action tied to a specific module, not a null
panel. Preserve the "No cards due" copy **for returning learners** — it is a measured KEEP and a
genuine strength; it simply stops being the first-run route.

**WHY:** `G-31-01`, `G-32-01`, and W1. Two independent readings converged: the empty state is good
*as an empty state* and harmful *as the primary first-run experience*. D-2's state-computed card is
the mechanism; this story supplies the cold branch.

**ACCEPTANCE:** zero-state renders a named module action; returning-with-nothing-due still renders
the KEEP copy. Both asserted as rendered strings.

**SHOULD-FAIL:** plant zero-state and assert the null panel is **not** the primary route. Plant
returning-nothing-due and assert the KEEP copy survives — regressing it is a net loss.

**GREEN-DOES-NOT-PROVE:** naming an action is not the same as naming the *right* action.

#### S-P6-2 · Section landmarks and honest proof

**WHAT:** the eight-card grid (`index.html:60-104`) has no owning heading or kicker. Add a landmark
that says what the group is for. Replace the implementation-property footer ("static shell ·
relative assets · no CDN") with facts a learner can act on — or drop it.

**WHY:** `G-31-02` and `G-31-03`. Our proof strip currently answers a question no learner asked.

**ACCEPTANCE:** every displayed number carries a named denominator and a real measurement. **No
invented efficacy numbers, ever** — this is where a rigor project would most easily betray itself.
Instruction density (F-23) must go down, not up.

**SHOULD-FAIL:** plant a stat with no registered denominator → build error.

**GREEN-DOES-NOT-PROVE:** a proof strip proves numbers render, not that they persuade or help.

#### S-P6-3 · Prerequisite and skip cue on Learn

**WHAT:** `NOTHING — no code exists.` `learn/01-mission-critical.html:41-50` has only a breadcrumb
and a noscript note. Add an explicit prerequisite note and a permitted skip.

**WHY:** `G-32-08`. A cold learner should get a prerequisite decision without being marched through
known material.

**ACCEPTANCE:** prerequisite renders; skip link does not displace skip-to-main-content as first in
tab order (a KEEP).

**SHOULD-FAIL:** plant a skip link ahead of skip-to-main-content in tab order → RED.

**GREEN-DOES-NOT-PROVE:** offering a skip is not knowing whether learners should take it.

---

## 8. Risks

| Risk | Why it is real here | Mitigation |
|---|---|---|
| **Proxy substitution** | The house failure mode. Option *length* stood in for distractor plausibility; length hit chance and the defect survived. `gate_shrink` used line count for "logic moved". | Every metric in this plan names its property, its proxy, and the gap **in its own output**. RC-A acceptance applies to this plan. |
| **KEEP regression** | 34 measurements could be traded away for a major, and the trade would look like progress. | KEEPs are a floor, re-verified after every phase. A fix that regresses one is a net loss, not a trade. |
| **Moving baseline** | P1 is in flight; `hub_mastery.js` already changed under Wave 3's read. | Re-read cited lines before implementing. Stale line numbers are not defects. |
| **Shared-file collision** | P4 and P5 both live in `results.js`/`review.js`; P1 and P6 both touch the hub card. | Dependency edges, not hope. Serialize same-file stories. Reserve before editing. |
| **Designing past the constraints** | The corpus is Next.js/React/Framer. Eight patterns already rejected. | §3 is the invariant box; §4.4 is the rejection record. |
| **Honesty drift under progress language** | Progress bars, streaks and mastery badges are exactly how a study tool starts implying a credential. | `claim-not-epi-certified` is load-bearing; `G-32-13` rejected achievement framing on the record. |
| **Plan churn** | If >30% of materialized beads are rewritten before being touched, plan-space was not converged. | Track churn at materialization. Above threshold: stop draining, return to review rounds. |

---

## 9. What this plan does not do

Two defects outrank everything in this document. Neither is in scope here and neither is unblocked
by any story above.

- **F-01 — the key is the only plausible option.** 31 of 117 findings merged. It is live in the
  default form's first item: three options carry absolute or impossible markers and one does not.
  A candidate answers by eliminating the three that sound wrong, with no data-centre knowledge. That
  is a content programme, gated behind the W1a/W1b measurements in the RC-A lane.
- **F-04 — the teaching/test mismatch rate.** Still the most important unmeasured number in the
  project. If it is high, a learner **cannot pass by studying the material we ship**.

**A perfectly designed surface delivering items answerable by eliminating three absolutes is still a
tool that does not teach.** This plan makes the surface honest about what it is doing. It does not
make the questions good, and no green gate produced by this plan may be read as if it did.
