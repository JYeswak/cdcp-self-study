# Nightly engine receipt

schema_version: 1
skill: differential-conformance-oracles
family: cdcp_gate Python/Rust parity
math_lever: Unicode-domain closure with executable falsification
proof_gate: differential stdout/exit parity plus focused Rust tests
claim_marker: [[claim:claim-grade-byte-exact]]
ceiling_lines: 37283
status: BLOCKED_WITH_RECEIPT

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

## Slice 3 — `validate_grounding` Unicode decimal regex parity

- Bead: `bd-substrate-python-gates-viu`
- SHA: `aa11b0f` (`fix(grounding): close Unicode digit parity`)
- Change: mirrored the nine Unicode-16 `Nd` blocks in the grounding port's
  Python `re \d` emulation; `scripts/validate_grounding.py` remains present and
  unchanged.
- Fixture: `crates/cdcp_gate/tests/diff_validate_grounding.rs`,
  `unicode_16_nd_blocks_are_byte_identical_and_known_bad_is_red`.
- Known-bad RED: removing any one block makes Rust miss the hallucinated-clause
  finding that Python emits for the all-nine-block fixture; the differential
  assertion fails.
- Focused proof: `cargo test --locked -p cdcp_gate --test
  diff_validate_grounding unicode_ -- --nocapture` — 2 passed (existing
  Unicode/casefold control plus the new all-block fixture).
- Local `gate_shrink`: 36973 lines / 47 files; digest
  `fnv1a64:06b995d06c1332d1`; 310 lines below ceiling 37283; registry check
  GREEN.
- CI for `aa11b0f`: no GitHub run exists, so CI line count is unavailable; no
  same-SHA CI GREEN receipt exists and no ceiling change was made.

## Slice 4 — `validate_grounding` Unicode CLI-number parity

- Bead: `bd-substrate-python-gates-viu`
- SHA: `e00216f` (`fix(grounding): parse Unicode CLI numbers`)
- Change: normalize Unicode decimal digits before Rust parses Python-compatible
  `float()` and `int()` option values; underscore adjacency now uses the same
  Unicode `Nd` predicate. The Python oracle remains present and unchanged.
- Fixture: `crates/cdcp_gate/tests/diff_validate_grounding.rs`,
  `unicode_nd_cli_numbers_are_byte_identical_and_known_bad_is_red`.
- Known-bad RED: ASCII-only numeric parsing makes Python pass `--min-overlap
  ١.٠ --sample-report ٢` while Rust exits with usage error; the differential
  assertion catches that mismatch.
- Focused proof: `cargo test --locked -p cdcp_gate --test
  diff_validate_grounding unicode_ -- --nocapture` — 3 passed (two Unicode
  controls plus the CLI-number fixture).
- Local `gate_shrink`: 36992 lines / 47 files; digest
  `fnv1a64:80d5b39b707e2bde`; 291 lines below ceiling 37283; registry check
  GREEN.
- CI for `e00216f`: no GitHub run exists, so CI line count is unavailable; no
  same-SHA CI GREEN receipt exists and no ceiling change was made.

## Slice 5 — `verify_doc_consistency` Unicode milestone parity

- Bead: `bd-substrate-python-gates-viu`
- SHA: `de2bd90` (`fix(doc-consistency): parse Unicode milestone digits`)
- Change: normalize all Unicode `Nd` digits in the milestone range/token
  scanner before parsing Python-compatible integer captures; the Python oracle
  remains present and unchanged.
- Fixture: `crates/cdcp_gate/tests/diff_verify_doc_consistency.rs`,
  `unicode_nd_milestone_digits_are_byte_identical_and_known_bad_is_red`.
- Known-bad RED: the ASCII-only scanner misses the nine Unicode-16 milestone
  ranges that Python parses; the byte-exact differential assertion catches the
  missing rows.
- Focused proof: `cargo test --locked -p cdcp_gate --test
  diff_verify_doc_consistency unicode_ -- --nocapture` — 2 passed (the prior
  Unicode publication control plus the new all-block milestone fixture).
- Local `gate_shrink`: 37048 lines / 47 files; digest
  `fnv1a64:32eca5ad68ca8a6c`; 235 lines below ceiling 37283; registry check
  GREEN.
- CI for `de2bd90`: no GitHub run exists, so CI line count is unavailable; no
  same-SHA CI GREEN receipt exists and no ceiling change was made.

## Slice 6 — `verify_doc_consistency` Unicode regex whitespace parity

- Bead: `bd-substrate-python-gates-viu`
- SHA: `f06ab3d` (`fix(doc-consistency): match Unicode regex whitespace`)
- Change: use Python's Unicode `\s` predicate, including ASCII information
  separators, when scanning milestone ranges; the Python oracle remains
  present and unchanged.
- Fixture: `crates/cdcp_gate/tests/diff_verify_doc_consistency.rs`,
  `unicode_regex_whitespace_in_milestone_ranges_is_byte_identical_and_known_bad_is_red`.
