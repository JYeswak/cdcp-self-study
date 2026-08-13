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
| NE-PRE-13 | A topic in `topics.toml` that no bank item assesses | Coverage prose outruns the bank; wired by `scripts/verify_orphans.py` |
| NE-PRE-14 | A bank item whose `topic_ids` resolves to nothing | Item is unroutable by weak-links / micro-checks / Learn; wired by `scripts/verify_orphans.py` |
| NE-PRE-15 | Roadmap docs stating two truths about one milestone | The prose a stranger reads first; wired by `scripts/verify_doc_consistency.py` |
| NE-PRE-16 | A doc describing publication as still to come | The repo is public; wired by `scripts/verify_doc_consistency.py` |
| NE-PRE-17 | A hand-typed known-bad count nobody checks | Self-signed certificate about the anti-self-signing machinery; wired by `scripts/verify_injection_count.py` |
| NE-PRE-18 | Counting injections by grepping asserts or reading header comments | Measured: 3 suites declare zero cases in their headers and 7 hand-roll their assert idiom — a grep counter under-counts and certifies a wrong number GREEN. Suites self-report at runtime instead |
| NE-PRE-19 | A selftest suite that emits no `INJECTIONS=` receipt treated as zero | A suite that stopped reporting must not read like a suite with nothing to report |
