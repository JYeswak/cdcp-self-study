# STANDARDS KNOWLEDGE BASE — validation without piracy

## What “validated against IEC” means here

We **do not** ship offline full-text ISO/IEC/TIA libraries unless Joshua holds a license.

We **do** maintain a **citation graph**:

```text
CDCP domain → topic atoms → items → standards families → free/public or licensed evidence
```

| Phrase | Meaning |
|--------|---------|
| Aligned to public CDCP domain map | Syllabus coverage gate |
| Cross-referenced to ISO/IEC 22237 **series structure** | Part map + domain crosswalk |
| Grounded in free ASHRAE TC 9.9 pubs where cited | Fact lint |
| Clause-by-clause to paid full text | Only with licensed audit — not default |

**Badge:** `Standards literacy: family-mapped · free-evidence cited · not SDO certified`

## Epistemic layers

0. Codes (AHJ): NFPA, local electrical/fire  
1. Paid SDO full text: ISO/IEC 22237, EN 50600, TIA-942, BICSI-002  
2. Free guidelines: ASHRAE TC 9.9 white papers, TIA overviews  
3. Industry frameworks: Uptime Tier, OCP Ready  
4. Training maps: EPI/EXIN public syllabus + exam form  
5. Our original teaching (must cite 0–4 when load-bearing)

## Families (see `knowledge/standards_families.toml`)

- ISO/IEC 22237 (parts 1–7 structure)  
- EN 50600 (European sibling family)  
- ANSI/TIA-942 (Rated 1–4; not “Uptime Tier”)  
- Uptime Institute Tier Standard (contrast only)  
- ASHRAE TC 9.9  
- NFPA 75/76/110 (+ AHJ)  
- OCP (optional hyperscale lens)  

## Validation modes

| A Syllabus coverage | Public CDCP topics have items |
| B Standards family coverage | Each domain ≥1 family + public source |
| C Fact hygiene | Numbers need free_url \| licensed_note \| qualitative_only |

Gates (when CLI lands): `standards-map verify` · `facts lint` · `sources hygiene`

## Forbidden

- Committing pirated standard PDFs  
- Using exam dumps as “standards knowledge”  
- Claiming TIA/Uptime/IEC certification from this course  

## Open research

See [`research/STANDARDS-TENSIONS.md`](./research/STANDARDS-TENSIONS.md).
