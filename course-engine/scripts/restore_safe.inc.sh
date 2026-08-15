#!/usr/bin/env sh
# restore_safe.inc.sh — THE restore helper that cannot leave cargo a stale mtime.
#
# Source:  . scripts/restore_safe.inc.sh
#          cdcp_restore_safe DEST BACKUP
# Run:     sh scripts/restore_safe.inc.sh
#          (known-bad mv-skip is RED, helper rebuilds, scan of converted sites)
#
# ─────────────────────────────────────────────────────────────────────────────
# THE TRAP (bd-stale-binary-mtime-trap-p65w)
# ─────────────────────────────────────────────────────────────────────────────
#
#   cp file file.bak          # backup inode, mtime = then
#   # …perturb file, cargo compiles the PERTURBED source…
#   mv file.bak file          # dest IS the backup inode: content is right,
#                             # mtime is OLDER than the perturbed compile
#
# Cargo decides what to rebuild by comparing mtimes. It sees nothing newer
# than the artifact it already built, skips, and the next invocation runs
# the binary compiled from the PERTURBED source. Measured 2026-08-14 as a
# false RED (goldens/PROVENANCE.md). The same mechanism is a FALSE GREEN
# when what you restore is the assertion you deleted in leg 2 of a
# meta-test pair. A meta-test that cannot fail is a fooled certificate.
#
# `mv backup dest` ALONE is forbidden in any restore path this helper owns.
#
# ─────────────────────────────────────────────────────────────────────────────
# THE RULE
# ─────────────────────────────────────────────────────────────────────────────
#
#   Restore by writing the original bytes INTO THE EXISTING FILE, then
#   touch it. That is this function. There is no second step to forget.
#
#   Accepted forms (all leave dest newer than the perturbed compile):
#     • in-place write of original bytes   (cat BACKUP > DEST)
#     • that write + explicit touch        (what this helper does)
#     • copy over the inode                (cp BACKUP DEST, never cp -p)
#
#   Forbidden: `mv BACKUP DEST`, `git mv`, any rename of the backup over
#   dest. `touch` after a rename is the RECOVERY move, not the pattern —
#   its omission is silent, which is the failure class this file closes.
#
#   Do NOT "prove" freshness with artifact_mtime > source_mtime. The
#   poisoned tree satisfies that ordering. Compare the artifact's mtime
#   BEFORE a build with its mtime AFTER, or (for this helper itself)
#   compare dest's mtime to the aged backup's mtime.
#
# Rust authors: crates/cdcp_gate/tests/support/rebuild.rs is the same
# argument with Restorable::restore + build_proving_rebuild. The agent-facing
# receipt is this file's prove-rebuild subcommand (bd-stale-artifact-gate-urj0):
#
#   sh scripts/restore_safe.inc.sh prove-rebuild --artifact <bin> -- \
#       cargo test -p <crate> --offline --no-run
#
# EXTRACT-THEN-DELETE (bd-extract-restore-safe-python-iiv8): first-level
# python3 is retired. mtime-ns is `cdcp recon mtime-ns`; replace-once is
# `cdcp snap-rewrite replace-once`; the prove-rebuild CHARTER weaken is
# `cdcp snap-rewrite charter --kind weaken-if`. Missing $CDCP is RED —
# no interpreter fallback, no cargo run.
#
# ─────────────────────────────────────────────────────────────────────────────

# cdcp_restore_safe DEST BACKUP
#   Write BACKUP's bytes into DEST (same inode if DEST exists) and touch
#   DEST so its mtime is now. Never renames. Never cp -p.
cdcp_restore_safe() {
  _dest="${1:-}"
  _bak="${2:-}"
  if [ -z "$_dest" ] || [ -z "$_bak" ]; then
    echo "restore_safe: usage: cdcp_restore_safe DEST BACKUP" >&2
    return 2
  fi
  if [ ! -f "$_bak" ]; then
    echo "restore_safe: missing backup: $_bak" >&2
    return 2
  fi
  if [ "$_dest" = "$_bak" ]; then
    echo "restore_safe: DEST and BACKUP are the same path: $_dest" >&2
    return 2
  fi
  # In-place write: open DEST for truncate+write so the inode stays and
  # the mtime becomes now. `mv` would hand DEST the backup's inode and
  # its older mtime; that is the trap, and it is not expressible here.
  cat "$_bak" >"$_dest" || return 2
  # Same function, not a second step: even if a future edit "tidies"
  # the write into a rename, touch still fires. Omission is not silent.
  touch "$_dest" || return 2
}

