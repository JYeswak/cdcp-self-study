# CFS loop study: what its machinery does that ours does not

Read-only study of the CFS checkout on Studio. `CFS/` is an alias for that
checkout in this document; the client name is intentionally not repeated.

## Executive result

The short answer is not “CFS has a better tick guard.” It does not have a
local `emit_tick` or `tick_guard`, and its durable audit data is not a tick
ledger. What it does have is a wider operational perimeter around workers:
launchd entries for idle/stuck detection, a dispatch acknowledgement design,
typed topology/audit-row validation, and a large rule-sharding/manifest
surface. Most of the interesting local scripts are present but not wired.

Measured counts:

- **Local scripts:** 9 executable files are present in `CFS/.flywheel/scripts/`;
  **0 are operationally wired and 9 are orphaned as local runtime entrypoints**.
  “Orphaned” means no active launchd or installed Git hook reaches the local
  path. There are 12 plist files, but their `ProgramArguments` point at the
  shared external flywheel path, as the loop-revive plist demonstrates
  (`CFS/.flywheel/launchd/com.zeststream.flywheel-loop-revive.plist:7-15`). The
  peer-freeze plist is explicitly disabled (`CFS/.flywheel/launchd/ai.zeststream.peer-orch-freeze-monitor.plist:7-15`).
- **Rules:** the current tree has **103** `L*.md` shards, not the 105 stated in
  the dispatch packet. **0/103 are runtime-enforced by a CFS-local rule
  runner; 103/103 are prose/presentation shards.** The shard extractor checks
  frontmatter, IDs, and duplicates (`CFS/.flywheel/scripts/agents-md-shard-extract.sh:158-173`)
  and renders an index/manifest (`:176-197`, `:208-225`); that is integrity
  checking, not execution of the rule bodies.
- **Guard:** no local source contains `emit_tick` or `tick_guard`; the closest
  machine guards are cleanup mode/path/audit-row validation and topology-row
  validation. Therefore CFS has no evidenced rejection condition for a direct
  tick-log append, forbidden loop phrase, or escalation-after-N blockers.
- **History:** `.planning/audit/convergence-audit-ledger.jsonl` has 15 audit
  rows from `2026-07-02T19:00:00Z` through `2026-07-02T23:55:00Z`
  (`CFS/.planning/audit/convergence-audit-ledger.jsonl:1-15`). None is a
  `RED`/`GREEN` tick row, and none records a product-move field. The requested
  RED/product-move ratio is therefore **not computable from CFS’s real ledger**,
  rather than 0/0 being silently converted into a favorable ratio.

The single mechanism I would port first is a **durable, typed dispatch
acknowledgement gate**: a dispatch is not complete when text was sent; it is
complete only when a machine probe sees live worker evidence, otherwise the
gate emits a durable failure with diagnostics. CFS has this design in
`dispatch-and-verify.sh`, although the local copy is orphaned. It is the
closest missing control to our current “commits landed while the observation
loop stopped” failure. It must be joined to our tick choke-point, not used as
a substitute for it.

## 1. Machinery: present versus wired

The distinction matters here. A file in `.flywheel/scripts/` proves only that
someone installed it. A plist proves only what its XML names; it does not prove
that the named path is local, enabled, or semantically healthy.

