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

## Residual adjudication: 22 IDs

Adjudication was performed after pane 3's 87-item restore. RESTORE means the
stem, choices, correct key, and explanation were returned to `955a8f1`; the
wave's citation/source comments and unchanged metadata were retained. KEEP and
UNSURE items were not edited.

| Module | ID | Verdict | Reason |
| --- | --- | --- | --- |
| M03 | `m03-q094` | RESTORE | The wave changed a question about which plant assets and operability flood exposure threatens into a generic site-risk classification, losing the material consequence being tested. |
| M03 | `m03-q098` | RESTORE | The current answer drops geographic site rejection from a site-selection question; that is a material consequence of seismic/geotechnical risk, not a harmless paraphrase. |
| M06 | `m06-q248` | RESTORE | The replacement is an OSHA-style grounding/overcurrent requirement and does not answer the “because” stem's distinction between equipotential bonding and fault clearing. |
| M07 | `m07-q201` | RESTORE | The D→A move is answer-text replacement, not a position move: the units question became an H/B terminology question. Original key D restored. |
| M08 | `m08-q206` | RESTORE | The B→A move is answer-text replacement, changing facility grounding/noise-control benefits into a different telecom-bonding formulation. Original key B restored. |
| M09 | `m09-q105` | KEEP | The current answer is a direct, less quantified paraphrase of the same IT-power-to-cooling-heat proposition; the key and tested concept remain unchanged. |
| M09 | `m09-q108` | KEEP | The current answer legitimately broadens economization from outdoor conditions to airside, waterside, and refrigerant paths while preserving the compressor/mechanical-work reduction. |
| M09 | `m09-q148` | KEEP | The current wording broadens HAC to full containment and precise airflow control but retains the same inlet-stability/fan-energy purpose; it is a technically coherent improvement. |
| M09 | `m09-q161` | KEEP | Direct-to-chip and rear-door architectures are concrete ways to answer why liquid cooling is discussed for high-density racks; the high-density heat-removal target remains. |
| M10 | `m10-q115` | UNSURE | The current answer drops the optional fire-system dependency but preserves water-dependent cooling/humidity reasoning; I cannot establish whether that omission is a legitimate scope boundary or a material loss. |
| M12 | `m12-q053` | KEEP | “Permit safe exit before discharge” is equivalent to release/free-egress while making the life-safety timing explicit. |
| M12 | `m12-q057` | KEEP | The added nonconductive-media condition is the precise Class C definition and preserves the energized-electrical-equipment concept. |
| M12 | `m12-q059` | KEEP | The explicit OSHA definition in the stem makes one-letter class options coherent; the item still tests that ordinary combustibles are Class A, with key C unchanged. |
| M12 | `m12-q208` | KEEP | The replacement preserves restoration/verification after impairment and adds notification and temporary precautions; the tested readiness concept remains. |
| M12 | `m12-q221` | KEEP | The current answer makes the same egress/life-safety rationale concrete through marked, unobstructed exits and operable safeguards. |
| M12 | `m12-q224` | KEEP | The current answer is a more operational version of the same maintenance test, adding restoration after alarms/repairs and operability. |
| M12 | `m12-q225` | RESTORE | The current question removes the UPS-room versus BESS-yard fire-playbook split and reduces a scenario judgment to document-role recall. |
| M12 | `m12-q226` | RESTORE | The current question removes chemistry, EOP, interconnect, and evacuation analysis, materially narrowing an analyze-level packet judgment into document-role recall. |
| M13 | `m13-q088` | RESTORE | The wave replaced door-forced operational response and synchronized forensics with generic VSS integration vocabulary, changing the learning target. |
| M13 | `m13-q208` | UNSURE | Lost/stolen-credential mitigation and authentication assurance are related but not identical MFA rationales; the item alone cannot prove improvement versus loss. |
| M14 | `m14-q116` | KEEP | The current wording preserves separation of life-safety alarm generation from operations-tool supervision while making the interface boundary explicit. |
| M14 | `m14-q205` | RESTORE | The replacement drops prioritization, ownership, and alarm-noise control—the alarm-philosophy target—and leaves only generic operator notification. |

