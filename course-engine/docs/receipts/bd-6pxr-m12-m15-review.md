# bd-6pxr: final m12/m15 superlative-stem review

Date: 2026-08-20

## Scope and denominator

Using the audit signature `best|most|primarily|mainly|chiefly|greatest|strongest`,
the current approved tree contains 11 m12 items and 9 m15 items. I read all 20.
All 11 m12 files and all 9 m15 files had a stem/choice/key surface change
relative to `e25a03e4`; this is a changed-file count, not a claim that every
changed field was wrong.

The three m12 escalation repairs were checked in their current form. `m12-q043`
and `m12-q047` are in the literal 11-item set. `m12-q046` was also repaired in
the escalation and was read, but its stem says “more typical” rather than using
one of the seven literal audit tokens, so it is recorded separately rather than
silently changing the 11-item denominator.

## Results

| module | read | changed | (a) arguably true | (b) ambiguous | (d) several distractors instantiate key | result |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| m12 | 11 (+ q046 escalation check) | 11 | 0 new | 0 | 0 | clean in current state; q043/q046/q047 were prior repairs |
| m15 | 9 | 9 | 0 | 0 | 0 | clean on first read |

The other nine m12 items were read for the same failure mode as the escalation
and did not contain a second defensible option. No choice, stem, key,
explanation, citation comment, or web artifact was changed in this slice.

## Closing bd-6pxr accounting

Across the six audit slices, 306 item files were read:

- m05/m07/m14 full-module reread: 112;
- m08/m10 superlative read: 16;
- m06/m09 superlative read: 68;
- m01/m02/m03/m04/m11/m13 superlative read: 90; and
- m12/m15 final superlative read: 20.

The full bead accounting names 10 repaired correctness defects: (a)=5
(`m05-q139`, `m09-q132`, `m01-q002`, `m12-q043`, `m12-q046`), (b)=3
(`bank-m14-q105`, `m06-q211`, `m06-q045`), and (d)=2 (`m12-q047`,
`m06-q226`). The first eight were repaired in the audit/escalation sequence;
`m06-q226` and `m06-q045` were already repaired during the explanation pass
and are included here because the audit read their current items and the bead
tracks those confirmed cases.

Clean on first read: m02, m03, m04, m07, m08, m10, m11, m13, and m15. The
other modules had a finding repaired during or before their review; “clean” is
not being used to erase that history.

## What this does not establish

The audit covers the superlative-stem population, 217 of 931 approved items.
Items with a direct stem were not read. Roughly 480 items had choices changed
today, so this is not a correctness certificate for every changed item. It is
a human read of the population where multiple-defensible options are most
likely by construction, not a gate or an oracle for the remaining bank.
