# WAVE 03 — site / floor / racks / water: M03 · M04 · M08 · M10

**Bead:** `bd-curriculum-truth-ebrr.29.3` · **Date:** 2026-08-17 · **Pane:** Claude, `cdcp` 0.1
**Scope:** public-source research + taught-vs-2026 gap + bank rigor + named CDCS-calc holes for Modules 03, 04, 08, 10.
**Not done here:** no module prose, no README, no `check.sh`, no CHARTER, no `cargo`, no bead closed, no pane-2 CDCS file touched, `WAVE-01.md` / `WAVE-02.md` unmodified.

> Completing this program does not certify anyone. 27/40 is a **CDCP study signal**, not a pass mark — and not a CDCS 45/60 or CDFOS 42/60 cut.
> Rated ≠ Tier ≠ Availability Class. No nines-to-Tier crosswalk. W-classes are **facility-water supply** classes. No outage, water or fire percentage is invented.

## Source grading, per `ATTRIBUTION-BAR.md`

The bar requires a **current public standard or primary technical body**, with edition and URL; *"vendor blogs and staging help pages are not enough"* and vendor application notes *"may illustrate a calc after a standard cite exists."*

This ledger marks every source **[PRIMARY]** or **[SECONDARY]**. Where only secondary material was found, that is stated as a **sourcing gap and a leftover bead**, not written up as fact. Two of the four modules below are in that condition, and it is the most useful finding in this wave.

---

## Counts, re-measured for this ledger

```bash
ls course-engine/bank/items/*.toml | wc -l                            # 854
grep -h '^status = ' course-engine/bank/items/*.toml | sort | uniq -c # 829 approved / 25 retired
for m in m03 m04 m08 m10; do ls course-engine/bank/items/*${m}* | wc -l; done   # 44 / 37 / 37 / 35
```

| Module | Items | analyze | apply | understand | remember |
|---|---|---|---|---|---|
| M03 site/building | 44 | 3 | 14 | 21 | 6 |
| M04 floor/ceiling | 37 | 3 | 11 | 14 | 9 |
| M08 racks | 37 | **2** | 10 | 18 | 7 |
| M10 water | 35 | 4 | 11 | 13 | 7 |

All 854 items remain `quantity_evidence = "qualitative_only"`.

---

## Module 03 — Location, Building & Construction

### Sources

| Grade | Source | URL |
|---|---|---|
| **[PRIMARY]** | ERCOT — Interconnection and Grid Analysis Update (Apr 2026), large-load process | <https://www.ercot.com/files/docs/2026/04/13/9-Interconnection-and-Grid-Analysis-Update.pdf> |
| **[PRIMARY]** | ANSI/TIA-942-C (2024) — site/space context; TIA's own standard page | <https://tiaonline.org/products-and-services/tia942certification/ansi-tia-942-standard/> |
| **[PRIMARY]** | EN 50600 / ISO/IEC 22237 — Availability **and separate Protection** Classes; the Protection Class axis is the siting-relevant one | <https://standards.iteh.ai/catalog/standards/clc/955e2fa0-4413-4b81-a9b6-e22d9797025c/en-50600-3-1-2026> |
| [SECONDARY] | Public reporting on Texas SB 6 / FERC large-load direction | illustrative only; not used for any claim below |

### Taught-vs-2026 — strong, one named hole

Measured in the live notes: `interconnect queue` **12** mentions, `financial security / queue position` **10**, `behind-the-meter` **5**. `ebrr.4` landed this and it holds — M03 is the best-updated siting module in the corpus.

**Hole: `curtailment` is 1 mention in the notes and 0 bank items.** Queue and BTM are taught as the *energization* story; curtailment is the *operating* story that follows — the site energized behind a queue deal or a BTM arrangement can still be told to shed. Module 15 lists a curtailment EOP, so the procedure exists downstream of a siting concept the siting module barely names.

### Bank rigor

| Concept | Files |
|---|---|
| flood | 17 |
| seismic | 6 |
| substation | 6 |
| interconnect queue | 3 |
| neocloud | 3 |
| water rights | 3 |
| AI factory | 2 |
| GPU colo | 2 |
| behind-the-meter | 2 |
| **curtailment** | **0** |

Classic siting (flood 17, seismic 6) is deep; the 2026 site-type vocabulary sits at 2–3 items each. `m03-q094` (flood/storm surge threatens plant yards and fuel, with "raised floor indoors" as a distractor) is the model near-miss and was drawn in sampled seeds.

### CDCS-calc hole

