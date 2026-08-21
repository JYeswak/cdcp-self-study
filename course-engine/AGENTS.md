# AGENTS — cdcp-course engine

## Product

Local-first CDCP **study** tool. Not EPI certification software.

Canonical physical workspace root: `/Users/josh/cdcp-self-study/course-engine`

Before any reservation, build, receipt, or commit command, run the physical
workspace preflight from this checkout using the `workspace-preflight` row in
[`docs/VERIFICATION-MATRIX.toml`](docs/VERIFICATION-MATRIX.toml).
It compares `pwd -P` semantics with this declared root. Symlink entrypoints
normalize to this identity; a different checkout is refused. Evidence must
use the resolved physical root as its path and Agent Mail project key.

## Read first

`docs/ORACLE-GAUNTLET.md` · `docs/STANDARDS-KB.md` · `docs/OQ_REGISTER.md` · `docs/NEGATIVE_EVIDENCE.md`

## Hard rules

1. No exam dumps in `bank/`.  
2. No pirated SDO full-text PDFs.  
3. Grade path: pure Rust; browser must dual-path match.  
4. No LLM as grade-of-record.  
5. No unresolved OQ that a wave depends on.  
6. G1 > G2: correctness before UI polish.  
7. `scripts/check.sh` is the only full gate story. The pre-commit hook is a
   courtesy; use the `hook-install` and `hook-check` rows in
   [`docs/VERIFICATION-MATRIX.toml`](docs/VERIFICATION-MATRIX.toml), never a
   second command contract. An unlisted `.py` fails check.sh even if the hook
   never ran.

## Honest Work and Anti-Ceremony (binding for agents and humans alike)

The purpose of agent work here is working, deployable capability. Process
serves that outcome and never becomes the product.

- A process artifact (certificate, ledger, dashboard, matrix, meta-report,
  speculative check) may be created only if it names a concrete consumer,
  the named feature it gates, the observed defect class justifying it, and
  its deletion condition. Otherwise it does not get created. Boundary test:
  if running code branches on it, it is product; if only humans and status
  reports read it, it is process and the creation-gate rule above applies;
  code written just to flip this answer counts as the pathology, not as a
  consumer. Sole exception: a minimal
  integrity/recovery control (crash-recovery state, provenance snapshot) is
  legitimate when it prevents a named evidence-loss or corruption mode and
  is necessary and minimal.
- Real code + real tests in the same unit of work. Forbidden: faked tests,
  fixtures/mocks presented as live proof, weakened assertions, golden
  regeneration to force green, hard-coded success paths, placeholder macros
  in commits, editing the spec instead of implementing it, narrowing scope
  while claiming full success.
- No self-certification: work is closed by an independent verifier citing
  evidence at an exact revision. Solo sessions re-verify by re-execution and
  state what was not independently verified.
- A typed refusal beats a fabricated result and is less valuable than the
  real capability; refusal-only work stays open and says so.
- Truthful null results ("checked X, found no material increment") are
  successful outcomes. Unsupported claims are worse than silence.
- Metrics predeclare denominator and countermetric; agreement between
  agents may raise confidence but is never independent evidence; never
  silence stderr in evidence-bearing commands.
- Name these pathologies when they occur (gate self-weakening, proof-class
  inflation, golden regeneration, tolerance widening, suppression-pragma
  laundering, refusal farming, follow-up laundering); the names are the
  deterrent. The full catalog with countermeasures lives in the
  just-say-no-to-process-porn-and-ceremony skill; ask the operator for it
  if you cannot resolve that reference.

### Measured here, 2026-08-20

The rule above is not imported theory. Across 91 commits in one session:
**65 were PROCESS, 43 of those pure claim/release churn.**
`.flywheel/QUEUE-2026-08-20.md` absorbed **453** changed lines; `bank/items`
— the product — absorbed **8**. The claim-by-commit protocol that produced
the churn was invented by the controller, gated nothing, and named no
deletion condition. Queue claims are no longer committed: state the item you
took in your pane and start work. Commit code, tests, bank content, and
receipts that record a measurement someone will act on. Nothing else.

## Hermetic test runner

For a test lane, use the `hermetic-test` row in
[`docs/VERIFICATION-MATRIX.toml`](docs/VERIFICATION-MATRIX.toml), rather than
copying or invoking a direct `cargo test` command.

The wrapper owns `target/cdcp-hermetic/<lane-name>`, rejects caller target,
manifest, and Cargo-environment overrides, and fingerprints `HEAD` plus the
product input set before and after the child.  Exit 3 with a `DRIFT:` message
means the source moved during the run; it is not a passing test result and not
an ordinary assertion failure.  `scripts/check.sh` routes its production Cargo
test lanes through this wrapper and names each lane.

The wrapper's own tests and isolated known-bad fixture scripts may launch a
fake or explicitly pinned Cargo process because that is the specimen under
test.  Those are proof harnesses, not a sanctioned production test lane, and
must report their fixture/target scope explicitly.

## Parent corpus

Study notes live in `../modules/`, practice in `../practice/`. Import into bank; don’t duplicate without hash.

## Commits

Conventional: `feat(cdcp-course): …` / `docs(cdcp-course): …`
