# Prepublish scrub receipt — 2026-08-14 (corpus rights)

**Beads:** `bd-ashrae-redistribution-contradiction-p5n` (closed), `bd-doe-guide-embeds-ashrae-p54` (closed)
**Scope:** `course-engine/knowledge/corpus/**` — rights posture only. No git-history rewrite.
**Repo state:** PUBLIC at `github.com/JYeswak/cdcp-self-study` since 2026-08-12.
**Successor to:** `PREPUBLISH-SCRUB-2026-08-12.md` (secrets scrub + ASHRAE PDF purge).

---

## ⛔ AI-policy compliance statement for this pass

ASHRAE's published AI policy prohibits entering content from any ASHRAE publication or
related ASHRAE IP into any AI tool, and prohibits AI-created derivative works without express
written permission. The agent that performed this pass is an AI tool.

**No ASHRAE-sourced content was read, quoted, summarised or paraphrased at any point.**
The work was done entirely from filenames, byte counts, `manifest.json`, `.meta.toml`
sidecars, and git metadata — plus, where a structural fact was needed, **regex-count scans
that emit an integer and never surface the matched text**. Removing a file does not require
reading it: `git rm` plus a metadata edit is sufficient, and that is what was done.

The same discipline was applied to the DOE guide: identification of its embedded ASHRAE
material came from an **attribution/credit-line scan**. The tables and figures themselves were
not opened.

---

## Bead 1 — ASHRAE captures marked `redistribution: not-licensed` were published

### Finding

Two files sat in the public tree while `manifest.json` recorded them
`rights = "publisher-retains-copyright"`, `redistribution = "not-licensed"`:

| File | Bytes | Status |
|---|---|---|
| `knowledge/corpus/public/src-ashrae-datacom.txt` | 11,257 | **removed from HEAD** |
| `knowledge/corpus/public/src-ashrae-power-wp.txt` | 586 | **removed from HEAD** |

The manifest hygiene was **good** — rights were recorded honestly rather than assumed
permissive. The defect is the contradiction between the record and the act. Retaining a
capture for internal grounding and pushing it to a public remote are different acts; the
second is redistribution, which the manifest itself says is not licensed. `access = "free"`
is a price fact about ashrae.org and authorises neither redistribution nor ingestion.

These two survived the 2026-08-12 purge of three ASHRAE TC 9.9 PDFs because that purge
targeted `free-pdfs/*.pdf` and these are `.txt` under `public/`.

### Removed vs retained

**Removed:** both `.txt` bodies (`git rm`), and their `path` / `bytes` keys from the manifest.

**Retained:** a `capture = "citation-only"` manifest row per source, carrying exactly
`{id, title, publisher, url, fetched_at, access, rights, redistribution, ai_ingestion,
retention_basis}` plus a removal audit trail
(`body_removed_at`, `body_removed_bead`, `body_removed_sha256_of_prior_capture`).
**Zero publisher-derived prose.**

The manifest's own `retention_basis` stated the purpose as *"so the claim can be re-verified at
source"*. A citation row serves that purpose completely. The body served it no better and
carried all of the exposure. The `sha256` of the prior capture is kept so the removal is
auditable without the bytes.

Also retained: `knowledge/sources.toml` ids `src-ashrae-datacom` and `src-ashrae-power-wp`,
and the downstream references in `topics.toml` / `standards_families.toml`, all of which
remain valid — the source records still exist, only the bodies are gone.

### Verification

- `ls knowledge/corpus/public/` → neither file present.
- `grep -ril ashrae knowledge/corpus/public/` → `manifest.json` (bibliographic fields only)
  plus `src-tia-942-c-fotc.txt` (3 hits) and `src-curriculum-map-local.txt` (2 hits).
  Those two are a TIA page and our own curriculum map **naming the organisation**; a
  count-only 600-char-window scan for thermal-envelope markers (`A1`–`A4`, dew point, °C,
  relative humidity) returned **0** in both. Naming ASHRAE is not reproducing ASHRAE IP.
- `python3 scripts/validate_grounding.py` → `PASS`, `high_severity=0` (unchanged).
- `bash tests/publishability-bar.sh` → **23 passed, 0 failed**, including
  *"every scraped corpus source records its rights"* and *"no ASHRAE PDFs anywhere in history"*.

---

## Bead 2 — the DOE/FEMP guide embeds ASHRAE tables

### Decision: **(a) DO NOT VENDOR. Cite by URL and section locator only.**

Decision record: `knowledge/corpus/free-pdfs/lbnl_femp_best_practice_guide_dc_design.meta.toml`
(`capture = "not-vendored"`, `decision = "a-do-not-vendor"`).

`best-practice-guide-data-center-design.pdf` (48pp, DOE/FEMP/LBNL/NREL) carries the standard
US Government disclaimer but reproduces ASHRAE material under attribution. Two independent
blockers, either sufficient:

1. **Rights.** 17 USC 105 places only the government-**authored** portions in the public
   domain. Third-party material embedded in a government work does not become public domain by
   being bound into it. NREL *contractor* authorship weakens a blanket §105 reading further.
   Vendoring into this public repo would republish ASHRAE tables. `rights` is therefore
   recorded as `mixed-us-government-work-with-third-party-material` — **not** bare
   `public-domain`.
2. **AI policy.** The prohibition binds on **content, not container**. Reading Table 3-1
   violates it whether the bytes arrive from ashrae.org or from a government wrapper. This
   repo is AI-built end to end, so vendoring *is* ingesting.

**Option (b), a redacted extract, was considered and rejected for agents.** Producing one
requires locating the ASHRAE pages in order to exclude them — i.e. opening them. The redaction
act is itself the violation. A human may do it; an AI tool may not. The exclusion list is
recorded in `third_party_figures` anyway, so a human can execute (b) later without re-deriving it.

**Option (c), seeking ASHRAE written permission,** is an external, irreversible action taken in
the project's name. Escalated to Josh, not assumed.

### `third_party_figures` (identified by attribution/credit-line scan; content NOT read)

- `Table 3-1. ASHRAE 2021 Thermal Guidelines for Air Cooling`
- `Source: Thermal Guidelines for Data Processing Environments, ASHRAE` (×3)
- `Source: Emergence and Expansion of Liquid Cooling in Mainstream Data Centers, ASHRAE` (×2)

### Consequence, accepted deliberately

Table 3-1 is the A1–A4 thermal envelope this project has been marking **UNGROUNDED**. The
guide is a side-door to exactly that data. It stays ungrounded until a **human** reads the
source and supplies the values. `Gap_rci-unusable-without-the-envelope` stays open on purpose —
that gap is honest, and closing it through this PDF would not be.

### What is still legitimately usable

Nothing changes for existing citations. The graph already handles this correctly: **25 nodes**
in `knowledge/graph/dc-lifecycle.graph.json` cite this guide, and a scan of their
`source_location` fields shows every one points at §4.x air management, §5.x cooling or §6.x
electrical — DOE-authored chapters, cited by section number, with no text vendored. **No node
cites §3.x or Table 3-1.** Off-limits sections are recorded in the sidecar so this stays true.

---

## The generalisation — this was a CLASS, not two instances

Both beads named a class. The invariants are now recorded in a machine-readable enforcement
target rather than in prose that the next agent may not read:

**`knowledge/corpus/rights-policy.toml`** — `schema = "cdcp.corpus.rights-policy.v1"`

| ID | Invariant |
|----|-----------|
| **CORPUS-R1** | No record with `redistribution != "permitted"` may be `capture = "body-retained"` with a path inside the published tree. |
| **CORPUS-R2** | `rights`, `redistribution`, `capture` are **required**; missing or out-of-vocabulary ⇒ **ERROR**, never default-permissive. |
| **CORPUS-R3** | A record declaring `third_party_figures` may **not** be `rights = "public-domain"` or `redistribution = "permitted"`. |
| **CORPUS-R4** | An R1 exception needs `rights_review = "OPEN"` **plus** a bead **plus** a non-empty reason. A bare exemption is a schema error. |
| **CORPUS-R5** | Zero records scanned ⇒ **ERROR**. Never vacuously green. |
| **CORPUS-R6** | `ai_ingestion = "PROHIBITED"` ⇒ `capture` must be `citation-only` or `not-vendored`. |

The file also ships **6 `[[known_bad]]` fixtures**, one per invariant, so the gate can be
*proven to trip* rather than assumed to work.

Prose was updated to match, not to substitute: `knowledge/corpus/README.md` (policy rewritten —
the old OQ-09 *"store as much free/public material as possible"* conflated price with licence
and is superseded) and `knowledge/corpus/FREE-ACCESS-CAPTURE.md` (standing notes **SN-1..SN-5**,
plus §3 and §5 moved from open checklists to recorded verdicts).

### Open violation, tracked not tolerated

The strict reading of CORPUS-R1 implicates **ten more sources**, not two. Every remaining
capture under `public/` carries `rights = "publisher-retains-copyright"`,
`redistribution = "not-licensed"` and is committed to the public tree:

`src-epi-cdcp-page`, `src-exin-cdcp-page`, `src-nh-cdcp`, `src-tuv-22237`,
`src-en-50600-overview`, `src-tia-942`, `src-tia-942-c-fotc`, `src-uptime-tiers`,
`src-ocp-ready`, `src-curriculum-map-local`.

Legally the same act, minus the AI-policy aggravation that makes ASHRAE acute. They are **not**
removed in this pass because `scripts/validate_grounding.py` consumes these bodies as its
grounding corpus (~601k chars) across 804 items; deleting them without a replacement grounding
source would take the build red under four concurrently active agents — a cross-cutting
destructive change that belongs in its own bead, not smuggled into an ASHRAE fix.

Each of the ten now carries `rights_review = "OPEN"`, `rights_review_bead`, and a
`rights_review_reason` in `manifest.json`, and they are listed under `[[open_violation]]` in
`rights-policy.toml`. **A reasoned, bead-tracked exception — not a silent one, and not a
widened allowlist.** That table shrinks to empty; it must never grow without a bead.

