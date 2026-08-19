# Construction-fault gate — bd-0cjy.2

The gate measures option-set construction cues only. It does not measure
discrimination, answer-key truth, currency, redundancy, cognitive level, or
grounding. Those require other evidence; discrimination specifically requires
response data, which this repository does not have.

## Detectors

Thresholds are in registries/construction_faults.toml, not in the decision
path:

- longest-option-correct: the keyed choice is at least 25% and three words
  longer than the longest distractor.
- grammatical-disagreement: conservative explicit article or
  present/past auxiliary agreement at the end of the stem, with a keyed choice
  matching and at least one distractor using an opposite form.
- absolute-language-distractor: a distractor contains always, never, all,
  none, or only while the key contains a configured hedge.
- all-none-of-the-above: an option is an all/none-of-the-above shortcut.

The gate is intentionally diagnostic: a live construction finding makes
cdcp_gate construction-faults RED and the output includes capped sample IDs.
The preserved damaged corpus is measured for comparison but does not itself
make the live gate RED.

## Measured populations

Measured by cdcp_gate construction-faults at the worktree containing the
gate:

| Population | Items | Longest-key | Grammar | Absolute distractor | All/none | Detector hits |
|---|---:|---:|---:|---:|---:|---:|
| Live approved pool | 931 | 694 (74.5%) | 0 (0.0%) | 116 (12.5%) | 0 (0.0%) | 810 |
| Damaged corpus | 448 | 210 (46.9%) | 0 (0.0%) | 16 (3.6%) | 0 (0.0%) | 226 |

Delta, damaged minus live:

- longest-key: -484 items, -27.7 percentage points
- grammatical disagreement: 0, 0.0 percentage points
- absolute-language distractor: -100 items, -8.9 percentage points
- all/none: 0, 0.0 percentage points

This does not support the claim that the grounding-wave damaged construction
on these four mechanically visible axes. The live pool is substantially worse
under these thresholds, so these cues are presently a baseline construction
problem (or a threshold/proxy that needs human calibration), not evidence of
wave-specific degradation. The result says nothing about the truth or
discrimination of either population.

## Verification

- cargo test -q -p cdcp_bank construction_faults: 4 passed, including
  known-bad, known-good, per-detector, and zero-item ERROR legs.
- cargo test -q -p cdcp_gate --test construction_faults_e2e: 2 passed,
  including dispatcher registration, both population counts, and unknown-flag
  rejection.
- cargo clippy -q -p cdcp_bank -p cdcp_gate --all-targets -- -D warnings:
  GREEN.
- cargo run -q -p cdcp_registry_check -- .: GREEN,
  cdcp_gate=34029/34040, leaving 11 lines of real slack. The ceiling was not
  raised.
- The full scripts/check.sh attempt did not reach this new step: pane 2's
  required-tests changes make the substrate prove-wired scratch run fail at
  required-tests before the construction gate. Exact blocker:

  required-tests: FAIL: tree=worktree suite=cdcp_bank-lib exited 101 (the suite is RED)

  This receipt does not weaken, skip, or reorder that chain failure.

No bank/items file and no registries/grounding_wave.toml file was changed.
