#!/usr/bin/env sh
# check.sh — fail-closed gate for cdcp-course engine
# Waves incomplete: exit 2 with clear message until L3+ tools exist.
#
# L4 selftests: scripts/selftest_known_bad.sh injects known-bad fixtures,
# asserts RED, restores. Never leave goldens/bank dirty.
set -eu

ROOT="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() { echo "check.sh: FAIL: $*" >&2; exit 2; }
GAPS=""
ok() { echo "check.sh: ok: $*"; }

# ── Concurrency lock [bd-gl4j] ─────────────────────────────────────────────
# Measured 2026-08-14, four concurrent runs live during a six-agent wave: one
# run exited 2 on `L5 learner pack shape (n_items=39)` because another run was
# inside selftest_reconstructed.sh, which mutates web/data/mock40_seed42.json IN
# THE WORKING TREE and restores it afterwards. The second run read the mutated
# state and reported a product defect that did not exist.
#
# The cost is not the failed run, it is the FALSE VERDICT. A red from a
# concurrency artifact is indistinguishable from a red from a real regression,
# and the natural response — re-run it — makes it disappear, which teaches the
# reader to dismiss reds. That is the fooled-certificate failure arriving through
# the harness instead of through a gate.
#
# STEPS THAT MUTATE TRACKED FILES while this script runs. Every one is now
# serialised behind this lock. [M] = observed 2026-08-14 by polling
# `git status --porcelain` while the step ran; [S] = read from the writing
# script's source, not yet observed under a poller.
#   selftest_known_bad.sh      goldens/mock40_seed42_all_correct.sha256 [M] restored
#                              goldens/bank_hash.txt                    [M] restored
#                              docs/_selftest_known_bad_planted.md      [S] planted/removed
#   selftest_l5.sh             web/_selftest_l5_honesty_planted.html    [S] planted/removed
#   selftest_reconstructed.sh  web/data/mock40_seed42.json              [S] restored
#                              web/data/keys_seed42.json                [S] restored
#                              web/drill.html                           [S] restored
#                              crates/cdcp_cli/src/main.rs              [S] restored
#   cdcp_gate build-units          web/data/units_index.json            [M] regenerated
#   cdcp_gate build-glossary-json  web/data/glossary.json               [M] regenerated
#   smoke_feedback_links.py    web/data/topic_anchors.json              [M] regenerated
#   export_anki.py             dist/anki/**                             [M] untracked output
# The three regenerated files are byte-identical today, so `git status` stays
# clean — but the WRITE still happens (mtime moves), and a concurrent reader can
# still catch a truncated file. Rewritten-identical is not the same as untouched.
# Everything else writes only under $TMPDIR or target/ (gitignored): the
# substrate-guard behavioural probe, the content.lock mutate-selftest,
# smoke_slo.sh, the L6 multi-seed export-web, and every selftest's TMP_ROOT.
#
# The lock is a DIRECTORY: mkdir is the only atomic create-or-fail primitive
# POSIX sh has. A stale lock (recorded holder pid gone) is reclaimed and SAYS SO
# — a lock that can wedge the build is worse than the race it prevents. An
# unwritable lock path is an ERROR, never a green run without a lock; a lock that
# quietly does nothing is the same fooled certificate as a gate that quietly
# scans nothing.
#
# Re-entrancy: selftest_reconstructed.sh runs this script again in the SAME root,
# five times. CDCP_CHECK_LOCK_HELD carries the held lock path to those
# descendants so they run under the ancestor's lock instead of deadlocking on it.
# `substrate-guard --prove-wired` runs check.sh from a tree materialised under
# target/ — a different ROOT, hence a different lock, taken independently.
LOCK_DIR="$ROOT/target/check.lock"
LOCK_HELD=0
# CDCP_CHECK_LOCK_DIR relocates the lock for the L4 selftest below and NOWHERE
# ELSE: it is honoured only in probe mode, which exits before any gate runs, so
# it cannot be used to run two real gate chains side by side.
if [ "${CDCP_CHECK_LOCK_PROBE:-0}" = "1" ]; then
  CDCP_CHECK_LOCK_HELD=""
  if [ -n "${CDCP_CHECK_LOCK_DIR:-}" ]; then
    LOCK_DIR="$CDCP_CHECK_LOCK_DIR"
  fi
fi

# ── L4 drift guard plumbing ────────────────────────────────────────────────
# Every selftest suite prints one `INJECTIONS=<n> SUITE=<name>` receipt on its
# success path. run_selftest runs the suite for real, forwards its output, and
# tees the receipts into INJ_LOG. scripts/verify_injection_count.py then sums
# them and compares against the count README.md advertises — so the badge can
# never drift from the machinery it describes.
INJ_LOG=""

# Single cleanup for both resources. Installed BEFORE the lock is taken, so a
# signal arriving between mkdir and the first gate still releases it. It cannot
# survive SIGKILL — hence stale reclamation below. Explicit `if` blocks and a
# terminal `return 0`: an `&&` chain whose test is false would return non-zero
# from an EXIT trap under `set -e`.
cleanup() {
  if [ -n "$INJ_LOG" ]; then rm -f "$INJ_LOG"; fi
  if [ "$LOCK_HELD" = "1" ]; then rm -rf "$LOCK_DIR"; fi
  return 0
}
trap 'cleanup' EXIT INT TERM HUP

