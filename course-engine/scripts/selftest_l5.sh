#!/usr/bin/env sh
# selftest_l5.sh — L5 gates-proven-to-trip (honesty + e2e digest known-bad)
#
# Runs:
#   1) scripts/selftest_l5_honesty.sh  — planted cert string under web/ → RED
#   2) clean e2e_l5_digest.sh           — must PASS (matched digests)
#   3) flipped golden pins in TEMP dir  — e2e → non-zero + GOLDEN MISMATCH
#   4) empty golden dir (zero fixtures) — e2e → ERROR (no vacuous green)
#
# Never mutates committed goldens/ (temp fixture path only). Trap cleans TEMP.
# S8 will invoke this from check.sh after the clean L5 e2e stage is green.
set -eu

ROOT="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() { echo "selftest_l5: FAIL: $*" >&2; exit 2; }
ok() { echo "selftest_l5: ok: $*"; }

# -- L4 drift guard: self-reported RED-injection count ----------------------
# INJ counts the injections this run actually asserted RED (green controls are
# NOT counted). Emitted once, on the success path only, as a machine-readable
# line that scripts/verify_injection_count.py aggregates. A suite that stops
# emitting the line is an ERROR to that gate, never a silent zero.
INJ=0
SUITE_NAME="selftest_l5"
inject_counted() { INJ=$((INJ + 1)); }

TMP_ROOT=""
restore_all() {
  if [ -n "${TMP_ROOT:-}" ] && [ -d "${TMP_ROOT}" ]; then
    rm -rf "${TMP_ROOT}" 2>/dev/null || true
  fi
}
trap restore_all EXIT INT TERM HUP

# Assert cmd exits non-zero AND combined output contains needle.
assert_fails_with() {
  label="$1"
  needle="$2"
  shift 2
  rc=0
  out="$("$@" 2>&1)" || rc=$?
  if [ "$rc" -eq 0 ]; then
    printf '%s\n' "$out" >&2
    fail "expected RED for $label but command exited 0"
  fi
  case "$out" in
    *"$needle"*)
      inject_counted
      ok "$label trips RED (rc=$rc, saw: $needle)"
      ;;
    *)
      printf '%s\n' "$out" >&2
      fail "$label exited $rc but missing expected signal '$needle'"
      ;;
  esac
}

echo "==> selftest_l5 (L5 honesty + e2e digest known-bad)"

[ -f scripts/selftest_l5_honesty.sh ] || fail "missing scripts/selftest_l5_honesty.sh"
[ -f scripts/e2e_l5_digest.sh ] || fail "missing scripts/e2e_l5_digest.sh"
[ -f scripts/smoke_results_wasm.mjs ] || fail "missing scripts/smoke_results_wasm.mjs"
[ -f goldens/mock40_seed42_all_correct.sha256 ] || fail "missing goldens/mock40_seed42_all_correct.sha256"
[ -f goldens/mock40_seed42_all_wrong.sha256 ] || fail "missing goldens/mock40_seed42_all_wrong.sha256"
command -v node >/dev/null 2>&1 || fail "node required"
command -v rg >/dev/null 2>&1 || fail "rg required"

# --- (1) UI honesty known-bad (plant under web/, restore) ---
echo "==> (1) selftest_l5_honesty.sh"
sh scripts/selftest_l5_honesty.sh || fail "selftest_l5_honesty.sh"
ok "honesty selftest PASS"

# --- (2) Clean e2e digest match (committed goldens, real wasm) ---
echo "==> (2) e2e_l5_digest clean pass"
rc=0
clean_out="$(sh scripts/e2e_l5_digest.sh 2>&1)" || rc=$?
printf '%s\n' "$clean_out"
[ "$rc" -eq 0 ] || fail "clean e2e_l5_digest exited $rc (wasm/fixtures must be green before known-bad)"
printf '%s\n' "$clean_out" | grep -q 'matched digests: all-correct=' \
  || fail "clean e2e did not print matched digests"
ok "clean e2e digest match"

# --- (3) Flipped golden pins in TEMP dir (never touch goldens/) ---
echo "==> (3) flipped golden pin → GOLDEN MISMATCH"
TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/selftest_l5.XXXXXX")"
FLIP_DIR="$TMP_ROOT/goldens_flipped"
mkdir -p "$FLIP_DIR"
# Wrong expected digests (64 hex) — real WASM digests must not match these
printf '%s\n' "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" \
  >"$FLIP_DIR/mock40_seed42_all_correct.sha256"
printf '%s\n' "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee" \
  >"$FLIP_DIR/mock40_seed42_all_wrong.sha256"

assert_fails_with "flipped-golden-e2e" "GOLDEN MISMATCH" \
  sh scripts/e2e_l5_digest.sh --golden-dir "$FLIP_DIR"
ok "committed goldens/ untouched (used TEMP --golden-dir only)"

# --- (4) Zero fixtures → ERROR (no vacuous green) ---
echo "==> (4) empty golden dir → vacuous ERROR"
EMPTY_DIR="$TMP_ROOT/goldens_empty"
mkdir -p "$EMPTY_DIR"
assert_fails_with "empty-golden-dir" "vacuous" \
  sh scripts/e2e_l5_digest.sh --golden-dir "$EMPTY_DIR"
ok "empty fixtures refuse green"

# --- recheck committed goldens still match (sanity) ---
echo "==> recheck clean e2e after known-bad"
sh scripts/e2e_l5_digest.sh >/dev/null || fail "clean e2e broken after known-bad"
ok "clean e2e still green after known-bad"

# Confirm committed pins unchanged (still 64 hex and match smoke)
real_correct="$(tr -d ' \n\r\t' <goldens/mock40_seed42_all_correct.sha256)"
case "$real_correct" in
  ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff)
    fail "committed golden all_correct looks flipped — tree dirty"
    ;;
esac
ok "committed golden pins not dirty"

echo "INJECTIONS=$INJ SUITE=$SUITE_NAME"
echo "selftest_l5: PASSED (honesty · clean e2e · flipped golden RED · empty fixtures ERROR)"
exit 0
