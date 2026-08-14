# Free-access capture sheet (opened 2026-08-13)

**The distinction that governs everything here:**

| Tier | Means | What we may do |
|---|---|---|
| **PD** public domain | US Gov work, no copyright | **ship it** — vendor into `corpus/free-pdfs/` with a `.meta.toml` |
| **RO** read-only | free account, no print/copy/download | **ground it, never ship it** — a human reads it and records a VERIFIED clause locator |
| **REG** registration | free download, still copyrighted | link + cite; check the licence before vendoring anything |

**RO is the important one and the reason this session matters.** Research agents cannot read
paywalled standards, so they must mark every such claim `UNGROUNDED` and are forbidden from
inventing clause numbers. A human with a free login can read the text and supply a verified
locator. The repo never stores the text, so the rights posture is untouched — but the citation
stops being a guess. **Most of the grounding gap closes at $0.**

---

## Standing notes — read before capturing anything

These are not tips. They are the rules the corpus gate enforces; the machine-readable
form is `knowledge/corpus/rights-policy.toml`.

**SN-1 · `access` is a PRICE fact. It is never a permission.**
`access = "free"` means the publisher serves the bytes without charge. It says nothing
about whether *this* repository may republish them, and nothing about whether an AI tool
may read them. Rights live in `rights` / `redistribution` / `ai_ingestion`, recorded
per source. A free download is still copyrighted.