# cdcp_prove_rebuild --artifact PATH -- CMD...
#   Record PATH's mtime, run CMD, require PATH's mtime to have moved.
#   That is the only freshness proof that the poisoned tree fails:
#   after `mv backup dest` cargo exits 0, prints Finished in 0.00s, and
#   rebuilds nothing — a forced cargo build is itself vacuous.
#   Missing PATH is ERROR (empty/absent is not "nothing to check").
#   Unchanged mtime after CMD is FAIL (ANTI-VACUOUS).
cdcp_prove_rebuild() {
  _pr_art=""
  while [ $# -gt 0 ]; do
    case "$1" in
      --artifact)
        if [ $# -lt 2 ]; then
          echo "prove-rebuild: USAGE: --artifact needs a path" >&2
          return 2
        fi
        _pr_art="$2"
        shift 2
        ;;
      --)
        shift
        break
        ;;
      -h|--help)
        echo "usage: cdcp_prove_rebuild --artifact PATH -- CMD..." >&2
        return 2
        ;;
      *)
        echo "prove-rebuild: USAGE: unknown argument $1 (known: --artifact --)" >&2
        return 2
        ;;
    esac
  done
  if [ -z "$_pr_art" ]; then
    echo "prove-rebuild: USAGE: --artifact PATH is required (an absent artifact is an ERROR, not 'nothing to check')" >&2
    return 2
  fi
  if [ $# -eq 0 ]; then
    echo "prove-rebuild: USAGE: missing build command after --" >&2
    return 2
  fi
  if [ ! -f "$_pr_art" ]; then
    echo "prove-rebuild: ERROR: artifact missing: $_pr_art (empty/absent is an ERROR, not a pass)" >&2
    return 1
  fi
  _pr_before="$(_cdcp_mtime_ns "$_pr_art")"
  _pr_rc=0
  "$@" || _pr_rc=$?
  if [ "$_pr_rc" -ne 0 ]; then
    echo "prove-rebuild: ERROR: build command failed rc=$_pr_rc" >&2
    return 1
  fi
  if [ ! -f "$_pr_art" ]; then
    echo "prove-rebuild: ERROR: artifact vanished after build: $_pr_art" >&2
    return 1
  fi
  _pr_after="$(_cdcp_mtime_ns "$_pr_art")"
  # CHARTER-NEEDLE-CHECK — the pair mutates this comparison to `if false &&`.
  if [ "$_pr_after" -eq "$_pr_before" ]; then
    echo "prove-rebuild: FAIL: ANTI-VACUOUS — rebuilt nothing (artifact mtime still $_pr_before). The binary on disk was built from the PERTURBED source; any verdict read from it is fabricated." >&2
    return 1
  fi
  echo "prove-rebuild: ok: artifact mtime $_pr_before -> $_pr_after"
  return 0
}

# ── internals for the demonstration / scan (used when executed, or called) ──

_cdcp_restore_safe_fail() {
  echo "restore_safe: FAIL: $*" >&2
  return 1
}

_cdcp_restore_bin=""

# Resolve the LIVE helper binary. Honour CDCP, else CDCP_BIN_DIR, else
# CARGO_TARGET_DIR/debug, else the executed-form engine target. Missing
# is RED — the retired python3 path is not a fallback.
_cdcp_resolve() {
  if [ -n "${CDCP:-}" ] && [ -x "$CDCP" ]; then
    printf '%s\n' "$CDCP"
    return 0
  fi
  if [ -n "${CDCP_BIN_DIR:-}" ] && [ -x "${CDCP_BIN_DIR%/}/cdcp" ]; then
    printf '%s\n' "${CDCP_BIN_DIR%/}/cdcp"
    return 0
  fi
  if [ -n "${CARGO_TARGET_DIR:-}" ] && [ -x "${CARGO_TARGET_DIR%/}/debug/cdcp" ]; then
    printf '%s\n' "${CARGO_TARGET_DIR%/}/debug/cdcp"
    return 0
  fi
  if [ -n "${_CDCP_ENGINE_ROOT:-}" ] && [ -x "${_CDCP_ENGINE_ROOT}/target/debug/cdcp" ]; then
    printf '%s\n' "${_CDCP_ENGINE_ROOT}/target/debug/cdcp"
    return 0
  fi
  return 1
}

