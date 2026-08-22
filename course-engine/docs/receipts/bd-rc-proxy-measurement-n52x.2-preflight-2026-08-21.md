# W4 content-wave preflight — 2026-08-21

Bead: `bd-rc-proxy-measurement-n52x.2` (`W4: content wave — GATED on W1a and W1b measurements`)

This is a preflight intersection, not a rewrite wave.  No bank item, key,
explanation, stem, seed, or module page was changed.  The explicit operator
instruction to measure before editing is still honored.

## Preconditions

Both measurement gates are now present and independently reproducible:

- **W1a / plausibility:** `docs/receipts/plausibility-detector-2026-08-21.md`;
  the live `cdcp_gate plausibility-detector` exits `2` with the named branch
  `absolute-universal-lone-plausible`, reporting `126/135 = 93.3%` bank-wide
  and `91.4%` through assembler seeds `0..99`.
- **W1b / teaching-test census:**
  `docs/receipts/bd-rc-unjoined-artifacts-3ri3.1-teaching-test-census-2026-08-22.md`;
  the live `teaching_mismatch` example reports `105/931 = 11.3%` as a
  **lexical review floor**, not a semantic teaching rate.  Its five prior
  human cases remain lower-bound evidence, not an extrapolation.

The W1a causal tests remain green: five product tests and three dispatcher E2E
tests pass.  W1b's declared should-fail (`m03-q217`, initially judged absent,
then restored to TAUGHT after reading the dedicated BTM section) remains in the
receipt.  Neither measurement is being promoted to a learner-outcome claim.

## Predeclared intersection

The candidate review set is the exact intersection of:

1. item IDs emitted by the live W1a detector's named branch, and
2. item IDs emitted by the live W1b `FINDING` rows (SHALLOW or ABSENT).

This is a prioritization set, not the W4 denominator and not an automatic
rewrite list.  A marker hit can be a bounded, correct answer; a lexical
teaching-floor hit can be a false review lead.  A human must read the stem,
all options, explanation, module prose, and supporting source before changing
anything.

## Result

```text
W4_PREFLIGHT plausibility_hits=126 teaching_review_floor=105 intersection=14
intersection_by_module=m02=1,m03=2,m06=3,m09=1,m10=2,m11=4,m14=1
```

Exact intersection (14 items):

| module | item IDs |
|---:|---|
| m02 | `m02-q206` |
| m03 | `m03-q205`, `m03-q240` |
| m06 | `m06-q082`, `m06-q086`, `m06-q252` |
| m09 | `m09-q145` |
| m10 | `m10-q210`, `m10-q211` |
| m11 | `m11-q107`, `m11-q122`, `m11-q206`, `m11-q235` |
| m14 | `m14-q209` |

The remaining 112 detector hits are not in the W1b lexical review-floor set;
the remaining 91 W1b rows do not match the absolute/universal detector branch.
That split is exactly why the two proxies must not be collapsed into a single
"bad item" count.

## Reproduction

```text
./target/debug/cdcp_gate plausibility-detector
# expected exit 2; parse only the named FAIL branch item IDs

cargo run -q -p cdcp_assemble --example teaching_mismatch
# expected exit 0; parse the declared FINDING rows
```

The detector's limitation remains binding: lexical absolute/universal wording
does not decide semantic absurdity or all off-topic distractors.  The W1b
limitation remains binding: topic/evidence token presence does not decide
whether a learner is taught to make the decision.

## Next step and stop condition

The 14-item intersection is ready for human adjudication.  It is not permission
to rewrite.  Before any bank edit, the reviewer must decide whether each row is
actually a plausible distractor defect, a teaching mismatch, both, or neither,
and must record at least one should-fail row that is deliberately left alone.
Only after that review can a content patch re-run the full coupled property
table: plausibility, option length, key skew, lone/avoid hedge, stem overlap,
all/none, grammatical agreement, and the independent key cross-check.

W4 is therefore **preflight complete, content edits intentionally not started**.
