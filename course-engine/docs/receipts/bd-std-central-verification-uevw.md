# bd-std-central-verification-uevw — central verification surface

## Consumer and defect class

Consumer: `cdcp_gate verify-doc-consistency`, through its product-owned
implementation in `cdcp_registry_check::verify_doc_consistency`.

Defect class: verification-command contract drift. Before this change, the
same runnable checks appeared in several operating documents with different
flags and context. A reader could not tell which invocation was authoritative,
and a later edit could leave a plausible but stale command behind.

The canonical surface is now
[`docs/VERIFICATION-MATRIX.toml`](../VERIFICATION-MATRIX.toml). Each row owns
an `id`, `scope`, exact `command`, and `required_when` condition. The matrix
owns verification commands only; learner/runtime examples such as `serve`,
`study`, `grade`, `repair`, and `export-web` remain product instructions and
are deliberately outside this contract.

## Migration measurement

The baseline was `cd566026` at the start of this tick. A
manual inventory found 17 runnable verification-command copies:

- `scripts/check.sh`: engine README and outer README (2)
- workspace preflight (1)
- gate-binary build, hook install/check, and hermetic-test instructions (6)
- registry-check build/invocation copies (4)
- goldens-check copies (2)
- restore-rebuild proof (1)
- direct workspace-test copy (1)

All 17 were replaced by pointers. There is no engine-root or repository-root
`CLAUDE.md` in this checkout, so there was no CLAUDE command copy to remove.
The live verifier scans four operating documents and reports:

```text
verification_matrix=course-engine/docs/VERIFICATION-MATRIX.toml
command_rows=14
operating_docs_scanned=4
verification_command_copies=0
```

The matrix is not a second CI driver: `scripts/check.sh` remains the only
full-chain invocation.

## Drift proof

The existing `verify-doc-consistency` implementation now validates the matrix
schema, rejects an empty matrix, scans the four operating documents, and names
the matrix row when a runnable verification copy reappears. Its committed
known-bad test changes the full-chain command to
`./scripts/check.sh --diagnostic` without changing the matrix and gets RED
naming row `full-chain`. The pointer-only fixture is GREEN; the empty-matrix
fixture is an ERROR, not a pass.

Focused proof on the hermetic lane:

```text
13 verify_doc_consistency tests passed
clippy --locked -p cdcp_registry_check --all-targets -- -D warnings: PASS
```

The live gate printed `verification_command_copies=0`. Its overall exit was
still 2 because an older local-CI receipt contains a pre-existing doc-truth
finding at line 124; that unrelated finding was not weakened or hidden.

## Boundary and deletion condition

This mechanism decides whether the declared operating-document verification
surface is centralized and non-vacuous. It cannot decide whether the command
behind a stable invocation is semantically substantive, whether `check.sh`
internals still implement the promised behavior, or whether a product runtime
command is a verification command. Those remain the responsibility of the
full chain and its focused tests.

Delete this receipt and retire the matrix only when the project has an
explicitly superseding, machine-checked canonical verification contract and
all four operating documents have migrated to that replacement; removing the
matrix while command copies remain is an ERROR.
