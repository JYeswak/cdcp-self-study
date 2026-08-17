# SYLLABUS GAP — public CDCS (11) and CDFOS (9) vs what we teach and examine

**Date:** 2026-08-17 · **Pane:** Claude, `cdcp` 0.1 · **Bead context:** `bd-curriculum-truth-ebrr.29.3` (wave 3), informing `bd-epi-ecosystem-ms4j.1` / `.2`
**Governed by:** `ATTRIBUTION-BAR.md` — a heading is only claimed as covered when it maps to a public EPI heading **and** is examined against a citable current standard.

> **Completing this program does not certify anyone.** No EPI®/EXIN® credential follows from any track here.
> **27/40 is a CDCP study signal**, not a pass mark — and explicitly **not** a CDCS 45/60 cut or a CDFOS 42/60 cut. Those two numbers are public exam facts about EPI's exams; they are **not** adopted as bars for anything in this repo.
> Original items only. No dumps. Citation metadata only; no course body retained.

---

## Public exam facts, verified today against the live EPI pages

| Track | Source | Questions | Duration | Pass mark | Prerequisite |
|---|---|---|---|---|---|
| **CDCP** | <https://www.epi-ap.com/services/1/3/4/Certified_Data_Centre_Professional_(CDCP)> | 40 MCQ | 1 hour | **27 of 40** | Training mandatory (EXIN) |
| **CDCS** | <https://www.epi-ap.com/services/1/3/5> | 60 MCQ | 90 min | **45 correct** | **valid CDCP at time of passing** |
| **CDFOS** | <https://www.epi-ap.com/services/1/3/136/Certified_Data_Centre_Facilities_Operations_Specialist_(CDFOS)> | 60 MCQ | 90 min | **42 correct** | none; DCFC/CDCP *recommended* |

CDFOS is stated as *"fully aligned with the DCOS® (Data Centre Operations Standard)."* **Name the standard; do not vendor its body.**

Both module lists below were re-fetched from those pages today and match `ATTRIBUTION-BAR.md` heading-for-heading.

---

## What exists today, measured

```bash
ls modules/*.md | wc -l                                          # 15  (14 CDCP facility domains + 1 ops-adjacent)
ls course-engine/bank/items/*.toml | wc -l                       # 854 files / 829 approved / 25 retired
ls course-engine/tracks/cdcs/bank/ | wc -l                       # 10 heading files
ls course-engine/tracks/cdfos/ 2>/dev/null                       # (none)
grep -rho 'quantity_evidence = "[^"]*"' course-engine/bank/items/ | sort | uniq -c   # 854 qualitative_only
```

- **CDCP:** 15 taught modules, 854 examined items, **all qualitative**.
- **CDCS:** a `tracks/cdcs/bank/` skeleton of **10 heading files** — `cdcs-airflow-cmh`, `cdcs-battery-autonomy`, `cdcs-emf-attenuation`, `cdcs-fire-gas`, `cdcs-fuel-autonomy`, `cdcs-generator-paralleling`, `cdcs-one-line`, `cdcs-psychrometric`, `cdcs-sensible-delta-t`, `cdcs-ups-parallel`. **Owned by pane 2; read-only here.**
- **CDFOS:** **no track exists.**

---

## CDCS — 11 public headings vs our surface

Legend — **CDCP-adjacent** = the concept is taught somewhere in our 15 CDCP modules at CDCP depth. **CDCS calc** = a heading file exists in `tracks/cdcs/bank/`. Neither means the heading is covered at CDCS depth.

