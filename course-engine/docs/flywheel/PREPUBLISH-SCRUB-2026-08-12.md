# Prepublish scrub receipt — 2026-08-12

**Bead:** `bd-2nj.3` (partial — scrub only)  
**Scope:** working tree text assets under `course-engine/` (not git history rewrite)  
**Visibility flip:** **PERFORMED** (2026-08-12; public at github.com/JYeswak/cdcp-self-study)

## Scans run

1. Private-key / AWS AKIA / api_key / password / sk- / ghp_ / Slack xox patterns  
   → **0 hits**
2. Personal emails (`@gmail.com`, `@zeststream` in docs)  
   → **0 hits**
3. Infisical token dumps / Bearer JWT  
   → **0 hits**

## Corpus policy check

- `knowledge/corpus/free-pdfs/` — 3 ASHRAE TC 9.9 PDFs (power, storage, edge) purged from HEAD and history; NIST SP 800-123 ships (US Government public domain); `.meta.toml` sidecars retained for url + sha256 grounding (OQ-09/10)

## Honesty check

- README + banners retain “not EPI/EXIN certified” [[claim:claim-not-epi-certified]]
- FEATURE_SURFACE + LEARN-v2 non_claims intact
- Completing this scrub is a study-tool hygiene signal only — not a CDCP credential.

## L88 doctor (quality bar)

```text
.flywheel/scripts/publishability-bar.sh --doctor --json
→ status=pass score=7/7 public_repo=false
```

## Completion outcome

| Step | Owner | Status |
|------|-------|--------|
| Working-tree scrub | agent | **done** this receipt |
| Optional `git log`/`gitleaks` on full history before first public remote | agent/Josh | optional |
| `gh` public create / visibility flip | **Josh** | **DONE** (2026-08-12) |
| Set audit `Public repo: yes` + re-score voice after flip | agent after flip | **DONE** (reflected in PUBLISHABILITY-AUDIT.md) |

## Verdict

- **Scrub + prepublish quality:** PASS for local tree  
- **Visibility flip:** DONE (2026-08-12; public at github.com/JYeswak/cdcp-self-study)
