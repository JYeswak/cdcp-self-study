#!/usr/bin/env sh
# selftest_reconstructed.sh — L4 gates-proven-to-trip for the L5→V11 stages
# reconstructed in check.sh (2026-08-12), plus the reimplemented CLI verbs.
#
# Contract (mirrors scripts/selftest_known_bad.sh):
#   for each stage: inject a known-bad, assert check.sh goes RED, restore.
#   Never leave the tree dirty. An injection that stays GREEN is a FAILURE —
#   a gate that cannot fail is not a gate.
#
# RESTORE (bd-stale-binary-mtime-trap-p65w): case (e) perturbs
# crates/cdcp_cli/src/main.rs; the S0/C1 CHARTER pair perturbs
# substrate_guard.rs, s0_charter_pair.rs, cdcp_assemble/src/lib.rs, and
# c1_charter_pair.rs. Restore MUST go through scripts/restore_safe.inc.sh —
# `mv backup dest` would hand the file the backup's older mtime, cargo would
# skip, and the next run would test the PERTURBED binary. The helper writes
# bytes into the existing inode. After each pair, we force a build and
# require the test artifact mtime to move.
#
# Runs the FULL check.sh per case (slow but honest): it proves the stage is
# wired into the real gate chain, not merely that a script exists.
set -eu

ROOT="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

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

. "$ROOT/scripts/restore_safe.inc.sh"
# Wired prove: naive-mv leaves the backup mtime (RED), helper does not,
# and the scan of converted cargo-touching restore sites is non-vacuous.
# Cargo skip-vs-rebuild is the helper's own executable selftest — not run
# here, so this suite does not take the workspace cargo lock.
cdcp_restore_safe_mtime_demo || fail "restore_safe mtime demo"
cdcp_restore_safe_scan "$ROOT" || fail "restore_safe converted-site scan"

BAK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/cdcp_recon.XXXXXX")"
RESTORE_LIST=""

restore_all() {
  for rel in $RESTORE_LIST; do
    enc="$(printf '%s' "$rel" | tr '/' '_')"
    [ -f "$BAK_DIR/$enc" ] && cdcp_restore_safe "$rel" "$BAK_DIR/$enc"
  done
  rm -rf "$BAK_DIR"
}
trap restore_all EXIT INT TERM

stash() {
  enc="$(printf '%s' "$1" | tr '/' '_')"
  cp "$1" "$BAK_DIR/$enc"
  RESTORE_LIST="$RESTORE_LIST $1"
}

unstash() {
  enc="$(printf '%s' "$1" | tr '/' '_')"
  cdcp_restore_safe "$1" "$BAK_DIR/$enc"
}

# assert_red <label> <file-to-restore>  — check.sh must exit non-zero
assert_red() {
  label="$1"; target="$2"
  set +e
  CDCP_IN_SELFTEST=1 sh scripts/check.sh >/dev/null 2>&1
  rc=$?
  set -e
  unstash "$target"
  [ "$rc" -eq 0 ] && fail "$label stayed GREEN under known-bad injection"
  inject_counted
  ok "$label trips RED (rc=$rc)"
}

echo "==> selftest_reconstructed (L5–V11 reconstructed stages prove they bite)"

# ── (a) L5 learner pack shape ───────────────────────────────────────────────
F=web/data/mock40_seed42.json
stash "$F"
python3 -c "
import json,pathlib
p=pathlib.Path('$F'); d=json.load(open(p)); d['n_items']=39
p.write_text(json.dumps(d,indent=2,sort_keys=True)+chr(10))"
assert_red "L5 learner pack shape (n_items drift)" "$F"

# ── (b) L5 learner pack must not leak answer letters ────────────────────────
stash "$F"
python3 -c "
import json,pathlib
p=pathlib.Path('$F'); d=json.load(open(p)); d['items'][0]['correct']='A'
p.write_text(json.dumps(d,indent=2,sort_keys=True)+chr(10))"
assert_red "L5 learner pack answer-key leak" "$F"

# ── (c) L6 multi-seed export-web byte-stability ─────────────────────────────
K=web/data/keys_seed42.json
stash "$K"
python3 -c "
import json,pathlib
p=pathlib.Path('$K'); d=json.load(open(p))
d['keys'][0]['correct']='A' if d['keys'][0]['correct']!='A' else 'B'
p.write_text(json.dumps(d,indent=2,sort_keys=True)+chr(10))"
assert_red "L6 export-web seed42 golden-stability" "$K"

# ── (d) L6 session shapes present in the Drill surface ──────────────────────
D=web/drill.html
stash "$D"
sed 's/Miss review/MISS_REVIEW_REMOVED/g' "$D" > "$D.tmp" && mv "$D.tmp" "$D"
assert_red "L6 session shapes (Miss review removed)" "$D"

# ── (e) L7 CLI product verbs listed in --help ───────────────────────────────
M=crates/cdcp_cli/src/main.rs
stash "$M"
sed 's/^    ExportWeb {/    ExportWebHidden {/' "$M" > "$M.tmp" && mv "$M.tmp" "$M"
sed 's/Cmd::ExportWeb {/Cmd::ExportWebHidden {/' "$M" > "$M.tmp" && mv "$M.tmp" "$M"
assert_red "L7 CLI product verbs (export-web renamed away)" "$M"

# ── CHARTER pair: S0 + C1 (bd-single-leg-metatest-closes-illw) ──────────────
# `.flywheel/CHARTER.md`: (1) mutate the gate → suite non-zero;
# (2) mutation still in place, delete the assertion → suite zero.
# A suite that only runs leg 1 is the defect — both legs increment PAIR.
# Restore of cargo-compiled sources goes through cdcp_restore_safe, never mv.
# True exit codes: cargo test is redirected to a FILE, never a pipe.

S0_GATE=crates/cdcp_gate/src/gates/substrate_guard.rs
S0_ASSERT=crates/cdcp_gate/tests/s0_charter_pair.rs
C1_GATE=crates/cdcp_assemble/src/lib.rs
C1_ASSERT=crates/cdcp_assemble/tests/c1_charter_pair.rs

PAIR=0
pair_counted() { PAIR=$((PAIR + 1)); }

replace_once() {
  _file="$1"
  _old="$2"
  _new="$3"
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

[ "$PAIR" -eq 4 ] || fail "ANTI-VACUOUS: CHARTER pair ran $PAIR legs, want 4 (a suite that only runs leg 1 is the defect)"
echo "CHARTER_PAIR_LEGS=$PAIR S0_MUTATE=$S0_MUTATE_RC S0_DELETE_ASSERT=$S0_DELETE_RC C1_MUTATE=$C1_MUTATE_RC C1_DELETE_ASSERT=$C1_DELETE_RC"
ok "CHARTER pair S0+C1 (mutate/delete, 4/4 legs, restore_safe)"

echo "INJECTIONS=$INJ SUITE=$SUITE_NAME"
echo "selftest_reconstructed: PASSED (a learner shape · b key leak · c export byte-stability · d session shapes · e CLI verbs · f S0/C1 CHARTER pair)"
exit 0
