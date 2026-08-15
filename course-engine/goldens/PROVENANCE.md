# Goldens provenance (L3 GradeExact)

**Generated:** 2026-08-13 (re-frozen from the Rust sampler — bd-golden-sampler-divergence-09q)
**Last audited:** 2026-08-14 — 24 duplicate items retired and the draw itself re-frozen for the
first time (bd-tetz); bank_hash re-frozen to cover objective_ids/citation_ids/tags/status
(bd-hardening-c-status-hzs.2); regeneration path reduced to one (bd-z3x)
**Law:** `docs/CANONICAL.md` · `docs/ORACLE-GAUNTLET.md` · floor = 0

**Not live law.** This file is the dated re-freeze ledger for `goldens/`.
Present-tense claims about the bank content address, unknown-field reject, and
the sampler PRNG live in `docs/CANONICAL.md` / `docs/PLAN-A-TO-Z.md` and are
polarity-checked by `cdcp_gate doc-facts` against `registries/doc-facts.toml`
(`fact-hash-payload-*`, `fact-bank-item-denies-unknown-fields`,
`fact-assemble-uses-stdrng`, `fact-assemble-rng-is-chacha12`). Sentences below
are receipts of what was measured on a date.

## Which tool regenerates these files

There is exactly **one** regeneration path. It is the four-command block under
[Regenerate](#regenerate-local-only). Nothing else may write anything in `goldens/`.

| Tool | Status | What happens if you use it |
|------|--------|----------------------------|
| `cargo run -p cdcp_cli -- goldens fixture` | **AUTHORITATIVE** — the only writer of `item_ids` | Calls `cdcp_assemble::assemble()`, the shipped sampler |
| `cargo run -p cdcp_cli -- goldens generate` | **AUTHORITATIVE** — the only writer of the digests and `bank_hash.txt` | Grades the fixture's `item_ids` with `cdcp_grade`, the shipped grader |
| `cargo run -p cdcp_cli -- export-web` | **AUTHORITATIVE** — the only writer of `web/data/*_seed42.json` | Runs the sampler; `scripts/check.sh` L6 `cmp`s its output against the committed packs |
| `scripts/sample_mock.py` | **DELETED 2026-08-15 (bd-sample-mock-draws-retired-1qv9)** — do not restore it from git history | Drew from the unfiltered file set (retired included). A second sampler that disagrees with `cdcp_assemble` is a liability. Measurements below |
| `scripts/regen_goldens_after_bank.py` | **DELETED 2026-08-14 (bd-z3x)** — do not restore it from git history | It manufactured the divergence it was checked against. Measurements below |

If you are here because you edited `bank/items` and something is red: run the
[Regenerate](#regenerate-local-only) block, in order, all four commands. That is the whole
procedure. There is no shortcut and no helper script.

### Why `regen_goldens_after_bank.py` was deleted, not repaired (bd-z3x)

It advertised itself as a "pure-Python twin of `goldens generate`" that "does NOT reshuffle mock40
`item_ids` (fixture stays golden-pinned)". Every clause of that was a defect once
`golden_fixture_is_the_rust_sampler_output` started asserting the fixture equals the sampler's
output. Measured 2026-08-14 against the live 804-item bank:

| Axis | Rust (law) | The deleted script | Consequence |
|------|-----------|--------------------|-------------|
| `item_ids` after a bank change | recomputed from `cdcp_assemble` | **frozen**, never recomputed | fixture goes stale; `cargo test -p cdcp_cli` exit **101** |
| `bank_hash` on the *pristine* bank | `e82817572a82d13f…` | `af6f14fac45df4c1…` | wrote a hash the Rust law never produces — `goldens check` RED with **zero** bank edits |
| grade digest | `cdcp_grade` canonical `GradeReport` | its own re-implementation | `GOLDEN MISMATCH all-correct` — a second, silently divergent grader |
| C1 approved-only pool | enforced, anti-vacuous | the file never reads `status` at all | a bank with 804 items and 0 approved would still print `regen_goldens_after_bank ok` |
| exit code on all of the above | — | **0**, with the line `regen_goldens_after_bank ok` | a repair tool that reports success while leaving the gate RED |

Three independent re-implementations of three shipped algorithms, all drifted, all reported green.
Repairing it would have meant making it shell out to the three `cargo` commands below — a fourth
name for a procedure that already has one, in the substrate the repo is migrating off, which could
never be more correct than the commands it wrapped and would silently rot away from them again.
The footgun was the second path itself, so the second path is gone.

Deleting the file **requires** deleting its `[[allow]]` row in
`registries/substrate_allowlist.toml` in the same commit: `cdcp_gate substrate-guard` exits **4**
(`no file at this path — the allowlist is the worklist; it shrinks to zero`) until both halves
land. The two cannot drift apart.

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

Because stratification quality was equal, the divergence measured 2026-08-13 was
purely the PRNG stream (CPython MT19937 vs rand 0.8.7's then-default generator,
which happened to be ChaCha12) and there was nothing to trade away by choosing
Rust. `cdcp_assemble` wins on every other axis: it is the shipped path, it
enforces the C1 approved-only pool with anti-vacuous errors, and it survives the
Python substrate migration. `scripts/sample_mock.py` was deleted 2026-08-15
(bd-sample-mock-draws-retired-1qv9); keeping a second sampler that draws retired
items and disagrees with `cdcp_assemble` was the remaining liability. Live PRNG
law is `registries/doc-facts.toml`
(`fact-assemble-uses-stdrng`, `fact-assemble-rng-is-chacha12`).

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
  turns it RED (`cargo test` exit 101). Re-proven 2026-08-14 (bd-z3x) against the *stale-fixture*
  failure mode as well: drafting one selected item and running `goldens generate` without
  `goldens fixture` turns it RED (exit 101) while every other test stays green; deleting this one
  test with that same breakage still in the tree returns `cargo test -p cdcp_cli` to exit 0. This
  assertion — and nothing else — is what catches a stale fixture.
- `export_web_seed42_runs_the_sampler_not_the_fixture` — asserts `export-web --seed 42` reports
  `golden_pinned=false` and emits the sampler's ids. Restoring the implicit preference turns it RED.
- `export_web_explicit_fixture_flag_still_replays` — the retained bypass is opt-in (`--fixture`)
  and reports `golden_pinned=true`.

`goldens check` alone does **not** cover this: it reads `item_ids` out of the fixture and grades
them, so it stays green under a perturbed sampler (measured: exit 0). Assembly determinism is
covered by `cargo test`, which `scripts/check.sh` runs immediately before `goldens check`.

## Regenerate (local only)

Run all four, **in this order**. Order is load-bearing, see below.

```sh
UPDATE_GOLDENS=1 cargo run -p cdcp_cli -- goldens fixture --seed 42
UPDATE_GOLDENS=1 cargo run -p cdcp_cli -- goldens generate \
  --fixture goldens/fixtures/mock40_seed42.json
cargo run -p cdcp_cli -- export-web --bank bank/items --seed 42 --out web/data
# Review: git diff goldens/ web/data/
cargo run -p cdcp_cli -- goldens check
cargo test --workspace
```

The steps are one unit and `goldens fixture` **must** come first: the fixture's `item_ids` are the
input to the grade digests, so running `goldens generate` alone re-pins the digests around whatever
`item_ids` are already on disk. That is exactly the stale-fixture defect bd-z3x removed — measured
2026-08-14: `goldens check` stays exit 0 (the digests agree with the stale ids) while
`cargo test -p cdcp_cli` exits 101. A green `goldens check` is not evidence the fixture is current.

The browser packs under `web/data/` must equal `export-web --seed 42` byte-for-byte
(`scripts/check.sh` L6 `cmp`s them), so step 3 is not optional either.

**Anti-vacuous.** A regeneration run that would produce an empty or unsampled exam is an ERROR, not
a pass. Measured 2026-08-14, `goldens fixture` with `UPDATE_GOLDENS=1`:

| Input | Exit | stderr | Output file |
|-------|------|--------|-------------|
| empty bank directory | 1 | `empty bank` | not written |
| 1 item loaded, 0 `approved` | 1 | `no approved items in bank (1 items loaded, 0 approved) — an empty approved pool is an ERROR, not an empty exam` | not written |

**Never** set `UPDATE_GOLDENS=1` in CI. Flipped golden content must make `goldens check` exit
non-zero.

**Never** restore `scripts/sample_mock.py` from git history to regenerate this fixture.
That reinstates the fooled certificate; the Rust assertion above will catch it.

## Bank drift — `bank_hash` is NOT the drift detector

If `bank/items` change hashed fields, `bank_hash` changes and goldens must be regenerated with an
explicit commit reason (not silent).

**An unchanged `bank_hash` still proves nothing about the fixture** — but for a narrower reason
than it used to. C2 (below) closed the `status` hole, so a status flip now *does* move the hash.
What `bank_hash` still cannot see is the **sampler**: it addresses the pool, not the draw. A
`rand` bump, a change in `sample_item_ids`, or a stale fixture leaves `bank_hash` byte-identical
while moving `item_ids`. `golden_fixture_is_the_rust_sampler_output` — and nothing else — catches
that, which is why `goldens fixture` must run before `goldens generate`. After **any**
`bank/items` edit, run the full block above.

### C2 re-freeze — `bd-hardening-c-status-hzs.2`, 2026-08-14

The bank content-address function used to exclude `status`, and did not model
`objective_ids`, `citation_ids` or `tags` at all — serde silently discarded those
three on load, across all 804 items. C1 made the first one load-bearing: assembly
draws `approved` items only, so flipping one item `approved` → `draft` changes
what a learner can be assessed on. Measured 2026-08-14 on `m04-q129`, **before**
C2:

| Quantity | Before flip | After flip |
|----------|-------------|------------|
| `bank_hash` | `e82817572a82d13f…` | `e82817572a82d13f…` (**identical** — the defect) |
| fixture `item_ids` positions changed | — | **38 of 40** |
| ids entering/leaving the set | — | 10 (symmetric difference); `m04-q129` dropped out |

C2 (2026-08-14) folded `objective_ids`, `citation_ids`, `tags` and `status` into
the payload and made an empty bank an ERROR rather than a hash. Whether the
payload is still total over the modelled fields is the live claim in
`docs/CANONICAL.md` and `registries/doc-facts.toml`
(`fact-hash-payload-covers-objective-ids`, `fact-hash-payload-covers-status`,
`fact-bank-item-denies-unknown-fields`, `fact-hash-payload-parity-test-exists`)
— this section is the re-freeze receipt, not the law. Every pinned copy below
was re-frozen in one commit through the four-command block above plus
`cdcp content-lock` (was `UPDATE_CONTENT_LOCK=1 python3 scripts/gen_content_lock.py`):

| Pin | Before | After |
|-----|--------|-------|
| `goldens/bank_hash.txt` · fixture `bank_hash` · `content.lock` · `web/data/*_seed42.json` | `e82817572a82d13f…` | `a413d32593c954fe…` |
| `mock40_seed42_all_correct.sha256` | `7bb20d74e6308304…` | `f8d43ed62fc58bc5…` |
| `mock40_seed42_all_wrong.sha256` | `deb1de3b023c0d65…` | `ca3208b866f803f7…` |

`item_ids` are **unchanged**: the sampler's seed derivation does not read `bank_hash`, and no
item's `status` moved. Only the content address and the two grade digests (which carry
`bank_hash`) re-froze — the smallest re-freeze this change can produce.

### C3 residual re-freeze — `bd-tetz`, 2026-08-14 — **the draw moved**

24 near-duplicate items retired (`cdcp_gate near-duplicate-items` 24 pairs → 0; the rule and the
import decision are in `bank/IMPORT-POLICY.md`). The approved pool went **803 → 779**; the file
count is unchanged at 804, because retirement never deletes.

| Pin | Before | After |
|-----|--------|-------|
| `goldens/bank_hash.txt` · fixture `bank_hash` · `content.lock` · `web/data/*_seed42.json` | `173057eb9385cfc5…` | `3404d85437e3ad47…` |
| `mock40_seed42_all_correct.sha256` | `68d670e0d5c2b3d1…` | `2b0eacc9db0fe872…` |
| `mock40_seed42_all_wrong.sha256` | `81937bf032800987…` | `229ffe3daaa7467b…` |
| `web/assets/wasm/cdcp_wasm.wasm` | `d7459b7d525dc757…` | `51b45211767aaf92…` |

**Unlike C2 and unlike the single C3 retirement, `item_ids` MOVED**: all 40 positions differ, 33
ids left and 33 entered, and only 7 survived (`bank-m14-q121`, `m02-q077`, `m02-q081`, `m02-q085`,
`m05-q148`, `m09-q123`, `m13-q202`). Verified by parsing the fixture JSON and comparing the id
lists, never by counting diff lines. No retired id appears in the drawn form.

#### Why retiring `mock40-q40` alone had moved nothing — measured, not guessed

The previous wave retired one item, the pool went 804 → 803, and the fixture diff was exactly one
line. That looked like the sampler ignoring the approved filter. It is not. Measured 2026-08-14 by
re-running `export-web --bank <temp copy> --seed 42` over single-item perturbations of the live
bank and diffing the parsed `items[].id` arrays:

| Retired item | Sorted position in its module's approved list | Drawn positions changed | Ids swapped |
|---|---|---|---|
| `bank-m13-q077` | 1 of 48 | 3 of 40 | 3 |
| `bank-m14-q109` | 5 of 43 | 3 of 40 | 3 |
| `m06-q047` | 6 of 136 | 3 of 40 | 3 |
| `m09-q108` | 8 of 121 | 3 of 40 | 3 |
| `mock40-q02` | 33 of 36 | 2 of 40 | 2 |
| `m12-q219` | 54 of 63 | **0** | 0 |
| `mock40-q40` | **43 of 44 — last** | **0** | 0 |

Two facts fall out, and both matter more than the anomaly did.

1. **The filter is fine; the draw is just narrow.** `sample_item_ids` sorts each module's approved
   items by id, shuffles, and then reads only the first two or three entries of each list. A
   removal perturbs its own module's permutation from its sorted position onward, so a removal near
   the TAIL of a long list usually cannot reach the front — and `mock40-q40` sat at the very end of
   module 14's list. Zero change was the likely outcome, not a surprising one.
2. **A single removal does not shift the global PRNG stream.** If it did, every downstream module
   shuffle and the final presentation shuffle would change and essentially all 40 positions would
   move. They do not: single removals move 0–3 positions. Twenty-four removals across eleven
   modules move all 40. So the blast radius scales with the perturbation, which is what a correct
   stratified sampler should do.

**A one-line fixture diff after a bank edit is therefore not evidence that the sampler ignored the
edit.** Read `item_ids` out of the parsed JSON and compare the lists; the diff line count answers a
different question.

## PRNG (C4) — named ChaCha12, 2026-08-14

| | v1 (pre-C4) | v2 (C4) |
|---|---|---|
| Generator | rand 0.8.7 default (happened to be ChaCha12) | named ChaCha12 (`rand_chacha` 0.3.1) |
| Crate pin | workspace caret `rand = "0.8"` | `rand = "=0.8.7"` + `rand_chacha = "=0.3.1"` |
| Seed | `SeedableRng::seed_from_u64` | same seeder |
| seed-42 first 8 `u64`s | `crates/cdcp_assemble` `SEED42_FIRST_8_U64` | **identical** (measured) |
| fixture `item_ids` | then-current bank | **did not move** |

v1 happened to be ChaCha12. v2 (2026-08-14) named ChaCha12 and pinned the crate
that owns the stream. Whether the sampler still uses that named generator — and
no longer seeds from rand's non-portable default — is the live claim in
`registries/doc-facts.toml` (`fact-assemble-rng-is-chacha12`,
`fact-assemble-uses-stdrng`) and `crates/cdcp_assemble/tests/prng_c4_migration.rs`.
The known-bad is swapping the algorithm (ChaCha20 at the same seed differs on
the first `u64`).

**No four-command re-freeze was required for C4.** A future algorithm change
would move `item_ids` and then this fixture is re-frozen once, through the
block above, with a new row here.
