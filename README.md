# CDCP Self-Study Program

<div align="center">
  <img src="visual/hero.jpg" alt="Professor Yuzu in a data hall cold aisle, inspection sheet in hand">
</div>

<div align="center">

[![License: MIT (code)](https://img.shields.io/badge/code-MIT-blue.svg)](./LICENSE)
[![Content: CC BY-NC-SA 4.0](https://img.shields.io/badge/content-CC_BY--NC--SA_4.0-blue.svg)](./LICENSE)
[![gate: 75 steps](https://img.shields.io/badge/gate-75_ordered_steps-success.svg)](#the-gate)
[![known-bad (shell selftest suites): 70 injections](https://img.shields.io/badge/known--bad_(shell_selftest_suites)-70_injections_all_RED-success.svg)](#gates-proven-to-trip)
[![grading: byte-exact](https://img.shields.io/badge/grading-Rust_%3D%3D_WASM_byte--exact-success.svg)](#how-grading-works)
[![unsafe: forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![offline](https://img.shields.io/badge/runtime-fully_offline-teal.svg)](#running-it)
[![not a certification](https://img.shields.io/badge/not_a_certification-read_this-critical.svg)](#the-honesty-constitution)

</div>

**A free, offline, self-hosted course that teaches the data-centre facilities domain — and a Rust engine that grades you the same way twice.** Fourteen modules of original writing (~54,000 words) covering the publicly advertised EPI® CDCP® syllabus domains, an 804-item question bank (779 approved — a **pool size**, not a count of distinct propositions), and a browser course whose grader is a pure-Rust core compiled to WASM, pinned so the native and browser paths produce **byte-identical** result digests. No account, no telemetry, no network at runtime, no LLM in the grading path.

**This does not certify you.** It is a study tool. Only the official EXIN/EPI exam after authorised training grants the credential. That sentence is not boilerplate — it is a registered claim (`claim-not-epi-certified`) that a linter enforces across every document in this repository, and the build fails if a load-bearing page asserts it without citation.

<div align="center">
<h3>Run it</h3>

```bash
git clone <this-repo> cdcp-self-study && cd cdcp-self-study/course-engine
cargo run -p cdcp_cli -- serve --bind 127.0.0.1:8766
# → http://127.0.0.1:8766/
```

</div>

`serve` is a local-only static server written on Rust's standard library — zero added dependencies, loopback bind, GET/HEAD only, path-traversal guarded. After the clone, nothing in this project talks to the network again.

---

## TL;DR

| | |
|---|---|
| **What** | 15-module data-centre facilities curriculum (14 public EPI domains + 1 ops-adjacent supplement) + offline course engine |
| **Who it's for** | Someone who wants to walk a white-space tour and explain trade-offs — TPM, deploy engineer, ops, or a career switcher |
| **What it is not** | A certification, an exam dump, a paid course, an LLM tutor |
| **Study bar** | Mock exam 40 questions / 60 minutes / **27 correct is a study signal, not a pass mark** |
| **Bank** | 804 original item files / 779 approved (pool size, not distinct propositions) [[fact:fact-bank-item-count-804=yes]] [[fact:fact-bank-approved-count-779=yes]] · 15 modules · 106 topics |
| **Engine** | 7 Rust crates, 3,763 lines, `#![forbid(unsafe_code)]`, 281 KB WASM |
| **Gate** | 75 ordered steps; 9 selftest suites; 70 known-bad injections (shell selftest suites) that must all go RED |
| **Runtime deps** | None. Rust toolchain to build; a browser to use |

---

## The problem this solves

Data-centre facilities knowledge sits behind a paywall and a classroom door. The EPI CDCP course is two instructor-led days; the syllabus domains are public, the teaching is not. If you're a network deploy engineer who can pull fibre and configure a switch but has never had to explain why the UPS is upstream of the PDU, or what containment actually buys you, there is no good free path from "I work in this building" to "I can hold a conversation about how this building works."

The second problem is subtler and it is the reason this repo is built the way it is. **Self-study tools lie to you.** A quiz app that shuffles questions and shows a score is trivially fooled — by a stale question bank, by a grading bug, by a rubric that drifted from the material, by the tool cheerfully reporting success when it did nothing at all. You cannot tell a green screen that means "you learned this" from a green screen that means "the check never ran."

So this project treats *its own honesty* as the engineering problem. The curriculum is the product; the machinery exists to make the curriculum's claims checkable.

---

## The honesty constitution

Four rules, enforced mechanically rather than promised in prose:

1. **This is not a certification.** Completing anything here grants no credential. The claim `claim-not-epi-certified` is registered in `course-engine/registries/claims.toml`, and `claims-lint` fails the build if any document makes a certification-adjacent statement without citing it. Measured: five documents were caught asserting it uncited and had to be fixed before the gate would pass.
2. **No exam dumps, ever.** All 804 item files are original, written against public syllabus domains and industry-standard references. `source_class=original` is verified for every item on every run. 804/779 is a file-set / approved-pool size, not a count of distinct propositions.
3. **A score is a study signal, not a pass mark.** 27/40 is the internal bar. It is registered as `claim-study-signal-27` and the phrase "study signal" is a load-bearing marker the linter tracks.
4. **Third-party material is not redistributed.** Standards bodies own their standards. The corpus records each source's URL, fetch date, SHA-256, and rights — and the ASHRAE white papers this project *grounds against* are deliberately **not** in this repository. Their metadata sidecars are, so grounding still verifies; fetch the PDFs yourself.

The last one has teeth: `tests/publishability-bar.sh` fails if any corpus source lacks a rights field, and treats an empty source list as an error rather than a pass.

---

## Why it exists the way it does

### Why Rust, and why the grader is pure

The grader is the one component that must never be wrong, because a wrong grader silently teaches you the wrong thing. So `cdcp_grade` is a pure function — bank plus attempt in, digest out — with no I/O, no clock, no randomness, and `#![forbid(unsafe_code)]`. That purity is what makes the next decision possible.

### Why the browser and the CLI must agree byte-for-byte

The course runs in your browser; the gate runs in CI. If those two graders can disagree, every claim the gate makes about the product is void. So the same Rust core is compiled twice — natively and to `wasm32-unknown-unknown` — and the gate asserts the two paths produce **identical digests** on pinned fixtures, with the two engines carrying deliberately *distinct* identity strings so a test cannot accidentally compare a path against itself.

That is the `claim-grade-byte-exact` claim, and it is checked on every run against `goldens/mock40_seed42_all_correct.sha256` and its all-wrong twin.

### Why there is no LLM in the product

An LLM grader cannot be pinned to a golden. It would make the byte-exactness claim meaningless and turn every score into an unfalsifiable opinion. LLMs were used to *help build* this — that's what the `.planning/` trail records — but nothing in the shipped path calls a model. Grading is deterministic arithmetic over a hashed bank.

### Why claims are a registry and not a style guide

Prose drifts. A README says "byte-exact" long after the property stopped holding, and nobody notices because prose has no build step. So load-bearing claims live in `registries/claims.toml` on a strength lattice — `invariant`(6) > `proof`(5) > `bounded_model`(4) > `statistical`(3) > `slo`(2) > `benchmark`(1) — and a weaker claim may never justify a stronger one. `cdcp_registry_check` is a dedicated crate **with its own test suite**: the checker is itself checked. An empty registry is an error, never a pass.
---

## System map

```text
cdcp-self-study/
├── modules/                  14 EPI domains + ops-adjacent · ~62k words · original writing
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
    │   ├── cdcp_grade        THE ORACLE — pure grade() → digest
    │   ├── cdcp_schedule     short-interval ladder + mastery bars (not SRS)
    │   ├── cdcp_wasm         same core, wasm32 — the dual-path subject
    │   ├── cdcp_cli          bank-hash · grade · goldens · export-web · serve
    │   └── cdcp_registry_check  L1 claims constitution (tested checker)
    ├── bank/items/           804 original item files, one TOML each (file count, not distinct propositions)
    ├── knowledge/            curriculum + standards citation graph (git truth)
    ├── registries/           claims.toml · claims_lint.toml · objectives.toml
    ├── goldens/              pinned digests — the byte-exactness evidence
    ├── web/                  static course: Hub · Learn · Drill · Mock · Reference
    ├── scripts/              42 scripts; check.sh is THE gate
    └── tests/                publishability + voice gates
```

**Data flow, once:**

```text
bank/items/*.toml ──► cdcp_bank ──► bank_hash (SHA-256 over the sorted bank)
                                        │
              seed ──► cdcp_assemble ────┤ stratified 40 of 779 approved (804 files)
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
less 00-curriculum-map.md          # the 14 domains and their objectives
less STUDY-PLAN-14-DAY.md          # ~1–2 h/day capstone plan
less modules/01-mission-critical.md
```

### Run the course

Requires a Rust toolchain (`rustup`). Nothing else.

```bash
cd course-engine
cargo run -p cdcp_cli -- serve --bind 127.0.0.1:8766
```

Then open `http://127.0.0.1:8766/`. Hub → **Learn** for a module walkthrough,
**Drill** for spaced repetition, **Mock** for a timed 40-question exam.

> Do **not** open `web/index.html` as a `file://` URL if you want the quiz or
> WASM grading. Browsers block `fetch` on `file://`, so the question packs never
> load. Use `serve`. Port 8765 is often taken — 8766 is the documented default.

### Command reference

```bash
# Print the bank fingerprint. Every pack and golden is keyed to this.
cargo run -p cdcp_cli -- bank-hash

# Grade a fixture without a browser. Modes: all-correct | all-wrong | json answers.
cargo run -p cdcp_cli -- grade \
  --bank bank/items \
  --fixture goldens/fixtures/mock40_seed42.json \
  --mode all-correct

# Verify the pinned digests still hold (this is the byte-exactness check).
cargo run -p cdcp_cli -- goldens check --bank bank/items --dir goldens

# Regenerate browser exam packs. Seed 42 is golden-pinned; any other seed is
# practice-only variety and must never be used to refresh a golden.
cargo run -p cdcp_cli -- export-web --bank bank/items --seed 42 --out web/data
cargo run -p cdcp_cli -- export-web --seed 7 --out web/data

# Serve the course locally (loopback only, GET/HEAD, traversal-guarded).
cargo run -p cdcp_cli -- serve --root web --bind 127.0.0.1:8766
```

`export-web` writes three files per seed, and the split is deliberate:

| File | Audience | Contains |
|---|---|---|
| `mock40_seed{N}.json` | the learner UI | stems + choices **only** — no answer letters |
| `keys_seed{N}.json` | e2e tests + post-grade explanations | correct letters + explanations |
| `bank_items_seed{N}.json` | WASM grader | full bank incl. `correct` — required for offline GradeExact |

The learner pack omits answers by design, and the gate asserts it: strip that
property and `L5 learner pack answer-key leak` turns the build red.

---

## The gate

One ordered chain. It is the only definition of "done" in this project.

```bash
cd course-engine && ./scripts/check.sh
```

75 steps, fail-closed, each naming the script that failed so the repair is
obvious. Roughly: constitution docs → knowledge pack → **L1 claims registry** →
bank validation → **L3 GradeExact + goldens** → **L4 known-bad selftests** →
**L4 Rust==WASM dual path** → L5 browser surface + e2e digests → L6 coverage,
mastery, multi-seed stability → L7 SLO budgets, `content.lock`, a11y,
objectives → V11 Anki, diagrams, serve, runbooks → publishability + voice gates.

CI runs exactly this script and nothing else. Duplicating gate commands in a
workflow file is how CI and local drift apart until only one of them is true.

### Gates proven to trip

A green gate is worthless unless it can go red. Nine selftest suites inject
**70 known-bad faults** — shell selftest suites only; the Rust ports' own known-bad cases
emit no `INJECTIONS=` receipt, so they are not in this total [[fact:fact-injections-enforced=yes]] — and assert the
build fails, then restore the tree:

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

**These numbers are enforced, not maintained.** Each suite counts only the
injections it *observed* go RED and prints `INJECTIONS=<n> SUITE=<name>` on its
success path; `verify_injection_count.py` sums those receipts and fails the build
if the total disagrees with any number this README advertises. The per-suite `n`
column is compared to those same receipts — a cell that disagrees is RED.
Editing a count here without changing the suites turns the gate red. A suite that
emits no receipt is an **error, never a silent zero** — otherwise a suite that
stopped reporting would read exactly like a suite with nothing to report.

**The step count is enforced the same way, and it was the last one that wasn't.**
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

`cdcp_grade::grade_digest` is pure. Same bank, same attempt, same digest —
forever. The WASM build compiles that identical code to `wasm32-unknown-unknown`
and the dual-path test asserts:

- the two engines report **distinct identity strings** (so the test cannot
  compare a path against itself), and
- their digests are **equal** on the pinned all-correct and all-wrong fixtures.

Fixtures are pinned by `content.lock`, which covers `bank_hash`, the knowledge
pack, and module markdown. Change a question and the lock breaks until you
regenerate it deliberately — content drift cannot slip in unnoticed.
---

## How it was built

The engine was written by AI agents under a human charter, in a loop designed so
the agents could not grade their own homework.

**The charter came first.** `CHARTER.md` fixes the buyer, the product, the
honesty constitution, the value bar, and which irreversible actions require a
human. It is edited in place, never superseded — a charter that spawns successor
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
  committed seed-42 packs byte-for-byte — which is how we know the
  reimplementation is faithful and not merely plausible.
- A latent bug surfaced: `0xC_DCP_5UFF_1E` is not valid hexadecimal (`P` and `U`
  are not hex digits). It had never compiled, and a stale `cargo` cache had
  masked it for days while the gate reported green.
- `selftest_known_bad.sh` had a `set -e` leak that aborted the script before its
  fourth check ever ran — a gate silently testing three of four things.

Both defects existed before the incident and were found by *forcing a rebuild*,
not by reading. That is the argument for gates that trip over gates that pass.

The full trail — decisions, waves, scorecards, open questions — lives in
`course-engine/docs/` and `course-engine/scorecards/`.

---

## Rigor: which layers apply

This project adopts a subset of a seven-layer artifact-rigor standard. Claiming
a layer you have not wired is worse than not claiming it, so here is the honest
table:

| Layer | Applies | Evidence |
|---|---|---|
| **L1 — claims constitution** | ✅ | `registries/*.toml` + `cdcp_registry_check` (tested crate) + claims-lint over README/docs |
| **L2 — SLO as code** | ✅ partial | `slo.toml` + `smoke_slo.sh` walls on grade / export / bank-verify |
| **L3 — external oracle** | ⚠️ **weakest link** | The oracle is the *native* grader and the public syllabus domains. There is no independent third-party conformance suite for "did we teach this correctly" |
| **L4 — gates proven to trip** | ✅ strongest | 9 suites, 70 injections (shell selftest suites; Rust legs uncounted), count drift-guarded, anti-vacuous throughout |
| **L5 — adversarial input floor** | ✅ partial | `cargo-fuzz` targets present; property tests on assemble/grade |
| **L6 — formal lane** | ❌ | Not warranted at this gauntlet tier |
| **L7 — ecosystem lock** | ✅ scoped | `content.lock` pins bank_hash + knowledge + module markdown |

**L3 is the honest gap.** Byte-exact grading proves the engine is
self-consistent; it says nothing about whether Module 09 teaches cooling
correctly. The only real external oracle for that is a reader who knows the
domain telling us we got it wrong — which is what the issue tracker is for.

---

## Roadmap

Tracked in `CHARTER.md` §9 and `course-engine/docs/PHASE-NEXT.md`; this is a
summary, those are the source of truth.

| ID | Milestone | Status |
|---|---|---|
| M0–M7 | Charter · registries · bank · grade/goldens · web mock · learn · short-interval review · WASM | **done** |
| V11 | Anki export · power-path diagram · `serve` · runbooks | **done** |
| M8 | Learn v2 — 127 units · TOC · micro-checks · diagram system · glossary | **done** |
| M9 | Publishability bar · OSS meta · visibility flip | **DONE** (2026-08-12; public at github.com/JYeswak/cdcp-self-study) |
| M10 | Free/public corpus expansion | **done** — 4 free PDFs referenced, rights recorded (further sourcing tracked as OQ-09/10, not M10) |
| P1 | More diagrams: fire sequence, standards map, cooling topologies | planned (`DIAGRAM-REGISTRY.md`) |
| — | **L3 external oracle** | **open, and the most valuable thing to fix** |

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
- **No CDCS/CDCE depth.** Those are advanced design tracks and out of scope.
- **The question bank is unaudited by a third party.** 804 original item files
  (779 approved — a pool size, not a distinct-proposition count), self-reviewed.
  Errors are likely; report them.
- **L3 is thin** (see the rigor table). Self-consistency is proven; pedagogical
  correctness is not externally validated.

---

## FAQ

**Will this get me the CDCP certification?** No. Take an authorised EPI/EXIN
course and sit the official exam. This builds the underlying knowledge.

**Are these real exam questions?** No, and that would be both illegal and
useless. All 804 item files are original, written against public syllabus domains.

**Why is 27/40 the bar?** It is the internal study signal this project uses to
say "this domain is probably solid" — a threshold for your own review loop, not
a pass mark, and not affiliated with any official cut score.

**Can I use this commercially?** The engine, yes (MIT). The curriculum, no —
CC BY-NC-SA 4.0. Studying it yourself for a job or interview is *not* commercial
use; reselling it as paid training is.

**Does it phone home?** No. There is no telemetry, no account, no network call
at runtime. `serve` binds loopback and serves static files.

**Why is a citrus fruit teaching me about UPS topologies?** Yuzu is the
ZestStream mascot — a craftsman who carries receipts. The joke is the thesis:
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

Dual-licensed — see [`LICENSE`](./LICENSE).

- **Software** (`course-engine/{crates,scripts,web,fuzz}`) — MIT
- **Curriculum** (`modules/`, `practice/`, `reference/`, bank, knowledge) —
  CC BY-NC-SA 4.0

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
