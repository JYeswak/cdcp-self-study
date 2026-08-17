# CDCS coverage ledger — 2026-08-17

Public syllabus source: [EPI CDCS syllabus](https://www.epi-ap.com/services/1/3/5/Certified_Data_Centre_Specialist_%28CDCS%29).
The live bank contains 27 TOML items. This ledger separates a public heading
mapping from an attribution-ready item: a source that exposes only a topic
summary is not treated as a clause or formula.

## Eleven-module ledger

| # | Exact public EPI module | Live bank | Current standard or primary anchor | Status and next action |
| --- | --- | --- | --- | --- |
| 1 | Data Centre Design/Life Cycle Overview | 2: `cdcs-lifecycle-risk-analysis`, `cdcs-lifecycle-aging-horizon` | ISO/IEC 22237-1:2021 Clauses 5 and 7, plus public abstract | READY: planned-lifetime classification, risk/cost analysis, and an explicit aging/horizon failure mode. |
| 2 | Standards and Rating Level Definitions | 2: `cdcs-rating-lattice`, `cdcs-rating-nameplate-scope` | ANSI/TIA-942-C:2024 public ratings; ISO/IEC 22237-1:2021 Clause 7; EN 50600-1:2019; Uptime Institute award terms | READY: rejects label equivalence and component-nameplate substitution; no uptime percentage is invented. |
| 3 | Building Considerations | 2: `cdcs-building-floor-loading`, `cdcs-building-floor-units` | ANSI/TIA-569-E:2019 §6.3.6.1.4; NIST Guide to the SI Appendix B.9; TIA-942-C:2024 building/site context | READY: concentrated-load evidence and a unit-normalized structural limit are both checked. |
| 4 | Advanced Raised Floor & Suspended Ceiling | 2: `cdcs-raised-floor-layout`, `cdcs-raised-floor-load-testing` | ANSI/TIA-569-E:2019 §§9.6.2, 9.6.5.1; TIA-942-C:2024 access-floor context | READY for installation order and static/dynamic/impact test evidence; seismic and suspended-ceiling variants remain open. |
| 5 | Advanced Power | 5: battery, UPS parallel, SLD, fuel, generator | IEEE 485-2020; IEC 62040-3:2021; ISO/IEC 22237-3:2021; NFPA 110:2025; IEC 60034-3:2020 | PARTIAL: most public pages expose scope, not the needed sizing/parallel clause; OCPD, harmonics, charging/testing, flywheel, and hydrogen remain missing. |
| 6 | Advanced Electro Magnetic Fields | 2: `cdcs-emf-attenuation`, `cdcs-emf-unit-conversion` | IEEE C95.3-2021; NIST Guide to the SI Appendix B.9; NIOSH measurement references | LIMITED/READY for measured ratio and unit normalization; no public attenuation law or exposure limit is asserted. |
| 7 | Advanced Cooling | 4: airflow, sensible ΔT, psychrometric, `cdcs-cooling-altitude-adjustment` | ASHRAE Handbook—Fundamentals 2025, Chapter 18 §§3–4 | PARTIAL/READY for four anchors: liquid, CFD, SHR, efficiency, humidity, redundancy, and installation remain uncovered. |
| 8 | Advanced Fire Protection | 2: `cdcs-fire-gas`, `cdcs-fire-ahj-evidence` | NFPA 2001:2025; OSHA 1910.162 | LIMITED/READY for the gas screen and AHJ/occupancy evidence catch; no public agent-design or hold-time formula is asserted. |
| 9 | Designing and Installing Scalable Network Cabling Systems | 2: `cdcs-cabling-wireless-ap`, `cdcs-cable-tray-fill` | ANSI/TIA-942-C:2024 Clause 13 public preview; ANSI/TIA-569-E:2019 §9.7.1.1 | READY for wireless-AP and cable-tray-fill variants; broader topology, copper/fiber, ToR/EoR, and labeling remain open. |
| 10 | Environmental Specifications / Contamination Control | 2: `cdcs-contamination-sampling`, `cdcs-contamination-size-boundary` | ISO 14644-1:2015 Clause 1/classification definitions; ISO 14644-2:2015 Clause 4.3 | READY: particle evidence, monitoring, and the 0.1 µm–5 µm classification boundary are checked. |
| 11 | Data Centre Efficiency | 2: `cdcs-pue-boundary`, `cdcs-mpue-mixed-use` | ISO/IEC 30134-2:2026 Definitions 3.1.2 and 3.1.7; public OBP Clauses 5–6 | READY for PUE and mixed-use PUE boundary arithmetic; pPUE and other efficiency metrics remain open. |

## Item-to-heading map

The source IDs below resolve to editioned URLs in `corpus/sources.toml`; the
full required fields (`syllabus_heading`, `syllabus_url`, `source_ids`, and
`claim`) are duplicated in `item-attribution.toml` so a review can be done
without guessing from a stem.

| Item | Exact syllabus heading | Source IDs | Claim under review | Attribution |
| --- | --- | --- | --- | --- |
| `cdcs-battery-autonomy` | Calculating battery banks | `ieee-485-2020` + secondary citations | Bounded first-pass runtime, not standards sizing or a guarantee | LIMITED |
| `cdcs-ups-parallel` | UPS parallel configuration | `iec-62040-3-2021` + secondary citations | Equal-sharing claim fails without impedance/control/bypass/protection evidence | LIMITED |
| `cdcs-one-line` | Single Line Diagram (SLD) | `iso-22237-3-2021` + secondary citation | Shared ATS defeats an unproved independent path | LIMITED |
| `cdcs-airflow-cmh` | Calculating air volume displacement (CFM/CMH) | `nist-flow` | Unit-bearing CFM-to-CMH conversion | READY |
| `cdcs-sensible-delta-t` | Delta-T and impact | `ashrae-f25-ch18-2025` + secondary citation | Solve the stated sensible-load equation | READY |
| `cdcs-psychrometric` | Psychrometric chart | `ashrae-f25-ch18-2025` + secondary citations | Dry-bulb-only selection omits latent/enthalpy evidence | READY |
| `cdcs-fire-gas` | Calculate gas content | `nfpa-2001-2025` + OSHA/secondary citations | Screen volume × concentration without overclaiming design | LIMITED |
| `cdcs-emf-attenuation` | Calculation of EMF attenuation factors | `ieee-c95-3-2021` + NIOSH citations | Measured ratio only; no universal clearance rule | LIMITED |
| `cdcs-fuel-autonomy` | Fuel storage and calculation | `nfpa-110-2025` + regulatory/secondary citations | Usable fuel runtime plus evidence boundary | LIMITED |
| `cdcs-generator-paralleling` | Generator parallelling | `iec-60034-3-2020` + secondary citations | Reject unsynchronized common-bus closure | LIMITED |
| `cdcs-lifecycle-risk-analysis` | Phases of the data centre life cycle | `iso-22237-1-2021`, `iso-22237-1-2021-preview` | Missing planned-lifetime classification and risk/cost inputs | READY |
| `cdcs-rating-lattice` | Rating levels history and definitions | `tia-942-c-2024`, `tia-942-ratings`, `uptime-certification-types`, `en-50600-1-2019`, `iso-22237-1-2021` | Rated, Tier, and availability labels are not interchangeable | READY |
| `cdcs-contamination-sampling` | Contamination measurements, standards and limits | `iso-14644-1-2015`, `iso-14644-1-2015-preview`, `iso-14644-2-2015`, `iso-14644-2-2015-preview` | “Clean” requires particle and monitoring evidence | READY |
| `cdcs-pue-boundary` | Power Usage Effectiveness (PUE) | `iso-30134-2-2026`, `iso-30134-2-2026-obp` | `PUE = E_DC / E_IT` for the same boundary and period | READY |
| `cdcs-building-floor-loading` | Floor and hanging loads requirements | `tia-942-c-2024`, `tia-569-e-2019`, `tia-569-e-2019-preview` | Distributed-only floor evidence omits concentrated loads and the structural-engineer limit | READY |
| `cdcs-raised-floor-layout` | Raised Floor installation requirements | `tia-942-c-2024`, `tia-569-e-2019`, `tia-569-e-2019-preview` | Access-floor layout must precede equipment and telecommunications-cabling installation | READY |
| `cdcs-cabling-wireless-ap` | TIA-942 cabling structure topology | `tia-942-c-2024`, `tia-942-c-2024-preview` | Balanced twisted-pair wireless AP needs at least two Cat 6A-or-higher runs | READY |
| `cdcs-raised-floor-load-testing` | Common raised floor problems | `tia-569-e-2019`, `tia-569-e-2019-preview` | Static-only raised-floor testing omits dynamic and impact loading evidence | READY |
| `cdcs-cooling-altitude-adjustment` | Cooling capacity calculations | `ashrae-f25-ch18-2025` | Sea-level sensible-air factor used at elevation without pressure or air-property correction | READY |
| `cdcs-cable-tray-fill` | Installation best practices | `tia-569-e-2019`, `tia-569-e-2019-preview` | 40% initial tray fill exceeds the 25% initial maximum while remaining below the 50% absolute maximum | READY |
| `cdcs-mpue-mixed-use` | PUE categories | `iso-30134-2-2026`, `iso-30134-2-2026-obp` | Mixed-use PUE must use total energy inside the same data-centre boundary as the IT denominator | READY |
| `cdcs-lifecycle-aging-horizon` | Phases of the data centre life cycle | `iso-22237-1-2021`, `iso-22237-1-2021-preview` | A five-year worksheet does not evidence a named 15-year planned lifetime | READY |
| `cdcs-rating-nameplate-scope` | Rating levels history and definitions | `tia-942-c-2024`, `tia-942-ratings` | A component nameplate is not a TIA facility certification with defined scope and validity | READY |
| `cdcs-building-floor-units` | Floor and hanging loads requirements | `tia-569-e-2019`, `tia-569-e-2019-preview`, `nist-si-appendix-b9` | 125 lbf/ft² converts to about 5.99 kPa, not 125 kPa | READY |
| `cdcs-emf-unit-conversion` | Calculation of EMF attenuation factors | `ieee-c95-3-2021`, `nist-si-appendix-b9` | 1.2 mT ÷ 8 G after normalization is a 1.5 measured ratio | READY |
| `cdcs-fire-ahj-evidence` | Calculate gas content | `nfpa-2001-2025`, `osha-gas` | An 8% screen does not replace AHJ/occupancy, timing, alarm, and egress evidence | READY |
| `cdcs-contamination-size-boundary` | Contamination measurements, standards and limits | `iso-14644-1-2015`, `iso-14644-1-2015-preview`, `iso-14644-2-2015`, `iso-14644-2-2015-preview` | 0.05 µm-only counts do not establish an ISO 14644-1 class in the 0.1–5 µm range | READY |

## Attribution decisions

The remaining secondary vendor citations are illustrations, not the sole
authority. Existing LIMITED rows are mapped but should be rewritten only when
an accessible current clause or formula is found. The seventeen added rows are
narrow clause/formula-backed variants, not broad coverage claims: lifecycle and
aging horizon, ratings and nameplate scope, contamination and particle-size
boundary, PUE boundary, building concentrated-load review and unit conversion,
access-floor sequencing and testing, wireless-AP cabling, cable-tray fill,
cooling altitude correction, EMF unit conversion, fire AHJ evidence, and
mixed-use PUE.
Seismic, suspended ceiling, broader cabling topology/installation, and most
Advanced Power remain open where a public clause or formula has not been
resolved. This is coverage, not a pass.

This study track does not grant EPI/EXIN certification or any credential.
