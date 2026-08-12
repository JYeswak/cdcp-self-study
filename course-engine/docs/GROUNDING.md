# GROUNDING — anti-hallucination for the item bank

## Problem

A large MCQ library is easy to fill with **plausible falsehoods** (fake clause numbers, invented setpoints, dump-shaped trivia). GradeExact only proves scoring math. It does **not** prove teaching content is true.

## Early lock-in strategy (what we do)

### 1. Citation graph first (already started)

`knowledge/sources.toml` + `standards_crosswalk.toml` + `fact_policy.toml`  
→ every domain maps to standards **families**; numbers need `quantity_evidence`.

### 2. Local grounding corpus (not pirate PDFs)

| Include | Why |
|---------|-----|
| `../modules/*.md` | Our authored study notes (primary free corpus) |
| `../reference/*.md` | Glossary + cheatsheet |
| `knowledge/*.toml` | Topic labels + family names as term dictionary |
| Fetched **public** HTML/text snapshots under `knowledge/corpus/public/` | EPI/EXIN/TIA/Uptime **public** pages, ASHRAE free WP **if** redistributable |

| Exclude | Why |
|---------|-----|
| Paid ISO/IEC/TIA/NFPA full text | License |
| Exam dump PDFs | Contamination + copyright |
| Random web scrape of “CDCP questions” | Dump risk |

**Do we download source material?**  
**Yes — only free/public, license-safe snapshots**, with `sources.toml` provenance (url, fetch_date, access). Prefer **link-only** for anything unclear (OQ-09).  
**Do we index with SocratiCode?** Optional for research; **CI grounding does not require Qdrant** — offline scripts only.

### 3. Three validator layers (fail-closed)

| Layer | Script | Catches |
|-------|--------|---------|
| **Structural** | `verify_bank.py` | schema, topics, pool size, letter balance |
| **Heuristic anti-hallucination** | `validate_grounding.py` | fake clause patterns, uncited numbers, banned dump phrases |
| **Corpus grounding** | same | stem/explanation term overlap with modules+topics; low-score → REVIEW/FAIL |

### 4. What “grounded” means (honest)

An item **passes grounding** if:

1. Structural OK  
2. No high-severity hallucination heuristics  
3. At least one of:
   - `topic_ids` terms appear in stem/explanation or mapped topic labels match keywords, **and**  
   - token overlap with parent module notes above threshold, **or**  
   - `quantity_evidence=exam_form_public` and claim is only exam form public numbers  

Items can be **true but poorly tagged** (false positive review) or **false but fluent** (false negative if heuristics miss). Heuristics are a **floor**, not a substitute for human expert review of a sample.

### 5. Best early lock-in (ordered)

1. **fact_policy + banned patterns** in CI (cheap, high value)  
2. **Module corpus overlap** gate (uses content we already wrote)  
3. **Sample human review** of 20 random items/month (process, not code)  
4. Later: optional embedding similarity vs corpus (needs models; not required for ship)  
5. Later: licensed full-text audits for specific numeric claims only  

### 6. Grade vs ground

| System | Job |
|--------|-----|
| `cdcp_grade` | Lawful score given bank_hash + answers |
| `validate_grounding` | Bank content is not inventing standards trivia |
| Human / Fluidstack domain | Interview realism |

Both must pass before calling the library “trustworthy for practice.”