| # | Public CDCS heading | CDCP-adjacent | CDCS calc file | Assessment |
|---|---|---|---|---|
| 1 | Data Centre Design / Life Cycle Overview | partial — CHARTER/M01 frame lifecycle informally | — | **absent** |
| 2 | Standards and Rating Level Definitions | **yes** — M02 is the corpus's strongest module (lattice, three plaques, 942-C pin, 99.982 kill) | — | **CDCP depth only.** No rating-level *definition* work at CDCS depth. |
| 3 | Building Considerations | **yes** — M03 (queue, BTM, flood/seismic) | — | **absent** as calc. Wave 3 named floor-loading + fuel autonomy. |
| 4 | Advanced Raised Floor & Suspended Ceiling | **yes** — M04, floor types now taught | — | **absent, and blocked on sourcing** (EN 12825 / ISO 22496 / CISCA are paid or membership; see WAVE-03) |
| 5 | Advanced Power (SLD, OCPD, generators, fuel, UPS parallel, batteries, flywheel, H2) | **yes** — M06, 143 items, largest module | `cdcs-one-line`, `cdcs-ups-parallel`, `cdcs-battery-autonomy`, `cdcs-fuel-autonomy`, `cdcs-generator-paralleling` | **best-covered heading.** Still missing from the file set: **OCPD / protective-device coordination**, **flywheel**, **hydrogen**. |
| 6 | Advanced Electro Magnetic Fields | **yes** — M07 | `cdcs-emf-attenuation` | heading file present |
| 7 | Advanced Cooling (psychrometric, ASHRAE, CFM/CMH, ΔT, liquid) | **yes** — M09, 125 items | `cdcs-psychrometric`, `cdcs-airflow-cmh`, `cdcs-sensible-delta-t` | heading files present. **Gap:** no CDU **approach-temperature** file, and WAVE-02 found TCS/FWS untaught at CDCP depth too. |
| 8 | Advanced Fire Protection | **yes** — M12, thinnest plant module | `cdcs-fire-gas` | heading file present |
| 9 | Designing and Installing Scalable Network Cabling Systems | **yes** — M11 at CDCP depth | — | **absent.** No loss-budget / link-length calc. |
| 10 | Environmental Specifications / Contamination Control | **thin** — M09 touches envelopes; **contamination control is not a taught topic** | — | **absent.** No ISO 14644 / G-class corrosion coverage anywhere. |
| 11 | Data Centre Efficiency | **partial** — PUE/WUE/CUE named in M02, WUE depth in M10 | — | **absent.** `cdcp_metrics` crate exists; no CDCS efficiency calc. |

### CDCS scorecard

- **Headings with a calc file: 4 of 11** (5, 6, 7, 8) — and heading 5 is covered by five files while 1–4 and 9–11 have none.
- **Headings absent entirely from both surfaces: 2 of 11** — **10 (contamination control)** and, as calc, **1 (life cycle)**.
- **`ATTRIBUTION-BAR.md` states this as "covers a slice of 5–8 only. Missing 1–4 and 9–11, and most of Advanced Power." Re-measured today: accurate.**

**Sharpest CDCS finding:** heading **10, Environmental Specifications / Contamination Control**, has no home in either surface. It is not a calc gap — it is a *topic* gap. ISO 14644 particulate classes and gaseous-contamination severity levels are the public anchors, and nothing in 854 CDCP items or 10 CDCS heading files touches them.

---

## CDFOS — 9 public headings vs our surface

**No CDFOS track exists.** The mapping below is therefore against the CDCP corpus only, principally Module 15 (ops-adjacent, 47 items) and Module 13 (security).

