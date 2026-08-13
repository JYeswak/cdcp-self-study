# CDCP Self-Study Program (Educational Reconstruction)

**Free, open self-study** covering the same *substance* as the publicly advertised EPI® CDCP® (Certified Data Centre Professional) syllabus domains.

| | |
|---|---|
| **Depth mode** | Standard (interview-ready) |
| **Official course duration** | 2 days instructor-led |
| **This program estimate** | ~28–40 hours self-paced |
| **14-day capstone plan** | ~1–2 h/day → [`STUDY-PLAN-14-DAY.md`](./STUDY-PLAN-14-DAY.md) |
| **Official affiliation** | **None** — not EPI, not EXIN, not a cert path |

---

## Honesty note (read this first)

- This is **not** an official EPI course, exam prep pack, or certification.
- It does **not** copy EPI textbooks, slides, or exam questions.
- It reconstructs **publicly known syllabus topics** (from EPI marketing pages, authorized training partners, and industry-standard knowledge) so you can learn the domain for **jobs, interviews, and on-the-floor competence**.
- Completing this program does **not** make you “CDCP certified.” Only passing the official EXIN/EPI exam after authorized training does that.
- Trademarks: EPI®, CDCP®, and related marks belong to their owners. Used here only for domain identification.

If your goal is the **credential**, take an authorized CDCP course and exam. If your goal is **knowing how data centres work**, this program is for you.

---

## Start here — Day 1 (5 minutes to launch)

### Path A — Interactive engine (recommended)

```bash
cd course-engine
cargo run -p cdcp_cli -- serve --bind 127.0.0.1:8766
# open http://127.0.0.1:8766/  (use a free port; 8765 may be Agent Mail)
```

Hub → **Learn** Module 01 → quiz / Learn-15. Charter: [`CHARTER.md`](./CHARTER.md). Engine docs: [`course-engine/README.md`](./course-engine/README.md).

### Path B — Markdown-only (offline notes)

```bash
cd cdcp-self-study
```

1. Read this honesty note (above).
2. Skim [`00-curriculum-map.md`](./00-curriculum-map.md) once (14 domains + objectives).
3. Open the **14-day plan**: [`STUDY-PLAN-14-DAY.md`](./STUDY-PLAN-14-DAY.md).
4. **Day 1 work:** study Module 1 notes:
   - [`modules/01-mission-critical.md`](./modules/01-mission-critical.md)
   - folder [`modules/01-mission-critical-site/`](./modules/01-mission-critical-site/)
5. Keep nearby: [`reference/GLOSSARY.md`](./reference/GLOSSARY.md) (look up terms; don’t memorize the whole list yet).

**Day 1 exit check:** Explain “mission-critical” without only saying “because it has servers.”

Prefer a denser calendar? Follow Week 1/Week 2 tables below. Prefer paced interview prep? Use the 14-day plan (interview mode vs exam mode built in). Full how-to: [`HOW-TO-USE.md`](./HOW-TO-USE.md).

---

## Purpose

Build interview-ready fluency in the facilities and infrastructure domains that CDCP-level roles discuss daily:

- Why data centres fail (availability, business impact)
- Standards landscape (TIA-942, ISO, EN, ASHRAE, local codes)
- Site, building, floor, lighting
- Power chain (utility → UPS → PDU → rack)
- EMF, racks, cooling (including liquid), water
- Structured cabling and TIA-942 topologies
- Fire, physical security/safety, monitoring (BMS / DCIM / EMS)

After finishing, you should be able to walk a white-space tour, explain trade-offs (N vs N+1, hot aisle vs containment, gas vs water suppression), and answer mid-level facilities/IT hybrid interview questions with correct terminology.

---

## Directory layout

```text
cdcp-self-study/
├── README.md                          ← you are here
├── 00-curriculum-map.md               ← 14 modules + learning objectives
├── STUDY-PLAN-14-DAY.md               ← day-by-day capstone schedule
├── modules/                           ← study notes (flat .md + topic folders)
├── practice/
│   ├── PRACTICE-EXAM.md               ← 40 original MCQs + answer key
│   └── DRILL-CARDS.md                 ← 40 flash-style prompts
└── reference/
    ├── GLOSSARY.md                    ← 100+ terms
    └── POWER-AND-REDUNDANCY-CHEATSHEET.md
```

