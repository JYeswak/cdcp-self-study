# bd-truth-oracle-quote-or-drop-fmm8

Consumer: the `cdcp_gate quote-or-drop` citation-grounding gate. Feature: stop
an item from treating an unchecked or unsupported public URL as evidence.
Observed defect: 917 of 957 item files carried URLs, but no code opened them or
checked that source text supported the item claim. Deletion condition: remove
this receipt and gate only after a successor truth oracle preserves the same
fail-closed and causal legs.

## Design

The network operation is an explicit periodic Rust sweep:

```text
cdcp_gate quote-or-drop --refresh
```

It is deliberately not part of `check.sh`: network availability is not
hermetic, and a slow standards site must not make the deterministic chain
flaky. The ordinary `cdcp_gate quote-or-drop` invocation reads the committed
receipt, checks it against the current bank SHA and the predeclared policy
denominators, and fails closed. A 2xx response without an exact recorded source
quote is `UNVERIFIABLE`; it is not a pass. PDF sources are excluded by policy.

## First sweep

The policy predeclares 957 item files and 1,307 unique item/URL citation rows.
The 2026-08-21 sweep recorded:

| measure | count |
| --- | ---: |
| cited | 1,307 |
| HTTP-resolved | 879 |
| resolved for grounding | 0 |
| supporting | 0 |
| unreachable | 422 |
| non-supporting | 0 |
| unverifiable | 885 |

The live gate exits 2 (RED), naming the unresolved rows. The zero supporting
count is an honest finding: existing item comments contain source summaries,
not exact source quotations, so the sweep cannot promote them. A resolved page
without a byte-present quote is not treated as supporting.

The committed causal fixtures cover all three branches: intact support passes;
a 404 falsely marked supporting fails; and a 200 response whose body lacks the
claim, falsely marked supporting, fails. The product tests read those fixtures;
the live receipt is not used as a green fixture.

## Boundary

GREEN would prove only that the citation receipt resolves and its exact excerpt
supports the claim. It would not prove pedagogical usefulness, source authority
for the learner's jurisdiction, or that the source remains current.
