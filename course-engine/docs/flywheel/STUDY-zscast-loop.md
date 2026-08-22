# Study: the zeststream-cast loop

Snapshot: read-only inspection of `~/Developer/zeststream-cast` on `studio`,
2026-08-22. No command in the target checkout wrote, committed, installed, or
removed anything. The local comparison is limited to the existing `.flywheel`
files named in the assignment; this study creates no other local file.

## Executive answer

zeststream-cast has a stronger *wiring* discipline than course-engine: it
distinguishes an executable that is present from one named by a stage, hook, or
live scheduler, and its close-wiring gate refuses to call `MANUAL` a pass
(`scripts/lib/close_wiring_gate.py:4-15`, `:177-193`, `:307-319`). It also has
a real external publication record for its launch announcement: the ledger
contains an X thread URL and four tweet IDs, then a LinkedIn URL, activity ID,
and URN (`.planning/launch-2026-05-18/ledger.md:75-109`).

That is not the same as proving that a zscast episode reached a viewer. The
episode smoke path creates a **private** YouTube broadcast and deletes it on
exit (`lib/cmd-smoke-youtube.sh:12-29`, `:289-305`, `:308-337`). The production
`publish --apply` path can upload to YouTube/R2/S3 and can emit a local upload
receipt (`lib/cmd-publish.sh:62-78`, `:101-179`, `:230-243`), but the checked-in
receipt corpus has no external-signal, publish, watch, or product-move fields,
and no publish-specific test was found in the tracked test/hooks search. The
repo therefore has a real signal for *social launch*, a plausible but
unsubstantiated signal for *media publication*, and no measured signal for
*watched by an audience*.

The answer to the comparison question is consequently two-part:

1. zscast does what course-engine currently does not by requiring a caller for
   artifacts and by retaining audience-facing launch identifiers; and
2. it does **not** close loop #3 for episodes in the evidence that exists. Its
   own P6 document makes the intended standard explicit—three clean drills by
   three paid strangers, with a survey and receipts—but that is a plan/rubric,
   not a recorded run (`docs/plans/zscast-product-experience-v1.md:587-601`).

The first mechanism I would port is an external-delivery receipt consumed by a
forward gate: immutable artifact digest, sink, external object ID/URL, a
successful post-delivery read, and an audience observation or human report.
Absent that receipt, “published” remains an internal assertion. Do not port a
launchd heartbeat as a substitute for that signal.

## Evidence rules: PRESENT is not WIRED

I use these labels throughout:

- **PRESENT** means a file, schema, plan, or helper exists.
- **WIRED** means this study found the caller and, where applicable, a live
  hook/scheduler path. A comment saying “consumer” is not a caller.
- **ORPHAN / UNPROVEN** means the artifact is present but this checkout has no
  automatic caller for it.
- **PARKED** means the repo deliberately excludes it from the validation chain
  with a written reason; that is not the same as an accidental orphan.

The remote checkout has no `.flywheel/` directory. The nearest loop entry
artifact is `loop-kit/loop-start-chokepoint.sh`; its own source is evidence of
presence, not of invocation. The tracked-source search found only its own
references and self-test references, no caller. This is why it is marked
ORPHAN below rather than WIRED.

## 1. Machinery map

