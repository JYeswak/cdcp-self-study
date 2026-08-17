# PASS-03 — Limitations, FAQ, and North-Star Ledger

**Bead:** `bd-readme-public-rigor-8y0r`  
**Pass:** 3 of 6  
**Scope:** ledger only. No README, CHARTER, engine, cargo, bead, or NTM mutation was performed.

## Measurement receipt

| Surface | Measure command | Observed result |
|---|---|---|
| Curriculum size | `find modules -maxdepth 1 -type f -name '*.md' -print \| sort` | 15 module files: `01`–`15`, including `15-ops-adjacent.md`. |
| Bank size | `find course-engine/bank/items -maxdepth 1 -type f -name '*.toml' \| wc -l`; `rg -l '^status = "approved"' course-engine/bank/items \| wc -l`; `rg -l '^status = "retired"' course-engine/bank/items \| wc -l` | 846 files; 821 approved; 25 retired. |
| Bank evidence class | `awk -F'"' '/^quantity_evidence = /{print $2}' course-engine/bank/items/*.toml \| sort \| uniq -c` | 846 `qualitative_only`. This supports the future CDCS-calculation direction; it does not show CDCS calculations shipped. |
| Public README direction | `rg -n -i '\b(CDCS|CDFOS)\b|out of scope|not shipped|credential' README.md` | README names CDCS only in the current out-of-scope limitation and has no CDFOS direction statement. |
| Named shipped artifacts | `find course-engine/crates course-engine/web -type f \( -iname '*cdcs*' -o -iname '*cdfos*' \) -print` | No named CDCS/CDFOS artifact returned. This is an absence measurement, not a claim about future work. |

## Ledger rows

| ID | File:section | Current measured fact | Disposition for a later apply pass |
|---|---|---|---|
| P03-L1 | `README.md:377-386`, Limitations | The limitation correctly says the tool cannot certify a learner, explains regional/code boundaries, and says the standards are not reproduced. The final bullet says: “No CDCS/CDCE depth. Those are advanced design tracks and out of scope.” | **Replace only the CDCS-out-of-scope sentence.** Candidate wording: “**Advanced direction, not shipped:** CDCS calculation depth and CDFOS operations depth are the first planned tracks after this study surface. Neither is a credential.” Do not imply either track, its calculations, or its exam is present today. |
| P03-L2 | `README.md:379-385`, Limitations | The no-credential, regional-practice, licensed-standard, and licensed-engineer boundaries are concrete and useful. | **Keep.** These are honest limitations and should remain adjacent to the future-direction sentence. |
| P03-L3 | `README.md:387-391`, Limitations | The bank is described as self-reviewed and error-prone, and L3 is called thin. The prose’s `804/779` count is stale against the measured `846/821/25` bank state. | **Keep the limitation; hand the count to Pass 1.** Do not use Pass 3 to silently repair unrelated hero/status numbers. |
| P03-F1 | `README.md:395-398`, FAQ — certification | The answer explicitly says the tool does not grant CDCP certification and directs learners to authorised training and the official exam. The registered claim is `course-engine/registries/claims.toml:53-56`. | **Keep.** Any future CDCS/CDFOS mention must carry the same non-credential boundary. |
| P03-F2 | `README.md:400-401`, FAQ — exam questions | The FAQ correctly rejects real-exam-question framing and says the items are original. `course-engine/bank/MANIFEST.toml:5` records `source_class = "original"`; the file count in the FAQ is stale and belongs to Pass 1. | **Keep the no-dump answer; hand the measured count to Pass 1.** Do not add exam-like marketing language. |
| P03-F3 | `README.md:403-405`, FAQ — 27/40 | The FAQ frames 27/40 as an internal review-loop study signal, explicitly not a pass mark and not a credential. The UI repeats the same boundary at `course-engine/web/mock.html:17-18,40`. | **Keep.** Do not call the bar a certification threshold, readiness guarantee, or official cut score in this pass. |
| P03-F4 | `README.md:407-416`, FAQ — licensing, runtime, mascot | The licensing split, offline/runtime explanation, and Yuzu explanation are outside the CDCS/CDFOS scope. | **Keep unless a later measured pass finds a contradiction.** No speculative FAQ expansion is justified here. |
| P03-N1 | `README.md:20,44-49`, public product/north-star framing | The README describes the currently measured 15-module study surface and engine/bank, while the shipped-artifact probe finds no named CDCS/CDFOS surface. | **Add later, not now:** state that **CDCS calculations + CDFOS first are the direction**. Mark them **not shipped** and **not a credential**. This is roadmap language, not a feature claim or certification promise. |
| P03-N2 | `README.md:386` versus this ledger | “Out of scope” is too final for the intended direction, while “shipped” would be false. The safe distinction is planned scope versus present product. | **Kill the stale framing, preserve the boundary:** direction = CDCS calculation depth first, then CDFOS operations depth; present product = CDCP self-study surface; credential = none. |

## Pass verdict

Pass 3 is a ledger pass, not an apply pass. The README’s certification, study-signal, original-corpus, regional-practice, and licensed-standard limitations are measured keepers. The one required correction is to remove the blanket “CDCS … out of scope” framing and replace it later with the narrower, honest north-star: **CDCS calculations + CDFOS first; planned direction only; not shipped; not a credential.**