M03 maps to public CDCS heading **3. Building Considerations**. Named, not implemented: **structural floor loading in kPa / lb·ft⁻²** against a rack row weight, and **fuel storage autonomy** (tank volume ÷ consumption at load). A `cdcs-fuel-autonomy.toml` heading file already exists in the CDCS track; the *building-loading* calc does not.

---

## Module 04 — Raised Floor & Suspended Ceiling

### Sources — **sourcing gap, stated as such**

| Grade | Source | URL |
|---|---|---|
| **[PRIMARY]** | ANSI/TIA-942-C (2024) — data-centre context for floors/pathways | <https://tiaonline.org/products-and-services/tia942certification/ansi-tia-942-standard/> |
| **[PRIMARY, paywalled]** | **BS EN 12825** — raised access floors, classification by **ultimate load class / deflection class / safety factor / dimensional tolerance**. Catalogue entry only; full text is paid and is not in this repo | <https://standards.iteh.ai/catalog/standards/cen/> (catalogue search: EN 12825) |
| **[PRIMARY, paywalled]** | **ISO 22496** — access-floor product standard, named in the notes | ISO catalogue |
| **[PRIMARY, membership]** | **CISCA** *Recommended Test Procedures for Access Floors* — concentrated, ultimate, uniform, drop-impact, rolling load | <https://www.cisca.org/> |
| [SECONDARY] | Access-flooring trade pages describing the EN 12825 class-code structure and typical data-centre classes | **Illustrative only.** Not cited as clause text below. |

**This is the honest state:** the three product standards M04 depends on — EN 12825, ISO 22496, CISCA — are **paid or membership documents**, and the only freely readable descriptions of their class-code structure are vendor and trade pages. Under the attribution bar those cannot carry a numeric claim. **Do not write a floor-class item from a trade page.** The leftover is a sourcing bead, not a content bead.

### Taught-vs-2026

The notes now teach the two floor **types** — `04-floor-ceiling.md:8`, *"Distinguish the two main types of raised floors: stringered vs free-standing / stringerless"* — which was Pass 1's flagged thinness and `ebrr.5`'s target. **Closed in the notes.** EN 12825 is named at `:120` and `:414`, with deflection discussed conceptually at `:289`.

**What is not taught is the class-code structure itself** — that an EN 12825 classification is a *compound* of ultimate-load class, deflection class, safety factor, and tolerance, so "EN 12825 compliant" without the code is as soft as "per TIA-942" without an edition. That is the same edition-pinning discipline M02 teaches, applied to floors, and the module does not make the connection. **It cannot be written from current sources** (above) — hence a sourcing bead.

### Bank rigor — notes fixed, bank behind

| Concept | Files |
|---|---|
| plenum | 31 |
| psf / kPa / kN (as words) | 24 |
| slab | 9 |
| rolling load | 8 |
| concentrated load | 4 |
| CISCA | 3 |
| stringer | 3 |
| uniform load | 3 |
| **free-standing / stringerless** | **1** |
| **cementitious / wood-core** | **1** |
| **EN 12825** | **0** |
| **ISO 22496** | **0** |

The **types** distinction the notes now teach has **one** bank item on the free-standing side against three on stringer. Two named product standards have **zero** items. This is the Wave-1 shape again: prose corrected, bank not yet caught up.

Note `psf / kPa / kN` appears in 24 files as *vocabulary* while every item is `qualitative_only` — units are named, never computed.

### CDCS-calc hole

M04 maps to public CDCS heading **4. Advanced Raised Floor and Suspended Ceiling**. Named, not implemented: **concentrated vs uniform vs rolling load** against a stated panel class, and **point load from a populated rack** (rack mass ÷ caster or foot count) versus panel working load. **Blocked on sourcing** — the class tables are paywalled, so the calc cannot be re-derived from a public clause today. That blocker should be recorded on the CDCS lane, not silently worked around.

---

## Module 08 — Equipment Racks

### Sources

| Grade | Source | URL |
|---|---|---|
| **[PRIMARY, paywalled]** | **IEC 60297** series — mechanical structures, 482.6 mm (19″) rack dimensions | IEC catalogue |
| **[PRIMARY, paywalled]** | **EIA-310** — cabinet/rack panel and rail dimensions | ANSI/ECA catalogue |
| **[PRIMARY]** | **OCP Open Rack v3 (ORv3)** and the **UQD** (Universal Quick Disconnect) specification work — Open Compute Project publishes its specifications openly | <https://www.opencompute.org/> |
| **[PRIMARY]** | ASHRAE TC 9.9 thermal guidelines — inlet envelopes governing blanking / containment at the rack | <https://www.ashrae.org/technical-resources/bookstore/datacom-series> |
| [SECONDARY] | Connector-vendor pages describing ORv3 blind-mate quick connects and UQD v2.0 revision | **Illustrative only.** Used to identify the vocabulary to pull from OCP, not as the source of any claim. |

