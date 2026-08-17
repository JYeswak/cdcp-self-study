# WAVE 04 — remainder: M05 · M07 · M11 · M13 · M14

**Bead:** `bd-curriculum-truth-ebrr.29.4` · **Date:** 2026-08-17 · **Pane:** Claude, `cdcp` 0.1
**Scope:** public-source research + taught-vs-2026 gap + bank rigor + named CDCS-calc holes for Modules 05 (lighting), 07 (EMF), 11 (network), 13 (security), 14 (auxiliary).
**Not done here:** no module prose, no README, no `check.sh`, no CHARTER, no `cargo`, no bead closed, no pane-2 CDCS file touched, Waves 01–03 and `SYLLABUS-GAP.md` unmodified.

> Completing this program does not certify anyone. **27/40 is a CDCP study signal**, not a pass mark, and not a CDCS 45/60 or CDFOS 42/60 cut.
> Rated ≠ Tier ≠ Availability Class. No nines-to-Tier crosswalk. No exposure limit, loss budget, or lux figure is invented.

## Source grading per `ATTRIBUTION-BAR.md`

Every source is **[PRIMARY]** or **[SECONDARY]**. The bar's test — *"the numeric or one-line claim is re-derivable from that source"* — is applied per claim, not per module.

**Wave 4 is the cheapest wave to source.** Unlike Wave 3's floor standards (EN 12825 / ISO 22496 / CISCA, all paid or membership), **ICNIRP and OSHA publish their full text free**. Two of the five modules below can meet the bar today at zero cost, which makes their gaps actionable rather than blocked.

---

## A measurement correction, stated first

Naive `grep -i` over the bank inflated four counts badly, and the corrected numbers change the M11 finding materially:

| Term | Naive `grep -ril` | **Word-boundary** | What inflated it |
|---|---|---|---|
| `RoCE` | 57 | **2** | matches "p**roce**ss" |
| `MPO` | 185 | **3** | matches "com**po**nent", "i**mpo**rtant" |
| `EMS` | 142 | **9** | matches "syst**ems**", "it**ems**" |
| `CDU` | 11 | **8** | — |

Every count below is word-boundary measured. A substring count presented as coverage would itself be an unattributed claim.

```bash
for t in InfiniBand '\bRoCE\b' '\bMPO\b' '\bEMS\b' '\bCDU\b'; do
  grep -rl "$t" course-engine/bank/items/ | wc -l; done      # 1 / 2 / 3 / 9 / 8
```

---

## Counts, re-measured

```bash
ls course-engine/bank/items/*.toml | wc -l                            # 854 (829 approved / 25 retired)
for m in m05 m07 m11 m13 m14; do ls course-engine/bank/items/*${m}* | wc -l; done   # 31 / 32 / 74 / 48 / 46
```

| Module | Items | analyze | apply | evaluate | understand | remember |
|---|---|---|---|---|---|---|
| M05 lighting | 31 | 2 | 9 | — | 12 | 8 |
| M07 EMF | 32 | 3 | 7 | 1 | 11 | 10 |
| M11 network | **74** | 5 | 26 | 1 | 30 | 12 |
| M13 security | 48 | 5 | 15 | — | 18 | 10 |
| M14 auxiliary | 46 | **6** | 14 | — | 18 | 8 |

M14 has the **highest analyze count in the corpus** (6 of 46). M05 and M07 are the two smallest modules.

---

## Module 05 — Light

### Sources

| Grade | Source | URL |
|---|---|---|
| **[PRIMARY, paywalled]** | **EN 12464-1** — light and lighting of work places, indoor | CEN catalogue |
| **[PRIMARY, paywalled]** | **EN 1838** — lighting applications, emergency lighting | CEN catalogue |
| **[PRIMARY, free]** | **NFPA 101** *Life Safety Code* — free read-only access via NFPA's public portal | <https://www.nfpa.org/codes-and-standards/nfpa-101-standard-development/101> |
| **[PRIMARY, free]** | **NFPA 70 (NEC) Article 700** — emergency systems; same free-access portal | <https://www.nfpa.org/codes-and-standards/nfpa-70-standard-development/70> |

**Sourcing note:** NFPA offers free read-only access to its codes; EN 12464-1 and EN 1838 do not. So the *emergency-lighting* half of M05 is sourceable to primary text and the *workplace-illuminance* half is not — the same split Wave 3 found for floors, but only partial here.

### Taught-vs-2026

