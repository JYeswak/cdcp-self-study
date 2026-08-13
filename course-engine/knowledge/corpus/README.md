# Grounding corpus

**Policy (OQ-09, Josh 2026-08-12):** Store as much **free/public** source material as possible in-repo for offline reference and grounding.

| Path | Contents |
|------|----------|
| `public/` | Text/HTML snapshots of public pages (UTF-8), provenance in `manifest.json` |
| `free-pdfs/` | Free redistributable-or-publicly-posted PDFs (ASHRAE TC 9.9 WPs/TB, NIST SP) with `*.meta.toml` sidecars |
| *(never)* | Paid ISO/IEC/TIA/NFPA/BICSI full text (OQ-10 until buy decision); exam dumps |

Each free PDF should have:
- `*.pdf` binary
- `*.meta.toml` — source_id, url, fetch_date, sha256, org, title, access=free

Paid catalog URLs stay in `knowledge/sources.toml` as `access = "paid"` with **no** full-text file.
