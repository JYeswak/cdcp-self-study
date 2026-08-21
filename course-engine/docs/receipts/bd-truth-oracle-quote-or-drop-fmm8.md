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
The response classifier now uses the HTTP status itself: 403 is BOT_BLOCKED
regardless of host; 404/410 and explicit DNS resolution failures are DEAD; and
429, 5xx, timeouts, connection refusals, and other transport failures are
INDETERMINATE. The refresh applies a 75 ms per-host request interval and two
exponential-backoff retries. The current-bank sweep recorded:

| measure | count |
| --- | ---: |
| cited | 1,307 |
| HTTP-resolved | 889 |
| resolved for grounding | 5 |
| DEAD | 0 |
| BOT_BLOCKED | 410 |
| INDETERMINATE | 0 |
| supporting | 5 |
| non-supporting | 7 |
| unverifiable | 885 |

The live gate exits 2 (RED), naming DEAD, NON_SUPPORTING, and UNVERIFIABLE rows.
BOT_BLOCKED and INDETERMINATE are reported but do not pretend to be source
evidence; both fail closed for admission, while only BOT_BLOCKED is exempt from
the citation-defect count. Five real items now have byte-present source quotes
and return SUPPORTING. A resolved page without a byte-present quote is still not
treated as supporting.

The excerpt is currently carried by the item schema's `# Source quote:` comment
convention, which `comment_claim` reads; it is not yet a typed TOML field. The
smallest successful excerpts were a single source sentence or clause. The first
q105 attempt used an older DOE sentence and correctly returned NON_SUPPORTING;
the replacement is the current sentence about dissolved solids causing scale
and corrosion.

The committed causal fixtures cover the branches: intact support passes; a 404
falsely marked supporting fails; a 200 response whose body lacks the claim,
falsely marked supporting, fails; an arbitrary 403 is BOT_BLOCKED without
failing; a 429/timeout is INDETERMINATE rather than DEAD; and deliberately
bypassed BOT_BLOCKED, INDETERMINATE, and resolved-200 classifications fail. The
product tests read those fixtures; the live receipt is not used as a green
fixture.

## URL extraction correction

The next refresh found and fixed a transport defect in `clean_url`: it had
removed every terminal `)`, including the balanced `(CDCP)` and `(CDFOS)` path
segments. The extractor now removes only unmatched closing paired delimiters and
leaves legal URL punctuation such as commas and semicolons intact. The causal fixture
`crates/cdcp_bank/tests/fixtures/quote_or_drop/parenthesized_url.toml` proves
that the complete URL reaches the real `curl` fetch path as
`GET /source_(v1)`. Reverting that extraction change makes the fixture fail
before fetch and on the request path.

The corrected refresh still measured `non_supporting=7`, not a drop. The four
EPI rows now contain the complete URLs and return HTTP 200, but their fetched
pages do not contain the recorded excerpts; the URL truncation was real, but it
was not the cause of those four verdicts. The three other rows remain
non-supporting: the two HSG250 URLs are explicitly edition pins with no claim
taken in their item comments, while OSHA 1910.304 contains related overcurrent
requirements but not the exact 1910.303 excerpt recorded for m06-q248. This
exposes a separate modeling issue: the current one-quote-per-item schema
cannot distinguish a claim-bearing source from a heading or edition-pin URL.
That issue is not silently reclassified as a URL fix.

The inventory found 343 of 957 items (36%) with an EPI `/services/` course
page. All 343 also carry at least one non-EPI URL; zero are EPI-only by URL
inventory. That is a mechanical pointer count, not a claim that all 343 other
URLs are evidentiary: URL shape can identify the EPI marketing pointer, but it
cannot determine whether another page actually supports this item's claim.
That role requires reading the citation in context. The durable schema should
therefore separate `scope_pointers` (syllabus/topical links, not evidence) from
`evidence_sources` (claim-bearing URLs with an excerpt or a human-verification
record). Quote-or-drop should skip scope pointers for support adjudication but
require each one to carry the syllabus heading it points to; it should remain
fail-closed for evidence sources. The current one-quote-per-item comment cannot
make that distinction, so no 343-item reclassification is being automated.

## Boundary

GREEN would prove only that the citation receipt resolves and its exact excerpt
supports the claim. It would not prove pedagogical usefulness, source authority
for the learner's jurisdiction, or that the source remains current.