- Known-bad RED: the narrower Rust whitespace predicate misses the range
  around U+001F that Python matches; the differential comparison catches it.
- Focused proof: `cargo test --locked -p cdcp_gate --test
  diff_verify_doc_consistency unicode_ -- --nocapture` — 3 passed (publication
  casefold, Unicode milestone digits, and Unicode regex whitespace).
- Local `gate_shrink`: 37069 lines / 47 files; digest
  `fnv1a64:83316036bb7cedf2`; 214 lines below ceiling 37283; registry check
  GREEN.
- CI for `f06ab3d`: no GitHub run exists, so CI line count is unavailable; no
  same-SHA CI GREEN receipt exists and no ceiling change was made.

## Slice 7 — `verify_coverage` Unicode `isdigit` conflict parity

- Bead: `bd-substrate-python-gates-viu`
- SHA: `6550001` (`fix(coverage): match Unicode isdigit screening`)
- Change: mirror Python's full `str.isdigit()` screen when deciding whether a
  `domain_min` row conflicts with a recorded exemption; the seven Python
  oracles remain present and unchanged.
- Fixture: `crates/cdcp_gate/tests/diff_verify_coverage.rs`,
  `unicode_isdigit_conflicting_floor_and_exemption_are_byte_identical_and_known_bad_is_red`.
- Known-bad RED: the ASCII-only screen accepts an Arabic-Indic `domain_min`
  row as non-conflicting and can pass; Python reports the conflict and the
  differential assertion catches the mutant.
- Focused proof: `cargo test --locked -p cdcp_gate --test
  diff_verify_coverage unicode_ -- --nocapture` — 2 passed (both Unicode
  coverage fixtures).
- Local `gate_shrink`: 37113 lines / 47 files; digest
  `fnv1a64:d686bc13c1e3d88a`; 170 lines below ceiling 37283; registry check
  GREEN.
- CI for `6550001`: no GitHub run exists, so CI line count is unavailable; no
  same-SHA CI GREEN receipt exists and no ceiling change was made.

## Slice 8 — `verify_orphans` Unicode format-repr parity

- Bead: `bd-substrate-python-gates-viu`
- SHA: `7230f8f` (`fix(orphans): escape Unicode format repr`)
- Change: classify U+0890/U+0891 as non-printable in the orphan gate's Python
  `repr(str)` emulation; the Python oracle remains present and unchanged.
- Fixture: `crates/cdcp_gate/tests/diff_verify_orphans.rs`,
  `unicode_format_topic_repr_is_byte_identical_and_known_bad_is_red`.
- Known-bad RED: the printable mutant emits the Arabic format mark literally,
  while Python emits `\\u0890`; the byte-exact differential assertion catches
  the mismatch.
- Focused proof: `cargo test --locked -p cdcp_gate --test diff_verify_orphans
  unicode_format_topic_repr_is_byte_identical_and_known_bad_is_red --
  --exact` — 1 passed.
- Local `gate_shrink`: 37146 lines / 47 files; digest
  `fnv1a64:81c3e0a7eedc6c25`; 137 lines below ceiling 37283; registry check
  GREEN.
- CI for `7230f8f`: no GitHub run exists, so CI line count is unavailable; no
  same-SHA CI GREEN receipt exists and no ceiling change was made.

## Fixture correction — Python runtime Unicode version

- The original Unicode-16 code-point fixtures in Slices 1, 2, 3, and 5 were
  not valid differential inputs for the installed Python 3.11 oracle: that
  runtime does not classify those future code points as `Nd`. The Rust source
  tables remain unchanged; `dd8049d` replaces only the affected fixtures with
  Python-known non-ASCII `Nd` cases so the RED-mutant proofs are executable.
- Post-correction focused proofs are green: `diff_verify_bank` 1/1,
  `diff_verify_coverage unicode_` 2/2, `diff_validate_grounding unicode_`
  3/3, and `diff_verify_doc_consistency` 36/36. This correction does not add a
  gate slice or certify a person or product.

## Slice 9 — `verify_doc_consistency` Unicode `str.strip` parity

- Bead: `bd-substrate-python-gates-viu`
- SHA: `3697c1c` (`fix(doc-consistency): match Python Unicode strip`)
- Change: use Python's Unicode `str.strip()`/`str.lstrip()` whitespace,
  including U+001C–U+001F, across markdown row/header parsing and publication
  rendering; the Python oracle remains present and unchanged.
- Fixture: `crates/cdcp_gate/tests/diff_verify_doc_consistency.rs`,
  `unicode_strip_whitespace_in_milestone_headers_is_byte_identical_and_known_bad_is_red`.
- Known-bad RED: default Rust trim leaves U+001F around `Status`, misses the
  status column, and diverges from Python; the byte-exact assertion catches
  that mutant.
- Focused proof: `cargo test --locked -p cdcp_gate --test
  diff_verify_doc_consistency` — 36 passed, including 4 Unicode cases.
