# WAVE 01 — spine research: M01 · M15 · M02

**Bead:** `bd-curriculum-truth-ebrr.29.1` · **Date:** 2026-08-17 · **Pane:** Claude, `cdcp` 0.1
**Scope:** public-source research + taught-vs-2026 gap + bank rigor for Modules 01, 15, 02.
**Not done here:** no module prose, no README, no `check.sh`, no CHARTER, no `cargo`, no bead closed.

> Completing this program does not certify anyone. 27/40 is a study signal, not a pass mark.
> Rated ≠ Tier ≠ Availability Class. No outage percentage is invented; survey figures are attributed.

---

## Counts, re-measured for this ledger

```bash
ls course-engine/bank/items/*.toml | wc -l                              # 854
grep -h '^status = ' course-engine/bank/items/*.toml | sort | uniq -c   # 829 approved / 25 retired
ls course-engine/bank/items/*m01* | wc -l                               # 42
ls course-engine/bank/items/*m02* | wc -l                               # 48
ls course-engine/bank/items/*m15* | wc -l                               # 47
grep -rho 'quantity_evidence = "[^"]*"' course-engine/bank/items/ | sort | uniq -c
#   854 quantity_evidence = "qualitative_only"
```

| Module | Items | analyze | apply | understand | remember |
|---|---|---|---|---|---|
| M01 | 42 | 3 | 10 | 20 | 9 |
| M02 | 48 | **1** | 13 | 24 | 10 |
| M15 | 47 | 5 | 14 | 25 | 3 |

**M15 grew 39 → 47 since this morning** (`bd-curriculum-truth-ebrr.28`, committed `717f0b8`). **M02 carries one analyze item in forty-eight.** No count in this ledger is inherited; each was re-run above. Gate numbers (steps / injections / suites) are receipt-enforced and deliberately not restated here.

**Sitting baseline:** `pass-practice-sittings-2026-08-17.md` records **287/287** across 13 assessments. That is an easiness measurement, not a competence one — the ledger's own reading is that three of four choices are usually absurd.

---

## Module 01 — The Mission-Critical Site

### Public sources

| Source | URL | What it settles |
|---|---|---|
| Uptime Institute **Annual Outage Analysis 2026** (8th ed.), public press summary | <https://uptimeinstitute.com/about-ui/press-releases/uptime-announces-annual-outage-analysis-report-2026> · <https://intelligence.uptimeinstitute.com/resource/annual-outage-analysis-2026> | **Power remains the leading cause of impactful outages**; failures involving **UPS systems, transfer switches and generators** are dominant. **"Failures to follow established procedures remain the leading driver of human error-related outages."** Frequency per site declining a fifth consecutive year. |
| EPI CDCP public course outline | <https://www.epi-ap.com/services/1/3/4/Certified_Data_Centre_Professional_(CDCP)> | Module 1 heading is "The Mission Critical Site"; 40 MCQ / 1 hour / closed book. |

**Attribution discipline:** Uptime's cost figures (from its 2025 survey: 57% of most-recent major outages >\$100k; 1 in 5 >\$1M; ~1 in 10 of last outages serious or severe) are *survey findings*, cited as such. They are **not** imported into any module or item, and no percentage is invented.

### Taught-vs-2026

**The notes are correct and externally corroborated.** `modules/01-mission-critical.md:17` frames facility unavailability as power-path-led, cooling-as-cascade, human/process as **contributing factor** — which is precisely what the Uptime 2026 summary describes (procedure-failure as the driver *within* human error, not a peer bucket). The 2026 site types (neocloud, GPU colo, AI factory, BTM campus) are taught at `:64-81`. The four-seat org split is at `:82-98`.

**No 2026 gap found in M01's prose.** This module was closed by `ebrr.2` and the close holds.

### Bank rigor — one live contradiction, unchanged since Pass 7

`bank/items/m01-q004.toml` is **still `status = "approved"`**:

