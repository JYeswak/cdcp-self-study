#!/usr/bin/env sh
# Thin operator entry point; the implementation stays in the allowlisted
# scripts/check.sh so the substrate scan sees one CI driver, not a hidden
# second chain.  The standalone path is intentionally explicit and pinned.
set -eu
_local_ci_dir="$(CDPATH= cd -- "$(dirname "$0")" && pwd)"
exec sh "$_local_ci_dir/../../scripts/check.sh" --local-ci "$@"
