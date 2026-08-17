#!/usr/bin/env sh
# assert_generator_fresh.sh — after a generator runs, its owned paths must be clean
# [bd-installability-sm4g.10]
#
# Usage:
#   sh scripts/assert_generator_fresh.sh GENERATOR PATH [PATH...]
#   sh scripts/assert_generator_fresh.sh --selftest
#
# A successful generator that leaves dirt is RED and names the generator plus
# the dirty paths. An empty owned-path set is an ERROR, not a pass. A watch
# path that is missing and untracked cannot go dirty — that is also ERROR.
#
# Reports only. Never stages, never restores the tree.
#
# --selftest plants (NOT via run_selftest — same as N.7 installer / N.16
# learner verbs, so REGISTERED_SUITES and the 72-count do not grow):
#   1) empty path set → RED
#   2) missing untracked path → RED
#   3) dirty owned path in a PRIVATE git repo → RED naming generator + path
# Live tracked files are not written.
set -eu

ROOT="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"

fail() { echo "assert_generator_fresh: FAIL: $*" >&2; exit 2; }
ok() { echo "assert_generator_fresh: ok: $*"; }

repo_root() {
  if [ -n "${CDCP_GENERATOR_FRESH_ROOT:-}" ]; then
    printf '%s\n' "$CDCP_GENERATOR_FRESH_ROOT"
  else
    printf '%s\n' "$ROOT"
  fi
}

# Live path: GENERATOR PATH [PATH...]
assert_fresh() {
  if [ "$#" -lt 1 ]; then
    echo "assert_generator_fresh: FAIL: generator name required" >&2
    return 2
  fi
  _gen=$1
  shift
  if [ "$#" -eq 0 ]; then
    echo "assert_generator_fresh: FAIL: ${_gen}: empty owned-path set is an ERROR, not a pass" >&2
    return 2
  fi
  command -v git >/dev/null 2>&1 \
    || { echo "assert_generator_fresh: FAIL: ${_gen}: git is required to assert generator freshness" >&2; return 2; }

  _repo=$(repo_root)
  [ -n "$_repo" ] || { echo "assert_generator_fresh: FAIL: ${_gen}: empty repo root" >&2; return 2; }

  for _p in "$@"; do
    if [ ! -e "$_repo/$_p" ]; then
      if ! git -C "$_repo" --no-optional-locks ls-files --error-unmatch -- "$_p" >/dev/null 2>&1; then
        echo "assert_generator_fresh: FAIL: ${_gen}: owned path missing and untracked: ${_p} (a watch that cannot go dirty is an ERROR)" >&2
        return 2
      fi
    fi
  done

  _st=0
  _porcelain=$(git -C "$_repo" --no-optional-locks status --porcelain -- "$@" 2>&1) || _st=$?
  if [ "$_st" -ne 0 ]; then
    echo "assert_generator_fresh: FAIL: ${_gen}: git status failed: ${_porcelain}" >&2
    return 2
  fi
  if [ -n "$_porcelain" ]; then
    echo "assert_generator_fresh: FAIL: ${_gen} stale artifacts:" >&2
    printf '%s\n' "$_porcelain" >&2
    return 2
  fi
  return 0
}

