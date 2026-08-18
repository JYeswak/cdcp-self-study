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

## Breadth pass 04 — cooling controls, liquid cooling, heat reuse, economization, and thermal/water measurement

**Search date:** 2026-08-18. This pass retained official DOE/FEMP, EPA WaterSense, NREL/National Laboratory of the Rockies, and Lawrence Berkeley National Laboratory HTML pages. PDF links surfaced by those pages were not opened or copied.

### Sources retained

| Source | What it can safely support | Boundaries |
|---|---|---|
| DOE FEMP, Cooling Water Efficiency Opportunities for Federal Data Centers — https://www.energy.gov/cmei/femp/cooling-water-efficiency-opportunities-federal-data-centers | Cooling-system heat paths, PUE/WUE definitions, space temperature/humidity control, air- and water-side economizing, cycles of concentration, side-stream filtration, and direct-liquid-cooling system boundaries | FEMP’s examples and savings statements are contextual. Do not turn them into universal setpoints, savings, water quality limits, or availability guarantees |
| DOE FEMP, Best Practices Guide for Energy-Efficient Data Center Design — https://www.energy.gov/cmei/femp/articles/best-practices-guide-energy-efficient-data-center-design | Public design categories covering IT conditions, air management, cooling/electrical systems, heat recovery, and performance metrics; use it to formulate evidence questions across design and operations | FEMP expressly presents this as guidance for varied scenarios, not a single “most efficient” design or a mandatory performance threshold |
| DOE FEMP BMP 10, Cooling Tower Management — https://www.energy.gov/cmei/femp/best-management-practice-10-cooling-tower-management | Evaporation, drift, blowdown, dissolved-solids control, chemistry, heat-transfer efficiency, and maintenance evidence | The page does not establish one acceptable cycles-of-concentration value for every climate, tower, water chemistry, or treatment regime |
| DOE FEMP, Side Stream Filtration for Cooling Towers — https://www.energy.gov/cmei/femp/water-efficient-technology-opportunity-side-stream-filtration-cooling-towers | Fouling, suspended solids, filtration, heat-transfer performance, water/energy trade-offs, and O&M questions | FEMP describes an opportunity and its trade-offs; it does not prove that filtration alone solves corrosion, microbiological control, or water treatment |
| EPA WaterSense Best Management Practices — https://www.epa.gov/watersense/best-management-practices | Water-management planning, metering/submetering, leak detection, benchmarking, single-pass cooling, cooling towers, chilled-water systems, and water-energy nexus questions | The page links public BMP headings and release dates; linked PDF bodies were not fetched. EPA BMPs are guidance, not a data-center certification or universal water target |
| NLR/NREL Chip-to-Grid Data Center Initiative — https://www.nrel.gov/computational-science/chip-to-grid-data-center-initiative | Current systems framing for liquid cooling, modularization, load forecasting, flexibility, grid integration, and end-to-end performance/resource trade-offs | NLR describes an initiative and research direction; it does not certify a technology, guarantee a PUE/WUE result, or replace a design review |
| NLR/NREL COOLERCHIPS announcement — https://www.nrel.gov/news/detail/program/2023/nrel-joins-effort-to-advance-data-center-cooling-efficiency | Public research framing for standardized cooling evaluation from component and rack levels through edge data centers, and for liquid-cooling/heat-reuse questions | The program’s targets and a laboratory installation are research context, not a universal operating envelope or production acceptance criterion |
| NLR/NREL RTES analysis — https://www.nrel.gov/news/detail/program/2025/nlr-analysis-identifies-reservoir-thermal-energy-storage-as-a-solution-for-data-center-cooling-needs | Reservoir thermal energy storage, cold/warm wells, dry coolers/chillers, peak-shifting, heat exchangers, and heat-recovery scenario questions | The page reports an analysis and simulated scenarios; do not convert its COP or savings into a guaranteed field result or permit requirement |
| NREL/NLR sustainable buildings research — https://www.nrel.gov/news/feature/2020/buildings-research-sets-foundation-for-future-design | Warm-water liquid cooling, waste-heat reuse, evaporative cooling, thermosyphon/dry-cooler interaction, PUE dashboard, and measured-case-study questions | The NLR facility is a case study. Its PUE and reuse results are not universal targets and must not be presented as a generic design requirement |
| NLR/NREL liquid-cooling validation — https://www.nrel.gov/news/detail/program/2019/aquarius-hpc-cooling-system-yields-impressive-energy-efficiency-results-after-nrel-sandia-test | Cold-plate liquid-cooling heat capture, warmer supply/return water, dry-cooler potential, and test-method/evidence questions | The result is a particular evaluation of a particular system; the reported heat-capture percentage is not a universal rack or facility claim |
| Lawrence Berkeley National Laboratory, Liquid Cooling — https://datacenters.lbl.gov/liquid-cooling | Liquid-cooling technology families, heat transfer close to the source, warmer coolant opportunities, hybrid air/liquid architectures, and open resource discovery | LBNL’s overview is a technical landscape, not a standard or approval. Hybrid systems require an explicit residual-air-load and failure-mode review |

### Pass-04 question frontier

| Module | New question families |
|---|---|
| M01 Mission Critical | Trace heat from chip to rack, room, CDU/heat exchanger, chilled/condenser loop, tower/dry cooler/UTES, and heat-reuse sink; identify the dependency that turns a cooling-efficiency claim into a continuity risk. |
| M02 Standards | Classify a PUE/WUE formula, a FEMP BMP, an EPA WaterSense heading, an NLR case study, and a site control sequence as metric definition, guidance, research evidence, or local procedure; reject a case-study number presented as a standard. |
| M03 Site / Building | Compare air-cooled, evaporative, water-side-economizer, direct-liquid, immersion, dry-cooler, and thermal-storage concepts against climate, water source, discharge, heat-reuse sink, contamination, maintenance, noise, and expansion evidence. |
| M04 Floor / Ceiling | Review a liquid-cooling change for CDU location, pipe routing, leak detection, containment, service clearances, rack loading, floor penetrations, thermal movement, and the boundary between facility water and IT-side coolant. |
| M05 Light | Design inspection and emergency-lighting questions for wet mechanical rooms, CDUs, cooling towers, roof equipment, heat exchangers, and low-visibility leak response; distinguish a lighting control from a leak/safety interlock. |
| M06 Power | Evaluate fan/pump/chiller/tower/heat-recovery energy paths and control modes; identify the power and restart sequence when economizer, chiller, dry cooler, CDU, or thermal storage control is unavailable. |
| M07 EMF | Keep pump motors, variable-speed drives, heat-rejection equipment, and liquid-cooling power electronics in the field-source inventory; require frequency, location, instrument, boundary, and work-practice evidence before accepting an EMF claim. |
| M08 Racks | Compare rear-door heat exchangers, cold plates, immersion, and hybrid racks; ask what rack metadata identifies coolant type, supply/return, leak detection, service isolation, residual air load, and safe removal procedure. |
| M09 Cooling | Diagnose competing temperature, humidity, pressure, airflow, valve, fan, pump, and economizer loops; identify sensor placement, sequence-of-operations, fail-safe state, alarm owner, and manual fallback before changing a setpoint. |
| M10 Water | Build a mass-balance evidence pack for makeup, evaporation, drift, blowdown, leaks, treatment, reuse, discharge, and IT energy; calculate WUE only after agreeing on meter boundaries, time period, and water sources. |
| M11 Network | Map controls and telemetry for CRAH/CRAC, chiller, tower, dry cooler, CDU, heat exchanger, leak detection, water meters, and thermal storage; require timestamp, trend retention, command authority, segmentation, and local fallback. |
| M12 Fire | Review liquid-cooling and thermal-storage hazards separately from IT-room fire protection: leak response, electrical isolation, coolant properties, detection, suppression interfaces, water availability, egress, and impairment/return-to-service evidence. |
| M13 Security | Protect cooling controllers, water-treatment systems, thermal-storage controls, and heat-reuse interfaces as cyber-physical assets; identify the risk created by shared credentials, unsupported firmware, or unnecessary remote vendor access. |
| M14 Auxiliary | Design an alarm board that correlates temperature, dew point, pressure, flow, valve position, pump state, water chemistry, leak, tower drift/blowdown, and heat-reuse demand; distinguish a trend from an actionable alarm. |
| M15 Operations | Create an operating loop for seasonal economizer transitions, tower chemistry, filter maintenance, CDU isolation, leak response, thermal-storage charge/discharge, heat-reuse demand, and post-change verification; require evidence of actual performance rather than design intent. |

### Pass-04 factual boundaries

- FEMP identifies temperature and humidity over-control as a potential energy and water problem and describes both air- and water-side economizing. A question must preserve the need to evaluate air quality, humidity tolerance, climate, controls, and IT protection.
- Cooling-tower water leaves through distinct mechanisms, including evaporation, drift, and blowdown. The correct operational question is usually which measurement or control is missing; a universal cycles-of-concentration number is not justified by the public page.
- Direct liquid cooling can move heat closer to the source and reduce air-side work, but hybrid designs retain an air load. A question must identify the residual load and the CDU/heat-rejection boundary rather than treating “liquid cooled” as a complete architecture.
- EPA WaterSense provides public headings and guidance for metering, leak detection, single-pass cooling, cooling towers, and chilled-water systems. Its linked PDF bodies were not fetched, and its guidance does not certify a site.
- NLR/NREL’s cooling and heat-reuse pages are laboratory or analytical evidence. Reported PUE, heat-capture, COP, or savings values must remain attributed to the named installation or scenario, not promoted to universal targets.
- Reservoir thermal energy storage can shift cooling energy and may pair with heat recovery in a modeled scenario. It introduces wells, heat exchangers, water quality, controls, pumping, permitting, and recovery dependencies that must be tested as part of the operating boundary.
- PUE and WUE are boundary-sensitive ratios. A question must name the meter boundary, time basis, included water sources, and IT-energy denominator before comparing values.

### Updated pass tracker

| Pass | Focus | Status |
|---:|---|---|
| 01 | DOE/FEMP/NREL energy, cooling, water, commissioning; CISA/NIST OT and exposure; eCFR safety; ISO/TIA/IEEE edition anchors | COMPLETE |
| 02 | Site risk, utility interconnection, resilience, climate/flood/seismic, public AHJ/code sources | COMPLETE |
| 03 | Electrical distribution, switching, protection, UPS/generator/BESS, maintenance testing | COMPLETE |
| 04 | Cooling controls, liquid cooling, heat reuse, economization, thermal/water measurement | COMPLETE — this section |
| 05 | Cabling, outside plant, network management, BMS/DCIM segmentation, remote access | OPEN |
| 06 | Fire/life safety, emergency power, permits, PPE, incident command, return-to-service | OPEN |
| 07 | Physical security, personnel/vendor/visitor controls, asset lifecycle, media disposition | OPEN |
| 08 | People systems: skills matrices, shift turnover, training, fatigue, contractor governance, succession | OPEN |
| 09 | Commissioning, change control, MOP/SOP/EOP, alarms, metrics, reliability-centered maintenance | OPEN |
| 10 | Cross-module adversarial review: stale editions, unsupported numbers, false equivalences, and uncovered operational decisions | OPEN |

The objective remains open after pass 04. The next unresolved frontier is cabling, outside plant, network management, BMS/DCIM segmentation, and remote access.

## Breadth pass 05 — cabling, outside plant, network management, OT segmentation, and remote access

**Search date:** 2026-08-18. This pass retained official TIA, IEEE, NIST, and CISA HTML/abstract pages. PDF links surfaced by those pages were not opened or copied; vendor implementation blogs and marketing pages were excluded.

### Sources retained

| Source | What it can safely support | Boundaries |
|---|---|---|
| TIA-942-C, Telecommunications Infrastructure Standard for Data Centers — https://tiaonline.org/standard/tia-942/ | Current Revision C edition pin and public abstract for data-center infrastructure, telecommunications cabling, power, cooling, architecture, fire protection, safety, physical security, and management systems | The standard body is purchased; the public page pins scope and edition, not clause text, a rating decision, or certification of a facility or person |
| TIA-942 certification information — https://tiaonline.org/products-and-services/tia942certification/ | Distinguish the infrastructure standard from the separate certification program, licensed certification bodies, ratings, and public listing workflow | TIA certification information does not make a course completion a certification and does not replace an AHJ, owner, or network acceptance record |
| IEEE 802.3 Ethernet Working Group — https://www.ieee802.org/3/ | Public Ethernet standard-family and working-group anchor for PHY/media, link, interoperability, maintenance, and current project questions; the page lists active work and free-download pathways | Do not infer a universal distance, connector loss, power budget, or support matrix without the named IEEE PHY/media edition and link design evidence |
| NIST SP 800-215, Guide to a Secure Enterprise Network Landscape — https://csrc.nist.gov/pubs/sp/800/215/final | Multi-site/cloud network landscape, firewalls, microsegmentation, VPN/ZTNA, SASE, secure operations, and network-architecture evidence questions | NIST SP 800-215 is guidance, not a data-center cabling standard, a mandatory architecture, or proof that a chosen product is secure |
| NIST SP 800-82 Rev. 3, Guide to OT Security — https://csrc.nist.gov/pubs/sp/800/82/r3/final | OT topology, building automation, physical access control, environment monitoring/control, safety/reliability constraints, and security-countermeasure questions | NIST marks a Rev. 4 draft and possible updates; Rev. 3 final remains the edition anchor here. It is guidance, not an EPI heading, code adoption, or automatic compliance claim |
| CISA Cross-Sector Cybersecurity Performance Goals — https://www.cisa.gov/cybersecurity-performance-goals | Voluntary baseline questions for asset inventory, account controls, MFA, segmentation, monitoring, response, recovery, IT/OT ownership, and measurable risk reduction | CISA states the CPGs are voluntary and prioritized practices. They are not a data-center certification, an AHJ requirement, or permission to expose OT to the internet |
| CISA Modern Approaches to Network Access Security — https://www.cisa.gov/news-events/alerts/2024/06/18/cisa-and-partners-release-guidance-modern-approaches-network-access-security | Remote-access threat, VPN/misconfiguration risk, visibility, zero-trust/SSE/SASE, and review/approval questions | The alert is guidance for risk reduction; it does not select an architecture, require a product, or prove that a remote path is necessary |

### Pass-05 question frontier

| Module | New question families |
|---|---|
| M01 Mission Critical | Trace a service from application and management plane through core/edge switching, meet-me, carrier, outside plant, building pathways, BMS/DCIM, utility controls, and operator; identify the shared physical or logical dependency hidden by a “diverse” label. |
| M02 Standards | Classify TIA-942-C, IEEE 802.3, NIST SPs, CISA CPGs, adopted code, and site standards as catalog/abstract, technical standard, guidance, voluntary baseline, or binding rule; reject a stale TIA revision or an invented rating taxonomy. |
| M03 Site / Building | Review outside-plant entry, meet-me rooms, carrier demarcation, ducts, manholes, risers, firestopping, grounding/bonding, flood exposure, physical security, maintenance access, and carrier evidence before accepting a “carrier diverse” design. |
| M04 Floor / Ceiling | Inspect pathways, tray fill, bend radius, separation, support, access, firestopping, overhead/underfloor constraints, fiber/copper segregation, and change records; identify when a cable path change invalidates a tested link or maintenance route. |
| M05 Light | Build safe inspection and emergency-access questions for meet-me, riser, pathway, roof, manhole, and remote edge spaces; separate adequate illumination from egress, confined-space, traffic-control, and electrical-work requirements. |
| M06 Power | Trace power to active network, optical, BMS/DCIM, and carrier equipment; test diverse power feeds, UPS/generator dependency, PoE load, battery-backed edge cabinets, and the recovery sequence for a shared upstream power failure. |
| M07 EMF | Identify intentional RF, optical, copper, PoE, power, and grounding sources separately; require source, frequency, measurement method, worker boundary, and equipment documentation before accepting an EMF or interference conclusion. |
| M08 Racks | Review rack/cabinet identity, patching, fiber polarity, cable management, high-density switch cooling, structured pathways, labels, power, grounding, physical access, and spare capacity; locate the record needed for safe MAC work. |
| M09 Cooling | Test network-room thermal load, airflow, blanking, hot/cold aisle relation, switch inlet conditions, BMS alarms, and remote-room fallback; identify whether a cabling or switch expansion changes cooling and power envelopes. |
| M10 Water | Add outside-plant flood/water intrusion, manhole drainage, leak detection, water paths near meet-me/riser rooms, and carrier restoration constraints to the water/dependency plan; distinguish a site map from a tested protection measure. |
| M11 Network | Design a source-of-truth map for physical links, logical segments, VLAN/VRF/ACL/firewall boundaries, control protocols, management planes, time sources, telemetry, and failover; bind each claim to a named interface, owner, and test record. |
| M12 Fire | Review firestopping, pathway penetrations, cable flame/smoke evidence, detection, suppression, egress, emergency power, and impairment records; identify which cable or room change requires AHJ review and return-to-service testing. |
| M13 Security | Segment IT, OT, BMS, DCIM, access control, carrier, and vendor zones; define conduits, DMZs, allowlists, least privilege, MFA, jump access, logging, dormant-account removal, and an emergency break-glass path with review. |
| M14 Auxiliary | Build a network-management and OT-monitoring board that distinguishes inventory, link state, flow telemetry, alarm, command, configuration, log, time synchronization, and remote-access session evidence; identify who can acknowledge versus change state. |
| M15 Operations | Turn cabling and remote access into an operating loop: install/test/label, baseline, monitor, patch/change, authorize vendor access, review logs, exercise failover, restore, and retire credentials/ports; require diagrams and test evidence to stay current. |

### Pass-05 factual boundaries

