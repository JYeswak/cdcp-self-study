# A→Z — upgrading and hardening the ecosystem

**Status:** the execution plan. `ROADMAP-PHASES.md` is the scope rationale;
`ROADMAP-WAVES.md` holds the review history (v1→v4) and why two theses were killed;
`FRANKENSIM-ADOPTION.md` holds the patterns. This file is what gets materialized as beads.

**Measured baseline 2026-08-14:** 10 open beads · 2 epics · **0 dependency edges** (a flat
list, not a DAG) · knowledge graph 1,221 nodes / 1,587 edges with **213 Gap** nodes
(66 SEV-0, 131 SEV-1), 41 `Decision`, 125 `contradicts` · `check.sh` 59 ok-steps, exit 0 ·
804 bank items, all `qualitative_only`, all `objective_ids = []`.

---

## The one structural rule

**Two DAGs, hard-ordered.**

| | Engineering DAG | Curriculum DAG |
|---|---|---|
| Builds | crates, gates, ledgers | units, items, scenarios |
| Size | ~50 beads | ~213 Gap-derived candidates |
| Acceptance | *removing the guard turns a test RED* | *approved item, mapped to an objective, citing evidence* |
| Blocked by | — | **item status + assembly filter (C)** |

Nothing from the curriculum DAG may land before **C**. `BankItem` has no status field and
`cdcp_assemble` samples every loaded item, so content written today drops into the same
undifferentiated pool as the 804 — a pool already known to contain an undetectable duplicate.

---

## A · Truth — DONE (commit 467b429)

Factual-content L3 → **NO**; dual-path renamed to cross-target conformance; SRS → short-interval
review (verified against `INTERVAL_STEPS=[1,3]`); module 15 disclosed; corpus `access="free"`
split into `rights`/`redistribution`/`ai_ingestion`. check.sh 59 steps, exit 0.

## B · Ledgers — evidence stops living in prose

Adopt frankensim's machine ledgers. **B is first because it is what makes A's failure impossible
to repeat**: `L3: YES · wired` survived for months because a maturity claim lived in a table
nobody could mechanically falsify.

- **B1 `capability-maturity.toml`** — every capability row carries `level`, `owner`,
  `last_review`, and `evidence[]` pointing at a **named test function**. `staleness_days = 90`.
  A row whose review date expires fails the build.
- **B2 `goldens-couplings.toml`** — **live defect.** Each golden declares the semantic surfaces
  it depends on, at pinned version consts, with a written justification. Today `UPDATE_GOLDENS`
  lets grader semantics change and goldens re-freeze with nothing recording that a surface moved.
- **B3 doc-facts inventory** — generalise `verify_doc_consistency.py` beyond milestone tables to
  all load-bearing prose, tying claims to the artifacts that substantiate them.
- **B4 step-count receipt** (`bd-1sd.13`) — the third number in README's gate sentence is still
  hand-maintained while the other two are enforced.

Each ships with a known-bad injection and a meta-test PAIR. The wording that stood here —
"delete the assertion → selftest RED" — was retired in `.flywheel/CHARTER.md` on 2026-08-14 as
incoherent: deleting an assertion WEAKENS a test, it cannot make it fail. The pair is
(1) mutate the gate → suite non-zero; (2) mutation still in place, delete the assertion → suite
back to zero; (3) restore by **writing bytes, never a rename**, then force a build and confirm it
rebuilt something — see `docs/TESTING.md`, "Meta-test pairs: step 3 is the one that rots".

## C · Status & filter — the gate everything else waits on

