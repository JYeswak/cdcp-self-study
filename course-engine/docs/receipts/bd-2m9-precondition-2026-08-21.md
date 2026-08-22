# bd-2m9 exit-code flip — precondition recheck — 2026-08-21

This is a verification-only receipt. It does not change exit-code mappings or
touch the pane-3-owned gate surface.

## Measured current tree

- Revision: `333397c6`.
- `git ls-files -- '*.py'`: **0** tracked Python files.
- Python files on disk under `scripts/`: **0**.
- `crates/cdcp_gate/tests/diff_*.rs`: **0** differential harnesses.
- Non-comment `python3` invocations in `scripts/check.sh`: **0**.
- Live `check.sh` gate calls use the Rust `cdcp_gate` dispatcher, including
  `verify-orphans`, `pack-freshness`, `verify-knowledge-paths`,
  `verify-coverage`, `verify-objectives`, `verify-content-lock`,
  `verify-doc-consistency`, `verify-injection-count`, and `verify-step-count`.

The remaining `python3` strings in the Rust source and shell are test fixtures,
plants, or retired-path comments; they are not tracked first-level Python
oracles or live differential consumers.

## Disposition

The migration precondition recorded in `bd-2m9` is now met: the seven remaining
gate-path Python differential oracles and their `diff_*.rs` consumers are gone.
The exit-code flip itself remains **UNSTARTED** in this receipt because Q4 is
claimed by pane 3 and the required one-commit mapping change still needs its
owned files and selftests. This receipt is the handoff measurement, not a claim
that `bd-2m9` is closed.