- TIA-942-C is the current public TIA edition pin used here: Revision C, published May 2024. The public abstract supports infrastructure and scope questions; it does not authorize copying paid body text or treating a rating as a universal operational target.
- IEEE 802.3 is a family of standards and active work items. Link reach, optical budget, copper category, connector, and power assumptions must be tied to the named PHY/media and tested installation rather than a generic Ethernet claim.
- NIST SP 800-82 Rev. 3 explicitly includes building automation, physical access control, and physical-environment monitoring/control systems in OT examples. Its security controls must be adapted to safety, reliability, and performance constraints; “segment it” is not a sufficient sequence-of-operations answer.
- NIST SP 800-215 covers the modern enterprise network landscape, including multi-site data centers, cloud access, microsegmentation, VPN, and ZTNA. It is architecture guidance, not a certification or a replacement for physical cabling evidence.
- CISA’s Cross-Sector CPGs are voluntary prioritized practices for IT and OT owners. They support questions about inventory, MFA, segmentation, monitoring, and recovery but do not create a universal legal requirement or availability class.
- Remote access to BMS/DCIM/OT is a risk-bearing operating decision. Necessity, asset scope, user role, least privilege, MFA, private path/DMZ, logging, time limit, approval, and local fallback must be explicit; vendor convenience is not evidence of necessity.
- Physical diversity is not established by different labels alone. A robust question checks carrier, provider, duct bank, manhole, riser, entrance, room, power, network equipment, control plane, regional hazard, and maintenance dependencies.

### Updated pass tracker

| Pass | Focus | Status |
|---:|---|---|
| 01 | DOE/FEMP/NREL energy, cooling, water, commissioning; CISA/NIST OT and exposure; eCFR safety; ISO/TIA/IEEE edition anchors | COMPLETE |
| 02 | Site risk, utility interconnection, resilience, climate/flood/seismic, public AHJ/code sources | COMPLETE |
| 03 | Electrical distribution, switching, protection, UPS/generator/BESS, maintenance testing | COMPLETE |
| 04 | Cooling controls, liquid cooling, heat reuse, economization, thermal/water measurement | COMPLETE |
| 05 | Cabling, outside plant, network management, BMS/DCIM segmentation, remote access | COMPLETE — this section |
| 06 | Fire/life safety, emergency power, permits, PPE, incident command, return-to-service | OPEN |
| 07 | Physical security, personnel/vendor/visitor controls, asset lifecycle, media disposition | OPEN |
| 08 | People systems: skills matrices, shift turnover, training, fatigue, contractor governance, succession | OPEN |
| 09 | Commissioning, change control, MOP/SOP/EOP, alarms, metrics, reliability-centered maintenance | OPEN |
| 10 | Cross-module adversarial review: stale editions, unsupported numbers, false equivalences, and uncovered operational decisions | OPEN |

The objective remains open after pass 05. The next unresolved frontier is fire/life safety, emergency power, permits, PPE, incident command, and return-to-service.

## Breadth pass 06 — fire/life safety, emergency response, PPE, permits, and return-to-service

**Search date:** 2026-08-18. This pass retained official OSHA, FEMA/USFA, and NFPA catalog/preview HTML pages. No PDF body was fetched or copied.

### Sources retained

| Source | What it can safely support | Boundaries |
|---|---|---|
| OSHA 1910.132, General PPE requirements — https://www.osha.gov/laws-regs/regulations/standardnumber/1910/1910.132 | Written hazard assessment, PPE selection/fit, condition, training, demonstration of understanding, retraining, and employer responsibility questions | The rule is workplace PPE law; it does not select a data-center arc-flash boundary, fire-system design, or AHJ permit condition by itself |
| OSHA 1910.137, Electrical Protective Equipment — https://www.osha.gov/laws-regs/regulations/standardnumber/1910/1910.137 | Electrical protective-equipment maintenance and periodic-test evidence; this is carried forward from pass 03 for the emergency-work interface | Preserve the rule’s equipment scope and tables; do not substitute a generic interval, vendor statement, or training certificate |
| OSHA 1910.333, Selection and use of work practices — https://www.osha.gov/laws-regs/regulations/standardnumber/1910/1910.333 | Deenergization, lockout/tagging, stored-energy control, verification, qualified-person, energized-work, and reenergization evidence | OSHA is the legal anchor for workplace electrical practices; local electrical/fire code and owner procedures still govern other acceptance questions |
| OSHA 1910.38 / emergency preparedness resources — https://www.osha.gov/emergency-preparedness/getting-started | Emergency action plan triggers, reporting, evacuation, critical-operations shutdown, accountability, rescue/medical roles, contacts, training, and review questions | OSHA’s EAP requirements are workplace requirements; they do not create a complete incident-command plan, facility recovery objective, or fire permit |
| OSHA Evacuation Plans and Procedures — https://www.osha.gov/etools/evacuation-plans-procedures/eap/ | Public EAP purpose and minimum worksite-specific planning questions, including layout, emergency systems, and worker actions | Use as an operations/safety source, not as a substitute for AHJ-approved life-safety design or emergency-services coordination |
| NFPA 72, National Fire Alarm and Signaling Code, 2025 preview — https://link.nfpa.org/all-publications/72/2025 | Public edition pin and headings for documentation, circuits/pathways, inspection/testing/maintenance, emergency-control interfaces, emergency communications, and protected-premises systems | The standard body is subscription-controlled; preview headings do not prove a clause, test interval, design value, or AHJ acceptance |
| NFPA 75, Standard for the Fire Protection of Information Technology Equipment, 2024 catalog — https://link.nfpa.org/all-publications/75/2024 | Public heading and edition pin for IT-equipment fire-protection question families | Catalog receipt is not the paid body and does not determine the local fire-protection design, suppression agent, or permit |
| NFPA 76, Standard for the Fire Protection of Telecommunications Facilities, 2024 catalog — https://link.nfpa.org/all-publications/76/2024 | Public heading and edition pin for telecommunications-facility fire-protection, continuity, and impairment questions | Do not conflate telecommunications-facility guidance with NFPA 75, NFPA 72, an adopted fire code, or a facility certification |
| NFPA 110, Standard for Emergency and Standby Power Systems, 2025 preview — https://link.nfpa.org/all-publications/110/2025 | Emergency/standby power source, transfer, testing, maintenance, and impairment question labels; carried forward from pass 03 | Paid body not fetched; no universal classification, generator schedule, fuel autonomy, or return-to-service criterion is inferred |
| USFA/FEMA National Incident Management System — https://www.usfa.fema.gov/a-z/nims/index.html | Shared vocabulary, systems, and processes for government, nongovernmental, and private-sector incident prevention, response, and recovery; incident-command exercise prompts | NIMS is a framework, not a credential, site-specific command chart, legal delegation, or guarantee that an incident will be managed successfully |

### Pass-06 question frontier

| Module | New question families |
|---|---|
| M01 Mission Critical | Connect life safety, fire protection, emergency power, communications, staffing, utility control, evacuation, critical-load shutdown, damage assessment, and recovery; identify which dependency makes “continuous operation” unsafe or unproven. |
| M02 Standards | Classify OSHA rules, NFPA catalog/preview headings, adopted fire/electrical code, AHJ permit, NIMS framework, and site EOP; pin editions and authority; reject a paid catalog receipt as proof of body-level compliance. |
| M03 Site / Building | Review fire department access, water supply, fire pump/standpipe interfaces, fire separations, battery/generator rooms, fuel, egress, emergency access, alarm transmission, and AHJ inspection/permit records before occupancy or energization. |
| M04 Floor / Ceiling | Inspect penetrations, rated assemblies, firestopping, cable trays, raised-floor plenums, overhead obstructions, egress paths, signage, emergency lighting, and equipment clearances after a build or cabling change. |
| M05 Light | Test normal/emergency illumination, exit visibility, alarm strobes/notification, generator-room work lighting, battery and mechanical room access, and the inspection record for a failed or impaired lighting circuit. |
| M06 Power | Coordinate emergency/standby sources, ATS/UPS bypass, load shedding, generator room safety, battery/storage isolation, fire alarm interfaces, transfer testing, and safe restoration; distinguish emergency power from an availability promise. |
| M07 EMF | Tie electrical protective equipment and work boundaries to the hazard assessment, energized/deenergized state, measurement/verification, PPE condition, and qualified-person role; reject a “PPE solves it” answer without task-specific evidence. |
| M08 Racks | Evaluate rack and aisle fire detection, suppression interface, cable penetrations, liquid-cooling leak response, emergency shutdown, asset accountability, egress, and return-to-service records after an event or impairment. |
| M09 Cooling | Test cooling shutdown/interlock behavior during fire alarm, smoke control, loss of power, loss of water, leak, and evacuation; identify the safe state and the evidence required before restarting cooling or IT load. |
| M10 Water | Review fire-water supply, storage, impairment, drainage, cooling-water isolation, contaminated runoff, battery/generator spill response, and utility notifications; distinguish fire-flow evidence from cooling-water availability. |
| M11 Network | Verify alarm signaling, emergency communications, carrier diversity, fire panel/BMS/DCIM boundaries, out-of-band operations, local manual control, and communications fallback when normal network paths fail. |
| M12 Fire | Build an evidence matrix for detection, alarm, signaling, emergency control functions, suppression, egress, fire doors/penetrations, inspection/testing/maintenance, impairment, AHJ acceptance, and return-to-service. |
| M13 Security | Coordinate responder access, keys/badges, visitor/vendor control, incident evidence, emergency overrides, cyber-physical isolation, and re-entry authorization; identify where security controls could obstruct life-safety response. |
| M14 Auxiliary | Assign owners for alarm receipt, acknowledgement, escalation, fire-panel/BMS/DCIM interlocks, emergency communications, event logging, and manual fallback; distinguish an alarm indication from a verified protective action. |
| M15 Operations | Run a NIMS-informed tabletop from detection through command, evacuation/shelter, critical operations shutdown, accountability, responder handoff, damage assessment, impairment control, staged restoration, test, and documented return to service. |

### Pass-06 factual boundaries

- OSHA 1910.132 requires hazard assessment, appropriate PPE selection, fit, training, and demonstrated understanding. The PPE requirement is task and hazard based; it is not fulfilled by possessing generic PPE or a course completion.
- OSHA 1910.38 and OSHA’s emergency-planning resources support written EAP elements, worker roles, reporting, evacuation, accountability, critical operations, training, and review. They do not replace an AHJ-approved fire/life-safety design or a site-specific incident command plan.
- NFPA 72’s public 2025 preview exposes headings for documentation, inspection/testing/maintenance, emergency-control interfaces, pathways, and emergency communications. The paid body was not fetched, so no clause or interval is asserted.
- NFPA 75 and NFPA 76 are distinct public headings for IT-equipment and telecommunications-facility fire protection. Do not merge their scopes or infer suppression design from a title.
- NFPA 110 provides an emergency/standby-power heading and edition receipt, but an emergency source is not automatically a life-safety circuit, an availability class, or a full return-to-service plan.
- NIMS provides common incident-management vocabulary and processes across public and private stakeholders. It does not create a private operator credential, replace local command authority, or certify that a facility is ready.
- Fire alarm, suppression, emergency power, egress, emergency communications, PPE, and incident command are separate evidence domains. A single “fire test passed” receipt cannot prove all of them.
- Return to service requires an explicit impairment record, hazard controls, inspection/test results, restoration sequence, authority/owner sign-off, and monitoring period. “The equipment restarted” is not acceptance evidence.

### Updated pass tracker

| Pass | Focus | Status |
|---:|---|---|
| 01 | DOE/FEMP/NREL energy, cooling, water, commissioning; CISA/NIST OT and exposure; eCFR safety; ISO/TIA/IEEE edition anchors | COMPLETE |
| 02 | Site risk, utility interconnection, resilience, climate/flood/seismic, public AHJ/code sources | COMPLETE |
| 03 | Electrical distribution, switching, protection, UPS/generator/BESS, maintenance testing | COMPLETE |
| 04 | Cooling controls, liquid cooling, heat reuse, economization, thermal/water measurement | COMPLETE |
| 05 | Cabling, outside plant, network management, BMS/DCIM segmentation, remote access | COMPLETE |
| 06 | Fire/life safety, emergency power, permits, PPE, incident command, return-to-service | COMPLETE — this section |
| 07 | Physical security, personnel/vendor/visitor controls, asset lifecycle, media disposition | OPEN |
| 08 | People systems: skills matrices, shift turnover, training, fatigue, contractor governance, succession | OPEN |
| 09 | Commissioning, change control, MOP/SOP/EOP, alarms, metrics, reliability-centered maintenance | OPEN |
| 10 | Cross-module adversarial review: stale editions, unsupported numbers, false equivalences, and uncovered operational decisions | OPEN |

The objective remains open after pass 06. The next unresolved frontier is physical security, personnel/vendor/visitor controls, asset lifecycle, and media disposition.

## Breadth pass 07 — physical security, people and vendors, asset lifecycle, and media disposition

**Search date:** 2026-08-18. This pass retained official NIST and CISA HTML/catalog pages. NIST SP 800-88 Rev. 2 is current; its standard body and all PDF resources were not fetched.

### Sources retained

| Source | What it can safely support | Boundaries |
|---|---|---|
| NIST SP 800-53 Rev. 5, Security and Privacy Controls — https://csrc.nist.gov/pubs/sp/800/53/r5/upd1/final | Control-family and evidence questions spanning access control, awareness/training, audit, contingency, maintenance, media protection, physical/environmental protection, personnel security, system acquisition, supply chain, and risk assessment | NIST identifies a 5.2.0 minor release in the planning note. The catalog is customizable guidance, not a universal compliance claim, facility rating, or employee credential |
| NIST SP 800-88 Rev. 2, Guidelines for Media Sanitization — https://csrc.nist.gov/pubs/sp/800/88/r2/final | Current media-sanitization program, sensitivity-based disposition, validation, cryptographic-erase, reuse, destruction, custody, and disposal evidence questions | Rev. 2 supersedes withdrawn Rev. 1. The final page is the edition pin; no body or PDF text was fetched, and no fixed sanitization method is inferred without media, sensitivity, and approved-process evidence |
| NIST SP 800-53 control catalog DOI — https://doi.org/10.6028/NIST.SP.800-53r5 | Stable official publication identity for control-family references and source receipts | A DOI identifies the publication; it does not prove that a specific control was selected, implemented, assessed, or accepted for a facility |
| CISA Commercial Facilities Publications — https://www.cisa.gov/commercial-facilities-resources | Owner/operator security planning, protective measures, evacuation, coordination, training, and resilience question families | CISA describes voluntary resources and sector collaboration. Many resources are venue-oriented or access-controlled; do not silently turn them into data-center requirements or fetch restricted documents |
| CISA Hometown Security / Physical Security — https://www.cisa.gov/hometown-security | Foundational physical-security planning, security-plan development, threat/vulnerability/protective-measure framing, and owner/operator coordination | CISA guidance is not an AHJ inspection, a guard-force staffing rule, or a certification program |
| CISA Personal Security Considerations Action Guide — https://www.cisa.gov/resources-tools/resources/personal-security-considerations-action-guide | Personnel security posture, on/off-job considerations, suspicious activity, and worker protective-awareness questions | The guide is general critical-infrastructure guidance; it does not provide a background-check legal conclusion or replace HR, labor, privacy, or local law review |
| CISA Cross-Sector Cybersecurity Performance Goals — https://www.cisa.gov/cybersecurity-performance-goals | Asset inventory, account lifecycle, privileged access, MFA, vendor/third-party access, logging, recovery, and risk-prioritization questions | CISA states the CPGs are voluntary practices and notes there is no official assessor certification program; do not claim CPG certification |

### Pass-07 question frontier

| Module | New question families |
|---|---|
| M01 Mission Critical | Map service criticality to people, perimeter, visitor/vendor, keys/badges, control systems, asset records, media, and emergency response; identify the human or custody dependency hidden by a technical redundancy claim. |
| M02 Standards | Classify NIST controls, CISA guidance, owner policy, contract, AHJ requirement, privacy/labor rule, and site procedure; identify the selected control, evidence owner, assessment method, and residual risk without inventing a security taxonomy. |
| M03 Site / Building | Review perimeter, vehicle approach, loading dock, roof/mechanical access, utility/telecom entries, camera coverage, lighting, barriers, blast/flood/wildfire exposure, responder access, and zoning records as one layered physical-security design. |
| M04 Floor / Ceiling | Inspect secure zones, cages, mantraps, doors, locks, ceiling voids, raised-floor access, penetrations, shared corridors, cameras, tamper evidence, and maintenance paths; find the bypass created by a ceiling or pathway change. |
| M05 Light | Evaluate lighting for perimeter, parking/loading, camera identification, badge/visitor processing, emergency egress, rooftop/mechanical access, and guard patrol; distinguish a lux claim from a documented security and life-safety need. |
| M06 Power | Protect switchgear, UPS, generator, fuel, battery, and BMS/DCIM rooms from unauthorized access while preserving safe emergency response; test badge failure, mechanical override, key custody, and secure local operation. |
| M07 EMF | Separate access to field sources, measurement equipment, energized rooms, PPE, and exposure records; require authorized personnel, instrument custody, measurement method, and data integrity rather than a generic “restricted area” claim. |
| M08 Racks | Track rack/cabinet ownership, serials, removable drives, smart PDUs, console ports, spares, labels, locks, tamper evidence, and maintenance access; identify who may remove a component and what record closes the custody loop. |
| M09 Cooling | Secure cooling controls, valves, pumps, CDUs, water treatment, heat-reuse interfaces, roofs, towers, and plant rooms; identify how a vendor or visitor can alter a cooling state and how the action is authorized, logged, and reversed. |
| M10 Water | Include water-treatment chemicals, cooling-tower access, drains, sampling points, alternative-water connections, leak sensors, and wastewater interfaces in the asset and access register; distinguish physical access control from water-quality verification. |
| M11 Network | Reconcile physical ports, cabinets, meet-me rooms, management-plane accounts, jump hosts, vendor VPNs, carrier demarcations, and out-of-band paths; require owner, purpose, expiry, monitoring, and removal evidence for third-party access. |
| M12 Fire | Balance security doors, locks, turnstiles, barriers, cameras, and visitor controls with fire egress, responder override, alarm release, emergency access, and accountability; identify a control that increases security while creating a life-safety failure. |
| M13 Security | Build a layered program across deterrence, detection, delay, response, cyber-physical protection, personnel trust, vendor governance, and incident evidence; assess threats and consequences without claiming a universal perimeter design. |
| M14 Auxiliary | Design a source-of-truth board for badge/visitor, camera, intrusion, asset, alarm, BMS/DCIM, vendor-session, and media-disposition events; assign retention, access, timestamp, privacy, escalation, and audit ownership. |
| M15 Operations | Run asset lifecycle from procurement and receiving through install, custody, maintenance, relocation, loan/spare, decommission, sanitization, destruction/reuse, vendor return, record retention, and audit; require evidence at every handoff. |