- **C1** `status: draft|approved|retired` on `BankItem`; `cdcp_assemble` samples **approved only**.
- **C2** `bank_hash` repaired to cover `objective_ids` [[fact:fact-hash-payload-covers-objective-ids=yes]],
  citation ids, `tags` and `status` [[fact:fact-hash-payload-covers-status=yes]] —
  **LANDED 2026-08-14**. `hash_payload()` now covers every modelled field, asserted by
  `hash_payload_covers_every_modelled_field` [[fact:fact-hash-payload-parity-test-exists=yes]],
  and `deny_unknown_fields` [[fact:fact-bank-item-denies-unknown-fields=yes]] makes a stray
  field a load error rather than a silent discard. `objective_ids` had never been a field on
  `BankItem` at all: all 804 files carried it on disk and serde dropped it. Residual: the
  domain string is still `cdcp-bank-v1` under a v2 definition (`bd-6ycw`).
- **C3** near-duplicate detector (`bd-near-duplicate-item-gate-i5v`). Exact-stem hashing finds
  **0** duplicates; `m14-q040`/`m14-q121` are one item twice with different keys.
- **C4** portable PRNG — **LANDED 2026-08-14.** Sampler is `rand_chacha::ChaCha12Rng`,
  crates pinned `rand = "=0.8.7"` / `rand_chacha = "=0.3.1"`. It no longer seeds from
  `StdRng` [[fact:fact-assemble-uses-stdrng=no]]. Seed-42 stream equals the pre-C4
  `StdRng`/rand-0.8.7 stream, so `item_ids` did not move.
- **C5** module 15 decision — teach it or exclude it from assembly. Record which and why.
- **C6** `min_modules` is no longer accepted and discarded — the parameter reaches
  `sample_item_ids` and a pool spanning too few modules is a `ModuleShortfall` error, so the
  binding is not `_min_modules` any more [[fact:fact-assemble-discards-min-modules=no]]. It is
  enforced as a **precondition on the approved pool only**; whether the 40 *selected* items span
  `min_modules` is still unchecked. Close that half, or state the weaker guarantee.

## D · Evidence spine — `cdcp_evidence`

frankensim's `Evidence<T>` adapted: `NumericalCertificate` (lo/hi enclosure) ·
`ModelEvidence { cards, assumptions, validity: ValidityDomain, discrepancy_rel, in_domain }`.
Composition conservative by construction — domains **intersect**, discrepancy **adds**,
enclosures round **outward**. `in_domain` goes false the moment any constituent is queried out
of domain, and propagates.

Plus `SourceArtifact` / `ClaimRecord` / `ReviewRecord` / `ItemEvidence`, and the licence policy
enforcing A's three-field split. Gate: an **approved** item making a normative claim with no
resolvable evidence row is RED. Zero citations scanned is an ERROR.

## E · Data corpus — `cdcp_data`

Licence-gated snapshot loader: **refuses to load an artifact whose `.meta.toml` lacks a licence
line**. No network I/O — snapshots pinned at build time, preserving offline-first. `content.lock`
extended to cover data artifacts (constellation.lock pattern).

Priority vendoring, all PD-GOV: **OSHA/eCFR** (the only normative safety text we may quote in
full — and note L9's correction that 1910.147 *excludes* electric-utilization installations,
which sends electrical work to Subpart S), NOAA climate, USGS seismic, FEMA flood, EPA eGRID,
EIA, NICE framework.

## F · Computation + the first real external oracle

`cdcp_site` (lat/lon → climate bin, seismic PGA, flood zone, grid carbon, power price) and
`cdcp_metrics` (PUE/WUE/CUE/ERE with **explicit boundary declarations** — the boundary is where
the dishonesty lives; L10 traced the 1.8 L/kWh water constant to a source that excludes hydro
twice).

**The differential harness that finally makes L3 honest:** compute for known locations, compare
against published reference values from sources we do not control. Disagreement = RED. Plus
**MMS-style manufactured cases** with closed-form answers for every computed quantity.

## G · Typed assessment — `cdcp_assess`

Beyond A–D: multi-select, ordering, numeric-range, topology-selection, procedural-sequence.
Integer/rational arithmetic only, same fixtures native and WASM. **Without G, every computed
scenario flattens back into four letters and we have built a better answer-key generator inside
the same recognition interface.**

