# Publishability Audit — cdcp-self-study

- **Repo:** cdcp-self-study (single repo; `course-engine/` is a subtree, not a separate repo)
- **Date:** 2026-08-12
- **Auditor:** agent (post-recovery pass)
- **Public repo:** no
- **Exemption:** ZestStream public-voice gate — **does not apply**. This is a
  personal educational repo, not a ZS-branded product surface (`grep -i
  zeststream README.md` returns nothing; the only ZS mention in public copy is
  the vulnerability contact in `SECURITY.md`). The ZS banned-word list is
  marketing vocabulary and flags **"mission-critical"**, which is the name of
  syllabus Module 01 ("The Mission Critical Site", `00-curriculum-map.md:13`) —
  passing it would mean renaming an EPI syllabus domain. The binding's other
  terms (first-person-singular ZS voice, claims mapping to
  `capabilities-ground-truth.yaml`, Yuzu Method trademark rendering) have no
  referent in a data-centre curriculum. Per the bar's own clause, non-ZS repos
  are exempt. **The applicable slice is still enforced mechanically** by
  `tests/voice-slop.sh` (marketing slop minus domain terms · honesty note ·
  no certification overclaim), wired into `scripts/check.sh` and bite-verified.
- **Target:** public — all agent-side gates closed; awaiting Josh's visibility flip

> **How to read this file.** `.flywheel/scripts/publishability-bar.sh` PARSES this
> markdown — it does not compute the score. A number typed here is therefore a
> *claim*, not a measurement. Every factual claim below is verified by
> `course-engine/tests/publishability-bar.sh`, which is wired into
> `scripts/check.sh` and fails the gate if a claim stops being true.
> Measured 2026-08-12: F1 previously asserted the README documented `cdcp serve`
> on :8766 when the README contained neither string, and the doctor still
> reported `pass 7/7`. That is the failure the test now prevents.

## Voice metrics (table fields for doctor)

| Field | Value |
|-------|-------|
| ZestStream voice score | null |
| Ungrounded claims count | 0 |
| Scorecard log | exempt-non-zs-repo |

**Voice gate: EXEMPT, with the reason recorded above and the applicable slice
enforced in CI.** No composite is asserted, because none was measured — the
deterministic slice (banned words + rules) was run and is clean apart from the
domain-term false positive documented in the exemption.

## Known limitation of the fleet doctor (not a repo defect)

`.flywheel/scripts/publishability-bar.sh --doctor --json` reports
**`status: fail`** with exactly one error, `brand_voice_composite_low`.

Cause: the doctor's exemption vocabulary is `EXEMPT_CLIENT_OWNED |
EXEMPT_PUBLIC_FACING` (defined only inside the script; undocumented elsewhere).
Neither class covers "not a ZestStream-branded surface", so this repo's honest
`null` composite reads as below-floor. Recording a number instead would clear it
— and would be an asserted score that was never measured, which is the exact
failure this audit exists to prevent.

The repo-local gate is authoritative and green: `scripts/check.sh` runs
`tests/voice-slop.sh` and `tests/publishability-bar.sh`, and the latter **pins
the doctor's error set**, so a new doctor error fails the build rather than
passing unnoticed.

**Fleet follow-up (Josh's call, not repo-local):** the shared doctor needs a
third exemption class, e.g. `EXEMPT_NON_ZS_SURFACE`.

## Facet scorecard

Each verdict names evidence that `tests/publishability-bar.sh` checks mechanically.

| ID | Facet | Verdict | Evidence (machine-checked unless noted) |
|----|-------|---------|------------------------------------------|
| F1 | README front-door | YES | Root `README.md` (307 lines): honesty note first, `cargo run -p cdcp_cli -- serve --bind 127.0.0.1:8766` start path, `check.sh` gate, `LICENSE` pointer, 14-module index, all relative links resolve |
| F2 | Doctrine clarity | YES | `CHARTER.md` · `course-engine/AGENTS.md` · `docs/ORACLE-GAUNTLET.md` · `docs/TESTING.md` · `docs/FEATURE_SURFACE.md` · `docs/PUBLICIZE-PROCESS.md` · `docs/loop3/PROTOCOL.md` |
| F3 | Doctor/health/repair triad | YES | `scripts/check.sh` (52 ordered steps, fail-closed, names the failing script); `.flywheel/scripts/publishability-bar.sh --doctor --json` emits `publishability_bar_score`; repair = re-run the named script |
| F4 | Executable tests | YES | `./scripts/check.sh` = 52 steps. **6 known-bad selftest suites / 21 injections, all proven RED**: `selftest_known_bad` (4), `selftest_l5_honesty`, `selftest_l6_coverage` (3), `selftest_l7_objectives` (5), `selftest_reconstructed` (5), `tests/publishability-bar.sh` (bite-verified) |
| F5 | Idempotent install + uninstall | YES | Clone + rustup + `cargo build`; no global install, no daemon, no writes outside the tree; `serve` binds loopback only; uninstall = delete the directory |
| F6 | Code aesthetic | YES | Named crates (`cdcp_core`/`bank`/`assemble`/`grade`/`wasm`/`cli`/`registry_check`); `#![forbid(unsafe_code)]`; TOML registries as constitution; `serve` on pure std with zero added deps |
| F7 | Demo-ability | YES | One command: `cd course-engine && cargo run -p cdcp_cli -- serve --bind 127.0.0.1:8766` → Hub/Learn/Drill/Mock, offline, WASM grading |

