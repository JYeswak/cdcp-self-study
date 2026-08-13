#!/usr/bin/env sh
# selftest_known_bad.sh — L4 gates-proven-to-trip for course-engine check.sh
#
# Injects known-bad fixtures, asserts the relevant gate goes RED, restores
# the tree. Never leaves goldens/bank/docs dirty (trap + best-effort restore).
#
# Cases:
#   a) flipped golden digest content → goldens check fails
#   b) empty bank directory (temp) → Bank::load / goldens check fails
#   c) bank_hash pin drift → goldens check fails
#   d) planted credential-inflation string under docs/ → honesty scan fails
#
# Invoked from scripts/check.sh after the clean goldens path is green.
set -eu

ROOT="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() { echo "selftest_known_bad: FAIL: $*" >&2; exit 2; }
ok() { echo "selftest_known_bad: ok: $*"; }

# -- L4 drift guard: self-reported RED-injection count ----------------------
# INJ counts the injections this run actually asserted RED (green controls are
# NOT counted). Emitted once, on the success path only, as a machine-readable
# line that scripts/verify_injection_count.py aggregates. A suite that stops
# emitting the line is an ERROR to that gate, never a silent zero.
INJ=0
SUITE_NAME="selftest_known_bad"
inject_counted() { INJ=$((INJ + 1)); }

# Restore helpers (idempotent)
GOLDEN_CORRECT="goldens/mock40_seed42_all_correct.sha256"
BANK_HASH_PIN="goldens/bank_hash.txt"
HONESTY_PLANT="docs/_selftest_known_bad_planted.md"
_GOLDEN_BAK=""
_BANK_HASH_BAK=""

restore_all() {
  if [ -n "${_GOLDEN_BAK:-}" ] && [ -f "${_GOLDEN_BAK}" ]; then
    mv -f "${_GOLDEN_BAK}" "$GOLDEN_CORRECT" 2>/dev/null || true
  fi
  if [ -n "${_BANK_HASH_BAK:-}" ] && [ -f "${_BANK_HASH_BAK}" ]; then
    mv -f "${_BANK_HASH_BAK}" "$BANK_HASH_PIN" 2>/dev/null || true
  fi
  rm -f "$HONESTY_PLANT" 2>/dev/null || true
}
trap restore_all EXIT INT TERM HUP

assert_nonzero() {
  # usage: assert_nonzero "label" cmd...
  label="$1"
  shift
  rc=0
  out="$("$@" 2>&1)" || rc=$?
  if [ "$rc" -eq 0 ]; then
    printf '%s\n' "$out" >&2
    fail "expected RED for $label but command exited 0"
  fi
  inject_counted
  ok "$label trips RED (rc=$rc)"
}

# ── honesty_scan: same contract as check.sh (must stay in sync) ─────────────
# Prints nothing on success. Prints hits on inflation.
# Sets global HONESTY_RC: 0=clean, 1=inflation found, 2=scanner error
# (Avoid `return N` under set -e — non-zero return aborts the script.)
run_honesty_scan() {
  HONESTY_RC=0
  if ! command -v rg >/dev/null 2>&1; then
    echo "rg required for honesty scan" >&2
    HONESTY_RC=2
    return 0
  fi
  # --no-config: ignore ~/.ripgreprc --type-not=video (would exit 2 and fail-open)
  hits=""
  rc=0
  hits="$(rg --no-config -n --glob '*.md' --glob '*.toml' \
    'you are (now )?CDCP certified|officially certified by EPI' \
    docs knowledge 2>&1)" || rc=$?
  case "$rc" in
    0)
      filtered=""
      filtered="$(printf '%s\n' "$hits" | rg --no-config -v 'not |never |FORBIDDEN|forbidden' || true)"
      if [ -n "$filtered" ]; then
        printf '%s\n' "$filtered"
        HONESTY_RC=1
        return 0
      fi
      HONESTY_RC=0
      return 0
      ;;
    1)
      HONESTY_RC=0
      return 0
      ;;
    *)
      printf '%s\n' "$hits" >&2
      HONESTY_RC=2
      return 0
      ;;
  esac
}

echo "==> selftest_known_bad (L4 gates-proven-to-trip)"

# Preconditions
[ -f "$GOLDEN_CORRECT" ] || fail "missing $GOLDEN_CORRECT"
[ -f "$BANK_HASH_PIN" ] || fail "missing $BANK_HASH_PIN"
[ -d bank/items ] || fail "missing bank/items"
command -v cargo >/dev/null 2>&1 || fail "cargo required"
command -v rg >/dev/null 2>&1 || fail "rg required"

# ── (a) flipped golden content ──────────────────────────────────────────────
_GOLDEN_BAK="$(mktemp "${TMPDIR:-/tmp}/golden_correct.XXXXXX")"
cp "$GOLDEN_CORRECT" "$_GOLDEN_BAK"
# Flip: replace digest so it cannot match live GradeExact output
printf '%s\n' "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" >"$GOLDEN_CORRECT"
assert_nonzero "flipped-golden" \
  cargo run -q -p cdcp_cli -- goldens check --bank bank/items --dir goldens
mv -f "$_GOLDEN_BAK" "$GOLDEN_CORRECT"
_GOLDEN_BAK=""
ok "restored $GOLDEN_CORRECT"

# ── (b) empty bank directory (temp) ─────────────────────────────────────────
empty_bank="$(mktemp -d "${TMPDIR:-/tmp}/empty_bank.XXXXXX")"
# Bank::load_dir on empty dir must fail (no items / empty bank)
assert_nonzero "empty-bank" \
  cargo run -q -p cdcp_cli -- goldens check --bank "$empty_bank" --dir goldens
rmdir "$empty_bank" 2>/dev/null || rm -rf "$empty_bank"
ok "empty bank temp removed"

# ── (c) bank_hash pin drift ─────────────────────────────────────────────────
_BANK_HASH_BAK="$(mktemp "${TMPDIR:-/tmp}/bank_hash.XXXXXX")"
cp "$BANK_HASH_PIN" "$_BANK_HASH_BAK"
printf '%s\n' "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef" >"$BANK_HASH_PIN"
assert_nonzero "bank_hash-drift" \
  cargo run -q -p cdcp_cli -- goldens check --bank bank/items --dir goldens
mv -f "$_BANK_HASH_BAK" "$BANK_HASH_PIN"
_BANK_HASH_BAK=""
ok "restored $BANK_HASH_PIN"

# ── (d) planted credential inflation under docs/ ────────────────────────────
printf '%s\n' "you are CDCP certified" >"$HONESTY_PLANT"
run_honesty_scan >/dev/null
h_rc="$HONESTY_RC"
rm -f "$HONESTY_PLANT"
if [ "$h_rc" -eq 0 ]; then
  fail "honesty scan stayed green with planted 'you are CDCP certified' in docs/"
fi
if [ "$h_rc" -ge 2 ]; then
  fail "honesty scan error (rc=$h_rc) — scanner must not fail-open on config"
fi
inject_counted
ok "planted honesty string trips RED (rc=$h_rc)"

# ── clean-tree recheck (honesty alone; goldens restored above) ──────────────
run_honesty_scan >/dev/null
h_rc="$HONESTY_RC"
[ "$h_rc" -eq 0 ] || fail "honesty scan not clean after restore (rc=$h_rc)"
ok "honesty clean after restore"

echo "INJECTIONS=$INJ SUITE=$SUITE_NAME"
echo "selftest_known_bad: PASSED (a golden flip · b empty bank · c bank_hash drift · d honesty plant)"
exit 0