- Local `gate_shrink`: 37168 lines / 47 files; digest
  `fnv1a64:6f8d2467aaf825e4`; 115 lines below ceiling 37283; registry check
  GREEN.
- CI for `3697c1c`: `gh run list --commit 3697c1c` returned no runs, so CI
  line count is unavailable; no same-SHA CI GREEN receipt exists and no
  ceiling change was made.

## Fixture correction commit — `dd8049d`

- Bead: `bd-substrate-python-gates-viu`
- SHA: `dd8049d` (`test(cdcp-gate): align Unicode fixtures with Python`)
- Scope: test-only correction in the bank, coverage, and grounding
  differential fixtures; no gate source, Python oracle, corpus, README, or
  ceiling file changed. The corrected fixtures retain known-bad RED assertions
  while using code points Python 3.11 actually recognizes.
- Focused proof: bank 1/1, coverage Unicode 2/2, grounding Unicode 3/3.
- Local `gate_shrink`: 37168 lines / 47 files; digest
  `fnv1a64:5a826c7a8c635db9`; 115 lines below ceiling 37283; registry check
  GREEN.
- CI for `dd8049d`: `gh run list --commit dd8049d` returned no runs, so CI
  line count is unavailable; no same-SHA CI GREEN receipt exists and no
  ceiling change was made.

## Slice 10 — `verify_doc_consistency` publication-regex whitespace parity

- Bead: `bd-substrate-python-gates-viu`
- SHA: `0affc77` (`fix(doc-consistency): match Unicode publication whitespace`)
- Change: use the Python `\s` predicate, including U+001C–U+001F, in both
  whitespace runs of the publication `public repo:\s*\**\s*no` pattern; the
  Python oracle remains present and unchanged.
- Fixture: `crates/cdcp_gate/tests/diff_verify_doc_consistency.rs`,
  `unicode_regex_whitespace_in_publication_pattern_is_byte_identical_and_known_bad_is_red`.
- Known-bad RED: the narrower Rust predicate misses `public repo:` followed by
  U+001F and therefore misses Python's audit finding; the byte-exact assertion
  catches that mutant.
- Focused proof: `cargo test --locked -p cdcp_gate --test
  diff_verify_doc_consistency` — 37 passed, including 5 Unicode cases.
- Local `gate_shrink`: 37183 lines / 47 files; digest
  `fnv1a64:514d4655ad2db867`; 100 lines below ceiling 37283; registry check
  GREEN.
- CI for `0affc77`: no GitHub run exists, so CI line count is unavailable; no
  same-SHA CI GREEN receipt exists and no ceiling change was made.

## Slice 11 — `validate_grounding` Unicode negative-number parity

- Bead: `bd-substrate-python-gates-viu`
- SHA: `7b65335` (`fix(grounding): classify Unicode negative numbers`)
- Change: match Python argparse's Unicode-digit negative-number classification
  before parsing `float()`/`int()` values; `scripts/validate_grounding.py`
  remains present and unchanged.
- Fixture: `crates/cdcp_gate/tests/diff_validate_grounding.rs`,
  `unicode_nd_cli_numbers_are_byte_identical_and_known_bad_is_red`, now passes
  `--min-overlap -١.٠ --sample-report ٢`.
- Known-bad RED: an ASCII-only negative-number matcher treats `-١.٠` as an
  unknown option while Python accepts it; the byte-exact differential assertion
  catches that parser mismatch.
- Focused proof: `cargo test --locked -p cdcp_gate --test
  diff_validate_grounding unicode_ -- --nocapture` — 3 passed.
- Local `gate_shrink`: 37183 lines / 47 files; digest
  `fnv1a64:514d4655ad2db867`; 100 lines below ceiling 37283; registry check
  GREEN.
- CI for `7b65335`: no GitHub run exists, so CI line count is unavailable; no
  same-SHA CI GREEN receipt exists and no ceiling change was made.

## Slice 12 — `verify_orphans` unassigned Unicode repr parity

- Bead: `bd-substrate-python-gates-viu`
- SHA: `f61a4c1` (`fix(orphans): escape unassigned Unicode repr`)
- Change: classify the Python-3.11-unassigned U+0378/U+0379 range as
  non-printable in the orphan gate's `repr(str)` emulation; the Python oracle
  remains present and unchanged.
- Fixture: `crates/cdcp_gate/tests/diff_verify_orphans.rs`,
  `non_ascii_non_printable_status_repr_is_byte_identical` with U+0378.
- Known-bad RED: the approximation mutant emits U+0378 literally while Python
  emits `\\u0378`; the byte-exact differential assertion catches the drift.
- Focused proof: `cargo test --locked -p cdcp_gate --test diff_verify_orphans
  non_ascii_non_printable_status_repr_is_byte_identical -- --exact` — 1
  passed.
- Local `gate_shrink`: 37185 lines / 47 files; digest
  `fnv1a64:0ce5f1c5478668c1`; 98 lines below ceiling 37283; registry check
  GREEN.