### Pass-07 factual boundaries

- NIST SP 800-53 is a customizable control catalog. A control family or identifier is not evidence of implementation, and implementation is not the same as an independent assessment or acceptance.
- NIST SP 800-88 Rev. 2 is the current media-sanitization edition; Rev. 1 is withdrawn. Media disposition must be sensitivity- and media-aware, validated, documented, and tied to an approved program. Do not reuse obsolete Rev. 1 receipts.
- CISA commercial-facilities and hometown-security pages support protective planning, coordination, training, and owner/operator questions. They do not issue a facility rating or define a universal guard, camera, badge, fence, or visitor policy.
- CISA CPGs are voluntary, prioritized practices and CISA states there is no official assessor certification program. They can anchor evidence questions but cannot be represented as a credential or compliance seal.
- Personnel, visitor, vendor, and contractor controls intersect privacy, labor, procurement, safety, and local law. A data-center question should ask for the governing authority and evidence owner rather than inventing a “trusted operator” classification.
- Physical security must preserve emergency egress and responder access. A locked door, anti-tailgating control, or badge failure mode is not acceptable if it blocks required evacuation or emergency intervention.
- Asset identity and custody are separate from media sanitization. A wiped drive without a verified asset record, disposition authority, chain of custody, and closure evidence is not a complete lifecycle outcome.
- NIST SP 800-88 Rev. 2 final was recorded from the official publication page; no PDF body was opened or copied.

### Updated pass tracker

| Pass | Focus | Status |
|---:|---|---|
| 01 | DOE/FEMP/NREL energy, cooling, water, commissioning; CISA/NIST OT and exposure; eCFR safety; ISO/TIA/IEEE edition anchors | COMPLETE |
| 02 | Site risk, utility interconnection, resilience, climate/flood/seismic, public AHJ/code sources | COMPLETE |
| 03 | Electrical distribution, switching, protection, UPS/generator/BESS, maintenance testing | COMPLETE |
| 04 | Cooling controls, liquid cooling, heat reuse, economization, thermal/water measurement | COMPLETE |
| 05 | Cabling, outside plant, network management, BMS/DCIM segmentation, remote access | COMPLETE |
| 06 | Fire/life safety, emergency power, permits, PPE, incident command, return-to-service | COMPLETE |
| 07 | Physical security, personnel/vendor/visitor controls, asset lifecycle, media disposition | COMPLETE — this section |
| 08 | People systems: skills matrices, shift turnover, training, fatigue, contractor governance, succession | OPEN |
| 09 | Commissioning, change control, MOP/SOP/EOP, alarms, metrics, reliability-centered maintenance | OPEN |
| 10 | Cross-module adversarial review: stale editions, unsupported numbers, false equivalences, and uncovered operational decisions | OPEN |

The objective remains open after pass 07. The next unresolved frontier is people systems: skills matrices, shift turnover, training, fatigue, contractor governance, and succession.

## Breadth pass 08 — skills, shift turnover, training, fatigue, contractors, and succession

**Search date:** 2026-08-18. This pass retained official OSHA, CDC/NIOSH, DOE/FEMP, and NIST/NICE HTML pages. No PDF body was fetched or copied. Workforce frameworks are used for role and capability language only, not to claim credentials.

### Sources retained

| Source | What it can safely support | Boundaries |
|---|---|---|
| OSHA 1910.332, Electrical Safety Training — https://www.osha.gov/laws-regs/regulations/standardnumber/1910/1910.332 | Job-assignment-based electrical training, qualified/unqualified worker distinctions, and familiarity with applicable safety-related work practices | OSHA training requirements do not create a data-center operator credential or prove competence for every task, voltage, system, or local procedure |
| OSHA 1910.147, Control of Hazardous Energy — https://www.osha.gov/laws-regs/regulations/standardnumber/1910/1910.147 | Authorized/affected employee roles, energy-control procedures, training, periodic inspection, and outside-personnel coordination questions | Apply the standard’s actual scope and employer program; do not invent a handover taxonomy or claim that a sign-in sheet proves LOTO competence |
| OSHA Emergency Preparedness and Response — https://www.osha.gov/emergency-preparedness/getting-started | Training, emergency action plan review, rescue/medical roles, PPE, electrical protection, hazardous energy, and permit-related source discovery | OSHA’s list is a navigation and compliance resource, not a complete skills matrix or staffing model |
| CDC/NIOSH Fatigue and Work — https://www.cdc.gov/niosh/fatigue/about/index.html | Fatigue sources, nonstandard schedules, extended hours, effects on reaction/attention/judgment, joint employer-worker risk reduction, and research/training questions | NIOSH is research and prevention guidance; it does not impose a universal shift length, rest roster, medical screen, or staffing ratio |
| NIOSH Center for Work and Fatigue Research — https://www.cdc.gov/niosh/centers/fatigue.html | Fatigue-risk assessment, work-schedule design, collaboration with workers/employers, and evidence-review questions | The center’s research agenda is not an operational certification or one-size-fits-all fatigue program |
| DOE/FEMP workforce development offerings — https://www.energy.gov/femp/articles/femps-workforce-development-offerings | Public training-curriculum and workforce-development framing for building operations and maintenance, energy management, and related federal roles | FEMP offerings do not certify a private operator, define a data-center qualification, or replace site-specific supervised task sign-off |
| NIST NICE Framework Resource Center — https://www.nist.gov/itl/applied-cybersecurity/nice/nice-framework-resource-center | Task, knowledge, skill, competency-area, work-role, team, hiring, training, capability-planning, and career-development language; current component version discovery | NICE describes cybersecurity work and capabilities, not all data-center operations. Work roles are not job titles, and framework use does not certify a person |
| NIST NICE Framework Current Versions — https://www.nist.gov/itl/applied-cybersecurity/nice/nice-framework-resource-center/nice-framework-current-versions | Current component version pin (2.2.0 shown on the page), change/version review, and machine-readable workforce-component discovery | The component version is a workforce-language receipt, not an EPI/CDFOS/CDFOM syllabus heading or credentialing decision |
| NIST SP 800-53 Rev. 5 control catalog — https://csrc.nist.gov/pubs/sp/800/53/r5/upd1/final | Awareness/training, personnel security, maintenance, contingency, supply-chain, audit, and assessment evidence families for role and contractor-governance questions | Controls are customizable guidance. A mapped control or training record does not prove practical task competence or an approved staffing level |

### Pass-08 question frontier

| Module | New question families |
|---|---|
| M01 Mission Critical | Map service risk to minimum roles, on-call coverage, decision rights, fatigue exposure, shift handoff, contractor dependency, cross-training, and succession; identify the capability that is single-threaded despite technical redundancy. |
| M02 Standards | Classify OSHA requirements, NIOSH research, FEMP workforce guidance, NICE work-role language, NIST control families, owner policy, and supervised task authorization; reject a framework label used as a certification claim. |
| M03 Site / Building | Design staffing and access coverage for utility rooms, mechanical plant, meet-me, security desk, loading dock, roofs, remote sites, and emergency assembly; identify the tasks requiring local qualified personnel rather than remote observation. |
| M04 Floor / Ceiling | Include safe access, lifting, cabling, leak, egress, and equipment-room task hazards in a skills matrix; identify which training and supervised demonstration is required after a physical layout or equipment change. |
| M05 Light | Train workers to recognize lighting/egress deficiencies during night shift, emergency response, and contractor work; include reporting, temporary controls, and escalation rather than treating illumination as a facilities-only issue. |
| M06 Power | Build role-based authorization for switching, LOTO, UPS bypass, generator/ATS tests, battery/storage work, and restoration; verify training currency, task-specific competence, second-person requirements, and handoff evidence before a window. |
| M07 EMF | Define who may measure fields, interpret results, establish boundaries, select PPE, authorize access, and release the work area; distinguish awareness training from qualified electrical work and occupational-health review. |
| M08 Racks | Create skill and authorization paths for rack install, fiber/copper work, liquid-cooling service, PDU/branch-circuit handling, console access, asset custody, and safe removal; require mentoring or supervised demonstration for new tasks. |
| M09 Cooling | Assign capabilities for control-sequence review, sensor validation, CDU isolation, water chemistry, tower work, leak response, heat-reuse operation, and seasonal transition; test whether the shift can operate manually when the automation path fails. |
| M10 Water | Train operators and contractors on water-source isolation, treatment chemicals, cooling-tower sampling, spill/leak response, makeup/blowdown records, wastewater notifications, and safe restart; require role and permit evidence for each task. |
| M11 Network | Define role boundaries for physical cabling, network operations, OT/BMS/DCIM, security, vendor remote access, change approval, break-glass, and incident response; verify shift turnover includes topology, active maintenance, alarms, and expired access. |
| M12 Fire | Train and exercise roles for alarm response, evacuation, impairment, fire-watch, responder escort, emergency shutdown, re-entry, and return-to-service; distinguish awareness from firefighter, electrical, confined-space, or AHJ-required qualifications. |
| M13 Security | Govern guard, operator, engineer, vendor, escort, administrator, and incident-manager roles; require background/access decisions, least privilege, separation of duties, incident reporting, and immediate access revocation on role change. |
| M14 Auxiliary | Define who owns alarms, logs, dashboards, shift reports, work orders, training records, contractor permits, access reviews, and evidence retention; identify the handoff datum that is missing when a dashboard is green but the operator is unaware of a risk. |
| M15 Operations | Build a people operating loop: roster, competency matrix, task authorization, shift brief, fatigue check, MOP/SOP/EOP review, execution, debrief, learning record, retraining, cross-training, succession, and contractor performance review. |

### Pass-08 factual boundaries

- OSHA 1910.332 ties electrical safety training to the hazards and job assignments employees face. A generic “data-center trained” label is not proof that a worker is qualified for switching, energized work, LOTO, battery, or restoration tasks.
- OSHA 1910.147 includes training and coordination for hazardous-energy control and outside personnel. Contractor participation requires compatible procedures, scope clarity, communication, and responsibility; a vendor badge is not an energy-control authorization.
- NIOSH identifies fatigue as a safety and health risk associated with nonstandard schedules, extended hours, stress, demanding work, and hot environments. Use risk assessment and worker reporting; do not assert a universal roster or medical conclusion.
- FEMP workforce offerings support curriculum and development planning for federal building O&M roles. Completion of training is not a facility qualification, an EPI/CDFOS/CDFOM credential, or supervised task authorization.
- The NICE Framework describes cybersecurity work through tasks, knowledge, skills, competencies, roles, and teams. NIST says work roles are not synonymous with job titles; do not invent a data-center OLA or job taxonomy from NICE.
- NIST’s NICE current-version page shows components are separately maintained and versioned. A version receipt supports reproducibility of role language, not competence, hiring, or certification.
- Shift turnover is an evidence transfer, not a ceremonial meeting. It should carry current state, active risk, inhibited alarms, work permits, access changes, pending decisions, and a clear receiving owner.
- Succession planning is a resilience control. It does not mean every person can perform every task; critical tasks still require authority, training, supervised demonstration, and current procedure access.

### Updated pass tracker

| Pass | Focus | Status |
|---:|---|---|
| 01 | DOE/FEMP/NREL energy, cooling, water, commissioning; CISA/NIST OT and exposure; eCFR safety; ISO/TIA/IEEE edition anchors | COMPLETE |
| 02 | Site risk, utility interconnection, resilience, climate/flood/seismic, public AHJ/code sources | COMPLETE |
| 03 | Electrical distribution, switching, protection, UPS/generator/BESS, maintenance testing | COMPLETE |
| 04 | Cooling controls, liquid cooling, heat reuse, economization, thermal/water measurement | COMPLETE |
| 05 | Cabling, outside plant, network management, BMS/DCIM segmentation, remote access | COMPLETE |
| 06 | Fire/life safety, emergency power, permits, PPE, incident command, return-to-service | COMPLETE |
| 07 | Physical security, personnel/vendor/visitor controls, asset lifecycle, media disposition | COMPLETE |
| 08 | People systems: skills matrices, shift turnover, training, fatigue, contractor governance, succession | COMPLETE — this section |
| 09 | Commissioning, change control, MOP/SOP/EOP, alarms, metrics, reliability-centered maintenance | OPEN |
| 10 | Cross-module adversarial review: stale editions, unsupported numbers, false equivalences, and uncovered operational decisions | OPEN |

The objective remains open after pass 08. The next unresolved frontier is commissioning, change control, MOP/SOP/EOP, alarms, metrics, and reliability-centered maintenance.

## Breadth pass 09 — commissioning, change control, procedures, alarms, metrics, and reliability-centered maintenance

**Search date:** 2026-08-18. This pass retained official DOE/FEMP, NIST, and CISA HTML/abstract pages. No PDF body was fetched or copied.

### Sources retained

| Source | What it can safely support | Boundaries |
|---|---|---|
| DOE FEMP Commissioning Process for Federal Facilities — https://www.energy.gov/cmei/femp/commissioning-process-federal-facilities | A public four-step commissioning loop: plan, investigate, implement, and hand off/integrate; team, documentation, functional tests, deficiencies, retesting, monitoring, reporting, and future-commissioning questions | FEMP is federal guidance. The process does not by itself establish acceptance criteria, a data-center availability class, or an EPI/CDFOS/CDFOM credential |
| DOE FEMP Commissioning in Federal Buildings — https://www.energy.gov/cmei/femp/commissioning-federal-buildings | New, ongoing, recommissioning, and retro-commissioning distinctions; functional testing, monitoring, documentation, training, and selection by building condition | The page gives process definitions and examples, not a universal commissioning frequency or performance guarantee |
| DOE FEMP Operations and Maintenance in Federal Facilities — https://www.energy.gov/cmei/femp/operations-and-maintenance-federal-facilities | Reliability, safety, energy/water efficiency, controls, leaks, O&M programs, EMIS, re-tuning, commissioning, and operations-resource questions | O&M guidance is not a legal maintenance interval, RCM certification, or substitute for manufacturer/AHJ/owner requirements |
| DOE FEMP Performance Assurance Planning for UESCs — https://www.energy.gov/cmei/femp/performance-assurance-planning-utility-energy-service-contracts | Commissioning, training, implementation, sustained O&M, life-cycle measurement, verification, and performance-assurance evidence | UESC-specific statutory/contract context must not be generalized to every data-center project or used as an acceptance shortcut |
| DOE FEMP Federal UESC Phase 5 — https://www.energy.gov/cmei/femp/federal-uesc-process-phase-5-post-acceptance-performance | Post-acceptance commissioning, witnessed tests, actual operating conditions, records, performance assurance, and sustained-performance questions | Paying invoices or finishing construction is not performance acceptance; UESC roles and contract terms remain context-specific |
| NIST SP 800-128, Security-Focused Configuration Management — https://csrc.nist.gov/pubs/sp/800/128/upd1/final | Baselines, configuration identification/control, change impact, monitoring, risk, and documented rollback/evidence questions for IT and control-support systems | NIST SP 800-128 is security-focused configuration guidance, not a facility MOP/SOP/EOP standard or a permit-to-work procedure |
| NIST SP 800-137, Information Security Continuous Monitoring — https://csrc.nist.gov/pubs/sp/800/137/final | Monitoring strategy, asset/control visibility, risk tolerance, effectiveness, timely response, and trend/metric questions | ISCM is information-security guidance; do not use it as a universal BMS alarm policy or infer that telemetry equals control authority |
| NIST SP 800-61 Rev. 3, Incident Response Recommendations — https://csrc.nist.gov/pubs/sp/800/61/r3/final | Current incident-preparation, detection, response, recovery, lessons-learned, and integration-with-risk-management question families | This is cyber incident-response guidance. It does not replace physical emergency procedures, fire command, utility switching rules, or local legal obligations |
| NIST SP 800-53 Rev. 5 control catalog — https://csrc.nist.gov/pubs/sp/800/53/r5/upd1/final | Configuration management, contingency, incident response, continuous monitoring, maintenance, audit, training, and assessment evidence mappings | A control mapping is not proof that a procedure works in live operations or that a metric has an accountable owner |

### Pass-09 question frontier

| Module | New question families |
|---|---|
| M01 Mission Critical | Build an end-to-end assurance loop from owner objective to design intent, commissioning, operating procedure, alarm, work order, metric, incident, corrective action, and recommissioning; identify where evidence stops being current. |
| M02 Standards | Separate statute/code, standard catalog, guidance, contract, owner requirement, MOP/SOP/EOP, alarm rule, metric definition, and acceptance record; reject a procedure label that claims authority it does not have. |
| M03 Site / Building | Review commissioning scope for utility, structure, electrical, mechanical, controls, fire, security, network, water, documentation, seasonal conditions, and operator training; identify missing integrated tests before occupancy or load growth. |
| M04 Floor / Ceiling | Apply change control to penetrations, pathways, equipment placement, airflow, firestopping, access, load, and maintainability; require an updated drawing, impact review, test, and owner acceptance rather than an as-built filename alone. |
| M05 Light | Include normal/emergency lighting in commissioning, alarm, inspection, and change-control evidence; test failure modes, local override, notification, egress, and restoration rather than accepting a fixture count. |
| M06 Power | Design MOP/SOP/EOP and test evidence for normal switching, maintenance bypass, transfer, generator, UPS, battery, protection, load shed, black-start/islanding claims, and restoration; identify an alarm that can be acknowledged but not safely acted upon. |
| M07 EMF | Put field measurement, hazard assessment, PPE, work authorization, signage, monitoring, and post-change verification into the change/commissioning record; distinguish a measurement from a control and a control from an acceptance decision. |
| M08 Racks | Commission rack power, airflow/liquid boundary, network, asset identity, monitoring, alarms, service isolation, and failure recovery; require a testable acceptance package for each rack class or deployment wave. |
| M09 Cooling | Test sequences for economizer/chiller/tower/CDU, sensors, valves, pumps, heat reuse, leak response, power loss, fire alarm, manual fallback, and seasonal transitions; close a deficiency only after retest under the stated condition. |
| M10 Water | Build a metric/assurance package for WUE, makeup, blowdown, evaporation, leaks, chemistry, flow, discharge, and reuse; name meter boundaries, data quality, period, owner, corrective threshold, and verification action. |
| M11 Network | Change-control physical/logical topology, firewall/DMZ/OT paths, BMS/DCIM alarms, time source, remote access, link diversity, and monitoring; require a pre-change risk review, rollback, observation window, and current diagram. |
| M12 Fire | Commission detection, signaling, emergency controls, suppression interfaces, egress, firestopping, emergency power, impairment, alarm ownership, and return-to-service; identify which deficiency is life-safety blocking versus documentation-only. |
| M13 Security | Integrate physical/cyber change approval, access reviews, vendor sessions, configuration baselines, incident response, evidence preservation, and recovery; test whether the control remains effective after a maintenance exception. |
| M14 Auxiliary | Define alarm lifecycle: source, detection, quality, priority, owner, acknowledgement, escalation, command/interlock, suppression, shelving/inhibition, restoration, audit, and trend review; distinguish nuisance reduction from alarm deletion. |
| M15 Operations | Compare reactive, preventive, predictive, condition-based, and reliability-centered maintenance by failure consequence, evidence, task effectiveness, and risk; keep manufacturer, AHJ, code, and owner intervals distinct from locally chosen optimization. |

