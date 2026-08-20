# bd-installability-sm4g.20 — registry-root proof and diagnostic inventory

Date: 2026-08-20

## Stop-the-line boundary

This work did not edit or stage any of these learner artifacts:

- `web/data/bank_items_seed42.json`
- `web/data/keys_seed42.json`
- `web/data/mock40_seed42.json`
- `web/data/units_index.json`

Their worktree dirt, and the `goldens-couplings` RED caused by the in-flight
bank rewrite, remain intentional pending `bd-q6gl`. No pack was silently
re-frozen.

## Task 1 — registry root resolution

The failure is a root-selection bug, not evidence that the engine registries
are absent. The git root is `/Users/josh/cdcp-self-study`; the engine root is
its direct child `course-engine/`, which owns `registries/claims.toml` and
`registries/gate_shrink.toml`. The old upward-only resolver correctly handled
an engine path and an engine subdirectory, but an invocation from the git root
had no anchor at that level and failed before checking the child engine.

`resolve_repo_root` now keeps the upward walk, then examines only direct
children of the supplied workspace directory. Exactly one child containing
`registries/claims.toml` is accepted; zero remains an error and multiple
children are an ambiguity error. This does not guess through arbitrary
descendants or make a missing registry pass.

Proofs after the fix:

| start | result |
|---|---|
| `/Users/josh/cdcp-self-study/course-engine` | `cdcp_registry_check` exit 0 |
| `/Users/josh/cdcp-self-study` | exit 0; nested `course-engine/` selected |
| `/Users/josh/cdcp-self-study/course-engine/crates/cdcp_registry_check` | exit 0; upward walk selected engine |
| temporary root with no anchored engine, passed explicitly | exit 2; `missing_registry — registries/claims.toml missing (empty/deleted registry = ERROR)` plus missing `gate_shrink.toml` |
| temporary workspace with two anchored engine children | resolver test exits with `ambiguous course-engine roots` |

The two resolver tests and the existing registry suite pass: `cargo test -q
-p cdcp_registry_check` reports 92 passed, 0 failed. `cargo fmt --all --
--check` is also green. The earlier 16-step run from the engine directory was
the correct invocation; the git-root invocation was the wrong-context run,
not a certificate that the engine registries had ever been absent.

## Task 2 — diagnostic inventory

Source: `/tmp/cdcp-check-diagnostic.scoped-2.log` and its timestamped table
`/tmp/cdcp-check-diagnostic-table.tsv`. Command was the opt-in diagnostic
mode of `scripts/check.sh`; it was not the default CI path.

The table below starts at diagnostic ordinal 24, as requested. A diagnostic
ordinal is an attempted step invocation, including a recorded RED; it is not
the successful-step receipt count maintained by `verify-step-count`.

