#!/usr/bin/env sh
# selftest_learner_verbs.sh — L4 plants for study / demo / test
# [bd-installability-sm4g.16] [bd-installability-sm4g.17]
#
# BUILT != WIRED. The verbs shipped; check.sh did not invoke them.
# This suite proves the live wrappers are fail-closed:
#   1) missing/corrupt wasm → `cdcp test` RED, names test/wasm
#   2) missing bundle      → `cdcp demo` RED, names the verb / bundle
#   3) missing bundle      → `cdcp study` exit 4, names bundle
#   4) ignore-exit is RED  — check.sh must contain `run_cdcp_cli <verb> || fail`
#      A wrapper that runs the verb and continues is the vacuous pass already
#      shipped once (build-learn as a generator, .10).
#   5) study stop reaps the *cdcp* process. `run_cdcp_cli study … &` then
#      `kill $!` kills the function subshell; the child reparents to PID 1
#      and the listener stays up (POST_KILL_HTTP=200). Plant is RED if
#      check.sh is reverted to that stop.
#
# Not via run_selftest: same as selftest_install.sh (N.7). Registering a new
# suite grows cdcp_gate past gate_shrink and moves the 72-count. Plants still
# fire. Does not mutate the live tree.
set -eu

ROOT="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() { echo "selftest_learner_verbs: FAIL: $*" >&2; exit 2; }
ok() { echo "selftest_learner_verbs: ok: $*"; }

INJ=0
SUITE_NAME=learner_verbs
inject_counted() { INJ=$((INJ + 1)); }

CDCP="${CDCP_BIN_DIR:-$ROOT/target/debug}/cdcp"
[ -x "$CDCP" ] || fail "cdcp binary missing at $CDCP — check.sh must build first"

CHECK="$ROOT/scripts/check.sh"
[ -f "$CHECK" ] || fail "missing $CHECK"

TMP=
STUDY_REAP=
study_reap_port() {
  _rp=$1
  [ -n "$_rp" ] || return 0
  # Occupied 8766 is a long-lived `cdcp serve`. Never our ephemeral :0 bind.
  [ "$_rp" != "8766" ] || return 0
  _rpids=$(lsof -nP -iTCP:"$_rp" -sTCP:LISTEN -t 2>/dev/null || true)
  for _rp_pid in $_rpids; do
    kill "$_rp_pid" 2>/dev/null || true
  done
  return 0
}
cleanup() {
  if [ -n "${STUDY_REAP:-}" ]; then
    for _cp in $STUDY_REAP; do
      study_reap_port "$_cp"
    done
  fi
  [ -n "${TMP:-}" ] && rm -rf "$TMP"
  return 0
}
trap cleanup EXIT INT TERM HUP
TMP=$(mktemp -d "${TMPDIR:-/tmp}/cdcp-selftest-learner-verbs.XXXXXX")

assert_red() {
  label=$1
  needle=$2
  shift 2
  rc=0
  out=$("$@" 2>&1) || rc=$?
  if [ "$rc" -eq 0 ]; then
    printf '%s\n' "$out" >&2
    fail "expected RED for $label but command exited 0"
  fi
  case "$out" in
    *"$needle"*) ;;
    *)
      printf '%s\n' "$out" >&2
      fail "expected RED for $label to name '$needle' (rc=$rc)"
      ;;
  esac
  inject_counted
  ok "$label trips RED (rc=$rc, saw: $needle)"
}

require_wired() {
  verb=$1
  pattern=$2
  # The live step must invoke the helper AND fail closed. A line that runs
  # the verb and ignores the status is the vacuous pass.
  if ! grep -F "$pattern" "$CHECK" >/dev/null 2>&1; then
    fail "check.sh is missing fail-closed wiring for $verb (expected: $pattern)"
  fi
  if grep -E "run_cdcp_cli[[:space:]]+$verb([^[:alnum:]_-].*)?[[:space:]]*\\|\\|[[:space:]]*true" "$CHECK" >/dev/null 2>&1; then
    fail "check.sh runs $verb and ignores its exit code (\`|| true\`) — vacuous pass"
  fi
  inject_counted
  ok "check.sh wires $verb fail-closed ($pattern)"
}

