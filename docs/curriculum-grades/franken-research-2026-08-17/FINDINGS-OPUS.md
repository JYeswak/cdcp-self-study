# FINDINGS — Opus lens (curriculum truth · 2026 floor · EPI suite · honesty · orals · standards lattice)

**Date:** Monday 17 August 2026 (America/Denver)
**Pane:** Claude Opus, NTM session `cdcp`, host joshs-brain. Live repo `/Users/josh/cdcp-self-study`.
**Mission:** Validate, contradict, or extend the BRIEF's starting gap register. Research only.
**Constraints honored:** no `cargo build/test/run`; no `modules/*.md` edit; no `CHARTER.md` edit; no `br`/`bd` close; no `ntm send --all`; no invented outage percentages; no proprietary EPI body text; no dump sites opened.

> Completing this program does not certify anyone.

---

## Headline

**The starting gap register is largely a snapshot of the pre-`.1–.27` tree.** `bd-curriculum-truth-ebrr` has children **`.1` through `.27` CLOSED** and only **`.28` open**. I re-ran the register against the live files rather than rubber-stamping it: **11 of ~22 register rows are already closed in the notes**, and one row (M06 mermaid) misreads a deliberate teaching exhibit as a defect.

The live defect has **moved from the notes to the bank and the chrome**. The notes are, on the evidence I could check externally, *accurate and 2026-current*. What has not caught up: the assembled item bank, the capstone practice exam, six lines of prose in README/index.html, and the ecosystem map.

I also found **four defects that are in no register and no bead**, three of which sit inside the honesty machinery this project exists to run.

---

## Ranked register

Severity: **CRITICAL** (teaches something false / breaks the honesty constitution) · **HIGH** (a Fluidstack-style oral fails tomorrow, or a claim outruns its evidence) · **MED** · **LOW**.

