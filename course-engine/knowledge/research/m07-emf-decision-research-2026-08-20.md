# M07 EMF decision-level source research — 2026-08-20

Consumer: the human reviewer deciding which M07 recall items can be converted to apply-level items.

Feature gated: a sourced conversion queue for the M07 EMF module; a reviewer must be able to turn a fact into a real site, routing, protection, or survey decision without inventing a practice.

Observed defect: Q20 classified 9 of 21 M07 recall items as not-convertible. The module is thin (26,299 characters), but the defect is level, not corpus size: source-list, unit, definition, and mechanism-recall items do not yet carry a documented practitioner decision.

Deletion condition: delete this research receipt and its fragment when every disposition below has been reviewed and either grounded in a changed item or recorded as an explicit unresolved gap in the maintained conversion ledger.

## Source bar

Only public official catalog, regulation, or vocabulary pages were used. No paid standard body or shadow archive was fetched.

### IEC TR 61000-5-1:2023, edition 2.0

Official catalog: <https://webstore.iec.ch/en/publication/61223>. The catalog describes EMC mitigation for industrial, commercial, and residential installations, identifies cable selection/installation, shielding, filters, isolating transformers, and surge protective devices as techniques, and places selection of the appropriate technique for the particular installation on the designer or installer. Catalog edition 2.0, published 2023-02-23.

Decision use: a scenario can require choosing or checking an installation mitigation technique. This does not establish a universal EMF exposure limit or prove the Faraday mechanism by itself.

### IEC TS 61000-5-10:2017, edition 1.0

Official catalog: <https://webstore.iec.ch/en/publication/30054>. The catalog describes guidance for protecting commercial facilities from HEMP/IEMI and says it applies to existing or new buildings when the customer has decided that protecting critical electronics is important.

Decision use: a scenario can test whether a facility has crossed the customer/program boundary for specialist HEMP/IEMI protection. It does not make ordinary commercial designs HEMP-hardened by default.

### IEC 62271-208:2025, edition 1.0

Official catalog: <https://webstore.iec.ch/en/publication/69470>. The catalog provides practical guidance for evaluating and documenting external steady-state power-frequency magnetic and electric fields from high-voltage switchgear and substations, including measurement or calculation where practicable. It expressly does not specify EMF limits or human-exposure methods.

Decision use: a site-adjacency scenario can require an evaluation/documentation decision, while preserving the important conclusion that this source is not a universal relocation threshold.

### OSHA 29 CFR 1910.308

Public regulation: <https://www.osha.gov/laws-regs/regulations/standardnumber/1910/1910.308>. The communications-circuit provisions specify separation from power/lightning conductors and require listed primary protection for interbuilding circuits exposed to lightning; the regulation gives concrete separation and protection conditions rather than a generic source list.

Decision use: a communications-routing scenario can test whether a proposed route meets the public separation/protection rule. This is an electrical-safety/routing source, not evidence that every induced-noise mechanism has the same legal threshold.

### Scope boundary: human exposure is not equipment immunity

ICNIRP's public low-frequency page, <https://www.icnirp.org/EN/FREQUENCIES/LOW-FREQUENCY/INDEX.HTML>, explains basic restrictions and derived reference levels for human exposure. IEC 61786-2:2014's public catalog, <https://webstore.iec.ch/en/publication/5907>, covers measurement of quasi-static magnetic and electric fields and explicitly includes human exposure in its scope. Neither source supplies an IT-equipment immunity or relocation threshold. This boundary prevents a human-exposure limit from being silently repurposed as an equipment-design rule.

## Item dispositions

### Unblocked for human conversion review

These are source-backed candidates, not changed bank items. A future rewrite must still pass a human correctness review.

- `m07-q044` — **unblocked**. Replace the source-list recall with a site adjacent to HV switchgear/substation equipment where the professional must choose measurement or calculation and document the field evaluation. IEC 62271-208 supplies that decision and its scope boundary; it also prevents inventing a universal EMF limit.
- `m07-q052` — **unblocked with a boundary**. Replace the mechanism-only recall with a communications-routing case requiring the candidate to select a compliant separation/protection response when power and data routes share a path. OSHA 1910.308 and IEC TR 61000-5-1 ground the routing/mitigation decision. The source set does not by itself ground a claim that OSHA's distance rule is a complete model of Faraday-induced noise.
- `m07-q054` — **unblocked**. Replace awareness recall with a customer decision: a facility has identified critical electronics whose protection matters and must decide whether a HEMP/IEMI protection program and specialist evaluation are in scope. IEC TS 61000-5-10 grounds the customer/program boundary. It does not support claiming that standard colocation is hardened by default.
- `m07-q055` — **unblocked**. Replace the source list with a site-adjacency triage case: an HV substation or other external source is nearby, and the candidate must choose evaluation/documentation rather than assert a universal limit; a lightning-exposed interbuilding communications route adds a concrete protection check. IEC 62271-208 and OSHA 1910.308 ground the two decisions, with their different scopes kept explicit.

### Still blocked or scope-limited

- `m07-q042` — **blocked for the current equipment-focused conversion**. IEC 61786-2 and ICNIRP can ground a human-exposure measurement scenario, but not an equipment-immunity or relocation decision. Converting V/m recall without choosing and sourcing the population/scope would conflate domains.
- `m07-q043` — **blocked for the current equipment-focused conversion**. The public IEC catalog grounds the measurement quantity and range, but not a practitioner choice that follows from Tesla/microtesla units alone.
- `m07-q049` — **blocked as written**. IEC vocabulary grounds the EMI definition; IEC TR 61000-5-1 can support a new mitigation-selection item, but not a source-backed conversion of the acronym definition without changing the proposition.
- `m07-q201` — **blocked as written**. Public IEC vocabulary distinguishes magnetic field strength from flux density, but the current unit question does not imply a sourced site decision. A rewrite would need a separately sourced measurement-scope decision and would not be a mere conversion.
- `m07-q207` — **blocked as written**. The current item asks for the basic current-flow association. IEC 62271-208 can support evaluating fields around HV equipment, but it does not by itself supply the causal mechanism or a unique mitigation choice for this stem.
- `m07-q023` — **retired** in the Q20 inventory; no conversion work proposed.

## Should-fail search

I specifically searched the official IEC catalog pages above, ICNIRP's low-frequency guidance, IEC 61786-2, and public OSHA electrical/communications provisions for a vendor-neutral data-centre equipment-immunity threshold or universal `µT`/`mG` relocation or survey trigger. I did not find one. IEC 62271-208 expressly avoids specifying EMF limits, while ICNIRP and IEC 61786-2 are human-exposure/measurement scope. That claim remains a `Gap`; it must not be invented to make `m07-q042`, `q043`, or `q201` convertible.

## Tick result

No bank item changed in this pass, so no learner-facing product tick is claimed. The four IDs above are now source-backed candidates for human conversion review; the graph fragment is a traceable research byproduct, not certification that any item has been improved. Passing local grounding gates would establish repository grounding shape only, not the truth or pedagogical quality of a future rewrite.