- CI for `f61a4c1`: no GitHub run exists, so CI line count is unavailable; no
  same-SHA CI GREEN receipt exists and no ceiling change was made.

## Slice 13 — `verify_bank` Unicode `repr(str)` parity

- Bead: `bd-substrate-python-gates-viu`
- SHA: `d0f60d6` (`fix(bank): escape Unicode format repr`)
- Change: mirror Python's non-printable Unicode escaping for the bank gate's
  `repr(str)` path, including U+200B and the related format/private-use ranges;
  `scripts/verify_bank.py` remains present and unchanged.
- Fixture: `crates/cdcp_gate/tests/diff_verify_bank.rs`,
  `an_unmodelled_status_is_a_named_finding_in_both` with a U+200B suffix.
- Known-bad RED: the prior ASCII-only repr emits U+200B literally while Python
  emits `\\u200b`; the byte-exact differential assertion catches the mutant.
- Focused proof: `cargo test --locked -p cdcp_gate --test diff_verify_bank
  an_unmodelled_status_is_a_named_finding_in_both -- --exact` — 1 passed.
- Local `gate_shrink`: 37188 lines / 47 files; digest
  `fnv1a64:632ca00a4483e9e1`; 95 lines below ceiling 37283; registry check
  GREEN.
- CI for `d0f60d6`: no GitHub run exists, so CI line count is unavailable; no
  same-SHA CI GREEN receipt exists and no ceiling change was made.

## Slice 14 — `verify_coverage` Unicode `repr(str)` parity

- Bead: `bd-substrate-python-gates-viu`
- SHA: `1d51cef` (`fix(coverage): escape Unicode format repr`)
- Change: mirror Python's non-printable Unicode escaping for coverage error
  values, including U+200B and the related format/private-use ranges;
  `scripts/verify_coverage.py` remains present and unchanged.
- Fixture: `crates/cdcp_gate/tests/diff_verify_coverage.rs`,
  `malformed_registry_rows_and_bank_files_are_byte_identical` with U+200B in
  the malformed module value.
- Known-bad RED: the prior ASCII-only repr emits U+200B literally while Python
  emits `\\u200b`; the byte-exact differential assertion catches the mutant.
- Focused proof: `cargo test --locked -p cdcp_gate --test diff_verify_coverage
  malformed_registry_rows_and_bank_files_are_byte_identical -- --exact` — 1
  passed.
- Local `gate_shrink`: 37188 lines / 47 files; digest
  `fnv1a64:632ca00a4483e9e1`; 95 lines below ceiling 37283; registry check
  GREEN.
- CI for `1d51cef`: no GitHub run exists, so CI line count is unavailable; no
  same-SHA CI GREEN receipt exists and no ceiling change was made.

## Slice 15 — `validate_grounding` Unicode `repr(str)` parity

- Bead: `bd-substrate-python-gates-viu`
- SHA: `7570ec4` (`fix(grounding): escape Unicode format repr`)
- Change: mirror Python's non-printable Unicode escaping for grounding
  argument-error values, including U+200B and the related format/private-use
  ranges; `scripts/validate_grounding.py` remains present and unchanged.
- Fixture: `crates/cdcp_gate/tests/diff_validate_grounding.rs`,
  `the_argument_parser_matches_byte_for_byte` with `--min-overlap` U+200B.
- Known-bad RED: the prior ASCII-only repr emits U+200B literally while Python
  emits `\\u200b`; the byte-exact differential assertion catches the mutant.
- Focused proof: `cargo test --locked -p cdcp_gate --test diff_validate_grounding
  the_argument_parser_matches_byte_for_byte -- --exact` — 1 passed.
- Local `gate_shrink`: 37189 lines / 47 files; digest
  `fnv1a64:5bdb84d6803b75a4`; 94 lines below ceiling 37283; registry check
  GREEN.
- CI for `7570ec4`: no GitHub run exists, so CI line count is unavailable; no
  same-SHA CI GREEN receipt exists and no ceiling change was made.

## Slice 16 — `verify_bank` unassigned Unicode `repr(str)` parity

- Bead: `bd-substrate-python-gates-viu`
- SHA: `0c2c4b40386f2ecdd81bf1d727c5d951aac6f545`
- Change: mirror Python 3.11's non-printable `repr(str)` treatment of the
  unassigned U+0378–U+0379 range in `cdcp_bank::verify_bank`; the Python oracle
  remains present and unchanged.
- Fixture: `crates/cdcp_gate/tests/diff_verify_bank.rs`,
  `unassigned_unicode_status_repr_is_byte_identical_and_known_bad_is_red`.
- Known-bad RED: before the range was added, the fixture failed with Python
  emitting `published\\u0378` while Rust emitted literal U+0378. The focused
  test now passes and therefore proves that mutant is not accepted.
- Focused proof: `cargo test --locked -p cdcp_gate --test diff_verify_bank
  unassigned_unicode_status_repr_is_byte_identical_and_known_bad_is_red --
  --exact` — 1 passed.
