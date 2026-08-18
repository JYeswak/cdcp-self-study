# Nightly engine receipt

schema_version: 1
skill: differential-conformance-oracles
family: cdcp_gate Python/Rust parity
math_lever: Unicode-domain closure with executable falsification
proof_gate: differential stdout/exit parity plus focused Rust tests
claim_marker: [[claim:claim-grade-byte-exact]]
ceiling_lines: 37283
status: IN_PROGRESS

## Scope and invariants

- Beads: `bd-substrate-python-gates-viu` + `bd-engine-not-gate-ar39.15`
- Restricted paths preserved: no `README`, `bank/items`, `tracks/`, CDFOM/CDFOS corpus,
  `check.sh`, or ceiling edits.
- Python oracles retained on disk:
  `validate_grounding.py`, `verify_bank.py`, `verify_coverage.py`,
  `verify_doc_consistency.py`, `verify_injection_count.py`,
  `verify_objectives.py`, `verify_orphans.py` (7).
- `bd-2m9`: OPEN; Python oracles are not dead.
- The 17 open beads are not treated as READY or shipped; no epic was closed.

## Baseline

- SHA: `d3ef1cab7f31ea87cd3bba30d8d26f3f5ef878a8`
- Local `gate_shrink`: 36864 lines / 47 files; digest
  `fnv1a64:d738c4d64f049f09`; ceiling 37283; local gate GREEN.
- CI for this SHA: no GitHub run exists (CI line count unavailable).
- Historical CI evidence remains separate: prior run reported 37472 lines versus
  the 37283 ceiling; it is not evidence for this SHA and does not authorize a
  ceiling change.

## Slice 1 — `verify_bank` Unicode decimal parity

- Bead: `bd-substrate-python-gates-viu`
- SHA: `b928da4` (`fix(cdcp-bank): close Unicode digit parity`)
- Change: added the nine Unicode-16 `Nd` blocks missing from Rust's
  `int(str)` emulation; `scripts/verify_bank.py` remains present and unchanged.
- Fixture: `crates/cdcp_gate/tests/diff_verify_bank.rs`,
  `unicode_nd_blocks_are_byte_identical_and_known_bad_is_red`.
- Known-bad RED: removing any one of the nine block starts makes Rust reject
  the mixed Unicode-digit policy value while Python still passes; the
  byte-exact differential assertion therefore fails.
- Focused proof: `cargo test --locked -p cdcp_gate --test diff_verify_bank
  unicode_nd_blocks_are_byte_identical_and_known_bad_is_red -- --exact` —
  1 passed.
- Full bank differential: 46/47 passed; the sole failure is the pre-existing
  live-tree `MANIFEST item_count 904 != loaded 957` drift. The restricted bank
  corpus was not changed.
- Local `gate_shrink`: 36891 lines / 47 files; digest
  `fnv1a64:78590d54b3a8635d`; 392 lines below ceiling 37283; GREEN.
- CI for `b928da4`: no GitHub run exists, so CI line count is unavailable; no
  same-SHA CI GREEN receipt exists and no ceiling change was made.

## Slice 2 — `verify_coverage` Unicode decimal parity

- Bead: `bd-substrate-python-gates-viu`
- SHA: `8ce04ac` (`fix(coverage): close Unicode digit parity`)
- Change: mirrored the nine Unicode-16 `Nd` blocks in the coverage port's
  `int(str)` emulation; `scripts/verify_coverage.py` remains present and
  unchanged.
- Fixture: `crates/cdcp_gate/tests/diff_verify_coverage.rs`,
  `unicode_nd_blocks_for_domain_min_are_byte_identical_and_known_bad_is_red`.
- Known-bad RED: removing any one block makes Rust reject the Unicode policy
  module value while Python reports the same module-1 shortfall; the
  byte-exact differential assertion then fails.
- Focused proof: `cargo test --locked -p cdcp_gate --test diff_verify_coverage
  unicode_nd_blocks_for_domain_min_are_byte_identical_and_known_bad_is_red
  -- --exact` — 1 passed.
- Local `gate_shrink`: 36935 lines / 47 files; digest
  `fnv1a64:b5c183f9be40fde7`; 348 lines below ceiling 37283; registry check
  GREEN after adding the required claim marker to this receipt.
- CI for `8ce04ac`: no GitHub run exists, so CI line count is unavailable; no
  same-SHA CI GREEN receipt exists and no ceiling change was made.

Each further slice will record its bead, SHA, local line count, CI line count or
explicit unavailable status, focused-test result, fixture reference, and the
known-bad RED condition it falsifies.

## Ship-test

Not yet true: the local count is below the ceiling, but CI has not been GREEN on
the eventual SHA at ceiling 37283. No ceiling reduction is authorized without
that same-SHA CI receipt.
