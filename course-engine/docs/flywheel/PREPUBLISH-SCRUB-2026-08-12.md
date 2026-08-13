# Prepublish scrub receipt — 2026-08-12

**Bead:** `bd-2nj.3` (partial — scrub only)  
**Scope:** working tree text assets under `course-engine/` (not git history rewrite)  
**Visibility flip:** **not performed** (Josh escalation)

## Scans run

1. Private-key / AWS AKIA / api_key / password / sk- / ghp_ / Slack xox patterns  
   → **0 hits**
2. Personal emails (`@gmail.com`, `@zeststream` in docs)  
   → **0 hits**
3. Infisical token dumps / Bearer JWT  
   → **0 hits**

## Corpus policy check

- `knowledge/corpus/free-pdfs/` — free/public ASHRAE power white paper + meta (OQ-09)
- No paid SDO bodies observed in tree (OQ-10)

## Honesty check

- README + banners retain “not EPI/EXIN certified” [[claim:claim-not-epi-certified]]
- FEATURE_SURFACE + LEARN-v2 non_claims intact
- Completing this scrub is a study-tool hygiene signal only — not a CDCP credential.

## L88 doctor (quality bar)

```text
.flywheel/scripts/publishability-bar.sh --doctor --json
→ status=pass score=7/7 public_repo=false
```

## Remaining for full S3 close

| Step | Owner | Status |
|------|-------|--------|
| Working-tree scrub | agent | **done** this receipt |
| Optional `git log`/`gitleaks` on full history before first public remote | agent/Josh | optional |
| `gh` public create / visibility flip | **Josh** | blocked |
| Set audit `Public repo: yes` + re-score voice after flip | agent after flip | deferred |

## Verdict (partial)

- **Scrub + prepublish quality:** PASS for local tree  
- **Visibility flip:** DEFERRED — human decision required
