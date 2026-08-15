#!/usr/bin/env sh
# selftest_reconstructed.sh — L4 gates-proven-to-trip for the L5→V11 stages
# reconstructed in check.sh (2026-08-12), plus the reimplemented CLI verbs.
#
# Contract (mirrors scripts/selftest_known_bad.sh):
#   for each stage: inject a known-bad, assert check.sh goes RED, restore.
#   Never leave the tree dirty. An injection that stays GREEN is a FAILURE —
#   a gate that cannot fail is not a gate.
#
# PRIVATE TREE [bd-791t]
#   Mutations run against an isolated snapshot under
#   $LIVE/target/cdcp-recon-<pid>/snap/course-engine — NEVER against the live
#   crate. gl4j's check.sh lock does not serialise a sibling `cargo build`;
#   injecting crates/cdcp_cli/src/main.rs (or the S0/C1/C2 CHARTER sources)
#   into the live worktree lets that sibling compile injected source. The
#   live-tree proof is not worth that.
#   The snapshot is git archive HEAD (not the live desk) into its own repo
#   so install-hooks cannot touch the live common-dir hooks, with
#   objects/info/alternates pointed at the live object store so
#   capability-maturity can still name commit evidence. Mid-wave dirty
#   goldens / an in-flight check.sh are not imported.
#   Coverage lost: we no longer inject into the live working tree, and we
#   no longer re-enter the full check.sh (a HEAD snapshot is mid-wave RED
#   at registry-check / clippy / goldens before L5). Each case runs the
#   SAME predicate check.sh uses on that stage. CHARTER 6/6 is unchanged.
#
# RESTORE (bd-stale-binary-mtime-trap-p65w): case (e) perturbs
# crates/cdcp_cli/src/main.rs; the S0/C1/C2 CHARTER pairs perturb
# substrate_guard.rs, s0_charter_pair.rs, cdcp_assemble/src/lib.rs,
# c1_charter_pair.rs, cdcp_bank/src/lib.rs, and c2_charter_pair.rs.
# Those writes land in the private snapshot. Restore MUST still go through
# scripts/restore_safe.inc.sh — `mv backup dest` would hand the copy the
# backup's older mtime, cargo would skip, and the next CHARTER leg would
# test the PERTURBED binary. A put-back that does not restore bytes is
# ERROR, not a pass.
#
# Each case runs the same predicate check.sh uses for that stage. Re-entering
# the full check.sh from a HEAD snapshot is mid-wave RED before L5 (measured
# 2026-08-15: registry-check, clippy doc lints, goldens-couplings) and would
# make every assert_red vacuous. The predicates are the proof.
set -eu

LIVE_ROOT="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$LIVE_ROOT"
ROOT="$LIVE_ROOT"
PRIVATE_BASE=""
GIT_TOP=""

fail() { echo "selftest_reconstructed: FAIL: $*" >&2; exit 1; }
ok()   { echo "selftest_reconstructed: ok: $*"; }

# -- L4 drift guard: self-reported RED-injection count ----------------------
# INJ counts the injections this run actually asserted RED (green controls are
# NOT counted). Emitted once, on the success path only, as a machine-readable
# line that scripts/verify_injection_count.py aggregates. A suite that stops
# emitting the line is an ERROR to that gate, never a silent zero.
INJ=0
SUITE_NAME="selftest_reconstructed"
inject_counted() { INJ=$((INJ + 1)); }

. "$LIVE_ROOT/scripts/restore_safe.inc.sh"
# Wired prove: naive-mv leaves the backup mtime (RED), helper does not,
# and the scan of converted cargo-touching restore sites is non-vacuous.
# Cargo skip-vs-rebuild is the helper's own executable selftest — not run
# here, so this suite does not take the workspace cargo lock.
cdcp_restore_safe_mtime_demo || fail "restore_safe mtime demo"
cdcp_restore_safe_scan "$LIVE_ROOT" || fail "restore_safe converted-site scan"

BAK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/cdcp_recon.XXXXXX")"
RESTORE_LIST=""
COLD_CARGO_S=""

# The four files this bead named. CHARTER sources are also copied, but
# siblings edit those live; policing them would fail the suite on someone
# else's staged hunk. main.rs is the cargo-poison case.
WATCH="\
web/data/mock40_seed42.json \
web/data/keys_seed42.json \
web/drill.html \
crates/cdcp_cli/src/main.rs"

