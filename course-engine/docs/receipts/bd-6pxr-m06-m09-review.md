# bd-6pxr: m06/m09 superlative-stem correctness review

Date: 2026-08-20

## Scope and denominator

I reviewed every approved `m06-*` and `m09-*` item whose stem matched the
case-insensitive signature `best|most|primarily|mainly|chiefly|greatest|strongest`.
The dispatch described 42 m06 items and 25 m09 items.  The current tree contains
43 m06 matches and 25 m09 matches under that rule, so I read all 68 rather than
silently dropping the extra m06 match.  The count discrepancy is recorded here
as a measurement difference, not treated as a clean result.

Using `e25a03e4` as the pre-pass comparison point, 33 m06 files and 19 m09
files had stem/choice/key surface changes in the recent construction passes.
The review compared the complete choice sets and keys, with priority to the
items whose fields changed on 2026-08-19/20.  Explanations, citations, keys,
stems, status, and `web/data` were not changed except where noted below: the
two repairs changed choices only and preserved their keys and explanations.

## Verdicts

| id | verdict | finding and action |
| --- | --- | --- |
| `m06-q211` | (b) ambiguous | The changed single-cord distractor could be read as promising that a catcher arrangement protects a rack without tracing the primary path or catcher-transfer dependency. It was tightened to a specifically false claim about protection without that dependency analysis. Key `D` and explanation were preserved. |
| `m09-q132` | (a) arguably true / correctness defect | The keyed option `A` was the false claim that evaporative towers eliminate water use, while distractor `B` described evaporation toward wet-bulb-limited conditions and was the defensible answer. The true proposition was restored to `A`; `B` was rewritten as the false claim that evaporation eliminates makeup-water and treatment planning. The unrelated false option about heating condenser water was also made specifically false without an absolute cue. Key `A` and explanation were preserved. |

No item was classified (d): no reviewed item had several distractors that were
instances of the keyed concept. There are no unresolved (a), (b), or (d)
findings after these two choice-only repairs. Thus m06 and m09 both came back
with zero unresolved correctness findings, although neither was clean on the
first read: m06 had one ambiguity repaired and m09 had one second-defensible
answer repaired.

## Paraphrase-duplicate findings

The near-duplicate gate returned zero, but its own output says that paraphrases
below its thresholds are invisible. The following strong candidate clusters
were noticed during the read and are recorded for a separate duplicate-content
review; they were not used as a correctness verdict or mechanically edited:

- m06: `m06-q013`, `m06-q044`, `m06-q239` (utility-to-rack critical-power path);
  `m06-q076`, `m06-q220` (three-phase balanced loading); `m06-q084`,
  `m06-q236` (IP protection against environmental ingress); and `m06-q069`,
  `m06-q247` (natural-gas versus diesel trade-offs).
- m09: `m09-q028`, `m09-q100` (IT heat is sensible); `m09-q112`,
  `m09-q201` (ASHRAE recommended versus allowable envelopes); `m09-q132`,
  `m09-q239` (evaporative-tower heat rejection); `m09-q140`, `m09-q209`
  (blanking panels); `m09-q149`, `m09-q207` (cold-aisle containment);
  `m09-q160`, `m09-q229` (facility/IT loop isolation); and `m09-q206`
  (hot-aisle containment) with its closely parallel containment family.

These are findings, not proof that every pair is redundant: similar teaching
propositions can be intentional. The current gate result is therefore not
evidence that this list is empty.

## Verification after repair

- `verify-bank`: rc 0; 957 scanned, 931 approved.
- `answer-key-skew`: rc 0; `A=235, B=243, C=229, D=224` (25.2/26.1/24.6/24.1%).
- `construction-faults`: rc 0; live verdict PASS; length-rank counts
  `[254,231,219,227]`; grammatical, absolute-language, and all/none cues 0.
- `near-duplicate-items`: rc 0; 0 pairs at its configured thresholds, with
  the paraphrase limitation above.
- `key-contradiction`: rc 0; numeric contradictions 0.
- `verify-orphans`: rc 0; orphan topics and item references 0.
- `validate-grounding`: rc 0; high-severity findings 0.

The bank-wide key counts and all explanations remain untouched by this audit.
The 26 retired items and any proposition-level duplicate adjudication remain
outside this slice.
