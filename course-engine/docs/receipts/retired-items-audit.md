# Retired-items audit

Audit only: at the audited snapshot, no `bank/items` or `crates/` file was
changed by this audit. The later required-tests work is separate from these
findings.

The current snapshot is `87d7850` and the comparison baseline is `955a8f1`
(`2026-08-17 23:59:38 -0600`). The grounding-wave inventory begins at
`3a52cca` (`2026-08-18 18:34:51 -0600`), so the baseline is before that wave.
Each current retired TOML file was parsed with `tomllib`; only the parsed
`stem`, `choices`, `correct`, and `explanation` values were compared. Raw TOML
formatting and comments were not used as drift evidence.

## Carry-forward human judgement

These are still open from the choices-integrity review and are unrelated to the
retired-item denominator:

- `m10-q115` — **UNSURE**: the current answer preserves water-dependent
  cooling/humidity reasoning but omits the optional fire-system dependency.
- `m13-q208` — **UNSURE**: lost/stolen-credential mitigation and authentication
  assurance are related but not identical rationales.

## Denominators and verdicts

| Measure | Count |
| --- | ---: |
| Current retired item files | 26 |
| Items compared against `955a8f1` | 26 |
| Items with no `955a8f1` baseline | 0 |
| Parsed stem/choices/correct/explanation differences | 0 |
| CLEAN | 26 |
| DRIFTED | 0 |
| RETIRED-BY-WAVE | 0 |

Every current retired item was already `status = "retired"` at `955a8f1`.
The status history from `955a8f1..87d7850` contains no status transition for
any of the 26 files. Therefore none was retired by the grounding wave.

## Per-item results, grouped by module

`parsed diff` is the set of differing fields in
`stem/choices/correct/explanation`; `none` means all four parsed values match
the baseline.

### M05

| ID | Status at baseline | Retirement provenance | Parsed diff | Verdict |
| --- | --- | --- | --- | --- |
| `m05-q200` | retired | `acd4d43` — wave 7, 2026-08-14 | none | **CLEAN** |

### M06

| ID | Status at baseline | Retirement provenance | Parsed diff | Verdict |
| --- | --- | --- | --- | --- |
| `m06-q014` | retired | `acd4d43` — wave 7, 2026-08-14 | none | **CLEAN** |
| `m06-q016` | retired | `acd4d43` — wave 7, 2026-08-14 | none | **CLEAN** |
| `m06-q018` | retired | `acd4d43` — wave 7, 2026-08-14 | none | **CLEAN** |
| `m06-q019` | retired | `acd4d43` — wave 7, 2026-08-14 | none | **CLEAN** |
| `m06-q020` | retired | `acd4d43` — wave 7, 2026-08-14 | none | **CLEAN** |
| `m06-q021` | retired | `acd4d43` — wave 7, 2026-08-14 | none | **CLEAN** |

### M07

| ID | Status at baseline | Retirement provenance | Parsed diff | Verdict |
| --- | --- | --- | --- | --- |
| `m07-q023` | retired | `acd4d43` — wave 7, 2026-08-14 | none | **CLEAN** |

### M08

| ID | Status at baseline | Retirement provenance | Parsed diff | Verdict |
| --- | --- | --- | --- | --- |
| `m08-q024` | retired | `acd4d43` — wave 7, 2026-08-14 | none | **CLEAN** |
| `m08-q025` | retired | `acd4d43` — wave 7, 2026-08-14 | none | **CLEAN** |

### M09

| ID | Status at baseline | Retirement provenance | Parsed diff | Verdict |
| --- | --- | --- | --- | --- |
| `m09-q026` | retired | `acd4d43` — wave 7, 2026-08-14 | none | **CLEAN** |
| `m09-q029` | retired | `acd4d43` — wave 7, 2026-08-14 | none | **CLEAN** |
| `m09-q221` | retired | `acd4d43` — wave 7, 2026-08-14 | none | **CLEAN** |
| `m09-q226` | retired | `acd4d43` — wave 7, 2026-08-14 | none | **CLEAN** |
| `m09-q301` | retired | `643da115` — orphan cleanup, 2026-08-17 | none | **CLEAN** |

### M10

| ID | Status at baseline | Retirement provenance | Parsed diff | Verdict |
| --- | --- | --- | --- | --- |
| `m10-q022` | retired | `acd4d43` — wave 7, 2026-08-14 | none | **CLEAN** |
| `m10-q031` | retired | `acd4d43` — wave 7, 2026-08-14 | none | **CLEAN** |

### M11

| ID | Status at baseline | Retirement provenance | Parsed diff | Verdict |
| --- | --- | --- | --- | --- |
| `m11-q032` | retired | `acd4d43` — wave 7, 2026-08-14 | none | **CLEAN** |
| `m11-q033` | retired | `acd4d43` — wave 7, 2026-08-14 | none | **CLEAN** |
| `m11-q226` | retired | `acd4d43` — wave 7, 2026-08-14 | none | **CLEAN** |

### M12

| ID | Status at baseline | Retirement provenance | Parsed diff | Verdict |
| --- | --- | --- | --- | --- |
| `m12-q217` | retired | `acd4d43` — wave 7, 2026-08-14 | none | **CLEAN** |
| `m12-q219` | retired | `acd4d43` — wave 7, 2026-08-14 | none | **CLEAN** |

### M13

| ID | Status at baseline | Retirement provenance | Parsed diff | Verdict |
| --- | --- | --- | --- | --- |
| `m13-q036` | retired | `acd4d43` — wave 7, 2026-08-14 | none | **CLEAN** |
| `m13-q037` | retired | `acd4d43` — wave 7, 2026-08-14 | none | **CLEAN** |

### M14

| ID | Status at baseline | Retirement provenance | Parsed diff | Verdict |
| --- | --- | --- | --- | --- |
| `m14-q039` | retired | `acd4d43` — wave 7, 2026-08-14 | none | **CLEAN** |
| `m14-q040` | retired | `bbc5cc97` — wave 6, 2026-08-14 | none | **CLEAN** |

## Historical wave note

Six retired IDs (`m06-q018`, `m06-q019`, `m06-q021`, `m08-q024`,
`m08-q025`, and `m10-q022`) appear in the grounding-wave classification as
rewritten items. Their parsed fields match `955a8f1` in the audited snapshot,
so they are **CLEAN now**, not current `DRIFTED` findings; the repair restored
their values. This receipt does not claim they were never transiently changed.

The result closes the retired-item comparison gap at this snapshot: all 26
retired files are pre-wave retirements with no remaining parsed body drift.
It does not certify the quality of the `955a8f1` baseline.