---

## Module index (all lesson files)

Each domain has a **primary notes file** (`.md`) and often a **topic folder** for extras. Study the `.md` first; use the folder for expansions.

| # | Domain | Primary notes | Folder |
|---|---|---|---|
| 01 | The Mission Critical Site | [`modules/01-mission-critical.md`](./modules/01-mission-critical.md) | [`modules/01-mission-critical-site/`](./modules/01-mission-critical-site/) |
| 02 | Data Centre Standards | [`modules/02-standards.md`](./modules/02-standards.md) | [`modules/02-data-centre-standards/`](./modules/02-data-centre-standards/) |
| 03 | Location, Building & Construction | [`modules/03-site-building.md`](./modules/03-site-building.md) | [`modules/03-location-building-construction/`](./modules/03-location-building-construction/) |
| 04 | Raised Floor & Suspended Ceiling | [`modules/04-floor-ceiling.md`](./modules/04-floor-ceiling.md) | [`modules/04-raised-floor-suspended-ceiling/`](./modules/04-raised-floor-suspended-ceiling/) |
| 05 | Light | [`modules/05-lighting.md`](./modules/05-lighting.md) | [`modules/05-light/`](./modules/05-light/) |
| 06 | Power Infrastructure | [`modules/06-power.md`](./modules/06-power.md) | [`modules/06-power-infrastructure/`](./modules/06-power-infrastructure/) |
| 07 | Electro Magnetic Fields (EMF) | [`modules/07-emf.md`](./modules/07-emf.md) | [`modules/07-emf/`](./modules/07-emf/) |
| 08 | Equipment Racks | [`modules/08-racks.md`](./modules/08-racks.md) | [`modules/08-equipment-racks/`](./modules/08-equipment-racks/) |
| 09 | Cooling Infrastructure | [`modules/09-cooling.md`](./modules/09-cooling.md) | [`modules/09-cooling-infrastructure/`](./modules/09-cooling-infrastructure/) |
| 10 | Water Supply | [`modules/10-water.md`](./modules/10-water.md) | [`modules/10-water-supply/`](./modules/10-water-supply/) |
| 11 | Scalable Network Infrastructure | [`modules/11-network.md`](./modules/11-network.md) | [`modules/11-scalable-network-infrastructure/`](./modules/11-scalable-network-infrastructure/) |
| 12 | Fire Protection | [`modules/12-fire.md`](./modules/12-fire.md) | [`modules/12-fire-protection/`](./modules/12-fire-protection/) |
| 13 | Physical Security & Safety | [`modules/13-security.md`](./modules/13-security.md) | [`modules/13-physical-security-safety/`](./modules/13-physical-security-safety/) |
| 14 | Auxiliary Systems | [`modules/14-auxiliary.md`](./modules/14-auxiliary.md) | [`modules/14-auxiliary-systems/`](./modules/14-auxiliary-systems/) |

**Curriculum map (objectives for every module):** [`00-curriculum-map.md`](./00-curriculum-map.md)

---

## Capstone materials

| Path | What it is |
|---|---|
| [`STUDY-PLAN-14-DAY.md`](./STUDY-PLAN-14-DAY.md) | Day-by-day plan (~1–2 h/day); **interview mode** vs **exam mode** |
| [`reference/GLOSSARY.md`](./reference/GLOSSARY.md) | 100+ crisp terms (ATS, STS, UPS, PDU, CRAH, PUE, N+1, MOP, IST, …) |
| [`reference/POWER-AND-REDUNDANCY-CHEATSHEET.md`](./reference/POWER-AND-REDUNDANCY-CHEATSHEET.md) | Power path, redundancy table, UPS types, cooling at a glance |
| [`practice/DRILL-CARDS.md`](./practice/DRILL-CARDS.md) | 40 front/back flash prompts |
| [`practice/PRACTICE-EXAM.md`](./practice/PRACTICE-EXAM.md) | 40 original MCQs + answer key (study signal only; not official exam) |

---

## How to use

