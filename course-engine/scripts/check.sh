#!/usr/bin/env sh
# check.sh — fail-closed gate for cdcp-course engine
# Waves incomplete: exit 2 with clear message until L3+ tools exist.
#
# L4 selftests: scripts/selftest_known_bad.sh injects known-bad fixtures,
# asserts RED, restores. Never leave goldens/bank dirty.
set -eu

ROOT="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() { echo "check.sh: FAIL: $*" >&2; exit 2; }
ok() { echo "check.sh: ok: $*"; }

echo "==> cdcp-course check (W0 knowledge scaffold)"

# Required constitution docs
for f in \
  docs/ORACLE-GAUNTLET.md \
  docs/STANDARDS-KB.md \
  docs/TESTING.md \
  docs/VISUAL.md \
  docs/OQ_REGISTER.md \
  docs/NEGATIVE_EVIDENCE.md \
  docs/research/STANDARDS-TENSIONS.md \
  README.md
do
  [ -f "$f" ] || fail "missing $f"
done
ok "constitution docs present"

# Required knowledge pack
for f in \
  knowledge/exam_form.toml \
  knowledge/sources.toml \
  knowledge/domains.toml \
  knowledge/topics.toml \
  knowledge/standards_families.toml \
  knowledge/standards_crosswalk.toml \
  knowledge/fact_policy.toml \
  knowledge/claims.toml
do
  [ -f "$f" ] || fail "missing $f"
done
ok "knowledge pack files present"

# L1 claims constitution (frankengraphdb-style registries + registry-check)
for f in \
  registries/claims.toml \
  registries/claims_lint.toml \
  registries/objectives.toml
do
  [ -f "$f" ] || fail "missing $f (L1 empty/deleted registry = ERROR)"
done
ok "L1 registry files present"

echo "==> cdcp_registry_check (L1 claims constitution)"
cargo run -q -p cdcp_registry_check || fail "registry-check"
ok "L1 registry-check"


# exam_form hard numbers (public CDCP form)
grep -q 'n_items = 40' knowledge/exam_form.toml || fail "exam_form n_items"
grep -q 'duration_sec = 3600' knowledge/exam_form.toml || fail "exam_form duration"
grep -q 'pass_correct = 27' knowledge/exam_form.toml || fail "exam_form pass_correct"
grep -q 'credential_claim = "forbidden"' knowledge/exam_form.toml || fail "credential must be forbidden"
ok "exam_form public CDCP format pins"

# Honesty: no "you are certified" in engine docs (allow "not certified").
# CRITICAL: ~/.ripgreprc may contain --type-not=video (and friends). Those
# types are not registered → rg exits 2. Piping that through another filter
# under set -eu (no pipefail) fail-OPENs the honesty gate. Always:
#   1) rg --no-config  (ignore broken global type filters)
#   2) treat rc>=2 as hard fail (never green on scanner error)
if ! command -v rg >/dev/null 2>&1; then
  fail "rg required for honesty scan"
fi
honesty_rc=0
honesty_hits="$(rg --no-config -n --glob '*.md' --glob '*.toml' \
  'you are (now )?CDCP certified|officially certified by EPI' \
  docs knowledge 2>&1)" || honesty_rc=$?
case "$honesty_rc" in
  0)
    filtered="$(printf '%s\n' "$honesty_hits" | rg --no-config -v 'not |never |FORBIDDEN|forbidden' || true)"
    if [ -n "$filtered" ]; then
      printf '%s\n' "$filtered" >&2
      fail "possible credential inflation string"
    fi
    ;;
  1) ;; # no matches — clean
  *)
    fail "honesty scan error (rg rc=$honesty_rc) — refusing fail-open: $honesty_hits"
    ;;
esac
ok "honesty string smoke"

# Crosswalk: all primary domains 01-14 appear
for d in 01-mission-critical 02-standards 03-site-building 04-floor-ceiling \
  05-lighting 06-power 07-emf 08-racks 09-cooling 10-water \
  11-network 12-fire 13-security 14-auxiliary
do
  grep -q "domain = \"$d\"" knowledge/standards_crosswalk.toml || fail "crosswalk missing $d"
done
ok "standards crosswalk covers domains 01-14"

# Topics non-empty
topic_count="$(grep -c '^\[\[topic\]\]' knowledge/topics.toml || true)"
[ "$topic_count" -ge 50 ] || fail "topics.toml too thin ($topic_count)"
ok "topics.toml count=$topic_count"

# Sources have fetch_date
grep -q 'fetch_date' knowledge/sources.toml || fail "sources need fetch_date"
ok "sources fetch_date present"

# L2 bank (when present)
if [ -f scripts/verify_bank.py ] && [ -d bank/items ]; then
  echo "==> verify_bank.py"
  python3 scripts/verify_bank.py || fail "bank verify"
  ok "bank pool"
fi

# Anti-hallucination heuristics + corpus overlap
if [ -f scripts/validate_grounding.py ] && [ -d bank/items ]; then
  echo "==> validate_grounding.py"
  python3 scripts/validate_grounding.py || fail "grounding"
  ok "grounding heuristics"
fi

# L3 GradeExact — cargo + goldens (BUILT must be WIRED here)
if [ ! -f Cargo.toml ]; then
  fail "Cargo.toml missing (L3 workspace required)"
fi

echo "==> cargo fmt/clippy/test"
cargo fmt --check || fail "cargo fmt"
cargo clippy --locked --workspace -- -D warnings || fail "clippy"
cargo test --locked --workspace || fail "cargo test"
ok "cargo fmt + clippy -D warnings + test"

for f in \
  goldens/fixtures/mock40_seed42.json \
  goldens/mock40_seed42_all_correct.sha256 \
  goldens/mock40_seed42_all_wrong.sha256 \
  goldens/bank_hash.txt \
  goldens/PROVENANCE.md \
  docs/CANONICAL.md
do
  [ -f "$f" ] || fail "missing L3 artifact $f"
done
ok "L3 golden artifacts present"

echo "==> cdcp goldens check"
cargo run -q -p cdcp_cli -- goldens check --bank bank/items --dir goldens \
  || fail "goldens check"
ok "GradeExact goldens"

# L4 — gates proven to trip (inject known-bad → assert RED → restore)
if [ -x scripts/selftest_known_bad.sh ] || [ -f scripts/selftest_known_bad.sh ]; then
  echo "==> selftest_known_bad.sh (L4)"
  sh scripts/selftest_known_bad.sh || fail "known-bad selftests"
  ok "known-bad selftests (gates trip, tree clean)"
else
  fail "missing scripts/selftest_known_bad.sh (L4 required)"
fi

# Wave status
echo "check.sh: WAVE STATUS: W0+L2+L3 GREEN; L4 selftests WIRED; L4 WASM dual-path + L5 UI still OPEN"
echo "check.sh: next: L4 Rust==WASM digest parity, then L5 browser mock path"

# Knowledge primary_notes path resolution (parent ../modules/)
if [ -f scripts/verify_knowledge_paths.py ]; then
  echo "==> verify_knowledge_paths.py"
  python3 scripts/verify_knowledge_paths.py || fail "knowledge primary_notes paths"
  ok "knowledge primary_notes paths"
fi

echo "==> check.sh PASSED (W0 + L2 bank + L3 GradeExact + L4 known-bad)"
exit 0
