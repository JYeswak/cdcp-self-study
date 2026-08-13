# How to Use This Program

**Start here** if you want the shortest path from zero to interview-ready data-centre facilities fluency.

---

## Disclaimer — read before anything else

**This is NOT official EPI® certification.**

- This program is an **independent educational reconstruction** of publicly advertised CDCP® syllabus *domains*.
- It is **not** an EPI course, **not** an EXIN exam prep pack, and **not** affiliated with EPI, EXIN, or authorized training partners.
- Completing this material **does not** grant CDCP® (or any other) **post-nominals**, certificates, or credentials.
- Only an **authorized** CDCP course and a **passed official exam** can lead to the CDCP credential.
- No EPI textbooks, slides, or exam questions are copied here. Trademarks belong to their owners and are used only for domain identification.

If you need the **credential**, book authorized training and the official exam.  
If you need **competence** for jobs, interviews, and white-space tours, continue below.

---

## Start here (Day 0 — ~20 minutes)

### A) Interactive course engine (preferred)

Constitution: [`CHARTER.md`](./CHARTER.md). Product: `course-engine/`.

```bash
cd /Users/josh/cdcp-self-study/course-engine
# 8765 is often Agent Mail (uvicorn) — if bind fails or browser shows {"detail":"Not Found"}, use 8766
cargo run -p cdcp_cli -- serve --bind 127.0.0.1:8766
```

Open **http://127.0.0.1:8766/** → Learn · Drill · Mock · Reference.

Session shapes: Learn-15 · module quiz · Drill-10 · Mock-40 · Miss-review.  
Loop #3 external signal: [`course-engine/docs/loop3/PROTOCOL.md`](./course-engine/docs/loop3/PROTOCOL.md).

### B) Markdown corpus (fallback / deep reading)

```bash
cd /Users/josh/cdcp-self-study
```

1. Read the disclaimer above (and the honesty note in [`README.md`](./README.md)).
2. Skim [`00-curriculum-map.md`](./00-curriculum-map.md) once — 14 domains, objectives only.
3. Open [`STUDY-PLAN-14-DAY.md`](./STUDY-PLAN-14-DAY.md) and pick your start date.
4. Bookmark these three references (do **not** memorize yet):
   - [`reference/GLOSSARY.md`](./reference/GLOSSARY.md)
   - [`reference/POWER-AND-REDUNDANCY-CHEATSHEET.md`](./reference/POWER-AND-REDUNDANCY-CHEATSHEET.md)
   - [`practice/DRILL-CARDS.md`](./practice/DRILL-CARDS.md)
5. Optional career track: skim [`job-research/01-JOBS-SHORTLIST.md`](./job-research/01-JOBS-SHORTLIST.md) so study emphasis matches target roles.

**Day 0 exit check:** You can state in one sentence why this program does *not* make you “CDCP certified.”

---

## Study order

### Recommended path (standard / interview-ready)

Follow module numbers **01 → 14**. Public CDCP domains are designed as a stack: business criticality and standards first, then site/building, then the heavy infrastructure (power, cooling), then network/fire/security/ops systems.

| Order | Module | Primary notes | Folder hub |
|---|---|---|---|
| 1 | Mission Critical Site | [`modules/01-mission-critical.md`](./modules/01-mission-critical.md) | [`modules/01-mission-critical-site/`](./modules/01-mission-critical-site/) |
| 2 | Data Centre Standards | [`modules/02-standards.md`](./modules/02-standards.md) | [`modules/02-data-centre-standards/`](./modules/02-data-centre-standards/) |
| 3 | Location, Building & Construction | [`modules/03-site-building.md`](./modules/03-site-building.md) | [`modules/03-location-building-construction/`](./modules/03-location-building-construction/) |
| 4 | Raised Floor & Suspended Ceiling | [`modules/04-floor-ceiling.md`](./modules/04-floor-ceiling.md) | [`modules/04-raised-floor-suspended-ceiling/`](./modules/04-raised-floor-suspended-ceiling/) |
| 5 | Light | [`modules/05-lighting.md`](./modules/05-lighting.md) | [`modules/05-light/`](./modules/05-light/) |
| 6 | Power Infrastructure *(stretch day)* | [`modules/06-power.md`](./modules/06-power.md) | [`modules/06-power-infrastructure/`](./modules/06-power-infrastructure/) |
| 7 | EMF | [`modules/07-emf.md`](./modules/07-emf.md) | [`modules/07-emf/`](./modules/07-emf/) |
| 8 | Equipment Racks | [`modules/08-racks.md`](./modules/08-racks.md) | [`modules/08-equipment-racks/`](./modules/08-equipment-racks/) |
| 9 | Cooling Infrastructure *(stretch day)* | [`modules/09-cooling.md`](./modules/09-cooling.md) | [`modules/09-cooling-infrastructure/`](./modules/09-cooling-infrastructure/) |
| 10 | Water Supply | [`modules/10-water.md`](./modules/10-water.md) | [`modules/10-water-supply/`](./modules/10-water-supply/) |
| 11 | Scalable Network Infrastructure | [`modules/11-network.md`](./modules/11-network.md) | [`modules/11-scalable-network-infrastructure/`](./modules/11-scalable-network-infrastructure/) |
| 12 | Fire Protection | [`modules/12-fire.md`](./modules/12-fire.md) | [`modules/12-fire-protection/`](./modules/12-fire-protection/) |
| 13 | Physical Security & Safety | [`modules/13-security.md`](./modules/13-security.md) | [`modules/13-physical-security-safety/`](./modules/13-physical-security-safety/) |
| 14 | Auxiliary Systems | [`modules/14-auxiliary.md`](./modules/14-auxiliary.md) | [`modules/14-auxiliary-systems/`](./modules/14-auxiliary-systems/) |