### Pass-09 factual boundaries

- FEMP’s commissioning loop is plan, investigate, implement, and hand off/integrate. A handoff package should carry systems information, people, actions, deficiencies, retests, monitoring, future tests, and budget/ownership; a document transfer alone is not handoff.
- FEMP distinguishes commissioning, ongoing commissioning, recommissioning, and retro-commissioning. These terms describe different lifecycle contexts; they are not interchangeable and do not imply that a facility is certified.
- FEMP performance-assurance material emphasizes sustained O&M, training, measurement, and verification after acceptance. Construction completion, invoice payment, or an initial functional test cannot stand in for life-cycle performance evidence.
- NIST SP 800-128 supports security-focused configuration management; it does not create an all-facility change-control board or MOP/SOP/EOP taxonomy. A facility should explicitly name the governing procedure and approving authority.
- NIST SP 800-137 treats continuous monitoring as visibility into assets, threats, vulnerabilities, and control effectiveness. A dashboard reading is not automatically a verified measurement, a reliable alarm, or a safe command.
- NIST SP 800-61 Rev. 3 is the current final incident-response publication and supersedes Rev. 2. It is cyber-response guidance and must be integrated with, not substituted for, physical, fire, electrical, and operational emergency procedures.
- Reliability-centered maintenance is a decision method, not a universal interval schedule. A question should expose function, functional failure, consequence, failure mode, detectable condition, task effectiveness, and residual risk before selecting work.
- Alarm quality is a control-system property and a people/process property. Suppressing, shelving, or retuning an alarm requires ownership, risk review, expiry, auditability, and a safe fallback; “fewer alarms” is not itself a performance improvement.
- Metrics need a boundary, definition, source, timestamp, quality flag, owner, decision threshold, and action. PUE, WUE, uptime, availability, MTBF, and response time cannot be compared without compatible definitions.

### Updated pass tracker

| Pass | Focus | Status |
|---:|---|---|
| 01 | DOE/FEMP/NREL energy, cooling, water, commissioning; CISA/NIST OT and exposure; eCFR safety; ISO/TIA/IEEE edition anchors | COMPLETE |
| 02 | Site risk, utility interconnection, resilience, climate/flood/seismic, public AHJ/code sources | COMPLETE |
| 03 | Electrical distribution, switching, protection, UPS/generator/BESS, maintenance testing | COMPLETE |
| 04 | Cooling controls, liquid cooling, heat reuse, economization, thermal/water measurement | COMPLETE |
| 05 | Cabling, outside plant, network management, BMS/DCIM segmentation, remote access | COMPLETE |
| 06 | Fire/life safety, emergency power, permits, PPE, incident command, return-to-service | COMPLETE |
| 07 | Physical security, personnel/vendor/visitor controls, asset lifecycle, media disposition | COMPLETE |
| 08 | People systems: skills matrices, shift turnover, training, fatigue, contractor governance, succession | COMPLETE |
| 09 | Commissioning, change control, MOP/SOP/EOP, alarms, metrics, reliability-centered maintenance | COMPLETE — this section |
| 10 | Cross-module adversarial review: stale editions, unsupported numbers, false equivalences, and uncovered operational decisions | COMPLETE |
| 11 | Live CDFOS/CDFOM syllabus gap audit and current catalog receipts | COMPLETE |
| 12 | Vendor due diligence, spares, and end-of-support | COMPLETE |

The objective remains open after pass 12: the research frontier is now item-level
expansion and grounding across the full 957-item bank, not another generic source list.

## Breadth pass 10 — adversarial freshness, authority, numeric, and coverage review

**Review date:** 2026-08-18. This pass challenged the nine prior source passes and their question frontiers instead of adding a generic source list. Official pages were rechecked for the primary EPI syllabus, ISO/IEC 22237-1:2021, TIA-942-C, NIST SP 800-88 Rev. 2, NIST SP 800-82 Rev. 3, NIST SP 800-61 Rev. 3, NICE current-version language, and IEEE 802.3. No PDF body was fetched or copied.

### Adversarial checks and dispositions

| Challenge | Result | Disposition |
|---|---|---|
| EPI syllabus drift or invented headings | The official CDCP page currently exposes the fifteen public module headings and detailed syllabus subheadings, including current power, cooling, liquid-cooling, STER, BESS, network, fire, security, BMS/DCIM, and alarm language | Keep research prompts tied to the published EPI headings; do not invent CDFOS/CDFOM headings, and do not treat the page’s separate exam/certificate language as a claim made by this corpus |
| ISO edition drift | ISO/IEC 22237-1:2021 is published, Edition 1, with an OBP preview; the page shows the 2018 technical-specification predecessor withdrawn and identifies scope exclusions including IT/network equipment selection and safety/EMC | Keep the 2021 catalog/preview pin; reject the withdrawn 2018 pin; do not use Part 1 as a network, safety, or EMC body substitute |
| TIA edition drift | TIA’s current standard page shows TIA-942, Revision C, published May 2024, with data-center infrastructure scope | Keep TIA-942-C; remove any implication that a TIA rating, certification program, or standard receipt certifies a learner or proves an operational result |
| NIST media-sanitization drift | NIST SP 800-88 Rev. 2 is final from September 2025 and supersedes withdrawn Rev. 1 | Use Rev. 2 only; do not retain Rev. 1 methods or certificates as current evidence |
| NIST OT and incident-response drift | NIST SP 800-82 Rev. 3 remains the final OT guide while a later revision is in draft/pre-draft status; NIST SP 800-61 Rev. 3 is the April 2025 final and supersedes Rev. 2 | Pin final editions, identify drafts as excluded, and do not let a draft or cyber-only guide silently become a facility operating rule |
| NICE workforce drift | NIST’s current-version page separates maintained components from the SP 800-181 Rev. 1 structure and exposes a current component version | Use NICE only for capability/work-role language; no credential, certification, job-title, or invented operations taxonomy is derived from it |
| NFPA/IEC/IEEE paid-body leakage | NFPA preview/catalog pages, IEC webstore pages, and IEEE records provide public headings, scope, edition, or abstract receipts; none supplies permission to paste a paid standard body | Keep receipts and source boundaries; mark body-level claims BLOCKED where no legal public text supports them |
| Numeric overclaim | Review found no new universal PUE/WUE, temperature/humidity, cycles-of-concentration, COP, heat-capture, availability, runtime, fuel-autonomy, link-reach, or test-interval claim added by the ten passes | Keep numbers attributed to a named source, installation, scenario, meter boundary, or legal table; otherwise convert the item to an evidence-selection question or BLOCKED receipt |
| Guidance-versus-law confusion | OSHA/eCFR and AHJ-adopted code are legal anchors; DOE/FEMP, NIST, CISA, NIOSH, FEMA/NIMS, NREL/NLR, and EPA are guidance, research, or planning sources unless a separate authority says otherwise | Every future item must state the authority boundary; a guidance page cannot be presented as an adopted code or universal compliance result |
| False equivalence: power systems | UPS, ATS, STS, generator, BESS, SEPSS, emergency power, microgrid, and utility interconnection remain distinct | Require named equipment, topology, mode, protection, control, test, and authority; never infer ride-through, islanding, availability, or resilience from a product label |
| False equivalence: cooling systems | Air cooling, water-side economizing, evaporative towers, dry coolers, direct liquid cooling, immersion, CDU loops, heat reuse, and thermal storage remain distinct and often hybrid | Require heat path, residual air load, water/energy boundary, control sequence, fallback, and evidence; “liquid cooled” is not a complete architecture |
| False equivalence: security and people | Physical security, cyber-physical security, worker safety, PPE, visitor/vendor control, training, qualification, and credentialing are separate domains | Require the governing authority, role, evidence owner, and task scope; do not treat a badge, course, framework, or CPG as competence or certification |
| False equivalence: dashboard and control | A dashboard, alarm, trend, command, interlock, notification, acknowledgement, and restoration record are different artifacts | Future items must name state, authority, action, timestamp, audit, and fallback; a green dashboard is not proof of safe operation |
| PDF/vendor/shadow leakage | The log contains no PDF URL, no vendor blog as a retained source, and no shadow-library source; pages that merely link PDFs are described as not fetched where relevant | Preserve the current legal source bar; do not add a PDF receipt as if it were a public body, and do not promote vendor marketing into syllabus evidence |

### Uncovered-decision challenge

The ten passes cover the fifteen EPI module headings with operational question families, but research coverage is not the same as bank completion. The following decisions remain explicit next-work targets for item-level review:

- procurement, spares, obsolescence, warranty, end-of-support, and supply-chain substitution;
- local utility/AHJ/permit evidence and the boundary between site screening and engineering approval;
- refrigerant, chemical, wastewater, noise, emissions, and environmental-justice obligations where a jurisdiction makes them applicable;
- accessibility and worker accommodation in normal, emergency, and maintenance states;
- multi-site service dependencies, data sovereignty, recovery priorities, and customer communication;
- AI/HPC load transients, workload flexibility, liquid-cooling adoption boundaries, and power/cooling co-design;
- alarm quality, sensor calibration, data retention, metric definitions, and false-positive/false-negative trade-offs;
- incident evidence preservation, privacy, labor, vendor liability, and post-event learning;
- decommissioning, media disposition, facility reuse, and closure of physical, logical, and contractual access.

These are unresolved item-authoring and source-pinning targets, not permission to invent new syllabi or taxonomies. A future item is PASS only when its public EPI heading and legal/current source receipt are present; otherwise it remains BLOCKED or a research candidate.

### Ten-pass tracker — adversarial closeout

| Pass | Focus | Status |
|---:|---|---|
| 01 | DOE/FEMP/NREL energy, cooling, water, commissioning; CISA/NIST OT and exposure; eCFR safety; ISO/TIA/IEEE edition anchors | COMPLETE |
| 02 | Site risk, utility interconnection, resilience, climate/flood/seismic, public AHJ/code sources | COMPLETE |
| 03 | Electrical distribution, switching, protection, UPS/generator/BESS, maintenance testing | COMPLETE |
| 04 | Cooling controls, liquid cooling, heat reuse, economization, thermal/water measurement | COMPLETE |
| 05 | Cabling, outside plant, network management, BMS/DCIM segmentation, remote access | COMPLETE |
| 06 | Fire/life safety, emergency power, permits, PPE, incident command, return-to-service | COMPLETE |
| 07 | Physical security, personnel/vendor/visitor controls, asset lifecycle, media disposition | COMPLETE |
| 08 | People systems: skills matrices, shift turnover, training, fatigue, contractor governance, succession | COMPLETE |
| 09 | Commissioning, change control, MOP/SOP/EOP, alarms, metrics, reliability-centered maintenance | COMPLETE |
| 10 | Cross-module adversarial review: stale editions, unsupported numbers, false equivalences, and uncovered operational decisions | COMPLETE — this section |

Ten breadth passes are now recorded, but the objective remains open: the research corpus still needs item-level expansion and source receipts across the 957-item bank. No ship/READY claim, credential claim, ms4j closure, gate-shrink, README change, or oracle-port action follows from this research pass.

## Breadth pass 11 — live EPI gap audit and current catalog receipts

**Review date:** 2026-08-18. This pass re-opened the live EPI CDFOS and CDFOM
syllabus pages and compared their exact public bullets with the existing item and
corpus maps. The official ISO/IEC catalog and OBP preview pages were used only to
pin current editions and scope. No PDF or paid standard body was opened.

### Sources retained

| Source | Current receipt | Exact heading frontier | Boundary |
|---|---|---|---|
| EPI CDFOS syllabus | https://www.epi-ap.com/services/1/3/136/Certified_Data_Centre_Facilities_Operations_Specialist_(CDFOS) | Calibration of measurement and test equipment; Sensor / alarm point testing and calibration | Syllabus heading only; it does not make ISO/IEC 17025 or IEC 62682 an EPI requirement |
| EPI CDFOM syllabus | https://www.epi-ap.com/services/1/3/8/Certified_Data_Centre_Facilities_Operations_Manager_%28CDFOM%29 | Renewable energy factor (REF); ICT utilisation management; Environmental performance measurements | Syllabus heading only; no invented environmental KPI taxonomy |
| ISO/IEC 17025:2017, Ed. 3, confirmed current 2023 | https://www.iso.org/standard/66912.html | Testing and calibration laboratory competence | Catalog/OBP preview only; not a data-centre calibration procedure |
| IEC 62682:2022, Ed. 2.0 | https://webstore.iec.ch/en/publication/65543 | Alarm-system catalog anchor | Official abstract only; no CDFOS matrix/calibration clause inferred |
| ISO/IEC 30134-3:2016, with Amd 1:2018 | https://www.iso.org/standard/66127.html | Renewable energy factor (REF) | Edition page says current but under systematic review; no REF formula or target copied |
| ISO/IEC 30134-5:2017, with Amd 1:2025 | https://www.iso.org/standard/66934.html | IT equipment utilization for servers | Catalog/abstract and amendment pin only; no KPI threshold or accelerator claim added |
| ISO 14031:2021, Ed. 3 | https://www.iso.org/standard/81453.html | Environmental performance evaluation | Generic guidance; no data-centre measurement program or performance level invented |

### Disposition

The five new receipt rows are catalog-only **BLOCKED** entries in the CDFOS/CDFOM
corpora. They do not change the 957-item bank or its 109 PASS / 848 BLOCKED ledger
count. The next item-level frontier remains the existing source-backed bank, not
an expansion by inference from a standard title.

## Breadth pass 12 — vendor due diligence, spares, and end-of-support

**Review date:** 2026-08-18. This pass targeted the unresolved procurement,
spares, obsolescence, warranty, and supply-chain-substitution decisions from the
adversarial review. The live EPI pages supplied the exact public headings; NIST
and DOE pages supplied bounded operational evidence. No PDF body was fetched.

### Sources retained

| Source | Current receipt | Exact EPI heading frontier | Boundary |
|---|---|---|---|
| EPI CDCP syllabus | https://www.epi-ap.com/services/1/3/4/Certified_Data_Centre_Professional_%28CDCP%29 | Equipment Racks — Standards; Power Infrastructure — High Performance Computing; Auxiliary Systems — BMS / DCIM | Headings only; no vendor-selection or lifecycle taxonomy is inferred |
| EPI CDFOM syllabus | https://www.epi-ap.com/services/1/3/8/Certified_Data_Centre_Facilities_Operations_Manager_%28CDFOM%29 | Governance, Risk and Compliance — Vendor management; Facilities Management — Spart part management | The page’s “Spart part management” spelling is retained exactly; it does not define a scorecard or stocking formula |
| NIST SP 1326, *Cybersecurity Supply Chain Risk Management: Due Diligence Assessment Quick-Start Guide*, final 2026-07-08 | https://csrc.nist.gov/pubs/sp/1326/final | Vendor due diligence for ICT suppliers | The public abstract names FOCI, provenance, resilience, foundational cyber practices, and supply-chain tiers; it is ICT-supplier guidance, not a facilities procurement law or universal vendor taxonomy |
| DOE FEMP, *Equipment Operations and Maintenance Summaries* | https://www.energy.gov/cmei/femp/equipment-operations-and-maintenance-summaries | Maintenance evidence and equipment lifecycle questions | FEMP describes equipment O&M summaries, safety issues, checklists, and maintenance components; it does not set a universal spare-parts quantity or interval |
| DOE FEMP, *Optimizing Solar Photovoltaic Performance for Longevity* | https://www.energy.gov/cmei/femp/optimizing-solar-photovoltaic-performance-longevity | Spare-part and replacement evidence as a bounded analogy | The page discusses spare inventory, failure history, repair/replace criteria, and reacceptance for PV/energy-storage systems; do not generalize those details to data-centre plant without a matching source |
| DOE FEMP, *Federal UESC Process Phase 4* | https://www.energy.gov/cmei/femp/federal-uesc-process-phase-4-project-implementation-and-construction | Project handoff, O&M training, spare-parts lists | UESC project guidance is a procurement/project context, not a universal data-centre maintenance contract rule |

### Question frontier

| Module / course heading | Candidate decision question | Evidence boundary |
|---|---|---|
| CDFOM — Governance, Risk and Compliance — Vendor management | A critical BMS supplier is being renewed. Which evidence should be requested before approval: provenance, resilience, foundational cyber practices, supply-chain tiers, or only a price sheet? | NIST SP 1326 supports the due-diligence dimensions for ICT suppliers; it does not create a CDFOM vendor score or approval threshold |
| CDFOM — Facilities Management — Spart part management | A replacement controller is nearing end-of-support. Which records should drive action: failure history, current supplier/part identity, repair-versus-replace criteria, reacceptance evidence, and an approved spare plan? | FEMP’s PV page is a bounded example; no universal stock level, lead time, or warranty rule is minted |
| CDCP — Auxiliary Systems — BMS / DCIM | A monitoring vendor changes ownership and remote-support terms. What must remain visible before renewal: asset/provenance, access path, support boundary, maintenance evidence, and recovery/exit assumptions? | NIST SP 1326 and FEMP O&M pages support evidence questions, not a proprietary BMS/DCIM lifecycle taxonomy |
| CDCP — Power Infrastructure — High Performance Computing | A high-performance-computing load adds a replacement power component with a different supplier. What evidence distinguishes a procurement substitution from an accepted power-design change? | The EPI heading is public; no electrical equivalence, rating, protection, or acceptance claim is made without an applicable engineering/code source |

### Disposition