## H · Curriculum from the graph — *unblocked by C*

213 Gap nodes → units. Highest-value first:

- **125 `contradicts` edges** — these test discrimination, not recall. Uptime vs TIA-942 vs
  EN 50600 are not synonyms; that conflation is the field's signature competence failure.
- **Module 15's 28-Gap curriculum** (L9), each with a named free-to-redistribute source,
  including *evidence hygiene* — teaching learners to ask who measured the domain's most-quoted
  statistic, using its unverifiability as the worked example.
- **The quantitative hole**: 804/804 items are `qualitative_only`. Cause identified —
  `forbid_uncited_numeric = true` (correct) plus **zero capacity/density topic atoms** means the
  policy forbids exactly the items the syllabus needs. Fix by supplying citable numbers, never
  by relaxing the policy.
- **Thermal ride-through**: 0 of 121 cooling items mention it, and it is the mechanism that
  turns cooling loss into an IT event.
- **Commissioning**: ~2 substantive propositions in 804.

## I · Scheduling — `cdcp_schedule`

Pure scheduling, injected time, versioned state, migration rules, reference fixtures. Compiled
to WASM; removes the duplicated JS scheduling law. Either adopt a versioned reference
implementation with conformance fixtures or keep calling it short-interval review.

## J · PX — plant model, gated

Pre-registered falsification experiment, **outside the production bank**, per `ROADMAP-WAVES` §5.
Power-first finite-state service availability. Says *"service-preserving under the model"*, never
*"safe"* — OSHA Subpart S and NFPA 70E are the authorities on safe, and they are citable.
Kill conditions written before it runs. **D's `ValidityDomain` replaces the crude
refuse-cooling-wholesale line**: model what you can, refuse out-of-domain structurally.

## K · Scenario capstone

The owner's-engineer proposal: land closed → what to order, when, to what standard, with what
tooling, commissioned how. Consumes E/F for a real site. The graph already holds 41 `Decision`
nodes and a live worked example: **a chiller ordered today (20–30 wk) lands after the
2027-01-01 ≥700 GWP install prohibition (20.0 wk out)** — legal to buy, illegal to install.

## L · Attempt capture — no psychometrics yet

Record `item_version, bank_hash, learner_pseudonym, mode, exposure_count, chosen_option,
correctness, latency, timestamp, prior_attempts`. **Do not build `cdcp_stats` before there is
data** — difficulty and discrimination are sample-dependent and unreliable at small N.

## M · Distribution

Publish the licence-audited public data corpus as a standalone contribution. Publish the
lifecycle knowledge graph. Neither depends on the study tool being finished.

---

## Dependency graph

```
A (done)
 └─► B ──► C ──┬─► D ──► E ──► F ──► K
               ├─► G ──────────────► K
               ├─► H  (curriculum — gated on C)
               ├─► I
               └─► J (PX, gated: pass/kill)
M rides alongside from E
L rides alongside from G
```

## Materialization

Per `planning-workflow` MATERIALIZE: epics solo and slugged first (capture real ids), children
via `br create -f`, **`dag-validate-gate` BEFORE any `br` write** (br drops cycle-forming edges
with a stderr warning and exit 0 — your DB stays acyclic while your DAG silently loses edges),
`br dep cycles` + `br lint` + `br graph --all` after, then `br sync --flush-only`.

**Materialize the engineering DAG (A–G, I–M) now. Materialize H after C lands** — Gap-derived
curriculum beads written before item status exists would encode acceptance criteria that cannot
be satisfied.

**Churn tripwire:** if >30% of materialized beads are rewritten or obsoleted before they are
ever touched, plan-space was not converged — stop draining and re-review the affected epic.

---

## Claims referenced

This plan discusses byte-exact grading analytically — including in §A/§F where it records that the
property does NOT extend to factual correctness. The claim itself is registered, not asserted here:
[[claim:claim-grade-byte-exact]].
