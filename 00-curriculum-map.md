# Curriculum Map — 15 Modules

Maps this free self-study program to the **publicly advertised** syllabus headings: **14 facility domains** (mission-critical site through auxiliary systems) plus **2.1 Operational Considerations**, taught here as Module 15. Objectives are original educational targets for **standard (interview-ready)** depth.

**Sources for domain list (public):** EPI CDCP course page syllabus headings; authorized partner outlines (e.g. HPE HK258S module titles); EXIN/EPI preparation-guide **2.1** topic titles. No proprietary EPI text is reproduced.

**Count:** 15 taught modules. Modules 01–14 are the public facility headings. Module 15 is the ops-adjacent supplement for **2.1** (`exam_weight_unknown` — not one of the 14 marketing facility domains; the bank still assesses it).

---

## At-a-glance

| # | Module | Public CDCP domain | Est. hours (standard) |
|---|---|---|---|
| 01 | The Mission Critical Site | The Mission Critical Site | 2 |
| 02 | Data Centre Standards | Data Centre Standards | 2 |
| 03 | Location, Building & Construction | Data Centre Location, Building and Construction | 2–2.5 |
| 04 | Raised Floor & Suspended Ceiling | Raised Access Flooring and Suspended Ceiling | 2 |
| 05 | Light | Light | 1.5 |
| 06 | Power Infrastructure | Power Infrastructure | 6–8 |
| 07 | Electro Magnetic Fields (EMF) | Electro Magnetic Fields (EMF) | 1.5 |
| 08 | Equipment Racks | Equipment Racks | 1.5–2 |
| 09 | Cooling Infrastructure | Cooling Infrastructure | 5–7 |
| 10 | Water Supply | Water Supply | 1 |
| 11 | Scalable Network Infrastructure | Designing a Scalable Network Infrastructure | 3–4 |
| 12 | Fire Protection | Fire Protection | 2–2.5 |
| 13 | Physical Security & Safety | Physical Security and Safety | 2 |
| 14 | Auxiliary Systems | Auxiliary Systems | 2–2.5 |
| 15 | Operational Considerations | 2.1 Operational Considerations (ops-adjacent supplement; not a 14-domain facility heading) | 2–3 |

**Total (standard):** ~30–43 hours including review (14 public facility domains ~28–40 + Module 15 ~2–3).

---

## Module 01 — The Mission Critical Site

**Public domain topics:** Business organization; types of data centres; importance of a data centre; elements of a data centre; causes of unavailability.

### Learning objectives
By the end of this module you can:

1. Explain how the data centre sits in business continuity and revenue/risk language (not only “IT”).
2. Differentiate major DC types (enterprise, colo, hyperscale, edge, modular, telco) and typical SLA drivers.
3. List primary elements of a DC (white space, grey space, MEP, network, security, ops) and who owns them.
4. Distinguish first-class unavailability **objects** (power path — UPS / ATS / generator; cooling plant, often as a cascade; network/fiber; external events) from human/process as a **contributing mechanism** on those objects — not a third peer pie slice. Module 15 owns contributor-vs-root and refuses an unverifiable human-error majority statistic.
5. Relate availability percentages (“nines”) to annual downtime and discuss why five-nines is costly and hard.

### Interview anchors
- “Walk me through what makes a site mission-critical.”
- “A hall is dark: start at the power path, not a three-bucket pie. Where do you look first?”

### Notes path
`modules/01-mission-critical-site/`

---

## Module 02 — Data Centre Standards

**Public domain topics:** Standards and guidelines; standards for sub-components; international vs national standards.

