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
  and `.flywheel` publishability bar scaffold (visibility flip 2026-08-12; public at github.com/JYeswak/cdcp-self-study)

### Honesty

- Study tool only — does not grant EPI/EXIN CDCP certification.
- 804/779 is a **pool size** (item files / approved), not a count of distinct
  propositions. Measured paraphrase pairs the C3 cosmetic gate cannot see are
  tracked in `course-engine/registries/paraphrase_pairs.toml` (bd-e1yt).

## [0.1.0] — 2026-08-12

### Added

- Local-first HTML course + browser grading (Learn · Drill · Mock 40Q)
- Rust GradeExact oracle + WASM dual-path goldens
- Item bank (~804) with verify/grounding/coverage gates
- Constitution docs, claims registry (L1), scorecards L5–L7 · V11
- Loop#3 hybrid protocol + free/public corpus policy (OQ-09/10)
- M10 free/public corpus expansion: 4 sources referenced with rights validation
  - NIST SP 800-123 (US Government public domain, redistributable)
  - 3 ASHRAE TC 9.9 PDFs (power, storage, edge) purged from HEAD and history; `.meta.toml` sidecars retained for url + sha256 grounding
