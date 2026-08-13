#!/usr/bin/env sh
# selftest_l6_coverage.sh — L6 domain coverage gate proven to trip
#
# In-tree known-bad against TEMP dirs only (never mutates bank/items):
#   a) empty bank → RED (vacuous empty = ERROR)
#   b) filtered bank missing at least one primary module → RED
#   c) live bank still GREEN after (a)(b)
#
# Trap cleans TEMP. Never leaves bank dirty.
set -eu

ROOT="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() { echo "selftest_l6_coverage: FAIL: $*" >&2; exit 2; }
ok() { echo "selftest_l6_coverage: ok: $*"; }

# -- L4 drift guard: self-reported RED-injection count ----------------------
# INJ counts the injections this run actually asserted RED (green controls are
# NOT counted). Emitted once, on the success path only, as a machine-readable
# line that scripts/verify_injection_count.py aggregates. A suite that stops
# emitting the line is an ERROR to that gate, never a silent zero.
INJ=0
SUITE_NAME="selftest_l6_coverage"
inject_counted() { INJ=$((INJ + 1)); }

TMP_ROOT=""
restore_all() {
  if [ -n "${TMP_ROOT:-}" ] && [ -d "${TMP_ROOT}" ]; then
    rm -rf "${TMP_ROOT}" 2>/dev/null || true
  fi
}
trap restore_all EXIT INT TERM HUP

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

echo "==> selftest_l6_coverage (L6 domain coverage known-bad)"

[ -f scripts/verify_coverage.py ] || fail "missing scripts/verify_coverage.py"
[ -d bank/items ] || fail "missing bank/items"
command -v python3 >/dev/null 2>&1 || fail "python3 required"

TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/selftest_l6_coverage.XXXXXX")"

# --- (a) empty bank directory → vacuous ERROR ---
echo "==> (a) empty bank → ERROR"
empty_bank="$TMP_ROOT/empty_bank"
mkdir -p "$empty_bank"
assert_fails_with "empty-bank" "empty bank" \
  python3 scripts/verify_coverage.py --bank "$empty_bank"
ok "empty bank temp removed path kept under TMP only"

# --- (b) filtered set: only module-1 items → modules 2–14 SHORT ---
# Plant in TEMP only (never touch live bank/items). Non-empty bank but
# missing primary modules must RED — not only the vacuous-empty case.
echo "==> (b) filtered bank (module 1 only) → SHORT / RED"
filt_bank="$TMP_ROOT/m01_only"
mkdir -p "$filt_bank"
cat >"$filt_bank/planted-m01.toml" <<'EOF'
id = "selftest-m01-only"
module = 1
stem = "selftest planted item — not for exam use"
choices = ["A", "B", "C", "D"]
correct = "A"
explanation = "planted for coverage selftest only"
topic_ids = ["m01-importance"]
bloom = "remember"
source_class = "original"
quantity_evidence = "qualitative_only"
EOF
# Needle: shortfall line for module 2 (policy min or OQ-05 floor of 1)
assert_fails_with "m01-only-bank" "module 2:" \
  python3 scripts/verify_coverage.py --bank "$filt_bank"
ok "filtered missing-module bank trips RED"

# --- (c) live bank still GREEN (no dirt left) ---
echo "==> (c) live bank coverage GREEN"
rc=0
live_out="$(python3 scripts/verify_coverage.py --bank bank/items 2>&1)" || rc=$?
printf '%s\n' "$live_out"
[ "$rc" -eq 0 ] || fail "live bank verify_coverage exited $rc (selftest must not dirty bank)"
printf '%s\n' "$live_out" | grep -q 'coverage GREEN' \
  || fail "live bank output missing 'coverage GREEN'"
ok "live bank still GREEN"

# Confirm temp is only place we wrote; bank/items untouched by selftest
# (no planted file under live bank)
if [ -f bank/items/planted-m01.toml ]; then
  fail "planted file leaked into live bank/items"
fi

echo "INJECTIONS=$INJ SUITE=$SUITE_NAME"
echo "selftest_l6_coverage: PASSED (a empty RED · b missing-module RED · c live GREEN)"
exit 0
