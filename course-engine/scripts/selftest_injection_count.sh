#!/usr/bin/env sh
# selftest_injection_count.sh — the drift guard, proven to trip.
#
# Contract (mirrors the other selftest_*.sh suites): write real specimen
# receipt logs and README fixtures into TEMP, inject one known-bad at a time,
# assert verify_injection_count.py goes RED with the expected signal. Nothing
# in the live tree is mutated and no patch is ever applied — the specimens are
# files this script writes, so they cannot silently no-op the way `git apply`
# does on an index mismatch.
#
# Cases:
#   a) log + README agree                 → GREEN (baseline)
#   b) README off by one                  → RED (advertised count drifted)
#   c) a suite's INJECTIONS= line deleted → RED (MISSING, never a silent zero)
#   d) a suite self-reports zero          → RED (a suite asserting no RED is not a gate)
#   e) unregistered suite in the log      → RED (new suite must be registered)
#   f) empty log                          → ERROR (anti-vacuous)
#   g) README advertises no count at all  → ERROR (nothing to check is not a pass)
#   h) README suite count wrong           → RED
set -eu

ROOT="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() { echo "selftest_injection_count: FAIL: $*" >&2; exit 2; }
ok() { echo "selftest_injection_count: ok: $*"; }

# -- L4 drift guard: self-reported RED-injection count ----------------------
# INJ counts the injections this run actually asserted RED (green controls are
# NOT counted). Emitted once, on the success path only, as a machine-readable
# line that scripts/verify_injection_count.py aggregates. A suite that stops
# emitting the line is an ERROR to that gate, never a silent zero.
INJ=0
SUITE_NAME="selftest_injection_count"
inject_counted() { INJ=$((INJ + 1)); }

TMP_ROOT=""
restore_all() {
  if [ -n "${TMP_ROOT:-}" ] && [ -d "${TMP_ROOT}" ]; then
    rm -rf "${TMP_ROOT}" 2>/dev/null || true
  fi
}
trap restore_all EXIT INT TERM HUP

CHECKER="scripts/verify_injection_count.py"

# Two specimen suites only — the registry under test is passed via --require,
# so this selftest never depends on the live suite roster.
REQUIRE="spec_alpha,spec_beta"

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

assert_green() {
  label="$1"
  shift
  rc=0
  out="$("$@" 2>&1)" || rc=$?
  if [ "$rc" -ne 0 ]; then
    printf '%s\n' "$out" >&2
    fail "$label expected GREEN but exited $rc"
  fi
  case "$out" in
    *"injection count GREEN"*) ok "$label GREEN" ;;
    *)
      printf '%s\n' "$out" >&2
      fail "$label exited 0 without the 'injection count GREEN' receipt"
      ;;
  esac
}

echo "==> selftest_injection_count (drift-guard known-bad)"

[ -f "$CHECKER" ] || fail "missing $CHECKER"
command -v python3 >/dev/null 2>&1 || fail "python3 required"

TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/selftest_injection_count.XXXXXX")"

# specimen README advertising 7 injections across 2 suites
write_readme() {
  cat >"$1" <<EOF
# Specimen readme

[![known-bad: $2 injections](https://img.shields.io/badge/known--bad-$2_injections_all_RED-success.svg)](#x)

| **Gate** | $3 selftest suites; $2 known-bad injections that must all go RED |

Two selftest suites inject **$2 known-bad faults** and assert the build fails.

| **L4 — gates proven to trip** | ok | $3 suites, $2 injections, anti-vacuous |
EOF
}

GOOD_LOG="$TMP_ROOT/good.log"
printf 'INJECTIONS=3 SUITE=spec_alpha\nINJECTIONS=4 SUITE=spec_beta\n' >"$GOOD_LOG"
GOOD_README="$TMP_ROOT/README.md"
write_readme "$GOOD_README" 7 2

run_checker() {
  python3 "$CHECKER" --log "$1" --readme "$2" --require "$REQUIRE"
}

# ── (a) baseline ────────────────────────────────────────────────────────────
echo "==> (a) log and README agree → GREEN"
assert_green "baseline" run_checker "$GOOD_LOG" "$GOOD_README"

# ── (b) README advertises one too many ──────────────────────────────────────
echo "==> (b) README off by one → RED"
off_readme="$TMP_ROOT/README_off.md"
write_readme "$off_readme" 8 2
assert_fails_with "readme-off-by-one" "the suites self-reported 7" \
  run_checker "$GOOD_LOG" "$off_readme"

# ── (c) a suite stopped reporting — MISSING, not zero ───────────────────────
echo "==> (c) suite receipt deleted → RED"
missing_log="$TMP_ROOT/missing.log"
printf 'INJECTIONS=3 SUITE=spec_alpha\n' >"$missing_log"
assert_fails_with "suite-receipt-missing" "emitted no INJECTIONS= line" \
  run_checker "$missing_log" "$GOOD_README"

# ── (d) a suite self-reports zero injections ────────────────────────────────
echo "==> (d) suite reports zero → RED"
zero_log="$TMP_ROOT/zero.log"
printf 'INJECTIONS=3 SUITE=spec_alpha\nINJECTIONS=0 SUITE=spec_beta\n' >"$zero_log"
assert_fails_with "suite-reports-zero" "is not a gate" \
  run_checker "$zero_log" "$GOOD_README"

# ── (e) an unregistered suite appears ───────────────────────────────────────
echo "==> (e) unregistered suite in log → RED"
extra_log="$TMP_ROOT/extra.log"
printf 'INJECTIONS=3 SUITE=spec_alpha\nINJECTIONS=4 SUITE=spec_beta\nINJECTIONS=2 SUITE=spec_rogue\n' \
  >"$extra_log"
assert_fails_with "unregistered-suite" "is not registered" \
  run_checker "$extra_log" "$GOOD_README"

# ── (f) empty log → anti-vacuous ERROR ──────────────────────────────────────
echo "==> (f) empty log → ERROR"
empty_log="$TMP_ROOT/empty.log"
: >"$empty_log"
assert_fails_with "empty-log" "injection log is empty" \
  run_checker "$empty_log" "$GOOD_README"

# ── (g) README advertises no count → anti-vacuous ERROR ─────────────────────
echo "==> (g) README advertises nothing → ERROR"
silent_readme="$TMP_ROOT/README_silent.md"
printf '%s\n' '# Specimen readme with no advertised count at all.' >"$silent_readme"
assert_fails_with "readme-silent" "advertises no known-bad injection count" \
  run_checker "$GOOD_LOG" "$silent_readme"

# ── (h) README miscounts the suites ─────────────────────────────────────────
echo "==> (h) README suite count wrong → RED"
suites_readme="$TMP_ROOT/README_suites.md"
write_readme "$suites_readme" 7 5
assert_fails_with "readme-suite-count" "advertises 5 selftest suites" \
  run_checker "$GOOD_LOG" "$suites_readme"

# ── baseline still GREEN (specimens were the only defect) ───────────────────
assert_green "baseline-restored" run_checker "$GOOD_LOG" "$GOOD_README"

echo "INJECTIONS=$INJ SUITE=$SUITE_NAME"
echo "selftest_injection_count: PASSED (b off-by-one · c missing receipt · d zero · e unregistered · f empty log · g silent README · h suite count)"
exit 0
