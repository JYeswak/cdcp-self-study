# Citation record — Uptime Tier Standard: Topology (2026)

> **Rights posture.** ©2026 Uptime Institute, ALL RIGHTS RESERVED. The document states that
> written permission is required *at each and every occasion* that its IP "or portions of the
> intellectual property are reproduced **or used**", and that copyright extends to all media.
>
> **Therefore this file contains ZERO text from the document.** It records only:
> (a) facts *about* the document — presence, absence, counts;
> (b) locators;
> (c) our own analysis.
>
> Obtained legitimately via free registration download, 2026-08-14. Do not vendor the PDF,
> do not quote it, do not paraphrase closely. If product content ever needs to derive from it,
> the route is a Copyright Reprint Permission Request — not a workaround.

```
authority:      Uptime Institute
document:       Data Center Site Infrastructure Tier Standard: Topology
edition:        2026
pages:          13
access:         free registration download; all rights reserved
verified_by:    Josh
verified_at:    2026-08-14
local_copy:     NOT vendored (rights) — re-download from uptimeinstitute.com to re-verify
```

---

## Verified findings (facts about the document, not its content)

### UT-01 · The circulated Tier availability percentages are NOT in the standard — **CONFIRMED**

| | |
|---|---|
| **Claim under test** | The figures 99.671% / 99.741% / 99.982% / 99.995%, universally cited as "Tier I–IV availability", are not endorsed by Uptime. Uptime withdrew downtime predictions in 2009. |
| **Method** | Full-text scan of the 2026 edition for any `99.xx` availability figure |
| **Result** | **0 occurrences** |
| **Verdict** | **CONFIRMED at primary source.** The current standard contains no availability percentages at all. |
| **Confidence** | EXTRACTED (direct observation of the primary document) |

**Why this matters:** those four numbers appear in a large fraction of data-centre training
material worldwide, attributed to a standard that does not contain them. Any item, module, or
generated question asserting a Tier→availability-percentage mapping is **unsupported by the
authority it names**.

**Repo status:** the mapping is absent from our bank and Learn content (verified
2026-08-13 — module 01 teaches nines as arithmetic with correct independence caveats, and
never maps them to Tier levels). This record exists so the absence stays deliberate rather
than accidental.

**Action:** promote to a gate — an item asserting `Tier N ⇒ specific availability %` should be
RED, citing UT-01.

### UT-02 · Structural vocabulary present in the 2026 edition — **CONFIRMED**

Presence counts only, as evidence the concepts are load-bearing in the current edition:

| Concept | Occurrences |
|---|---|
| Tier I | 46 |
| Distribution Path | 18 |
| Tier IV | 13 |
| Fault Tolerant | 9 |
| Redundant Capacity | 9 |
| Concurrently Maintainable | 8 |
| **Continuous Cooling** | 5 |

**Continuous Cooling is the notable one** — a Tier IV concern routinely omitted from training
material that treats Tier purely as an electrical-redundancy ladder. Check whether module 09
(cooling, 121 items) connects cooling continuity to Tier classification at all.

### UT-03 · Scope disclaimer (codes / weather / security) — **INCONCLUSIVE, not refuted**

L3 recorded, from Uptime's own comparison material, that Tier topology excludes factors such
as codes, weather and security. A scan of this edition for disclaimer phrasing
(`does not address` / `outside the scope` / `excluded`) returned 0 matches — **but the search
patterns were narrow and this is not evidence of absence.** Do not record L3's claim as
disproven. Re-check by reading the scope section directly.

---

## What still cannot be grounded from this document

TIA-942 and EN 50600 clause content remain **paywalled and UNGROUNDED**. The L3 `contradicts`
edges between Uptime / TIA-942 / EN 50600 rest on secondary sources for the TIA and EN sides.
Holding the Uptime primary source grounds only one leg of that three-way comparison.