```toml
id = "mock40-q04"
stem = "Which factor is frequently cited as a major contributor to data-centre outages in industry analyses?"
correct = "B"        # "Human error during change/maintenance"
explanation = "Process and human error during change/maintenance work is a classic major outage driver."
bloom = "remember"
topic_ids = ["m01-unavailability"]
```

`m01-q210.toml` is also approved, shares `topic_ids = ["m01-unavailability"]`, and makes that same proposition the **wrong** answer. The sampler can draw either. `modules/01-mission-critical.md:409` lists the claim in a misconception table as an *unsourced majority to refuse*.

The stem's hedge ("frequently cited… in industry analyses") is survivable. **The explanation is not** — it asserts the claim flatly, and it is the text a learner reads after answering. The same item is `practice/PRACTICE-EXAM.md:40-45`, Q4 of the capstone.

Ranked #1 by Pass 7, #1 again by the 2026-08-17 sittings, and #2 in `FINDINGS-OPUS.md`. **It has never been beaded.** Beads `.2` and `.11` retired the cartoon from the M01 and M10 *notes*; no bead covers the capstone or this item.

**Cartoon-vs-exam-depth:** 9 of 42 M01 items are `remember`. The sittings ledger names the M01 pattern directly — "white space = IT floor" as Q1, "~99.99% ≈ an hour" as a table lookup. The exam-depth items that exist (`m01-q212` scope-of-nines, the two-green-objects demarc item) were **not drawn** in sampled seeds 42/7/99.

### CDCS-calc hole (name only — belongs on `bd-epi-ecosystem-ms4j.1`)

M01 teaches availability arithmetic conceptually — MTBF, MTTR, nines-to-annual-downtime — and **no item computes anything**, because all 854 items are `qualitative_only`. The named calc for the CDCS lane: **availability from MTBF and MTTR** (`A = MTBF / (MTBF + MTTR)`), and nines → annual downtime over 8760 h. Do not implement here.

---

## Module 15 — Operational Considerations (2.1)

### Public sources

| Source | URL | What it settles |
|---|---|---|
| EPI CDCP public outline | <https://www.epi-ap.com/services/1/3/4/Certified_Data_Centre_Professional_(CDCP)> | The published outline lists **14 facility modules and no 2.1 module**. This is why M15 correctly stays `exam_weight_unknown = true` — the weight is not public, so none is claimed. |
| EXIN EPI CDCP | <https://www.exin.com/technologies-software/exin-epi-data-centre-management/certified-data-centre-professional/> | Names **three** subject areas: Facilities, Operations, **and Management**. 40 Q / 1 h / closed book / **Training Mandatory: Yes**. |
| EN 50600-3-1:2026 (management and operational processes) | <https://standards.iteh.ai/catalog/standards/clc/955e2fa0-4413-4b81-a9b6-e22d9797025c/en-50600-3-1-2026> | A **2026** edition exists for the operational-process part — the standards anchor for ops content. |

**Open question, not a defect:** EXIN names a third area, *Management*. The corpus maps Facilities (14) + Operations (2.1) only. Whether that is a third syllabus area or EXIN's grouping of the same 85/15 split is **not resolvable from the pages read**. The EXIN *Preparation Guide* PDF is the correct next source, **topic titles only, never body text**. Do not move prose on this until it is resolved.

### Taught-vs-2026

The notes now teach all nine 2.1 headings (`modules/15-ops-adjacent.md:106-200`) plus the four 2026 EOPs (CDU-leak isolation, Li-ion/BESS, seconds-scale shed, curtailment/BTM islanding). `ebrr.16` closed this and the close holds.

### Bank rigor — the `.28` items landed; two headings remain unbanked

Measured across all 854 item files:

| 2.1 heading | Files |
|---|---|
| service catalog | 1 |
| `OLA` (acronym) | 2 |
| "operational level agreement" (spelled) | **0** |
| security matrix | 1 |
| level of use | 1 |
| vendor management | 1 |
| training program | 1 |
| Subpart S | 2 |
| alarm ≠ status indicator | 1 |
| **floor management** | **0** — taught in notes |
| **document management** | **0** — taught in notes as "document-management" |

