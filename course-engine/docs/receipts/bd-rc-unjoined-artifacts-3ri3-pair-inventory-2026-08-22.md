# RC-B artifact-pair inventory

Date: 2026-08-22

Bead: `bd-rc-unjoined-artifacts-3ri3`

Scope: the committed artifacts that a learner, installer, or verification
surface can consume. The source-of-truth column names the owner; the join
column names the automatic surface that compares, regenerates, or resolves
the two sides.

This is an inventory of relationships, not a new validator. A `CHECKED` row
means an existing command or gate compares the named relationship. `PARTIAL`
means the existing check covers a narrower structural property than the
relationship a reader might infer. `DOCUMENTED ABSENCE` means the pair is
known, the missing comparison is named, and the reason/date are explicit.
Process state (`.beads`, `.flywheel`) is listed only where it was previously
mistaken for a product artifact; it is not silently promoted into one.

## Pair inventory

| ID | Artifacts that must agree | Source of truth / owner | Existing automatic join and forward surface | Verdict and boundary |
|---|---|---|---|---|
| P01 | `bank/items` ↔ `web/data/bank_items_seed42.json`, `keys_seed42.json`, and `mock40_seed42.json` | `cdcp_assemble` and `cdcp export-web`; the committed JSON files are shipped learner inputs | `cdcp_gate pack-freshness` at `scripts/check.sh:1422`, then seed-42 `export-web` byte comparison and `assert_generator_fresh` at `scripts/check.sh:1810-1827` | **PARTIAL** — freshness and seed-42 bytes are joined; `pack-freshness` alone is timestamp-only and cannot prove content correctness. |
| P02 | `bank/items` ↔ `content.lock` bank hash | `cdcp_bank::compute_bank_hash` and `knowledge/content.lock` | `cdcp verify-data-lock` plus its mutation selftest at `scripts/check.sh:1988-1998`; `verify-content-lock` checks the content-lock side | **CHECKED** — a changed bank hash or unlisted locked file is named; this does not make a semantically wrong item correct. |
| P03 | `bank/items` ↔ grade goldens and frozen report digests | `cdcp_grade` / `goldens/` | `cdcp goldens check` at `scripts/check.sh:1676-1679` and `cdcp_gate goldens-couplings` at `scripts/check.sh:1178` | **CHECKED** — the bytes and declared source surfaces are coupled; a deliberate re-freeze still requires human review. |
| P04 | `bank/items` and grade source ↔ `web/assets/wasm/cdcp_wasm.wasm` | `cdcp_wasm` release build | `scripts/selftest_wasm_freshness.sh` and the wasm rebuild/hash comparison at `scripts/check.sh:1698-1714` | **CHECKED** when the wasm toolchain is available; the declared skip path is visible, not a false green. |
| P05 | `bank/items` ↔ `web/data/units_index.json` counts | `cdcp_learn` unit compiler; `units_index.json` is generated learner navigation data | `cdcp_registry_check count-pins` at `scripts/check.sh:858-859`, followed by `build-units` and `assert_generator_fresh` at `scripts/check.sh:1851-1854` | **CHECKED** for file/approved/unit counts and freshness; it does not decide whether a unit teaches every item assigned to it. |
| P06 | `web/content/modules/*.md` ↔ generated learn pages and module indexes | module Markdown is the authored source; `cdcp build-learn` owns generated output | `build-learn`, `assert_generator_fresh`, and `smoke-learn` at `scripts/check.sh:1780-1800` | **CHECKED** structurally — generated pages and indexes are regenerated and smoked; prose quality remains outside this join. |
| P07 | `web/content/reference/*.md` ↔ `web/reference.html` | reference Markdown is the authored source; `cdcp build-reference` owns the page | `build-reference` plus `assert_generator_fresh` at `scripts/check.sh:1791-1793` | **CHECKED** for regeneration/freshness; it does not prove that a reference claim is sufficient for an item. |
| P08 | `bank/items` ↔ the module that teaches each tested proposition | bank item is owned by `bank/items`; teaching source is `web/content/modules` | W1b census in `crates/cdcp_assemble/examples/teaching_mismatch.rs` and receipts `bd-rc-unjoined-artifacts-3ri3.1-teaching-test-{2026-08-22,census-2026-08-22}.md`; no `check.sh` invocation | **DOCUMENTED ABSENCE (2026-08-22)** — 957 total rows / 931 approved shipped rows measured; 105 lexical teaching/test mismatches (11.3% of shipped rows). The census cannot decide semantic teaching truth, so it is a reproducible review floor rather than an automatic gate. |
| P09 | item citations ↔ public source pages and supporting excerpts | `bank/items` citation IDs plus `registries/quote_or_drop.toml` | `cdcp_gate quote-or-drop` / receipt `docs/receipts/quote-or-drop.json`; the periodic `--refresh` is an explicit authoring operation | **PARTIAL** — URL reachability and excerpt support are joined; authority, currentness, and whether the source teaches the item remain human judgements. |
| P10 | `knowledge/topics.toml` ↔ item `topic_ids` ↔ learner topic anchors | topics registry is authoritative for IDs; item references and generated anchors are consumers | `cdcp_gate verify-orphans`, `smoke-feedback-links`, and generated `web/data/topic_anchors.json` in the learn build path | **CHECKED** for referential integrity and page anchors; it does not prove topical pedagogy. |
| P11 | `registries/objectives.toml` ↔ item `objective_ids` ↔ module coverage | objectives registry is authoritative; item claims and module coverage consume it | `cdcp_gate verify-objectives` and `scripts/selftest_l7_objectives.sh` at `scripts/check.sh:1971-1975` | **CHECKED** for resolution and declared coverage; it does not prove the objective is actually taught. |
| P12 | `knowledge/domains.toml` / `standards_crosswalk.toml` ↔ item and module domain labels | domain and standards registries | registry-check plus `cdcp_gate verify-coverage` and its selftest at `scripts/check.sh:1836-1841` | **PARTIAL** — IDs and declared coverage are joined; the truth of a standards interpretation is not mechanically decidable here. |
| P13 | `knowledge/data/**` / source metadata ↔ rights policy and locks | `knowledge/corpus/rights-policy.toml` and `content.lock` | `cdcp corpus-rights`, `cdcp verify-data-lock`, and the corpus hermetic lane at `scripts/check.sh:1058-1086` | **CHECKED** for declared rights, file presence, and hashes; the check does not open capture bodies or assess downstream teaching. |
| P14 | `knowledge/primary_notes` references ↔ files actually shipped in the knowledge pack | knowledge files are the source; registry paths are references | `cdcp_gate verify-knowledge-paths` at `scripts/check.sh:1749-1750` | **CHECKED** — missing path references are named; path existence is not evidence that the note supports every claim. |
| P15 | `README.md` / public count claims ↔ generated `web/data/units_index.json` and measured WASM size | generated indexes and the full-chain receipt are the measured sources | `cdcp docs sync --check` in the final chain and `verify-step-count` at `scripts/check.sh:2129-2135` | **CHECKED** for the registered count and step-count claims; prose outside registered claims can still drift and is covered only where marked. |
| P16 | `registries/goldens-couplings.toml` ↔ source regions and golden files | coupling ledger is the declaration; source regions and frozen bytes are the joined sides | `cdcp_gate goldens-couplings` re-extracts pins at `scripts/check.sh:1170-1179` | **CHECKED** — moved regions, unbumped versions, and unaffirmed re-freezes are named. It cannot judge whether a justification is honest. |
| P17 | `registries/count_pins.toml` ↔ the source-derived counts it records | source trees (`bank/items`, `web/data/units_index.json`) are authoritative; registry rows are observations | `cdcp_registry_check count-pins` before downstream tests at `scripts/check.sh:858-859` | **CHECKED** as movement detection; a wrong initial observation remains wrong, by design and by the registry's own documentation. |
| P18 | `registries/doc-facts.toml` ↔ code, tests, generated ledgers, and prose claims | each fact row names its resolving artifact; the artifact is authoritative | `cdcp_gate doc-facts` at `scripts/check.sh:1190-1191` | **CHECKED** for registered yes/no claims and resolvable evidence; an unregistered sentence is outside this particular contract. |
| P19 | `knowledge/exam_form.toml` ↔ assembly, grading, and browser exam shape | exam form owns 40 questions / 3600 seconds / pass threshold; assembly and grade consume it | check.sh form pins plus `cdcp assemble` / `cdcp export-web` / `cdcp study` and the learner-pack shape check | **PARTIAL** — the structural form values are joined; this does not prove every browser interaction or generated explanation matches the form. |
| P20 | local release archive ↔ full source/tree/`Cargo.lock` identity and staged bytes | `cdcp release build` / `cdcp release verify` and `cdcp_data::artifact_identity` | The producer/verifier and receipt `bd-installability-sm4g.29-release-producer.md`; `cdcp release` is a deliberate local command, not a GitHub Actions lane | **CHECKED** for the local producer contract — one root-level `cdcp`, archive SHA-256, full object IDs, dirty-tree refusal, and installer selftest. It is not automatically run by `check.sh`; release production remains an operator invocation. |
| P21 | installed archive ↔ installed bundle, doctor root, and learner HTTP surface | release archive is the input; installer/doctor/serve consume it | `scripts/selftest_install.sh`, `cdcp doctor`, and the current-release receipts | **PARTIAL** — install, missing-bundle refusal, and learner HTTP reachability are exercised; cross-platform delivery and producer path-remap remain release acceptance concerns. |
| P22 | `scripts/check.sh` ↔ its required test registry and advertised step count | `check.sh` is the chain source; `registries/required_tests.toml` and README receipt are declarations | `cdcp_gate required-tests` at `scripts/check.sh:867-868` and `verify-step-count` at `scripts/check.sh:2129-2135` | **CHECKED** as process wiring, not a product-artifact join; it prevents silent step/test omission but does not make every assertion substantive. |
| P23 | `.beads/issues.jsonl` / `.flywheel/tick-ledger.jsonl` ↔ product artifacts | no learner or delivery path owns these; they are coordination state | `tick-reconcile` was retired on 2026-08-21; its retired block in `scripts/check.sh` records the decision | **DOCUMENTED ABSENCE (2026-08-21)** — these files exist for agent bookkeeping, not product output. No product gate is required to compare them; the accepted consequence is that a close can now lack a tick receipt without this chain noticing. |
| P24 | UX findings log ↔ shipped learner behavior | findings are research notes, not a shipped artifact or source of truth | no automatic comparison; de-duplication and human observation are the consumer | **DOCUMENTED ABSENCE (2026-08-22)** — the log informs product decisions but is not itself a learner artifact. A future fix should add a product test at the affected surface, not a validator of the log. |

