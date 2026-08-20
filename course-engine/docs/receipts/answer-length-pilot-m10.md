# Answer-length pilot: M10

Date: 2026-08-19
Bead: bd-2qvn
Scope: the 35 approved single-select items in M10 only. The two retired M10
files were not changed, and no item outside M10 was changed.

## Measurement

The metric is strict-longest: the keyed choice is longer than every one of
the three distractors. Measurements use parsed TOML fields; the baseline is
`HEAD` before this pilot.

| population | before | after |
| --- | ---: | ---: |
| M10 approved strict-longest | 35/35 (100.0%) | 0/35 (0.0%) |
| all approved strict-longest | 838/931 (90.0%) | 803/931 (86.3%) |

All 35 M10 `correct` fields remain unchanged. The approved answer-key
distribution is unchanged and was rechecked by `answer-key-skew`:

```text
A=274 (29.4%), B=246 (26.4%), C=209 (22.4%), D=202 (21.7%)
```

`verify-bank` also passed with 957 scanned items and 931 approved items.
Citation/source comments were retained in every edited item.

## Example diffs

These illustrate the two permitted approaches: make distractors substantive
and comparable, or tighten an overlong keyed proposition without changing its
meaning or key letter.

- `m10-q100` (key A unchanged):
  - old distractors: `It is unrelated to facility availability`; `It powers the UPS inverter directly`; `It replaces fire suppression in all designs`
  - new distractors: `It is unrelated to facility availability because process water never affects cooling equipment`; `It supplies the UPS inverter directly rather than supporting heat rejection or humidity control`; `It replaces fire suppression in every design, so separate fire-water planning is unnecessary`
- `m10-q107` (key D unchanged and tightened):
  - old key: `Evaporative methods can be more energy-efficient but consume water; dry methods save water but may use more energy in heat`
  - new key: `Evaporative cooling can save energy but uses water; dry cooling saves water but may use more energy`
  - the three distractors were expanded into explicit, domain-relevant but false resource claims.
- `m10-q300` (key B unchanged and tightened):
  - old key: `Contain and isolate the leak per site procedure, protect nearby electrical/IT assets, escalate per notification tree, and document actions — do not wait for a full rupture`
  - new key: `Contain and isolate the leak, protect nearby electrical/IT assets, notify the response chain, and document the action`
  - the wrong sequences were expanded to comparable operational propositions rather than padded with generic words.

## Boundary

This pilot removes one mechanical cue; it does not establish item quality or
discrimination. The remaining 803 longest-key items are outside this pilot and
were intentionally not edited.

GREEN-DOES-NOT-PROVE: removing the length cue does not make an item good. An item can be length-neutral and still fail to discriminate — that needs response data we do not have. This removes one mechanical cue, nothing more.

## Follow-up: rank-uniformity repair

The first pilot overcorrected: the keyed length-rank distribution was
`[0,12,9,14]` for ranks 1 through 4 (0.0%, 34.3%, 25.7%, 40.0%). The
follow-up changes only substantive M10 distractor/key wording and leaves every
`correct` field unchanged. Using character length with A-D order as the
deterministic tie-break, the distribution is now:

```text
rank 1 (longest): 9/35 (25.7%)
rank 2:           9/35 (25.7%)
rank 3:           8/35 (22.9%)
rank 4 (shortest):9/35 (25.7%)
```

The key letters before and after are identical, and the approved distribution
remains `A=274, B=246, C=209, D=202`.

`registries/construction_faults.toml` now has `[rank_uniformity]` with
`max_deviation_pct = 10` and `min_items = 4`. Each population must keep all
four keyed rank shares within 10 percentage points of the 25% target; both
all-longest and all-shortest fixture populations therefore go RED. The gate
reports rank counts and shares for live and damaged populations.

Known-bad fixtures:

- `crates/cdcp_bank/tests/fixtures/construction_faults/rank_longest/`
- `crates/cdcp_bank/tests/fixtures/construction_faults/rank_shortest/`

Known-good `construction_faults/good` now has one item at each rank. The full
`cdcp_bank` suite passed (253 unit tests plus integration suites), and the
construction-faults dispatcher e2e passed both tests. The live gate remains
RED, as expected, with `length-rank-uniformity` reported.

GREEN-DOES-NOT-PROVE: uniform length rank removes length as a signal. It says nothing about whether the distractors are plausible to someone who knows the material; that still needs response data.
