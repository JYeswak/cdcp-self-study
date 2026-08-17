# CDCP Franken research season — 2026-08-17

Season: `cdcp` (NTM). Host: joshs-brain. Live repo: this working directory.
Studio copy at the same path is scaffold-only. Do not teach from it.

## Hard rules

RESEARCH ONLY. No build. No `cargo build`, `cargo test`, `cargo run`. No WASM rebuild.
No module prose rewrites (`modules/*.md`). No CHARTER edit. No HOW-TO-USE rewrite.
No `br close` / `bd close` of product beads. No exam dumps. No proprietary EPI body text.
No invented Uptime / outage percentages. Completing this program does not certify anyone.
Original items only. Never `ntm send --all`.

If you file a NEW research bead it must have WHAT / WHY / ACCEPTANCE and a VERDICT
in-bead. Prefer writing markdown in this folder over new beads. Do not close existing
children of `bd-curriculum-truth-ebrr` or `bd-epi-ecosystem-ms4j`.

## Arsenal first

Do not re-derive primitives. Read, then use:

- `~/.grok/skills/charter/references/frankensuite-arsenal.md`
- `/Users/josh/Developer` as flat siblings (do not clone Dicklesworthstone repos)
- Canonical skills on Studio are symlinked; Brain copies of `ntm` / `vibing-with-ntm` may be older. You are already inside NTM.
- Beads: from `course-engine/` use `br` (on Brain, `bv` may be aliased to `br show`; real `bv` is `/Users/josh/.local/bin/bv`). Agents: `bv --robot-*` never bare `bv`. Use `-label` / `-l` as this host accepts.

## Read before you write

1. `CHARTER.md`, `00-curriculum-map.md`, `HOW-TO-USE.md`
2. `docs/curriculum-grades/` — all `pass-0*.md`, pedagogy, practice sittings, `epi-ecosystem-map.md`
3. `modules/01-mission-critical.md`, `02-standards.md`, `15-ops-adjacent.md` (spot-check others)
4. `practice/PRACTICE-EXAM.md`, `practice/DRILL-CARDS.md`
5. Open beads: `bd-curriculum-truth-ebrr` (+ children), `bd-epi-ecosystem-ms4j` (+ children)
6. Public web: EPI Data Center Training Framework pages, Uptime Institute 2026 Annual Outage Analysis (cite, do not paste proprietary tables as law), ASHRAE TC 9.9 2021/2026 thermal guidelines headlines, NFPA 855 / UL 9540A headlines, TIA-942-C headlines, EN 50600 / ISO 22237 / ISO 30134 headlines

## Starting gap register (validate, contradict, or extend — do not rubber-stamp)

**Syllabus / 2.1.** Service catalog, SLM/OLA, org, training program, security matrix. Map titled “14 Modules.” M15 = D on domain 2.1.

**2026 floor.** W-classes. Neocloud / GPU colo / AI factory / behind-the-meter. Interconnect queue. NFPA 855 / UL 9540A. Thermal ride-through as seconds at 40–100 kW, not minutes. Catcher as a drawable. Liquid-ready racks, GPU east-west fabric, CDU-loop leak, 2026 EOPs.

**Honesty / chrome.** CHARTER SRS contradiction. Unsourced “most outages” and 20–40% energy lines. Mock toolbar “27” without “not a pass mark.” learn.html “Fourteen EPI domains.”

**Framing.** M01/map still examine the 2015 three-bucket cartoon. M15 already teaches contributor-vs-root. PRACTICE Q4 and `mock40-q04` still key the leftover cartoon.

**Standards (M02 thin).** Availability Class unnamed. EN 50600 unnamed. Three Uptime plaques not taught as three. TIA-942-C unnamed. Cx and ISO 30134 omitted.

**Orals / drawings.** M06 mermaid collapses A/B through one ATS. No integrated tour demarc → dual-cord → CRAH/liquid → fire. One-line-as-CM is not an item type.

**Quiz / assemble.** M15 apply items unassembled. 2.1 zero bank. Assemble under-draws catcher, W-class, one-ATS SPOF, scope-of-nines. Corpus is `qualitative_only` — no 3φ, psychrometric, or battery math.

**Formulas.** Name-and-kill 99.982%. N+2. Example floor psf. ASHRAE speakable band. UPS autonomy band. 1φ P=V×I×PF.

**Pedagogy.** Unit-as-default, Continue-in-unit, Drill-10 home, one produced artifact per heavy module. No cert XP. Grade: B content / C packaging / D+ delight.

**Unbuilt EPI tracks (public framework, not credentials).** CDCS, CDFOS, CDFOM, CDESS, CDRP, CDMS, CNCDP, CDCE, CTDC, CTIA/CTLA, chrome+engine, 2026 overlay. Proposed ship order: CDCS + CDFOS first.

**Engine (context only — do not implement).** Installability epic N `bd-installability-sm4g`, invert-the-graph, machine ledgers, historical syspolicyd halt.

## Output (write files here, not to chat only)

- `FINDINGS.md` — ranked gap register. Each row: gap, evidence (file:line or URL), severity, already-beaded?, recommend (keep / rewrite-when-ACK / new-bead / drop-as-folklore)
- `SOURCES.md` — every public source you actually opened, with one-line what it confirmed or killed
- `VERDICT.md` — 20-line executive. What is real. What we over-claimed. What a Fluidstack-style oral would fail on tomorrow. What CDCS science is still missing.

Do not start implementing remaining open beads. Reality-check YES means “implementing them later would produce the thing.” This season stops at the register.
