# Grounding corpus

**This directory is inside a PUBLIC repository** (`github.com/JYeswak/cdcp-self-study`).
Committing a file here publishes it to the world. That is the fact every rule below follows
from.

## Policy

**Superseded 2026-08-14.** The original OQ-09 policy (Josh, 2026-08-12) read *"store as much
free/public source material as possible in-repo"*. It conflated two different things and is
replaced by:

> **Store as much source material as we are LICENSED to redistribute. Everything else gets a
> citation row.**

`access = "free"` is a **price** fact about the publisher's own site. It is not a licence, and
it is not permission for an AI tool to read the bytes. Rights are recorded separately, per
source, in `rights` / `redistribution` / `ai_ingestion` — and a **missing** field is an
**ERROR**, never a default-permissive.

Retaining a capture for internal grounding and publishing it worldwide are different acts.
The second is redistribution. Where redistribution is not licensed, the capture is reduced to
a **citation row** — `{url, title, publisher, fetched_at, rights, redistribution}` with **zero
publisher-derived prose**. That fully serves the recorded retention basis ("so the claim can
be re-verified at source"); the body adds nothing to that purpose and all of the exposure.

## Layout

| Path | Contents |
|------|----------|
| `rights-policy.toml` | **The invariants, machine-readable.** Enforcement target for the corpus-rights gate. |
| `public/manifest.json` | Per-source provenance + rights, schema `cdcp.corpus.manifest.v2` |
| `public/*.txt` | Text snapshots — **only** for sources whose `redistribution` permits it |
| `free-pdfs/*.meta.toml` | Rights + provenance sidecars. **A sidecar may exist with no PDF** — that is the citation-only / not-vendored shape, and it is the normal outcome. |
| `free-pdfs/*.pdf` | Only where `rights = "public-domain"` **and** no `third_party_figures` |
| *(never)* | Paid ISO/IEC/TIA/NFPA/BICSI full text (OQ-10 until buy decision); exam dumps |

Paid catalog URLs stay in `knowledge/sources.toml` as `access = "paid"` with **no** full-text
file.

## The invariants

Normative form: [`rights-policy.toml`](rights-policy.toml). Summary:

| ID | Rule |
|----|------|
| **CORPUS-R1** | No record with `redistribution != "permitted"` may be `capture = "body-retained"` with a path inside the published tree. |
| **CORPUS-R2** | `rights`, `redistribution` and `capture` are **required**. Missing or out-of-vocabulary ⇒ **ERROR**, never permissive. |
| **CORPUS-R3** | A record declaring `third_party_figures` may **not** be `rights = "public-domain"` or `redistribution = "permitted"`. |
| **CORPUS-R4** | An exception to R1 needs `rights_review = "OPEN"` **plus** a bead **plus** a non-empty reason. A bare exemption is a schema error. |
| **CORPUS-R5** | Zero records scanned ⇒ **ERROR**. Never vacuously green. |
| **CORPUS-R6** | `ai_ingestion = "PROHIBITED"` ⇒ `capture` must be `citation-only` or `not-vendored`. |

**R3 exists because government-published is not government-authored throughout.** 17 USC 105
covers the government-authored portions only; third-party tables and figures reproduced inside
a federal document under attribution do not become public domain. Check the figure credits,
not the cover. Worked case:
[`free-pdfs/lbnl_femp_best_practice_guide_dc_design.meta.toml`](free-pdfs/lbnl_femp_best_practice_guide_dc_design.meta.toml).

**R6 exists because ASHRAE prohibits entering ASHRAE IP into any AI tool.** That binds on
content, not container — it follows an ASHRAE table into a government PDF or a vendor deck.
This repo is AI-built end to end, so vendoring ASHRAE material *is* ingesting it. ASHRAE
content is human-read-only, cited by locator. See `FREE-ACCESS-CAPTURE.md` §3 and the
standing notes SN-1..SN-5 at the top of that file.

## Adding a source — checklist

1. Record `rights`, `redistribution`, `ai_ingestion`. Do not infer them from `access`.
2. Scan every table/figure **credit line** for third-party material. Populate
   `third_party_figures` if any. Do not read third-party content you are not licensed to read.
3. Pick `capture`: `body-retained` only if `redistribution = "permitted"`; otherwise
   `citation-only`, or `not-vendored` if nothing is stored at all.
4. If you cannot determine rights, the value is `unknown` — which blocks `body-retained`.
   Guessing permissive is the failure mode this whole file exists to prevent.

## Open violations

`rights-policy.toml` `[[open_violation]]` lists records that breach an invariant and are
tracked by a bead. That table shrinks to empty; it must never grow without a bead.
