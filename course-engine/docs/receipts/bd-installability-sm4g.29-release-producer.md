# bd-installability-sm4g.29 — local release producer checkpoint

## Consumer and defect

Consumer: the local release producer and the stranger-facing verifier used
before an installable artifact is staged. Feature: one local `.tar.gz` with a
single root-level `cdcp` member, checksum, and byte-bound source/tree/
dependency identity. Observed defect: a release label could be paired with
bytes from another or dirty source revision, while a digest mismatch was not
bound to the build identity. This receipt is deleted when the producer has a
clean-tree build receipt and the installed/release verifier consumes the same
identity contract.

## Implemented at `14544f66`

- `cdcp release build` refuses a dirty source tree, refuses to overwrite an
  existing archive or identity metadata, builds `cdcp_cli` with `--locked`,
  stages exactly one root-level `cdcp` member, writes `.sha256`, records full
  source/tree/dependency object ids, and verifies before reporting PASS.
- `cdcp release verify` checks the archive checksum, exact one-member archive
  shape, regular-file mode, and `cdcp_data::verify_identity_manifest` before
  extraction. It performs no network, tag, upload, or publish operation.

## Measured legs

| Leg | Result | Evidence |
|---|---|---|
| Rust release verifier unit tests | PASS | 2 tests: one-member archive accepted; one-character identity digest mismatch rejected |
| Known-good staged archive | PASS | `cdcp release verify` returned 0 and printed `release: verified archive=... member=cdcp` |
| Known-bad one-character digest | RED as required | returned nonzero: `artifact identity digest mismatch ... expected=... computed=...` |
| Dirty source refusal | RED as required | `cdcp release build --target aarch64-apple-darwin --out dist/release-byda` returned 1: `source worktree differs from cd566026...; refusing to label a dirty build` |
| Clean source build + installer selftest | BLOCKED | pane2 has intentional uncommitted source changes in the shared worktree; building would mislabel or collide with them |

The bead remains open until a quiescent clean tree permits the real
`cargo build --locked`, archive staging, and installer selftest. No release was
published or activated.
