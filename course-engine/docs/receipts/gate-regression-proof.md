# Gate regression proof: grounding wave

Date: 2026-08-18/19

## Method

I built the current `target/debug/cdcp_gate` from worktree `60d8564`
(`3b19f74075c11de211aed1ba270217e7574d7c2ee7848eefb4c551f17439b85a`
SHA-256) and ran it with `--root` pointing at detached Git worktrees under
`target/gate-regression-proof-worktrees.DrecM7/`. Each scratch root had the
current gate sources and registries; only `bank/items` was overlaid from the
historical commit. The live checkout, `bank/items`, crates, and `scripts/check.sh`
were not edited by this audit.

I parsed every one of the 355 bank-changing snapshots in
`955a8f1..HEAD`. The unique maximum A-share was:

| tree | commit | approved single-select | A / B / C / D |
| --- | --- | ---: | --- |
| pre-wave | `955a8f1` | 931 | 274 / 246 / 209 / 202 |
| peak | `f2a55eb` (`2026-08-18 17:20:38 -0600`, `docs(curriculum): align environmental control scope`) | 931 | 539 / 159 / 112 / 121 (57.894737% A) |
| current | `60d8564` | 931 | 274 / 246 / 209 / 202 |

## Per-gate results

The “baseline” and “peak” roots are historical bank content tested by the
current binary. The “HEAD” root is the current bank in a detached worktree.

### `answer-key-skew`

| tree | verdict | exit | printed measurement |
| --- | --- | ---: | --- |
| baseline | PASS | 0 | approved=931; A=274 (29.4%), B=246 (26.4%), C=209 (22.4%), D=202 (21.7%); band=15.0%..35.0% |
| peak | RED | 2 | approved=931; A=539 (57.9%), B=159 (17.1%), C=112 (12.0%), D=121 (13.0%); band=15.0%..35.0%; outside=A,C,D |
| HEAD | PASS | 0 | approved=931; A=274 (29.4%), B=246 (26.4%), C=209 (22.4%), D=202 (21.7%); band=15.0%..35.0% |

This gate would have fired at the measured peak and is green on the known-good
baseline.

### `grounding-wave`

| tree | verdict | exit | printed measurement |
| --- | --- | ---: | --- |
| baseline | RED | 2 | approved=931; template-stem=2, unexcepted=1 (`m09-q206`); recall-only-stem=0, unexcepted=0; adjudicated-exceptions=1 (`template-stem:m09-q207`) |
| peak | RED | 2 | approved=931; template-stem=60, unexcepted=59; recall-only-stem=52, unexcepted=52; adjudicated-exceptions=1 (`template-stem:m09-q207`) |
| HEAD | PASS | 0 | approved=931; template-stem=1, unexcepted=0; recall-only-stem=0, unexcepted=0; adjudicated-exceptions=1; all currently flagged ids are adjudicated exceptions |

The baseline RED is a falsifier: the current detector fires on known-good
`955a8f1` because `m09-q206` (“Hot-aisle containment primarily aims to:”) is
an unexcepted member of the same legitimate containment family that includes
the excepted `m09-q207`. The peak RED is therefore not sufficient to call this
gate a regression proof. No detector or exception was changed to make history
look green.

### `required-tests`

| tree | verdict | exit | printed measurement |
| --- | --- | ---: | --- |
| baseline | PASS | 0 | `tree=worktree`; suites=2; required=6; every suite reported `0 filtered out`; `cdcp_anki-export:required=2 cdcp_bank-lib:required=4` |
| peak | RED | 2 | `tree=worktree suite=cdcp_bank-lib exited 101 (the suite is RED)`; direct suite diagnostic: 239 passed, 1 failed, 0 filtered out (`tests::load_real_bank_items`, `bank-m15-q154`); Anki export: 14 passed, 0 filtered out |
| HEAD | PASS | 0 | `tree=worktree`; suites=2; required=6; every suite reported `0 filtered out`; `cdcp_anki-export:required=2 cdcp_bank-lib:required=4` |

On the peak root all six registry-named test identities were present and both
suites ran unfiltered; the RED was caused by the bank suite’s content assertion
against historical peak content, not by a missing required test or a filtered
run.

## Falsifier disposition

- `answer-key-skew` passes the requested historical leg: baseline GREEN,
  peak RED at 57.894737% A, current GREEN.
- `grounding-wave` does not pass the requested historical leg: it is RED on
  the known-good baseline as well as at peak. This is a finding for a separate
  calibration/adjudication tick, not a reason to weaken the detector here.
- `required-tests` proves the six named identities ran unfiltered in these
  worktrees, but the peak suite itself is RED because historical content is
  incompatible with a current bank assertion. It is not a clean content-wave
  detector.

