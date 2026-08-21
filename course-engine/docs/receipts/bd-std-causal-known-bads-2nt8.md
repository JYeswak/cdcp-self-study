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
| 10 | `selftest_l6_coverage / m01-only-bank` | required-module floor; `module 2:` | live coverage GREEN | **PROVEN** (scratch counterfactual) |
| 11 | `selftest_l7_objectives / empty-objectives` | objective registry anti-vacuity; `zero [[objective]]` | live objectives GREEN | CANNOT_DETERMINE |
| 12 | `selftest_l7_objectives / missing-claim-ref` | objective→claim link; `unresolved claim_id` | live objectives GREEN | **PROVEN** (scratch counterfactual) |
| 13 | `selftest_l7_objectives / empty-claim-ids` | objective schema; `claim_ids empty` | live objectives GREEN | CANNOT_DETERMINE |
| 14 | `selftest_l7_objectives / empty-bank` | objective bank domain; `empty bank` | live objectives GREEN | CANNOT_DETERMINE |
| 15 | `selftest_l7_objectives / declared-module-starved` | derived module floor; `domain module 15: 0 approved < min 1` | reasoned exemption control GREEN | **PROVEN** (scratch counterfactual) |
| 16 | `selftest_l7_objectives / exemption-without-reason` | exemption schema/floor; `coverage_exempt module 15 has no reason` | reasoned exemption control GREEN | CANNOT_DETERMINE |
| 17 | `selftest_l7_objectives / domain-min-undeclared` | registry cross-source drift; `[[domain_min]] module 15 is not declared` | declared-domain control GREEN | CANNOT_DETERMINE |
| 18 | `selftest_l7_objectives / topic-undeclared-domain` | topic/domain cross-source drift; `topics.toml: topic in an undeclared domain` | declared-topic control GREEN | CANNOT_DETERMINE |
| 19 | `selftest_orphan / empty-bank` | orphan scan anti-vacuity; `empty bank` | live orphan integrity GREEN | CANNOT_DETERMINE |
| 20 | `selftest_orphan / empty-topics` | topic-registry anti-vacuity; `empty topic registry` | live orphan integrity GREEN | CANNOT_DETERMINE |
| 21 | `selftest_orphan / orphan-item-ref` | forward reference integrity; `unknown topic_id` | clean specimen bank/live control GREEN | **PROVEN** (scratch counterfactual) |
| 22 | `selftest_orphan / unanchored-item` | item anchoring; `missing/empty topic_ids` | clean specimen bank/live control GREEN | **PROVEN** (scratch counterfactual) |
| 23 | `selftest_orphan / silently-empty-file` | file-granular anti-vacuity; `items[] yielded zero items` | clean specimen bank/live control GREEN | **PROVEN** (scratch counterfactual) |
| 24 | `selftest_orphan / orphan-topic` | reverse reference integrity; `orphan topic 'zz-selftest-orphan-topic'` | live orphan integrity GREEN | **PROVEN** (scratch counterfactual) |

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

## Rust negative-test census (measurement, not a new causal denominator)

The bead names every negative fixture in the `cdcp_gate` and `cdcp_bank` test
trees, so the five shell suites above are not the whole review surface. A
source census at this commit found **20 Rust test sources and 310 test
functions**. It deliberately reports two reproducible name-based populations
without pretending that a function name is a production fixture contract:

* **125 explicit-negative names** match `bad_`, `anti_vacuous`, `known_bad`, or
  `known-bad`.
* **232 broad candidates** match the wider failure vocabulary (`empty`,
  `missing`, `zero`, `drift`, `red`, `invalid`, `unlisted`, and related
  terms).

The broad set includes known-good controls, usage/schema tests, and tests whose
name merely mentions a failure mode. Therefore neither 125 nor 232 is added
to the predeclared 24-row production denominator. Each candidate still needs
an explicit detector/marker mapping and an independent bypass run before it
can become a causal row.

