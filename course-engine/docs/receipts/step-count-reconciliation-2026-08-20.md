# `check.sh` step-count reconciliation — 2026-08-20

Evaluation tree: worktree at `1a043dec356d56386c610170ac4becc6525e64e0`.
This receipt does not certify a completed chain; the worktree also contains
unrelated edits from other panes, which were not staged here.

## Definition

A step is one outer `scripts/check.sh` chain slot. A slot contributes exactly
one terminal `ok()` or one honest `skipped_step()` to the depth-0 receipt.
The advertised chain length is therefore `OK + SKIPPED` in:

```text
CHECK_STEPS=<total> OK=<n> SKIPPED=<k> NESTED_OK=<m> DEPTH=0 RUN=<id>
```

Nested `check.sh` output is not part of the outer chain. A fail-fast run that
stops before the receipt has no runtime step-count result. Diagnostic mode is
also not a substitute for that result: after a diagnostic failure, its helper
prints the failed slot but does not increment `OK`, so its successful count is
an inventory of green calls, not the chain cardinality.

## The three numbers

| measurement | value | meaning |
|---|---:|---|
| source call-site scan | 98 | `verify-step-count` finds 98 `ok "..."` sites before the single receipt boundary; this is the source-derived slot count after branch accounting |
| README before this reconciliation | 90 | stale generated prose at four sites, left over from an earlier chain shape |
| current runtime receipt | BLOCKED | the normal chain stops at `substrate_guard_e2e` step 40 (`identified=40`, `required=42`), before it can emit a depth-0 receipt |

The seven `skipped_step()` sites are not seven extra steps. Each is the
alternate arm of one existing `ok()` slot:

| slot | `ok()` arm | `skipped_step()` arm |
|---|---:|---:|
| concurrency/snapshot selftest | 782 | 468 |
| WASM freshness | 1395 | 1408 |
| SLO budgets | 1655 | 1659 |
| reconstructed stages | 1680 | 1682 |
| voice slop | 1688 | 1690 |
| publishability bar | 1708 | 1710 |
| `serve` verb | 1748 | 1752 |

Thus the source describes 98 semantic slots, not 98 plus 7. The boundary scan
also finds one receipt boundary and no `ok()` call after it. A complete normal
run is expected to emit `CHECK_STEPS=98`, with the `OK`/`SKIPPED` split depending
only on those honest branch outcomes.

## Correction and evidence

The README count is generated, not hand-maintained. The gate's
`--write-readme` path rewrites all parseable sites. It was run against a
source-structure reconstruction of `CHECK_STEPS=98` after the branch audit;
the output reported:

```text
measured_steps=98 (ok=98 skipped=0 run=source-structure-reconciliation)
ok_call_sites=98
readme_claims=[98]
regenerated ...: 4 site(s) now advertise 98
```

That preflight is deliberately not called a CI run. The resulting README
advertisement is now 98, while the actual runtime comparison remains pending
until the floor is adjudicated and a real depth-0 receipt is emitted.

The earlier diagnostic inventory's `CHECK_STEPS=82 OK=82 SKIPPED=0` is not a
contradictory third chain length: it counted only successful diagnostic calls
after red steps and was never a complete normal-chain receipt.

No `scripts/check.sh` change was made. The next complete normal run must reach
`verify-step-count`, emit `CHECK_STEPS=98`, and let that gate compare the live
receipt against the corrected README.
