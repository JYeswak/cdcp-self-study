# Local CI consumed-input receipt

- sha: 85dc19a79f63dd2e18a82810f7d8a10a17268c01
- tree: detached worktree at that SHA (check.sh tree=worktree)
- mode: executed
- exit: 2
- wall_seconds: 427
- source_slots: 89
- executed_slots: 98
- skipped_slots: 0
- diagnostic_note: --diagnostic is inventory-only; its non-zero verdict is preserved and it never replaces default fail-fast CI.
- quality_boundary: completion proves termination and attribution, not that every assertion is substantive.

## Scope hashes

```text
live	bank	7dd60e3b4816aa90925d18d8e7f9a8a720093c6ff786c2b4ba2e44f008c4a492
live	registry	9cc15d6d8ec64d9734cf56f1be465ac5794e4324bfca1572650c87e4b7e03cf3
live	rust	209637bd80566965ef1dc980eaf0852d347de95c2784dd370dcd6288f55fc382
live	artifact	2166346cf88e54fbfb948085595922872f754a8b92171c849c9ba0e46437337f
live	docs	6b8aac036f2af49e0ab240608f55a98f1d40671d04cd42987b523310ebe0fbc8
live	all	c4f87abf2490cbe530589ce6af080a65d8d38900b84546acbf182cd2cbc05e4f
pinned	bank	7dd60e3b4816aa90925d18d8e7f9a8a720093c6ff786c2b4ba2e44f008c4a492
pinned	registry	f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885
pinned	rust	ec124c50734ab4e06d41243c8d65e197dfe53e9074235be517be0d84e2348bfc
pinned	artifact	ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493
pinned	docs	9c61124fd0b56bc92a25a89df98527aac69c45560840253a1ec43c4ea4779ee1
pinned	all	1d008f03bfae47fe84794620b6bcef88056548cce49de6285875de754023d29a
```

## Steps

