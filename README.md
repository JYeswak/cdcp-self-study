# CDCP Self-Study Program

<div align="center">
  <img src="visual/hero.jpg" alt="Professor Yuzu in a data hall cold aisle, inspection sheet in hand">
</div>

<div align="center">

[![License: MIT (code)](https://img.shields.io/badge/code-MIT-blue.svg)](./LICENSE)
[![Content: CC BY-NC-SA 4.0](https://img.shields.io/badge/content-CC_BY--NC--SA_4.0-blue.svg)](./LICENSE)
[![gate: 90 steps](https://img.shields.io/badge/gate-90_ordered_steps-success.svg)](#the-gate)
[![known-bad (shell selftest suites): 72 injections](https://img.shields.io/badge/known--bad_(shell_selftest_suites)-72_injections_all_RED-success.svg)](#gates-proven-to-trip)
[![grading: byte-exact](https://img.shields.io/badge/grading-Rust_%3D%3D_WASM_byte--exact-success.svg)](#how-grading-works)
[![unsafe: forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![offline](https://img.shields.io/badge/runtime-fully_offline-teal.svg)](#running-it)
[![not a certification](https://img.shields.io/badge/not_a_certification-read_this-critical.svg)](#the-honesty-constitution)

</div>

**A free, offline, self-hosted course that teaches the data-centre facilities domain, plus a Rust engine that grades you the same way twice.** 15 modules of original writing (84,299 words) cover the fourteen publicly advertised EPI® CDCP® facility domains and one ops-adjacent supplement. The source bank contains 957 item files (931 approved, 25 retired); the indexed learner export ships 957 items (929 approved: a **pool size**, not a count of distinct propositions). The browser course's grader is a pure-Rust core compiled to WASM, pinned so the native and browser paths produce **byte-identical** result digests. No account, no telemetry, no network at runtime, no LLM in the grading path.

**This does not certify you.** It is a study tool. Only the official EXIN/EPI exam after authorised training grants the credential. That sentence is not boilerplate: it is a registered claim (`claim-not-epi-certified`) that `claims-lint` enforces across this README, `CHARTER.md`, and the engine's `README` and `docs/`, and the build fails if a load-bearing page asserts it without citation. [[claim:claim-not-epi-certified]]

<div align="center">
<h3>Run it</h3>

```bash
git clone https://github.com/JYeswak/cdcp-self-study.git
cd cdcp-self-study
cargo build --manifest-path course-engine/Cargo.toml -p cdcp_cli --locked
./course-engine/target/debug/cdcp study
# → http://127.0.0.1:8766/  (binds loopback, prints the URL, opens a browser)
```

</div>

`study` is the learner verb. `serve` is the same loopback static server without opening a browser: Rust std only, GET/HEAD, path-traversal guarded. After the clone, nothing in this project talks to the network again. There is no GitHub Release tarball yet, so there is no working `curl | bash` install.

---

## TL;DR

| | |
|---|---|
| **What** | 15-module data-centre facilities curriculum (14 public EPI domains + 1 ops-adjacent supplement) + offline course engine |
| **Who it's for** | Someone who wants to walk a white-space tour and explain trade-offs: TPM, deploy engineer, ops, or a career switcher |
| **What it is not** | A certification, an exam dump, a paid course, an LLM tutor |
| **Study bar** | Mock exam 40 questions / 60 minutes / **27 correct is a study signal, not a pass mark** |
| **Bank** | 957 source item files / 931 approved · 25 retired; indexed learner export: 957 items / 929 approved (pool size, not distinct propositions) [[fact:fact-bank-item-count-804=yes]] [[fact:fact-bank-approved-count-779=yes]] · 15 modules · 106 topics |
| **Engine** | `#![forbid(unsafe_code)]` · 518 KiB WASM |
| **Gate** | 90 ordered steps; 10 selftest suites (registered; two more emit receipts but are deliberately unregistered); 72 known-bad injections that must all go RED |
| **Runtime deps** | None. Rust toolchain to build; a browser to use |

---

## The problem this solves

Data-centre facilities knowledge sits behind a paywall and a classroom door. The EPI CDCP course is two instructor-led days; the syllabus domains are public, the teaching is not. If you're a network deploy engineer who can pull fibre and configure a switch but has never had to explain why the UPS is upstream of the PDU, or what containment actually buys you, there is no good free path from "I work in this building" to "I can hold a conversation about how this building works."

The second problem is subtler and it is the reason this repo is built the way it is. **Self-study tools lie to you.** A quiz app that shuffles questions and shows a score is trivially fooled: by a stale question bank, by a grading bug, by a rubric that drifted from the material, by the tool cheerfully reporting success when it did nothing at all. You cannot tell a green screen that means "you learned this" from a green screen that means "the check never ran."

So this project treats *its own honesty* as the engineering problem. The curriculum is the product; the machinery exists to make the curriculum's claims checkable.

---

## The honesty constitution

Four rules, each enforced by a check:

1. **This is not a certification.** Completing anything here grants no credential. The claim `claim-not-epi-certified` is registered in `course-engine/registries/claims.toml`, and `claims-lint` fails the build if a scanned document makes a certification-adjacent statement without citing it. The scan set is the public claim surface: this README, `CHARTER.md`, and the engine's `README` and `docs/`. The research ledgers under `docs/curriculum-grades/` are outside it.
2. **No exam dumps, ever.** All 957 indexed learner-export items are original, written against public syllabus domains and industry-standard references. `source_class=original` is verified for every indexed item on every run. The source bank currently measures 957 files (931 approved, 25 retired); 957/929 is the indexed learner file-set / approved-pool size, not a count of distinct propositions.
3. **A score is a study signal, not a pass mark.** 27/40 is the internal bar. It is registered as `claim-study-signal-27` and the phrase "study signal" is a load-bearing marker the linter tracks.
4. **Third-party material is not redistributed.** Standards bodies own their standards. The corpus records each source's URL, fetch date, SHA-256, and rights. The ASHRAE white papers this project *grounds against* are deliberately **not** in this repository; their metadata sidecars are, so grounding still verifies. Fetch the PDFs yourself.

The last one has teeth: `tests/publishability-bar.sh` fails if any corpus source lacks a rights field, and treats an empty source list as an error rather than a pass.

---

## Why it exists the way it does

### Why Rust, and why the grader is pure

The grader is the one component that must never be wrong, because a wrong grader silently teaches you the wrong thing. So `cdcp_grade` is a pure function (bank plus attempt in, digest out) with no I/O, no clock, no randomness, and `#![forbid(unsafe_code)]`. That purity is what makes the next decision possible.

### Why the browser and the CLI must agree byte-for-byte

The course runs in your browser; the gate runs in CI. If those two graders can disagree, every claim the gate makes about the product is void. So the same Rust core is compiled twice: natively and to `wasm32-unknown-unknown`. The gate asserts the two paths produce **identical digests** on pinned fixtures, with the two engines carrying deliberately *distinct* identity strings so a test cannot accidentally compare a path against itself.

That is the `claim-grade-byte-exact` claim, and it is checked on every run against `goldens/mock40_seed42_all_correct.sha256` and its all-wrong twin. [[claim:claim-grade-byte-exact]]

### Why there is no LLM in the product

An LLM grader cannot be pinned to a golden. It would make the byte-exactness claim meaningless and turn every score into an unfalsifiable opinion. LLMs were used to *help build* this (that is what `course-engine/docs/` and `course-engine/scorecards/` record), but nothing in the shipped path calls a model. Grading is deterministic arithmetic over a hashed bank.

### Why claims live in a registry

Prose drifts. A README says "byte-exact" long after the property stopped holding, and nobody notices because prose has no build step. So load-bearing claims live in `registries/claims.toml` on a strength lattice: `invariant`(6) > `proof`(5) > `bounded_model`(4) > `statistical`(3) > `slo`(2) > `benchmark`(1). A weaker claim may never justify a stronger one. `cdcp_registry_check` is a dedicated crate **with its own test suite**: the checker is itself checked. An empty registry is an error, never a pass.

---

## System map

```text
cdcp-self-study/
├── modules/                  15 module files (14 EPI domains + ops-adjacent) · 84,299 words · original writing
├── practice/                 40-question practice exam · 40 drill cards
├── reference/                glossary (100+ terms) · power & redundancy cheatsheet
├── 00-curriculum-map.md      every module's learning objectives
├── STUDY-PLAN-14-DAY.md      day-by-day capstone schedule
├── CHARTER.md                what this is, what "done" means, what gates it
└── course-engine/
    ├── crates/
    │   ├── cdcp_core         types: items, attempts, choice letters
    │   ├── cdcp_bank         load + validate bank/items/*.toml → bank_hash
    │   ├── cdcp_assemble     seeded stratified sampling + choice shuffle
    │   ├── cdcp_grade        the pure grader: grade() → digest
    │   ├── cdcp_schedule     short-interval ladder + mastery bars (not SRS)
    │   ├── cdcp_wasm         same core, wasm32; the dual-path subject
    │   ├── cdcp_cli          bank-hash · grade · goldens · export-web · serve · study
    │   ├── cdcp_registry_check  L1 claims constitution (tested checker)
    │   ├── cdcp_data         L3 site-quantity oracle vs NREL / USGS / EPA
    │   ├── cdcp_gate         verify-step-count · verify-injection-count · doc-facts
    │   └── … 8 more          anki · assess · attempts · learn · root · site · metrics · evidence
    ├── bank/items/           957 source item files, one TOML each (source-tree count, not distinct propositions)
    ├── knowledge/            curriculum + standards citation graph (git truth)
    ├── registries/           10 files: claims · claims_lint · doc-facts · objectives · capability-maturity · …
    ├── goldens/              pinned digests — the byte-exactness evidence
    ├── web/                  static course: Hub · Learn · Drill · Mock · Reference
    ├── scripts/              check.sh is THE gate
    └── tests/                publishability + voice gates
```

**Data flow, once:**

```text
bank/items/*.toml ──► cdcp_bank ──► bank_hash (SHA-256 over the sorted bank)
                                        │
              seed ──► cdcp_assemble ────┤ stratified 40 of 929 approved items in indexed export (957 files)
                                        ▼
                              cdcp_grade (pure) ──► result digest
                                   ║        ║
                        native ────╝        ╚──── wasm32
                                   ╚═ MUST BE EQUAL ═╝
                                        │
                                   goldens/*.sha256
```

---

## Running it

### Study without the engine

The curriculum is plain Markdown and stands alone.

```bash
less 00-curriculum-map.md          # the 15 modules and their objectives
less STUDY-PLAN-14-DAY.md          # ~1–2 h/day capstone plan
less modules/01-mission-critical.md
```

### Run the course

Requires a Rust toolchain (`rustup`). Nothing else.

```bash
cargo build --manifest-path course-engine/Cargo.toml -p cdcp_cli --locked
./course-engine/target/debug/cdcp study
# same server, no browser:  ./course-engine/target/debug/cdcp serve --bind 127.0.0.1:8766
```

Then open `http://127.0.0.1:8766/` if it did not open itself. Hub → **Learn** for a module walkthrough,
**Drill** for short-interval review (1-day/3-day ladder, not SRS), **Mock** for a
timed 40-question exam.

> Do **not** open `web/index.html` as a `file://` URL if you want the quiz or
> WASM grading. Browsers block `fetch` on `file://`, so the question packs never
> load. Use `study` or `serve`. Port 8765 is often taken; 8766 is the documented default.

### Command reference

```bash
# from the repo root
cargo build --manifest-path course-engine/Cargo.toml -p cdcp_cli --locked

# Learner path: bind loopback, print the URL, open a browser.
./course-engine/target/debug/cdcp study
./course-engine/target/debug/cdcp study --no-open   # print the URL only

# Print the bank fingerprint. Every pack and golden is keyed to this.
./course-engine/target/debug/cdcp bank-hash --bank course-engine/bank/items

# Grade a fixture without a browser. Modes: all-correct | all-wrong | json answers.
./course-engine/target/debug/cdcp grade \
  --bank course-engine/bank/items \
  --fixture course-engine/goldens/fixtures/mock40_seed42.json \
  --mode all-correct

# Verify the pinned digests still hold (this is the byte-exactness check).
./course-engine/target/debug/cdcp goldens check --bank course-engine/bank/items --dir course-engine/goldens

# Regenerate browser exam packs. Seed 42 is golden-pinned; any other seed is
# practice-only variety and must never be used to refresh a golden.
./course-engine/target/debug/cdcp export-web --bank course-engine/bank/items --seed 42 --out course-engine/web/data
./course-engine/target/debug/cdcp export-web --seed 7 --out course-engine/web/data

# Serve the course locally (loopback only, GET/HEAD, traversal-guarded).
./course-engine/target/debug/cdcp serve --root course-engine/web --bind 127.0.0.1:8766
```

`export-web` writes three files per seed, and the split is deliberate:

| File | Audience | Contains |
|---|---|---|
| `mock40_seed{N}.json` | the learner UI | stems + choices **only**; no answer letters |
| `keys_seed{N}.json` | e2e tests + post-grade explanations | correct letters + explanations |
| `bank_items_seed{N}.json` | WASM grader | full bank incl. `correct`, required for offline GradeExact |

The learner pack omits answers by design, and the gate asserts it: strip that
property and `L5 learner pack answer-key leak` turns the build red.

---

## The gate

One ordered chain. It is the only definition of "done" in this project.

```bash
cd course-engine && ./scripts/check.sh
```

90 steps, fail-closed, each naming the script that failed so the repair is
obvious. Roughly: constitution docs → knowledge pack → **L1 claims registry** →
bank validation → **L3 GradeExact + goldens** → **L4 known-bad selftests** →
**L4 Rust==WASM dual path** → L5 browser surface + e2e digests → L6 coverage,
mastery, multi-seed stability → L7 SLO budgets, `content.lock`, a11y,
objectives → V11 Anki, diagrams, serve, runbooks → publishability + voice gates.

CI runs that script as its only gate, then adds two assertions around it: a
re-run under an injected fault, and a tree-clean check. Duplicating gate commands
in a workflow file is how CI and local drift apart until only one of them is true.

### Gates proven to trip

A green gate is worthless unless it can go red. 10 selftest suites inject **72 known-bad faults** (shell selftest suites), assert the build fails, then restore the tree.

Two populations are deliberately outside that total [[fact:fact-injections-enforced=yes]]. The Rust ports' own
known-bad cases emit no `INJECTIONS=` receipt at all. And two further *shell*
suites, `selftest_install.sh` (`installer`) and `selftest_learner_verbs.sh`
(`learner_verbs`), do emit receipts but are kept out of `REGISTERED_SUITES` to
hold `cdcp_gate` inside its `gate_shrink` budget. Counting either would be a
number with no receipt behind it:

| Suite | n | Injections |
|---|---|---|
| `selftest_known_bad` | 6 | flipped golden · empty bank · bank_hash drift · planted honesty violation |
| `selftest_l5` | 2 | flipped golden pins (GOLDEN MISMATCH) · empty golden dir (zero fixtures) |
| `selftest_l5_honesty` | 1 | credential-inflation string planted in web copy |
| `selftest_l6_coverage` | 2 | empty bank · single-module bank |
| `selftest_l7_objectives` | 8 | empty objectives · missing claim ref · empty claim_ids · empty bank · declared module starved of items · exemption without a reason (module stays required) · `[[domain_min]]` for an undeclared module · topic in an undeclared domain |
| `selftest_reconstructed` | 5 | learner-pack shape · answer-key leak · export byte-stability · session shapes · CLI verb presence |
| `selftest_orphan` | 6 | empty bank · empty topic registry · unknown `topic_id` · empty `topic_ids` · orphan topic · file whose `items[]` yields nothing |
| `selftest_doc_consistency` | 7 | duplicate milestone row · cross-doc status conflict · unreadable status vocabulary · stale pre-flip visibility claim · zero markdown scanned · roadmap doc missing · row too short to reach its Status column |
| `selftest_injection_count` | 33 | **injection count (19):** off-by-one count · deleted receipt (MISSING, never zero) · suite reporting 0 · unregistered suite · empty log · README advertising nothing · wrong suite count · word-spelled site drifted · a finding naming a file it did not scan · a suite named twice in `--require` · an advertisement site removed (site floor) · `--write-readme` refusing to write an unsound total · per-suite cell low · per-suite cell high · missing suite row · unregistered suite row · empty table · `--write-readme` refusing unsound cells · known-bad advertisement without a shell/selftest qualifier<br>**step count (14):** missing receipt log · empty receipt log · receipt shape drifted · only a nested `DEPTH>0` receipt (never a fallback to the child's number) · two `DEPTH=0` receipts (never a sum) · a run that counted **zero** steps · a receipt that does not add up · `NESTED_OK=0` (the nested hazard never occurred) · a step added with README untouched · README edited with the chain untouched · README advertising no step count · a step advertisement site removed · an `ok` call below the receipt boundary · `--write-readme` refusing an unsound step total |
| `wasm-freshness` | 2 | flipped committed `cdcp_wasm.wasm` byte (RED naming the wasm) · grade-affecting constant rebuilt native-only (dual-path mismatch) |

**Nothing here maintains these numbers by hand; the suites emit them.** Each
suite counts only the injections it *observed* go RED and prints
`INJECTIONS=<n> SUITE=<name>` on its success path; `verify_injection_count.py`
sums those receipts and fails the build if the total disagrees with any number
this README advertises. The per-suite `n` column is compared to those same
receipts. A cell that disagrees is RED. Editing a count here without changing the
suites turns the gate red. A suite that emits no receipt is an **error, never a
silent zero**; otherwise a suite that stopped reporting would read exactly like a
suite with nothing to report.

**The step count was the last advertised number here that nothing enforced. It is now enforced the same way.**
`check.sh` counts as it runs (`ok` + honest `skip`), emits one `CHECK_STEPS=`
receipt on the success path [[fact:fact-check-steps-enforced=yes]], and `verify-step-count` compares that receipt to
every step-count this README advertises. A wrong number here turns the gate
RED. The count cannot be parsed out of the script (several legs are
conditional) and it cannot be grepped out of a transcript either: a nested
`--prove-wired` child writes `check.sh: ok:` lines that a transcript counter
would swallow. The receipt is written by the process that did the counting.

`tests/publishability-bar.sh` is deliberately excluded from that total: it
asserts facts about the repo and plants no known-bad, so counting it would
inflate the number.

Anti-vacuous discipline runs throughout: an **empty input set is an error, not a
pass**. A scan that finds zero files fails, because a deliverable that was never
checked reports identically to one that passed.

### How grading works

`cdcp_grade::grade_digest` is pure: the same bank and the same attempt always
produce the same digest. The WASM build compiles that identical code to
`wasm32-unknown-unknown` and the dual-path test asserts:

- the two engines report **distinct identity strings** (so the test cannot
  compare a path against itself), and
- their digests are **equal** on the pinned all-correct and all-wrong fixtures.

Fixtures are pinned by `content.lock`, which covers four sections: `bank_hash`,
the knowledge pack, module markdown, and the vendored `[data]` snapshots. Change
a question and the lock breaks until you regenerate it deliberately. Content
drift cannot slip in unnoticed.

---

## How it was built

The engine was written by AI agents under a human charter, in a loop designed so
the agents could not grade their own homework.

**The charter came first.** `CHARTER.md` fixes the buyer, the product, the
honesty constitution, the value bar, and which irreversible actions require a
human. It is edited in place, never superseded. A charter that spawns successor
charters is a project that has stopped shipping.

**Claims before code.** Load-bearing properties were registered in
`registries/claims.toml` with a strength rank before the code that would justify
them existed. `claims-lint` then refuses to let prose outrun the registry: write
"byte-exact" in a document without citing a registered claim and the build
fails, naming the file and line.

**Gates before features.** Each wave (W0 → L7 → V11 → M8) landed with its own
known-bad selftest, so "this wave is green" always meant "and here is the
injection proving it can go red."

**The recovery test.** On 2026-08-12 an agent ran `git reset --hard` and
destroyed a day of uncommitted work in this repo. That incident is why several
things here look the way they do:

- The `export-web` verb was lost and **reimplemented from the specs alone**
  (`web/data/README.md`, `docs/PHASE-L6.md`). It now reproduces all three
  committed seed-42 packs byte-for-byte, which is how we know the
  reimplementation is faithful and not merely plausible.
- A latent bug surfaced: `0xC_DCP_5UFF_1E` is not valid hexadecimal (`P` and `U`
  are not hex digits). It had never compiled, and a stale `cargo` cache had
  masked it for days while the gate reported green.
- `selftest_known_bad.sh` had a `set -e` leak that aborted the script before its
  fourth check ever ran: a gate silently testing three of four things.

Both defects existed before the incident and were found by *forcing a rebuild*,
not by reading. That is the argument for gates that trip over gates that pass.

The full trail of decisions, waves, scorecards, and open questions lives in
`course-engine/docs/` and `course-engine/scorecards/`.

---

## Rigor: which layers apply

This project adopts a subset of a seven-layer artifact-rigor standard. Claiming
a layer you have not wired is worse than not claiming it, so here is the honest
table:

| Layer | Applies | Evidence |
|---|---|---|
| **L1 — claims constitution** | ✅ | `registries/*.toml` + `cdcp_registry_check` (tested crate) + claims-lint over README/docs |
| **L2 — SLO as code** | ✅ partial | `slo.toml` budgets; `cdcp slo check` judges an elapsed sample against the live `grade_ms` wall (`crates/cdcp_cli/tests/slo.rs`, with a planted over-budget known-bad). The `smoke_slo.sh` shell leg is conditional on the `export-web` verb and honours a documented `CDCP_SKIP_SLO=1` bypass |
| **L3 — external oracle** | ✅ scoped · ⚠️ **bank keys unguarded** | `cdcp_data` compares computed site quantities (free-cooling hours, seismic design values, grid carbon) against published NREL TMY3 / USGS ASCE 7-16 / EPA eGRID2023 references we do not control, with pre-declared tolerances — `crates/cdcp_data/tests/oracle.rs`, wired via `cargo test --workspace`, with a perturb-by-one-tolerance known-bad and anti-vacuous floors. **No external suite checks the 929 indexed bank item keys** for "did we teach this correctly." The native grader and the goldens are **not** oracles (CHARTER §5a) |
| **L4 — gates proven to trip** | ✅ strongest | 10 suites, 72 injections (shell selftest suites; Rust legs uncounted), count drift-guarded, anti-vacuous throughout |
| **L5 — adversarial input floor** | ⚠️ partial | proptest floor on assemble/grade shapes (wired). `cargo-fuzz` targets exist, but `fuzz/` is **outside the workspace**, so no libFuzzer campaign is ever run by `check.sh` or CI [[fact:fact-fuzz-is-a-workspace-member=no]] |
| **L6 — formal lane** | ❌ | Not warranted at this gauntlet tier |
| **L7 — ecosystem lock** | ✅ scoped: content, not dependencies | `content.lock` pins bank_hash + knowledge + module markdown + `[data]` by sha256. Cargo deps are pinned by `Cargo.lock` and `--locked` only; there is no git-rev lockfile |

**Bank content is the honest gap.** Byte-exact grading proves the engine is
self-consistent; it says nothing about whether Module 09 teaches cooling
correctly. The site-quantity oracle closed part of L3, and it can be wrong in a
way we did not author. But it checks computed values against NREL/USGS/EPA, not
answer keys against the field. For those, the only real external oracle is a
reader who knows the domain telling us we got it wrong. That is what the issue
tracker is for.

---

## Roadmap

Tracked in `CHARTER.md` §9 and `course-engine/docs/PHASE-NEXT.md`; this is a
summary, those are the source of truth.

| ID | Milestone | Status |
|---|---|---|
| M0–M7 | Charter · registries · bank · grade/goldens · web mock · learn · short-interval review · WASM | **done** |
| V11 | Anki export · power-path diagram · `serve` · runbooks | **done** |
| M8 | Learn v2: 134 Learn units · TOC · micro-checks · diagram system · glossary [[fact:fact-learn-unit-count-134=yes]] | **done** |
| M9 | Publishability bar · OSS meta · visibility flip | **done** (2026-08-12; public at github.com/JYeswak/cdcp-self-study) |
| M10 | Free/public corpus expansion | **done**, 5 free PDFs referenced with rights recorded (further sourcing tracked as OQ-09/10, not M10) |
| P1 | More diagrams: fire sequence, standards map, cooling topologies | planned (`DIAGRAM-REGISTRY.md`) |
| — | **L3 external oracle.** Site quantities now check against NREL / USGS / EPA (`cdcp_data/tests/oracle.rs`, 2026-08-15); the 929 indexed bank answer keys remain unguarded | **open, and the most valuable thing to fix** |

Charter §10 lists what an agent may never do alone: spend, publish, sign, or
flip repository visibility. Those stay human.

---

## Limitations

- **It cannot certify you, and it will not pretend to.** No credential, no CEUs.
- **Facilities knowledge is regional.** Codes, fire regulation, and electrical
  practice vary by jurisdiction. This teaches concepts and vocabulary, not your
  local code. It does not replace a licensed engineer.
- **The standards themselves are not here.** TIA-942, EN 50600, and the ASHRAE
  guidelines are paid documents. This teaches *around* them from public
  descriptions and free material; it is not a substitute for the standards.
- **Advanced direction, not shipped.** CDCS calculation depth and CDFOS
  operations depth are the first planned tracks after this study surface
  (`bd-epi-ecosystem-ms4j.1` / `.2`). Neither is present today, and neither
  track would grant the EPI®/EXIN® credential of that name.
- **The question bank is unaudited by a third party.** The source bank currently
  contains 957 item files (931 approved, 25 retired); the public learner export
  indexes 957 items (929 approved: a pool size, not a distinct-proposition count).
  All are self-reviewed.
  Errors are likely; report them.
- **L3 is partial** (see the rigor table). Computed site quantities are checked
  against published NREL/USGS/EPA references; the bank's answer keys are not.
  Pedagogical correctness is still not externally validated.

---

## FAQ

**Will this get me the CDCP certification?** No. Take an authorised EPI/EXIN
course and sit the official exam. This builds the underlying knowledge.

**Are these real exam questions?** No, and that would be both a licence
violation and useless. All 957 indexed learner-export items are original, written against public
syllabus domains.

**Why is 27/40 the bar?** It mirrors the public exam form. EPI's course page
states the CDCP passing mark is 27 out of 40; EXIN states 68% (27/40 is 67.5%,
so treat 28 as the stricter reading). The mock reuses that shape so practice
feels calibrated, but here the number is a study signal only: a threshold for
your own review loop. This mock is scored by this project, means nothing to EPI
or EXIN, and grants no credential; only the official exam after authorised
training does that. [[claim:claim-study-signal-27]] [[claim:claim-not-epi-certified]]

**Can I use this commercially?** The engine, yes (MIT). The curriculum, no:
CC BY-NC-SA 4.0. Studying it yourself for a job or interview is *not* commercial
use; reselling it as paid training is.

**Does it phone home?** No. There is no telemetry, no account, no network call
at runtime. `serve` binds loopback and serves static files.

**Why is a citrus fruit teaching me about UPS topologies?** Yuzu is the
ZestStream mascot, a craftsman who carries receipts. The joke is the thesis:
claims are worth what their evidence proves.

---

## Contributing

Corrections to the curriculum are the most valuable contribution, especially
from people who do this work. See [`CONTRIBUTING.md`](./CONTRIBUTING.md).

The one hard rule: **the gate must stay green, and it must stay able to go red.**
A change that makes `check.sh` pass by weakening a check will be rejected. Add
the known-bad injection alongside the feature.

---

## Licence

Dual-licensed. See [`LICENSE`](./LICENSE).

- **Software** (`course-engine/{crates,scripts,web,fuzz}`): MIT
- **Curriculum** (`modules/`, `practice/`, `reference/`, `00-curriculum-map.md`,
  `STUDY-PLAN-14-DAY.md`, `HOW-TO-USE.md`, bank, knowledge): CC BY-NC-SA 4.0

Third-party trademarks (EPI®, EXIN®, CDCP®, ASHRAE, TIA) belong to their owners.
This project is independent and **not affiliated with, endorsed by, or certified
by EPI or EXIN**. Marks are used for domain identification only.

---

## See also

- [`CHARTER.md`](./CHARTER.md) — what this is and what "done" means
- [`course-engine/docs/ORACLE-GAUNTLET.md`](./course-engine/docs/ORACLE-GAUNTLET.md) — the testing constitution
- [`course-engine/docs/FEATURE_SURFACE.md`](./course-engine/docs/FEATURE_SURFACE.md) — what is actually built vs planned
- [`course-engine/docs/TESTING.md`](./course-engine/docs/TESTING.md) — skip policy and honest receipts
- [`SECURITY.md`](./SECURITY.md) · [`CODE_OF_CONDUCT.md`](./CODE_OF_CONDUCT.md)

*Educational reconstruction of public CDCP syllabus domains. Independent project. No warranty.*
