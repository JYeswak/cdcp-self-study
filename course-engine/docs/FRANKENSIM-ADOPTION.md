# frankensim adoption — the patterns we take, and what each one fixes

**Source:** `/Volumes/ZestData/dicklesworthstone-mirror/frankensim` (Studio-local mirror).
~155 Rust crates, Cargo workspace `resolver = "3"`, Cargo.lock committed. Read 2026-08-14.

**Scope — I got this wrong first time and the correction matters.**

I initially called this "a decade-class effort." Measured from the mirror's git history:

| | |
|---|---|
| first commit | 2026-07-05 |
| last commit | 2026-08-13 |
| **span** | **39 days** |
| commits | 3,987 (~102/day) |
| Rust | 1,649 files · **1,706,066 lines** |
| beads | **2,976** |

**Wrong by two orders of magnitude.** And the shape matters more than the size:

- `COMPREHENSIVE_PLAN_FOR_FRANKENSIM.md` — **840 lines**, committed **2026-07-05**
- first crate committed **2026-07-06** — *one day later*
- `COMPREHENSIVE_PLAN_TO_EXTEND_FRANKENSIM_TO_NEW_DOMAINS.md` — 3,025 lines, written **later**

**The plan was 840 lines and code started the next day. The plan grew as it shipped.**

That is a direct rebuke of how this project has been run today: four roadmap versions, three
adversarial review rounds, and bead materialization deferred four times on "not at steady-state."
Two of those rounds were load-bearing — codex killed two wrong theses on substance. The rest was
gate-on-gate.

**Revised guidance: we are not depending on the solver stack because we do not need multiphysics
— not because the scale is unreachable.** The scale is reachable. Take the patterns, materialize
the DAG, and let the plan grow while crates land.

---

## 0. Why this matters — it independently confirms our review findings

`fs-convection`, verbatim:

> **"Formula implementation can be numerically verified; physical prediction quality cannot."**

That is codex round 3's three-layer objection — solver correctness and scenario correctness are
mechanizable, **plant correctness is not** — arrived at independently, in shipped Rust.

`fs-evidence`, verbatim:

> *"Without model-form evidence, FrankenSim can produce beautifully certified WRONG answers —
> mesh error 0.7%, residual irrelevant, but turbulence-closure discrepancy 8–15%, so the design
> ranking is not decision-grade."*

**"Beautifully certified wrong answers" is exactly what our v3 plant model would have shipped.**
Codex said we had moved the human assertion into the graph, where it "wears mathematical
clothing". frankensim hit the identical wall and built the fix instead of abandoning the model.

---

## 1. `Evidence<T>` — certificates travel INSIDE values

The core pattern. A value carries four uncertainty slices; the certificate cannot fall off,
because it is part of the type.

| Slice | Carries |
|---|---|
| `NumericalCertificate` | `kind` (bound strength) + `lo`/`hi` enclosure of the quantity |
| `StatisticalCertificate` | e-values / confidence half-widths |
| `ModelEvidence` | which model cards, whose assumptions, validity domain, discrepancy band |
| `SensitivitySummary` | `d_qoi: {param → d(qoi)/d(param)}` |

```rust
pub struct ModelEvidence {
    pub cards: Vec<String>,          // model cards in play (sorted, deduped)
    pub assumptions: Vec<String>,    // union of stated assumptions
    pub validity: ValidityDomain,    // composition INTERSECTS
    pub discrepancy_rel: f64,        // composition ADDS (first-order conservative)
    pub in_domain: bool,             // false the moment ANY constituent was queried out of domain
}

pub struct ValidityDomain { bounds: BTreeMap<String, (f64, f64)> }
```

**Composition is conservative by construction:** numerical enclosures round outward, validity
domains intersect, discrepancy bands add. You cannot launder a weak input into a strong
conclusion — the arithmetic of the certificate forbids it.

**`in_domain` is the killer field.** It is `false` the moment *any* constituent was queried
outside its domain, and it propagates. That is mechanized abstention.

### What it fixes for us

Codex's fatal objection to v3 was that every omitted control supply, tie, or derating rule
becomes an **invisible axiom**. `Evidence<T>` makes assumptions *visible, attached, and
non-bypassable*. It does not make plant correctness mechanizable — nothing does — but it makes
the model state, in data, exactly how much it does not know.

---

## 2. `ValidityDomain` + model cards — the honest fidelity line, solved properly

`fs-convection`: every formula has one `CorrelationCard`, uses the shared
`fs_evidence::ValidityDomain`, and **"refuses missing or out-of-domain groups."**