**SN-2 · Retaining is not publishing. This repo is PUBLIC.**
Keeping a capture on a laptop for grounding and committing it to
`github.com/JYeswak/cdcp-self-study` are different acts. The second is redistribution.
Any source recorded `redistribution != permitted` gets a **citation row** — `{url, title,
publisher, fetched_at, rights, redistribution}` and zero publisher-derived prose. A
citation row fully serves the stated retention basis ("so the claim can be re-verified at
source"); the body adds nothing to that purpose and adds all of the exposure.

**SN-3 · Government-PUBLISHED is not government-AUTHORED throughout.**
17 USC 105 puts the government-authored portions of a work in the public domain — not the
third-party tables, figures and photographs reproduced inside it under attribution.
**Check the figure credits, not the cover.** Before marking any government PDF
`rights = "public-domain"`, scan every table/figure caption and credit line and record what
you find in `third_party_figures`. Contractor authorship (NREL, national-lab subcontracts)
weakens a blanket 17 USC 105 reading further. A record that declares `third_party_figures`
may never be marked `public-domain` or `redistribution = "permitted"` — the gate rejects it.
Worked case: `free-pdfs/lbnl_femp_best_practice_guide_dc_design.meta.toml` (§5 below).

**SN-4 · ASHRAE is AI-prohibited, not merely unlicensed.**
ASHRAE's published AI policy prohibits entering content from any ASHRAE publication or
related ASHRAE IP into any AI tool, and prohibits AI-created derivative works without
express written permission. The policy binds on **content, not container** — the prohibition
follows an ASHRAE table into a government PDF, a vendor slide deck, or a third-party
summary. This project is AI-built end to end, so *vendoring* ASHRAE material *is* ingesting
it. ASHRAE-sourced content is human-read-only, cited by locator, forever.

**SN-5 · A missing rights field is an ERROR, never a default-permissive.**
No field ⇒ no permission ⇒ the gate fails and names the file. Absence must never read as
"probably fine". An exception needs `rights_review = "OPEN"` plus a reason string plus a
bead — a bare exemption is a schema error.

---

## 1 · NFPA free access — **RO**, needs free account

> ⚠️ **NFPA LiNK® is NOT the free tier.** LiNK is the paid subscription (~1,700 standards,
> all current editions + 5 legacy editions, Spanish titles, handbook commentary). Landing on
> the LiNK product page and reading its FAQ is easy to mistake for the free offering.
>
> **The free path is per-standard:** go to the individual standard's page on nfpa.org and use
> the free read-only viewer with a free NFPA account. Read-only — no print, copy, download.

`nfpa.org/…/free-access` → then navigate to each standard

### ✅ CONFIRMED FREE — the entire NFPA catalogue (verified 2026-08-13)

NFPA's free-access page states it directly: *"free online access to **any** code or
standard."* 421 standards listed. **NFPA 75 is free to read. No purchase, no subscription.**

**Earlier guidance in this file was wrong and is retracted:** the $165 purchase is
unnecessary, and so is the $13.99/mo LiNK subscription previously recommended here. The
product page for NFPA 75 shows only paid formats, which is what caused the confusion — the
free path is not on the product page.

**Access flow (per standard):**
1. Codes & Standards list → search the number → **"Read More"**
2. On the standard's page, choose the **edition year** from the drop-down
3. Click **"View Free Access"**
4. New window → accept terms & conditions
5. Navigate by Table of Contents / paging

**Restriction:** read-only. Cannot download or print. Free account required. Perfect for
locator extraction; useless for vendoring — which is exactly our posture.

### NFPA read list, priority order (all $0)

| Std | Serves | Extract |
|---|---|---|
| **75** Fire Protection of IT Equipment | 12 | the governing standard for module 12 — currently cited by **0 of 63** items |
| **76** Telecommunications Facilities | 12, 11 | telecom-space fire protection |
| **70E** Electrical Safety in the Workplace | 06, ops | **arc flash, LOTO, approach boundaries** — the safety layer the plant model may never compute |
| **110** Emergency & Standby Power Systems | 06 | genset classes/types, testing regime |
| **111** Stored Electrical Energy | 06 | UPS and battery systems |
| **37** Stationary Combustion Engines & Gas Turbines | 06 | generator installation/use |
| **70** NEC | 06 | electrical baseline |
| **70B** Electrical Equipment Maintenance | ops | maintenance intervals |
| **72** Fire Alarm & Signaling | 12 | detection — already cited by 2 items |
| **2001** Clean Agent Extinguishing | 12 | already cited by 2 items |
| **3** Commissioning of Fire Protection Systems | L8 Cx | bank has 6 commissioning items total |
| **4** Integrated Fire Protection System Testing | L8 Cx | **IST** |
| **25** ITM of Water-Based Systems | ops | inspection/test/maintenance cadence |

**Why 70E matters most strategically:** review round 3 forbade the plant model from ever
claiming a sequence is "safe" — not electrically, not arc-flash, not LOTO-correct. That
prohibition stands. 70E is the actual authority on that, now free to read, so the safety
layer becomes **taught and cited content** instead of a permanent blind spot. The model says
"service-preserving under the model"; 70E defines safe. Keep them separate; cover both.

- [x] Free NFPA account created
- [x] **NFPA 75 (2024) "View Free Access" button verified on the live page** — confirmed free

### 🔎 Discovery — committee records are DOWNLOADABLE (verify licence before use)

The NFPA 75 standard-development page offers **Download** links on the process documents,
unlike the standard itself which is view-only:

- First Draft Report · Second Draft Report
- **Public Input with Responses** · **Public Comments with Responses**
- First Revisions Report · Second Revisions Report
- Committee Input Report · meeting notices & agendas

**Why this matters more than it looks:** these carry the *rationale* — why a requirement
exists, what was proposed and rejected, and where the committee split. Public Input/Comment
reports record **where competent experts disagreed**, which is the highest-value assessment
material available: an item built on a genuine committee disagreement tests judgement, while
an item built on a settled requirement tests recall.

**Guardrails:**
- [ ] A "Download" button is **not** a licence grant. Check NFPA Terms of Use before
      vendoring any of it. Record the licence line verbatim; blank licence = ERROR, not
      default-permissive.
- [ ] Committee reasoning is **not normative**. It may never be cited as if it were the
      requirement. Tag any such node `source_class = "committee_rationale"`.

### Scope note worth teaching (from the NFPA 75 page, verbatim scope)

NFPA 75 covers protection of ITE and ITE areas "from fire damage by fire **or its associated
effects — smoke, corrosion, heat, and water**." **Water damage from suppression is inside
scope.** Most fire content treats suppression as purely protective; the standard itself
treats the suppression agent as a damage vector. Check whether any of module 12's 63 items
make this distinction.
- [ ] **Confirm NFPA 75** (Fire Protection of Information Technology Equipment) is in the free viewer
- [ ] **Confirm NFPA 76** (Telecommunications Facilities)
- [ ] Also check **NFPA 70** (NEC) and **NFPA 110** (emergency/standby power) — both are
      directly load-bearing for module 06 (power) and 12 (fire), and NEC 2026 is reportedly free online
- [ ] Record for each: edition year, and the clause numbers that actually govern DC fire
      suppression, detection, and IT-room construction

**Serves:** module 12 (fire), module 06 (power/NFPA 110), module 03 (site/building)
**Never:** paste body text into the repo. Locator + edition + your own paraphrase only.

## 2 · ASHRAE — ⛔ **HUMAN-ONLY LANE. NO AI INGESTION.** (policy read 2026-08-13)

> **ASHRAE AI policy, verbatim from ashrae.org:** *"ASHRAE prohibits the entry of content from
> any ASHRAE publication or related ASHRAE intellectual property (IP) into any AI tool,
> including but not limited to ChatGPT. Additionally, creating derivative works of ASHRAE IP
> using AI is also prohibited without express written permission from ASHRAE."*

**This is stricter than NFPA and the difference is load-bearing.** NFPA restricts
download/print; ASHRAE additionally restricts **AI processing**. Both are "read-only" but they
are not the same permission.

**Binding rules for this project:**

1. **Never paste ASHRAE text into Claude, Codex, ChatGPT, or any agent.** Not standards, not
   the free Commissioning Guide, not the refrigerant PDFs — the policy says *"any ASHRAE
   publication or related IP"*, which is broader than the standards catalogue.
2. **No research agent may be pointed at ASHRAE content.** The L4 (mechanical/thermal) lineage
   prompt must carry this prohibition explicitly before dispatch.
3. **No AI-assisted drafting of anything derived from ASHRAE text** without written permission.
4. **Citation remains fine.** Recording `{ASHRAE 90.4, 2025 edition, §x.y}` is a fact *about*
   a document, not entry of its content. A human may read and record locators by hand.
5. If ASHRAE-derived material is ever wanted in the product, the route is **express written
   permission from ASHRAE**, not a workaround.

**Consequence for grounding:** ASHRAE-dependent claims (notably the TC 9.9 A1–A4 envelope
classes) stay `UNGROUNDED` in the graph unless a human grounds them by hand, outside any AI
tool. Do not treat the read-only viewer as an AI-accessible source.

**Strategic read:** this materially raises the value of PD-GOV data (NOAA, USGS, FEMA, EPA
eGRID, EIA, OSHA/eCFR) — US Government works carry **no** equivalent restriction and are
freely AI-ingestible, quotable, and redistributable. Lead with those.

### Inventory retained below for HUMAN reading only — ✅ INVENTORIED 2026-08-13

**CONFIRMED PRESENT — `Standard 90.4-2025, Energy Standard for Data Centers`**, plus legacy
90.4-2016 / -2019 / -2022 under "Referenced in Code". The core ASHRAE DC standard is
readable at $0.

**CONFIRMED ABSENT — TC 9.9 Thermal Guidelines for Data Processing Environments.** As
predicted: it is a *book*, not a standard under continuous maintenance, so it is not in the
read-only viewer. **The A1–A4 envelope classes and their recommended-vs-allowable numbers
therefore remain `UNGROUNDED`.** This is the single highest-risk hallucination site in the
whole corpus — an agent asked for "the ASHRAE allowable range" will produce confident,
plausible, wrong numbers. Any such value must carry ground_contact=UNGROUNDED until read in
an open secondary source or the book is purchased.

### Read-priority (all RO — record locators, never vendor text)

| Priority | Standard | Serves | Extract |
|---|---|---|---|
| 1 | **90.4-2025** Energy Standard for Data Centers | 06, 09 | MLC/ELC compliance paths, scope, what it does NOT cover |
| 2 | **202-2024** Commissioning Process for Buildings & Systems | L8 Cx | Cx process phases, roles, deliverables — L8 is otherwise ungrounded |
| 3 | **Guideline 36-2024** High-Performance Sequences of Operation | 09, 14 | control sequences — the layer where real cooling failures occur |
| 4 | **188-2021** + **Guideline 12-2023** Legionellosis | **10 water** | cooling-tower risk management. Module 10 has 35 items; verify any cover this |
| 5 | **Guideline 1.4-2019** Preparing Systems Manuals | 15 ops | O&M documentation → the ancestor of MOP/SOP/EOP |
| 6 | **135-2024** BACnet | 14, 11 | BMS/EPMS/DCIM protocol reality |
| 7 | **169-2025** Climatic Data for Building Design | 03 site | economiser hours by location → feeds L1 site selection |
| 8 | **15-2024** / **34-2024** Refrigeration safety + refrigerant classes | 09 | A2L/low-GWP in modern CDUs — safety classification |
| 9 | **111-2024** Testing, Adjusting, Balancing | L8 Cx | TAB as a commissioning deliverable |
| 10 | **90.1-2025** Buildings | 03, 04 | the envelope/mechanical baseline 90.4 references |

**Why 2 and 3 matter more than they look:** the public postmortems that drive L3
(Google `europe-west2-a`, AWS Tokyo) were **control-dependency** failures, not capacity
failures. Guideline 36 specifies the layer that actually failed. Commissioning (202) is the
process that would have caught it.

## 3 · ASHRAE free resources — **RESOLVED: RO, human-read-only, never vendor**
`ashrae.org/technical-resources/free-resources`

**Verdict (2026-08-14, `bd-ashrae-redistribution-contradiction-p5n`): no ASHRAE bytes live in
this repo, in any format, regardless of price.** Not "check the licence" — the answer is
already known and does not vary per document (SN-4).

- **2026-08-12:** three ASHRAE TC 9.9 PDFs purged from HEAD **and git history**; only the
  `.meta.toml` sidecars remain, for url + sha256 grounding.
- **2026-08-14:** two text captures that survived that purge —
  `public/src-ashrae-datacom.txt`, `public/src-ashrae-power-wp.txt` — removed from HEAD and
  replaced with `capture = "citation-only"` rows in `public/manifest.json`.

**Do not re-capture.** `access = "free"` on an ASHRAE page is a price fact and authorises
nothing (SN-1). ASHRAE's AI policy makes this stricter than ordinary "unlicensed": an agent
may not *read* the material, so it may not summarise, paraphrase, or extract from it either.
Only a human may read it, and only a clause/table locator comes back.

- [x] Datacom-related free downloads identified → all are RO for our purposes
- [x] Licence posture recorded per source in `free-pdfs/*.meta.toml` and `public/manifest.json`

## 4 · Uptime Tier Standard: Topology — **REG**, free download
`uptimeinstitute.com/resources/asset/tier-standard-topology`

- [ ] Download; record the copyright/permission line **verbatim**
- [ ] Extract the four Tier definitions and the performance-confirmation test concept
- [ ] **The high-value capture:** what Topology explicitly does NOT cover — Uptime's own
      material says topology and operational sustainability are separate, and that topology
      excludes codes, weather, security. That exclusion is exactly the `contradicts` axis
      against TIA-942 Rated and EN 50600 Class.

**Serves:** module 01 (mission-critical), 02 (standards) — and the L3 conflict map, which is
the highest-value curriculum content in the whole research wave.
**Note:** repo already holds `corpus/public/src-uptime-tiers.txt` — reconcile, don't duplicate.

## 5 · LBNL / DOE-FEMP Best Practices Guide (rev. 2024-07) — **RESOLVED: DO NOT VENDOR**
`datacenters.lbl.gov/…/best-practice-guide-data-center-design.pdf`

**Verdict (2026-08-14, `bd-doe-guide-embeds-ashrae-p54`): cite by URL and section locator
only. Never vendor.** Decision record:
`free-pdfs/lbnl_femp_best_practice_guide_dc_design.meta.toml`.

Round 2's suspicion is now **confirmed** by an attribution-line scan (credit lines read;
table and figure *content* deliberately not read):

