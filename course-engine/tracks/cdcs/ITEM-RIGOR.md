# CDCS Item Rigor Ledger

This ledger proves that the twenty-seven shipped CDCS items require arithmetic or a
one-line defect catch. It is an audit of the bank, not a second learner-facing
coverage ledger. All twenty-seven items are original (`source_class = "original"`), and
none is a name-recall item.

## Arithmetic items

| Item | Required operation | Why recall alone fails |
| --- | --- | --- |
| `cdcs-battery-autonomy` | `480 V × 600 Ah × 0.85 × 0.90 ÷ 180 kW = 73.4 min` | The answer depends on applying usable fraction, efficiency, load, and unit conversion. |
| `cdcs-airflow-cmh` | `12,000 CFM × 1.699 = 20,388 CMH` | The four choices are conversion outcomes; the unit-bearing multiplication selects the answer. |
| `cdcs-sensible-delta-t` | `68,000 ÷ (1.085 × 17) = 3,687 CFM` | The learner must isolate CFM from the stated sensible-load equation. |
| `cdcs-fire-gas` | `920 m³ × 0.08 = 73.6 m³ equivalent`, then compare with 60 m³ | The arithmetic catches the offer, while the hold-time and enclosure boundary prevents slogan-level overclaiming. |
| `cdcs-emf-attenuation` | `80 μT ÷ 20 μT = 4×` | The result is a measured ratio, not a term-definition lookup or universal rule. |
| `cdcs-fuel-autonomy` | `1,000 L × 0.85 ÷ 180 L/h = 4.72 h` | The usable-volume adjustment must be applied before dividing by consumption. |
| `cdcs-cable-tray-fill` | `0.08 m² ÷ 0.20 m² = 40%`, then compare with 25% initial and 50% absolute limits | The learner must calculate the proposed fill and distinguish the initial-planning limit from the absolute limit. |
| `cdcs-mpue-mixed-use` | `1,800 MWh ÷ 1,200 MWh = 1.50` using the same data-centre boundary | The whole-building numerator is a boundary error; the learner must select the in-boundary energy before dividing. |
| `cdcs-building-floor-units` | `125 lbf/ft² × 47.88026 Pa/(lbf/ft²) ÷ 1,000 = 5.99 kPa` | The learner must convert the structural limit before comparing it with the schedule. |
| `cdcs-emf-unit-conversion` | `8 G = 0.8 mT`; `1.2 mT ÷ 0.8 mT = 1.5×` | The measured attenuation ratio is invalid until the field units are normalized. |

## One-line defect items

| Item | Defect that must be found | Why recall alone fails |
| --- | --- | --- |
| `cdcs-ups-parallel` | Unequal output-path impedance invalidates the equal-sharing claim; controls, bypass, cabling, and protection evidence are required. | The stem supplies a concrete topology and claim to challenge, not a UPS vocabulary prompt. |
| `cdcs-one-line` | Utility and generator sources converge through one shared ATS, creating a common-mode point despite the 2N label. | The reviewer must trace the path and reject the independence claim. |
| `cdcs-psychrometric` | Dry-bulb/ΔT-only coil selection omits humidity ratio, latent load, and enthalpy evidence. | The learner must identify the missing process evidence in the offer. |
| `cdcs-generator-paralleling` | Common-bus closure lacks synchronization and phase-sequence interlock; sharing and protection evidence are also required. | The learner must reject an unsafe one-line condition and name the missing controls. |
| `cdcs-building-floor-loading` | Distributed-only review omits concentrated equipment loads and the structural-engineer floor limit. | The learner must reject incomplete floor-load evidence. |
| `cdcs-raised-floor-layout` | Equipment and telecommunications cabling began before the access-floor layout was determined. | The learner must catch the installation-order defect. |
| `cdcs-raised-floor-load-testing` | Static-only raised-floor evidence omits dynamic and impact loading conditions. | The learner must catch the missing test-condition evidence rather than accept the word “tested.” |
| `cdcs-cabling-wireless-ap` | A balanced twisted-pair wireless AP has one Cat 6A run where the TIA-942-C summary states a two-run minimum. | The learner must catch the topology deficiency. |
| `cdcs-cooling-altitude-adjustment` | A sea-level sensible-air factor is used at elevation with no pressure or air-property correction. | The learner must catch the unsupported high-elevation calculation basis. |
| `cdcs-lifecycle-aging-horizon` | A five-year classification worksheet is used for a named 15-year planned lifetime without remaining-life assumptions. | The learner must catch the lifecycle-horizon evidence gap. |
| `cdcs-rating-nameplate-scope` | A UPS nameplate is treated as proof of a facility Rated-3 certification without type, scope, or validity evidence. | The learner must distinguish a component label from a site certification. |
| `cdcs-fire-ahj-evidence` | An 8% gas screen is called approved without AHJ/occupancy, concentration-timing, alarm, or egress evidence. | The learner must stop the unsupported approval claim and escalate the safety evidence gap. |
| `cdcs-contamination-size-boundary` | A claimed ISO class is based on 0.05 µm counts outside the cited 0.1–5 µm classification range. | The learner must catch the particle-size boundary defect. |

