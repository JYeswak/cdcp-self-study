# Open Research Questions (OQ)

**Hard rule:** No wave that depends on an OQ may exit while that OQ is unresolved.  
Promote OPEN → ASSUMED/VERIFIED only by editing this file with evidence.

| ID | Question | Blocks | Status | Resolution |
|----|----------|--------|--------|------------|
| OQ-01 | Choice-shuffle PRNG algorithm native == wasm? | L4 | OPEN | Spec in assemble + dual tests |
| OQ-02 | Canonical JSON encoder rules (sorted keys library)? | L3 | **VERIFIED** | `docs/CANONICAL.md` + `cdcp_core` BTreeMap/serde compact JSON |
| OQ-03 | bank_hash: sha256 vs blake3 + domain separation? | L2/L3 | **VERIFIED** | SHA-256, prefix `cdcp-bank-v2\\0`, items sorted by id — see CANONICAL.md |
| OQ-04 | Mock: fixed 40-set vs sample-from-pool? | L3–L5 | ASSUMED | v1: fixed seed → fixed 40 from pool ≥40 |
| OQ-05 | min_items per topic for coverage green? | L6 | ASSUMED | Start 1; ratchet later |
| OQ-06 | Official EPI domain weights? | claims | FORBIDDEN unknown | Equal weight ASSUMED; never claim official blueprint |
| OQ-07 | WASM: wasm-bindgen browser + wasmtime CI? | L4 | ASSUMED | Yes preferred |
| OQ-08 | MD render: hand export vs franken_markdown? | L5 Learn | ASSUMED | Thin hand first; fmd optional W7 |
| OQ-09 | Redistribute ASHRAE free PDFs in-repo vs link-only? | R0b | OPEN | Prefer link-only until license clear |
| OQ-10 | Buy any paid standard full text for clause audits? | G1.13 | OPEN | Joshua decision |

## Claim tags (for prose)

`[VERIFIED]` in-repo primary · `[REPORTED]` public page fetch-dated · `[ASSUMED]` design choice · `[OPEN]` · `[FORBIDDEN]`
