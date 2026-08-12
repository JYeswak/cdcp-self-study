# CANONICAL serialization (OQ-02 / OQ-03 VERIFIED)

## OQ-02 — GradeReport / JSON canonical form

| Rule | Value |
|------|--------|
| Encoding | UTF-8 |
| Object keys | Sorted lexicographically at every nesting level |
| Arrays | Schema order (item_results = exam order; do not sort by id) |
| Numbers | Integers only in GradeReport scores |
| Whitespace | Compact JSON (serde default `to_vec`) — no pretty print in digests |
| Timestamps | **Forbidden** in digest body |
| Floats | **Forbidden** in GradeReport |

**Implementation** (crate `cdcp_core`, `crates/cdcp_core/src/lib.rs`):

- `canonical_json(&T) -> Vec<u8>` — `serde_json::to_value` then recursive `sort_value` that rebuilds every object via `BTreeMap` (lexicographic keys at every nesting level), then compact `serde_json::to_vec`.
- `digest_report(&GradeReport) -> String` — `sha256_hex(canonical_json(report))` (SHA-256 hex, 64 chars).
- Struct field order on `GradeReport` / nested types is fixed in the type definitions; maps/objects still go through key-sort so insertion order cannot drift digests.
- There is **no** `cdcp_core::canonical` module or `digest_hex` alias — use `canonical_json` + `sha256_hex` / `digest_report`.

## OQ-03 — bank_hash

```text
bank_hash = hex( SHA-256(
  b"cdcp-bank-v1\0"
  || for item in items sorted by id:
       canonical_json(hash_payload(item)) || b"\0"
) )
```

Implemented in `cdcp_bank::compute_bank_hash` (`crates/cdcp_bank/src/lib.rs`). `BankItem::hash_payload` builds a `BTreeMap` with: id, module, stem, choices, correct, explanation, topic_ids (**sorted**), bloom, source_class, quantity_evidence — then `canonical_json` on that map.

Flipping any load-bearing field changes `bank_hash`.

## content.lock (example)

```toml
schema_version = 1
bank_hash = "<hex>"
knowledge_hash = "<optional later>"
canonical = "cdcp-bank-v1"
hash_alg = "sha256"
```

## UPDATE_GOLDENS

Regenerate grade digests only with `UPDATE_GOLDENS=1` locally; never auto in CI. Review `git diff goldens/` before commit.
