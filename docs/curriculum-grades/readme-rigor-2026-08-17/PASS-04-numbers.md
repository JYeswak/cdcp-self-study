# PASS 04 — Advertised gate / engine numbers vs receipts and FEATURE_SURFACE

**Bead:** `bd-readme-public-rigor-8y0r` · **Skill:** `readme-writing` · **Date:** 2026-08-17
**Agent:** VioletIsland (Grok pane `cdcp` `0.3` / `%4`)
**Scope:** advertised **gate** and **engine** numbers in `README.md`, checked against
`course-engine/docs/FEATURE_SURFACE.md` and the receipt machinery the README
itself names. Ledger only.

No README rewrite. No `cargo`. No `CHARTER.md` edit. No `br close`. No `ntm send --all`.

> Completing this program does not certify anyone. 27/40 is a study signal, not a pass mark.

**Oracle for this pass:** a command per number. A number with no command is
unverified. A script grep is **not** a `CHECK_STEPS=` / `INJECTIONS=` receipt —
`README.md:260-267` is explicit about that, and this pass obeys it.

**Overlap (do not reopen):**
- Pass 1 already owns 14 vs 15 modules and 804/779 vs 846/821 in the hero.
- Pass 2 already owns the TL;DR bank row and **proposed** the engine-row
  replacement. Independent remasurement **agrees** with Pass 2 T2. This pass
  confirms; it does not invent a second engine number.
- Pass 2 T3 deferred `85` / `72` here. That is this cell.
- Pass 3 owns Limitations / FAQ / CDCS-out-of-scope.

---

## Verdict up front

| Claim | Advertised | FEATURE_SURFACE | Command result | Receipt? | Verdict |
|---|---|---|---|---|---|
| Rust crates | 7 | **absent** | **18** workspace members = 18 `crates/*/` | none (doc-facts: out of reach) | **STALE — apply** |
| Lines of Rust | 3,763 | **absent** | **66,938** `crates/*/src/` · **47,852** `crates/*/tests/` | none | **STALE — apply** (same replacement as Pass 2 T2) |
| WASM size | 281 KB | file present, **no size** | **530,385 B = 518 KiB** committed `web/assets/wasm/cdcp_wasm.wasm` | none this pass (freshness needs a rebuild; cargo forbidden) | **STALE — apply** |
| `#![forbid(unsafe_code)]` | claimed | absent | 18/18 crates; 0 `unsafe {` under `crates/*/src/` | n/a | **TRUE — keep** |
| Ordered gate steps | 85 at 4 parse sites | **absent** | 87 `ok "` tokens before the sealed boundary — **not** the receipt | **no live `CHECK_STEPS=` in this tree today** | **UNVERIFIED — do not change** |
| Selftest suites | 10 at 3 sites | **absent** | 10 `scripts/selftest*.sh`; 10 `REGISTERED_SUITES`; 10 `SUITE_NAME=` | n/a | **TRUE — keep** |
| Known-bad injections | 72 at 5 parse sites (floor 5) | **absent** | per-suite `n` cells **sum to 72** (circular) | **no live `INJECTIONS=` log** | **UNVERIFIED — do not change** |
| Per-suite `n` column | 6+2+1+2+8+5+6+7+33+2 | absent | same table; no suite receipts | **missing** | **UNVERIFIED — do not change** |
| `scripts/` count | 42 | **absent** | **30** files under `course-engine/scripts/` (no `__pycache__`) | n/a | **STALE — apply** |
| Bank 846/821/25 | README still says 804/779 | **846 / 821 / 25** | 846 files; 821 approved; 25 retired | `units_index.json` needles | **TRUE on FEATURE_SURFACE; README drift is Pass 1/2** |
| Learn units | 134 (M8 row) | **15 modules / 134 units** | `unit_count` 134 · `module_count` 15 | fact-learn-unit-count-134 | **TRUE** |
| Topics | 106 (bank row) | absent | 106 `[[topic]]` in `knowledge/topics.toml` | n/a | **TRUE** (Pass 2 already kept) |
| Approved-pool multiplier | not in README engine/gate | **~20.5×** | `821/40 = 20.525` | fact-approved-pool-multiplier-19-5 | **TRUE on FEATURE_SURFACE** |

