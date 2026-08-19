#!/usr/bin/env sh
# check.sh — fail-closed gate for cdcp-course engine
# Waves incomplete: exit 2 with clear message until L3+ tools exist.
#
# L4 selftests: scripts/selftest_known_bad.sh injects known-bad fixtures,
# asserts RED, restores. Never leave goldens/bank dirty.
set -eu

# ── Snapshot re-exec [bd-o4bc] ─────────────────────────────────────────────
# sh reads a script INCREMENTALLY. Editing scripts/check.sh while a run is in
# flight splices the file. Measured 2026-08-14: exit 127, `line 418: en:
# command not found` — a token sheared in half mid-read.
#
# The run-lock [bd-gl4j] serialises concurrent RUNS. This writer is not a run
# — it is an editor. No lock between runs can stop a third party writing the
# script a running shell is still reading.
#
# Remedy: copy this file to a private path under target/ (gitignored) and
# exec the copy BEFORE any lock is taken. The snapshot process is what takes
# the gl4j lock. Nested same-root descendants invoke the live scripts/check.sh
# again; that live file copies + re-execs a new snapshot (new PID) and then
# sees the ancestor's lock via CDCP_CHECK_LOCK_HELD. prove-wired and bd-791t
# reconstructed use a different ROOT, hence a different snapshot dest and a
# different lock, taken independently.
#
# Recursion guard is the PATH of $0, not an env var: a caller who exports
# CDCP_CHECK_ROOT (or any CDCP_CHECK_SNAPSHOT*) still hits the live-file
# branch and still copies. An env-only guard is how this becomes a silent
# fall-through to the live file.
#
# Assembled so the contiguous token appears ONCE (the next line). replace_once
# shears that single occurrence; grep source must not contain it as a substring.
_SNAP_INTACT="bd-o4bc-SNAPSHOT"-"INTACT"
_SNAP_SHEARED="bd-o4bc-SNAPSHOT"-"SHEARED"
# bd-o4bc-SNAPSHOT-INTACT

snapshot_error() { echo "check.sh: SNAPSHOT ERROR: $*" >&2; exit 2; }

_chk_dir="$(CDPATH= cd -- "$(dirname "$0")" && pwd)" \
  || snapshot_error "cannot resolve dirname of \$0=$0"
_chk_self="$_chk_dir/$(basename "$0")"
_SNAP_CLEAN=""

case "$_chk_self" in
  */scripts/check.sh)
    ROOT="$(CDPATH= cd -- "$_chk_dir/.." && pwd)" \
      || snapshot_error "cannot resolve engine root from $_chk_dir"
    _snap_dir="$ROOT/target/check.snap.$$"
    if [ "${CDCP_CHECK_SNAPSHOT_PROBE:-0}" = "1" ] && [ -n "${CDCP_CHECK_SNAP_DIR:-}" ]; then
      _snap_dir="$CDCP_CHECK_SNAP_DIR"
    fi
    mkdir -p "$_snap_dir" 2>/dev/null || true
    if [ ! -d "$_snap_dir" ]; then
      snapshot_error "cannot create $_snap_dir to hold the running copy; an unwritable snapshot path is an ERROR, not a fall-through to the live file"
    fi
    _snap="$_snap_dir/check.sh"
    if ! cp "$_chk_self" "$_snap"; then
      snapshot_error "cannot copy $_chk_self to $_snap; refusing to run the live file"
    fi
    if [ ! -f "$_snap" ] || [ ! -s "$_snap" ]; then
      snapshot_error "copy at $_snap is missing or empty; refusing to run the live file"
    fi
    CDCP_CHECK_ROOT="$ROOT"
    export CDCP_CHECK_ROOT
    exec sh "$_snap" "$@" || snapshot_error "exec sh $_snap failed; refusing to fall through to the live file"   # CHARTER-NEEDLE-EXEC
    ;;
  *)
    [ -n "${CDCP_CHECK_ROOT:-}" ] \
      || snapshot_error "running from $_chk_self without CDCP_CHECK_ROOT (refusing to guess ROOT from a snapshot path)"
    ROOT="$CDCP_CHECK_ROOT"
    case "$_chk_self" in
      */target/check.snap.$$/check.sh)
        # Probe children leave the copy for the parent to inspect; the
        # private-tree selftest rm -rf's the whole scratch dir.
        if [ "${CDCP_CHECK_SNAPSHOT_PROBE:-0}" != "1" ]; then
          _SNAP_CLEAN="$(dirname "$_chk_self")"
        fi
        ;;
    esac
    ;;
esac

# Early trap so a probe child that exits before the main cleanup is installed
# still removes its private copy.
_snap_early_cleanup() {
  if [ -n "${_SNAP_CLEAN:-}" ]; then rm -rf "$_SNAP_CLEAN"; fi
  return 0
}
trap '_snap_early_cleanup' EXIT INT TERM HUP

# Snapshot probe: handshake + shear assertion. Reached only after re-exec
# (or after a CHARTER-mutated fall-through). Exits before the lock.
if [ "${CDCP_CHECK_SNAPSHOT_PROBE:-0}" = "1" ] && [ -n "${CDCP_CHECK_SNAP_HANDSHAKE:-}" ]; then
  printf '%s\n' "$_chk_self" >"$CDCP_CHECK_SNAP_HANDSHAKE/running" \
    || snapshot_error "cannot write handshake running-path"
  _hs_try=0
  while [ ! -f "$CDCP_CHECK_SNAP_HANDSHAKE/go" ]; do
    _hs_try=$((_hs_try + 1))
    [ "$_hs_try" -lt 50 ] || snapshot_error "timed out waiting for handshake go"
    sleep 0.1
  done
  # CHARTER-NEEDLE-ASSERT
  snapshot_probe_assert() { grep -q "$_SNAP_SHEARED" "$1" && return 1; grep -q "$_SNAP_INTACT" "$1" || return 1; return 0; }
  if ! snapshot_probe_assert "$_chk_self"; then
    snapshot_error "running script is not an isolated copy ($_chk_self) — an in-flight edit of the tree file reached the running script"
  fi
  echo "check.sh: snapshot probe: isolated (running $_chk_self)"
  exit 0
fi

cd "$ROOT"

fail() { echo "check.sh: FAIL: $*" >&2; exit 2; }
GAPS=""
# Step counter [bd-1sd.13]. The advertised chain length is OK + SKIPPED, so a
# machine that honestly cannot run a leg (no wasm32, nested reconstructed run)
# still advertises the same number as one that ran every leg. A transcript
# grep of `check.sh: ok:` over-counts: substrate-guard --prove-wired copies a
# nested child's oks into this process's stdout. The receipt is emitted by
# THIS process, after the sealed boundary, and is what the drift guard reads.
STEP_OK=0
STEP_SKIPPED=0
ok() { STEP_OK=$((STEP_OK + 1)); echo "check.sh: ok: $*"; }
skipped_step() { STEP_SKIPPED=$((STEP_SKIPPED + 1)); echo "check.sh: skip: $*"; }

# After each generator, owned paths must be clean. [bd-installability-sm4g.10]
# A successful run that leaves dirt is stale committed artifacts — RED.
# Reports only. The helper never stages or restores.
assert_generator_fresh() {
  sh scripts/assert_generator_fresh.sh "$@"
}

# ── Concurrency lock [bd-gl4j] ─────────────────────────────────────────────
# Measured 2026-08-14, four concurrent runs live during a six-agent wave: one
# run exited 2 on `L5 learner pack shape (n_items=39)` because another run was
# inside selftest_reconstructed.sh, which then mutated web/data/mock40_seed42.json
# IN THE WORKING TREE and restored it afterwards. The second run read the
# mutated state and reported a product defect that did not exist.
# bd-791t (2026-08-15): reconstructed now injects in a private tree under
# target/cdcp-recon-*/ — live tracked files, including crates/cdcp_cli/src/main.rs,
# are not written. The lock remains because other steps below still mutate live.
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
#   selftest_wasm_freshness.sh does NOT mutate live tracked files: plant 1
#                              flips a TEMP copy of the wasm; plant 2 uses
#                              isolated CARGO_TARGET_DIR + RUSTFLAGS cfg
#   selftest_l5.sh             web/_selftest_l5_honesty_planted.html    [S] planted/removed
#   selftest_reconstructed.sh  (bd-791t) private tree under target/cdcp-recon-*/
#                              live tracked files are not written
#   cdcp build-learn               web/data/modules_index.json          [M] regenerated
#                                  web/data/topic_anchors.json          [M] regenerated
#                                  web/learn.html + web/learn/*.html    [M] regenerated
#                                  web/content/modules/*.md             [M] recopied
#   cdcp build-reference           web/reference.html                   [M] regenerated
#                                  web/content/reference/*.md           [M] recopied
#   cdcp build-units               web/data/units_index.json            [M] regenerated
#   cdcp build-glossary            web/data/glossary.json               [M] regenerated
#   cdcp build-learn-slugs         web/data/module_learn_slugs.js       [M] regenerated
#   cdcp export-web                web/data/{mock40,keys,bank_items}_seed42.json [S] regenerated
#   cdcp export-anki               dist/anki/**                         [M] untracked output
# The regenerated Learn artifacts are byte-identical today, so `git status` stays
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
# Re-entrancy: selftest_reconstructed.sh used to re-enter this script in the
# SAME root, five times (and then from a private tree). As of bd-791t it
# mutates a private snapshot under target/cdcp-recon-*/snap and runs the
# same per-stage predicates; it does not re-enter this script. Live tracked
# files, including crates/cdcp_cli/src/main.rs, are not written.
# CDCP_CHECK_LOCK_HELD still exists for any same-root descendant.
# `substrate-guard --prove-wired` also runs check.sh from a tree materialised
# under target/ — a different ROOT, hence a different lock, taken independently.
#
# SNAPSHOT vs LOCK [bd-o4bc × bd-gl4j]: copy+re-exec happens FIRST. The live
# process never holds the lock; the snapshot process does. An editor writing
# scripts/check.sh is not a run, so the lock does not serialise against it —
# the snapshot is what makes that write harmless to an in-flight shell.
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
# tees the receipts into INJ_LOG. cdcp_gate verify-injection-count then sums
# them and compares against the count README.md advertises — so the badge can
# never drift from the machinery it describes.
#
# CDCP_INJECTION_COUNT_WRITE_README=1 is the reachable caller of --write-readme
# [bd-injection-count-regen-unreachable-lu45]. Without it, drift is still RED.
# The flag cannot launder an unsound total: the gate refuses to write when the
# receipts themselves are not sound.
#
# STEP_LOG is the sibling receipt for the chain's own length [bd-1sd.13].
# check.sh writes one CHECK_STEPS= line on the success path; verify-step-count
# compares it to every "<N> ordered steps" claim in README.md. A missing log
# is an ERROR, never a silent zero.
INJ_LOG=""
STEP_LOG=""

