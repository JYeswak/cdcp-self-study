# README public-rigor ledger

**Bead:** `bd-readme-public-rigor-8y0r`
**Measured:** 2026-08-17 against HEAD `b90a959` + this apply
**Consumer:** the next agent who has to change an advertised number. Every row is a decision: keep, update, or hold.

> Completing this program does not certify anyone. 27/40 is a study signal, not a pass mark. [[claim:claim-study-signal-27]] [[claim:claim-not-epi-certified]]

This file is the apply list. Research passes (`PASS-0*.md`, `GRADE-ROUND-01.md`) stay uncommitted so they cannot trip `doc-facts`.

---

## How to re-run (the process)

From the corpus root (`cdcp-self-study/`):

1. Re-measure every command in the table below. Do not copy a number from an older ledger.
2. If a live value disagrees with `README.md` or the GitHub about blurb, update **only those rows**.
3. Bank file-set / approved-pool IDs are stable names. Change the **prose** number; do **not** rename `fact-bank-item-count-804` or `fact-bank-approved-count-779`. Their needles already track the live `units_index.json`.
4. Do not hand-edit the gate step count, the known-bad injection total, or the registered-suite count. Those are regenerated from a real `check.sh` receipt.
5. Probe (not full `check.sh`):

```bash
# bank the README advertises
./course-engine/target/debug/cdcp_gate --root course-engine verify-bank
python3 -c 'import json;d=json.load(open("course-engine/web/data/units_index.json"));print(d["bank_item_count"],d["approved_item_count"],d["module_count"],d["unit_count"])'

# claims-lint over the public surface (same binary also runs gate-shrink; ignore shrink here)
./course-engine/target/debug/cdcp_registry_check course-engine

# GitHub about
unset GITHUB_TOKEN GH_TOKEN
gh repo view --json description
```

6. Isolated commit of `README.md`, `course-engine/registries/claims_lint.toml`, and this ledger. Nothing else.

---

## Measured now

| Token | Live | Command |
|---|---|---|
| Modules | **15** | `ls modules/*.md \| wc -l` |
| Words | **81860** (~82,000 / ~82k) | `wc -w modules/*.md \| tail -1` |
| Domain registry | **15** (`exam_weight_unknown` only on `15-ops-adjacent`) | `grep -c '^\[\[domain\]\]' course-engine/knowledge/domains.toml` |
| Bank files | **854** [[fact:fact-bank-item-count-804=yes]] | `ls course-engine/bank/items/*.toml \| wc -l` |
| Approved / retired | **829 / 25** [[fact:fact-bank-approved-count-779=yes]] | `grep -h '^status = ' course-engine/bank/items/*.toml \| sort \| uniq -c` |
| `verify-bank` | `items=854 scanned, 829 approved` · `topics_registry=106` · `domain_floors=15` · `source_class=original` · multiplier ≈20.7× on the approved pool [[fact:fact-approved-pool-multiplier-19-5=yes]] | `./course-engine/target/debug/cdcp_gate --root course-engine verify-bank` |
| `units_index.json` | `854 829 15 134` [[fact:fact-learn-unit-count-134=yes]] | `python3 -c 'import json;d=json.load(open("course-engine/web/data/units_index.json"));print(d["bank_item_count"],d["approved_item_count"],d["module_count"],d["unit_count"])'` |
| Topics | **106** | `grep -c '^\[\[topic\]\]' course-engine/knowledge/topics.toml` |
| Crates | **18** (all `forbid(unsafe_code)`, zero `unsafe {`) | `ls -d course-engine/crates/*/ \| wc -l` |
| `crates/*/src` lines | **67438** (~67k; ~16k of that is inline `#[cfg(test)]` modules) | `find course-engine/crates -path '*/src/*' -name '*.rs' \| xargs cat \| wc -l` |
| `crates/*/tests` lines | **48356** (~48k) | `find course-engine/crates -path '*/tests/*' -name '*.rs' \| xargs cat \| wc -l` |
| WASM blob | **530385 B = 518 KiB** (committed; not rebuilt this bead) | `stat -f '%z' course-engine/web/assets/wasm/cdcp_wasm.wasm` |
| `scripts/` files | **33** | `find course-engine/scripts -maxdepth 1 -type f \| wc -l` |
| Registry tomls | **10** | `ls course-engine/registries/*.toml \| wc -l` |
| Learn HTML pages | **15** | `ls course-engine/web/learn/*.html \| wc -l` |
| Glossary term rows | **135** (advertised as 100+) | `python3 -c 'from pathlib import Path;print(sum(1 for ln in Path("reference/GLOSSARY.md").read_text().splitlines() if ln.startswith("| **")))'` |
| Practice exam / drill | **40 / 40** | `rg -c '^### [0-9]+\.' practice/PRACTICE-EXAM.md` · `rg -c '^### Card ' practice/DRILL-CARDS.md` |
| Free PDF sidecars | **5**, all `access = "free"` | `ls course-engine/knowledge/corpus/free-pdfs/*.meta.toml \| wc -l` |
| Study bar | **40 / 60 min / 27** | `n_items` / `duration_sec` / `pass_correct` in `course-engine/knowledge/exam_form.toml` |
| GitHub about (before this apply) | stale **14 modules, 804-question bank** | `gh repo view --json description` |

