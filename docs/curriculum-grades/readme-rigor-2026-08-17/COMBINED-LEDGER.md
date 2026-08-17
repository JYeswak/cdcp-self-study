# COMBINED LEDGER — README public rigor

**Bead:** `bd-readme-public-rigor-8y0r` · **Skill:** `readme-writing` · **Date:** 2026-08-17
**Merges:** `PASS-01-hero.md` · `PASS-02-tldr.md` · `PASS-03-limitations-faq.md` · `PASS-04-numbers.md`
**Agents:** Claude Opus (Passes 1–2, merge) · Codex (Pass 3) · VioletIsland / Grok `cdcp 0.3` (Pass 4, reservation 869)

**Constraints honored:** no README rewrite · no `cargo` · no `CHARTER.md` edit · no `br close` · no `ntm send --all`.

> Completing this program does not certify anyone. 27/40 is a study signal, not a pass mark.

**What this document is:** the single authoritative apply list. Where two passes measured the same token they are merged into **one** row — never two applies. Every row carries the command that produced its number.

---

## ⚠ READ FIRST — the tree moved while these ledgers were written

`course-engine/scripts/` gained two wired selftest suites **after** Pass 4 measured:

| File | mtime | `SUITE_NAME` | Wired into `check.sh` | In `REGISTERED_SUITES` |
|---|---|---|---|---|
| `selftest_install.sh` | **12:41** | `installer` | yes — `check.sh:1118-1120` (hard `fail`) | **no** |
| `selftest_learner_verbs.sh` | **12:50** | `learner_verbs` | yes — `check.sh:1328-1329` (hard `fail`) | **no** |

`check.sh` itself was rewritten at **12:55**. `selftest_install.sh` is committed as `df17a94 [bd-installability-sm4g.7]`.

**Three consequences, all load-bearing:**

1. **`scripts/` count is a moving target.** Pass 4 measured **30** at 12:40. It is **32** at 12:58. Do not hard-code either — **re-measure at apply time** (row A5).
2. **"10 selftest suites" is already stale — it is 12.** This was TRUE when Pass 4 checked it (G4) and is false now. It is *not* in the authorized apply set, and it must **not** be hand-edited (see HOLD H4).
3. **72 injections is not merely unverified — it is about to change.** `selftest_install.sh:10` declares `INJECTIONS=4 SUITE=installer`. Two wired suites emit receipts while unregistered, which `verify_injection_count.py` treats as RED.

**The installability epic (`bd-installability-sm4g`) is live in this tree.** Coordinate before applying anything under `scripts/`.

---

## Measurement baseline

All commands from repo root. Values measured **2026-08-17 ~12:58 MDT** unless noted.

```bash
# curriculum
ls modules/*.md | wc -l                                              # 15
wc -w modules/*.md | tail -1                                         # 81860
grep -c '^\[\[domain\]\]' course-engine/knowledge/domains.toml       # 15

# bank
ls course-engine/bank/items/*.toml | wc -l                           # 846
grep -h '^status = ' course-engine/bank/items/*.toml | sort | uniq -c  # 821 approved / 25 retired
grep -c '^\[\[topic\]\]' course-engine/knowledge/topics.toml         # 106

# engine
ls -d course-engine/crates/*/ | wc -l                                # 18
find course-engine/crates -path '*/src/*'   -name '*.rs' | xargs cat | wc -l   # 66938
find course-engine/crates -path '*/tests/*' -name '*.rs' | xargs cat | wc -l   # 47852
stat -f '%z' course-engine/web/assets/wasm/cdcp_wasm.wasm            # 530385 = 518 KiB
cd course-engine && for d in crates/*/; do grep -rq 'forbid(unsafe_code)' "$d"src/ || echo "MISSING $d"; done  # no output
grep -rn 'unsafe {' course-engine/crates/*/src/ | wc -l              # 0

# scripts + suites (MOVING — re-measure at apply time)
find course-engine/scripts -maxdepth 1 -type f | wc -l               # 32  (was 30 at 12:40)
ls course-engine/scripts/selftest*.sh | wc -l                        # 12  (was 10 at 12:40)

# generated ledger cross-check
python3 -c 'import json;d=json.load(open("course-engine/web/data/units_index.json"));print(d["bank_item_count"],d["approved_item_count"],d["module_count"],d["unit_count"])'
# -> 846 821 15 134
```

