#!/usr/bin/env sh
# build_web_wasm.sh — build cdcp_wasm for browser and install under web/assets/wasm/
#
# Usage (from repo root or any cwd):
#   ./scripts/build_web_wasm.sh
#   ./scripts/build_web_wasm.sh --debug   # faster compile, larger binary
#
# Prerequisites:
#   rustup target add wasm32-unknown-unknown
#
# Output:
#   web/assets/wasm/cdcp_wasm.wasm
#
# Headless grade without browser WASM (oracle only):
#   cdcp grade \
#     --fixture goldens/fixtures/mock40_seed42.json --mode all-correct
set -eu

ROOT="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

PROFILE="release"
CARGO_PROFILE_FLAG="--release"
OUT_SUBDIR="release"

for arg in "$@"; do
  case "$arg" in
    --debug|-d)
      PROFILE="debug"
      CARGO_PROFILE_FLAG=""
      OUT_SUBDIR="debug"
      ;;
    -h|--help)
      sed -n '2,20p' "$0"
      exit 0
      ;;
    *)
      echo "build_web_wasm.sh: unknown arg: $arg" >&2
      exit 2
      ;;
  esac
done

if ! command -v rustup >/dev/null 2>&1; then
  echo "build_web_wasm.sh: rustup required" >&2
  exit 2
fi

if ! rustup target list --installed | grep -qx 'wasm32-unknown-unknown'; then
  echo "==> adding rustup target wasm32-unknown-unknown"
  rustup target add wasm32-unknown-unknown
fi

echo "==> cargo build -p cdcp_wasm --target wasm32-unknown-unknown ($PROFILE)"
# shellcheck disable=SC2086
cargo build -p cdcp_wasm --target wasm32-unknown-unknown $CARGO_PROFILE_FLAG

SRC="target/wasm32-unknown-unknown/${OUT_SUBDIR}/cdcp_wasm.wasm"
if [ ! -f "$SRC" ]; then
  echo "build_web_wasm.sh: missing artifact $SRC" >&2
  exit 2
fi

DEST_DIR="web/assets/wasm"
mkdir -p "$DEST_DIR"
cp "$SRC" "$DEST_DIR/cdcp_wasm.wasm"

BYTES="$(wc -c < "$DEST_DIR/cdcp_wasm.wasm" | tr -d ' ')"
echo "build_web_wasm.sh: ok"
echo "  profile=$PROFILE"
echo "  src=$SRC"
echo "  dest=$DEST_DIR/cdcp_wasm.wasm"
echo "  bytes=$BYTES"