| # | exit | wall s | step | triage |
|---:|---:|---:|---|---|
| 24 | 0 | 0.019 | honesty string smoke | — |
| 25 | 0 | 0.039 | standards crosswalk covers every declared domain (n=15) | — |
| 26 | 0 | 0.003 | topics.toml count=118 | — |
| 27 | 0 | 0.002 | sources fetch_date present | — |
| 28 | 0 | 0.158 | bank pool | — |
| 29 | 0 | 0.216 | answer-key distribution | — |
| 30 | 0 | 0.411 | bank-internal contradiction floor | — |
| 31 | 0 | 0.390 | construction-fault scan | — |
| 32 | 0 | 0.526 | grounding heuristics | — |
| 33 | 0 | 3.534 | grounding-wave stem regression detector | — |
| 34 | 0 | 0.148 | no orphan topics / item refs / unanchored items | — |
| 35 | 0 | 1.411 | orphan selftest | — |
| 36 | 0 | 4.636 | no cosmetic near-duplicates in approved pool | — |
| 37 | 0 | 0.151 | near-duplicate selftest | — |
| 38 | 0 | 15.516 | paraphrase pair ledger intact | — |
| 39 | 0 | 1.346 | rustfmt #[path] gate selftests | — |
| 40 | 2 | 41.708 | cargo fmt + clippy -D warnings + test | **(i)** stale q210 letter assertion; fixed in `fc1905f` |
| 41 | 0 | 0.000 | L3 golden artifacts present | — |
| 42 | 2 | 0.839 | GradeExact goldens | **(i)** stale mid-restoration pins; hold for `bd-q6gl` |
| 43 | 0 | 0.556 | known-bad selftests | — |
| 44 | 0 | 2.019 | installer known-bad selftest | — |
| 45 | 2 | 12.423 | knowledge primary_notes paths | **(ii)** diagnostic carry-forward from WASM RED; `bd-installability-sm4g.24` |
| 46 | 0 | 0.000 | L5 product files present | — |
| 47 | 0 | 0.000 | L5 wasm artifact present | — |
| 48 | 0 | 0.005 | L5 learner pack n_items=40 | — |
| 49 | 2 | 0.227 | L5 selftest | **(i)** stale golden pins; hold for `bd-q6gl` |
| 50 | 2 | 0.099 | L5 e2e digest match | **(i)** stale golden pins; hold for `bd-q6gl` |
| 51 | 0 | 0.261 | Learn surface | — |
| 52 | 0 | 0.029 | Reference surface | — |
| 53 | 0 | 0.007 | L5 learn smoke | — |
| 54 | 0 | 0.050 | L6 short-interval review smoke | — |
| 55 | 0 | 0.044 | L6 mastery smoke | — |
| 56 | 0 | 0.006 | L6 weak-links smoke | — |
| 57 | 2 | 0.083 | L6 hub mastery + recommend smoke | **(ii)** README first-contact contract; `bd-hop9` |
| 58 | 0 | 0.000 | L6-S4 hub mastery surface wired | — |
| 59 | 2 | 0.573 | L6 multi-seed export-web --seed 42 | **(i)** dirty learner packs; hold for `bd-q6gl` |
| 60 | 0 | 0.005 | L6 session shapes | — |
| 61 | 0 | 0.145 | L6 coverage GREEN | — |
| 62 | 0 | 0.179 | L6 coverage selftest | — |
| 63 | 0 | 0.000 | L7 surfaces | — |
| 64 | 0 | 0.004 | M8-A learn chrome smoke | — |
| 65 | 2 | 0.225 | M8-B units_index | **(i)** dirty `web/data/units_index.json`; hold for `bd-q6gl` |
| 66 | 0 | 0.026 | M8-D glossary.json | — |
| 67 | 0 | 0.027 | MODULE_LEARN_SLUGS | — |
| 68 | 0 | 0.049 | no learner surface draws non-approved item | — |
| 69 | 0 | 0.033 | M8-B/D learn v2 smoke | — |
| 70 | 0 | 0.011 | M8-C diagrams smoke | — |
| 71 | 0 | 0.014 | L7 a11y baseline | — |
| 72 | 0 | 0.044 | L7-S2 feedback section-anchor links smoke | — |
| 73 | 0 | 0.000 | L7 feedback section links | — |
| 74 | 2 | 0.087 | L7 CLI product verbs listed | **(ii)** torn concurrent scratch/build snapshot; related concurrency bead `bd-gl4j`; current binary shows all five verbs |
| 75 | 0 | 0.024 | cdcp test | — |
| 76 | 0 | 0.103 | cdcp demo --no-open | — |
| 77 | 0 | 0.259 | cdcp study served HTTP 200 | — |
| 78 | 0 | 1.080 | learner verbs known-bad | — |
| 79 | 0 | 0.159 | L7 objective coverage | — |
| 80 | 0 | 0.708 | L7 objectives known-bad selftest | — |
| 81 | 0 | 5.075 | L7 SLO budgets | — |
| 82 | 2 | 0.303 | L7 content.lock | **(i)** bank hash follows current dirty bank; hold for `bd-q6gl` |
| 83 | 0 | 0.312 | L7 content.lock selftest | — |
| 84 | 2 | 130.641 | L5–V11 reconstructed stages | **(ii)** known private-copy/selftest issue; `bd-791t` |
| 85 | 0 | 0.175 | public copy free of marketing slop | — |
| 86 | 2 | 9.473 | roadmap milestone status and publication truth | **(ii)** citation-status false positives; `bd-readme-public-rigor-8y0r.2` |
| 87 | 0 | 0.323 | roadmap selftest | — |
| 88 | 0 | 0.377 | L88 publishability bar | — |
| 89 | 0 | 0.014 | V11 Anki planted all-retired is RED | — |
| 90 | 0 | 0.293 | V11 Anki export | — |
| 91 | 0 | 1.600 | V11 Anki .apkg deck | — |
| 92 | 0 | 0.023 | V11 diagram honesty | — |
| 93 | 0 | 0.003 | V11 serve subcommand | — |
| 94 | 0 | 0.007 | V11 runbook bank items | — |
| 95 | 0 | 0.389 | drift-guard selftest | — |
| 96 | 2 | 0.146 | advertised known-bad injection count | **(ii)** upstream red suites emitted no receipts; known drift mechanism `bd-1sd.4` |
| 97 | 0 | 0.030 | advertised content counts vs units_index + WASM KiB | — |