- Bank unit proof: `cargo test --locked -p cdcp_bank verify_bank` — 35 passed.
- Full bank differential: 47/48 passed; the only failure is the pre-existing
  live-tree `MANIFEST item_count 904 != loaded 957` mismatch. The restricted
  bank corpus was not changed.
- Local `gate_shrink`: 37211 lines / 47 files; digest
  `fnv1a64:d7f3661cc0457278`; 72 lines below ceiling 37283; registry check
  GREEN.
- CI for this SHA: unavailable (`gh run list --commit
  0c2c4b40386f2ecdd81bf1d727c5d951aac6f545` has no run). The latest remote-main
  count remains 37472 at a different SHA, so it is not a same-SHA receipt and
  does not authorize a ceiling change.

## Slice 17 — `verify_coverage` unassigned Unicode `repr(str)` parity

- Bead: `bd-substrate-python-gates-viu`
- SHA: `0c5f60e5938a4dcbe549a95629fb7f09741e848d`
- Change: mirror Python 3.11's non-printable `repr(str)` treatment of the
  unassigned U+0378–U+0379 range in `cdcp_bank::verify_coverage`; the Python
  oracle remains present and unchanged.
- Fixture: `crates/cdcp_gate/tests/diff_verify_coverage.rs`,
  `unicode_unassigned_module_repr_is_byte_identical_and_known_bad_is_red`.
- Known-bad RED: before the range was added, the fixture failed with Python
  emitting `nope\\u0378` while Rust emitted literal U+0378. The byte-exact
  differential assertion catches that mutant.
- Focused proof: `cargo test --locked -p cdcp_gate --test
  diff_verify_coverage unicode_unassigned_module_repr_is_byte_identical_and_known_bad_is_red
  -- --exact` — 1 passed.
- Coverage unit proof: `cargo test --locked -p cdcp_bank verify_coverage` — 10
  passed.
- Full coverage differential: 27/28 passed; the only failure is the
  pre-existing anti-vacuity inventory assertion (`scanned only 7 Python
  sources`). No corpus or bank files were changed.
- Local `gate_shrink`: 37249 lines / 47 files; digest
  `fnv1a64:ae0783c1cb744742`; 34 lines below ceiling 37283; registry check
  GREEN. The ceiling was not edited.
- CI for this SHA: unavailable (`gh run list --commit
  0c5f60e5938a4dcbe549a95629fb7f09741e848d` has no run). The latest remote-main
  count remains 37472 at a different SHA, so it is not a same-SHA receipt and
  does not authorize a ceiling change.

## Slice 18 — `validate_grounding` unassigned Unicode `repr(str)` parity

- Bead: `bd-substrate-python-gates-viu`
- SHA: `2bf05a0` (`fix(cdcp-grounding): match unassigned Unicode repr`)
- Change: mirror Python 3.11's non-printable `repr(str)` treatment of the
  unassigned U+0378–U+0379 range in `cdcp_bank::validate_grounding`; the Python
  oracle remains present and unchanged.
- Fixture: the existing parser differential
  `the_argument_parser_matches_byte_for_byte` in
  `crates/cdcp_gate/tests/diff_validate_grounding.rs`, extended with
  `--min-overlap` U+0378.
- Known-bad RED: before the range was added, Python's argparse diagnostic
  emitted `invalid float value: '\\u0378'` while Rust emitted the raw U+0378
  scalar; the byte-exact assertion failed.
- Focused proof: `cargo test --locked -p cdcp_gate --test
  diff_validate_grounding the_argument_parser_matches_byte_for_byte -- --exact`
  — 1 passed after the fix.
- Full grounding differential: 25/25 passed.
- Grounding unit proof: `cargo test --locked -p cdcp_bank validate_grounding` —
  40 passed.
- Local `gate_shrink`: 37250 lines / 47 files; digest
  `fnv1a64:fb16a8c8df434ef1`; 33 lines below ceiling 37283; registry check
  GREEN. The ceiling was not edited.
- CI for this SHA: unavailable (`gh run list --commit
  2bf05a0` has no run). The latest remote-main count remains 37472 at a
  different SHA, so it is not a same-SHA receipt and does not authorize a
  ceiling change.

## Slice 19 — `verify_doc_consistency` unassigned Unicode `repr(str)` parity

- Bead: `bd-substrate-python-gates-viu`
- SHA: `1270adb` (`fix(cdcp-docs): match unassigned Unicode repr`)
- Change: mirror Python 3.11's non-printable `repr(str)` treatment of the
  unassigned U+0378–U+0379 range in the doc-consistency gate; the Python oracle
  remains present and unchanged.
- Fixture: `repr_quoting_in_status_errors_matches` in
  `crates/cdcp_gate/tests/diff_verify_doc_consistency.rs`, with U+0378 added to
  the both-quotes status error.
