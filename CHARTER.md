# CHARTER — CDCP Interactive Course Engine

**Status:** ACTIVE — **W0–L7 + V11 + M8 Learn v2 + M9 GREEN** (2026-08-12) · repo public at github.com/JYeswak/cdcp-self-study  
**Product codename:** `cdcp-course` (engine at `cdcp-self-study/course-engine/`)  
**Operator:** Joshua Nowak  
**Date:** 2026-08-12 (constitution refresh; edit in place)

**Locked:** engine inside self-study · grade in browser (WASM dual-path) · no dumps ·
standards citation graph (not pirated SDO full text) · franken_ocr visual DNA ·
free/public corpus in-repo (OQ-09) · paid SDO spend deferred (OQ-10) · public target via L88

**Constitution:** `course-engine/docs/ORACLE-GAUNTLET.md` · `STANDARDS-KB.md` · `TESTING.md` ·
`VISUAL.md` · `LEARN-PEDAGOGY.md` · `DIAGRAM-REGISTRY.md` · `PUBLICIZE-PROCESS.md` ·
`loop3/PROTOCOL.md` · `OQ_REGISTER.md`

> One charter. Defects fixed in place. Not a deliverable by itself — the product is a
> runnable course + grader you can trust. Completing this program does **not** grant
> EPI®/EXIN CDCP® certification. [[claim:claim-not-epi-certified]]

---

## 0. One-sentence product

A **local-first, HTML course + practice-test engine** for data-centre professional
substance (public CDCP domain map), with a **memory-safe Rust core** that grades exams
deterministically and proves integrity with **byte-exact golden oracles** — so “I studied”
and “the software scored me” are both machine-checkable.

---

## 1. Buyer and “shipped”

