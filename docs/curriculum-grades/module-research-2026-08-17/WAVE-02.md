# WAVE 02 — plant research: M06 · M09 · M12

**Bead:** `bd-curriculum-truth-ebrr.29.2` · **Date:** 2026-08-17 · **Pane:** Claude, `cdcp` 0.1
**Scope:** public-source research + taught-vs-2026 gap + bank rigor + named CDCS-calc holes for Modules 06 (power), 09 (cooling), 12 (fire).
**Not done here:** no module prose, no README, no `check.sh`, no CHARTER, no `cargo`, no bead closed, `WAVE-01.md` not rewritten.

> Completing this program does not certify anyone. 27/40 is a study signal, not a pass mark.
> Rated ≠ Tier ≠ Availability Class. No nines-to-Tier crosswalk; no 99.982% = Tier III.
> W-classes are **facility-water supply** classes, not marketing labels. No outage percentage is invented.

**Calc discipline:** every formula below is **named as a hole and left unimplemented.** Calcs belong to `bd-epi-ecosystem-ms4j.1` (CDCS lane), not to this bead and not to the CDCP bank.

---

## Counts, re-measured for this ledger

```bash
ls course-engine/bank/items/*.toml | wc -l                              # 854
grep -h '^status = ' course-engine/bank/items/*.toml | sort | uniq -c   # 829 approved / 25 retired
for m in m06 m09 m12; do ls course-engine/bank/items/*${m}* | wc -l; done   # 143 / 125 / 65
grep -rho 'quantity_evidence = "[^"]*"' course-engine/bank/items/ | sort | uniq -c
#   854 quantity_evidence = "qualitative_only"
```

| Module | Items | analyze | apply | evaluate | understand | remember |
|---|---|---|---|---|---|---|
| M06 power | **143** | 9 | 50 | 1 | 64 | 19 |
| M09 cooling | **125** | 5 | 42 | 1 | 59 | 18 |
| M12 fire | **65** | 3 | 17 | — | 30 | **15** |

M06 and M09 are the two largest modules in the corpus and carry the most `apply` items. **M12 is less than half their size and the most recall-heavy of the three** (15 of 65 `remember`). Gate numbers are receipt-enforced and not restated here.

---

## Module 06 — Power Infrastructure

### Public sources

| Source | URL | What it settles |
|---|---|---|
| UL Solutions — UL 9540 vs UL 9540A | <https://www.ul.com/services/ul-9540a-test-method> · <https://sustainableenergyaction.org/resources/informational-bulletin-on-the-ul-9540-safety-standard-and-the-ul-9540a-test-method-2/> | **UL 9540** is the product **safety standard** producing a *Listed* status. **UL 9540A is a test method, NOT a certification — "there is no 'listing' that results from performing UL 9540A testing."** UL 9540 *references* 9540A, and acceptable 9540A results are needed for portions of 9540 — but **acceptable 9540A results do not automatically mean UL 9540 listing.** |
| NFPA 855 (2026) + UL 9540A 6th ed. | <https://www.ul.com/thecodeauthority/knowledge/understanding-UL-9540A-NFPA-855> · <https://www.telgian.com/nfpa-855-changes-in-2026/> · <https://www.mayfield.energy/technical-articles/the-6th-edition-of-ul-9540a-is-here/> | NFPA 855 **2026** adds **Annex G.11** large-scale fire test (LSFT) guidance for BESS-to-BESS spread. **UL 9540A 6th edition published 13 March 2026**, revising Section 10 to an LSFT method aligned to Annex G.11. 9540A is the only consensus standard NFPA 855 cites for LSFT. |
| ERCOT large-load interconnection | <https://www.ercot.com/files/docs/2026/04/13/9-Interconnection-and-Grid-Analysis-Update.pdf> | Interconnect queue as a first-class 2026 site risk. Cited as a public process document; no share or percentage imported. |

### Taught-vs-2026 — the module is current

`modules/06-power.md:238-239` states the 855/9540A split correctly, **including the part the industry gets wrong**: 855 is the *installation* standard whose **adopted edition** governs and is "not automatically the law"; 9540A means "we have test data", is **"not a certification"**, and **"UL 9540 is the listing path."** Catcher and distributed-redundant are drawable (`:399-407`), the 2N figure and the deliberate one-ATS SPOF exhibit sit together at `:357-397`, and interconnect queue / BTM are taught.

