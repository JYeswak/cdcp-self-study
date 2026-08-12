# cdcp-course engine

Local-first **HTML course + browser grading** for the CDCP self-study corpus.

| | |
|--|--|
| **Product** | Learn · Drill · Mock exam (40 Q / 60 min / study bar 27) |
| **Grade law** | Pure Rust → WASM dual-path (byte-exact goldens) |
| **Not** | EPI®/EXIN CDCP® certification · exam dumps · ML grader |

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

## Layout

```text
knowledge/     # curriculum + standards citation graph (git truth)
bank/          # MCQ items (content-addressed)
crates/        # Rust: core, knowledge, bank, grade, wasm, cli
web/           # static HTML/CSS/JS + WASM glue
goldens/       # GradeReport digests
scripts/       # check.sh
docs/          # constitution + research extracts
```

## Models in this product

**None neural.** Scoring is deterministic code. See plan Part M / STANDARDS-KB.

## Status

| Wave | Gate |
|------|------|
| W0 knowledge | green |
| L2 bank pool (~798 / ~20×) | green (`verify_bank` + grounding) |
| **L3 GradeExact** | **green** — `cargo test` + `cdcp goldens check` in `check.sh` |
| L4 WASM dual-path | open |
| L5 browser UI | open |

CLI (from `course-engine/`):

```bash
cargo run -p cdcp_cli -- bank-hash
cargo run -p cdcp_cli -- grade --fixture goldens/fixtures/mock40_seed42.json --mode all-correct
cargo run -p cdcp_cli -- goldens check
# UPDATE_GOLDENS=1 cargo run -p cdcp_cli -- goldens generate   # local + human review only
```
