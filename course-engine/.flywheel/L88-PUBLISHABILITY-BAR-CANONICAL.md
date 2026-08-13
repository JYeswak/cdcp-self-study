## L88 — PUBLISHABILITY-BAR-CANONICAL

---
id: L88
title: Publishability bar canonical
status: long_term
shipped: 2026-05-04
review_due: 2026-11-04
trauma_class: publishability-drift
---

Every flywheel-owned repo should clear a first-look publishability bar: Jeff
would trust the substrate mechanics, Donella would see the feedback system, and
Joshua would be willing to show the work as ZestStream AaaS-grade.

**How to apply:**
- Store the rubric at `.flywheel/PUBLISHABILITY-BAR.md`.
- Store each repo's current assessment at `.flywheel/PUBLISHABILITY-AUDIT.md`.
- Score the seven facets: README front-door, doctrine clarity,
  doctor/health/repair triad, executable tests, idempotent install/uninstall,
  code aesthetic, and demo-ability.
- `flywheel-loop doctor --repo <repo> --json` MUST expose
  `publishability_bar_score` and nested `publishability_bar` evidence.
- Scores below 5 warn. Scores below 3 fail readiness.
- `/flywheel:plan` Phase 3 MUST include the three-judges publishability pass
  for new repos and major features.

**Forbidden outputs:**
- Calling a repo publishable without a recorded `.flywheel/PUBLISHABILITY-AUDIT.md`.
- Treating fixture-only docs as a substitute for doctor JSON and tests.
- Hiding a NO facet in prose instead of filing a targeted follow-up bead or
  explicit no-bead reason.

**Evidence:** bead `flywheel-wcq5`; rubric `.flywheel/PUBLISHABILITY-BAR.md`;
audit template `.flywheel/PUBLISHABILITY-AUDIT.md`; doctor probe
`.flywheel/scripts/publishability-bar.sh`; prompt
`~/.claude/skills/.flywheel/prompts/three-judges-rubric.md`; tests
`tests/publishability-bar.sh`.

**Companion rules:** L50 (Socraticode survey), L52 (issues-to-beads), L60
(doctor signal shape), L71 (validate-and-redispatch discipline), L80
(DID/DIDNT/GAPS), and L83 (file-length discipline).

