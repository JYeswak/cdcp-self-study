# Publishability Bar

Every flywheel-owned repo should be good enough that Jeff, Donella, and Joshua
would understand the system, trust the feedback loops, and feel comfortable
pointing another serious builder at it on first read.

This is not a prettiness rule. It is the public-facing form of three operating
questions: is the surface validated, documented, and surfaced?

## Three Judges

| Judge | What They Look For | Pass Signal |
|---|---|---|
| Jeff | CLI surfaces, doctor/health/repair triads, schemas, fixtures, idempotency | A new operator can run the probe, see JSON, and repair safely. |
| Donella | Systems legibility, stocks/flows, leverage, feedback loops | The repo exposes what accumulates, what drains, and where intervention works. |
| Joshua | ZestStream taste, ZestStream brand voice, AaaS-grade polish, pride to publicize | The first screen and first command read as Joshua's work: first-person, evidence-grounded, specific, and free of generic agency voice. |

Meadows #3 goal-level leverage asks whether the repo participates in the
learning loop, not just whether it has code. Meadows #2 paradigm leverage asks
whether the repo makes the right mental model hard to miss. The publishability
bar is the local test for both.

## Seven Facets

Score one point for each YES. A repo passes normal readiness at 5/7, warns below
5/7, and fails readiness below 3/7.

| Facet | YES Example | NO Example |
|---|---|---|
| F1 README front-door | README names the repo owner, purpose, start path, core commands, and passes ZestStream brand voice when public. | README is stale, missing, generic, or only describes install internals. |
| F2 Doctrine clarity | AGENTS/INCIDENTS/mission docs explain the rules and why they exist. | Rules are scattered through scrollback, comments, or old dispatch files. |
| F3 Doctor/health/repair triad | There is a JSON doctor signal and a repair or explicit no-repair path. | Health is only a prose checklist or one-off command. |
| F4 Executable tests | The main behavior has fixture-backed or smoke tests with a named command. | Validation is "ran manually" with no reproducible command. |
| F5 Idempotent install + uninstall | Setup can be rerun and removed without orphaned state. | Install mutates global state with no dry-run, rollback, or uninstall story. |
| F6 Code aesthetic | Important code is small, named, local, parseable by the next agent, and reads like a Joshua-built system rather than anonymous glue. | Important behavior hides in oversized files, anonymous glue, or copied blocks. |
| F7 Demo-ability | A user can see the value in one command, screenshot, sample, or live flow, with public copy passing the ZestStream voice gate. | The repo only makes sense after an oral explanation. |

## ZestStream Voice Binding

For public ZestStream-owned repos, the Joshua judge is concrete, not abstract:

- README, MISSION, and landing copy must pass `zeststream-brand-voice` with
  composite >=95 and no dimension below 9 before publish.
- Readiness fails if composite <90, any banned word appears, or any factual
  claim is ungrounded.
- Public copy uses first-person singular for ZestStream, not corporate "we".
- Factual claims must map to
  `~/.claude/skills/zeststream-brand-voice/data/capabilities-ground-truth.yaml`.
- Trademark rendering must be exact where those marks are used:
  `The Yuzu Method ™` and `Peel. Press. Pour.™`. The Yuzu Method uses `™`
  while U.S. application SN 99572208 is pending (disclaimer response filed
  2026-05-14); switch to `®` only when the registration certificate issues.
- AI is framed as partnership, not enemy or doomer replacement.
- Jeff Emanuel's tools (NTM, Agent Mail, beads, CASS) stay attributed to Jeff.

Private/internal repos, client repos, and Jeff-owned repos are exempt from the
ZestStream public-voice gate unless they are being prepared for public release.
Client repos follow the client brand config under `brands/<slug>/`.

## Audit Contract

Each repo stores its latest assessment at `.flywheel/PUBLISHABILITY-AUDIT.md`.
The doctor signal reads that file and emits:

```json
{
  "schema_version": "publishability-bar/v1",
  "publishability_bar_score": {
    "score": 5,
    "brand_voice_composite": 100,
    "banned_words_count": 0,
    "ungrounded_claims_count": 0
  },
  "max_score": 7,
  "status": "pass"
}
```

The audit file is allowed to say NO. A visible NO is better than a hidden gap.
Follow-up beads should target specific missing facets instead of re-litigating
the bar.

## Close-Gate Enforcement

Plan Phase 5 readiness is enforced by
`.flywheel/scripts/quality-bar-close-gate.sh`. A plan may not move from
`current_phase=polish` to `current_phase=ready` unless its `STATE.json` and
`03-AUDIT-FINDINGS.md` prove `quality_bar_passed`, Jeff/Donella/Joshua scores,
composite score, and zero critical findings. The gate appends a ledger row only
with `--apply`; it does not mutate plan state.