| Surface | PRESENT evidence | WIRED evidence / verdict |
|---|---|---|
| `.beads/` | `.beads/config.yaml:1-4` declares the project prefix and defaults; the checkout also contains the Beads DB/issues/lock state. | **WIRED for pane ownership:** `scripts/lib/pane_watch.sh:161-168` invokes `br list --status in_progress --json` and treats an unavailable claim board as a divergence, not free capacity. This is a real caller, not a directory listing. |
| Loop-start chokepoint | `loop-kit/loop-start-chokepoint.sh:2-18` defines the signed-charter refusal contract and exit classes. | **ORPHAN / UNPROVEN:** no tracked caller was found. Its `--selftest` is self-invocation only (`loop-kit/loop-start-chokepoint.sh:143-221`). A self-test proves that the script can fail; it does not prove the loop asks it. |
| Git hook installer | `scripts/install-hooks.sh:10-23` specifies fast pre-commit, full pre-push, and LFS-only post-commit hooks. | **WIRED:** the installer writes the pre-commit bytes (`scripts/install-hooks.sh:45-100`) and pre-push bytes (`:102-162`); the installed `.git/hooks/pre-commit` runs staged shell syntax, shellcheck, gitleaks, and the close-wiring gate (`.git/hooks/pre-commit:9-75`). The pre-push hook creates a committed-state temporary worktree and invokes `scripts/ci-local.sh` (`.git/hooks/pre-push:39-54`). |
| Local CI mirror | `scripts/ci-local.sh:1-39` defines the job set and says missing tools are UNRUN rather than pass. | **WIRED by pre-push:** `.git/hooks/pre-push:45-52` calls it on the pushed SHA; its bats job runs the tracked tests (`scripts/ci-local.sh:345-424`) and the summary returns 2 for UNRUN and 1 for failures (`:514-530`). |
| Ordered `scripts/check.sh` chain | `scripts/check.sh:1-7` claims to be the one ordered chain; its stage array is explicit and fail-fast (`scripts/check.sh:1700-1763`). | **ORPHAN / UNPROVEN locally:** the tracked `.github/workflows` set was empty, and the source search found only self-references, manifests, and comments—not a workflow, hook, or scheduler that executes `scripts/check.sh`. Its header is a claim of a CI caller, not evidence of one. The close-wiring scanner treats it as an invoker input (`scripts/lib/close_wiring_gate.py:47-62`), which makes it eligible evidence when present, not automatically invoked. |
| Pane liveness scheduler | `scripts/lib/pane_watch_launchd.sh:1-23` defines a LaunchAgent wrapper and provenance receipt. | **WIRED:** `--install` writes a LaunchAgent with `RunAtLoad` and `StartInterval=20` and bootstraps it (`scripts/lib/pane_watch_launchd.sh:126-164`); its `--run-once` caller invokes `pane_watch.sh --once` and writes `zs.pane_watch_receipt.v1` (`:31-72`). A live `ai.zscast.pane-watch.plist` was present on `studio`; read-only `--status` returned FAIL because the last receipt was a failed run. The status code is consumed before zscast dispatch (`scripts/lib/pane_dispatch.sh:38-67`). |
| Pane dispatch | `scripts/lib/pane_dispatch.sh:1-18` describes a delivery gate and its deletion condition. | **PARKED by policy, not an accidental orphan:** `close_wiring_gate.py:82-105` excludes it because automatic dispatch from `check.sh` is unsafe. The live caller is an explicit orchestrator invocation; the script itself checks pane identity, pane text, send output, and output-sequence advance (`scripts/lib/pane_dispatch.sh:70-146`). |
| Media CLI | `bin/zscast:165-166` sources `lib/cmd-publish.sh`; `bin/zscast:876` dispatches `publish`, and `:904-906` dispatches `smoke-youtube` and `post-stream`. | **WIRED as user-facing commands.** This is stronger than a helper sitting in `lib/` with no route. It does not by itself prove that anyone ran the command or that a viewer watched. |

### Count

The repo's own close-wiring command reported:

```
20 wired, 10 parked, 0 manual of 30 tracked scripts/lib artifacts
```

That is the **strict scanned domain**, from `scripts/lib/close_wiring_gate.py`
(`INVOKER_PATHS`/`PARKED` at `:47-85`, scheduler expansion at `:145-174`,
classification at `:177-193`, and the live result from
`python3 scripts/lib/close_wiring_gate.py --json`). Thus the requested
wired-vs-orphan count is **20 WIRED vs 0 accidental orphan in the scanner, plus
10 deliberately PARKED**. Once the unscanned loop artifacts are included, at
least two are locally unproven: `loop-start-chokepoint.sh` and
`scripts/check.sh`. I do not silently count either as wired merely because a
document names it.

## 2. Choke-point, mode enum, and forbidden phrases

### Tick/choke-point behavior

There is no zscast `.flywheel` tick emitter or canonical tick ledger in the
target checkout. The loop-kit chokepoint instead authorizes loop entry by
finding a charter and running `charter-lint --require-signed`; it refuses the
two latch files and emits `SKIP`, `REFUSE`, or `PASS`
(`loop-kit/loop-start-chokepoint.sh:99-140`). Its JSON output calls its own
trust level `advisory-floor-not-unforgeable` (`:89-96`, `:224-237`). That is a
real refusal gate when called, but the missing caller is decisive: PRESENT is
not WIRED.

