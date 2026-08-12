# FRANKEN-EXTRACT — Assessment-System mapping

**Purpose.** Mine FrankenSuite rigor patterns and **adapt them to Assessment-System**
for `cdcp-course` (mock exams + learn/drill; grade in WASM dual-path to Rust).
This is **not** an ML ULP/CER port. Class assignment is fixed in
[`../ORACLE-GAUNTLET.md`](../ORACLE-GAUNTLET.md): Assessment-System.

---

## Provenance (paths + dates)

| Source | Studio mirror path | Key files mined | Tree notes |
|--------|--------------------|-----------------|------------|
| **franken_ocr** (primary) | `/Volumes/ZestData/dicklesworthstone-mirror/franken_ocr` | `AGENTS.md`, `docs/conformance/PARITY_LADDER.md`, `GOLDEN.md`, `LADDER_HARNESS.md`, `docs/gauntlet/METHODOLOGY.md`, `scripts/check.sh`, `tests/parity_ladder.rs`, `tests/support/parity_harness.rs` | AGENTS.md mtime 2026-07-16; git head observed 2026-08-11 |
| **franken_tts** | `…/franken_tts` | `AGENTS.md` Doctrine #0, `docs/CONFORMANCE_AND_LISTENING.md`, plan §9 ConformanceExact / ProductionQuality, `scripts/check.sh` skip-honest | AGENTS.md mtime 2026-08-08; git head observed 2026-08-11 |
| **franken_markdown** | `…/franken_markdown` | `AGENTS.md` WASM/determinism, `scripts/check-determinism.sh`, `check-wasm-package.sh` native parity, `SOURCE_DATE_EPOCH`, self-contained HTML (inlined CSS, no CDN asset path) | AGENTS.md mtime 2026-07-08; git head observed 2026-08-11 |
| **cdcp local stamp** | `/Users/josh/cdcp-self-study/course-engine` | `docs/ORACLE-GAUNTLET.md`, `docs/TESTING.md`, `scripts/check.sh` | W0 knowledge scaffold; GradeExact/WASM open |

**Access path for this extract (2026-08-11):** MacBook `/Volumes/ZestData` absent → SSH `studio` read of the Dicklesworthstone mirror. SocratiCode semantic search against `franken_ocr` (EngineIdentity, parity ladder, UPDATE_GOLDENS, skip-with-SUCCESS) confirmed the same design docs.

**Not used as oracle for CDCP product claims:** any EPI exam dump, unofficial “pass rate” marketing, or invented certification language.

---

## One Rule / G1>G2 / Doctrine #0

### One Rule (OCR gauntlet → our stamp)

**OCR (`METHODOLOGY.md` §0):** agent forbidden from declaring victory on one pillar
while another regresses (perf vs conformance vs surface).

**OCR ladder (`PARITY_LADDER.md` §0):** discrete output BIT-EXACT where the
reference is deterministic; continuous output only within **measured** tolerance.

**Our One Rule** ([`ORACLE-GAUNTLET.md`](../ORACLE-GAUNTLET.md)):

> No victory while grade paths disagree, required goldens are missing, or honesty
> claims inflate credentials.

Assessment-System simplification of the discrete half of OCR’s rule: **grade digests
are discrete and fully deterministic** — oracle floor = 0 (no ULP envelope). Continuous
“fuzzy” OCR rungs do **not** transfer.

### G1 > G2

From franken_ocr / franken_tts AGENTS doctrine #1:

- **G1 = correctness / parity first.**
- **G2 = speed / UX polish.**
- A faster kernel (or prettier UI) that drifts the product answer is **reverted** and
  ledgered in `NEGATIVE_EVIDENCE.md` — never “landed with a note.”

**cdcp mapping:** GradeExact (lawful score) outranks Hub polish, animation, or
“helpful” coverage heuristics that would change a score. PedagogySignal may improve
only when GradeExact stays green.

### Doctrine #0 — Anti-ceremony (franken_tts AGENTS, load-bearing)