_cdcp_cli() {
  if [ -z "${_cdcp_restore_bin}" ]; then
    _cdcp_restore_bin="$(_cdcp_resolve)" || {
      echo "restore_safe: cdcp binary absent — cargo build -p cdcp_cli --locked must run first (no fallback to python3 or cargo run)" >&2
      return 2
    }
  fi
  "$_cdcp_restore_bin" "$@"
}

_cdcp_mtime_ns() {
  _cdcp_cli recon mtime-ns "$1"
}

# Roster of house-pattern selftests that restore cargo-compiled sources
# through this helper. A scan that finds zero of these is an ERROR — a
# helper nobody calls cannot stop a stale binary from being blessed.
#
# Add a line when you convert another cargo-touching restore site.
# Shell smokes that only write TEMP specimens are NOT this bug.
CDCP_CONVERTED_RESTORE_SITES="scripts/selftest_reconstructed.sh"

# Anti-vacuous scan. Zero converted sites → ERROR.
# A cargo-touching selftest_*.sh that still restores without this helper → ERROR.
cdcp_restore_safe_scan() {
  _root="${1:-}"
  if [ -z "$_root" ]; then
    echo "restore_safe: scan needs the engine root" >&2
    return 2
  fi
  _n=0
  _missing=0
  for _f in $CDCP_CONVERTED_RESTORE_SITES; do
    if [ ! -f "$_root/$_f" ]; then
      echo "restore_safe: claimed converted site missing: $_f" >&2
      _missing=$((_missing + 1))
      continue
    fi
    if ! grep -q 'restore_safe.inc.sh' "$_root/$_f"; then
      echo "restore_safe: $_f does not source restore_safe.inc.sh" >&2
      _missing=$((_missing + 1))
      continue
    fi
    if ! grep -q 'cdcp_restore_safe ' "$_root/$_f"; then
      echo "restore_safe: $_f does not call cdcp_restore_safe" >&2
      _missing=$((_missing + 1))
      continue
    fi
    echo "restore_safe: converted: $_f"
    _n=$((_n + 1))
  done
  if [ "$_n" -eq 0 ]; then
    echo "restore_safe: FAIL: ANTI-VACUOUS — zero converted restore sites" >&2
    return 1
  fi
  if [ "$_missing" -ne 0 ]; then
    echo "restore_safe: FAIL: $_missing claimed site(s) did not convert" >&2
    return 1
  fi

  # Discover remaining cargo-touching selftests that do not source us.
  _remain=0
  for _f in "$_root"/scripts/selftest_*.sh; do
    [ -f "$_f" ] || continue
    grep -q 'restore_safe.inc.sh' "$_f" && continue
    if grep -E -q 'crates/[^[:space:]"'\'']+\.rs' "$_f"; then
      echo "restore_safe: REMAINING cargo-touching restore site: ${_f#"$_root"/}" >&2
      _remain=$((_remain + 1))
    fi
  done
  if [ "$_remain" -ne 0 ]; then
    echo "restore_safe: FAIL: $_remain unconverted cargo-touching selftest(s)" >&2
    return 1
  fi
  echo "restore_safe: scan ok ($_n converted, 0 remaining cargo-touching selftests)"
}

