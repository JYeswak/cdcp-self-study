# Value-bearing gate audit — bd-std-value-bearing-gates-1ij7

Date: 2026-08-21  
Audit tree: `055f922e999b185ce0a77d68385ce5dc23eea080` (worktree)  
Owner: pane 2

## Definition and method

A gate is a distinct verdict-producing check in the outer `scripts/check.sh`
transaction.  A known-bad leg, a fixture, or a helper folded into that gate is
not a second gate: it is evidence that the parent gate can RED.  The complete
runtime inventory is the 98-slot table in
`docs/receipts/bd-installability-sm4g.22-local-ci.md`; the table below gives
the Rule-Zero admission trace for every distinct gate action represented by
those slots.

For each row I asked two questions:

1. Backward: what source-of-truth or shipped artifact does the check consume,
   and what existing product/build path produces or maintains that input?
2. Forward: which automatic hook, CI lane, or delivery command invokes it?

The forward surface for every row below is the default local CI transaction,
`scripts/check.sh`; additional surfaces are named where they exist.  A row
marked `JUSTIFIED-OPS` is deliberately a process/integrity gate rather than a
learner-quality claim.  It has no learner artifact to pretend to consume, so
the reason for retaining it is written here and dated.  No gate is retained
because another agent might remember to run it.

## Admission table