# Single cleanup for both resources. Installed BEFORE the lock is taken, so a
# signal arriving between mkdir and the first gate still releases it. It cannot
# survive SIGKILL — hence stale reclamation below. Explicit `if` blocks and a
# terminal `return 0`: an `&&` chain whose test is false would return non-zero
# from an EXIT trap under `set -e`.
cleanup() {
  if [ -n "$INJ_LOG" ]; then rm -f "$INJ_LOG"; fi
  if [ -n "$STEP_LOG" ]; then rm -f "$STEP_LOG"; fi
  if [ "$LOCK_HELD" = "1" ]; then rm -rf "$LOCK_DIR"; fi
  if [ -n "${_SNAP_CLEAN:-}" ]; then rm -rf "$_SNAP_CLEAN"; fi
  # bd-installability-sm4g.23: never leave L4 plants in a production scan path,
  # including fail()/signal. Do not replace this trap — fold into it.
  if [ -n "${_fmt_selftest_root:-}" ]; then rm -rf "$_fmt_selftest_root"; fi
  rm -f "${ROOT}/scripts/__cdcp_probe_unlisted__.py"
  rm -f "${ROOT}/crates/cdcp_gate/src/gates/__cdcp_fmt_plant__.rs"
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
  # Snapshot CHARTER rewrites call `cdcp snap-rewrite`. That binary exists
  # only after compile-once; the L4 isolation suite (and the combined
  # lock+snapshot ok) runs there so this stays one advertised step.
  # EXTRACT-THEN-DELETE: python3 -c CHARTER helpers are retired.
else
  skipped_step "concurrency lock selftest (running under an ancestor's lock) · snapshot re-exec selftest"
fi

INJ_LOG="$(mktemp "${TMPDIR:-/tmp}/cdcp_injections.XXXXXX")"
STEP_LOG="$(mktemp "${TMPDIR:-/tmp}/cdcp_steps.XXXXXX")"

run_selftest() {
  _lbl="$1"
  shift
  # Output is captured so the receipt can be teed; it therefore appears only
  # once the suite finishes. selftest_reconstructed.sh used to re-enter the
  # full gate five times; as of bd-791t it runs against a private snapshot.
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

# [bd-checksh-cargo-run-attribution-tebe]
# Compile once. Later steps invoke the debug binaries so rustc output from a
# sibling crate cannot be attributed to a gate, and a compile failure names
# THIS step. Honours CARGO_TARGET_DIR so the nested --prove-wired child
# (which sets it) is idempotent and looks in the same profile's target/.
# The child dies on the planted .py at substrate-guard, so this cannot
# recurse into --prove-wired.
require_cdcp_bins() {
  [ -n "${CDCP_BIN_DIR:-}" ] \
    || fail "CDCP_BIN_DIR unset — cargo build -p cdcp_gate -p cdcp_cli -p cdcp_registry_check --locked must run first (no fallback to cargo run)"
  [ -x "$CDCP_BIN_DIR/cdcp_gate" ] \
    || fail "cdcp_gate binary absent at $CDCP_BIN_DIR/cdcp_gate — cargo build -p cdcp_gate -p cdcp_cli -p cdcp_registry_check --locked did not produce it (no fallback to cargo run)"
  [ -x "$CDCP_BIN_DIR/cdcp" ] \
    || fail "cdcp binary absent at $CDCP_BIN_DIR/cdcp — cargo build -p cdcp_gate -p cdcp_cli -p cdcp_registry_check --locked did not produce it (no fallback to cargo run)"
  [ -x "$CDCP_BIN_DIR/cdcp_registry_check" ] \
    || fail "cdcp_registry_check binary absent at $CDCP_BIN_DIR/cdcp_registry_check — cargo build -p cdcp_gate -p cdcp_cli -p cdcp_registry_check --locked did not produce it (no fallback to cargo run)"
}
# argv[0] is ./target/debug/<bin> (or $CARGO_TARGET_DIR/debug/<bin>).
run_cdcp_gate() {
  require_cdcp_bins
  "$CDCP_BIN_DIR/cdcp_gate" "$@"
}
run_cdcp_cli() {
  require_cdcp_bins
  "$CDCP_BIN_DIR/cdcp" "$@"
}
run_cdcp_registry_check() {
  require_cdcp_bins
  "$CDCP_BIN_DIR/cdcp_registry_check" "$@"
}

echo "==> cargo build -p cdcp_gate -p cdcp_cli -p cdcp_registry_check --locked (once; later steps run the binary)"
cargo build -p cdcp_gate -p cdcp_cli -p cdcp_registry_check --locked \
  || fail "cargo build -p cdcp_gate -p cdcp_cli -p cdcp_registry_check --locked (compile failure is this step's, not a later gate's)"
if [ -n "${CARGO_TARGET_DIR:-}" ]; then
  CDCP_BIN_DIR="${CARGO_TARGET_DIR%/}/debug"
else
  CDCP_BIN_DIR="$ROOT/target/debug"
fi
require_cdcp_bins
ok "cargo build -p cdcp_gate -p cdcp_cli -p cdcp_registry_check --locked (debug binaries in $CDCP_BIN_DIR)"

# ── L4: snapshot re-exec is proven to isolate [bd-o4bc] ─────────────────
# Plants run against a PRIVATE tree that contains only scripts/check.sh.
# The live scripts/check.sh is never sheared. CHARTER pair: (1) skip exec
# → isolation RED; (2) mutation still in place, delete the assertion →
# GREEN. Restore of the temp tree is rm -rf (not a cargo artifact).
# EXTRACT-THEN-DELETE: python3 -c CHARTER helpers retired; this block
# calls `cdcp snap-rewrite`. Putting python3 back on this path is RED.
if [ "$LOCK_HELD" = "1" ]; then
  echo "==> check.sh snapshot selftest (L4: shear isolated · empty copy ERRORs · CHARTER pair · env guard)"
  _ss="$ROOT/target/cdcp-snap-selftest"
  rm -rf "$_ss"
  mkdir -p "$_ss/tree/scripts" "$_ss/tree/target" || fail "snapshot selftest: cannot create $_ss"
  cp "$ROOT/scripts/check.sh" "$_ss/tree/scripts/check.sh" \
    || fail "snapshot selftest: cannot copy check.sh into the private tree"

  _snap_replace_once() {
    run_cdcp_cli snap-rewrite replace-once --file "$1" --from "$2" --to "$3"
  }

  _snap_wait_file() {
    _wf="$1"
    _wtry=0
    while [ "$_wtry" -lt 50 ]; do
      [ -f "$_wf" ] && return 0
      sleep 0.1
      _wtry=$((_wtry + 1))
    done
    return 1
  }

  # _snap_run_isolation ROOT_DIR HS_DIR — shear the TREE copy after the child
  # has re-exec'd, then return the child's exit code in _ISO_RC.
  _snap_run_isolation() {
    _iso_root="$1"
    _iso_hs="$2"
    _iso_script="$_iso_root/scripts/check.sh"
    rm -rf "$_iso_hs"
    mkdir -p "$_iso_hs" || return 2
    (
      CDPATH= cd -- "$_iso_root" &&
        CDCP_CHECK_SNAPSHOT_PROBE=1 CDCP_CHECK_SNAP_HANDSHAKE="$_iso_hs" \
          sh scripts/check.sh
    ) >"$_iso_hs/out" 2>&1 &
    _iso_pid=$!
    if ! _snap_wait_file "$_iso_hs/running"; then
      kill "$_iso_pid" 2>/dev/null || true
      wait "$_iso_pid" 2>/dev/null || true
      echo "snapshot selftest: child never wrote handshake: $(cat "$_iso_hs/out" 2>/dev/null)" >&2
      _ISO_RC=2
      return 0
    fi
    _snap_replace_once "$_iso_script" "$_SNAP_INTACT" "$_SNAP_SHEARED" \
      || { kill "$_iso_pid" 2>/dev/null || true; wait "$_iso_pid" 2>/dev/null || true; _ISO_RC=2; return 0; }
    : >"$_iso_hs/go"
    _ISO_RC=0
    wait "$_iso_pid" || _ISO_RC=$?
    # Write-bytes restore (never mv): put the intact sentinel back so a later
    # CHARTER mutate still sees one needle.
    _snap_replace_once "$_iso_script" "$_SNAP_SHEARED" "$_SNAP_INTACT" || true
  }

  # Line rewriter: needles are matched by shape so this block does not
  # duplicate the production strings replace_once would count.
  _snap_charter() {
    run_cdcp_cli snap-rewrite charter --file "$1" --kind "$2"
  }

  # (1) Anti-vacuous: unwritable snapshot dest is ERROR, not a live-file run.
  : >"$_ss/notadir"
  _ss_rc=0
  _ss_out="$(
    CDPATH= cd -- "$_ss/tree" &&
      CDCP_CHECK_SNAPSHOT_PROBE=1 CDCP_CHECK_SNAP_DIR="$_ss/notadir/snap" \
        sh scripts/check.sh 2>&1
  )" || _ss_rc=$?
  [ "$_ss_rc" -ne 0 ] \
    || fail "snapshot selftest: an unwritable snapshot path ran GREEN by falling through to the live file"
  printf '%s\n' "$_ss_out" | grep -q "SNAPSHOT ERROR" \
    || fail "snapshot selftest: unwritable snapshot path failed without naming SNAPSHOT ERROR: $_ss_out"

  # (2) Env guard: a caller-set CDCP_CHECK_ROOT must not skip the copy.
  mkdir -p "$_ss/hs-env"
  (
    CDPATH= cd -- "$_ss/tree" &&
      CDCP_CHECK_ROOT="/tmp/cdcp-wrong-root-o4bc" \
      CDCP_CHECK_SNAPSHOT_PROBE=1 CDCP_CHECK_SNAP_HANDSHAKE="$_ss/hs-env" \
        sh scripts/check.sh
  ) >"$_ss/hs-env/out" 2>&1 &
  _ss_env_pid=$!
  if ! _snap_wait_file "$_ss/hs-env/running"; then
    kill "$_ss_env_pid" 2>/dev/null || true
    wait "$_ss_env_pid" 2>/dev/null || true
    fail "snapshot selftest: env-guard child never wrote handshake: $(cat "$_ss/hs-env/out" 2>/dev/null)"
  fi
  _ss_env_run="$(cat "$_ss/hs-env/running")"
  case "$_ss_env_run" in
    */target/check.snap.*/check.sh) ;;
    *)
      kill "$_ss_env_pid" 2>/dev/null || true
      wait "$_ss_env_pid" 2>/dev/null || true
      fail "snapshot selftest: CDCP_CHECK_ROOT skipped the copy (running=$_ss_env_run)"
      ;;
  esac
  : >"$_ss/hs-env/go"
  _ss_env_rc=0
  wait "$_ss_env_pid" || _ss_env_rc=$?
  [ "$_ss_env_rc" -eq 0 ] \
    || fail "snapshot selftest: env-guard child exited $_ss_env_rc: $(cat "$_ss/hs-env/out" 2>/dev/null)"

  # (3) Known-good: shear the TREE copy; the running snapshot stays intact.
  cp "$ROOT/scripts/check.sh" "$_ss/tree/scripts/check.sh"
  _snap_run_isolation "$_ss/tree" "$_ss/hs-good"
  [ "$_ISO_RC" -eq 0 ] \
    || fail "snapshot selftest: known-good isolation went RED (rc=$_ISO_RC): $(cat "$_ss/hs-good/out" 2>/dev/null)"
  _ss_good_run="$(cat "$_ss/hs-good/running")"
  case "$_ss_good_run" in
    */target/check.snap.*/check.sh) ;;
    *) fail "snapshot selftest: known-good did not re-exec a snapshot (running=$_ss_good_run)" ;;
  esac
  grep -q "$_SNAP_INTACT" "$_ss_good_run" \
    || fail "snapshot selftest: running copy lost the intact sentinel"
  if grep -q "$_SNAP_SHEARED" "$_ss_good_run"; then
    fail "snapshot selftest: running copy contains the sheared token"
  fi
  grep -q "$_SNAP_SHEARED" "$_ss/tree/scripts/check.sh" \
    && fail "snapshot selftest: source restore after known-good left the sheared token in the tree copy"

  # (4) CHARTER pair. A suite that only runs leg 1 is the defect.
  _sp_pair=0
  cp "$ROOT/scripts/check.sh" "$_ss/tree/scripts/check.sh"
  _snap_charter "$_ss/tree/scripts/check.sh" skip-exec \
    || fail "snapshot selftest: CHARTER mutate (skip exec) failed"
  _snap_run_isolation "$_ss/tree" "$_ss/hs-mutate"
  [ "$_ISO_RC" -ne 0 ] \
    || fail "CHARTER pair leg 1: skipping exec stayed GREEN when the source was sheared"
  _sp_pair=$((_sp_pair + 1))

  _snap_charter "$_ss/tree/scripts/check.sh" delete-assert \
    || fail "snapshot selftest: CHARTER delete-assertion failed"
  _snap_run_isolation "$_ss/tree" "$_ss/hs-del"
  [ "$_ISO_RC" -eq 0 ] \
    || fail "CHARTER pair leg 2: deleting the shear assertion did not return to GREEN (rc=$_ISO_RC): $(cat "$_ss/hs-del/out" 2>/dev/null)"
  _sp_pair=$((_sp_pair + 1))
  [ "$_sp_pair" -eq 2 ] \
    || fail "ANTI-VACUOUS: CHARTER pair ran $_sp_pair legs, want 2 (a suite that only runs leg 1 is the defect)"

  rm -rf "$_ss"
  ok "concurrency lock proven (second run refused naming pid $$ · dead holder reclaimed · unwritable lock path ERRORs) · snapshot re-exec proven (shear isolated · empty copy ERRORs · CHARTER pair 2/2 · env guard)"