# Known-bad mtime demonstration, no cargo. Restore-via-mv inherits the
# backup's aged mtime (the trap). The helper leaves dest newer than that
# backup. Clock-resolution safe: we compare dest to the aged backup, never
# dest to an artifact.
cdcp_restore_safe_mtime_demo() {
  # No trap: this file is sourced by selftests that already own EXIT.
  # A RETURN/EXIT trap here would replace theirs and leave their tree dirty.
  _d="$(mktemp -d "${TMPDIR:-/tmp}/cdcp_restore_safe.XXXXXX")"
  if ! _cdcp_restore_safe_mtime_demo_body "$_d"; then
    rm -rf "$_d"
    return 1
  fi
  rm -rf "$_d"
}

_cdcp_restore_safe_mtime_demo_body() {
  _d="$1"
  printf 'CLEAN\n' >"$_d/file.rs"
  cp "$_d/file.rs" "$_d/file.rs.bak"
  # Age the backup the way `cp file file.bak` before a perturb-and-compile
  # ages it in the house pattern. 2001 is older than any compile this
  # process can have produced.
  touch -t 200109090146.40 "$_d/file.rs.bak"
  _bak_m="$(_cdcp_mtime_ns "$_d/file.rs.bak")"

  # Perturb, as a meta-test would, then restore THE UNSAFE WAY.
  printf 'DIRTY\n' >"$_d/file.rs"
  mv "$_d/file.rs.bak" "$_d/file.rs"
  [ "$(cat "$_d/file.rs")" = "CLEAN" ] \
    || { echo "restore_safe: naive mv lost content (wrong bug)" >&2; return 1; }
  _naive_m="$(_cdcp_mtime_ns "$_d/file.rs")"
  if [ "$_naive_m" -ne "$_bak_m" ]; then
    echo "restore_safe: FAIL: naive mv did not reproduce the trap (mtime $_naive_m != backup $_bak_m)" >&2
    return 1
  fi
  echo "restore_safe: known-bad RED — mv inherited backup mtime $_bak_m (cargo would skip)"

  # Same bytes, helper path. Recreate an aged backup of CLEAN, perturb, restore.
  printf 'CLEAN\n' >"$_d/file.rs.bak"
  touch -t 200109090146.40 "$_d/file.rs.bak"
  printf 'DIRTY\n' >"$_d/file.rs"
  cdcp_restore_safe "$_d/file.rs" "$_d/file.rs.bak" || return 1
  [ "$(cat "$_d/file.rs")" = "CLEAN" ] \
    || { echo "restore_safe: helper lost content" >&2; return 1; }
  _help_m="$(_cdcp_mtime_ns "$_d/file.rs")"
  if [ "$_help_m" -eq "$_bak_m" ]; then
    echo "restore_safe: FAIL: helper left the backup mtime (stale)" >&2
    return 1
  fi
  if [ "$_help_m" -le "$_bak_m" ]; then
    echo "restore_safe: FAIL: helper mtime $_help_m is not newer than backup $_bak_m" >&2
    return 1
  fi
  echo "restore_safe: helper GREEN — dest mtime $_help_m > backup $_bak_m (cargo cannot skip)"
}

# Isolated cargo: never the workspace target, never inherited wrappers.
# A forced `cargo build` AFTER a rename-restore is itself vacuous (it
# exits 0 and rebuilds nothing). We therefore require the artifact mtime
# to move, which is the anti-vacuous clause aimed at the remedy.
_cdcp_isolated_cargo() {
  _manifest="$1"
  _target="$2"
  shift 2
  env -u CARGO_TARGET_DIR -u CARGO_BUILD_TARGET -u CARGO_BUILD_TARGET_DIR \
     -u CARGO_ENCODED_RUSTFLAGS -u RUSTFLAGS -u RUSTDOCFLAGS \
     -u RUSTC_WRAPPER -u RUSTC_WORKSPACE_WRAPPER -u CARGO_MAKEFLAGS \
     -u CARGO_MANIFEST_DIR -u CARGO_PKG_NAME -u LLVM_PROFILE_FILE \
     CARGO_TARGET_DIR="$_target" \
     cargo "$@" --offline --manifest-path "$_manifest"
}

cdcp_restore_safe_cargo_demo() {
  if ! command -v cargo >/dev/null 2>&1; then
    echo "restore_safe: cargo not on PATH — cannot demonstrate skip-vs-rebuild" >&2
    return 2
  fi
  _d="$(mktemp -d "${TMPDIR:-/tmp}/cdcp_restore_safe_cargo.XXXXXX")"
  if ! _cdcp_restore_safe_cargo_demo_body "$_d"; then
    rm -rf "$_d"
    return 1
  fi
  rm -rf "$_d"
}