1. **A process artifact exists only as a hard gate for a named capability** — name
   consumer, gate, observed defect class, deletion condition. Test: *does running code
   or a release gate branch on this artifact?*
2. **Process work earns zero capability credit.** “Wrote the harness” ≠ rung green.
3. **Meta-trap is occupational hazard.** Working engine is the deliverable.
4. **No counterfeit green.** Skip ≠ pass (skip-honest receipts). No silent tolerance
   bumps. Beads closed without exit criteria reopen with incident comment.
5. **Honest credit.** State equivalence tier per artifact class; refuse bare “done.”

**cdcp mapping:** do not mint successor charter/gauntlet docs when the stamp already
exists — edit [`ORACLE-GAUNTLET.md`](../ORACLE-GAUNTLET.md) in place. Research notes
(this file) are admissible only if L3–L5 gates **branch** on the checklist below.

---

## Subject Oracle Comparator + EngineIdentity

### OCR pattern (`METHODOLOGY.md` §1.1)

```
EngineIdentity::Subject  = "franken_ocr"           // Rust port under test
EngineIdentity::Oracle   = "unlimited-ocr-oracle"  // pinned external torch ref
```

- Labels **asserted-distinct at comparator entry** — blocks the highest-value false
  green: both pointers at the same engine.
- Oracle bridge is **test-only**, never linked into the shipping binary.
- Doctor preflight checks identity strings at harness entry.

### Assessment-System adaptation (cdcp)

| Identity | Implementation | Role |
|----------|----------------|------|
| **Oracle** | `cdcp_grade` native Rust (`#![forbid(unsafe_code)]`) | Reference of record for every score |
| **Subject** | Same crate → **WASM** (browser path) | What the learner’s browser runs |
| **Bank** | Content-addressed items; `bank_hash` in every report | Frozen exam body identity |

**Comparator law:** same `(bank_hash, exam_id, seed, answers)` → same
`sha256(canonical_json(GradeReport))` on both identities.

**EngineIdentity-Guard (must wire at L4):**

```text
assert!(subject_identity != oracle_identity);
assert_eq!(oracle_identity, "cdcp_grade_native");
assert_eq!(subject_identity, "cdcp_grade_wasm"); // or browser-e2e harness label
```

Never “compare WASM to WASM” or “re-run native twice and call it dual-path.” That is
the OCR self-compare false green, translated.

**What we do not copy:** PyO3 torch bridge, ULP tables, CUDA-host fixture generation.
Our oracle is **in-repo Rust** — simpler, and the floor is zero (byte-exact).

---

## Ladder L0–L5 (OCR) → our L0–L5 (grade)

| OCR rung | OCR compares | OCR tolerance | **cdcp grade rung** | **cdcp compares** | **tolerance** |
|----------|--------------|---------------|---------------------|-------------------|---------------|
| **L0** preprocess | tensors, tile geometry, image-token ids | EXACT | **L0** Bank + knowledge schema | bank schema, knowledge pack, sources fetch_date, domain crosswalk | EXACT schema + stable hashes |
| **L1** per-op | kernel activations | cosine / ULP | **L1** Single-item grade | one item key + answer → correct/incorrect + rationale id | EXACT |
| **L2** per-layer | decoder/vision seams | cosine ledger | **L2** Seeded assemble + shuffle remap | same seed → same exam; choice shuffle remaps key (metamorphic) | EXACT |
| **L3** logits | pre-sampling logits + argmax | measured + exact argmax | **L3** Full GradeReport golden | full report: scores, pass/fail vs form, domain tallies, digests | EXACT golden digest |
| **L4** tokens | decoded token ids (greedy prefix) | exact-prefix | **L4** Rust == WASM | native digest == wasm digest for frozen attempts | EXACT (floor 0) |
| **L5** e2e OCR | text/bbox + CER budget | exact-where-det + budget | **L5** UI e2e digest match | browser/UI path produces same digest as oracle; honesty banner; form 40/60/27 | EXACT digest + honesty gates |