| Rust source | tests | explicit-negative names | broad candidates |
|---|---:|---:|---:|
| `crates/cdcp_bank/tests/c2_charter_pair.rs` | 1 | 0 | 0 |
| `crates/cdcp_bank/tests/leftover_honesty.rs` | 8 | 0 | 5 |
| `crates/cdcp_bank/tests/near_duplicate.rs` | 12 | 1 | 10 |
| `crates/cdcp_bank/tests/paraphrase_pairs.rs` | 8 | 0 | 7 |
| `crates/cdcp_gate/tests/anti_vacuous_topics.rs` | 9 | 1 | 4 |
| `crates/cdcp_gate/tests/capability_maturity_e2e.rs` | 30 | 19 | 25 |
| `crates/cdcp_gate/tests/census_charter_pair.rs` | 3 | 0 | 1 |
| `crates/cdcp_gate/tests/construction_faults_e2e.rs` | 3 | 0 | 3 |
| `crates/cdcp_gate/tests/differential_verdict_census.rs` | 18 | 1 | 6 |
| `crates/cdcp_gate/tests/dispatch.rs` | 6 | 0 | 5 |
| `crates/cdcp_gate/tests/doc_facts_e2e.rs` | 41 | 26 | 32 |
| `crates/cdcp_gate/tests/goldens_couplings_e2e.rs` | 42 | 31 | 36 |
| `crates/cdcp_gate/tests/near_duplicate_items_e2e.rs` | 2 | 0 | 2 |
| `crates/cdcp_gate/tests/rebase_module_bounds.rs` | 41 | 16 | 33 |
| `crates/cdcp_gate/tests/repo_surface.rs` | 15 | 0 | 8 |
| `crates/cdcp_gate/tests/restore_rebuild_trap.rs` | 9 | 0 | 6 |
| `crates/cdcp_gate/tests/s0_charter_pair.rs` | 1 | 0 | 1 |
| `crates/cdcp_gate/tests/substrate_guard_e2e.rs` | 61 | 30 | 48 |
| `crates/cdcp_gate/tests/support/mod.rs` | 0 | 0 | 0 |
| `crates/cdcp_gate/tests/support/rebuild.rs` | 0 | 0 | 0 |
| **total** | **310** | **125** | **232** |

The census also found existing source-level mutation/delete specimens that
must not be mistaken for executed causal rows: the C2 status-hash pair in
`c2_charter_pair.rs`, the two assertion-deletion branches in
`census_charter_pair.rs`, the restore/rebuild trap's mutation-plus-deletion
legs, and the S0 pair driven by `selftest_reconstructed.sh`. They demonstrate
the shape of a bypass proof, but this receipt does not promote them to the
24-row result without a fresh run recording the exact production detector and
both exit outcomes.

**Updated status:** the primary result remains `causal=0/24`,
`intact=24/24`, `bypass=0/24 proven`, `CANNOT_DETERMINE=24`. The new census
expands the worklist; it does not make a name match, a clean control, or a
source-level pair into a causal certificate. The bead remains open.

## Fresh intact-leg re-run

At commit `8eac0d40`, with `CDCP_BIN_DIR=target/debug`, the five production
selftest suites were re-run individually after the census. No live bank,
registry, or shared pane file was modified; each suite kept its plants in a
temporary directory and restored its clean control.

| suite | RED injections reported | clean control / result |
|---|---:|---|
| `selftest_known_bad.sh` | 6 | goldens control PASS; suite PASS |
| `selftest_l5.sh` | 2 | honesty + WASM digest controls PASS; suite PASS |
| `selftest_l5_honesty.sh` (nested) | 1 | honesty control PASS; suite PASS |
| `selftest_l6_coverage.sh` | 2 | live coverage PASS; suite PASS |
| `selftest_l7_objectives.sh` | 8 | live objective coverage PASS; suite PASS |
| `selftest_orphan.sh` | 6 | live orphan integrity PASS; suite PASS |

The suites therefore re-confirmed the intact branch markers and the five
clean controls. This is **not** a bypass result: no detector was neutralized,
so the primary causal status remains `0/24`, with all 24 bypass results still
`CANNOT_DETERMINE`. The re-run proves the intact leg was not merely stale; it
does not prove that any named detector was the sole cause of its RED.

## Cited-gate causal counterfactuals

The three gates cited in the Q24 dispatch are a **supplemental denominator of
3**. They are not rows in the predeclared 24-row shell-injection denominator;
adding them to that denominator would silently change the question being
measured. The primary result therefore remains `causal=0/24`, while the
separately reported cited-gate result is `causal=3/3`.