### Taught-vs-2026 — the entire 2026 open-rack layer is absent

The notes do teach a liquid-ready SKU: `08-racks.md:19` specifies *"a liquid-ready / GPU-hall cabinet SKU (typically 800 mm for hoses/manifolds, rear-door / RDHx…)"*, and `:48` gives 600 mm vs 800 mm outer widths. `ebrr.9` landed that.

**But the 2026 standards layer is not present at all.** Measured across notes **and** bank:

| Term | Notes | Bank |
|---|---|---|
| OCP / Open Compute | **0** | **0** |
| ORv3 | **0** | **0** |
| UQD / universal quick disconnect | **0** | **0** |
| blind-mate / BMQC | **0** | **0** |
| dry-break | **0** | **0** |

A 2026 GPU-hall rack conversation is conducted in OCP vocabulary — ORv3 as the open rack, UQD/BMQC as the coupling interface — and the course speaks only 19″ EIA-310 plus a width. This is the largest single vocabulary gap found across all three waves, and unlike M04 it is **not** blocked on sourcing: **OCP publishes its specifications publicly**, so the attribution bar can be met.

### Bank rigor — thinnest `analyze` count in the wave

| Concept | Files |
|---|---|
| blanking | 25 |
| rack PDU / zero-U | 19 |
| drip / leak tray | 6 |
| seismic | 6 |
| 800 mm | 4 |
| liquid-ready | 3 |
| manifold | 3 |
| **IEC 60297** | **1** |
| **EIA-310** | **1** |
| **quick disconnect / QDC** | **1** |
| **rear door** | **1** |
| **busway tap** | **1** |

**M08 has 2 `analyze` items in 37** — the lowest ratio of the four modules in this wave. The two rack **dimension standards** the module is built on have one item each. `m08-q212` (hybrid GPU hall, ops wants to drop blanking "because the hall is liquid" — keep it, ToR/spine stay air-cooled) is the module's best item and the sittings ledger flagged it as a keeper.

### CDCS-calc hole

M08 sits under public CDCS heading **4** and touches **7. Advanced Cooling**. Named, not implemented: **rack heat load → required airflow** at a stated ΔT, and **cabinet static/dynamic load** against floor class (shared blocker with M04). The airflow calc already has a CDCS heading file (`cdcs-airflow-cmh.toml`, `cdcs-sensible-delta-t.toml`); the rack-loading calc does not.

---

## Module 10 — Water Supply

### Sources

| Grade | Source | URL |
|---|---|---|
| **[PRIMARY]** | **ISO/IEC 30134-9** — Water Usage Effectiveness (WUE) as a standardised KPI; the series' catalogue entry (Part 2, PUE, has a **2026** edition) | <https://www.iso.org/standard/30134-2> · series overview: <https://www.future-tech.co.uk/introduction-the-iso-iec-30134-series-of-standardised-kpis/> |
| **[PRIMARY]** | EN 50600-4-x — the European KPI parts that pair with 30134 | <https://standards.iteh.ai/catalog/standards/clc/955e2fa0-4413-4b81-a9b6-e22d9797025c/en-50600-3-1-2026> (series entry) |
| **[PRIMARY]** | US EPA WaterReuse — reclaimed-water case study, Quincy WA | <https://www.epa.gov/waterreuse/water-reuse-case-study-quincy-washington> |
| **[PRIMARY]** | ASHRAE TC 9.9 — W-classes as **facility-water supply** limits, the loop water is supplied *to* | <https://www.ashrae.org/technical-resources/ai-data-center-framework/introduction-and-purpose> |
| [SECONDARY] | Operator and trade reporting on closed-loop / non-evaporative migration and WUE benchmark bands | **Illustrative only.** No benchmark number is imported (see below). |

**Explicitly not imported:** published WUE benchmark bands (e.g. "traditional ~1.8 L/kWh", "best-in-class <0.05 L/kWh") and hyperscale daily-consumption figures. They come from vendor and trade sources, they vary by boundary and climate, and M10 already teaches the correct posture for exactly this class of number. Adopting them would repeat the "20–40% of site energy" mistake `modules/09-cooling.md:34` exists to prevent. **If a WUE band is ever taught, it must come from ISO/IEC 30134-9 with its measurement boundary stated.**

### Taught-vs-2026 — one genuine 2026 shift, half-taught