_cdcp_restore_safe_cargo_demo_body() {
  _d="$1"
  mkdir -p "$_d/src"
  printf '%s\n' \
    '[package]' \
    'name = "cdcp_mtime_trap"' \
    'version = "0.0.0"' \
    'edition = "2021"' \
    '' \
    '[workspace]' \
    >"$_d/Cargo.toml"
  printf 'pub const MARKER: &str = "CLEAN";\n' >"$_d/src/lib.rs"
  _manifest="$_d/Cargo.toml"
  _target="$_d/target"
  _src="$_d/src/lib.rs"
  _art="$_d/target/debug/libcdcp_mtime_trap.rlib"

  echo "restore_safe: cargo demo — initial CLEAN build (isolated target, --offline)"
  _cdcp_isolated_cargo "$_manifest" "$_target" build >/dev/null \
    || { echo "restore_safe: FAIL: initial cargo build" >&2; return 1; }
  [ -f "$_art" ] || { echo "restore_safe: FAIL: missing $_art" >&2; return 1; }

  cp "$_src" "$_d/lib.rs.bak"
  touch -t 200109090146.40 "$_d/lib.rs.bak"

  printf 'pub const MARKER: &str = "DIRTY";\n' >"$_src"
  echo "restore_safe: cargo demo — perturbed DIRTY build"
  _cdcp_isolated_cargo "$_manifest" "$_target" build >/dev/null \
    || { echo "restore_safe: FAIL: perturbed cargo build" >&2; return 1; }
  _dirty_m="$(_cdcp_mtime_ns "$_art")"

  # Naive restore: mv the aged CLEAN backup over the source.
  mv "$_d/lib.rs.bak" "$_src"
  [ "$(cat "$_src")" = 'pub const MARKER: &str = "CLEAN";' ] \
    || { echo "restore_safe: FAIL: naive restore lost CLEAN bytes" >&2; return 1; }
  echo "restore_safe: cargo demo — prove-rebuild after naive mv (must REFUSE)"
  if cdcp_prove_rebuild --artifact "$_art" -- _cdcp_isolated_cargo "$_manifest" "$_target" build; then
    echo "restore_safe: FAIL: prove-rebuild stayed GREEN on a stale tree (art mtime was $_dirty_m)" >&2
    return 1
  fi
  echo "restore_safe: known-bad RED — prove-rebuild refused the stale artifact (mtime still $_dirty_m)"

  # Recreate aged CLEAN backup, perturb again, helper-restore, prove rebuild.
  printf 'pub const MARKER: &str = "CLEAN";\n' >"$_d/lib.rs.bak"
  touch -t 200109090146.40 "$_d/lib.rs.bak"
  printf 'pub const MARKER: &str = "DIRTY";\n' >"$_src"
  echo "restore_safe: cargo demo — rebuild DIRTY so the helper has something to invalidate"
  _cdcp_isolated_cargo "$_manifest" "$_target" build >/dev/null \
    || { echo "restore_safe: FAIL: second perturbed cargo build" >&2; return 1; }

  cdcp_restore_safe "$_src" "$_d/lib.rs.bak" || return 1
  [ "$(cat "$_src")" = 'pub const MARKER: &str = "CLEAN";' ] \
    || { echo "restore_safe: FAIL: helper lost CLEAN bytes" >&2; return 1; }
  echo "restore_safe: cargo demo — prove-rebuild after helper (must REBUILD)"
  cdcp_prove_rebuild --artifact "$_art" -- _cdcp_isolated_cargo "$_manifest" "$_target" build \
    || { echo "restore_safe: FAIL: prove-rebuild did not observe a rebuild after helper restore" >&2; return 1; }
  echo "restore_safe: helper GREEN — prove-rebuild observed a rebuild"
}

