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

## Breadth pass 02 — site, grid, climate, and resilience

**Search date:** 2026-08-18. This pass used official FEMA, USGS, NOAA/NCEI, FERC, DOE, LBNL, CISA, and Ready.gov HTML pages. PDF links surfaced by those pages were not opened or copied.

### Sources retained

| Source | What it can safely support | Boundaries |
|---|---|---|
| FEMA Flood Map Service Center — https://msc.fema.gov/portal/home | Locate official NFIP flood-hazard products for a site-screening workflow; require map date, effective product, datum, and local review record | A flood-map lookup is not a complete geotechnical, drainage, insurance, or permitting conclusion |
| USGS Earthquake Hazards — https://www.usgs.gov/programs/earthquake-hazards/hazards | Use current national seismic hazard models, faults, hazard tools, and site-specific-data pathways in a preliminary site-risk review | USGS explicitly distinguishes hazard from risk; local geology can amplify shaking, and design values must follow the applicable code workflow |
| NOAA/NCEI U.S. Climate Normals — https://www.ncei.noaa.gov/products/land-based-station/us-climate-normals | Use the current 1991–2020 normals and supplemental recent-period data to frame temperature, precipitation, degree-day, and seasonal energy-load assumptions | Normals characterize climate; they are not a forecast, design guarantee, or substitute for extreme-event analysis |
| NOAA Sea Level Calculator — https://coast.noaa.gov/sealevelcalculator/ | Build a coastal screening question using observed trends, scenarios, flooding frequency, datum, and vertical-land-motion assumptions | NOAA labels it a planning/screening tool, not navigation, permitting, or a legal determination |
| LBNL Queued Up 2024 — https://emp.lbl.gov/publications/queued-2024-edition-characteristics | Teach what an interconnection queue is, why studies and upgrades affect dates, and why queue evidence needs a project status and study phase | This publication is about generation/storage queues; do not silently convert its statistics into data-center-load timelines |
| FERC RM26-4 — https://www.ferc.gov/rm26-4 | Frame current large-load interconnection issues: study process, upgrade cost allocation, co-location, flexible load, and readiness evidence | The page describes an ANOPR/docket and stakeholder input; it is not a final universal tariff or permission to bypass local rules |
| FERC large-load action — https://www.ferc.gov/news-events/news/ferc-launches-aggressive-targeted-action-speed-large-load-integration | Track current regulatory movement around RTO/ISO tariffs, co-location, behind-the-meter generation, flexible transmission service, and speculative requests | Treat as current commission action and docket context, not as a settled design standard |
| DOE microgrids and large loads — https://www.energy.gov/oe/articles/microgrids-large-electric-loads-grid-support-how-leverage-microgrids-support-utilities | Ask how microgrids may interact with utility capacity, reliability, affordability, and large-load deployment | DOE describes a promising approach; it does not guarantee islanding, interconnection approval, or fuel availability |
| DOE i2X interconnection roadmap — https://www.energy.gov/cmei/i2x/doe-distributed-energy-resource-interconnection-roadmap | Require better technical data, clearer study inputs, and process evidence for DER/storage/hybrid facilities and large-load interfaces | A roadmap is guidance, not an adopted tariff or AHJ approval |
| CISA dependency primer — https://www.cisa.gov/topics/critical-infrastructure-security-and-resilience/resilience-services/infrastructure-dependency-primer/implement | Map bidirectional dependencies, redundant substations, water/wastewater, communications, transport, and maintenance responsibilities; test mitigation actions | CISA’s framework is voluntary resilience guidance; do not label a dependency map a certification |
| CISA critical-infrastructure resilience — https://www.cisa.gov/topics/critical-infrastructure-security-and-resilience | Frame facility importance, essential workers, sector interdependence, risk assessment, and exercise planning | CISA guidance supports planning and assessment; it does not replace an owner’s emergency plan, code, or regulator |
| CISA Regional Resiliency Assessment Program — https://www.cisa.gov/resources-tools/programs/regional-resiliency-assessment-program | Ask what a regional assessment can reveal about utilities, transport, emergency response, public/private partners, and investment choices | RRAP is voluntary and assessment-oriented; it does not issue a facility rating |
| Ready Business — https://www.ready.gov/business | Add continuity questions covering communications, IT recovery, employee safety, and essential functions | General preparedness guidance; no invented data-center-specific recovery objective |

### Pass-02 question frontier