- Known-bad RED: before the range was added, Python emitted
  `it\\'s \\u0378 "murky"` while Rust emitted the raw U+0378 scalar; the
  byte-exact differential assertion failed.
- Focused proof: `cargo test --locked -p cdcp_gate --test
  diff_verify_doc_consistency repr_quoting_in_status_errors_matches -- --exact`
  — 1 passed after the fix.
- Full doc-consistency differential: 37/37 passed.
- Local `gate_shrink`: 37251 lines / 47 files; digest
  `fnv1a64:65647a7c5cbe8ffc`; 32 lines below ceiling 37283; registry check
  GREEN. The ceiling was not edited.
- CI for this SHA: unavailable (`gh run list --commit 1270adb` has no run).
  The latest remote-main count remains 37472 at a different SHA, so it is not
  a same-SHA receipt and does not authorize a ceiling change.

## Slice 20 — `verify_bank` adjacent unassigned Unicode `repr(str)` parity

- Bead: `bd-substrate-python-gates-viu`
- SHA: `dc745442d2f48fc6ab62c3297e6d30f285b51597`
- Change: mirror Python 3.11's non-printable `repr(str)` treatment of the
  adjacent unassigned U+0380–U+0383 range in `cdcp_bank::verify_bank`; the
  Python oracle remains present and unchanged.
- Fixture: `unassigned_unicode_status_repr_is_byte_identical_and_known_bad_is_red`
  in `crates/cdcp_gate/tests/diff_verify_bank.rs`, switched to U+0380.
- Known-bad RED: before the range was added, Python emitted
  `published\\u0380` while Rust emitted raw `published΀`; the byte-exact
  differential assertion failed.
- Focused proof: `cargo test --locked -p cdcp_gate --test diff_verify_bank
  unassigned_unicode_status_repr_is_byte_identical_and_known_bad_is_red --
  --exact` — 1 passed after the fix.
- Bank unit proof: `cargo test --locked -p cdcp_bank verify_bank` — 35 passed.
- Full bank differential: 47/48 passed; the sole failure remains the
  pre-existing live-tree `MANIFEST item_count 904 != loaded 957` mismatch. The
  restricted bank corpus was not changed.
- Local `gate_shrink`: 37251 lines / 47 files; digest
  `fnv1a64:65647a7c5cbe8ffc`; 32 lines below ceiling 37283; registry check
  GREEN. The ceiling was not edited.
- CI for this SHA: unavailable (`gh run list --commit
  dc745442d2f48fc6ab62c3297e6d30f285b51597` has no run). The historical
  37472-line result is a different SHA and does not authorize a ceiling change.

## Slice 21 — `verify_coverage` adjacent unassigned Unicode `repr(str)` parity

- Bead: `bd-substrate-python-gates-viu`
- SHA: `15c8909a5472911e9c1f609ef4d13238a4b7207e`
- Change: mirror Python 3.11's non-printable `repr(str)` treatment of the
  adjacent unassigned U+0380–U+0383 range in `cdcp_bank::verify_coverage`; the
  Python oracle remains present and unchanged.
- Fixture: `unicode_unassigned_module_repr_is_byte_identical_and_known_bad_is_red`
  in `crates/cdcp_gate/tests/diff_verify_coverage.rs`, switched to U+0380.
- Known-bad RED: before the range was added, Python emitted
  `nope\\u0380` while Rust emitted raw `nope΀`; the byte-exact differential
  assertion failed.
- Focused proof: `cargo test --locked -p cdcp_gate --test
  diff_verify_coverage unicode_unassigned_module_repr_is_byte_identical_and_known_bad_is_red
  -- --exact` — 1 passed after the fix.
- Coverage unit proof: `cargo test --locked -p cdcp_bank verify_coverage` — 10
  passed.
- Full coverage differential: 27/28 passed; the sole failure remains the
  pre-existing anti-vacuity inventory assertion (`scanned only 7 Python
  sources`). No corpus or bank files were changed.
- Local `gate_shrink`: 37251 lines / 47 files; digest
  `fnv1a64:65647a7c5cbe8ffc`; 32 lines below ceiling 37283; registry check
  GREEN. The ceiling was not edited.
- CI for this SHA: unavailable (`gh run list --commit
  15c8909a5472911e9c1f609ef4d13238a4b7207e` has no run). The historical
  37472-line result is a different SHA and does not authorize a ceiling change.

## Slice 22 — `validate_grounding` adjacent unassigned Unicode `repr(str)` parity

- Bead: `bd-substrate-python-gates-viu`
- SHA: `91c3af3858d36090baa7b8be5852939afff0f917`
- Change: mirror Python 3.11's non-printable `repr(str)` treatment of the
  adjacent unassigned U+0380–U+0383 range in `cdcp_bank::validate_grounding`;
  the Python oracle remains present and unchanged.
- Fixture: the existing `the_argument_parser_matches_byte_for_byte`
  differential in `crates/cdcp_gate/tests/diff_validate_grounding.rs`, switched
  its invalid `--min-overlap` value to U+0380.
