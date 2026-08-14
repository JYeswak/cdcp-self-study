# DIAGRAM REGISTRY — interactive mental models

**Status:** P0 set **present** (2026-08-12 Wave C) · P1 set **present** (2026-08-13, bd-1sd.9.1)  
**Stack:** static SVG/HTML + vanilla JS · franken visual tokens · no React  
**Honesty:** study aids only — not EPI/EXIN certification [[claim:claim-not-epi-certified]]

---

## Rules

1. Live under `web/diagrams/{id}.html`.  
2. Honesty banner + interview one-liner.  
3. `scripts/smoke_diagrams.py` fail-closed for present rows.  
4. Prefer steppers / toggles / label-the-node.

---

## Inventory

| ID | Title | Modules | Priority | Status | Path |
|----|-------|---------|----------|--------|------|
| `power-path` | N vs 2N power path | 06 | P0 | **present** | `web/diagrams/power-path.html` |
| `site-stack` | Business → IT → MEP stack | 01 | P0 | **present** | `web/diagrams/site-stack.html` |
| `heat-path` | Chip → plant → outdoors | 09 | P0 | **present** | `web/diagrams/heat-path.html` |
| `fire-sequence` | Detect → interlock → suppress | 12 | P1 | **present** | `web/diagrams/fire-sequence.html` |
| `standards-map` | Who owns what | 02 | P1 | **present** | `web/diagrams/standards-map.html` |
| `floor-airflow` | Raised vs hard floor | 04, 09 | P1 | **present** | `web/diagrams/floor-airflow.html` |
| `dual-cord-spof` | Shared upstream SPOF | 01, 06 | P1 | **present** | `web/diagrams/dual-cord-spof.html` |
| `security-layers` | Perimeter → white space | 13 | P2 | planned | — |

## Interaction model per diagram

Rule 4 says prefer steppers / toggles / label-the-node. What each present diagram actually does:

| ID | Interaction | Learner does |
|----|-------------|--------------|
| `power-path` | label-the-node + MCQ self-check | places labels on blank nodes, answers N vs 2N checks |
| `site-stack` | click-to-expand layers | reveals why each layer depends on the one below |
| `heat-path` | linear stepper | walks chip → plant → outdoors |
| `fire-sequence` | stepper + jump rail + suppression branch toggle | steps origin → detect → confirm → interlock → suppress → recover, and switches the water/gas branch so the same detection event ends two ways |
| `standards-map` | click-to-expand authority ladder + pairwise scheme comparator | expands what force each rung carries, then picks two classification schemes and reads the axis on which they genuinely disagree |
| `floor-airflow` | paradigm toggle + defect injection | switches raised ↔ hard floor, injects a defect, sees which element of the cross-section fails and whether it is bypass, recirculation, distribution or ride-through |
| `dual-cord-spof` | scenario picker + node-state map | runs a real public incident against a dual-corded facility, classifies it as independent / common-cause / antecedent violation, and sees which shared node the one-line never drew |

## Content grounding (P1 set)

P1 content is drawn from `knowledge/graph/dc-lifecycle.graph.json` rather than authored free-hand.
Load-bearing constraints honoured by the P1 set:

- **`dual-cord-spof`** — Google `europe-west4-a` (2026-07-15), AWS `sa-east-1` / São Paulo (2013-12-17)
  and Azure Australia East (2023-08-30) are characterised as **common-mode / control-dependency**
  failures and **antecedent violations**, never as counterexamples to N+1 or 2N. The independence
  antecedent is stated on the page and carried into every scenario verdict, and the nine coupling
  mechanisms are given as the learner's classifier.
- **`standards-map`** — Uptime Tier, ANSI/TIA-942 Rated and EN 50600 / ISO/IEC 22237 Availability
  Class are presented as three instruments from three owners, with the specific disagreement axes
  (evaluation philosophy, assessor independence, certificate decay clock, one-rating-vs-vector,
  maintenance state, classification geometry). No tidy hierarchy, no crosswalk table.
- **`fire-sequence`** — NFPA 75 is named as covering ITE protection from fire *or its associated
  effects*, including **water**; suppression water is inside the protected-against damage, not
  outside it. Tier IV's own exclusion of fire alarm, suppression and EPO is stated at the interlock
  step.
- **ASHRAE** — named only. No ASHRAE publication content is reproduced anywhere in the diagram set
  (ASHRAE AI policy). `standards-map` states this constraint on the page.

## Validation

```bash
python3 scripts/smoke_diagrams.py
./scripts/check.sh
```

### Wiring still owed (not owned by the P1 diagram work)

The four P1 files satisfy every assertion `scripts/smoke_diagrams.py` makes about a present row
(file exists · honesty + certification language · `data-diagram="{id}"` root marker), but the
checker's `PRESENT` list is hard-coded and still names only the three P0 rows. **Until those four
rows are added, `smoke_diagrams` is green without having looked at them** — a present row this
registry advertises that the gate does not cover.

1. `scripts/smoke_diagrams.py` — add to `PRESENT`:
   `("fire-sequence", WEB / "diagrams/fire-sequence.html", "fire-sequence")` and the same shape for
   `standards-map`, `floor-airflow`, `dual-cord-spof`.
2. Module CTA links (`.diagram-cta`), matching the M01/M06/M09 pattern:
   `12-fire.html` → `fire-sequence`, `02-standards.html` → `standards-map`,
   `04-floor-ceiling.html` and `09-cooling.html` → `floor-airflow`,
   `01-mission-critical.html` and `06-power.html` → `dual-cord-spof`.
3. `web/README.md` diagram inventory rows, and hub/index cards if the P1 set should surface there.
