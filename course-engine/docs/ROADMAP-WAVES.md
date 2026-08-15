# ROADMAP — waves (v4 DRAFT, 2026-08-13)

> ## v3 was REJECTED at review round 3. Read this before anything below.
>
> v3 proposed an executable plant model that would **compute** answer keys, on the thesis
> that "the key becomes a theorem rather than an opinion." Round 3 rejected it as the **same
> category error as v1, in more sophisticated form**:
>
> - v1 mistook *citation integrity* for factual validation.
> - v3 mistook *deduction from an authored model* for truth about a plant.
>
> A model makes an answer logically necessary **given declared premises**. It cannot make the
> premises true. Three layers, only two mechanizable:
>
> | Layer | Mechanizable? |
> |---|---|
> | Solver correctness — implementation evaluates declared semantics correctly | yes |
> | Scenario correctness — given model M and failure set F, outcome O follows | yes |
> | **Plant correctness** — modelled dependencies/capacities/failure modes match reality | **NO** |
>
> Every omitted control supply, normally-open tie, protection dependency, derating rule or
> shared route becomes an **invisible axiom**. Common-cause is the sharpest case: the model
> computes consequences of every common cause the author *remembered*.
>
> **Also flagged: scope substitution.** This repo explicitly models a 40-question study exam
> and forbids treating its score as a credential (`exam_form.toml`, `ORACLE-GAUNTLET.md`).
> v3 changed the target to "operational competence", then faulted the existing system for
> missing the newly-substituted target. W1 must establish the construct before engineering
> is allowed to define it.
>
> **Language cut from the plan as unsupported:** "unlimited items", "safe switching", "the key
> is a theorem rather than an opinion", the aviation/nuclear simulator equivalence (NRC/FAA
> qualification requires proof-of-match data and continuing validation — computation alone is
> nowhere near sufficient; calling this a full-scope simulator is borrowed prestige), the
> "safety-critical" framing (data centres are **mission**-critical), and any competitor
> superlative.
>
> **The thesis permitted into v4:**
> *For fully specified, versioned power-service scenarios, derive answer keys from a REVIEWED
> finite-state dependency/capacity model, expose assumptions and witnesses, and empirically
> test whether generated tasks measure intended reasoning more efficiently than matched
> human-authored items.*
>
> **The human reviewer wins now, plainly** — time-to-value, cost, breadth, defensibility. The
> model requires that reviewer anyway. Hire the reviewer first; make the model earn its place.

---

# (v2 body retained below — superseded where it conflicts with the v4 header)

# ROADMAP — waves (v2 DRAFT, 2026-08-13)

**Status:** DRAFT · planning artifact, not doctrine · review round 2 complete
**Method:** `planning-workflow` — plan converges in markdown, then the whole bead DAG is
materialized at once.
**Wave IDs are `W<n>`, not `M<n>`** — `verify_doc_consistency.py` parses milestone tables
across CHARTER/README/PHASE-NEXT; a draft must not inject phantom milestones.

> **v1 → v2 is a structural rewrite, not a polish pass.** An adversarial review
> (codex `gpt-5.6-sol`, high reasoning) falsified the central v1 thesis and caught a
> measurement error in the "verified state" table. Both are recorded in §1 rather than
> quietly corrected, because the failure mode is the point.

---

## 0. The one-sentence goal

**v1 said:** make the content as provably correct as the grader.
**v2 says:** make the **score interpretation** defensible — establish what a score means,
what it does not mean, and what evidence supports the claim.

Correctness of individual facts is necessary but subordinate. An item can be perfectly true
and still measure nothing.

---

## 1. Corrections to v1 — recorded, not hidden

### 1a. I reported `cdcp_cli` as having ZERO tests. It has 5.

`crates/cdcp_cli/tests/cli.rs` holds 5 `#[test]`s; `cdcp_wasm/tests/` holds 3 more. My probe
counted only `crates/*/src/`, silently excluding Rust's conventional `tests/` directory. The
figure went into the plan, into the review prompt as "verified current state," and into a
status report.