### The mode enum, verbatim

There is no typed Rust/Python loop-mode enum in the loop machinery found. The
closest thing is a shell pseudo-enum:

```text
MODE=run
--selftest  -> MODE=selftest
--capabilities -> MODE=capabilities
-h|--help -> MODE=help
```

Those exact assignments are at `loop-kit/loop-start-chokepoint.sh:40-60`; the
dispatch is `help | capabilities | selftest | run` at `:224-237`. A bad value
returns usage/error. I would not describe this as a strict enum: it is a case
statement over a mutable shell variable, with no machine-checked vocabulary
outside that script.

### Forbidden phrases

The forbidden-phrase list equivalent to course-engine's
`standing by`, `queue empty`, `blocked on Josh`, and `wait_josh` is **absent**.
The read-only tracked-source search found none of those phrases and no
`forbidden_phrases`/`wait_josh` list in the zscast loop surfaces. The one
explicit “forbidden” rule I found is unrelated to loop mode: pane dispatch
rejects the non-canonical `--pane=N` selector (`scripts/lib/pane_dispatch.sh:23-30`).

What zscast does enforce instead is a set of *conditions*, not prose bans:
signed charter or refusal (`loop-kit/loop-start-chokepoint.sh:124-140`), latch
refusal (`:104-112`), explicit `PASS/FAIL/UNRUN` tokens
(`scripts/lib/gate_verdict.sh:32-50`), and no dispatch unless both activity and
pane text are safe (`scripts/lib/pane_dispatch.sh:70-106`). This is a different
and narrower enum/choke-point design than our phrase-level anti-stall rule.

## 3. What counts as an external ship signal?

### Proven: social launch, not an episode

The launch ledger is the strongest actual loop-#3 evidence in this repo. It
records a live X thread URL, four concrete tweet IDs, methods, and a LinkedIn
activity URL/URN (`.planning/launch-2026-05-18/ledger.md:75-109`). Those are
external identifiers a stranger can resolve. This is genuinely stronger than a
green internal gate or a local render.

It is also a different product event from publishing a live episode. The same
ledger describes the content as a launch post for the project, not a recorded
zscast stream (`.planning/launch-2026-05-18/ledger.md:34-73`, `:102-109`).

### Potential: media publication, but not evidenced in the corpus

The `publish` command is wired to the CLI (`bin/zscast:165-166`, `:876`). Its
YouTube branch gets a token, uploads bytes, extracts a `video_id`, constructs a
`https://www.youtube.com/watch?v=...` URL, and appends an audit event
(`lib/cmd-publish.sh:101-179`). R2/S3 branches hash the source bytes and write a
local receipt with sink, key, SHA-256, and completion time
(`lib/cmd-publish.sh:230-243`, `:246-303`, `:306-368`).

That is a useful *producer* contract, not yet an external-signal *verifier*:
the default is dry-run and the file calls itself “Scaffold-only”
(`lib/cmd-publish.sh:8-16`, `:17-29`, `:78-98`); no tracked publish-receipt or
smoke-receipt file exists in the repo; no `publish`/`_publish_*` test caller
was found in the tracked tests/hooks; and the 153 checked-in `receipts/*.json`
contain zero `external_signal`, `published`, `watch_url`, or `video_id` fields
(measured with `jq -s`). The source can do it; the evidence does not show that
the loop did it.

### Not a ship signal: smoke, render, or post-stream analysis

`smoke-youtube` is deliberately private and deletes the broadcast/stream on
exit (`lib/cmd-smoke-youtube.sh:12-29`, `:343-369`). Its cleanup trap records
`confirmed-deleted` or `delete_pending` and always writes a local receipt
(`lib/cmd-smoke-youtube.sh:269-337`). Its tests prove those local/API behaviors,
including a planted pending-delete row (`tests/test_zscast.bats:2626-2677`),
and `test-stream` proves only a private dry-run (`:2875-2941`). That is an
excellent safety test and a **bad ship certificate**: a deleted private object
cannot be watched by a real audience.

