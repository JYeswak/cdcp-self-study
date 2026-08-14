# Research waves — full data-centre lifecycle → knowledge graph

**Method:** `research-to-graph` dual-emit. Every agent produces `prose.md` **and**
`<slug>.fragment.json`; fragments merge into one durable graph via
`rtg-merge.py --doctor`. Nothing downstream re-extracts from prose.

**Organising scenario (the spine):** *"We have just closed on the land for a large
data-centre campus. Produce the proposal: what we order, when, from whom, under what
standard, managed with what tooling, commissioned how, operated to what procedures."*

Every lineage answers one slice of that scenario. The scenario keeps research
**actionable** rather than encyclopedic — a finding that cannot change a decision in the
proposal is not a finding.

---

## Scope note — this deliberately exceeds CDCP

CDCP's 14 modules are **operate**-weighted. This scenario spans **select → design →
procure → build → commission → operate**, which maps to CDCS/CDCE, Uptime ATD/ATS/AOS,
BICSI DCDC, and PMP/PRINCE2 territory. That is intentional: the repo's own bank already
contains 39 `15-ops-adjacent` items with no Learn surface, which is evidence the 14-module
frame is already too small for what is being assessed.

---

## SHARED PREAMBLE — prepend verbatim to every lineage prompt

```
You are one lineage in a multi-agent research wave building a knowledge graph for
data-centre lifecycle competence. Another agent is covering each adjacent lineage; do not
try to cover theirs. Depth over breadth.

=== GROUNDING RULES (violating these poisons the graph) ===

1. PAYWALLED STANDARDS. You almost certainly CANNOT read TIA-942, ANSI/BICSI 002,
   EN 50600 / ISO-IEC 22237, ASHRAE TC 9.9, NFPA 75/76, or the Uptime Institute Tier
   Standard. They are paid documents.
   - NEVER invent, guess, or reconstruct a clause/section number. A fabricated locator is
     worse than no locator: it is unfalsifiable-looking and will be cited downstream.
   - If you did not READ the text, set attrs.ground_contact = "UNGROUNDED" and put the
     secondary source you actually read in source_file.
   - Reporting "the scope of TIA-942 is X per <vendor summary>" is fine. Reporting
     "TIA-942 §5.3.2 requires X" when you never opened TIA-942 is a defect.
   - You may quote freely from genuinely open sources: NIST, DOE/FEMP/LBNL/NREL, EPA,
     Open Compute Project, public utility interconnection queues, published vendor docs,
     public postmortems, and government code portals.

2. NEVER VENDOR. Cite and link. Do not copy paid standard body text into any artifact.

3. VOLATILE FACTS MUST BE DATED. Equipment lead times, interconnection queue depths,
   pricing, and market capacity change monthly. Any such node MUST carry
   attrs.recorded_at (ISO date) and attrs.valid_from. An undated lead time is a liability.

4. DISAGREEMENT IS THE PRODUCT. Where two standards/bodies genuinely conflict — Uptime
   Tier vs TIA-942 Rated vs EN 50600 Availability Class are NOT synonyms — emit an
   explicit `contradicts` edge between the two Standard nodes with a note naming the
   precise axis of disagreement. Do not smooth this into "standards vary."

5. EVERY Tech NODE MUST EARN ITS PLACE. A tool/technology node must have an `addresses`
   edge to a Gap node. A tool list with no gap it closes is noise.

6. CONFIDENCE IS NOT DECORATION. EXTRACTED 1.0 only for something you read verbatim.
   INFERRED .95-.55 for reasoning. AMBIGUOUS .1-.3 for contested/unclear. Never 0.5.

=== DELIVERABLES ===
- <slug>.md      — prose findings, decision-relevant, no filler
- <slug>.fragment.json — the typed graph fragment (schema below). THIS is what merges.
```

Then append the skill's **THE EXACT PROMPT — dual-emit block** verbatim.

---

## The lineages

Each entry: the mission, and the specific traps that lineage must avoid.

### L1 · Site & post-acquisition due diligence
Land is closed. What now kills or delays the project? Utility interconnect queue position
and energisation date; substation distance and MW availability; water source, rights, and
stress; fibre routes and carrier diversity (how many *physically distinct* paths, not how
many contracts); geotech, seismic, flood, wildfire; zoning, entitlement, permitting
sequence; environmental/noise/air permits for generators.
**Trap:** treating "power is available" as a fact rather than a queue position with a date.
**Gold source:** public utility interconnection queues, state siting dockets — genuinely open.

### L2 · Capacity programme & density
IT load forecast, phasing, rack density trajectory (traditional vs AI/HPC at 50–150 kW/rack),
what density does to floor plan, structural load, busway, and cooling choice. Design vs
installed vs utilised capacity — three different numbers people conflate.
**Trap:** stale density assumptions. AI-era rack density has moved fast; date everything.

