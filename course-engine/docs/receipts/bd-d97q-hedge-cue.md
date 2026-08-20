# bd-d97q hedge-cue audit

Status: product-level cue closed; the literal post-edit exactly-one denominator is recorded as a measurement finding below.

## Intervention

81 approved items whose keyed option was the sole option containing one of the
hedge words were reviewed. Each retained its stem, keyed letter, and
conditional keyed claim. A distractor was qualified where that remained
unambiguously false; five qualifications were tightened after the construction
gate caught residual `only`/`never` distractor cues. No `correct` field changed.

The pre-edit population supplied by the bead was 143 items with exactly one
hedged option, of which 81 keyed that option (56.6%). After distributing a
hedge to each of those 81 distractor sets, the live bank has 62 exactly-one
items and 0 keyed exactly-one items. Thus the literal post-edit exactly-one
hit rate is 0/62, not approximately 25%: the 81 formerly cue-bearing items
are intentionally no longer in that denominator, while the remaining 62 are
all non-key hedges. Treating that denominator as a success target would reward
leaving part of the cue intact.

The product-level replacement measurement uses every assembled item with at
least one hedge; multiple hedges are resolved by the harness's fixed
first-occurrence rule after assembly. Across the predeclared seed set 0..99
(100 seeds, unchanged from bd-2qvn):

| strategy | mean / 40 | min | max | pass >=27 |
|---|---:|---:|---:|---:|
| longest | 10.38 | 3 | 19 | 0/100 |
| always-A | 10.20 | 5 | 19 | 0/100 |
| always-B | 9.91 | 5 | 18 | 0/100 |
| always-C | 9.61 | 4 | 16 | 0/100 |
| always-D | 10.28 | 4 | 18 | 0/100 |
| hedged | 10.37 | 5 | 18 | 0/100 |
| stem-overlap | 11.86 | 6 | 18 | 0/100 |
| uniform-random | 9.94 | 5 | 20 | 0/100 |

The product harness reports 771 applicable assembled instances, 210 keyed
first-hedge choices, or 27.2%, inside its 20--30% control band. The strict
exactly-one diagnostic is 289 assembled instances, 0 keyed, as explained
above. The harness now asserts the hedged mean is 9--11/40 and the
applicable-any rate is 20--30%.

The original product comparison was hedged 12.27/40 versus random 9.94/40;
the current result is 10.37/40 versus the same 9.94/40 control. No seed
reached 27 on any strategy.

## Conditional-key spot checks

These ten keyed claims retain their original conditional or qualified force;
none was strengthened to remove `can`, `may`, `often`, or `typically`.

| id | key claim retained |
|---|---|
| m01-q002 | Reliability is about how often failures occur; availability includes uptime fraction considering repair |
| m01-q210 | Power path often leads facility events; cooling can cascade; human and process factors contribute — do not invent a fixed pie |
| m03-q200 | Flooding can disable power, cooling, access, and life-safety systems for extended periods |
| m04-q212 | A missing tile dumps plenum static so distant perforated tiles deliver less air; monitoring can alarm on a non-adjacent row |
| m06-q208 | Bypass can leave the load unprotected |
| m06-q104 | Can harm availability despite PUE gains |
| m09-q200 | Higher temperatures can improve plant efficiency |
| m10-q107 | Evaporative cooling can save energy but uses water; dry cooling saves water but may use more energy |
| m12-q034 | Can detect smoke at very low concentrations before conventional detectors |
| m15-q148 | Uncontrolled live-system work can cause outages |

## Should-fail review

For `m08-q209`, the distractor “Door design is decided by aesthetics rather
than airflow” was not qualified: adding `can` would make it arguably true on
a real project. Instead the solid-door distractor was qualified as “Solid
doors can improve inlet temperatures for front-to-back gear”; it remains
false because solid doors impede the intended front-to-back equipment airflow.

## Verification

All of these returned success after the final five cue corrections:

`construction-faults` (live detector hits 0; length-rank PASS),
`answer-key-skew`, `verify-bank`, `key-contradiction`, `verify-orphans`,
`validate-grounding`, `near-duplicate-items`, and
`cargo test --locked -p cdcp_assemble --lib` (9/9).

The construction-faults damaged corpus remains the expected RED control; it
is not the live-bank verdict. As always, these are four named guessing-route
checks, not evidence of truth, discrimination, or certification.
