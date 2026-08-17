# PHASE L3 — GradeExact (charter + oracle aligned)

**North star for this wave:** A pure, deterministic grader produces byte-exact `GradeReport` digests for frozen attempts; `check.sh` fails if goldens drift; known-bad fixtures trip.

**Constitution:** `CHARTER.md` §1 shipped means (3) · `ORACLE-GAUNTLET.md` L3 · floor=0 · no LLM grade.

**Loop engineering value bar:** GREEN only if product path changed (grader/CLI/goldens) **and** gate wired in `check.sh` **and** known-bad fires.

**Out of scope this phase:** WASM dual-path (L4), HTML UI (L5), SRS (L6).

---

## Preconditions (already done)

- W0 knowledge pack + standards graph
- L2 bank library 840 files / 815 approved / 25 retired (~20.4× exam on the approved pool) + `cdcp_gate verify-bank` + `validate_grounding` [[fact:fact-bank-item-count-804=yes]] [[fact:fact-bank-approved-count-779=yes]] [[fact:fact-approved-pool-multiplier-19-5=yes]]
- `cdcp_assemble` / `cdcp goldens fixture` assembly

## OQs to resolve before/with L3

| OQ | Decision for L3 |
|----|-----------------|
| OQ-02 | Canonical JSON: sorted object keys, no whitespace variance, UTF-8, integers only → `serde_json::to_vec` with sorted map or BTreeMap serialization |
| OQ-03 | `bank_hash` = SHA-256 of canonical item serialization (sorted by id); domain sep prefix `cdcp-bank-v3\0` |

Promote both to VERIFIED when implemented + tested.

---

## Stories (bead DAG)

### S1 — Resolve OQ-02 / OQ-03 (hash + canonical law)
**AC:** `OQ_REGISTER.md` marks OQ-02/03 VERIFIED with pointer to code; `content.lock` schema documented.  
**Tests:** unit tests for key order stability (HashMap order must not affect digest).

### S2 — Cargo workspace scaffold
**AC:** `Cargo.toml` workspace with crates `cdcp_core`, `cdcp_bank`, `cdcp_grade`, `cdcp_cli`; each `forbid(unsafe_code)`; `cargo test --locked` runs empty/smoke.  
**Deps:** none (can parallel S1 after).

### S3 — cdcp_core types + canonical_json + sha256
**AC:** `GradeReport`, `ExamAttempt`, `ChoiceLetter`; `canonical_json(&[u8])` / `digest_hex`; double-encode same report → identical digest.  
**Deps:** S2.

### S4 — cdcp_bank load + bank_hash
**AC:** Load all `bank/items/*.toml`; compute `bank_hash`; mismatch if item flipped.  
**Deps:** S3.

### S5 — cdcp_grade pure grade()
**AC:** `grade(bank, attempt) -> Result<GradeReport>`; wrong choice letter rejected; score_correct counts; `passed_study_signal` uses exam_form 27; weak_modules deterministic; grade twice → same digest.  
**Deps:** S4.

### S6 — Goldens + known-bad
**AC:**  
- Fixture attempt from `cdcp goldens fixture --seed 42` (`cdcp_assemble`) with all-correct answers → golden `.sha256`  
- Fixture with 0 correct → golden  
- Flipped golden → `goldens check` RED  
- `UPDATE_GOLDENS=1` regenerates locally only  
**Deps:** S5.

### S7 — CLI + check.sh wire
**AC:** `cargo build -p cdcp_cli --locked` then `./target/debug/cdcp grade ...` / `goldens check`; `scripts/check.sh` runs cargo test + goldens check; L3 scorecard written.  
**Deps:** S6.

---

## Explicit non-claims

- Completing L3 ≠ browser grading (L4)  
- Completing L3 ≠ product shipped (needs L5)  
- Study signal 27 ≠ EPI cert  

---

## Tick log template

```
mode: L1_IMPLEMENTATION | L3_REMEDIATION
product_change: <grader|goldens|check.sh>
gate: cargo test + goldens check
value_added: <one sentence>
```
