# Skill Loop Progress
# Skill: readme-writing
# Target: README.md + GitHub about blurb
# Bead: bd-readme-public-rigor-8y0r
# Total Passes: 6
# Started: 2026-08-17T18:25:00Z

## Status: IN PROGRESS — Passes 1-4 done, merged, APPLIED to README. Grading round 1 running.

## Missions
1. Hero / one-liner / badges vs live product (14 vs 15, bank pool, not-a-cert)
2. TL;DR table honesty
3. Limitations + FAQ + north-star (kill CDCS-out-of-scope; CDCS+CDFOS first, not shipped, not a credential)
4. Advertised gate/engine numbers vs receipts and FEATURE_SURFACE
5. Running-it / install path vs installability north-star
6. Pre-publish: GitHub about blurb + exact apply list

## Completed Passes

| # | Pass | Agent | Artifact |
|---|---|---|---|
| 1 | Hero / one-liner / badges | Claude Opus | `PASS-01-hero.md` |
| 2 | TL;DR table honesty | Claude Opus | `PASS-02-tldr.md` |
| 3 | Limitations + FAQ + north-star | Codex | `PASS-03-limitations-faq.md` |
| 4 | Gate/engine numbers vs receipts | VioletIsland (Grok, resv 869) | `PASS-04-numbers.md` |
| — | Merge + dedupe | Claude Opus | `COMBINED-LEDGER.md` |

## Applied to README.md — 2026-08-17 ~13:00 MDT

A1–A5 applied (12 insertions, 12 deletions). See `COMBINED-LEDGER.md` §"Apply record"
for the full site list. Summary: modules 14→15, words ~54k/~62k→~82k, bank
804/779→846/821/25 at **all seven** sites, engine row 7 crates/3,763 lines/281 KB
→ 18 crates/~67k(+~48k tests)/518 KiB, scripts 42→32, CDCS "out of scope"→
"Advanced direction, not shipped".

**FROZEN — do not hand-edit:** step count (now 90, updated by `bd-installability-sm4g`
at 12:50), `72` injections, per-suite `n` table, and `10 selftest suites`
(tree now has **12**). All are regenerated from `CHECK_STEPS=` / `INJECTIONS=`
receipts by a real `check.sh` run. The suite/injection gap belongs to
`bd-installability-sm4g`, not this bead.

## Grading round 1 — COMPLETE → `GRADE-ROUND-01.md`

Five read-only graders on the applied README: measurement truth · readme-writing
skill compliance · honesty constitution · de-slopify (skill refreshed from Studio
2026-08-17, sha `ca3be8a6dec0`) · internal + cross-doc consistency.

**A1–A5 held** — two graders independently re-measured every applied value; none wrong.

Headline findings, none applied:
- **T1.1** lines 97/290 render as `<h2>` on GitHub (setext underline). Found by 3 graders.
- **T1.2** `:167` "Drill for **spaced repetition**" — the one term CHARTER forbids,
  contradicted by `:115` in the same file. Unflagged because of T2.2.
- **T2.1** `:403-405` "not affiliated with any official cut score" is false —
  EPI publishes 27/40, and `knowledge/exam_form.toml` sourced it from them.
- **T2.2** claims-lint's scan root is `course-engine/`, so the root README,
  CHARTER, HOW-TO-USE, the curriculum map and `modules/` are **never scanned**.
  The sentence claiming "every document in this repository" is in an unscanned file.
- **T2.3** the L3 external oracle **landed** (`cdcp_data/tests/oracle.rs`, wired at
  `check.sh:1086`); the README still calls it open and calls the native grader "the oracle".
- **S1** the A1–A5 apply left `CHARTER.md:208` at 804/779 — sibling sync needed.

## Remaining missions
5. Running-it / install path vs installability north-star
6. Pre-publish: GitHub about blurb + exact apply list