Each row was declared before its fixture was run. The intact run had to reach
the named detector and print its branch-specific marker. The bypass run used
only a scratch copy of that detector's source, with the named branch
neutralized; no live product source or shared pane file was changed. Thus the
counterfactual expected by the harness is `intact RED` and `bypassed PASS` for
the known-bad fixture. If the same RED assertion were run against the bypass
copy, it would fail, which is the causal signal rather than a second green
test.

| cited gate | fixture and expected branch | positive control | intact result | bypass result | causal verdict |
|---|---|---|---|---|---|
| `cdcp_gate tick-reconcile` | one closed bead (`bd-unreceipted`) absent from the ledger; marker `closed ... with no tick receipt` / `Unreconciled` | scratch ledger contained one closed, receipted bead and no unrelated unreceipted close | `RED`, rc=2; marker printed | `PASS`, rc=0 after scratch-only receipt-membership bypass | **PROVEN** |
| `pack_freshness` | bank commit is newer than the three required learner packs; marker `learner pack is stale` | one committed bank item plus all three nonempty pack files; only the bank epoch was advanced | `RED`, rc=2; `bank_epoch > pack_epoch` and marker printed | `PASS`, rc=0 after scratch-only freshness-comparison bypass | **PROVEN** |
| derived substrate floor (`require_tree_derived_floor`) | `InvocationWalk` contains only `scripts/__q24_missing__.py`, with `exists=false`; marker `tree-derived invocation/presence floor is 0` | a second walk containing real `scripts/check.sh` with `exists=true` returned floor=1 | `Err`, with the zero-floor marker | `Ok(0)`, printed `BYPASSED PASS`, after scratch-only `n == 0` branch bypass | **PROVEN** |

The tick-reconcile scratch module was copied from the current worktree,
including its then-uncommitted bytes; the pack-freshness and substrate-floor
scratch modules were copied from the current source tree. Each scratch build
compiled the production helper and then ran the same fixture once intact and
once with only the named detector neutralized. The floor's positive control
also demonstrates that this is not an empty or unrun scan: a real walked path
is accepted while the ghost-only walk is rejected.

These three results move the cited supplemental measure from `0/3` to `3/3`
proven. They do **not** change the primary 24-row result: the five shell
suites still have 24 intact RED rows and no bypass proof. The receipt now has
an honest partial result rather than citing the three newly named gates as if
they were among the original 24.

## Primary causal progress: coverage shortfall

The predeclared primary denominator is still 24. Row 10 is now proven causal,
so the primary measure is **`causal=1/24`, `intact=24/24`**. This row used the
production `verify_coverage` implementation from `cdcp_bank` in an isolated
scratch Cargo package; it did not duplicate the detector in test code.

The fixture had a valid 15-module domain registry, a present policy whose
default floor is one, and exactly one approved item in module 1. No empty-bank,
missing-registry, malformed-policy, or other unrelated precondition was
introduced. The intact evaluation returned code 1 and its stdout contained
the branch-specific marker `module 2: 0 approved < min 1`. A positive control
with one approved item in each of all 15 modules returned code 0.

For the bypass, only the production shortfall condition
`i128::from(have) < need` was changed to `false` in the scratch source. The
same m01-only fixture then returned code 0 and printed
`BYPASSED PASS: shortfall detector disabled`. A RED assertion against that
bypassed run would fail, which is the required counterfactual. The remaining
23 primary rows retain their prior `CANNOT_DETERMINE` status; no row is
promoted by census or by a broad nonzero exit.

## Primary causal progress: orphan-topic reverse reference

Primary status is now **`causal=2/24`, `intact=24/24`**. For row 24, the
production `cdcp_bank::orphans` implementation ran from an isolated scratch
Cargo package over a valid topic registry containing `t-used` and `t-lonely`
and one approved item referencing only `t-used`. The intact result was RED
and included the exact marker `orphan topic 't-lonely': declared in
topics.toml`.