**Short-circuit rule (from OCR integration runner):** failed lower gate makes higher
gates meaningless — report L0 red, do not claim “L5 green with caveats.”

**Determinism keystone (OCR §2 → simplified):** OCR measures the *oracle’s own*
nondeterminism floor before setting L3/L4 tolerances. For grade, the oracle floor is
**0 by construction** (integers, sorted keys, no timestamps in digest body). The
remaining keystone is **double-run self-consistency**: same input twice → identical
digest (native and wasm separately, then cross-path).

---

## Golden artifacts + UPDATE_GOLDENS discipline

From `franken_ocr/docs/conformance/GOLDEN.md` (mandatory pattern-per-artifact):

| Pattern | OCR use | **cdcp use** |
|---------|---------|--------------|
| **exact** | CLI help, robot schema, structured JSON | `GradeReport` golden, CLI help/schema, exam form pins |
| **fuzzy / ULP** | logits / activations | **Do not import** for GradeExact (no floats in score-of-record) |
| **scrubbed** | robot NDJSON timings | robot events: scrub `elapsed_ms`, `run_id`, paths; keep field *presence* |
| **canonicalized** | cross-platform paths/line endings | `canonical_json`: sorted keys, `\n`, integer scores only |

**UPDATE_GOLDENS rules (non-negotiable, copy verbatim intent):**

1. Default `cargo test` / `check.sh` **never** rewrites goldens — mismatch writes
   `*.actual` / `*.snap.new` and **fails**.
2. Only `UPDATE_GOLDENS=1` (local, human) rewrites committed goldens.
3. **CI NEVER sets `UPDATE_GOLDENS` / `INSTA_UPDATE`.** Suite asserts env free of
   update flags in CI profile.
4. PR must show `git diff` of golden change; reviewer judges intent.
5. Provenance: every golden names `bank_hash`, exam_id, seed, engine version, and
   command that produced it. Stale provenance → incomplete golden.
6. `*.actual` / `*.snap.new` gitignored.

**Target artifact for L3:** e.g. `goldens/mock40_seed42.sha256` (+ sibling JSON
report) — already anticipated by local `scripts/check.sh` wave status.

---

## Two contracts Exact vs Quality → GradeExact vs PedagogySignal

### franken_tts (§9 / CONFORMANCE_AND_LISTENING)

| Contract | Question | Gate character |
|----------|----------|----------------|
| **ConformanceExact** | Does the implementation match the reference under canonical greedy / teacher-forced seams? | Exact discrete streams; ladder L0–L5 for kernels |
| **ProductionQuality** | Does the *shipping* sampler + quant stay within perceptual/WER budgets? | Distributional metrics + powered listening; **cannot loosen Exact** |

Doctrine: **one ladder cannot serve two masters** — sampling quality must not invent
tolerances that paper over Exact failures.

### cdcp mapping

| Contract | Question | May it change a score? |
|----------|----------|------------------------|
| **GradeExact** | Unique lawful score for `(bank_hash, exam_id, seed, answers)`? | **This is the score-of-record** |
| **PedagogySignal** | Coverage, weak modules, explanations, SRS priority? | **Never** — advisory only |

**Invariant:** PedagogySignal consumers (learn/drill UI, weak-topic banners) read
GradeExact outputs; they must not re-score. A UI that “helps” by altering pass/fail
is a dual-path lie.

---

## check.sh single gate + skip-honest

### OCR / TTS pattern

- **One script** is the gate: CI invokes `scripts/check.sh` as the single test step
  (no duplicated workflow command lists that drift).
- Stages cheapest-first; **stop on first failure**.
- **Skip-honest:** every stage receipts `PASS | FAIL | SKIP(reason)`. Closing banner
  is `GREEN` or **`GREEN WITH SKIPS`** listing skips — never fold SKIP into silent pass
  (TTS Doctrine #0.4; OCR model-gated e2e: `skip-with-SUCCESS` with structured log line
  `result=skip_no_model`, and prove native path via `/nonexistent` fallback when armed).

### Local cdcp today

