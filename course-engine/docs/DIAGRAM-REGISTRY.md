# DIAGRAM REGISTRY — interactive mental models

**Status:** P0 set **present** (2026-08-12 Wave C) · P1 set **present** (2026-08-13, bd-1sd.9.1)  
**Stack:** static SVG/HTML + vanilla JS · franken visual tokens · no React  
**Honesty:** study aids only — not EPI/EXIN certification [[claim:claim-not-epi-certified]]

---

## Rules

1. Live at exactly `web/diagrams/{id}.html`. A **present** row may name no other path —
   `cdcp smoke-diagrams` treats a mismatch as an ERROR, not a skip, because a row that
   redirects the check at some other existing file is how this gate was fooled on 2026-08-14.
2. Carry a `class="honesty-banner"` element that disclaims certification, and a
   `data-diagram="{id}"` element as the diagram root. Both are parsed structurally; a file that
   merely contains the words does not pass. Plus an interview one-liner.
3. `cdcp smoke-diagrams` is fail-closed for present rows and derives its set from the
   Inventory table below. The `Status` column is a closed enum — `**present**` or `planned`. Any
   other spelling is an ERROR, never a silent exclusion. The ID and path cells must be backticked.
   The present-row count is pinned by this registry's own unfenced line:

   present_count = 7

   Shipping a new diagram means raising that pin in the same commit as the new row. A row leaving
   the present set without lowering the pin is RED rather than invisible. A fenced, missing, zero,
   or duplicate pin is an ERROR — the count is not pinned in `cdcp_learn::diagrams`.
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
cargo run -q -p cdcp_cli -- smoke-diagrams
./scripts/check.sh
```

### Wiring still owed (not owned by the P1 diagram work)

1. ~~`scripts/smoke_diagrams.py` — add the four P1 rows to its hard-coded `PRESENT` list.~~
   **Closed 2026-08-14 (bd-hngz).** The list is gone: the checker now derives its set from the
   Inventory table above and binds each present row to `web/diagrams/{id}.html`. An eighth present
   row is covered when it is added and `present_count` is raised in the same commit
   (bd-smoke-diagrams-expected-present-pinned-twice-i40d). Fixing this also surfaced a real
   artifact defect the old checker had a hard-coded exemption for — `power-path.html` was shipped
   with no `data-diagram="power-path"` root marker, and the exemption meant nobody found out.
2. Module CTA links (`.diagram-cta`), matching the M01/M06/M09 pattern:
   `12-fire.html` → `fire-sequence`, `02-standards.html` → `standards-map`,
   `04-floor-ceiling.html` and `09-cooling.html` → `floor-airflow`,
   `01-mission-critical.html` and `06-power.html` → `dual-cord-spof`.
3. `web/README.md` diagram inventory rows, and hub/index cards if the P1 set should surface there.
