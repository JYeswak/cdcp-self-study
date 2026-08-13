# Changelog

All notable product-facing changes to **cdcp-course-engine** are recorded here.
Format inspired by Keep a Changelog. Dates are UTC calendar days.

## [Unreleased]

### Added

- M8 Learn v2 pedagogy packaging (waves A–D):
  - Sticky TOC, read progress, formula subset render, continue/ETA on learn hub
  - Lesson units (`web/data/units_index.json`, 127 units) + unit reader shell
  - Mid-unit micro-checks from bank `topic_ids` (offline, honest empty)
  - Interactive diagrams: `power-path`, `site-stack`, `heat-path`
  - Glossary popovers (`glossary.json`) + drill miss concept cards
  - Smokes: `smoke_learn_chrome`, `smoke_learn_v2`, `smoke_diagrams` in `check.sh`
- M9 publicize prep: OSS meta (LICENSE, SECURITY, CONTRIBUTING, CODE_OF_CONDUCT)
  and `.flywheel` publishability bar scaffold (visibility flip not done)

### Honesty

- Study tool only — does not grant EPI/EXIN CDCP certification.

## [0.1.0] — 2026-08-12

### Added

- Local-first HTML course + browser grading (Learn · Drill · Mock 40Q)
- Rust GradeExact oracle + WASM dual-path goldens
- Item bank (~804) with verify/grounding/coverage gates
- Constitution docs, claims registry (L1), scorecards L5–L7 · V11
- Loop#3 hybrid protocol + free/public corpus policy (OQ-09/10)