lock_error() { echo "check.sh: LOCK ERROR: $*" >&2; exit 2; }

lock_acquire() {
  _lk_parent="$(dirname "$LOCK_DIR")"
  mkdir -p "$_lk_parent" 2>/dev/null || true
  if [ ! -d "$_lk_parent" ]; then
    lock_error "cannot create $_lk_parent to hold the lock; an unwritable lock path is an ERROR, not a silent skip"
  fi
  _lk_try=0
  while [ "$_lk_try" -lt 4 ]; do
    _lk_try=$((_lk_try + 1))
    if mkdir "$LOCK_DIR" 2>/dev/null; then
      LOCK_HELD=1
      if ! printf '%s\n' "$$" >"$LOCK_DIR/pid" 2>/dev/null; then
        lock_error "took $LOCK_DIR but cannot record the holder pid; an unnamed holder can be neither reported nor reclaimed — ERROR, not a silent skip"
      fi
      if ! date -u +%Y-%m-%dT%H:%M:%SZ >"$LOCK_DIR/started" 2>/dev/null; then
        lock_error "took $LOCK_DIR but cannot record the start time — ERROR, not a silent skip"
      fi
      return 0
    fi
    if [ ! -d "$LOCK_DIR" ]; then
      lock_error "mkdir $LOCK_DIR failed and no lock is present there; the lock path is unusable — ERROR, not a silent skip"
    fi
    _lk_pid=""
    if [ -f "$LOCK_DIR/pid" ]; then
      _lk_pid="$(cat "$LOCK_DIR/pid" 2>/dev/null || true)"
    fi
    _lk_started="unknown"
    if [ -f "$LOCK_DIR/started" ]; then
      _lk_started="$(cat "$LOCK_DIR/started" 2>/dev/null || true)"
    fi
    if [ -z "$_lk_pid" ]; then
      # The holder may be between mkdir and the pid write. Give it a moment; if
      # the pid never appears, the holder died inside that window — stale.
      if [ "$_lk_try" -lt 3 ]; then
        sleep 1
        continue
      fi
      echo "check.sh: reclaimed stale lock $LOCK_DIR (no holder pid recorded after ${_lk_try}s)" >&2
      rm -rf "$LOCK_DIR"
      continue
    fi
    if kill -0 "$_lk_pid" 2>/dev/null; then
      echo "check.sh: REFUSING TO START: scripts/check.sh is already running." >&2
      echo "check.sh:   lock:      $LOCK_DIR" >&2
      echo "check.sh:   held by pid $_lk_pid, started $_lk_started" >&2
      echo "check.sh: Concurrent runs corrupt each other: several steps mutate tracked files" >&2
      echo "check.sh: under web/ · goldens/ · crates/ and restore them afterwards, so a second" >&2
      echo "check.sh: run reads half-injected state and reports a defect that does not exist." >&2
      echo "check.sh: Wait for pid $_lk_pid, or kill it if it is wedged — the lock is then" >&2
      echo "check.sh: reclaimed automatically by the next run." >&2
      exit 2
    fi
    echo "check.sh: reclaimed stale lock $LOCK_DIR (recorded holder pid $_lk_pid, started $_lk_started, is gone)" >&2
    rm -rf "$LOCK_DIR"
  done
  lock_error "could not settle $LOCK_DIR after $_lk_try attempts (contended and repeatedly reclaimed)"
}

if [ "${CDCP_CHECK_LOCK_HELD:-}" = "$LOCK_DIR" ]; then
  echo "check.sh: running under the lock an ancestor run already holds ($LOCK_DIR)"
else
  lock_acquire
  CDCP_CHECK_LOCK_HELD="$LOCK_DIR"
  export CDCP_CHECK_LOCK_HELD
fi

# Probe mode exists only to exercise the acquisition path above from a child
# process. It runs no gate: it reports the outcome and leaves.
if [ "${CDCP_CHECK_LOCK_PROBE:-0}" = "1" ]; then
  echo "check.sh: lock probe: acquired $LOCK_DIR"
  exit 0
fi

