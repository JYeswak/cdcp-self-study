# cdcp-course engine

Local-first **HTML course + browser grading** for the CDCP self-study corpus.

| | |
|--|--|
| **Product** | Learn · Drill · Mock exam (40 Q / 60 min / study bar 27) |
| **Grade law** | Pure Rust → WASM dual-path (byte-exact goldens) [[claim:claim-grade-byte-exact]] |
| **Not** | EPI®/EXIN CDCP® certification · exam dumps · ML grader [[claim:claim-not-epi-certified]] |

## Start here

1. Constitution: [`docs/ORACLE-GAUNTLET.md`](./docs/ORACLE-GAUNTLET.md)
2. Standards knowledge: [`docs/STANDARDS-KB.md`](./docs/STANDARDS-KB.md)
3. Testing philosophy: [`docs/TESTING.md`](./docs/TESTING.md)
4. Visual tokens: [`docs/VISUAL.md`](./docs/VISUAL.md)
5. Open questions: [`docs/OQ_REGISTER.md`](./docs/OQ_REGISTER.md)
6. Parent study corpus: `../modules/`, `../practice/`

## Gate

```bash
./scripts/check.sh
```

Fail-closed until waves land. See `scorecards/` for wave stamps.

L1 claims constitution: `registries/claims.toml` + `cargo run -p cdcp_registry_check`
(empty registry / unmapped coverage·ready prose = ERROR).

## Layout

```text
knowledge/     # curriculum + standards citation graph (git truth)
registries/    # L1 claims constitution + claims-lint config
bank/          # MCQ items (content-addressed)
crates/        # Rust: core, bank, grade, registry_check, cli
web/           # static HTML/CSS/JS + WASM glue
goldens/       # GradeReport digests
scripts/       # check.sh
docs/          # constitution + research extracts
scorecards/    # wave / layer outcome stamps
```

## Models in this product

**None neural.** Scoring is deterministic code. See plan Part M / STANDARDS-KB.

## Honesty / study signals

- Mock **≥27/40** is a **study signal only** — not a credential. [[claim:claim-study-signal-27]]
- **interview-ready** is a buyer study outcome, never `epi_certified`. [[claim:claim-interview-ready]]
- **domain coverage** tracks the public 14-domain syllabus map, not exam pass probability. [[claim:claim-domain-covered]]

## Status

| Wave | Gate |
|------|------|
| W0 knowledge | green |
| **L1 claims constitution** | **green** — `cdcp_registry_check` in `check.sh` |
| L2 bank pool (~798 / ~20×) | green (`verify_bank` + grounding) |
| **L3 GradeExact** | **green** — `cargo test` + `cdcp goldens check` in `check.sh` |
| L4 WASM dual-path | open |
| L5 browser UI | open |

CLI (from `course-engine/`):

```bash
cargo run -p cdcp_cli -- doctor                 # preflight: bank, wasm, goldens, port, python3
cargo run -p cdcp_cli -- health --robot         # versioned NDJSON envelope
cargo run -p cdcp_cli -- repair                 # units + glossary + export-web; never goldens
cargo run -p cdcp_cli -- bank-hash
cargo run -p cdcp_cli -- grade --fixture goldens/fixtures/mock40_seed42.json --mode all-correct
cargo run -p cdcp_cli -- goldens check
cargo run -p cdcp_registry_check
# UPDATE_GOLDENS=1 cargo run -p cdcp_cli -- goldens generate   # local + human review only
```

`repair` is not a golden laundromat. Re-freezing `goldens/` still requires `UPDATE_GOLDENS=1` and the four-command block in `goldens/PROVENANCE.md`.
