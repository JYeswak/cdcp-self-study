# Q26 adversarial value-bearing re-audit — 2026-08-21

This is a re-audit of `bd-std-value-bearing-gates-1ij7`, prompted by the
operator's challenge to the earlier all-pass table. It is deliberately about
the registered `cdcp_gate` surface, not about whether a green assertion is
substantively correct.

## Counting rule

`target/debug/cdcp_gate list` currently exposes 25 names. `emit-tick` is an
emitter that appends process bookkeeping; it is not a verdict-producing gate,
so it is outside this 24-row gate table. The other 24 names are the audit
population.

The current `scripts/check.sh` source dispatches 21 of those 24 names. The
three registered names with no `run_cdcp_gate` invocation in that driver are
`pack-freshness`, `quote-or-drop`, and (not counted here) `emit-tick`—the last
is excluded as an emitter. This distinction is part of the result: a compiled
subcommand is not automatically a delivery surface.

For each row, **backward** names an artifact or state emitted by an existing
product path; **forward** names an automatic invocation surface. A process
state file is not silently promoted to a product artifact. `JUSTIFIED-WITH-DATE`
is reserved for an operational/integrity gate whose lack of learner artifact
is explicit and whose retirement/re-homing condition is recorded.

## 24-gate table

| gate | backward: existing product artifact and producer | forward: automatic invocation surface | verdict |
|---|---|---|---|
| `answer-key-skew` | approved item `correct` fields; bank/export assembly emits the answer-key pool | `scripts/check.sh` L2 bank lane | KEEP |
| `capability-maturity` | `registries/capability-maturity.toml` and its dated evidence references; capability/claims authoring maintains them | `scripts/check.sh` B1 | JUSTIFIED-WITH-DATE (2026-08-21): operational claim-evidence admission; re-home when capability claims are emitted into a learner/public manifest |
| `construction-faults` | approved option sets; bank export emits the learner question pool | `scripts/check.sh` L2 and known-bad selftest lane | KEEP |
| `doc-facts` | public prose facts plus the bank/registry/generated artifacts that answer them; docs and export paths maintain both | `scripts/check.sh` B3 | KEEP |
| `goldens-couplings` | learner packs, WASM/goldens, and `content.lock`; export and golden-generation paths emit the frozen artifacts | `scripts/check.sh` B2 and the pack re-freeze transaction | KEEP |
| `grounding-wave` | approved stems/options and grounding/adjudication data; bank repair/export emits the learner questions | `scripts/check.sh` L2 | KEEP |
| `install-hooks` | committed hook shim and installer-owned hook bytes; `install.sh`/hook installer creates the install surface | `scripts/check.sh` plus the installed pre-commit hook | KEEP |
| `key-contradiction` | approved stems, choices, keys, and topic links; bank/export assembly emits the question pool | `scripts/check.sh` L2 | KEEP |
| `near-duplicate-items` | approved bank item bodies; learner/mock export emits the approved pool | `scripts/check.sh` L2 and its planted-clone selftest | KEEP |
| `pack-freshness` | `bank/items` and `web/data` pack bytes; export-web emits the learner packs | **No current `scripts/check.sh` dispatch**; no automatic hook/CI/installer invocation found | RETIRE |
| `quote-or-drop` | citation receipts and item citation fields; quote-or-drop authoring records exact supporting excerpts | **No current `scripts/check.sh` dispatch**; no automatic delivery invocation found | RETIRE |
| `required-tests` | compiled test identities and required-test registry; Cargo emits the executable test surface | `scripts/check.sh` immediately after the build | JUSTIFIED-WITH-DATE (2026-08-21): prevents filtered/deleted load-bearing tests from certifying a run; re-home when the shipped verification manifest carries test identity |
| `substrate-guard` | engine source tree and the substrate allowlist; the source/build/install path emits the tree that becomes the shipped CLI | `scripts/check.sh` S0/L4 and the installed pre-commit hook | KEEP |
| `tick-reconcile` | `.beads/issues.jsonl` and `.flywheel/tick-ledger.jsonl`; bead/tick bookkeeping emits them, not a learner or delivery path | `scripts/check.sh` early blocking edge | RETIRE: process accounting wearing a product-gate name; neither input is a product artifact and the gate was created without a deletion condition |
| `validate-grounding` | approved item citations/stems and loaded knowledge snapshots; grounding/export paths consume them | `scripts/check.sh` L2 | KEEP |
| `verify-bank` | `bank/MANIFEST.toml` and `bank/items/*.toml`; bank assembly/export emits the approved/retired pool | `scripts/check.sh` L2 and downstream export/build commands | KEEP |
| `verify-content-lock` | `content.lock`, bank hash, and source/artifact digests; lock/content generation emits the pins | `scripts/check.sh` L7 and artifact publication/re-freeze transaction | KEEP |
| `verify-coverage` | domain registry, minimums, and bank items; bank/module assembly emits the coverage population | `scripts/check.sh` L6 | KEEP |
| `verify-doc-consistency` | README/CHARTER/roadmap claims and the generated/public artifacts they describe; docs/build paths emit the surfaces | `scripts/check.sh` V11 | KEEP |
| `verify-injection-count` | known-bad suite receipts and their `INJECTIONS=` output; selftest suites emit the receipt log | `scripts/check.sh` final drift lane | JUSTIFIED-WITH-DATE (2026-08-21): operational RED-reachability accounting; re-home when injection coverage is represented in a product verification manifest |
| `verify-knowledge-paths` | `knowledge/*` primary-note paths and the parent corpus files; knowledge/reference builds consume them | `scripts/check.sh` before L5 | KEEP |
| `verify-objectives` | objectives registry and bank module items; objective authoring and learner export emit the objective-linked pool | `scripts/check.sh` L7 | KEEP |
| `verify-orphans` | topic registry and item `topic_ids`; bank/reference assembly emits the referential graph | `scripts/check.sh` L2 and orphan selftest | KEEP |
| `verify-step-count` | runtime `CHECK_STEPS` receipt and README claim; check.sh emits the receipt and docs publish the claim | `scripts/check.sh` final boundary | JUSTIFIED-WITH-DATE (2026-08-21): protects truthful fail-fast/skip accounting; re-home when the authoritative run receipt is itself the public delivery manifest |