| Module | New question families |
|---|---|
| M01 Mission Critical | Build a dependency graph from a customer service to utility, substation, fuel/water, carrier, controls, people, and emergency response; distinguish “critical service” from “critical facility” and identify the missing external dependency. |
| M02 Standards | Given an ISO/TIA reference, a FEMA/USGS/NOAA screening tool, and a local adopted code, classify each as catalog, hazard data, guidance, or binding local requirement; reject a stale edition or a map used outside its stated purpose. |
| M03 Site / Building | Compare two candidate sites using flood-map product/effective date, seismic model/site class, climate normals, coastal scenario, utility study phase, water dependency, emergency access, and commissioning evidence; identify what must be escalated to local engineers/AHJ. |
| M04 Floor / Ceiling | Translate flood and seismic site findings into questions about elevation, equipment placement, structural anchorage, access paths, and inspection evidence without pretending a screening map supplies the structural design. |
| M05 Light | Use climate and hazard conditions to review emergency egress visibility, ice/water intrusion response, outage lighting, and operator access during a severe-weather event; separate life-safety code evidence from comfort lighting. |
| M06 Power | Evaluate a large-load request for site control, forecast maturity, study phase, upgrade responsibility, flexible-load capability, co-location claims, microgrid boundaries, and black-start/islanding evidence; reject “MW available” as a complete interconnection commitment. |
| M07 EMF | Ask whether a site EMF claim has a defined source, frequency, measurement location, instrument method, exposure boundary, and responsible authority; keep seismic/electrical hazard data from being confused with an EMF safety conclusion. |
| M08 Racks | Use hazard maps and dependency records to inspect rack/campus placement, flood elevation, anchorage, water path, access, and recovery priority; distinguish rack-level protection from site-level resilience. |
| M09 Cooling | Compare climate-normal assumptions with extreme-event and water-availability assumptions; identify when an economizer, cooling tower, dry cooler, CDU, or heat-reuse loop loses its expected operating envelope; require a control and fallback sequence. |
| M10 Water | Review a site water-risk package: source, treatment, cooling-tower blowdown, alternative water, drought/flood exposure, wastewater dependency, and emergency supply; identify which data is screening-level and which needs utility/AHJ confirmation. |
| M11 Network | Map bidirectional dependencies between carriers, utility controls, BMS/DCIM, emergency communications, remote operations, and alternate sites; test whether a proposed “diverse” path shares a regional hazard, provider, duct bank, or power dependency. |
| M12 Fire | Convert flood, wildfire, wind, freeze, and seismic exposure into an emergency/fire protection evidence review: alarm availability, suppression water, generator/fuel access, egress, AHJ coordination, and return-to-service criteria. |
| M13 Security | Combine physical threat, cyber exposure, utility dependency, and regional partner data; ask which risks are reduced by access controls, segmentation, exercises, relocation, redundant services, or maintenance rather than by a single perimeter control. |
| M14 Auxiliary | Design a monitoring board that distinguishes hazard feeds, utility telemetry, BMS control, alarm acknowledgement, external notifications, and operator decisions; require source, timestamp, owner, escalation, and fallback when a feed is unavailable. |
| M15 Operations | Run a tabletop from forecasted flood/heat/ice/seismic risk through staffing, fuel/water/carrier constraints, load curtailment, vendor access, communications, recovery, and lessons learned; identify which dependencies need an owner and an exercise rather than a prose promise. |

### Pass-02 factual boundaries

- LBNL’s queue publication and FERC’s large-load docket address different populations and processes. A generation/storage queue is not proof of a data-center energization date.
- FERC RM26-4 and the June 2026 FERC action are live regulatory matters. They are useful current context, but they must not be rendered as a final universal interconnection rule.
- USGS hazard maps are public-domain planning inputs. USGS states that hazard is not risk and that local geology may amplify motion; a question must preserve that distinction.
- NCEI’s current U.S. normals are 1991–2020, with a recent 15-year supplemental set. A normal is a baseline characterization, not an extreme-weather design value.
- NOAA’s Sea Level Calculator is explicitly a screening/planning tool and not for permitting or legal use. Do not turn a scenario output into a site approval.
- CISA dependency and resilience tools support assessment, coordination, mitigation, and exercises. They do not certify a facility, replace an AHJ, or create an invented availability class.

### Updated pass tracker