**This is the exact defect class the repo's drift guard exists to prevent** — a hand-derived
number that reads as measured. The lesson is not "count better"; it is that a grep whose
scope is invisible in its output is indistinguishable from a correct one.

### 1b. The "flat coverage" finding was the wrong metric, chosen because it fit.

v1 claimed 13 modules × 9 units proved quota-driven content. Measured properly:

| module | bank items | | module | bank items |
|---|---|---|---|---|
| 06 power | **136** | | 05 lighting | 31 |
| 09 cooling | **121** | | 07 emf | 32 |
| 11 network | 72 | | 04 floor/ceiling | 33 |
| 12 fire | 63 | | 08 racks | 34 |

The bank **is** weighted toward power and cooling, ~4× over lighting/EMF. Unit count is a
packaging artifact: `cdcp_learn` (`crates/cdcp_learn/src/units.rs`) mechanically emits one
unit per `##` heading, so it measures outline shape, not depth. v1 selected the metric that
supported its thesis.

**W3 (rebalance) is therefore CUT.** The premise was false.

### 1c. Defects v1 missed entirely

- **Module 15 is assessed but never taught.** `domains.toml` declares `15-ops-adjacent`;
  the bank holds **39** items for it; `web/content/modules/` has **14** modules and no
  module 15. `cdcp_assemble` samples across every bank module, so a learner can be scored on
  material the Learn surface does not contain. This is a live fairness defect.
- **All 804 items have `objective_ids = []`.** Zero populated. `verify_objectives.py` admits
  there is no objective→item matrix. Bloom labels are self-asserted.
- **~~`bank_hash` omits load-bearing fields.~~ FIXED 2026-08-14 (C2).** `hash_payload()` now
  covers `objective_ids` [[fact:fact-hash-payload-covers-objective-ids=yes]], `citation_ids`,
  `tags` and `status` [[fact:fact-hash-payload-covers-status=yes]] alongside the original seven,
  and a test asserts the payload key set equals the serde field set so the two cannot drift
  again. Note the consequence: because `status` is now covered, retiring an item moves
  `bank_hash` by itself —
  which inverts the property `goldens/PROVENANCE.md` §"Bank drift" was written against. Residual:
  the domain string still reads `cdcp-bank-v1` under the v2 definition (`bd-6ycw`).
- **`StdRng` is not portable.** Closed 2026-08-14 (C4): the sampler no longer
  seeds from it [[fact:fact-assemble-uses-stdrng=no]]. It is `ChaCha12Rng` [[fact:fact-assemble-rng-is-chacha12=yes]] from
  `rand_chacha = "=0.3.1"`. The word "currently" was the tell; the named
  algorithm is the fix.
- **SRS is oversold.** `web/assets/js/review.js` (was `srs.js`) is a 1-day/3-day ladder capped at 3 days. That is
  short-interval review, not Anki-like long-term scheduling. Law is now `cdcp_schedule`.
- **The public-domain assumption was too casual.** The DOE/FEMP guide has NREL contractor
  authors and embeds ASHRAE-sourced figures. 17 U.S.C. §105 does not automatically place
  contractor work in the public domain. Redistribution is **UNRESOLVED**, not "yes".

---

## 2. What survived review

**L3 is not an external factual oracle** — upheld, with a sharper framing. Native-vs-WASM
dual-path is a legitimate *same-implementation cross-target conformance* check. The error is
CHARTER aggregating that under "L3 External oracle", which implies factual validation it does
not perform. Rename the check; mark factual-content L3 **absent**.

**What did NOT survive:** that a citation gate fixes it. A citation gate proves referential
integrity — the ID resolves, the digest is unchanged. It cannot prove the source *supports the
keyed answer*, *excludes the distractors*, or *applies in context*. Call it **evidence
conformance**, never a factual oracle.

**Postmortem falsification largely collapses.** The refutation edge `incident → claim_id` is
authored by a human; the "gate" then re-reads that human's judgment and reports it as
mechanized. Worse, the seed incidents do not falsify correct N+1 claims — Google
`europe-west2-a` and AWS Tokyo are **common-mode / control-dependency** failures that exceed
N+1's one-independent-failure antecedent. They are excellent teaching material about
common-cause failure; they are not counterexamples to N+1.

