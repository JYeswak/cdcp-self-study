# Damaged corpus — 2026-08-18 grounding wave, peak damage

**448 items, captured at `5c3d91ec` (2026-08-18 16:57), the measured peak.**

This is a REAL known-bad corpus, not a synthesized one. It is the output of an
actual agent wave that ran ~19 hours and rewrote 432 exam-question stems,
replacing applied-judgement questions with standard-heading recall
("Which topic is explicitly listed by ISO/IEC 22237-2:2024?").

## Why it is preserved

It was nearly destroyed. The repair restored the live bank to its pre-wave
state, which would have deleted the only realistic fixture this project will
ever have for this defect class. Recovering it was the second-highest-priority
item in an external grading of this repo.

A synthesized known-bad proves a gate fires on what its author imagined. This
corpus proves a gate fires on what actually happened.

## The damage curve (measured, sampled across 316 wave commits)

| time  | A-share |
|-------|---------|
| 03:16 | 30.1%   |
| 09:21 | 30.1%   |
| 10:07 | 32.2%   |  <- onset
| 13:01 | 42.3%   |
| 16:57 | 56.5%   |  <- peak, this corpus
| 22:51 | 29.4%   |  <- restored

Linear, ~3.7 points/hour over 7 hours. Baseline held for the first six hours of
the wave, so the wave's START was not the damage's start.

## What every gate must do against this corpus

- `answer-key-skew` must go RED. It reads 56.5% A against a 15-35% band.
- `grounding-wave` must go RED on the template-stem and recall-only detectors.
- Both must be GREEN against the pre-wave tree at `955a8f1`.

A gate that is green here would not have caught the wave, and is not a
regression proof (asupersync/AGENTS.md:372-409 — a test written after the repair
must be executed against the pre-repair state before it counts).

## What this corpus does NOT prove

It is one defect class from one wave. A gate that catches it has been shown to
catch THIS attack, not the next one. It also cannot establish item QUALITY:
whether an item discriminates between candidates who know the material and those
who do not is a property of RESPONSE DATA, which this project has none of.
