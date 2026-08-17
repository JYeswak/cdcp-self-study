#!/usr/bin/env bash
# selftest_install.sh — L4 plants for install.sh [bd-installability-sm4g.7 / .18]
#
#   1) tampered tarball (sha256 mismatch) → RED, installs nothing
#   2) release JSON with zero matching assets → ERROR, not success
#   3) neither sha256sum nor shasum on PATH → ERROR, never skip verify
#   4) documented curl|bash form (stdin is the script) does not install a
#      toolchain without consent (D1)
#   5) foreign occupier on 8766 is not --verify proof: installed doctor/test/demo
#      still run against the prefix; demo must not print occupier 8766 as ours.
#      Never kills a pre-existing occupier (including live pid 63897).
#
# Does not need a live GitHub release. Emits INJECTIONS=5 SUITE=installer.
# Unregistered: do not add to REGISTERED_SUITES.
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
OCC_PID=
OCC_OWNED=0
cleanup() {
  # Kill only an occupier this suite started. A pre-existing listener on
  # 8766 (source-checkout serve, pid 63897, anything else) is left alone.
  if [ "${OCC_OWNED:-0}" = 1 ] && [ -n "${OCC_PID:-}" ]; then
    kill "$OCC_PID" 2>/dev/null || true
    wait "$OCC_PID" 2>/dev/null || true
  fi
  [ -n "${TMP:-}" ] && rm -rf "$TMP"
  return 0
}
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

# ── (5) foreign occupier on 8766 is not --verify proof ────────────────────
echo "==> (5) foreign occupier on 8766 is not --verify proof"
P5=$TMP/p5
mkdir -p "$P5/prefix/bin" \
  "$P5/prefix/share/cdcp/web/data" \
  "$P5/prefix/share/cdcp/web/assets/wasm"

CDCP_SRC=
if [ -n "${CARGO_TARGET_DIR:-}" ] && [ -x "$CARGO_TARGET_DIR/debug/cdcp" ]; then
  CDCP_SRC=$CARGO_TARGET_DIR/debug/cdcp
elif [ -x "$ROOT/target/debug/cdcp" ]; then
  CDCP_SRC=$ROOT/target/debug/cdcp
else
  (CDPATH= cd -- "$ROOT" && cargo build -p cdcp_cli --locked) || \
    fail "cargo build -p cdcp_cli --locked failed (needed for --verify plant)"
  if [ -n "${CARGO_TARGET_DIR:-}" ] && [ -x "$CARGO_TARGET_DIR/debug/cdcp" ]; then
    CDCP_SRC=$CARGO_TARGET_DIR/debug/cdcp
  else
    CDCP_SRC=$ROOT/target/debug/cdcp
  fi
fi
[ -x "$CDCP_SRC" ] || fail "cdcp binary missing at $CDCP_SRC"
cp "$CDCP_SRC" "$P5/prefix/bin/cdcp"
chmod 0755 "$P5/prefix/bin/cdcp"
# Dummy installed tree: real seed-42 pack (test/demo grade) + stub wasm/index.
printf '<!doctype html><title>cdcp</title>\n' >"$P5/prefix/share/cdcp/web/index.html"
printf '\0asm\n' >"$P5/prefix/share/cdcp/web/assets/wasm/cdcp_wasm.wasm"
[ -f "$ROOT/web/data/mock40_seed42.json" ] || fail "missing seed-42 pack"
[ -f "$ROOT/web/data/bank_items_seed42.json" ] || fail "missing seed-42 bank"
[ -f "$ROOT/web/data/keys_seed42.json" ] || fail "missing seed-42 keys"
cp "$ROOT/web/data/mock40_seed42.json" "$P5/prefix/share/cdcp/web/data/mock40_seed42.json"
cp "$ROOT/web/data/bank_items_seed42.json" "$P5/prefix/share/cdcp/web/data/bank_items_seed42.json"
cp "$ROOT/web/data/keys_seed42.json" "$P5/prefix/share/cdcp/web/data/keys_seed42.json"