# ── L4: the lock is proven to trip ─────────────────────────────────────────
# BUILT != WIRED applies to the lock too. Nothing readable above establishes
# that a second run is actually refused — `mkdir` could be pointed at a path that
# never collides, the pid file could go unwritten, the refusal branch could be
# unreachable. These three legs run the real acquisition path in a child and
# require the observable outcome:
#   (1) while this run holds the lock, a second acquisition is REFUSED and names
#       the holder pid;
#   (2) a lock whose recorded holder is dead is RECLAIMED and says so;
#   (3) an unwritable lock path is an ERROR, not a green run with no lock.
# NOT via run_selftest: it emits no INJECTIONS= receipt and must not move the
# advertised known-bad count.
if [ "$LOCK_HELD" = "1" ]; then
  echo "==> check.sh lock selftest (L4: second run refused · stale lock reclaimed · unwritable path ERRORs)"
  _lk_self="$ROOT/scripts/check.sh"
  _lk_scratch="$ROOT/target/cdcp-lock-selftest"
  rm -rf "$_lk_scratch"
  mkdir -p "$_lk_scratch" || fail "lock selftest: cannot create $_lk_scratch"

  _lk_rc=0
  _lk_out="$(CDCP_CHECK_LOCK_PROBE=1 sh "$_lk_self" 2>&1)" || _lk_rc=$?
  [ "$_lk_rc" -ne 0 ] \
    || fail "lock selftest: a second run started while pid $$ holds $LOCK_DIR"
  printf '%s\n' "$_lk_out" | grep -q "held by pid $$" \
    || fail "lock selftest: the refusal never named the holder pid $$: $_lk_out"

  # (2) A holder pid that is provably gone: reap a child, then claim its pid.
  sh -c 'exit 0' &
  _lk_dead=$!
  wait "$_lk_dead" 2>/dev/null || true
  mkdir -p "$_lk_scratch/stale"
  printf '%s\n' "$_lk_dead" >"$_lk_scratch/stale/pid"
  printf '%s\n' "1970-01-01T00:00:00Z" >"$_lk_scratch/stale/started"
  _lk_rc=0
  _lk_out="$(CDCP_CHECK_LOCK_PROBE=1 CDCP_CHECK_LOCK_DIR="$_lk_scratch/stale" \
    sh "$_lk_self" 2>&1)" || _lk_rc=$?
  [ "$_lk_rc" -eq 0 ] \
    || fail "lock selftest: a stale lock (dead holder pid $_lk_dead) wedged the run (rc=$_lk_rc): $_lk_out"
  printf '%s\n' "$_lk_out" | grep -q "reclaimed stale lock" \
    || fail "lock selftest: the stale lock was taken silently; reclamation must be stated: $_lk_out"

  # (3) Anti-vacuous. The lock's parent is a regular FILE, so mkdir -p fails with
  # ENOTDIR for every user including root — no chmod trick a root CI defeats.
  : >"$_lk_scratch/notadir"
  _lk_rc=0
  _lk_out="$(CDCP_CHECK_LOCK_PROBE=1 CDCP_CHECK_LOCK_DIR="$_lk_scratch/notadir/lock" \
    sh "$_lk_self" 2>&1)" || _lk_rc=$?
  [ "$_lk_rc" -ne 0 ] \
    || fail "lock selftest: an unwritable lock path ran GREEN with no lock held — a lock that silently does nothing is a fooled certificate"
  printf '%s\n' "$_lk_out" | grep -q "LOCK ERROR" \
    || fail "lock selftest: the unwritable lock path failed without naming the lock as the reason: $_lk_out"

  rm -rf "$_lk_scratch"
  ok "concurrency lock proven (second run refused naming pid $$ · dead holder reclaimed · unwritable lock path ERRORs)"
fi

INJ_LOG="$(mktemp "${TMPDIR:-/tmp}/cdcp_injections.XXXXXX")"

run_selftest() {
  _lbl="$1"
  shift
  # Output is captured so the receipt can be teed; it therefore appears only
  # once the suite finishes. selftest_reconstructed.sh runs the full gate five
  # times and can sit here for minutes — say so rather than look hung.
  echo "check.sh: running $_lbl (output shown when it completes)"
  _out=""
  if ! _out="$("$@" 2>&1)"; then
    printf '%s\n' "$_out" >&2
    fail "$_lbl"
  fi
  printf '%s\n' "$_out"
  printf '%s\n' "$_out" | grep '^INJECTIONS=' >>"$INJ_LOG" || true
}

echo "==> cdcp-course check (W0 knowledge scaffold)"

# Required constitution docs
for f in \
  docs/ORACLE-GAUNTLET.md \
  docs/STANDARDS-KB.md \
  docs/TESTING.md \
  docs/VISUAL.md \
  docs/OQ_REGISTER.md \
  docs/NEGATIVE_EVIDENCE.md \
  docs/research/STANDARDS-TENSIONS.md \
  README.md
do
  [ -f "$f" ] || fail "missing $f"
done
ok "constitution docs present"

# Required knowledge pack
for f in \
  knowledge/exam_form.toml \
  knowledge/sources.toml \
  knowledge/domains.toml \
  knowledge/topics.toml \
  knowledge/standards_families.toml \
  knowledge/standards_crosswalk.toml \
  knowledge/fact_policy.toml \
  knowledge/claims.toml
do
  [ -f "$f" ] || fail "missing $f"
done
ok "knowledge pack files present"

# L1 claims constitution (frankengraphdb-style registries + registry-check)
for f in \
  registries/claims.toml \
  registries/claims_lint.toml \
  registries/objectives.toml
do
  [ -f "$f" ] || fail "missing $f (L1 empty/deleted registry = ERROR)"
done
ok "L1 registry files present"

echo "==> cdcp_registry_check (L1 claims constitution)"
cargo run -q -p cdcp_registry_check || fail "registry-check"
ok "L1 registry-check"