| CFS artifact | Present behavior, with source evidence | Runtime wiring finding |
|---|---|---|
| `agents-md-shard-extract.sh` | Parses `L###` headings and requires `id`, `title`, `status`, `shipped`, and `trauma_class` frontmatter (`CFS/.flywheel/scripts/agents-md-shard-extract.sh:16-22`, `:134-173`). It writes shards, manifest, and generated indexes only in apply mode (`:279-298`). | No active local caller found. Manual generator, not loop choke-point. |
| `bead-quality-mining.sh` | A Python quality miner with a testable-verb/artifact-hint vocabulary (`CFS/.flywheel/scripts/bead-quality-mining.sh:4-25`). | Present, but no active local launchd or hook reaches it. |
| `cleanup-scratch.sh` | Its contract exposes dry-run/apply, absolute-path, mode, and audit-row validation (`CFS/.flywheel/scripts/cleanup-scratch.sh:132-142`). | Present, but no active local launchd or hook reaches it. |
| `dispatch-and-verify.sh` | Sends via `ntm`, probes activity/content/changes, waits with hysteresis, and exits with diagnostics when a pane remains stuck (`CFS/.flywheel/scripts/dispatch-and-verify.sh:14-23`, `:292-323`). | Present, but no active local launchd or hook reaches it. This is the most valuable orphan. |
| `publishability-bar.sh` | Scores a publishability audit and rejects low score, banned-word, or ungrounded-claim conditions (`CFS/.flywheel/scripts/publishability-bar.sh:8-15`, `:132-149`). | Reachable from the present prepublish wrapper (`CFS/.flywheel/scripts/zeststream-public-prepublish-hook.sh:46-67`), but that wrapper is not installed as an executable Git hook: `.git/hooks/` contains sample files only. So it is a static call edge, not operational wiring. |
| `sync-canonical-doctrine.sh` | It is explicitly halted, exits 64, and says it is no longer invoked by the canonical loop or launchd (`CFS/.flywheel/scripts/sync-canonical-doctrine.sh:1-18`). | Orphan by its own declaration. Its later copy loops are unreachable in the current file. |
| `tmp-aggressive-prune.sh` | Defines dry-run-by-default, apply-with-idempotency-key, retention age, and deny-list exclusions (`CFS/.flywheel/scripts/tmp-aggressive-prune.sh:1-25`). | Present, but no active local launchd target. The active tmp-prune plist points to the shared external path, not this file. |
| `topology-tick-refresh.sh` | Exposes doctor, health, repair, validate, audit, and why surfaces (`CFS/.flywheel/scripts/topology-tick-refresh.sh:12-32`); health reads row count and last JSON row (`:129-149`). | Present, but no active local launchd or hook reaches it. |
| `zeststream-public-prepublish-hook.sh` | Detects a public target, runs the publishability probe, and fails invalid JSON (`CFS/.flywheel/scripts/zeststream-public-prepublish-hook.sh:46-67`). | Wrapper present; no installed hook caller. |

The 12 local plist files are therefore misleading if read as a local runtime
map. The enabled entries name shared scripts outside the CFS checkout; the
freeze monitor is disabled (`CFS/.flywheel/launchd/ai.zeststream.peer-orch-freeze-monitor.plist:7-19`).
The loop-revive entry is also explicitly `--dry-run --write-receipt`
(`CFS/.flywheel/launchd/com.zeststream.flywheel-loop-revive.plist:7-15`), so
even that entry does not evidence an in-repo automatic repair.

## 2. The choke-point and actual rejection conditions

### What is absent

The CFS tree has no source-level `emit_tick`, `tick_guard`, `tick-ledger`, or
equivalent direct-append rejection path in the inspected `.flywheel`, scripts,
hooks, and tests. The charter calls itself a minimum alignment surface rather
than a full loop-engineering charter (`CFS/.flywheel/CHARTER.md:1-11`). That
matches the executable evidence: the topology script reads an audit log and
reports status, but does not emit or guard a product tick
(`CFS/.flywheel/scripts/topology-tick-refresh.sh:129-149`).

Consequently, the actual CFS loop-level rejection conditions are **none
evidenced** for:

- a direct append to a loop ledger;
- an invalid mode enum on a tick;
- forbidden phrases in a tick;
- an escalation after N blockers; or
- a missing product move hidden behind a green observation.

### The closest executable rejection conditions

These are real guards, but they guard cleanup/topology inputs, not ticks:

1. `cleanup-scratch.sh` rejects a non-absolute scratch path with
   `reason:"not_absolute_path"` (`CFS/.flywheel/scripts/cleanup-scratch.sh:300-309`).
2. It accepts only the literal mode values `dry-run|apply`; any other value is
   rejected with `reason:"not_in_enum"` and `valid_modes:["dry-run","apply"]`
   (`CFS/.flywheel/scripts/cleanup-scratch.sh:312-325`).
3. Its audit-row subject rejects an unreadable file with
   `reason:"file_not_readable"`, then rejects a row missing `ts` or `action`
   with `reason:"missing_required_fields"`
   (`CFS/.flywheel/scripts/cleanup-scratch.sh:327-340`).
4. `topology-tick-refresh.sh` rejects invalid or missing row JSON, and rejects
   valid JSON lacking all four fields `schema_version`, `ts`, `status`, and
   `run_id` (`CFS/.flywheel/scripts/topology-tick-refresh.sh:243-250`).
5. Its apply repair path refuses an apply without an idempotency key
   (`CFS/.flywheel/scripts/topology-tick-refresh.sh:152-170`) and reports an
   unknown repair scope rather than mutating it (`:172-193`).

This is useful input validation, not the missing loop choke-point. The
important negative result is that CFS has no source-backed forbidden-phrase
catalog or blocker-streak rejection at the tick boundary.

## 3. Escalation and resolution

CFS has a **per-dispatch** stuck hysteresis path, not a durable loop stall
path. `dispatch-and-verify.sh` increments `stuck_reads` for `STUCK` probes,
fires an empty Enter after two consecutive reads, and eventually exits 1 after
`MAX_PROBES` with activity, content, changes, and conflict diagnostics
(`CFS/.flywheel/scripts/dispatch-and-verify.sh:302-323`). The threshold is
configurable through `DISPATCH_VERIFY_MAX_PROBES` and the probe mode through
`DISPATCH_VERIFY_PROBE_MODE` (`:27-32`), so this is a caller-configurable
recovery nudge, not an un-silenceable escalation.