# ── 4) static anti-vacuity (must fire even if plants cannot spawn) ────────
require_wired "test" 'run_cdcp_cli test || fail "cdcp test"'
require_wired "demo" 'run_cdcp_cli demo --no-open || fail "cdcp demo"'
if ! grep -F 'fail "cdcp study"' "$CHECK" >/dev/null 2>&1; then
  fail "check.sh is missing fail-closed wiring for study (expected: fail \"cdcp study\")"
fi
if ! grep -E 'curl .*cdcp study|cdcp study.*curl|_study_code' "$CHECK" >/dev/null 2>&1; then
  fail "check.sh study step never curls the bound URL — printed-URL-only is vacuous"
fi
inject_counted
ok "check.sh study step curls the bound URL and fail-closes"

# ── plant trees ───────────────────────────────────────────────────────────
WEB_SRC="$ROOT/web"
[ -f "$WEB_SRC/index.html" ] || fail "missing $WEB_SRC/index.html"
[ -f "$WEB_SRC/data/mock40_seed42.json" ] || fail "missing seed-42 pack"
[ -f "$WEB_SRC/assets/wasm/cdcp_wasm.wasm" ] || fail "missing shipped wasm"

copy_seed42() {
  dest=$1
  mkdir -p "$dest/web/data" "$dest/web/assets/wasm"
  cp "$WEB_SRC/index.html" "$dest/web/index.html"
  cp "$WEB_SRC/data/mock40_seed42.json" "$dest/web/data/mock40_seed42.json"
  cp "$WEB_SRC/data/bank_items_seed42.json" "$dest/web/data/bank_items_seed42.json"
  cp "$WEB_SRC/data/keys_seed42.json" "$dest/web/data/keys_seed42.json"
}

# 1) test: seed-42 assets present, wasm is not a wasm → RED naming test/wasm
TEST_PLANT="$TMP/test-nowasm"
copy_seed42 "$TEST_PLANT"
printf 'notw' >"$TEST_PLANT/web/assets/wasm/cdcp_wasm.wasm"
assert_red "test missing-wasm" "test" \
  env -u CDCP_HOME -u CDCP_REPO_ROOT \
  "$CDCP" test --root "$TEST_PLANT"

# Prove ignore-exit would GREEN this plant (the .10 defect). Then require
# the real wrapper is not that pattern — already asserted above — and that
# the plant itself is actually RED (already asserted). This block exists so
# a future editor who deletes assert_red and keeps only the grep still has
# a live RED against the verb.
vac_rc=0
"$CDCP" test --root "$TEST_PLANT" >/dev/null 2>&1 || vac_rc=$?
if [ "$vac_rc" -eq 0 ]; then
  fail "test plant is GREEN — the known-bad no longer breaks the verb"
fi
# Vacuous wrapper: run + ignore. This MUST exit 0. If it does not, the
# demonstration that ignore-exit hides the plant is itself broken.
if ! ( "$CDCP" test --root "$TEST_PLANT" >/dev/null 2>&1 || true ); then
  fail "vacuous \`|| true\` wrapper did not stay GREEN — cannot certify anti-vacuity"
fi
inject_counted
ok "ignore-exit would GREEN the test plant (|| true stays 0); real wiring does not"

# 2) demo: no web/index.html → RED naming bundle / demo
DEMO_PLANT="$TMP/demo-empty"
mkdir -p "$DEMO_PLANT/empty"
assert_red "demo missing-bundle" "bundle not found" \
  env -u CDCP_HOME -u CDCP_REPO_ROOT \
  "$CDCP" demo --no-open --root "$DEMO_PLANT/empty"

# 3) study: same missing bundle → exit 4, does not block
STUDY_PLANT="$TMP/study-empty"
mkdir -p "$STUDY_PLANT/empty"
study_rc=0
study_out=$(
  env -u CDCP_HOME -u CDCP_REPO_ROOT \
    "$CDCP" study --no-open --bind 127.0.0.1:0 --root "$STUDY_PLANT/empty" 2>&1
) || study_rc=$?
if [ "$study_rc" -eq 0 ]; then
  printf '%s\n' "$study_out" >&2
  fail "study missing-bundle exited 0 (must be 4)"