# Empty/absent artifact is ERROR, never "nothing to check".
cdcp_prove_rebuild_absent_plant() {
  _d="$(mktemp -d "${TMPDIR:-/tmp}/cdcp_prove_rebuild_absent.XXXXXX")"
  _rc=0
  cdcp_prove_rebuild --artifact "$_d/no-such-artifact" -- true || _rc=$?
  rm -rf "$_d"
  if [ "$_rc" -eq 0 ]; then
    echo "prove-rebuild: FAIL: absent artifact was GREEN" >&2
    return 1
  fi
  echo "prove-rebuild: known-bad RED — absent artifact is ERROR (rc=$_rc)"
}

_cdcp_replace_once() {
  _cdcp_cli snap-rewrite replace-once --file "$1" --from "$2" --to "$3"
}

# CHARTER pair for prove-rebuild, on a COPY of this helper. The live file
# is never mutated (791t sources it concurrently).
#   (1) weaken the mtime check → suite (stale must be RED) goes non-zero
#   (2) mutation still in place, delete the assertion → suite returns to zero
# Restore the copy through cdcp_restore_safe.
cdcp_prove_rebuild_charter_pair() {
  _live="${1:-}/scripts/restore_safe.inc.sh"
  if [ ! -f "$_live" ]; then
    echo "restore_safe: CHARTER pair needs $_live" >&2
    return 2
  fi
  _bin="$(_cdcp_resolve)" \
    || { echo "restore_safe: CHARTER pair needs cdcp (cargo build -p cdcp_cli --locked; no python3 fallback)" >&2; return 2; }
  CDCP="$_bin"
  export CDCP
  _d="$(mktemp -d "${TMPDIR:-/tmp}/cdcp_prove_rebuild_pair.XXXXXX")"
  if ! _cdcp_prove_rebuild_charter_pair_body "$_live" "$_d"; then
    rm -rf "$_d"
    return 1
  fi
  rm -rf "$_d"
}

_cdcp_prove_rebuild_charter_pair_body() {
  _live="$1"
  _d="$2"
  cp "$_live" "$_d/helper.sh" || return 1
  cp "$_live" "$_d/helper.sh.bak" || return 1

  mkdir -p "$_d/src"
  printf '%s\n' \
    '[package]' \
    'name = "cdcp_mtime_pair"' \
    'version = "0.0.0"' \
    'edition = "2021"' \
    '' \
    '[workspace]' \
    >"$_d/Cargo.toml"
  printf 'pub const MARKER: &str = "CLEAN";\n' >"$_d/src/lib.rs"
  _manifest="$_d/Cargo.toml"
  _target="$_d/target"
  _src="$_d/src/lib.rs"
  _art="$_d/target/debug/libcdcp_mtime_pair.rlib"

  _cdcp_isolated_cargo "$_manifest" "$_target" build >/dev/null \
    || { echo "restore_safe: CHARTER pair: initial cargo build" >&2; return 1; }
  cp "$_src" "$_d/lib.rs.bak"
  touch -t 200109090146.40 "$_d/lib.rs.bak"
  printf 'pub const MARKER: &str = "DIRTY";\n' >"$_src"
  _cdcp_isolated_cargo "$_manifest" "$_target" build >/dev/null \
    || { echo "restore_safe: CHARTER pair: dirty cargo build" >&2; return 1; }
  mv "$_d/lib.rs.bak" "$_src"
  [ -f "$_art" ] || { echo "restore_safe: CHARTER pair: missing $_art" >&2; return 1; }

  # Suite: stale tree must make prove-rebuild RED. Sourced helper is the COPY.
  printf '%s\n' \
    '#!/bin/sh' \
    'set -eu' \
    ". \"$_d/helper.sh\"" \
    '_s_rc=0' \
    "cdcp_prove_rebuild --artifact \"$_art\" -- _cdcp_isolated_cargo \"$_manifest\" \"$_target\" build || _s_rc=\$?" \
    '# CHARTER-NEEDLE-ASSERT' \
    '[ "$_s_rc" -ne 0 ] || exit 1' \
    'exit 0' \
    >"$_d/suite.sh"
  cp "$_d/suite.sh" "$_d/suite.sh.bak"

  _pair=0
  # Weaken via the unique marker on the comparison. The kind lives in
  # cdcp so this comment cannot match CHARTER-NEEDLE + -CHECK, and the
  # if-string in this file cannot become a second replace-once hit.
  _cdcp_cli snap-rewrite charter --file "$_d/helper.sh" --kind weaken-if \
    || { echo "restore_safe: CHARTER pair: mutate needle missing" >&2; return 1; }
  _rc=0
  sh "$_d/suite.sh" || _rc=$?
  if [ "$_rc" -eq 0 ]; then
    echo "restore_safe: CHARTER pair leg 1 stayed GREEN after weakening prove-rebuild" >&2
    return 1
  fi
  _pair=$((_pair + 1))
  echo "restore_safe: CHARTER pair leg 1 RED (mutated prove-rebuild, rc=$_rc)"

  _cdcp_replace_once "$_d/suite.sh" \
    '[ "$_s_rc" -ne 0 ] || exit 1' \
    ': # assertion deleted (CHARTER pair leg 2)' \
    || { echo "restore_safe: CHARTER pair: assert needle missing" >&2; return 1; }
  _rc=0
  sh "$_d/suite.sh" || _rc=$?
  if [ "$_rc" -ne 0 ]; then
    echo "restore_safe: CHARTER pair leg 2 stayed RED after deleting the assertion (rc=$_rc)" >&2
    return 1
  fi
  _pair=$((_pair + 1))
  echo "restore_safe: CHARTER pair leg 2 GREEN (assertion deleted, rc=$_rc)"

  [ "$_pair" -eq 2 ] \
    || { echo "restore_safe: ANTI-VACUOUS: CHARTER pair ran $_pair legs, want 2" >&2; return 1; }

  cdcp_restore_safe "$_d/helper.sh" "$_d/helper.sh.bak" || return 1
  cdcp_restore_safe "$_d/suite.sh" "$_d/suite.sh.bak" || return 1
  _rc=0
  sh "$_d/suite.sh" || _rc=$?
  if [ "$_rc" -ne 0 ]; then
    echo "restore_safe: CHARTER pair: restored helper+suite failed (rc=$_rc)" >&2
    return 1
  fi
  echo "restore_safe: CHARTER pair 2/2 (mutate RED · delete-assert GREEN · restore_safe)"
}