## Score

**7 / 7** on the seven facets (≥5 = readiness pass).

**Status: agent-side gates CLOSED; visibility is Josh's call.** The facet score
is a quality signal, not permission to publish. The ZS voice gate is exempt with
the reason recorded above and its applicable slice enforced in CI. Per L88, a NO
facet or an unmet gate must be surfaced rather than hidden in prose — there is
currently no NO facet and no unmet applicable gate.

## Licensing (verified by test)

| Part | Licence |
|------|---------|
| Software — `course-engine/{crates,scripts,web,fuzz}` | MIT |
| Curriculum — `modules/`, `practice/`, `reference/`, `bank/`, `knowledge/` | CC BY-NC-SA 4.0 |

Root `LICENSE` states both and carries the EPI®/EXIN®/CDCP®/ASHRAE/TIA
non-affiliation disclaimer. Root `README.md`, `CONTRIBUTING.md`,
`CODE_OF_CONDUCT.md`, `SECURITY.md` are all at the repo root where GitHub reads
them.

## IP posture (verified by test)

- `job-research/` (personal: profile, employer shortlist, compensation) — untracked, gitignored
- 3 ASHRAE TC 9.9 PDFs — purged from HEAD **and** history (0 objects); `.meta.toml`
  sidecars retained so url + sha256 grounding still verifies; re-fetch via
  `scripts/fetch_public_corpus.py`
- NIST SP 800-123 — ships (US Government work, public domain)
- Secrets — clean in tree and in full history

## Remaining before visibility flip

| Step | Owner | Status |
|------|-------|--------|
| ZS voice scorecard | agent | **closed — exempt, reason recorded, applicable slice enforced by `tests/voice-slop.sh`** |
| Corpus per-source rights column (M5) | agent | open (low) |
| `gh repo create --public` / visibility flip | **Josh** | blocked |

## Three judges

| Judge | Signal |
|-------|--------|
| Jeff | One ordered gate chain wired as THE CI step; registry-check with its own tests; 21 known-bad injections proving the gates trip; never-vacuously-green discipline (empty input = ERROR) |
| Donella | The loop is visible: claims registry → claims-lint → gate → selftests → audit → this file. Stocks (bank, knowledge, claims) and flows (export-web, grade, goldens) are named and pinned by `content.lock` |
| Joshua | Honesty note leads the README; the product refuses to overclaim (`claim-not-epi-certified` enforced by lint, not just written); the voice gate was exempted with a recorded reason and its applicable slice mechanised, rather than a composite being asserted from an unrun scorecard |