These results make `answer-key-skew` a regression proof for this wave. They do
not yet make `grounding-wave` one, and they say nothing about defect classes
that were not represented by these historical snapshots.

## Recalibration rerun: two-signal template detector

The baseline RED above was the required falsifier. It was not repaired with a
second exception: `m09-q206` is a legitimate member of the containment family,
and adding another waiver would make the exception list the detector's escape
hatch. The detector now requires both:

- stem similarity `>= 70%`; and
- correct-answer-text similarity `>= 60%` within a two-item cluster
  (`min_peers = 1`).

The thresholds are the registry row in `registries/grounding_wave.toml`; the
Rust code contains no calibration literals. Calibration measurements were:

- baseline: 26 stem-similar pairs, with maximum answer-text similarity 58.8%;
- current tree: same 58.8% maximum;
- intentional containment family: `m09-q149`/`m09-q207` answer similarity
  41.2% (and `m09-q206` is still more divergent);
- peak: 146 stem-similar pairs, 22 also clearing answer similarity 60%,
  producing 41 flagged ids.

The `m09-q207` exclusion was deleted. The live receipt now reports
`adjudicated-exceptions=0`.

This rerun used current worktree `ae6a0443da060bf1438f206b69c9991c3b1c4aa3`
and current binary SHA-256
`8c176669215d6d10cdd7b015a85c4fd0f074305f4412f4a969d2af5084d3e87a`.
The preserved fixture test
`grounding_wave::tests::preserved_peak_damage_fixture_is_red` passed against
`crates/cdcp_bank/tests/fixtures/damaged_corpus_2026_08_18/`; it is the
permanent real-wave known-bad leg, not a Git re-derivation.

| tree | `grounding-wave` | `answer-key-skew` |
| --- | --- | --- |
| `955a8f1` | PASS, exit 0; approved=931; template=0; recall=0; unexcepted=0; exceptions=0 | PASS, exit 0; A/B/C/D=274/246/209/202; band=15.0%..35.0% |
| `f2a55eb` peak | RED, exit 2; approved=931; template=41/unexcepted=41; recall=52/unexcepted=52; exceptions=0 | RED, exit 2; A/B/C/D=539/159/112/121; A=57.9%; outside=A,C,D |
| current HEAD | PASS, exit 0; approved=931; template=0; recall=0; unexcepted=0; exceptions=0 | PASS, exit 0; A/B/C/D=274/246/209/202; band=15.0%..35.0% |

The concurrent `required-tests` worktree rerun was measured separately (the
prior six-row proof remains above); that unrelated registry and dispatcher
work is intentionally not included in this grounding-detector commit.

### Calibration receipt metadata

```text
schema_version = "zeststream.spec_ops_math_receipt.v1"
skill = "false-positive-calibration"
family = "proof"
math_lever = "Bayesian calibration and posterior odds"
proof_gate = "precision/recall tracked on known fixtures"
fixture_refs = ["crates/cdcp_bank/tests/fixtures/damaged_corpus_2026_08_18/"]
result = "pass"
failure_class = "resolved_false_positive_by_signal_discrimination"
local_anchor = "Socraticode unavailable; deterministic rg fallback used over grounding_wave.rs, grounding_wave.toml, this receipt, and the preserved fixture."
```

The historical leg now satisfies the original falsifier contract: the
known-good baseline is GREEN, the measured peak is RED, and the exception
count is zero. This proves separation for the containment family and this
wave's collapsed-answer templates only; it does not prove coverage of an
unseen template family.

## Landed commit replay

Commit `cd2a6b2d3c183096112ef7c61666de4fd71bfc01` was replayed after landing.
The post-landing `cdcp_gate` binary SHA-256 was
`3b963bdab4b8df0a3833d59b8d415a99447c35a392511b718e37f880df899af3`.
The results were unchanged:

- `955a8f1`: `grounding-wave` exit 0 with template=0, recall=0,
  exceptions=0; `answer-key-skew` exit 0 with A/B/C/D=274/246/209/202.
- `f2a55eb` peak: `grounding-wave` exit 2 with template=41/unexcepted=41,
  recall=52/unexcepted=52, exceptions=0; `answer-key-skew` exit 2 with
  A/B/C/D=539/159/112/121 (A=57.9%, outside A,C,D).
- `HEAD`: `grounding-wave` exit 0 with template=0, recall=0,
  exceptions=0; `answer-key-skew` exit 0 with A/B/C/D=274/246/209/202.

Post-landing checks: `cdcp_bank` had 242 tests pass; bank and gate clippy
completed with zero warnings. No `bank/items` file or gate-shrink ceiling was
changed.
