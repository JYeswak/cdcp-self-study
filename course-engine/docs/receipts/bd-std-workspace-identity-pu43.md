# Q25 — canonical workspace identity

The first-read operating file now declares the canonical physical engine root:
`/Users/josh/cdcp-self-study/course-engine`. The `cdcp_root` product crate owns
the preflight; it canonicalizes the current path and the engine root before
comparing that root with the declaration in `AGENTS.md`. Its output names the
physical cwd, root, and declaration so path-bearing evidence can use the same
identity and Agent Mail can use the resolved root as its project key.

## Proof

`cargo test --locked -p cdcp_root`: 20 passed, 0 failed.

The binary `cargo run --locked -q -p cdcp_root --bin workspace-preflight`
returned the following results:

| specimen | result |
| --- | --- |
| engine root | `rc=0`, physical root `/Users/josh/cdcp-self-study/course-engine` |
| nested `crates/cdcp_root` directory | `rc=0`, same physical root |
| outer workspace root | `rc=0`, direct anchored `course-engine` child resolved to the same root |
| symlink entrypoint | `rc=0`, symlink normalized to the same physical root |
| different checkout carrying the original declaration | `rc=2`, `physical root mismatch`, names declared and actual roots |
| missing declaration | `rc=2`, missing `Canonical physical workspace root:` |

The different-checkout and missing-declaration cases are the known-bad
specimens. Removing the comparison/declaration cannot produce a false green:
the different checkout remains a physical-root mismatch, and an empty
operating file is a schema error. A symlink is intentionally an alias, not a
second identity.

This standard proves workspace attribution, not repository correctness. It
does not decide whether a command's inputs are the right product inputs, and
it cannot prevent a caller from bypassing the documented preflight with
`--no-verify` or a different command path; those remain separate controls.