The post-stream path ingests local audit/events/ffmpeg logs into SQLite for
highlight analysis and Datasette (`scripts/post-stream-ingest.sh:6-27`,
`lib/cmd-post-stream.sh:27-55`, `:108-152`). It can tell us what a local run
recorded; it does not establish distribution or viewership.

The intended external gate is written down: three consecutive clean, sub-ten-
minute drills by three paid strangers, zero interventions, and a survey of at
least 4/5 (`docs/plans/zscast-product-experience-v1.md:587-601`). The plan also
requires viewer-side VOD inspection in the reconnect rubric
(`docs/plans/zscast-product-experience-v1.md:580-585`). I found no corresponding
episode receipt or tick row. Verdict: **the external-signal mechanism is weaker
than the design suggests; it is present as a planned contract and proven for
social launch, but absent as measured episode/viewer evidence.**

The named `zscast-ops` skill is only referenced as a plan location for future
validation gates (`docs/plans/zscast-product-experience-v1.md:296-309`); no
`zscast-ops` or `zscast-truth` skill file was present in the target checkout or
the searched remote skill directories. It cannot be counted as a wired gate.

## 4. Output-quality gates

### What is WIRED and fail-closed

The machine-verdict contract is genuinely wired into the `scripts/check.sh`
stages: it restricts stdout to anchored `PASS`, `FAIL`, and `UNRUN`, treats
UNRUN as incomplete, and refuses an empty gate set
(`scripts/lib/gate_verdict.sh:1-14`, `:32-50`, `:79-100`). Its self-test plants a
failure, checks the named anchored token, checks the empty-set error, and checks
that UNRUN cannot contain the GREEN token (`scripts/lib/gate_verdict.sh:102-134`).
The chain itself is ordered and fail-fast (`scripts/check.sh:1-7`,
`:1700-1775`).

The close-wiring gate has a stronger causal shape than a directory scan: it
creates a tracked-but-uninvoked artifact alongside a wired artifact and must
name the uninvoked artifact as `MANUAL`; it separately rejects a `MANUAL` result
as pass and rejects an empty domain (`scripts/lib/close_wiring_gate.py:196-225`,
`:227-275`, `:278-319`). The pre-commit hook invokes this gate when staged
`scripts/lib` paths are involved (`.git/hooks/pre-commit:49-75`).

The media smoke tests are also wired to the pre-push test surface: pre-push
invokes `ci-local.sh` (`.git/hooks/pre-push:45-54`), and `ci-local.sh` invokes
the bats suite (`scripts/ci-local.sh:345-424`). Their known-bads are cleanup
ledger state and provider/API errors, not a real viewer publication
(`tests/test_zscast.bats:2643-2677`, `:2709-2759`).

### What is PRESENT but not a shipped-artifact gate

No stage in the observed `scripts/check.sh` array consumes a YouTube/R2/S3
publication receipt or asks a delivery endpoint whether the object remains
available. The publish implementation is CLI-wired, but the publication proof
is not part of the ordered chain. The `operator-smoke`/live-camera family is
present as scripts and tests, but no caller to it appeared in the
`scripts/check.sh` stage list or local hook path; it is therefore not evidence
that a shipped media artifact was checked.

This matters because “rendered,” “uploaded,” and “watched” are three different
states:

| State | Evidence in zscast | Verdict |
|---|---|---|
| Rendered/encoded | `smoke-youtube` runs `ffmpeg` through the existing run surface and records health (`lib/cmd-smoke-youtube.sh:6-20`, `:317-335`). | Internal runtime evidence. WIRED to tests, not external. |
| Published/uploaded | YouTube creates a video ID/watch URL; R2/S3 write byte-digest receipts (`lib/cmd-publish.sh:157-179`, `:278-300`, `:341-365`). | Producer capability is WIRED to CLI; no checked-in execution receipt. |
| Watched/reached audience | Launch ledger has social URLs/IDs (`.planning/launch-2026-05-18/ledger.md:75-109`); episode receipts/view counts/human reports are absent from the measured corpus. | Proven for announcement; **not proven for an episode**. |

## 5. Measured history and Rule Zero

### Canonical ticks

