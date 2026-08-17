# SOURCES — Opus lens, 2026-08-17

Every public source this pass actually reached, and what it **confirmed** or **killed**.

**Access honesty.** Two access classes, marked per row, because they are not equally strong:

- **FETCHED** — page retrieved and read directly (WebFetch). Strongest.
- **SEARCH** — search-result summary and snippets only; the underlying page was **not** opened. Adequate for a headline, **not** adequate to quote as clause-level law.

No paywalled standard was purchased or reproduced. No proprietary EPI/EXIN body text was read or pasted. **No dump site was opened** (see §3).

---

## 1. EPI / EXIN — the syllabus and exam record

| # | Source | Access | Confirmed / Killed |
|---|---|---|---|
| S1 | EPI — CDCP course page <br><https://www.epi-ap.com/services/1/3/4/Certified_Data_Centre_Professional_(CDCP)> | **FETCHED** | **CONFIRMED** the 14 facility module headings, in order, 1:1 with `00-curriculum-map.md:15-28` — the map is right. <br>**KILLED** `README.md:403-405` "not affiliated with any official cut score": this page states **"Passing Mark: 27 out of 40 questions."** <br>**CONFIRMED** 40 MCQ / 1 hour / closed-book. <br>**CONFIRMED** the published outline contains **no 2.1 module** — so M15's `exam_weight_unknown = true` is the honest reading. |
| S2 | EXIN — Certified Data Centre Professional <br><https://www.exin.com/technologies-software/exin-epi-data-centre-management/certified-data-centre-professional/> | **FETCHED** | **CONFIRMED** 40 questions / 1 hour / closed book / **Pass Mark 68%** / Foundation level / electronic equipment not allowed. <br>**CONFIRMED** **"Training: Mandatory"** — corroborates the repo's standing claim that only the official exam after authorised training grants the credential. <br>**OPENED A QUESTION:** names **three** subject areas — Facilities, Operations, **and Management**. The corpus maps Facilities + Operations only (FINDINGS #10). <br>**TENSION with S1:** 68% of 40 implies 28/40; S1 says 27/40. The two public sources disagree; the repo matches the more permissive one. |
| S3 | EPI — Data Center Training Framework <br><https://www.epi-ap.com/services/1/3/2/EPI_Data_Center_Training_Framework> | **FETCHED** | **CONFIRMED** five published groups and the full acronym set: Foundation **DCFC, CDCA**; Design/Build CDCP, CDCS, CDCE, CNCDP; Maintenance/Operations CDFOS, CDFOM, **CDESS**; Risk CDRP, CDMS; Standards/Compliance **CDLI, CDLA**, CTDC, CTIA, **CTLA**. <br>**CONFIRMED** the ecosystem map's `CTIA/CTLA` pairing is correct — **CTLA = Certified TIA-942 Lead Auditor**. This *killed my own working hypothesis* that it should read CTEA; I checked before writing and was wrong. <br>**KILLED** the ecosystem map's completeness: **DCFC, CDCA, CDLI, CDLA absent** (FINDINGS F4). <br>**FLAGGED** CDLI/CDLA are **DCOS**, a different scheme from the TIA-942 audit trio. |
| S4 | EPI framework track listing (corroboration) <br><https://www.epi-ap.com/services/1/3/136/Certified_Data_Centre_Facilities_Operations_Specialist_(CDFOS)> and Datacenter Forum framework page | SEARCH | **CONFIRMED** the four-track + foundation structure independently of S3. Used only to corroborate S3's grouping. |

---

## 2. Standards and 2026 floor

