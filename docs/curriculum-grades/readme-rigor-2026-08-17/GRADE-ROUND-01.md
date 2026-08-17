# GRADE ROUND 01 — five-lens crew grade of the applied README

**Bead:** `bd-readme-public-rigor-8y0r` · **Date:** 2026-08-17 · **Subject:** `README.md` after A1–A5
**Crew:** measurement-truth · readme-writing compliance · honesty constitution · de-slopify (skill refreshed from Studio, sha `ca3be8a6dec0`) · internal + cross-doc consistency
**All five read-only.** No `cargo`, no `check.sh`, no bead close. Nothing in this round is applied.

> Completing this program does not certify anyone. 27/40 is a study signal, not a pass mark.

---

## A1–A5 held

The measurement-truth and consistency graders independently re-measured every applied value against the tree. **All five corrections verified: 15 modules · 81,860 words · 846/821/25 · 18 crates · 530,385 B = 518 KiB · 32 scripts.** No applied number was found wrong.

---

## TIER 1 — broken right now, zero-risk fixes

### T1.1 — Two paragraphs render as `<h2>` on GitHub

**Found independently by three graders and verified directly.** Lines **97** and **290** are the only two of 15 `---` rules with no preceding blank line. In CommonMark `text\n---` is a **setext H2 underline**, so the paragraphs above them ("Prose drifts…" and "…regenerate it deliberately…") render as giant headings on the public repo page.

```bash
awk 'prev != "" && $0 == "---" {print "line " NR}; {prev=$0}' README.md   # -> 97, 290
```

**Fix:** insert a blank line before each. No text change.

### T1.2 — "spaced repetition" — the one term the project forbids

`README.md:167`: **"`Drill` for spaced repetition, `Mock` for a timed 40-question exam."**

Contradicted by the same file eleven lines up, and by two other documents:

| Source | Says |
|---|---|
| `README.md:115` | `cdcp_schedule  short-interval ladder + mastery bars (**not SRS**)` |
| `CHARTER.md:67` | "**Not spaced repetition** — no expanding interval, no forgetting curve… **Calling it SRS overstates it.**" |
| `CHARTER.md:106` | "short-interval review (1d/3d cap; **not SRS**)" |
| `FEATURE_SURFACE.md:20` | "cap 3 days (not SRS)" |

**Fix:** `**Drill** for short-interval review (1-day/3-day ladder, not SRS), **Mock** for a timed 40-question exam.`

**This is the smoking gun for T2.2.** A load-bearing honesty violation, in the most-read file in the repo, unflagged by every gate — because the file is outside claims-lint's scan set.

---

## TIER 2 — false or inverted claims

### T2.1 — 27/40 is the official cut score (CRITICAL)

`README.md:403-405` says the bar is *"not affiliated with any official cut score."*

- EPI's public course page: *"The passing mark is **27 out of 40**."*
- EXIN: **Pass Mark 68%** (27/40 = 67.5%, so 28 is the stricter reading — do not assert the two sources agree exactly).
- **The repo's own file:** `knowledge/exam_form.toml` → `pass_correct = 27`, `pass_ratio = 0.68`, `[sources] epi_exam = "src-epi-cdcp-page"`, `exin_exam = "src-exin-cdcp-page"`.

The number was **sourced from EPI and then described as unaffiliated with EPI**.

**Proposed replacement (from the honesty grader), lines 403–405:**

```markdown
**Why is 27/40 the bar?** It mirrors the public exam form. EPI's course page
states the CDCP passing mark is 27 out of 40; EXIN states 68% (27/40 is 67.5%,
so treat 28 as the stricter reading). The mock reuses that shape so practice
feels calibrated — but here the number is a study signal only, a threshold for
your own review loop. This mock is scored by this project, means nothing to EPI
or EXIN, and grants no credential; only the official exam after authorised
training does that. [[claim:claim-study-signal-27]] [[claim:claim-not-epi-certified]]
```

This is **stronger** on non-affiliation than the current text: it buys distance with a true fact ("means nothing to EPI or EXIN") instead of a false one. Note the two `[[claim:]]` markers stay inert until T2.2 is fixed; add them anyway so they go live when the scan root is corrected.