## Unjoined gaps that remain actionable

1. **W1b is measured, not wired.** The 105/931 lexical mismatch rows are now
   reproducible and receipt-backed, but no full-chain step compares item claims
   with the module prose. The reason for the absence is substantive: semantic
   teaching truth needs human adjudication, and the lexical census explicitly
   says it cannot decide that truth. The next product decision is whether to
   build a review instrument or accept a dated review queue; it is not to turn
   the lexical proxy into an authority.
2. **Release production is local and operator-invoked.** The artifact identity
   contract is implemented and independently verified, but `scripts/check.sh`
   does not build a release archive on every product check. That is deliberate:
   a release build is an expensive delivery transaction and the local producer
   is the forward surface. The release receipt is the evidence boundary.
3. **Semantic joins remain human boundaries.** A citation excerpt can support a
   URL, an objective ID can resolve, and a module can contain matching words;
   none of those mechanical relationships proves that the source teaches the
   tested proposition. The inventory keeps those rows `PARTIAL` or
   `DOCUMENTED ABSENCE` rather than upgrading them to a false green.

## Inventory result

This pass records **16 CHECKED**, **5 PARTIAL**, and **3 DOCUMENTED ABSENCE**
rows. The three absences are intentional, dated decisions rather than omitted
work: the bank-to-module semantic relationship (W1b), coordination state
versus product artifacts, and UX research notes versus shipped behavior. The
release rows remain partial/checked at their respective byte and install
boundaries, with no claim that a local producer is an upload or publication
system.

The inventory therefore closes the pair-inventory portion of RC-B without
claiming that every product property has a mechanical oracle. The next work
should either wire a product-valued comparison for W1b or record its human
review disposition; it should not create a meta-validator that merely checks
whether this table has enough rows.