# S0 substrate floor. Placed next to the L1 registry gate because it is the same
# kind of thing — a registry constitution over what may exist in the tree — and it
# fails fast (only serde+toml compile).
echo "==> cdcp_gate substrate-guard (S0 substrate floor)"
cargo run -q -p cdcp_gate -- substrate-guard || fail "substrate guard (unreasoned .py/.sh)"
ok "S0 substrate floor (no unreasoned non-Rust file in scripts/ · crates/ · repo root)"

# L4 for the wiring claim ITSELF. Nothing read out of this file can establish that
# the line above executes (bd-bo6i): `: "cargo run ..."` is a no-op, `true # ...`
# is a comment, and `cargo run ... || true` runs the gate and throws the verdict
# away — all three read as an invocation. --prove-wired materialises the INDEX,
# plants an unlisted .py, runs this script for real, and requires it to exit
# non-zero. An inert line cannot satisfy that. Measured cost: ~16s.
# Terminates at depth 1: the copy dies on the plant above this line, and
# CDCP_SUBSTRATE_PROBE=1 in the child makes a nested probe an ERROR.
# NOT via run_selftest — it emits no INJECTIONS= receipt and must not move the
# advertised known-bad count.
echo "==> cdcp_gate substrate-guard --prove-wired (L4: the wiring is proven to trip)"
cargo run -q -p cdcp_gate -- substrate-guard --prove-wired \
  || fail "substrate-guard wiring does not stop check.sh"
ok "S0 wiring proven behaviourally (a planted unlisted .py stops check.sh)"

echo "==> cdcp_gate install-hooks --check (BUILT != WIRED)"
cargo run -q -p cdcp_gate -- install-hooks --check \
  || fail "pre-commit shim not installed (run: cargo run -q -p cdcp_gate -- install-hooks)"
ok "pre-commit shim installed and current"

# B1 (bd-hardening-b-ledgers-gvm.1): the machine ledger of capability claims.
# A row whose last_review has aged past registries/capability-maturity.toml's
# staleness_days, whose evidence names a test function nothing defines, or whose
# published CHARTER cell outruns the level its evidence can carry, is RED.
# It found two on its first run — L2 and L5 both published "YES · wired" over
# capabilities that no named test asserted. That is the L3 failure's shape, and
# this ledger exists so it cannot survive in a table nobody can falsify.
echo "==> cdcp_gate capability-maturity (B1 capability ledger: attributed, dated, expiring)"
cargo run -q -p cdcp_gate -- capability-maturity || fail "capability maturity ledger"
ok "capability claims attributed, dated, unexpired, and pointed at evidence that resolves"

# B2 (bd-hardening-b-ledgers-gvm.2): what each frozen artifact was frozen AGAINST.
# Every [[surface]] pin in registries/goldens-couplings.toml is re-extracted from
# source here; a surface that moved, a version left unbumped, a golden that did
# not re-affirm, or a golden re-frozen at all is RED. Before this step, a grader
# semantics change and a bank typo fix produced the same diff — two hex strings —
# and UPDATE_GOLDENS=1 could re-freeze both without recording that anything moved.
# It found ten violations on its first run: C2 redefining bank_hash under all
# seven frozen artifacts, and all seven re-frozen the same hour with nothing
# naming the surface. Paid off by the walk, not by striking the rows.
echo "==> cdcp_gate goldens-couplings (B2 coupling ledger: no silent re-freeze)"
cargo run -q -p cdcp_gate -- goldens-couplings || fail "goldens coupling ledger"
ok "every golden names the surfaces it was frozen against, and both sides agree"


# exam_form hard numbers (public CDCP form)
grep -q 'n_items = 40' knowledge/exam_form.toml || fail "exam_form n_items"
grep -q 'duration_sec = 3600' knowledge/exam_form.toml || fail "exam_form duration"
grep -q 'pass_correct = 27' knowledge/exam_form.toml || fail "exam_form pass_correct"
grep -q 'credential_claim = "forbidden"' knowledge/exam_form.toml || fail "credential must be forbidden"
ok "exam_form public CDCP format pins"

# Honesty: no "you are certified" in engine docs (allow "not certified").
# CRITICAL: ~/.ripgreprc may contain --type-not=video (and friends). Those
# types are not registered → rg exits 2. Piping that through another filter
# under set -eu (no pipefail) fail-OPENs the honesty gate. Always:
#   1) rg --no-config  (ignore broken global type filters)
#   2) treat rc>=2 as hard fail (never green on scanner error)
if ! command -v rg >/dev/null 2>&1; then
  fail "rg required for honesty scan"
fi
honesty_rc=0
honesty_hits="$(rg --no-config -n --glob '*.md' --glob '*.toml' \
  'you are (now )?CDCP certified|officially certified by EPI' \
  docs knowledge 2>&1)" || honesty_rc=$?
case "$honesty_rc" in
  0)
    filtered="$(printf '%s\n' "$honesty_hits" | rg --no-config -v 'not |never |FORBIDDEN|forbidden' || true)"
    if [ -n "$filtered" ]; then
      printf '%s\n' "$filtered" >&2
      fail "possible credential inflation string"
    fi
    ;;
  1) ;; # no matches — clean
  *)
    fail "honesty scan error (rg rc=$honesty_rc) — refusing fail-open: $honesty_hits"
    ;;
esac
ok "honesty string smoke"