Pass 12 adds research targets and source boundaries only. No new bank items were
minted, no vendor blog was retained, and the current 957-item ledger is unchanged.
The next frontier is to test whether any candidate can be grounded by a public
clause and exact EPI heading; otherwise it remains a research candidate or BLOCKED.

## Breadth pass 13 — item-level M15 safety and alarm grounding

**Review date:** 2026-08-18. This pass revisited two M15 items whose operational
claims could be bounded by a current public CDCP heading and official legal or
public-authority source. No PDF body was fetched.

### Sources retained

| Source | Current receipt | Exact CDCP heading | Boundary |
|---|---|---|---|
| EPI CDCP syllabus | https://www.epi-ap.com/services/1/3/4/Certified_Data_Centre_Professional_%28CDCP%29 | Physical Security and Safety — Components for physical safety; Auxiliary Systems — Alarm panels; Auxiliary Systems — Notification | Public syllabus headings only; no EPI credential or site-specific compliance claim |
| eCFR 29 CFR 1910.147 | https://www.ecfr.gov/current/title-29/subtitle-B/chapter-XVII/part-1910/subpart-J/section-1910.147 | Physical safety | Current eCFR display dated 2026-08-14; supports the hazardous-energy control boundary, not a site procedure |
| eCFR 29 CFR 1910.132 | https://www.ecfr.gov/current/title-29/subtitle-B/chapter-XVII/part-1910/subpart-I/section-1910.132 | Physical safety | Supports hazard-based PPE controls; no universal PPE selection is inferred |
| eCFR 29 CFR 1910.333 | https://www.ecfr.gov/current/title-29/subtitle-B/chapter-XVII/part-1910/subpart-S/section-1910.333 | Physical safety | Supports the electrical-work-practice boundary; no AHJ or task-specific procedure is replaced |
| HSE, CHIS6 Better alarm handling | https://www.hse.gov.uk/comah/sragtech/techmeasalidings.htm | Auxiliary Systems — Alarm panels; Auxiliary Systems — Notification | Public process-safety guidance supports actionable alarm and priority reasoning; no universal alarm threshold is inferred |
| U.S.–Canada Power System Outage Task Force report page | https://www.energy.gov/oe/downloads/blackout-2003-final-report-august-14-2003 | Auxiliary Systems — Alarm panels; Auxiliary Systems — Notification | Official report page supports the silent-alarm-path lesson; no outage percentage or universal alarm design is inferred |

### Item disposition

| Item | Heading and receipt | Result |
|---|---|---|
| m15-q216 | Physical Security and Safety — Components for physical safety; eCFR 1910.147, 1910.132, and 1910.333 | PASS |
| m15-q224 | Auxiliary Systems — Alarm panels; Auxiliary Systems — Notification; HSE CHIS6 and DOE blackout report | PASS |

The ledger moves from 109 PASS / 848 BLOCKED to 111 PASS / 846 BLOCKED, with
957 rows and zero bare FAIL rows. The other reviewed M15 candidates remain
BLOCKED because no exact current public heading plus qualifying official source
was established. This pass does not close a bead, certify a learner, close
ms4j, or change the named manifest drift.

## Breadth pass 14 — bounded LOTO operations slice

**Review date:** 2026-08-18. This pass re-audited the remaining CDFOS/CDFOM
receipts and found one current public legal source that supports six items when
their claims are narrowed to hazardous-energy control. The eCFR page was used
as the public code text; no PDF was fetched.

### Source retained

| Source | Current receipt | Relevant clauses | Boundary |
|---|---|---|---|
| eCFR 29 CFR 1910.147, The control of hazardous energy (lockout/tagout) | https://www.ecfr.gov/current/title-29/subtitle-B/chapter-XVII/part-1910/subpart-J/section-1910.147 | (c)(4) documented energy-control procedures and effectiveness testing; (c)(6) periodic inspection and certification; (f)(3) group-lockout responsibility and coordination; (f)(4) shift/personnel-change transfer | Current Title 29 display dated 2026-08-14; applies only in the rule's covered context and does not establish a general data-centre handover, patrol, or MOP taxonomy |

### Item disposition

| Item | Public syllabus heading | Bounded claim | Result |
|---|---|---|---|
| m15-q356 | CDFOS — Data Centre Operations — Shift handover | Orderly transfer of LOTO protection during a shift/personnel change | PASS |
| m15-q357 | CDFOS — Facilities Maintenance — Maintenance operations procedures (MOP) | Documented, used, tested, and periodically inspected hazardous-energy procedure | PASS |
| m15-q369 | CDFOS — Safety and Crisis Management — The roles and responsibilities of appointed safety staff | Authorized-employee responsibility and group-LOTO coordination | PASS |
| m15-q374 | CDFOS — Data Centre Operations — Walk around duties | Bounded LOTO periodic-inspection record and deviation correction | PASS |
| m15-q382 | CDFOM — The Data Center Organization — Organization chart | Group-LOTO responsibility, exposure status, and coordination mapping | PASS |
| m15-q383 | CDFOM — The Data Center Organization — Shift management | LOTO continuity through orderly shift/personnel transfer | PASS |

The ledger is now 117 PASS / 840 BLOCKED across 957 rows, with zero bare FAIL
rows. m15-q348 (succession/knowledge hand-off), m15-q351 (independent
commissioning provider), and m15-q363 (career development/job rotation) remain
BLOCKED because the reviewed public receipts do not expose the needed clauses;
no invented handover or career taxonomy was added. This pass does not certify a
learner, close ms4j, or alter the manifest drift.

## Breadth pass 15 — paid-only CDFOM receipt refresh

**Review date:** 2026-08-18. The final three blocked CDFOM rows were rechecked
against current official ISO catalog pages. The catalog abstracts were used to
pin the edition and scope only; no paid standard body or PDF was fetched.

| Item | Public CDFOM heading | Official catalog receipt | Disposition |
|---|---|---|---|
| m15-q348 | The Data Center Organization — Succession planning; Career development | ISO 30401:2018 + Amd 1:2022, Knowledge management systems — Requirements — https://www.iso.org/standard/68683.html and https://www.iso.org/standard/79489.html | **BLOCKED** — public abstract does not expose a succession/career clause; body is paid |
| m15-q351 | Facilities Management — Maintenance policies and procedures | ISO 41001:2018 + Amd 1:2024, Facility management — Management systems — Requirements with guidance for use — https://www.iso.org/standard/68021.html and https://www.iso.org/standard/88425.html | **BLOCKED** — public abstract does not expose an independent commissioning-provider clause; body is paid |
| m15-q363 | The Data Center Organization — Career development; Job rotation | ISO 10015:2019, Quality management — Guidelines for competence management and people development — https://www.iso.org/standard/69459.html | **BLOCKED** — catalog page says the edition was confirmed current in 2025, but no public job-rotation clause is exposed and the body is paid |

The three rows now carry official edition-pinned catalog receipts rather than
weaker placeholder URLs. Counts remain 117 PASS / 840 BLOCKED across 957 rows;
no heading, handover program, career taxonomy, or credential claim was invented.

## Breadth pass 16 — CDCP fire-document roles and electrical boundary

**Review date:** 2026-08-18. This pass revisited two Li-ion fire rows and two
electrical-safety rows whose stems could be made source-bounded without copying
standard bodies. Public EPI headings and official preview/catalog or public-code
receipts were retained; no PDF was fetched.

### Sources retained

| Source | Current receipt | Supported distinction |
|---|---|---|
| EPI CDCP syllabus | https://www.epi-ap.com/services/1/3/4/Certified_Data_Centre_Professional_%28CDCP%29 | 1.12 Fire Protection; Physical Security and Safety — Components for physical safety |
| NFPA 855, 2026 edition | https://link.nfpa.org/all-publications/855/2026 | Public NFPA preview pins the current stationary-ESS installation standard and its installation/operation chapters; adoption remains an AHJ question |
| UL 9540A, Edition 6 | https://www.shopulstandards.com/ProductDetail.aspx?UniqueKey=49792 | Active UL catalog identifies the thermal-runaway/fire-propagation test method and its March 13, 2026 edition |
| UL 9540, Edition 3 | https://www.shopulstandards.com/ProductDetail.aspx?productId=UL9540_3_S_20230628 | Active UL catalog identifies the ESS product/system safety standard and its March 7, 2025 revision |
| eCFR 29 CFR 1910.333 | https://www.ecfr.gov/current/title-29/subtitle-B/chapter-XVII/part-1910/subpart-S/section-1910.333 | Public electrical-work-practice receipt; it does not replace qualification, PPE selection, or a site procedure |

### Item disposition

| Item | Bounded result | Result |
|---|---|---|
| m12-q225 | Distinguishes NFPA 855 installation, UL 9540A test method/data, and UL 9540 ESS product/system safety; no listing, fire percentage, agent mass, or AHJ result inferred | PASS |
| m12-q226 | Rejects a packet that treats a UL 9540A report as a listing or NFPA 855 as automatically adopted law; requires edition/AHJ verification | PASS |
| m12-q227 | Retains the UL 9540 listing versus UL 9540A test-method distinction with official UL/NFPA receipts only | PASS |
| m13-q217 | Distinguishes electrical-utilization work practice from machine LOTO and separates badge access from electrical authorization | PASS |
| m13-q218 | Keeps the published electrical boundary/deenergization decision bounded without inventing an incident-energy value or reproducing NFPA 70E | PASS |

The ledger is now 121 PASS / 836 BLOCKED across 957 rows, with zero bare FAIL
rows. Vendor blogs and third-party bulletins were removed from q227's retained
receipt set. No credential, AHJ approval, or universal fire-response claim is
made by this pass.

## Breadth pass 17 — CDFOM receipt currency check

**Review date:** 2026-08-18. The CDFOS/CDFOM item-file audit found the active
52-item set already carries a public syllabus heading and an external official
receipt in its file comments. The three unresolved CDFOM topics were checked
again against the live ISO catalog pages; q348's receipt was updated to the
latest published amendment. No paid standard body or PDF was fetched.

| Item | Public CDFOM heading | Current official catalog receipt | Disposition |
|---|---|---|---|
| m15-q348 | The Data Center Organization — Succession planning; Career development | ISO 30401:2018 + Amd 1:2022 + Amd 2:2024 — https://www.iso.org/standard/88416.html | **BLOCKED** — the current catalog confirms the amendment, but the paid body and public abstract do not expose a qualifying succession/career clause |
| m15-q351 | Facilities Management — Maintenance policies and procedures | ISO 41001:2018 + Amd 1:2024 — https://www.iso.org/standard/88425.html | **BLOCKED** — the paid body and public abstract do not expose a qualifying independent-commissioning-provider clause |
| m15-q363 | The Data Center Organization — Career development; Job rotation | ISO 10015:2019 — https://www.iso.org/standard/69459.html | **BLOCKED** — the catalog says the edition remains current after its 2025 confirmation, but no public job-rotation clause is exposed and the body is paid |

Counts remain 121 PASS / 836 BLOCKED across 957 rows, with zero bare FAIL.
This pass does not invent a heading, handover program, career taxonomy, or
credential claim, and does not close ms4j.

## Breadth pass 18 — CDCP fire protection: public code boundaries

**Review date:** 2026-08-18. Four Module 12 rows were reworked to claims
directly exposed by current public eCFR text. The receipts are public code text,
not standard-body copies; no PDF was fetched.

| Item | Public CDCP heading | Official public-code receipt | Bounded result |
|---|---|---|---|
| m12-q209 | 1.12 Fire Protection | 29 CFR 1910.252(a) — https://www.ecfr.gov/current/title-29/subtitle-B/chapter-XVII/part-1910/subpart-Q/section-1910.252 | **PASS** — hot-work fire hazards, authorization, extinguishing readiness, and fire-watch controls |
| m12-q218 | 1.12 Fire Protection | 29 CFR 1910.155(c)(10) — https://www.ecfr.gov/current/title-29/subtitle-B/chapter-XVII/part-1910/subpart-L/section-1910.155 | **PASS** — Class C is defined as energized electrical equipment requiring electrically nonconductive media for employee safety |
| m12-q221 | 1.12 Fire Protection | 29 CFR 1910.37 — https://www.ecfr.gov/current/title-29/subtitle-B/chapter-XVII/part-1910/subpart-E/section-1910.37 | **PASS** — marked, illuminated, unobstructed exits and operable emergency safeguards |
| m12-q224 | 1.12 Fire Protection | 29 CFR 1910.164 and 1910.160 — https://www.ecfr.gov/current/title-29/subtitle-B/chapter-XVII/part-1910/subpart-L/subject-group-ECFR76c69af98ee6ed7/section-1910.164 and https://www.ecfr.gov/current/title-29/subtitle-B/chapter-XVII/part-1910/subpart-L/subject-group-ECFR7a02737a205fd22/section-1910.160 | **PASS** — detection and fixed-system inspection, operability, restoration, and impairment handling |

ASD, agent selection, pre-action, cable-firestop, and other rows remain
BLOCKED where the reviewed public receipt did not expose the exact claim. The
ledger is now 126 PASS / 831 BLOCKED across 957 rows, with zero bare FAIL.
This pass does not certify a learner or close ms4j.

## Breadth pass 25 — CDCP fire-protection NFPA edition receipts

**Review date:** 2026-08-18. Four Module 12 rows were narrowed to the current
public EPI CDCP **Fire Protection** heading and official NFPA LiNK catalog or
preview URLs. No NFPA PDF or standard body was fetched.

| Item | Public CDCP heading | Official catalog/preview receipt | Bounded result |
|---|---|---|---|
| m12-q201 | Fire Protection | NFPA 72, 2025 — https://link.nfpa.org/all-publications/72/2025 | **PASS** — alarm/signaling scope supports early detection; it does not make ASD suppression or define a site runbook |
| m12-q202 | Fire Protection | NFPA 13, 2025 — https://link.nfpa.org/all-publications/13/2025 | **PASS** — sprinkler-installation scope supports engineered water protection; it does not establish a universal IT-room design or zero equipment impact |
| m12-q203 | Fire Protection | NFPA 2001, 2025 — https://link.nfpa.org/all-publications/2001/2025 | **PASS** — clean-agent extinguishing-system scope supports the water-damage distinction; detection, enclosure, maintenance, and life safety remain required boundaries |
| m12-q204 | Fire Protection | NFPA 10, 2026 — https://link.nfpa.org/all-publications/10/2026 | **PASS** — portable-extinguisher scope supports hazard-matched selection; trained use and site policy remain required |

The ledger is now 148 PASS / 809 BLOCKED across 957 rows, with zero bare FAIL.
This pass does not certify a learner, close any ms4j bead, or alter gate-shrink
or oracle scope.

## Breadth pass 26 — CDCP fire-protection NFPA follow-up receipts

**Review date:** 2026-08-18. Four additional active Module 12 rows were
narrowed to the same public EPI CDCP **Fire Protection** heading and current
NFPA LiNK catalog or preview URLs. The retired near-duplicate `m12-q219` was
not promoted; the active `m12-q220` was used for the portable-extinguisher
receipt instead. No NFPA PDF or standard body was fetched.

| Item | Public CDCP heading | Official catalog/preview receipt | Bounded result |
|---|---|---|---|
| m12-q211 | Fire Protection | NFPA 72, 2025 — https://link.nfpa.org/all-publications/72/2025 | **PASS** — early warning remains an alarm/signaling function, not suppression or a site-specific runbook |
| m12-q213 | Fire Protection | NFPA 13, 2025 — https://link.nfpa.org/all-publications/13/2025 | **PASS** — pre-action is retained as a design distinction; no universal IT-space arrangement or zero-water claim |
| m12-q215 | Fire Protection | NFPA 2001, 2025 — https://link.nfpa.org/all-publications/2001/2025 | **PASS** — clean-agent lower-residue distinction retained without removing detection, enclosure, maintenance, or life safety |
| m12-q220 | Fire Protection | NFPA 10, 2026 — https://link.nfpa.org/all-publications/10/2026 | **PASS** — hazard-matched portable-agent choice remains tied to site policy and trained use |

The ledger is now 152 PASS / 805 BLOCKED across 957 rows, with zero bare FAIL.
This pass does not certify a learner, close any ms4j bead, or alter gate-shrink
or oracle scope.

## Breadth pass 27 — CDCP fire-protection code-boundary receipts

**Review date:** 2026-08-18. Three additional active Module 12 rows were
narrowed to the public EPI CDCP **Fire Protection** heading and official NFPA
LiNK catalog or preview URLs. The AHJ/adoption item remains BLOCKED because a
generic catalog cannot establish a local jurisdiction's adopted code path. No
NFPA PDF or standard body was fetched.

| Item | Public CDCP heading | Official catalog/preview receipt | Bounded result |
|---|---|---|---|
| m12-q206 | Fire Protection | NFPA 1, 2024 — https://link.nfpa.org/all-publications/1/2024 | **PASS** — compartmentation and penetration integrity remain code-governed concerns; no project-specific firestop detail is inferred |
| m12-q207 | Fire Protection | NFPA 101, 2024 — https://link.nfpa.org/all-publications/101/2024 | **PASS** — life-safety/egress scope supports clear signage; no training replacement or local adoption claim is made |
| m12-q210 | Fire Protection | NFPA 921, 2024 — https://link.nfpa.org/all-publications/921/2024 | **PASS** — electricity-and-fire investigation scope supports qualitative ignition hazards; no universal cause ranking is claimed |

The ledger is now 155 PASS / 802 BLOCKED across 957 rows, with zero bare FAIL.
This pass does not certify a learner, close any ms4j bead, or alter gate-shrink
or oracle scope.

## Breadth pass 28 — CDCP fire-protection BLOCKED NFPA receipts

**Review date:** 2026-08-18. Four Module 12 rows now carry official NFPA
catalog/preview receipts while remaining BLOCKED because the public preview
does not expose the exact operational clause. The retired `m12-q217` remains
retired; its receipt is bookkeeping, not a promotion.

| Item | Public CDCP heading | Official catalog/preview receipt | Bounded result |
|---|---|---|---|
| m12-q212 | Fire Protection | NFPA 72, 2025 — https://link.nfpa.org/all-publications/72/2025 | **BLOCKED** — cross-zone/release clause not exposed in the public preview |
| m12-q214 | Fire Protection | NFPA 1, 2024 — https://link.nfpa.org/all-publications/1/2024 | **BLOCKED** — room-by-room strategy and adopted-AHJ decision not exposed |
| m12-q216 | Fire Protection | NFPA 2001, 2025 — https://link.nfpa.org/all-publications/2001/2025 | **BLOCKED** — HVAC-interlock/hold-time clause not exposed |
| m12-q217 | Fire Protection | NFPA 2001, 2025 — https://link.nfpa.org/all-publications/2001/2025 | **BLOCKED** — retired duplicate; abort-switch clause not exposed |

