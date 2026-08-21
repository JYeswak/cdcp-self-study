#!/usr/bin/env sh
# selftest_injection_count.sh — the two README drift guards, proven to trip.
#
# Contract (mirrors the other selftest_*.sh suites): write real specimen
# receipt logs and README fixtures into TEMP, inject one known-bad at a time,
# assert the guard goes RED with the expected signal. Nothing in the live tree
# is mutated and no patch is ever applied — the specimens are files this script
# writes, so they cannot silently no-op the way `git apply` does on an index
# mismatch.
#
# Two guards live here because they are one mechanism used twice on the same
# README sentence: (a-n, jj) cover `$CDCP_BIN_DIR/cdcp_gate verify-injection-count`,
# which enforces the known-bad injection total and the selftest-suite count;
# (dd-ii) cover the per-suite `n` column in the same table
# [bd-per-suite-injection-column-unguarded-aop9]; (o-cc) cover
# `$CDCP_BIN_DIR/cdcp_gate verify-step-count`, which enforces the third number
# in that sentence — the length of the check.sh chain [bd-1sd.13].
#
# Plants run the Rust binary (same helper contract as check.sh).
# The Rust implementation is the sole gate-path implementation; this selftest
# retains the known-bad drift cases.
#
# Cases (injection count):
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
#  dd) per-suite cell disagrees LOW       → RED (file:line, suite, both numbers)
#  ee) per-suite cell disagrees HIGH      → RED (the other direction)
#  ff) registered suite missing from table → RED
#  gg) unregistered suite row in table    → RED
#  hh) table parses to zero suite rows    → ERROR (anti-vacuous)
#  ii) --write-readme on a table specimen whose receipts are unsound
#      → RED, file byte-unchanged
#  jj) advertisement says known-bad without a shell/selftest qualifier → RED
#
# (ii-control) --write-readme regenerating drifted cells is known-GOOD and is
# NOT counted, on the same terms as (m).
#
# Cases (check.sh step count):
#   o) step receipt log missing           → ERROR
#   p) step receipt log empty             → ERROR (anti-vacuous)
#   q) receipt shape drifted              → ERROR (never a skipped line)
#   r) only a nested DEPTH>0 receipt      → ERROR (never a fallback to the child)
#   s) two DEPTH=0 receipts               → ERROR (never a sum, never last-wins)
#   t) a run that counted ZERO steps      → ERROR (0-vs-0 is the vacuous pass)
#   u) OK + SKIPPED != CHECK_STEPS        → RED (the receipt does not add up)
#   v) NESTED_OK=0                        → ERROR (the hazard never occurred)
#   w) a step added, README untouched     → RED
#   x) README edited, chain untouched     → RED (the direction measured in the wild)
#   y) README advertises no step count    → ERROR
#   z) one step advertisement site gone   → RED on the site floor
#  aa) an ok call BELOW the boundary      → RED (a step that could never be counted)
#  bb) --write-readme on an unsound total → RED, and the file is byte-unchanged
#
# (o-control) baseline, (r-control) a nested receipt beside the outer one, and
# (cc) --write-readme regenerating a drifted README are known-GOOD legs and are
# NOT counted, on the same terms as (i)'s control and (m).
#
# Live-tree pin (wire), not counted: check.sh honours
# CDCP_INJECTION_COUNT_WRITE_README=1 as the only path to --write-readme
# [bd-injection-count-regen-unreachable-lu45]. (wire-unsound) re-proves the
# (n) refusal through that if/else so the flag cannot launder a bogus total.
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
# line that cdcp_gate verify-injection-count aggregates. A suite that stops
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

# Same binary contract as check.sh: honour CARGO_TARGET_DIR, never cargo run.
if [ -z "${CDCP_BIN_DIR:-}" ]; then
  if [ -n "${CARGO_TARGET_DIR:-}" ]; then
    CDCP_BIN_DIR="${CARGO_TARGET_DIR%/}/debug"
  else
    CDCP_BIN_DIR="$ROOT/target/debug"
  fi
fi
[ -n "$CDCP_BIN_DIR" ] \
  || fail "CDCP_BIN_DIR unset — cargo build -p cdcp_gate -p cdcp_cli --locked must run first (no fallback to cargo run)"