What I did **not** find in the CFS-local loop is a stall streak, durable urgent
file, desktop notification, or automatic resolution/rename equivalent to
ours. The enabled launchd files do show shared external stuck/idle detector
entrypoints, but the detector implementations are outside this read-only CFS
scope; the plist proves invocation wiring, not their rejection semantics. The
only inspected local “revive” entry is dry-run (`CFS/.flywheel/launchd/com.zeststream.flywheel-loop-revive.plist:7-15`).

Our loop has the stronger explicit escalation record. It checks the charter’s
three-in-five RED pause condition (`course-engine/.flywheel/watchdog.sh:36-40`),
increments a durable streak and writes `ALERT` (`course-engine/.flywheel/watchdog-escalate.sh:54-61`),
and on threshold writes `URGENT_JOSH.md` plus a desktop notification
(`course-engine/.flywheel/watchdog-escalate.sh:63-102`). It also resolves on
the next healthy observation by renaming, never deleting, the urgent file
(`course-engine/.flywheel/watchdog-escalate.sh:47-51`, `:96-97`).

That is an important “ours does better” result, with a qualification: our
watchdog currently observes the pause condition without enforcing it
(`course-engine/.flywheel/watchdog.sh:36-40`), so better detection does not
mean the loop actually stopped. CFS’s dispatch probe is more concrete at its
narrow worker-start boundary; ours is more explicit and durable at loop-level
escalation.

## 4. Measured history: what the data does and does not prove

### CFS’s actual ledger

The 15-row convergence ledger spans 4 hours 55 minutes. It records rounds,
defect counts, decisions, convergence curves, and validation notes. For
example, the first row records 20 defects and a resolution decision
(`CFS/.planning/audit/convergence-audit-ledger.jsonl:1`), while the final row
declares convergence after a second zero round (`:15`). It contains no
`verdict:"RED"`/`"GREEN"` tick schema and no product-move field. The 15 rows are
thus **15 audit observations, 0 RED-coded ticks, and an undefined product-move
denominator**.

The 24 receipt files are also closeout/validation receipts, not tick rows. Their
status inventory was 6 `GREEN`, 3 `CLOSED_CALLBACK_SENT`, 1
`DONE_BR_CLOSED_CALLBACK_SENT`, 1 `DONE_BR_CLOSED_CALLBACK_SENT_RELEASED`, 2
`complete`, 1 `implemented`, 2 `pass`, and 8 with no top-level status. That is
evidence of receipt variety, not evidence of product velocity.

### Is the historical document churn still visible?

Not in the current Git window. There is one commit after 2026-08-07:
`3096024bbdfa2e4526736d00592189f1499e69c5` on 2026-08-18, deleting two
workflow files. The current checkout has 629 total commits, but only that one
commit in the 14-day window. The previous commit is 2026-08-02 and adds binary
assets (`84a2c29d7ba4f075e86945b2b91e677a33dccf8f`).

This means the stated historical “2,354 commits / eleven-day block” pattern is
not observable as a current 14-day product-vs-document ratio in this checkout.
It does **not** establish that the loop was fixed: the CFS-local audit ledger
never measured that ratio, and the current Git quietness could simply mean the
repo is between changes. The only defensible RED/product-move report from real
ledger data is **N/A: no RED field and no product-move field**.

## 5. The 103-rule directory: enforcement versus graveyard

The directory currently contains 103 shards. They are Markdown documents with
frontmatter and prose bodies. The extractor’s real enforcement is structural:
it loads every `L*.md`, errors on a missing heading, checks required
frontmatter, detects ID mismatch/duplicates, and writes a manifest/index
(`CFS/.flywheel/scripts/agents-md-shard-extract.sh:134-173`, `:267-298`). It
does not parse or execute the rule body.

The sampled rules explicitly *describe* mechanical gates. For example, L70
claims a gate that chains phases and fails when `ticks_punted_count >= 1`
(`CFS/.flywheel/rules/L024-L70-orch-no-punt-next-actionable-runs-same-tick-not-next-tick.md:82-91`),
and L105 claims a detector that auto-routes process gaps to beads
(`CFS/.flywheel/rules/L059-L105-process-gaps-are-measured-and-auto-routed.md:12-28`).
L110 describes required contract fields and says tick close gates refuse when
they are absent (`CFS/.flywheel/rules/L064-L110-substrate-primitives-declare-self-repair-loop.md:25-48`).
Those are strong prose contracts, but the CFS-local code search found no
runtime consumer of those rule IDs or bodies beyond the shard extractor.

