#!/usr/bin/env sh
# selftest_l5_honesty.sh — L5 gates-proven-to-trip for UI honesty under web/
#
# Plants a forbidden credential-inflation string under web/, asserts the
# honesty scan exits non-zero (RED), restores the plant. In-tree only —
# no git apply patch harness.
#
# Forbidden patterns (same family as check.sh honesty smoke, scoped to web/):
#   you are (now )?CDCP certified
#   officially certified by EPI
#
set -eu

ROOT="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() { echo "selftest_l5_honesty: FAIL: $*" >&2; exit 2; }
ok() { echo "selftest_l5_honesty: ok: $*"; }

# -- L4 drift guard: self-reported RED-injection count ----------------------
# INJ counts the injections this run actually asserted RED (green controls are
# NOT counted). Emitted once, on the success path only, as a machine-readable
# line that scripts/verify_injection_count.py aggregates. A suite that stops
# emitting the line is an ERROR to that gate, never a silent zero.
INJ=0
SUITE_NAME="selftest_l5_honesty"
inject_counted() { INJ=$((INJ + 1)); }

PLANT="web/_selftest_l5_honesty_planted.html"

restore_all() {
  rm -f "$PLANT" 2>/dev/null || true
}
trap restore_all EXIT INT TERM HUP

# Scan web/ for credential inflation. Sets SCAN_RC: 0 clean, 1 hit, 2 error.
# Never `return` non-zero under set -e.
run_web_honesty_scan() {
  SCAN_RC=0
  if ! command -v rg >/dev/null 2>&1; then
    echo "rg required for L5 honesty scan" >&2
    SCAN_RC=2
    return 0
  fi
  hits=""
  rc=0
  # --no-config: ignore broken ~/.ripgreprc type filters (fail-open risk)
  hits="$(rg --no-config -n \
    --glob '*.html' --glob '*.css' --glob '*.js' --glob '*.md' --glob '*.txt' \
    'you are (now )?CDCP certified|officially certified by EPI' \
    web 2>&1)" || rc=$?
  case "$rc" in
    0)
      # Allow negation / documentation of the forbid list in the same file.
      filtered="$(printf '%s\n' "$hits" | rg --no-config -v 'not |never |FORBIDDEN|forbidden|does not|does <strong>not</strong>' || true)"
      if [ -n "$filtered" ]; then
        printf '%s\n' "$filtered"
        SCAN_RC=1
      else
        SCAN_RC=0
      fi
      ;;
    1) SCAN_RC=0 ;; # no matches — clean
    *)
      printf '%s\n' "$hits" >&2
      SCAN_RC=2
      ;;
  esac
  return 0
}

echo "==> selftest_l5_honesty (L5 web credential-inflation gate)"

[ -d web ] || fail "missing web/"
[ -f web/index.html ] || fail "missing web/index.html"
[ -f web/assets/css/course.css ] || fail "missing web/assets/css/course.css"
command -v rg >/dev/null 2>&1 || fail "rg required"

# Token smoke: required VISUAL tokens present in course.css
# Patterns start with "--" — must use -e / -F so rg does not treat them as flags.
for tok in --bg --accent --honesty-bg --wrap-exam --touch-min; do
  rg --no-config -q -F -e "$tok" web/assets/css/course.css \
    || fail "course.css missing token $tok"
done
ok "VISUAL tokens present (--bg --accent --honesty-bg --wrap-exam --touch-min)"

# Banner smoke: honesty copy must deny EPI/EXIN certification
rg --no-config -q 'does.*not.*grant EPI/EXIN certification|does <strong>not</strong> grant EPI/EXIN certification' web/index.html \
  || fail "web/index.html honesty banner missing EPI/EXIN non-grant statement"
ok "index.html honesty banner states no EPI/EXIN certification"

# Clean tree first
run_web_honesty_scan >/dev/null
clean_rc="$SCAN_RC"
[ "$clean_rc" -eq 0 ] || fail "web/ honesty not clean before plant (rc=$clean_rc)"
ok "web/ honesty clean before plant"

# Plant forbidden claim under web/
printf '%s\n' '<p>you are CDCP certified</p>' >"$PLANT"
run_web_honesty_scan >/dev/null
plant_rc="$SCAN_RC"
rm -f "$PLANT"

if [ "$plant_rc" -eq 0 ]; then
  fail "honesty scan stayed green with planted credential-inflation phrase under web/"
fi
if [ "$plant_rc" -ge 2 ]; then
  fail "honesty scan error (rc=$plant_rc) — scanner must not fail-open"
fi
inject_counted
ok "planted certified claim trips RED (rc=$plant_rc)"

# Restore + recheck
run_web_honesty_scan >/dev/null
after_rc="$SCAN_RC"
[ "$after_rc" -eq 0 ] || fail "web/ honesty not clean after restore (rc=$after_rc)"
ok "web/ honesty clean after restore"

echo "INJECTIONS=$INJ SUITE=$SUITE_NAME"
echo "selftest_l5_honesty: PASSED (tokens · banner · plant RED · restore clean)"
exit 0
