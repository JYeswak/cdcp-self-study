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

`cargo test -p cdcp_cli --bin cdcp hermetic --locked` was run with
`CARGO_TARGET_DIR=target/cdcp-q23`; all four tests passed:

```text
running 4 tests
test hermetic::tests::rejects_target_and_config_overrides ... ok
test hermetic::tests::rejects_symlinked_target ... ok
test hermetic::tests::mid_run_product_mutation_returns_distinct_drift ... ok
test hermetic::tests::stable_source_returns_green_and_names_lane_target ... ok
test result: ok. 4 passed; 0 failed
```

The known-good test uses a temporary fake cargo that leaves the product input
unchanged and requires GREEN. The known-bad test mutates a product source file
while fake cargo is running; it requires the distinct `DRIFT:` result rather
than accepting the child's exit 0. Removing the `if before != after` comparison
would make that test fail. The other two tests prove override rejection and
symlink rejection.

## Wiring boundary

This commit does not edit `scripts/check.sh` or the gate-chain files: pane 2
owns those paths. Therefore the runner is built and tested but is not yet the
only `cargo test` path in the chain. That wiring, plus an agent-instruction
update, remains open and must be done by the owner of those files before this
bead can close. The runner does not claim that an unrun child is verified.