Filed: **`bd-corpus-public-captures-not-licensed-class-kej`**.

---

## Git-history purge — **NOT NEEDED. HEAD removal suffices.**

**Decision: HEAD-only. No history rewrite performed, and none recommended.**
(A rewrite is danger-list; had the answer gone the other way this receipt would say STOP,
not report a completed purge.)

### Evidence

Both files entered in a single commit, `5c98662`, which **is** on `origin/main` — so the bytes
are genuinely public right now. Exposure is real, which is why HEAD removal is required. What
follows is why *only* HEAD removal is warranted.

Structural characterisation (regex counts; **no content surfaced**):

| | `src-ashrae-power-wp.txt` | `src-ashrae-datacom.txt` |
|---|---|---|
| bytes / lines | 586 / 10 | 11,257 / 342 |
| our own `[PDF not extracted in-repo]` stub marker | **1** | 0 |
| thermal-envelope markers (`A1`–`A4`, dew point, °C, RH) | **0** | **0** |
| table/figure captions | **0** | **0** |
| store/navigation markers | 2 | 8 |
| sentences ≥ 40 chars | 3 | 10 |

1. **`src-ashrae-power-wp.txt` contains no ASHRAE content at all.** Its URL ends in `.pdf`, so
   `scripts/fetch_public_corpus.py` took its PDF branch and wrote *our own generated stub* —
   `[PDF not extracted in-repo]`, a URL and a title. It was already a citation row wearing a
   `.txt` extension. There is nothing to purge.
2. **`src-ashrae-datacom.txt` is a bookstore/catalogue landing page.** 342 lines but only ~10
   sentences and 8 store-navigation markers: product titles and site chrome, not technical
   prose. **Zero** thermal-envelope markers and **zero** table/figure captions — it carries no
   standards content, no tables, no thermal-guideline data.
3. **Materially different from what was purged on 2026-08-12.** That purge removed three full
   ASHRAE TC 9.9 *publications* — actual technical content, squarely the copyright and
   AI-policy concern. A public catalogue page is a different magnitude of artifact. Applying
   the same remedy is cargo-culting the precedent, not reasoning from it.
4. **These are the same bytes ASHRAE serves unauthenticated at that URL.** No access control
   was circumvented and nothing non-public is exposed. Removing them from HEAD ends this
   project's redistribution going forward, which is the entire harm.
5. **A rewrite would not achieve retraction anyway.** The repo has been public since
   2026-08-12. Force-pushing a rewritten `main` does not reach clones or forks, and GitHub
   keeps unreferenced objects fetchable by SHA until GC. The purge would buy near-zero actual
   retraction.
6. **Cost is real and lands on other people.** It would rewrite every commit since `5c98662`,
   force-push a public repo, and force a hard reset on four concurrently active agents.

**Reasoning had it gone the other way:** if either file had contained standards body text —
clause text, thermal-envelope tables, figure data — the calculus would invert. Publication
volume would be irrelevant next to the AI-policy and derivative-works exposure, and this
receipt would **STOP and escalate to Josh** for a supervised history rewrite rather than
perform one. It did not, so it does not.

---

## Changed files

| File | Change |
|---|---|
| `knowledge/corpus/public/src-ashrae-datacom.txt` | **deleted** (`git rm`) |
| `knowledge/corpus/public/src-ashrae-power-wp.txt` | **deleted** (`git rm`) |
| `knowledge/corpus/public/manifest.json` | schema `v1` → `v2`; 2 rows → citation-only; 10 rows carry tracked `rights_review`; `invariants` block added |
| `knowledge/corpus/rights-policy.toml` | **new** — machine-readable invariants + known-bad fixtures + open-violation register |
| `knowledge/corpus/free-pdfs/lbnl_femp_best_practice_guide_dc_design.meta.toml` | **new** — do-not-vendor decision record + `third_party_figures` |
| `knowledge/corpus/README.md` | policy rewritten; invariant table; add-a-source checklist |
| `knowledge/corpus/FREE-ACCESS-CAPTURE.md` | standing notes SN-1..SN-5; §3 and §5 resolved to verdicts |
| `docs/flywheel/PREPUBLISH-SCRUB-2026-08-14.md` | this receipt |

No `.py` or `.sh` created. `scripts/check.sh` not edited. Nothing committed.

## Verdict

- **Bead 1 (ASHRAE captures):** PASS — bodies removed from HEAD, citation-only rows retained,
  no publisher-derived prose remains in the published tree.
- **Bead 2 (DOE guide):** PASS — decision (a) do-not-vendor recorded with rights posture and
  exclusion list; no ASHRAE material entered the repo or the agent.
- **Class:** invariants recorded and enforceable; residual 10-source violation filed as
  `bd-corpus-public-captures-not-licensed-class-kej`.
- **History purge:** not needed, reasoned above, not performed.
- **Gate:** the Rust corpus-rights gate is **bead D4** and is not built here.
  `rights-policy.toml` is the artifact D4 enforces against.