The ledger remains 155 PASS / 802 BLOCKED across 957 rows, with zero bare FAIL.
This pass does not certify a learner, close any ms4j bead, or alter gate-shrink
or oracle scope.

## Breadth pass 30 — CDCP early-fire-detection BLOCKED receipts

**Review date:** 2026-08-18. Four Module 12 bank-expansion items now carry the
official NFPA 72 (2025) preview receipt while remaining BLOCKED: the public
preview does not expose their exact ASD, heat-detector, cross-zoning, or staged
threshold propositions. No NFPA PDF or standard body was fetched.

| Item | Public CDCP heading | Official catalog/preview receipt | Bounded result |
|---|---|---|---|
| bank-m12-q041 | Fire Protection | NFPA 72, 2025 — https://link.nfpa.org/all-publications/72/2025 | **BLOCKED** — exact ASD/high-airflow claim not exposed |
| bank-m12-q042 | Fire Protection | NFPA 72, 2025 — https://link.nfpa.org/all-publications/72/2025 | **BLOCKED** — exact heat-detector role claim not exposed |
| bank-m12-q043 | Fire Protection | NFPA 72, 2025 — https://link.nfpa.org/all-publications/72/2025 | **BLOCKED** — exact cross-zoning release proposition not exposed |
| bank-m12-q044 | Fire Protection | NFPA 72, 2025 — https://link.nfpa.org/all-publications/72/2025 | **BLOCKED** — exact staged-threshold claim not exposed |

The ledger remains 155 PASS / 802 BLOCKED across 957 rows, with zero bare FAIL.
This pass does not certify a learner, close any ms4j bead, or alter gate-shrink
or oracle scope.

## Breadth pass 29 — CDCP fire-protection additional BLOCKED receipts

**Review date:** 2026-08-18. Four Module 12 rows now carry official NFPA
catalog/preview receipts while remaining BLOCKED because their public pages do
not expose the exact historical, jurisdictional, penetration, or ASD-runbook
claim. No NFPA PDF or standard body was fetched.

| Item | Public CDCP heading | Official catalog/preview receipt | Bounded result |
|---|---|---|---|
| m12-q200 | Fire Protection | NFPA 921, 2024 — https://link.nfpa.org/all-publications/921/2024 | **BLOCKED** — historical data-centre cause mix not exposed |
| m12-q205 | Fire Protection | NFPA 1, 2024 — https://link.nfpa.org/all-publications/1/2024 | **BLOCKED** — local adopted code path/AHJ decision not exposed |
| m12-q223 | Fire Protection | NFPA 1, 2024 — https://link.nfpa.org/all-publications/1/2024 | **BLOCKED** — exact penetration/firestop clause not exposed |
| m12-q300 | Fire Protection | NFPA 72, 2025 — https://link.nfpa.org/all-publications/72/2025 | **BLOCKED** — exact ASD staged-threshold runbook not exposed |

The ledger remains 155 PASS / 802 BLOCKED across 957 rows, with zero bare FAIL.
This pass does not certify a learner, close any ms4j bead, or alter gate-shrink
or oracle scope.

## Breadth pass 31 — CDCP NFPA standards-awareness and blocked receipts

**Review date:** 2026-08-18. NFPA 72 and NFPA 2001 standards-awareness items
were promoted using their exact public titles. Adjacent jurisdictional,
impairment, signage, pull-station, fire-door, and abort-switch items received
official catalog/preview receipts but remain BLOCKED because their exact public
clauses were not exposed. No NFPA PDF or standard body was fetched.

| Item | Public CDCP heading | Official catalog/preview receipt | Bounded result |
|---|---|---|---|
| bank-m12-q068 | Fire Protection | NFPA 1, 2024 — https://link.nfpa.org/all-publications/1/2024 | **BLOCKED** — local adopted code/AHJ authority not exposed |
| bank-m12-q069 | Fire Protection | NFPA 72, 2025 — https://link.nfpa.org/all-publications/72/2025 | **PASS** — official title identifies the National Fire Alarm and Signaling Code |
| bank-m12-q070 | Fire Protection | NFPA 2001, 2025 — https://link.nfpa.org/all-publications/2001/2025 | **PASS** — official title identifies the clean-agent fire-extinguishing-systems standard |
| bank-m12-q071 | Fire Protection | NFPA 72, 2025 — https://link.nfpa.org/all-publications/72/2025 | **BLOCKED** — exact impairment-control procedure not exposed |
| bank-m12-q072 | Fire Protection | NFPA 2001, 2025 — https://link.nfpa.org/all-publications/2001/2025 | **BLOCKED** — exact warning-signage wording/placement not exposed |
| bank-m12-q073 | Fire Protection | NFPA 72, 2025 — https://link.nfpa.org/all-publications/72/2025 | **BLOCKED** — exact manual-pull-station placement not exposed |
| bank-m12-q074 | Fire Protection | NFPA 101, 2024 — https://link.nfpa.org/all-publications/101/2024 | **BLOCKED** — exact fire-door propping control not exposed |
| bank-m12-q075 | Fire Protection | NFPA 2001, 2025 — https://link.nfpa.org/all-publications/2001/2025 | **BLOCKED** — exact abort-switch control clause not exposed |

The ledger is now 157 PASS / 800 BLOCKED across 957 rows, with zero bare FAIL.
This pass does not certify a learner, close any ms4j bead, or alter gate-shrink
or oracle scope.

## Breadth pass 32 — CDCP physical-security NFPA frontier

**Review date:** 2026-08-18. Two M13 physical-security items were promoted
against the exact public EPI heading and the current NFPA 730 (2026) preview,
whose exposed structure includes security planning, administrative controls,
security perimeters, and security systems. Four narrower claims received
official NFPA 730/731 receipts but remain BLOCKED because their exact clauses
are not exposed. No NFPA PDF or standard body was fetched.

| Item | Public CDCP heading | Official catalog/preview receipt | Bounded result |
|---|---|---|---|
| bank-m13-q076 | Physical Security and Safety — Components for physical security | NFPA 730, 2026 — https://link.nfpa.org/all-publications/730/2026 | **PASS** — layered security is bounded to the exposed planning, administrative, perimeter, and system categories |
| bank-m13-q077 | Physical Security and Safety — Components for physical security | NFPA 731, 2026 — https://link.nfpa.org/all-publications/731/2026 | **BLOCKED** — exact mantrap/interlock proposition not exposed |
| bank-m13-q088 | Physical Security and Safety — Components for physical security | NFPA 731, 2026 — https://link.nfpa.org/all-publications/731/2026 | **BLOCKED** — exact door-alarm/video-integration proposition not exposed |
| bank-m13-q098 | Physical Security and Safety — Components for physical security | NFPA 730, 2026 — https://link.nfpa.org/all-publications/730/2026 | **BLOCKED** — exact deterrence/delay/detection/response/recovery framework not exposed |
| m13-q200 | Physical Security and Safety — Components for physical security | NFPA 730, 2026 — https://link.nfpa.org/all-publications/730/2026 | **PASS** — layered physical security is bounded to the exposed NFPA 730 categories |
| m13-q203 | Physical Security and Safety — Components for physical security | NFPA 731, 2026 — https://link.nfpa.org/all-publications/731/2026 | **BLOCKED** — exact unique-identity/access-log accounting clause not exposed |

The ledger is now 159 PASS / 798 BLOCKED across 957 rows, with zero bare FAIL.
This pass does not certify a learner, close any ms4j bead, or alter gate-shrink
or oracle scope.

## Breadth pass 33 — CDCP physical-access and safety BLOCKED receipts

**Review date:** 2026-08-18. Six additional M13 items now carry official
NFPA 730/731, ISO/IEC 27001, or NFPA 101 catalog/preview receipts. They remain
BLOCKED because the public pages do not expose the exact zoning, identity,
multi-factor, anti-passback, or access-hardware propositions. No PDF or paid
standard body was fetched.

| Item | Public CDCP heading | Official catalog/preview receipt | Bounded result |
|---|---|---|---|
| bank-m13-q080 | Physical Security and Safety — Components for physical security | NFPA 730, 2026 — https://link.nfpa.org/all-publications/730/2026 | **BLOCKED** — exact progressive-zoning/least-privilege proposition not exposed |
| bank-m13-q081 | Physical Security and Safety — Components for physical security | NFPA 730, 2026 — https://link.nfpa.org/all-publications/730/2026 | **BLOCKED** — exact grey-space sensitivity/zoning proposition not exposed |
| bank-m13-q082 | Physical Security and Safety — Components for physical security | ISO/IEC 27001:2022 + Amd 1:2024 — https://www.iso.org/standard/27001?browse=tc | **BLOCKED** — exact authentication/authorization/accounting distinction not exposed |
| bank-m13-q083 | Physical Security and Safety — Components for physical security | NFPA 731, 2026 — https://link.nfpa.org/all-publications/731/2026 | **BLOCKED** — exact multi-factor physical-access proposition not exposed |
| bank-m13-q084 | Physical Security and Safety — Components for physical security | NFPA 731, 2026 — https://link.nfpa.org/all-publications/731/2026 | **BLOCKED** — exact anti-passback proposition not exposed |
| bank-m13-q085 | Physical Security and Safety — Components for physical safety | NFPA 101, 2024 — https://link.nfpa.org/all-publications/101/2024 | **BLOCKED** — exact access-hardware release/free-egress proposition not exposed |

The ledger remains 159 PASS / 798 BLOCKED across 957 rows, with zero bare FAIL.
This pass does not certify a learner, close any ms4j bead, or alter gate-shrink
or oracle scope.

## Breadth pass 47 — CDCP industrial-adjacency receipt

**Review date:** 2026-08-18. M03 item `m03-q208` now carries the official
ISO/IEC 22237-2:2024 catalog receipt for building construction, environmental
risks, and site configuration. It remains BLOCKED because the public catalog
does not expose the exact industrial-neighbor hazard proposition. No PDF was
fetched.

| Item | Public CDCP heading | Official catalog/preview receipt | Bounded result |
|---|---|---|---|
| m03-q208 | Site location selection criteria | ISO/IEC 22237-2:2024 — https://webstore.iec.ch/en/publication/92577 | **BLOCKED** — exact industrial-neighbor hazard proposition not exposed |

The ledger remains 159 PASS / 798 BLOCKED across 957 rows, with zero bare FAIL.
This pass does not certify a learner, close any ms4j bead, or alter gate-shrink
or oracle scope.

## Breadth pass 46 — CDCP grid, growth, and emerging-site receipts

**Review date:** 2026-08-18. Thirteen remaining M03 rows now carry official
ISO/IEC 22237-1:2021, ISO/IEC 22237-2:2024, or TIA-942-C catalog/preview
receipts. They remain BLOCKED because the permitted public pages do not expose
the exact grid-queue, behind-the-meter, AI-factory, dispatch, or handoff
propositions. No PDF was fetched.

| Item | Public CDCP heading | Official catalog/preview receipt | Bounded result |
|---|---|---|---|
| m03-q209 | Supporting facilities and function | ISO/IEC 22237-2:2024 — https://webstore.iec.ch/en/publication/92577 | **BLOCKED** — exact generator-yard/fuel-logistics proposition not exposed |
| m03-q210 | Facility criteria | TIA-942-C (May 2024) — https://tiaonline.org/standard/tia-942/ | **BLOCKED** — exact climate-extremes design proposition not exposed |
| m03-q211 | Site location selection criteria | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact staffing/emergency-response access proposition not exposed |
| m03-q212 | Facility criteria | ISO/IEC 22237-2:2024 — https://webstore.iec.ch/en/publication/92577 | **BLOCKED** — exact future-growth limiting criterion not exposed |
| m03-q213 | Site location selection criteria | TIA-942-C (May 2024) — https://tiaonline.org/standard/tia-942/ | **BLOCKED** — exact water-rights/discharge proposition not exposed |
| m03-q214 | Site location selection criteria | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact interconnection queue/energization proposition not exposed |
| m03-q215 | Site location selection criteria | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact queue-position/study/financial-security proposition not exposed |
| m03-q216 | Site location selection criteria | TIA-942-C (May 2024) — https://tiaonline.org/standard/tia-942/ | **BLOCKED** — exact behind-the-meter site-type proposition not exposed |
| m03-q217 | Site location selection criteria | ISO/IEC 22237-2:2024 — https://webstore.iec.ch/en/publication/92577 | **BLOCKED** — exact behind-the-meter permitting proposition not exposed |
| m03-q218 | Site location selection criteria | TIA-942-C (May 2024) — https://tiaonline.org/standard/tia-942/ | **BLOCKED** — exact AI-factory/neocloud site-criteria taxonomy not exposed |
| m03-q219 | Site location selection criteria | TIA-942-C (May 2024) — https://tiaonline.org/standard/tia-942/ | **BLOCKED** — exact grid/fibre/AI-factory due-diligence proposition not exposed |
| m03-q220 | Site location selection criteria | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact dispatch-restriction proposition not exposed |
| m03-q221 | Site location selection criteria | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact M03-to-M15 dispatch/reduction handoff proposition not exposed |

The ledger remains 159 PASS / 798 BLOCKED across 957 rows, with zero bare FAIL.
This pass does not certify a learner, close any ms4j bead, or alter gate-shrink
or oracle scope.

## Breadth pass 45 — CDCP site-selection continuation

**Review date:** 2026-08-18. Fifteen additional M03 rows now carry official
ISO/IEC 22237-1:2021, ISO/IEC 22237-2:2024, ISO/IEC TS 22237-30:2022, or
TIA-942-C catalog/preview receipts. They remain BLOCKED because the public
pages do not expose the exact transport, adjacency, expansion, environmental,
or telecom-diversity propositions. No PDF was fetched.

| Item | Public CDCP heading | Official catalog/preview receipt | Bounded result |
|---|---|---|---|
| m03-q108 | Site location selection criteria | ISO/IEC 22237-2:2024 — https://webstore.iec.ch/en/publication/92577 | **BLOCKED** — exact transportation-access proposition not exposed |
| m03-q109 | Site location selection criteria | TIA-942-C (May 2024) — https://tiaonline.org/standard/tia-942/ | **BLOCKED** — exact shared-campus site-test proposition not exposed |
| m03-q110 | Site location selection criteria | TIA-942-C (May 2024) — https://tiaonline.org/standard/tia-942/ | **BLOCKED** — exact RF-adjacency proposition not exposed |
| m03-q111 | Site location selection criteria | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact short-listing combination not exposed |
| m03-q112 | Supporting facilities and function | ISO/IEC 22237-2:2024 — https://webstore.iec.ch/en/publication/92577 | **BLOCKED** — exact supporting-facility gap proposition not exposed |
| m03-q113 | Site location selection criteria | ISO/IEC 22237-2:2024 — https://webstore.iec.ch/en/publication/92577 | **BLOCKED** — exact future-expansion proposition not exposed |
| m03-q114 | Facility criteria | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact architecture comparison not exposed |
| m03-q200 | Site location selection criteria | ISO/IEC 22237-2:2024 — https://webstore.iec.ch/en/publication/92577 | **BLOCKED** — exact flood-data proposition not exposed |
| m03-q201 | Site location selection criteria | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact dual-utility-feed proposition not exposed |
| m03-q202 | Facility criteria | ISO/IEC 22237-2:2024 — https://webstore.iec.ch/en/publication/92577 | **BLOCKED** — exact heavy-plant installation/maintenance proposition not exposed |
| m03-q203 | Supporting facilities and function | ISO/IEC 22237-2:2024 — https://webstore.iec.ch/en/publication/92577 | **BLOCKED** — exact fuel/water/telecom/perimeter evaluation not exposed |
| m03-q204 | Site location selection criteria | ISO/IEC TS 22237-30:2022 — https://www.iso.org/standard/80622.html | **BLOCKED** — exact geotechnical-site proposition not exposed |
| m03-q205 | Site location selection criteria | ISO/IEC 22237-2:2024 — https://webstore.iec.ch/en/publication/92577 | **BLOCKED** — exact flight-path/rail/highway adjacency proposition not exposed |
| m03-q206 | Facility criteria | ISO/IEC 22237-2:2024 — https://webstore.iec.ch/en/publication/92577 | **BLOCKED** — exact greenfield/retrofit comparison not exposed |
| m03-q207 | Supporting facilities and function | TIA-942-C (May 2024) — https://tiaonline.org/standard/tia-942/ | **BLOCKED** — exact diverse-telecom-POE proposition not exposed |

The ledger remains 159 PASS / 798 BLOCKED across 957 rows, with zero bare FAIL.
This pass does not certify a learner, close any ms4j bead, or alter gate-shrink
or oracle scope.

## Breadth pass 44 — CDCP site-selection and facility-criteria receipts

**Review date:** 2026-08-18. Seventeen M03 site-selection and facility-criteria
rows now carry official ISO/IEC 22237-1:2021, ISO/IEC 22237-2:2024,
ISO/IEC TS 22237-30:2022, or TIA-942-C catalog/preview receipts. They remain
BLOCKED because the public pages do not expose the exact site, utility,
environmental, loading, or operations propositions. No PDF was fetched.

