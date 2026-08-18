# Open corpus expansion — breadth pass 01

**Date:** 2026-08-18
**Scope:** end-to-end data-centre operations, mapped to the 15 EPI CDCP modules and the CDFOS/CDFOM operations lane.
**Status:** research pass, not a credential, exam form, or readiness verdict.

This log records public HTML pages and official catalog/abstract pages only. No PDF body was fetched or copied. Vendor blogs, exam dumps, shadow libraries, and invented EPI headings are excluded. The question seeds below are research targets; they are not yet bank items until an item-level source and exact public syllabus heading are pinned.

## Search record

| Search family | Primary sources retained | Modules / use |
|---|---|---|
| EPI syllabus anchor | EPI CDCP course and syllabus page — https://www.epi-ap.com/services/1/3/4/Certified_Data_Centre_Professional_%28CDCP%29 | All CDCP modules; exact public headings only |
| Operations and maintenance | DOE FEMP, Operations and Maintenance Challenges and Solutions — https://www.energy.gov/cmei/femp/operations-and-maintenance-challenges-and-solutions | M06, M09, M10, M14, M15; maintenance strategy, controls, KPIs, reliability-centered maintenance |
| Federal facility O&M | DOE FEMP, Operations and Maintenance in Federal Facilities — https://www.energy.gov/cmei/femp/operations-and-maintenance-federal-facilities | M06, M09, M10, M14, M15; work orders, leaks, controls, reliability, safety, water and energy |
| Commissioning and handoff | DOE FEMP, Commissioning Process for Federal Facilities — https://www.energy.gov/cmei/femp/commissioning-process-federal-facilities | M03, M06, M09, M14, M15; plan, investigate, test, deficiency list, retest, handoff, future commissioning |
| Cooling and water operations | DOE FEMP, Cooling Water Efficiency Opportunities for Federal Data Centers — https://www.energy.gov/cmei/femp/cooling-water-efficiency-opportunities-federal-data-centers | M05, M06, M09, M10, M14; air/water economizing, cycles of concentration, WUE/PUE boundaries, liquid cooling controls |
| Cooling tower practice | DOE FEMP BMP 9, Single-Pass Cooling — https://www.energy.gov/cmei/femp/best-management-practice-9-single-pass-cooling; BMP 10, Cooling Tower Management — https://www.energy.gov/cmei/femp/best-management-practice-10-cooling-tower-management | M09, M10, M15; inventory, flow/temperature checks, blowdown, drift, evaporation, chemistry, maintenance |
| Grid-to-chip planning | National Laboratory of the Rockies, Chip-to-Grid Data Center Initiative — https://www.nrel.gov/computational-science/chip-to-grid-data-center-initiative | M01, M03, M05, M06, M09, M10, M11, M14, M15; load growth, grid integration, demand flexibility, cooling, campus and utility interfaces |
| New cooling frontiers | National Laboratory of the Rockies, COOLERCHIPS announcement — https://www.nrel.gov/news/detail/program/2023/nrel-joins-effort-to-advance-data-center-cooling-efficiency; RTES analysis — https://www.nrel.gov/news/detail/program/2025/nlr-analysis-identifies-reservoir-thermal-energy-storage-as-a-solution-for-data-center-cooling-needs.html | M03, M06, M09, M10, M14; emerging technology framing, not universal performance claims |
| Water efficiency | EPA WaterSense best-management-practices hub — https://www.epa.gov/watersense/best-management-practices | M09, M10, M14, M15; single-pass cooling, cooling-tower maintenance, energy/water coupling, facility water program |
| Physical and cyber-physical security | CISA Commercial Facilities resources — https://www.cisa.gov/commercial-facilities-resources; CISA Internet Exposure Reduction — https://www.cisa.gov/resources-tools/resources/exposure-reduction | M08, M11, M13, M14, M15; facility access, asset visibility, exposure review, jump hosts, MFA, monitoring |
| OT/BMS/DCIM security | NIST SP 800-82 Rev. 3 — https://csrc.nist.gov/pubs/sp/800/82/r3/final; NIST OT publications index — https://csrc.nist.gov/Projects/operational-technology-security/publications | M06, M08, M13, M14, M15; building automation, physical access control, monitoring/control systems, safety and reliability constraints |
| Enterprise network operations | NIST SP 800-215 — https://csrc.nist.gov/pubs/sp/800/215/final | M11, M13, M14, M15; multi-site/cloud network landscape, segmentation, operations and security program questions |
| Worker safety | eCFR 29 CFR 1910 Subpart I — https://www.ecfr.gov/current/title-29/subtitle-B/chapter-XVII/part-1910/subpart-I; existing LOTO and electrical-work pages are pinned in the CDFOS corpus | M05, M06, M12, M13, M15; PPE hazard assessment, electrical protective equipment, LOTO, role/training evidence |
| Facilities standard anchor | ISO/IEC 22237-1:2021 official OBP/catalog page — https://www.iso.org/standard/78550.html; TIA-942 official standard page — https://tiaonline.org/products-and-services/tia942certification/ansi-tia-942-standard/ | M02, M03, M04, M08, M11; edition pinning and terminology boundaries only |
| Network primary body | IEEE 802.3 Ethernet Working Group — https://www.ieee802.org/3/ | M11; PHY/media/reach discipline, no invented universal distance or loss values |

## Module-by-module question frontier

These are deliberately operational prompts: each asks the learner to inspect evidence, choose a control action, or explain a trade-off. They are candidate stems for later item work, not claims that the current bank already covers them.

