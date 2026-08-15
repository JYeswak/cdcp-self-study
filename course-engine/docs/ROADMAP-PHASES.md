# ROADMAP — phases (2026-08-13)

**Status:** DRAFT · supersedes the wave framing in `ROADMAP-WAVES.md` for scope; that file
retains the review history (v1→v4) and the falsification discipline. Read both.

> ⚠️ **Verification debt.** This session exhausted its web-search budget. Every dataset
> below is named from prior knowledge with its licence *asserted, not verified today*. **No
> dataset ships until its licence line is read and recorded verbatim** (see the anti-vacuous
> rule at the end). Treat every "PD" tag here as a hypothesis.

---

## 0. The reframe

Three review rounds killed two theses:

- **v1** — citation integrity mistaken for factual validation.
- **v3** — deduction from an authored model mistaken for truth about a plant.

Both failed the same way: a mechanism that emits confident output was called truth. The common
flaw is that **we authored the premises**.

**Public government datasets do not have that flaw.** They are external, authoritative about
physical reality, machine-readable, versioned, and public domain. NOAA did not ask us what the
weather was. If we compute free-cooling hours for a site and published climate analysis
disagrees, **we are wrong** — and that is what an oracle means.

**This is the L3 external oracle the project has been missing.** Not standards prose — data.

---

## 1. Legal tiers — the only gate that matters for "pull in everything public"

| Tier | Examples | May we vendor into a PUBLIC repo? |
|---|---|---|
| **PD-GOV** US Government work | NIST, NOAA, USGS, FEMA, EIA, EPA, FERC, OSHA/eCFR | **yes** — 17 U.S.C. §105, no copyright |
| **PD-GOV\*** gov-published, contractor-authored | DOE/FEMP/LBNL/NREL guides | **VERIFY PER ARTIFACT** — §105 does not automatically cover contractor work, and embedded third-party figures never become PD |
| **OPEN-LIC** permissive/share-alike | OCP specs, OpenStreetMap (ODbL), Wikidata (CC0/CC BY-SA) | **yes with attribution** — ODbL is share-alike, check reciprocity |
| **RO** read-only | NFPA (all 421), ASHRAE read-only set, Google SRE | **never** — ground only, record locators |
| **PAID** | TIA-942, BICSI 002, EN 50600, ASHRAE TC 9.9, Uptime Tier Standard | **never** — cite by clause only |

**The rule that makes "pull in everything" safe:** vendoring is decided per artifact by a
recorded licence line, never by tier assumption. A blank licence field is an **ERROR**, not a
default-permissive.

---

## 2. The public data corpus — what each dataset actually unlocks

This is the new material. Each row is a candidate `knowledge/data/<slug>/` with a pinned
snapshot, a `.meta.toml`, and a content hash.

| Dataset | Tier | Unlocks |
|---|---|---|
| **NOAA hourly climate / TMY** | PD-GOV | **free-cooling & economiser hours for any site**; ASHRAE design conditions (dry/wet bulb); the single highest-value dataset here |
| **USGS seismic hazard** | PD-GOV | PGA by lat/lon → structural and rack-restraint requirements |
| **FEMA flood layers (NFHL)** | PD-GOV | flood zone → siting go/no-go, insurance posture |
| **EPA eGRID** | PD-GOV | **grid emission factors by subregion → real carbon math**, not a made-up constant |
| **EIA-860 / EIA-923 / electricity price** | PD-GOV | generation mix, capacity, industrial power price by state → opex modelling |
| **FERC / ISO interconnection queues** (CAISO, ERCOT, PJM, MISO, SPP) | PD-GOV | **queue position and energisation timing — the real critical path** L1 identified |
| **OpenStreetMap** | ODbL | substations, transmission lines, fibre routes, existing DC locations |
| **OSHA regs via eCFR** (29 CFR 1910.147 LOTO, 1910.269, subpart S) | PD-GOV | **normative safety text that is FREE and quotable** — the layer the plant model may never compute |
| **eCFR / Federal Register** | PD-GOV | EPA generator air permitting, emissions thresholds |
| **NIST SP 800-series** | PD-GOV | security controls (already partially in corpus) |
| **NIST NICE framework** | PD-GOV | **job-task/knowledge/skill blueprint scaffolding for W1a** |
| **DOE/FEMP/LBNL/NREL guides** | PD-GOV\* | design best practice — verify authorship page first |
| **OCP specs** | OPEN-LIC | rack/power-shelf/busway/liquid-cooling hardware reality |
| **Public postmortems** | cite-only | failure-mode corpus (L3 characterised these correctly) |

**Two of these deserve emphasis:**

- **OSHA via eCFR is free, normative, quotable US law.** Review round 3 forbade the plant model
  from ever claiming a sequence is "safe". OSHA 1910.147 (LOTO) and 1910.269 are the actual
  authority on that, in the public domain. The safety layer stops being a blind spot and
  becomes *citable primary text* — the only normative material in this whole plan we can quote
  in full.
- **NICE framework** gives W1a's job-task blueprint a public, external scaffold instead of an
  invented taxonomy.

---

## 3. Crate architecture

Existing: `cdcp_core` (exam machinery) · `cdcp_bank` · `cdcp_assemble` · `cdcp_grade` ·
`cdcp_wasm` · `cdcp_cli` · `cdcp_registry_check`.

Proposed additions, each pure, deterministic, WASM-parity, `#![forbid(unsafe_code)]`:

| Crate | Contract |
|---|---|
| **`cdcp_data`** | Vendored-snapshot loader. Content-addressed, licence-gated: **refuses to load an artifact whose `.meta.toml` lacks a licence line.** No network I/O — snapshots are pinned at build time, matching the offline-first invariant. |
| **`cdcp_site`** | lat/lon → climate bin, seismic PGA, flood zone, grid carbon factor, power price. Pure function of vendored data. **This is where the external oracle bites:** outputs are checkable against published analyses. |
| **`cdcp_metrics`** | PUE/WUE/CUE/ERE with **explicit boundary declarations** (the boundary is where the dishonesty lives) + free-cooling hours from NOAA. Integer/rational arithmetic. |
| **`cdcp_evidence`** | One crate, not separate cite/incident ceremony (round 2). `SourceArtifact`, `ClaimRecord`, `ReviewRecord`, `ItemEvidence`, licence policy. |
| **`cdcp_assess`** | Typed assessment beyond A–D: multi-select, ordering, numeric-range, topology-selection, procedural-sequence. **Prerequisite for everything interesting** — today `ChoiceLetter` is 4 letters and assemble hard-codes 4 choices. |
| **`cdcp_plant`** | Power-first finite-state dependency/capacity model. **GATED behind PX** — the falsification experiment, outside the production bank. |
| **`cdcp_schedule`** | Pure scheduling, injected time, versioned state. Replaces the JS 1d/3d ladder currently mislabelled as SRS. |

---

## 4. Phases

Ordering rule: **nothing that produces content may land before the machinery that can mark
content approved.** `BankItem` has no status field and `cdcp_assemble` samples every loaded
item, so anything generated today drops into the same undifferentiated pool as the 804.

### P0 · Honesty patch *(small, unblocks all)*
Factual-content L3 → **NO**; rename dual-path to cross-target conformance; correct CLI/WASM
test counts; disclose the 14-Learn/15-bank mismatch; downgrade DOE/NIST to per-artifact rights
review; stop calling the 1d/3d ladder SRS.

### P1 · Construct + status *(the true prerequisite)*
Intended score interpretations and explicit **non-claims** · job-task blueprint (scaffold on
NICE) · `draft|approved|retired` on `BankItem` · assembly restricted to approved · objective
mapping · bank_hash repaired to cover objective_ids/citation/status · module-15 decision ·
portable PRNG (closed 2026-08-14: named `ChaCha12Rng` [[fact:fact-assemble-rng-is-chacha12=yes]], crates pinned;
`StdRng` is no longer the seeder [[fact:fact-assemble-uses-stdrng=no]]).

### P2 · Evidence spine
`cdcp_evidence` · citation registry · reading-pass output from RO sources (NFPA 75/70E/110/111,
ASHRAE 90.4/202/G36) · **OSHA/eCFR vendored as quotable primary text** · gate: an *approved*
item making a normative claim with no resolvable evidence row is RED. Anti-vacuous: zero
citations scanned is an ERROR.

### P3 · Public data corpus
`cdcp_data` + licence-gated loader · vendor NOAA/USGS/FEMA/eGRID/EIA/OSM snapshots with
`.meta.toml` + hashes · `content.lock` extended to cover data artifacts · **licence-missing =
build failure**, with a known-bad injection proving it trips.

### P4 · Computation + the real oracle
`cdcp_site` + `cdcp_metrics`. **The differential harness that finally makes L3 honest:**
compute free-cooling hours / design conditions / grid carbon for known locations and compare
against published reference values from sources we do not control. Disagreement = RED.
*This is the first genuinely external validation in the project's history.*

### P5 · Typed assessment
`cdcp_assess`. Without it, every computed scenario flattens back into A–D and we have built
"a better answer-key generator inside the same recognition interface."

### P6 · PX — plant model, gated
Pre-registered falsification experiment per `ROADMAP-WAVES.md` §5. Power-first, finite-state,
service-availability only. Says "service-preserving under the model", never "safe". Kill
conditions written before it runs.

### P7 · Scenario capstone
The owner's-engineer proposal: *land closed → what do we order, when, to what standard, with
what tooling, commissioned how.* Consumes P3/P4 data for a real site. This is the assessment
that measures something an interviewer would recognise.

### P8 · Ops discipline + distribution
MOP/SOP/EOP authoring and critique · incident command · blameless postmortem form (cite SRE,
never vendor) · attempt-event capture schema (**no psychometrics until there is data**) ·
public release of the data corpus as a standalone contribution.

---

## 5. Dependency graph

```
P0 ──► P1 ──┬──► P2 ──► P4 ──► P7
            ├──► P3 ──►─┘      ▲
            ├──► P5 ────────────┘
            └──► P6 (PX, gated: pass/kill)
P8 rides alongside from P2
```

---

## 6. What would make this genuinely best-in-class

Not the quiz bank. Three things nobody else ships:

1. **An offline, deterministic, WASM-capable site evaluator** built on public-domain data —
   climate, seismic, flood, grid carbon, power price — that a learner can run on any lat/lon.
2. **A published, licence-audited public data corpus** for data-centre education. That is a
   contribution in its own right, independent of the study tool.
3. **Evidence conformance with recorded human verification** — every normative claim traceable
   to a locator a person checked, with the paywalled ones honestly marked ungrounded.

## 7. Still missing (round 3, unresolved by any of the above)

Transfer evidence to job performance · hands-on/BMS/EPMS exposure · human factors
(communication, escalation, handover, fatigue) · qualified instructors and SME reviewers ·
regional AHJ variation · accessibility and fairness analysis · standard setting and
reliability · longitudinal retention · **a competitive benchmark defining what "best" means**.

**Do not claim "best in the world" without that benchmark.** It is currently undefined, which
makes the claim unfalsifiable — the same failure mode as v1 and v3, one level up.

---

## Claims referenced

This document discusses the byte-exact grading property analytically. The claim itself is
registered, not asserted here: [[claim:claim-grade-byte-exact]].