fi

echo "==> cdcp_registry_check (L1 claims constitution)"
run_cdcp_registry_check || fail "registry-check"
ok "L1 registry-check"

# D4: the three-field rights/redistribution/ai_ingestion split is a product
# check, not a gate. cdcp_evidence owns the rules; this is the wire.
echo "==> cdcp check-licence (D4 three-field rights split)"
run_cdcp_cli check-licence || fail "licence three-field split"
ok "licence three-field split (published unlicensed / missing rights / third-party public-domain / PROHIBITED index)"

# Published-corpus rights: metadata + tree vs rights-policy.toml. Product
# crate cdcp_data; never a cdcp_gate src/gates/*.rs (gate_shrink ceiling).
# Never opens a capture body (AI-ingestion forbid).
echo "==> cargo test -p cdcp_data --test corpus_rights (L4 CORPUS-R7/R8)"
cargo test --locked -p cdcp_data --test corpus_rights \
  || fail "corpus-rights tests"
ok "corpus-rights tests (R7 bare permitted RED · R8 unclaimed file RED · public-domain PASS)"

echo "==> cdcp corpus-rights (published corpus vs rights-policy.toml)"
run_cdcp_cli corpus-rights || fail "corpus-rights"
ok "corpus-rights (metadata+tree; never opens capture bodies)"

# E1: licence-gated snapshot loader. Product crate cdcp_data; D4 may_load
# is the rights check. This is the wire. No network I/O.
echo "==> cdcp load-snapshots (E1 licence-gated snapshot loader)"
run_cdcp_cli load-snapshots || fail "snapshot loader"
ok "licence-gated snapshot loader (may_load + sha256 pin + anti-vacuous)"

# E3: OSHA/eCFR facts. 1910.147(a)(1)(ii)(D) exclusion is first-class;
# 1910.333 isolation constraints are legal, not slogans.
echo "==> cdcp check-osha (E3 OSHA/eCFR facts)"
run_cdcp_cli check-osha || fail "OSHA facts"
ok "OSHA facts (147 exclusion · 333 isolation · no 147-as-electrical-LOTO)"

# E: content.lock [data] independently pins every snapshots.toml body + sidecar.
# Editing both the body and the pin still trips this check.
echo "==> cdcp verify-data-lock (E content.lock data artifacts)"
run_cdcp_cli verify-data-lock || fail "data lock"
ok "content.lock [data] pins every snapshots.toml file"

echo "==> cdcp verify-data-lock --selftest (L4 data lock known-bad)"
run_cdcp_cli verify-data-lock --selftest \
  || fail "data lock flip-selftest did not reach RED"
ok "data lock selftest (flipped vendored body trips RED)"

# F3: computed site quantities vs published references we do not control.
# Live GREEN after 4up.5 (full PLNT23 vs official SRCO2RTA). No network.
# qly.8 --selftest plants stay in the CLI; cargo test still runs them.
echo "==> cdcp oracle-check (F3 external oracle)"
run_cdcp_cli oracle-check || fail "oracle-check"
ok "external oracle (computed vs published refs; no network)"

# S0 substrate floor. Placed next to the L1 registry gate because it is the same
# kind of thing — a registry constitution over what may exist in the tree — and it
# fails fast (only serde+toml compile).
#
# bd-installability-sm4g.23: the L4 plant lives only in the prove-wired scratch
# tree. If it is in THIS scan root it is a leftover, not a new unlisted .py.
# Skip inside CDCP_SUBSTRATE_PROBE=1 — there the plant is the intentional
# known-bad that must reach `cdcp_gate substrate-guard` so --prove-wired can
# attribute the RED to the binary, not to this wrapper.
SUBSTRATE_STALE_PLANT="scripts/__cdcp_probe_unlisted__.py"
reject_stale_substrate_plant() {
  if [ -n "${CDCP_SUBSTRATE_PROBE:-}" ]; then
    return 0
  fi
  if [ -e "$SUBSTRATE_STALE_PLANT" ]; then
    echo "substrate-guard: STALE PLANT: $SUBSTRATE_STALE_PLANT is in the production scan root; this is a selftest-cleanup failure, not an ordinary unlisted-file violation" >&2
    return 2
  fi
  return 0
}