M05 is not a 2026-pressure domain and the notes say so implicitly. Measured: `lux/footcandle` 26 mentions, `emergency light` 28, `maintained/non-maintained` 11, `central battery/inverter` 19, `glare` 10. The module already teaches the two things a data-centre audit actually turns on — **IT UPS is not code emergency lighting**, and **containment/busway shadowing**.

**No 2026 gap found.** Pass 2 graded M05 an **A** on the 2026 floor and that holds. This module should not be grown for its own sake.

### Bank rigor

| Concept | Notes | Bank |
|---|---|---|
| emergency light | 28 | 19 |
| lux / footcandle | 26 | 20 |
| central battery / inverter | 19 | 12 |
| maintained / non-maintained | 11 | 9 |
| glare | 10 | 7 |
| uniformity | 6 | **1** |
| **EN 12464** | 2 | **0** |
| **EN 1838** | 5 | **0** |
| **NFPA 101** | 4 | **0** |
| containment shadow | 1 | **0** |

**All three named lighting standards have zero bank items** — and unlike M04, one of them (NFPA 101) is freely readable, so an item citing it is writable today. `uniformity` is taught six times and tested once.

**Cartoon risk:** 8 of 31 items are `remember`, 2 `analyze`. The sittings ledger did not sample M05; its lux-band items are the classic "order-of-magnitude, not the exam answer" shape the notes themselves warn about.

### CDCS-calc hole

M05 has **no dedicated CDCS heading** — lighting sits inside heading **3 (Building Considerations)**. Named, not implemented: **emergency-lighting autonomy** (battery capacity against a stated duration) and **illuminance uniformity ratio** (min ÷ average). Both are small; neither justifies a heading file on its own.

---

## Module 07 — Electro Magnetic Fields

### Sources

| Grade | Source | URL |
|---|---|---|
| **[PRIMARY, free full text]** | **ICNIRP 2010** — Guidelines for limiting exposure to time-varying electric and magnetic fields (1 Hz – 100 kHz). **This is the data-centre-relevant document** | <https://www.icnirp.org/en/frequencies/low-frequency/index.html> |
| **[PRIMARY, free full text]** | **ICNIRP 2020** — Guidelines for limiting exposure to EMF (100 kHz – 300 GHz), RF; updates the RF part of ICNIRP 1998 and the 100 kHz–10 MHz part of ICNIRP 2010 | <https://www.icnirp.org/cms/upload/publications/ICNIRPrfgdl2020.pdf> · <https://www.icnirp.org/en/differences.html> |
| **[PRIMARY, paywalled]** | **IEEE C95.1** — safety levels with respect to human exposure to RF EMF | IEEE catalogue |
| **[PRIMARY, paywalled]** | **IEC 61000** series — EMC | IEC catalogue |

### Taught-vs-2026 — one precise, cheap fix

The notes cite **ICNIRP** five times and IEEE C95.1 three times, with the right posture: *"revisions evolve — cite the current edition"* (`07-emf.md:105`) and *"point to primary documents by name"* (`:145`). The health-vs-EMC separation at `:26` is correct and is the distinction most sources blur.

**The gap: the notes never say which ICNIRP document.** ICNIRP publishes two, split by frequency — **2010 for low frequency (1 Hz – 100 kHz)** and **2020 for RF (100 kHz – 300 GHz)**. A data centre's EMF exposure question is overwhelmingly **power-frequency 50/60 Hz magnetic field** from busway, transformers and UPS rooms, which is squarely **ICNIRP 2010** territory. Citing "ICNIRP" unqualified is exactly the soft citation M02 teaches learners to reject in *"per TIA-942"* without an edition — and the module already teaches that discipline for its own standards.

**This is the cheapest correction in the wave:** ICNIRP's full text is free, so the fix is naming the right document, not acquiring one.

**Density-scaled EMF — the named 2026 lens — is thin, as Pass 2 recorded.** Measured: `busway/busbar` 8 notes, `multi-kA / kiloamp` **2 notes / 0 bank**, `skin depth` **1 note / 0 bank**, `separation distance` **0 / 0**. The physics does not change with density; the *magnitude* does, and no item scales the oral to a 2026 GPU hall's bus currents.

### Bank rigor

| Concept | Notes | Bank |
|---|---|---|
| shielding | 11 | 12 |
| busway / busbar | 8 | 19 |
| HEMP / E1-E3 | 15 | **1** |
| ICNIRP | 5 | **1** |
| IEC 61000 | 4 | **1** |
| mu-metal / permeability | 2 | 2 |
| **IEEE C95.1** | 3 | **0** |
| **multi-kA** | 2 | **0** |
| **skin depth** | 1 | **0** |

