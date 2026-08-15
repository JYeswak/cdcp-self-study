# 14-Day Study Plan — CDCP Self-Study Capstone

**Goal:** Interview-ready fluency across **15 taught modules** (14 public syllabus domains + Module 15 Operational Considerations) in ~1–2 hours/day (stretch days 2.5–3 h for power/cooling/ops). The calendar is still **14 days**; the module count is 15.

| | |
|---|---|
| **Duration** | 14 days |
| **Daily budget** | ~1–2 hours (see stretch notes) |
| **Total** | ~20–30 hours (standard depth; closer to full 28–40 h if you expand M6/M9) |
| **Outputs** | Notes touch per module · drill cards · practice exam · white-space tour narrative |

This plan **maps modules + practice exam + review**. It does not grant certification.

---

> **Engine path:** Prefer `course-engine` hub (Learn-15 / module quiz / mock) — see [HOW-TO-USE.md](./HOW-TO-USE.md) Path A. Markdown modules remain the deep-read source.

## Two modes (use both)

| Mode | When | How you study | Success looks like |
|---|---|---|---|
| **Interview mode** | Days 1–12 primary; always on M1, M6, M9, M14, M15 | Explain out loud: trade-offs, failure modes, “what I’d ask on a tour.” 30–60 s answers with an example. | You can teach a peer without reading slides. |
| **Exam mode** | Days 12–14; mini-drills after heavy modules | Timed recall, closed notes, MCQ discipline, crisp definitions. | ≥27/40 on practice exam; missed items have a module fix. |

**Rule:** Interview mode first (understanding). Exam mode last (precision). Do not skip interview mode and only memorize acronyms.

---

## Before Day 1 (15 minutes)

1. Read [`README.md`](./README.md) honesty note + purpose.  
2. Skim [`00-curriculum-map.md`](./00-curriculum-map.md) end-to-end.  
3. Open [`reference/GLOSSARY.md`](./reference/GLOSSARY.md) — do not memorize yet.  
4. Bookmark:  
   - [`reference/POWER-AND-REDUNDANCY-CHEATSHEET.md`](./reference/POWER-AND-REDUNDANCY-CHEATSHEET.md)  
   - [`practice/DRILL-CARDS.md`](./practice/DRILL-CARDS.md)  
   - [`practice/PRACTICE-EXAM.md`](./practice/PRACTICE-EXAM.md)

---

## Day-by-day plan

### Day 1 — Mission-critical mindset  
**Mode:** Interview  
**Time:** 1–1.5 h  

| Block | Action |
|---|---|
| 40–50 min | Study [`modules/01-mission-critical.md`](./modules/01-mission-critical.md) + folder `modules/01-mission-critical-site/` |
| 15 min | Out loud: types of DCs, white vs grey space, top outage causes, “nines” intuition |
| 10 min | Glossary: availability, reliability, RTO, RPO, colo, white/grey space |
| 10 min | Drill cards **13, 39** |

**Exit check:** Explain mission-critical without saying “because it has servers.”

---

### Day 2 — Standards landscape  
**Mode:** Interview  
**Time:** 1–1.5 h  

| Block | Action |
|---|---|
| 45 min | [`modules/02-standards.md`](./modules/02-standards.md) + `modules/02-data-centre-standards/` |
| 15 min | Map on paper: TIA-942, ISO/IEC, EN, ASHRAE, local code/AHJ, Uptime Tier (separate) |
| 10 min | Glossary: AHJ, TIA-942, Tier, ASHRAE |
| 10 min | Drill cards **30–31** |

**Exit check:** “Who wins if TIA guidance and local electrical code conflict?” → AHJ/code.

---

### Day 3 — Location, building, construction  
**Mode:** Interview  
**Time:** 1.5–2 h  

| Block | Action |
|---|---|
| 50 min | [`modules/03-site-building.md`](./modules/03-site-building.md) + `modules/03-location-building-construction/` |
| 20 min | List 8 walk-away site risks (flood, single utility, fiber, fuel, adjacency…) |
| 15 min | Supporting spaces checklist (NOC, staging, gen yard, intake) |
| 10 min | Drill: “What makes you walk away from a colo site?” (interview mode) |

**Exit check:** White space can look fine while supporting facilities doom ops.

---

### Day 4 — Raised floor & ceiling  
**Mode:** Interview  
**Time:** 1–1.5 h  

