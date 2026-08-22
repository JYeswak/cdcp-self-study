# Installability epic acceptance audit — 2026-08-21

This is an acceptance measurement, not a close. The parent epic's success criterion is
the stranger path: a release binary and its bundle, from an empty directory, with no
source checkout or Rust toolchain available.

## Revision and checks

- Audit revision: `9f7f04e5`.
- `target/debug/cdcp --version`: `0.1.0`.
- `cdcp doctor --root <course-engine> --json`: PASS, 4/4 installed-layer probes.
- `cdcp test --root <course-engine>`: PASS, 5/5.
- `cdcp demo --no-open --root <course-engine> --bind 127.0.0.1:0`: PASS, planted
  n=40 and both digests emitted.

Those three PASS results exercise the source-checkout fallback. They do not establish
the stranger installation path.

## Isolated missing-bundle leg

From a temporary directory with `CDCP_HOME` and `XDG_DATA_HOME` pointing at empty
directories:

- `cdcp doctor --json`: rc=4, `bundle not found: <temp>/data/web`.
- `cdcp study --no-open --bind 127.0.0.1:0`: rc=4, same explicit missing-bundle path.

This is the expected fail-closed behavior when the shipped bundle is absent. It is not
evidence that a release archive can install or serve successfully.

## Boundary and disposition

The current tree contains no release `.tar.gz` plus identity manifest to test. The clean
local producer evidence in `bd-installability-sm4g.29-release-producer.md` is real, but
it was built in a detached clean worktree at source revision
`ee81a21415384589b102aff4ccf429237c5bbd2f`, not at this audit revision. No Windows or
Rust-free runtime was available, so the no-toolchain stranger leg remains unverified.

Conclusion: the installability epic remains OPEN. The source-checkout path is green;
the isolated missing-bundle failure is green for its negative leg; release installation
and a stranger completing the learner path are not established by this audit.
