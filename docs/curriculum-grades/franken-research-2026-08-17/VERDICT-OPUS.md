# VERDICT — Opus lens, 2026-08-17

**Twenty lines. Detail in `FINDINGS-OPUS.md`, citations in `SOURCES-OPUS.md`.**

## What is real

1. **The notes are done, and they are externally correct.** I checked W-classes, NFPA 855 / UL 9540A / UL 9540, TIA-942-C, and EN 50600 / ISO 22237 / ISO 30134 against public sources: the corpus matches, including the details most secondary summaries drop — the shared 2 °C W-class floor, and "9540A is test data, not a listing."
2. **The 14 facility domains are EPI's published outline, 1:1, in order.** The map is right. M15's refusal to state an exam weight is the honest reading of the public record, because EPI's own outline has no 2.1 module.
3. **The 2026 outage reframe is corroborated by Uptime's own 2026 headlines** — power path leads; procedure-failure is the driver *within* human error, not a peer bucket.
4. **`bd-curriculum-truth-ebrr` .1–.27 are closed and the closes are real.** I spot-checked five and found the work done, not claimed.

## What we over-claimed

5. **`27/40` is the cut score EPI publishes.** `README.md:403-405` says the bar is "not affiliated with any official cut score." That sentence is false. Worse for the learner: EXIN states 68%, which implies 28/40 — so our bar sits one item *below* the real one while reading like reassurance.
6. **README advertises `804 files / 779 approved`; the tree holds `846 / 821` — and the registry probe already knows it.** The fact IDs are stale names whose needles assert the live numbers, so `claims-lint` goes green on a marker while the sentence beside it is 42 items wrong. That is this project's own nightmare arriving through its own gate: **the checker verifies that a row exists, not that a value agrees.**
7. **`README.md:20` says "Fourteen modules"; `README.md:48` says "15 modules."** Same file.
8. **One live self-contradiction in the teaching corpus:** `mock40-q04` (approved, in the capstone as PRACTICE Q4) keys the three-bucket cartoon that `m01-q210` (approved, same topic) marks wrong. Ranked #1 by Pass 7 and again by the 17 Aug sittings. **It is in no bead.**
9. **The starting gap register is mostly stale** — 11 rows already closed, and its "M06 mermaid collapses A/B through one ATS" misreads a deliberate SPOF exhibit that sits *below* a correct 2N drawing. Re-filing those would be motion, not work.

## What a Fluidstack-style oral fails on tomorrow

10. **Not the notes — the notes would carry the candidate.** The failure is inherited by anyone who studies by drilling.
11. **Any number at all.** 846 of 846 items are `qualitative_only`.
12. **"Here is the one-line — is concurrent maintainability real?"** Taught in M06 prose; no item makes the learner perform it.
13. **"What is in the catalog, and which OLA makes that SLA true?"** Taught in M15; **zero** of the nine public 2.1 headings appear in any of 846 items. The 39 `m15` items test labelling, MOP, MTBF/MTTR, cleaning, spares — the pre-2.1 set.
14. **Seed luck.** Catcher, W-class, one-ATS SPOF, and scope-of-nines exist in the bank and went undrawn across sampled seeds.

## What CDCS science is still missing

15. **Electrical:** `I = P / (√3 · V · PF)`, current from kW, transformer/PDU sizing, fault-current and bonding arithmetic. The syllabus names "single phase and three phase power" and "power sizing"; the bank tests the words.
16. **Thermal:** SHR, enthalpy, wet-bulb/approach, coil leaving-air, `ΔT × cfm × 1.08`. The corpus has "sensible" as a vocabulary item and a dew-point one-liner.
17. **Storage and fire:** battery `Ah × V × η` autonomy, gas concentration and hold time.
18. These are `bd-epi-ecosystem-ms4j.1`'s reason to exist. **CDCS is correctly the FIRST track** — it is the only one that closes a gap the CDCP corpus cannot close on its own.

## Call

19. **Reality-check: YES.** Implementing the open beads would produce the thing. But the two cheapest fixes are not on any bead and are both honesty defects — **the cut-score sentence and the count drift** — and honesty defects in an honesty-first product outrank everything queued behind them.
20. **Ship order I would argue for:** F1 cut-score → F2 retire the cartoon → F3 count drift → `.28` M15 2.1 bank → `ms4j.1` CDCS arithmetic. The first three are single-sentence or single-file edits; the last two are the ones that change what a learner can do.

---

*Nothing here was implemented. No module, charter, bank, or bead was edited. Completing this program does not certify anyone.*