cdcp_restore_safe_selftest() {
  _root="${1:-}"
  if [ -z "$_root" ]; then
    echo "restore_safe: selftest needs the engine root" >&2
    return 2
  fi
  cdcp_restore_safe_mtime_demo || return 1
  cdcp_restore_safe_scan "$_root" || return 1
  # Cargo demo is the skip-vs-rebuild proof. Isolated throwaway crate;
  # does not touch the workspace target. Required: a helper that cannot
  # be shown to invalidate a stale artifact has not been verified.
  cdcp_restore_safe_cargo_demo || return 1
  cdcp_prove_rebuild_absent_plant || return 1
  cdcp_prove_rebuild_charter_pair "$_root" || return 1
  echo "restore_safe: SELFTEST PASSED (naive-mv RED · prove-rebuild refuses stale · helper rebuilds · absent artifact ERROR · CHARTER pair · scan non-vacuous)"
}

# Executed, not sourced: run the full selftest, or the prove-rebuild
# subcommand. When sourced, $0 is the caller and we only define functions.
case "${0##*/}" in
  restore_safe.inc.sh)
    set -eu
    _here="$(CDPATH= cd -- "$(dirname "$0")" && pwd)"
    _engine="$(CDPATH= cd -- "$_here/.." && pwd)"
    _CDCP_ENGINE_ROOT="$_engine"
    if _resolved="$(_cdcp_resolve)"; then
      CDCP="$_resolved"
      export CDCP
    fi
    case "${1:-selftest}" in
      prove-rebuild)
        shift
        cdcp_prove_rebuild "$@"
        ;;
      selftest|"")
        cdcp_restore_safe_selftest "$_engine"
        ;;
      *)
        echo "restore_safe: usage: $0 [selftest|prove-rebuild --artifact PATH -- CMD...]" >&2
        exit 2
        ;;
    esac
    ;;
esac
