# NEGATIVE EVIDENCE

| ID | Forbid | Why |
|----|--------|-----|
| NE-PRE-01 | LLM as grade-of-record | Non-deterministic; honesty risk |
| NE-PRE-02 | Browser-only grader without Rust dual-path | False green [[claim:claim-grade-byte-exact]] |
| NE-PRE-03 | Float scores in golden digest | Platform variance |
| NE-PRE-04 | Empty bank = pass | Vacuous green |
| NE-PRE-05 | Silent UPDATE_GOLDENS in CI | Hides regressions |
| NE-PRE-06 | “CDCP certified” from mock score | Strength lattice [[claim:claim-not-epi-certified]] |
| NE-PRE-07 | HashMap order in GradeReport | Floor ≠ 0 |
| NE-PRE-08 | Ceremony dashboards nothing gates on | Doctrine #0 |
| NE-PRE-09 | Pirated ISO/TIA full-text in repo | Legal + honesty |
| NE-PRE-10 | Exam dump PDFs as bank source | Copyright + contamination |
| NE-PRE-11 | Interchange Uptime Tier ↔ TIA Rated names | Standards tension |
| NE-PRE-12 | Tokio in grade crate | WASM/dual-path contamination |
