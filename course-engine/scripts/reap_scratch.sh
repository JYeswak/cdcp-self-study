#!/usr/bin/env sh
# reap_scratch.sh — the process-level backstop for probe scratch trees.
#
# New probes own target/cdcp-scratch/<label>-<pid>-<attempt>.  Rust probes use
# cdcp_registry_check::scratch::ScratchDir; shell probes use the same root and
# EXIT traps.  This reaper handles SIGKILLs, interrupted shells, and the legacy
# direct target/* trees accumulated before bd-qlxp.
#
# Safety boundary: only children of target/cdcp-scratch and direct target
# directories other than the three warm build trees are candidates.  The
# target directory itself and target/debug are never removal targets.
set -eu

ROOT="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
TARGET="$ROOT/target"
POLICY="$ROOT/registries/scratch_lifecycle.toml"
SCRATCH_ROOT="$TARGET/cdcp-scratch"

fail() { echo "scratch-lifecycle: ERROR: $*" >&2; exit 2; }

[ -f "$POLICY" ] || fail "missing $POLICY (size floor is not configured)"
MAX_BYTES="$(sed -n 's/^[[:space:]]*max_bytes[[:space:]]*=[[:space:]]*//p' "$POLICY" \
  | sed 's/[[:space:]]*#.*//' | head -n 1)"
case "$MAX_BYTES" in
  ''|*[!0-9]*) fail "invalid max_bytes in $POLICY: $MAX_BYTES" ;;
esac

mkdir -p "$TARGET" || fail "cannot create $TARGET"