| Pass | Focus | Status |
|---:|---|---|
| 01 | DOE/FEMP/NREL energy, cooling, water, commissioning; CISA/NIST OT and exposure; eCFR safety; ISO/TIA/IEEE edition anchors | COMPLETE |
| 02 | Site risk, utility interconnection, resilience, climate/flood/seismic, public AHJ/code sources | COMPLETE — this section |
| 03 | Electrical distribution, switching, protection, UPS/generator/BESS, maintenance testing | OPEN |
| 04 | Cooling controls, liquid cooling, heat reuse, economization, thermal/water measurement | OPEN |
| 05 | Cabling, outside plant, network management, BMS/DCIM segmentation, remote access | OPEN |
| 06 | Fire/life safety, emergency power, permits, PPE, incident command, return-to-service | OPEN |
| 07 | Physical security, personnel/vendor/visitor controls, asset lifecycle, media disposition | OPEN |
| 08 | People systems: skills matrices, shift turnover, training, fatigue, contractor governance, succession | OPEN |
| 09 | Commissioning, change control, MOP/SOP/EOP, alarms, metrics, reliability-centered maintenance | OPEN |
| 10 | Cross-module adversarial review: stale editions, unsupported numbers, false equivalences, and uncovered operational decisions | OPEN |

The objective remains open after pass 02. The next unresolved frontier is electrical distribution and maintenance testing, not another site-risk summary.

## Breadth pass 03 — electrical distribution, backup power, storage, and maintenance testing

**Search date:** 2026-08-18. This pass retained official OSHA HTML, DOE/FEMP HTML, NFPA catalog/preview pages, IEC catalog pages, and an IEEE Xplore standard record. No PDF body was fetched or copied.

### Sources retained

| Source | What it can safely support | Boundaries |
|---|---|---|
| OSHA 1910.333, Selection and use of work practices — https://www.osha.gov/laws-regs/regulations/standardnumber/1910/1910.333 | Deenergization, lockout/tagging, stored-energy release, verification, reenergization, qualified-person, and energized-work evidence questions | This is an OSHA workplace requirement; it does not supply a facility design, an EPI heading, or permission to make a live-work claim without the applicable hazard analysis and employer program |
| OSHA 1910.137, Electrical Protective Equipment — https://www.osha.gov/laws-regs/regulations/standardnumber/1910/1910.137 | Electrical protective-equipment condition, periodic electrical tests, proof-test evidence, and in-service care questions | Use the current OSHA rule and its tables as the legal anchor; do not replace it with a vendor interval or a paraphrased NFPA value |
| DOE FEMP Equipment Operations and Maintenance Summaries — https://www.energy.gov/cmei/femp/equipment-operations-and-maintenance-summaries | Standby-generator components, ATS transfer logic, safety issues, maintenance checklists, monitoring, and planned O&M questions | FEMP is federal O&M guidance. Its standby-generator description does not establish a universal runtime, fuel autonomy, reliability class, or AHJ acceptance |
| DOE Uninterruptible Power Supplies — https://www.energy.gov/cmei/buildings/uninterruptible-power-supplies | Public UPS definition, continuity-of-load-power framing, current DOE test-procedure rulemaking, and the distinction between product energy testing and facility continuity planning | DOE’s appliance rule is not a data-center electrical design standard; do not turn its test procedure into a facility ride-through or availability claim |
| DOE FEMP Battery Energy Storage System Procurement Checklist — https://www.energy.gov/cmei/femp/articles/battery-energy-storage-system-procurement-checklist | BESS project-development questions, procurement inputs, technical specification handoffs, and interconnection-checklist evidence | The page is a procurement checklist for commercial-scale lithium-ion BESS; it is not a complete fire design, O&M program, chemistry-independent rule, or AHJ approval |
| DOE Office of Electricity Energy Storage — https://www.energy.gov/oe/energy-storage | Current public framing for storage safety, reliability, performance validation, and the storage program’s scope | DOE program framing is not an adopted code, a site permit, or proof that a particular BESS can support a particular critical load |
| NFPA 70B, Standard for Electrical Equipment Maintenance, 2026 preview/catalog — https://link.nfpa.org/all-publications/70B/2026 | Edition-pinned syllabus/source labels for electrical equipment maintenance, inspections, testing, records, and maintenance-program questions | The standard body is subscription-controlled; the preview/catalog pins the edition and public heading only. Do not reproduce paid text or invent test intervals |
| NFPA 70E, Standard for Electrical Safety in the Workplace, 2027 catalog — https://link.nfpa.org/all-publications/70E/2027 | Edition-pinned source for safety-related work practices, maintenance requirements, special equipment, and administrative-control question families | The catalog is the edition pin; OSHA remains the legal anchor. NFPA 70E is not automatically law unless adopted or incorporated by the applicable authority |
| NFPA 110, Standard for Emergency and Standby Power Systems, 2025 preview — https://link.nfpa.org/all-publications/110/2025 | Public heading and edition pin for emergency/standby power-system questions involving sources, transfer, testing, and maintenance | Paid body not fetched; do not assert a universal generator test schedule, classification, or fuel requirement from the catalog page |
| NFPA 111, Standard on Stored Electrical Energy Emergency and Standby Power Systems, 2025 catalog — https://link.nfpa.org/all-publications/111/2025 | Public heading and edition pin for stored-energy emergency/standby systems, transfer/protection, installation, routine maintenance, and operational testing | Paid body not fetched; do not collapse SEPSS, UPS, BESS, and emergency-power classifications into one invented taxonomy |
| IEC 62040-3:2021, UPS performance and test requirements — https://webstore.iec.ch/en/publication/60140 | Official edition pin and scope for electronic UPS performance and test questions, including complete UPS and functional-unit testing | The catalog states scope limits: conventional AC/DC distribution boards, standalone static-transfer systems, rotary UPS, and DC UPS are outside this document; the edition page is not the paid body |
| IEC 62477-1:2022, safety requirements for power electronic converter systems — https://webstore.iec.ch/en/publication/28936 | Official edition pin for converter control, protection, monitoring, measurement, and safety questions spanning UPS and bidirectional converters | Catalog scope is a safety standard for PECS; it is not a facility coordination study, a protection-setting prescription, or a substitute for local code |
| IEEE 1547-2018, DER interconnection and interoperability — https://ieeexplore.ieee.org/document/8332112/ | Official IEEE record for DER interface, interconnection, and interoperability question families relevant to BESS/microgrid boundaries | An IEEE standard record is not an interconnection approval or utility tariff; it does not establish a data-center UPS requirement or guarantee islanding |

