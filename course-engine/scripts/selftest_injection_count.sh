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
#   i) a WORD-spelled site drifted        → RED (and one that AGREES → GREEN)
#   j) --readme X names X                 → RED, and never says "README.md:"
#   k) --require names a suite twice      → RED (a double count inflates the total)
#   l) one advertisement site removed     → RED on the site floor, not on drift
#   m) --write-readme on a drifted README → GREEN (regenerated, never typed)
#   n) --write-readme on an unsound log   → RED, and the file is byte-unchanged
#
# (i)'s agreeing-word control and (m) are known-GOOD legs and are NOT counted:
# only assert_fails_with and assert_fails_without increment INJ. A suite that
# only ever attacks ships an over-strict gate, and over-strict gates get routed
# around, which is a slower death than no gate.
#
# The write-mode cases each build a FRESH specimen README, because the gate
# MUTATES the file it is pointed at; reusing one would make the next case
# measure the previous case's rewrite.
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

# assert_fails_without <label> <needle> <forbidden> cmd...
#
# assert_fails_with can only say what a finding CONTAINS. Case (j) is about what
# a finding must NOT contain: when --readme points somewhere else, a finding that
# still says "README.md:" sends the next reader to an innocent file, and the run
# is RED either way — so the positive needle alone cannot tell the two apart.
# Counts as ONE injection, on the same terms as assert_fails_with: only after a
# real RED with the right signal and without the wrong one.
assert_fails_without() {
  label="$1"
  needle="$2"
  forbidden="$3"
  shift 3
  rc=0
  out="$("$@" 2>&1)" || rc=$?
  if [ "$rc" -eq 0 ]; then
    printf '%s\n' "$out" >&2
    fail "expected RED for $label but command exited 0"
  fi
  case "$out" in
    *"$needle"*) ;;
    *)
      printf '%s\n' "$out" >&2
      fail "$label exited $rc but missing expected signal '$needle'"
      ;;
  esac
  case "$out" in
    *"$forbidden"*)
      printf '%s\n' "$out" >&2
      fail "$label exited $rc but its findings still say '$forbidden'"
      ;;
  esac
  inject_counted
  ok "$label trips RED naming the scanned file (rc=$rc, saw: $needle, no: $forbidden)"
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

# The registry under test is an ARGUMENT, so (k) can hand it a roster that names
# one suite twice without this suite depending on the live roster.
run_checker_require() {
  python3 "$CHECKER" --log "$1" --readme "$2" --require "$3"
}