| # | Source | Access | Confirmed / Killed |
|---|---|---|---|
| S5 | Uptime Institute — Annual Outage Analysis **2026** (8th ed.), public press summary <br><https://uptimeinstitute.com/about-ui/press-releases/uptime-announces-annual-outage-analysis-report-2026> · <https://intelligence.uptimeinstitute.com/resource/annual-outage-analysis-2026> | SEARCH | **CONFIRMED the corpus's 2026 reframing, externally:** *"Power remains the leading cause of impactful outages… failures involving UPS systems, transfer switches and generators are dominant"*; *"failures to follow established procedures remain the leading driver of human error-related outages"* — i.e. human error as a **mechanism within** a category, not a peer bucket. This is precisely `m01-q210` / M01 Q10. <br>**KILLED** `mock40-q04`'s explanation by the same token (FINDINGS F2). <br>Also public: outage frequency per site declining a fifth consecutive year; ~1 in 10 report serious/severe impact; from the 2025 survey, 57% of most-recent major outages cost >\$100k and 1 in 5 >\$1M. <br>**Recorded as survey findings with attribution — not adopted as law, and no percentage is imported into any module or item.** |
| S6 | ASHRAE — *AI Data Center Energy Performance Framework* + 5th-ed. *Thermal Guidelines* liquid chapter, via ASHRAE white paper and Upsite/CIBSE/DCD explainers <br><https://www.ashrae.org/file%20library/technical%20resources/bookstore/emergence-and-expansion-of-liquid-cooling-in-mainstream-data-centers_wp.pdf> · <https://www.upsite.com/blog/major-changes-to-ashraes-fifth-edition-of-thermal-guidelines-part-3-liquid-cooling-chapter-updates/> · <https://www.cibsejournal.com/cpd/modules/2025-09-lcdca/> | SEARCH | **CONFIRMED `modules/09-cooling.md:187-196` exactly**: rename W1→**W17**, W2→**W27**, W3→**W32**, **W40 new**, W4→**W45**, W5→**W+**; the number is the **maximum/upper supply-fluid temperature in °C**; **all classes share a 2 °C lower limit**. The corpus states the shared floor, which most secondary summaries omit. <br>**CONFIRMED** the rationale the module teaches (classes redesignated by upper limit for memorability; ~40 °C entering-water designs are the reason W40 exists). <br>**CONFIRMED** TC 9.9 maintains three complementary systems — water **quality**, water **temperature**, surface temperature — a distinction the corpus does not yet make (minor, not filed). |
| S7 | NFPA 855 (2026) + UL 9540A 6th ed. <br><https://www.ul.com/thecodeauthority/knowledge/understanding-UL-9540A-NFPA-855> · <https://www.telgian.com/nfpa-855-changes-in-2026/> · <https://www.mayfield.energy/technical-articles/the-6th-edition-of-ul-9540a-is-here/> · <https://cleanpower.org/wp-content/uploads/gateway/2026/05/ACP_FactSheet_Battery_LSFT.pdf> | SEARCH | **CONFIRMED `modules/06-power.md:238-239`, including the part the industry gets wrong:** UL 9540A is a **test method** (having it = having test data), **UL 9540 is the listing path**, NFPA 855 is the **installation** standard whose **adopted edition** governs. <br>**CONFIRMED** NFPA 855 **2026** adds **Annex G.11** large-scale fire test (LSFT) guidance for BESS-to-BESS spread; **UL 9540A 6th edition published 13 March 2026**, revising Section 10 to a large-scale fire test method aligned to Annex G.11. <br>**CONFIRMED** UL 9540A is the only consensus standard cited in NFPA 855 for LSFT. |
| S8 | ANSI/TIA-942-C (2024) <br><https://www.belden.com/blog/introducing-ansi-tia-942-c-recent-updates-to-data-center-standards> · <https://www.cablinginstall.com/standards/article/55245177/tia-942-c-data-center-standard-brings-a-host-of-changes-and-updates> · <https://tiaonline.org/products-and-services/tia942certification/ansi-tia-942-standard/> | SEARCH | **CONFIRMED** the `ANSI/TIA-942-C (2024)` edition pin in `modules/02-standards.md:13,62-66`: released April/May 2024, replaces 942-B, folds in the 942-B-1 edge addendum, adds AI/higher cabling-and-rack-density guidance and an **informative annex on liquid immersion cooling**. <br>**SURFACED A NEW DEFECT (FINDINGS #9):** secondary and vendor sources attach the **99.671 / 99.741 / 99.982 / 99.995** table to **Rated-1..4**, not only to Uptime Tiers — e.g. <https://thenetworkinstallers.com/blog/ansi-tia-942-c-standard/> and <https://www.score-grp.com/en/post/data-center-standards-iso-ansi-tia-942-and-tiers-in-2026-how-to-design-classify-and-operate-re>. M02 name-and-kills only the **Tier**-badged form. <br>**Not verified against TIA's own text** (paid). The finding is "the myth circulates in this second form," **not** "TIA publishes these nines." |
| S9 | EN 50600 / ISO/IEC 22237 / ISO/IEC 30134 <br><https://www.techerati.com/features-hub/explaining-the-new-family-of-iso-data-centre-standards/> · <https://www.hknow.de/en/planning/en50600/> · <https://www.stulz.com/newsroom/detail/making-efficiency-comparable-kpis-for-en-50600-compliant-data-centers/> · <https://standards.iteh.ai/catalog/standards/clc/955e2fa0-4413-4b81-a9b6-e22d9797025c/en-50600-3-1-2026> | SEARCH | **CONFIRMED `modules/02-standards.md:15,139-145`:** ISO/IEC 22237 carries **Availability Classes 1–4** with **separate Protection Classes** — public German-language sources add granularity levels **GN1–GN3** (VK1–4 / SK1–4 / GN1–3). <br>**CONFIRMED** ISO/IEC 30134 / EN 50600-4-x as the KPI series: **PUE** (50600-4-2), **WUE**, **CUE** (50600-4-8), plus REF and CER. <br>**NOTED** an **EN 50600-3-1:2026** edition (management and operational processes) exists — relevant to the CDFOM/CDFOS lanes. <br>**CAUTION LOGGED:** one source claims EN 50600 "merged with ISO 30134 to create ISO/IEC 22237." That garbles two distinct series (22237 = facilities; 30134 = KPIs). **Not adopted.** |