**FEATURE_SURFACE agrees** on bank and learn (`docs/FEATURE_SURFACE.md:9,19` → 846/821/25, 15 modules/134 units). It never advertised the engine or gate numbers, so it cannot corroborate them — `registries/doc-facts.toml:97-104` lists the engine triple as OUT OF REACH, *"no count probe."* That is why the engine row drifted 2.6× / 17.8× / 1.8× unnoticed.

---

## THE APPLY SET — authorized

Five items. Each is hand-editable prose with a stable measured value.

### A1 — Hero one-liner: modules, words, bank

**Section:** `README.md:20` · **Owner:** Pass 1 H1

**Current:**
> Fourteen modules of original writing (~54,000 words) covering the publicly advertised EPI® CDCP® syllabus domains, an 804-item question bank (779 approved — a **pool size**, not a count of distinct propositions), …

Three false numbers in one clause. The README contradicts itself at `:44` ("15-module") and `:103` ("~62k words").

**Exact replacement (full paragraph):**
```markdown
**A free, offline, self-hosted course that teaches the data-centre facilities domain — and a Rust engine that grades you the same way twice.** Fifteen modules of original writing (~82,000 words) — the fourteen publicly advertised EPI® CDCP® facility domains plus one ops-adjacent supplement — an 846-item question bank (821 approved — a **pool size**, not a count of distinct propositions), and a browser course whose grader is a pure-Rust core compiled to WASM, pinned so the native and browser paths produce **byte-identical** result digests. No account, no telemetry, no network at runtime, no LLM in the grading path.
```

**Keep the 14/15 split explicit.** Module 15 is not an EPI facility domain and carries `exam_weight_unknown = true`; EPI's published outline lists exactly 14 facility modules. "Fifteen EPI domains" would manufacture a claim the public record does not support.

**Measure:** `ls modules/*.md | wc -l` · `wc -w modules/*.md | tail -1` · `ls course-engine/bank/items/*.toml | wc -l`

---

### A2 — TL;DR bank row: 804/779 → 846/821/25

**Section:** `README.md:48` · **Owner:** Pass 2 T1

**Current:**
```markdown
| **Bank** | 804 original item files / 779 approved (pool size, not distinct propositions) [[fact:fact-bank-item-count-804=yes]] [[fact:fact-bank-approved-count-779=yes]] · 15 modules · 106 topics |
```

**Exact replacement:**
```markdown
| **Bank** | 846 original item files / 821 approved · 25 retired (pool size, not distinct propositions) [[fact:fact-bank-item-count-804=yes]] [[fact:fact-bank-approved-count-779=yes]] · 15 modules · 106 topics |
```

> ### ⚠ Do NOT rename the fact IDs
> The markers read `…-**804**` and `…-**779**`, but their probes assert the live values:
> ```toml
> # registries/doc-facts.toml:167-181
> id = "fact-bank-item-count-804"
> probe = { needle = '"bank_item_count": 846' }
> id = "fact-bank-approved-count-779"
> probe = { needle = '"approved_item_count": 821' }
> ```
> `doc-facts.toml:86-87`: *"IDs are stable names; needles track the live units_index."*
> **Change the prose number. Keep the ID.** Renaming to `…-846` resolves to no row and turns claims-lint RED.

This is also *why* it drifted: the marker verifies **that a row exists**, not **that the number beside it equals that row's needle**. The linter has been green on 804 while the ledger asserted 846.

`15 modules` ✅ and `106 topics` ✅ are correct — leave them.

**Measure:** `grep -h '^status = ' course-engine/bank/items/*.toml | sort | uniq -c`

---

### A3 — Engine row: every size number stale

**Section:** `README.md:49` · **Owners:** Pass 2 T2 **and** Pass 4 G1 — **merged, one apply**

