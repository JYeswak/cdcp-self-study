# Attribution bar — 2026-08-17

Joshua: AI stems with no verified attribution are not good enough. Ten calc files are a floor, not the course.

This program does not certify anyone. 27/40 is a CDCP study signal, not a CDCS/CDFOS cut score. No dumps. Original prose only. Citation metadata only. No course body retained.

## Rule

Do not write an item unless all three are true:

1. It maps to a **public syllabus heading** on the live EPI page (URL + heading text).
2. It is examined against a **current public standard or primary technical body** (edition + URL). Vendor blogs and staging help pages are not enough.
3. The numeric or one-line claim is **re-derivable from that source**. If you cannot point at the clause or formula, do not write the item.

If the public source is missing, file a leftover bead. Do not invent the number.

## Public CDCS modules

Source: https://www.epi-ap.com/services/1/3/5

1. Data Centre Design / Life Cycle Overview
2. Standards and Rating Level Definitions
3. Building Considerations
4. Advanced Raised Floor and Suspended Ceiling
5. Advanced Power (SLD, OCPD, generators, fuel, UPS parallel, batteries, flywheel, H2)
6. Advanced Electro Magnetic Fields
7. Advanced Cooling (psychrometric, ASHRAE, CFM/CMH, Delta-T, liquid)
8. Advanced Fire Protection
9. Designing and Installing Scalable Network Cabling Systems
10. Environmental Specifications / Contamination Control
11. Data Centre Efficiency

Public exam fact (do not copy items; do not adopt as our pass mark): 60 questions, 90 minutes, 45 correct. Prerequisite: valid CDCP.

Live track today: 10 heading files under `course-engine/tracks/cdcs/bank/`. That covers a slice of 5–8 only. Missing 1–4 and 9–11, and most of Advanced Power.

## Public CDFOS modules

Source: https://www.epi-ap.com/services/1/3/136/Certified_Data_Centre_Facilities_Operations_Specialist_(CDFOS)

Aligned with DCOS (name the standard; do not vendor the body).

1. Service Level Management
2. Safety and Crisis Management
3. Physical Security
4. Facilities Maintenance
5. Data Centre Operations
6. Monitoring / Reporting / Control
7. Project Management
8. Environmental Sustainability
9. Governance and Compliance

Public exam fact (do not copy items): 60 questions, 90 minutes, 42 correct.

## Current standards to pull (public pages, latest edition)

Do not law-ify a percentage. Rated ≠ Tier ≠ Availability Class.

- TIA-942-C (Rated, cabling topology)
- EN 50600 / Availability Class
- Uptime Institute: three certificate types, not nines-on-Tiers
- ASHRAE TC 9.9 thermal guidelines; W-classes = supply-water
- NFPA 855-2026; UL 9540A
- ISO/IEC 30134-2:2026 (PUE / mPUE)
- IEEE / IEC one-line and protective-device practice (public overviews only)
- NIST unit conversions where the calc is a unit trap
- OSHA 1910 Subpart S / LOTO and 1910.162 where ops/fire need them

Vendor application notes (Schneider, Cummins, Trane) are secondary. They may illustrate a calc after a standard cite exists. They do not replace the standard.

## Item fields required from here

- `syllabus_heading` = exact public heading
- `syllabus_url` = the EPI page above
- `source_ids` = corpus ids that resolve to edition + URL
- `claim` = the one thing the candidate must compute or catch
- If any field is empty, the item is not ready