### Pass-03 question frontier

| Module | New question families |
|---|---|
| M01 Mission Critical | Trace a critical load through utility service, switchgear, transformer, UPS, ATS, generator, storage, distribution path, cooling dependency, controls, and operator decision; identify the single evidence gap that makes a continuity claim unproven. |
| M02 Standards | Classify OSHA, NFPA, IEC, IEEE, DOE/FEMP, adopted code, and site procedure references; pin the edition from the official page; distinguish a paid standard’s catalog receipt from evidence that its body was reviewed. |
| M03 Site / Building | Review a power-plant brief for utility service, service entrance, transformer/switchgear rooms, generator and fuel path, battery/storage location, clearances, ventilation, fire separation, flood elevation, maintenance access, and AHJ/utility handoff records. |
| M04 Floor / Ceiling | Inspect equipment-room changes for switchgear/transformer/battery weight, seismic restraint, arc-flash boundaries, working space, cable/busway paths, heat and ventilation, access for replacement, and evidence that the structural and electrical reviews agree. |
| M05 Light | Test whether normal and emergency lighting remains adequate for switching, inspection, battery-room, generator-room, and egress work; separate illumination needed to perform electrical work from a general emergency-lighting claim. |
| M06 Power | Read a one-line for sources, buses, breakers, selective coordination evidence, grounding/bonding, UPS bypass, STS, ATS, generator, battery, and last-common-point failure; choose the safest switching sequence and the proof required before reenergization. |
| M07 EMF | Separate energized-source identification, field measurement, worker boundary, shielding/grounding claim, and PPE/work-practice control; reject an EMF answer that uses a generic distance or unqualified “safe” label. |
| M08 Racks | Match rack dual-cord architecture to independent distribution paths, breaker/panel capacity, maintenance bypass, power quality, inlet ratings, busway/PDU identity, and asset records; identify where a rack-level redundancy claim still shares a common upstream failure. |
| M09 Cooling | Analyze how UPS, generator, transformer, switchgear, battery, and power-conversion heat loads alter room cooling and ventilation; identify the fallback sequence when a generator room, battery room, or liquid-cooled power rack loses its normal heat-rejection path. |
| M10 Water | Add generator cooling, battery-room fire-water/containment interfaces, water-dependent utility systems, and storm/flood isolation to a site water dependency review; distinguish a water availability assertion from a tested emergency operating mode. |
| M11 Network | Trace monitoring and control paths for UPS, ATS, generator, switchgear, BMS/DCIM, storage controller, and utility interface; require segmentation, access control, time source, alarm ownership, local fallback, and a reversible remote-action record. |
| M12 Fire | Review electrical-room and storage-room detection, suppression, separation, egress, emergency shutdown, alarm, and AHJ evidence as separate controls; identify the missing test or impairment record that blocks return to service. |
| M13 Security | Combine physical access, key/control custody, vendor remote maintenance, firmware/configuration changes, battery-management systems, generator controls, and incident response; identify the control gap when an operational power endpoint is reachable through an unnecessary external path. |
| M14 Auxiliary | Build an alarm and telemetry map for utility, switchgear, UPS, ATS, generator, battery, fire, BMS/DCIM, and fuel systems; distinguish indication, command, trip, interlock, acknowledgement, escalation, and audit evidence. |
| M15 Operations | Turn maintenance testing into an evidence loop: scope and risk, isolation/permit, qualified staff, pre-checks, test execution, abnormal response, restoration, post-test inspection, defect disposition, and trend review; compare planned maintenance with condition evidence without inventing universal intervals. |

