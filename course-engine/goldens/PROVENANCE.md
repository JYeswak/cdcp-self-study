# Goldens provenance (L3 GradeExact)

**Generated:** 2026-08-11  
**Law:** `docs/CANONICAL.md` · `docs/ORACLE-GAUNTLET.md` · floor = 0  

## Fixture

| Field | Value |
|-------|--------|
| Path | `goldens/fixtures/mock40_seed42.json` |
| Seed | 42 |
| Assembly | `python3 scripts/sample_mock.py --seed 42 --out goldens/fixtures/mock40_seed42.json` |
| n_items | 40 |
| exam_id | mock40 |

Item order is the exam presentation order from the sampler (seed-stable for a fixed bank snapshot).

## Digests

| Golden | Meaning |
|--------|---------|
| `mock40_seed42_all_correct.sha256` | All 40 answers = item `correct` letter |
| `mock40_seed42_all_wrong.sha256` | All 40 answers = `correct.wrong_letter()` (`ChoiceLetter::wrong_letter`) |
| `bank_hash.txt` | Pin of `cdcp_bank` `bank_hash` (OQ-03) for this bank snapshot |

Digest = `hex(SHA-256(canonical_json(GradeReport)))`. No timestamps in body.

## Regenerate (local only)

```sh
python3 scripts/sample_mock.py --seed 42 --out goldens/fixtures/mock40_seed42.json
UPDATE_GOLDENS=1 cargo run -p cdcp_cli -- goldens generate \
  --fixture goldens/fixtures/mock40_seed42.json
# Review: git diff goldens/
cargo run -p cdcp_cli -- goldens check
```

**Never** set `UPDATE_GOLDENS=1` in CI. Flipped golden content must make `goldens check` exit non-zero.

## Bank drift

If `bank/items` change load-bearing fields, `bank_hash` changes and goldens must be regenerated with an explicit commit reason (not silent).