- "Table 3-1. ASHRAE 2021 Thermal Guidelines for Air Cooling"
- "Source: Thermal Guidelines for Data Processing Environments, ASHRAE" (×3)
- "Source: Emergence and Expansion of Liquid Cooling in Mainstream Data Centers, ASHRAE" (×2)

Two independent blockers, either sufficient (SN-3, SN-4):

1. **Rights.** 17 USC 105 covers the government-*authored* portions only. Vendoring the PDF
   into this public repo republishes ASHRAE tables. `rights` is recorded as
   `mixed-us-government-work-with-third-party-material`, **not** bare `public-domain`.
2. **AI policy.** ASHRAE prohibits entry of ASHRAE IP into any AI tool. The prohibition binds
   on content, not container — reading Table 3-1 violates it through the government wrapper
   just the same.

**A redacted extract was considered and rejected for agents.** Producing one requires
locating the ASHRAE pages in order to exclude them, i.e. opening them. The redaction act is
itself the violation. A human may do it; an AI tool may not.

**Consequence, accepted deliberately:** Table 3-1 is the A1–A4 thermal envelope this project
marks `UNGROUNDED`. This guide is a side-door to exactly that data. It stays ungrounded until
a **human** reads the source and supplies the values. `Gap_rci-unusable-without-the-envelope`
stays open on purpose — that gap is honest, and closing it via this PDF would not be.