1. **Read** [`00-curriculum-map.md`](./00-curriculum-map.md) once end-to-end — map the 14 domains.
2. **Pick a schedule:**
   - **14-day plan** → [`STUDY-PLAN-14-DAY.md`](./STUDY-PLAN-14-DAY.md) (**recommended for capstone**)
   - **2-week outline** → tables below (slightly denser weekdays)
3. **Study in order** (modules 1→14). Power (M6) and cooling (M9) are heavy; budget extra time.
4. For each module:
   - Read objectives in the curriculum map
   - Study the primary `.md` (and folder extras)
   - Drill matching cards in [`practice/DRILL-CARDS.md`](./practice/DRILL-CARDS.md)
   - Look up terms in [`reference/GLOSSARY.md`](./reference/GLOSSARY.md)
5. **Weekly self-check:** explain the module out loud in 5 minutes without notes (**interview mode**).
6. **Capstone exam day:** timed [`practice/PRACTICE-EXAM.md`](./practice/PRACTICE-EXAM.md) (**exam mode**); remediate misses.
7. **Tour narrative:** one-page white-space sketch — power path, cooling path, cabling pathways, fire zones, security layers, monitoring points (BMS/DCIM/EMS).

**Suggested study rhythm (standard depth):** 1.5–3 hours per module; 4–6 hours each for Power (M6) and Cooling (M9).

---

## How this maps to paid CDCP

| Dimension | Official EPI CDCP® (public info) | This self-study |
|---|---|---|
| Format | 2-day ILT / VILT / TOD | Self-paced reading + practice |
| Classroom hours | ~14–16 hours instruction | — |
| Self-study hours (this program) | — | **~28–40 hours** total |
| Exam | 40 MCQ, 60 min, pass 27/40, closed-book | Self-check only: [`practice/PRACTICE-EXAM.md`](./practice/PRACTICE-EXAM.md) (original Qs) |
| Certificate | EXIN-accredited CDCP® (3-year validity) | None |
| Content basis | Proprietary EPI materials | Public syllabus domains + industry knowledge |
| Best for | Credential + structured instructor Q&A | Cost-free domain literacy / interview prep |

### Hours estimate (2-day course → self-study)

A 2-day instructor course packs dense content with little dwell time. Self-study that reaches **interview-ready** depth typically needs **~2–2.5×** classroom hours because you supply your own examples, diagrams, and recall practice.

| Block | Official (approx.) | Self-study (standard) |
|---|---|---|
| M1–M5 (site, standards, floor, light) | ~3–4 h | 6–8 h |
| M6 Power | ~3–4 h | 6–8 h |
| M7–M8 EMF, racks | ~1–1.5 h | 2–3 h |
| M9 Cooling | ~2–3 h | 5–7 h |
| M10 Water | ~0.5 h | 1 h |
| M11 Network / cabling | ~1.5–2 h | 3–4 h |
| M12–M14 Fire, security, auxiliary | ~2–3 h | 4–6 h |
| Integration / review | residual | 3–4 h |
| **Total** | **~14–16 h** | **~28–40 h** |

If you later sit an official course, this prep makes the 2 days reinforcement instead of first exposure.

---

## 2-week study plan outline

Assume ~2–3 hours/day on weekdays, light review on weekends (**~30–35 hours**). For a tighter **1–2 h/day** track with exam day built in, use [`STUDY-PLAN-14-DAY.md`](./STUDY-PLAN-14-DAY.md) instead.

### Week 1 — Foundation + power path

| Day | Focus | Modules | Target hours |
|---|---|---|---|
| Mon | Mission-critical mindset; availability & downtime cost | M1 | 2 |
| Tue | Standards landscape (TIA / ISO / EN / ASHRAE / local) | M2 | 2 |
| Wed | Site selection, building, supporting facilities | M3 | 2–2.5 |
| Thu | Raised floor, suspended ceiling, grounding, cooling impact | M4 | 2 |
| Fri | Lighting + emergency lighting | M5 | 1.5 |
| Sat | **Power deep dive** (utility → transformer → gen → ATS/STS → UPS → PDU) | M6 (part 1) | 3 |
| Sun | Power cont. (redundancy N/N+1/2N, batteries, BESS, thermography) + review | M6 (part 2) | 3 |