| gate action in `check.sh` | backward input and producer | forward automatic surface | disposition |
|---|---|---|---|
| Constitution-doc presence | `docs/ORACLE-GAUNTLET.md`, `docs/STANDARDS-KB.md`, `docs/TESTING.md`, `README.md`; maintained public operating docs | `scripts/check.sh` before any build | product-doc trace; keep |
| Knowledge-pack presence | `knowledge/exam_form.toml`, `sources.toml`, `domains.toml`, `topics.toml`, `claims.toml`; authoring pack | `scripts/check.sh`; feeds CLI and learner/reference builds | product trace; keep |
| Registry-file presence | `registries/claims.toml`, `claims_lint.toml`, `objectives.toml`; registry inputs to product checks | `scripts/check.sh` registry-check lane | product/contract trace; keep |
| `cdcp_registry_check count-pins` | `bank/items/*.toml`, `web/data/units_index.json`, `registries/count_pins.toml`; bank/export generators produce the measured fields | `scripts/check.sh` immediately after the build | artifact-drift trace; keep |
| `cdcp_gate required-tests` | compiled workspace test binaries and `registries/required_tests.toml`; Cargo produces the test identities | `scripts/check.sh` | `JUSTIFIED-OPS (2026-08-21)`: prevents a filtered or deleted load-bearing test from certifying the ship; not a claim that the test proves learner quality |
| `cdcp_gate tick-reconcile` | `.beads/issues.jsonl` and `.flywheel/tick-ledger.jsonl`; the tick emitter and bead lifecycle produce them | `scripts/check.sh` before the chain can proceed | `JUSTIFIED-OPS (2026-08-21)`: prevents unreceipted closes from becoming invisible; bookkeeping is intentionally not presented as learner output |
| lock/snapshot selftest | `scripts/check.sh`, lock directory, and snapshot re-exec copy; the CI driver creates the private snapshot | `scripts/check.sh` lock boundary | `JUSTIFIED-OPS (2026-08-21)`: protects transaction isolation and fail-closed execution; parent check has the automatic consumer |
| `cdcp_registry_check` constitution | all declared registries and their referenced evidence; registry-check reads the product contract | `scripts/check.sh` L1 | product-contract trace; keep |
| `cdcp check-licence` | `registries/rights-policy.toml` and corpus metadata; `cdcp_data` consumes the policy to build the publishable corpus | `scripts/check.sh` D4 | corpus/delivery trace; keep |
| corpus-rights test and CLI check | corpus metadata/tree and rights policy; `cdcp_data` produces the published corpus view | `scripts/check.sh` and Cargo test lane | corpus artifact trace; keep |
| `cdcp load-snapshots` | `knowledge/snapshots.toml`, snapshot bodies/sidecars, and licence pins; `cdcp_data` loads them | `scripts/check.sh` E1 | grounding-input trace; keep |
| `cdcp check-osha` | OSHA snapshot/facts and `knowledge/sources.toml`; the facts checker reads the source pack used by grounding | `scripts/check.sh` E3 | source/learner-content trace; keep |
| `cdcp verify-data-lock` plus its flip leg | `content.lock` and the vendored snapshot bytes; snapshot/data generators produce the lock inputs | `scripts/check.sh` data lane | shipped-data-integrity trace; keep |
| `cdcp oracle-check` | published reference values and the CLI's computed quantities; knowledge/oracle data produces both sides | `scripts/check.sh` F3 | reference-output trace; keep |
| `cdcp_gate substrate-guard` | complete engine tree plus `registries/substrate_allowlist.toml`; source checkout and migration work produce the scanned tree | `scripts/check.sh` S0 and the pre-commit/install path | installability/substrate trace; keep |
| `substrate-guard --prove-wired` | the real `scripts/check.sh` dispatch path plus its planted unlisted file; the probe materializes this proof input | `scripts/check.sh` behavioural wiring leg | `JUSTIFIED-OPS (2026-08-21)`: meta-gate proving the gate is actually reached; its output is wiring evidence, not a learner-quality certificate |
| `cdcp_gate install-hooks` | hook installer source and `.git/hooks/pre-commit`; installer creates the commit-time hook | `scripts/check.sh` and the installed pre-commit hook | installability trace; keep |
| `cdcp_gate install-hooks --check` | current hook bytes versus installer-owned bytes | `scripts/check.sh` fresh-clone path | installability trace; keep |
| `cdcp_gate capability-maturity` | `registries/capability-maturity.toml`, CHARTER markers, named test evidence; ledgers are maintained with product claims | `scripts/check.sh` B1 | claim-to-evidence trace; keep |
| `cdcp_gate goldens-couplings` | `registries/goldens-couplings.toml`, goldens, web packs, and `content.lock`; export/golden generation produces the frozen artifacts | `scripts/check.sh` B2 and re-freeze workflow | shipped-artifact coupling trace; keep |
| `cdcp_gate doc-facts` | registered prose facts and the artifacts that answer them (bank, registries, generated packs); product/doc generators maintain both | `scripts/check.sh` B3 | public-truth trace; keep |
| exam-form pins | `knowledge/exam_form.toml`; public exam-form authoring produces the learner contract | `scripts/check.sh` W0 | learner-contract trace; keep |
| honesty scan | public `docs/` and `knowledge/` copy; those documents are what a learner/stranger reads | `scripts/check.sh` W0 | public-copy trace; keep |
| domain crosswalk coverage | `knowledge/domains.toml` and `knowledge/standards_crosswalk.toml`; knowledge authoring produces the crosswalk | `scripts/check.sh` W0/L2 | knowledge/reference trace; keep |
| topics count and source fetch dates | `knowledge/topics.toml` and `knowledge/sources.toml`; knowledge pack authoring produces them | `scripts/check.sh` W0 | knowledge-pack trace; keep |
| `cdcp_gate verify-bank` | `bank/MANIFEST.toml` and `bank/items/*.toml`; bank assembly produces the approved/retired pool | `scripts/check.sh` L2 and all export/build commands | learner-pool trace; keep |
| `cdcp_gate answer-key-skew` | approved item `correct` fields; bank/export/mock generators consume the answer keys | `scripts/check.sh` L2 | mock/question-pool construction trace; keep |
| `cdcp_gate key-contradiction` | approved stems, choices, keys, and `topic_ids`; bank authoring produces the question pool | `scripts/check.sh` L2 | bank-internal correctness-floor trace; keep |
| `cdcp_gate construction-faults` | approved option sets plus the damaged-corpus fixture; bank authoring and learner export produce the option sets | `scripts/check.sh` L2 and known-bad selftests | learner-question construction trace; keep |
| `cdcp_gate validate-grounding` | approved item citations/stems and loaded source snapshots; grounding/build paths consume them | `scripts/check.sh` L2 | grounding-input trace; keep |
| `cdcp_gate grounding-wave` | approved stems, choices, and adjudication registry; bank repair/export consumes the current questions | `scripts/check.sh` L2 | regression trace for learner content; keep |
| `cdcp_gate verify-orphans` | topic registry and item `topic_ids`; bank assembly and module/reference builds consume the relation | `scripts/check.sh` L2 | referential-integrity trace; keep |
| `cdcp_gate near-duplicate-items` | approved item bodies; mock and learner-pool assembly consume the approved set | `scripts/check.sh` L2 and its planted-clone leg | learner-pool quality trace; keep |
| `cdcp verify-paraphrase-pairs` | `registries/paraphrase_pairs.toml` and bank items; the authoring ledger records adjudicated proposition pairs | `scripts/check.sh` L2 | authoring/assembly trace; keep |
| `cargo fmt --check` | Rust workspace source; Cargo builds and ships that source | `scripts/check.sh` L3 | `JUSTIFIED-OPS (2026-08-21)`: build hygiene required for the shipped Rust path, not a learner-content assertion |
| gate-module `rustfmt` scan | `crates/cdcp_gate/src/gates/*.rs`; gate source is compiled into the checker | `scripts/check.sh` L3 | `JUSTIFIED-OPS (2026-08-21)`: checks a source surface cargo fmt cannot see; the compact-dispatcher exception is documented in the driver |
| `cargo clippy --all-targets -D warnings` | workspace source and test targets; Cargo produces the binaries/tests | `scripts/check.sh` L3 and pre-commit/build workflows | `JUSTIFIED-OPS (2026-08-21)`: prevents silent dead test assertions and unsafe build drift; not a claim about question truth |
| `cargo test --workspace` | workspace source, fixtures, and test targets; Cargo produces the executable/test evidence | `scripts/check.sh` L3 | `JUSTIFIED-OPS (2026-08-21)`: verifies implementation contracts before delivery; tests are evidence, not automatic proof of every assertion's substance |
| golden-artifact presence and `cdcp goldens check` | committed goldens, fixture answers, `bank_hash.txt`, provenance; grading/export generation produces them | `scripts/check.sh` L3 and learner grading path | GradeExact/grading-artifact trace; keep |
| aggregate known-bad selftests | temporary mutations of the live gates and their fixtures; each parent gate owns its injection | `scripts/check.sh` L4 | `JUSTIFIED-OPS (2026-08-21)`: proof of RED reachability for parent gates; no separate product claim |
| installer known-bad selftest | installer tarball/manifest/checksum contract; release/install packaging produces the artifact | `scripts/check.sh` L4 and installer selftest | installable-release trace; keep |
| WASM freshness and dual-path | `web/assets/wasm/cdcp_wasm.wasm` and `crates/cdcp_wasm` dependencies; release build produces the blob | `scripts/check.sh` L4 and learner browser grader | shipped-grader trace; keep |
| knowledge primary-note path check | `knowledge/*.toml` primary-note paths and `../modules/`; reference/learn builds consume those paths | `scripts/check.sh` before L5 | learner/reference trace; keep |
| L5 web-file and shipped-WASM presence | `web/index.html`, learn/drill/mock/reference pages, and WASM asset; web build/export produces them | `scripts/check.sh` L5 and learner delivery directory | learner-surface trace; keep |
| `cdcp check-learner-pack` | `web/data/mock40_seed42.json`, keys, bank export, and WASM; export-web produces the pack | `scripts/check.sh` L5 | learner-pack trace; keep |
| L5 selftest and e2e digest | learner pack/WASM plus frozen all-correct/all-wrong digests; demo/export/browser path produces the digests | `scripts/check.sh` L5 | learner-grading trace; keep |
| generator freshness plus `build-learn` | generated Learn pages, module index, topic anchors, and source bank/knowledge; `cdcp build-learn` produces them | `scripts/check.sh` L5 | learner-content build trace; keep |
| `build-reference` | generated reference HTML/content and knowledge source pack; `cdcp build-reference` produces it | `scripts/check.sh` L5 | learner/reference trace; keep |
| `smoke-learn` | generated learn surface and web data; build-learn produces the tested surface | `scripts/check.sh` L5 | learner-surface trace; keep |
| SRS/mastery/weak-link/hub smokes | generated JS/UI and learner state fixtures; web build and learner runtime produce the tested surfaces | `scripts/check.sh` L6 | learner-interaction trace; keep |
| multi-seed `export-web` and session shapes | approved bank, seed export, and `web/drill.html`; export-web/build paths produce the learner pack/session UI | `scripts/check.sh` L6 | learner-export trace; keep |
| `cdcp_gate verify-coverage` | `knowledge/domains.toml`, bank items, and domain minimums; bank/module assembly produces the coverage population | `scripts/check.sh` L6 | learner-domain coverage trace; keep |
| L6 coverage selftest | temporary empty/missing-domain copies plus live bank; parent coverage gate owns the proof | `scripts/check.sh` L6 | `JUSTIFIED-OPS (2026-08-21)`: RED reachability proof for the parent gate |
| L7 web surfaces and Chrome smoke | generated reference/learn pages, browser assets, and current pack; web build produces them | `scripts/check.sh` L7 | learner-surface trace; keep |
| units, glossary, and learn-slug generators | `knowledge/domains.toml`, bank, and generated `units_index.json`, glossary, slug JS; CLI generators produce them | `scripts/check.sh` L7 and web delivery | generated-product trace; keep |
| approved-only quiz smoke | `units_index.json`, approved bank, and quiz JS; export/build path produces the draw | `scripts/check.sh` L7 | learner-pool trace; keep |
| learn-v2, diagrams, a11y, and feedback-link smokes | generated web pages/assets and their source registries; web build produces the tested UI | `scripts/check.sh` L7 | learner-interaction trace; keep |
| CLI product-verb visibility | `cdcp_cli` binary/help and authoring/learner command surface; Cargo build produces the CLI | `scripts/check.sh` L7 | `JUSTIFIED-OPS (2026-08-21)`: ensures the shipped CLI surface is wired; not a content-quality claim |
| `cdcp test`, `cdcp demo`, and `cdcp study` | installed-tree CLI, learner pack, WASM/bundle, and HTTP study surface; Cargo/web build produces them | `scripts/check.sh` L7 and learner entry points | learner-entry trace; keep |
| learner-verb known-bad selftest | temporary missing-bundle/ignored-exit/stop-reap fixtures; CLI learner verbs own the proof | `scripts/check.sh` L7 | `JUSTIFIED-OPS (2026-08-21)`: RED reachability and process-lifecycle proof for learner verbs |
| `cdcp_gate verify-objectives` | `registries/objectives.toml` and bank module items; objective authoring and bank assembly produce them | `scripts/check.sh` L7 | learner-objective trace; keep |
| objectives selftest | temporary objective/item omissions; objective gate owns the proof | `scripts/check.sh` L7 | `JUSTIFIED-OPS (2026-08-21)`: parent-gate RED reachability proof |
| SLO smoke | generated export and learner web surfaces; `cdcp export-web`/web build produces them | `scripts/check.sh` L7 | learner-performance trace; keep |
| `cdcp_gate verify-content-lock` plus flip leg | `content.lock`, bank hash, and current source/artifacts; content-lock generator produces the pins | `scripts/check.sh` L7 and artifact publication | shipped-content-integrity trace; keep |
| reconstructed-stage selftests | temporary reconstructed stage fixtures and the product stages they exercise | `scripts/check.sh` V11 | `JUSTIFIED-OPS (2026-08-21)`: proof of historical stage RED reachability; parent chain is the consumer |
| voice-slop and publishability | public docs/copy and audit claims; documentation/publication workflow produces them | `scripts/check.sh` V11 | public-delivery trace; keep |
| `cdcp_gate verify-doc-consistency` plus its selftest | README/CHARTER/PHASE-NEXT/status tables and their registry facts; docs workflow produces them | `scripts/check.sh` V11 | public-truth trace; keep |
| Anki all-retired selftest | approved/retired bank and export output; `export-anki` produces the learner deck | `scripts/check.sh` V11 | `JUSTIFIED-OPS (2026-08-21)`: anti-vacuous proof for the export gate; parent export is the product consumer |
| `cdcp export-anki` and `--check` | approved bank, pinned clock, generated TSV/CSV/APKG and identity hashes; Anki export produces the deck | `scripts/check.sh` V11 and learner download | learner-deck trace; keep |
| diagram honesty and serve/runbook checks | reference/learn web copy, CLI serve verb, and bank item directory; web/CLI build produces the delivery surface | `scripts/check.sh` V11 | learner/public-entry trace; keep |
| injection-count selftest and `verify-injection-count` | `INJECTIONS=` receipts, registered suite names, and README count; selftest suites and check.sh produce the receipt log | `scripts/check.sh` final drift lane | `JUSTIFIED-OPS (2026-08-21)`: prevents known-bad coverage from silently shrinking; not a learner-quality claim |
| docs sync | `units_index.json`, README/CHARTER advertisement sites, and measured WASM size; generators and docs sync produce/verify them | `scripts/check.sh` final drift lane and public docs | public-artifact trace; keep |
| `verify-step-count` | depth-0 `CHECK_STEPS` receipt and README count; check.sh emits the receipt and verify-step-count compares it | `scripts/check.sh` final boundary | `JUSTIFIED-OPS (2026-08-21)`: prevents fail-fast/skip accounting from being advertised as a complete run; it is observability, not learner quality |