---

## 3. Deliberately NOT opened

Searching the public syllabus surfaced CDCP "exam questions" pages — **dumpspedia**, **killexams**, and an OpenExamPrep "100 Data Centre Questions" practice set. **None was opened**, per the honesty constitution (`CHARTER.md:50`, `claim-forbidden-dump-bank`) and the BRIEF's no-dumps rule.

Recorded because their existence is itself a finding: the free-CDCP-practice niche is dominated by dump-shaped material, which is the competitive reason `source_class = "original"` (846/846) is worth the cost of enforcing.

The EXIN *Preparation Guide* PDF (`epi-certification.com`, May 2016 edition) was surfaced by search and **not fetched** on this pass. Prior passes cite its public **topic titles** only. It is the correct next source to resolve FINDINGS #10 (the "Management" third area) — **titles only, never body text**.

---

## 4. In-repo evidence read (not a public source, listed for traceability)

`BRIEF.md` · `CHARTER.md` · `00-curriculum-map.md` · `docs/curriculum-grades/{pass-01,pass-02,pass-05,pass-practice-sittings-2026-08-17,epi-ecosystem-map}.md` · `modules/{01,02,15}` in full and `{06,09,12}` at cited lines · `course-engine/registries/doc-facts.toml` · `course-engine/bank/items/{m01-q004,m01-q210}.toml` + aggregate greps over all 846 item files · `course-engine/web/{index,mock,learn,results}.html` · `practice/PRACTICE-EXAM.md:40-45` · `br list` for `bd-curriculum-truth-ebrr` (.1–.27 closed, **.28 open**) and `bd-epi-ecosystem-ms4j` (.1–.12 open).

`~/.grok/skills/charter/references/frankensuite-arsenal.md` read per BRIEF §"Arsenal first"; no primitive was warranted this pass (see FINDINGS §"Reading of the arsenal").

**Not run:** `cargo build` / `test` / `run`; `check.sh`. Consequence stated plainly — where FINDINGS says a gate does or does not catch something, that is read **from registry source**, not from an observed run.

---

*Source ledger only. Completing this program does not certify anyone.*