## Slogan-only result

No heading is slogan-only in the current twenty-seven-item bank. Every item has a
numeric witness or a concrete defect witness in its `stem`, `item_class`, and
`work` fields. No new analyze item was added because no existing heading met
the trigger for one; the six new thin-module variants add distinct evidence
catches and calculations rather than duplicate IDs.

This rigor check does not certify a person, vendor offer, design, facility, or
credential. It only records why these item records are falsifiable study
prompts.

## Attribution-bar additions

The first missing-module additions are also falsifiable one-line or arithmetic
checks. Their public heading, standard source, and candidate claim are recorded
in `item-attribution.toml`.

| Item | Required operation | Public anchor |
| --- | --- | --- |
| `cdcs-lifecycle-risk-analysis` | Reject a lifecycle classification missing planned-lifetime criteria and risk/cost analysis. | ISO/IEC 22237-1:2021 Clauses 5 and 7. |
| `cdcs-rating-lattice` | Reject an unqualified equivalence between TIA Rated, Uptime Tier, and Availability Class labels. | TIA public ratings, Uptime terms, and ISO/IEC 22237-1:2021 Clause 7. |
| `cdcs-contamination-sampling` | Reject a “clean” claim with no particle sampling or monitoring plan. | ISO 14644-1:2015 Clause 1/classification definitions and ISO 14644-2:2015 Clause 4.3. |
| `cdcs-pue-boundary` | `1,200,000 ÷ 800,000 = 1.50`. | ISO/IEC 30134-2:2026 Definition 3.1.2 / public OBP. |
| `cdcs-building-floor-loading` | Reject average-only evidence; concentrated loads and structural-engineer limit are missing. | ANSI/TIA-569-E:2019 §6.3.6.1.4. |
| `cdcs-raised-floor-layout` | Reject equipment/cabling installation before access-floor layout determination. | ANSI/TIA-569-E:2019 §9.6.5.1. |
| `cdcs-cabling-wireless-ap` | Reject one Cat 6A run to a balanced twisted-pair wireless AP. | ANSI/TIA-942-C:2024 Clause 13 public preview and official TIA revision summary. |
| `cdcs-raised-floor-load-testing` | Reject static-only testing evidence that omits dynamic and impact conditions. | ANSI/TIA-569-E:2019 §9.6.2. |
| `cdcs-cooling-altitude-adjustment` | Reject a sea-level sensible-air factor at elevation without correction evidence. | ASHRAE Handbook—Fundamentals 2025, Chapter 18 §4. |
| `cdcs-cable-tray-fill` | Calculate 40% initial fill and compare it with the 25% initial and 50% absolute limits. | ANSI/TIA-569-E:2019 §9.7.1.1. |
| `cdcs-mpue-mixed-use` | Calculate 1.50 from same-boundary data-centre and IT energy. | ISO/IEC 30134-2:2026 Definition 3.1.7 / public OBP. |
| `cdcs-lifecycle-aging-horizon` | Reject first-five-year evidence applied to a named 15-year planned lifetime. | ISO/IEC 22237-1:2021 Clause 5 and public abstract. |
| `cdcs-rating-nameplate-scope` | Reject a facility rating inferred from a component nameplate without certification scope. | ANSI/TIA-942-C:2024 official page and TIA public certification definitions. |
| `cdcs-building-floor-units` | Convert 125 lbf/ft² to approximately 5.99 kPa. | ANSI/TIA-569-E:2019 §6.3.6.1.4 and NIST Guide to the SI Appendix B.9. |
| `cdcs-emf-unit-conversion` | Normalize 1.2 mT and 8 G before computing 1.5×. | IEEE C95.3-2021 and NIST Guide to the SI Appendix B.9. |
| `cdcs-fire-ahj-evidence` | Reject an 8% screen presented as fire-system approval without AHJ/occupancy and safety evidence. | NFPA 2001:2025 and OSHA 29 CFR 1910.162(b)(2), (b)(4), and (b)(5). |
| `cdcs-contamination-size-boundary` | Reject 0.05 µm-only evidence for an ISO 14644-1 classification range beginning at 0.1 µm. | ISO 14644-1:2015 official abstract/public preview and ISO 14644-2:2015. |

Seismic protection, suspended-ceiling requirements, broader cabling topology and
installation, and most Advanced Power variants remain ledger-only holes where
the public source did not expose a clause or formula. They were not turned into
unattributed questions.