assert_owned_child() {
  _p="$1"
  case "$_p" in
    "$SCRATCH_ROOT"/*) ;;
    *) fail "refusing to remove path outside $SCRATCH_ROOT: $_p" ;;
  esac
}

dir_bytes() {
  # `du -sk` is available on macOS and the Linux runners used for local CI.
  # This is an advisory measurement; failure is not permission to continue
  # silently because the size-floor claim would then be unmeasured.
  _n="$(du -sk "$1" 2>/dev/null | awk 'NR == 1 { print $1 }')" || return 1
  case "$_n" in
    ''|*[!0-9]*) return 1 ;;
    *) echo $((_n * 1024)) ;;
  esac
}

nonvacuous() {
  _discovered="$1"
  _observed="$2"
  if [ "$_observed" -gt 0 ] && [ "$_discovered" -eq 0 ]; then
    echo "scratch-lifecycle: ERROR: reaper discovered zero trees while $_observed scratch entries exist" >&2
    return 2
  fi
  return 0
}

is_kept_scratch_path() {
  _keep_list=":${CDCP_SCRATCH_KEEP:-}:"
  case "$_keep_list" in
    *:"$1":*) return 0 ;;
    *) return 1 ;;
  esac
}

reap_once() {
  mkdir -p "$SCRATCH_ROOT" || fail "cannot create named scratch root $SCRATCH_ROOT"
  [ -d "$SCRATCH_ROOT" ] && [ ! -L "$SCRATCH_ROOT" ] \
    || fail "$SCRATCH_ROOT is not a real directory"

  _discovered=0
  _observed=0
  _bytes_before=0

  # Children in the named root are always owned scratch trees. Hidden names
  # are included so a probe cannot evade cleanup with a dot-prefixed directory.
  for _d in "$SCRATCH_ROOT"/.[!.]* "$SCRATCH_ROOT"/..?* "$SCRATCH_ROOT"/*; do
    if [ ! -e "$_d" ] && [ ! -L "$_d" ]; then
      continue
    fi
    if is_kept_scratch_path "$_d"; then
      continue
    fi
    _observed=$((_observed + 1))
    if [ -L "$_d" ] || [ ! -d "$_d" ]; then
      fail "unmanaged non-directory entry in named scratch root: $_d"
    fi
    _n="$(dir_bytes "$_d")" || fail "cannot measure scratch tree $_d"
    _bytes_before=$((_bytes_before + _n))
    _discovered=$((_discovered + 1))
    assert_owned_child "$_d"
    rm -rf "$_d" || fail "could not remove scratch tree $_d"
  done

  # Migrate the old direct target/* namespace.  Preserve every build tree,
  # not just debug, so the reaper cannot turn a warm build into a cold one.
  for _d in "$TARGET"/.[!.]* "$TARGET"/..?* "$TARGET"/*; do
    if [ ! -e "$_d" ] && [ ! -L "$_d" ]; then
      continue
    fi
    [ -d "$_d" ] && [ ! -L "$_d" ] || continue
    if [ -n "${CDCP_LIFECYCLE_KEEP:-}" ] && [ "$_d" = "$CDCP_LIFECYCLE_KEEP" ]; then
      continue
    fi
    _name="${_d##*/}"
    case "$_name" in
      debug|release|wasm32-unknown-unknown|cdcp-scratch) continue ;;
    esac
    _observed=$((_observed + 1))
    _n="$(dir_bytes "$_d")" || fail "cannot measure legacy scratch tree $_d"
    _bytes_before=$((_bytes_before + _n))
    _discovered=$((_discovered + 1))
    # This check is intentionally direct-child-only; no broad recursive rm.
    case "$_d" in
      "$TARGET"/*) ;;
      *) fail "legacy path escaped target: $_d" ;;
    esac
    rm -rf "$_d" || fail "could not remove legacy scratch tree $_d"
  done

  nonvacuous "$_discovered" "$_observed" || return $?
  _bytes_after="$(dir_bytes "$SCRATCH_ROOT")" || fail "cannot measure $SCRATCH_ROOT after reaping"
  echo "scratch-lifecycle: tree=worktree root=$SCRATCH_ROOT discovered=$_discovered removed=$_discovered bytes-before=$_bytes_before bytes-after=$_bytes_after limit=$MAX_BYTES"
  if [ "$_bytes_before" -gt "$MAX_BYTES" ]; then
    echo "scratch-lifecycle: WARNING: scratch usage $_bytes_before bytes exceeded configured floor $MAX_BYTES bytes before reaping" >&2
  fi
  SCRATCH_DISCOVERED="$_discovered"
  SCRATCH_REMOVED="$_discovered"
}

SELFTEST_PLANT=""
cleanup() {
  if [ -n "${SELFTEST_PLANT:-}" ]; then
    case "$SELFTEST_PLANT" in
      "$SCRATCH_ROOT"/*) rm -rf "$SELFTEST_PLANT" 2>/dev/null || true ;;
    esac
  fi
}
trap cleanup EXIT INT TERM HUP

case "${1:-}" in
  --selftest)
    # First reap proves the existing-tree path; then plant a fresh owned tree
    # and exercise the same function again.  The direct assertion below is the
    # fail-closed leg: zero discovered while an entry exists is ERROR.
    reap_once
    SELFTEST_PLANT="$SCRATCH_ROOT/reaper-known-bad-$$"
    mkdir -p "$SELFTEST_PLANT/tree" || fail "cannot plant reaper fixture"
    printf '%s\n' planted >"$SELFTEST_PLANT/tree/payload"
    reap_once
    [ "$SCRATCH_DISCOVERED" -gt 0 ] \
      || fail "reaper selftest found zero planted trees"
    [ ! -e "$SELFTEST_PLANT" ] \
      || fail "reaper selftest left planted tree $SELFTEST_PLANT"
    if nonvacuous 0 1; then
      fail "reaper selftest accepted zero discovered trees while one exists"
    fi
    if [ -e "$TARGET/debug" ]; then
      [ -d "$TARGET/debug" ] \
        || fail "reaper selftest changed target/debug (warm build must be preserved)"
    else
      echo "scratch-lifecycle: target/debug absent in this fresh probe tree; there was no warm build to remove"
    fi
    SELFTEST_PLANT=""
    echo "scratch-lifecycle: selftest GREEN (planted tree removed; zero-find leg ERRORed; target/debug preserved)"
    ;;
  '')
    reap_once
    ;;
  *)
    fail "usage: scripts/reap_scratch.sh [--selftest]"
    ;;
esac