| # | Gap | Evidence | Sev | Beaded? | Recommend |
|---|---|---|---|---|---|
| 1 | **`27/40` is the *published EPI cut score*, while the README calls it "not affiliated with any official cut score."** | `README.md:403-405` vs <https://www.epi-ap.com/services/1/3/4/Certified_Data_Centre_Professional_(CDCP)> ("Passing Mark: 27 out of 40 questions"); EXIN "Pass Mark 68%" | **CRITICAL** | **No** | **new-bead** |
| 2 | **Practice capstone + bank still key the retired three-bucket cartoon.** `m01-q004.toml` (`id = "mock40-q04"`, `correct = "B"`, `status = "approved"`) keys "human error during change/maintenance"; `m01-q210.toml` makes that same claim the **wrong** answer. Both approved, both `topic_ids = ["m01-unavailability"]`. | `course-engine/bank/items/m01-q004.toml:2-13`; `m01-q210.toml:5-13`; `practice/PRACTICE-EXAM.md:40-45`; contradicted by `modules/01-mission-critical.md:409` | **CRITICAL** | **No** | **new-bead** |
| 3 | **README/CHARTER prose advertises `804 files / 779 approved`; the tree holds `846 / 821`, and the registry probe *knows* it.** The fact IDs are stale *names* whose needles assert 846/821 — so `claims-lint` resolves the marker while the sentence stays wrong. | `registries/doc-facts.toml:167-181` (probe needle `"bank_item_count": 846`) vs `README.md:20,48,70,119,133,387`, `CHARTER.md:208`; measured `ls bank/items/*.toml` = **846**, `status="approved"` = **821**, `retired` = **25** | **CRITICAL** | **No** | **new-bead** |
| 4 | **Zero bank coverage of all nine public 2.1 headings.** Grep of 846 item files: `service catalog` 0 · `operational level agreement` 0 · `OLA` 0 · `security matrix` 0 · `level of use` 0 · `floor management` 0 · `vendor management` 0 · `document management` 0 · `training program` 0. The 39 `m15` items are labelling / MOP / MTBF / MTTR / cleaning / spares / escort — the **pre-2.1** ops-adjacent set. | `course-engine/bank/items/*m15*.toml` stems; `modules/15-ops-adjacent.md:106-200` teaches the headings | **HIGH** | **Yes — `.28` open** | **keep** (`.28` is correctly scoped; add the stem inventory as evidence) |
| 5 | **`README.md:20` says "Fourteen modules of original writing" while `README.md:48` says "15 modules."** Same document, two counts. Map and `learn.html` are both correct at fifteen. | `README.md:20` vs `README.md:48`; `00-curriculum-map.md:1`; `web/learn.html:39` | **HIGH** | **No** (`.17` closed the map / CHARTER / learn.html, not this line) | **new-bead** (fold into #3) |
| 6 | **Hub card advertises the bar with no qualifier.** `index.html:88` — "study bar 27". `mock.html:40` is correct ("study signal / not a pass mark"). The register named the mock toolbar; the live leak is the hub. | `course-engine/web/index.html:88` vs `web/mock.html:40` | **HIGH** | **No** (`.17` covered `learn.html` count) | **new-bead** (fold into #1) |
| 7 | **Corpus is `qualitative_only` in all 846 items.** No 3φ, psychrometric, battery, or fault-current arithmetic anywhere. Syllabus names "single phase and three phase power" and "power sizing"; the bank tests the words. | `grep quantity_evidence` → `846 quantity_evidence = "qualitative_only"` | **HIGH** | **Yes — `bd-epi-ecosystem-ms4j.1`** | **keep** (this is the CDCS track's reason to exist) |
| 8 | **`epi-ecosystem-map.md` is missing four published EPI tracks** and does not use EPI's own five-group structure. Absent: **DCFC** (Data Centre Foundation Certificate), **CDCA** (Certified Data Center Associate), **CDLI** (Certified DCOS Lead Implementer), **CDLA** (Certified DCOS Lead Auditor). | <https://www.epi-ap.com/services/1/3/2/EPI_Data_Center_Training_Framework> vs `docs/curriculum-grades/epi-ecosystem-map.md:26-40` | **MED** | Partially (`bd-epi-ecosystem-ms4j` exists; these four are not children) | **new-bead** under `ms4j` |
| 9 | **The nines table also circulates badged to TIA `Rated-1..4`, not only to Uptime Tiers.** M02 name-and-kills the *Tier* form only. A 2026 RFP will hand you the Rated form. | `modules/02-standards.md:116-127` vs <https://thenetworkinstallers.com/blog/ansi-tia-942-c-standard/> and <https://www.score-grp.com/en/post/data-center-standards-iso-ansi-tia-942-and-tiers-in-2026-how-to-design-classify-and-operate-re> (both attach 99.671/99.741/99.982/99.995 to Rated-1..4) | **MED** | **No** | **rewrite-when-ACK** (extend the existing kill; do **not** add a crosswalk) |
| 10 | **EXIN names a third subject area — "Management" — alongside Facilities and Operations.** The corpus maps Facilities (14) + Operations (2.1) only. Unresolved whether this is a third area or EXIN's grouping of the same 85/15. | <https://www.exin.com/technologies-software/exin-epi-data-centre-management/certified-data-centre-professional/> vs `00-curriculum-map.md:3` | **MED** | **No** | **new-bead** (research-only; resolve from the public prep guide before any prose moves) |
| 11 | **`bank-m15-q154` still stems "root cause" singular** while M15 teaches contributing factors, plural. | `pass-practice-sittings-2026-08-17.md:347`; `modules/15-ops-adjacent.md:19` | **MED** | Adjacent to `.28` | **new-bead** or fold into `.28` |
| 12 | **One-line-as-isolation is still not an item type.** No item shows a one-line with no isolation point that leaves the load served and asks whether concurrent maintainability is real. Adjacent items exist (one-ATS SPOF, CDU this-loop). | `pass-practice-sittings-2026-08-17.md:422`; `modules/06-power.md:379` teaches the check in prose | **MED** | **No** | **new-bead** |
| 13 | **Assemble under-draws the hard cluster.** Catcher, W-class, one-ATS SPOF, scope-of-nines exist in bank and were absent from seeds 7 and partly 42. | `pass-practice-sittings-2026-08-17.md:436` (sittings 2–4) | **MED** | **No** | **new-bead** (stratify by objective, not only module) |
| 14 | **Cartoon distractors + lecture-dump keys make 40/40 cheap.** 287/287 across 13 sittings. `m09-q204` (recirculation vs bypass) is the near-miss model the rest of the bank does not use. | `pass-practice-sittings-2026-08-17.md:49-51,389-392` | **MED** | **No** | **new-bead** (distractor-quality pass) |
| 15 | **M12 in-note eight-pack is still the classic playbook** while notes and `m12-q226` moved to 855 / 9540A / room-vs-yard. | `pass-practice-sittings-2026-08-17.md:329`; `modules/12-fire.md:20,31` | **LOW** | `.26` closed (bank); in-note quiz not covered | **new-bead** (small) |

---

## Register rows I CONTRADICT (stale — do not re-open)

These were live on 15 Aug and are closed now. Re-filing them would be motion, not work.

| Register row | Live state 17 Aug | Evidence |
|---|---|---|
| "Map titled *14 Modules*" | **FALSE.** Title is `# Curriculum Map — 15 Modules`, and the header assigns 2.1 to M15 explicitly. | `00-curriculum-map.md:1,3,7`; `.17` closed |
| "CHARTER SRS contradiction" | **FIXED.** CHARTER now says "**Not spaced repetition** — no expanding interval, no forgetting curve… Calling it SRS overstates it." | `CHARTER.md:67,106`; `.17` closed |
| "learn.html *Fourteen EPI domains*" | **FIXED and now precise.** "Fifteen study modules… (fourteen facility domains plus 2.1 Operational Considerations)." | `web/learn.html:39`; `.17` closed |
| "Unsourced *most outages*" | **INVERTED.** The claim is now a named misconception to refuse, and a quiz **distractor** — M01 Q10 choice (a) and (c) are wrong, (b) is the key. | `modules/01-mission-critical.md:409,503-509` |
| "Unsourced 20–40% energy lines" | **INVERTED.** "**Do not recite 20–40% as law**," plus a misconception-table row. | `modules/09-cooling.md:34,367` |
| "W-classes missing" | **TAUGHT, and externally correct.** Full table with the 2 °C shared floor and W40 flagged as new in the 5th-ed rename. | `modules/09-cooling.md:187-196`, `modules/02-standards.md:18,185,353,394`; `.10`/`.24` closed |
| "NFPA 855 / UL 9540A missing" | **TAUGHT, and the subtle part is right:** 855 = installation standard, confirm adopted edition, "not automatically the law"; 9540A = test method, "**not a certification**," UL 9540 is the listing path. | `modules/06-power.md:238-239,618`; `modules/12-fire.md:20,31`; `.13`/`.26` closed |
| "Interconnect queue / BTM missing" | **TAUGHT** in M01, M03, M06, M15. | `grep -rln "interconnect queue\|behind-the-meter" modules/` → 4 files; `.4` closed |
| "M02 thin — Availability Class / EN 50600 / three plaques / 942-C unnamed" | **ALL FOUR NAMED.** Three-noun lattice table; TCCD/TCCF/TCOS as three plaques; `ANSI/TIA-942-C (2024)` pinned; EN 50600 as first-class twin with Availability Class 1–4 + separate Protection Classes. | `modules/02-standards.md:13-21,52-60,104-114,139-145`; `.1` closed |
| "M06 mermaid collapses A/B through one ATS" | **MISREAD.** M06 carries a **correct 2N drawing** (dual utility, dual ATS, dual UPS) at `:357-377`, followed by the one-ATS figure at `:385-397` as a **deliberate SPOF exhibit**, with the interview check "name the last common point. If the answer is 'the ATS,'… the sketch is not 2N." A drawable catcher bus follows at `:401`. | `modules/06-power.md:357-407`; `.7` closed |
| "M15 apply items unassembled / M15 = D" | **PARTLY STALE.** M15 holds 39 items — 14 `apply`, 4 `analyze`, 18 `understand`, 3 `remember`. Pass 1's grade **D** was against the pre-`.16` notes; the notes now teach all nine headings. The *live* defect is narrower and is #4: the 39 items do not test the 2.1 headings. | `bank/items/*m15*.toml` bloom counts + stems; `.16` closed, `.28` open |

---

## The four findings that are in no register and no bead

### F1 — `27/40` is the official cut score (CRITICAL)

`README.md:403-405` (FAQ):

> "**Why is 27/40 the bar?** It is the internal study signal this project uses… a threshold for your own review loop, not a pass mark, **and not affiliated with any official cut score**."

EPI's own public course page states: **"Passing Mark: 27 out of 40 questions."** EXIN's page states **"Pass Mark: 68%."**

The last clause of that FAQ answer is not true. The number *is* the published cut score. Two further wrinkles:

- 27/40 = **67.5%**, which is *below* EXIN's stated **68%**. Read strictly, EXIN's percentage implies 28/40. The two public sources do not agree with each other, and the repo's number matches the more permissive one.
- The honesty risk is not the disclaimer — it is a learner who hits 27 and reads it as "I would pass."

The rest of the chrome is genuinely good (`mock.html:40`, `results.html:40`, `index.html:44,56,111` all carry study-signal language). This is one sentence, and fixing it makes the constitution *stronger*, not weaker. Two lawful repairs: (a) state plainly that 27/40 is the number EPI publishes, that this tool uses the same bar, and that a score here is not an exam result; or (b) move the internal bar off the cut score so the disclaimer is true by construction. **Do not simply delete the sentence** — that trades a false claim for a silent one.

### F2 — the leftover cartoon is the only *active* contradiction in the corpus (CRITICAL)

`bank/items/m01-q004.toml`:

```toml
id = "mock40-q04"
stem = "Which factor is frequently cited as a major contributor to data-centre outages in industry analyses?"
correct = "B"   # "Human error during change/maintenance"
explanation = "Process and human error during change/maintenance work is a classic major outage driver."
bloom = "remember"
status = "approved"
topic_ids = ["m01-unavailability"]
```

`bank/items/m01-q210.toml` is `status = "approved"`, same `topic_ids`, and makes *the same proposition* a wrong answer — choice (d) "Human error is the unverifiable majority, so train harder and stop."

Both are drawable from one topic. `modules/01-mission-critical.md:409` lists "Most outages involve people and process" as an **unsourced majority — refuse it**. The stem's hedge ("frequently cited… in industry analyses") is defensible; **the explanation is not** — it asserts the claim flatly.

This survives in `practice/PRACTICE-EXAM.md:40-45` as Q4 of the capstone, which is the artifact a learner is most likely to treat as the exam. Beads `.2` and `.11` retired the cartoon from the M01 and M10 **notes**; no bead covers the **capstone or this item**. Pass 7 ranked it #1 and Pass-sittings ranked it #1 again. It is the oldest un-beaded defect in the corpus.

External note: the Uptime 2026 AOA public summary reports that **"failures to follow established procedures remain the leading driver of human error-related outages"** — human error framed as a *mechanism within* a category, and **"Power remains the leading cause of impactful outages… failures involving UPS systems, transfer switches and generators are dominant."** That is exactly what `m01-q210` teaches and exactly what `mock40-q04` contradicts. The corpus's 2026 framing is externally corroborated; the leftover item is not.

### F3 — the bank-count marker verifies a row, not a number (CRITICAL)

`registries/doc-facts.toml:167-181` registers `fact-bank-item-count-804` with probe needle `"bank_item_count": 846`, and `fact-bank-approved-count-779` with needle `"approved_item_count": 821`. The file's own comment (`:86-87`) says the live figures are "846 / 821 / ~20.5× pool size" and that "IDs are stable names; needles track the live units_index."

That is a defensible engineering choice for the **ID**. The defect is that the **prose reuses the stale number as its content**: README says "804 item files (779 approved)" in six places and CHARTER once. The marker `[[fact:fact-bank-item-count-804=yes]]` resolves — so the linter is green — while the sentence it is attached to states a number **42 files** and **42 approved items** off the truth.

This is the project's own stated failure mode, arriving through its own gate: *a claim that passes because the checker verifies the existence of a row rather than the agreement of a value.* I did **not** run `check.sh` (hard rule), so I cannot say whether a step catches this elsewhere; on the evidence in the registry file, the needle checks the ledger, not the prose.

Same class, one line up: `README.md:20` says "**Fourteen** modules of original writing" while `README.md:48` says "15 modules." The map, CHARTER, and `learn.html` are all correct at fifteen.

### F4 — the ecosystem map is missing four published EPI tracks (MED)

EPI's framework page publishes five groups. Measured against `epi-ecosystem-map.md:26-40`:

| EPI group | Published | In map? |
|---|---|---|
| Foundation | **DCFC**, **CDCA** | **Neither** |
| Design/Build | CDCP, CDCS, CDCE, CNCDP | all four ✓ |
| Maintenance/Operations | CDFOS, CDFOM, **CDESS** | ✓ (map files CDESS as its own track rather than under Maintenance/Operations) |
| Risk | CDRP, CDMS | ✓ |
| Standards/Compliance | **CDLI**, **CDLA**, CTDC, CTIA, **CTLA** | CTDC ✓, CTIA/CTLA ✓; **CDLI and CDLA absent** |

The map's `CTIA/CTLA` pairing is **correct** — CTLA is *Certified TIA-942 Lead Auditor* and is on EPI's page (I had expected CTEA and was wrong; verified before writing). The real gaps are **DCFC, CDCA, CDLI, CDLA**, plus a grouping that does not mirror EPI's own five-track structure.

Note for `.9`/`.10` scoping: **CDLI/CDLA are DCOS**, not TIA-942 — a different scheme from CTDC/CTIA/CTLA. Filing them under the TIA audit child would repeat the collapse M02 spends a page teaching learners to avoid.

---

## What is externally confirmed CORRECT (do not "fix")

Every one of these I checked against a public source, not against our own ledger.

1. **The 14 facility domains are exactly EPI's published outline**, in order, 1:1 with `00-curriculum-map.md:15-28`. The map is right.
2. **W-class teaching is accurate.** W17(←W1) / W27(←W2) / W32(←W3) / **W40 (new)** / W45(←W4) / W+(←W5); number = **upper** supply-fluid limit °C; **shared 2 °C lower limit**. `modules/09-cooling.md:187-196` matches the public ASHRAE/Upsite account exactly, including the detail most summaries drop (the shared floor).
3. **UL 9540A vs UL 9540 vs NFPA 855 is right, and it is the part the industry gets wrong.** 9540A is a *test method* ("we have 9540A" = we have test data); UL 9540 is the listing path; NFPA 855 is the installation standard whose **adopted edition** governs. `modules/06-power.md:238-239` states all three. NFPA 855 **2026** adds Annex G.11 large-scale fire test guidance; **UL 9540A 6th ed. published 13 March 2026** aligning Section 10 to it.
4. **`ANSI/TIA-942-C (2024)` pin is correct** — released April/May 2024, replaces 942-B, and adds AI/higher-density guidance plus an informative annex on liquid immersion cooling.
5. **EN 50600 / ISO/IEC 22237 with Availability Class 1–4 *and separate* Protection Classes** is correct; public sources also name granularity levels GN1–GN3. ISO/IEC 30134 as the KPI series (PUE/WUE/CUE) is correct.
6. **M15's `exam_weight_unknown = true` is well-calibrated.** EPI's *public course outline* lists only the 14 facility modules — 2.1 comes from the EXIN preparation-guide topic titles. Refusing to state a weight is the honest reading of the public record.
7. **"Training: Mandatory"** on EXIN's page corroborates the repo's standing claim that only the official exam *after authorised training* grants the credential.
8. **Dump hygiene holds.** Searching for the public syllabus surfaced dumpspedia, killexams, and OpenExamPrep "CDCP exam questions." I did **not** open them, and nothing in this corpus resembles them: `source_class = "original"`, 846/846 items.

---

## What a Fluidstack-style oral still fails on tomorrow

Not the notes. The notes would carry the candidate. These are the gaps a *drill-only* learner inherits:

1. **Any number.** 846/846 items are `qualitative_only`. Asked to size a feeder from a 40 kW rack — `I = P / (√3 · V · PF)` — there is nothing in the assessed corpus to have practised on. This is the CDCS gap and it is the single largest.
2. **Reading a one-line as a claim.** The oral is "here is the drawing; is concurrent maintainability real?" M06 teaches the check in prose (`:379`); no item makes the learner *perform* it.
3. **The 2.1 control set under pressure.** "What is in the catalog, and which OLA makes that SLA true?" is taught (`modules/15-ops-adjacent.md:136-160`) and tested **only** in the in-note self-check — zero assembled items.
4. **Seed luck.** A learner who drills mock-40s can miss catcher, W-class, one-ATS SPOF, and scope-of-nines entirely (measured across seeds 42/7/99).
5. **The Rated-badged nines table** (#9) — the form of the myth the course has not yet named.

---

## Recommended disposition

**new-bead (7):** F1 cut-score honesty (+#6 hub card) · F2 cartoon retirement across capstone+bank · F3 count drift (+#5 fourteen/fifteen) · F4 four missing EPI tracks under `ms4j` · #10 EXIN "Management" third area (research-only) · #12 one-line-as-isolation item type · #13/#14 assemble stratification + distractor quality.

**keep (2):** `.28` (correctly scoped — attach the stem inventory) · `bd-epi-ecosystem-ms4j.1` CDCS (owns the arithmetic gap).

**rewrite-when-ACK (1):** #9 — extend M02's existing name-and-kill to the Rated-badged form. **No crosswalk, no replacement percentage.**

**drop-as-folklore (11):** every row in the *"Register rows I CONTRADICT"* table. They are closed. Re-filing them would be motion.

---

## Reading of the arsenal (per BRIEF §"Arsenal first")

Read before writing. Nothing in this pass warranted a primitive: it is a documentation-and-register season with no code. Recorded so a later implementation pass does not re-derive — `bd-epi-ecosystem-ms4j.1`'s CDCS arithmetic lane should reach for **`fsci-stats`** (`wilson_ci`, `bootstrap_ci`, `ttest_rel`) rather than hand-rolling scoring statistics, and **note its measured footgun: those functions return `NaN` rather than `Err` on bad input — callers must check `is_nan()`.** `cdcp_metrics` already exists in-tree for the CDESS KPI lane (`Cargo.toml:26`). No dependency should be added on this pass.

---

## Files written / not written

| Path | Action |
|---|---|
| `docs/curriculum-grades/franken-research-2026-08-17/FINDINGS-OPUS.md` | **Written** (this file) |
| `.../SOURCES-OPUS.md` · `.../VERDICT-OPUS.md` | **Written** |
| `modules/*.md` · `CHARTER.md` · `practice/*` · `course-engine/bank/items/*` | Not edited |
| `br` / `bd` create or close | Not run |
| `cargo build` / `test` / `run` | Not run |
| Git commit | Not made |

*Pass ledger only. No module edits, no beads filed, no commit. Completing this program does not certify anyone.*
