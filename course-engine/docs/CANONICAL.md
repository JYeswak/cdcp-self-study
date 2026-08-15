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
  b"cdcp-bank-v2\0"
  || for item in items sorted by id:
       canonical_json(hash_payload(item)) || b"\0"
) )
```

Implemented in `cdcp_bank::compute_bank_hash` (`crates/cdcp_bank/src/lib.rs`). `BankItem::hash_payload` builds a `BTreeMap` with: id, module, stem, choices, correct, explanation, topic_ids (**sorted**), objective_ids (**sorted**) [[fact:fact-hash-payload-covers-objective-ids=yes]], citation_ids (**sorted**), tags (**sorted**), bloom, source_class, quantity_evidence, status [[fact:fact-hash-payload-covers-status=yes]] — then `canonical_json` on that map.

The payload is **total over the modelled fields**: every field of `BankItem` appears in it, and `BankItem` carries `deny_unknown_fields` [[fact:fact-bank-item-denies-unknown-fields=yes]], so no content in a bank file sits outside the content address. `hash_payload_covers_every_modelled_field` [[fact:fact-hash-payload-parity-test-exists=yes]] asserts the two field sets are equal, so adding a field without hashing it is RED rather than silent.

Set-valued lists (`topic_ids`, `objective_ids`, `citation_ids`, `tags`) are sorted: reordering them is cosmetic and must **not** move the hash. `choices` is **not** sorted — its order is the presentation order `correct` indexes into, so permuting it is a semantic change and **does** move the hash.

An empty item set is an **ERROR** (`BankError::Empty`), not a hash. `compute_bank_hash` used to return `sha256(domain)` for an empty bank — a well-formed 64-hex digest certifying nothing.

Flipping any load-bearing field changes `bank_hash`.

**C2 (bd-hardening-c-status-hzs.2), 2026-08-14.** `objective_ids`, `citation_ids`, `tags` and `status` were added to the payload; the first three were not even fields on `BankItem` (serde silently discarded them on load, on all 804 items). Before this, flipping one item `approved` → `draft` left `bank_hash` byte-identical while changing the seed-42 selection in 38 of 40 positions — C1 restricts assembly to `approved` items, so that flip changes what a learner is assessed on. Bank hash moved `e82817572a82d13f…` → `a413d32593c954fe…` in one deliberate re-freeze; see `goldens/PROVENANCE.md`.

## content.lock (example)

```toml
schema_version = 1
bank_hash = "<hex>"
knowledge_hash = "<optional later>"
canonical = "cdcp-bank-v2"
hash_alg = "sha256"
```

## UPDATE_GOLDENS

Regenerate grade digests only with `UPDATE_GOLDENS=1` locally; never auto in CI. Review `git diff goldens/` before commit.
