# Publishability Audit — cdcp-course-engine

- **Repo:** course-engine (cdcp-self-study)
- **Date:** 2026-08-12
- **Auditor:** agent (wave M9-S2)
- **Public repo:** no
- **Exemption:**
- **Target:** public after bar ≥5/7 + secrets scrub + Josh visibility flip (`bd-2nj.3`)

## Voice metrics (table fields for doctor)

| Field | Value |
|-------|-------|
| ZestStream voice score | 95 |
| Ungrounded claims count | 0 |
| Scorecard log | not-run-private-prep |

## Facet scorecard

| ID | Facet | Verdict | Evidence |
|----|-------|---------|----------|
| F1 | README front-door | YES | README names product, honesty (not EPI), start path (`cdcp serve` :8766), gate (`./scripts/check.sh`), layout |
| F2 | Doctrine clarity | YES | AGENTS.md · CHARTER parent · docs/ORACLE-GAUNTLET · LEARN-PEDAGOGY · FEATURE_SURFACE · PUBLICIZE-PROCESS |
| F3 | Doctor/health/repair triad | YES | `./scripts/check.sh` hard gate (W0–L7+V11+M8); `.flywheel/scripts/publishability-bar.sh --doctor --json` exposes score; repair = re-run failing script named in fail output |
| F4 | Executable tests | YES | `./scripts/check.sh` + cargo test + smokes (`smoke_learn_v2`, `smoke_diagrams`, goldens, selftests) — named, fail-closed |
| F5 | Idempotent install + uninstall | YES | Clone + rustup + `cargo build`; no global install required; serve binds local only; uninstall = delete tree (no orphan services by default) |
| F6 | Code aesthetic | YES | Named crates (`cdcp_core`, `cdcp_grade`, `cdcp_cli`…); web assets modular (`learn_units.js`, `concept_card.js`); registries as TOML constitution |
| F7 | Demo-ability | YES | One command: `cargo run -p cdcp_cli -- serve --bind 127.0.0.1:8766` → hub Learn/Mock; unit mode + diagrams offline |

## Score

**7 / 7** — pass (≥5 readiness).

Status: **pass** for quality bar. **Not** a visibility flip. Visibility remains private until `bd-2nj.3` (secrets scrub + Josh).

## Three judges (summary)

| Judge | Signal |
|-------|--------|
| Jeff | check.sh + registry-check + known-bad selftests + dual-path goldens |
| Donella | claims constitution · content.lock · mastery as study signal stocks · Loop#3 external log |
| Joshua | Honesty banners · FEATURE_SURFACE truth table · M8 pedagogy packaging shipped |

## Explicit NOs / remaining before public remote

| Item | Status |
|------|--------|
| Secrets/PII scrub of history & docs | open — `bd-2nj.3` |
| GitHub public remote / visibility flip | open — Josh only |
| ZestStream brand-voice scorecard on public copy | optional refresh when Public repo: yes |
| P1 diagrams (fire-sequence, …) | planned, not blocking bar |

## Non-claims

- Bar green ≠ EPI certification.
- Bar green ≠ auto-publish.
- Free/public corpus only (OQ-09); paid SDO out of tree (OQ-10).