**Still usable:** the DOE-authored chapters, cited by section number with nothing vendored.
The graph already does this correctly — 25 nodes cite §4.x air management, §5.x cooling and
§6.x electrical. **§3.x / Table 3-1 is off limits to any AI reader.**

**Serves:** modules 04, 06, 09, 10 (design/energy). **Scope limit:** energy-efficient *design*
only — it cannot validate fire, structural, security, or Tier terminology.

## 6 · LBNL Center of Expertise — **PD**
`datacenters.lbl.gov`

- [ ] Tools: DC Pro, PUE estimators, air-management assessment
- [ ] These are `Tech` nodes for the graph — each needs an `addresses` edge to a real Gap

## 7 · Open Compute Project specs — **open licence, VERIFY per-doc**
`opencompute.org/specs`

- [ ] OCP licences vary per specification (OWL / OCPHL, permissive vs reciprocal)
- [ ] Record the licence **per document** before vendoring anything
- [ ] Highest value: rack/power shelf, busway, and liquid-cooling specs — real hardware detail
      an interviewer probes and the current bank almost certainly lacks

**Serves:** module 08 (racks), 06 (power), 09 (cooling)

## 8 · Google SRE books — **free to read, © Google/O'Reilly — RO**
`sre.google/books`

- [ ] Postmortem Culture, Emergency Response, Incident Response chapters
- [ ] **Cite and link only. Never vendor.**
- [ ] Capture the *form*: postmortem structure, incident command roles, escalation. The form is
      teachable and is what an interviewer actually probes.

**Serves:** module 15 ops-adjacent (the 39 orphan items with no Learn surface), W5 runbook discipline

---

## Landing the results

For anything **downloadable and cleared**: `knowledge/corpus/free-pdfs/<slug>.pdf` +
`<slug>.meta.toml` recording `source_url`, `retrieved_at`, `licence`, `licence_verbatim`,
`authors`, `third_party_figures`, `scope_limit`.

For anything **read-only**: no file. Record a citation-registry row —
`{citation_id, authority, edition, clause_locator, verified_by, verified_at, access = "read-only"}`.
This is the *evidence conformance* backbone, not a factual oracle: it proves the locator was
checked by a human, not that the claim is true.

**Anti-vacuous rule, per this repo's own discipline:** a source entry with no licence line
recorded is an ERROR, not a default-permissive. Blank licence must fail review.
