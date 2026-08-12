# ORACLE · GAUNTLET — cdcp-course (stamp-in-stone)

**Class:** Assessment-System (not ML-System).  
**Product:** Browser mock exams + learn/drill; grade in WASM dual-path to Rust.

## One Rule

No victory while grade paths disagree, required goldens are missing, or honesty claims inflate credentials.

## Subject / Oracle / Comparator

| Identity | Implementation |
|----------|----------------|
| Oracle | `cdcp_grade` Rust, `#![forbid(unsafe_code)]` |
| Subject | Same crate → WASM (browser) |
| Bank | Content-addressed items; `bank_hash` in every report |

Assert identities distinct at comparator entry. Same `(bank_hash, exam_id, seed, answers)` → same digest.

## Byte-exact law

- `canonical_json(GradeReport)` + `sha256` — **oracle floor = 0**
- Integers only for scores; sorted keys; no timestamps in digest body
- `UPDATE_GOLDENS` local + human review only — never in CI auto-accept

## Two contracts

| Contract | Question |
|----------|----------|
| **GradeExact** | Unique lawful score? |
| **PedagogySignal** | Coverage, explanations, weak modules? (cannot loosen Exact) |

## Ladder L0–L5

| L0 | Bank + knowledge schema |
| L1 | Single-item grade |
| L2 | Seeded assemble + choice shuffle remap |
| L3 | Full GradeReport golden |
| L4 | Rust == WASM/browser |
| L5 | UI e2e digest match |

## Three pillars

(a) Integrity/conformance · (b) Surface honesty · (c) Pedagogy + **standards family map**

## Known-bad (must trip)

Empty bank · orphan item · flipped golden · dual-path lie · “certified” UI string · vacuous full-coverage

## Doctrine

G1>G2 (correctness > UI) · no counterfeit green · Doctrine #0 (artifacts only if gates branch) · skip-honest receipts

## Related

- [`STANDARDS-KB.md`](./STANDARDS-KB.md) — citation graph vs SDO full text  
- [`TESTING.md`](./TESTING.md) — per-layer epistemology  
- [`OQ_REGISTER.md`](./OQ_REGISTER.md) — no ship on unresolved OQ  

**Joshua ACK as immutable without amendment:** ________ date ________