[ -x "$CDCP_BIN_DIR/cdcp_gate" ] \
  || fail "cdcp_gate binary absent at $CDCP_BIN_DIR/cdcp_gate — cargo build -p cdcp_gate -p cdcp_cli --locked did not produce it (no fallback to cargo run)"

verify_injection_count() {
  "$CDCP_BIN_DIR/cdcp_gate" verify-injection-count "$@"
}

TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/selftest_injection_count.XXXXXX")"

# specimen README advertising 7 injections across 2 suites
write_readme() {
  cat >"$1" <<EOF
# Specimen readme

[![known-bad (shell selftest suites): $2 injections](https://img.shields.io/badge/known--bad-$2_injections_all_RED-success.svg)](#x)

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
  verify_injection_count --log "$1" --readme "$2" --require "$REQUIRE"
}

# The registry under test is an ARGUMENT, so (k) can hand it a roster that names
# one suite twice without this suite depending on the live roster.
run_checker_require() {
  verify_injection_count --log "$1" --readme "$2" --require "$3"
}

# Write mode. Separate from run_checker so no case can reach it by accident:
# this one REWRITES the file it is pointed at. Honour the lu45 env path:
# check.sh only passes --write-readme when CDCP_INJECTION_COUNT_WRITE_README=1.
run_checker_write() {
  verify_injection_count --log "$1" --readme "$2" --require "$REQUIRE" --write-readme
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

# ── (jj) known-bad advertisement without a shell/selftest qualifier [bd-n7uk]
# The badge is two of the five sites. A reader who sees "known-bad: N
# injections" without "shell" or "selftest" on that line takes N for every
# known-bad in the repo. Other lines stay qualified so this finding is about
# the badge, not a cascade.
echo "==> (jj) advertisement says known-bad without a shell/selftest qualifier → RED"
unqual="$TMP_ROOT/README_unqual.md"
write_readme "$unqual" 7 2
sed 's/known-bad (shell selftest suites):/known-bad:/' \
  "$unqual" >"$unqual.tmp" && mv "$unqual.tmp" "$unqual"
grep -q 'known-bad: 7 injections' "$unqual" \
  || fail "injection (jj) did not apply — specimen drifted"
grep -q 'shell selftest suites' "$unqual" \
  && fail "injection (jj) left a qualifier on the badge line"
assert_fails_with "unqualified-known-bad" \
  "advertises known-bad injections without a shell/selftest qualifier" \
  run_checker "$GOOD_LOG" "$unqual"

# ════════════════════════════════════════════════════════════════════════════
# PER-SUITE n COLUMN [bd-per-suite-injection-column-unguarded-aop9]
# ════════════════════════════════════════════════════════════════════════════
# Specimens only. Two registered selftest_* suites, receipts 6+4=10, five
# advertisement sites so the existing total/site-floor contract stays green
# and the new findings are about the table alone.

COL_REQUIRE="selftest_orphan,selftest_known_bad"
COL_LOG="$TMP_ROOT/col.log"
printf 'INJECTIONS=6 SUITE=selftest_orphan\nINJECTIONS=4 SUITE=selftest_known_bad\n' >"$COL_LOG"

write_column_readme() {
  # $1 path, $2 orphan cell
  cat >"$1" <<EOF
# Specimen readme

[![known-bad (shell selftest suites): 10 injections](https://img.shields.io/badge/known--bad-10_injections_all_RED-success.svg)](#x)

| **Gate** | 2 selftest suites; 10 known-bad injections that must all go RED |

Two selftest suites inject **10 known-bad faults** and assert the build fails.

| **L4 — gates proven to trip** | ok | 2 suites, 10 injections, anti-vacuous |

| Suite | n | Injections |
|---|---|---|
| \`selftest_known_bad\` | 4 | planted |
| \`selftest_orphan\` | $2 | planted |
EOF
}

run_column() {
  verify_injection_count --log "$COL_LOG" --readme "$1" --require "$COL_REQUIRE"
}

run_column_write() {
  verify_injection_count --log "$1" --readme "$2" --require "$COL_REQUIRE" --write-readme
}

echo "==> (dd) per-suite cell disagrees LOW → RED"
col_low="$TMP_ROOT/README_col_low.md"
write_column_readme "$col_low" 5
orphan_ln="$(grep -n '`selftest_orphan`' "$col_low" | head -n1 | cut -d: -f1)"
[ -n "$orphan_ln" ] || fail "(dd) specimen has no selftest_orphan row"
# Basename:line — pathlib normalises TMPDIR's trailing slash, so the full
# path the gate prints is not byte-identical to $col_low. The finding must
# still name the file, the line, the suite, and both numbers.
assert_fails_with "col-cell-low" \
  "README_col_low.md:${orphan_ln} suite selftest_orphan advertises 5 injections; the suite self-reported 6" \
  run_column "$col_low"

echo "==> (ee) per-suite cell disagrees HIGH → RED"
col_high="$TMP_ROOT/README_col_high.md"
write_column_readme "$col_high" 7
orphan_ln_h="$(grep -n '`selftest_orphan`' "$col_high" | head -n1 | cut -d: -f1)"
assert_fails_with "col-cell-high" \
  "README_col_high.md:${orphan_ln_h} suite selftest_orphan advertises 7 injections; the suite self-reported 6" \
  run_column "$col_high"

echo "==> (ff) registered suite missing from the table → RED"
col_miss="$TMP_ROOT/README_col_miss.md"
write_column_readme "$col_miss" 6
grep -v '`selftest_orphan`' "$col_miss" >"$col_miss.tmp" && mv "$col_miss.tmp" "$col_miss"
assert_fails_with "col-row-missing" "has no per-suite table row" \
  run_column "$col_miss"

echo "==> (gg) unregistered suite row in the table → RED"
col_extra="$TMP_ROOT/README_col_extra.md"
write_column_readme "$col_extra" 6
printf '| `selftest_not_a_real_suite` | 1 | planted |\n' >>"$col_extra"
assert_fails_with "col-row-unregistered" "is not in REGISTERED_SUITES" \
  run_column "$col_extra"

echo "==> (hh) table parses to zero suite rows → ERROR (anti-vacuous)"
col_empty="$TMP_ROOT/README_col_empty.md"
write_readme "$col_empty" 10 2
assert_fails_with "col-table-empty" "parsed to zero suite rows" \
  run_column "$col_empty"

echo "==> (ii) --write-readme on unsound receipts + drifted cells → RED, file byte-unchanged"
col_unsound="$TMP_ROOT/README_col_unsound.md"
write_column_readme "$col_unsound" 5
cp "$col_unsound" "$TMP_ROOT/README_col_unsound.before"
col_badlog="$TMP_ROOT/col_unsound.log"
printf 'INJECTIONS=6 SUITE=selftest_orphan\n' >"$col_badlog"
assert_fails_with "col-write-refuses-unsound" "regeneration SKIPPED" \
  run_column_write "$col_badlog" "$col_unsound"
cmp -s "$TMP_ROOT/README_col_unsound.before" "$col_unsound" \
  || fail "(ii) an unsound log rewrote the per-suite cells anyway"
ok "(ii) specimen README byte-unchanged after a refused cell regeneration"

echo "==> (ii-control) --write-readme regenerates drifted cells → GREEN (not counted)"
col_regen="$TMP_ROOT/README_col_regen.md"
write_column_readme "$col_regen" 5
assert_green "col-write-readme-regenerates" run_column_write "$COL_LOG" "$col_regen"
grep -q '| `selftest_orphan` | 6 |' "$col_regen" \
  || fail "(ii-control) the orphan cell was not regenerated from the receipt"
grep -q '| `selftest_orphan` | 5 |' "$col_regen" \
  && fail "(ii-control) a drifted cell survived regeneration"
ok "(ii-control) per-suite cells now carry the receipts"

# ════════════════════════════════════════════════════════════════════════════
# THE SECOND DRIFT GUARD: cdcp_gate verify-step-count [bd-1sd.13]
# ════════════════════════════════════════════════════════════════════════════
# Same shape, one field to the left. README's gate sentence carries three
# numbers; the suite count and the injection count are enforced above, and the
# STEP count was folklore — stale by thirteen for weeks, then wrong again ten
# minutes after a hand correction, through a fully green run.
#
# Specimens only: every case writes its own log, README and script into TEMP.
# Nothing in the live tree is read or mutated by these cases, so they cannot
# pass by accident on a tree that happens to be right.

STEP_GATE_ROOT="$TMP_ROOT/stepcount"
mkdir -p "$STEP_GATE_ROOT"

# A specimen README advertising the step count at FOUR sites, mirroring the
# shipped shape: badge label, badge path, TL;DR row, gate section.
write_step_readme() {
  cat >"$1" <<EOF
# Specimen readme

[![gate: $2 steps](https://img.shields.io/badge/gate-$2_ordered_steps-success.svg)](#the-gate)

| **Gate** | $2 ordered steps; 9 selftest suites |

$2 steps, fail-closed, each naming the script that failed so the repair is obvious.
EOF
}

# A specimen check.sh: two ok sites, one boundary, an emission after it.
write_step_script() {
  {
    echo '#!/usr/bin/env sh'
    echo 'ok "one"'
    echo 'ok "two"'
    echo '# a comment mentioning ok "three" is not a call site'
    echo '# STEP-COUNT-RECEIPT-BOUNDARY'
    echo 'echo CHECK_STEPS=2'
  } >"$1"
}

STEP_LOG_OK="$STEP_GATE_ROOT/steps.log"
printf 'CHECK_STEPS=2 OK=2 SKIPPED=0 NESTED_OK=5 DEPTH=0 RUN=pid1\n' >"$STEP_LOG_OK"
STEP_README="$STEP_GATE_ROOT/README.md"
write_step_readme "$STEP_README" 2
STEP_SCRIPT="$STEP_GATE_ROOT/check.sh"
write_step_script "$STEP_SCRIPT"

run_step_gate() {
  "$CDCP_BIN_DIR/cdcp_gate" verify-step-count \
    --log "$1" --readme "$2" --script "$3"
}

run_step_gate_write() {
  "$CDCP_BIN_DIR/cdcp_gate" verify-step-count \
    --log "$1" --readme "$2" --script "$3" --write-readme
}

assert_step_green() {
  label="$1"
  shift
  rc=0
  out="$("$@" 2>&1)" || rc=$?
  if [ "$rc" -ne 0 ]; then
    printf '%s\n' "$out" >&2
    fail "$label expected GREEN but exited $rc"
  fi
  case "$out" in
    *"step count GREEN"*) ok "$label GREEN" ;;
    *)
      printf '%s\n' "$out" >&2
      fail "$label exited 0 without the 'step count GREEN' receipt"
      ;;
  esac
}

echo "==> (o-control) step gate baseline: receipt and README agree → GREEN (not counted)"
assert_step_green "step-baseline" run_step_gate "$STEP_LOG_OK" "$STEP_README" "$STEP_SCRIPT"

echo "==> (o) step receipt log missing → ERROR"
assert_fails_with "step-log-missing" "step receipt log missing" \
  run_step_gate "$STEP_GATE_ROOT/nope.log" "$STEP_README" "$STEP_SCRIPT"

echo "==> (p) step receipt log empty → ERROR (anti-vacuous)"
step_empty="$STEP_GATE_ROOT/empty.log"
: >"$step_empty"
assert_fails_with "step-log-empty" "step receipt log is empty" \
  run_step_gate "$step_empty" "$STEP_README" "$STEP_SCRIPT"

echo "==> (q) receipt whose shape drifted → ERROR"
step_bad="$STEP_GATE_ROOT/unparseable.log"
printf 'STEPS=2 OK=2 SKIPPED=0 NESTED_OK=5 DEPTH=0 RUN=pid1\n' >"$step_bad"
assert_fails_with "step-receipt-unparseable" "unparseable receipt line" \
  run_step_gate "$step_bad" "$STEP_README" "$STEP_SCRIPT"

# ── the nested-child legs: the count must come from the OUTER run or nowhere ──
echo "==> (r) only a nested (DEPTH>0) receipt → ERROR, never a fallback"
step_nested_only="$STEP_GATE_ROOT/nested_only.log"
printf 'CHECK_STEPS=5 OK=5 SKIPPED=0 NESTED_OK=5 DEPTH=1 RUN=pid9\n' >"$step_nested_only"
assert_fails_with "step-nested-receipt-alone" "no DEPTH=0 receipt" \
  run_step_gate "$step_nested_only" "$STEP_README" "$STEP_SCRIPT"

echo "==> (r-control) a nested receipt BESIDE the outer one → GREEN at the outer number (not counted)"
# This is the contamination case decided rather than hoped: the child reports 5,
# the parent 2, and the answer is 2. A summing gate would say 7 and a
# last-wins gate would say 5; both are numbers no run measured.
step_both="$STEP_GATE_ROOT/both.log"
printf 'CHECK_STEPS=2 OK=2 SKIPPED=0 NESTED_OK=5 DEPTH=0 RUN=pid1\nCHECK_STEPS=5 OK=5 SKIPPED=0 NESTED_OK=0 DEPTH=1 RUN=pid9\n' >"$step_both"
assert_step_green "step-nested-beside-outer" run_step_gate "$step_both" "$STEP_README" "$STEP_SCRIPT"
run_step_gate "$step_both" "$STEP_README" "$STEP_SCRIPT" | grep -q 'nested_receipts_ignored=1' \
  || fail "(r-control) the nested receipt was not reported as ignored"
ok "(r-control) the nested receipt is ignored, not summed (2 wins over 5 and over 7)"

echo "==> (s) two DEPTH=0 receipts in one log → ERROR"
step_two="$STEP_GATE_ROOT/two_outer.log"
printf 'CHECK_STEPS=2 OK=2 SKIPPED=0 NESTED_OK=5 DEPTH=0 RUN=pid1\nCHECK_STEPS=3 OK=3 SKIPPED=0 NESTED_OK=5 DEPTH=0 RUN=pid2\n' >"$step_two"
assert_fails_with "step-two-outer-receipts" "2 DEPTH=0 receipts" \
  run_step_gate "$step_two" "$STEP_README" "$STEP_SCRIPT"

echo "==> (t) a run that counted ZERO steps → ERROR, never a 0-to-0 pass"
# The purest vacuous pass in the family: a counter that silently returns 0,
# compared against a README that was regenerated to 0, is GREEN under any gate
# that only checks equality.
step_zero="$STEP_GATE_ROOT/zero.log"
printf 'CHECK_STEPS=0 OK=0 SKIPPED=0 NESTED_OK=5 DEPTH=0 RUN=pid1\n' >"$step_zero"
step_zero_readme="$STEP_GATE_ROOT/README_zero.md"
write_step_readme "$step_zero_readme" 0
assert_fails_with "step-zero-steps" "counted ZERO ok steps" \
  run_step_gate "$step_zero" "$step_zero_readme" "$STEP_SCRIPT"

echo "==> (u) a receipt whose parts do not add up → RED"
step_sum="$STEP_GATE_ROOT/badsum.log"
printf 'CHECK_STEPS=9 OK=2 SKIPPED=0 NESTED_OK=5 DEPTH=0 RUN=pid1\n' >"$step_sum"
assert_fails_with "step-receipt-arithmetic" "receipt does not add up" \
  run_step_gate "$step_sum" "$STEP_README" "$STEP_SCRIPT"

echo "==> (v) NESTED_OK=0 → ERROR (the non-contamination leg never ran)"
step_nok="$STEP_GATE_ROOT/nonested.log"
printf 'CHECK_STEPS=2 OK=2 SKIPPED=0 NESTED_OK=0 DEPTH=0 RUN=pid1\n' >"$step_nok"
assert_fails_with "step-no-nested-observed" "NESTED_OK=0" \
  run_step_gate "$step_nok" "$STEP_README" "$STEP_SCRIPT"

# ── both drift directions, because the measured failure was the SECOND one ───
echo "==> (w) a step ADDED and README untouched → RED"
step_low="$STEP_GATE_ROOT/README_low.md"
write_step_readme "$step_low" 1
assert_fails_with "step-chain-grew" "this run measured 2" \
  run_step_gate "$STEP_LOG_OK" "$step_low" "$STEP_SCRIPT"

echo "==> (x) README edited and the chain untouched → RED"
step_high="$STEP_GATE_ROOT/README_high.md"
write_step_readme "$step_high" 3
assert_fails_with "step-readme-edited" "advertises 3 check.sh steps" \
  run_step_gate "$STEP_LOG_OK" "$step_high" "$STEP_SCRIPT"

echo "==> (y) README advertises no step count at all → ERROR"
step_silent="$STEP_GATE_ROOT/README_silent.md"
printf '%s\n' '# Specimen readme with no advertised step count at all.' >"$step_silent"
assert_fails_with "step-readme-silent" "advertises no check.sh step count" \
  run_step_gate "$STEP_LOG_OK" "$step_silent" "$STEP_SCRIPT"

echo "==> (z) one advertisement site removed → RED on the site floor, not on drift"
step_thin="$STEP_GATE_ROOT/README_thin.md"
write_step_readme "$step_thin" 2
grep -v 'fail-closed' "$step_thin" >"$step_thin.tmp" && mv "$step_thin.tmp" "$step_thin"
assert_fails_with "step-site-floor" "only 3 step advertisement site(s)" \
  run_step_gate "$STEP_LOG_OK" "$step_thin" "$STEP_SCRIPT"

echo "==> (aa) a step added AFTER the receipt boundary → RED"
# The hole a runtime counter alone leaves: an ok emitted below the boundary can
# never be counted, so the advertised number would stay green while the chain
# grew. Order is decidable from the script text even though the count is not.
step_after="$STEP_GATE_ROOT/check_after.sh"
write_step_script "$step_after"
echo 'ok "added below the boundary"' >>"$step_after"
assert_fails_with "step-ok-after-boundary" "after the STEP-COUNT-RECEIPT-BOUNDARY" \
  run_step_gate "$STEP_LOG_OK" "$STEP_README" "$step_after"

echo "==> (bb) --write-readme on an unsound receipt → RED, file byte-unchanged"
step_unsound_readme="$STEP_GATE_ROOT/README_unsound.md"
write_step_readme "$step_unsound_readme" 3
cp "$step_unsound_readme" "$STEP_GATE_ROOT/README_unsound.before"
assert_fails_with "step-write-refuses-unsound" "regeneration SKIPPED" \
  run_step_gate_write "$step_zero" "$step_unsound_readme" "$STEP_SCRIPT"
cmp -s "$STEP_GATE_ROOT/README_unsound.before" "$step_unsound_readme" \
  || fail "(bb) an unsound step total was written into the specimen README anyway"
ok "(bb) specimen README byte-unchanged after a refused regeneration"

echo "==> (cc) --write-readme regenerates a drifted README → GREEN (not counted)"
step_regen="$STEP_GATE_ROOT/README_regen.md"
write_step_readme "$step_regen" 3
assert_step_green "step-write-readme-regenerates" \
  run_step_gate_write "$STEP_LOG_OK" "$step_regen" "$STEP_SCRIPT"
grep -q 'gate-2_ordered_steps' "$step_regen" \
  || fail "(cc) the badge was not regenerated to the measured step count"
grep -q '3 ordered steps' "$step_regen" \
  && fail "(cc) a drifted step count survived regeneration"
ok "(cc) every step advertisement site now carries the measured total"

# ── check.sh snippet: --write-readme is reachable only via the env flag ────
# [bd-injection-count-regen-unreachable-lu45]
# Live-tree pin plus an executed extract of the if/else. Not counted: (n) already
# owns the unsound-write injection, (b) owns drift-without-write, (m) owns
# regeneration. This pin fails THIS suite if the reachable caller is deleted.
echo "==> (wire) check.sh honours CDCP_INJECTION_COUNT_WRITE_README"

inj_block=$(awk '
  /cdcp_gate verify-injection-count \(advertised known-bad count\)/ {p=1}
  p {print}
  p && /advertised known-bad injection count ==/ {exit}
' scripts/check.sh)
[ -n "$inj_block" ] || fail "(wire) could not extract the verify-injection-count block from check.sh"
printf '%s\n' "$inj_block" | grep -q 'CDCP_INJECTION_COUNT_WRITE_README:-0' \
  || fail "(wire) check.sh does not test CDCP_INJECTION_COUNT_WRITE_README"

inj_if=$(printf '%s\n' "$inj_block" | awk '
  /CDCP_INJECTION_COUNT_WRITE_README/ {p=1}
  p {print}
  p && /^  fi$/ {exit}
')
[ -n "$inj_if" ] || fail "(wire) could not extract the env-flag if/else from check.sh"

probe_dir="$TMP_ROOT/wire"
mkdir -p "$probe_dir"

run_inj_snippet() {
  _flag="$1"
  _out="$2"
  (
    set -eu
    INJ_LOG="$probe_dir/unused.log"
    run_cdcp_gate() { printf '%s\n' "$*" > "$_out"; return 0; }
    fail() { echo "snippet fail: $*" >&2; exit 2; }
    CDCP_INJECTION_COUNT_WRITE_README="$_flag"
    eval "$inj_if"
  )
}

run_inj_snippet "0" "$probe_dir/off.argv"
grep -q 'verify-injection-count' "$probe_dir/off.argv" \
  || fail "(wire) flag=0 did not invoke verify-injection-count"
if grep -q -- '--write-readme' "$probe_dir/off.argv"; then
  fail "(wire) flag=0 passed --write-readme — drift would be auto-rewritten, not RED"
fi
ok "(wire) without the flag, --write-readme is not passed (drift stays RED)"

run_inj_snippet "1" "$probe_dir/on.argv"
grep -q 'verify-injection-count' "$probe_dir/on.argv" \
  || fail "(wire) flag=1 did not invoke verify-injection-count"
grep -q -- '--write-readme' "$probe_dir/on.argv" \
  || fail "(wire) flag=1 did not pass --write-readme — regeneration is still unreachable"
ok "(wire) CDCP_INJECTION_COUNT_WRITE_README=1 passes --write-readme"

# Same if/else shape, real checker: the flag cannot launder an unsound total.
# Not counted — (n) already owns this injection. Reached here so a future
# change that makes the flag skip the refusal is visible next to the wiring.
echo "==> (wire-unsound) flag=1 cannot launder an unsound total"
wire_unsound="$TMP_ROOT/README_wire_unsound.md"
write_readme "$wire_unsound" 8 2
cp "$wire_unsound" "$TMP_ROOT/README_wire_unsound.before"
wire_rc=0
wire_out="$(
  CDCP_INJECTION_COUNT_WRITE_README=1
  INJ_LOG="$missing_log"
  if [ "${CDCP_INJECTION_COUNT_WRITE_README:-0}" = "1" ]; then
    "$CDCP_BIN_DIR/cdcp_gate" verify-injection-count --log "$INJ_LOG" --readme "$wire_unsound" --require "$REQUIRE" --write-readme
  else
    "$CDCP_BIN_DIR/cdcp_gate" verify-injection-count --log "$INJ_LOG" --readme "$wire_unsound" --require "$REQUIRE"
  fi
)" || wire_rc=$?
[ "$wire_rc" -ne 0 ] || fail "(wire-unsound) flag=1 + unsound log exited 0"
printf '%s\n' "$wire_out" | grep -q 'regeneration SKIPPED' \
  || fail "(wire-unsound) missing SKIPPED note"
cmp -s "$TMP_ROOT/README_wire_unsound.before" "$wire_unsound" \
  || fail "(wire-unsound) flag=1 rewrote README from an unsound log"
ok "(wire-unsound) flag cannot launder an unsound total (not counted)"

# ── baseline still GREEN (specimens were the only defect) ───────────────────
assert_green "baseline-restored" run_checker "$GOOD_LOG" "$GOOD_README"
assert_step_green "step-baseline-restored" run_step_gate "$STEP_LOG_OK" "$STEP_README" "$STEP_SCRIPT"

# Prove-it-bites [bd-1sd.13]: deleting one counted assertion must fail THIS
# suite. The parent injection-count guard would also catch a quieter receipt,
# but only if README was left at 63 — a paired edit of suite + README would
# hide the deletion. Pinning the count here makes the deletion red with no
# other file involved.
[ "$INJ" -eq 33 ] || fail "INJ=$INJ; expected 33 — deleting an assertion must fail this suite, not pass with a quieter receipt"
echo "INJECTIONS=$INJ SUITE=$SUITE_NAME"
echo "selftest_injection_count: PASSED (b off-by-one · c missing receipt · d zero · e unregistered · f empty log · g silent README · h suite count · i word-spelled drift · j finding names the scanned file · k duplicate --require · l site floor · m regenerated · n refused unsound write · jj unqualified known-bad · dd cell low · ee cell high · ff missing row · gg unregistered row · hh empty table · ii refused unsound cell write · o step log missing · p step log empty · q receipt shape drift · r nested receipt alone · s two outer receipts · t zero steps · u receipt arithmetic · v no nested observed · w chain grew · x README edited · y README silent · z step site floor · aa ok below the boundary · bb refused unsound step write)"
exit 0