### Week 2 — Thermal, network, life safety, ops

| Day | Focus | Modules | Target hours |
|---|---|---|---|
| Mon | EMF sources, units, shielding | M7 | 1.5 |
| Tue | Racks, dimensions, security, PDUs/rails | M8 | 1.5–2 |
| Wed | Cooling principles, CRAC/CRAH, containment, liquid, STER | M9 | 3–4 |
| Thu | Water supply + scalable cabling / TIA-942 topologies | M10, M11 | 3 |
| Fri | Fire protection (detection, water/gas, classes, signage) | M12 | 2–2.5 |
| Sat | Physical security & safety + BMS/DCIM/EMS/leaks/alarms | M13, M14 | 3 |
| Sun | Full walkthrough + practice exam + weak-topic drill | All | 2–3 |

### After two weeks

- [ ] Can draw a one-line power path and cooling path from memory  
- [ ] Can name 3–5 standards bodies and what each covers  
- [ ] Can contrast containment vs raised-floor flooded cooling  
- [ ] Can list fire detection vs suppression choices and when wet systems are avoided  
- [ ] Can explain BMS vs DCIM vs EMS in one sentence each  
- [ ] Practice exam self-score recorded; misses remediated  

---

## Depth modes

| Mode | Goal | Hours (guide) |
|---|---|---|
| **Standard** (this program default) | Interview-ready vocabulary + system-level trade-offs | 28–40 h |
| **Deep** (optional later) | Exam-max detail, edge cases, sizing drills, multi-standard comparison tables | 50–70 h |

**Study modes inside the 14-day plan**

| Mode | Goal |
|---|---|
| **Interview mode** | Trade-offs, failure stories, tour narrative (primary through Day 12) |
| **Exam mode** | Timed MCQ + crisp definitions (Days 13–14, mini-drills after heavy modules) |

Stick to standard until you can teach each module in plain language; only then deepen.

---

## What this program will not do

- Grant or substitute for CDCP® certification  
- Provide real exam questions or dumps  
- Replace licensed engineering (electrical, fire, structural) or local code authority  
- Cover full CDCS® / CDCE® design depth (those are advanced design tracks)

---

## Getting started (copy-paste)

```bash
git clone <this-repo> cdcp-self-study
cd cdcp-self-study

less 00-curriculum-map.md
less STUDY-PLAN-14-DAY.md
less modules/01-mission-critical.md
ls modules practice reference
```

Then follow **Day 1** in [`STUDY-PLAN-14-DAY.md`](./STUDY-PLAN-14-DAY.md).

---

## Run the course engine

The engine serves the Learn / Drill / Mock surfaces locally and grades in the
browser via WASM. Requires a Rust toolchain.

```bash
cd course-engine
cargo run -p cdcp_cli -- serve --bind 127.0.0.1:8766
# → http://127.0.0.1:8766/
```

Do **not** open `web/` as a `file://` URL if you need the quiz or WASM grading —
browsers block `fetch` on `file://`.

## Gate

Every change is fail-closed behind one ordered chain:

```bash
cd course-engine && ./scripts/check.sh
```

It runs 51 steps (W0–L7 + V11) and, critically, includes five known-bad
selftest suites that inject deliberate faults and assert the gate goes **RED** —
so a green run means the gates demonstrably trip, not merely that nothing ran.

## Licence

Dual-licensed — see [`LICENSE`](./LICENSE):

- **Software** (`course-engine/crates`, `scripts`, `web`) — MIT
- **Curriculum content** (`modules/`, `practice/`, `reference/`, question bank) —
  CC BY-NC-SA 4.0

Studying this yourself, including for a job or interview, is not a commercial
use. Reselling the curriculum as paid training is.

Contributions: [`CONTRIBUTING.md`](./CONTRIBUTING.md) ·
Security: [`SECURITY.md`](./SECURITY.md) ·
Conduct: [`CODE_OF_CONDUCT.md`](./CODE_OF_CONDUCT.md)

---

*Educational reconstruction of public CDCP syllabus domains. Independent project. No warranty.*