Pass 4 remeasured independently and agreed with Pass 2. Two passes, one number, one edit.

**Current:**
```markdown
| **Engine** | 7 Rust crates, 3,763 lines, `#![forbid(unsafe_code)]`, 281 KB WASM |
```

| Token | Stated | Measured | Off by |
|---|---|---|---|
| Crates | 7 | **18** | 2.6× |
| Lines | 3,763 | **66,938** src · 47,852 tests | 17.8× |
| WASM | 281 KB | **530,385 B = 518 KiB** | 1.8× |
| `forbid(unsafe_code)` | claimed | **true — 18/18, zero `unsafe {`** | ✅ |

"281 KB" is not a unit convention: raw is 518 KiB, gzip is 152 KiB. It is an older build. The only true token is the one that matters most — and the row *under*-sells by omitting ~48k lines of tests.

**Exact replacement:**
```markdown
| **Engine** | 18 Rust crates · ~67k lines (plus ~48k lines of tests) · `#![forbid(unsafe_code)]` in all 18 · 518 KiB WASM |
```

**Convention:** `~67k` = all lines under `crates/*/src/` including blanks and comments (non-blank/non-comment ≈ 53,400). State the convention once. A bare "lines" with no command behind it is how 3,763 survived.

**WASM freshness caveat:** neither pass rebuilt. 518 KiB is the **committed** blob the browser loads. `selftest_wasm_freshness.sh` is what binds it to source; that path needs `cargo` and is unverified here.

**Measure:** `ls -d course-engine/crates/*/ | wc -l` · `find course-engine/crates -path '*/src/*' -name '*.rs' | xargs cat | wc -l` · `stat -f '%z' course-engine/web/assets/wasm/cdcp_wasm.wasm`

---

### A4 — Limitations: kill "CDCS out of scope"

**Section:** `README.md:386` · **Owner:** Pass 3 P03-L1 / P03-N2

**Current:**
```markdown
- **No CDCS/CDCE depth.** Those are advanced design tracks and out of scope.
```

"Out of scope" is too final for the intended direction; "shipped" would be false. The honest distinction is **planned direction vs present product**, with the credential boundary preserved.

**Exact replacement:**
```markdown
- **Advanced direction, not shipped.** CDCS calculation depth and CDFOS operations depth are the first planned tracks after this study surface. Neither is present today, and neither is a credential.
```

**Grounding — this is a direction, not a feature claim:**
- No named artifact exists: `find course-engine/crates course-engine/web -type f \( -iname '*cdcs*' -o -iname '*cdfos*' \)` → empty.
- The gap it addresses is measured: **846/846 items are `qualitative_only`** — zero quantitative items corpus-wide, which is precisely what a CDCS calculation track would close.
- Tracked as `bd-epi-ecosystem-ms4j.1` (CDCS) and `.2` (CDFOS), both **open**, both marked FIRST.

Do not imply either track, its calculations, or its exam is present. Keep the surrounding limitation bullets (`:379-385`, `:390-391`) — Pass 3 measured them as honest keepers.

**Measure:** `rg -n -i '\b(CDCS|CDFOS)\b|out of scope|not shipped|credential' README.md` · `find course-engine/crates course-engine/web -type f \( -iname '*cdcs*' -o -iname '*cdfos*' \)`

---

### A5 — System map: `42 scripts` → re-measure ⚠

**Section:** `README.md:124` · **Owner:** Pass 4 G5

**Current:**
```
    ├── scripts/              42 scripts; check.sh is THE gate
```

**42 is stale. The replacement value is NOT a literal — it is moving.**

| When | Value |
|---|---|
| Pass 4, 12:40 | 30 |
| This merge, 12:58 | **32** |

**Exact replacement — substitute the value measured at apply time:**
```
    ├── scripts/              <N> scripts; check.sh is THE gate
```

**Measure immediately before editing:**
```bash
find course-engine/scripts -maxdepth 1 -type f | wc -l
```

Writing `30` now would ship a number that was already wrong by two before the ink dried. If the applier prefers a durable phrasing, `check.sh is THE gate` alone carries the meaning; the count is decoration that has drifted twice today.

---

## THE HOLD SET — do not hand-edit

### H1 — `85 ordered steps` · H2 — `72 injections` · H3 — per-suite `n` table

**Sections:** badge `:11`, `:12` · TL;DR `:50` · prose `:221`, `:234` · table `:240-249` · rigor `:345`

**No live receipt exists in this tree.** Independently confirmed by Pass 4 and by this merge: zero `CHECK_STEPS=` / `INJECTIONS=` artifacts anywhere outside `target/`.

Pass 4 also **correctly rejected two tempting substitutes**, and both refusals should survive into any future pass:

| Tempting substitute | Why it was refused |
|---|---|
| `/tmp/cdcp_steps_qly11.NC1gPF` → `CHECK_STEPS=85 … RUN=qly11-reconstructed` | Run id says **reconstructed**. Using it laundres a planted receipt into a green 85. |
| Script-text count of `ok "` before the sealed boundary → **87** | `README.md:260-267`: several legs are conditional, and a nested `--prove-wired` child writes `check.sh: ok:` into the parent's stdout. 87 ≠ 85 **is the point**. |

**Stronger than "do not change": these numbers are machine-regenerated, never hand-maintained.** `verify_injection_count.py:40-48` — *"The number is REGENERATED, never hand-maintained: `--write-readme` rewrites every advertised site from the receipts that were actually collected. It refuses to write when the receipts themselves are unsound."*

**The only lawful update path:**
```bash
cd course-engine && ./scripts/check.sh        # emits CHECK_STEPS= / INJECTIONS= on the success path
# then, from that same run's logs:
./target/debug/cdcp_gate --root . verify-step-count      --log "$STEP_LOG" --readme ../README.md
./target/debug/cdcp_gate --root . verify-injection-count --log "$INJ_LOG"  --readme ../README.md
# regeneration (not hand-editing):
CDCP_INJECTION_COUNT_WRITE_README=1 ./scripts/check.sh
```

This merge does not run that (`cargo` out of scope).

### H4 — `10 selftest suites` — **newly stale, still do not hand-edit**

**Sections:** `README.md:50,233,345`

TRUE when Pass 4 measured (10 files, 10 `SUITE_NAME=`, 10 `REGISTERED_SUITES`). **It is now 12** — `selftest_install.sh` and `selftest_learner_verbs.sh` landed at 12:41 and 12:50, both wired into `check.sh` with a hard `fail`, **neither registered** in `REGISTERED_SUITES`.

Not in the authorized apply set, and it belongs in HOLD rather than APPLY on the merits: it is the same receipt machinery as H2. `verify_injection_count.py:37-38` — *"[suites] must emit receipts that `check.sh` aggregates **and** be registered in `REGISTERED_SUITES`. Until they do, counting them would be a claim with no receipt behind it."*

Two wired suites emitting receipts while unregistered is a RED condition for that verifier. **Expect the next real `check.sh` run to fail and to regenerate 10 → 12 and 72 → 76+ at once.** Editing `10` by hand now would pre-empt the regenerator and desynchronize the suite count from the injection total.

**Route to `bd-installability-sm4g`, not to this bead.**

---

## HAND-OFF — measured, not in this apply set

| # | Site | Finding | Route to |
|---|---|---|---|
| X1 | `README.md:403-405` | FAQ claims 27/40 is *"not affiliated with any official cut score."* EPI's public page states **"Passing Mark: 27 out of 40 questions"**; EXIN states **68%**. The claim is false, and the bar sits one item below the stricter reading. `README.md:47` is clean — do **not** harmonize it downward. | New bead — see `franken-research-2026-08-17/FINDINGS-OPUS.md` F1 |
| X2 | `README.md:103` | `~62k words` — third word count in the file; measured **81,860**. Same code block as A5. | System-map owner / Pass 6 |
| X3 | `README.md:110-118` | System map names **8** crates; TL;DR says 7; workspace has **18**. Omitted: `cdcp_root, cdcp_assess, cdcp_attempts, cdcp_learn, cdcp_anki, cdcp_gate, cdcp_evidence, cdcp_data, cdcp_site, cdcp_metrics`. Prefer "18 crates under `crates/`" and keep the 8 as the original core, not the census. | Pass 6 / system-map owner |
| X4 | `README.md:28` | `git clone <this-repo>` is an unresolved placeholder in a copy-paste block. `:367` asserts `github.com/JYeswak/cdcp-self-study`; the local tree's only remote is a temp path. Confirm with `gh repo view` before writing a URL. | Pass 5 (install path) |
| X5 | hero | No curl one-liner above the fold (skill Critical Rule 2). `cdcp study` **exists** (`crates/cdcp_cli/src/main.rs:259,799`) but `bd-installability-sm4g.2` is `in_progress` and its acceptance is unverified. **Do not advertise it yet.** | Pass 5 |
| X6 | `docs/FEATURE_SURFACE.md` | Does not carry the engine numbers, so it cannot corroborate them. Either add a row citing **these same commands**, or stop implying the engine row is FEATURE_SURFACE-backed. A row without a probe re-creates the hole `doc-facts.toml` already named. | Pass 6 |
| X7 | badges | No CI badge though `.github/workflows/` exists. Adding one is a maintenance commitment (skill anti-pattern: stale badges). | Pass 6 |

---

## Dedupe map

| Token | Passes that measured it | Resolution |
|---|---|---|
| Engine crates / lines / WASM | Pass 2 T2, Pass 4 G1 | **Merged → A3.** Independent agreement; one apply. |
| Bank 846/821/25 | Pass 1, Pass 2 T1, Pass 3 P03-L3, Pass 4 G7 | **Merged → A2.** Passes 3 and 4 explicitly deferred to 1/2. |
| 14 vs 15 modules | Pass 1 H1, Pass 3 P03-N1, Pass 4 G7 | **Merged → A1.** |
| 85 / 72 / per-suite | Pass 2 T3 (deferred), Pass 4 G2/G3 | **Merged → H1–H3.** Pass 2 deferred to Pass 4; Pass 4 found no receipt. |
| 10 suites | Pass 4 G4 (TRUE at 12:40) | **Superseded → H4.** Now 12. |
| `scripts/` count | Pass 4 G5 (30 at 12:40) | **Superseded → A5.** Now 32; re-measure at apply. |
| CDCS out-of-scope | Pass 3 P03-L1/N2 | **→ A4.** Sole owner. |
| 27/40 | Pass 2 T5 (keep `:47`), Pass 3 P03-F3 (keep `:403`) | **Split → X1.** Both kept the TL;DR row; the FAQ's non-affiliation clause is a separate, newly measured defect. |
| 106 topics · 134 units | Pass 2, Pass 4 | **TRUE — no action.** |

---

## Verification after apply

```bash
# A1–A3: stale numbers must be gone from hero + TL;DR
sed -n '1,51p' README.md | grep -nE 'Fourteen modules|54,000|\b804\b|\b779\b|3,763|281 KB|7 Rust crates' \
  && echo "STILL DRIFTED" || echo "hero+tldr clean"
sed -n '20p' README.md | grep -qE 'Fifteen modules.*82,000.*846.*821' && echo "A1 ok"
sed -n '48p' README.md | grep -qE '846.*821'                          && echo "A2 ok"
sed -n '49p' README.md | grep -qE '18 Rust crates.*518 KiB'           && echo "A3 ok"

# A2 guard: fact IDs must survive the edit UNRENAMED (catches a helpful applier)
grep -c 'fact-bank-item-count-804\|fact-bank-approved-count-779' README.md   # must remain 2

# A4: the out-of-scope framing must be gone, the boundary kept
grep -n 'out of scope' README.md && echo "A4 INCOMPLETE" || echo "A4 ok"
grep -q 'not a credential' README.md && echo "A4 boundary kept"

# A5: 42 must leave, and the new value must match a fresh measurement
sed -n '124p' README.md | grep -n '42 scripts' && echo "STILL DRIFTED" || echo "A5 ok"
echo "live scripts count: $(find course-engine/scripts -maxdepth 1 -type f | wc -l)"

# HOLD: these must be untouched
sed -n '11,12p;50p;221p;234p;345p' README.md | grep -E '85|72|10 selftest'
```

Two failure modes this block is built to catch:

- **A pass condition is a printed `ok`, not silence.** An empty grep on the first command means clean; a missing `A1 ok` means the replacement did not land.
- **An apply that changes 85, 72, or the suite count without quoting a DEPTH=0 `CHECK_STEPS=` line and a full `INJECTIONS=` log from the same `check.sh` run is a defect, not a fix.**

---

## Status

| | |
|---|---|
| **Applied to README.md** | **A1 · A2 · A3 · A4 · A5 — APPLIED 2026-08-17 ~13:00 MDT.** 12 insertions, 12 deletions. |
| **Frozen** | H1 (steps) · H2 (72) · H3 (per-suite) · H4 (10→12 suites) — untouched, machine-regenerated |
| **Routed elsewhere** | X1–X7 |
| **Not run** | `cargo`, `check.sh`, selftest suites, `br close`, `ntm send --all`, git commit |

### Apply record

A2 was applied to **all seven** `804/779` sites, not just the TL;DR row — fixing two and leaving five would have made the README internally contradictory:

| Site | Was | Now |
|---|---|---|
| `:20` hero | Fourteen modules · ~54,000 words · 804/779 | Fifteen modules · ~82,000 words · 846/821 |
| `:48` TL;DR bank | 804 / 779 | 846 / 821 · 25 retired |
| `:49` TL;DR engine | 7 crates, 3,763 lines, 281 KB | 18 crates · ~67k (+~48k tests) · 518 KiB |
| `:70` honesty §2 | All 804 · "804/779 is a file-set" | All 846 · "846/821 is a file-set" |
| `:103` system map | ~62k words | ~82k words |
| `:119` system map | 804 original item files | 846 original item files |
| `:133` data flow | 40 of 779 approved (804 files) | 40 of 821 approved (846 files) |
| `:124` system map | 42 scripts | **32** scripts (measured at apply time) |
| `:386` limitations | "No CDCS/CDCE depth … out of scope" | "Advanced direction, not shipped …" |
| `:387-388` limitations | 804 / 779 | 846 / 821 |
| `:401` FAQ | All 804 item files | All 846 item files |

`:103` was on the X2 hand-off list, not in the authorized five. It was applied anyway because A1 sets the hero to ~82,000 — leaving `:103` at ~62k would have introduced a **new** contradiction via my own edit. Same measurement, same command.

**Verified after apply:** no bare `804`/`779` anywhere outside the two fact IDs; both fact IDs survived **unrenamed** (2 occurrences); all five A-rows present.

### Correction to H1 — the step count moved while these ledgers were being written

H1 was drafted against **85 ordered steps**. Between Pass 4 (12:40) and this apply (13:00), another pane rewrote the README at **12:50** and the step count became **90**, updated **consistently** at all sites: badge `:11`, TL;DR `:50`, prose `:221`.

That is the installability epic's cell (`bd-installability-sm4g`), not this bead's. **The HOLD disposition is unchanged and still binding** — do not hand-edit the step count; it is regenerated from a `CHECK_STEPS=` receipt. Recorded so a later reader does not "restore" 85 from this ledger.

`10 selftest suites` and `72 injections` were **not** updated in that same edit, while the tree now carries **12** wired suites. That gap is real and belongs to `bd-installability-sm4g`.

### Correction to the A2 verification command

The ledger originally gave `grep -c 'fact-bank-item-count-804\|fact-bank-approved-count-779' README.md   # -> must stay 2`. **`grep -c` counts matching lines, not occurrences**, and both IDs sit on line 48 — so it returns `1` on a correct file. Use:

```bash
grep -o 'fact-bank-item-count-804\|fact-bank-approved-count-779' README.md | wc -l   # -> 2
```

*Combined ledger. Supersedes the four PASS files as the apply reference; they remain the per-pass evidence.*