**No 2026 prose gap found.** `ebrr.7` closed this module and the close holds.

### Bank rigor — broad, and thin in two named places

| Concept | Files |
|---|---|
| BESS | 12 |
| thermography | 6 |
| catcher | 5 |
| N+2 | 5 |
| isolated-redundant | 3 |
| one-ATS / single-ATS SPOF | 3 |
| NFPA 855 | 3 |
| UL 9540A | 3 |
| interconnect queue | 3 |
| behind-the-meter | 2 |
| **isolation transformer** | **2** |
| **distributed-redundant** | **1** |
| **UL 9540 (the listing, bare)** | **0** |

Two real gaps. **`distributed-redundant` has a single item** while `catcher` has five, though the notes teach them as a contrast pair — a learner can be tested on one side of a distinction and never the other. **`isolation transformer` has two**, and Pass 1 already graded it the thinnest *taught* sub-heading in the public power list.

**The sharpest gap is `UL 9540` at zero.** The distinction the module teaches — listing vs test method — is exactly what the public sources say vendors blur, and **nothing in the bank makes a learner perform it.** An item of the shape *"a vendor sends 9540A test results as proof of listing"* is the natural apply item and does not exist.

**Cartoon-vs-exam-depth:** M06 is the strongest module by volume (50 `apply`, 9 `analyze`). The 2026-08-17 sittings drew good ones — `m06-q259` (N+2 ≠ Tier IV), `m06-q235` (PUE improved by shedding redundancy) — but recorded that seeds 42/7/99 **missed the catcher and one-ATS clusters entirely**. Coverage exists; sampling does not guarantee it.

### CDCS-calc hole — named, not implemented

All 854 items are `qualitative_only`, so no power item computes anything. For `ms4j.1`:

- **Three-phase current from load:** `I = P / (√3 · V_LL · PF)`; line-to-line vs line-to-neutral.
- **kW vs kVA and power factor**, plus the 80% continuous-load derate as *code-dependent*, not a universal.
- **Battery autonomy:** `Ah × V × η` against a kW load for a stated runtime.
- **Transformer / PDU sizing** and fault-current / bonding arithmetic.

The syllabus names "single phase and three phase power" and "power sizing"; the bank tests the words. That is the CDCS lane's reason to exist and it stays there.

---

## Module 09 — Cooling Infrastructure

### Public sources

| Source | URL | What it settles |
|---|---|---|
| ASHRAE liquid W-classes (5th ed. Thermal Guidelines / AI DC Framework) | <https://www.upsite.com/blog/major-changes-to-ashraes-fifth-edition-of-thermal-guidelines-part-3-liquid-cooling-chapter-updates/> · <https://www.ashrae.org/file%20library/technical%20resources/bookstore/emergence-and-expansion-of-liquid-cooling-in-mainstream-data-centers_wp.pdf> | W17(←W1) / W27(←W2) / W32(←W3) / **W40 (new)** / W45(←W4) / W+(←W5). The number is the **upper supply-fluid limit in °C**; **all classes share a 2 °C lower limit**. Redesignated by upper limit for memorability. |
| Dell — liquid coolants guidance for TCS and FWS | <https://www.delltechnologies.com/asset/en-us/products/servers/industry-market/liquid-coolants-guidance-for-technology-cooling-system-and-facility-water-system-whitepaper.pdf> | Names the two loops explicitly: **TCS** (Technology Cooling System, secondary, to the chip) and **FWS** (Facility Water System, primary). |
| Vertiv / DCX / Schneider — CDU architecture | <https://www.vertiv.com/en-us/insights/articles/educational-articles/understanding-coolant-distribution-units-cdus-for-liquid-cooling/> · <https://dcx.eu/facility-vs-technology-cooling-systems/> | The **CDU is the liquid-to-liquid interface** between TCS and FWS, transferring heat while keeping the circuits **physically separated**. Typical published design point: ΔT ≈ 15 °C, **approach temperature ≈ 2–4 K**, TCS supply ~25–35 °C against FWS supply ~23–33 °C. As of 2026 single-phase direct-to-chip is reported as the dominant liquid method. |

### Taught-vs-2026 — one structural gap, and one assessed-but-untaught term

**W-classes are taught correctly** at `modules/09-cooling.md:187-196`, including the shared 2 °C floor that most secondary summaries drop, and M02 correctly frames the choice as a **loop decision, not a CRAH setpoint**.

**What is missing is the loop the class applies to.** Measured across the live notes:

| Term | Notes | Bank |
|---|---|---|
| TCS / "technology cooling system" | **0** | **0** |
| FWS / "facility water system" | 1 | **0** |
| "approach temperature" | **0** | **2** |
| secondary loop | 2 | 2 |
| primary loop | **0** | 1 |

The course says the W-number is an **upper facility-water supply** limit — correct — but never introduces **FWS and TCS as two loops with a CDU between them**, and never introduces **approach temperature**. Those are the mechanism that makes a W-class a plant decision: W32 facility water does **not** mean 32 °C at the cold plate, because the CDU has an approach and the TCS runs its own supply/return. A 2026 oral that asks *"your CDU has a 3 K approach and the TCS needs 30 °C supply — what W-class does that make the facility loop?"* has no taught answer here.

**"Approach temperature" appears in 2 bank items and 0 notes.** That is the assessed-but-untaught shape — the same fairness defect Module 15 carried before `ebrr.16`, at much smaller scale. It should be either taught or the items retired; it should not stay assessed-only.

### Bank rigor — classic material deep, 2026 flagship shallow

| Concept | Files |
|---|---|
| containment | 45 |
| STER / STES | 34 |
| bypass | 30 |
| ride-through | 13 |
| CDU | 11 |
| recirculation | 10 |
| RDHx / rear-door | 9 |
| immersion | 9 |
| psychrometric / dew point | 8 |
| supply-water / facility water | 7 |
| **W17…W45** | **3** |

**The flagship 2026 teaching has three items.** Containment has forty-five. `m09-q204` (recirculation vs **bypass** as a genuine near-miss) is the model distractor pattern the sittings ledger singled out as the exception rather than the rule, and it lives here.

`m09-q250`–`q252` cover W-classes and were **not drawn in sampled seeds 42/7/99**. Coverage of three, sampled at stratum level, means a learner can complete a mock-40 without ever meeting the module's headline 2026 concept.

### CDCS-calc hole — named, not implemented

- **Sensible heat:** `Q = 1.08 × cfm × ΔT` (IP) — airflow for a stated kW at a stated ΔT.
- **Approach temperature across the CDU**, and TCS supply derived from FWS supply + approach — the arithmetic that turns a W-class into a cold-plate temperature.
- **Psychrometrics:** dew point, SHR, wet-bulb / coil leaving-air, economizer hours.
- **W-class selection** from a target FWS supply and a climate's rejection capability.

Notes teach "sensible vs latent" as vocabulary and dew point as a one-liner; nothing computes.

---

## Module 12 — Fire Protection

### Public sources

| Source | URL | What it settles |
|---|---|---|
| NFPA 855 2026 — Annex G.11 | <https://www.telgian.com/nfpa-855-changes-in-2026/> · <https://cleanpower.org/wp-content/uploads/gateway/2026/05/ACP_FactSheet_Battery_LSFT.pdf> | 2026 edition adds Annex G.11 LSFT guidance targeting **fire spread from one BESS to another**; §9.2 fire testing collects gas production at cell level, propagation at module level, and propagation **between** ESS units. |
| UL 9540A 6th edition | <https://www.mayfield.energy/technical-articles/the-6th-edition-of-ul-9540a-is-here/> · <https://internationalfireandsafetyjournal.com/battery-storage-ul9540a/> | Published **13 March 2026**; Section 10 revised to a clear large-scale fire test method aligned to NFPA 855 Annex G.11. |
| UL 9540 (the listing) | <https://www.ul.com/services/ul-9540a-test-method> · <https://mitsubishicritical.com/resources/blog/understanding-the-ul-9540-listing/> | The **safety standard** that yields a *Listed* ESS. 9540A results feed it but **do not confer it**. |

### Taught-vs-2026 — the playbook is present; one name is missing from the module that owns it

`modules/12-fire.md:188-204` teaches the Li-ion / BESS playbook: 855 as the installation standard, 9540A correctly described as *"Test Method for Evaluating Thermal Runaway Fire Propagation in Battery Energy Storage Systems"*, room-vs-yard, off-gas / deflagration, and the water-on-Li-ion controversy without inventing a fire percentage. `ebrr.13` closed this and it holds.

**Gap: `UL 9540` — the listing — is named in `modules/06-power.md:239` but not in `modules/12-fire.md`.** Every M12 hit is `UL 9540A`. The module that owns the fire playbook is the one place a learner will look for "which document makes this ESS *Listed*", and it answers only with the test method. The distinction is one sentence and already exists verbatim in M06.