The diagnostic run ended with the administrative verifier, not another
advertised chain step:

| verifier | exit | wall s | result |
|---|---:|---:|---|
| `verify-step-count` | 2 | 0.070 | receipt measured `82` successful steps, README advertises `90` |

### Counts and timing

- Diagnostic transcript attempts: **97**.
- Diagnostic results: **82 exit 0**, **15 exit 2**, **0 skipped**.
- `verify-step-count` measured: **82 successful receipts** (`OK=82`,
  `SKIPPED=0`).
- README advertises: **90**.
- Total wall time: **273.457 seconds**.

The 97-attempt transcript and the 82 successful-step receipt are deliberately
not collapsed: failed diagnostic attempts are inventory rows, while the
current step-count receipt counts only successful `ok` records. The resulting
82-versus-90 RED is the known step-count drift owned by `bd-1sd.13`; it is not
evidence that the 15 recorded failures were silently skipped.

### Three-way triage

**(i) Caused by today’s bank/registry/gate work**

- Step 40: q210’s positional-letter assertion; fixed in `fc1905f` by asserting
  the current key text instead of the permutable letter.
- Steps 42, 49, 50, 59, 65, and 82: stale goldens/export/index/content-lock
  artifacts against the in-flight bank rewrite. They remain RED by the
  stop-the-line rule and are held for `bd-q6gl`; no `web/data` file was edited
  or staged here.
- Step 21, already triaged before this receipt, is the same deliberate
  goldens-coupling RED and remains held for `bd-q6gl`.

**(ii) Pre-existing and previously known**

- Step 45 is an upstream diagnostic attribution from the known WASM release
  portability failure in `bd-installability-sm4g.24`; the primary-notes
  command itself printed PASS.
- Step 57 is the existing local-server first-contact documentation contract,
  `bd-hop9`.
- Step 74 was a torn concurrent scratch/build snapshot, not a missing CLI
  verb; the current binary exposes all five. The concurrency class is recorded
  by `bd-gl4j`.
- Step 84 is the known private-copy reconstructed-stage failure, `bd-791t`.
- Step 86 is the known publication-status false-positive class,
  `bd-readme-public-rigor-8y0r.2`.
- Step 96 is a downstream receipt-count consequence of the already-red suites,
  under the known injection-count drift guard `bd-1sd.4`.
- The final 82-versus-90 verifier RED is the known advertised step-count drift,
  `bd-1sd.13`.

**(iii) New and unexplained**

None surfaced in rows 24–97. No unclassified RED was touched or weakened.

## Verification boundary

The root-resolution code and tests are the only files changed for this
follow-up, plus this receipt. The four dirty `web/data` packs remain outside
the commit scope. The inventory is diagnostic evidence, not a claim that the
default fail-fast chain is green or that the product pack is ready to freeze.