- Known-bad RED: before the range was added, Python's argparse diagnostic
  emitted `invalid float value: '\\u0380'` while Rust emitted raw U+0380; the
  byte-exact assertion failed.
- Focused proof: `cargo test --locked -p cdcp_gate --test
  diff_validate_grounding the_argument_parser_matches_byte_for_byte -- --exact`
  — 1 passed after the fix.
- Full grounding differential: 25/25 passed.
- Grounding unit proof: `cargo test --locked -p cdcp_bank validate_grounding` —
  40 passed.
- Local `gate_shrink`: 37251 lines / 47 files; digest
  `fnv1a64:65647a7c5cbe8ffc`; 32 lines below ceiling 37283; registry check
  GREEN. The ceiling was not edited.
- CI for this SHA: unavailable (`gh run list --commit
  91c3af3858d36090baa7b8be5852939afff0f917` has no run). The historical
  37472-line result is a different SHA and does not authorize a ceiling change.

## Slice 23 — `verify_doc_consistency` adjacent unassigned Unicode `repr(str)` parity

- Bead: `bd-substrate-python-gates-viu`
- SHA: `584cf15e084ecf2816f78213449fb73e9fe242ca`
- Change: mirror Python 3.11's non-printable `repr(str)` treatment of the
  adjacent unassigned U+0380–U+0383 range in the doc-consistency gate; the
  Python oracle remains present and unchanged.
- Fixture: `repr_quoting_in_status_errors_matches` in
  `crates/cdcp_gate/tests/diff_verify_doc_consistency.rs`, switched its
  both-quotes status value to U+0380.
- Known-bad RED: before the range was added, Python emitted
  `it\\'s \\u0380 "murky"` while Rust emitted raw U+0380; the byte-exact
  differential assertion failed.
- Focused proof: `cargo test --locked -p cdcp_gate --test
  diff_verify_doc_consistency repr_quoting_in_status_errors_matches -- --exact`
  — 1 passed after the fix.
- Full doc-consistency differential: 37/37 passed.
- Local `gate_shrink`: 37252 lines / 47 files; digest
  `fnv1a64:e119332a48d14afa`; 31 lines below ceiling 37283; registry check
  GREEN. The ceiling was not edited.
- CI for this SHA: unavailable (`gh run list --commit
  584cf15e084ecf2816f78213449fb73e9fe242ca` has no run). The historical
  37472-line result is a different SHA and does not authorize a ceiling change.

## Slice 24 — `verify_orphans` adjacent unassigned Unicode `repr(str)` parity

- Bead: `bd-substrate-python-gates-viu`
- SHA: `7298fb3571fdc607b01c9011dbac7111e8d2af06`
- Change: mirror Python 3.11's non-printable `repr(str)` treatment of the
  adjacent unassigned U+0380–U+0383 range in `cdcp_bank::orphans`; the Python
  oracle remains present and unchanged.
- Fixture: `unicode_format_topic_repr_is_byte_identical_and_known_bad_is_red`
  in `crates/cdcp_gate/tests/diff_verify_orphans.rs`, switched to U+0380.
- Known-bad RED: before the range was added, Python emitted
  `zz-\\u0380` while Rust emitted raw `zz-΀`; the byte-exact differential
  assertion failed.
- Focused proof: `cargo test --locked -p cdcp_gate --test diff_verify_orphans
  unicode_format_topic_repr_is_byte_identical_and_known_bad_is_red -- --exact`
  — 1 passed after the fix.
- Full orphan differential: 10/10 passed.
- Orphan unit proof: `cargo test --locked -p cdcp_bank orphans` — 38 passed.
- Local `gate_shrink`: 37252 lines / 47 files; digest
  `fnv1a64:e119332a48d14afa`; 31 lines below ceiling 37283; registry check
  GREEN. The ceiling was not edited.
- CI for this SHA: unavailable (`gh run list --commit
  7298fb3571fdc607b01c9011dbac7111e8d2af06` has no run). The historical
  37472-line result is a different SHA and does not authorize a ceiling change.

## Slice 25 — coverage inventory floor matches seven retained Python oracles

- Bead: `bd-substrate-python-gates-viu`
- SHA: `70d1b69b2bab1204ba9c1637d9fe0bfddc663267`
- Change: correct the anti-vacuity inventory assertion in
  `diff_verify_coverage.rs` from eight sources to the authoritative seven
  retained Python oracles. No oracle was deleted or added.
- Known-bad RED: before the correction,
  `no_bank_loader_iterates_items_without_a_zero_yield_guard` REDed with
  `scanned only 7 python sources`; the existing
  `the_zero_yield_scan_fires_on_the_known_bad_and_stays_quiet_on_the_known_good`
  detector remained the falsification fixture and passed after the correction.
- Focused proofs: both of those tests — 1 passed each after the fix.
- Full coverage differential: 28/28 passed.
- Local `gate_shrink`: 37252 lines / 47 files; digest
  `fnv1a64:e119332a48d14afa`; 31 lines below ceiling 37283; registry check
  GREEN. The ceiling was not edited.