`.28` closed the bulk of the hole this morning. **Floor management and document management are taught and still untested**, and OLA appears only as an acronym, which a stem cannot rely on a learner expanding.

**Depth note:** M15 is the strongest module by bloom (5 analyze / 14 apply, only 3 remember). The sittings ledger flags one residual: `bank-m15-q154` still stems *"timeline, root cause, corrective actions"* singular, while the notes teach **contributing factors, plural**.

### CDCS-calc hole

Ops arithmetic is real but thin: **MTTR/MTBF composition** and **SLA credit computation against a measured availability window**. Both belong on `ms4j.1` (CDFOS/CDCS lane), not here. Named, not implemented.

---

## Module 02 — Data Centre Standards

### Public sources

| Source | URL | What it settles |
|---|---|---|
| ANSI/TIA-942-C (2024) | <https://www.belden.com/blog/introducing-ansi-tia-942-c-recent-updates-to-data-center-standards> · <https://tiaonline.org/products-and-services/tia942certification/ansi-tia-942-standard/> | Released April/May 2024; replaces 942-B; folds in the 942-B-1 edge addendum; adds AI / higher rack-density guidance and an **informative annex on liquid immersion cooling**. Confirms the course's `942-C (2024)` pin. |
| Uptime Tier Certification (three awards) | <https://uptimeinstitute.com/tier-certification> · <https://uptimeinstitute.com/tier-certification/construction> | Design documents → constructed facility → operational sustainability. **TCOS is described as applying to owners who have *previously received* TCCF** — the three are **sequential with a prerequisite**, and TCOS is the optional third. |
| ISO/IEC 30134 series | <https://www.future-tech.co.uk/introduction-the-iso-iec-30134-series-of-standardised-kpis/> · <https://www.iso.org/standard/30134-2> | **Eight KPI parts**, not three: -2 PUE · -3 REF · -4 ITEEsv · -5 ITEUsv · -6 **ERF** (Energy Reuse Factor) · -7 CER · -8 CUE · -9 WUE. Green Grid transferred PUE/WUE/CUE to ISO/IEC JTC1/SC39. **A 2026 edition of Part 2 (PUE) exists.** |
| EN 50600 / ISO/IEC 22237 | <https://www.hknow.de/en/planning/en50600/> · <https://www.techerati.com/features-hub/explaining-the-new-family-of-iso-data-centre-standards/> | Availability Classes 1–4 **plus separate Protection Classes**; German-language sources add granularity levels GN1–GN3. Confirms the course's three-noun lattice. |
| ASHRAE liquid W-classes | <https://www.upsite.com/blog/major-changes-to-ashraes-fifth-edition-of-thermal-guidelines-part-3-liquid-cooling-chapter-updates/> · <https://www.ashrae.org/file%20library/technical%20resources/bookstore/emergence-and-expansion-of-liquid-cooling-in-mainstream-data-centers_wp.pdf> | W17(←W1) / W27(←W2) / W32(←W3) / **W40 (new)** / W45(←W4) / W+(←W5). The number is the **upper facility-water supply limit in °C**; **all classes share a 2 °C lower limit**. Confirms `modules/09-cooling.md:187-196` and M02's loop-decision framing. |

### Taught-vs-2026 — three named gaps, all small

1. **ISO/IEC 30134 is named as three KPIs; the series has eight parts.** `modules/02-standards.md:21,195-197` says "PUE, WUE, CUE". The published series adds REF, ITEEsv, ITEUsv, **ERF**, CER. The module is right not to become an energy course, but "the three names you should be able to say" understates a series a 2026 RFP will cite. **ERF (Energy Reuse Factor, 30134-6) is distinct from the Green Grid's ERE** and the corpus names neither.
2. **The three plaques are taught as parallel; they are sequential.** `:104-114` presents TCCD / TCCF / TCOS as three certificates — correct and valuable — but Uptime's own material conditions TCOS on a prior TCCF. "Which plaque do you have, and did the one before it happen?" is a sharper RFP probe than "there are three."
3. **Acronym variance on the design plaque.** The module uses **TCCD**; public secondary sources also spell it **TCDD**. Pin the **full name** ("Tier Certification of Design Documents") rather than the acronym — the same edition-pinning discipline the module already teaches for TIA.

