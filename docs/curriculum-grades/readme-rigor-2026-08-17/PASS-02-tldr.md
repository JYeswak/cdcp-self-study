# PASS 02 — TL;DR table honesty

**Bead:** `bd-readme-public-rigor-8y0r` · **Skill:** `readme-writing` · **Date:** 2026-08-17
**Scope:** `README.md:40-51` — the eight-row TL;DR table.
**Ledger only.** No README rewrite. No `cargo`. No `CHARTER.md` edit. No `br close`. No `ntm send --all`.

> Completing this program does not certify anyone. 27/40 is a study signal, not a pass mark.

**Verdict up front:** 4 rows are true, 1 row is stale on two of three numbers, 1 row is stale on **every** number, and 2 rows are deferred to Pass 4. The engine row is the worst claim in the README.

---

## Row-by-row

| Row | Line | Status |
|---|---|---|
| **What** | 44 | ✅ **TRUE — keep verbatim** |
| **Who it's for** | 45 | ✅ no measurable claim |
| **What it is not** | 46 | ✅ no measurable claim |
| **Study bar** | 47 | ✅ **TRUE — keep verbatim** (see T5) |
| **Bank** | 48 | ❌ **T1 — 804/779 → 846/821** |
| **Engine** | 49 | ❌ **T2 — every number stale** |
| **Gate** | 50 | ⏸ **T3 — 10 suites verified; 85 / 72 are Pass 4** |
| **Runtime deps** | 51 | ✅ TRUE |

---

## T1 — Bank row: two stale numbers, and a marker trap (CRITICAL)

**Section:** `README.md:48`

**Current:**
```markdown
| **Bank** | 804 original item files / 779 approved (pool size, not distinct propositions) [[fact:fact-bank-item-count-804=yes]] [[fact:fact-bank-approved-count-779=yes]] · 15 modules · 106 topics |
```

Measured: **846** files / **821** approved. `15 modules` ✅ and `106 topics` ✅ are both correct.

### ⚠ Do NOT rename the fact IDs

The markers read `fact-bank-item-count-**804**` and `fact-bank-approved-count-**779**`, but their registry probes assert the **live** values:

```toml
# course-engine/registries/doc-facts.toml:167-181
id = "fact-bank-item-count-804"
probe = { kind = "file_contains", path = "web/data/units_index.json", needle = '"bank_item_count": 846' }

id = "fact-bank-approved-count-779"
probe = { kind = "file_contains", path = "web/data/units_index.json", needle = '"approved_item_count": 821' }
```

`doc-facts.toml:86-87` states the intent: *"IDs are stable names; needles track the live units_index."*

**Change the prose number. Keep the ID exactly as written.** Renaming the marker to `…-846` resolves to no registry row and turns claims-lint RED.

This is also *why* the row drifted: the marker verifies **that a registered row exists**, not **that the number beside it agrees with that row's needle**. The linter has been green on `804` while the ledger asserted `846`. Filed as `FINDINGS-OPUS.md` F3; not this pass's to fix structurally.

**Exact replacement:**
```markdown
| **Bank** | 846 original item files / 821 approved · 25 retired (pool size, not distinct propositions) [[fact:fact-bank-item-count-804=yes]] [[fact:fact-bank-approved-count-779=yes]] · 15 modules · 106 topics |
```

**Measure command:**
```bash
ls course-engine/bank/items/*.toml | wc -l                                   # -> 846
grep -h '^status = ' course-engine/bank/items/*.toml | sort | uniq -c        # -> 821 approved / 25 retired
grep -c '^\[\[domain\]\]' course-engine/knowledge/domains.toml               # -> 15
grep -o '"unit_count": *[0-9]*' course-engine/web/data/units_index.json      # -> 134 (cross-check)
grep -o '"bank_item_count": *[0-9]*'  course-engine/web/data/units_index.json  # -> 846
grep -o '"approved_item_count": *[0-9]*' course-engine/web/data/units_index.json # -> 821
```

Adding `· 25 retired` is optional but recommended: it makes 846 − 821 legible and pre-empts "why don't these add up."

---

## T2 — Engine row: every number is stale (CRITICAL)

**Section:** `README.md:49`