`registries/doc-facts.toml:104` already records the hole: *"7 Rust crates, 3,763 lines, 281 KB WASM — no count probe."* FEATURE_SURFACE does not carry those three numbers either. The engine row is unguarded prose. That is how it drifted 2.6× / 17.8× / 1.8×.

---

## Measurement baseline (repo root; no cargo)

```bash
# crates
python3 - <<'PY'
from pathlib import Path
import re
t = Path("course-engine/Cargo.toml").read_text()
m = re.search(r"members\s*=\s*\[(.*?)\]", t, re.S)
names = re.findall(r'"([^"]+)"', m.group(1))
dirs = sorted(p.name for p in Path("course-engine/crates").iterdir() if p.is_dir())
print("workspace", len(names), "dirs", len(dirs), "delta", set(dirs) ^ {Path(n).name for n in names})
PY
# -> workspace 18 dirs 18 delta set()

# lines
find course-engine/crates -path '*/src/*'   -name '*.rs' | xargs cat | wc -l   # -> 66938
find course-engine/crates -path '*/tests/*' -name '*.rs' | xargs cat | wc -l   # -> 47852

# wasm (committed artifact the browser loads — not a rebuild)
stat -f '%z' course-engine/web/assets/wasm/cdcp_wasm.wasm                     # -> 530385
python3 -c 'print(round(530385/1024), "KiB")'                                 # -> 518 KiB
gzip -c course-engine/web/assets/wasm/cdcp_wasm.wasm | wc -c                   # -> 156529 (~153 KiB)

# forbid / unsafe
cd course-engine
for d in crates/*/; do grep -rq 'forbid(unsafe_code)' "$d"src/ || echo "MISSING $d"; done
# -> no output
grep -rn 'unsafe {' crates/*/src/ | wc -l                                     # -> 0

# suites
ls scripts/selftest*.sh | wc -l                                               # -> 10
rg -n '^SUITE_NAME=' scripts/selftest*.sh
# 10 names; wasm file is selftest_wasm_freshness.sh, receipt name is wasm-freshness
# REGISTERED_SUITES in scripts/verify_injection_count.py is the same 10-tuple

# scripts/
find course-engine/scripts -type f ! -path '*__pycache__*' | wc -l            # -> 30

# FEATURE_SURFACE bank / learn (cross-check, not this pass's apply)
ls course-engine/bank/items/*.toml | wc -l                                    # -> 846
grep -h '^status = ' course-engine/bank/items/*.toml | sort | uniq -c
# 821 approved / 25 retired
python3 -c 'import json; d=json.load(open("course-engine/web/data/units_index.json")); print(d["bank_item_count"], d["approved_item_count"], d["module_count"], d["unit_count"])'
# -> 846 821 15 134
grep -c '^\[\[topic\]\]' course-engine/knowledge/topics.toml                  # -> 106
python3 -c 'print(821/40)'                                                    # -> 20.525
```

FEATURE_SURFACE (`course-engine/docs/FEATURE_SURFACE.md:9,19`) already states
846/821/25 and 15/134. Those rows are honest. The README engine/gate row is not
mirrored there.

---

## G1 — Engine row: every size number is stale (CRITICAL)

**Section:** `README.md:49` (TL;DR **Engine**). Pass 2 T2 found the same three
stale numbers; remasured here so two passes are not two opinions.