`verify-bank` is the bank oracle. `ls` / `grep status` agreed with it. Do not copy 846/821 from an earlier ledger: that was true before `bd-curriculum-truth-ebrr.28`.

---

## Apply set (file:section)

| ID | File:section | Was (HEAD `b90a959`) | Now | Measure |
|---|---|---|---|---|
| A1 | `README.md` hero (§1, one-liner) | Fourteen modules · ~54,000 words · 804/779 | Fifteen modules · ~82,000 words · 854/829 | modules / `wc -w` / `verify-bank` |
| A2 | `README.md` TL;DR **Bank** | 804 / 779 | 854 / 829 · 25 retired. Fact IDs **unrenamed**. | `verify-bank` + `units_index.json` |
| A3 | `README.md` TL;DR **Engine** | 7 crates, 3,763 lines, 281 KB | 18 crates · ~67k src (~16k inline tests) + ~48k tests · 518 KiB | crate / line / `stat` commands above |
| A4 | `README.md` Limitations | "CDCS/CDCE … out of scope" | Advanced direction, not shipped; neither track here is a credential | `find … -iname '*cdcs*'` empty in crates+web |
| A5 | `README.md` system map `scripts/` | 42, then a mid-apply 32 | **33 files** | `find course-engine/scripts -maxdepth 1 -type f \| wc -l` |
| A6 | `README.md` Run-it + Running it | `git clone <this-repo>` + `cdcp serve` | real origin URL + `cdcp study` as the learner verb; `serve` kept as no-open | `git remote -v` · `cdcp study --help` · `gh release list` (empty → no curl pipe) |
| A7 | `README.md` honesty scan-set sentence | "every document in this repository" | names this README, `CHARTER.md`, engine README + `docs/` | `registries/claims_lint.toml` `roots` |
| A8 | `README.md` FAQ 27/40 | "not affiliated with any official cut score" | mirrors public form; study signal here; grants no credential | `exam_form.toml` `pass_correct = 27` |
| A9 | `course-engine/registries/claims_lint.toml` | `roots = ["README.md", "docs"]` (engine only) | also `../README.md`, `../CHARTER.md`; exclude `docs/PLAN-N-INSTALLABILITY.md` | `cdcp_registry_check course-engine` |
| A10 | GitHub about blurb | "14 modules, 804-question bank" | see exact string below | `gh repo view --json description` |

Same 854/829 also applied at honesty §2, system-map bank line, data-flow, L3 row, limitations, FAQ so the file does not contradict itself.

### GitHub about (exact)

```
Free, offline self-study for the data-centre facilities domain — 15 modules (~82k words), 854-item bank (829 approved), Rust/WASM grader that is byte-exact. Not a certification.
```

---

## HOLD — do not hand-edit

| Token | Advertised | Why |
|---|---|---|
| Ordered steps | 90 | Regenerated from a `check.sh` receipt. Another epic owns it. |
| Known-bad injections | 72 (shell selftest suites) | Same receipt machinery. |
| Registered suites | 10 (two more emit receipts but stay unregistered) | `installer` and `learner_verbs` are out of `REGISTERED_SUITES` on purpose (`gate_shrink`). |

A hand edit here desynchronizes the README from the verifier.

---

## Fact-id drift (804 vs 854)

The markers still read `fact-bank-item-count-804` and `fact-bank-approved-count-779`. That is **not** a claim that the bank is 804/779.

`course-engine/registries/doc-facts.toml` says the IDs are stable names; the probes assert `"bank_item_count": 854` and `"approved_item_count": 829`. Renaming the IDs to `…-854` / `…-829` would resolve to no row and turn the gate red.

**Honest advertising rule:** prose next to the marker must be the live count (854/829). The ID string is a name.

---

## Status

| | |
|---|---|
| README apply | A1–A8 landed in `01fe7b3` (sibling CHARTER / PHASE-NEXT / `doc-facts` comment went with that commit; not this file) |
| claims-lint roots | public surface only; PLAN-N excluded as planning notes (`01fe7b3`) |
| This file | the measure-backed apply list + process. Pass/grade notes in this directory were committed separately as `4b980e6` and still trip `doc-facts` (they quote receipt tokens without polarity). |
| GitHub about | A10 (applied via `gh repo edit`, not git) |
| Frozen | HOLD row |
| Not in this commit | `CHARTER.md`, `check.sh`, bank, goldens, `.beads/`, `.flywheel/ALERT`, `diff_verify_*.rs` |
