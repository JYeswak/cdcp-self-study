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

## Implemented at `14544f66`, selftest enforcement at `7df0d607`

- `cdcp release build` refuses a dirty source tree, refuses to overwrite an
  existing archive or identity metadata, builds `cdcp_cli` with `--locked`,
  stages exactly one root-level `cdcp` member, writes `.sha256`, records full
  source/tree/dependency object ids, verifies, and runs the existing installer
  selftest before reporting PASS.
- `cdcp release verify` checks the archive checksum, exact one-member archive
  shape, regular-file mode, and `cdcp_data::verify_identity_manifest` before
  extraction. It performs no network, tag, upload, or publish operation.

## Measured legs

| Leg | Result | Evidence |
|---|---|---|
| Rust release verifier unit tests | PASS | 3 tests: one-member archive accepted; one-character identity digest mismatch rejected; existing output refused |
| Known-good staged archive | PASS | `cdcp release verify` returned 0 and printed `release: verified archive=... member=cdcp` |
| Known-bad one-character digest | RED as required | returned nonzero: `artifact identity digest mismatch ... expected=... computed=...` |
| Dirty source refusal | RED as required | `cdcp release build --target aarch64-apple-darwin --out dist/release-byda` returned 1: `source worktree differs from cd566026...; refusing to label a dirty build` |
| Clean source build + installer selftest | BLOCKED | pane2 has intentional uncommitted source changes in the shared worktree; building would mislabel or collide with them |

The bead remains open until a quiescent clean tree permits the real
`cargo build --locked`, archive staging, and installer selftest. No release was
published or activated.

## Follow-up measurement at `bb1d773b`

After the installer-selftest enforcement was committed, the explicit-target
producer was re-run against `aarch64-apple-darwin`. It exited 1 with:
`cdcp: source worktree differs from bb1d773bf6528b0c4af0ddb53c62d3e9d6f1777e; refusing to label a dirty build`.
The refusal is the intended safety result; the clean build and installer
selftest remain unmeasured until the shared worktree is quiescent.

## Clean detached measurement at `ee81a214` (2026-08-21)

The shared checkout was left untouched. A detached temporary worktree at the
committed source SHA supplied the clean-tree leg:

- Target: `aarch64-apple-darwin`.
- Source revision: `ee81a21415384589b102aff4ccf429237c5bbd2f`.
- Source tree: `90997b0cee2e471ad4dc7dfdd604ed65dbaa9fe3`.
- `Cargo.lock` dependency identity: `ff67ffb5c09d7f6c01a77c0505b724c21343b571`.
- Archive: one root-level regular `cdcp` member; SHA-256
  `3a61827e4e6d2c2619b3abc4408f3fc259624faad7c23dda06dfbc69c0f59606`.
- `cdcp release verify` on the staged archive: PASS, exit 0.
- `scripts/selftest_install.sh`: PASS, `INJECTIONS=5 SUITE=installer`.
- One-character checksum mutation: RED, exit 1 with the expected/computed
  digest mismatch; no extraction or publication occurred.

This supplies the previously blocked clean-build leg without claiming that a
shared dirty worktree was clean. The producer still performs no tag, upload,
network, or publish operation.
