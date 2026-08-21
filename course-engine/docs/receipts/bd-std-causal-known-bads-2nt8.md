# bd-std-causal-known-bads-2nt8 — causal known-bad inventory

## Scope and consumer

This receipt is consumed by `bd-std-causal-known-bads-2nt8` and by the
maintainer deciding whether a known-bad RED is evidence for the detector named
by the fixture. The defect class is **false causal attribution**: a fixture can
exit nonzero because of an unrelated error and still be reported as proof that
its intended detector fires. The receipt is deleted when every row has an
independent detector-bypass counterfactual recorded as `PROVEN` or an explicit
`CANNOT_DETERMINE` decision accepted by the bead owner; an intact RED alone is
not completion.

## Denominator

The primary shipped selftest population is predeclared as the 24 injection
rows emitted by the five outer suites:

| suite | rows |
|---|---:|
| `selftest_known_bad` | 6 |
| `selftest_l5` | 2 |
| `selftest_l6_coverage` | 2 |
| `selftest_l7_objectives` | 8 |
| `selftest_orphan` | 6 |
| **primary total** | **24** |

`selftest_l5_honesty` is a nested helper and emits one additional duplicate
credential-inflation injection. It was run and remains noted below, but is not
silently added to the primary 24-row denominator. Counting the complete shell
receipt surface would therefore be 25, with one duplicate detector leg.

## Inventory and run

The production commands were run from the current worktree with
`CDCP_BIN_DIR=target/debug`; no detector logic was duplicated in the fixture
scripts. Every primary row fired its named marker, and every suite's clean
control passed. The `selftest_known_bad` marker assertions were strengthened in
`7e89f07e` so its former bare nonzero checks now require the branch-specific
signal.

| # | suite / fixture | production detector and branch marker | clean control | bypass counterfactual |
|---:|---|---|---|---|
| 1 | `selftest_known_bad / flipped-golden` | `cdcp goldens check`; `GOLDEN MISMATCH` | clean goldens control PASS | CANNOT_DETERMINE |
| 2 | `selftest_known_bad / empty-bank` | bank loader; `empty bank` | clean goldens control PASS | CANNOT_DETERMINE |
| 3 | `selftest_known_bad / bank_hash-drift` | golden bank-hash comparison; `bank_hash drift` | clean goldens control PASS | CANNOT_DETERMINE |
| 4 | `selftest_known_bad / honesty-plant` | credential scan; `CDCP certified` | clean honesty scan PASS | CANNOT_DETERMINE |
| 5 | `selftest_known_bad / bank_hash-absent` | required-golden schema; `missing required golden(s)` | clean goldens control PASS | CANNOT_DETERMINE |
| 6 | `selftest_known_bad / goldens-vacuous-scan` | golden discovery anti-vacuity; `discovered 0 golden files` | clean goldens control PASS | CANNOT_DETERMINE |
| 7 | `selftest_l5 / flipped-golden-e2e` | WASM e2e digest; `GOLDEN MISMATCH` | clean e2e digest match PASS | CANNOT_DETERMINE |
| 8 | `selftest_l5 / empty-golden-dir` | e2e discovery anti-vacuity; `vacuous` | clean e2e digest match PASS | CANNOT_DETERMINE |
| 9 | `selftest_l6_coverage / empty-bank` | coverage domain scan; `empty bank` | live coverage GREEN | CANNOT_DETERMINE |
| 10 | `selftest_l6_coverage / m01-only-bank` | required-module floor; `module 2:` | live coverage GREEN | CANNOT_DETERMINE |
| 11 | `selftest_l7_objectives / empty-objectives` | objective registry anti-vacuity; `zero [[objective]]` | live objectives GREEN | CANNOT_DETERMINE |
| 12 | `selftest_l7_objectives / missing-claim-ref` | objective→claim link; `unresolved claim_id` | live objectives GREEN | CANNOT_DETERMINE |
| 13 | `selftest_l7_objectives / empty-claim-ids` | objective schema; `claim_ids empty` | live objectives GREEN | CANNOT_DETERMINE |
| 14 | `selftest_l7_objectives / empty-bank` | objective bank domain; `empty bank` | live objectives GREEN | CANNOT_DETERMINE |
| 15 | `selftest_l7_objectives / declared-module-starved` | derived module floor; `domain module 15: 0 approved < min 1` | reasoned exemption control GREEN | CANNOT_DETERMINE |
| 16 | `selftest_l7_objectives / exemption-without-reason` | exemption schema/floor; `coverage_exempt module 15 has no reason` | reasoned exemption control GREEN | CANNOT_DETERMINE |
| 17 | `selftest_l7_objectives / domain-min-undeclared` | registry cross-source drift; `[[domain_min]] module 15 is not declared` | declared-domain control GREEN | CANNOT_DETERMINE |
| 18 | `selftest_l7_objectives / topic-undeclared-domain` | topic/domain cross-source drift; `topics.toml: topic in an undeclared domain` | declared-topic control GREEN | CANNOT_DETERMINE |
| 19 | `selftest_orphan / empty-bank` | orphan scan anti-vacuity; `empty bank` | live orphan integrity GREEN | CANNOT_DETERMINE |
| 20 | `selftest_orphan / empty-topics` | topic-registry anti-vacuity; `empty topic registry` | live orphan integrity GREEN | CANNOT_DETERMINE |
| 21 | `selftest_orphan / orphan-item-ref` | forward reference integrity; `unknown topic_id` | clean specimen bank/live control GREEN | CANNOT_DETERMINE |
| 22 | `selftest_orphan / unanchored-item` | item anchoring; `missing/empty topic_ids` | clean specimen bank/live control GREEN | CANNOT_DETERMINE |
| 23 | `selftest_orphan / silently-empty-file` | file-granular anti-vacuity; `items[] yielded zero items` | clean specimen bank/live control GREEN | CANNOT_DETERMINE |
| 24 | `selftest_orphan / orphan-topic` | reverse reference integrity; `orphan topic 'zz-selftest-orphan-topic'` | live orphan integrity GREEN | CANNOT_DETERMINE |

Observed suite receipts: `selftest_known_bad` `INJECTIONS=6`, `selftest_l5`
`INJECTIONS=2`, `selftest_l6_coverage` `INJECTIONS=2`,
`selftest_l7_objectives` `INJECTIONS=8`, and `selftest_orphan`
`INJECTIONS=6`; all exited 0 after their RED legs and clean controls. The
nested `selftest_l5_honesty` also exited 0 with its one planted credential
injection and clean-after-restore control.

## Causal status

The intact detector leg is proven for 24/24 primary rows, including a
branch-specific marker for every row. Positive controls are present at the
detector-family/suite level and were green. **The bypass leg is not proven for
any row.** These production commands have no safe, explicit detector-bypass
mode; adding a magic bypass flag would create a new production escape hatch,
not evidence. Source-mutating scratch rebuilds are the remaining work needed
to turn each `CANNOT_DETERMINE` into `PROVEN` or a human-accepted boundary.

Therefore this receipt is a measured partial result, not a close: `causal =
0/24`, `intact = 24/24`, `clean controls = 5/5 outer suites`,
`bypass = 0/24 proven`, `unrun = 24`, `cannot determine = 24`.

What this inventory cannot decide: a marker proves that the production command
reported the named branch, not that the branch was the sole cause of its exit;
the clean control proves the suite is not unconditionally red, not that every
detector has been causally isolated. No learner correctness or credential claim
follows from this fixture audit.
