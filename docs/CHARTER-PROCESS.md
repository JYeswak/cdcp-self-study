# CHARTER PROCESS — Educational product track (`/charter`)

**Purpose:** A repeatable process to define *what* we build, *how*, *rigor*, and *depth* before agents fan out into HTML, Rust, and content. First product on this track: **CDCP Interactive Course Engine** ([`../CHARTER.md`](../CHARTER.md)).

This is the **meta-loop**. The product charter is the **instance**.

---

## 1. When to run `/charter`

Run (or re-run) when:

- Starting a new educational or assessment product  
- Changing “shipped means,” claim strength, or oracle  
- Adding a surface that could invent false credentials (cert language)  
- After a rigor near-miss (vacuous green, LLM grading creep)

Do **not** mint a successor charter file. Edit in place; bump `Status` / date.

---

## 2. Inputs (must exist before drafting)

| Input | CDCP instance |
|-------|----------------|
| Learner / buyer | Joshua — interview-ready DC fluency |
| Domain corpus | `cdcp-self-study` modules + practice (saved ✓) |
| Public map / oracle candidate | 14-domain public CDCP syllabus headings |
| Forbidden claims | EPI/EXIN certification identity |
| Analog rigor systems | franken_ocr / franken_tts goldens; frankengraphdb registries; Artifact Rigor L1–L7 |

---

## 3. Process phases (human + agent)

```text
P0 SURVEY     → what corpus already exists; honesty constraints
P1 BUYER      → who uses it; what “done” looks like in one sentence
P2 PRODUCT    → surfaces (learn / drill / exam / CLI); non-goals
P3 DEPTH      → hours, bloom levels, pass bars (study signals only)
P4 RIGOR      → L1–L7 selection; named oracles; gauntlet tier
P5 ARCH       → Rust core boundaries; UI tech; dual-grader rule
P6 MILESTONES → M0…Mn with gates that can go RED
P7 PROCESS    → workflow name + phase list for build orchestration
P8 FREEZE     → Joshua ACK on open decisions; then M1 may start
P9 REFRESH    → after ship: edit charter in place (status, shipped_means, next M#)
              → sync FEATURE_SURFACE · ORACLE · PHASE-NEXT · LEARN-PEDAGOGY
              → validate: check.sh + claims-lint + doc consistency checklist
```