### Adjudication totals

- RESTORE: 9 (`m03-q094`, `m03-q098`, `m06-q248`, `m07-q201`,
  `m08-q206`, `m12-q225`, `m12-q226`, `m13-q088`, `m14-q205`).
- KEEP: 11.
- UNSURE: 2 (`m10-q115`, `m13-q208`); both remain unchanged for human review.

The two correct-letter changes were both RESTORE decisions, so their keys were
returned to the original D and B positions rather than selected for answer-key
balance. The 22-item adjudication closes only this choices residual at the
audited snapshot; it does not clear retired items or fields outside the
choices/correct/explanation comparison.

## Full-tree reconciliation: 18 reported residuals

The full-tree reconciliation reported 792 approved items differing from
`955a8f1` in comments only, 44 differing in explanation only, and 18 named
items with a reported stem/choices body difference. The 18 reconcile as
follows. This section does not reopen the 22-item adjudication; it records
which prior decisions stand and closes the earlier-restore serialization gap.

### (a) Prior KEEP/UNSURE decisions stand

These 13 items are intentional under the decisions above; no bank edit was
needed:

- `m09-q105` — KEEP: the less-quantified answer still tests IT power becoming
  cooling heat load.
- `m09-q108` — KEEP: the expanded airside/waterside/refrigerant wording keeps
  the economization-to-lower-mechanical-work proposition.
- `m09-q148` — KEEP: the broader full-containment wording preserves the
  inlet-stability and fan-energy purpose of HAC.
- `m09-q161` — KEEP: direct-to-chip/rear-door architectures still answer the
  high-density liquid-cooling rationale.
- `m10-q115` — UNSURE: the current answer preserves water-dependent
  cooling/humidity reasoning but omits the optional fire-system dependency;
  its legitimacy remains unresolved.
- `m12-q053` — KEEP: explicit safe exit before discharge is equivalent to
  release/free-egress.
- `m12-q057` — KEEP: the added nonconductive-media condition is the precise
  Class C definition.
- `m12-q059` — KEEP: the explicit OSHA stem makes the one-letter Class A/B/C/D
  options coherent and preserves the Class A test.
- `m12-q208` — KEEP: impairment notification and restoration/verification
  preserve the post-discharge readiness test.
- `m12-q221` — KEEP: marked unobstructed exits and operable safeguards make the
  same egress/life-safety rationale concrete.
- `m12-q224` — KEEP: restoration after alarms/repairs and operability strengthen
  the same fire-maintenance proposition.
- `m13-q208` — UNSURE: lost/stolen-credential mitigation and authentication
  assurance are related but not identical MFA rationales; it remains open for
  human review.
- `m14-q116` — KEEP: the current wording still separates life-safety alarm
  generation from operations-tool supervision.

### (b) Earlier restores are complete

`m06-q239`, `m09-q118`, `m09-q122`, and `m14-q209` already have the same
parsed stem, choices, and correct key as `955a8f1`. Their remaining content
difference is the deliberately rewritten explanation; their citation comments
are retained. The only raw choice-text discrepancy was a trailing comma on the
last TOML choice line, so this reconciliation normalizes that serialization to
the pre-wave form without changing any choice value or key.

### (c) Unlisted item: `m10-q112`

`m10-q112` is a legitimate unrelated edit, not an omitted restore. Its stem
change originated in commit `5408ccf` (`docs(curriculum): ground cooling water
operations receipts`), not in the grounding-wave restore sets. The choices and
key are unchanged. The current stem broadens “dry coolers or adiabatic-limited
designs” to the general water-saving heat-rejection strategy, while preserving
the same water-dependency versus energy-tradeoff question; its current DOE
receipt and explanation explicitly support that broader framing. It remains
unchanged.

### Reconciliation boundary

The four group-(b) choice serializations were normalized; no answer key moved,
so the approved distribution is unchanged. The 26 retired items remain an open,
out-of-scope finding: neither this choices audit nor the earlier body audit
compared them against `955a8f1`. The approved-pool reconciliation is not a
quality certificate for the pre-wave baseline, and it says nothing about fields
outside the audited body fields.