snapshot_live() {
  # shellcheck disable=SC2086
  python3 -c '
import hashlib, os, subprocess, sys
root, out, clean_out = sys.argv[1], sys.argv[2], sys.argv[3]
rels = sys.argv[4:]
missing = [r for r in rels if not os.path.isfile(os.path.join(root, r))]
if missing:
    sys.stderr.write("live snapshot missing: %s\n" % " ".join(missing))
    sys.exit(2)
top = subprocess.check_output(["git", "-C", root, "rev-parse", "--show-toplevel"], text=True).strip()
prefix = os.path.relpath(root, top)
if prefix == ".":
    prefix = ""
with open(out, "w") as fh, open(clean_out, "w") as cfh:
    for rel in rels:
        p = os.path.join(root, rel)
        digest = hashlib.sha256(open(p, "rb").read()).hexdigest()
        fh.write("%s %s\n" % (digest, rel))
        repo_rel = os.path.join(prefix, rel) if prefix else rel
        st = subprocess.check_output(
            ["git", "-C", top, "status", "--porcelain", "--", repo_rel], text=True
        )
        if not st.strip():
            cfh.write(rel + "\n")
' "$LIVE_ROOT" "$BAK_DIR/live.fp" "$BAK_DIR/live.clean" $WATCH
}

assert_live_unmoved() {
  _label="$1"
  python3 -c '
import hashlib, os, sys
root, fp, label = sys.argv[1], sys.argv[2], sys.argv[3]
bad = 0
for line in open(fp):
    digest, rel = line.split(None, 1)
    rel = rel.strip()
    p = os.path.join(root, rel)
    if not os.path.isfile(p):
        sys.stderr.write("live %s vanished during %s\n" % (rel, label))
        bad += 1
        continue
    now = hashlib.sha256(open(p, "rb").read()).hexdigest()
    if now != digest:
        sys.stderr.write("live %s bytes moved during %s\n" % (rel, label))
        bad += 1
sys.exit(1 if bad else 0)
' "$LIVE_ROOT" "$BAK_DIR/live.fp" "$_label" \
    || fail "ANTI-VACUOUS: live tracked file moved ($_label) — isolation broken"
}

assert_live_git_unmoved() {
  # Porcelain of the whole watched set is not a lock: a sibling staging an
  # already-dirty file (` M` → `M `) is not our mutation. Bytes are the
  # isolation property (assert_live_unmoved). For files that started CLEAN,
  # porcelain must stay empty — that is the "git status is clean" proof.
  _label="$1"
  python3 -c '
import os, subprocess, sys
live, clean_path, label = sys.argv[1], sys.argv[2], sys.argv[3]
started = set(x.strip() for x in open(clean_path) if x.strip())
top = subprocess.check_output(["git", "-C", live, "rev-parse", "--show-toplevel"], text=True).strip()
prefix = os.path.relpath(live, top)
if prefix == ".":
    prefix = ""
bad = 0
for rel in started:
    repo_rel = os.path.join(prefix, rel) if prefix else rel
    out = subprocess.check_output(
        ["git", "-C", top, "status", "--porcelain", "--", repo_rel], text=True
    )
    if out.strip():
        sys.stderr.write("live %s was clean and is now dirty (%s): %r\n" % (rel, label, out))
        bad += 1
sys.exit(1 if bad else 0)
' "$LIVE_ROOT" "$BAK_DIR/live.clean" "$_label" \
    || fail "ANTI-VACUOUS: a live file that started clean is now dirty ($_label)"
}

safe_rm_private() {
  [ -n "${PRIVATE_BASE:-}" ] || return 0
  case "$PRIVATE_BASE" in
    "$LIVE_ROOT/target/cdcp-recon-"*) ;;
    *) echo "selftest_reconstructed: refusing to rm unexpected PRIVATE_BASE=$PRIVATE_BASE" >&2; return 0 ;;
  esac
  [ -d "$PRIVATE_BASE" ] && rm -rf "$PRIVATE_BASE"
}