**HEMP is taught fifteen times and tested once.** `m07-q046` (ordinary aluminium is a poor low-frequency magnetic shield — μ and skin depth) was drawn in the sittings and graded **STRONG science**, rare in this bank. It is the model; there is one of it.

### CDCS-calc hole

M07 maps to public CDCS heading **6 (Advanced Electro Magnetic Fields)**; `cdcs-emf-attenuation.toml` exists in the track. Named, not implemented at CDCP depth: **field falloff with distance** from a conductor, and **shield attenuation in dB** with skin depth as the frequency-dependent term. The CDCS heading file covers the attenuation half; falloff-with-distance is the one a facilities walkthrough actually uses.

---

## Module 11 — Designing a Scalable Network Infrastructure

### Sources

| Grade | Source | URL |
|---|---|---|
| **[PRIMARY]** | **ANSI/TIA-942-C (2024)** — cabling topology and the MDA/HDA/ZDA/EDA space model; TIA's standard page | <https://tiaonline.org/products-and-services/tia942certification/ansi-tia-942-standard/> |
| **[PRIMARY, paywalled]** | **ISO/IEC 11801** series — generic cabling | ISO catalogue |
| **[PRIMARY]** | **IEEE 802.3** — the Ethernet standard defining 400G/800G PHYs and their reach; IEEE publishes 802 standards free via GET after a delay | <https://standards.ieee.org/ieee/802.3/> |
| **[PRIMARY]** | **EN 50600** — pathway/space redundancy alongside TIA | <https://standards.iteh.ai/catalog/standards/clc/955e2fa0-4413-4b81-a9b6-e22d9797025c/en-50600-3-1-2026> |

### Taught-vs-2026 — notes ahead of bank by a wide margin

`ebrr.12` landed the GPU-fabric lens and the **notes teach it well**: `east-west` 9 mentions, `InfiniBand/RoCE` 9, `400G/800G` 8, `training job` 7, `spine-leaf` 6. The framing Pass 2 asked for — *one fabric cut = one training job*, fabric as an availability path equal to power — is present.

**The bank did not follow.** Word-boundary measured:

| Concept | Notes | Bank |
|---|---|---|
| MDA / HDA / EDA | 37 | 13 |
| dark fibre / wave | 5 | 10 |
| loss budget | 2 | 4 |
| **east-west** | 9 | **2** |
| **400G / 800G** | 8 | **2** |
| **training job** | 7 | **2** |
| **RoCE** | 9 | **2** |
| **InfiniBand** | 9 | **1** |
| **MPO** | 13 | **3** |
| **OM4 / OM5 / OS2** | 6 | **1** |
| **spine-leaf** | 6 | **0** |
| **duct bank / manhole** | 6 | **0** |

**M11 is the largest module in this wave (74 items, 26 apply) and its entire 2026 layer is 1–2 items per concept.** The classic space model (MDA/HDA/EDA, 13 items) outweighs the GPU-fabric layer roughly six to one. `spine-leaf` — taught six times — has **zero** items.

`duct bank / manhole` at **0** is notable: the *shared-manhole* trap ("two carriers ≠ diverse OSP") is one of the module's best teachings and one of the sittings' STRONG items conceptually, yet the physical vocabulary is untested.

### CDCS-calc hole