`scripts/check.sh` is already the single gate for W0 knowledge scaffold (constitution
docs, knowledge pack, exam_form 40/3600/27, honesty smoke, crosswalk). Wave status
explicitly reports GradeExact/WASM **OPEN** rather than counterfeit product-green.

### Target shape as L3–L5 land

```text
check.sh
  → knowledge / bank schema (always-on)
  → claims-lint / honesty (always-on)
  → unit grade L1–L2 (always-on)
  → golden GradeReport L3 (always-on once goldens committed)
  → wasm dual-path L4 (SKIP if wasm toolchain absent — listed in banner)
  → UI e2e L5 (SKIP if browser harness unarmed — listed; never silent)
  → known-bad fixture suite (always-on; must RED on fixtures)
```

Empty bank / missing golden set = **ERROR**, not vacuous pass (anti-vacuous discipline
from FrankenSuite L4 / check.sh empty-scan ethic).

---

## Known-bad / gates that must trip

**Principle (OCR METHODOLOGY anti-patterns + frankensim “gates proven to trip”):**
green means gates **demonstrably trip** on known-bad fixtures. A never-armed dual-path
test is not dual-path coverage.

| Known-bad fixture | Must | Maps from |
|-------------------|------|-----------|
| Empty bank | ERROR (not 0/0 pass) | OCR empty-scan / vacuous green ban |
| Orphan item (ref without body / dangling topic) | ERROR | bank integrity |
| Flipped golden (deliberate digest flip) | RED on L3 compare | golden suite self-test |
| Dual-path lie (subject_identity == oracle_identity) | harness abort | EngineIdentity guard |
| “You are certified” / EPI-official pass language | honesty fail | surface pillar |
| Vacuous full-coverage (all domains green with zero items) | ERROR | anti-vacuous |
| Shuffle without key remap | grade mismatch vs unshuffled key | metamorphic L2 |
| Seeded assemble nondeterminism | double-run digest diverge | determinism gate |

