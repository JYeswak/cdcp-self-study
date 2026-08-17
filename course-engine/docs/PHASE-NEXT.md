# PHASE NEXT — after M8 / M10 (2026-08-12)

**Supersedes planning intent of** `PHASE-REMAINING.md` (historical; L7/V11 done).

## Done (do not re-plan)

| Wave | Outcome |
|------|---------|
| W0–L4 | knowledge · claims · bank · GradeExact · WASM dual-path [[claim:claim-grade-byte-exact]] |
| L5 | browser Hub/Learn/Quiz/Drill/Mock/Results + e2e digests |
| L6 | coverage · mastery · weak→learn · multi-seed · sessions |
| L7 | closed-notes · reference · Learn-15 · SLO · content.lock · CLI · a11y |
| V11 | Anki · power-path · serve · runbooks |
| HUMAN | OQ-09 store free/public · OQ-10 defer spend · H-PUB L88 process · Loop#3 hybrid + T1 log |
| **M8 Learn v2** | units (134) · TOC · micro-checks · P0 diagrams · glossary · concept cards · LEARN-v2 pass |
| **M9-S1/S2/S3** | OSS meta + L88 bar doctor **7/7**; visibility flip **done** 2026-08-12 (`public_repo=true`, verified `gh repo view` → PUBLIC) |
| **M10** | free-pdfs ×5 (ASHRAE power/storage/edge + LBNL FEMP + NIST SP 800-123) + meta |

## Open (product / human)

```text
(M9-S3 secrets scrub + visibility flip completed 2026-08-12; repo is PUBLIC.)
P1     diagrams shipped (bd-1sd.9.1). P2 `security-layers` shipped (bd-1sd.9.2).
```

## Gate

```bash
./scripts/check.sh   # must stay green
.flywheel/scripts/publishability-bar.sh --doctor --json
```

## Try the product

```bash
cargo build -p cdcp_cli --locked
./target/debug/cdcp serve --bind 127.0.0.1:8766
# http://127.0.0.1:8766/learn/01-mission-critical.html  (unit shell + site-stack)
# http://127.0.0.1:8766/diagrams/heat-path.html
# http://127.0.0.1:8766/drill.html  (miss → concept card)
```

## Non-claims

- Completing modules ≠ EPI certified [[claim:claim-not-epi-certified]]  
- Study signal 27 ≠ credential [[claim:claim-study-signal-27]]  
- L88 bar green ≠ auto public remote

## Bead status

| Epic | ID | Status |
|------|-----|--------|
| M8 Learn v2 | `bd-1t3` | **closed** |
| M9 Publicize L88 | `bd-2nj` | **closed** |
| M9-S3 flip | `bd-2nj.3` | **closed** (2026-08-12; repo PUBLIC) |
| M10 Free corpus | `bd-3ps` | **closed** |