| Item | Public CDCP heading | Official catalog/preview receipt | Bounded result |
|---|---|---|---|
| mock40-q07 | Site location selection criteria | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact dual-utility feasibility proposition not exposed |
| mock40-q08 | Supporting facilities and function | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact white-space/supporting-facilities proposition not exposed |
| m03-q093 | Site location selection criteria | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact multi-path utility-diversity proposition not exposed |
| m03-q094 | Site location selection criteria | ISO/IEC 22237-2:2024 — https://webstore.iec.ch/en/publication/92577 | **BLOCKED** — exact flood/storm-surge proposition not exposed |
| m03-q095 | Site location selection criteria | TIA-942-C (May 2024) — https://tiaonline.org/standard/tia-942/ | **BLOCKED** — exact fibre/meet-me diversity proposition not exposed |
| m03-q096 | Facility criteria | ISO/IEC 22237-2:2024 — https://webstore.iec.ch/en/publication/92577 | **BLOCKED** — exact clear-height operational proposition not exposed |
| m03-q097 | Supporting facilities and function | ISO/IEC 22237-2:2024 — https://webstore.iec.ch/en/publication/92577 | **BLOCKED** — exact supporting-facilities list not exposed |
| m03-q098 | Site location selection criteria | ISO/IEC TS 22237-30:2022 — https://www.iso.org/standard/80622.html | **BLOCKED** — exact seismic/structural site proposition not exposed |
| m03-q099 | Site location selection criteria | ISO/IEC 22237-2:2024 — https://webstore.iec.ch/en/publication/92577 | **BLOCKED** — exact adjacency red-flag proposition not exposed |
| m03-q100 | Facility criteria | ISO/IEC 22237-2:2024 — https://webstore.iec.ch/en/publication/92577 | **BLOCKED** — exact slab-loading proposition not exposed |
| m03-q101 | Supporting facilities and function | ISO/IEC 22237-2:2024 — https://webstore.iec.ch/en/publication/92577 | **BLOCKED** — exact generator-yard proposition not exposed |
| m03-q102 | Site location selection criteria | TIA-942-C (May 2024) — https://tiaonline.org/standard/tia-942/ | **BLOCKED** — exact water-availability proposition not exposed |
| m03-q103 | Site location selection criteria | ISO/IEC 22237-2:2024 — https://webstore.iec.ch/en/publication/92577 | **BLOCKED** — exact perimeter-planning proposition not exposed |
| m03-q104 | Supporting facilities and function | ISO/IEC 22237-2:2024 — https://webstore.iec.ch/en/publication/92577 | **BLOCKED** — exact staging/loading-dock proposition not exposed |
| m03-q105 | Site location selection criteria | TIA-942-C (May 2024) — https://tiaonline.org/standard/tia-942/ | **BLOCKED** — exact climate/free-cooling proposition not exposed |
| m03-q106 | Facility criteria | TIA-942-C (May 2024) — https://tiaonline.org/standard/tia-942/ | **BLOCKED** — exact roof/yard heat-rejection constraint not exposed |
| m03-q107 | Supporting facilities and function | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact human-operations facility criterion not exposed |

The ledger remains 159 PASS / 798 BLOCKED across 957 rows, with zero bare FAIL.
This pass does not certify a learner, close any ms4j bead, or alter gate-shrink
or oracle scope.

## Breadth pass 43 — CDCP KPI terminology receipt

**Review date:** 2026-08-18. M02 item `m02-q217` now has the official
ISO/IEC 30134-6:2021 catalog receipt. It remains BLOCKED because the public
catalog pins Energy Reuse Factor (ERF) but does not expose the item’s exact
ERF-versus-ERE distinction. No PDF was fetched.

| Item | Public CDCP heading | Official catalog/preview receipt | Bounded result |
|---|---|---|---|
| m02-q217 | Standards and guidelines landscape | ISO/IEC 30134-6:2021 — https://www.iso.org/standard/71717.html | **BLOCKED** — exact ERF-versus-ERE distinction not exposed |

The ledger remains 159 PASS / 798 BLOCKED across 957 rows, with zero bare FAIL.
This pass does not certify a learner, close any ms4j bead, or alter gate-shrink
or oracle scope.

## Breadth pass 42 — CDCP standards-landscape continuation

**Review date:** 2026-08-18. Thirteen additional M02 rows now carry official
NFPA 1, ISO/IEC 22237-1:2021, or IEC ISO/IEC 22237-2:2024 catalog receipts.
They remain BLOCKED because the public pages do not expose the exact owner
specification, AHJ, EN-series, local-requirements, or component-marking
propositions. No PDF was fetched.

| Item | Public CDCP heading | Official catalog/preview receipt | Bounded result |
|---|---|---|---|
| m02-q081 | Standards and guidelines landscape | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact guidelines-versus-code distinction not exposed |
| m02-q082 | Standards for sub-components | ISO/IEC 22237-2:2024 — https://webstore.iec.ch/en/publication/92577 | **BLOCKED** — exact fire-detection subcomponent proposition not exposed |
| m02-q084 | AHJ/code vs voluntary standard | NFPA 1, 2024 — https://link.nfpa.org/all-publications/1/2024 | **BLOCKED** — exact “AHJ wins” shorthand proposition not exposed |
| m02-q086 | Standards and guidelines landscape | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact owner-specification position not exposed |
| m02-q089 | International vs national standards | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact international-standards-development aims not exposed |
| m02-q090 | Standards for sub-components | ISO/IEC 22237-2:2024 — https://webstore.iec.ch/en/publication/92577 | **BLOCKED** — exact cabling/firestop subcomponent proposition not exposed |
| m02-q091 | AHJ/code vs voluntary standard | NFPA 1, 2024 — https://link.nfpa.org/all-publications/1/2024 | **BLOCKED** — exact local-code-silent professional approach not exposed |
| m02-q200 | AHJ/code vs voluntary standard | NFPA 1, 2024 — https://link.nfpa.org/all-publications/1/2024 | **BLOCKED** — exact adopted-code conflict principle not exposed |
| m02-q203 | EN 50600 series awareness | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact EN 50600 series description not exposed |
| m02-q204 | International vs national standards | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact international/local requirements tension not exposed |
| m02-q205 | Standards for sub-components | ISO/IEC 22237-2:2024 — https://webstore.iec.ch/en/publication/92577 | **BLOCKED** — exact cabling/firestop/equipment-mark proposition not exposed |
| m02-q207 | AHJ/code vs voluntary standard | NFPA 1, 2024 — https://link.nfpa.org/all-publications/1/2024 | **BLOCKED** — exact guideline/code distinction not exposed |
| m02-q209 | AHJ/code vs voluntary standard | NFPA 1, 2024 — https://link.nfpa.org/all-publications/1/2024 | **BLOCKED** — exact insurer/customer questionnaire response not exposed |

The ledger remains 159 PASS / 798 BLOCKED across 957 rows, with zero bare FAIL.
This pass does not certify a learner, close any ms4j bead, or alter gate-shrink
or oracle scope.

## Breadth pass 41 — CDCP standards-landscape legal receipts

**Review date:** 2026-08-18. Ten M02 standards-awareness rows now carry
official NFPA 1, ISO/IEC 22237-1:2021, or IEC ISO/IEC 22237-2:2024 catalog
receipts. The prior vendor EN 50600 URL was removed. These rows remain BLOCKED
because the official pages do not expose the exact code, regional-series,
international/national, or subcomponent proposition. No PDF was fetched.

| Item | Public CDCP heading | Official catalog/preview receipt | Bounded result |
|---|---|---|---|
| mock40-q05 | AHJ/code vs voluntary standard | NFPA 1, 2024 — https://link.nfpa.org/all-publications/1/2024 | **BLOCKED** — exact general code-versus-standard adoption proposition not exposed |
| m02-q063 | AHJ/code vs voluntary standard | NFPA 1, 2024 — https://link.nfpa.org/all-publications/1/2024 | **BLOCKED** — exact general code-versus-standard adoption proposition not exposed |
| m02-q064 | AHJ/code vs voluntary standard | NFPA 1, 2024 — https://link.nfpa.org/all-publications/1/2024 | **BLOCKED** — exact conflict-of-law precedence proposition not exposed |
| m02-q068 | EN 50600 series awareness | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact EN 50600 regional-series description not exposed |
| m02-q070 | International vs national standards | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact international-versus-national distinction not exposed |
| m02-q071 | Standards for sub-components | ISO/IEC 22237-2:2024 — https://webstore.iec.ch/en/publication/92577 | **BLOCKED** — exact sub-component standards proposition not exposed |
| m02-q072 | Standards and guidelines landscape | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact standards-and-guidelines landscape proposition not exposed |
| m02-q075 | AHJ/code vs voluntary standard | NFPA 1, 2024 — https://link.nfpa.org/all-publications/1/2024 | **BLOCKED** — exact AHJ-versus-voluntary-guidance proposition not exposed |
| m02-q076 | International vs national standards | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact international-versus-national distinction not exposed |
| m02-q078 | EN 50600 series awareness | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact EN 50600 regional-series description not exposed |
| m02-q079 | Standards for sub-components | ISO/IEC 22237-2:2024 — https://webstore.iec.ch/en/publication/92577 | **BLOCKED** — exact sub-component standards proposition not exposed |

The ledger remains 159 PASS / 798 BLOCKED across 957 rows, with zero bare FAIL.
This pass does not certify a learner, close any ms4j bead, or alter gate-shrink
or oracle scope.

## Breadth pass 40 — CDCP current-operations foundation receipts

**Review date:** 2026-08-18. Ten additional Module 1 rows now carry official
ISO/IEC 22237-1:2021 or TIA-942-C catalog/preview receipts. They remain
BLOCKED because the public pages do not expose the exact current-operations,
new-site-type, or maintenance-risk propositions. No PDF or paid standard body
was fetched.

| Item | Public CDCP heading | Official catalog/preview receipt | Bounded result |
|---|---|---|---|
| m01-q206 | Causes of unavailability | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact single-point-of-failure proposition not exposed |
| m01-q207 | Importance of a data centre | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact criticality-to-investment proposition not exposed |
| m01-q208 | Causes of unavailability | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact balanced-cause proposition not exposed |
| m01-q209 | Business organization / DC in the business | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact IT/facilities partnership proposition not exposed |
| m01-q210 | Causes of unavailability | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact 2026 survey-share/root-cause proposition not exposed |
| m01-q211 | Types of data centres | TIA-942-C (May 2024) — https://tiaonline.org/standard/tia-942/ | **BLOCKED** — exact neocloud/GPU-colo/AI-factory taxonomy not exposed |
| m01-q212 | Importance of a data centre | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact vendor-availability-claim pinning proposition not exposed |
| m01-q213 | Causes of unavailability | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact high-density cooling-dependency proposition not exposed |
| m01-q214 | Business organization / DC in the business | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact colo ownership-boundary response not exposed |
| m01-q215 | Causes of unavailability | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact degraded-path maintenance proposition not exposed |

The ledger remains 159 PASS / 798 BLOCKED across 957 rows, with zero bare FAIL.
This pass does not certify a learner, close any ms4j bead, or alter gate-shrink
or oracle scope.

## Breadth pass 39 — CDCP data-centre foundations continuation

**Review date:** 2026-08-18. Seventeen additional Module 1 rows now carry
official ISO/IEC 22237-1:2021, ISO 22301:2019/Amd 1:2024, or TIA-942-C
catalog/preview receipts. They remain BLOCKED because the public pages do not
expose the exact workload, taxonomy, organizational, or resilience propositions.
No PDF or paid standard body was fetched.

| Item | Public CDCP heading | Official catalog/preview receipt | Bounded result |
|---|---|---|---|
| m01-q052 | Elements of a data centre | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact IT-equipment-versus-facility proposition not exposed |
| m01-q053 | Importance of a data centre | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact payment/trading workload priority proposition not exposed |
| m01-q054 | Causes of unavailability | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact root-cause-category proposition not exposed |
| m01-q055 | Types of data centres | TIA-942-C (May 2024) — https://tiaonline.org/standard/tia-942/ | **BLOCKED** — exact managed-hosting ownership proposition not exposed |
| m01-q056 | Importance of a data centre | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact executive redundancy-investment proposition not exposed |
| m01-q057 | Causes of unavailability | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact cascading-unavailability scenario not exposed |
| m01-q058 | Types of data centres | TIA-942-C (May 2024) — https://tiaonline.org/standard/tia-942/ | **BLOCKED** — exact colocation-customer design-awareness proposition not exposed |
| m01-q059 | Causes of unavailability | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact reliability-culture proposition not exposed |
| m01-q060 | Importance of a data centre | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact regulated-industry requirements proposition not exposed |
| m01-q061 | Types of data centres | TIA-942-C (May 2024) — https://tiaonline.org/standard/tia-942/ | **BLOCKED** — exact careful-taxonomy statement not exposed |
| m01-q062 | Business organization / DC in the business | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact capacity-planning linkage not exposed |
| m01-q200 | Business organization / DC in the business | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact business-perspective proposition not exposed |
| m01-q201 | Causes of unavailability | ISO 22301:2019/Amd 1:2024 — https://www.iso.org/standard/88412.html | **BLOCKED** — exact organizational-cause proposition not exposed |
| m01-q202 | Types of data centres | TIA-942-C (May 2024) — https://tiaonline.org/standard/tia-942/ | **BLOCKED** — exact enterprise/colo/hyperscale comparison not exposed |
| m01-q203 | Elements of a data centre | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact beyond-IT-load element list not exposed |
| m01-q204 | Importance of a data centre | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact board-metrics proposition not exposed |
| m01-q205 | Types of data centres | TIA-942-C (May 2024) — https://tiaonline.org/standard/tia-942/ | **BLOCKED** — exact edge-versus-central tradeoff not exposed |

The ledger remains 159 PASS / 798 BLOCKED across 957 rows, with zero bare FAIL.
This pass does not certify a learner, close any ms4j bead, or alter gate-shrink
or oracle scope.

## Breadth pass 34 — CDCP physical-access control BLOCKED receipts

**Review date:** 2026-08-18. Six additional M13 items now carry official
NFPA 730/731 or ISO/IEC 27001 catalog/preview receipts. They remain BLOCKED
because the public pages do not expose the exact fail-state, time-sync,
visitor, tailgating, shared-credential, or physical-revocation propositions.
No PDF or paid standard body was fetched.

| Item | Public CDCP heading | Official catalog/preview receipt | Bounded result |
|---|---|---|---|
| bank-m13-q086 | Physical Security and Safety — Components for physical security | NFPA 731, 2026 — https://link.nfpa.org/all-publications/731/2026 | **BLOCKED** — exact fail-safe/fail-secure power-loss semantics not exposed |
| bank-m13-q087 | Physical Security and Safety — Components for physical security | NFPA 731, 2026 — https://link.nfpa.org/all-publications/731/2026 | **BLOCKED** — exact fail-secure power-loss/egress distinction not exposed |
| bank-m13-q089 | Physical Security and Safety — Components for physical security | NFPA 731, 2026 — https://link.nfpa.org/all-publications/731/2026 | **BLOCKED** — exact ACS/VMS time-sync and reconstruction proposition not exposed |
| bank-m13-q090 | Physical Security and Safety — Components for physical security | NFPA 730, 2026 — https://link.nfpa.org/all-publications/730/2026 | **BLOCKED** — exact visitor-credential/escort/restricted-zone proposition not exposed |
| bank-m13-q091 | Physical Security and Safety — Components for physical security | NFPA 730, 2026 — https://link.nfpa.org/all-publications/730/2026 | **BLOCKED** — exact tailgating/social-engineering proposition not exposed |
| bank-m13-q092 | Physical Security and Safety — Components for physical security | NFPA 731, 2026 — https://link.nfpa.org/all-publications/731/2026 | **BLOCKED** — exact shared-badge/access-accounting proposition not exposed |
| bank-m13-q093 | Physical Security and Safety — Components for physical security | ISO/IEC 27001:2022 + Amd 1:2024 — https://www.iso.org/standard/27001?browse=tc | **BLOCKED** — exact physical-credential revocation proposition not exposed |

The ledger remains 159 PASS / 798 BLOCKED across 957 rows, with zero bare FAIL.
This pass does not certify a learner, close any ms4j bead, or alter gate-shrink
or oracle scope.

## Breadth pass 35 — CDCP physical-security operations BLOCKED receipts

**Review date:** 2026-08-18. Seven additional M13 items now carry official
NFPA 730 (2026) catalog/preview receipts. They remain BLOCKED because the
public preview does not expose the exact loading-dock, colocation, detection,
dual-control, door-propping, CCTV-priority, or security-theatre propositions.
No PDF or paid standard body was fetched.

| Item | Public CDCP heading | Official catalog/preview receipt | Bounded result |
|---|---|---|---|
| bank-m13-q096 | Physical Security and Safety — Components for physical security | NFPA 730, 2026 — https://link.nfpa.org/all-publications/730/2026 | **BLOCKED** — exact loading-dock material-flow/inspection proposition not exposed |
| bank-m13-q097 | Physical Security and Safety — Components for physical security | NFPA 730, 2026 — https://link.nfpa.org/all-publications/730/2026 | **BLOCKED** — exact colocation customer-cage/common-plant zoning proposition not exposed |
| bank-m13-q099 | Physical Security and Safety — Components for physical security | NFPA 730, 2026 — https://link.nfpa.org/all-publications/730/2026 | **BLOCKED** — exact lock-without-detection limitation not exposed |
| bank-m13-q100 | Physical Security and Safety — Components for physical safety | NFPA 730, 2026 — https://link.nfpa.org/all-publications/730/2026 | **BLOCKED** — exact two-person/dual-control proposition not exposed |
| bank-m13-q101 | Physical Security and Safety — Components for physical security | NFPA 730, 2026 — https://link.nfpa.org/all-publications/730/2026 | **BLOCKED** — exact propped-door/change-window proposition not exposed |
| bank-m13-q102 | Physical Security and Safety — Components for physical security | NFPA 730, 2026 — https://link.nfpa.org/all-publications/730/2026 | **BLOCKED** — exact CCTV coverage-priority proposition not exposed |
| bank-m13-q103 | Physical Security and Safety — Components for physical security | NFPA 730, 2026 — https://link.nfpa.org/all-publications/730/2026 | **BLOCKED** — exact security-theatre/rear-layer proposition not exposed |

The ledger remains 159 PASS / 798 BLOCKED across 957 rows, with zero bare FAIL.
This pass does not certify a learner, close any ms4j bead, or alter gate-shrink
or oracle scope.

## Breadth pass 36 — CDCP physical-security and safety receipt completion

**Review date:** 2026-08-18. Six additional M13 items now carry official
NFPA 731 (2026) or NFPA 70E (2024) catalog/preview receipts. They remain
BLOCKED because the public pages do not expose the exact mantrap, material-flow,
LOTO, EPO, egress, or eyewash propositions. No PDF or paid standard body was
fetched.