The target has no `.flywheel/tick-ledger.jsonl`, and none of the 153 tracked
`receipts/*.json` contains a top-level `tick` key (read-only `find`, `git
ls-files`, and `jq -s` measurement). The receipt corpus is mostly
`flywheel.closeout-receipt/v1` (118 of 153); the remainder are GUI/P4/product
spike/worker receipts. A representative closeout identifies a bead and
validation commands but no tick or product-move field
(`receipts/zeststream-cast-27lu-cod4-20260821T030026Z.json:1-8`, `:36-70`);
the current gate receipt similarly records chain exit, stage verdicts, and
accepted risks (`receipts/zeststream-cast-c9-artifact-headers-bve9-cod4-20260821T045450Z.json:1-18`, `:46-117`).

The raw measurements are therefore:

| Measure | Result | Interpretation |
|---|---:|---|
| Canonical tick rows | **0** | No tick series exists to count or span. |
| Top-level receipt RED/FAIL statuses | **0/153** | Not a green result; the receipt schema is not the tick verdict schema. Many nested validation commands can have expected failures (`...27lu...:10-34`), but there is no canonical RED fraction. |
| Explicit `product_moved` fields | **0/153** | The corpus does not record Rule Zero at receipt level. This is not proof that no commit changed product bytes; it is proof that the loop did not measure the property. |
| Explicit external/published/watch fields | **0/153** | No checked-in episode ship/watch receipt. |
| Receipt-touching commits in the inspected 2026-08-20→2026-08-22 window | **38** | Receipts moved, but not as a tick ledger. The same window contained 331 commits in the remote HEAD history; commit activity is not tick activity. |

Thus the requested RED/product-move ratio is **N/A, 0/0 ticks**. Reporting
`0% RED` or `0% product move` would be the same category error as the current
course-engine stall: a missing observation window masquerading as a clean one.
For the weaker receipt-level proxy, the measured explicit product-move and
external-signal counts are both 0/153, with the qualification above.

### The actual historical ship row

The launch ledger has a dated, externally resolvable social ship on 2026-05-19
and marks its IDs/URNs as logged (`.planning/launch-2026-05-18/ledger.md:75-109`).
The agent-operable runbook, by contrast, has one dated pass-log baseline—860
canonical points and 4/15 runbook steps on 2026-06-14—and says to rerun both
harnesses each loop (`docs/runbooks/zscast-agent-operable-LEDGER.md:5-12`,
`:68-72`). That is a measured internal progress baseline, not a repeated
audience signal.

## 6. What zscast has that course-engine does not

| zscast mechanism | Would it catch our current stall? | Evidence and limit |
|---|---|---|
| Structural built-vs-wired classifier | **Partly.** It would catch a newly added local gate/helper with no caller, but not nine arbitrary commits landing while no tick is emitted. | The classifier makes WIRED/MANUAL/PARKED explicit and fails on MANUAL (`scripts/lib/close_wiring_gate.py:4-30`, `:287-319`). It is invoked by the installed pre-commit hook only for staged `scripts/lib` paths (`.git/hooks/pre-commit:49-75`), so it does not create a universal receipt boundary. |
| Live scheduler receipt with provenance, freshness, exit code, and anchored output | **Yes for a scheduler failure; no for the full stall.** It would expose a dead/failing watcher before dispatch, as the current live `--status` did. It would not force each product commit through a tick. | The wrapper writes schema/source/label/run/exit/verdict/output fields (`scripts/lib/pane_watch_launchd.sh:57-72`) and rejects stale/non-PASS output (`:100-123`). `pane_dispatch.sh` consumes that status and refuses dispatch on failure (`:38-67`). |
| Two-signal dispatch proof | **Only if installed as the commit/dispatch choke-point.** It catches false idle and undelivered prompts, not silent product commits. | It cross-checks activity with pane text (`scripts/lib/pane_dispatch.sh:70-106`), then requires send confirmation and output-sequence advance (`:108-146`). |
| Fail-closed tri-state gate vocabulary | **It would prevent a missing prerequisite from becoming a green transcript, but ours already has the more explicit product/value doctrine.** | zscast's shared contract is `PASS/FAIL/UNRUN`, with UNRUN blocking green (`scripts/lib/gate_verdict.sh:32-50`, `:79-99`). |
| External launch ledger with IDs/URLs/URNs | **No.** It would give us loop #3 evidence only after a real external publication; it would not by itself detect a frozen tick ledger. | zscast records X and LinkedIn identifiers (`.planning/launch-2026-05-18/ledger.md:75-109`). Course-engine explicitly requires a practitioner who did not build it and says the differential harness is not that signal (`.flywheel/CHARTER.md:138-142`); its queue forbids an agent from claiming the human mock (`.flywheel/QUEUE-2026-08-20.md:127-152`). |

