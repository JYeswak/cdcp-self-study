# Goldens provenance (L3 GradeExact)

**Generated:** 2026-08-13 (re-frozen from the Rust sampler — bd-golden-sampler-divergence-09q)
**Last audited:** 2026-08-14 (regeneration path reduced to one — bd-z3x)
**Law:** `docs/CANONICAL.md` · `docs/ORACLE-GAUNTLET.md` · floor = 0

## Which tool regenerates these files

There is exactly **one** regeneration path. It is the four-command block under
[Regenerate](#regenerate-local-only). Nothing else may write anything in `goldens/`.

| Tool | Status | What happens if you use it |
|------|--------|----------------------------|
| `cargo run -p cdcp_cli -- goldens fixture` | **AUTHORITATIVE** — the only writer of `item_ids` | Calls `cdcp_assemble::assemble()`, the shipped sampler |
| `cargo run -p cdcp_cli -- goldens generate` | **AUTHORITATIVE** — the only writer of the digests and `bank_hash.txt` | Grades the fixture's `item_ids` with `cdcp_grade`, the shipped grader |
| `cargo run -p cdcp_cli -- export-web` | **AUTHORITATIVE** — the only writer of `web/data/*_seed42.json` | Runs the sampler; `scripts/check.sh` L6 `cmp`s its output against the committed packs |
| `scripts/sample_mock.py` | **FORBIDDEN** on the golden path — pinned as history only | Reinstates the fooled certificate; `golden_fixture_is_the_rust_sampler_output` goes RED |
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

**Never** regenerate this fixture with `scripts/sample_mock.py`. That reinstates the fooled
certificate; the Rust assertion above will catch it.

## Bank drift — `bank_hash` is NOT the drift detector

If `bank/items` change hashed fields, `bank_hash` changes and goldens must be regenerated with an
explicit commit reason (not silent).

**But an unchanged `bank_hash` proves nothing about the fixture.** `BankItem::hash_payload`
deliberately excludes `status` (`crates/cdcp_bank/src/lib.rs` — folding it in would move the hash
for all 804 items; that migration is C2). So a *selection-only* change — flipping one item
`approved` → `draft` — changes which items the sampler may draw while leaving `bank_hash`
byte-identical. Measured 2026-08-14 on `m04-q129`:

| Quantity | Before | After |
|----------|--------|-------|
| `bank_hash` | `e82817572a82d13f…` | `e82817572a82d13f…` (**identical**) |
| fixture `item_ids` positions changed | — | **38 of 40** |
| ids entering/leaving the set | — | 10 (symmetric difference); `m04-q129` dropped out |

This is the blind spot the deleted script institutionalised: it recomputed `bank_hash`, saw the
class of change it could see, and re-pinned digests around `item_ids` it never recomputed. After
**any** `bank/items` edit — including one that touches only `status` — run the full block above.

## PRNG (C4 interaction)

The fixture is frozen under `cdcp_assemble`'s current `StdRng` (ChaCha12), which is **not
portable** across `rand` versions. When C4 lands a portable, named PRNG, the sampler's seed-42
output changes and `golden_fixture_is_the_rust_sampler_output` goes RED. That is the intended
behaviour: C4 re-freezes this fixture **once**, deliberately, under the portable PRNG, using the
regeneration block above. Before this bead that re-freeze would have been invisible, because
`export-web` never called the sampler at seed 42 at all.