| Item | Public CDCP heading | Official catalog/preview receipt | Bounded result |
|---|---|---|---|
| bank-m13-q078 | Physical Security and Safety — Components for physical security | NFPA 731, 2026 — https://link.nfpa.org/all-publications/731/2026 | **BLOCKED** — exact anti-tailgating mantrap proposition not exposed |
| bank-m13-q079 | Physical Security and Safety — Components for physical security | NFPA 731, 2026 — https://link.nfpa.org/all-publications/731/2026 | **BLOCKED** — exact people-versus-equipment mantrap/material-flow proposition not exposed |
| bank-m13-q094 | Physical Security and Safety — Components for physical safety | NFPA 70E, 2024 — https://link.nfpa.org/all-publications/70E/2024 | **BLOCKED** — exact LOTO proposition not exposed |
| bank-m13-q095 | Physical Security and Safety — Components for physical safety | NFPA 70E, 2024 — https://link.nfpa.org/all-publications/70E/2024 | **BLOCKED** — exact EPO/security-balance proposition not exposed |
| m13-q201 | Physical Security and Safety — Components for physical security | NFPA 731, 2026 — https://link.nfpa.org/all-publications/731/2026 | **BLOCKED** — exact mantrap/anti-passback proposition not exposed |
| m13-q202 | Physical Security and Safety — Components for physical safety | NFPA 70E, 2024 — https://link.nfpa.org/all-publications/70E/2024 | **BLOCKED** — exact EPO/egress/eyewash distinction not exposed |

The ledger remains 159 PASS / 798 BLOCKED across 957 rows, with zero bare FAIL.
This pass does not certify a learner, close any ms4j bead, or alter gate-shrink
or oracle scope.

## Breadth pass 37 — CDCP physical-security frontier receipts

**Review date:** 2026-08-18. Fifteen remaining M13 rows, including two
retired provenance rows, now carry official NFPA 730, NFPA 731, NFPA 70E, or
ISO/IEC 27001 catalog/preview receipts. They remain BLOCKED because the public
pages do not expose the exact item propositions. No PDF or paid standard body
was fetched.

| Item | Public CDCP heading | Official catalog/preview receipt | Bounded result |
|---|---|---|---|
| mock40-q36 | Physical Security and Safety — Components for physical security | NFPA 731, 2026 — https://link.nfpa.org/all-publications/731/2026 | **BLOCKED** — retired duplicate; exact mantrap proposition not exposed |
| mock40-q37 | Physical Security and Safety — Components for physical safety | NFPA 70E, 2024 — https://link.nfpa.org/all-publications/70E/2024 | **BLOCKED** — retired cross-module duplicate; exact MOP proposition not exposed |
| m13-q204 | Physical Security and Safety — Components for physical security | NFPA 731, 2026 — https://link.nfpa.org/all-publications/731/2026 | **BLOCKED** — exact CCTV coverage/retention/lighting/time-sync proposition not exposed |
| m13-q205 | Physical Security and Safety — Components for physical security | NFPA 731, 2026 — https://link.nfpa.org/all-publications/731/2026 | **BLOCKED** — exact piggybacking/tailgating identity-to-entry proposition not exposed |
| m13-q206 | Physical Security and Safety — Components for physical safety | NFPA 70E, 2024 — https://link.nfpa.org/all-publications/70E/2024 | **BLOCKED** — exact hazard-signage/authorized-worker proposition not exposed |
| m13-q207 | Physical Security and Safety — Components for physical security | NFPA 730, 2026 — https://link.nfpa.org/all-publications/730/2026 | **BLOCKED** — exact loading-dock freight/contraband/people-control proposition not exposed |
| m13-q208 | Physical Security and Safety — Components for physical security | NFPA 731, 2026 — https://link.nfpa.org/all-publications/731/2026 | **BLOCKED** — exact two-factor physical credential proposition not exposed |
| m13-q210 | Physical Security and Safety — Components for physical security | NFPA 730, 2026 — https://link.nfpa.org/all-publications/730/2026 | **BLOCKED** — exact colocation tenant-space separation proposition not exposed |
| m13-q211 | Physical Security and Safety — Components for physical security | NFPA 730, 2026 — https://link.nfpa.org/all-publications/730/2026 | **BLOCKED** — exact guard-force escalation/reporting proposition not exposed |
| m13-q212 | Physical Security and Safety — Components for physical security | NFPA 730, 2026 — https://link.nfpa.org/all-publications/730/2026 | **BLOCKED** — exact raised-floor/ceiling pathway-bypass proposition not exposed |
| m13-q213 | Physical Security and Safety — Components for physical security | NFPA 730, 2026 — https://link.nfpa.org/all-publications/730/2026 | **BLOCKED** — exact white-space visitor-escort/guest-audit proposition not exposed |
| m13-q214 | Physical Security and Safety — Components for physical safety | NFPA 731, 2026 — https://link.nfpa.org/all-publications/731/2026 | **BLOCKED** — exact duress-alarm/secure-communications proposition not exposed |
| m13-q215 | Physical Security and Safety — Components for physical security | ISO/IEC 27001:2022 — https://www.iso.org/standard/27001?browse=tc | **BLOCKED** — exact periodic physical-access rights-review proposition not exposed |
| m13-q216 | Physical Security and Safety — Components for physical security | NFPA 730, 2026 — https://link.nfpa.org/all-publications/730/2026 | **BLOCKED** — exact time-bounded maintenance-access/credential-return proposition not exposed |
| m13-q300 | Physical Security and Safety — Components for physical security | NFPA 731, 2026 — https://link.nfpa.org/all-publications/731/2026 | **BLOCKED** — exact emergency tailgate challenge/verify runbook proposition not exposed |

The ledger remains 159 PASS / 798 BLOCKED across 957 rows, with zero bare FAIL.
This pass does not certify a learner, close any ms4j bead, or alter gate-shrink
or oracle scope.

## Breadth pass 38 — CDCP data-centre foundations receipts

**Review date:** 2026-08-18. Fifteen Module 1 rows now carry official
ISO/IEC 22237-1:2021, ISO 22301:2019/Amd 1:2024, or TIA-942-C catalog/preview
receipts. They remain BLOCKED because the public pages do not expose the exact
mission-critical, MTBF/MTTR, ownership, taxonomy, or change-control
propositions. No PDF or paid standard body was fetched.

| Item | Public CDCP heading | Official catalog/preview receipt | Bounded result |
|---|---|---|---|
| mock40-q01 | Importance of a data centre | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact mission-critical test not exposed |
| mock40-q02 | MTBF / MTTR | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact MTBF/MTTR proposition not exposed |
| mock40-q03 | Types of data centres | TIA-942-C (May 2024) — https://tiaonline.org/standard/tia-942/ | **BLOCKED** — exact retail-colocation ownership proposition not exposed |
| mock40-q04 | Causes of unavailability | ISO 22301:2019/Amd 1:2024 — https://www.iso.org/standard/88412.html | **BLOCKED** — exact causes proposition not exposed |
| m01-q041 | Business organization / DC in the business | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact IT/facilities-silo proposition not exposed |
| m01-q042 | Types of data centres | TIA-942-C (May 2024) — https://tiaonline.org/standard/tia-942/ | **BLOCKED** — exact ownership-model proposition not exposed |
| m01-q043 | Types of data centres | TIA-942-C (May 2024) — https://tiaonline.org/standard/tia-942/ | **BLOCKED** — exact wholesale-colocation characterization not exposed |
| m01-q044 | Importance of a data centre | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact mission-critical framing not exposed |
| m01-q045 | Elements of a data centre | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact combined element set not exposed |
| m01-q046 | Types of data centres | TIA-942-C (May 2024) — https://tiaonline.org/standard/tia-942/ | **BLOCKED** — exact latency/backhaul proposition not exposed |
| m01-q047 | Causes of unavailability | ISO/IEC 22237-1:2021 — https://www.iso.org/standard/78550.html?browse=tc | **BLOCKED** — exact SPOF/change-error proposition not exposed |
| m01-q048 | Types of data centres | TIA-942-C (May 2024) — https://tiaonline.org/standard/tia-942/ | **BLOCKED** — exact hyperscale characterization not exposed |
| m01-q049 | Business organization / DC in the business | ISO 22301:2019/Amd 1:2024 — https://www.iso.org/standard/88412.html | **BLOCKED** — exact BIA-to-RTO/RPO proposition not exposed |
| m01-q050 | Types of data centres | TIA-942-C (May 2024) — https://tiaonline.org/standard/tia-942/ | **BLOCKED** — exact governance/SLA-boundary comparison not exposed |
| m01-q051 | Causes of unavailability | ISO 22301:2019/Amd 1:2024 — https://www.iso.org/standard/88412.html | **BLOCKED** — exact maintenance-change-control proposition not exposed |

The ledger remains 159 PASS / 798 BLOCKED across 957 rows, with zero bare FAIL.
This pass does not certify a learner, close any ms4j bead, or alter gate-shrink
or oracle scope.

## Breadth pass 24 — CDCP BMS and environmental-monitoring OT boundary

**Review date:** 2026-08-18. Two Module 14 rows were narrowed to the current
official NIST SP 800-82 Rev. 3 abstract. It names building automation and
physical-environment monitoring systems as OT and preserves performance,
reliability, and safety constraints. No PDF body was fetched.

| Item | Public CDCP heading | Current official abstract | Bounded result |
|---|---|---|---|
| bank-m14-q126 | Monitoring challenges | NIST SP 800-82 Rev. 3 — https://csrc.nist.gov/pubs/sp/800/82/r3/final | **PASS** — BMS/environmental units interact with the physical environment and require safety/reliability-aware security |
| m14-q202 | Building Management System (BMS) | NIST SP 800-82 Rev. 3 — https://csrc.nist.gov/pubs/sp/800/82/r3/final | **PASS** — BMS is bounded to building automation and physical-environment monitoring/control |

Vendor-specific BMS/DCIM taxonomies, protocol claims, default-password
claims, and alarm/control sequences remain BLOCKED. The ledger is now 144 PASS /
813 BLOCKED across 957 rows, with zero bare FAIL. This pass does not certify a
learner or close ms4j.

## Breadth pass 23 — CDCP TIA-942-C public topology scope

**Review date:** 2026-08-18. Two Module 11 rows were narrowed to the current
official TIA-942-C abstract. TIA’s page identifies Revision C, published May
2024, and states the infrastructure/topology scope. The purchased standard
body was not fetched.

| Item | Public CDCP heading | Current official abstract | Bounded result |
|---|---|---|---|
| m11-q100 | Importance of network cabling infrastructure | TIA-942-C, May 2024, Version C — https://tiaonline.org/standard/tia-942/ | **PASS** — the abstract covers data-center/computer-room infrastructure and topology |
| m11-q102 | Planning considerations | TIA-942-C, May 2024, Version C — https://tiaonline.org/standard/tia-942/ | **PASS** — the abstract says its topology is intended for any size data center |

MDA/HDA/ZDA role definitions, cabling test limits, labeling procedures, and
physical-diversity claims remain BLOCKED without public clause text. The ledger
is now 142 PASS / 815 BLOCKED across 957 rows, with zero bare FAIL. This pass
does not certify a learner or close ms4j.

## Breadth pass 22 — CDCP water-supply and cooling-tower operating boundaries

**Review date:** 2026-08-18. Five Module 10 rows were narrowed to current
public DOE/FEMP cooling-tower and federal-data-center guidance. The public EPI
syllabus supplies the exact “Importance of water” heading. No linked PDF body
was fetched.

| Item | Public CDCP heading | Official public receipt | Bounded result |
|---|---|---|---|
| m10-q103 | Importance of water | DOE FEMP Cooling Tower Management — https://www.energy.gov/cmei/femp/best-management-practice-10-cooling-tower-management | **PASS** — makeup replaces evaporation, drift, and blowdown losses |
| m10-q104 | Importance of water | DOE FEMP Cooling Tower Management — https://www.energy.gov/cmei/femp/best-management-practice-10-cooling-tower-management | **PASS** — blowdown removes concentrated water to control dissolved-solids concentration |
| m10-q105 | Importance of water | DOE FEMP Cooling Tower Management — https://www.energy.gov/cmei/femp/best-management-practice-10-cooling-tower-management | **PASS** — uncontrolled concentration can produce scale/corrosion risk; no treatment program is invented |
| m10-q113 | Importance of water | DOE FEMP Cooling Tower Management — https://www.energy.gov/cmei/femp/best-management-practice-10-cooling-tower-management | **PASS** — basin level, makeup, and loss monitoring can expose leaks or unaccounted losses |
| m10-q211 | Importance of water | DOE FEMP Cooling Water Efficiency Opportunities for Federal Data Centers — https://www.energy.gov/cmei/femp/cooling-water-efficiency-opportunities-federal-data-centers | **PASS** — WUE is annual site water usage divided by annual IT equipment energy |

Water-quality thresholds, universal cycles, backup-water sizing, permit status,
and the hydronic-leak runbook remain BLOCKED. The ledger is now 140 PASS / 817
BLOCKED across 957 rows, with zero bare FAIL. This pass does not certify a
learner or close ms4j.

## Breadth pass 21 — CDCP cooling controls, containment, and liquid-cooling boundaries

**Review date:** 2026-08-18. Four Module 09 rows were narrowed to claims
exposed by the current official ASHRAE preview and public ASHRAE AI Data
Center Energy Performance Framework. The preview page lists Standard 90.4-2025;
no standard body or PDF was fetched.

| Item | Public CDCP heading | Official preview/public receipt | Bounded result |
|---|---|---|---|
| m09-q108 | Cooling principles | ASHRAE Standard 90.4-2025 preview — https://www.ashrae.org/technical-resources/standards-and-guidelines/read-only-versions-of-ashrae-standards; public framework — https://www.ashrae.org/technical-resources/ai-data-center-framework/energy-and-thermal-efficiency | **PASS** — economizer pathways reduce compressor hours and mechanical cooling demand |
| m09-q148 | Containment | ASHRAE Standard 90.4-2025 preview — https://www.ashrae.org/technical-resources/standards-and-guidelines/read-only-versions-of-ashrae-standards; public framework — https://www.ashrae.org/technical-resources/ai-data-center-framework/energy-and-thermal-efficiency | **PASS** — containment and precise airflow control reduce fan energy and stabilize equipment inlet temperatures |
| m09-q161 | Liquid cooling | ASHRAE Standard 90.4-2025 preview — https://www.ashrae.org/technical-resources/standards-and-guidelines/read-only-versions-of-ashrae-standards; public framework — https://www.ashrae.org/technical-resources/ai-data-center-framework/energy-and-thermal-efficiency | **PASS** — direct-to-chip and rear-door approaches support high-density heat removal without inventing a rack class or setpoint |
| m09-q300 | Containment | ASHRAE Standard 90.4-2025 preview — https://www.ashrae.org/technical-resources/standards-and-guidelines/read-only-versions-of-ashrae-standards; public framework — https://www.ashrae.org/technical-resources/ai-data-center-framework/energy-and-thermal-efficiency | **PASS** — restore the disturbed airflow path and verify inlet conditions before setpoint changes |

W-class taxonomy, universal temperature/setpoint claims, and thermal ride-through
remain BLOCKED. The ledger is now 135 PASS / 822 BLOCKED across 957 rows, with
zero bare FAIL. This pass does not certify a learner or close ms4j.

### Module 12 follow-up

| Item | Public CDCP heading | Official public-code receipt | Bounded result |
|---|---|---|---|
| m12-q208 | 1.12 Fire Protection | 29 CFR 1910.160 — https://www.ecfr.gov/current/title-29/subtitle-B/chapter-XVII/part-1910/subpart-L/subject-group-ECFR7a02737a205fd22/section-1910.160 | **PASS** — impairment notification, temporary precautions, trained restoration, inspection, and agent-container readiness |

The follow-up does not promote room-integrity testing, a universal recovery
procedure, or any unexposed agent-selection claim.

## Breadth pass 19 — CDCP physical-safety egress boundary

**Review date:** 2026-08-18. One Module 13 row was narrowed to current public
eCFR exit-route requirements. Badge, mantrap, CCTV, and access-review claims
remain BLOCKED where no qualifying public code clause was exposed.

| Item | Public CDCP heading | Official public-code receipt | Bounded result |
|---|---|---|---|
| m13-q209 | Physical Security and Safety — Components for physical safety | 29 CFR 1910.37 — https://www.ecfr.gov/current/title-29/subtitle-B/chapter-XVII/part-1910/subpart-E/section-1910.37 | **PASS** — access hardware must not obstruct usable emergency egress, and required safeguards must remain operable |

The ledger is now 127 PASS / 830 BLOCKED across 957 rows, with zero bare FAIL.
No physical-security taxonomy, credential claim, or ms4j closure was added.

## Breadth pass 20 — CDCP grounding and power-sizing public-code boundaries

**Review date:** 2026-08-18. Four Module 06 rows were narrowed to claims
directly exposed by the current public OSHA electrical rules. The public EPI
syllabus supplies the exact headings “Grounding and bonding” and “Power sizing.”
No standard body or PDF was fetched.

| Item | Public CDCP heading | Official public-code receipt | Bounded result |
|---|---|---|---|
| m06-q217 | Grounding and bonding | OSHA 1910.304 — https://www.osha.gov/laws-regs/regulations/standardnumber/1910/1910.304 | **PASS** — grounding-connection location is tested without inferring a universal downstream N-G topology |
| m06-q218 | Grounding and bonding | OSHA 1910.304 — https://www.osha.gov/laws-regs/regulations/standardnumber/1910/1910.304 | **PASS** — metal cable trays, raceways, and conductor enclosures are bounded to the public grounding requirement |
| m06-q248 | Grounding and bonding | OSHA 1910.303 and 1910.304 — https://www.osha.gov/laws-regs/regulations/standardnumber/1910/1910.303 and https://www.osha.gov/laws-regs/regulations/standardnumber/1910/1910.304 | **PASS** — grounding-path and overcurrent-protection duties remain distinct; no setting or coordination result is invented |
| m06-q253 | Power sizing | OSHA 1910.304 — https://www.osha.gov/laws-regs/regulations/standardnumber/1910/1910.304 | **PASS** — conductors and equipment are protected from overcurrent according to their safe current-carrying ability; no universal continuous-load percentage is claimed |

Harmonics, thermography, IP grades, and metering rows remain BLOCKED because
the reviewed official pages did not expose a qualifying public claim. The
ledger is now 131 PASS / 826 BLOCKED across 957 rows, with zero bare FAIL.
This pass does not certify a learner or close ms4j.