| Block | Action |
|---|---|
| 45 min | [`modules/04-floor-ceiling.md`](./modules/04-floor-ceiling.md) + `modules/04-raised-floor-suspended-ceiling/` |
| 15 min | Loading: concentrated / rolling / uniform — one sentence each |
| 15 min | Airflow: missing tile failure story |
| 10 min | Drill card **32** |

**Exit check:** Why high-density halls may skip raised floor.

---

### Day 5 — Lighting  
**Mode:** Interview + light exam  
**Time:** 1 h  

| Block | Action |
|---|---|
| 35 min | [`modules/05-lighting.md`](./modules/05-lighting.md) + `modules/05-light/` |
| 15 min | Normal vs emergency vs egress lighting |
| 10 min | Mini exam mode: write 5 lighting audit issues from memory |

**Exit check:** Emergency lighting ≠ “nice LEDs on UPS.”

---

### Day 6 — Power deep dive (part 1)  
**Mode:** Interview  
**Time:** 2–2.5 h *(stretch day)*  

| Block | Action |
|---|---|
| 70 min | [`modules/06-power.md`](./modules/06-power.md) + `modules/06-power-infrastructure/` — path, transformers, gens, ATS/STS |
| 20 min | Draw power path from memory; compare to [`reference/POWER-AND-REDUNDANCY-CHEATSHEET.md`](./reference/POWER-AND-REDUNDANCY-CHEATSHEET.md) |
| 20 min | ATS vs STS table from memory |
| 15 min | Drill cards **1–3, 8–10, 37** |

**Exit check:** Explain ATS vs STS without notes.

---

### Day 7 — Power deep dive (part 2)  
**Mode:** Interview → exam  
**Time:** 2–2.5 h *(stretch day)*  

| Block | Action |
|---|---|
| 60 min | Redundancy N/N+1/2N, UPS types, batteries/BESS, PDU/busbar, thermography, PUE/WUE |
| 20 min | Redundancy table from cheatsheet — cover answers, recite |
| 20 min | Drill cards **4–7, 11–12, 27, 38** |
| 15 min | Exam mode: define N+1 vs 2N for a CFO in 45 seconds |

**Exit check:** Dual-cord + A-B path story without “magic redundancy” hand-waving.

---

### Day 8 — EMF + racks  
**Mode:** Interview  
**Time:** 1.5 h  

| Block | Action |
|---|---|
| 35 min | [`modules/07-emf.md`](./modules/07-emf.md) + `modules/07-emf/` |
| 40 min | [`modules/08-racks.md`](./modules/08-racks.md) + `modules/08-equipment-racks/` |
| 15 min | Drill cards **24–25, 28–29** |

**Exit check:** Would you put UPS next to network core? Why/why not?

---

### Day 9 — Cooling deep dive  
**Mode:** Interview  
**Time:** 2.5–3 h *(stretch day)*  

| Block | Action |
|---|---|
| 90 min | [`modules/09-cooling.md`](./modules/09-cooling.md) + `modules/09-cooling-infrastructure/` |
| 20 min | CRAC/CRAH, containment, sensible/latent, liquid/CDU |
| 20 min | Cheatsheet “cooling at a glance” section |
| 20 min | Drill cards **14–20** |

**Exit check:** Heat path chip → outdoors in one breath.

---

### Day 10 — Water + network  
**Mode:** Interview  
**Time:** 1.5–2 h  

| Block | Action |
|---|---|
| 30 min | [`modules/10-water.md`](./modules/10-water.md) + `modules/10-water-supply/` |
| 50 min | [`modules/11-network.md`](./modules/11-network.md) + `modules/11-scalable-network-infrastructure/` |
| 15 min | Drill cards **36** + MMR/pathways narrative |
| 10 min | WUE + process water risk one-liner |

**Exit check:** Scalable cabling = pathways + plan, not “more patch cords.”

---

### Day 11 — Fire protection  
**Mode:** Interview + exam  
**Time:** 1.5–2 h  

| Block | Action |
|---|---|
| 50 min | [`modules/12-fire.md`](./modules/12-fire.md) + `modules/12-fire-protection/` |
| 20 min | Detection vs suppression; clean agent vs water trade-offs |
| 15 min | Drill cards **34–35** |
| 15 min | Exam mode: 5 fire MCQ-style self-questions aloud |

**Exit check:** Early detection buys time; agent choice is design/AHJ, not blog preference.

---

### Day 12 — Security + auxiliary + operational considerations  
**Mode:** Interview → exam  
**Time:** 2.5 h  