# The known-bad is production-only. Inside --prove-wired the detector is
# a no-op so the intentional plant can reach the binary; running this
# block there would "stay GREEN" and fail the probe for the wrong reason.
if [ -z "${CDCP_SUBSTRATE_PROBE:-}" ]; then
  echo "==> L4 stale-plant detector (substrate scan path)"
  printf '%s\n' 'print("stale substrate plant")' > "$SUBSTRATE_STALE_PLANT"
  _stale_sub_diag=""
  if _stale_sub_diag="$(reject_stale_substrate_plant 2>&1)"; then
    fail "substrate stale-plant detector stayed GREEN with $SUBSTRATE_STALE_PLANT in the scan path"
  fi
  printf '%s\n' "$_stale_sub_diag" | grep -F "STALE PLANT" >/dev/null \
    || fail "substrate leftover plant must be named STALE PLANT, not an ordinary unlisted-file violation"
  printf '%s\n' "$_stale_sub_diag" | grep -F "not an ordinary unlisted-file violation" >/dev/null \
    || fail "substrate stale plant must be distinguished from a real unlisted-file violation"
  printf '%s\n' "$_stale_sub_diag" | grep -F "no row in registries/substrate_allowlist.toml" >/dev/null \
    && fail "substrate stale plant was misreported as a real unlisted-file violation"
  rm -f "$SUBSTRATE_STALE_PLANT"
fi

echo "==> cdcp_gate substrate-guard (S0 substrate floor)"
reject_stale_substrate_plant || fail "stale L4 substrate plant in the production scan path"
run_cdcp_gate substrate-guard || fail "substrate guard (unreasoned .py/.sh)"
ok "S0 substrate floor (no unreasoned py/sh-family file, shebang script, symlink or submodule anywhere in the engine tree · stale plant STALE PLANT)"

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
run_cdcp_gate substrate-guard --prove-wired \
  || fail "substrate-guard wiring does not stop check.sh"
ok "S0 wiring proven behaviourally (a planted unlisted .py stops check.sh)"

# bd-m67m: a FRESH CLONE HAS NO HOOK, and CI is a fresh clone — so the --check
# below was RED on every CI run and green on every developer machine, which is
# the worst possible split. install-hooks is idempotent, so this is a no-op
# locally and the thing that makes the next line meaningful in CI.
echo "==> cdcp_gate install-hooks (a fresh clone has no hook; CI is a fresh clone)"
run_cdcp_gate install-hooks || fail "install-hooks"
ok "pre-commit shim installed (idempotent)"

echo "==> cdcp_gate install-hooks --check (BUILT != WIRED)"
run_cdcp_gate install-hooks --check \
  || fail "pre-commit shim not installed (run: cdcp_gate install-hooks)"
ok "pre-commit shim installed and current"

# B1 (bd-hardening-b-ledgers-gvm.1): the machine ledger of capability claims.
# A row whose last_review has aged past registries/capability-maturity.toml's
# staleness_days, whose evidence names a test function nothing defines, or whose
# published CHARTER cell outruns the level its evidence can carry, is RED.
# It found two on its first run — L2 and L5 both published "YES · wired" over
# capabilities that no named test asserted. That is the L3 failure's shape, and
# this ledger exists so it cannot survive in a table nobody can falsify.
echo "==> cdcp_gate capability-maturity (B1 capability ledger: attributed, dated, expiring)"
run_cdcp_gate capability-maturity || fail "capability maturity ledger"
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
run_cdcp_gate goldens-couplings || fail "goldens coupling ledger"
ok "every golden names the surfaces it was frozen against, and both sides agree"

# B3 (bd-hardening-b-ledgers-gvm.3): present-tense claims about code, in prose,
# carry a yes/no the tree recomputes. verify-doc-consistency reads milestone
# TABLES and stayed exit 0 while PLAN §C2 said "hash_payload() omits them" and
# ROADMAP-WAVES said bank_hash covers "not objective_ids, citation ids, or item
# status" — both false, both narrative, both structurally invisible to a table
# parser. Every [[fact:id=yes|no]] marker is re-answered from the tree here.
# The polarity lives in the PROSE, not the registry, so no single registry edit
# can relicense every site at once.
echo "==> cdcp_gate doc-facts (B3 doc-truth: prose claims about code match the tree)"
run_cdcp_gate doc-facts || fail "prose claims about code disagree with the tree"
ok "every registered prose claim about code agrees with the artifact that answers it"


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

# Crosswalk: every domain the registry declares appears (bd-smvb / bd-lt7).
# Same source verify_coverage uses — knowledge/domains.toml — never a 01-14
# literal. Module 15 exists; a bound that cannot see it stays green by luck.
#
# declared_domain_ids FILE
#   Prints "COUNT id1 id2 ..." for unindented-or-indented `id = "..."` lines.
#   Missing file or zero ids is ERROR (vacuous crosswalk is not a pass).
declared_domain_ids() {
  _dd_file="$1"
  if [ ! -f "$_dd_file" ]; then
    echo "declared_domain_ids: not a file: $_dd_file" >&2
    return 2
  fi
  _dd_n=0
  _dd_ids=""
  while IFS= read -r _dd_line || [ -n "$_dd_line" ]; do
    _dd_trim="${_dd_line#"${_dd_line%%[![:space:]]*}"}"
    case "$_dd_trim" in
      id\ =\ \"*\")
        _dd_id="${_dd_trim#id = \"}"
        _dd_id="${_dd_id%%\"*}"
        [ -n "$_dd_id" ] || continue
        _dd_ids="$_dd_ids $_dd_id"
        _dd_n=$((_dd_n + 1))
        ;;
    esac
  done < "$_dd_file"
  if [ "$_dd_n" -eq 0 ]; then
    echo "declared_domain_ids: zero domain ids in $_dd_file — vacuous crosswalk is ERROR" >&2
    return 2
  fi
  echo "$_dd_n$_dd_ids"
  return 0
}

# crosswalk_covers_declared DOMAINS CROSSWALK
#   Every derived id must appear as `domain = "ID"` in the crosswalk.
crosswalk_covers_declared() {
  _cc_dom="$1"
  _cc_xw="$2"
  if [ ! -f "$_cc_xw" ]; then
    echo "crosswalk_covers_declared: not a file: $_cc_xw" >&2
    return 2
  fi
  _cc_got="$(declared_domain_ids "$_cc_dom")" || return 2
  _cc_n="${_cc_got%% *}"
  _cc_ids="${_cc_got#* }"
  [ -n "$_cc_ids" ] || {
    echo "crosswalk_covers_declared: no ids after count — vacuous crosswalk is ERROR" >&2
    return 2
  }
  for _cc_d in $_cc_ids; do
    if ! grep -q "domain = \"$_cc_d\"" "$_cc_xw"; then
      echo "crosswalk missing $_cc_d" >&2
      return 2
    fi
  done
  echo "$_cc_n"
  return 0
}

_xw_n="$(crosswalk_covers_declared knowledge/domains.toml knowledge/standards_crosswalk.toml)" \
  || fail "standards crosswalk missing a domain the registry declares"

# L4: these functions must trip. A parser that always succeeds would pass the
# live tree and never notice a missing row; the plants make a crash ≠ a pass.
_xw_plant="$(mktemp -d "${TMPDIR:-/tmp}/cdcp_xw_plant.XXXXXX")"
_xw_selftest_cleanup() { rm -rf "$_xw_plant"; }

# (a) empty registry → ERROR
: >"$_xw_plant/empty.toml"
if declared_domain_ids "$_xw_plant/empty.toml" >/dev/null; then
  _xw_selftest_cleanup
  fail "declared_domain_ids stayed GREEN on an empty domains file"
fi

# (b) a declared id missing from the crosswalk → RED
printf '%s\n' '[[domain]]' 'id = "99-missing"' 'order = 99' >"$_xw_plant/dom.toml"
printf '%s\n' '[[map]]' 'domain = "01-other"' >"$_xw_plant/xw.toml"
if crosswalk_covers_declared "$_xw_plant/dom.toml" "$_xw_plant/xw.toml" >/dev/null; then
  _xw_selftest_cleanup
  fail "crosswalk_covers_declared stayed GREEN when 99-missing was absent"
fi

# (c) green control: a formatted pair must pass (always-fail would spoof the plants)
printf '%s\n' '[[domain]]' 'id = "01-ok"' 'order = 1' >"$_xw_plant/ok_dom.toml"
printf '%s\n' 'domain = "01-ok"' >"$_xw_plant/ok_xw.toml"
if ! crosswalk_covers_declared "$_xw_plant/ok_dom.toml" "$_xw_plant/ok_xw.toml" >/dev/null; then
  _xw_selftest_cleanup
  fail "crosswalk_covers_declared went RED on a matching domains/crosswalk pair"
fi

_xw_selftest_cleanup
ok "standards crosswalk covers every domain the registry declares (n=$_xw_n)"

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
run_cdcp_gate verify-bank || fail "bank verify"
ok "bank pool"

echo "==> cdcp_gate answer-key-skew (approved answer-key position distribution)"
run_cdcp_gate answer-key-skew || fail "answer-key skew"
ok "answer-key distribution"


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
run_cdcp_gate validate-grounding || fail "grounding"
ok "grounding heuristics"

echo "==> cdcp_gate grounding-wave (template and recall-only stems)"
run_cdcp_gate grounding-wave || fail "grounding-wave"
ok "grounding-wave stem regression detector"

# Orphan referential integrity (topics <-> bank) — ORACLE-GAUNTLET "orphan item".
# Hard-required: an absent checker is a fooled certificate, not a skip.
[ -f scripts/verify_orphans.py ] || fail "missing scripts/verify_orphans.py (orphan gate required)"
echo "==> cdcp_gate verify-orphans (topic<->item referential integrity)"
run_cdcp_gate verify-orphans || fail "orphan referential integrity"
ok "no orphan topics · no orphan item refs · no unanchored items"

[ -f scripts/selftest_orphan.sh ] || fail "missing scripts/selftest_orphan.sh (L4 orphan known-bad required)"
echo "==> selftest_orphan.sh (L4 orphan known-bad)"
run_selftest "orphan known-bad selftest" sh scripts/selftest_orphan.sh
ok "orphan selftest (empty bank/topics ERROR · orphan ref RED · unanchored RED · orphan topic RED · live GREEN)"

