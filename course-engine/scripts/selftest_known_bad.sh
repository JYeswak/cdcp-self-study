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
  set +e
  out="$("$@" 2>&1)"
  rc=$?
  set -e
  if [ "$rc" -eq 0 ]; then
    printf '%s\n' "$out" >&2
    fail "expected RED for $label but command exited 0"
  fi
  ok "$label trips RED (rc=$rc)"
}

# ── honesty_scan: same contract as check.sh (must stay in sync) ─────────────
# exit 0 = clean, 1 = inflation found, 2+ = scanner error
honesty_scan() {
  if ! command -v rg >/dev/null 2>&1; then
    echo "rg required for honesty scan" >&2
    return 2
  fi
  # --no-config: ignore ~/.ripgreprc --type-not=video (would exit 2 and fail-open)
  set +e
  hits="$(rg --no-config -n --glob '*.md' --glob '*.toml' \
    'you are (now )?CDCP certified|officially certified by EPI' \
    docs knowledge 2>&1)"
  rc=$?
  set -e
  case "$rc" in
    0)
      filtered="$(printf '%s\n' "$hits" | rg --no-config -v 'not |never |FORBIDDEN|forbidden' || true)"
      if [ -n "$filtered" ]; then
        printf '%s\n' "$filtered"
        return 1
      fi
      return 0
      ;;
    1) return 0 ;; # no matches
    *)
      printf '%s\n' "$hits" >&2
      return 2
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
# Flip: replace first hex nibble so digest cannot match
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
set +e
honesty_scan >/dev/null 2>&1
h_rc=$?
set -e
rm -f "$HONESTY_PLANT"
if [ "$h_rc" -eq 0 ]; then
  fail "honesty scan stayed green with planted 'you are CDCP certified' in docs/"
fi
if [ "$h_rc" -ge 2 ]; then
  fail "honesty scan error (rc=$h_rc) — scanner must not fail-open on config"
fi
ok "planted honesty string trips RED (rc=$h_rc)"

# ── clean-tree recheck (honesty alone; goldens restored above) ──────────────
set +e
honesty_scan >/dev/null 2>&1
h_rc=$?
set -e
[ "$h_rc" -eq 0 ] || fail "honesty scan not clean after restore (rc=$h_rc)"
ok "honesty clean after restore"

echo "selftest_known_bad: PASSED (a golden flip · b empty bank · c bank_hash drift · d honesty plant)"
exit 0