### Learning objectives
1. Map the standards landscape: **ANSI/TIA-942-C (2024)** (**Rated**, not Tier), **Uptime** Tiers as **three plaques** (TCCD / TCCF / TCOS), **ISO/IEC 22237** **alongside** **EN 50600** (Availability Class 1–4 and separate Protection Classes), national electrical/fire codes, **ASHRAE** thermal guidelines, vendor best practices. **ISO/IEC 30134** KPIs (PUE / WUE / CUE) are named here; WUE depth is Module 10. W-classes (W17 / W27 / W32 / W40 / W45 / W+) are a loop decision pointed at Module 09. Cx / ASHRAE Guideline 0 is named as design vs as-built vs process; Module 15 owns isolation (HMI bypass ≠ isolation).
2. Distinguish *standard* vs *guideline* vs *code* (enforceable) vs *certification scheme*. Hold the third noun: **Rated ≠ Tier ≠ Availability Class**. Never Class 3 = Rated-3 = Tier III. Uptime Tier is a separate commercial rating system—know the difference without conflating brands. “Tier III equivalent” is a marketing defect to probe.
3. Identify which bodies typically govern power, fire, cabling, environmental conditions, and security/access. Name-and-kill **99.982% = Tier III** (and the I–IV downtime table); Tier is topology, not a nine — no replacement percentage, no nines-to-Tier crosswalk.
4. Explain why multi-standard environments create compliance and design trade-offs (e.g. international owner + local AHJ). AHJ/code wins. Fit-out order: codes + landlord → owner TIA/ISO/EN → ASHRAE → cabling → fire listings → OEM → Tier last as a claim to verify.
5. Locate public overviews of TIA Rated, Uptime’s three plaques, and EN/ISO Availability Class at a conceptual level suitable for interviews—not audit-level citation, and not a conversion table.

### Interview anchors
- “Which standards would you check before a white-space fit-out?”
- “TIA-942 vs local electrical code—who wins?”
- “Rated, Tier, Availability Class — same thing?”

### Notes path
`modules/02-data-centre-standards/`

---

## Module 03 — Location, Building & Construction

**Public domain topics:** Site location criteria; facility criteria; supporting facilities and functions.

### Learning objectives
1. Evaluate site factors: flood/seismic/weather risk, power grid quality, fiber routes, latency to users, security/threat environment, zoning, expansion land.
2. Describe building attributes: structural loading, slab vs multi-story, ceiling height, column grid, loading docks, generator yards, fuel storage constraints.
3. List supporting facilities: NOC, security ops, staging, storage, workshops, admin, parking, utility intake, water, waste heat rejection areas.
4. Connect location choices to cost, risk, and time-to-repair (spares, technician access, dual utility feeds feasibility).
5. Flag “hidden” adjacency issues (rail lines, flight paths, chemical plants, RF sources—preview of EMF module).

### Interview anchors
- “What would make you walk away from a potential colo site?”
- “How do supporting spaces fail a design even when white space looks fine?”

### Notes path
`modules/03-location-building-construction/`

---

## Module 04 — Raised Access Flooring & Suspended Ceiling

**Public domain topics:** Standards; types of raised floor; loading factors; guidelines; grounding; ramp/landing; suspended ceiling; impact on cooling.

### Learning objectives
1. Compare raised-floor vs slab/non-raised designs and when each is preferred today.
2. Explain loading: concentrated vs uniform load, rolling load, pedestal systems, stringer types.
3. Describe floor grounding/bonding practices and ESD considerations at a facilities level.
4. Specify accessibility: ramps, landings, tile cuts, cable egress, safety.
5. Explain airflow roles of floor and ceiling plenums (supply/return) and how bad tile placement destroys cooling efficiency.
6. Discuss suspended ceiling use for cable pathways vs return air vs aesthetics/fire smoke management trade-offs.

### Interview anchors
- “Why might a new AI hall skip raised floor?”
- “How does a missing floor tile take down a row?”

### Notes path
`modules/04-raised-floor-suspended-ceiling/`

---

## Module 05 — Light

**Public domain topics:** Light measurements; standards; fixture connection/positioning; emergency light; types of emergency light.

### Learning objectives
1. Use basic photometric language (lux/fc, uniformity) appropriate for facilities walkthroughs.
2. Apply practical placement rules: aisles, above cabinets (avoid glare on LEDs/LCDs), maintenance areas.
3. Differentiate normal vs emergency vs egress lighting and battery/central inverter concepts.
4. Connect lighting to energy (LED retrofit, controls) and to safety (emergency duration requirements conceptually).
5. Note interaction with cameras/security and with hot/cold aisle visibility.

### Interview anchors
- “What lighting issues show up on a site audit?”
- “Emergency lighting vs UPS-backed lighting—what’s the point of each?”

### Notes path
`modules/05-light/`

---

## Module 06 — Power Infrastructure

