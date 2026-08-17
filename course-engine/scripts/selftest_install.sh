#!/usr/bin/env bash
# selftest_install.sh — L4 plants for install.sh [bd-installability-sm4g.7]
#
#   1) tampered tarball (sha256 mismatch) → RED, installs nothing
#   2) release JSON with zero matching assets → ERROR, not success
#   3) neither sha256sum nor shasum on PATH → ERROR, never skip verify
#   4) documented curl|bash form (stdin is the script) does not install a
#      toolchain without consent (D1)
#
# Does not need a live GitHub release. Emits INJECTIONS=4 SUITE=installer.
set -eu

ROOT="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
INSTALL="$ROOT/install.sh"
cd "$ROOT"

fail() { echo "selftest_install: FAIL: $*" >&2; exit 2; }
ok() { echo "selftest_install: ok: $*"; }

INJ=0
SUITE_NAME=installer
inject_counted() { INJ=$((INJ + 1)); }

TMP=
cleanup() { [ -n "${TMP:-}" ] && rm -rf "$TMP"; return 0; }
trap cleanup EXIT INT TERM HUP

[ -f "$INSTALL" ] || fail "missing $INSTALL"
TMP=$(mktemp -d "${TMPDIR:-/tmp}/cdcp-selftest-install.XXXXXX")

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
  case $out in
    *"$needle"*) ;;
    *)
      printf '%s\n' "$out" >&2
      fail "expected RED for $label to name '$needle'"
      ;;
  esac
  inject_counted
  ok "$label trips RED (rc=$rc)"
}

# Tools the installer needs, minus a denylist. Selftest uses command -v;
# the installer under test uses type -P.
stage_path() {
  dest=$1
  shift
  mkdir -p "$dest"
  for t in bash sh uname mkdir cat rm mv cp chmod mktemp date awk tr sed \
           grep find sort tar gzip gunzip git ln sleep kill head wc \
           dirname basename touch tee env true false printf; do
    skip=0
    for d in "$@"; do
      [ "$t" = "$d" ] && skip=1
    done
    [ "$skip" -eq 1 ] && continue
    p=$(command -v "$t" 2>/dev/null) || continue
    [ -e "$dest/$t" ] || ln -s "$p" "$dest/$t"
  done
  # checksum tools unless denied
  deny_sum=0
  for d in "$@"; do
    [ "$d" = sha256sum ] && deny_sum=1
    [ "$d" = shasum ] && deny_sum=1
  done
  if [ "$deny_sum" -eq 0 ]; then
    if command -v sha256sum >/dev/null 2>&1; then ln -s "$(command -v sha256sum)" "$dest/sha256sum"
    elif command -v shasum >/dev/null 2>&1; then ln -s "$(command -v shasum)" "$dest/shasum"
    else fail "selftest host has neither sha256sum nor shasum — cannot plant"; fi
  fi
}

host_sha256() {
  if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | awk 'NR==1{print $1; exit}'
  elif command -v shasum >/dev/null 2>&1; then shasum -a 256 "$1" | awk 'NR==1{print $1; exit}'
  else fail "host has no checksum tool"; fi
}

# ── (1) tampered tarball ──────────────────────────────────────────────────
echo "==> (1) tampered tarball → RED, installs nothing"
P1=$TMP/p1
mkdir -p "$P1/stage" "$P1/prefix"
printf '#!/bin/sh\necho dummy-cdcp\n' >"$P1/stage/cdcp"
chmod 0755 "$P1/stage/cdcp"
tar -C "$P1/stage" -czf "$P1/good.tar.gz" cdcp
GOOD_SHA=$(host_sha256 "$P1/good.tar.gz")
cp "$P1/good.tar.gz" "$P1/bad.tar.gz"
echo tampered-byte >>"$P1/bad.tar.gz"
assert_red "tampered-tarball" "sha256 mismatch" \
  bash "$INSTALL" --tarball "$P1/bad.tar.gz" --sha256 "$GOOD_SHA" \
    --prefix "$P1/prefix" --no-modify-path
if [ -e "$P1/prefix/bin/cdcp" ] || [ -e "$P1/prefix/share/cdcp/install-receipt.json" ]; then
  fail "tampered tarball installed something (bin or receipt present)"
fi
ok "tampered tarball installed nothing"

# ── (2) release with zero matching assets ─────────────────────────────────
echo "==> (2) empty-asset release → ERROR, not success"
P2=$TMP/p2
mkdir -p "$P2/prefix"
printf '%s\n' '{"tag_name":"v0.1.0","assets":[]}' >"$P2/empty.json"
assert_red "empty-assets" "no .tar.gz asset matching triple" \
  bash "$INSTALL" --release-json "$P2/empty.json" --prefix "$P2/prefix" --no-modify-path
if [ -e "$P2/prefix/bin/cdcp" ]; then
  fail "empty-asset release fell through to an install"
fi
ok "empty-asset release did not fall through to success"

# ── (3) neither checksum tool ─────────────────────────────────────────────
echo "==> (3) no sha256sum/shasum → ERROR"
P3=$TMP/p3
mkdir -p "$P3/bin" "$P3/prefix"
stage_path "$P3/bin" sha256sum shasum
# A matching tarball so we would otherwise proceed.
cp "$P1/good.tar.gz" "$P3/good.tar.gz"
assert_red "missing-checksum-tool" "neither sha256sum nor shasum" \
  env PATH="$P3/bin" bash "$INSTALL" --tarball "$P3/good.tar.gz" --sha256 "$GOOD_SHA" \
    --prefix "$P3/prefix" --no-modify-path
ok "missing checksum tool refused to skip verification"

# ── (4) curl|bash does not install a toolchain (D1) ───────────────────────
echo "==> (4) piped install.sh (stdin is the script) does not rustup-init"
P4=$TMP/p4
mkdir -p "$P4/bin" "$P4/home" "$P4/prefix"
stage_path "$P4/bin" cargo rustc rustup curl sha256sum shasum
# Keep a checksum tool so we reach need_cargo rather than dying earlier.
if command -v sha256sum >/dev/null 2>&1; then ln -sf "$(command -v sha256sum)" "$P4/bin/sha256sum"
elif command -v shasum >/dev/null 2>&1; then ln -sf "$(command -v shasum)" "$P4/bin/shasum"
fi
# Documented form: curl -fsSL ... | bash  (stdin is the script, no TTY consent).
rc=0
out=$(cat "$INSTALL" | env HOME="$P4/home" PATH="$P4/bin" bash -s -- \
  --from-source --prefix "$P4/prefix" --no-modify-path 2>&1) || rc=$?
[ "$rc" -ne 0 ] || { printf '%s\n' "$out" >&2; fail "piped --from-source exited 0 without cargo"; }
case $out in
  *"refusing to install a toolchain"*|*"cargo not on PATH"*) ;;
  *)
    printf '%s\n' "$out" >&2
    fail "piped install did not refuse a toolchain"
    ;;
esac
if [ -d "$P4/home/.rustup" ] || [ -d "$P4/home/.cargo" ] || [ -e "$P4/bin/rustup" ]; then
  fail "piped install created a rustup/cargo toolchain without consent"
fi
if [ -e "$P4/prefix/bin/cdcp" ]; then
  fail "piped install wrote a binary without cargo"
fi
inject_counted
ok "D1: curl|bash form installed no toolchain without consent (rc=$rc)"

[ "$INJ" -eq 4 ] || fail "expected 4 injections, counted $INJ"
echo "INJECTIONS=$INJ SUITE=$SUITE_NAME"
echo "selftest_install: PASSED (tampered · empty-assets · no-checksum · D1)"
exit 0