# C3 near-duplicate items — no two assembly-eligible items may read as the same
# question twice (bd-near-duplicate-item-gate-i5v; the 25 pairs it found were
# adjudicated in bd-tetz). Jaccard over CORRECT-ANSWER token sets, not stems:
# the decisive pair m11-q226/m11-q139 is 100% answer overlap at 16% stem, which
# no stem-based detector could ever find. Anti-vacuous: zero items, zero
# approved, or fewer than 2 approved is an ERROR (exit 4).
[ -d bank/items ] || fail "missing bank/items (near-duplicate gate required)"
echo "==> cdcp_gate near-duplicate-items (C3 near-duplicates in the approved pool)"
run_cdcp_gate near-duplicate-items || fail "near-duplicate items in the approved pool"
ok "no cosmetic near-duplicates in the approved pool (NOT a distinct-proposition count)"

# L4: proven to TRIP, not merely to pass. Plants a cosmetically-reworded clone
# of a real approved item in memory and asserts it is flagged against its source.
echo "==> cdcp_gate near-duplicate-items selftest (L4 known-bad injection)"
CDCP_NEAR_DUPLICATE_SELFTEST=1 run_cdcp_gate near-duplicate-items \
  || fail "near-duplicate selftest did not reach RED on its planted clone"
ok "near-duplicate selftest (planted clone trips RED)"

# bd-e1yt / bd-substrate-rust-migration-jhd.21 — C3 is cosmetic only. The
# rust product (`cdcp verify-paraphrase-pairs` → cdcp_bank::paraphrase) is
# the honesty tripwire for the four measured pairs the Jaccard floor cannot
# see. It is NOT a second grader: it FAILs if a listed pair disappears
# without an adjudication reason, and it prints a stem-overlap REPORT of
# candidates. Empty ledger or a scan of zero items is ERROR. C3 is unchanged.
# EXTRACT-THEN-DELETE: python3 scripts/verify_paraphrase_pairs.py is retired
# (rust selftests replace --selftest). Putting that invoke back is RED.
[ -f registries/paraphrase_pairs.toml ] || fail "missing registries/paraphrase_pairs.toml (paraphrase ledger required)"
[ -d bank/items ] || fail "missing bank/items (paraphrase ledger required)"
echo "==> cdcp verify-paraphrase-pairs (measured paraphrase debt; pool size ≠ proposition count)"
run_cdcp_cli verify-paraphrase-pairs || fail "paraphrase pair ledger"
ok "paraphrase pair ledger intact (804/779 is a pool size; report is not a verdict)"

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
# SHAPE A (bd-u2x): keep the #[path] + glob registration. Shape B (a committed
# mod.rs cargo fmt can walk) reintroduces the six-way shared-file collision
# build.rs exists to prevent, and fights gate_shrink. The parallelism is worth
# keeping; the unchecked surface is not. A green fmt leg over files it never
# opened is the same vacuous-scan pattern this script hard-fails on elsewhere.
#
# #[path] / OUT_DIR tool reach, MEASURED 2026-08-15 (bd-u2x), not inferred:
#   cargo fmt                  BLIND  — plant in install_hooks.rs: rustfmt
#                                       --check rc=1; cargo fmt --check -p
#                                       cdcp_gate rc=0, zero mention of the
#                                       file. THIS function is the compensating
#                                       leg.
#   rustfmt --edition 2021     REACH  — filesystem glob, this function.
#   clippy -p cdcp_gate --lib  REACH  — unused-binding plant named
#                                       src/gates/install_hooks.rs:64 under
#                                       `-D warnings`. No compensating clippy
#                                       leg. (clippy compiles #[path].)
#   rustdoc / cargo test --doc REACH  — broken doctest plant FAILED as
#                                       gates::install_hooks::run. Doctests
#                                       run under `cargo test --workspace`
#                                       below. `cargo doc` HTML is NOT a
#                                       check.sh step — recorded: not required.
#   cargo-mutants              NOT in check.sh, not installed — not required.
#   cargo-tarpaulin / llvm-cov NOT in check.sh (tarpaulin is on PATH, unused)
#                                       — not required.
#   cargo-udeps                NOT in check.sh, not installed; udeps walks
#                                       the dep graph, not source modules —
#                                       not required.
#
# rustfmt_gate_modules DIR
#   rustfmt --check every *.rs in DIR. rustfmt missing from PATH is ERROR,
#   not a skip. Files cargo fmt can already see (mod.rs) are still checked
#   (harmless) but do NOT satisfy the anti-vacuous floor — a directory that
#   holds only mod.rs is the same defect as an empty glob.
rustfmt_gate_modules() {
  _rgm_dir="$1"
  if ! command -v rustfmt >/dev/null 2>&1; then
    echo "rustfmt_gate_modules: rustfmt not on PATH" >&2
    return 2
  fi
  if [ ! -d "$_rgm_dir" ]; then
    echo "rustfmt_gate_modules: not a directory: $_rgm_dir" >&2
    return 2
  fi
  # Unique L4 plant name. If it is in THIS scan dir, the selftest leaked.
  # Distinct from "rustfmt --check failed" so a leftover plant is not a
  # formatting defect. Isolation is the scan root, not a filename allowlist.
  if [ -e "$_rgm_dir/__cdcp_fmt_plant__.rs" ]; then
    echo "rustfmt_gate_modules: STALE PLANT: $_rgm_dir/__cdcp_fmt_plant__.rs is in the scan path; this is a selftest-cleanup failure, not a rustfmt defect" >&2
    return 2
  fi
  _rgm_n=0
  for _rgm_f in "$_rgm_dir"/*.rs; do
    [ -f "$_rgm_f" ] || continue
    rustfmt --edition 2021 --check "$_rgm_f" || {
      echo "rustfmt_gate_modules: rustfmt --check failed: $_rgm_f" >&2
      return 2
    }
    case "${_rgm_f##*/}" in
      mod.rs) ;;
      *) _rgm_n=$((_rgm_n + 1)) ;;
    esac
  done
  if [ "$_rgm_n" -eq 0 ]; then
    echo "rustfmt_gate_modules: scanned 0 #[path] gate files in $_rgm_dir — a vacuous scan is an ERROR, not a pass" >&2
    return 2
  fi
  echo "$_rgm_n"
  return 0
}

_gate_fmt_n="$(rustfmt_gate_modules crates/cdcp_gate/src/gates)" \
  || fail "rustfmt over crates/cdcp_gate/src/gates (cargo fmt cannot see these)"

# L4 meta-test of THIS function (same code path as the live scan).
# Three known-bads must RED; one formatted specimen must GREEN.
# A function that always fails would pass the plants; the green control
# is what makes a crash distinguishable from a pass.
#
# The fixtures live under target/, beside (not below) the production
# `crates/cdcp_gate/src/gates` root passed above. That separation is by the
# scan's root, not by a filename exclusion. A leftover root is a stale plant,
# not a reason to silently reuse or delete someone else's fixture.
_fmt_selftest_root="$ROOT/target/cdcp-gate-rustfmt-selftest"
if [ -e "$_fmt_selftest_root" ]; then
  fail "rustfmt_gate_modules: STALE PLANT: $_fmt_selftest_root remains from an interrupted selftest"
fi
mkdir -p "$_fmt_selftest_root" \
  || fail "rustfmt_gate_modules: cannot create selftest root $_fmt_selftest_root"
_fmt_plant="$_fmt_selftest_root/plant"
_fmt_empty="$_fmt_selftest_root/empty"
_fmt_modonly="$_fmt_selftest_root/modonly"
_fmt_green="$_fmt_selftest_root/green"
mkdir -p "$_fmt_plant" "$_fmt_empty" "$_fmt_modonly" "$_fmt_green" \
  || fail "rustfmt_gate_modules: cannot create isolated selftest fixtures"
_fmt_selftest_cleanup() {
  rm -rf "$_fmt_selftest_root"
}

# (a) planted space/indent in a gate-shaped file
printf '%s\n' 'fn plant( ){let     x=1 ;}' > "$_fmt_plant/plant.rs"
_fmt_plant_diag=""
if _fmt_plant_diag="$(rustfmt_gate_modules "$_fmt_plant" 2>&1 >/dev/null)"; then
  _fmt_selftest_cleanup
  fail "rustfmt_gate_modules stayed GREEN on a planted space/indent in a gate file"
fi
printf '%s\n' "$_fmt_plant_diag" | grep -F "$_fmt_plant/plant.rs" >/dev/null \
  || { _fmt_selftest_cleanup; fail "rustfmt_gate_modules did not report the planted file as RED"; }

# (b) empty glob
_fmt_empty_diag=""
if _fmt_empty_diag="$(rustfmt_gate_modules "$_fmt_empty" 2>&1 >/dev/null)"; then
  _fmt_selftest_cleanup
  fail "rustfmt_gate_modules stayed GREEN on an empty glob — vacuous scan must be ERROR"
fi
printf '%s\n' "$_fmt_empty_diag" | grep -F "scanned 0" >/dev/null \
  || { _fmt_selftest_cleanup; fail "rustfmt_gate_modules did not report the empty plant surface as ERROR"; }

# (c) only mod.rs (the one file cargo fmt can see) is still vacuous
printf '%s\n' '//! decoy' > "$_fmt_modonly/mod.rs"
rustfmt --edition 2021 "$_fmt_modonly/mod.rs"
_fmt_modonly_diag=""
if _fmt_modonly_diag="$(rustfmt_gate_modules "$_fmt_modonly" 2>&1 >/dev/null)"; then
  _fmt_selftest_cleanup
  fail "rustfmt_gate_modules stayed GREEN on only-mod.rs — that is the file cargo fmt already sees"
fi
printf '%s\n' "$_fmt_modonly_diag" | grep -F "scanned 0" >/dev/null \
  || { _fmt_selftest_cleanup; fail "rustfmt_gate_modules did not report only-mod.rs as a vacuous ERROR"; }

