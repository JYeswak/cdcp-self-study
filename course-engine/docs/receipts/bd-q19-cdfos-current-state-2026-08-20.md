# Q19 — CDFOS daily-ops current state

Date: 2026-08-20

## Finding

The current tree contains 26 approved CDFOS-tagged M15 items. Every one points
to the public CDFOS syllabus page:

<https://www.epi-ap.com/services/1/3/136/Certified_Data_Centre_Facilities_Operations_Specialist_(CDFOS)>

The five `bd-epi-ecosystem-ms4j.2.*` children are already closed in the local
bead database. Their item work is present in `bank/items`; no new item is
written in this receipt. The parent `.2` remains open, correctly, because its
acceptance still requires the ACK-gated track/module surface and a map/link
integration. The current tree has no `tracks/cdfos/manifest.toml` and no
`web/tracks/cdfos.html`; `docs/curriculum-grades/epi-ecosystem-map.md` still
lists CDFOS as planned/in progress.

## Measured heading coverage

| Public CDFOS heading family | Approved item ids |
|---|---|
| Service Level Management — service portfolio/catalogue, SLA, SIP, reporting, customer satisfaction | m15-q352–q354, m15-q376–q377 |
| Safety and Crisis Management — emergency response, appointed safety staff, OH&S/WHS manual, PTW, LOTO | m15-q355, m15-q369, m15-q375, m15-q228–q231 |
| Data Centre Operations — shift handover, floor management, walk-around duties | m15-q356, m15-q359, m15-q374 |
| Facilities Maintenance — MOP, tools, service reports | m15-q357, m15-q362, m15-q370 |
| Physical Security — security SOP, delivery/holding area, incident reporting | m15-q358, m15-q371–q372 |
| Monitoring / Reporting / Control — facilities matrix and notification/escalation | m15-q232–q233, m15-q360 |
| Governance and Compliance — document management and asset recording | m15-q361, m15-q373 |

Count check: 26 files, 26 approved, 0 retired. The item comments retain the
public heading and URL; no EPI/EXIN certification or pass claim is made.

## Honest residuals

- A standalone OLA taxonomy is not added. The current material supports the
  bounded SLA/OLA/underpinning-contract distinction in existing items, but not
  an invented universal OLA programme or taxonomy.
- The six-step document-management taxonomy is not expanded beyond the
  bounded, sourced document-management item; no public official source in the
  current ledger supports the six undocumented subprocesses.
- A universal shift-handover programme is not claimed; the existing shift-
  handover item is bounded to its cited operational continuity proposition.
- The bead history refers to vendor lifecycle items `m15-q234` and `m15-q235`,
  but those files are absent from this tree. They are not silently recreated:
  the current item set does not supply a citable, current proposition for
  select/score/underpinning-contract/performance as a new CDFOS item.

These are sourcing/ACK blockers, not evidence that the CDFOS headings are
missing from the existing item bank. The parent `.2` and the ms4j epic remain
open; completing this track does not certify anyone.
