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
| **CORPUS-R7** | `body-retained` + `redistribution = "permitted"` over rights that are **not** self-evidencing needs `redistribution_evidence {licence, url, clause}`. A bare "permitted" is an assertion, not a licence. |
| **CORPUS-R8** | A file under a published root that **no record claims** is an **ERROR**. |

**Enforcement is not yet wired on the check path.** The policy and the records
are law. The implementation that would trip on a planted R7/R8 fixture exists as
untracked `cdcp_gate` sources and **must not be added under `src/gates/`**
(`gate_shrink` ceiling 49422; adding a globbed `src/gates/*.rs` grows the crate).
EXTRACT-THEN-DELETE: land the checker in the engine (`cdcp_data` or a sibling),
then a thin CLI step. Until that extract, R1–R8 are recorded invariants plus
this metadata sweep, not a proven-to-trip gate. A README that claimed
`cdcp_gate corpus-redistribution` is wired would be BUILT ≠ WIRED.

The intended checker reads this directory's metadata and the tree on disk, and
**it reads no capture body** — it cannot, because some records forbid AI
ingestion of their content, and a rights gate that had to open a file to decide
whether the file may be opened would be self-defeating. It therefore cannot
decide that the metadata is honest; it decides that the metadata exists, is
drawn from the vocabulary below, and agrees with what is actually on disk.

**R7 is the GROWTH path, not only a brake.** It is how a genuinely open document gets to keep
its body: name the licence, link it, cite the clause. **R8 is the direction everything else
misses** — every other rule reasons from a record outward, so a capture that arrives with no
record at all is invisible to all of them, and that is exactly how the next one arrives.

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

**It is EMPTY as of 2026-08-14** (bd-corpus-public-captures-not-licensed-class-kej). Nine
public-page body captures recorded `redistribution = "not-licensed"` were removed from the
published tree and reduced to citation rows; each row records what was removed, when, under
which bead, and the sha256 of the capture that went, so "we removed it" is checkable rather
than asserted. A tenth record on that list, `src-curriculum-map-local`, was never a violation —
it is this project's own work, swept in on an inherited default and corrected on the record.

The stated blocker had been that `scripts/validate_grounding.py` consumed these bodies as its
grounding corpus. Measured: **545,935 of the corpus's 659,149 characters came from outside
`knowledge/corpus/public`**, against a floor of 20,000. After the removal the corpus measures
**571,074 characters — 28× the floor.** The gate never needed this material. What it needed,
and got first under bd-yje7, was to stop reporting green on a corpus of zero characters, so
that a corpus which really did disappear would go RED on the way down instead of passing in
silence. The legal remediation and the anti-vacuous fix were one job seen from two ends.

**An entry here grants no exemption.** When the checker is extracted and wired,
it must read this table only to check it for rot — an entry naming a record that
no longer exists is an ERROR — and a tracked violation is still a violation.
This table records that someone owns the problem; it does not turn the build
green. Today the table is empty and the records agree with the tree; that is
the product of this sweep, not a green gate certificate.