## Focus findings

### `tick-reconcile`

The honest backward answer is **no**: the ledger and beads exist so agents can
account to one another. They are process state, not learner artifacts. The
forward edge is real, but that only proves the check runs; it does not make the
check value-bearing under Rule Zero. The existing implementation also says it
does not check whether a receipt is true, and its creation had no deletion
condition. This is a **RETIRE** finding, not a proposal to weaken it silently.
Removing or replacing the blocking edge is a separate gate-removal decision;
this receipt does not alter `scripts/check.sh` or the tick ledger.

### Derived substrate floor

`require_tree_derived_floor` inside `substrate_guard_e2e` is a sub-assertion,
not a separate registered `cdcp_gate` name. Its backward input is the
`InvocationWalk`/allowlist/on-disk tree, which is scan machinery rather than a
learner artifact. Its forward surface is Cargo's substrate test lane through
`scripts/check.sh`. It is retained as
**JUSTIFIED-WITH-DATE (2026-08-21)** for anti-vacuous scan integrity, with a
real deletion/re-homing condition: when the installer/release product emits
and verifies a substrate manifest from the same tree, move this assertion into
that product contract or delete the duplicate test; until then it is the only
guard against a scan whose target set silently becomes all ghosts. The
anti-vacuous plant/removal tests remain the separate safety legs.

### Actual retirement count

This audit has **3 RETIRE verdicts** (`tick-reconcile`, `pack-freshness`, and
`quote-or-drop`). The repository also has **1 actual prior gate retirement**:
`gate_shrink` was removed in `c0cee641` after its extraction reached 14907
lines and both self-deletion conditions were met. No current gate file was
deleted in this re-audit: `quote-or-drop` is pane-3-owned, and removing a
blocking process gate is not an unreviewed threshold change. The missing
forward surfaces are findings, not silently converted green results.

This audit does not prove that KEEP gates are true, discriminating, or
complete. It proves only that each KEEP row has a product-facing backward
trace and an automatic forward surface; each process row names its operational
reason and date; and gates with neither trace are identified for retirement.
