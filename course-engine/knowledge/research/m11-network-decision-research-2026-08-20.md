# M11 network decision-level source research — 2026-08-20

Consumer: the human reviewer deciding which M11 recall items can be converted to apply-level items.

Feature gated: a sourced conversion queue for the M11 network/cabling module; a reviewer must be able to turn a cabling fact into a real selection, acceptance, or failure-response decision.

Observed defect: Q20 classified 18 of 42 M11 recall items as not-convertible. The corpus is substantial, but much of the blocked set names topology spaces, connector types, or demarcation terms without exposing a sourced decision condition.

Deletion condition: delete this receipt and its fragment when each disposition is reviewed and either grounded in a changed item or recorded as an explicit unresolved gap in the maintained conversion ledger.

## Sources and decision-level limits

Only official TIA, ISO, and IEC public pages were used. No paid standard body, PDF body, vendor blog, or shadow archive was fetched.

### ANSI/TIA-942-C, May 2024

Official TIA catalog/abstract: <https://tiaonline.org/standard/tia-942/>. TIA identifies revision C, published May 2024, as the Telecommunications Infrastructure Standard for Data Centers; its abstract says the standard specifies data-centre and computer-room infrastructure requirements and that its topology is intended for any size data centre.

Official TIA standards article: <https://standards.tiaonline.org/big-data-overload>. The public article describes cabling design, pathways and spaces, reliability, redundancy, scalability, and physically separated services/cross-connected areas/pathways. It is useful for decision framing, but the current TIA-942-C abstract does not expose the internal MDA/HDA/EDA definitions or a universal distance rule.

### ISO/IEC 11801-5:2017

Official ISO preview: <https://www.iso.org/standard/62247.html>. The public abstract specifies generic cabling within and to computer-room spaces in data-centre premises. It grounds a scenario about whether a proposed cabling change is within the data-centre generic-cabling system, but the public preview does not expose the item-level topology vocabulary used by the M11 definitions.

### IEC 60793-2-10:2019+A1:2022, consolidated edition 7.1

Official IEC catalog: <https://webstore.iec.ch/en/publication/73767>. The catalog covers A1-OM1 through A1-OM5 multimode fibre categories, their bandwidth grades and macrobend-loss suffixes, and explicitly lists short-reach, high-bit-rate and intra-/inter-building data-centre applications.

Decision use: a scenario can require selecting a multimode category/application and checking that the candidate is appropriate for the link. The catalog does not yield one universal distance for every OM grade.

### IEC TR 61282-15:2017

Official IEC catalog: <https://webstore.iec.ch/en/publication/34000>. The catalog gives guidance for testing multi-fibre cable plant terminated with MPO connectors in data-centre premises, including attenuation, polarity, length, and optical-return-loss measurements.

Decision use: a commissioning scenario can require choosing the acceptance measurements for an MPO plant. It does not make the connector name alone a performance result.

### IEC 61300-3-35:2022 and IEC TR 62627-01:2023

Official IEC catalogs: <https://webstore.iec.ch/en/publication/64254> and <https://webstore.iec.ch/en/publication/72878>. IEC 61300-3-35 defines visual inspection criteria for fibre end faces and says visual inspection is additional to, not a replacement for, attenuation and return-loss measurement. IEC TR 62627-01 provides connector-cleaning methods and procedures.

Decision use: a fault-response scenario can require inspection/cleaning followed by optical qualification; visual cleanliness alone is not sufficient acceptance evidence.

## Item dispositions

### Unblocked for human conversion review

These are source-backed candidates, not changed bank items. A future rewrite still requires a human correctness review for one defensible answer and a real operating scenario.

- `m11-q111` — **unblocked**. Turn the multimode association into a link-selection case: given a short-reach, high-bit-rate intra-building data-centre link and a specified OM category/application, choose the candidate that remains within the sourced fibre category and application scope. IEC 60793-2-10:2019+A1:2022 supplies the category, bandwidth, macrobend, and data-centre application constraints. Do not add a universal metre claim.
- `m11-q113` — **unblocked**. Turn the MPO definition into a commissioning case: an MPO-terminated plant is ready for acceptance, and the candidate must choose the evidence set including attenuation, polarity, length, and optical return loss, with visual inspection/cleaning treated as supporting work rather than a substitute. IEC TR 61282-15:2017, IEC 61300-3-35:2022, and IEC TR 62627-01:2023 supply the decision and its boundary.

### Still blocked or scope-limited

- `m11-q032` and `m11-q033` — **retired**; no conversion work proposed.
- `m11-q101` — **blocked**. The current MMR proposition is a definition of interconnection use. The public TIA/ISO abstracts establish data-centre cabling scope but do not expose a source-backed carrier-demarcation or cross-connect decision specific enough to rewrite it safely.
- `m11-q103`, `m11-q104`, `m11-q105`, `m11-q106`, `m11-q107`, `m11-q129`, `m11-q130`, `m11-q201`, `m11-q217`, `m11-q218` — **blocked as exact topology-term conversions**. TIA-942-C publicly confirms data-centre infrastructure and topology scope, and the TIA article confirms pathways, spaces, redundancy, and scalability, but neither public page exposes the exact MDA/HDA/EDA/ZDA/entrance/backbone/horizontal propositions needed to ground a new answer set. A reviewer could create a generic pathway-failure scenario, but that would be a new proposition rather than a defensible conversion of the named term.
- `m11-q123` — **blocked**. The public TIA article mentions cross-connected areas and pathways, but does not define the cross-connect/interconnect distinction in the current item. Rewriting it would require importing an unverified terminology boundary.
- `m11-q138` — **blocked**. The public pages establish data-centre cabling infrastructure but do not expose the provider-responsibility transition represented by demarcation. Do not invent a service-contract rule.

## Should-fail search

I searched the official TIA-942-C page, TIA's public cabling/pathway article, ISO/IEC 11801-5's public abstract, and IEC catalog pages for a public exact definition or decision clause for MDA/HDA/EDA/ZDA, carrier demarcation, and a universal OM3/OM4/OM5 reach number. I did not find a legal public clause that supports those exact topology rewrites or one universal multimode distance. The sources explicitly support scope, application classes, tests, and failure-response boundaries instead. Those unresolved claims remain `Gap` nodes rather than being filled from memory.

## Tick result

No bank item changed in this pass, so no learner-facing product tick is claimed. The two IDs above are now source-backed candidates for human conversion review; the graph fragment is a research byproduct, not evidence that the item has been improved. Passing a grounding gate would establish repository grounding shape only, not truth, uniqueness, or pedagogical quality of a future rewrite.