| # | Public CDFOS heading | Where it lives today | State |
|---|---|---|---|
| 1 | Service Level Management | M15 — SLA / OLA / UC stack taught; `service catalog` 1 item, `OLA` 2 (acronym only), *"operational level agreement"* spelled **0** | **taught, thinly examined** |
| 2 | Safety and Crisis Management | M15 — incident command, blameless postmortem, OSHA **Subpart S** (2 items) | **taught, thinly examined** |
| 3 | Physical Security | **M13** — layers, mantrap, fail-safe vs fail-secure, CCTV; `security matrix` 1 item | **best-covered CDFOS heading** |
| 4 | Facilities Maintenance | M15 — maintenance contract as underpinning contract, CMMS-auditable terms, spares↔MTTR | **taught, thinly examined** |
| 5 | Data Centre Operations | M15 — MOP/SOP/EOP, level of use (1 item), floor management **0 items** | **partial; floor management taught, untested** |
| 6 | Monitoring / Reporting / Control | **M14** — BMS vs EMS vs DCIM, alarm fatigue; alarm≠status 1 item | **taught, thinly examined** |
| 7 | **Project Management** | — | **ABSENT.** No module, no items. |
| 8 | **Environmental Sustainability** | partial — PUE/WUE/CUE named (M02), WUE depth (M10); no sustainability *management* content | **largely absent** |
| 9 | **Governance and Compliance** | partial — M13 touches SOC2/ISO visitor logs "without overclaiming"; document-management lifecycle taught in M15 with **0 items** | **largely absent** |

### CDFOS scorecard

- **Headings with real CDCP-side coverage: 4 of 9** (1, 2, 3, 4/6 partially).
- **Headings absent or near-absent: 3 of 9** — **7 Project Management**, **8 Environmental Sustainability**, **9 Governance and Compliance**.
- **DCOS alignment is unaddressed.** The public page says CDFOS is *fully aligned with DCOS®*; nothing in this repo names DCOS. That is the single highest-leverage CDFOS action: **name the standard** and map the nine headings to it.

---

## The through-line

Across CDCS and CDFOS the same asymmetry appears, and it is the useful conclusion:

**Where CDCP already teaches a facility domain, the sister-track heading is "taught at CDCP depth, unexamined at track depth."** M06→CDCS-5, M09→CDCS-7, M13→CDFOS-3 are all in this state.

**Where CDCP has no domain, the heading is simply missing** — CDCS **10** (contamination control), CDFOS **7** (project management), **8** (sustainability management), **9** (governance). These are not depth problems; nothing in 854 items touches them.

That distinction should drive ship order. Deepening an existing domain is item work against a corpus that already has notes; opening a missing heading needs notes, sources, and items from zero.

---

## Recommended sequence — for `ms4j.1` / `.2`, not filed here

1. **CDCS heading 5 (Advanced Power)** — already five calc files; add **OCPD/coordination**, **flywheel**, **H2**. Best return: the module has 143 CDCP items and the largest `apply` count to build on.
2. **CDCS heading 7 (Advanced Cooling)** — add **CDU approach temperature**; WAVE-02 showed TCS/FWS is untaught at CDCP depth too, so this closes a gap in both tracks at once.
3. **CDFOS heading 1 (Service Level Management)** — M15 already teaches the SLA/OLA/UC stack; needs items, not notes. Cheapest CDFOS win.
4. **Name DCOS** on the CDFOS lane before writing any CDFOS heading. The public page anchors the whole track to it.
5. **CDCS heading 10 (contamination control)** — the true topic hole. Public anchors exist (ISO 14644 particulate classes; gaseous-corrosion severity). Needs notes first.
6. **Blocked / do not attempt yet:** CDCS heading **4** floor-class calcs — EN 12825 / ISO 22496 / CISCA class tables are paid or membership, and the attribution bar forbids writing the numbers from trade pages. File the sourcing bead; do not work around it.

**Not recommended:** adopting 45/60 or 42/60 as any bar in this repo. They are public facts about EPI's exams, recorded here for accuracy, and this project scores nothing against them.

---

## What this document is not

It is not a claim that any track here prepares anyone for a CDCS or CDFOS exam, and not a claim that our 15 modules "cover" any sister-track heading. Coverage above means *a concept appears in our taught corpus*, at CDCP depth, examined by original items — nothing more.

*Gap map only. No module prose, no README, no CHARTER, no `check.sh`, no `cargo`, no bead filed or closed, no pane-2 CDCS file touched, no course body retained.*
