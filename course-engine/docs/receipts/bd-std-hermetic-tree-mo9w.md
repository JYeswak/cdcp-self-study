# bd-std-hermetic-tree-mo9w — hermetic runner evidence

The product-owned runner is `cdcp hermetic-test`. It resolves the workspace,
rejects caller target/config/manifest overrides and target-related environment
overrides, creates `target/cdcp-hermetic/<lane>`, rejects symlinked target
paths, and fingerprints `HEAD` plus every tracked or non-ignored product input
under `bank/items`, `knowledge`, `tracks`, `crates/*/src`, `web`, and
`install.sh` before and after `cargo test`.

If the fingerprints differ after the child exits, the runner returns exit 3
with a `DRIFT:` verdict naming both source fingerprints. A passing child is not
reported GREEN when its source moved.

## In-tree proof

The product-owned runner was run from the engine root with
`./target/debug/cdcp hermetic-test --lane q23-proof -- --locked -p cdcp_cli
--bin cdcp hermetic`; all five tests passed in the lane-owned target
`target/cdcp-hermetic/q23-proof`:

```text
running 5 tests
test hermetic::tests::rejects_target_and_config_overrides ... ok
test hermetic::tests::resolves_engine_root_from_outer_git_workspace ... ok
test hermetic::tests::rejects_symlinked_target ... ok
test hermetic::tests::mid_run_product_mutation_returns_distinct_drift ... ok
test hermetic::tests::stable_source_returns_green_and_names_lane_target ... ok
test result: ok. 5 passed; 0 failed
```

The known-good test uses a temporary fake cargo that leaves the product input
unchanged and requires GREEN. The known-bad test mutates a product source file
while fake cargo is running; it requires the distinct `DRIFT:` result rather
than accepting the child's exit 0. Removing the `if before != after` comparison
would make that test fail. The other two tests prove override rejection and
symlink rejection.

## Initial wiring boundary

The initial runner commit did not edit `scripts/check.sh` or the gate-chain
files, so it was not yet the only `cargo test` path in the chain. That
historical boundary is closed below; the runner does not claim that an unrun
child is verified.

## Wiring completion — 2026-08-21

The production Cargo-test lanes are now routed through the runner:

| check.sh lane | wrapper invocation | target directory |
|---|---|---|
| corpus rights | `cdcp hermetic-test --lane check-corpus-rights -- --locked -p cdcp_data --test corpus_rights` | `target/cdcp-hermetic/check-corpus-rights` |
| workspace tests | `cdcp hermetic-test --lane check-workspace -- --locked --workspace` | `target/cdcp-hermetic/check-workspace` |
| WASM dual path | `cdcp hermetic-test --lane check-wasm-dual -- --locked -p cdcp_wasm --test dual_path -- --include-ignored` | `target/cdcp-hermetic/check-wasm-dual` |
| WASM schedule | `cdcp hermetic-test --lane check-wasm-schedule -- --locked -p cdcp_wasm --test schedule --` | `target/cdcp-hermetic/check-wasm-schedule` |

The production `scripts/check.sh` source has no direct `cargo test` command
left; it calls `cdcp hermetic-test` for those lanes.  The two remaining direct
Cargo-test families are deliberately isolated fixture harnesses
(`selftest_reconstructed.sh` and `selftest_wasm_freshness.sh`) and are not
production suite invocations; the `AGENTS.md` rule names that boundary.

The known-good, mid-run mutation, override-rejection, and symlink-rejection
legs remain the four in-tree `hermetic` tests.  Removing the before/after
comparison still makes `mid_run_product_mutation_returns_distinct_drift` fail;
the wrapper is not an always-red or always-green certificate.  The runner
returns exit 3 for `DRIFT:` and reports the lane-owned target directory, so a
completed test is attributable to one source snapshot and one target tree.

The root-resolution proof covers the engine cwd and the outer Git workspace.
The first outer-root attempt exposed the old assumption that `.git` and the
engine `Cargo.toml` shared a directory; it returned an empty-input error rather
than a stale green. The resolver now recognizes the engine anchor
(`Cargo.toml` plus `registries/claims.toml`) and accepts exactly one direct
anchored child, while ambiguous or missing children remain errors.

The wiring and instruction update are present in commit `32e63d42`. The
production check path therefore has one sanctioned runner for its Cargo test
lanes, and the wrapper's five tests cover known-good stability, in-run source
mutation (`DRIFT`), target/config override rejection, symlink rejection, and
outer-root resolution. This is evidence of the runner and its wiring, not a
claim that every downstream test assertion is meaningful.