| | |
|---|---|
| **Buyer (loop #3)** | Joshua: interview-ready DC fluency (Fluidstack / facilities hybrid seats); secondary: future self on cold recall [[claim:claim-interview-ready]] |
| **Product surface** | Browser UI (static HTML/CSS/JS + WASM) for learn + drill + timed mock; CLI for bank verify, export, goldens, serve |
| **Shipped means (2026-08-12)** | (1) Open offline hub → Learn 15 modules (14 EPI domains + ops-adjacent) → module quiz / Learn-15 / Drill-SRS / Mock-40 → score + explanations + weak modules; (2) `./scripts/check.sh` green (W0–L7 + V11); (3) fixed seed exam → **byte-identical** grade digest vs golden (native==WASM); (4) Loop#3 hybrid protocol + ≥1 T1 run log; (5) free/public corpus store policy |
| **Not shipped yet** | Public GitHub remote (L88 bar quality PASS; flip is Josh); P1 diagrams beyond P0 set; KaTeX full (offline subset shipped) |
| **Not shipped ever (forbidden)** | Pretty markdown only as “done”; AI essay grade-of-record; official EPI affiliation; claim of being CDCP certified |

**RULE ZERO:** A tick counts only if the **product surface** changed (UI path, grader behavior, bank integrity, wired gate) or a **constitution gate** newly trips on known-bad. Charter-only edits are not product ticks; they wire truth so product work can ship.

**Loop #3:** External-to-`check.sh` signal per `course-engine/docs/loop3/PROTOCOL.md` (hybrid T1 self-log floor; T2 peer/interview upgrade). First log: `docs/loop3/runs/2026-08-12.md`.

---

## 2. Honesty constitution (non-negotiable)

1. **Not EPI / EXIN.** No trademark claim of certification; no copied proprietary slides/exam dumps. [[claim:claim-not-epi-certified]]
2. **Original item bank only.** Every MCQ is original or clearly attributed public knowledge check. [[claim:claim-forbidden-dump-bank]]
3. **Strength of claim lattice** (see §5): weaker claims never justify stronger ones.
   - `interview_ready` ≠ `epi_certified` [[claim:claim-interview-ready]]
   - `domain_covered` ≠ `exam_pass_probability` [[claim:claim-domain-covered]]
4. **Sources for domain map only:** public EPI marketing syllabus headings / authorized partner outlines — never proprietary body text.
5. **Human error is data:** wrong answers drive spaced drills; no shame theater.
6. **Paid SDO full text:** not in repo until spend decision reopens OQ-10; free/public PDFs and page snapshots **are** stored (OQ-09 VERIFIED). [[claim:claim-forbidden-iec-clause-without-license]]

---

## 3. What we are building (scope)

### Shipped (v1 engine — do not re-plan as greenfield)

| Surface | Behavior |
|---------|----------|
| **Learn** | **15 modules** from notes (14 public EPI domains + 1 ops-adjacent supplement); offline reader; visit progress; Learn-15 + module quiz. **Resolved 2026-08-15 — the assessed/taught gap is closed.** The bank's 39 `15-ops-adjacent` items were **taught, not excluded**: `modules/15-ops-adjacent.md` is a full Learn surface covering all six assessed topics, grounded in freely redistributable primary sources (29 CFR, 10 CFR, DOE, NASA, NIST, FEMA, HSE, public post-incident reports). Enforced by `crates/cdcp_assemble/tests/learn_surface_coverage.rs`: every module holding an approved bank item must carry a navigable Learn surface, anti-vacuous. Module 15 stays `exam_weight_unknown = true` — it is **not** an EPI syllabus domain. |
| **Drill / short-interval review** | Missed-item review; **1-day/3-day ladder capped at 3 days**; Drill-10 due mode. **Not spaced repetition** — no expanding interval, no forgetting curve, no scheduler state beyond the cap. Calling it SRS overstates it (`web/assets/js/srs.js`). |
| **Practice test** | Module quizzes + full 40-item mock; soft timer; closed-notes mode |
| **Feedback** | Correct/incorrect + explanation + module/section links |
| **Mastery path** | Practiced ≥80% · mastered 90%×2 ≥24h; hub recommend next |
| **CLI** | `check` · `bank-hash` · `grade` · `goldens` · `export-web` · `serve` · bank verify |
| **Integrity** | Content-addressed bank; goldens; WASM dual-path; content.lock; claims constitution |
| **Stretch shipped (V11)** | Anki export · power-path N vs 2N diagram · axum serve · runbook vignettes |

### In scope next — **M8 Learn v2** (pedagogy packaging)

Codified in `course-engine/docs/LEARN-PEDAGOGY.md` and `DIAGRAM-REGISTRY.md`:

| Surface | Behavior |
|---------|----------|
| **Lesson units** | 5–8 min chunks from existing MD (not 4k-word walls) |
| **Chrome** | Sticky TOC · reading progress · Continue chip · time estimates |
| **Micro-checks** | 2–3 bank items mid-unit via `topic_ids` (instant feedback) |
| **Diagram system** | Interactive SVG registry (franken visual DNA); embed in modules |
| **Formulas** | Rendered math (not raw LaTeX) |
| **Miss coach** | Concept card (definition + diagram + similar Q) on miss |

### Explicit non-goals

- Official cert prep dump / brain-dump marketplace  
- Multi-tenant cloud SaaS / accounts / payments  
- Video LMS / SCORM / React-Next mandatory SPA  
- Free-form essay auto-grading by LLM as authority  
- XP / streaks that imply certification  
- Replacing authorized EPI training if Joshua later wants the credential  

---

## 4. Pedagogy — emulate the best, not the average LMS

Borrow mechanisms, not skins. Browser audit 2026-08-12: content is strong; packaging is still handbook-like (wall of prose, 0 images in modules, one orphan interactive diagram).

| Inspiration | Mechanism we copy | Product mapping | Status |
|-------------|-------------------|-----------------|--------|
| **Khan Academy** | Mastery on weak skills | practiced / mastered | **shipped** |
| **Anki** | Spaced misses; atomic cards | SRS + Anki export | **shipped** |
| **LeetCode** | Timed mock; retry wrong set | Mock-40 + miss review | **shipped** |
| **Coursera** | Module → unit path → quiz gate | Learn v2 units before capstone | **shipped M8** |
| **LinkedIn Learning** | Continue + short chapters | Sticky continue + TOC | **shipped M8** |
| **Duolingo** | Micro-step + instant check | Mid-unit micro-checks | **shipped M8** |
| **Brilliant** | Worked example → try it | Interview drills as UI cards | **partial** (concept cards + drills) |

**Content layers (must stay honest about which exist):**

```text
L1  Knowledge graph     domains · topics · claims · bank          SHIPPED
L2  Lesson narrative    15 module markdown files (14 EPI + ops)   SHIPPED
L3  Teaching atoms      units · diagrams · micro-checks           SHIPPED M8
L4  Practice atoms      quiz · drill · mock · SRS                 SHIPPED
L5  Progress model      visited · practiced · mastered            SHIPPED (basic)
```

**Session shapes (product must support):**

1. **Learn-15** — module-scoped 5Q · **shipped**  
2. **Drill-10** — SRS due only · **shipped**  
3. **Quiz-module** — 8–12 items · **shipped**  
4. **Mock-60** — 40 items / 60 minutes · **shipped**  
5. **Miss-review** — incorrect from last attempt · **shipped**  
6. **Unit-8** (M8) — one teaching atom + micro-check · **shipped**

**Pass bars (study signals, not credentials):** [[claim:claim-study-signal-27]]

| Mode | Bar |
|------|-----|
| Module quiz | 80% → “practiced” |
| Full mock | 27/40 (67.5%) “interview study signal” |
| Mastery tag | 90% on two spaced attempts ≥24h apart |

---

## 5. Artifact rigor (layers that apply)

| Layer | Applies? | How |
|-------|----------|-----|
| **L1 Claims constitution** | **YES · wired** | `registries/*.toml` + `cdcp_registry_check` in check.sh |
| **L2 SLO-as-code** | **YES · wired** | `slo.toml` + `smoke_slo.sh` |
| **L3 External oracle (factual content)** | **NO** | See §5a. No oracle external to this project validates whether bank items are factually correct about data centres. |
| **L3a Cross-target conformance** | **YES · wired** | Native `cdcp_grade` vs the same source compiled to wasm32; asserts equal digests. A self-consistency invariant, **not** external validation. |
| **L3b Content-integrity oracles** | **YES · wired** | Domain coverage vs our own `topics.toml`; grade goldens generated by our own grader. Both internal by construction. |
| **L4 Gates proven to trip** | **YES · wired** | known-bad / L5 honesty / L6 coverage / L7 objectives selftests |
| **L5 Adversarial floor** | **YES · wired** | property/fuzz floors present |
| **L6 Formal** | NO (v1) | — |
| **L7 Ecosystem lock** | **YES · wired** | `content.lock` |
| **Learn surface gates** | **SHIPPED M8** | smoke_learn_v2 · smoke_diagrams · build_units in check.sh |

**Gauntlet tier:** **T2 educational software** (deterministic grader + content integrity).

**Jeff-style patterns:** byte-exact goldens · named oracles · checker has tests · one `check.sh` · never vacuously green.

---

## 6. Architecture (how)

```text
cdcp-self-study/
  CHARTER.md                    # this file
  docs/CHARTER-PROCESS.md
  modules/ practice/ reference/ # educational corpus
  course-engine/                # product (in-tree)
    registries/ bank/ crates/ web/ goldens/ scripts/
    docs/                       # ORACLE · LEARN-PEDAGOGY · DIAGRAM-REGISTRY · …
```

### Grading contract (invariant) [[claim:claim-grade-byte-exact]]

```text
grade(bank_hash, exam_id, seed, answers[]) -> GradeReport
```

Same inputs ⇒ **byte-identical** canonical JSON. LLM is **not** in this function.  
Reference = Rust `cdcp_grade`; subject = the **same source** compiled to wasm32; comparator
asserts equal digests.

**This is cross-target conformance, not an external oracle.** Both sides are the same
implementation compiled twice, so a factually wrong answer key produces the same wrong answer
on both paths and every digest agrees. The check catches build/codegen divergence — genuinely
valuable — and proves nothing about content truth.

### 5a. What L3 actually requires, and why we do not have it

An external oracle is a reference implementation, RFC, spec suite, or real-world corpus
**the project does not control**. Measured 2026-08-14, none of our three claimed oracles
qualifies:

| Claimed oracle | Compares against | External? |
|---|---|---|
| Domain coverage | our own `topics.toml` registry | no |
| Grade goldens | output of the grader that produced them | no |
| Dual-path | the same source compiled twice | no |

**Consequence, stated plainly:** the repo proves the grader is *deterministic*. It does not
prove the 804 bank items are *correct about data centres*. A wrong answer key is invisible to
every step of `check.sh`. For a study tool whose value proposition is that the learner ends up
correct, the unguarded property is the one that matters most.

**Route to a real L3** (`docs/ROADMAP-PHASES.md` P3–P4): public-domain datasets — NOAA climate,
USGS seismic, FEMA flood, EPA eGRID, EIA — are authoritative about physical reality, versioned,
and outside our control. Computing a quantity from them and comparing against published
reference values is falsifiable in the way an oracle must be.

### Substrate Law

- Hot path: **Rust**, `#![forbid(unsafe_code)]`.  
- UI: static HTML/CSS/vanilla JS + WASM — **no React/Next/Tailwind** for learner app.  
- Diagrams: interactive SVG + small JS (franken visual DNA); not a SPA framework.  
- Python: content build / smokes with stated permanence or migration path.

---

## 7. Content model (depth)

| Mode | Hours | Default |
|------|-------|---------|
| **Interview-ready (standard)** | ~28–40 h | **YES** |
| Exam-format drill | mock 40×60m | YES |
| Credential path | authorized EPI course | Link only |

**Coverage:** product objectives in `registries/objectives.toml` + domain floor modules 1–14; full LO×item matrix is **aspirational** (documented gap in `verify_objectives.py`). [[claim:claim-domain-covered]]

**Corpus:** free/public under `knowledge/corpus/{public,free-pdfs}/` (OQ-09); no paid full-text (OQ-10 ASSUMED defer).

---

## 8. Value bar (what makes a tick GREEN)

1. **Product path** changed, or a **wired** gate newly trips on known-bad.  
2. **`scripts/check.sh` green** (or declared subset).  
3. **No honesty violation.**  
4. **Bead/note** records growth.  

RED: markdown without import; unlinked AI items; goldens mutated without root cause; claiming Learn v2 shipped before gates exist.

---

## 9. Milestones

| ID | Milestone | Status |
|----|-----------|--------|
| **M0–M7** | Charter · registries · bank · grade/goldens · web mock · learn · SRS · polish · WASM | **DONE** (maps W0–L7) |
| **V11** | Anki · power-path diagram · serve · runbooks | **DONE** |
| **M8** | **Learn v2** — units · TOC · micro-checks · diagram system · formulas | **GREEN** |
| **M9** | Publicize — L88 bar ≥5/7 · OSS meta · then visibility flip | **DONE** |
| **M10** | Free/public corpus expansion (OQ-09) | **GREEN** (4 free PDFs) |

Ambition: M8 makes Learn feel like Coursera/Duolingo-class **packaging** without losing offline/static integrity.

---

## 10. Irreversible / external actions

| Action | Gate |
|--------|------|
| Publishing as “CDCP certified prep” | **Forbidden** |
| Public GitHub without honesty + L88 bar | Escalation — see `PUBLICIZE-PROCESS.md` |
| Paid SDO full-text purchase | Escalation — OQ-10 reopen |
| Force-push / delete goldens history | Escalation |

Local builds, bank, UI, tests, free corpus fetch: **autonomous**.

---

## 11. Decisions (resolved)

| # | Decision | Resolution |
|---|----------|------------|
| 1 | Repo shape | **In-tree** `cdcp-self-study/course-engine/` |
| 2 | Grade locus | **Rust oracle + WASM dual-path** in browser |
| 3 | Public publish | **Yes, via L88 publicize process** (not ad-hoc) |
| 4 | OQ-09 ASHRAE/free PDFs | **Store free/public in-repo** (VERIFIED) |
| 5 | OQ-10 paid SDO | **Defer spend**; public-first research (ASSUMED) |
| 6 | Loop #3 | **Hybrid** protocol; T1 log required (filed 2026-08-12) |
| 7 | Credential track | Self-study until interview signal; official CDCP separate |
| 8 | **Module 15 — assessed but untaught** (`bd-hardening-c-status-hzs.4`, 2026-08-15) | **TEACH, not exclude.** `modules/15-ops-adjacent.md` now teaches all six assessed topics. Reasoning: (a) both options close the fairness defect, but only teaching also *improves* the product — the buyer's target seat (facilities hybrid) probes operations discipline harder than any other body of knowledge, so excluding would have made the exam fair by deleting the material the buyer is hired for; (b) the cost was bounded, not open-ended — research lineage L9 had already read 20 primary documents and emitted a 28-unit curriculum with a named free-to-redistribute source per unit, so this was curation, not original research; (c) C1's `status` field made *exclude* cheap, which is precisely why the decision had to be argued on product value rather than on implementation cost. Module 15 remains `exam_weight_unknown` and is labelled in-page as **not** one of the 14 public EPI domains. Enforced by `learn_surface_coverage.rs`, which also keeps the exclude branch lawful for any future module. |

Ledger: `course-engine/docs/OQ_REGISTER.md` · HUMAN beads closed under epic `bd-1z2`.

---

## 12. Workflow entry

Primary: **`cdcp-course-build`** (optional Grok workflow).

Phases now:

1. Charter-lint / claims-lint  
2. Bank + grade + goldens  
3. Web shell + WASM  
4. Pedagogy (M8)  
5. Publicize (M9) when bar green  
6. `check.sh` ship  

Process detail: [`docs/CHARTER-PROCESS.md`](./docs/CHARTER-PROCESS.md).

---

## 13. Sign-off

```text
shipped_means: offline hub + 15-module Learn (14 EPI domains + ops-adjacent) + drill/short-interval-review + mock40 + WASM cross-target-conformance goldens + check.sh W0–L7/V11
next: P1 diagrams (bd-1sd.9.1); OQ-09/10 corpus research remains open (M10 itself is DONE)
non_goals: official EPI cert, LLM-as-grader-of-record, SaaS, React LMS
layers: L1 L2 L3 L4 L5 L7 wired; L6 no; Learn-surface gates M8 wired
oracle: public 14-domain map (coverage) + frozen grade reports (scoring)
gauntlet_tier: T2 educational software
oq: 09 VERIFIED store free/public; 10 ASSUMED defer spend
loop3: hybrid T1+; protocol + first run logged
public: target yes via L88
```

**Joshua ACK to start M8 product work:** _________________ date ________

Docs constitution refresh (this file) is autonomous; large M8 fan-out may wait for ACK if preferred.
