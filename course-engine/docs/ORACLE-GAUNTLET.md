# ORACLE · GAUNTLET — cdcp-course (stamp-in-stone)

**Class:** Assessment-System (not ML-System).  
**Product:** Browser mock exams + learn/drill; grade in WASM dual-path to Rust. [[claim:claim-grade-byte-exact]]

## One Rule

No victory while grade paths disagree, required goldens are missing, or honesty claims inflate credentials. [[claim:claim-not-epi-certified]]

## Subject / Oracle / Comparator

| Identity | Implementation |
|----------|----------------|
| Oracle | `cdcp_grade` Rust, `#![forbid(unsafe_code)]` |
| Subject | Same crate → WASM (browser) |
| Bank | Content-addressed items; `bank_hash` in every report |

Assert identities distinct at comparator entry. Same `(bank_hash, exam_id, seed, answers)` → same digest.

## Byte-exact law

- `canonical_json(GradeReport)` + `sha256` — **oracle floor = 0** [[claim:claim-grade-byte-exact]]
- Integers only for scores; sorted keys; no timestamps in digest body
- `UPDATE_GOLDENS` local + human review only — never in CI auto-accept

## Two contracts

| Contract | Question |
|----------|----------|
| **GradeExact** | Unique lawful score? |
| **PedagogySignal** | Coverage, explanations, weak modules? (cannot loosen Exact) [[claim:claim-domain-covered]] |

## Ladder L0–L5

| L0 | Bank + knowledge schema |
| L1 | Single-item grade |
| L2 | Seeded assemble + choice shuffle remap |
| L3 | Full GradeReport golden |
| L4 | Rust == WASM/browser |
| L5 | UI e2e digest match |

## Three pillars

(a) Integrity/conformance · (b) Surface honesty · (c) Pedagogy + **standards family map** / **14-domain** coverage oracle [[claim:claim-domain-covered]] [[claim:claim-syllabus-mapped]]

## Known-bad (must trip)

Empty bank · orphan item · flipped golden · dual-path lie · “certified” UI string · vacuous full-coverage  
[[claim:claim-not-epi-certified]] [[claim:claim-grade-byte-exact]]

Study signal **27/40** is never a credential. [[claim:claim-study-signal-27]]

## Doctrine

G1>G2 (correctness > UI) · no counterfeit green · Doctrine #0 (artifacts only if gates branch) · skip-honest receipts

## L4 — Rust == WASM dual-path (EngineIdentity)

| Role | Label (`cdcp_wasm`) | Implementation |
|------|---------------------|----------------|
| **Oracle** | `cdcp_grade-native` | Host `cdcp_grade::grade_digest` / `grade_digest_json` |
| **Subject** | `cdcp_wasm-wasm32` | Same pure path compiled to `wasm32-unknown-unknown` (`crates/cdcp_wasm`) |
| **Comparator** | assert identities **distinct**, digests **equal** | `cargo test -p cdcp_wasm --test dual_path` via wasmtime |

**Contract:** same `(bank_json, attempt_json)` → same hex SHA-256 of `canonical_json(GradeReport)`.

**Fixtures (wired):** `goldens/fixtures/mock40_seed42` × all-correct / all-wrong against full `bank/items` (digests must also pin to L3 goldens).

**Surface:** `cdcp_wasm::grade_digest_json` + C ABI (`cdcp_alloc` / `cdcp_grade_digest` / `cdcp_last_ptr`) for host runtimes. Fixed-bank grade only — assemble/shuffle dual-path waits on OQ-01 / L2 assemble readiness.

**check.sh:** optional stage — if `wasm32-unknown-unknown` + wasm build succeed, runs dual-path with `CDCP_REQUIRE_WASM=1`. Otherwise prints `SKIP wasm: toolchain missing` and does **not** claim full L4 green.

**Not L5:** browser UI e2e digest match remains open.

## Related

- [`STANDARDS-KB.md`](./STANDARDS-KB.md) — citation graph vs SDO full text  
- [`TESTING.md`](./TESTING.md) — per-layer epistemology  
- [`OQ_REGISTER.md`](./OQ_REGISTER.md) — no ship on unresolved OQ  

**Joshua ACK as immutable without amendment:** ________ date ________