**Public domain topics:** Sustainability; microgrid; transformers; generators; ATS/STS; redundancy levels/techniques; distribution/busbar; single vs three phase; grounding/bonding; isolation transformers; PDU form factors; IP grades; power quality; sizing; HPC; UPS; parallel UPS; batteries; BESS; thermographic scanning.

### Learning objectives
1. Draw the **critical power path** from utility intake through transformation, generation, transfer switches, UPS, PDUs/busway, rack PDUs to IT load.
2. Compare redundancy models conceptually: N, N+1, 2N, distributed redundant; active-active vs capture.
3. Explain ATS vs STS roles and failure modes (break-before-make, transfer time).
4. Characterize UPS topologies at interview level (VFI/online double-conversion vs line-interactive concepts) and parallel configurations.
5. Discuss battery technologies, autonomy, BESS as grid/flexibility asset, and maintenance (impedance, thermal runaway awareness).
6. Apply power quality ideas: voltage, frequency, harmonics, grounding, isolation; when thermography is used.
7. Size at a conceptual level: kW vs kVA, power factor, derating, diversity—know what engineers calculate and what ops must never exceed.
8. Relate HPC/AI racks (high density kW/rack) to feeder and UPS design pressure.
9. Touch sustainability: PUE awareness, renewable/microgrid concepts without greenwashing.

### Interview anchors
- “Explain N+1 vs 2N to a CFO.”
- “What does a brownout look like on the floor before the UPS saves you—or doesn’t?”

### Notes path
`modules/06-power-infrastructure/`

---

## Module 07 — Electro Magnetic Fields (EMF)

**Public domain topics:** Types of EMF; units; standards/best practices; sources; shielding.

### Learning objectives
1. Distinguish electric vs magnetic field concerns relevant to IT equipment and people (high-level).
2. Identify common sources: busbars, transformers, UPS rooms, elevators, RF, welding, adjacent industrial load.
3. Use units and “distance falls off quickly” intuition; when to call a specialist survey.
4. Describe mitigation: separation, orientation, shielding, cable routing, room layout.
5. Connect EMF issues to data corruption myths vs real interference cases—speak carefully and evidence-based.

### Interview anchors
- “Would you put a UPS next to the network core? Why/why not?”
- “What would trigger an EMF survey?”

### Notes path
`modules/07-emf/`

---

## Module 08 — Equipment Racks

**Public domain topics:** Standards; dimensions; rack types; security; power strips/rails.

### Learning objectives
1. Use standard rack language: 19" EIA, RU height, depth, width, open frame vs cabinet, seismic.
2. Specify airflow-aware rack features: blanking panels, brush strips, chimneys, front/rear door types.
3. Plan power strips/rack PDUs: A+B feeds, metering, outlet types, whip management, fail-open risk.
4. Apply physical security: lockable doors, side panels, intelligent locks, cage vs cabinet.
5. Address weight, seismic bracing, grounding bars, and cable management that doesn’t block exhaust.

### Interview anchors
- “How do you prevent bypass airflow in a dense rack?”
- “What goes wrong with ‘just add another PDU’?”

### Notes path
`modules/08-equipment-racks/`

---

## Module 09 — Cooling Infrastructure

**Public domain topics:** Cooling principles; temperature/humidity; system types; raised-floor cooling; non-raised; supplemental; containment; liquid cooling; Seasonal Thermal Energy Storage (STER).

### Learning objectives
1. State heat transfer basics: remove heat at the chip → room → plant → outside; sensible vs latent where relevant.
2. Apply ASHRAE-style environmental envelopes conceptually (recommended vs allowable ranges; humidity/static risk).
3. Compare CRAC/CRAH, chilled water, DX, free cooling, evaporative approaches at a systems level.
4. Design airflow strategies: hot/cold aisle, containment (HAC/CAC), flooded room, in-row, rear-door HX.
5. Explain raised-floor vs hard-floor cooling and when supplemental cooling appears.
6. Cover **liquid cooling** interview vocabulary: direct-to-chip, rear-door, single-phase vs two-phase immersion (conceptual), CDU role, leak detection.
7. Introduce STER / seasonal thermal storage as an efficiency concept (store coolth/heat across seasons where climate allows).
8. Discuss controls, setpoints, and how bad setpoints waste energy or risk condensation.

### Interview anchors
- “Containment or more CRAC tons—which first?”
- “How does liquid cooling change the facilities conversation for AI racks?”

