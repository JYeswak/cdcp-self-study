# DISPATCH LOG — 2026-08-20

One row per dispatch: when, which pane, what, why, and what came back. Kept by the
controller. **Rows before 18:10Z are reconstructed from the session transcript, not written
at dispatch time** — they are accurate but they are a reconstruction, and that distinction
is the whole reason this file now exists.

## The rule this file enforces

**A worker pane that is idle is the controller's defect, not the pane's.** Twice today the
operator had to point out an idle pane while the controller was mid-task and had simply not
looked. Checking `fleet-tick.sh` before anything else, every tick, is the mechanism;
"remembering to check" is not.

Grade the controller on two numbers, recorded per tick:

- **`idle_found`** — worker panes idle at the start of a tick. Target 0. Any non-zero is a
  controller miss, whether or not the automation should have caught it.
- **`time_to_dispatch`** — how long a pane stayed idle before it got work.

## Ledger

| # | UTC | Pane | Dispatched | Why | Outcome |
|---|-----|------|-----------|-----|---------|
| 1 | ~14:57 | 2 | `bd-installability-sm4g.22` + `bd-engine-not-gate-ar39.15` | Both P0; CI had never completed a run | **Done** — 98 steps executed, 0 skipped, 427s, pinned SHA `85dc19a7` |
| 2 | ~14:58 | 3 | `bd-9qbo` stem-overlap cue | Last measurable construction cue above chance | **Done** — 32.4% → 27.2% vs 25% floor; declined to churn for 2pp |
| 3 | ~15:05 | 2 | Reframe: CI is *unsatisfiable*, not slow | Found 52s wall time + rustfmt/ceiling contradiction in an existing receipt | Accepted; drove the pinned-run design |
| 4 | ~16:10 | 3 | Q12 stranger test | Loop-3 precondition; never once attempted | **Done** — install from empty dir 66.78s, 40 questions via WASM, 6/40; filed `.26/.27/.28` |
| 5 | ~17:25 | 2 | `local-ci.sh` unlisted; floor no longer blocks | Substrate guard RED on their new file | Fixed by pane 2 |
| 6 | ~17:45 | 3 | Redirect off curriculum → the three new P0s | My dispatch predated the beads they filed | `.26`, `.27` **done** (`6b21e0d8`) |
| 7 | 17:58 | 3 | Standing never-idle order + `.28` | Pane idle; doctor source-checkout fallback outranks | In flight |
| 8 | 18:10 | 2 | Standing never-idle order + close the two P0s + `--prove-wired` exit 4 | **Pane idle ~10 min — controller miss #2** | In flight |
| 9 | 18:29 | 3 | autofeed `FED rc=0` (automatic) | Pane idled after `.26`/`.27`; **automation caught it, not me** | Self-served: read queue, found it dry, correctly refused to steal pane 2's beads, resumed `bd-readme-public-rigor-8y0r` |
| 10 | 18:36 | 3 | Queue refill (Q15–Q19) + claim hygiene | **Queue went functionally dry — controller miss #3.** Pane 3 held 6 claims; only unclaimed items were gate-chain work fenced off from it | In flight |
| 11 | 19:05 | 2 | Q17 cognitive-level inventory (self-served) | Pane 2 finished P0s and pulled from queue | **Done** — 534 recall / 318 apply / 105 analyse; 56% recall, 11% analyse |
| 12 | 19:07 | 2 | STOP on Q14 + build the instrument instead | Pane 2 had claimed the human-only loop-3 item | Released claim; built `docs/loop3/EXTERNAL-HUMAN-MOCK.md` with blank score fields |
| 13 | 19:08 | 3 | Redirect off Q17 (collision) | I dispatched pane 3 to an item pane 2 had already claimed | Redirected to pack/goldens lane |
| 14 | 19:11 | 2 | Quiescence request | Pane 3 needed a consistent tree to regenerate | Tree measured already clean; released pane 3 in ~3 min |
| 15 | 19:12 | 3 | RESUME regeneration + coupling gate | Unblocked | In flight |

## Controller misses

| UTC | What | Cause | Fix |
|-----|------|-------|-----|
| ~17:50 | Pane 3 idle, operator had to say so | Watcher printed `IDLE` into a log with no consumer — BUILT ≠ WIRED | Built `ntm-autofeed.py` to consume the events |
| 18:05 | Pane 2 idle ~10 min, operator had to say so again | **My autofeed regex could never match.** It required `pane=0.2(pid…)`; the watcher emits `pane=2(pid…)` because it formats from `--robot-activity`, whose `pane` is a bare index. I "verified" it against samples I invented from the *snapshot* schema instead of a real watcher line. | Regex accepts both forms, re-verified against live `--robot-activity` data. Fired for real at 12:08:17 local, `FED cdcp pane=2 rc=0` |
| 18:31 | Queue functionally dry for pane 3 | I stocked 14 items and then stopped restocking. One pane claimed six of them; the two genuinely unclaimed were gate-chain work I had fenced off from that pane. Not an idle pane — a pane with nowhere to go next. | Refilled with Q15–Q19 (`17480b25`) and told pane 3 to hold one claim at a time. **Keeping the queue stocked is a standing controller duty, not a one-time setup task.** |
| 19:08 | Pane 3 blocked on me ~10 min | It paused before regenerating and asked for quiescence — exactly as instructed — and I did not answer. `fleet-tick.sh` reported it as "idle", which is the **wrong label**: blocked-on-controller and out-of-work are different failures needing opposite responses, and the check cannot tell them apart. | Ran the quiescence protocol; tree was already clean, so the 10 minutes bought nothing. Open gap: fleet-tick cannot distinguish the two states. |
| 19:10 | Q14 human-only marker silently failed to apply | My `str.replace` did not match because a pane had already rewritten that line, and my script **printed "marked human-only" unconditionally** without verifying the replacement happened. I reported a success I had not checked. | Re-applied against the real current text and verified with `git show HEAD:` — 1 occurrence — rather than trusting the edit |
| — | Collision: two panes on Q17 | I dispatched pane 3 to an item pane 2 had claimed minutes earlier. I was not reading the queue's claim state before dispatching. | Redirected pane 3; pane 2 keeps the inventory |

The second miss is the more instructive one: I wrote a gate, tested it against a fixture I
made up, and reported it as armed. That is the same defect this project has been cataloguing
all day — a check proven against the author's belief rather than against reality — committed
by the person writing the doctrine about it, twenty minutes after writing it.

## Mechanism

- `~/.local/bin/fleet-tick.sh <session>` — exit **3** if any worker pane is not affirmatively
  `working`. `unknown`/`degraded` count as idle, never as busy. Zero agents is an ERROR, not a
  pass.
- `~/.local/bin/ntm-autofeed.py` — consumes the watcher's `IDLE`/`STUCK` events and sends the
  pane a self-serve order against `QUEUE-2026-08-20.md`. Excludes panes 0 and 1. 180s cooldown.
- Cron respawn guard every 5 min, with absolute paths and an explicit `PATH`, because cron does
  not source the shell env and would otherwise resolve a different `ntm` or none at all.
- Both panes additionally carry a standing never-idle order, so a failure of the automation
  degrades to the pane self-serving rather than to an idle pane.