### Pass-03 factual boundaries

- OSHA 1910.333 requires safety-related work practices around energized or potentially energized parts and gives a public sequence for deenergization, lockout/tagging, stored-energy control, verification, and reenergization. Questions must preserve the qualified-person and employer-program boundaries.
- OSHA 1910.137 requires electrical protective equipment to remain safe and reliable and requires periodic electrical tests. Do not replace its current rule and tables with an unsourced “annual” or “every six months” claim for equipment outside the stated scope.
- FEMP’s standby-generator and ATS pages support component, transfer, monitoring, and maintenance questions. They do not establish a universal standby rating, fuel autonomy, generator runtime, or data-center availability class.
- The NFPA 70B/70E/110/111 pages pin current public headings and editions. Their paid bodies were not fetched. The ledger may record a catalog/preview receipt, but an item cannot quote a clause or interval absent a public source.
- IEC 62040-3:2021 is explicitly about electronic UPS performance and test requirements and excludes conventional distribution boards and certain static-transfer/rotary/DC UPS subjects. Scope exclusions should be tested, not silently generalized.
- IEC 62477-1:2022 covers safety of power electronic converter systems and their control, protection, monitoring, and measurement. It does not provide a site protection-coordination study or local-code approval.
- IEEE 1547-2018 is an official DER interconnection/interoperability record. It can anchor BESS/microgrid interface questions, but it does not prove utility acceptance, islanding capability, or a critical-load ride-through result.
- A generator, UPS, ATS, STS, BESS, SEPSS, and emergency-power system are related but not interchangeable terms. No question should infer an availability class or handover taxonomy from a product label.

### Updated pass tracker

| Pass | Focus | Status |
|---:|---|---|
| 01 | DOE/FEMP/NREL energy, cooling, water, commissioning; CISA/NIST OT and exposure; eCFR safety; ISO/TIA/IEEE edition anchors | COMPLETE |
| 02 | Site risk, utility interconnection, resilience, climate/flood/seismic, public AHJ/code sources | COMPLETE |
| 03 | Electrical distribution, switching, protection, UPS/generator/BESS, maintenance testing | COMPLETE — this section |
| 04 | Cooling controls, liquid cooling, heat reuse, economization, thermal/water measurement | OPEN |
| 05 | Cabling, outside plant, network management, BMS/DCIM segmentation, remote access | OPEN |
| 06 | Fire/life safety, emergency power, permits, PPE, incident command, return-to-service | OPEN |
| 07 | Physical security, personnel/vendor/visitor controls, asset lifecycle, media disposition | OPEN |
| 08 | People systems: skills matrices, shift turnover, training, fatigue, contractor governance, succession | OPEN |
| 09 | Commissioning, change control, MOP/SOP/EOP, alarms, metrics, reliability-centered maintenance | OPEN |
| 10 | Cross-module adversarial review: stale editions, unsupported numbers, false equivalences, and uncovered operational decisions | OPEN |

The objective remains open after pass 03. The next unresolved frontier is cooling controls, liquid cooling, heat reuse, economization, and thermal/water measurement.