`README.md:47` ("27 correct is a study signal, not a pass mark") is milder and was kept by two graders. One argued for "not a pass mark **here**". Low priority; decide with T2.1.

### T2.2 — The honesty linter does not read the honesty document (HIGH)

`README.md:22`: *"a registered claim (`claim-not-epi-certified`) that a linter enforces **across every document in this repository**."* `README.md:69` repeats it as *"any document"*.

```
claims_lint.toml:16      roots = ["README.md", "docs"]
cdcp_gate/src/root.rs    ANCHOR = registries/claims.toml  →  resolves to course-engine/
```

Scanned set is therefore `course-engine/README.md` + `course-engine/docs/**`. **Outside it:** the root `README.md`, `CHARTER.md`, `HOW-TO-USE.md`, `00-curriculum-map.md`, all of `modules/`.

Measured on the root README: **0** `[[claim:]]` markers, against **6** matches for the `not-certified-honesty` pattern and **3** for `study-signal-27`. It would trip at least two `load_bearing` rules on first scan. It passes because it is never read.

The sibling gate disagrees: `doc_facts.rs:1161 corpus_root()` climbs to the parent holding `CHARTER.md`, which is why `[[fact:]]` markers work in the same file. **Two gates, two definitions of "the repository," and the honesty sentence rides the narrower one.**

**Two fixes, in preference order:**
1. **Widen the scan** — add `"../README.md"` and `"../docs"` to `claims_lint.toml [scan] roots`, or give claims-lint the same `corpus_root` climb `doc_facts` uses. Then the sentence becomes true and T1.2 gets caught automatically. Expect an initial RED: the root README needs markers added.
2. **Narrow the sentence** — "…across `course-engine/README.md` and `course-engine/docs/`." Honest, but leaves the root README ungated.

Config note: `claims_lint.toml:20` comments its exclusion paths as *"relative to repo root"* while roots resolve against the engine root. Worth reconciling.

### T2.3 — The L3 external oracle landed; the README still says it is open (HIGH)

This is the **only** drift running the other direction — the README understates its own rigor, in the place it can least afford to.

| Location | Says |
|---|---|
| `README.md:344` | "⚠️ **weakest link** — the oracle is the *native* grader and the public syllabus domains" |
| `README.md:370` | "**L3 external oracle** — **open, and the most valuable thing to fix**" |
| `README.md:390` | "**L3 is thin**" |
| `CHARTER.md:148` | "**L3 External oracle (factual content) · YES · test-backed**" |

Verified wired:

```
crates/cdcp_data/tests/oracle.rs          (11,908 B, 2026-08-15)
  published_references_match_or_red()
  perturb_one_tolerance_unit_is_red()     ← known-bad
  zero_reference_locations_is_error()     ← anti-vacuous
  fewer_than_five_locations_is_error()    ← anti-vacuous
  selftest_delete_comparison_is_nonzero()
reached by  scripts/check.sh:1086  cargo test --locked --workspace
CLI verb    cdcp oracle-check       (main.rs:168, 763)
```

It compares computed site quantities (free-cooling hours, seismic design values, grid carbon) against NREL TMY3 / USGS ASCE 7-16 / EPA eGRID2023 — sources the project does not control.

Worse, `README.md:344` calls the native grader "the oracle," which is exactly the conflation `CHARTER.md §5a` was written to kill ("Grade goldens → output of the grader that produced them → External? **no**").

**Fix — do not just delete the caveat.** CHARTER agrees bank keys remain unchecked. Rewrite to: *"✅ scoped — `cdcp_data` compares computed site quantities against NREL/USGS/EPA published references we do not control (`crates/cdcp_data/tests/oracle.rs`). **Bank keys remain unguarded**: no external suite validates whether a module teaches its domain correctly. The native grader and goldens are **not** oracles (CHARTER §5a)."* Roadmap row narrows to *"L3 external oracle **for bank content**."*

---

## TIER 3 — measurement precision

