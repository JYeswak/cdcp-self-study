# Goldens provenance (L3 GradeExact)

**Generated:** 2026-08-13 (re-frozen from the Rust sampler — bd-golden-sampler-divergence-09q)
**Law:** `docs/CANONICAL.md` · `docs/ORACLE-GAUNTLET.md` · floor = 0

## Fixture

| Field | Value |
|-------|--------|
| Path | `goldens/fixtures/mock40_seed42.json` |
| Seed | 42 |
| Assembly | `UPDATE_GOLDENS=1 cargo run -p cdcp_cli -- goldens fixture --seed 42` |
| Sampler | `cdcp_assemble::assemble()` — **authoritative** |
| n_items | 40 |
| exam_id | mock40 |

Item order is the exam presentation order from `cdcp_assemble::sample_item_ids`
(seed-stable for a fixed bank snapshot and a fixed PRNG).

### The fixture is NOT a Python artifact any more

Until 2026-08-13 this fixture was produced by `python3 scripts/sample_mock.py --seed 42`
(CPython MT19937) while `cdcp_cli export-web` silently **preferred** that fixture whenever
`seed == 42`. The Rust sampler was therefore never exercised by any gate, and the golden — which
reads as evidence that the assembler is deterministic — was pinning a Python script's output.

Measured before the fix, at seed 42 against the live 804-item bank:

| Axis | Result |
|------|--------|
| Set | 37 of 40 ids differed each way; intersection was 3 ids (`m02-q085`, `bank-m15-q143`, `m13-q210`) |
| Order | 0 of those 3 shared ids sat at the same index — divergence is set-level **and** order-level |
| Stratification | **identical quality**: both spanned all 15 modules, both peaked at 3 items/module, both inside `max_per_module=8` / `min_modules=8` |
| Reproducibility | the committed fixture was reproducible by **neither** side — `sample_mock.py` re-run yielded 23/40 different ids, because the bank had drifted (fixture `bank_fingerprint` `0557953e8a49a3cf` vs live `5ff22c310349b2bd`) while `regen_goldens_after_bank.py` deliberately froze `item_ids` |

Because stratification quality was equal, the divergence was purely the PRNG stream (MT19937 vs
`StdRng`/ChaCha12) and there was nothing to trade away by choosing Rust. `cdcp_assemble` wins on
every other axis: it is the shipped path, it enforces the C1 approved-only pool with anti-vacuous
errors, and it survives the Python substrate migration. `scripts/sample_mock.py` is pinned as a
historical reference, not a regeneration path.

## Digests

| Golden | Meaning |
|--------|---------|
| `mock40_seed42_all_correct.sha256` | All 40 answers = item `correct` letter |
| `mock40_seed42_all_wrong.sha256` | All 40 answers = `correct.wrong_letter()` (`ChoiceLetter::wrong_letter`) |
| `bank_hash.txt` | Pin of `cdcp_bank` `bank_hash` (OQ-03) for this bank snapshot |

Digest = `hex(SHA-256(canonical_json(GradeReport)))`. No timestamps in body.

## What now exercises the sampler

`crates/cdcp_cli/tests/cli.rs`:

- `golden_fixture_is_the_rust_sampler_output` — asserts `assemble(bank, 42).item_ids` equals this
  fixture's `item_ids`, in order. **Known-bad proven:** perturbing the sampler's seed derivation
  turns it RED (`cargo test` exit 101).
- `export_web_seed42_runs_the_sampler_not_the_fixture` — asserts `export-web --seed 42` reports
  `golden_pinned=false` and emits the sampler's ids. Restoring the implicit preference turns it RED.
- `export_web_explicit_fixture_flag_still_replays` — the retained bypass is opt-in (`--fixture`)
  and reports `golden_pinned=true`.

`goldens check` alone does **not** cover this: it reads `item_ids` out of the fixture and grades
them, so it stays green under a perturbed sampler (measured: exit 0). Assembly determinism is
covered by `cargo test`, which `scripts/check.sh` runs immediately before `goldens check`.

## Regenerate (local only)

```sh
UPDATE_GOLDENS=1 cargo run -p cdcp_cli -- goldens fixture --seed 42
UPDATE_GOLDENS=1 cargo run -p cdcp_cli -- goldens generate \
  --fixture goldens/fixtures/mock40_seed42.json
cargo run -p cdcp_cli -- export-web --bank bank/items --seed 42 --out web/data
# Review: git diff goldens/ web/data/
cargo run -p cdcp_cli -- goldens check
cargo test --workspace
```

The three steps are one unit: the fixture's `item_ids` drive the grade digests, and the browser
packs under `web/data/` must equal `export-web --seed 42` byte-for-byte (`scripts/check.sh` L6
compares them). Regenerating one without the others leaves the chain inconsistent.

**Never** set `UPDATE_GOLDENS=1` in CI. Flipped golden content must make `goldens check` exit
non-zero.

**Never** regenerate this fixture with `scripts/sample_mock.py`. That reinstates the fooled
certificate; the Rust assertion above will catch it.

## Bank drift

If `bank/items` change load-bearing fields, `bank_hash` changes and goldens must be regenerated
with an explicit commit reason (not silent).

## PRNG (C4 interaction)

The fixture is frozen under `cdcp_assemble`'s current `StdRng` (ChaCha12), which is **not
portable** across `rand` versions. When C4 lands a portable, named PRNG, the sampler's seed-42
output changes and `golden_fixture_is_the_rust_sampler_output` goes RED. That is the intended
behaviour: C4 re-freezes this fixture **once**, deliberately, under the portable PRNG, using the
regeneration block above. Before this bead that re-freeze would have been invisible, because
`export-web` never called the sampler at seed 42 at all.