Therefore the ratio is **0/103 runtime-wired rules : 103/103 prose/indexed
rules**. This is not a claim that the shared external flywheel lacks those
tools; it is a claim about what is wired from this repository. A directory
listing alone would have incorrectly called all 103 “active.”

## 6. What CFS has that our loop does not

| Mechanism evidenced in CFS | Absent from our `.flywheel/` | Would it have caught the current 97-stall / nine-commit episode? |
|---|---|---|
| **Dispatch acknowledgement with hysteresis and diagnostic failure.** A worker-start claim is tested with activity/content/changes; two consecutive stuck reads get a nudge; the final failure includes snapshots (`CFS/.flywheel/scripts/dispatch-and-verify.sh:292-323`). | Our named local surfaces include watchdog/escalation and tick artifacts, but no equivalent dispatch-start acknowledgement gate. | **No, not alone.** It would catch a pane that accepted text but never started, not commits bypassing tick emission. It becomes relevant only when attached to tick close. |
| **Typed topology/audit row validation.** Missing/invalid JSON and missing `schema_version`, `ts`, `status`, or `run_id` fail (`CFS/.flywheel/scripts/topology-tick-refresh.sh:243-250`). | Our current watchdog reads the tick ledger’s verdict and ready queue; the cited local surface does not validate a row schema before accepting it. | **Partly.** It could reject malformed rows, but would not reject a well-formed absent row. |
| **Idempotency-key/refusal contract for repair.** Apply without a key returns a refusal and code 3 (`CFS/.flywheel/scripts/topology-tick-refresh.sh:152-170`). | No equivalent repair contract is named in our watchdog path. | **No.** The failure was observation bypass, not duplicate repair. |
| **Launchd perimeter for idle/stuck worker checks.** Multiple plists invoke shared idle/stuck detector entrypoints; the local peer monitor is disabled (`CFS/.flywheel/launchd/ai.zeststream.peer-orch-freeze-monitor.plist:7-19`). | Our loop has no evidenced worker-pane detector in `course-engine/.flywheel/`. | **No, not directly.** The current evidence says work continued and commits landed; it does not say a pane was idle. |
| **Canonical rule-shard manifest.** Rule count, per-rule IDs/status, and a round-trip hash are emitted (`CFS/.flywheel/scripts/agents-md-shard-extract.sh:208-225`). | Our loop has doctrine and state files, but not this CFS-local 103-shard manifest contract. | **No.** A manifest cannot force a tick or stop commits. |
| **Public prepublish semantic bar.** The wrapper only invokes the bar for a public target and rejects malformed probe JSON (`CFS/.flywheel/scripts/zeststream-public-prepublish-hook.sh:53-67`); the bar rejects low score, banned words, and ungrounded claims (`CFS/.flywheel/scripts/publishability-bar.sh:132-149`). | Our current loop does not have an analogous content/publishability gate. | **No.** It guards public copy, not loop liveness. |

The mechanism I would port first remains **dispatch acknowledgement**, but only
as a subordinate signal to `emit_tick -> tick_guard`: successful worker-start
evidence should be one required input to a tick, and timeout should create a
durable, actionable observation rather than silently leave the ledger stale.

## 7. What not to adopt

Do **not** copy the CFS pattern of a large prose rule directory as if volume
were enforcement. The current evidence is 103 shards, 0 CFS-local semantic
consumers, and a generator that validates metadata rather than rule behavior
(`CFS/.flywheel/scripts/agents-md-shard-extract.sh:158-173`, `:267-298`). Our
current stall is exactly the kind of failure a rule can name while the runtime
continues past it. Port the typed gate and its receipt, not the graveyard.

Also do not port a clock-driven revive as the first response. The CFS plist’s
revive entry is explicitly dry-run (`CFS/.flywheel/launchd/com.zeststream.flywheel-loop-revive.plist:7-15`),
and our charter says the watchdog observes rather than dispatches
(`course-engine/.flywheel/watchdog.sh:4-9`). Adding a timer that launches more
work while the tick choke-point is bypassed would increase the exact
“commits while blind” failure, not resolve it.

## Bottom line

CFS’s distinctive strength is **operational acknowledgement at the worker
boundary**, plus typed, idempotent probes around the edges. Its distinctive
weakness is that many of those mechanisms are only present, and its own local
ledger cannot answer RED/product-move questions. Our loop does better on
durable loop-level RED detection, urgent escalation, and rename-on-resolution;
it currently fails to enforce its own pause condition and lacks a hard,
machine-checked path that makes work visible before it lands.

The first port should therefore be a measured dispatch/tick acknowledgement
gate, not more prose, more timers, or more rule shards.