**Post-L7/V11 (2026-08-12):** P8 decisions resolved (repo in-tree, WASM dual-path, public via L88,
OQ-09/10, Loop#3 hybrid). Next product milestone **M8 Learn v2** is pedagogy packaging —
see `course-engine/docs/LEARN-PEDAGOGY.md`. Publicize is **M9** (`PUBLICIZE-PROCESS.md`), not a
charter fork.

### Phase exit criteria

| Phase | Exit when |
|-------|-----------|
| P0 | Inventory path listed; “not official cert” stated |
| P1 | `shipped_means` is observable by a human in &lt;15 min |
| P2 | In-scope table + non-goals table complete |
| P3 | Depth mode + pass bars numeric |
| P4 | Every applied layer has a **named** mechanism; empty scan = ERROR |
| P5 | Grading contract one-liner; `#![forbid(unsafe_code)]` on core |
| P6 | Each milestone has a gate command or manual smoke |
| P7 | Workflow name reserved; no agent work before P8 if design still open |
| P8 | Open decisions resolved or explicitly deferred with owner |

---

## 4. Required charter sections (lint checklist)

A product charter is incomplete if any are missing:

- [ ] One-sentence product  
- [ ] Buyer + shipped_means + not-shipped  
- [ ] Honesty / trademark / claim lattice  
- [ ] In-scope / non-goals  
- [ ] Pedagogy mechanisms (borrowed from named products)  
- [ ] L1–L7 table (apply or explicit NO)  
- [ ] Named external oracle(s)  
- [ ] Architecture sketch + grading contract  
- [ ] Content model / depth  
- [ ] Value bar (GREEN/RED)  
- [ ] Milestones with gates  
- [ ] Irreversible actions  
- [ ] Open decisions for Joshua  
- [ ] Workflow entry name  

**Charter-lint (future automation):** markdown section headers or YAML frontmatter; for now, human checklist + agent review.

---

## 5. Rigor defaults for *any* quiz/course product

Copy-paste defaults (override only with reason):

1. **Grader of record is deterministic code**, never an LLM.  
2. **Byte-exact goldens** for at least one frozen full exam.  
3. **Claims lattice** forbids credential inflation.  
4. **Coverage oracle** separate from **score oracle**.  
5. **Known-bad fixtures** must trip CI (wrong key, empty bank, orphan item).  
6. **Dual implementation** (e.g. Rust + JS) ⇒ differential tests or single export path.  
7. **`check.sh` is the only CI story** — no duplicate ad-hoc gates that can diverge.

### Byte-exact oracle recipe (franken_* style)

```text
1. Define GradeReport schema (serde).
2. canonical_json(report) → stable bytes (sorted keys, no insignificant whitespace variance).
3. sha256(canonical_json) stored in goldens/<fixture>.sha256
4. cdcp goldens check regrades fixture attempts and compares digest.
5. UPDATE_GOLDENS=1 only with explicit human reason in commit body.
6. Mutation test: flip one correct answer in bank → golden must fail (L4).
```

---

## 6. Depth scale (shared vocabulary)

| Depth | Meaning | Assessment |
|-------|---------|------------|
| **D1 Literacy** | Terms + “what is this for” | Remember/understand MCQ |
| **D2 Interview** | Trade-offs, walk a white space | Apply/analyze MCQ + oral anchors |
| **D3 Floor** | Procedures, MOP awareness | Scenario multi-step (later) |
| **D4 Credential** | Official body exam | Out of band (authorized provider) |

CDCP v1 product targets **D1–D2**. D3 vignettes are stretch. D4 never claimed by this software.

---

## 7. Workflow contract (`/workflow`)

After P8 ACK, build via named workflow (not ad-hoc agent sprawl):

| Workflow | Job |
|----------|-----|
| `cdcp-charter-lint` | Verify CHARTER sections + honesty strings present |
| `cdcp-course-build` | M1→M3 fan-out: bank · grader · goldens · web shell · adversarial verify |

**Rules for agents in the workflow:**

- Read CHARTER + this process first  
- No inventing EPI exam questions  
- Every new item links `objective_ids`  
- Do not update goldens silently  
- Prefer expanding `cdcp-self-study` corpus only when Learn surface needs it  

Workflow scripts live in `~/.grok/workflows/` or project `.grok/workflows/` once authored.

---

## 8. Relationship to existing CDCP files

| Path | Role after charter |
|------|--------------------|
| `README.md` | Human study entry (honesty preserved) |
| `modules/*` | Content pack |
| `practice/*` | Seed for bank import |
| `CHARTER.md` | Product constitution for the **engine** |
| `docs/CHARTER-PROCESS.md` | This process |
| Future `course-engine/` or `cdcp-course` | Rust + web implementation |

Markdown self-study remains valid **without** the engine. The engine makes practice tests **operational and integrity-checked**.

---

## 9. Anti-patterns (measured elsewhere; forbid here)

- Gate-on-gate with no learner-visible product  
- “AI tutor” as sole scorer  
- Vacuous green (0 questions = pass)  
- Successor charters (`CHARTER-v2.md`) instead of edit-in-place  
- Claiming Fluidstack interview readiness as a **machine invariant** (at most a study signal)

---

## 10. Minimal session script (operator)

```text
1. Confirm corpus: ls cdcp-self-study && head README honesty
2. Open CHARTER.md — edit P1–P6 until no open contradictions
3. Resolve §11 open decisions or defer with date
4. Joshua ACK → M1
5. /workflow cdcp-course-build (or stepwise M1 manually)
6. Stop when M3 smoke: full mock graded + golden check green
```

---

## 11. Status for CDCP instance

| Step | Status |
|------|--------|
| Corpus saved | **DONE** (`/Users/josh/cdcp-self-study`) |
| Product CHARTER drafted | **DONE** (`CHARTER.md`) |
| Process documented | **DONE** (this file) |
| Joshua ACK (P8) | **PENDING** |
| Workflow authored | **NEXT** |
| M1 bank import | Blocked on ACK (or explicit “build anyway”) |