**Current:**
```markdown
| **Engine** | 7 Rust crates, 3,763 lines, `#![forbid(unsafe_code)]`, 281 KB WASM |
```

| Token | Stated | Measured | Command |
|---|---|---|---|
| Crates | 7 | **18** | workspace `members` in `course-engine/Cargo.toml` (18) equals `ls -d course-engine/crates/*/` (18). `fuzz/` is `exclude`, not a member. |
| Lines | 3,763 | **66,938** src / **47,852** tests | `find … \| xargs cat \| wc -l` — same convention as Pass 2 (all lines, including blanks/comments). Non-blank src is 62,353; non-blank/non-`//` src is ~53,400. |
| WASM | 281 KB | **530,385 B = 518 KiB** | `stat -f '%z'` on the committed blob. Gzip of that blob is 156,529 B. 281 KB is neither raw nor gzip — it is an older build. |
| `forbid(unsafe_code)` | claimed | **true** | 18/18 crates; 0 `unsafe {` in `crates/*/src/` |

The system map at `README.md:110-118` names **eight** crates (`core bank assemble grade schedule wasm cli registry_check`) while the TL;DR says **seven**. Neither is the workspace. The ten crates the map omits are: `cdcp_root`, `cdcp_assess`, `cdcp_attempts`, `cdcp_learn`, `cdcp_anki`, `cdcp_gate`, `cdcp_evidence`, `cdcp_data`, `cdcp_site`, `cdcp_metrics`.

**Exact replacement** (identical in substance to Pass 2 T2; one apply, not two):

```markdown
| **Engine** | 18 Rust crates · ~67k lines (plus ~48k lines of tests) · `#![forbid(unsafe_code)]` in all 18 · 518 KiB WASM |
```

**Convention:** `~67k` = all lines under `crates/*/src/` (`wc -l`). State that once. A bare "lines" with no command is how 3,763 survived.

**WASM freshness (this pass's extra cell):** I did not rebuild. 518 KiB is the
committed `web/assets/wasm/cdcp_wasm.wasm` — the bytes a clone actually serves.
`scripts/selftest_wasm_freshness.sh` is what binds that blob to a
`wasm32-unknown-unknown --release` rebuild; that path is `cargo`, so it is
**unverified here**. FEATURE_SURFACE:19 only claims the file is present.

---

## G2 — 85 ordered steps: advertised consistently, no live receipt (DO NOT CHANGE)

**Section:** badge `README.md:11` · TL;DR `:50` · gate prose `:221` · (badge URL is a second parse on `:11`).
`cdcp_gate verify-step-count` floor is `MIN_STEP_SITES = 4`. Four parse sites hold. Good.

**What would verify it:** a DEPTH=0 line from **this** tree's `check.sh`:

```text
CHECK_STEPS=<n> OK=<a> SKIPPED=<b> NESTED_OK=<m> DEPTH=0 RUN=pid<pid>
```

with `n = a + b`, `a > 0`, `m > 0` (the nested child must have run, or a
transcript-grep would have been untested). Then
`cdcp_gate --root course-engine verify-step-count --log <that file> --readme README.md`.

**What I actually have:**

| Candidate | Why it is not a receipt |
|---|---|
| Live `check.sh` run | Not executed. Cargo is out of scope; `check.sh` compiles. |
| `course-engine/target/cdcp-substrate-probe/check_sh.log` | Nested probe, 13 lines, **no** `CHECK_STEPS=`. Ends in substrate-guard FAIL on a planted unlisted `.py`. |
| `/tmp/cdcp_steps_qly11.NC1gPF` (mtime 2026-08-15) | `CHECK_STEPS=85 OK=85 SKIPPED=0 NESTED_OK=5 DEPTH=0 RUN=qly11-reconstructed`. The run id says **reconstructed**. Using it would launder a planted receipt into a green 85. **Rejected.** |
| Script-text count of `ok "` before `# STEP-COUNT-RECEIPT-BOUNDARY` | **87** tokens on 87 lines. `skipped_step "` appears 7 times as the other arm of a conditional. This is the failure mode `README.md:260-267` names: several legs are conditional; a nested `--prove-wired` child writes `check.sh: ok:` into stdout; the receipt is written by the process that counted. **87 ≠ 85**, which is the point. |

**Disposition:** keep `85` until a real DEPTH=0 receipt from this tree disagrees.
Do not "fix" 85 to 87 from a grep. Do not keep 85 *because* the reconstructed
tmpfile said 85.

**Measure command for the next apply pass (the one allowed to run the gate):**

```bash
cd course-engine && ./scripts/check.sh
# on the success path the same process prints and writes CHECK_STEPS=
# then: ./target/debug/cdcp_gate --root . verify-step-count --log "$STEP_LOG" --readme ../README.md
```

This pass does not run that.

---

## G3 — 72 injections: five advertisement sites, circular table, no live log (DO NOT CHANGE)

**Section:** badge `:12` (two parses: alt text + shield URL) · TL;DR `:50` · prose `:234` · rigor table `:345`.
`verify_injection_count.py` `MIN_ADVERTISEMENT_SITES = 5`. Five parses hold. Qualifier
"shell" / "selftest" is on every site (`fact-injections-enforced` / bd-n7uk).
Rust `#[cfg(test)]` known-bad legs are **correctly excluded** — they emit no receipt.

**Per-suite `n` cells** (`README.md:240-249`):

| Suite (`SUITE_NAME=`) | Advertised `n` |
|---|---|
| `selftest_known_bad` | 6 |
| `selftest_l5` | 2 |
| `selftest_l5_honesty` | 1 |
| `selftest_l6_coverage` | 2 |
| `selftest_l7_objectives` | 8 |
| `selftest_reconstructed` | 5 |
| `selftest_orphan` | 6 |
| `selftest_doc_consistency` | 7 |
| `selftest_injection_count` | 33 |
| `wasm-freshness` | 2 |
| **sum** | **72** |

The sum matching the headline is **not** evidence. The gate exists because that
sum used to be folklore. Each suite must print `INJECTIONS=<n> SUITE=<name>`
after observing RED; `check.sh` tees those lines; `verify-injection-count`
compares the sum **and** each cell to the receipts.

`INJ_LOG` is `mktemp` inside `check.sh` (`:371`). It is not committed. No
leftover `INJECTIONS=` log exists under `course-engine/target/` or `docs/`.

I did not run the selftest suites. They mutate goldens/bank and restore; other
panes are live on this repo.

**Disposition:** keep `72` and the per-suite cells until a real injection log
from this tree disagrees. Do not regenerate from header comments — the Python
module's own docstring says three suites declare zero cases in headers yet
inject.

**Measure command for the next apply pass:**

```bash
cd course-engine && ./scripts/check.sh
# then: ./target/debug/cdcp_gate --root . verify-injection-count --log "$INJ_LOG" --readme ../README.md
# or: python3 scripts/verify_injection_count.py --log "$INJ_LOG" --readme ../README.md
```

---

## G4 — 10 selftest suites: true (KEEP)

**Section:** `README.md:50,233,345`.

```bash
ls course-engine/scripts/selftest*.sh | wc -l
# 10
rg -n '^SUITE_NAME=' course-engine/scripts/selftest*.sh
```

| File | `SUITE_NAME` | In `REGISTERED_SUITES` |
|---|---|---|
| `selftest_known_bad.sh` | `selftest_known_bad` | yes |
| `selftest_l5.sh` | `selftest_l5` | yes |
| `selftest_l5_honesty.sh` | `selftest_l5_honesty` | yes |
| `selftest_l6_coverage.sh` | `selftest_l6_coverage` | yes |
| `selftest_l7_objectives.sh` | `selftest_l7_objectives` | yes |
| `selftest_reconstructed.sh` | `selftest_reconstructed` | yes |
| `selftest_orphan.sh` | `selftest_orphan` | yes |
| `selftest_doc_consistency.sh` | `selftest_doc_consistency` | yes |
| `selftest_injection_count.sh` | `selftest_injection_count` | yes |
| `selftest_wasm_freshness.sh` | `wasm-freshness` | yes |

Roster, registration, and advertised suite count agree. `tests/publishability-bar.sh`
is deliberately not a suite (plants no known-bad; documented exclusion in the
Python module). Do not bump "10" if someone later counts that file.

---

## G5 — System map "42 scripts" is stale (MED)

**Section:** `README.md:124` (`scripts/              42 scripts; check.sh is THE gate`).

```bash
find course-engine/scripts -type f ! -path '*__pycache__*' | wc -l    # -> 30
```

30 files: 13 `selftest_*.sh` + `check.sh` + smokes + verifiers + `restore_safe.inc.sh`
+ `_run_build_learn.sh` + `build_web_wasm.sh` + `_module_page_template.html`.
Engine-wide `*.sh` outside `target/` and `.git` is a larger junk set (flywheel,
beads_compliance_audit raw extracts) and is **not** what the map is labelling.

**Exact replacement** (`README.md:124` cell):

```
    ├── scripts/              30 scripts; check.sh is THE gate
```

Optional: `30 files under scripts/` if the applier does not want to call the
HTML template a script.

---

## G6 — FEATURE_SURFACE vs the engine/gate claims

`course-engine/docs/FEATURE_SURFACE.md` is 20 lines. It states:

- bank **846 / 821 / 25** (~20.5×) — **measured true**
- Learn **15 modules / 134 units** — **measured true**
- WASM dual-path **present** as `web/assets/wasm/cdcp_wasm.wasm` — **file exists**; size not claimed
- `cdcp_grade` + goldens + `check.sh` wire — presence, not a step count

It does **not** state 7 crates, 3,763 lines, 281 KB, 85 steps, 10 suites, or 72
injections. `doc-facts.toml:97-104` lists the engine triple as OUT OF REACH
("no count probe"). So:

1. FEATURE_SURFACE is the honest surface for bank/learn counts.
2. FEATURE_SURFACE cannot corroborate the engine/gate numbers because it never
   advertised them.
3. The next apply pass should either add a FEATURE_SURFACE row that cites the
   **same commands** as this ledger, or stop implying the README engine row is
   backed by FEATURE_SURFACE. Adding a row without a probe re-creates the hole
   doc-facts already named.

No FEATURE_SURFACE edit in this pass.

---

## G7 — Adjacent advertised numbers this pass does not apply

Owned elsewhere; remasured so Pass 6 does not get two apply lists for one token.

| Token | Sites | Measured | Owner |
|---|---|---|---|
| 804 / 779 files | hero, bank row, honesty §, system map, data-flow, limitations, FAQ | **846 / 821** | Pass 1 / Pass 2 |
| Fourteen modules / 15 modules | hero vs TL;DR | **15** module files; 14 public EPI + ops-adjacent | Pass 1 |
| 106 topics | bank row | **106** | Pass 2 (kept) |
| 134 Learn units | M8 roadmap | **134** | already marked; FEATURE_SURFACE agrees |
| 27/40 study signal | TL;DR, FAQ | not a gate/engine number | Pass 2 / Pass 3 |

---

## Apply list — Pass 4

| # | Site | Change | Blocking? |
|---|---|---|---|
| G1 | `README.md:49` | `7 crates, 3,763 lines, 281 KB` → `18 crates · ~67k src lines (plus ~48k tests) · 518 KiB WASM`. Same as Pass 2 T2. **One apply.** | **Apply** |
| G2 | `README.md:11,50,221` | No change to `85` | Wait for a live `CHECK_STEPS=` receipt |
| G3 | `README.md:12,50,234,240-249,345` | No change to `72` or per-suite `n` | Wait for a live `INJECTIONS=` log |
| G4 | `README.md:50,233,345` | No change to `10` suites | — |
| G5 | `README.md:124` | `42 scripts` → `30 scripts` | **Apply** |
| G6 | FEATURE_SURFACE | No edit this pass | Optional later row **with** a probe |
| G7 | system-map crate list `:110-118` | Optional: stop naming eight crates as if they were the engine. Prefer "18 crates under `crates/`" and keep the eight as the original core, not the census. | Hand-off to Pass 6 / system-map owner |

**Not applied by this pass. Ledger only.**

---

## Coordination (this session)

- Agent Mail: project `/Users/josh/cdcp-self-study` (id 40). Identity **VioletIsland**.
  Exclusive reservation **869** on this file, reason `bd-readme-public-rigor-8y0r`.
  Thread `bd-readme-public-rigor-8y0r`, message 4376.
  Claude and Codex were **not registered** on this project when the thread was
  opened; the durable record is that thread. They were asked to
  `macro_start_session` on the absolute path and fetch.
- `ntm send cdcp --pane=0.1` and `--pane=0.2` (not `--all`, not this pane).
  JSON receipts: `delivered: 1` each. Both panes moved to THINKING after send.

---

## Verification after apply (whoever applies)

```bash
# engine triple must leave the TL;DR
sed -n '40,51p' README.md | grep -nE '7 Rust crates|3,763|281 KB' && echo "ENGINE STILL DRIFTED" || echo "engine prose clean"
sed -n '49p' README.md | grep -qE '18 Rust crates.*518 KiB' && echo "engine measured-true"

# 85 / 72 must still be receipt-shaped, not grepped
sed -n '11,12p;50p;221p;234p;345p' README.md | grep -E '85|72'

# 42 must leave the system map
sed -n '124p' README.md | grep -n '42 scripts' && echo "SCRIPTS STILL DRIFTED" || echo "scripts cell clean"

# do not treat absence of CHECK_STEPS= as confirmation of 85
```

An apply that changes 85 or 72 without quoting a DEPTH=0 `CHECK_STEPS=` line and
a full `INJECTIONS=` log from the same `check.sh` run is a defect, not a fix.

*Pass 4 ledger only. No README edit, no cargo, no CHARTER edit, no bead closed, no commit.*
