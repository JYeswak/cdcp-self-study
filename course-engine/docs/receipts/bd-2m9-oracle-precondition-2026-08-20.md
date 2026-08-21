# bd-2m9 oracle-retirement precondition

Measured 2026-08-20 after queue claim `cf003fff`. This is an enumeration only;
no oracle, differential harness, gate-chain file, or registry was changed.

## Historical seven-oracle set

The Q4 acceptance and its prior precondition notes name these seven Python
differential oracles:

1. `scripts/validate_grounding.py`
2. `scripts/verify_bank.py`
3. `scripts/verify_coverage.py`
4. `scripts/verify_doc_consistency.py`
5. `scripts/verify_injection_count.py`
6. `scripts/verify_objectives.py`
7. `scripts/verify_orphans.py`

This is the set that must not be treated as retired until each corresponding
allowlist row and differential harness is retired with it.

## Current tree split

`git ls-tree -r --name-only HEAD -- scripts` and the index currently contain
these four Python oracle paths:

- `scripts/verify_bank.py`
- `scripts/verify_coverage.py`
- `scripts/verify_injection_count.py`
- `scripts/verify_objectives.py`

The worktree is not a safe retirement baseline: it has an uncommitted deletion
of `scripts/verify_injection_count.py` and an uncommitted modification of
`scripts/verify_objectives.py`, alongside pane2 changes elsewhere. The three
historical paths absent from current HEAD/index are not silently re-created or
declared retired here; their allowlist, harness, and ownership disposition
must be verified in the migration work before bd-2m9 can flip exit codes.

## Decision

The exit-code flip remains BLOCKED. No current count alone proves that the
retirement protocol is complete: the seven-path acceptance set, the committed
index, the working tree, the differential harness registry, and the allowlist
must agree before changing `cdcp_gate` behavior. The reported gate-shrink
count of 29,546 is not used to weaken or raise any ceiling here.