**Already correct, do not disturb:** the three-noun lattice (Rated ≠ Tier ≠ Availability Class) at `:52-60`; the 99.982% name-and-kill at `:116-127` with no replacement percentage offered; the `942-C (2024)` pin; EN 50600 as a first-class twin with separate Protection Classes; W-classes as a loop decision pointing at M09.

### Bank rigor — coverage good, depth thin

| Concept | Files |
|---|---|
| ISO/IEC 22237 | 12 |
| EN 50600 | 10 |
| Availability Class | 8 |
| three plaques (TCCD/TCCF/TCOS) | 3 |
| W-classes | 3 |
| 99.982 name-and-kill | 3 |
| Protection Class | 2 |
| TIA-942-C | 2 |
| **ISO/IEC 30134** | **0** |

**M02 carries one `analyze` item in forty-eight** — `m02-q215`, a vendor "proving" Availability Class 3 / Tier III with a UPS nameplate photo and a PUE dashboard. That item is the model; the module needs more of that shape and fewer name-recall items (10 `remember`).

Two concrete gaps: **ISO/IEC 30134 has zero items** despite being a stated module objective, and **TIA-942-C has two** despite edition-pinning being one of the module's sharpest teachings.

### CDCS-calc hole

**None.** Standards is a lattice-and-vocabulary domain; there is no arithmetic to hold back. Availability Class is a classification, not a computation — and inventing a calc here would re-create the nines-to-Tier crosswalk the module exists to kill.

---

## Follow-on beads — proposed titles only

Children of `bd-curriculum-truth-ebrr`. **Not filed by this wave**; a real leftover is named for each.

| Proposed title | Module | Leftover it closes |
|---|---|---|
| M01 capstone leftover: retire the three-bucket cartoon from PRACTICE Q4 and `mock40-q04` | M01 | `m01-q004.toml` approved and contradicting `m01-q210`; same `topic_ids`; also `practice/PRACTICE-EXAM.md:40-45`. Never beaded. |
| M15 bank leftover: floor management + document-management lifecycle items | M15 | Two of nine 2.1 headings taught with **0** bank files; OLA banked only as an acronym. |
| M15 item fix: `bank-m15-q154` root-cause singular → contributing factors plural | M15 | Stem contradicts the notes' plural framing. |
| M02 bank leftover: ISO/IEC 30134 KPI items + a second TIA-942-C edition-pin item | M02 | 30134 is a module objective with **0** files; 942-C has 2. |
| M02 depth: TCOS-requires-TCCF sequencing as an apply item | M02 | Three plaques taught as parallel; public material conditions TCOS on prior TCCF. |

**Deliberately not proposed:** anything that adds a nines-to-Tier crosswalk, any invented outage percentage, any calc item (those belong on `bd-epi-ecosystem-ms4j.1`), and any new epic.

---

## Wave verdict

**M01 prose: correct and externally corroborated by Uptime AOA 2026. M01 bank: one live self-contradiction, still unbeaded, ranked #1 three times.**
**M15 prose: complete. M15 bank: `.28` closed most of it; floor management and document management remain taught-but-untested.**
**M02 prose: the lattice is the strongest teaching in the corpus. M02 bank: broad coverage, one analyze item in 48, and ISO/IEC 30134 at zero.**

Corpus-wide, all **854** items are `qualitative_only`; every calc named above belongs to `ms4j.1` and none was implemented here.

*Wave 1 research ledger. No module prose, no README, no CHARTER, no `check.sh`, no `cargo`, no bead closed, no commit.*