### Notes path
`modules/09-cooling-infrastructure/`

---

## Module 10 — Water Supply

**Public domain topics:** Importance of water; backup water supply.

### Learning objectives
1. Explain where water is critical: humidification, evaporative cooling, chilled-water makeup, fire systems (where used), personnel.
2. Identify failure modes: municipal outage, contamination, freezing, makeup water quality for cooling towers.
3. Describe backup strategies: tanks, dual feeds, wells (where permitted), prioritization of critical loops.
4. Connect water risk to cooling capacity and to environmental sustainability debates (WUE).

### Interview anchors
- “Your city issues a boil-water notice—what in the DC cares?”
- “Why do some designs minimize on-site water use?”

### Notes path
`modules/10-water-supply/`

---

## Module 11 — Designing a Scalable Network Infrastructure

**Public domain topics:** Importance of network cabling; planning; copper; fibre; TIA-942 cabling topologies; testing/verification; redundancy; site-to-site connectivity.

### Learning objectives
1. Argue why structured cabling is a 10–15+ year facility asset, not a “network team only” detail.
2. Plan pathways: trays, baskets, underfloor, overhead, separation from power, fill ratios conceptually.
3. Compare copper (Cat6A etc.) vs multimode/single-mode fibre use cases and distances.
4. Describe TIA-942-style topologies at conceptual level (e.g. hierarchical distribution, redundancy of pathways and spaces—MDA/HDA/ZDA style thinking without memorizing clause numbers).
5. List test/verify basics: certification testers, polarity, loss budgets, labeling, as-built docs.
6. Design redundancy: diverse paths, dual Meet-Me / carrier entry, site-to-site dark fibre / waves / IP diversity.

### Interview anchors
- “How do you keep cabling from becoming spaghetti in year three?”
- “What does diverse fiber entry actually mean on a site plan?”

### Notes path
`modules/11-scalable-network-infrastructure/`

---

## Module 12 — Fire Protection

**Public domain topics:** Causes of fire; suppression requirements; standards; detection; water-based systems; gas-based systems; fire classes; best practices; handheld extinguishers; signage/safety; regulatory requirements.

### Learning objectives
1. List common DC fire causes (electrical, batteries, hot work, external spread) and prevention culture.
2. Explain detection layers: smoke (VESDA/aspirating vs spot), heat, flame—and early-warning value.
3. Compare suppression families: wet pipe, pre-action, clean agent / inert gas, water mist—trade-offs for IT spaces.
4. Use fire classes (A/B/C/… regional variants) and correct extinguisher selection at a safety-briefing level.
5. Cover life safety: egress, signage, interlocks with HVAC/dampers, abort switches, agency (AHJ) authority.
6. Stress: design and discharge decisions require licensed fire professionals—your job is literate collaboration.

### Interview anchors
- “Why pre-action instead of wet pipe over servers?”
- “What happens to cooling and dampers on fire alarm?”

### Notes path
`modules/12-fire-protection/`

---

## Module 13 — Physical Security & Safety

**Public domain topics:** Components for physical security; components for physical safety.

### Learning objectives
1. Layer security: perimeter → building → DC suite → cage/rack; mantraps, badges, biometrics, anti-tailgating.
2. Describe surveillance, intrusion detection, visitor management, loading dock controls.
3. Separate **security** (protect assets/data) from **safety** (protect people): PPE, LOTO awareness, arc flash boundaries (conceptual), confined spaces, slip/trip, emergency response.
4. Connect physical controls to compliance narratives (SOC2/ISO visitor logs, etc.) without overclaiming.
5. Discuss insider threat and social engineering as facilities realities.

### Interview anchors
- “How would you harden a colo cage without breaking fire code?”
- “Security wanted a single exit for control—why might safety refuse?”

### Notes path
`modules/13-physical-security-safety/`

---

## Module 14 — Auxiliary Systems

**Public domain topics:** Monitoring challenges/requirements; EMS; BMS; DCIM; water leak detection; alarm panels; notification; best practices.

