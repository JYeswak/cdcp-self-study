# Absolute/universal plausibility detector — 2026-08-21

## Decision and boundary

This is a measurement-first floor raise for the lexical absolute/universal
sub-case of F-01. It does not rewrite any item. The detector reports a branch
when exactly three options in a four-option set carry a marker derived from the
bank corpus and the one unmarked option is the key. It cannot decide semantic
absurdity (for example, whether an option means “serve only as a diesel storage
yard” for an edge data centre), and it cannot decide every off-topic
distractor. Stem overlap covers part of off-topicness; this detector does not
claim the rest.

The production implementation is `cdcp_bank::plausibility`, with the thin
auto-registered dispatcher `cdcp_gate plausibility-detector`. No content was
rewritten and no production bypass flag was added.

## Predeclared denominators

These denominators were declared before reading the result:

1. **Assembler:** seeds `0..99` inclusive, exactly 100 forms, 40 items per
   form, so 4,000 assembled item instances. The forms are the return values of
   `cdcp_assemble::assemble` over the approved pool; the example does not
   resample `bank/items` itself.
2. **Bank-wide:** all 957 four-choice single-select rows in `bank/items`,
   including 931 approved rows and 26 retained non-approved rows. This is the
   authored-corpus denominator, not an approved-only rewrite denominator.
3. **Applicable population:** exactly three marked options and one unmarked
   option. Rows with one or two markers are non-pattern rows; rows with zero
   markers have no lexical evidence; rows with four markers have no unique
   unmarked option. Zero- and four-marker rows are explicitly excluded as
   required, and one-/two-marker rows are also outside the declared cue shape.
   The reported hit rate is `key_is_lone_plausible / applicable_exactly_three`.

The chance floor for a key matching the one unmarked option is 25.0%.

## Corpus-derived marker inventory

The inventory starts from the absolute, universal, and totalising phrases
observed in the UX findings and filters the candidate phrases by occurrence in
actual option text. A phrase absent from the corpus is not active in the
detector. The run saw 3,828 option texts and derived these terms; the final
number is the count of option texts containing each term:

| category | observed marker: option occurrences |
|---|---|
| absolute | `all`: 145; `every`: 111; `always`: 119; `never`: 79; `none`: 2; `zero`: 30; `any`: 82; `only`: 488; `no`: 194 |
| universal | `regardless of`: 92 |
| totalising | `immunity`: 2; `immunity from all`: 1; `guarantee`: 7; `guarantees`: 3; `guaranteed`: 11; `universally`: 2; `permanently`: 25; `entirely`: 27; `automatic`: 43 |

`in all cases`, `without exception`, and `eliminates entirely` were candidate
generalizations but did not occur in an option and therefore were not activated.

## Real assembler result

Command:

```text
cargo run -q -p cdcp_assemble --example plausibility_detector
```

All 100 predeclared forms assembled successfully. The exact-three population
and key comparison were:

```text
ASSEMBLER seeds=100 forms=100 items=4000 applicable=545 key_is_lone_plausible=498 rate=91.4% chance_floor=25.0% delta=+66.4pp
FORM_MEAN per_40_items key_is_lone_plausible=4.98 random_control_expected=1.36 random_control_realized=1.35
```

The random control uses a separately seeded ChaCha12 draw, one uniform option
index per applicable item. `random_control_expected` is the exact 25% control
(`applicable / 4`) and `random_control_realized` is the observed seeded draw.
The measured key cue is therefore 498/545 = 91.4%, versus the 25.0% chance
floor and 1.36 expected hits per 40-item form.

## Bank-wide and per-module result

```text
BANK_WIDE scanned=957 distribution=[0:263,1:317,2:240,3:135,4:2] applicable_exactly_three=135 non_pattern_one_or_two=557 excluded_zero=263 excluded_all_four=2 key_is_lone_plausible=126 rate=93.3% chance_floor=25.0% delta=+68.3pp
```

The bank-wide rate is 126/135 = 93.3%. The two four-marker rows have no
unique unmarked option; the 263 zero-marker rows have no lexical evidence; and
the 557 one-/two-marker rows do not have the declared three-marked/one-unmarked
shape. None was silently counted as a clean negative.

| module | scanned | applicable exactly 3 | 1–2 markers | zero | all 4 | key hits | rate |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 01 | 43 | 11 | 23 | 8 | 1 | 11 | 100.0% |
| 02 | 52 | 9 | 30 | 13 | 0 | 5 | 55.6% |
| 03 | 55 | 11 | 33 | 11 | 0 | 11 | 100.0% |
| 04 | 39 | 4 | 28 | 7 | 0 | 4 | 100.0% |
| 05 | 33 | 2 | 17 | 14 | 0 | 2 | 100.0% |
| 06 | 146 | 25 | 91 | 29 | 1 | 25 | 100.0% |
| 07 | 35 | 0 | 21 | 14 | 0 | 0 | n/a |
| 08 | 39 | 5 | 15 | 19 | 0 | 5 | 100.0% |
| 09 | 127 | 25 | 86 | 16 | 0 | 25 | 100.0% |
| 10 | 37 | 7 | 23 | 7 | 0 | 7 | 100.0% |
| 11 | 81 | 15 | 59 | 7 | 0 | 15 | 100.0% |
| 12 | 66 | 3 | 16 | 47 | 0 | 1 | 33.3% |
| 13 | 50 | 6 | 30 | 14 | 0 | 5 | 83.3% |
| 14 | 48 | 4 | 29 | 15 | 0 | 4 | 100.0% |
| 15 | 106 | 8 | 56 | 42 | 0 | 6 | 75.0% |

Module 07 has no exact-three population, so its rate is not rounded into a
pass or fail. The concentration is systemic: 14 of 15 modules have at least
one applicable row, and 126 of 135 applicable rows key the lone unmarked
option.

## Causal known-bad and known-good

The product test plants this known-bad option set before evaluating it:

```text
A  A bounded response                         (key; unmarked)
B  Always remove every safeguard              (marked)
C  No human factor ever matters               (marked)
D  Guarantees immunity from all failures      (marked)
```

The production `detect_item` path returns the named branch marker
`absolute-universal-lone-plausible`, including the item id and marker option
letters. The known-good control gives each option a marker, so it has no unique
unmarked option and does not fire. A zero-marker control is also not
applicable.

The bypass counterfactual injects a test-only detector function at the same
call boundary; it does not add a production CLI or environment escape hatch.
The intact known-bad fixture is RED with the branch marker, and the scratch
bypassed detector is PASS. The verdict changes only when the named detector is
neutralized, so this row is **causal=1/1 for this detector**. A fixture that
remained RED under the bypass would have been recorded as non-causal.

## Gate output

`cdcp_gate plausibility-detector` exits 2 on the current bank and prints the
same bank-wide/module measurement plus named branch findings. Its limitation
line is deliberate:

```text
LIMITATION: lexical absolute/universal sub-case of F-01 only; semantic absurdity and some off-topic distractors remain outside this detector (stem-overlap covers only part of off-topicness)
```

This is a floor raise and a review queue, not a claim that all 126 rows are
semantically defective. No item was rewritten before measuring.

## Verification

The focused product and dispatcher checks passed:

```text
cargo test -p cdcp_bank --lib plausibility::tests -- --nocapture
5 passed; 0 failed
cargo test -p cdcp_gate --test plausibility_detector_e2e -- --nocapture
3 passed; 0 failed
cargo check -p cdcp_assemble --example plausibility_detector
Finished successfully
```