| Block | Action |
|---|---|
| 35 min | [`modules/13-security.md`](./modules/13-security.md) + `modules/13-physical-security-safety/` |
| 35 min | [`modules/14-auxiliary.md`](./modules/14-auxiliary.md) + `modules/14-auxiliary-systems/` |
| 30 min | [`modules/15-ops-adjacent.md`](./modules/15-ops-adjacent.md) — 2.1 Operational Considerations (ops-adjacent; exam weight unknown) |
| 20 min | BMS vs DCIM vs EMS; MOP; leak detection; contributor-vs-root (not a three-bucket pie) |
| 20 min | Drill cards **21–27, 33, 40** |

**Exit check:** White-space tour in 5 bullets (Card 40). Name one 2.1 control (MOP level-of-use or maintenance SLA).

---

### Day 13 — Practice exam  
**Mode:** Exam  
**Time:** 1.5–2 h  

| Block | Action |
|---|---|
| 60 min | Closed notes: [`practice/PRACTICE-EXAM.md`](./practice/PRACTICE-EXAM.md) (40 Q) |
| 20 min | Score with answer key; tag each miss to a module |
| 20–40 min | Re-read only missed-topic sections + related drill cards |

**Target:** ≥27/40. If below, schedule Day 14 as remediation-heavy (still do tour narrative).

---

### Day 14 — Integration review + weak-topic drill  
**Mode:** Interview (primary) + exam (gaps)  
**Time:** 1.5–2 h  

| Block | Action |
|---|---|
| 25 min | Full power path + cooling path from blank paper |
| 20 min | Shuffle all 40 drill cards; mark still-cold cards |
| 20 min | Interview mode: 5 anchors from curriculum map (M1, M2, M6, M9, M15) |
| 15 min | Capstone: one-page “class A white space” sketch — power, cooling, cabling, fire, security, monitoring |
| 10 min | Glossary skim of still-cold terms |
| 10 min | Optional: re-answer only missed practice-exam questions |

**Exit checklist (program complete signal)**

- [ ] Draw one-line power path and cooling path from memory  
- [ ] Name 3–5 standards/bodies and what each covers  
- [ ] Contrast containment vs chaotic flooded room airflow  
- [ ] List detection vs suppression ideas and wet-system caution near IT  
- [ ] Explain BMS vs DCIM vs EMS in one sentence each  
- [ ] Separate a contributing factor from a primary failure object (M15)  
- [ ] Practice exam self-score recorded; misses remediated  

---

## Quick calendar (at a glance)

| Day | Focus | Modules | Hours | Mode |
|---|---|---|---|---|
| 1 | Mission-critical | M1 | 1–1.5 | Interview |
| 2 | Standards | M2 | 1–1.5 | Interview |
| 3 | Site & building | M3 | 1.5–2 | Interview |
| 4 | Floor & ceiling | M4 | 1–1.5 | Interview |
| 5 | Lighting | M5 | 1 | Both |
| 6 | Power path | M6 p1 | 2–2.5 | Interview |
| 7 | Power redundancy/UPS | M6 p2 | 2–2.5 | Both |
| 8 | EMF + racks | M7–M8 | 1.5 | Interview |
| 9 | Cooling | M9 | 2.5–3 | Interview |
| 10 | Water + network | M10–M11 | 1.5–2 | Interview |
| 11 | Fire | M12 | 1.5–2 | Both |
| 12 | Security + auxiliary + ops | M13–M15 | 2.5 | Both |
| 13 | Practice exam | All | 1.5–2 | Exam |
| 14 | Integration | All | 1.5–2 | Interview + gaps |

---

## If you only have 1 hour some days

**Priority order:** M6 → M9 → M1 → M12 → M2 → M15 → everything else.  
Never skip Days 13–14 entirely; shrink earlier light modules (M5, M7) first.

## If power/cooling is brand new

Add a **Day 6b / 9b** buffer weekend: re-draw diagrams + cards only (no new modules). Total becomes ~16 calendar days at same depth.

## Interview vs exam — sample prompts

**Interview mode**
- “Explain N+1 vs 2N to a CFO.”  
- “Walk me through a white-space tour.”  
- “Cooling failed — what do you look at first?”  

**Exam mode**
- “Define STS in one sentence.”  
- “PUE formula?”  
- “CRAC vs CRAH?”  

---

## After the 14 days

| Goal | Next step |
|---|---|
| Job interviews | Keep cheatsheet + Card 40 cold; practice aloud weekly |
| Deeper design literacy | Expand M6/M9 notes; optional “Deep” hours in README |
| Official CDCP® credential | Authorized EPI training + EXIN exam — this repo is not a substitute |

---

*Educational plan. Independent of EPI/EXIN.*
