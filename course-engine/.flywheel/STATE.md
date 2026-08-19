# STATE — cdcp-self-study loop

**Updated:** 2026-08-19 · operational audit for `bd-0cjy.1`

## Current product state

The shipped product baseline remains the A truth patch and C approved-only
assembly. The current bank/export baseline is 957 item files and 929 approved
items. The curriculum grounding wave is still RED; this audit did not touch
`bank/items` or either remaining draft item.

Recent landed work relevant to this state:

- `ac65c94` regenerated the learner exports from the current 957-file,
  929-approved bank and refreshed the golden pins.
- `2a6d870` added `watchdog-escalate.sh` and a cron reader for stalled loops.
  It observes and escalates; it does not emit ticks or dispatch work.
- `4b07321` adds the local Rust tick emitter and its fail-closed tests;
  `T9` was emitted by that command for the prior audit commit `8286885`.
  The bead remains open until the final scoped checks are complete.

## Grounding-wave non-bank audit (`bd-0cjy.1`)

The historical 21 serialized aliases all resolve to real bank items. The
current `web/data/bank_items_seed42.json` contains 957 rows and is exact against
the 957 `bank/items` files: zero mismatches in stem, choices, key, explanation,
status, module, or source class. The earlier 904-row export was stale against
the then-measured 931-approved source snapshot (27 approved-pool rows behind)
and had zero table-of-contents-like stems; that result described stale data,
not a clean current bank.

| serialized id | source TOML | generated surface | current freshness | wave-era TOC stem |
|---|---|---|---|---|
| `bank-m13-q078` | `bank/items/m13-q078.toml` | `web/data/bank_items_seed42.json`; units check ref | exact | no |
| `bank-m13-q080` | `bank/items/m13-q080.toml` | `web/data/bank_items_seed42.json` | exact | no |
| `bank-m13-q081` | `bank/items/m13-q081.toml` | `web/data/bank_items_seed42.json`; units check ref | exact | no |
| `bank-m13-q096` | `bank/items/m13-q096.toml` | `web/data/bank_items_seed42.json`; units check ref | exact | no |
| `bank-m13-q100` | `bank/items/m13-q100.toml` | `web/data/bank_items_seed42.json`; units check ref | exact | no |
| `bank-m13-q101` | `bank/items/m13-q101.toml` | `web/data/bank_items_seed42.json`; units check ref | exact | no |
| `bank-m13-q102` | `bank/items/m13-q102.toml` | `web/data/bank_items_seed42.json` | exact | no |
| `bank-m13-q103` | `bank/items/m13-q103.toml` | `web/data/bank_items_seed42.json` | exact | no |
| `bank-m14-q114` | `bank/items/m14-q114.toml` | `web/data/bank_items_seed42.json`; units check ref | exact | no |
| `bank-m14-q115` | `bank/items/m14-q115.toml` | `web/data/bank_items_seed42.json`; units check ref | exact | no |
| `bank-m14-q120` | `bank/items/m14-q120.toml` | `web/data/bank_items_seed42.json`; units check ref | exact | no |
| `bank-m14-q129` | `bank/items/m14-q129.toml` | `web/data/bank_items_seed42.json`; units index (duplicated mirror) | exact | no |
| `bank-m14-q131` | `bank/items/m14-q131.toml` | `web/data/bank_items_seed42.json` | exact | no |
| `bank-m15-q137` | `bank/items/m15-q137.toml` | `web/data/bank_items_seed42.json` | exact | no |
| `bank-m15-q141` | `bank/items/m15-q141.toml` | `web/data/bank_items_seed42.json` | exact | no |
| `bank-m15-q146` | `bank/items/m15-q146.toml` | `web/data/bank_items_seed42.json` | exact | no |
| `bank-m15-q148` | `bank/items/m15-q148.toml` | `web/data/bank_items_seed42.json` | exact | no |
| `bank-m15-q149` | `bank/items/m15-q149.toml` | `web/data/bank_items_seed42.json` | exact | no |
| `bank-m15-q153` | `bank/items/m15-q153.toml` | `web/data/bank_items_seed42.json` | exact | no |
| `bank-m15-q154` | `bank/items/m15-q154.toml` | `web/data/bank_items_seed42.json` | exact | no |
| `mock40-q15` | `bank/items/m06-q015.toml` | `web/data/bank_items_seed42.json` only; absent from current 40-item mock pack and golden fixture | exact | yes — current stem is the ISO heading recall rewrite |

`units_index.json` is also fresh at 957/929 with 134 units and 337 distinct
check references: every reference resolves to an approved bank item. It carries
no stems. The current mock pack and golden fixture do not contain `mock40-q15`.
The web-data and golden paths were clean at audit time; no pane-2 regeneration
was contested or overwritten.

## Tick-loop truth

The ledger now has exactly 10 rows, `T0` through `T9`; the last row is `T9`
from the emitter, recording commit `8286885`. That commit changed only
`.flywheel/STATE.md`, so the receipt correctly computes
`product_moved=false` and records the caller's contrary claim as a
disagreement. The row is a real emitter output, not a hand-written repair.
Product-move rate, RED ratio, and value density derived from this ledger still
need a live controller to become a current per-tick series.

This repository cannot emit a tick today. There is no local `emit_tick`
function, command, loop-kit script, or reachable writer for
`.flywheel/tick-ledger.jsonl`. `watchdog.sh` reads the ledger; the escalation
wrapper appends `STALL` observations to `ALERT` and writes `URGENT_JOSH.md`.
Neither writes a `zs.tick-receipt` ledger row. A tidy `STATE.md` is therefore
not a running loop.

The local writer is directly invokable as `cdcp_gate emit-tick`. It computes
product movement from the named commit, applies the Charter denylist, rejects
fabricated classes and forbidden status prose, and refuses a missing ledger
instead of creating one. A live receipt for the prior audit commit is still
present, proving installation. Nothing currently invokes this command
automatically per tick: the watchdog remains an observer/escalator, and cron
does not dispatch the emitter. A running loop still needs its tick controller
to call `cdcp_gate emit-tick` once per completed tick.