**This replaces the crude fidelity line in `ROADMAP-WAVES.md` §5.** We wrote "power-first,
refuse cooling because we cannot model it honestly." The better design: model what you can,
attach a `ValidityDomain` to every correlation, and let out-of-domain queries **refuse
structurally**.

Codex demanded exactly this as a PX kill condition — *"an explicit scope/abstention result when
a question exceeds the semantics"*, and *"either out-of-scope case receives a confident computed
answer rather than abstention"* as a fail. frankensim implements it as a type, not a check.

`fs-convection` also states the discipline we must copy verbatim: *"Empirical-card discrepancy
allowances remain model-form evidence, and analytic limiting rows name their idealizing
assumptions."*

---

## 3. `fs-mms` — Method of Manufactured Solutions

Manufactured-solution refinement ladders with convergence-rate checking. You choose an exact
analytic solution, derive the source term producing it, run the solver, and assert convergence
at the theoretical rate.

**This is a genuine external oracle** — mathematics the project does not control. It verifies
*solver correctness* (codex's mechanizable layer 1) without any claim about plant fidelity.

For us: any computed quantity (cut sets, capacity headroom, free-cooling hours) gets a
manufactured case with a known closed-form answer, and the gate asserts we hit it.

---

## 4. `golden-couplings.json` — the gap in OUR goldens

> *"Every golden/replay hash declares the upstream SEMANTIC SURFACES it depends on and the
> surface version it was frozen against. Surfaces declare a version const in source; `xtask
> check-goldens` fails when a source const drifts from this registry or a golden's pin drifts
> from a surface row — an upstream semantic change must point at every downstream golden to
> re-freeze deliberately, never silently."*

Schema `frankensim-golden-couplings-v1`: **138 surfaces, 25 goldens**.

```json
surface: {"id":"fs-adjoint:dwr-accept", "file":"crates/.../dwr_accept.rs",
          "const":"DWR_EVIDENCE_IDENTITY_VERSION", "version":7,
          "domain":"fs-adjoint-dwr-accept-identity-v7", "schema_fingerprint":"v7-ae65033..."}
golden:  {"golden":"fs-euler-disc-e2e:frozen-context", "const":"FROZEN_CONTEXT_HASH_HEX",
          "depends_on":"fs-euler-disc-e2e:scientific-contract=1,fs-evidence:vv-artifact=3",
          "justification":"..."}
```

### What it fixes for us — this is a live defect

We have byte-exact goldens and **no coupling registry**. Our `UPDATE_GOLDENS` path is a silent
escape hatch: change grader semantics, regenerate goldens, gate stays green, and nothing records
that a semantic surface moved. frankensim makes re-freezing **deliberate and justified** — the
golden names the surfaces it depends on, at pinned versions, with a written justification.

**Adopt:** `goldens-couplings.toml` mapping each golden to the semantic surfaces (bank schema,
grade report shape, canonical-JSON rules) at pinned version consts, checked in `check.sh`.

---

## 5. `capability-maturity.json` — evidence ledger with enforced staleness

Schema `frankensim-capability-maturity-v1`, `staleness_days: 90`, 15 capabilities.

```json
{"id":"evidence.colour-algebra","title":"Evidence colours and conservative composition",
 "level":"L2","owner":"jemanuel","last_review":"2026-07-22","crates":["fs-evidence"],
 "evidence":[{"kind":"contract","ref":"crates/fs-evidence/CONTRACT.md"},
             {"kind":"test","ref":"crates/fs-evidence/tests/conformance.rs::evd_001_g0_composition_conservativeness_battery"}]}
```

Every capability claim points at a **named test function**, has an **owner**, a **level**, and a
**review date that expires**. Levels defined in `docs/MATURITY_LEVELS.md` and enforced by tooling.

### What it fixes for us

Our maturity claims live in CHARTER prose — which is precisely how we ended up asserting
`L3 External oracle: YES · wired` on internal-only evidence for months. A machine ledger where
each claim names a test and goes stale in 90 days makes that failure **expire loudly**.

---

## 6. `doc-facts-inventory.json` — prose cannot outrun the code

Schema `frankensim-doc-facts-inventory-v1`: **156 manifests, 156 contracts, 676 integration
tests**. The doc-facts inventory ties documentation claims to the artifacts that substantiate
them.

We have `claims-lint` (L1) which resolves prose markers to registry rows. This extends it: every
*contract* and *manifest* is inventoried, so a doc claim without a backing artifact is
detectable. Our `verify_doc_consistency.py` does this for roadmap status only — this generalises
it to all load-bearing prose.

---

## 7. `constellation.lock` — ecosystem pinning (L7)

Schema `frankensim-constellation-lock-v2`, `identity_domain: org.frankensim.xtask.constellation-lock.v1`,
`lock_hash: 057a58b75b10639e`, **7 libraries**.

> *"lock_hash covers (lib, version, git_head) only — paths are …"*

Cross-repo deps pinned by `git_head` under a covering `lock_hash`, schema-versioned, validated at
bootstrap. Our `content.lock` covers content; this is the pattern for pinning **data snapshots**
in roadmap phase P3 (NOAA/USGS/FEMA/eGRID vendored corpora) — each dataset pinned by content hash
under a covering lock, so a corpus change cannot slip in unnoticed.

---

## 8. The V&V crate cluster — what each is for

| Crate | Role |
|---|---|
| `fs-vvreg` | the Gauntlet G1/G2 benchmark & **V&V registry** |
| `fs-checker` | standalone **evidence-package checker** — the checker is its own artifact |
| `fs-propcheck` | in-house property-based testing **with integrated shrinking** |
| `fs-uq` | uncertainty quantification that **wraps solvers** (propagation, not point estimates) |
| `fs-conform` | restriction-map plugin **conformance SDK** |
| `fs-evidence-runner` | frozen, bounded runner contracts; declarations + pure validation only |

`fs-propcheck` is notable: they wrote their own property-testing crate with shrinking rather
than take a dependency, because the shrinker is part of the evidence story.

---

## 9. Adoption plan — mapped onto our phases

| Take | Into | Priority |
|---|---|---|
| `goldens-couplings.toml` + check | **P1** (status/hash work) | **first — fixes a live silent-refreeze hole** |
| `capability-maturity.toml` w/ staleness + test refs | **P1** | first — kills prose-maturity claims |
| `Evidence<T>` four-slice type | **P2** `cdcp_evidence` | high |
| `ValidityDomain` + `in_domain` propagation | **P2/P6** | high — this IS the abstention mechanism |
| Model cards w/ stated assumptions | **P2** | high |
| MMS-style manufactured cases | **P4** (`cdcp_site`/`cdcp_metrics`) | high — real external oracle |
| Data-snapshot lock (constellation pattern) | **P3** | medium |
| doc-facts inventory generalisation | **P2** | medium |
| Property testing w/ shrinking | **P4/P5** | medium |
| UQ wrapping | later | low — needs real data first |

### What this does NOT change

Codex round 3's rejection of v3 **stands**, and `fs-convection` independently confirms its core.
`Evidence<T>` does not make plant correctness mechanizable. It makes the model *state its own
ignorance in data*, which is a different and achievable thing. PX remains a gated falsification
experiment with pre-registered kill conditions.

The honest upgrade is this: instead of refusing cooling wholesale, attach validity domains and
let out-of-domain queries refuse structurally — and record `in_domain: false` when they do.

---

## 10. STOP PLANNING. The build sequence.

frankensim's birth order was: plan (840 lines) → beads → crates, one day apart. Ours is
over-planned and under-built. The remaining work is not another review round.

**Immediate — no new design needed, all four are small and unblock everything:**

1. **P0 honesty patch.** CHARTER L3 → NO; rename dual-path to cross-target conformance; fix the
   CLI/WASM test counts (5 and 3, not 0); disclose the 14-Learn/15-bank mismatch; stop calling
   the 1d/3d ladder SRS. Pure edits, hours.
2. **`goldens-couplings.toml`** — the live hole. Our `UPDATE_GOLDENS` path lets grader semantics
   change and goldens re-freeze silently. Copy frankensim's surface/version/justification schema.
3. **`capability-maturity.toml`** with `staleness_days` and test refs. Kills prose-maturity
   claims like the L3 one, permanently, by making them expire.
4. **Item status `draft|approved|retired` on `BankItem` + assembly filter.** The prerequisite
   for every content decision, and the reason nothing generated can land safely today.

**Then materialize the full bead DAG** per `planning-workflow` MATERIALIZE — epics solo and
slugged, children via `br create -f`, `dag-validate-gate` BEFORE any `br` write, `br dep cycles`
after. Not incrementally. The whole graph, from `ROADMAP-PHASES.md` P0–P8 plus this document's
adoption table.

**Then build in this order** (each is one crate, each has a named contract):
`cdcp_evidence` (Evidence<T> + ValidityDomain) → `cdcp_data` (licence-gated snapshot loader) →
`cdcp_site` + `cdcp_metrics` (MMS-verified) → `cdcp_assess` (typed items) → PX `cdcp_plant`.

**Churn tripwire stays:** if >30% of materialized beads are rewritten before being touched,
plan-space was not converged and we stop and re-review. That is the honest guard against
over-correcting from too-much-planning into too-little.

---

## Claims referenced

This document discusses the byte-exact grading property analytically. The claim itself is
registered, not asserted here: [[claim:claim-grade-byte-exact]].
