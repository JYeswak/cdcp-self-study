# Grounding-wave residue adjudication (bd-0cjy)

Review base: `955a8f1`. This receipt closes the four remaining item-level decisions after the independent review of `docs/receipts/.gw-narrowed.txt`. No `crates/` file was changed.

## Residual detector flags

Before this pass, `grounding-wave` reported:

```text
approved=929; template-stem=1 [m09-q207]; recall-only-stem=1 [m09-q144]
```

| id | detector | adjudication | action and evidence |
|---|---|---|---|
| `m09-q207` | template-stem | **KEEP — pre-existing and sound** | The stem, choices, key, and explanation are the pre-wave substantive containment item, not a document-heading question. The similarity is the intentional three-item containment teaching family (`m09-q149`, `m09-q206`, `m09-q207`), with distinct hot/cold propositions and keyed answers. No edit. |
| `m09-q144` | recall-only-stem | **RESTORE — wave damage** | The post-baseline `f2a55eb` version replaced “Supplemental cooling generally means” and its applied choices with ISO/IEC 22237-4 heading recall. The exact pre-wave item was restored and remains `approved`. |

After the restoration, the detector reports `recall-only-stem=0`; the one remaining template hit is the adjudicated, pre-existing `m09-q207` family. The gate therefore remains RED for that known legitimate detector hit; it was not made green by weakening or rewriting the detector.

## Draft adjudication

Both drafts were `approved` at `955a8f1`, both use `source_class = "original"`, and neither is left unresolved. The source-citation premise that caused the withdrawal was not applied as a veto to these original educational questions.

| id | baseline stem | decision | resulting status |
|---|---|---|---|
| `m14-q121` | `Integrated Systems Testing (IST) is valuable primarily because it:` | Restore the pre-wave stem, choices, key, explanation, and baseline item record. | `approved` |
| `m15-q357` | `An MOP was approved once, but its revision was never tested and an obsolete PDF remains at the operator console. What control is missing?` | Restore the pre-wave stem, choices, key, explanation, and NE O 422.1 source record. | `approved` |

The live result is `verify-bank: PASS`, with 957 items scanned and 931 approved. There are zero remaining `status = "draft"` items.

## Answer-key bead decision (`bd-opyi`)

The requested test uses the settled pool before re-approving the two drafts, because that is the reported `n=929` measurement:

| quantity | value |
|---|---:|
| approved single-select items, n | 929 |
| A keys, x | 274 |
| observed A share | 29.4941% |
| uniform-null expectation | 232.25 |
| excess over 25% | 4.4941 percentage points |
| binomial standard deviation | 13.1980 |
| continuity-corrected z | 3.1255 |
| exact one-sided `P(X >= 274 \| n=929, p=0.25)` | 0.0010524 |

Under the narrow iid-binomial model, this is statistically distinguishable from exactly uniform answer-key selection; it should not be described as random noise under that model. But the approved bank is a fixed, authored population, not 929 independent random draws from a key-generation process. The p-value therefore does not identify a construction cause or prove that remediation is warranted. The measured 29.5% A share is inside the declared answer-key gate band of 15–35%, and the original 54.8% premise was wave damage rather than a pre-existing pool property.

After the two justified re-approvals, the final approved distribution is A=276 (29.6%), B=245 (26.3%), C=209 (22.4%), D=201 (21.6%), n=931; the same one-sided test is `p=0.0007329`. That confirms a mild measurable A-lean, not the former P0 construction failure. Recommendation: close `bd-opyi` as the completed answer-key-skew instrument/gate bead; do not infer that the bank is perfectly balanced, and do not open a remediation bead solely from this binomial test.

## Closure accounting

- 407 `REPLACED` items are restored.
- 13 `GENUINE` items and 10 `KEEP` items are deliberately unchanged.
- All 130 narrowed-review IDs are adjudicated; no ID remains unadjudicated.
- `verify-bank` is GREEN.
- `grounding-wave` is intentionally still RED only for the documented, pre-existing `m09-q207` template family.