| # | Finding | Location | Fix |
|---|---|---|---|
| T3.1 | **`~67k lines` is misleading — my own edit.** 16,836 of 67,519 `src` lines are inline `#[cfg(test)]` (73 files). Production ≈ 50,683; total test code ≈ 63.5k. The "(plus ~48k of tests)" parenthetical invites reading 67k as production. | `:49` | Name the measurement: "~67k lines under `crates/*/src` (~16k of it inline test modules), plus ~48k in `crates/*/tests`" |
| T3.2 | **The "shell selftest suites" qualifier is wrong.** README names only the Rust legs and `publishability-bar.sh` as exclusions. Two **shell** suites also emit real `INJECTIONS=` receipts and are excluded — `check.sh:1114`: *"adding it grows cdcp_gate past gate_shrink"*, a code-size budget, not a rigor call. A reader runs `ls scripts/selftest_*.sh`, counts **12**, cannot reconcile with 10. | `:12,50,234,345` | "10 **registered** shell selftest suites; `installer` and `learner_verbs` emit receipts but are deliberately unregistered (`gate_shrink` budget)" |
| T3.3 | **System map lists 8 of 18 crates, no ellipsis.** Omits `cdcp_gate` (which hosts `verify-step-count`/`verify-injection-count`, the machinery of the README's longest section) and `cdcp_data` (which hosts T2.3's oracle). | `:110-118` | Add `└── … 10 more` or "8 of 18 shown" |
| T3.4 | `registries/` listed as 3 files; there are **10** — including `doc-facts.toml`, which enforces the `[[fact:]]` markers the README uses. | `:121` | Add truncation marker |
| T3.5 | `content.lock` described as covering 3 things; it has **4** sections (`[data]` unmentioned). | `:287,348` | Add `[data]` |
| T3.6 | M10 "4 free PDFs referenced"; `knowledge/corpus/free-pdfs/` holds **5** sidecars, all `access = "free"`. Duplicated in `CHARTER.md:254`. | `:368` | 5, or state the not-vendored/body-retained split |
| T3.7 | **`.planning/` does not exist** — absent from `git ls-files` and the working tree. | `:92` | Repoint at `course-engine/docs/` + `scorecards/` (already named at `:330`) |
| T3.8 | "CI runs exactly this script and nothing else." `.github/workflows/check.yml` has two further steps (an injected-fault re-run, and a tree-clean assertion). | `:228` | "…as its only gate, plus an anti-vacuous re-run and a tree-clean assertion" |
| T3.9 | `less 00-curriculum-map.md  # the 14 domains` — the map is titled **15 Modules**. | `:152` | 15 |
| T3.10 | L2 row says `smoke_slo.sh` **"walls"**. CHARTER §5: one shell step, conditional on `export-web`, with a documented `CDCP_SKIP_SLO=1` bypass and no known-bad. (A real Rust leg exists in `crates/cdcp_cli/tests/slo.rs` and is *not* cited.) | `:343` | Cite the Rust leg; name the bypass |
| T3.11 | L5 row: `cargo-fuzz` targets "present" is existence only — `fuzz/` is outside the workspace, so no campaign ever runs. `doc-facts.toml` explicitly exempts this cell. | `:346` | Append "targets exist; `fuzz/` is outside the workspace, so `check.sh` never runs them" |
| T3.12 | L7 "ecosystem lock" — `content.lock` pins **content**, not cross-repo deps by git-rev. The "scoped" qualifier saves it, but it redefines the layer. | `:348` | "scoped — content, not dependencies" |
| T3.13 | "that would be both **illegal** and useless" — unsupported legal conclusion, jurisdiction-varying, no registry row. | `:400` | "a licence violation and useless" |
| T3.14 | `- **Advanced direction, not shipped** … neither is a credential.` Read literally this denies CDCS/CDFOS are real credentials — they are. Intent is "neither track *here* would be." | `:386` | "…neither track would grant the EPI®/EXIN® credential of that name" |

---

## TIER 4 — de-slopify sweep

**Register: low-slop.** Zero AI-vocabulary hits across the full core list (scan run, null result reported as evidence). No significance inflation, no puffery, no weasel attribution. The specificity floor is high and load-bearing throughout.

**56 emdashes → 45 genuine prose.** Correctly rejected: 3 inside the ```text``` system-map fence, 1 table placeholder, 7 rigor-table label separators, plus numeric en dashes, `→` pipeline arrows, and `·` separators.

**The real finding is cadence, not punctuation.** `X, not Y` fires **~19 times** and has become the document's default sentence engine — which dilutes the four negations that are actually protected. Flatten the five most removable (`:67`, `:94`, `:251`, and two others) and the honesty claims regain their weight. **Leave `:47`, `:71`, `:273`, `:379`, `:404`, `:425` alone.**