## 7. What course-engine does better

Our doctrine is clearer about the difference between internal verification and
shipping. The Charter defines GREEN as either a learner-visible product move or
an observed denial of a real bad input, and requires a `value_added` sentence
(`.flywheel/CHARTER.md:36-45`). It says a green `check.sh` is only loop-#1
evidence and requires an outside practitioner for loop #3
(`.flywheel/CHARTER.md:5-11`, `:138-142`). The Q14 marker makes the external
human condition non-claimable by an agent (`.flywheel/QUEUE-2026-08-20.md:127-152`).

Our local writer also has the right *shape* for measuring product movement: it
computes movement from the named commit, applies a denylist, rejects fabricated
classes, and refuses a missing ledger (`.flywheel/STATE.md:79-86`). The bad
news is that the same state file says nothing currently invokes it
automatically (`.flywheel/STATE.md:72-86`). This is a case where our rule is
better specified but less operationally enforced.

The local Charter also makes the pause condition explicit: three consecutive
RED ticks must stop the loop (`.flywheel/CHARTER.md:225`). The local watchdog
observes that condition but its source says “observed not enforced”
(`.flywheel/watchdog.sh:36-42`), while the escalation wrapper writes an urgent
decision record rather than dispatching work (`.flywheel/watchdog-escalate.sh:63-103`).
The current URGENT file proves the exact failure mode: last ledger row, commit
count, and a 4-of-5 RED streak are recorded, but the loop still needs a human
decision (`.flywheel/URGENT_JOSH.md:3-24`). Our doctrine names the failure more
honestly; it does not yet make the boundary impossible to cross.

## 8. One mechanism not to adopt

Do **not** port zscast's launchd pane heartbeat as the course's product ship
signal. It is useful orchestration machinery, but it is a clock-paced
observer/scheduler with a live failure already recorded, not evidence that a
learner or viewer received anything (`scripts/lib/pane_watch_launchd.sh:2-12`,
`:75-123`). Porting it would add another process that can say “the loop is
alive” while leaving the external-validation gap untouched. It would also cut
against our explicit work-paced doctrine: the local watchdog says it must never
become a clock-paced dispatcher (`.flywheel/watchdog.sh:2-7`).

Likewise, do not count `smoke-youtube --cleanup auto` as a shipped episode. Its
deletion guarantee is exactly why it is a good safety test and exactly why it
cannot be a viewer signal (`lib/cmd-smoke-youtube.sh:289-305`).

## 9. Recommendation and hold point

Port first:

1. Define one external-delivery receipt schema for a real release: source
   commit/object ID, staged-byte digest, sink, external object ID/URL, upload
   response, and a post-upload GET/read result.
2. Add an audience leg where the product actually requires it: a human report,
   viewer-side observation, or another independently produced signal. “Rendered”
   and “uploaded” remain separate states.
3. Make the next loop boundary consume that receipt and fail closed when it is
   absent. A watcher may alert on a frozen ledger, but it must not manufacture
   loop-#3 credit.

This would catch the current class of failure only when coupled to the commit
boundary: no receipt/tick means no product credit and an explicit pause. The
zscast scheduler and close-wiring classifier are valuable secondary mechanisms;
neither is the missing external signal.

**Final verdict:** zscast's wired-vs-present discipline is real and worth
borrowing; its social launch ledger is a genuine external signal; its episode
publication/viewership loop is not evidenced by the current receipt corpus.
The RED/product-move ratio for zscast ticks is **N/A (0/0)**, not green. Our
course-engine loop has the better definition of “shipped” and the better
anti-vacuity/value doctrine, but the current stall demonstrates that the
definition is not yet enforced at the commit boundary.

**HOLD.**