fi
if [ "$study_rc" -ne 4 ]; then
  printf '%s\n' "$study_out" >&2
  fail "study missing-bundle rc=$study_rc (must be 4)"
fi
case "$study_out" in
  *"bundle not found"*) ;;
  *)
    printf '%s\n' "$study_out" >&2
    fail "study missing-bundle did not name 'bundle not found'"
    ;;
esac
inject_counted
ok "study missing-bundle exits 4 naming bundle (did not block)"

# ── 5) study stop must reap cdcp, not the function subshell (N.17) ─────
# Known-bad: background a function that runs cdcp as a child, kill $!,
# listener survives. This MUST still leak — if it does not, the plant is
# uncalibrated and a revert of check.sh to function-$! would read GREEN.
# Then start the way the live wrapper must start (binary, so $! is cdcp),
# kill that PID, and assert the bound port is dead.
# Static: check.sh must not background run_cdcp_cli study (unless it kills
# the process group) and must fail-close when post-kill HTTP is still 200.
command -v curl >/dev/null 2>&1 \
  || fail "curl required to prove the study listener died (printed-URL-only is vacuous)"

wait_study_url() {
  _wlog=$1
  _wpid=$2
  _wi=0
  _wurl=""
  while [ "$_wi" -lt 50 ]; do
    if ! kill -0 "$_wpid" 2>/dev/null; then
      wait "$_wpid" 2>/dev/null || true
      cat "$_wlog" >&2
      return 1
    fi
    _wurl=$(sed -n 's/.*cdcp study: \(http:\/\/[^[:space:]]*\).*/\1/p' "$_wlog" | awk 'NR==1{print; exit}')
    if [ -n "$_wurl" ]; then
      printf '%s\n' "$_wurl"
      return 0
    fi
    _wi=$((_wi + 1))
    sleep 0.2
  done
  cat "$_wlog" >&2
  return 1
}

study_port_of() {
  printf '%s' "$1" | sed -n 's|.*:\([0-9][0-9]*\)/.*|\1|p'
}

# (a) known-bad still leaks — plant is calibrated
leaky_wrap() {
  env -u CDCP_HOME -u CDCP_REPO_ROOT \
    "$CDCP" study --no-open --bind 127.0.0.1:0
}
leak_log="$TMP/study-leak.log"
set +e
leaky_wrap >"$leak_log" 2>&1 &
leak_wrap_pid=$!
set -e
leak_url=$(wait_study_url "$leak_log" "$leak_wrap_pid") \
  || fail "known-bad function-\$! plant did not bind (see log above)"
leak_port=$(study_port_of "$leak_url")
if [ -z "$leak_port" ] || [ "$leak_port" = "8766" ]; then
  kill "$leak_wrap_pid" 2>/dev/null || true
  wait "$leak_wrap_pid" 2>/dev/null || true
  fail "known-bad function-\$! plant bound occupier port or no port (url=$leak_url)"
fi
STUDY_REAP="$STUDY_REAP $leak_port"
kill "$leak_wrap_pid" 2>/dev/null || true
wait "$leak_wrap_pid" 2>/dev/null || true
leak_post=$(curl -fsS -o /dev/null -w "%{http_code}" --connect-timeout 1 --max-time 1 "$leak_url" 2>/dev/null || true)
if [ "$leak_post" != "200" ]; then
  study_reap_port "$leak_port"
  fail "known-bad function-\$! no longer leaks (POST_KILL_HTTP=$leak_post) — plant uncalibrated"
fi
study_reap_port "$leak_port"
inject_counted
ok "known-bad function-\$! still leaks (POST_KILL_HTTP=200); plant is calibrated"

# (b) process is actually gone: $! is the cdcp PID
good_log="$TMP/study-good.log"
set +e
env -u CDCP_HOME -u CDCP_REPO_ROOT \
  "$CDCP" study --no-open --bind 127.0.0.1:0 >"$good_log" 2>&1 &
good_pid=$!
set -e
good_url=$(wait_study_url "$good_log" "$good_pid") \
  || fail "exec-start plant did not bind (see log above)"
