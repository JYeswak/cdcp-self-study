# bd-ionf: paraphrase-cluster resolution

Date: 2026-08-20

All five remaining clusters were differentiated; none was retired. The approved
pool remains 931 items. Keys and item statuses were preserved, and `web/data`
was not touched.

## Verdicts

### m06-q013 / m06-q044 / m06-q239 — DIFFERENTIATE

`m06-q013` remains the single `understand` definition anchor for the normal
utility-to-rack sequence. `m06-q044` is now an `apply` question about which
stage bridges a utility-transfer interruption before downstream distribution;
its key `D` remains the UPS stage. `m06-q239` was already tagged `apply`, but
its old wording was still a definition; it is now an `apply` maintenance-review
scenario that rejects a proposed utility-to-rack-PDU path that bypasses
transfer, UPS, and distribution. The three items now test definition,
transfer-event reasoning, and diagram-review diagnosis respectively.

### m09-q112 / m09-q201 — DIFFERENTIATE

`m09-q112` remains the `understand` definition of recommended versus allowable
ASHRAE envelopes. `m09-q201` is now an `apply` scenario in which an operator
proposes running outside the recommended range but within the allowable range;
the learner must identify the need for an explicit risk and duration review.
Key `A` remains unchanged.

### m09-q140 / m09-q209 — DIFFERENTIATE

`m09-q140` remains the `understand` definition of blanking-panel function; its
stale `apply` tag was corrected to `understand`. `m09-q209` remains `apply` but
now presents unused rack U-spaces causing cold-air short-circuiting and asks for
the direct corrective action. Key `A` remains unchanged.

### m09-q149 / m09-q207 — DIFFERENTIATE

`m09-q149` remains the `understand` definition of cold-aisle containment.
`m09-q207` is now an `apply` high-density-row scenario involving bypass and hot/
cold mixing; the correct response is to focus supply at IT intakes and reduce
mixing. Key `C` remains unchanged.

### m09-q160 / m09-q229 — DIFFERENTIATE

`m09-q160` remains the `understand` definition of heat-exchanger isolation.
`m09-q229` is now an `apply` liquid-cooling design review involving different
facility/IT chemistry and pressure domains; the learner must select a heat
exchanger that isolates the loops while transferring heat. Key `A` remains
unchanged.

## Verification

- `verify-bank`: rc 0; 957 scanned, 931 approved.
- `answer-key-skew`: rc 0; `A=235, B=243, C=229, D=224`.
- `construction-faults`: rc 0; live verdict PASS, absolute-language and other
  verdict-bearing cues 0; length-rank uniformity PASS.
- `near-duplicate-items`: rc 0; 0 lexical-threshold pairs. Its paraphrase
  limitation remains and is why this receipt exists.
- `key-contradiction`, `verify-orphans`, and `validate-grounding`: rc 0.

The first application rewrite briefly introduced an `always` absolute in
`m09-q201`; construction-faults caught it before commit, and the distractor
was rewritten to a specific false claim without weakening the item.

## Final m15 slice — seven clusters

The seven m15 clusters recorded by pane 3 were adjudicated side by side. No
item was retired or had its key changed; the approved pool remains 931 and
`web/data` remains untouched.

### m15-q134 / m15-q200 / m15-q208 — DIFFERENTIATE

`m15-q134` remains an `understand` definition of why clear circuit and asset
labels reduce human error. `m15-q208` remains an `apply` question about using
those labels during emergency isolation and load migration. `m15-q200` was
the overlapping definition, so it is now an `apply` rack-move troubleshooting
scenario requiring labels tied to current records to trace a disputed path.

### m15-q137 / m15-q201 — DISTINCT

`m15-q137` tests what an as-built record represents: the facility as actually
constructed and changed. `m15-q201` tests the consequence of stale records:
responders can be misled about paths and isolation points. The definition and
operational consequence are different propositions, so neither was changed.

### m15-q142 / m15-q204 — DIFFERENTIATE

`m15-q142` is the `understand` principle that cleaning must control
particulates without damaging equipment or creating electrical hazards; its
stale `apply` tag was corrected. `m15-q204` is now an `apply` live-hall
scenario in which the supervisor must choose a particulate-controlled method
instead of blowing debris into intakes, skipping documented contamination, or
using an unverified chemical.

### m15-q143 / m15-q202 — DISTINCT

`m15-q143` is a `remember` definition of MTBF. `m15-q202` is an `understand`
question about using MTBF and MTTR together as reliability and maintainability
measures. The terms overlap because the relationship requires the definition,
but the propositions and cognitive tasks differ; neither was changed.

### m15-q146 / m15-q203 — DIFFERENTIATE

`m15-q146` is the `understand` definition of the measurable fields a
maintenance SLA should contain; its stale `apply` tag was corrected. `m15-q203`
is now an `apply` incident-contract scenario: a critical cooling alarm under
“best effort” requires response, restore, coverage, and escalation clauses.

### m15-q149 / m15-q205 — DISTINCT

`m15-q149` tests the general safety and security purpose of escorting visitors
through a plant. `m15-q205` applies that principle to vendor access controls,
including scoped access, identity checks, and escorts. The general purpose and
the operational control set are distinct propositions; neither was changed.

### m15-q138 / m15-q139 / m15-q207 / m15-q209 — DIFFERENTIATE

`m15-q138` remains an `understand` definition of MOP structure; its stale
`apply` tag was corrected. `m15-q139` remains an `understand` rationale for
verification and backout. `m15-q207` is now an `apply` scenario about the risk
of a dual-cord cutover with no controlled record or MOP. `m15-q209` remains an
`analyze` scenario about rollback criteria and restoring a known-good state.
Those roles separate the procedure, its rationale, its failure mode, and its
review decision.

## Closing bound

This completes 13 human-found paraphrase clusters: six previously resolved
clusters plus these seven m15 clusters. The lists contain 30 item memberships
(17 new m15 members and 13 earlier members), which is the scale described as
“roughly 28” in the dispatch. Differentiation makes the affected items test
different propositions going forward, but it does not turn this partial read
into evidence that the original pool contained 931 distinct propositions.

The honest bound is: 13 clusters were found in the portion we read; the whole
931-item approved pool was not scanned for paraphrase duplicates, and
`near-duplicate-items` cannot establish their absence because these pairs score
below its lexical thresholds. Therefore the advertised 23.3x 40-question
multiplier is a count ratio, not a verified count of 931 distinct propositions;
the effective pool is smaller by at least the overlap found here, with the
remaining amount unknown.

Final m15 verification: `verify-bank`, `answer-key-skew`,
`construction-faults`, `near-duplicate-items`, `key-contradiction`,
`verify-orphans`, and `validate-grounding` all exited 0. The one draft
absolute-language cue in m15-q207 was caught by `construction-faults` and
removed before commit.