### Bank rigor — the thinnest plant module, and the 2026 layer is single-digit

| Concept | Files |
|---|---|
| pre-action | 7 |
| VESDA / aspirating | 6 |
| clean agent | 4 |
| NFPA 855 | 3 |
| UL 9540A | 3 |
| off-gas / deflagration | 3 |
| Class C | 3 |
| water-on-Li-ion | 2 |
| **hold time** | **2** |
| **LSFT / large-scale fire test** | **1** |
| **UL 9540 (bare listing)** | **0** |

**65 items, 15 of them `remember`, 3 `analyze`.** This is the most name-recall-weighted module of the three, and the sittings ledger says the in-note eight-pack is still "the classic playbook" — ignition category, ASD, double-interlock, Class C, ABC residue — while the notes and `m12-q226` have moved to 855 / 9540A / room-vs-yard. A module-quiz-only learner never meets the 2026 fire oral.

`m12-q226` is the one strong 2026 item and the sittings graded it **claims-strong, form-weak**: the keyed choice is an ~80-word lecture. Keep the claims; the shape wants splitting into two or three apply items with plausible near-misses (855 vs 9540 vs 9540A; room vs yard).

### CDCS-calc hole — named, not implemented

- **Clean-agent design concentration and hold time**, and enclosure-integrity / door-fan retention.
- **Room volume → agent quantity** for a stated concentration.
- **Deflagration venting / ventilation rate** for off-gas accumulation.

Notes teach hold time as a dependency ("clean agent depends on enclosure integrity"); nothing computes, and `hold time` has two bank items.

---

## Follow-on beads — proposed titles only

Children of `bd-curriculum-truth-ebrr`. **Not filed by this wave.** No new epic. No calc item proposed for the CDCP bank — those are `ms4j.1`.

| Proposed title | Module | Leftover it closes |
|---|---|---|
| M12 notes leftover: name UL 9540 (the listing) beside 9540A in the fire module | M12 | Distinction taught in `06-power.md:239`, absent from the module that owns the playbook. |
| M12 bank leftover: UL 9540 listing-vs-test-method apply item; LSFT and hold-time depth | M12 | `UL 9540` bare = **0** files; LSFT = 1; hold time = 2. |
| M09 notes leftover: teach TCS / FWS / CDU-approach as the loop the W-class applies to | M09 | TCS = 0 notes / 0 bank; FWS = 1 note; **approach temperature = 0 notes but 2 bank items** (assessed-but-untaught). |
| M09 bank leftover: W-class items beyond three; retire or ground the approach-temperature items | M09 | W-classes = 3 files against containment 45; sampled seeds drew none. |
| M06 bank leftover: distributed-redundant contrast item; isolation-transformer depth | M06 | distributed-redundant = 1 against catcher 5, taught as a contrast pair; isolation transformer = 2 and Pass 1's thinnest taught sub-heading. |
| M12 item shape: split `m12-q226` lecture-dump into apply items with near-miss distractors | M12 | Sittings graded it claims-strong / form-weak; keyed choice is an ~80-word paragraph. |

**Deliberately not proposed:** any calc item in the CDCP bank, any invented fire or outage percentage, any nines-to-Tier crosswalk, any new epic.

---

## Wave verdict

**M06:** prose current and externally correct on the 855 / 9540A / 9540 split; the largest and deepest bank in the corpus; two named thin spots (`distributed-redundant` 1, `isolation transformer` 2) and **`UL 9540` at zero** despite the module teaching the distinction.

**M09:** W-classes taught correctly as facility-water supply, but **the loop they apply to is not taught** — TCS/FWS/CDU-approach is absent from the notes while `approach temperature` is already assessed in two items. Classic material is deep (containment 45); the 2026 flagship is **three items** and went undrawn in sampled seeds.

**M12:** playbook present and honest; the **thinnest plant module** (65 items, 15 `remember`, 3 `analyze`), with the 2026 layer in single digits and `UL 9540` unnamed in the module that owns it.

Corpus-wide, all **854** items remain `qualitative_only`. Every calc named in this ledger belongs to `bd-epi-ecosystem-ms4j.1` and none was implemented here.

*Wave 2 research ledger. No module prose, no README, no CHARTER, no `check.sh`, no `cargo`, no bead closed, no commit. `WAVE-01.md` untouched.*