restore_all() {
  # Restore only inside the private snapshot. Never write into LIVE_ROOT —
  # a cwd drift plus a relative restore is the poison this bead closes.
  if [ -n "${ROOT:-}" ] && [ "$ROOT" != "$LIVE_ROOT" ] && [ -d "$ROOT" ]; then
    for rel in $RESTORE_LIST; do
      enc="$(printf '%s' "$rel" | tr '/' '_')"
      if [ -f "$BAK_DIR/$enc" ]; then
        case "$ROOT/$rel" in
          "$PRIVATE_BASE"/*) cdcp_restore_safe "$ROOT/$rel" "$BAK_DIR/$enc" || true ;;
          *) echo "selftest_reconstructed: refusing restore of $ROOT/$rel (not under private tree)" >&2 ;;
        esac
      fi
    done
  fi
  rm -rf "$BAK_DIR"
  safe_rm_private
}
trap restore_all EXIT INT TERM

assert_private_path() {
  _p="$1"
  [ -n "${PRIVATE_BASE:-}" ] || fail "private tree is not materialised"
  case "$_p" in
    "$PRIVATE_BASE"/*) ;;
    *) fail "refusing to mutate $_p — not under private tree $PRIVATE_BASE" ;;
  esac
  _rel="${_p#"$ROOT"/}"
  if [ -n "$_rel" ] && [ -f "$LIVE_ROOT/$_rel" ] && [ -f "$_p" ]; then
    _same="$(python3 -c 'import os,sys
try:
    print(int(os.path.samefile(sys.argv[1], sys.argv[2])))
except OSError:
    print(0)
' "$_p" "$LIVE_ROOT/$_rel")"
    [ "$_same" != "1" ] || fail "ANTI-VACUOUS: $_p is the same inode as the live file"
  fi
}

stash() {
  assert_private_path "$ROOT/$1"
  enc="$(printf '%s' "$1" | tr '/' '_')"
  cp "$ROOT/$1" "$BAK_DIR/$enc"
  RESTORE_LIST="$RESTORE_LIST $1"
}

unstash() {
  assert_private_path "$ROOT/$1"
  enc="$(printf '%s' "$1" | tr '/' '_')"
  [ -f "$BAK_DIR/$enc" ] || fail "no backup for $1"
  cdcp_restore_safe "$ROOT/$1" "$BAK_DIR/$enc" || fail "restore_safe failed for $1"
  cmp -s "$ROOT/$1" "$BAK_DIR/$enc" || fail "ANTI-VACUOUS: put-back of $1 did not restore bytes"
}

materialise_private_tree() {
  GIT_TOP="$(git -C "$LIVE_ROOT" rev-parse --show-toplevel)" \
    || fail "live engine is not inside a git work tree"
  PRIVATE_BASE="$LIVE_ROOT/target/cdcp-recon-$$"
  SNAP="$PRIVATE_BASE/snap"
  mkdir -p "$SNAP" || fail "cannot create $SNAP"
  # HEAD, not the live worktree. Mid-wave the desk has uncommitted
  # goldens/registry drift and an in-flight check.sh (bd-o4bc). Copying
  # those bytes made the unmutated snapshot RED and vacated every injection.
  _copy_report="$(
    python3 -c '
import os, subprocess, sys, time, tarfile
from io import BytesIO
top, snap = sys.argv[1], sys.argv[2]
t0 = time.time()
blob = subprocess.check_output(["git", "-C", top, "archive", "--format=tar", "HEAD"])
n = 0
with tarfile.open(fileobj=BytesIO(blob), mode="r:") as tf:
    for m in tf.getmembers():
        if m.isfile():
            n += 1
    tf.extractall(snap)
dt = time.time() - t0
if n < 1:
    sys.stderr.write("git archive HEAD was empty\n")
    sys.exit(2)
sys.stdout.write("PRIVATE_TREE_COPY_S=%.2f FILES=%d REV=HEAD\n" % (dt, n))
' "$GIT_TOP" "$SNAP"
  )" || fail "git archive HEAD into private snapshot failed"
  printf '%s\n' "$_copy_report"
  # Belt: even if archive were somehow the index, the orchestrator is HEAD.
  git -C "$GIT_TOP" show HEAD:course-engine/scripts/check.sh \
    >"$SNAP/course-engine/scripts/check.sh" \
    || fail "cannot pin HEAD scripts/check.sh into the private tree"
  # Isolated git repo so install-hooks writes to THIS common-dir, not the
  # live clone's. --show-prefix stays course-engine/ (same layout as live).
  git -C "$SNAP" init -q -b main || fail "git init of private snapshot"
  git -C "$SNAP" -c core.hooksPath=/dev/null add -A -f -- . || fail "git add private snapshot"
  git -C "$SNAP" -c core.hooksPath=/dev/null \
    -c user.email=cdcp-recon@test -c user.name=cdcp-recon \
    commit -qm "cdcp-recon private snapshot" || fail "git commit private snapshot"
  # Alternates: capability-maturity names historical SHAs (e.g. 467b429).
  # A fresh repo without this made every assert_red vacuous (already RED).
  _live_objects="$(git -C "$GIT_TOP" rev-parse --path-format=absolute --git-path objects)" \
    || fail "cannot resolve live git objects"
  _snap_git="$(git -C "$SNAP" rev-parse --absolute-git-dir)" \
    || fail "cannot resolve snapshot git dir"
  mkdir -p "$_snap_git/objects/info"
  printf '%s\n' "$_live_objects" >"$_snap_git/objects/info/alternates"
  git -C "$SNAP" rev-parse --verify --quiet HEAD >/dev/null \
    || fail "private snapshot has no HEAD"
  ROOT="$SNAP/course-engine"
  [ -f "$ROOT/scripts/check.sh" ] || fail "private tree missing scripts/check.sh"
  [ -d "$ROOT/../modules" ] || fail "private tree missing ../modules (parent corpus)"
  [ "$ROOT" != "$LIVE_ROOT" ] || fail "private ROOT equals LIVE_ROOT"
  # gitignored live dirs that HEAD archive omits but gates still name.
  # doc-facts [[exclude]] course-engine/beads_compliance_audit/ is ERROR
  # when the path matches nothing; the live tree carries the directory.
  if [ -d "$LIVE_ROOT/beads_compliance_audit" ]; then
    mkdir -p "$ROOT/beads_compliance_audit"
    # No nested .git: a second repo here would confuse vcs lookups.
    rsync -a --exclude '.git' "$LIVE_ROOT/beads_compliance_audit/" \
      "$ROOT/beads_compliance_audit/" \
      || fail "cannot copy beads_compliance_audit into the private tree"
  fi
  _same="$(python3 -c 'import os,sys
try:
    print(int(os.path.samefile(sys.argv[1], sys.argv[2])))
except OSError:
    print(0)
' "$ROOT/crates/cdcp_cli/src/main.rs" "$LIVE_ROOT/crates/cdcp_cli/src/main.rs")"
  [ "$_same" != "1" ] || fail "ANTI-VACUOUS: private main.rs is the live inode"
  mkdir -p "$ROOT/target"
  unset CARGO_BUILD_TARGET CARGO_BUILD_TARGET_DIR || true
  unset CARGO_ENCODED_RUSTFLAGS RUSTFLAGS RUSTDOCFLAGS || true
  unset RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER CARGO_MAKEFLAGS || true
  CARGO_TARGET_DIR="$ROOT/target"
  export CARGO_TARGET_DIR
  case "$CARGO_TARGET_DIR" in
    "$PRIVATE_BASE"/*) ;;
    *) fail "CARGO_TARGET_DIR $CARGO_TARGET_DIR is not under the private tree" ;;
  esac
  cd "$ROOT" || fail "cd private ROOT"
  # Mid-wave HEAD can be rustfmt-dirty under the current nightly. Nested
  # check.sh must reach the L5–L7 stages; fmt of the snapshot is not the
  # injection. CHARTER needles were checked to survive rustfmt.
  echo "selftest_reconstructed: DECISION=private-copy [bd-791t] live tracked files are not mutated"
  echo "selftest_reconstructed: private tree $ROOT"
  echo "selftest_reconstructed: CARGO_TARGET_DIR=$CARGO_TARGET_DIR"
}

prove_sigkill_leaves_live_clean() {
  # ACCEPTANCE: kill the suite mid-run → live git status of watched files
  # is unchanged. The child mutates ONLY the copy; SIGKILL skips EXIT traps.
  _m="$ROOT/crates/cdcp_cli/src/main.rs"
  _ready="$BAK_DIR/sigkill.ready"
  rm -f "$_ready"
  stash "crates/cdcp_cli/src/main.rs"
  (
    sed 's/^    ExportWeb {/    ExportWebHidden {/' "$_m" > "$_m.tmp" && mv "$_m.tmp" "$_m"
    : > "$_ready"
    sleep 120
  ) &
  _kid=$!
  _waited=0
  while [ ! -f "$_ready" ]; do
    sleep 1
    _waited=$((_waited + 1))
    if [ "$_waited" -ge 15 ]; then
      kill "$_kid" 2>/dev/null || true
      fail "sigkill probe: child never became ready"
    fi
  done
  grep -q ExportWebHidden "$_m" \
    || { kill "$_kid" 2>/dev/null || true; fail "sigkill probe: copy was not mutated (vacuous)"; }
  grep -q '^    ExportWeb {' "$LIVE_ROOT/crates/cdcp_cli/src/main.rs" \
    || { kill "$_kid" 2>/dev/null || true; fail "sigkill probe: live main.rs lost ExportWeb"; }
  assert_live_unmoved "after private mutation (pre-kill)"
  assert_live_git_unmoved "after private mutation (pre-kill)"
  kill -9 "$_kid" 2>/dev/null || true
  wait "$_kid" 2>/dev/null || true
  assert_live_unmoved "after SIGKILL of mutator"
  assert_live_git_unmoved "after SIGKILL of mutator"
  unstash "crates/cdcp_cli/src/main.rs"
  grep -q '^    ExportWeb {' "$_m" \
    || fail "sigkill probe: copy main.rs not restored after probe"
  ok "SIGKILL mid-mutation leaves live tracked files unmoved"
}

# Finish a case: restore the copy, prove live files did not move, count it.
finish_case() {
  _fc_label="$1"
  _fc_target="$2"
  unstash "$_fc_target"
  assert_live_unmoved "after $_fc_label"
  assert_live_git_unmoved "after $_fc_label"
  inject_counted
  ok "$_fc_label trips RED in the private copy"
}

echo "==> selftest_reconstructed (L5–V11 reconstructed stages prove they bite)"

snapshot_live
materialise_private_tree
prove_sigkill_leaves_live_clean

# ── (a) L5 learner pack shape ───────────────────────────────────────────────
# Same python predicate check.sh uses.
F=web/data/mock40_seed42.json
stash "$F"
python3 -c "
import json,pathlib
p=pathlib.Path('$F'); d=json.load(open(p)); d['n_items']=39
p.write_text(json.dumps(d,indent=2,sort_keys=True)+chr(10))"
if python3 -c "
import json
d=json.load(open('$F'))
assert d['n_items']==40, 'n_items=%r' % d['n_items']
assert len(d['items'])==40, 'items=%d' % len(d['items'])
"; then
  fail "L5 learner pack shape stayed GREEN under n_items=39"
fi
finish_case "L5 learner pack shape (n_items drift)" "$F"

# ── (b) L5 learner pack must not leak answer letters ────────────────────────
stash "$F"
python3 -c "
import json,pathlib
p=pathlib.Path('$F'); d=json.load(open(p)); d['items'][0]['correct']='A'
p.write_text(json.dumps(d,indent=2,sort_keys=True)+chr(10))"
if python3 -c "
import json
d=json.load(open('$F'))
assert all('correct' not in i for i in d['items']), 'learner pack leaks correct letters'
"; then
  fail "L5 learner pack answer-key leak stayed GREEN"
fi
finish_case "L5 learner pack answer-key leak" "$F"

# ── (c) L6 multi-seed export-web byte-stability ─────────────────────────────
K=web/data/keys_seed42.json
stash "$K"
python3 -c "
import json,pathlib
p=pathlib.Path('$K'); d=json.load(open(p))
d['keys'][0]['correct']='A' if d['keys'][0]['correct']!='A' else 'B'
p.write_text(json.dumps(d,indent=2,sort_keys=True)+chr(10))"
_ms="$(mktemp -d "${TMPDIR:-/tmp}/cdcp_recon_ms.XXXXXX")"
_c0="$(date +%s)"
set +e
cargo run -q -p cdcp_cli --offline -- export-web --bank bank/items --seed 42 --out "$_ms" >/dev/null 2>"$BAK_DIR/export_web.err"
_ex=$?
set -e
COLD_CARGO_S=$(( $(date +%s) - _c0 ))
echo "selftest_reconstructed: COLD_CARGO_S=$COLD_CARGO_S (private target, cargo run -p cdcp_cli export-web)"
[ "$_ex" -eq 0 ] || fail "L6 export-web failed to run on the copy: $(cat "$BAK_DIR/export_web.err")"
if cmp -s "$_ms/keys_seed42.json" "$K"; then
  rm -rf "$_ms"
  fail "L6 export-web seed42 stayed golden-stable under keys mutation"
fi
rm -rf "$_ms"
finish_case "L6 export-web seed42 golden-stability" "$K"

# ── (d) L6 session shapes present in the Drill surface ──────────────────────
D=web/drill.html
stash "$D"
sed 's/Miss review/MISS_REVIEW_REMOVED/g' "$D" > "$D.tmp" && mv "$D.tmp" "$D"
grep -q "MISS_REVIEW_REMOVED" "$D" || fail "session-shape injection did not land in the copy"
# Same assertion check.sh uses (L6 session shapes). Full nested check.sh on
# a HEAD snapshot dies at clippy (mid-wave dirty docs) before this stage —
# running the stage here is the proof that is still reachable.
if grep -q "Miss review" "$D"; then
  fail "L6 session shapes stayed GREEN (Miss review still present in the copy)"
fi
unstash "$D"
assert_live_unmoved "after L6 session shapes"
assert_live_git_unmoved "after L6 session shapes"
inject_counted
ok "L6 session shapes (Miss review removed) trips in the private copy"

# ── (e) L7 CLI product verbs listed in --help ───────────────────────────────
M=crates/cdcp_cli/src/main.rs
stash "$M"
sed 's/^    ExportWeb {/    ExportWebHidden {/' "$M" > "$M.tmp" && mv "$M.tmp" "$M"
sed 's/Cmd::ExportWeb {/Cmd::ExportWebHidden {/' "$M" > "$M.tmp" && mv "$M.tmp" "$M"
grep -q ExportWebHidden "$M" || fail "CLI-verb injection did not land in the copy"
_help_log="$BAK_DIR/cli_help.log"
set +e
cargo run -q -p cdcp_cli --offline -- --help >"$_help_log" 2>&1
_help_rc=$?
set -e
[ "$_help_rc" -eq 0 ] || fail "copy cargo run --help failed after ExportWeb rename: $(cat "$_help_log")"
# clap kebab-cases ExportWebHidden to export-web-hidden, which CONTAINS
# the substring export-web. Match the subcommand token, not a prefix.
if grep -E '^[[:space:]]+export-web[[:space:]]' "$_help_log" >/dev/null; then
  fail "copy CLI still lists export-web after ExportWeb was renamed — sibling cargo would have compiled this"
fi
grep -E '^[[:space:]]+export-web-hidden[[:space:]]' "$_help_log" >/dev/null \
  || fail "copy CLI did not expose export-web-hidden (rename did not reach clap)"
ok "copy CLI --help lost export-web (live crate not compiled)"
# Same assertion check.sh uses (L7 CLI product verbs). Full nested check.sh
# on this snapshot dies at clippy before --help is consulted.
unstash "$M"
assert_live_unmoved "after L7 CLI product verbs"
assert_live_git_unmoved "after L7 CLI product verbs"
inject_counted
ok "L7 CLI product verbs (export-web renamed away) trips in the private copy"

# ── CHARTER pair: S0 + C1 + C2 ─────────────────────────────────────────────
# S0/C1: bd-single-leg-metatest-closes-illw
# C2:    bd-metatest-rerun-blocked-yhg6 (sample re-run of the recorded C2 pair
#        under cdcp_restore_safe; originally named at hash_payload).
# `.flywheel/CHARTER.md`: (1) mutate the gate → suite non-zero;
# (2) mutation still in place, delete the assertion → suite zero.
# A suite that only runs leg 1 is the defect — both legs increment PAIR.
# Restore of cargo-compiled sources goes through cdcp_restore_safe, never mv.
# True exit codes: cargo test is redirected to a FILE, never a pipe.
# Mutations are in the private snapshot; live crates/ is not written.

S0_GATE=crates/cdcp_gate/src/gates/substrate_guard.rs
S0_ASSERT=crates/cdcp_gate/tests/s0_charter_pair.rs
C1_GATE=crates/cdcp_assemble/src/lib.rs
C1_ASSERT=crates/cdcp_assemble/tests/c1_charter_pair.rs
C2_GATE=crates/cdcp_bank/src/lib.rs
C2_ASSERT=crates/cdcp_bank/tests/c2_charter_pair.rs

PAIR=0
pair_counted() { PAIR=$((PAIR + 1)); }

replace_once() {
  _file="$1"
  _old="$2"
  _new="$3"
  assert_private_path "$ROOT/$_file"
  python3 -c '
from pathlib import Path
import sys
p = Path(sys.argv[1])
old, new = sys.argv[2], sys.argv[3]
text = p.read_text()
n = text.count(old)
if n != 1:
    sys.stderr.write("CHARTER pair: needle count %d in %s (want 1)\n" % (n, p))
    sys.exit(2)
p.write_text(text.replace(old, new, 1))
' "$_file" "$_old" "$_new" || fail "CHARTER pair: could not mutate $_file"
}

# cargo test → SUITE_RC. Redirect to a file so the exit code is the child's,
# not a pipe's. Never `cmd | tee`.
run_pair_suite() {
  _pkg="$1"
  _t="$2"
  _log="$BAK_DIR/${_pkg}__${_t}.log"
  set +e
  cargo test -p "$_pkg" --offline --test "$_t" >"$_log" 2>&1
  SUITE_RC=$?
  set -e
}

newest_pair_bin() {
  python3 -c '
import glob, os, sys
name = sys.argv[1]
cands = []
for p in glob.glob(os.path.join("target", "debug", "deps", name + "-*")):
    if os.path.splitext(p)[1]:
        continue
    if os.path.isfile(p) and os.access(p, os.X_OK):
        cands.append((os.stat(p).st_mtime_ns, p))
if not cands:
    sys.exit(0)
print(max(cands)[1])
' "$1"
}

mtime_ns() {
  python3 -c 'import os,sys; print(os.stat(sys.argv[1]).st_mtime_ns)' "$1"
}

# After cdcp_restore_safe: prove cargo actually rebuilt the test artifact.
# A restore that leaves the perturbed binary in place is a FALSE GREEN.
prove_pair_rebuild() {
  _pkg="$1"
  _t="$2"
  _art="$(newest_pair_bin "$_t")"
  [ -n "$_art" ] || fail "CHARTER pair: no test binary for $_t before restore-rebuild"
  _before="$(mtime_ns "$_art")"
  set +e
  cargo test -p "$_pkg" --offline --test "$_t" --no-run >"$BAK_DIR/rebuild_${_t}.log" 2>&1
  _rc=$?
  set -e
  [ "$_rc" -eq 0 ] || fail "CHARTER pair: cargo --no-run after restore failed rc=$_rc ($(cat "$BAK_DIR/rebuild_${_t}.log"))"
  _art2="$(newest_pair_bin "$_t")"
  [ -n "$_art2" ] || fail "CHARTER pair: no test binary for $_t after restore-rebuild"
  _after="$(mtime_ns "$_art2")"
  if [ "$_art2" = "$_art" ] && [ "$_after" = "$_before" ]; then
    fail "ANTI-VACUOUS: restore of $_t rebuilt nothing (stale $_art mtime $_before)"
  fi
  ok "CHARTER pair restore rebuilt $_t via cdcp_restore_safe (artifact mtime moved)"
}

# ── S0: substrate-floor detector ──────────────────────────────────────────
stash "$S0_GATE"
stash "$S0_ASSERT"

replace_once "$S0_GATE" \
  '    if has_scanned_extension(&e.path, scan) {
        return Some(ScanReason::Extension(' \
  '    if false && has_scanned_extension(&e.path, scan) {
        return Some(ScanReason::Extension('

run_pair_suite cdcp_gate s0_charter_pair
S0_MUTATE_RC=$SUITE_RC
[ "$S0_MUTATE_RC" -ne 0 ] || {
  printf '%s\n' "$(cat "$BAK_DIR/cdcp_gate__s0_charter_pair.log")" >&2
  fail "S0 mutate stayed GREEN (want non-zero)"
}
pair_counted
ok "S0 mutate RED (rc=$S0_MUTATE_RC)"

assert_private_path "$ROOT/$S0_ASSERT"
printf '%s\n' '// assertion deleted (meta-test leg 2) — bd-single-leg-metatest-closes-illw' >"$S0_ASSERT"
run_pair_suite cdcp_gate s0_charter_pair
S0_DELETE_RC=$SUITE_RC
[ "$S0_DELETE_RC" -eq 0 ] || {
  printf '%s\n' "$(cat "$BAK_DIR/cdcp_gate__s0_charter_pair.log")" >&2
  fail "S0 delete-assert stayed RED (rc=$S0_DELETE_RC; want 0)"
}
pair_counted
ok "S0 delete-assert GREEN (rc=$S0_DELETE_RC)"

unstash "$S0_GATE"
unstash "$S0_ASSERT"
prove_pair_rebuild cdcp_gate s0_charter_pair
assert_live_unmoved "after S0 CHARTER pair"
assert_live_git_unmoved "after S0 CHARTER pair"

# ── C1: approved-only assembly filter ─────────────────────────────────────
stash "$C1_GATE"
stash "$C1_ASSERT"

replace_once "$C1_GATE" \
  '    let approved: Vec<&BankItem> = bank.items.values().filter(|i| i.is_approved()).collect();' \
  '    let approved: Vec<&BankItem> = bank.items.values().filter(|_i| true).collect();'

run_pair_suite cdcp_assemble c1_charter_pair
C1_MUTATE_RC=$SUITE_RC
[ "$C1_MUTATE_RC" -ne 0 ] || {
  printf '%s\n' "$(cat "$BAK_DIR/cdcp_assemble__c1_charter_pair.log")" >&2
  fail "C1 mutate stayed GREEN (want non-zero)"
}
pair_counted
ok "C1 mutate RED (rc=$C1_MUTATE_RC)"

assert_private_path "$ROOT/$C1_ASSERT"
printf '%s\n' '// assertion deleted (meta-test leg 2) — bd-single-leg-metatest-closes-illw' >"$C1_ASSERT"
run_pair_suite cdcp_assemble c1_charter_pair
C1_DELETE_RC=$SUITE_RC
[ "$C1_DELETE_RC" -eq 0 ] || {
  printf '%s\n' "$(cat "$BAK_DIR/cdcp_assemble__c1_charter_pair.log")" >&2
  fail "C1 delete-assert stayed RED (rc=$C1_DELETE_RC; want 0)"
}
pair_counted
ok "C1 delete-assert GREEN (rc=$C1_DELETE_RC)"

unstash "$C1_GATE"
unstash "$C1_ASSERT"
prove_pair_rebuild cdcp_assemble c1_charter_pair
assert_live_unmoved "after C1 CHARTER pair"
assert_live_git_unmoved "after C1 CHARTER pair"

# ── C2: bank_hash covers status ───────────────────────────────────────────
stash "$C2_GATE"
stash "$C2_ASSERT"

replace_once "$C2_GATE" \
  '        m.insert("status".into(), serde_json::json!(self.status.as_str()));' \
  '        // mutated: status omitted from hash_payload (CHARTER pair C2)'

run_pair_suite cdcp_bank c2_charter_pair
C2_MUTATE_RC=$SUITE_RC
[ "$C2_MUTATE_RC" -ne 0 ] || {
  printf '%s\n' "$(cat "$BAK_DIR/cdcp_bank__c2_charter_pair.log")" >&2
  fail "C2 mutate stayed GREEN (want non-zero)"
}
pair_counted
ok "C2 mutate RED (rc=$C2_MUTATE_RC)"

assert_private_path "$ROOT/$C2_ASSERT"
printf '%s\n' '// assertion deleted (meta-test leg 2) — bd-metatest-rerun-blocked-yhg6' >"$C2_ASSERT"
run_pair_suite cdcp_bank c2_charter_pair
C2_DELETE_RC=$SUITE_RC
[ "$C2_DELETE_RC" -eq 0 ] || {
  printf '%s\n' "$(cat "$BAK_DIR/cdcp_bank__c2_charter_pair.log")" >&2
  fail "C2 delete-assert stayed RED (rc=$C2_DELETE_RC; want 0)"
}
pair_counted
ok "C2 delete-assert GREEN (rc=$C2_DELETE_RC)"

unstash "$C2_GATE"
unstash "$C2_ASSERT"
prove_pair_rebuild cdcp_bank c2_charter_pair
assert_live_unmoved "after C2 CHARTER pair"
assert_live_git_unmoved "after C2 CHARTER pair"

[ "$PAIR" -eq 6 ] || fail "ANTI-VACUOUS: CHARTER pair ran $PAIR legs, want 6 (a suite that only runs leg 1 is the defect)"
echo "CHARTER_PAIR_LEGS=$PAIR S0_MUTATE=$S0_MUTATE_RC S0_DELETE_ASSERT=$S0_DELETE_RC C1_MUTATE=$C1_MUTATE_RC C1_DELETE_ASSERT=$C1_DELETE_RC C2_MUTATE=$C2_MUTATE_RC C2_DELETE_ASSERT=$C2_DELETE_RC"
ok "CHARTER pair S0+C1+C2 (mutate/delete, 6/6 legs, restore_safe)"

assert_live_unmoved "end of suite"
assert_live_git_unmoved "end of suite"
[ -n "$COLD_CARGO_S" ] || fail "ANTI-VACUOUS: private-tree cargo was never timed"

echo "INJECTIONS=$INJ SUITE=$SUITE_NAME"
echo "selftest_reconstructed: COLD_CARGO_S=$COLD_CARGO_S"
echo "selftest_reconstructed: PASSED (a learner shape · b key leak · c export byte-stability · d session shapes · e CLI verbs · f S0/C1/C2 CHARTER pair · private-tree isolation)"
exit 0
