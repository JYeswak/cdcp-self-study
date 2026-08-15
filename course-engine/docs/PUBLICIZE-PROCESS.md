# Publicize process (course-engine)

**Decision (Josh 2026-08-12, H-PUB / `bd-1m9`):** **Public target** — execute the **flywheel publishability bar** used for private→public quality on zscast, skillos, foundry-owned repos. Do **not** ad-hoc `gh` public without the bar.

## Where the process lives (Studio-canonical)

| Source | Path | Role |
|--------|------|------|
| **L88 rule** | `foundry/_retired-lrules-20260617/rules/L042-L88-publishability-bar-canonical.md` (also `.flywheel/rules/L042-L88-…` on mobile-eats, zeststream-infra, alps, CFS, …) | Doctrine: every flywheel-owned repo clears first-look bar |
| **Rubric** | `.flywheel/PUBLISHABILITY-BAR.md` (template: `zeststream-apple-toolkit/templates/`) | Seven facets F1–F7 |
| **Audit** | `.flywheel/PUBLISHABILITY-AUDIT.md` | Per-repo scored assessment + evidence |
| **Doctor probe** | `.flywheel/scripts/publishability-bar.sh` | `publishability_bar_score` in doctor JSON |
| **Flagship public proof** | `zeststream-cast` (public) | 7/7 facets, PII scrub, OSS meta |
| **Heavyweight private-client lock** | ALPS `docs/operator/publishability-checklist.md` PB-001…PB-020 | Binary receipts; **repo_lock_ready** ≠ visibility flip; still not auto-publish |
| **skillos** | private Studio repo | Same quality bar for “would Jeff publish”; skill registry publish ≠ GitHub public |

**Single-copy rule (`bd-1sd.6`):** the rubric and the L88 canonical live at
`.flywheel/PUBLISHABILITY-BAR.md` and `.flywheel/L88-PUBLISHABILITY-BAR-CANONICAL.md` — **there is no
second copy.** Duplicates previously existed under `docs/flywheel/` and were deleted 2026-08-12; two
copies of a rubric drift silently and the stale one is indistinguishable from the live one.

## L88 — what “publishable” means

From L88 (verbatim intent):

- Store rubric at `.flywheel/PUBLISHABILITY-BAR.md`
- Store assessment at `.flywheel/PUBLISHABILITY-AUDIT.md`
- Score **seven facets**: README front-door · doctrine · doctor/health/repair · executable tests · idempotent install/uninstall · code aesthetic · demo-ability
- Doctor MUST expose `publishability_bar_score` + nested evidence
- **&lt;5 warn · &lt;3 fail readiness**
- Forbidden: claim publishable without recorded audit; hide a NO facet in prose

Three-judges pass (`/flywheel:plan` Phase 3): Jeff substrate · Donella feedback · Joshua show-as-AaaS.

## Seven facets (F1–F7) — course-engine mapping

| Facet | Pass when | course-engine today (rough) |
|-------|-----------|------------------------------|
| F1 README front-door | Owner, purpose, start path, commands, operator voice | Partial — has README; needs public-grade honesty + start path |
| F2 Doctrine | AGENTS/CHARTER explain rules | Present |
| F3 Doctor triad | JSON doctor + repair path | `./scripts/check.sh` is the hard gate; expose as doctor signal |
| F4 Executable tests | Named green command | check.sh + cargo test |
| F5 Install/uninstall | Rerunnable, no orphan global state | Document clone+rustup; no global install required |
| F6 Code aesthetic | Small named modules | crates/* OK |
| F7 Demo-ability | One command sees value | `./target/debug/cdcp serve` (http://127.0.0.1:8766/) + mock seed 42 |

## Pre-public sequence (zscast-shaped)

1. Scaffold `.flywheel/{PUBLISHABILITY-BAR.md,PUBLISHABILITY-AUDIT.md,scripts/publishability-bar.sh}` from apple-toolkit template  
2. OSS meta: LICENSE, SECURITY, CONTRIBUTING, CODE_OF_CONDUCT, CHANGELOG (zscast has all)  
3. PII/private-context scrub (zscast: operator names → placeholders)  
4. Secrets scan — no Infisical values; free/public corpus only (OQ-09/10)  
5. Honesty: not EPI certified [[claim:claim-not-epi-certified]]  
6. Score audit ≥5/7 (target 7/7)  
7. Optional ALPS-style PB receipts only if we want lock-grade; **not required for a study tool** unless Josh wants that rigor  
8. **Visibility flip** (`gh repo create --public` or make public) — only after audit PASS + Josh if still escalated

## Explicit non-claims

- Publishability bar green ≠ auto-push. Visibility is a separate operator action.  
- Private status is **not** a quality exemption (ALPS PB-004).  
- skillos “publish” often means **skill registry**, not GitHub public.  
- Paid SDO PDFs never enter public tree (OQ-10 defer spend).

## Status

- **Decision:** public **target** recorded on `bd-1m9`.  
- **Execution (2026-08-12):**
  - OSS meta present (LICENSE · SECURITY · CONTRIBUTING · CoC · CHANGELOG) — `bd-2nj.1` closed  
  - `.flywheel` bar + audit + doctor **7/7 pass** (`public_repo=false`) — `bd-2nj.2` closed  
  - Working-tree secrets scrub receipt: `docs/flywheel/PREPUBLISH-SCRUB-2026-08-12.md`  
  - **Visibility flip DONE** (2026-08-12; `bd-2nj.3` closed; public at github.com/JYeswak/cdcp-self-study)


## Studio research addendum (2026-08-12)

### Private fleet truth (gh)

| Repo | Visibility |
|------|------------|
| JYeswak/SkillOS | **PRIVATE** |
| JYeswak/foundry | **PRIVATE** |
| JYeswak/flywheel | **PRIVATE** |
| JYeswak/zeststream-cast | **PRIVATE** (can still ship a *public* remote after gate — see prepublish) |

Private is normal. Public is a **gated promotion**, not the default clone state.

### Full gate stack (flywheel)

1. **L88 bar** — 7 facets + `.flywheel/PUBLISHABILITY-AUDIT.md` (≥5/7; target 7/7)
2. **Doctor probe** — `.flywheel/scripts/publishability-bar.sh` → `publishability_bar_score`
3. **Three-judges validator** — `three-judges-publishability-validator.sh` (Jeff/Donella/Joshua)
4. **Brand voice** (when public-facing) — composite ≥95, banned_words=0, or exemption
5. **Prepublish hook** — `zeststream-public-prepublish-hook` / receipts like `prepublish-public.json`  
   - Fails closed on low voice / banned words / bar shortfalls before push to a **public** remote
6. **PUBLISH-POLICY.json** (schema `flywheel.publish_policy.v1`) — after public: main freshness (default 24h), auto-merge policy, feature-branch push cadence. Solo-trust default: `auto_merge_policy=disabled`, direct-push main OK for Joshua+agents

### Prepublish fail example (fixture)

`publishability_bar_score=5` can still **fail** public push if brand_voice_composite=82 and banned words present. Bar score alone is not enough when voice gate is on.

### course-engine path (pragmatic)

Study tool, not ALPS client SaaS → use **L88 7-facet + secrets scrub + honesty + OSS meta**, not full PB-001…020 receipt mill unless you want that rigor.

Sequence:
1. Scaffold `.flywheel/` from flywheel/apple-toolkit templates  
2. Score audit to ≥5/7  
3. Optional: run prepublish-style voice/honesty scan for “certified” / dump language  
4. Add `PUBLISH-POLICY.json` when remote is public  
5. Visibility flip or dual remote `public` only after gate green  