**Per-module ritual (30–90 minutes):**

1. Read the **primary notes** file end-to-end.
2. Skim the **folder hub** coverage list.
3. Answer the hub **self-check** out loud, closed notes.
4. Look up unknown terms in the glossary; do not mass-memorize the glossary.
5. On power/cooling days, keep the [power & redundancy cheatsheet](./reference/POWER-AND-REDUNDANCY-CHEATSHEET.md) open.

### Calendar options

| Pace | How |
|---|---|
| **14-day capstone** | Follow [`STUDY-PLAN-14-DAY.md`](./STUDY-PLAN-14-DAY.md) (~1–2 h/day; stretch on M6/M9). |
| **Dense weekend** | Modules 1–5 + 7–8 + 10 one block; 6 + 9 + 11 next; 12–14 + practice last. |
| **Interview-first** | Deepen M1, M6, M9, M11, M14; skim lighter modules but still run their self-checks. |

**Rule:** Interview mode first (explain trade-offs out loud). Exam-style precision last (timed MCQ + drills).

---

## How to self-test

### 1. Module self-checks (continuous)

After each module, close notes and answer the folder hub self-check items. Fail = re-read that section, then re-test the same day.

### 2. Drill cards (after every 2–3 modules, and before the exam)

Open [`practice/DRILL-CARDS.md`](./practice/DRILL-CARDS.md):

- **Exam mode:** 5 seconds think → answer out loud → check Back. Miss → restudy that module.
- **Interview mode:** 30–45 second answer with a trade-off or floor example, not only a definition.

### 3. Practice exam (capstone)

Open [`practice/PRACTICE-EXAM.md`](./practice/PRACTICE-EXAM.md):

| Setting | Value |
|---|---|
| Questions | 40 original MCQs (not official) |
| Time | ~60 minutes, closed notes first pass |
| Self-pass bar | **≥27/40 (67.5%)** — study signal only, not a credential |
| After | Score against the answer key; map misses to modules + drill cards |

**Retest protocol:** Misses become a short list of modules. Re-read those sections, re-drill the related cards, wait ≥24 hours, re-sit the full exam once.

### 4. White-space tour narrative (optional, high value)

Without notes, walk an imaginary site out loud: utility → switchgear → UPS → PDU → rack → heat path → detection/suppression → access control → BMS/DCIM. If you stall, that domain needs another pass.

### 5. Job-track check (optional)

If applying to DC/TPM roles: re-read gap remediations in [`job-research/graph/GRAPH.md`](./job-research/graph/GRAPH.md) and confirm you can speak power, cooling, and availability in interview language—not just network language.

---

## What “done” means here

You are **done with this program** when:

1. You have touched all 14 primary note files.
2. Module self-checks and most drill cards are solid without peeking.
3. Practice exam ≥27/40 with a written fix plan for every miss (or ≥32/40 on a retest).
4. You can run a white-space tour narrative without the cheatsheet.

You are **not** certified. You have **not** earned CDCP post-nominals. You have built domain fluency you can use in interviews and on the floor.

---

## File map

Full inventory: [`MANIFEST.md`](./MANIFEST.md).  
Program overview: [`README.md`](./README.md).
