# cdcp-course engine

Local-first **HTML course + browser grading** for the CDCP self-study corpus.

| | |
|--|--|
| **Product** | Learn · Drill · Mock exam (40 Q / 60 min / study bar 27) |
| **Grade law** | Pure Rust → WASM dual-path (byte-exact goldens) [[claim:claim-grade-byte-exact]] |
| **Not** | EPI®/EXIN CDCP® certification · exam dumps · ML grader [[claim:claim-not-epi-certified]] |

## Start here

```bash
curl -fsSL https://raw.githubusercontent.com/JYeswak/cdcp-self-study/main/course-engine/install.sh | bash
```

Then `cdcp study`. `--prefix DIR` · `--from-source` · `--verify` · `--uninstall` (keeps `var/attempts`).

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

`check.sh` runs `cdcp_gate substrate-guard` (presence scan). That is the
substrate floor: an unlisted `.py`/`.sh` fails the build even if no git hook
ran (`git commit --no-verify`, merge, cherry-pick, rebase, `git am`,
`commit-tree`, or a fresh clone). The committed shim `hooks/pre-commit` is a
courtesy on ordinary `git commit` only. Install it in a clone:

```bash
cargo build -p cdcp_gate --locked
./target/debug/cdcp_gate install-hooks
./target/debug/cdcp_gate install-hooks --check
```

L1 claims constitution: `registries/claims.toml` + `./target/debug/cdcp_registry_check`
after `cargo build -p cdcp_registry_check --locked` (empty registry / unmapped
coverage·ready prose = ERROR).

## Layout

```text
knowledge/     # curriculum + standards citation graph (git truth)
registries/    # L1 claims constitution + claims-lint config
bank/          # MCQ items (content-addressed)
crates/        # Rust: core, bank, grade, schedule, registry_check, cli
web/           # static HTML/CSS/JS + WASM glue
goldens/       # GradeReport digests
scripts/       # check.sh (the substrate floor; hooks are a courtesy)
hooks/         # committed pre-commit shim — install with `cdcp_gate install-hooks`
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
| L2 bank pool (904 files / 879 approved / 25 retired ≈ 20.9× exam on the approved pool — pool size, not distinct propositions) [[fact:fact-bank-item-count-804=yes]] [[fact:fact-bank-approved-count-779=yes]] [[fact:fact-approved-pool-multiplier-19-5=yes]] | green (`cdcp_gate verify-bank` + grounding) |
| **L3 GradeExact** | **green** — `cargo test` + `cdcp goldens check` in `check.sh` |
| **L4 WASM dual-path** | **green** — `cdcp_wasm` built to `wasm32-unknown-unknown`; dual-path digests asserted equal, `selftest_wasm_freshness` guards the committed blob |
| **L5 browser UI** | **green** — Hub · Learn · Drill · Mock · Reference served by `cdcp serve`; e2e digests + learner-pack answer-key-leak known-bad |

CLI (from `course-engine/`):

```bash
cargo build -p cdcp_cli -p cdcp_registry_check --locked
./target/debug/cdcp serve                  # local HTTP hub (http://127.0.0.1:8766/; not file://)
./target/debug/cdcp doctor                 # preflight: bank, wasm, goldens, port
./target/debug/cdcp health --robot         # versioned NDJSON envelope
./target/debug/cdcp repair                 # units + glossary + export-web; never goldens
./target/debug/cdcp bank-hash
./target/debug/cdcp grade --fixture goldens/fixtures/mock40_seed42.json --mode all-correct
./target/debug/cdcp goldens check
./target/debug/cdcp_registry_check
# UPDATE_GOLDENS=1 ./target/debug/cdcp goldens generate   # local + human review only
```

`doctor` still probes `python3`, but only because surviving dual-path oracles
may need it. It is not a product surface.

`repair` is not a golden laundromat. Re-freezing `goldens/` still requires `UPDATE_GOLDENS=1` and the four-command block in `goldens/PROVENANCE.md`.