## Adversarial result

No row is a gate whose only consumer is another agent. Every row is invoked by
the automatic `scripts/check.sh` transaction; several also run from a hook,
Cargo, an installer, a generator, or the learner delivery path. The rows marked
`JUSTIFIED-OPS` do not have a learner artifact because they protect the
transaction, build, test, or evidence machinery itself. They are retained with
the dated reasons above, rather than being mislabeled as evidence that a
question is true or discriminating.

No gate was retired in this audit. The non-vacuity condition is met by the
explicit dated justifications for those process-only gates; a table in which
every row was silently declared a product gate would have failed this audit.

This audit does not prove that any gate's assertion is substantive. In
particular, `required-tests` proves named tests ran, not that they test enough;
the selftest rows prove RED reachability, not coverage of every future defect;
and a green learner gate still does not certify the bank's truth or
discrimination.

## Focused evidence

The complete local-CI receipt records the historical 98-slot inventory,
consumed-scope hashes, and the rule that nested selftests are not outer slots:
`docs/receipts/bd-installability-sm4g.22-local-ci.md` and
`docs/receipts/step-count-reconciliation-2026-08-20.md`.  This audit did not
modify `scripts/check.sh`, build a second validator, touch bank content, or
claim that a full chain rerun was necessary to establish the traces.

At least one automatic invocation and one consumed input are named for every
row above.  The honest boundary is that process-only rows consume process
state, not learner content; their retention is a dated operational
justification, not a value-bearing certificate.