And the absolutist-word detector is unusable as designed: **603 of 804 items** contain
`always|never|only|guarantee|eliminate` in a *distractor* but not the key. The bank uses
absolutes as deliberately-wrong options. The detector would drown in its own noise.

**Mechanizable residue worth keeping:** an executable **capacity graph / fault tree** where
component removals are enumerated. If an item claims survival of every single-component
failure, a generated one-component counterexample is a genuine RED — computed, not looked up.

---

## 3. Rights posture (revised)

| Source | Ship it? | Role |
|---|---|---|
| NIST SP 800-series | per-artifact review | cite; scope is **server security**, not power/fire/structural |
| DOE/FEMP/LBNL guide | **UNRESOLVED** — contractor authorship + ASHRAE figures | cite; scope is **energy-efficient design** only |
| Google SRE, cloud postmortems | no | cite + link |
| ASHRAE · Uptime · TIA-942 · BICSI 002 · EN 50600 | no (paid) | cite by clause; licensed-reviewer attestation only |

**The available free corpus cannot validate the 14-domain bank.** Two narrow-scope documents
do not cover electrical protection, fire code, structural load, Tier/Rated terminology, or
operational switching. Any plan claiming otherwise is stretching sources past their authority.

---

## 4. Waves (resequenced)

### W0 — Honesty patch *(small, unblocks everything)*
Change factual-content L3 to **NO**; rename dual-path to cross-target conformance; correct the
CLI/WASM test counts; disclose the 14-Learn/15-bank mismatch; downgrade DOE/NIST to
per-artifact rights review; stop calling the 1d/3d ladder SRS.

### W1 — Assessment validity + bank quarantine *(was: 804 citations — CHANGED)*
1. State intended score interpretations **and explicit non-claims**.
2. Job-task/knowledge/skill blueprint, reviewed by someone who has actually run a data hall.
3. Item status `draft | approved | retired`; **only approved items enter mocks**.
4. Map active items to observable objectives.
5. Deep-audit a **smaller** active bank. Retire giveaway-distractor items. **Do not preserve
   804 as a vanity metric.**
6. Decide module 15: give it a Learn surface, or exclude it from assembly.

### W2 — Structured tasks (the competence claim)
8–12 deterministic scenarios with pure Rust/WASM scoring: minimal cut set that drops a row;
order an MOP/EOP safely; reject unsafe switching; three-phase load and headroom; interpret an
alarm timeline; distinguish *recommended* vs *allowable* vs *code* vs *design intent*.
MCQs stay valid for terminology — they just cannot carry an operational-competence claim.

### W3 — ~~Rebalance by domain weight~~ **CUT** (premise falsified, §1b)

### W4 — Rust/WASM contracts
One `cdcp_evidence` crate (not separate cite/incident ceremony); `cdcp_assess` (MCQ,
multi-select, ordering, numeric-range, topology, procedural — integer/rational only);
`cdcp_schedule` (injected time, versioned state, removes duplicated JS scheduling law);
`cdcp_engine` orchestration. Fix `bank_hash` coverage, name a portable PRNG, enforce
`min_modules`, extend native/WASM parity through assembly and shuffling. Keep `cdcp_cli` thin
and **expand** its 5 tests rather than "rescuing" it from zero.

### W5 — Attempt-event capture *(no psychometrics yet)*
Record `item_version, bank_hash, learner_pseudonym, mode, exposure_count, chosen_option,
correctness, latency, timestamp, prior_attempts`. **Do not build `cdcp_stats` before there is
data** — difficulty and discrimination are sample-dependent and unreliable at small N.

---

## 5. Graph (v4 — dependency inverted, round 3)

v2 was internally inverted: W2 promised structured topology/procedural tasks while the typed
assessment schema needed to *represent* them sat in W4. Split and reversed:

```
W0
 │
 ▼
W1a  score claims · job blueprint · item status+filter · bank/content hash · module-15 decision
 ├──────────────────────┐
 ▼                      ▼
W1b                     PX
human bank audit        plant-model FALSIFICATION experiment
(reviewer-led)          (OUTSIDE the production bank)
 └──────────┬───────────┘
            ▼
        pass / kill
            │
            ▼
W4a  typed assessment schema + model/proof schemas
            │
            ▼
W2b  reviewed production scenarios
            │
            ▼
W4b  native/WASM parity + full gates
```

**Why PX cannot touch the production bank:** `BankItem` has no status field and
`cdcp_assemble` samples every loaded item. Generated items would enter the same
undifferentiated pool. W1a's status+filter is a hard prerequisite.

### PX — the falsification experiment (pre-registered kill conditions)

Four dev scenarios, then **freeze** DSL, semantic version, query types, failure-event
ontology, generator templates, abstention rules. *Post-freeze special cases count as evidence
against tractability.* Then 8 unseen in-scope + **2 deliberately out-of-scope** (selective
coordination, thermal transient). SME A authors keys blind to the modeller; SME B reviews the
rendered graph before solver output is revealed.

**Kill if:** either out-of-scope case gets a confident answer instead of abstention · <80% of
held-out models survive first review without a key-changing correction · any accepted query
disagrees with both SME judgments · optimised solver and independent exhaustive enumeration
disagree on any small graph · <80% of generated items accepted by both SMEs without rewriting
· one generator-family flaw invalidates sibling items · marginal time per accepted item is not
≥50% below the matched manual workflow · think-aloud learners solve via template cues rather
than dependency reasoning.

### Honest fidelity line

**Power-first, finite-state service-availability.** NOT a unified power+cooling plant model —
cooling is where the abstraction turns dishonest fastest: `chiller → CRAH → containment → rack`
is not a power path, containment does not deliver fungible capacity, and airflow/recirculation/
sensor placement matter even when aggregate capacity looks sufficient. Zonal cooling capacity
may be a **stated scenario input**; rack inlet conditions may never be *derived* from
reachability.

May claim: "under maintenance state M, modelled event F removes all qualifying power paths to
rack R" · "capacity < declared demand" · "minimal cut set over the enumerated basic-event
universe" · "this sequence passes through a state with no remaining redundant feed."

May **not** claim: electrically or occupationally *safe* · selective coordination · arc-flash ·
grounding/LOTO correctness · real thermal ride-through · Tier/Rated compliance · real
availability probability · exhaustive common-cause coverage. Use **"service-preserving under
the model"**, never "safe".

**Distractor rule from v3 was incoherent and is cut.** A size-*n+1* set containing a minimal
cut set still causes the outage, so it is not a distractor to "which set causes failure"; and
if the question asks for minimality, cardinality leaks the key. Distractors need documented
**misconception transforms** — overlooking a shared bus, confusing component with path
redundancy, treating isolation as failure, ignoring capacity, assuming dual cords imply
independent feeds. Minimality is over **basic events**, not component count.

W3: CUT (v2 §1b).

---

## 6. Open — cannot be resolved inside the plan

1. **OQ-10 paid SDO spend.** Escalation-class. Without licensed text, normative claims
   ("code requires…") rest on secondary sources.
2. **Who reviews?** W1 needs a competent operations reviewer. Absent that, "approved" means
   only "the author agreed with themselves."
3. **Incident frequency is a convenience sample** — hyperscaler, spectacular, publishable. It
   cannot weight occupational importance.

---

## 7. Review status

| Round | Reviewer | Outcome |
|---|---|---|
| 1 | author | thesis: external factual oracle |
| 2 | codex `gpt-5.6-sol`, xhigh | **thesis falsified**; measurement error found; W1 resequenced, W3 cut |
| 3+ | pending | — |

Round 2 produced structural revisions, so this is **not** at steady-state and **not** eligible
for bead materialization. Churn from v1 to v2 is far above the ~30% tripwire — by design; that
is what a real review round looks like when the plan was wrong.

---

## Claims referenced

This document discusses byte-exact grading and domain-coverage analytically — including where
those properties do NOT extend to factual correctness (§2). The claims themselves are
registered, not asserted here: [[claim:claim-grade-byte-exact]] ·
[[claim:claim-domain-covered]] · [[claim:claim-syllabus-mapped]].