| Module | New question families from pass 01 |
|---|---|
| M01 Mission Critical | Trace a chip-to-grid dependency from business service to utility, campus, cooling, network, and operator decision; distinguish a criticality claim from a density claim; identify which interdependency is absent from an outage scenario. |
| M02 Standards | Given ISO/IEC 22237, TIA-942, code, and internal policy references, classify each as terminology, design requirement, adopted code, or local procedure; reject a withdrawn edition pin; state what an official catalog page proves and what it does not. |
| M03 Site / Building | Compare a site brief that names MW capacity with one that includes grid study stage, water path, heat-rejection option, commissioning plan, and community/utility interface; identify the missing handoff evidence before construction release. |
| M04 Floor / Ceiling | Review a raised-floor or ceiling change request for concentrated load, access, airflow, grounding, and maintenance impacts; identify which evidence belongs to structural approval versus operations acceptance. |
| M05 Light | Build an inspection plan that separates working illumination, emergency lighting, egress, and task visibility; identify when a lighting change creates a safety or maintenance-control change rather than a cosmetic change. |
| M06 Power | Read a one-line for a last-common-point failure, test a UPS/generator maintenance sequence, distinguish electrical capacity from available IT capacity, and identify the operator evidence needed before a switching window. |
| M07 EMF | Separate field source, measurement unit, shielding claim, and safe-work control; reject an EMF statement that has no frequency, location, measurement method, or authority boundary. |
| M08 Racks | Review a rack deployment for dimensions, power distribution, physical security, airflow, liquid-cooling service boundary, and asset identity; identify the missing record that prevents safe maintenance. |
| M09 Cooling | Compare air-side and water-side economizer sequences; diagnose a controls loop that fights humidity or temperature; identify what must be monitored when a liquid-cooling CDU changes the heat-rejection path; distinguish a case-study result from a universal setpoint. |
| M10 Water | Calculate or bound WUE from site water and IT energy records; explain evaporation, drift, blowdown, and cycles of concentration; choose a measurement and maintenance response before claiming savings. |
| M11 Network | Trace a diverse path through meet-me, outside plant, pathway, patching, and active equipment; bind reach to a named IEEE PHY/media class; identify whether remote access to BMS/management networks is necessary, isolated, monitored, and reversible. |
| M12 Fire | Review detection, suppression, alarm, interlock, egress, and AHJ evidence as separate controls; identify which missing test record blocks return to service; keep agent quantity and design calculations out unless the cited body supports them. |
| M13 Security | Combine physical access, cyber-physical controls, visitor/vendor access, asset inventory, and incident response; identify the control gap when an operator can reach a BMS or DCIM path from an internet-exposed service. |
| M14 Auxiliary | Treat BMS/DCIM/EMS/alarm panels as operational technology: define alarm ownership, source-of-truth, notification, acknowledgement, escalation, change control, and recovery evidence; distinguish a dashboard from a control. |
| M15 Operations | Turn FEMP commissioning into an operating loop: plan, investigate, test, deficiency list, retest, handoff, and periodic review; compare reactive, preventive, predictive, and reliability-centered maintenance; build a role/training/permit/handover record without inventing an OLA taxonomy. |

## High-value factual boundaries retained

- FEMP’s cooling-water page gives public definitions and formulas for PUE and WUE, but its example values are contextual; do not turn them into universal targets.
- FEMP’s cooling-tower material separates evaporation, drift, and blowdown and ties blowdown to dissolved-solids control; a question should ask for the missing measurement or control, not assert a universal cycles-of-concentration limit.
- NREL’s Chip-to-Grid page is a systems-integration and research framing source. It supports questions about interfaces, demand flexibility, forecasting, and grid integration; it does not certify a particular design or guarantee an operating result.
- NIST SP 800-82 Rev. 3 covers OT, including building automation, physical access control, and physical-environment monitoring/control systems. It is guidance, not an EPI syllabus heading or an automatic compliance claim.
- CISA Internet Exposure Reduction supports asset inventory, necessity review, restricted access, secure remote access, MFA, and routine reassessment. It does not justify exposing a BMS, DCIM, or control endpoint to the public internet.
- ISO/IEC 22237-1:2021 is the current official page used for this pass; the previous 2018 technical-specification page is withdrawn. The catalog/preview pins an edition and scope, not permission to reproduce the body.

## Ten-pass tracker

| Pass | Focus | Status |
|---:|---|---|
| 01 | DOE/FEMP/NREL energy, cooling, water, commissioning; CISA/NIST OT and exposure; eCFR safety; ISO/TIA/IEEE edition anchors | COMPLETE — this file |
| 02 | Site risk, utility interconnection, resilience, climate/flood/seismic, public AHJ/code sources | OPEN |
| 03 | Electrical distribution, switching, protection, UPS/generator/BESS, maintenance testing | OPEN |
| 04 | Cooling controls, liquid cooling, heat reuse, economization, thermal/water measurement | OPEN |
| 05 | Cabling, outside plant, network management, BMS/DCIM segmentation, remote access | OPEN |
| 06 | Fire/life safety, emergency power, permits, PPE, incident command, return-to-service | OPEN |
| 07 | Physical security, personnel/vendor/visitor controls, asset lifecycle, media disposition | OPEN |
| 08 | People systems: skills matrices, shift turnover, training, fatigue, contractor governance, succession | OPEN |
| 09 | Commissioning, change control, MOP/SOP/EOP, alarms, metrics, reliability-centered maintenance | OPEN |
| 10 | Cross-module adversarial review: stale editions, unsupported numbers, false equivalences, and uncovered operational decisions | OPEN |

The objective remains open after pass 01. A pass is only complete when its source claims are recorded, its question frontier is explicit, and the next pass has a narrower unresolved frontier rather than a generic “research more” note.
