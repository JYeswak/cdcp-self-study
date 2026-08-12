# TESTING — epistemology + infra

## Per-layer “true iff”

| Layer | True iff |
|-------|----------|
| Honesty | claims-lint green; no certified language |
| Knowledge | topics enumerated; no dangling refs; knowledge_hash stable |
| Bank | schema valid; bank_hash stable; empty bank ERROR |
| Assemble | same seed → same exam; shuffle remaps key (MR) |
| GradeExact | golden digest match; double-run floor 0 |
| Dual path | native digest == wasm for frozen attempts |
| UI | e2e score matches oracle; honesty banner; exam form 40/60/27 |
| Standards map | every domain crosswalked; sources fetch-dated |

## Skills (load when implementing)

- testing-anti-patterns — never mock the grader  
- testing-golden-artifacts — exact GradeReport goldens  
- testing-conformance-harnesses — syllabus + standards MUST matrix  
- testing-metamorphic — shuffle / idempotent / subset  
- testing-fuzzing — parse_item, parse_attempt boundaries  

## Oracle hierarchy

1. Rust grade (reference)  
2. Coverage/standards shadow matrices  
3. Round-trip bank export  
4. Metamorphic relations  
5. Crash-only fuzz (insufficient alone)

## Anti-patterns

Mock dual-path · silent UPDATE_GOLDENS · vacuous empty pass · dump PDFs as oracle · LLM as grade-of-record

## Bank validation parity (Rust ⇔ verify_bank.py)

Item floors are enforced in both places so a bad item cannot load in `Bank::load_dir`
and still pass `scripts/verify_bank.py` (or the reverse for shared fields).

| Check | `Bank::load_dir` / `BankItem::validate` | `scripts/verify_bank.py` |
|-------|-------------------------------------------|---------------------------|
| empty bank / zero items | **ERROR** (`BankError::Empty`) | **ERROR** |
| non-empty `stem` | yes (trim) | yes (trim) |
| `choices` length 4 | yes | yes |
| each choice non-empty | yes (trim) | yes (strip) |
| `correct` ∈ {A,B,C,D} uppercase | yes | yes (`ALLOWED_CORRECT`) |
| `explanation` min length 12 | yes | yes |
| `topic_ids` non-empty | yes | yes (required) |
| `topic_ids` ∈ topics.toml | — (pool/registry; py-only) | yes |
| `source_class` == `original` | yes | yes |
| `quantity_evidence` allowlist | yes (default fact_policy set) | yes (+ fact_policy.toml) |
| `bloom` ∈ taxonomy | yes | yes (`ALLOWED_BLOOM`) |
| `module` parseable int | yes (serde `u32`) | yes |
| duplicate ids | yes | yes |
| pool_min / domain_min / letter diversity | — (corpus policy; py-only) | yes |
| MANIFEST item_count | — | yes if present |
| `read_dir` IO errors | fail closed (no `e.ok()` swallow) | N/A |

**Commands:** `cargo test -p cdcp_bank` (includes real `bank/items` load) and
`python3 scripts/verify_bank.py`.

## Skip policy (honest receipts)

- Tests **must not** silently `return` when a fixture or bank path is missing.
- Required fixtures: `assert!(path.is_dir(), "…")` so absence fails the suite.
- Optional fixtures only: `#[ignore = "reason"]` (reason string required).
- Override bank path with env `CDCP_BANK` when needed.
- A green suite with zero ignored tests is the default L3 receipt; ignored tests must print as skips, never as silent pass.

## Property tests (proptest)

| Property | Crate | Claim |
|----------|-------|-------|
| Equal `GradeReport` fields ⇒ same `digest_report` | `cdcp_core` | digest stability / floor-0 double-run |
| `ChoiceLetter::parse(as_str)` identity | `cdcp_core` | letter roundtrip |
| `compute_bank_hash` independent of BTreeMap insert order | `cdcp_bank` | bank_hash reorder independence |

Filesystem visit order for `Bank::load_dir` is already sorted before insert; the BTreeMap property is the load-bearing invariant under that.