good_port=$(study_port_of "$good_url")
if [ -z "$good_port" ] || [ "$good_port" = "8766" ]; then
  kill "$good_pid" 2>/dev/null || true
  wait "$good_pid" 2>/dev/null || true
  fail "exec-start plant bound occupier port or no port (url=$good_url)"
fi
STUDY_REAP="$STUDY_REAP $good_port"
good_pre=$(curl -fsS -o /dev/null -w "%{http_code}" --connect-timeout 1 --max-time 1 "$good_url" 2>/dev/null || true)
if [ "$good_pre" != "200" ]; then
  kill "$good_pid" 2>/dev/null || true
  wait "$good_pid" 2>/dev/null || true
  study_reap_port "$good_port"
  fail "exec-start plant did not serve 200 (got $good_pre)"
fi
# The live wrapper's stop: kill the cdcp PID (not a function subshell).
kill "$good_pid" 2>/dev/null || true
wait "$good_pid" 2>/dev/null || true
good_post=$(curl -fsS -o /dev/null -w "%{http_code}" --connect-timeout 1 --max-time 1 "$good_url" 2>/dev/null || true)
if [ "$good_post" = "200" ]; then
  study_reap_port "$good_port"
  fail "cdcp-pid kill left listener up (POST_KILL_HTTP=200) — process is not gone"
fi
if command -v lsof >/dev/null 2>&1; then
  leftover=$(lsof -nP -iTCP:"$good_port" -sTCP:LISTEN 2>/dev/null || true)
  if [ -n "$leftover" ]; then
    study_reap_port "$good_port"
    fail "cdcp-pid kill left lsof listener on $good_port: $leftover"
  fi
fi
inject_counted
ok "process is actually gone after cdcp-pid kill (POST_KILL_HTTP!=200, port $good_port clear)"

# (c) static: RED if check.sh stop is reverted to function-$!
# Command lines only — a comment that names the defect (`run_cdcp_cli study … &`)
# is not a start.
if grep -E '^[[:space:]]*run_cdcp_cli[[:space:]]+study' "$CHECK" | grep '&' >/dev/null 2>&1; then
  # Process-group kill of the wrapper is an allowed alternative to invoking
  # the binary. `kill -- -$pid` / `kill -TERM -$pid` reaps the child too.
  if ! grep -E 'kill[[:space:]]+--[[:space:]]+-"\$_study_(pid|pgid)"' "$CHECK" >/dev/null 2>&1 \
     && ! grep -E 'kill[[:space:]]+-TERM[[:space:]]+-"\$_study_(pid|pgid)"' "$CHECK" >/dev/null 2>&1 \
     && ! grep -E 'kill[[:space:]]+-s[[:space:]]+TERM[[:space:]]+-"\$_study_(pid|pgid)"' "$CHECK" >/dev/null 2>&1; then
    fail "check.sh backgrounds run_cdcp_cli study without a process-group kill — function-\$! leak"
  fi
fi
if ! grep -F '"$CDCP_BIN_DIR/cdcp" study --no-open --bind 127.0.0.1:0' "$CHECK" >/dev/null 2>&1; then
  fail "check.sh study start does not invoke \$CDCP_BIN_DIR/cdcp (stop cannot target the listener)"
fi
if ! grep -F '_study_post=$(curl' "$CHECK" >/dev/null 2>&1; then
  fail "check.sh study stop never curls the bound URL after kill — stop is unproven"
fi
if ! grep -F '"$_study_post" = "200"' "$CHECK" >/dev/null 2>&1; then
  fail "check.sh study stop never fail-closes when post-kill HTTP is still 200"
fi
inject_counted
ok "check.sh study start invokes cdcp (not the function) and fail-closes if stop leaves HTTP 200"

if [ "$INJ" -lt 10 ]; then
  fail "suite asserted $INJ RED(s); floor is 10 (3 static wirings + study-curl + 3 plants + ignore-exit + leak-calibrated + process-gone + stop-wiring)"
fi

echo "INJECTIONS=$INJ SUITE=$SUITE_NAME"
echo "selftest_learner_verbs: PASSED ($INJ injections)"
