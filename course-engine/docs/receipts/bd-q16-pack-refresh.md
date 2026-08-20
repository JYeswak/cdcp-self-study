# bd-q16 learner-pack refresh and freshness coupling

## Finding

This was the narrow stale-pack case, not a day-wide stale pack. The committed
learner pack was refreshed by `01ac3e9c` at 03:08:17 local time. Since then the
authored bank advanced by exactly two commits, both from the hedge repair:

- `9c8ce779` at 08:27: 8 item files;
- `00406504` at 12:52: 4 item files.

The 12 changed item files were `bank-m12-q074`, `m01-q202`, `m02-q076`,
`m02-q210`, `m03-q216`, `m06-q201`, `m08-q053`, `m09-q126`, `m09-q132`,
`m10-q100`, `m10-q102`, and `m12-q200`. Earlier stem-overlap, cartoon, and
key-position repairs preceded the pack refresh and were already shipped. The
pack was therefore stale by about 9h44m, narrowly missing these 12 hedge edits.

## Refresh

The pack and goldens were regenerated from the bank at authored-bank commit
`00406504` (the newest bank commit at refresh time). The content bank hash
pinned by the new artifacts is:

`4320c3fb89a2000b4aae09406db63eb2a6f5f82034a4e868e0c6a705a39c5f60`

New golden values:

- all-correct: `26003203d24cd328c9a4716e3c1f47c304e79a4bfc13137ce16f7184e1d412b2`;
- all-wrong: `03da57cbf2e8076873ddba17fb913d81e66c960dd4d13145075d2cf2aef838bd`.

## Evidence

- `goldens check`: PASS; all-correct, all-wrong, and bank-hash pin checked.
- `check-learner-pack`: PASS; seed-42 mock has 40 items.
- `verify-content-lock`: PASS; bank hash, knowledge, module, and data pins agree.
- forced WASM `native_equals_wasm_mock40_seed42`: PASS, 1 test.
- `cdcp_gate pack-freshness` before this refresh commit: RED; bank
  `00406504c56a6a350476837e1735840ee7d93082` at epoch `1787251959` was newer
  than pack `01ac3e9c26e1f0d737fa0820241d4620c715b4f6` at epoch `1787216897`.
- after the refresh commit: GREEN; the same check sees the committed pack
  newer than the bank.

The freshness gate is intentionally not a correctness oracle. It proves that
the committed pack is not older than the newest committed bank, but it cannot
prove that regeneration copied the right content. Content-lock, golden, pack,
and WASM checks remain necessary for that claim.