| # | result | name | consumed scope | pinned hash | reason |
|---:|---|---|---|---|---|
<!-- source_slots=89 counts literal `ok` call sites in the pinned driver; executed_slots=98 counts diagnostic runtime emissions, including conditional and selftest rows. -->
<!-- worktree_layout: Git worktree root is the outer repository; the engine checkout is its course-engine/ child. -->
| 1 | 0 | constitution docs present | docs,registry | docs=9c61124fd0b56bc92a25a89df98527aac69c45560840253a1ec43c4ea4779ee1,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 2 | 0 | knowledge pack files present | docs,registry | docs=9c61124fd0b56bc92a25a89df98527aac69c45560840253a1ec43c4ea4779ee1,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 3 | 0 | L1 registry files present | registry,artifact | registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885,artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493 | executed |
| 4 | 0 | cargo build -p cdcp_gate -p cdcp_cli -p cdcp_registry_check --locked + count pins (debug binaries in /Users/josh/cdcp-self-study/course-engine/target/cdcp-scratch/local-ci-85dc19a79f63-59326/course-engine/target/debug) | registry,artifact | registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885,artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493 | executed |
| 5 | 0 | required test identities ran unfiltered in the worktree | rust,registry | rust=ec124c50734ab4e06d41243c8d65e197dfe53e9074235be517be0d84e2348bfc,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 6 | 0 | every bead closed since the baseline is named by a tick receipt | registry,artifact | registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885,artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493 | executed |
| 7 | 0 | concurrency lock proven (second run refused naming pid 96020 · dead holder reclaimed · unwritable lock path ERRORs) · snapshot re-exec proven (shear isolated · empty copy ERRORs · CHARTER pair 2/2 · env guard) | registry,artifact | registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885,artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493 | executed |
| 8 | 0 | L1 registry-check | registry,artifact | registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885,artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493 | executed |
| 9 | 0 | licence three-field split (published unlicensed / missing rights / third-party public-domain / PROHIBITED index) | docs,registry | docs=9c61124fd0b56bc92a25a89df98527aac69c45560840253a1ec43c4ea4779ee1,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 10 | 0 | corpus-rights tests (R7 bare permitted RED · R8 unclaimed file RED · public-domain PASS) | registry,artifact | registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885,artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493 | executed |
| 11 | 0 | corpus-rights (metadata+tree; never opens capture bodies) | all | all=1d008f03bfae47fe84794620b6bcef88056548cce49de6285875de754023d29a | executed |
| 12 | 0 | licence-gated snapshot loader (may_load + sha256 pin + anti-vacuous) | all | all=1d008f03bfae47fe84794620b6bcef88056548cce49de6285875de754023d29a | executed |
| 13 | 0 | OSHA facts (147 exclusion · 333 isolation · no 147-as-electrical-LOTO) | all | all=1d008f03bfae47fe84794620b6bcef88056548cce49de6285875de754023d29a | executed |
| 14 | 0 | content.lock [data] pins every snapshots.toml file | artifact,registry | artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 15 | 0 | data lock selftest (flipped vendored body trips RED) | rust,registry | rust=ec124c50734ab4e06d41243c8d65e197dfe53e9074235be517be0d84e2348bfc,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 16 | 0 | external oracle (computed vs published refs; no network) | all | all=1d008f03bfae47fe84794620b6bcef88056548cce49de6285875de754023d29a | executed |
| 17 | 0 | S0 substrate floor (no unreasoned py/sh-family file, shebang script, symlink or submodule anywhere in the engine tree · stale plant STALE PLANT) | all | all=1d008f03bfae47fe84794620b6bcef88056548cce49de6285875de754023d29a | executed |
| 18 | 2 | S0 wiring proven behaviourally (a planted unlisted .py stops check.sh) | all | all=1d008f03bfae47fe84794620b6bcef88056548cce49de6285875de754023d29a | substrate-guard wiring proof emitted no nested-ok-receipts measurement |
| 19 | 0 | pre-commit shim installed (idempotent) | all | all=1d008f03bfae47fe84794620b6bcef88056548cce49de6285875de754023d29a | executed |
| 20 | 0 | pre-commit shim installed and current | all | all=1d008f03bfae47fe84794620b6bcef88056548cce49de6285875de754023d29a | executed |
| 21 | 0 | capability claims attributed, dated, unexpired, and pointed at evidence that resolves | registry,artifact | registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885,artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493 | executed |
| 22 | 0 | every golden names the surfaces it was frozen against, and both sides agree | registry,artifact | registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885,artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493 | executed |
| 23 | 2 | every registered prose claim about code agrees with the artifact that answers it | registry,artifact | registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885,artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493 | prose claims about code disagree with the tree |
| 24 | 0 | exam_form public CDCP format pins | docs,registry | docs=9c61124fd0b56bc92a25a89df98527aac69c45560840253a1ec43c4ea4779ee1,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 25 | 0 | honesty string smoke | docs,registry | docs=9c61124fd0b56bc92a25a89df98527aac69c45560840253a1ec43c4ea4779ee1,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 26 | 0 | standards crosswalk covers every domain the registry declares (n=15) | bank,registry | bank=7dd60e3b4816aa90925d18d8e7f9a8a720093c6ff786c2b4ba2e44f008c4a492,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 27 | 0 | topics.toml count=118 | bank,registry | bank=7dd60e3b4816aa90925d18d8e7f9a8a720093c6ff786c2b4ba2e44f008c4a492,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 28 | 0 | sources fetch_date present | all | all=1d008f03bfae47fe84794620b6bcef88056548cce49de6285875de754023d29a | executed |
| 29 | 0 | bank pool | bank,registry | bank=7dd60e3b4816aa90925d18d8e7f9a8a720093c6ff786c2b4ba2e44f008c4a492,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 30 | 0 | answer-key distribution | bank,registry | bank=7dd60e3b4816aa90925d18d8e7f9a8a720093c6ff786c2b4ba2e44f008c4a492,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 31 | 0 | bank-internal contradiction floor | bank,registry | bank=7dd60e3b4816aa90925d18d8e7f9a8a720093c6ff786c2b4ba2e44f008c4a492,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 32 | 0 | construction-fault scan | bank,registry | bank=7dd60e3b4816aa90925d18d8e7f9a8a720093c6ff786c2b4ba2e44f008c4a492,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 33 | 0 | grounding heuristics | bank,registry | bank=7dd60e3b4816aa90925d18d8e7f9a8a720093c6ff786c2b4ba2e44f008c4a492,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 34 | 0 | grounding-wave stem regression detector | bank,registry | bank=7dd60e3b4816aa90925d18d8e7f9a8a720093c6ff786c2b4ba2e44f008c4a492,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 35 | 0 | no orphan topics · no orphan item refs · no unanchored items | bank,registry | bank=7dd60e3b4816aa90925d18d8e7f9a8a720093c6ff786c2b4ba2e44f008c4a492,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 36 | 0 | orphan selftest (empty bank/topics ERROR · orphan ref RED · unanchored RED · orphan topic RED · live GREEN) | bank,registry | bank=7dd60e3b4816aa90925d18d8e7f9a8a720093c6ff786c2b4ba2e44f008c4a492,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 37 | 0 | no cosmetic near-duplicates in the approved pool (NOT a distinct-proposition count) | bank,registry | bank=7dd60e3b4816aa90925d18d8e7f9a8a720093c6ff786c2b4ba2e44f008c4a492,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 38 | 0 | near-duplicate selftest (planted clone trips RED) | bank,registry | bank=7dd60e3b4816aa90925d18d8e7f9a8a720093c6ff786c2b4ba2e44f008c4a492,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 39 | 0 | paraphrase pair ledger intact (804/779 is a pool size; report is not a verdict) | bank,registry | bank=7dd60e3b4816aa90925d18d8e7f9a8a720093c6ff786c2b4ba2e44f008c4a492,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 40 | 2 | rustfmt over checked=17 compressed-dispatchers=6 non-dispatcher gate module(s) (compact dispatchers classified structurally; L4 plant RED · empty glob ERROR · only-mod.rs ERROR · formatted GREEN · stale plant STALE PLANT) | rust,registry | rust=ec124c50734ab4e06d41243c8d65e197dfe53e9074235be517be0d84e2348bfc,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | cargo fmt |
| 41 | 2 | cargo fmt + clippy -D warnings + test | rust,registry | rust=ec124c50734ab4e06d41243c8d65e197dfe53e9074235be517be0d84e2348bfc,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | cargo test |
| 42 | 0 | L3 golden artifacts present | registry,artifact | registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885,artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493 | executed |
| 43 | 2 | GradeExact goldens | registry,artifact | registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885,artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493 | goldens check |
| 44 | 0 | known-bad selftests (gates trip, tree clean) | rust,registry | rust=ec124c50734ab4e06d41243c8d65e197dfe53e9074235be517be0d84e2348bfc,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 45 | 2 | installer known-bad (tampered tarball · empty assets · missing checksum · D1) | all | all=1d008f03bfae47fe84794620b6bcef88056548cce49de6285875de754023d29a | installer known-bad |
| 46 | 2 | knowledge primary_notes paths | docs,registry | docs=9c61124fd0b56bc92a25a89df98527aac69c45560840253a1ec43c4ea4779ee1,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | L4 WASM dual-path failed (toolchain present but digests disagree or test/build error) |
| 47 | 0 | L5 product files present | artifact,registry | artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 48 | 0 | L5 wasm artifact present under web/assets/wasm/ | artifact,registry | artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 49 | 0 | L5 learner pack n_items=40 | artifact,registry | artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 50 | 0 | L5 selftest (honesty plant RED · digest match · flipped golden RED · empty fixtures ERROR) | registry,artifact | registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885,artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493 | executed |
| 51 | 0 | L5 e2e digest match (seed42 all-correct/all-wrong) | artifact,registry | artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 52 | 0 | Learn surface (modules_index · topic_anchors · pages · copies) | artifact,registry | artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 53 | 0 | Reference surface (reference.html · glossary · power cheatsheet) | artifact,registry | artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 54 | 0 | L5 learn smoke | artifact,registry | artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 55 | 0 | L6 short-interval review smoke | artifact,registry | artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 56 | 0 | L6 mastery smoke | artifact,registry | artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 57 | 0 | L6 weak-links smoke | artifact,registry | artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 58 | 0 | L6 hub mastery + recommend smoke | artifact,registry | artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 59 | 0 | L6-S4 hub mastery surface wired | artifact,registry | artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 60 | 2 | L6 multi-seed export-web --seed 42 (fixture golden-stable) | registry,artifact | registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885,artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493 | export-web stale artifacts |
| 61 | 0 | L6 session shapes (Drill due · Miss review) present | artifact,registry | artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 62 | 0 | L6 coverage GREEN (every module the domain registry declares ≥ domain_min) | bank,registry | bank=7dd60e3b4816aa90925d18d8e7f9a8a720093c6ff786c2b4ba2e44f008c4a492,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 63 | 0 | L6 coverage selftest (empty RED · missing-module RED · live GREEN) | bank,registry | bank=7dd60e3b4816aa90925d18d8e7f9a8a720093c6ff786c2b4ba2e44f008c4a492,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 64 | 0 | L7 surfaces (reference · closed-notes · Learn-15) | artifact,registry | artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 65 | 0 | M8-A learn chrome smoke | all | all=1d008f03bfae47fe84794620b6bcef88056548cce49de6285875de754023d29a | executed |
| 66 | 0 | M8-B units_index | all | all=1d008f03bfae47fe84794620b6bcef88056548cce49de6285875de754023d29a | executed |
| 67 | 0 | M8-D glossary.json | all | all=1d008f03bfae47fe84794620b6bcef88056548cce49de6285875de754023d29a | executed |
| 68 | 0 | MODULE_LEARN_SLUGS from domains.toml | all | all=1d008f03bfae47fe84794620b6bcef88056548cce49de6285875de754023d29a | executed |
| 69 | 0 | no learner surface draws a non-approved item | artifact,registry | artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 70 | 0 | M8-B/D learn v2 smoke | all | all=1d008f03bfae47fe84794620b6bcef88056548cce49de6285875de754023d29a | executed |
| 71 | 0 | M8-C diagrams smoke | artifact,registry | artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 72 | 0 | L7 a11y baseline | artifact,registry | artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 73 | 0 | L7-S2 feedback section-anchor links smoke | artifact,registry | artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 74 | 0 | L7 feedback section links | artifact,registry | artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 75 | 2 | L7 CLI product verbs listed | artifact,registry | artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | learner --help missing repair |
| 76 | 0 | cdcp test | rust,registry | rust=ec124c50734ab4e06d41243c8d65e197dfe53e9074235be517be0d84e2348bfc,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 77 | 0 | cdcp demo --no-open | all | all=1d008f03bfae47fe84794620b6bcef88056548cce49de6285875de754023d29a | executed |
| 78 | 0 | cdcp study served HTTP 200 | artifact,registry | artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 79 | 0 | learner verbs known-bad (test wasm · demo/study missing-bundle · ignore-exit is RED · study stop reaps cdcp) | rust,registry | rust=ec124c50734ab4e06d41243c8d65e197dfe53e9074235be517be0d84e2348bfc,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 80 | 0 | L7 objective coverage (registry objectives resolve · every declared module carries items) | bank,registry | bank=7dd60e3b4816aa90925d18d8e7f9a8a720093c6ff786c2b4ba2e44f008c4a492,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 81 | 0 | L7 objectives known-bad selftest | bank,registry | bank=7dd60e3b4816aa90925d18d8e7f9a8a720093c6ff786c2b4ba2e44f008c4a492,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 82 | 0 | L7 SLO budgets | artifact,registry | artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 83 | 2 | L7 content.lock | artifact,registry | artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | L7 content.lock |
| 84 | 0 | L7 content.lock selftest (mutated bank_hash trips RED) | bank,registry | bank=7dd60e3b4816aa90925d18d8e7f9a8a720093c6ff786c2b4ba2e44f008c4a492,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 85 | 2 | L5–V11 reconstructed stages proven to trip RED | artifact,registry | artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | reconstructed-stage selftests |
| 86 | 0 | public copy free of marketing slop; honesty note intact | docs,registry | docs=9c61124fd0b56bc92a25a89df98527aac69c45560840253a1ec43c4ea4779ee1,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 87 | 0 | roadmap milestone status agrees across docs; publication truth holds | docs,registry | docs=9c61124fd0b56bc92a25a89df98527aac69c45560840253a1ec43c4ea4779ee1,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 88 | 0 | roadmap selftest (dup row RED · cross-doc conflict RED · unreadable status RED · pending-publication RED · zero markdown ERROR) | rust,registry | rust=ec124c50734ab4e06d41243c8d65e197dfe53e9074235be517be0d84e2348bfc,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 89 | 0 | L88 publishability bar (audit claims verified against the repo) | registry,artifact | registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885,artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493 | executed |
| 90 | 0 | V11 Anki planted all-retired is RED and writes nothing | artifact,registry | artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 91 | 0 | V11 Anki export tsv/csv/apkg (approved count checked by count-pin-drift, pinned crt) | artifact,registry | artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 92 | 0 | V11 Anki .apkg deck | artifact,registry | artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 93 | 0 | V11 diagram honesty present | artifact,registry | artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 94 | 0 | V11 serve subcommand present | artifact,registry | artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 95 | 0 | V11 runbook bank items present | bank,registry | bank=7dd60e3b4816aa90925d18d8e7f9a8a720093c6ff786c2b4ba2e44f008c4a492,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 96 | 0 | drift-guard selftest (off-by-one RED · missing receipt RED · zero RED · unregistered RED · empty log ERROR) | rust,registry | rust=ec124c50734ab4e06d41243c8d65e197dfe53e9074235be517be0d84e2348bfc,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |
| 97 | 2 | advertised known-bad injection count == suites' self-reported total | all | all=1d008f03bfae47fe84794620b6bcef88056548cce49de6285875de754023d29a | known-bad injection count drift (README vs suites); re-run with CDCP_INJECTION_COUNT_WRITE_README=1 to regenerate |
| 98 | 0 | advertised content counts == units_index + measured WASM KiB | artifact,registry | artifact=ece77d91328df16b2cb3fe497f40a69c412afb9d77830846ed71bc7387ba9493,registry=f2e984366ae98bbd336d43039140e05dbb2198dede4ffe9a2380baed68b63885 | executed |

## First failure

- exit=2 name=S0 wiring proven behaviourally (a planted unlisted .py stops check.sh) reason=substrate-guard wiring proof emitted no nested-ok-receipts measurement
- bank selftest: byte plant changed bank scope 7dd60e3b4816aa90925d18d8e7f9a8a720093c6ff786c2b4ba2e44f008c4a492 -> 55c876409f251b1af3db5e87e9676709675644d925714bd39539c6e22f5113e9; scheduler=RUN; direct verify-bank exit=0; plant removed by trap.