lsof_bin=
if command -v lsof >/dev/null 2>&1; then lsof_bin=$(command -v lsof)
elif [ -x /usr/sbin/lsof ]; then lsof_bin=/usr/sbin/lsof
elif [ -x /usr/bin/lsof ]; then lsof_bin=/usr/bin/lsof
fi
occ_pid() {
  if [ -n "$lsof_bin" ]; then
    "$lsof_bin" -nP -iTCP:8766 -sTCP:LISTEN -t 2>/dev/null | awk 'NR==1{print; exit}'
  fi
}
OCC_PID=$(occ_pid || true)
if [ -n "$OCC_PID" ]; then
  OCC_OWNED=0
  ok "using pre-existing 8766 occupier pid=$OCC_PID (will not kill)"
else
  # Exact plant: a foreign `cdcp serve` on the source checkout, not the prefix.
  "$CDCP_SRC" serve --bind 127.0.0.1:8766 --root "$ROOT/web" --no-open \
    >"$P5/foreign-serve.log" 2>&1 &
  OCC_PID=$!
  OCC_OWNED=1
  n=0
  while [ "$n" -lt 20 ]; do
    got=$(occ_pid || true)
    if [ -n "$got" ]; then
      OCC_PID=$got
      break
    fi
    if ! kill -0 "$OCC_PID" 2>/dev/null; then
      printf '%s\n' "$(cat "$P5/foreign-serve.log" 2>/dev/null || true)" >&2
      fail "foreign occupier exited before listen"
    fi
    sleep 1
    n=$((n+1))
  done
  [ -n "$(occ_pid || true)" ] || fail "foreign occupier did not bind 8766"
  ok "started foreign cdcp serve on 8766 pid=$OCC_PID"
fi

resolved_bin=$(CDPATH= cd -- "$P5/prefix/bin" && pwd -P)/cdcp
resolved_root=$(CDPATH= cd -- "$P5/prefix/share/cdcp" && pwd -P)
rc=0
out=$(bash "$INSTALL" --verify --prefix "$P5/prefix" --no-modify-path 2>&1) || rc=$?

# Occupier must still be alive — --verify does not "fix" a busy 8766.
if ! kill -0 "$OCC_PID" 2>/dev/null; then
  printf '%s\n' "$out" >&2
  fail "verify killed occupier pid=$OCC_PID"
fi
after=$(occ_pid || true)
if [ -z "$after" ]; then
  printf '%s\n' "$out" >&2
  fail "verify left 8766 with no listener (occupier was pid=$OCC_PID)"
fi

if [ "$rc" -ne 0 ]; then
  printf '%s\n' "$out" >&2
  fail "--verify exited $rc against a valid installed prefix (occupier is not a fail)"
fi

case $out in
  *"verify: installed-bin=$resolved_bin"*) ;;
  *)
    printf '%s\n' "$out" >&2
    fail "verify did not record installed-bin=$resolved_bin"
    ;;
esac
case $out in
  *"verify: install-root=$resolved_root"*) ;;
  *)
    printf '%s\n' "$out" >&2
    fail "verify did not record install-root=$resolved_root"
    ;;
esac
case $out in
  *"not our proof"*) ;;
  *)
    printf '%s\n' "$out" >&2
    fail "verify did not refuse occupier 8766 as proof"
    ;;
esac
case $out in
  *'http://127.0.0.1:8766/'*|*'http://127.0.0.1:8766'*)
    printf '%s\n' "$out" >&2
    fail "verify/demo treated 8766 as its own proof"
    ;;
esac
case $out in
  *"doctor --root $resolved_root"*) ;;
  *)
    printf '%s\n' "$out" >&2
    fail "verify did not run doctor against the installed prefix"
    ;;
esac
case $out in
  *"test --root $resolved_root"*) ;;
  *)
    printf '%s\n' "$out" >&2
    fail "verify did not run test against the installed prefix"
    ;;
esac
case $out in
  *"demo --no-open --root $resolved_root"*|*"demo --root $resolved_root"*) ;;
  *)
    printf '%s\n' "$out" >&2
    fail "verify did not run demo against the installed prefix"
    ;;
esac
case $out in
  *"curl"*8766*)
    printf '%s\n' "$out" >&2
    fail "verify curled occupier 8766"
    ;;
esac

inject_counted
ok "foreign occupier on 8766 is not verify proof (pid=$OCC_PID still alive, prefix verbs ran)"

[ "$INJ" -eq 5 ] || fail "expected 5 injections, counted $INJ"
echo "INJECTIONS=$INJ SUITE=$SUITE_NAME"
echo "selftest_install: PASSED (tampered · empty-assets · no-checksum · D1 · occupier-8766)"
exit 0
