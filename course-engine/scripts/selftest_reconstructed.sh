#!/usr/bin/env sh
# selftest_reconstructed.sh — L4 gates-proven-to-trip for the L5→V11 stages
# reconstructed in check.sh (2026-08-12), plus the reimplemented CLI verbs.
#
# Contract (mirrors scripts/selftest_known_bad.sh):
#   for each stage: inject a known-bad, assert check.sh goes RED, restore.
#   Never leave the tree dirty. An injection that stays GREEN is a FAILURE —
#   a gate that cannot fail is not a gate.
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

BAK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/cdcp_recon.XXXXXX")"
RESTORE_LIST=""

restore_all() {
  for rel in $RESTORE_LIST; do
    enc="$(printf '%s' "$rel" | tr '/' '_')"
    [ -f "$BAK_DIR/$enc" ] && cp "$BAK_DIR/$enc" "$rel"
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
  cp "$BAK_DIR/$enc" "$1"
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

echo "INJECTIONS=$INJ SUITE=$SUITE_NAME"
echo "selftest_reconstructed: PASSED (a learner shape · b key leak · c export byte-stability · d session shapes · e CLI verbs)"
exit 0
