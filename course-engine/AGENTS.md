# AGENTS — cdcp-course engine

## Product

Local-first CDCP **study** tool. Not EPI certification software.

## Read first

`docs/ORACLE-GAUNTLET.md` · `docs/STANDARDS-KB.md` · `docs/OQ_REGISTER.md` · `docs/NEGATIVE_EVIDENCE.md`

## Hard rules

1. No exam dumps in `bank/`.  
2. No pirated SDO full-text PDFs.  
3. Grade path: pure Rust; browser must dual-path match.  
4. No LLM as grade-of-record.  
5. No unresolved OQ that a wave depends on.  
6. G1 > G2: correctness before UI polish.  
7. `scripts/check.sh` is the only full gate story.

## Parent corpus

Study notes live in `../modules/`, practice in `../practice/`. Import into bank; don’t duplicate without hash.

## Commits

Conventional: `feat(cdcp-course): …` / `docs(cdcp-course): …`