### L3 · Electrical topology & the resilience-class map
Utility → MV distribution → generation → UPS/DRUPS → PDU/RPP → rack A/B. N, N+1, 2N,
2(N+1), distributed-redundant/catcher. Then the map that matters: **Uptime Tier I–IV vs
TIA-942 Rated 1–4 vs EN 50600 Availability Class 1–4** — where they align, where they do
NOT, and why "Tier 3" is used wrongly in the market (self-declared vs Uptime-certified).
**Trap:** presenting the three schemes as interchangeable. They are not. `contradicts` edges required.
**Feeds:** the `cdcp_plant` topology model — emit Tech nodes for each topology pattern.

### L4 · Mechanical & thermal
Air vs liquid; direct-to-chip, rear-door, immersion; containment; chilled water vs DX vs
economiser; ASHRAE envelope classes A1–A4 and *recommended* vs *allowable* (a distinction
operators routinely get wrong); water-side vs air-side economisation; thermal ride-through
— the interval where a cooling loss becomes an IT event.
**Trap:** ASHRAE is paid. Class names and the recommended/allowable *concept* are widely
published; specific envelope numbers must be marked UNGROUNDED unless read in an open source.

### L5 · Standards, certification & training landscape
EPI CDCP/CDCS/CDCE/CDFOM; Uptime ATD/ATS/AOS; BICSI DCDC; CNet; DCD/DCPro. For each:
what it actually certifies, who accredits, exam form, recertification, and — critically —
**what it does NOT claim**. Then the design/ops standards: TIA-942, BICSI 002, EN 50600,
ISO 22237, NFPA 75/76, local code.
**Trap:** this repo must never imply EPI affiliation. Findings here are for a *domain map*,
never for reproducing proprietary syllabus body text.

### L6 · Procurement & supply chain
Long-lead equipment reality: generators, MV switchgear, transformers, chillers, UPS
modules, busway. Current lead times **with dates**. RFP/RFQ structure, vendor
qualification, factory witness testing, spares strategy, and the sequencing consequence —
which orders must be placed before design is finished, and what that costs in change risk.
**Trap:** lead times are the most volatile facts in this entire wave. Undated = worthless.

### L7 · Project management & controls
WBS, stage gates, critical path and what actually drives it (usually utility + long-lead),
EVM, change control, RACI, risk register, contingency. Tooling: Primavera P6, MS Project,
Procore, Smartsheet, Autodesk Build, BIM/digital twin handover — each as a Tech node with
an `addresses` edge to the specific Gap it closes.
**Trap:** producing a tool catalogue. If a tool addresses no named gap, cut it.

### L8 · Commissioning
Cx levels L1–L5, integrated systems test, black-building / pull-the-plug testing, load
banks, witness scripts, issues log, and the handover documentation set. What "commissioned"
must mean before IT load lands.
**Trap:** Cx level definitions vary by author — capture the variance as `contradicts`, don't
pick one silently.

### L9 · Operations
MOP/SOP/EOP structure and approval flow; maintenance strategy (run-to-fail vs preventive vs
predictive) and CMMS; DCIM and what it actually delivers vs markets; capacity management;
incident/problem/change per ITIL; on-call, escalation, blameless postmortem; drills.
**Gold source:** Google SRE (free to read), public cloud postmortems — cite, never vendor.

### L10 · Sustainability, metrics & reporting
PUE/WUE/CUE/ERE/REF — definitions, boundaries, and how each is gamed; heat reuse; carbon
accounting scopes; water stress; EU Energy Efficiency Directive reporting and equivalents.
**Trap:** PUE boundary conditions are where the dishonesty lives. Capture the boundary, not
just the number.

---

## Merge & verify (after every wave)

```bash
python3 ~/.claude/skills/research-to-graph/scripts/rtg-merge.py \
  --fragments <out>/*.fragment.json --graph <graph.json> --doctor
```

Then: shrink-guard must pass (a merge that shrinks the graph is a bug); inspect
`contradicts` edges first — they are the highest-value output; then `Gap` nodes with no
`addresses` edge, which are the curriculum's real to-do list.

---

## How this lands back in the product

1. `Gap` nodes with no `addresses` edge → candidate new modules/units.
2. `contradicts` edges → the highest-value assessment items in the entire bank, because
   they test discrimination rather than recall.
3. `Standard` nodes → the citation registry backbone (evidence conformance, not an oracle).
4. `Tech` nodes tied to topology patterns → seed the `cdcp_plant` model.
5. The scenario itself → the capstone assessment: the learner IS the owner's engineer.