- CI for this SHA: unavailable (`gh run list --commit
  70d1b69b2bab1204ba9c1637d9fe0bfddc663267` has no run). The historical
  37472-line result is a different SHA and does not authorize a ceiling change.

## Blocker audit — audited code tree

- Audited code SHA: `70d1b69b2bab1204ba9c1637d9fe0bfddc663267`; the receipt
  update below changes no gate source.
- The live worktree measures 37251 lines / 47 files, digest
  `fnv1a64:65647a7c5cbe8ffc`. Pane-owned dirty files were preserved and are not
  used as a CI claim.
- `origin/main` is still `5f178d2a7730a82d212d8ec2e96244bae5c99050`, measuring
  37472 / 47, digest `fnv1a64:67f95ea56dbda888`. Its latest completed check run
  (`32095229956`) failed at that old SHA's gate-shrink count; it is not a
  same-SHA result for this branch.
- `gh run list --commit 70d1b69b2bab1204ba9c1637d9fe0bfddc663267` returns no
  run. Local proof includes the seven-oracle coverage inventory detector,
  coverage differential 28/28, and a GREEN registry-check.
- This audit adds no parity slice and makes no certification claim. The
  remaining blocker is external: a CI run on the current SHA is required.
  `check.sh`/workflow changes and publishing the shared branch are outside
  this receipt's permitted file scope; no ceiling change was made.

## Ship-test

BLOCKED_WITH_RECEIPT: the local and proof conditions are satisfied, but the
external CI leg cannot be observed on the current code SHA.

- Code SHA: `70d1b69b2bab1204ba9c1637d9fe0bfddc663267`; `gh run list --commit
  70d1b69b2bab1204ba9c1637d9fe0bfddc663267` returned no runs.
- Local `gate_shrink`: 37252 lines / 47 files; digest
  `fnv1a64:e119332a48d14afa`; ceiling remains exactly 37283.
- Exact historical CI discrepancy retained for `bd-engine-not-gate-ar39.15`:
  run context `5f178d2` measured 37472 against the 37283 ceiling; the named
  CI-only deltas were `crates/cdcp_gate/src/gates/substrate_guard.rs` (+128
  lines) and `crates/cdcp_gate/src/vcs.rs` (+61 lines), +189 total. Those are
  pane-owned files and were not touched here.
- No claim is made that the historical mismatch is a same-SHA result; it is the
  exact blocker evidence, not a CI GREEN receipt. No ceiling raise or lowering
  was performed.
- All 7 Python oracles are present; `bd-2m9` remains OPEN. The open beads are
  not READY and no epics were marked shipped.
- Workflow evidence: `br dep cycles` reports no cycles; `bv --robot-next`
  selects `bd-installability-sm4g.22` (CI has never completed a run).

## Final parity sweep and blockers

- The read-only Unicode sweep found U+0378–U+0379 present in the edited
  `verify_bank`, `verify_coverage`, `validate_grounding`, `verify_orphans`, and
  `verify_doc_consistency` implementations; `verify_bank`, `verify_coverage`,
  `validate_grounding`, `verify_doc_consistency`, and `verify_orphans` now also
  cover adjacent U+0380–U+0383. Their differential fixtures remain backed by
  the Python files on disk.
- The only remaining matching omission is in
  `crates/cdcp_gate/src/gates/verify_injection_count.rs`'s `py_is_printable`
  helper. That file is pane-owned and explicitly excluded from this work; it
  was not edited, and no extraction was taken from it.
- The read-only audit also finds `cdcp_learn/src/objectives.rs::py_repr`
  escapes C0 controls but not Python's non-printable Unicode ranges. That file
  and `diff_verify_objectives.rs` are pane-owned dirty work and were preserved
  without edits or staging.
- Current read-only pane proofs are green: `diff_verify_objectives` 20/20 and
  `diff_verify_injection_count` 33/33. Those suites do not cover the residual
  U+0380 classification gaps, so their green status is not treated as parity
  closure.
- Current `HEAD` is `3fcd339d0b735263f58c6612520d1f2d7ed3cf9f`, a receipt-only
  follow-up to code SHA `70d1b69b2bab1204ba9c1637d9fe0bfddc663267`; no gate
  source changed after the measured code SHA.
- `gh run list --commit 7298fb3571fdc607b01c9011dbac7111e8d2af06` returned no
  runs, so same-SHA CI line count and GREEN status remain unavailable. The historical 37472-line remote-main
  result is a different SHA and is not used as a same-SHA claim.
- Socraticode was unavailable in this environment; the local fallback anchor
  was the byte-exact differential/known-bad fixture inventory in the five
  permitted parity test files.
- `bd-substrate-python-gates-viu` and `bd-engine-not-gate-ar39.15` remain
  IN_PROGRESS; `bd-2m9` remains OPEN. No bead or epic was marked READY or
  shipped.