A positive control added one approved item referencing `t-lonely`; the same
production implementation returned GREEN. The bypass changed only the
production orphan-topic filter to an always-false predicate in the scratch
source. The original one-item fixture was then rerun and returned GREEN,
printing `BYPASSED PASS: orphan-topic detector disabled`. A RED assertion
against that bypass would fail, proving that this fixture's intact RED is
caused by the reverse-reference detector rather than by a shared scan error.

## Primary causal progress: unanchored item

Primary status is now **`causal=4/24`, `intact=24/24`**. Row 22 used a
valid topic registry and one anchored approved control item, plus one approved
item with no `topic_ids` field. The intact production implementation returned
RED and printed `q24-unanchored: missing/empty topic_ids`.

Removing the unanchored item produced GREEN. The scratch bypass removed only
the production `unanchored.push(...)` action in the missing-topic branch; the
same bad item then returned GREEN as `BYPASSED PASS: unanchored detector
disabled`. The bypass would fail the intact RED assertion, isolating this
fixture to the item-anchoring detector.

## Primary causal progress: orphan-item forward reference

Primary status is now **`causal=3/24`, `intact=24/24`**. Row 21 used a
valid topic registry and a bank containing one good approved item referencing
`t-used` plus one planted approved item referencing unknown `t-nope`. The
intact production `cdcp_bank::orphans` evaluation returned RED and printed
the branch marker `q24-orphan-ref: unknown topic_id 't-nope'`.

Removing only the planted item produced the positive-control GREEN. In the
scratch source, bypassing only the `if !known.contains(s)` branch caused the
same two-item fixture to return GREEN, printed as
`BYPASSED PASS: orphan-ref detector disabled`. The bypass therefore fails the
fixture's RED assertion, proving the forward-reference detector is causal.

## Primary causal progress: file-granular vacuity

Primary status is now **`causal=5/24`, `intact=24/24`**. Row 23 used one
valid approved item and a second TOML file containing `items = []`. The intact
production `cdcp_bank::orphans` implementation returned RED with the exact
marker `empty.toml: items[] yielded zero items`.

Removing that empty file produced GREEN. The scratch bypass changed only the
`Some(tables) if tables.is_empty()` arm to an unreachable condition; the same
fixture then returned `BYPASSED PASS: empty-items-file detector disabled`.
The branch-specific RED assertion would fail under that bypass, so this row
is causal rather than merely a generic nonzero result.

## Current measured status

This final status line supersedes earlier chronological progress paragraphs:
the predeclared primary denominator is **24**, with **5/24 causal**, **24/24
intact**, and **19/24 CANNOT_DETERMINE**. The separately predeclared cited-gate
denominator is **3/3 causal**. No source detector, bank item, or shared pane
file was changed; all bypasses were scratch-only source mutations.

## Primary causal progress: unresolved objective claim

Primary status is now **`causal=6/24`, `intact=24/24`**. Row 12 used one
objective whose `claim_ids` named `c-missing`, while the claims registry
contained a valid unrelated `c-valid` row. The domain, policy, topic, and
bank controls were valid and topic coverage was explicitly skipped as the
fixture contract permits. The intact production `cdcp_learn::objectives`
implementation returned RED with `objective o1: unresolved claim_id
'c-missing'`.

Changing the objective to cite `c-valid` produced GREEN. The scratch bypass
changed only the `!known_claims.contains(&s)` branch; restoring `c-missing`
then returned GREEN as `BYPASSED PASS: unresolved-claim detector disabled`.
The fixture's RED assertion therefore fails under that bypass, proving the
objective-to-claim detector is causal.

## Primary causal progress: objective module floor

Primary status is now **`causal=7/24`, `intact=24/24`**. Row 15 used a
15-module domain registry, valid objectives/claims/policy/topics, and one
approved item in each of modules 1–14. The intact production
`cdcp_learn::objectives` evaluation returned RED with
`domain module 15: 0 approved < min 1`.

Adding module 15 produced the positive-control GREEN. The scratch bypass
changed only the `i128::from(have) < MIN_ITEMS_PER_MODULE` branch; restoring
the starved 14-module bank then returned GREEN as
`BYPASSED PASS: module-floor detector disabled`. This isolates the fixture to
the derived module-floor detector.

Updated current status: primary **7/24 causal, 24/24 intact, 17/24
CANNOT_DETERMINE**; cited-gate supplemental **3/3 causal**.