The notes are strong: `closed-loop` **4** mentions, `reclaim` **5**, `water-stressed` **3**, and WUE with a `L/kWh` unit mention. The "liquid-cooled IT removes water risk" misconception is taught and killed.

**The 2026 shift the module has not absorbed:** `non-evaporative` is **0** mentions. The current design answer is not only "liquid IT still rejects heat at an evaporative plant" — it is that **non-evaporative closed-loop rejection is now actively chosen** for regulatory and siting reasons, which turns water from a pure efficiency topic into a **siting-permission** topic and links M10 to M03's queue/BTM/curtailment story. `rainwater` (0) is a secondary omission.

### Bank rigor

| Concept | Files |
|---|---|
| tank / on-site storage | 16 |
| cooling tower | 14 |
| contamination / boil-water | 9 |
| dry cooler / adiabatic | 7 |
| WUE | 4 |
| drought / scarcity | 4 |
| makeup water | 3 |
| water rights | 3 |
| **reclaim / reuse** | **1** |
| **Legionella** | **1** |
| **potable** | **0** |
| **closed-loop** | **0** |

**`closed-loop` is 0 items** while the notes mention it four times, and `reclaim` is taught five times with one item. Water-as-availability-path is well covered on the *failure* side (tanks 16, towers 14, contamination 9) and thin on the *2026 design-response* side.

### CDCS-calc hole

M10 maps to public CDCS heading **11. Data Centre Efficiency**. Named, not implemented: **WUE = site water ÷ IT energy** with its measurement boundary, and **makeup-water rate from tower evaporation and cycles of concentration**. No CDCS heading file covers water today.

---

## Follow-on beads — proposed titles only

Children of `bd-curriculum-truth-ebrr`. **Not filed by this wave.** No new epic. No calc item proposed for the CDCP bank.

| Proposed title | Module | Leftover |
|---|---|---|
| M08 notes + bank: OCP ORv3 / UQD as the 2026 open-rack and coupling vocabulary | M08 | OCP, ORv3, UQD, BMQC, dry-break = **0 notes / 0 bank**. Sourceable — OCP publishes publicly. |
| **Sourcing bead:** obtain citable access to EN 12825 / ISO 22496 / CISCA class tables, or record the gap | M04 | All three are paid/membership; class-code structure is only on trade pages, which the attribution bar rejects. Blocks any floor-class item **and** the CDCS floor-loading calc. |
| M04 bank leftover: floor-types items to match the taught stringered-vs-free-standing distinction | M04 | free-standing = 1 item vs stringer 3; EN 12825 and ISO 22496 = 0. Types are taught, barely tested. |
| M08 bank leftover: EIA-310 / IEC 60297 dimension items; raise `analyze` above 2 | M08 | Both dimension standards = 1 item each; 2 analyze in 37. |
| M10 notes + bank: non-evaporative closed-loop as a siting-permission answer | M10 | `non-evaporative` = 0 notes; `closed-loop` = 4 notes / **0 bank**; reclaim 5 notes / 1 bank. |
| M03 bank leftover: curtailment as the operating consequence of a queue/BTM deal | M03 | `curtailment` = 1 note / **0 bank**, while M15 carries a curtailment EOP. |

**Deliberately not proposed:** any floor-class or WUE numeric item sourced from a trade page; any calc item in the CDCP bank; any invented percentage; any new epic.

---

## Wave verdict

**M03:** best-updated siting module; queue and BTM taught deeply; **curtailment** is the one named hole (1 note, 0 items).
**M04:** notes closed by `ebrr.5` (floor types now taught); bank behind (free-standing 1, EN 12825 0, ISO 22496 0); and the **class-code structure cannot be written from public sources today** — a sourcing bead, not a content bead.
**M08:** the **entire OCP ORv3 / UQD layer is absent from notes and bank**, and unlike M04 it is publicly sourceable. Lowest `analyze` ratio in the wave (2 of 37); both rack dimension standards have one item each.
**M10:** failure-side coverage deep, 2026 design-response side thin — `closed-loop` 4 notes / **0 items**, `non-evaporative` absent. Published WUE bands deliberately **not** imported.

Across three waves the corpus remains **854** items, all `qualitative_only`. Every calc named in this ledger belongs to `bd-epi-ecosystem-ms4j.1`; none was implemented, and the M04 floor-loading calc is **blocked on sourcing**, which is recorded rather than worked around.

*Wave 3 research ledger. No module prose, no README, no CHARTER, no `check.sh`, no `cargo`, no bead closed, no commit, no pane-2 CDCS file touched.*
