# LEARN PEDAGOGY — teaching atoms & edtech map

**Status:** doctrine (2026-08-12) · product Waves A–D **planned** · not false-shipped  
**Charter:** parent `CHARTER.md` §4 · M8 Learn v2  
**Honesty:** study signals only — never EPI/EXIN certification [[claim:claim-not-epi-certified]] [[claim:claim-study-signal-27]]

---

## 1. Problem (browser audit 2026-08-12)

| Observation | Detail |
|-------------|--------|
| Content quality | High — interview-ready LO → why → concepts → drills [[claim:claim-interview-ready]] |
| Size | ~3.5–4.3k words **per** module (~55k+ words total) |
| Images | **0** in module markdown |
| Diagrams | ASCII in some modules; **one** interactive (`power-path`) not embedded in M06 |
| Math | Raw LaTeX `\[…\]` not rendered |
| Practice | Strong end-of-module quiz / Learn-15 / SRS — **end-loaded** |
| Progress | Visited + practiced/mastered — not section/unit level |

**Verdict:** strong **study engine**; weak **lesson packaging** vs Coursera / LinkedIn Learning / Duolingo.

---

## 2. Content layers

```text
L1  Knowledge graph     domains · topics · claims · bank     SHIPPED
L2  Lesson narrative    14 module .md files                  SHIPPED
L3  Teaching atoms      units · diagrams · micro-checks      PLANNED
L4  Practice atoms      quiz · drill · mock · SRS            SHIPPED
L5  Progress            visited · practiced · mastered       SHIPPED (basic)
```

**Rule:** do not claim L3 complete until FEATURE_SURFACE rows flip to **present** and smokes wire into `check.sh`.

---

## 3. Teaching atom model (per module)

| Atom | Size | Role |
|------|------|------|
| **Hook** | 1 screen | Why this matters for ops/TPM interview |
| **Concept card** | 1 idea | Definition + non-example |
| **Worked scene** | 1–2 | Cascading failure narrative |
| **Diagram** | interactive | Mental model (see DIAGRAM-REGISTRY) |
| **Micro-check** | 2–3 Q | Bank items via `topic_ids`; instant feedback |
| **Interview drill** | 2–3 prompts | Already in MD — promote to UI |
| **Capstone** | Learn-15 / module quiz | Existing |

---

## 4. Edtech map (borrow mechanisms)

| Source | Copy | Skip |
|--------|------|------|
| Coursera | Unit path before quiz gate | Forums, video dependency |
| LinkedIn Learning | Continue + short chapters | Cloud accounts |
| Duolingo | Micro-step + immediate check | Guilt UX, paywalls, cert XP |
| Khan | Mastery on weak | Video-first |
| Brilliant | Try after example | Heavy animation only |
| Anki | Atomic cards / export | Cloze-only UX |

Offline static HTML + WASM remains non-negotiable (VISUAL.md / Substrate Law).

---

## 5. Product waves (M8)

| Wave | Outcome | BUILT ≠ WIRED until |
|------|---------|---------------------|
| **A** | Sticky TOC · reading progress · KaTeX or pre-render · embed power-path in M06 · Continue on learn hub | smoke + FEATURE_SURFACE |
| **B** | Split units from `##` / frontmatter · micro-checks from `topic_ids` | unit list UI + check |
| **C** | Diagram registry P0 (site-stack, heat-path, …) | DOM smoke per diagram |
| **D** | Miss → concept card · glossary popovers | results path |

---

## 6. Related

- [`DIAGRAM-REGISTRY.md`](./DIAGRAM-REGISTRY.md)  
- [`VISUAL.md`](./VISUAL.md)  
- [`FEATURE_SURFACE.md`](./FEATURE_SURFACE.md)  
- [`ORACLE-GAUNTLET.md`](./ORACLE-GAUNTLET.md)  
- Bank `topic_ids` · mastery.js · srs.js  