**Current:**
```markdown
| **Engine** | 7 Rust crates, 3,763 lines, `#![forbid(unsafe_code)]`, 281 KB WASM |
```

| Claim | Stated | Measured | Off by |
|---|---|---|---|
| Crates | 7 | **18** | 2.6× |
| Lines | 3,763 | **66,938** (`crates/*/src/`) | 17.8× |
| WASM | 281 KB | **530,385 B = 518 KiB** | 1.8× |
| `forbid(unsafe_code)` | claimed | **true — 18/18 crates, 0 `unsafe {`** | ✅ |

**"281 KB" is not a different unit convention.** The committed artifact is 518 KiB raw and 152 KiB gzipped. 281 KB is neither — it is simply a number from an older build.

The only true element of this row is the one that matters most (`forbid(unsafe_code)`), and it is *under*-sold: the row omits ~48k lines of in-tree tests.

**Exact replacement:**
```markdown
| **Engine** | 18 Rust crates · ~67k lines (plus ~48k lines of tests) · `#![forbid(unsafe_code)]` in all 18 · 518 KiB WASM |
```

**Measure command:**
```bash
cd course-engine
ls -d crates/*/ | wc -l                                                        # -> 18
find crates -path '*/src/*'   -name '*.rs' | xargs cat | wc -l                 # -> 66938
find crates -path '*/tests/*' -name '*.rs' | xargs cat | wc -l                 # -> 47852
stat -f '%z' web/assets/wasm/cdcp_wasm.wasm                                    # -> 530385  (518 KiB)
for d in crates/*/; do grep -rq 'forbid(unsafe_code)' "$d"src/ || echo "MISSING $d"; done  # -> no output
grep -rn 'unsafe {' crates/*/src/ | wc -l                                      # -> 0
```

**Convention note for the applier:** `~67k` counts all lines under `crates/*/src/` including blanks and comments (non-blank/non-comment is ~53,400). Whichever is chosen, state it once and use the same convention everywhere. Quoting a bare "lines" number with no command behind it is how this row drifted 17× in the first place.

**WASM freshness caveat:** I did not rebuild (`cargo` is out of scope this pass). `518 KiB` is the size of the **committed** `web/assets/wasm/cdcp_wasm.wasm` — which is the artifact a reader can verify and the one the browser loads. The `wasm-freshness` selftest is what binds it to source; that binding is Pass 4's cell.

---

## T3 — Gate row: one third verified, two thirds deferred (PASS/DEFER)

**Section:** `README.md:50`

**Current:**
```markdown
| **Gate** | 85 ordered steps; 10 selftest suites; 72 known-bad injections (shell selftest suites) that must all go RED |
```

**Verified this pass — `10 selftest suites` is TRUE:**
```bash
ls course-engine/scripts/selftest*.sh | wc -l    # -> 10
```
`selftest_doc_consistency · selftest_injection_count · selftest_known_bad · selftest_l5 · selftest_l5_honesty · selftest_l6_coverage · selftest_l7_objectives · selftest_orphan · selftest_reconstructed · selftest_wasm_freshness`

**Deferred — `85 ordered steps` and `72 injections` are PASS 4's cell.** Both are receipt-enforced (`CHECK_STEPS=` / `INJECTIONS=`) and the receipts are only emitted by a real `check.sh` run, which this pass may not perform. **No change proposed.** Confirming them by any means other than the receipt would re-derive the number the guard exists to own — and `README.md:260-267` is explicit that a transcript grep over-counts because nested `--prove-wired` children write `check.sh: ok:` lines into the parent's stdout.

**Hand-off to Pass 4:** verify `85` and `72` from the receipts, not from the script text, and confirm the per-suite `n` column at `README.md:240-249` still agrees.

---

## T4 — "What" row: correct, and the 14/15 distinction is load-bearing (KEEP)

**Section:** `README.md:44`

```markdown
| **What** | 15-module data-centre facilities curriculum (14 public EPI domains + 1 ops-adjacent supplement) + offline course engine |
```

**Keep verbatim.** This is the sentence `README.md:20` should have been (Pass 1, H1). It is also externally correct: EPI's published CDCP outline lists exactly 14 facility modules and no 2.1 module, so "14 public EPI domains + 1 ops-adjacent supplement" is the honest decomposition. Do not "simplify" it to "15 EPI domains."

---

## T5 — "Study bar" row: correct as written (KEEP)

**Section:** `README.md:47`

```markdown
| **Study bar** | Mock exam 40 questions / 60 minutes / **27 correct is a study signal, not a pass mark** |
```

**Keep verbatim.** 40 questions / 60 minutes matches the public EPI and EXIN exam format, and the row makes the weaker, defensible claim ("study signal, not a pass mark") rather than a non-affiliation claim.

**Contrast with `README.md:403-405`,** which does claim 27/40 is *"not affiliated with any official cut score"* — contradicted by EPI's published *"Passing Mark: 27 out of 40 questions."* **That is the FAQ, not the TL;DR.** This row is clean; do not "harmonize" it downward. Filed as `FINDINGS-OPUS.md` F1 for Pass 3.

---

## T6 — Structural note for Pass 6 (no action)

The skill's TL;DR shape is **Problem → Solution → "Why use X?" feature table**. This README puts a what/who/what-it-is-not table at `:40-51` and its problem statement *below* it at `:55` (`## The problem this solves`). That section is strong — it names the real pain ("self-study tools lie to you") — but a scanner meets the spec sheet before the pain.

**No change proposed by this pass.** A rigor pass moves numbers toward truth; re-ordering the sales funnel is a different decision and belongs to Pass 6 with the pre-publish checklist.

---

## Apply list — Pass 2

| # | Site | Change | Blocking? |
|---|---|---|---|
| T1 | `README.md:48` | `804/779` → `846 / 821 · 25 retired`. **Keep fact IDs unchanged.** | **Apply** |
| T2 | `README.md:49` | `7 crates, 3,763 lines, 281 KB` → `18 crates · ~67k lines (plus ~48k tests) · 518 KiB` | **Apply** |
| T3 | `README.md:50` | No change — 85 / 72 are Pass 4 | Defer |
| T4 | `README.md:44` | No change | — |
| T5 | `README.md:47` | No change | — |
| T6 | `README.md:40-55` | No change — Pass 6 | Defer |

**Not applied by this pass. Ledger only.**

---

## Verification after apply

```bash
# stale numbers must be gone from the TL;DR block
sed -n '40,51p' README.md | grep -nE '804|779|3,763|281 KB|7 Rust crates' && echo "STILL DRIFTED" || echo "tldr clean"

# corrected numbers must be present
sed -n '48p' README.md | grep -qE '846.*821' && echo "bank ok"
sed -n '49p' README.md | grep -qE '18 Rust crates.*518 KiB' && echo "engine ok"

# and the fact IDs must have survived the edit unrenamed
grep -c 'fact-bank-item-count-804\|fact-bank-approved-count-779' README.md   # -> must stay 2 in this row
```

The last command is the one that catches a well-meaning applier renaming the markers to match the new numbers. **The ID keeps the old name; only the prose number changes.**

*Pass 2 ledger only. No README edit, no cargo, no CHARTER edit, no bead closed, no commit.*