# (d) green control: a formatted non-mod.rs file must pass
printf '%s\n' 'pub fn plant() {}' > "$_fmt_green/ok.rs"
rustfmt --edition 2021 "$_fmt_green/ok.rs"
if ! rustfmt_gate_modules "$_fmt_green" >/dev/null 2>&1; then
  _fmt_selftest_cleanup
  fail "rustfmt_gate_modules went RED on a formatted gate-shaped file (always-fail would spoof the plants)"
fi

# (e) known-bad: leftover plant IN the production scan path. Must be
# STALE PLANT, not "rustfmt --check failed". cleanup() removes it on every
# exit; this block also removes it so a later cargo fmt cannot see it.
_fmt_stale_plant="crates/cdcp_gate/src/gates/__cdcp_fmt_plant__.rs"
printf '%s\n' 'fn plant( ){let     x=1 ;}' > "$_fmt_stale_plant"
_fmt_stale_diag=""
if _fmt_stale_diag="$(rustfmt_gate_modules crates/cdcp_gate/src/gates 2>&1 >/dev/null)"; then
  rm -f "$_fmt_stale_plant"
  fail "rustfmt_gate_modules stayed GREEN with a stale plant in crates/cdcp_gate/src/gates"
fi
printf '%s\n' "$_fmt_stale_diag" | grep -F "STALE PLANT" >/dev/null \
  || { rm -f "$_fmt_stale_plant"; fail "stale rustfmt plant must be named STALE PLANT, not a rustfmt --check failure"; }
printf '%s\n' "$_fmt_stale_diag" | grep -F "rustfmt --check failed" >/dev/null \
  && { rm -f "$_fmt_stale_plant"; fail "stale rustfmt plant was misreported as a rustfmt --check failure"; }
rm -f "$_fmt_stale_plant"

_fmt_selftest_cleanup
ok "rustfmt over $_gate_fmt_n #[path] gate module(s) cargo fmt cannot reach (L4 plant RED · empty glob ERROR · only-mod.rs ERROR · formatted GREEN · stale plant STALE PLANT)"
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
run_cdcp_cli goldens check --bank bank/items --dir goldens \
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

# L4 installer plants (bd-installability-sm4g.7). Not via run_selftest: SUITE=installer
# is not in REGISTERED_SUITES (adding it grows cdcp_gate past gate_shrink). The
# suite still emits INJECTIONS=4 SUITE=installer on its own success path.
[ -f scripts/selftest_install.sh ] || fail "missing scripts/selftest_install.sh (L4 installer known-bad required)"
echo "==> selftest_install.sh (L4 installer)"
sh scripts/selftest_install.sh || fail "installer known-bad"
ok "installer known-bad (tampered tarball · empty assets · missing checksum · D1)"

# L4 WASM: rebuild --release --locked, pin sha256 to the shipped blob, then
# dual-path against THAT blob (not target/.../debug). [bd-installability-sm4g.4]
# Missing wasm32: SKIP the rebuild, do NOT advertise full-green (GAPS + skip).
# Plants always run: they do not need the wasm32 target.
echo "==> L4 WASM dual-path + shipped-blob freshness"
L4_WASM="SKIP"
if command -v rustup >/dev/null 2>&1   && rustup target list --installed 2>/dev/null | grep -q '^wasm32-unknown-unknown$'
then
  # --include-ignored is load-bearing on dual_path: native_equals_wasm_mock40_seed42
  # is #[ignore] so `cargo test --workspace` cannot score a silent skip as PASS.
  # Without this flag L4 would go GREEN having compared nothing.
  cargo build -p cdcp_wasm --target wasm32-unknown-unknown --release --locked \
    || fail "L4 WASM release rebuild (cdcp_wasm wasm32-unknown-unknown --release --locked)"
  sh scripts/selftest_wasm_freshness.sh --assert-fresh \
    --committed web/assets/wasm/cdcp_wasm.wasm \
    --built target/wasm32-unknown-unknown/release/cdcp_wasm.wasm \
    || fail "L4 WASM freshness: web/assets/wasm/cdcp_wasm.wasm sha256 != release rebuild"
  if CDCP_REQUIRE_WASM=1 cargo test -p cdcp_wasm --test dual_path --locked -- --nocapture --include-ignored \
    && CDCP_REQUIRE_WASM=1 cargo test -p cdcp_wasm --test schedule --locked -- --nocapture
  then
    [ -x scripts/selftest_wasm_freshness.sh ] || [ -f scripts/selftest_wasm_freshness.sh ] \
      || fail "missing scripts/selftest_wasm_freshness.sh (L4 wasm-freshness required)"
    run_selftest "wasm-freshness known-bad" sh scripts/selftest_wasm_freshness.sh
    ok "L4 WASM dual-path native==shipped wasm (mock40_seed42 + schedule) + freshness"
    L4_WASM="GREEN"
  else
    fail "L4 WASM dual-path failed (toolchain present but digests disagree or test/build error)"
  fi
else
  echo "check.sh: SKIP wasm freshness rebuild: toolchain missing"
  echo "check.sh: L4 WASM is NOT full-green — install: rustup target add wasm32-unknown-unknown"
  GAPS="${GAPS}L4-wasm-freshness "
  [ -x scripts/selftest_wasm_freshness.sh ] || [ -f scripts/selftest_wasm_freshness.sh ] \
    || fail "missing scripts/selftest_wasm_freshness.sh (L4 wasm-freshness required)"
  run_selftest "wasm-freshness known-bad" sh scripts/selftest_wasm_freshness.sh
  L4_WASM="SKIP"
  skipped_step "L4 WASM freshness rebuild (wasm32 target missing — not full-green)"
fi

# Wave status
echo "check.sh: WAVE STATUS: W0+L1+L2+L3 GREEN; L4 known-bad WIRED; L4 WASM=$L4_WASM; L5 UI still OPEN"
if [ "$L4_WASM" = "GREEN" ]; then
  echo "check.sh: next: L5 browser mock path (UI e2e digest match)"
else
  echo "check.sh: L4 WASM=$L4_WASM is NOT full-green — enable wasm32 target, then L5 UI"
fi

# Knowledge primary_notes path resolution (parent ../modules/)
# The `if [ -f ... ]` guard this replaces was FAIL-OPEN. [bd-...-jhd.5]
echo "==> cdcp_gate verify-knowledge-paths (knowledge primary_notes resolve)"
run_cdcp_gate verify-knowledge-paths || fail "knowledge primary_notes paths"
ok "knowledge primary_notes paths"

# ─── L5 browser surface ─────────────────────────────────────────────────────
echo "==> L5 browser surface (require product files)"
for f in web/index.html web/learn.html web/drill.html web/mock.html web/reference.html; do
  [ -f "$f" ] || fail "L5 product file missing: $f"
done
ok "L5 product files present"
[ -f web/assets/wasm/cdcp_wasm.wasm ] || fail "L5 wasm artifact missing under web/assets/wasm/"
ok "L5 wasm artifact present under web/assets/wasm/"
echo "==> cdcp check-learner-pack (L5 learner pack shape)"
run_cdcp_cli check-learner-pack || fail "L5 learner pack shape"
ok "L5 learner pack n_items=40"

echo "==> selftest_l5.sh (honesty + e2e digest known-bad)"
run_selftest "L5 selftest" sh scripts/selftest_l5.sh
ok "L5 selftest (honesty plant RED · digest match · flipped golden RED · empty fixtures ERROR)"

echo "==> e2e_l5_digest.sh (UI dual-path digest match)"
sh scripts/e2e_l5_digest.sh || fail "L5 e2e digest"
ok "L5 e2e digest match (seed42 all-correct/all-wrong)"

# L4 generator freshness [bd-installability-sm4g.10]. Plants live in
# assert_generator_fresh.sh --selftest (not via run_selftest — same as N.7
# installer / N.16 learner verbs, so REGISTERED_SUITES does not grow).
[ -f scripts/assert_generator_fresh.sh ] || fail "missing scripts/assert_generator_fresh.sh (generator freshness required)"
echo "==> assert_generator_fresh.sh --selftest (L4: empty set ERROR · dirty owned path RED)"
sh scripts/assert_generator_fresh.sh --selftest || fail "generator freshness helper known-bad"

echo "==> cdcp build-learn (Learn surface)"
run_cdcp_cli build-learn || fail "build-learn"
assert_generator_fresh build-learn \
  web/learn.html \
  web/learn/ \
  web/data/modules_index.json \
  web/data/topic_anchors.json \
  web/content/modules/ \
  || fail "build-learn stale artifacts"
ok "Learn surface (modules_index · topic_anchors · pages · copies)"

echo "==> cdcp build-reference (Reference surface)"
run_cdcp_cli build-reference || fail "build-reference"
assert_generator_fresh build-reference \
  web/reference.html \
  web/content/reference/ \
  || fail "build-reference stale artifacts"
ok "Reference surface (reference.html · glossary · power cheatsheet)"

echo "==> cdcp smoke-learn (L5 learn surface)"
run_cdcp_cli smoke-learn || fail "L5 learn smoke"
ok "L5 learn smoke"

# ─── L6 mastery / coverage ──────────────────────────────────────────────────
echo "==> smoke_srs.mjs";        node scripts/smoke_srs.mjs        || fail "L6 review smoke";     ok "L6 short-interval review smoke"
echo "==> smoke_mastery.mjs";    node scripts/smoke_mastery.mjs    || fail "L6 mastery smoke";    ok "L6 mastery smoke"
echo "==> cdcp smoke-weak-links (L6-S3)"; run_cdcp_cli smoke-weak-links || fail "L6 weak-links smoke"; ok "L6 weak-links smoke"
echo "==> smoke_hub_mastery.mjs"; node scripts/smoke_hub_mastery.mjs || fail "L6-S4 hub mastery"; ok "L6 hub mastery + recommend smoke"
ok "L6-S4 hub mastery surface wired"

