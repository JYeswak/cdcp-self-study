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

## Bank validation parity (item schema vs live pool gate)

`scripts/check.sh` runs `cargo run -q -p cdcp_gate -- verify-bank` as the live
bank-pool gate. `scripts/verify_bank.py` is the differential oracle for
`tests/diff_verify_bank.rs` — it is not the command to run.

Item floors are enforced so a bad item cannot load in `Bank::load_dir`
and still pass `cdcp_gate verify-bank` (or the reverse for shared fields).

| Check | `Bank::load_dir` / `BankItem::validate` | `cdcp_gate verify-bank` (check.sh live) |
|-------|-------------------------------------------|------------------------------------------|
| empty bank / zero items | **ERROR** (`BankError::Empty`) | **ERROR** |
| non-empty `stem` | yes (trim) | yes (trim) |
| `choices` length 4 | yes | yes |
| each choice non-empty | yes (trim) | yes (strip) |
| `correct` ∈ {A,B,C,D} uppercase | yes | yes (`ALLOWED_CORRECT`) |
| `explanation` min length 12 | yes | yes |
| `topic_ids` non-empty | yes | yes (required) |
| `topic_ids` ∈ topics.toml | — (not an item-schema check) | yes |
| `source_class` == `original` | yes | yes |
| `quantity_evidence` allowlist | yes (default fact_policy set) | yes (+ fact_policy.toml) |
| `bloom` ∈ taxonomy | yes | yes (`ALLOWED_BLOOM`) |
| `module` parseable int | yes (serde `u32`) | yes |
| duplicate ids | yes | yes |
| pool_min / domain_min / letter diversity | — (not an item-schema check) | yes (approved pool) |
| MANIFEST item_count | — | yes if present |
| `read_dir` IO errors | fail closed (no `e.ok()` swallow) | fail closed |
| `status` ∈ {draft,approved,retired} | **yes** — unknown value is a load error, absent = draft (C1) | **yes** — unknown status is ERROR; absent = draft |
| unknown / unmodelled field | **yes** — load error naming the field (`deny_unknown_fields` [[fact:fact-bank-item-denies-unknown-fields=yes]], C2) | — (silently ignored; same as the oracle) |

The last row is a recorded **divergence**: `Bank::load_dir` is stricter on
unknown fields. A green `cdcp_gate verify-bank` is not evidence that a bank
file carries no content outside `bank_hash`. Status *is* checked on the live
path (the retired "py-only" reading was false after the port).

**Commands:** `cargo test -p cdcp_bank` (includes real `bank/items` load) and
`cargo run -q -p cdcp_gate -- verify-bank`.

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

## L4 — gates proven to trip (`scripts/selftest_known_bad.sh`)

Wired from `scripts/check.sh` after a clean goldens pass. Each case injects a
known-bad fixture, asserts non-zero exit, then restores the tree (trap +
explicit restore). Vacuous green is a failure.

| Case | Injection | Gate that must RED |
|------|-----------|--------------------|
| a flipped golden | rewrite `goldens/mock40_seed42_all_correct.sha256` | `cdcp goldens check` |
| b empty bank | temp empty `--bank` dir | `Bank::load_dir` / goldens check |
| c bank_hash drift | rewrite `goldens/bank_hash.txt` pin | `cdcp goldens check` pin compare |
| d honesty plant | `docs/_selftest_known_bad_planted.md` with the FORBIDDEN credential-inflation phrase | honesty scan |

### Honesty scan must not fail-open (bd-1tw)

`~/.ripgreprc` may contain `--type-not=video` (types not registered → `rg` exit 2).
Piping a failed `rg` under `set -eu` without `pipefail` used to leave the honesty
gate green. Fix:

1. Always invoke `rg --no-config` for honesty (ignore broken global type filters).
2. Treat `rg` exit ≥2 as hard FAIL (never green on scanner error).

Plant proof lives in selftest case (d); clean tree still exits 0 from `./scripts/check.sh`.


## Fuzz (cargo-fuzz) — crash-only floor

| Target | Path | Claim |
|--------|------|-------|
| `choice_letter_parse` | `fuzz/fuzz_targets/choice_letter_parse.rs` | `ChoiceLetter::parse` never panics on arbitrary UTF-8 |
| `canonical_json_bytes` | `fuzz/fuzz_targets/canonical_json_bytes.rs` | `canonical_json` never panics on arbitrary JSON Values |

**Commands:** `cargo fuzz run choice_letter_parse` · `cargo fuzz run canonical_json_bytes`  
Requires nightly + `cargo install cargo-fuzz`. Package is workspace-`exclude`d (`fuzz/`), so it
is **not** a workspace member [[fact:fact-fuzz-is-a-workspace-member=no]] and no `check.sh` step,
CI job or `cargo test --workspace` ever builds or runs these targets.

Crash-only fuzz is **insufficient alone** (see oracle hierarchy). Property tests cover digest stability / bank_hash reorder.


## L4 dual-path (`cdcp_wasm`)

| True iff | Command |
|----------|---------|
| native digest == wasm digest for mock40_seed42 all-correct/all-wrong | `CDCP_REQUIRE_WASM=1 cargo test -p cdcp_wasm --test dual_path -- --include-ignored` |
| native schedule == wasm schedule (interval + mastery) | `CDCP_REQUIRE_WASM=1 cargo test -p cdcp_wasm --test schedule` |
| EngineIdentity labels distinct at comparator | unit test in `crates/cdcp_wasm` |

Build subject: `cargo build -p cdcp_wasm --target wasm32-unknown-unknown`.  
Host runtime: wasmtime (dev-dep). Missing toolchain → skip-honest receipt in `check.sh`, not full L4 green.