M11 maps directly to public CDCS heading **9 (Designing and Installing Scalable Network Cabling Systems)** — which `SYLLABUS-GAP.md` records as having **no calc file at all**. Named, not implemented: **optical loss budget** (connector + splice + fibre attenuation against a PHY's link budget) and **channel length limits by media class and application**, both re-derivable from IEEE 802.3 and the cabling standards. This is the single most obviously missing CDCS heading with a clean public source.

---

## Module 13 — Physical Security & Safety

### Sources

| Grade | Source | URL |
|---|---|---|
| **[PRIMARY, free full text]** | **29 CFR 1910 Subpart S** — electrical; **§1910.333** selection and use of work practices | <https://www.osha.gov/laws-regs/regulations/standardnumber/1910/1910.333> |
| **[PRIMARY, free full text]** | **29 CFR 1910.147** — the control of hazardous energy (lockout/tagout) | <https://www.osha.gov/laws-regs/regulations/standardnumber/1910/1910.147> |
| **[PRIMARY, free]** | **OSHA eTool** — the explicit relationship between 1910.147, 1910.269 and 1910.333 | <https://www.osha.gov/etools/lockout-tagout/hot-topics/relationship-electric-power/lockout-tagout-selection-work-practices-standard> |
| **[PRIMARY, paywalled]** | **NFPA 70E** — electrical safety in the workplace (arc flash) | NFPA catalogue (free read-only access available) |
| **[PRIMARY]** | **EN 50600-2-5** — security systems, the European axis pairing with Protection Classes | CENELEC catalogue |

### Taught-vs-2026 — the LOTO pointer is the finding

Security proper is strong: `mantrap/airlock` 22 notes / 11 items, `anti-passback` 7/5, `fail-safe vs fail-secure` 7/2, `EPO` 7/4. Pass 2 graded M13 an **A**; that holds.

**The safety half carries a real defect.** `modules/13-security.md:197` teaches LOTO generically — *"OSHA-style (US) / equivalent procedures to de-energize equipment before work"* — and `:427` points at *"OSHA lockout/tagout and electrical safety overview pages."* Neither names which regulation governs which hazard.

OSHA's own eTool is explicit and free: **1910.147 does not cover exposure to electrical hazards from work on, near, or with conductors or equipment in electric-utilization installations** — that is Subpart S, and **§1910.333 is the primary standard for electrical work**. Subpart S also **lacks the "exclusive control" exception** 1910.147 has. OSHA will accept a single 1910.147-conforming program **only if** it also meets specified 1910.333 provisions.

For a data centre this is not academic: **isolating a UPS or switchgear is Subpart S, not 1910.147.** `modules/15-ops-adjacent.md:641` already states it correctly — *"Subpart S on switchgear / UPS; 1910.147 on machines"* — and `:686` cites `1910.147(f)(2)` for the contractor clause. **M13 has the weaker version of a distinction M15 already teaches**, and M13 is where a learner meets LOTO first.

Measured: `1910.147` **0 notes / 1 item**; `Subpart S` **1 note / 2 items**; `arc flash` **0 / 0**; `70E` **0 / 0**; `OSDP` 3 notes / **0 items**.

**`arc flash` at 0/0 is a genuine absence** — the curriculum map's M13 objective names "arc flash boundaries (conceptual)" and neither the notes nor the bank deliver it.

### CDCS-calc hole

M13 has no CDCS heading (security sits under CDFOS **3**). **No CDCP-depth calc is appropriate** — arc-flash incident energy is licensed-engineer work and the module correctly teaches literate collaboration, not calculation. **Naming a calc hole here would be a mistake**; recorded deliberately as *none*.

---

## Module 14 — Auxiliary Systems

### Sources

| Grade | Source | URL |
|---|---|---|
| **[PRIMARY]** | **EN 50600-2-5 / -3-1** — management and operational processes, monitoring | <https://standards.iteh.ai/catalog/standards/clc/955e2fa0-4413-4b81-a9b6-e22d9797025c/en-50600-3-1-2026> |
| **[PRIMARY]** | **ISO/IEC 30134** series — the KPI set a DCIM reports against | <https://www.future-tech.co.uk/introduction-the-iso-iec-30134-series-of-standardised-kpis/> |
| **[PRIMARY]** | **NFPA 72** — fire alarm and signaling, for the FACP boundary the module draws | <https://www.nfpa.org/codes-and-standards/nfpa-72-standard-development/72> (free read-only access) |
| **[SECONDARY]** | Vendor CDU / leak-detection literature | **Illustrative only.** No numeric claim taken. |

**Honest note:** BMS / EMS / **DCIM have no governing standard.** The module's three-way split is industry vocabulary, not a normative one, and no primary source can be cited for the taxonomy itself. That is worth saying in the notes rather than leaving a reader to assume a standard exists.

### Taught-vs-2026 — the CDU-loop leak landed

`ebrr.15` closed this and it holds. Measured: `CDU` **19 notes / 8 items**, `leak detection / leak rope` 9/10, `rate-of-rise` **9 notes / 2 items**, `secondary loop` 5/2, `alarm fatigue` 5/4, `sensor offline is a fault` 2/1.

**M14 has the highest `analyze` count in the corpus (6 of 46)** and the sittings drew `bank-m14-q125` — correlate utility-fail + ATS transfer + UPS-on-battery into one incident instead of an alarm storm — graded **STRONG ops**, and noted as closer to M15's alarm≠status claim than anything tagged M15 in that seed.

**Residual gap: `rate-of-rise` is taught nine times and tested twice.** For a GPU row the seconds-scale thermal rate-of-rise is the alarm that matters, and it is the thinnest-tested of M14's 2026 additions. `flow / pressure` telemetry for the secondary loop is taught (20 mentions) but the word-boundary bank count is unreliable for such generic terms and is **not cited here**.

### CDCS-calc hole

M14 maps loosely to public CDCS heading **11 (Data Centre Efficiency)** for the KPI half. Named, not implemented: **rate-of-rise in °C/min** from a step load and room thermal mass, and **alarm-count rationalisation ratios**. Neither is a classic CDCS calc; the efficiency KPI calc (PUE/WUE boundaries) is the one that belongs on the track, and `SYLLABUS-GAP.md` records heading 11 as having no calc file.

---

## Follow-on beads — proposed titles only

Children of `bd-curriculum-truth-ebrr`. **Not filed by this wave.** No new epic. No calc item proposed for the CDCP bank.

| Proposed title | Module | Leftover |
|---|---|---|
| M13 notes: name Subpart S / §1910.333 vs 1910.147 where LOTO is first taught | M13 | `:197` teaches LOTO generically; M15 `:641` already has the correct split. OSHA text is free — no sourcing blocker. |
| M13 bank + notes: arc-flash boundary as a conceptual objective | M13 | `arc flash` and `70E` are **0 notes / 0 bank**, though the curriculum map names the objective. |
| M11 bank leftover: GPU-fabric items to match the taught lens | M11 | spine-leaf **0**, InfiniBand 1, east-west 2, 400G/800G 2, training job 2, duct bank/manhole **0** — against 74 items and 37 MDA/HDA/EDA mentions. |
| M07 notes: pin **ICNIRP 2010** (low frequency) as the data-centre document | M07 | Notes cite "ICNIRP" unqualified; ICNIRP publishes 2010 (1 Hz–100 kHz) and 2020 (RF) and full text is free. Same edition-pinning discipline M02 teaches. |
| M07 bank leftover: HEMP and density-scaled busway items | M07 | HEMP 15 notes / **1** item; multi-kA and skin depth **0** items. |
| M05 bank leftover: emergency-lighting items citing NFPA 101 / NEC 700 | M05 | EN 12464, EN 1838, NFPA 101 all **0** items; NFPA offers free read-only access, so NFPA-cited items are writable now. |
| M14 bank leftover: seconds-scale rate-of-rise items for GPU rows | M14 | `rate-of-rise` 9 notes / **2** items. |

**Deliberately not proposed:** an arc-flash *calculation* item at CDCP depth (licensed-engineer work — the module's "literate collaboration" posture is correct); any item citing EN 12464-1 or EN 1838 clause numbers while those remain paywalled; any invented lux band, exposure limit or loss budget.

---

## Wave verdict

**M05:** no 2026 gap; Pass 2's **A** holds. All three named lighting standards have **0** items, but NFPA 101 is free, so the fix is writable today.
**M07:** posture correct, **edition unpinned** — "ICNIRP" needs to become "ICNIRP 2010" for the low-frequency case. Cheapest correction in the wave; source is free. HEMP 15 notes / 1 item.
**M11:** **the widest notes-to-bank gap found in any wave.** The GPU-fabric lens is taught in 6–9 mentions per concept and tested at 1–2 items, with `spine-leaf` and `duct bank/manhole` at **zero**. Also maps to CDCS heading 9, which has no calc file and a clean public source (IEEE 802.3).
**M13:** security strong; **the LOTO pointer is weaker in M13 than in M15**, and OSHA's free text settles it. `arc flash` is 0/0 against a stated objective.
**M14:** best `analyze` ratio in the corpus; CDU-loop leak landed; `rate-of-rise` is the thin one at 9 notes / 2 items. BMS/EMS/DCIM has **no governing standard** and the notes should say so.

Corpus-wide: **854** items, all `qualitative_only`. Every calc named across four waves belongs to `bd-epi-ecosystem-ms4j.1`; none implemented. **M13 is recorded as having no appropriate CDCP-depth calc** — an explicit *none*, not an oversight.

*Wave 4 research ledger. No module prose, no README, no CHARTER, no `check.sh`, no `cargo`, no bead filed or closed, no commit, no pane-2 CDCS file touched.*
