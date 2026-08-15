# FEATURE SURFACE

| Surface | Status | Notes |
|---------|--------|-------|
| Constitution docs | **present** | ORACLE, STANDARDS-KB, TESTING, VISUAL, OQ |
| Knowledge pack | **present** | exam_form, domains, topics, standards_* |
| Standards crosswalk | **present** | 01–15 |
| Franken rigor extract | **present** | [`research/FRANKEN-EXTRACT.md`](./research/FRANKEN-EXTRACT.md) — Assessment-System map (not ML ULP/CER) |
| Item bank library | **present** | 804 files / 779 approved / 25 retired (~20× exam) [[fact:fact-bank-item-count-804=yes]] [[fact:fact-bank-approved-count-779=yes]]. Sampler is `cdcp_assemble` (`cdcp goldens fixture`). |
| Mock sampling | **present** | seeded stratified without replacement (`ChaCha12Rng` [[fact:fact-assemble-rng-is-chacha12=yes]] [[fact:fact-assemble-uses-stdrng=no]], rand_chacha 0.3.1) |
| Rust grade oracle | **present** | L3 GradeExact: `cdcp_grade` + goldens + `check.sh` wire |
| CLI grade/goldens | **present** | `cdcp bank-hash` · `grade` · `goldens check|generate` |
| CLI export-web | **present** | `cdcp export-web --bank --seed --out` — browser packs per `web/data/README.md`; seed 42 golden-pinned, other seeds practice-only |
| CLI serve | **present** | `cdcp serve --root web --bind 127.0.0.1:8766` — local-only static server (pure std, no deps); GET/HEAD, path-traversal guarded |
| WASM dual-path | **present** | `web/assets/wasm/cdcp_wasm.wasm`; exercises `cdcp_wasm` crate |
| Hub UI | **present** | `web/index.html` + routing; served by `cdcp serve` |
| Mock exam UI | **present** | `web/mock.html`; byte-exact digest match with native grader |
| Learn reader | **present** | `web/learn.html` + `web/data/units_index.json`; 15 modules / 134 Learn units [[fact:fact-learn-unit-count-134=yes]] with TOC and micro-checks |
| Drill / short-interval review | **present** | `web/drill.html`; 1-day/3-day ladder capped at 3 days (not Anki-like SRS) + miss concept cards |
