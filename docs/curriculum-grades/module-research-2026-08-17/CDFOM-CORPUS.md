# CDFOM corpus — public headings × current primary sources

**Bead:** `bd-epi-ecosystem-ms4j.3` (parent `bd-epi-ecosystem-ms4j` — **not closed**).
**Date:** 2026-08-18. **Bar:** `ATTRIBUTION-BAR.md`.

This is a heading-to-source ledger, not CDFOM course prose and not a question
dump. No proprietary DCOS/EPI body is used. Completing this work does not
certify anyone.

## Public syllabus

**FETCHED:** EPI, *Certified Data Center Facilities Operations Manager (CDFOM)*:
https://www.epi-ap.com/services/1/3/8/Certified_Data_Centre_Facilities_Operations_Manager_%28CDFOM%29

The item headings below use the page's exact syllabus headings and bullets:
The Data Center Organization; Managing Safety & Statutory Requirements;
Managing Physical Security; Facilities Management; Project Management;
Organizational Resilience; and Governance, Risk and Compliance.

## Cites used

| ID | Edition / date | URL | Clauses used |
|---|---|---|---|
| `src-epi-cdfom-page` | Live public syllabus fetched 2026-08-18 | https://www.epi-ap.com/services/1/3/8/Certified_Data_Centre_Facilities_Operations_Manager_%28CDFOM%29 | Exact public headings and bullets named in each item |
| `src-nist-sp800-53-r5-5.2.0` | NIST SP 800-53 Rev. 5, current minor Release 5.2.0 (2025-08-27); control text in the free Rev. 5 publication | https://csrc.nist.gov/pubs/sp/800/53/r5/upd1/final ; PDF: https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-53r5.pdf | AT-2(a)-(d), AT-3(a)(1)-(2), (c), AT-3(1); PE-2(a)-(d), (1), PE-8(a)-(c); CP-2(a)(1)-(4), (2), CP-4(a)-(c), (1), CP-7(a)-(c), (4); MA-2(a)-(c), (e), MA-5(a)-(c), MA-6; SA-9(a)-(c); SR-3(a)-(c); RA-3(a)-(f); IR-4(a)-(c) and discussion |
| `src-doe-o360-1d` | DOE O 360.1D, *Federal Employee Training*, approved 2022-12-09 | https://www.energy.gov/documents/federal-employee-training | 4.a.(4)(a)-(b) and 4.a.(6)(a)-(b): individual development plans tied to job requirements, professional development, and competencies; competency development through training and rotational/developmental assignments |
| `src-ecfr-1910-147-2026-08-14` | 29 CFR 1910.147, eCFR display current 2026-08-14; Title 29 last amended 2026-08-04 | https://www.ecfr.gov/current/title-29/subtitle-B/chapter-XVII/part-1910/subpart-J/section-1910.147 | (c)(6)(i)-(ii) periodic inspection and record; (c)(7)(iv) training record |
| `src-ecfr-1910-132-2026-08-14` | 29 CFR 1910.132, eCFR display current 2026-08-14; Title 29 last amended 2026-08-04 | https://www.ecfr.gov/current/title-29/subtitle-B/chapter-XVII/part-1910/subpart-I/section-1910.132 | (d)(1)-(2) hazard assessment and written certification; (f)(1)-(3) PPE training, demonstration, and retraining |
| `src-doe-femp-cx-2006` | DOE FEMP, *Commissioning for Federal Facilities: A Practical Guide to Building Commissioning, Recommissioning, Retrocommissioning, and Continuous Commissioning* (2006) | https://www.energy.gov/sites/default/files/2014/07/f17/commissioning_fed_facilities.pdf | Introduction, pp. 2-7: verification/documentation, integrated systems, O&M training, and commissioning through acceptance/post-occupancy |
| `src-nist-sp800-160-v1r1` | NIST SP 800-160 Vol. 1 Rev. 1, *Engineering Trustworthy Secure Systems* (November 2022) | https://csrc.nist.gov/pubs/sp/800/160/v1/r1/final ; PDF: https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-160v1r1.pdf | H.14 and H.14.3 DS-1.1, DS-2.1-DS-2.4: prepare for secure disposal, preserve remaining functions, then deactivate, remove, and disassemble system elements |
| `src-nist-sp800-88-r2` | NIST SP 800-88 Rev. 2, *Guidelines for Media Sanitization* (September 2025) | https://csrc.nist.gov/pubs/sp/800/88/r2/final ; PDF: https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-88r2.pdf | §§4.3.1, 4.3.6, 4.5.1-4.5.2, 4.6, and Appendix C: life-cycle disposition decisions, validation, sanitization records, and post-sanitization destination tracking |
| `src-gsa-p100-2024` | GSA, *Facilities Standards for the Public Buildings Service (PBS P100 2024, v2)* (2024 edition; page updated 2026-02-03) | https://www.gsa.gov/real-estate/facilities-standards-for-the-public-buildings-service ; PDF: https://www.gsa.gov/system/files/P100%202024%20Final%2010012024.pdf | §§1.8.4-1.8.5 and A.1.9.1: Total Building Commissioning verification/documentation, independent commissioning provider, operator preparation, and O&M hand-off |

NIST's 2025 planning note lists the controls changed by Release 5.2.0; the
AT-3, PE-2, CP-2, MA-2, SA-9, and SR-3 clauses used here are not among those
listed changes. The PDF remains the public clause text behind those control
IDs.

## Blocked-on-sourcing rows

- Generic physical data-center demolition/decommissioning: **blocked-on-sourcing**.
  NIST SP 800-160 Vol. 1 Rev. 1 and SP 800-88 Rev. 2 now support narrower
  secure system-element and storage-asset disposition items, but neither is a
  facility-demolition standard. DOE G 430.1-4 (effective 1999-09-02) is public
  and active in the DOE directives listing, but its stated scope is contaminated
  excess facilities; it is not a current generic data-center decommissioning
  clause.
- ASHRAE Guideline 0/202, ISO 41001, ISO 55001, and ISO 22301: named only;
  public clause text was not available for a verifiable current-edition item.
  The public GSA PBS P100 2024 source is used instead for the narrower
  independent-commissioning and operator-handoff claim.

## Follow-up items

The second CDFOM pass adds only clauses that were not used by the CDFOS `.2`
wave: `m15-q363` (DOE O 360.1D career development), `m15-q364`
(NIST CP-4 continuity-plan testing), `m15-q365` (NIST CP-7 alternate-site
readiness), `m15-q366` (NIST MA-5 outsourced maintenance personnel),
`m15-q367` (NIST MA-6 spare-part timing), and `m15-q368` (NIST PE-8
visitor-record review). This continuation adds four further public-heading
items: `m15-q378` (safety awareness training; 29 CFR 1910.132(d)(1)-(2),
(f)(1)-(3)), `m15-q379` (risk management; NIST RA-3(a)-(f)), `m15-q380`
(security awareness; NIST AT-2(a)-(d)), and `m15-q381` (security incident
management; NIST IR-4(a)-(c)). The NIST SP 800-53 and eCFR rows above carry
the current editions and public clause URLs; no CDFOS monitoring or
document-management item was re-minted here. Demolition and the named
paywalled standards remain blocked-on-sourcing.
