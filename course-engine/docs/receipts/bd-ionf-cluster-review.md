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