### Learning objectives
1. Distinguish **BMS** (building: HVAC plant, generators, access integration…), **EMS** (environmental: temp/humidity/leak), and **DCIM** (IT + facilities inventory, capacity, sometimes power chain visibility).
2. List what must be monitored: power chain points, cooling plant, room environment, leaks, fire panels, security, capacity (kW, space, ports).
3. Design notification: severity tiers, on-call, avoid alarm fatigue, escalation paths.
4. Explain water leak detection placement (under raised floor, around CDUs, chilled water valves).
5. State best practices: single pane vs integrated stack, change control for setpoints, sensor calibration, mapping alarms to runbooks.

### Interview anchors
- “BMS says fine, DCIM says rack is hot—who do you trust first?”
- “How do you stop 500 emails from becoming zero response?”

### Notes path
`modules/14-auxiliary-systems/`

---

## Module 15 — Operational Considerations (2.1)

**Public domain topics (2.1):** Service catalog; Service Level Management (SLA / OLA / underpinning contract); data-centre organizational structure; training-program requirements; safety roles; security matrix (role × zone × privilege); maintenance-agreement content; floor management; monitoring activities; document-management steps (create, review, approve, issue, supersede, archive); vendor management (select, score, UC, performance).

This is the **ops-adjacent** Learn surface (`15-ops-adjacent`). It is **not** one of the 14 public facility headings. Exam weight is unknown; the item bank still samples it. Module 01 Q3 / the old three-bucket sentence is retired here.

### Learning objectives
1. Map the 2.1 operational-considerations headings (catalog, SLM/SLA/OLA/UC, org, training, safety roles, security matrix, maintenance agreements, floor management, monitoring, document management, vendor management) as a control set, not a facilities add-on.
2. Distinguish **MOP / SOP / EOP** and a procedure’s **level of use** (use-each-time vs reference) — the attribute that decides behaviour at 03:00.
3. Specify a **maintenance contract / SLA** in measurable, CMMS-auditable terms (response, restore, spare, proof) as an **underpinning contract**, not as the customer SLA itself.
4. Treat human/process as a **contributing mechanism** on a power/cooling/network object — contributing factors, plural — and refuse an unverifiable “human error is the majority root cause” statistic. Unlearn the Module 01 Q3 peer-bucket cartoon.
5. State what **as-built / labelling / document currency** are for, walk the document-management lifecycle, and why documentation that diverges from the plant is more dangerous than missing documentation.
6. Walk four **2026 EOPs**: isolate a leaking CDU without killing the pod; Li-ion / BESS fire (not Class A); load-shed when ride-through is seconds; grid curtailment / BTM islanding. Plant in M06 / M09 / M12; procedure here.

### Interview anchors
- “Walk me through the last MOP you executed.”
- “Contributing factors or root cause — which sentence survives a postmortem?”
- “What is in the catalog, and which OLA makes that SLA true?”
- “CDU leaking on row 12 — what do you isolate?”

### Notes path
`modules/15-ops-adjacent.md`

---

## Cross-module themes (integrate continuously)

| Theme | Appears in |
|---|---|
| Availability & risk | M1, M6, M9, M12–M15 |
| Standards & AHJ | M2, M3, M12, M13 |
| Energy & sustainability | M5, M6, M9, M10 |
| Human error / process | M1, M13, M14, M15 |
| High-density / HPC / AI | M6, M8, M9, M11 |
| Documentation & labeling | M8, M11, M14, M15 |

---

## Suggested practice progression

| After modules | Practice focus |
|---|---|
| M1–M3 | “Site go/no-go” scenario memo |
| M4–M6 | Draw power + airflow on one diagram |
| M7–M9 | Dense rack thermal + EMF layout critique |
| M10–M12 | Life-safety + water dependency table |
| M13–M15 | Monitoring, access control, and 2.1 ops storyboard |
| All | 15-minute facility tour narrative |

Practice files live under `practice/`; add your own scenarios as you study.

---

## Completion checklist (standard depth)

- [ ] Each module’s learning objectives self-rated ≥ 4/5  
- [ ] Power path and cooling path drawn from memory  
- [ ] Can name and contrast ≥ 3 fire suppression approaches for IT space  
- [ ] Can explain BMS vs DCIM vs EMS  
- [ ] Can discuss liquid cooling options without claiming hands-on install skill  
- [ ] Can separate a contributing factor from a primary failure object (M15)  
- [ ] Honesty note understood: no official cert claimed  

---

*Curriculum map for independent study. Not affiliated with EPI or EXIN. Domains aligned to public CDCP syllabus headings only.*