# ── --selftest ────────────────────────────────────────────────────────────
if [ "${1:-}" = "--selftest" ]; then
  INJ=0
  SUITE_NAME=generator-freshness
  inject_counted() { INJ=$((INJ + 1)); }

  TMP=
  cleanup() { [ -n "${TMP:-}" ] && rm -rf "$TMP"; return 0; }
  trap cleanup EXIT INT TERM HUP

  CHECK="$ROOT/scripts/check.sh"
  SELF="$ROOT/scripts/assert_generator_fresh.sh"
  [ -f "$CHECK" ] || fail "missing $CHECK"
  [ -f "$SELF" ] || fail "missing $SELF"

  assert_red() {
    _label=$1
    _needle=$2
    shift 2
    _rc=0
    _out=$("$@" 2>&1) || _rc=$?
    if [ "$_rc" -eq 0 ]; then
      printf '%s\n' "$_out" >&2
      fail "expected RED for ${_label} but command exited 0"
    fi
    case $_out in
      *"$_needle"*) ;;
      *)
        printf '%s\n' "$_out" >&2
        fail "expected RED for ${_label} to name '${_needle}' (rc=${_rc})"
        ;;
    esac
    inject_counted
    ok "${_label} trips RED (rc=${_rc}, saw: ${_needle})"
  }

  echo "==> assert_generator_fresh --selftest (empty set · missing path · dirty private repo)"

  # (1) empty owned-path set
  assert_red "empty-path-set" "empty owned-path set" \
    sh "$SELF" build-learn

  # (2) a watch that cannot go dirty
  assert_red "missing-untracked-path" "owned path missing and untracked" \
    sh "$SELF" build-learn this-path-is-not-a-generator-artifact

  # (3) dirty owned path — PRIVATE repo, live tree untouched
  TMP=$(mktemp -d "${TMPDIR:-/tmp}/cdcp-assert-generator-fresh.XXXXXX")
  mkdir -p "$TMP/web/data"
  printf 'committed\n' >"$TMP/web/data/units_index.json"
  (
    CDPATH=
    cd "$TMP" || exit 1
    git init >/dev/null 2>&1
    git config user.email "plant@cdcp.test"
    git config user.name "plant"
    git add web/data/units_index.json
    git commit -m plant >/dev/null
  ) || fail "could not create private git repo for the dirty-path plant"
  printf 'stale\n' >"$TMP/web/data/units_index.json"
  _dirty_out=0
  _dirty=$(CDCP_GENERATOR_FRESH_ROOT="$TMP" sh "$SELF" build-units web/data/units_index.json 2>&1) || _dirty_out=$?
  [ "$_dirty_out" -ne 0 ] || { printf '%s\n' "$_dirty" >&2; fail "dirty owned path stayed GREEN"; }
  case $_dirty in
    *build-units*) ;;
    *)
      printf '%s\n' "$_dirty" >&2
      fail "dirty-path RED did not name the generator 'build-units'"
      ;;
  esac
  case $_dirty in
    *units_index.json*) ;;
    *)
      printf '%s\n' "$_dirty" >&2
      fail "dirty-path RED did not name the dirty path units_index.json"
      ;;
  esac
  inject_counted
  ok "dirty-owned-path trips RED naming build-units and units_index.json"

  # Clean tree in the same private repo is GREEN (control, not an injection).
  printf 'committed\n' >"$TMP/web/data/units_index.json"
  CDCP_GENERATOR_FRESH_ROOT="$TMP" sh "$SELF" build-units web/data/units_index.json \
    || fail "clean private tree went RED"
  ok "clean private tree GREEN"

  # Static anti-vacuity: check.sh must invoke the helper fail-closed after
  # each generator, and must not collect this suite via run_selftest.
  if grep -E 'run_selftest[[:space:]].*assert_generator_fresh' "$CHECK" >/dev/null; then
    fail "check.sh routes assert_generator_fresh through run_selftest — that grows the 72-count"
  fi
  grep -F 'assert_generator_fresh.sh --selftest' "$CHECK" >/dev/null \
    || fail "check.sh does not invoke assert_generator_fresh.sh --selftest"
  for _g in build-learn build-reference build-units build-glossary build-learn-slugs export-web; do
    grep -F "assert_generator_fresh ${_g}" "$CHECK" >/dev/null \
      || fail "check.sh is missing fail-closed freshness assert for ${_g}"
    grep -F "fail \"${_g} stale artifacts\"" "$CHECK" >/dev/null \
      || fail "check.sh is missing fail-closed wiring for ${_g} stale artifacts"
  done
  ok "check.sh wires all six generators fail-closed (not via run_selftest)"

  [ "$INJ" -eq 3 ] || fail "INJ=$INJ; expected 3"
  echo "INJECTIONS=$INJ SUITE=$SUITE_NAME"
  echo "assert_generator_fresh: PASSED (empty set RED · missing path RED · dirty private repo RED naming generator+path)"
  exit 0
fi

assert_fresh "$@"
exit $?