Ship a **ci-self-test** (or `check.sh --self-test-known-bad`) that injects each fixture
and asserts exit ≠ 0 / assertion fire. Without that, the checklist is ceremony
(Doctrine #0.1 fail).

---

## What we DO NOT import (ML-only)

Explicit **non-import** list so agents do not cargo-cult ML-System machinery into an
Assessment-System product:

| ML-System artifact | Why not |
|--------------------|---------|
| Per-op **ULP** tables / cosine ≥ 0.9999 | Grade has no continuous tensors in score-of-record |
| **CER / TEDS / Formula-CDM** budgets | OCR end-to-end metrics; not exam scoring |
| PyO3 / CUDA torch **oracle bridge** | Oracle is our own Rust grade crate |
| Oracle **nondeterminism floor** derivation for tolerances | Floor is 0; double-run only |
| int8/int4 **quant** kill-switches as score path | N/A |
| Conformal lower-bound **release score** on mean CER | Overkill; Exact digests replace it |
| “Listening protocol” / MUSHRA / ABX | TTS ProductionQuality only |
| Guessed epsilon “close enough” on scores | Forbidden — integers only |
| Auto-`UPDATE_GOLDENS` in CI | Same ban, but restated for vigilance |
| LLM-as-grade-of-record | [`TESTING.md`](../TESTING.md) anti-pattern |

**What we *do* import (portable rigor):** EngineIdentity, ladder short-circuit, golden
pattern discipline, skip-honest receipts, single `check.sh`, G1>G2, Doctrine #0,
known-bad self-tests, dual-path native/WASM parity (from franken_markdown proof gates),
determinism pins (`SOURCE_DATE_EPOCH`-style: fixed seed/epoch for any rendered artifact
that might otherwise timestamp).

### franken_markdown extras (Assessment UI / pack render)

- **WASM native parity:** same core, byte-identical outputs native vs wasm over a
  corpus; package gate proves real `.wasm`, not skeleton marketing.
- **`SOURCE_DATE_EPOCH`:** deterministic PDF/HTML metadata when learner materials are
  exported; double-run `cmp` gates.
- **Self-contained HTML:** inlined CSS/fonts — **no CDN hand-waved stylesheets** for
  ship surfaces (demo may be local ESM only). Hub/mock UI assets stay first-party or
  vendored; claim-discipline if README says “ships” before artifacts exist.

---

## Concrete checklist for cdcp-course L3–L5

Use as the **value bar** for product ticks. Each box is a gate branch, not a doc todo.

### L3 — Full GradeReport golden

- [ ] `cdcp_grade` crate exists; `#![forbid(unsafe_code)]`; integers-only scores.
- [ ] `canonical_json(GradeReport)` + `sha256` helper; volatile fields excluded from
      digest body (no wall-clock).
- [ ] Frozen attempt fixtures: `(bank_hash, exam_id, seed, answers[])` committed.
- [ ] Golden digest file(s) under `goldens/` with PROVENANCE (command, bank_hash, seed).
- [ ] Double-run native: identical digest.
- [ ] `UPDATE_GOLDENS=1` local-only path; CI assert env clean; `*.actual` gitignored.
- [ ] Known-bad: flipped golden RED; empty bank ERROR.
- [ ] `check.sh` runs L3 always-on once goldens land (wave status line retires).

### L4 — Rust == WASM dual-path

- [ ] Same crate builds `wasm32-unknown-unknown` (or documented wasm target).
- [ ] EngineIdentity labels distinct; comparator aborts on self-compare.
- [ ] Frozen attempts: `digest_native == digest_wasm` for every committed fixture.
- [ ] Skip-honest if wasm toolchain missing: banner `GREEN WITH SKIPS`, not silent.
- [ ] No mock dual-path (JS reimplementation that only copies expected digests).
- [ ] franken_markdown-style proof: real wasm artifact size/hash in gate, not README claim.

### L5 — UI e2e digest match

- [ ] Mock exam UI submits answers through WASM grade (or documented bridge that still
      invokes the wasm/native engine — not a parallel scorer).
- [ ] UI-displayed score/pass matches oracle digest fields for the attempt.
- [ ] Honesty: no “certified / officially EPI” success chrome; form pins 40 items /
      60 min / 27 correct remain visible and enforced.
- [ ] e2e harness digests the attempt payload the UI would send; compares to golden.
- [ ] Known-bad: dual-path lie + certified string fixtures trip CI.

### Cross-cutting (all L3–L5)

- [ ] G1>G2: no UI merge that changes GradeExact without golden update + review.
- [ ] PedagogySignal features (weak modules, explanations) cannot alter digests.
- [ ] Doctrine #0: no new process doc unless a gate branches on it.
- [ ] `check.sh` remains the single CI step; stages receipt PASS/FAIL/SKIP.
- [ ] Known-bad self-test suite proves gates trip (L4 artifact rigor).

---

## Three pillars (OCR) → three pillars (cdcp)

| OCR pillar | cdcp pillar | Evidence |
|------------|-------------|----------|
| (a) Performance | (a) Integrity / conformance | Ladder L0–L5 digests, dual-path |
| (b) Conformance | *(folded into a for Assessment)* | — |
| (c) Surface parity | (b) Surface honesty | No credential inflation; FEATURE_SURFACE status honest |
| — | (c) Pedagogy + standards family map | Crosswalk, coverage matrices — **not** score-of-record |

No victory on Hub UI (b/c) while GradeExact (a) is red.

---

## Related local docs

- [`../ORACLE-GAUNTLET.md`](../ORACLE-GAUNTLET.md) — stamp-in-stone class + ladder
- [`../TESTING.md`](../TESTING.md) — per-layer epistemology
- [`../FEATURE_SURFACE.md`](../FEATURE_SURFACE.md) — present/missing surfaces
- [`../NEGATIVE_EVIDENCE.md`](../NEGATIVE_EVIDENCE.md) — rejected levers
- [`../OQ_REGISTER.md`](../OQ_REGISTER.md) — no ship on unresolved OQ

---

*Extract only. Does not grant EPI certification claims. Does not contain exam dumps.*
