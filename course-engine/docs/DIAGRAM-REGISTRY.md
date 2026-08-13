# DIAGRAM REGISTRY — interactive mental models

**Status:** P0 set **present** (2026-08-12 Wave C)  
**Stack:** static SVG/HTML + vanilla JS · franken visual tokens · no React  
**Honesty:** study aids only — not EPI/EXIN certification

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
| `fire-sequence` | Detect → interlock → suppress | 12 | P1 | planned | — |
| `standards-map` | Who owns what | 02 | P1 | planned | — |
| `floor-airflow` | Raised vs hard floor | 04, 09 | P1 | planned | — |
| `dual-cord-spof` | Shared upstream SPOF | 01, 06 | P1 | planned | — |
| `security-layers` | Perimeter → white space | 13 | P2 | planned | — |

## Validation

```bash
python3 scripts/smoke_diagrams.py
./scripts/check.sh
```
