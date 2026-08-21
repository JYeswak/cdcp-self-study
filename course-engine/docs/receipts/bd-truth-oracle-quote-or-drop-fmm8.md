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
The corrected 2026-08-21 sweep recorded the current bank after the three sampled
DEAD URLs were changed by the bank owner. `DEAD=0` is a correction plus nine
real repairs: pane 2 repaired nine genuinely broken links, while twenty-two of
the original thirty-one were response-classification errors corrected here.
The response classifier now uses the HTTP status itself: 403/429 are
BOT_BLOCKED regardless of host, while 404/410, DNS/connection failures remain
DEAD. The current-bank sweep recorded:

| measure | count |
| --- | ---: |
| cited | 1,307 |
| HTTP-resolved | 888 |
| resolved for grounding | 5 |
| DEAD | 0 |
| BOT_BLOCKED | 411 |
| supporting | 5 |
| non-supporting | 7 |
| unverifiable | 884 |

The live gate exits 2 (RED), naming DEAD, NON_SUPPORTING, and UNVERIFIABLE rows.
BOT_BLOCKED is reported but does not fail the item: the gate cannot verify that
class by machine. Five real items now have byte-present source quotes and return
SUPPORTING. A resolved page without a byte-present quote is still not treated as
supporting.

The excerpt is currently carried by the item schema's `# Source quote:` comment
convention, which `comment_claim` reads; it is not yet a typed TOML field. The
smallest successful excerpts were a single source sentence or clause. The first
q105 attempt used an older DOE sentence and correctly returned NON_SUPPORTING;
the replacement is the current sentence about dissolved solids causing scale
and corrosion.

The committed causal fixtures cover the branches: intact support passes; a 404
falsely marked supporting fails; a 200 response whose body lacks the claim,
falsely marked supporting, fails; an arbitrary 403 is BOT_BLOCKED without
failing; and deliberately bypassed BOT_BLOCKED and resolved-200 classifications
fail. The product tests read those fixtures; the live receipt is not used as a
green fixture.

## Boundary

GREEN would prove only that the citation receipt resolves and its exact excerpt
supports the claim. It would not prove pedagogical usefulness, source authority
for the learner's jurisdiction, or that the source remains current.