echo "==> L6 multi-seed export-web (fixture golden-stable)"
_MS_TMP="$(mktemp -d "${TMPDIR:-/tmp}/cdcp_multiseed.XXXXXX")"
run_cdcp_cli export-web --bank bank/items --seed 42 --out "$_MS_TMP" >/dev/null \
  || fail "L6 multi-seed export-web"
for f in mock40_seed42.json keys_seed42.json bank_items_seed42.json; do
  cmp -s "$_MS_TMP/$f" "web/data/$f" || fail "L6 export-web seed42 not golden-stable: $f"
done
rm -rf "$_MS_TMP"
# In-tree write + porcelain: same contract as the other generators. The temp
# cmp above proves generated == committed; this write is a no-op when they
# already match (write_bytes_if_changed) and RED via porcelain if they don't.
run_cdcp_cli export-web --bank bank/items --seed 42 --out web/data >/dev/null \
  || fail "export-web"
assert_generator_fresh export-web \
  web/data/mock40_seed42.json \
  web/data/keys_seed42.json \
  web/data/bank_items_seed42.json \
  || fail "export-web stale artifacts"
ok "L6 multi-seed export-web --seed 42 (fixture golden-stable)"

echo "==> L6 session shapes"
for _shape in "Drill due" "Miss review"; do
  grep -q "$_shape" web/drill.html || fail "L6 session shape missing from web/drill.html: $_shape"
done
ok "L6 session shapes (Drill due · Miss review) present"

[ -f scripts/verify_coverage.py ] || fail "missing scripts/verify_coverage.py (differential oracle for verify-coverage)"
echo "==> cdcp_gate verify-coverage (L6 domain coverage)"
run_cdcp_gate verify-coverage || fail "L6 coverage"
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

echo "==> cdcp smoke-learn-chrome (M8-A)"; run_cdcp_cli smoke-learn-chrome || fail "M8-A learn chrome"; ok "M8-A learn chrome smoke"
echo "==> cdcp build-units (M8-B units_index)"
run_cdcp_cli build-units || fail "M8-B units_index"
assert_generator_fresh build-units web/data/units_index.json \
  || fail "build-units stale artifacts"
ok "M8-B units_index"
echo "==> cdcp build-glossary (M8-D glossary)"
run_cdcp_cli build-glossary || fail "M8-D glossary"
assert_generator_fresh build-glossary web/data/glossary.json \
  || fail "build-glossary stale artifacts"
ok "M8-D glossary.json"
echo "==> cdcp build-learn-slugs (MODULE_LEARN_SLUGS)"
run_cdcp_cli build-learn-slugs || fail "MODULE_LEARN_SLUGS"
assert_generator_fresh build-learn-slugs web/data/module_learn_slugs.js \
  || fail "build-learn-slugs stale artifacts"
ok "MODULE_LEARN_SLUGS from domains.toml"
# After regenerate so this smoke sees THIS run's units_index, not last run's
# (bd-wire-smoke-quiz-approved-7pju.1).
echo "==> smoke_quiz_approved.mjs"; node scripts/smoke_quiz_approved.mjs || fail "approved-only quiz/units draw"; ok "no learner surface draws a non-approved item"
echo "==> cdcp smoke-learn-v2 (M8-B/D)"; run_cdcp_cli smoke-learn-v2 || fail "M8-B/D learn v2";  ok "M8-B/D learn v2 smoke"
echo "==> cdcp smoke-diagrams (M8-C)";   run_cdcp_cli smoke-diagrams || fail "M8-C diagrams"; ok "M8-C diagrams smoke"
echo "==> cdcp smoke-a11y (L7-S5)";      run_cdcp_cli smoke-a11y || fail "L7 a11y";          ok "L7 a11y baseline"
echo "==> cdcp smoke-feedback-links (L7-S2)"; run_cdcp_cli smoke-feedback-links || fail "L7 feedback links"; ok "L7-S2 feedback section-anchor links smoke"
ok "L7 feedback section links"

echo "==> L7 CLI product verbs"
# Authoring verbs live behind CDCP_DEV=1 (bd-installability-sm4g.6).
_HELP="$(CDCP_DEV=1 run_cdcp_cli --help 2>&1)"
for v in bank-hash grade goldens export-web serve build-learn build-reference build-units build-glossary build-learn-slugs smoke-learn smoke-learn-chrome smoke-feedback-links smoke-diagrams smoke-a11y smoke-weak-links smoke-learn-v2 export-anki verify-paraphrase-pairs check-licence load-snapshots check-osha verify-data-lock check-learner-pack corpus-rights; do
  printf '%s' "$_HELP" | grep -q -- "$v" || fail "L7 CLI verb missing from CDCP_DEV=1 --help: $v"
done
_LEARNER_HELP="$(env -u CDCP_DEV run_cdcp_cli --help 2>&1)"
for v in study doctor demo test repair; do
  printf '%s' "$_LEARNER_HELP" | grep -q -- "$v" || fail "learner --help missing $v"
done
for v in bank-hash build-learn goldens export-web serve; do
  printf '%s\n' "$_LEARNER_HELP" | grep -E "^  ${v}( |$)" >/dev/null \
    && fail "learner --help still lists authoring verb $v (hide is not wired)"
done
ok "L7 CLI product verbs listed"

# ── learner verbs wired (bd-installability-sm4g.16) ────────────────────────
# BUILT != WIRED: study / demo / test shipped and were never invoked here.
# Each step fail-closes on the verb's exit code. A line that runs the verb
# and ignores the status is the vacuous pass build-learn used to be before
# .10 asserted generator freshness. Plants live in selftest_learner_verbs.sh
# (not via run_selftest — same as N.7 installer, so REGISTERED_SUITES does
# not grow).
echo "==> cdcp test (installed-tree smoke)"
run_cdcp_cli test || fail "cdcp test"
ok "cdcp test"

echo "==> cdcp demo --no-open"
run_cdcp_cli demo --no-open || fail "cdcp demo"
ok "cdcp demo --no-open"

echo "==> cdcp study (bind + HTTP 200 + stop)"
command -v curl >/dev/null 2>&1 || fail "cdcp study: curl is required to prove the listener (printed-URL-only is vacuous)"
_study_log=$(mktemp "${TMPDIR:-/tmp}/cdcp-study-gate.XXXXXX")
# N.17: do not `run_cdcp_cli study … &`. $! of a function is the subshell;
# kill of that subshell reparents cdcp to PID 1 and the listener stays up
# (POST_KILL_HTTP=200, LEAKED=1). Invoke the binary so $! is the cdcp PID.
# Stop is proven by a second curl, not by kill returning 0.
require_cdcp_bins
set +e
"$CDCP_BIN_DIR/cdcp" study --no-open --bind 127.0.0.1:0 >"$_study_log" 2>&1 &
_study_pid=$!
set -e
_study_url=""
_study_i=0
while [ "$_study_i" -lt 50 ]; do
  if ! kill -0 "$_study_pid" 2>/dev/null; then
    wait "$_study_pid" || true
    cat "$_study_log" >&2
    rm -f "$_study_log"
    fail "cdcp study"
  fi
  _study_url=$(sed -n 's/.*cdcp study: \(http:\/\/[^[:space:]]*\).*/\1/p' "$_study_log" | awk 'NR==1{print; exit}')
  if [ -n "$_study_url" ]; then
    break
  fi
  _study_i=$((_study_i + 1))
  sleep 0.2
done
if [ -z "$_study_url" ]; then
  kill "$_study_pid" 2>/dev/null || true
  wait "$_study_pid" 2>/dev/null || true
  cat "$_study_log" >&2
  rm -f "$_study_log"
  fail "cdcp study"
fi
_study_code=$(curl -fsS -o /dev/null -w "%{http_code}" "$_study_url" || true)
kill "$_study_pid" 2>/dev/null || true
wait "$_study_pid" 2>/dev/null || true
# Reap is not proven by kill(2) of $!. The bound URL must die.
_study_post=$(curl -fsS -o /dev/null -w "%{http_code}" --connect-timeout 1 --max-time 1 "$_study_url" || true)
if [ "$_study_post" = "200" ]; then
  _study_port=$(printf '%s' "$_study_url" | sed -n 's|.*:\([0-9][0-9]*\)/.*|\1|p')
  # Emergency reap of OUR ephemeral port only. Never touch 8766 (occupied
  # `cdcp serve` is not this step's process).
  if [ -n "$_study_port" ] && [ "$_study_port" != "8766" ]; then
    _study_leaked=$(lsof -nP -iTCP:"$_study_port" -sTCP:LISTEN -t 2>/dev/null || true)
    for _study_lp in $_study_leaked; do
      kill "$_study_lp" 2>/dev/null || true
    done
  fi
  rm -f "$_study_log"
  fail "cdcp study"
fi
rm -f "$_study_log"
if [ "$_study_code" != "200" ]; then
  fail "cdcp study"
fi
ok "cdcp study served HTTP 200"

[ -f scripts/selftest_learner_verbs.sh ] || fail "missing scripts/selftest_learner_verbs.sh (L4 learner-verb known-bad required)"
echo "==> selftest_learner_verbs.sh (L4 study/demo/test known-bad)"
export CDCP_BIN_DIR
sh scripts/selftest_learner_verbs.sh || fail "learner verbs known-bad"
ok "learner verbs known-bad (test wasm · demo/study missing-bundle · ignore-exit is RED · study stop reaps cdcp)"

[ -f scripts/verify_objectives.py ] || fail "missing scripts/verify_objectives.py (differential oracle for verify-objectives)"
echo "==> cdcp_gate verify-objectives (L7-S7 objective coverage)"
run_cdcp_gate verify-objectives || fail "L7 objective coverage"
ok "L7 objective coverage (registry objectives resolve · every declared module carries items)"
echo "==> selftest_l7_objectives.sh"
run_selftest "L7 objectives selftest" sh scripts/selftest_l7_objectives.sh
ok "L7 objectives known-bad selftest"