# Crosswalk: all primary domains 01-14 appear
for d in 01-mission-critical 02-standards 03-site-building 04-floor-ceiling \
  05-lighting 06-power 07-emf 08-racks 09-cooling 10-water \
  11-network 12-fire 13-security 14-auxiliary
do
  grep -q "domain = \"$d\"" knowledge/standards_crosswalk.toml || fail "crosswalk missing $d"
done
ok "standards crosswalk covers domains 01-14"

# Topics non-empty
topic_count="$(grep -c '^\[\[topic\]\]' knowledge/topics.toml || true)"
[ "$topic_count" -ge 50 ] || fail "topics.toml too thin ($topic_count)"
ok "topics.toml count=$topic_count"

# Sources have fetch_date
grep -q 'fetch_date' knowledge/sources.toml || fail "sources need fetch_date"
ok "sources fetch_date present"

# L2 bank pool floors — Rust gate [bd-substrate-rust-migration-jhd.7]
# The `if [ -f ... ]` guard this replaces was FAIL-OPEN: deleting the script made
# the gate vanish silently, and an unchecked bank reports exactly like a clean one.
[ -d bank/items ] || fail "missing bank/items (bank pool gate required)"
echo "==> cdcp_gate verify-bank (bank pool floors)"
cargo run -q -p cdcp_gate -- verify-bank || fail "bank verify"
ok "bank pool"


# Anti-hallucination heuristics + corpus overlap
# PORTED (bd-substrate-rust-migration-jhd.9). scripts/validate_grounding.py stays
# as the differential oracle for tests/diff_validate_grounding.rs; an absent
# checker is a fooled certificate, not a skip.
# Anti-vacuous (bd-yje7, closed 2026-08-14): zero items, a sub-floor corpus, and a
# missing corpus root are each RED and named. Floors are 40 items (one exam form)
# and 20000 corpus chars (one median module), recorded with their reasons in the
# oracle and mirrored in the port.
[ -f scripts/validate_grounding.py ] || fail "missing scripts/validate_grounding.py (differential oracle for validate-grounding)"
[ -d bank/items ] || fail "missing bank/items (grounding gate required)"
echo "==> cdcp_gate validate-grounding (anti-hallucination heuristics + corpus overlap)"
cargo run -q -p cdcp_gate -- validate-grounding || fail "grounding"
ok "grounding heuristics"

# Orphan referential integrity (topics <-> bank) — ORACLE-GAUNTLET "orphan item".
# Hard-required: an absent checker is a fooled certificate, not a skip.
[ -f scripts/verify_orphans.py ] || fail "missing scripts/verify_orphans.py (orphan gate required)"
echo "==> cdcp_gate verify-orphans (topic<->item referential integrity)"
cargo run -q -p cdcp_gate -- verify-orphans || fail "orphan referential integrity"
ok "no orphan topics · no orphan item refs · no unanchored items"

[ -f scripts/selftest_orphan.sh ] || fail "missing scripts/selftest_orphan.sh (L4 orphan known-bad required)"
echo "==> selftest_orphan.sh (L4 orphan known-bad)"
run_selftest "orphan known-bad selftest" sh scripts/selftest_orphan.sh
ok "orphan selftest (empty bank/topics ERROR · orphan ref RED · unanchored RED · orphan topic RED · live GREEN)"

# L3 GradeExact — cargo + goldens (BUILT must be WIRED here)
if [ ! -f Cargo.toml ]; then
  fail "Cargo.toml missing (L3 workspace required)"
fi

echo "==> cargo fmt/clippy/test"
cargo fmt --check || fail "cargo fmt"