Two named sentences: `:260` `**…and it was the last one that wasn't.**` (paraprosdokian, bold, paragraph-opening) and `:279` `Same bank, same attempt, same digest — forever.` (triad + emdash + one-word punchline).

Full itemised table with per-line replacements is in the grader output. Net if applied: 56 → ~11 emdashes, **zero numbers changed, zero claim markers touched, zero honesty statements softened**. `:404` takes a colon only, so every protected word survives byte-for-byte.

---

## SIBLING DOCUMENTS — not README, separate authorization

Every cross-document contradiction except T2.3 runs the same direction: **the README was updated and a sibling was not.**

| # | File | Stale content | Note |
|---|---|---|---|
| S1 | `CHARTER.md:208` | 804 / 779 | **My A1–A5 apply created this divergence.** README and FEATURE_SURFACE now say 846/821; CHARTER does not. Fact markers are fine (stable IDs). |
| S2 | `CHARTER.md:73` §3 | "**axum** serve" | No `axum` in `Cargo.lock` or any crate manifest. `serve` is pure std — README `:36` and FEATURE_SURFACE `:14` agree. |
| S3 | `course-engine/README.md` | "L4 WASM dual-path \| **open**", "L5 browser UI \| **open**" | Both shipped. **This file *is* claims-lint's scan root**, so it is authoritative-by-wiring and wrong. |
| S4 | `docs/PHASE-NEXT.md:17` | "units (**127**)" | `units_index.json` says 134. `doc-facts.toml:92` records the root README being corrected 127→134; PHASE-NEXT never got the same fix — and README `:359` names PHASE-NEXT a source of truth. |
| S5 | `docs/PHASE-NEXT.md` | "M9 … open until S3 flip", `public_repo=false` | README/CHARTER say public since 2026-08-12. This is exactly the "stale pre-flip visibility claim" `selftest_doc_consistency` plants as a known-bad — the real instance sits in a table shape the gate cannot parse. |
| S6 | `.github/workflows/check.yml` | header comment "51 ordered steps … **five** known-bad selftest suites" | README says 90 / 10. |
| S7 | `registries/doc-facts.toml` header | quotes "7 Rust crates, 3,763 lines, 281 KB WASM" | Stale pre-correction README values living in a registry comment. |

---

## STRUCTURAL — worth beads, not edits

1. **claims-lint scan root excludes the corpus root** (T2.2). Root README, CHARTER, HOW-TO-USE, curriculum map, and all of `modules/` are unscanned. T1.2 is the proof this is not theoretical.
2. **`verify_doc_consistency` only parses milestone tables with a Status column.** `PHASE-NEXT.md`'s stale rows live in a `| Wave | Outcome |` table, so the gate that plants "stale pre-flip visibility claim" as a known-bad cannot see the real one two files away (S5).
3. **The step/injection advertisement sites sit exactly at their floors** — 4 sites against `MIN_STEP_SITES = 4`, 5 against `MIN_ADVERTISEMENT_SITES = 5`. Intended behaviour, but zero margin: removing any single site trips the gate.

---

## Recommended order

1. **T1.1 + T1.2** — broken now, zero risk, no wording debate.
2. **T2.1 + T2.3** — the two false claims, both with finished replacement prose.
3. **T2.2** — decide widen-the-scan vs narrow-the-sentence. Widening is the real fix and will initially go RED.
4. **T3.1 + T3.2** — the two measurement claims that mislead, including one of mine.
5. **S1–S7** — sibling sync, needs its own authorization (CHARTER is edit-protected in this bead).
6. **T4** — de-slopify sweep, last, since it touches the most lines and none of the facts.
7. **T3.3–T3.14** — precision cleanup, batchable.

**Frozen throughout:** the step count (90), `72` injections, the per-suite table, and the suite count. All are regenerated from `CHECK_STEPS=` / `INJECTIONS=` receipts by a real `check.sh` run. T3.2 changes only the *prose describing the exclusions*, never a number.

*Round 1 grade ledger. Nothing applied. No cargo, no CHARTER edit, no bead closed, no commit.*