echo "==> smoke_slo.sh"
if run_cdcp_cli export-web --help >/dev/null 2>&1; then
  sh scripts/smoke_slo.sh || fail "L7 SLO budgets"
  ok "L7 SLO budgets"
else
  echo "check.sh: GAP: L7 SLO budgets NOT RUN — cdcp_cli lacks the 'export-web' verb" >&2
  GAPS="${GAPS}L7-SLO "
  skipped_step "L7 SLO budgets (export-web verb absent)"
fi

echo "==> cdcp_gate verify-content-lock (L7 content.lock)"
run_cdcp_gate verify-content-lock || fail "L7 content.lock"
ok "L7 content.lock"

# L4: the content-lock gate must be proven to TRIP, not merely to pass. This
# selftest already existed on both sides, fully implemented and fully unwired —
# nothing in check.sh ran it. BUILT != WIRED, found 2026-08-14. It flips the
# pinned bank_hash in a TEMP copy (the committed content.lock is never touched)
# and asserts the RED path is reached.
echo "==> cdcp_gate verify-content-lock selftest (L4 content.lock known-bad)"
CDCP_CONTENT_LOCK_SELFTEST=1 run_cdcp_gate verify-content-lock \
  || fail "L7 content.lock mutate-selftest did not reach RED"
ok "L7 content.lock selftest (mutated bank_hash trips RED)"

# ─── V11 stretch surfaces ───────────────────────────────────────────────────
if [ -f scripts/selftest_reconstructed.sh ] && [ "${CDCP_IN_SELFTEST:-0}" != "1" ]; then
  echo "==> selftest_reconstructed.sh (L5–V11 reconstructed stages)"
  run_selftest "reconstructed-stage selftests" env CDCP_IN_SELFTEST=1 CDCP_BIN_DIR="$CDCP_BIN_DIR" sh scripts/selftest_reconstructed.sh
  ok "L5–V11 reconstructed stages proven to trip RED"
else
  skipped_step "L5–V11 reconstructed stages (nested run or missing script)"
fi

if [ -f tests/voice-slop.sh ]; then
  echo "==> tests/voice-slop.sh (applicable slice of the ZS voice gate)"
  sh tests/voice-slop.sh >/dev/null || fail "voice slop / honesty in public copy"
  ok "public copy free of marketing slop; honesty note intact"
else
  skipped_step "voice-slop (script absent)"
fi

# Roadmap doc truth — the prose a stranger reads first must not contradict
# itself. Hard-required: an absent checker is a fooled certificate, not a skip.
[ -f scripts/verify_doc_consistency.py ] || fail "missing scripts/verify_doc_consistency.py (roadmap-truth gate required)"
echo "==> cdcp_gate verify-doc-consistency (CHARTER §9 · README roadmap · PHASE-NEXT)"
run_cdcp_gate verify-doc-consistency || fail "roadmap doc consistency"
ok "roadmap milestone status agrees across docs; publication truth holds"

[ -f scripts/selftest_doc_consistency.sh ] || fail "missing scripts/selftest_doc_consistency.sh (L4 roadmap known-bad required)"
echo "==> selftest_doc_consistency.sh (L4 roadmap known-bad)"
run_selftest "roadmap doc consistency selftest" sh scripts/selftest_doc_consistency.sh
ok "roadmap selftest (dup row RED · cross-doc conflict RED · unreadable status RED · pending-publication RED · zero markdown ERROR)"

if [ -f tests/publishability-bar.sh ]; then
  echo "==> tests/publishability-bar.sh (L88 — audit claims must be true)"
  sh tests/publishability-bar.sh >/dev/null || fail "publishability bar (audit claim is false)"
  ok "L88 publishability bar (audit claims verified against the repo)"
else
  skipped_step "L88 publishability bar (script absent)"
fi

echo "==> V11 Anki planted all-retired (must print FAIL: and write no deck)"
_anki_plant=$(mktemp -d)
mkdir -p "$_anki_plant/bank/items"
printf '%s\n' \
  'id = "r1"' 'status = "retired"' 'module = 1' \
  'stem = "retired-only-planted-stem"' \
  'choices = ["a"]' 'correct = "A"' \
  > "$_anki_plant/bank/items/r1.toml"
set +e
_anki_plant_out=$(run_cdcp_cli export-anki --root "$_anki_plant" --format tsv --out "$_anki_plant/dist/anki" 2>&1)
_anki_plant_rc=$?
set -e
printf '%s\n' "$_anki_plant_out"
if [ "$_anki_plant_rc" -eq 0 ]; then
  rm -rf "$_anki_plant"
  fail "V11 Anki planted all-retired unexpectedly GREEN"
fi
printf '%s' "$_anki_plant_out" | grep -q "FAIL: zero approved items to export" \
  || { rm -rf "$_anki_plant"; fail "V11 Anki planted all-retired missing FAIL:"; }
if [ -e "$_anki_plant/dist/anki/cdcp_bank.tsv" ] || [ -e "$_anki_plant/dist/anki/cdcp_bank.apkg" ]; then
  rm -rf "$_anki_plant"
  fail "V11 Anki planted all-retired wrote a retired deck"
fi
rm -rf "$_anki_plant"
ok "V11 Anki planted all-retired is RED and writes nothing"

echo "==> cdcp export-anki (V11 · learner .apkg, pinned clock, approved-only)"
run_cdcp_cli export-anki --format tsv,csv,apkg --out dist/anki || fail "V11 Anki export"
ok "V11 Anki export tsv/csv/apkg (879 approved, pinned crt)"
echo "==> cdcp export-anki --check (planted clock leak RED, two-run identity GREEN)"
run_cdcp_cli export-anki --check || fail "V11 Anki .apkg not byte-reproducible"
ok "V11 Anki .apkg deck"
grep -q "study aid" web/reference.html 2>/dev/null || grep -rq "not.*certif" web/ 2>/dev/null || fail "V11 diagram honesty"
ok "V11 diagram honesty present"
if run_cdcp_cli serve --help >/dev/null 2>&1; then
  ok "V11 serve subcommand present"
else
  echo "check.sh: GAP: V11 serve subcommand ABSENT from cdcp_cli source" >&2
  GAPS="${GAPS}V11-serve "
  skipped_step "V11 serve subcommand (absent)"
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
  if [ "${CDCP_INJECTION_COUNT_WRITE_README:-0}" = "1" ]; then
    run_cdcp_gate verify-injection-count --log "$INJ_LOG" --write-readme \
      || fail "known-bad injection count drift (README vs suites)"
  else
    run_cdcp_gate verify-injection-count --log "$INJ_LOG" \
      || fail "known-bad injection count drift (README vs suites); re-run with CDCP_INJECTION_COUNT_WRITE_README=1 to regenerate"
  fi
  ok "advertised known-bad injection count == suites' self-reported total"

  # Content-count regenerator [bd-readme-public-rigor-8y0r.1].
  # Thin: one verb. Assertions live in `cdcp_cli` docs_sync tests.
  # Default --check. CDCP_DOCS_SYNC_WRITE=1 reaches --write (refuses unsound).
  echo "==> cdcp docs sync --check (advertised content counts vs units_index)"
  if [ "${CDCP_DOCS_SYNC_WRITE:-0}" = "1" ]; then
    run_cdcp_cli docs sync --write \
      || fail "docs-sync --write refused (ledger unsound, decoration, or ratchet)"
  else
    run_cdcp_cli docs sync --check \
      || fail "advertised content count drift (README/CHARTER vs units_index + WASM); re-run with CDCP_DOCS_SYNC_WRITE=1 to regenerate"
  fi
  ok "advertised content counts == units_index + measured WASM KiB"

  # STEP-COUNT-RECEIPT-BOUNDARY
  # Sealed: no `ok` call site may appear below this marker. verify-step-count
  # reads the marker and fails if one does. The receipt is written by THIS
  # process so a nested child's `check.sh: ok:` lines cannot enter the count.
  # [bd-1sd.13]
  [ -n "$STEP_LOG" ] || fail "step receipt log was never created"
  [ -f crates/cdcp_gate/src/gates/verify_step_count.rs ] \
    || fail "missing crates/cdcp_gate/src/gates/verify_step_count.rs (step-count drift guard required)"
  _nested_ok=0
  _probe_log="$ROOT/target/cdcp-substrate-probe/check_sh.log"
  if [ -f "$_probe_log" ]; then
    _nested_ok="$(grep -c 'check.sh: ok:' "$_probe_log" || true)"
  fi
  _step_total=$((STEP_OK + STEP_SKIPPED))
  _step_receipt="CHECK_STEPS=${_step_total} OK=${STEP_OK} SKIPPED=${STEP_SKIPPED} NESTED_OK=${_nested_ok} DEPTH=0 RUN=pid$$"
  printf '%s\n' "$_step_receipt" >"$STEP_LOG"
  printf '%s\n' "$_step_receipt"
  echo "==> cdcp_gate verify-step-count (advertised check.sh step count)"
  if [ "${CDCP_STEP_COUNT_WRITE_README:-0}" = "1" ]; then
    run_cdcp_gate verify-step-count --log "$STEP_LOG" --write-readme \
      || fail "advertised check.sh step count drift (README vs this run)"
  else
    run_cdcp_gate verify-step-count --log "$STEP_LOG" \
      || fail "advertised check.sh step count drift (README vs this run)"
  fi
fi

if [ -n "$GAPS" ]; then
  echo "check.sh: KNOWN GAPS (not green, not silent): $GAPS" >&2
fi
echo "check.sh: complete != EPI certified (study signal / mastery only)"
if [ "$L4_WASM" = "GREEN" ]; then
  echo "==> check.sh PASSED (W0-L7 + V11 stretch; L4 WASM=GREEN)"
else
  echo "==> check.sh PASSED with SKIPPED legs (L4 WASM=$L4_WASM — not full-green)"
fi
exit 0