# `cargo fmt` CANNOT SEE crates/cdcp_gate/src/gates/*.rs. Those modules are pulled
# in by `#[path = ...]` from a build.rs-generated OUT_DIR file, and cargo fmt walks
# the module tree from src/lib.rs — it never resolves that indirection. Measured
# 2026-08-14: a deliberately misformatted substrate_guard.rs returned `cargo fmt
# --check` exit 0 while `rustfmt --check` on the same file returned 1 with a
# 47-line diff. Three gate files were in fact unformatted and had never been
# checked by anything, including two already committed.
#
# This is the cost of the glob-based registration that lets N port agents each add
# one file with no shared-file collision. The parallelism is worth keeping; the
# unchecked surface it created is not. A green fmt leg over files it never opened
# is the same vacuous-scan pattern this script hard-fails on elsewhere.
_gate_fmt_n=0
for _f in crates/cdcp_gate/src/gates/*.rs; do
  [ -f "$_f" ] || continue
  rustfmt --edition 2021 --check "$_f" || fail "rustfmt: $_f"
  _gate_fmt_n=$((_gate_fmt_n + 1))
done
[ "$_gate_fmt_n" -gt 0 ] || fail "rustfmt scanned 0 gate files — a vacuous scan is an ERROR, not a pass"
ok "rustfmt over $_gate_fmt_n gate module(s) cargo fmt cannot reach"
# --all-targets is load-bearing, not cosmetic. Without it clippy never compiles
# test targets, so a deleted assertion leaves an unused binding that nothing
# complains about — measured 2026-08-14: the golden-sampler meta-test went RED
# under `clippy --all-targets` and GREEN under this line as it stood. A gate
# suite whose own meta-tests can be silently gutted is a fooled certificate.
cargo clippy --locked --workspace --all-targets -- -D warnings || fail "clippy"
cargo test --locked --workspace || fail "cargo test"
ok "cargo fmt + clippy -D warnings + test"

for f in \
  goldens/fixtures/mock40_seed42.json \
  goldens/mock40_seed42_all_correct.sha256 \
  goldens/mock40_seed42_all_wrong.sha256 \
  goldens/bank_hash.txt \
  goldens/PROVENANCE.md \
  docs/CANONICAL.md
do
  [ -f "$f" ] || fail "missing L3 artifact $f"
done
ok "L3 golden artifacts present"

echo "==> cdcp goldens check"
cargo run -q -p cdcp_cli -- goldens check --bank bank/items --dir goldens \
  || fail "goldens check"
ok "GradeExact goldens"

# L4 — gates proven to trip (inject known-bad → assert RED → restore)
if [ -x scripts/selftest_known_bad.sh ] || [ -f scripts/selftest_known_bad.sh ]; then
  echo "==> selftest_known_bad.sh (L4)"
  run_selftest "known-bad selftests" sh scripts/selftest_known_bad.sh
  ok "known-bad selftests (gates trip, tree clean)"
else
  fail "missing scripts/selftest_known_bad.sh (L4 required)"
fi

# L4 WASM dual-path (optional until toolchain present — skip-honest, not full L4 green)
echo "==> L4 WASM dual-path (optional)"
L4_WASM="SKIP"
if command -v rustup >/dev/null 2>&1   && rustup target list --installed 2>/dev/null | grep -q '^wasm32-unknown-unknown$'
then
  if cargo build -p cdcp_wasm --target wasm32-unknown-unknown --locked \
    && CDCP_REQUIRE_WASM=1 cargo test -p cdcp_wasm --test dual_path --locked -- --nocapture
  then
    ok "L4 WASM dual-path native==wasm (mock40_seed42)"
    L4_WASM="GREEN"
  else
    fail "L4 WASM dual-path failed (toolchain present but digests disagree or test/build error)"
  fi
else
  echo "check.sh: SKIP wasm: toolchain missing"
  echo "check.sh: L4 dual-path is NOT full green — install: rustup target add wasm32-unknown-unknown"
  L4_WASM="SKIP"
fi

# Wave status
echo "check.sh: WAVE STATUS: W0+L1+L2+L3 GREEN; L4 known-bad WIRED; L4 WASM=$L4_WASM; L5 UI still OPEN"
if [ "$L4_WASM" = "GREEN" ]; then
  echo "check.sh: next: L5 browser mock path (UI e2e digest match)"
else
  echo "check.sh: next: enable wasm32 target for L4 dual-path GREEN, then L5 UI"
fi

# Knowledge primary_notes path resolution (parent ../modules/)
# The `if [ -f ... ]` guard this replaces was FAIL-OPEN. [bd-...-jhd.5]
echo "==> cdcp_gate verify-knowledge-paths (knowledge primary_notes resolve)"
cargo run -q -p cdcp_gate -- verify-knowledge-paths || fail "knowledge primary_notes paths"
ok "knowledge primary_notes paths"

# ─── L5 browser surface ─────────────────────────────────────────────────────
echo "==> L5 browser surface (require product files)"
for f in web/index.html web/learn.html web/drill.html web/mock.html web/reference.html; do
  [ -f "$f" ] || fail "L5 product file missing: $f"
done
ok "L5 product files present"
[ -f web/assets/wasm/cdcp_wasm.wasm ] || fail "L5 wasm artifact missing under web/assets/wasm/"
ok "L5 wasm artifact present under web/assets/wasm/"
python3 -c "
import json,sys
d=json.load(open('web/data/mock40_seed42.json'))
assert d['n_items']==40, 'n_items=%r' % d['n_items']
assert len(d['items'])==40, 'items=%d' % len(d['items'])
assert all('correct' not in i for i in d['items']), 'learner pack leaks correct letters'
" || fail "L5 learner pack shape"
ok "L5 learner pack n_items=40"

echo "==> selftest_l5.sh (honesty + e2e digest known-bad)"
run_selftest "L5 selftest" sh scripts/selftest_l5.sh
ok "L5 selftest (honesty plant RED · digest match · flipped golden RED · empty fixtures ERROR)"

echo "==> e2e_l5_digest.sh (UI dual-path digest match)"
sh scripts/e2e_l5_digest.sh || fail "L5 e2e digest"
ok "L5 e2e digest match (seed42 all-correct/all-wrong)"

echo "==> smoke_learn.py"
python3 scripts/smoke_learn.py || fail "L5 learn smoke"
ok "L5 learn smoke"

# ─── L6 mastery / coverage ──────────────────────────────────────────────────
echo "==> smoke_srs.mjs";        node scripts/smoke_srs.mjs        || fail "L6 srs smoke";        ok "L5 srs smoke"
echo "==> smoke_mastery.mjs";    node scripts/smoke_mastery.mjs    || fail "L6 mastery smoke";    ok "L6 mastery smoke"
echo "==> smoke_weak_links.py";  python3 scripts/smoke_weak_links.py || fail "L6 weak-links smoke"; ok "L6 weak-links smoke"
echo "==> smoke_hub_mastery.mjs"; node scripts/smoke_hub_mastery.mjs || fail "L6-S4 hub mastery"; ok "L6 hub mastery + recommend smoke"
ok "L6-S4 hub mastery surface wired"

echo "==> L6 multi-seed export-web (fixture golden-stable)"
_MS_TMP="$(mktemp -d "${TMPDIR:-/tmp}/cdcp_multiseed.XXXXXX")"
cargo run -q -p cdcp_cli -- export-web --bank bank/items --seed 42 --out "$_MS_TMP" >/dev/null \
  || fail "L6 multi-seed export-web"
for f in mock40_seed42.json keys_seed42.json bank_items_seed42.json; do
  cmp -s "$_MS_TMP/$f" "web/data/$f" || fail "L6 export-web seed42 not golden-stable: $f"
done
rm -rf "$_MS_TMP"
ok "L6 multi-seed export-web --seed 42 (fixture golden-stable)"

echo "==> L6 session shapes"
for _shape in "Drill due" "Miss review"; do
  grep -q "$_shape" web/drill.html || fail "L6 session shape missing from web/drill.html: $_shape"
done
ok "L6 session shapes (Drill due · Miss review) present"

[ -f scripts/verify_coverage.py ] || fail "missing scripts/verify_coverage.py (differential oracle for verify-coverage)"
echo "==> cdcp_gate verify-coverage (L6 domain coverage)"
cargo run -q -p cdcp_gate -- verify-coverage || fail "L6 coverage"
ok "L6 coverage GREEN (every module the domain registry declares ≥ domain_min)"
echo "==> selftest_l6_coverage.sh"
run_selftest "L6 coverage selftest" sh scripts/selftest_l6_coverage.sh
ok "L6 coverage selftest (empty RED · missing-module RED · live GREEN)"

# ─── L7 product surfaces ────────────────────────────────────────────────────
echo "==> L7 product surfaces"
for f in web/reference.html web/learn.html; do
  [ -f "$f" ] || fail "L7 surface missing: $f"
done
ok "L7 surfaces (reference · closed-notes · Learn-15)"

echo "==> smoke_learn_chrome.py (M8-A)"; python3 scripts/smoke_learn_chrome.py || fail "M8-A learn chrome"; ok "M8-A learn chrome smoke"
echo "==> cdcp_gate build-units (M8-B units_index)";         cargo run -q -p cdcp_gate -- build-units         || fail "M8-B units_index"; ok "M8-B units_index"
echo "==> cdcp_gate build-glossary-json (M8-D glossary)";    cargo run -q -p cdcp_gate -- build-glossary-json || fail "M8-D glossary";    ok "M8-D glossary.json"
echo "==> smoke_learn_v2.py";            python3 scripts/smoke_learn_v2.py      || fail "M8-B/D learn v2";  ok "M8-B/D learn v2 smoke"
echo "==> smoke_diagrams.py";            python3 scripts/smoke_diagrams.py      || fail "M8-C diagrams";    ok "M8-C diagrams smoke"
echo "==> smoke_a11y.py";                python3 scripts/smoke_a11y.py          || fail "L7 a11y";          ok "L7 a11y baseline"
echo "==> smoke_feedback_links.py";      python3 scripts/smoke_feedback_links.py || fail "L7 feedback links"; ok "L7-S2 feedback section-anchor links smoke"
ok "L7 feedback section links"

echo "==> L7 CLI product verbs"
_HELP="$(cargo run -q -p cdcp_cli -- --help 2>&1)"
for v in bank-hash grade goldens export-web serve; do
  printf '%s' "$_HELP" | grep -q -- "$v" || fail "L7 CLI verb missing from --help: $v"
done
ok "L7 CLI product verbs listed"

[ -f scripts/verify_objectives.py ] || fail "missing scripts/verify_objectives.py (differential oracle for verify-objectives)"
echo "==> cdcp_gate verify-objectives (L7-S7 objective coverage)"
cargo run -q -p cdcp_gate -- verify-objectives || fail "L7 objective coverage"
ok "L7 objective coverage (registry objectives resolve · every declared module carries items)"
echo "==> selftest_l7_objectives.sh"
run_selftest "L7 objectives selftest" sh scripts/selftest_l7_objectives.sh
ok "L7 objectives known-bad selftest"

echo "==> smoke_slo.sh"
if cargo run -q -p cdcp_cli -- export-web --help >/dev/null 2>&1; then
  sh scripts/smoke_slo.sh || fail "L7 SLO budgets"
  ok "L7 SLO budgets"
else
  echo "check.sh: GAP: L7 SLO budgets NOT RUN — cdcp_cli lacks the 'export-web' verb" >&2
  GAPS="${GAPS}L7-SLO "
fi

echo "==> cdcp_gate verify-content-lock (L7 content.lock)"
cargo run -q -p cdcp_gate -- verify-content-lock || fail "L7 content.lock"
ok "L7 content.lock"

# L4: the content-lock gate must be proven to TRIP, not merely to pass. This
# selftest already existed on both sides, fully implemented and fully unwired —
# nothing in check.sh ran it. BUILT != WIRED, found 2026-08-14. It flips the
# pinned bank_hash in a TEMP copy (the committed content.lock is never touched)
# and asserts the RED path is reached.
echo "==> cdcp_gate verify-content-lock selftest (L4 content.lock known-bad)"
CDCP_CONTENT_LOCK_SELFTEST=1 cargo run -q -p cdcp_gate -- verify-content-lock \
  || fail "L7 content.lock mutate-selftest did not reach RED"
ok "L7 content.lock selftest (mutated bank_hash trips RED)"

# ─── V11 stretch surfaces ───────────────────────────────────────────────────
if [ -f scripts/selftest_reconstructed.sh ] && [ "${CDCP_IN_SELFTEST:-0}" != "1" ]; then
  echo "==> selftest_reconstructed.sh (L5–V11 reconstructed stages)"
  run_selftest "reconstructed-stage selftests" env CDCP_IN_SELFTEST=1 sh scripts/selftest_reconstructed.sh
  ok "L5–V11 reconstructed stages proven to trip RED"
fi

if [ -f tests/voice-slop.sh ]; then
  echo "==> tests/voice-slop.sh (applicable slice of the ZS voice gate)"
  sh tests/voice-slop.sh >/dev/null || fail "voice slop / honesty in public copy"
  ok "public copy free of marketing slop; honesty note intact"
fi

# Roadmap doc truth — the prose a stranger reads first must not contradict
# itself. Hard-required: an absent checker is a fooled certificate, not a skip.
[ -f scripts/verify_doc_consistency.py ] || fail "missing scripts/verify_doc_consistency.py (roadmap-truth gate required)"
echo "==> cdcp_gate verify-doc-consistency (CHARTER §9 · README roadmap · PHASE-NEXT)"
cargo run -q -p cdcp_gate -- verify-doc-consistency || fail "roadmap doc consistency"
ok "roadmap milestone status agrees across docs; publication truth holds"

[ -f scripts/selftest_doc_consistency.sh ] || fail "missing scripts/selftest_doc_consistency.sh (L4 roadmap known-bad required)"
echo "==> selftest_doc_consistency.sh (L4 roadmap known-bad)"
run_selftest "roadmap doc consistency selftest" sh scripts/selftest_doc_consistency.sh
ok "roadmap selftest (dup row RED · cross-doc conflict RED · unreadable status RED · pending-publication RED · zero markdown ERROR)"

if [ -f tests/publishability-bar.sh ]; then
  echo "==> tests/publishability-bar.sh (L88 — audit claims must be true)"
  sh tests/publishability-bar.sh >/dev/null || fail "publishability bar (audit claim is false)"
  ok "L88 publishability bar (audit claims verified against the repo)"
fi

echo "==> V11 stretch surfaces"
python3 scripts/export_anki.py --check >/dev/null 2>&1 || python3 scripts/export_anki.py >/dev/null 2>&1 || fail "V11 Anki export"
ok "V11 Anki export"
grep -q "study aid" web/reference.html 2>/dev/null || grep -rq "not.*certif" web/ 2>/dev/null || fail "V11 diagram honesty"
ok "V11 diagram honesty present"
if cargo run -q -p cdcp_cli -- serve --help >/dev/null 2>&1; then
  ok "V11 serve subcommand present"
else
  echo "check.sh: GAP: V11 serve subcommand ABSENT from cdcp_cli source" >&2
  GAPS="${GAPS}V11-serve "
fi
ls bank/items/*.toml >/dev/null 2>&1 || fail "V11 runbook bank items"
ok "V11 runbook bank items present"


# ── L4 drift guard: the advertised known-bad count must be the measured one ──
# Skipped inside a nested check.sh (CDCP_IN_SELFTEST=1) because that run
# deliberately omits selftest_reconstructed.sh: aggregating a knowingly partial
# roster would turn every nested run RED for the wrong reason and mask the
# stage the nested run exists to prove.
if [ "${CDCP_IN_SELFTEST:-0}" != "1" ]; then
  [ -f scripts/verify_injection_count.py ] || fail "missing scripts/verify_injection_count.py (drift guard required)"
  echo "==> selftest_injection_count.sh (L4 drift-guard known-bad)"
  run_selftest "injection-count selftest" sh scripts/selftest_injection_count.sh
  ok "drift-guard selftest (off-by-one RED · missing receipt RED · zero RED · unregistered RED · empty log ERROR)"

  echo "==> cdcp_gate verify-injection-count (advertised known-bad count)"
  cargo run -q -p cdcp_gate -- verify-injection-count --log "$INJ_LOG" \
    || fail "known-bad injection count drift (README vs suites)"
  ok "advertised known-bad injection count == suites' self-reported total"
fi

if [ -n "$GAPS" ]; then
  echo "check.sh: KNOWN GAPS (not green, not silent): $GAPS" >&2
fi
echo "check.sh: complete != EPI certified (study signal / mastery only)"
echo "==> check.sh PASSED (W0-L7 + V11 stretch; L4 WASM=$L4_WASM)"
exit 0
