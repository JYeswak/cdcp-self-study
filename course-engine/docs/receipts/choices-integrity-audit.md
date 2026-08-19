# Choices-integrity audit

Bead: `bd-0cjy`

## Scope and snapshot

This is an audit, not a bank edit. The audit uses the immutable task-start tree at
`8d46c1d`, before pane 3's later choice-restore batch (`59ef2f0`), because the
shared worktree was changing during the audit. It compares each approved
`bank/items/<filename>.toml` in that snapshot with the same filename at
`955a8f1`. The IDs below use the filename stem (`m15-q149`), which is also the
form used by `.gw-choices-restore.txt`; a few current TOML `id` fields carry a
`bank-` prefix.

For every item, TOML was parsed and the `choices`, `correct`, and `explanation`
fields were compared. `stem` was deliberately not used for the integrity
finding. The short-heading fingerprint requires a changed choices array, a
current correct choice of at most four whitespace-delimited words that reads as
a noun/heading label, and a pre-wave correct choice that is a full proposition
(more than four words).

## Denominators and findings

| Measure | Count |
| --- | ---: |
| Item files in the snapshot | 957 |
| Approved items audited | 931 |
| Approved items with a `955a8f1` baseline | 931 |
| Approved items with no baseline (new items) | 0 |
| Choices arrays differing from pre-wave | 109 |
| Correct-letter fields differing | 2 |
| Correct-answer text differing | 109 |
| Explanations differing | 78 |
| Items with any choices/correct/explanation difference | 158 |
| Short-heading/full-proposition fingerprint | 14 |

The two correct-letter changes are `m07-q201` (`D -> A`) and `m08-q206`
(`B -> A`). There are no new/unbaselined approved IDs to list separately.

The 87 IDs in `docs/receipts/.gw-choices-restore.txt` account for 87 of the
109 choice differences, so they are already in flight and are not duplicated
by this audit. The remaining 22 choice-drift IDs are outside that restore set
and remain findings, not cleared items.

## All choice differences, grouped by module

`†` means listed in the pane-3 in-flight restore set. `*` means the
short-heading/full-proposition fingerprint.

- M01: `m01-q041†`, `m01-q044†*`, `m01-q049†`, `m01-q051†*`, `m01-q052†`,
  `m01-q053†*`, `m01-q056†`, `m01-q060†`, `m01-q200†`, `m01-q204†`,
  `m01-q207†`, `m01-q208†`
- M03: `m03-q094`, `m03-q098`, `m03-q103†`, `m03-q211†`
- M05: `m05-q208†`, `m05-q212†`
- M06: `m06-q017†`, `m06-q061†`, `m06-q073†`, `m06-q081†`, `m06-q101†`,
  `m06-q109†`, `m06-q204†`, `m06-q208†`, `m06-q216†`, `m06-q224†`,
  `m06-q248`
- M07: `m07-q045†`, `m07-q051†`, `m07-q058†`, `m07-q201`, `m07-q203†`,
  `m07-q210†`, `m07-q214†`, `m07-q215†`
- M08: `m08-q045†`, `m08-q205†`, `m08-q206`, `m08-q209†`
- M09: `m09-q104†*`, `m09-q105`, `m09-q108`, `m09-q116†`, `m09-q132†*`,
  `m09-q148`, `m09-q161`
- M10: `m10-q115`, `m10-q215†`
- M11: `m11-q100†`, `m11-q112†*`, `m11-q124†`, `m11-q211†`
- M12: `m12-q041†`, `m12-q045†`, `m12-q053`, `m12-q057`, `m12-q059`,
  `m12-q065†`, `m12-q208`, `m12-q221`, `m12-q224`, `m12-q225`, `m12-q226`
- M13: `m13-q081†*`, `m13-q088`, `m13-q089†`, `m13-q090†`, `m13-q097†`,
  `m13-q101†*`, `m13-q208`
- M14: `m14-q038†`, `m14-q113†`, `m14-q116`, `m14-q117†`, `m14-q125†`,
  `m14-q129†`, `m14-q133†`, `m14-q200†`, `m14-q204†`, `m14-q205`,
  `m14-q208†`
- M15: `m15-q137†*`, `m15-q141†*`, `m15-q149†*`, `m15-q153†*`,
  `m15-q205†`, `m15-q213†*`, `m15-q219†`, `m15-q340†*`, `m15-q342†`,
  `m15-q345†`, `m15-q347†`, `m15-q348†`, `m15-q349†`, `m15-q350†`,
  `m15-q351†`, `m15-q353†`, `m15-q354†`, `m15-q356†`, `m15-q358†`,
  `m15-q364†`, `m15-q369†`, `m15-q374†`, `m15-q376†`, `m15-q379†`,
  `m15-q382†`, `m15-q383†`

## Fingerprint findings

All 14 fingerprint findings are already in pane 3's in-flight 87-item set.
The arrows show pre-wave correct-answer text to current correct-answer text:

- M01: `m01-q044` — “Because outage consequences can include severe financial,
  legal, safety, or reputational harm” -> “Security”; `m01-q051` — “Working
  without a change window, peer check, or clear isolation boundaries” ->
  “Improved risk-management processes”; `m01-q053` — “Tolerance for service
  interruption relative to regulatory and financial exposure” -> “Energy
  efficiency”.
- M09: `m09-q104` — “Airflow volume and heat removed for a given cooling
  capacity” -> “Vibration”; `m09-q132` — “Using evaporation so leaving water
  can approach wet-bulb-limited conditions” -> “Relative humidity control”.
- M11: `m11-q112` — “Can prevent links from coming up even when loss budgets are
  fine” -> “Testing of MPO cabling”.
- M13: `m13-q081` — “Plant compromise can take down IT without touching a
  server” -> “Security Perimeters”; `m13-q101` — “Temporary exceptions become
  permanent habits and create alarm fatigue or silent bypass” -> “Security
  Systems”.
- M15: `m15-q137` — “Records how the facility actually is after
  construction/changes, not only the original design intent” -> “As-built
  documentation and training”; `m15-q141` — “Impair cooling airflow,
  contaminate equipment, and increase fire fuel/ignition risk” -> “Particulate
  control”; `m15-q149` — “Prevent untrained persons from hazardous energy
  exposure and unauthorized access simultaneously” -> “Security”; `m15-q153` —
  “Recognize contractual response may not meet internal restore goals—plan
  spares, skills, and bridging procedures” -> “Availability”; `m15-q213` — “As
  part of the change process when the field changes—not months later if ever”
  -> “Data centres”; `m15-q340` — “Update role training before the changed duty
  begins” -> “Personnel involvement and competence”.

## Choice differences outside the in-flight restore set

These 22 IDs are not in `.gw-choices-restore.txt` and require separate
adjudication; none matches the short-heading fingerprint in this snapshot:

- M03: `m03-q094`, `m03-q098`
- M06: `m06-q248`
- M07: `m07-q201`
- M08: `m08-q206`
- M09: `m09-q105`, `m09-q108`, `m09-q148`, `m09-q161`
- M10: `m10-q115`
- M12: `m12-q053`, `m12-q057`, `m12-q059`, `m12-q208`, `m12-q221`,
  `m12-q224`, `m12-q225`, `m12-q226`
- M13: `m13-q088`, `m13-q208`
- M14: `m14-q116`, `m14-q205`

This cleanly identifies the blind spot: the duplicate/near-duplicate gate is
not needed to find the 14 heading collapses, and the other 95 choice changes
do not become safe merely because they did not collapse onto a shared word.