# Write mode. Separate from run_checker so no case can reach it by accident:
# this one REWRITES the file it is pointed at.
run_checker_write() {
  python3 "$CHECKER" --log "$1" --readme "$2" --require "$REQUIRE" --write-readme
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

# ── (i) a count spelled in ENGLISH WORDS is still under the gate ────────────
# The failure this closes is not "no site parses" — that is case (g). It is ONE
# site quietly leaving the scanner while the others still parse: coverage drops
# and the report is indistinguishable from full coverage. Rewriting a site in
# prose must keep it checked, not remove it from the check.
echo "==> (i) word-spelled advertisement drifted → RED"
word_drift="$TMP_ROOT/README_word_drift.md"
write_readme "$word_drift" 7 2
sed 's/\*\*7 known-bad faults\*\*/**thirty-six known-bad faults**/' \
  "$word_drift" >"$word_drift.tmp" && mv "$word_drift.tmp" "$word_drift"
grep -q 'thirty-six' "$word_drift" || fail "injection (i) did not apply — specimen drifted"
assert_fails_with "word-spelled-drift" "advertises 36 known-bad injections" \
  run_checker "$GOOD_LOG" "$word_drift"

# …and the known-GOOD half. A word-spelled site that AGREES stays GREEN, so the
# gate reads prose rather than merely refusing it. NOT counted — assert_green
# does not touch INJ.
echo "==> (i-control) word-spelled advertisement that AGREES → GREEN (not counted)"
word_ok="$TMP_ROOT/README_word_ok.md"
write_readme "$word_ok" 7 2
sed 's/\*\*7 known-bad faults\*\*/**seven known-bad faults**/' \
  "$word_ok" >"$word_ok.tmp" && mv "$word_ok.tmp" "$word_ok"
grep -q 'seven known-bad' "$word_ok" || fail "control (i) did not apply — specimen drifted"
assert_green "word-spelled-agreeing" run_checker "$GOOD_LOG" "$word_ok"

# ── (j) a finding names the README that was actually scanned ────────────────
# It hardcoded "README.md" until bd-wf2, which sent the reader to an innocent
# file whenever --readme pointed elsewhere. Both halves are load-bearing: the
# run is RED either way, so only the ABSENCE of the wrong filename separates the
# fixed gate from the broken one.
echo "==> (j) --readme X names X, and never says README.md → RED"
named_readme="$TMP_ROOT/README_named.md"
write_readme "$named_readme" 8 2
assert_fails_without "readme-named" "README_named.md:3 advertises 8" "README.md:" \
  run_checker "$GOOD_LOG" "$named_readme"

# ── (k) a suite named twice in --require ────────────────────────────────────
# It was summed twice until bd-wf2. An INFLATED measured_total is the one
# direction that turns real drift GREEN, so a caller that does not know its own
# roster is an ERROR, never a silently de-duplicated total.
echo "==> (k) --require names a suite twice → RED"
assert_fails_with "duplicate-require" "more than once" \
  run_checker_require "$GOOD_LOG" "$GOOD_README" "spec_alpha,spec_alpha"

# ── (l) one advertisement site removed → the site FLOOR, not drift ──────────
# Every surviving site still advertises the right number, so nothing here is
# drift; what is lost is coverage. Without the floor this README reports exactly
# like one where all five sites still parse.
echo "==> (l) an advertisement site removed → RED on the site floor"
thin_readme="$TMP_ROOT/README_thin.md"
write_readme "$thin_readme" 7 2
grep -v 'known-bad faults' "$thin_readme" >"$thin_readme.tmp" \
  && mv "$thin_readme.tmp" "$thin_readme"
assert_fails_with "advertisement-site-removed" "only 4 advertisement site(s) parsed" \
  run_checker "$GOOD_LOG" "$thin_readme"

# ── (m) --write-readme REGENERATES a drifted specimen → GREEN ───────────────
# The number is regenerated from the receipts that were actually collected,
# never typed. Fresh specimen: this case rewrites the file it is given.
# NOT counted (assert_green).
echo "==> (m) --write-readme regenerates a drifted README → GREEN (not counted)"
regen_readme="$TMP_ROOT/README_regen.md"
write_readme "$regen_readme" 8 2
assert_green "write-readme-regenerates" run_checker_write "$GOOD_LOG" "$regen_readme"
grep -q 'known--bad-7_injections_all_RED' "$regen_readme" \
  || fail "(m) the badge was not regenerated to the measured total"
grep -q '8 known-bad' "$regen_readme" \
  && fail "(m) a drifted count survived regeneration"
ok "(m) every advertisement site now carries the measured total"

# ── (n) --write-readme REFUSES an unsound total ─────────────────────────────
# spec_beta never reported, so the total is not a number worth writing. A gate
# that wrote it anyway would launder a wrong number into README under its own
# authority — the exact defect this gate exists to remove. Fresh specimen again.
echo "==> (n) --write-readme on an unsound log → RED, file byte-unchanged"
unsound_readme="$TMP_ROOT/README_unsound.md"
write_readme "$unsound_readme" 8 2
cp "$unsound_readme" "$TMP_ROOT/README_unsound.before"
assert_fails_with "write-readme-refuses-unsound" "regeneration SKIPPED" \
  run_checker_write "$missing_log" "$unsound_readme"
cmp -s "$TMP_ROOT/README_unsound.before" "$unsound_readme" \
  || fail "(n) an unsound total was written into the specimen README anyway"
ok "(n) specimen README byte-unchanged after a refused regeneration"

# ── baseline still GREEN (specimens were the only defect) ───────────────────
assert_green "baseline-restored" run_checker "$GOOD_LOG" "$GOOD_README"

echo "INJECTIONS=$INJ SUITE=$SUITE_NAME"
echo "selftest_injection_count: PASSED (b off-by-one · c missing receipt · d zero · e unregistered · f empty log · g silent README · h suite count · i word-spelled drift · j finding names the scanned file · k duplicate --require · l site floor · m regenerated · n refused unsound write)"
exit 0
