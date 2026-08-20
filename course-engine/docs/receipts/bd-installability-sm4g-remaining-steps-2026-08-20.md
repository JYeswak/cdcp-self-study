# Post-substrate coverage probe — 2026-08-20

This is a diagnostic inventory, not a replacement for `scripts/check.sh`.
The normal chain is still fail-fast and stopped at `substrate_guard_e2e`, actual
step 40, with 40 executed and 39 receipts. The commands below were run
individually against the current worktree after the behavior-equivalent WASM
refresh commit `6ec3dc5d`.

## Denominator

`verify-step-count` reports **98** current `ok` call sites in
`scripts/check.sh`; the public README advertises **90**. No legitimate
depth-0 chain receipt exists because the chain stopped at step 40, so
`verify-step-count` itself was not reached. A synthetic receipt was rejected as
expected: it measured 88 while the source has 98 call sites and README says 90.

There are therefore 58 current post-substrate call sites (98 − 40), versus 50
nominal post-substrate slots implied by README (90 − 40). The standalone probe
attempted all 58: 54 passed, 3 were known-red, and 1 could not be evaluated as
a valid standalone verdict because it requires the receipt emitted by a
previous red suite. In the real fail-fast chain, all 58 remain unreachable
until step 40 is unblocked.

The first SLO invocation was blocked by the probe shell lacking Cargo on
`PATH`; it was rerun with the same `/Users/josh/.cargo/bin` path used by the
toolchain and passed. That invocation problem is not a repository failure.

## Step table

Times are wall-clock measurements from the individual probes; the prior rows
41–79 were measured in the same worktree before the WASM refresh, except step
45, which was rerun after the refresh. The source and all non-WASM inputs were
unchanged between those probes.

| Step | Surface | Result | Wall |
|---:|---|---|---:|
| 41 | L3 golden artifacts present | 0 | 0.027s |
| 42 | GradeExact goldens | 0 | 0.270s |
| 43 | known-bad selftests | 0 | 0.571s |
| 44 | installer known-bad | 0 | 2.175s |
| 45 | WASM dual-path, freshness, schedule | 0 | 10.3s |
| 46 | knowledge primary-notes paths | 0 | 0.034s |
| 47 | L5 product files | 0 | 0.030s |
| 48 | shipped WASM artifact present | 0 | 0.025s |
| 49 | learner pack shape | 0 | 0.031s |
| 50 | L5 known-bad selftest | 0 | 0.469s |
| 51 | L5 e2e digest | 0 | 0.132s |
| 52 | Learn surface | 0 | 0.107s |
| 53 | Reference surface | 0 | 0.076s |
| 54 | L5 learn smoke | 0 | 0.028s |
| 55 | short-interval review smoke | 0 | 0.068s |
| 56 | mastery smoke | 0 | 0.063s |
| 57 | weak-links smoke | 0 | 0.028s |
| 58 | hub mastery + recommend smoke | 1 | 0.102s |
| 59 | hub mastery surface wired | 0 | 0.022s |
| 60 | multi-seed export-web | 0 | 0.594s |
| 61 | session shapes | 0 | 0.034s |
| 62 | coverage | 0 | 0.174s |
| 63 | coverage selftest | 0 | 0.194s |
| 64 | L7 surfaces | 0 | 0.027s |
| 65 | M8-A learn chrome | 0 | 0.026s |
| 66 | units index | 0 | 0.245s |
| 67 | glossary | 0 | 0.049s |
| 68 | learn slugs | 0 | 0.051s |
| 69 | approved-only quiz | 0 | 0.069s |
| 70 | Learn v2 smoke | 0 | 0.055s |
| 71 | diagrams smoke | 0 | 0.032s |
| 72 | accessibility baseline | 0 | 0.035s |
| 73 | feedback links smoke | 0 | 0.064s |
| 74 | feedback links wired | 0 | 0.022s |
| 75 | CLI product verbs | 0 | 0.104s |
| 76 | `cdcp test` | 0 | 0.044s |
| 77 | `cdcp demo --no-open` | 0 | 0.125s |
| 78 | `cdcp study` HTTP 200 | 0 | 0.357s |
| 79 | learner verbs known-bad | 0 | 1.012s |
| 80 | objective coverage | 0 | 0.020s |
| 81 | objectives selftest | 0 | 0.554s |
| 82 | SLO budgets | 0 | 1.564s |
| 83 | content.lock | 0 | 0.174s |
| 84 | content.lock selftest | 0 | 0.146s |
| 85 | reconstructed stages | 1 | 27.735s |
| 86 | voice-slop | 0 | 0.012s |
| 87 | roadmap doc consistency | 1 | 9.656s |
| 88 | roadmap selftest | 0 | 0.180s |
| 89 | publishability bar | 0 | 0.114s |
| 90 | Anki all-retired plant | 0 (inner plant RED) | 0.014s |
| 91 | Anki export | 0 | 0.170s |
| 92 | Anki deck check | 0 | 1.528s |
| 93 | diagram honesty | 0 | 0.023s |
| 94 | `cdcp serve --help` | 0 | 0.003s |
| 95 | bank items runbook | 0 | 0.007s |
| 96 | injection-count selftest | 0 | 0.192s |
| 97 | injection-count verifier | BLOCKED; simulated incomplete log rc=1 | 0.2s |
| 98 | docs sync | 0 | <0.01s |

Step 90's outer result is green because it is a known-bad harness: the planted
all-retired export itself exits nonzero and prints `FAIL: zero approved items
to export`, then the harness validates that result and returns zero.

Step 97 cannot be a valid standalone green: `verify-injection-count` needs
the `INJECTIONS=` receipt from every registered suite. Step 85 stopped before
emitting `selftest_reconstructed`, so the simulated incomplete log correctly
reported that suite as MISSING and measured 67 versus the README's 72.

## Red triage

### (i) Caused by today's bank/registry/gate/artifact work

None. The earlier WASM freshness red was stale-artifact state and is now green
on the committed blob and fresh release build.

### (ii) Pre-existing and known

- Step 58: `bd-hop9`; the hub smoke still reports that `web/README.md` lacks
  the required first-contact `cdcp_cli serve` documentation.
- Step 85: `bd-791t`; the private-copy reconstructed-stage harness reaches its
  learner-pack and export checks, then fails its stale `export-web-hidden`
  mutation leg. The live tracked tree remained untouched. This is the same
  known private-copy/selftest issue recorded in the earlier inventory.
- Step 87: `bd-readme-public-rigor-8y0r.2`; `verify-doc-consistency` finds
  attribution-ledger rows where citation-receipt `BLOCKED` cells were
  misclassified by the detector.
- Step 97: downstream of the missing step-85 suite receipt, under the known
  injection-count drift mechanism `bd-1sd.4`; it is blocked, not a new clean
  result.

### (iii) New and unexplained

None found by this probe.

## Artifact evidence

The refreshed committed WASM is 524,997 bytes,
`8d2e12c522d240d6450cffb249f76b4a95f072d174ae946286387b1adae68b06`.
Fresh and committed paths both produce all-correct
`89ec985451a8d2f9c82e5513fdc4f29fa8568808c4c4bd2eaf390e4e83699a6a` and
all-wrong `593516029bbbb3accd51aa9310cd82bab6d2990b3ada65f52959adf7dc45858d`.
`content.lock` has no WASM-derived field and was unchanged; docs sync, count
pins, goldens couplings, `cdcp doctor`, and `cdcp test` all pass.
