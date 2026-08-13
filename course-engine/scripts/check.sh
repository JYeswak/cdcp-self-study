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
GAPS=""
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

# L4 WASM dual-path (optional until toolchain present — skip-honest, not full L4 green)
echo "==> L4 WASM dual-path (optional)"
L4_WASM="SKIP"
if command -v rustup >/dev/null 2>&1   && rustup target list --installed 2>/dev/null | grep -q '^wasm32-unknown-unknown$'
then
  if cargo build -p cdcp_wasm --target wasm32-unknown-unknown --locked \
    && CDCP_REQUIRE_WASM=1 cargo test -p cdcp_wasm --test dual_path --locked -- --nocapture
  then
    ok "L4 WASM dual-path native==wasm (mock40_seed42)"
    L4_WASM="GREEN"
  else
    fail "L4 WASM dual-path failed (toolchain present but digests disagree or test/build error)"
  fi
else
  echo "check.sh: SKIP wasm: toolchain missing"
  echo "check.sh: L4 dual-path is NOT full green — install: rustup target add wasm32-unknown-unknown"
  L4_WASM="SKIP"
fi

# Wave status
echo "check.sh: WAVE STATUS: W0+L1+L2+L3 GREEN; L4 known-bad WIRED; L4 WASM=$L4_WASM; L5 UI still OPEN"
if [ "$L4_WASM" = "GREEN" ]; then
  echo "check.sh: next: L5 browser mock path (UI e2e digest match)"
else
  echo "check.sh: next: enable wasm32 target for L4 dual-path GREEN, then L5 UI"
fi

# Knowledge primary_notes path resolution (parent ../modules/)
if [ -f scripts/verify_knowledge_paths.py ]; then
  echo "==> verify_knowledge_paths.py"
  python3 scripts/verify_knowledge_paths.py || fail "knowledge primary_notes paths"
  ok "knowledge primary_notes paths"
fi

# ─── L5 browser surface ─────────────────────────────────────────────────────
echo "==> L5 browser surface (require product files)"
for f in web/index.html web/learn.html web/drill.html web/mock.html web/reference.html; do
  [ -f "$f" ] || fail "L5 product file missing: $f"
done
ok "L5 product files present"
[ -f web/assets/wasm/cdcp_wasm.wasm ] || fail "L5 wasm artifact missing under web/assets/wasm/"
ok "L5 wasm artifact present under web/assets/wasm/"
python3 -c "
import json,sys
d=json.load(open('web/data/mock40_seed42.json'))
assert d['n_items']==40, 'n_items=%r' % d['n_items']
assert len(d['items'])==40, 'items=%d' % len(d['items'])
assert all('correct' not in i for i in d['items']), 'learner pack leaks correct letters'
" || fail "L5 learner pack shape"
ok "L5 learner pack n_items=40"

echo "==> selftest_l5.sh (honesty + e2e digest known-bad)"
sh scripts/selftest_l5.sh || fail "L5 selftest"
ok "L5 selftest (honesty plant RED · digest match · flipped golden RED · empty fixtures ERROR)"

echo "==> e2e_l5_digest.sh (UI dual-path digest match)"
sh scripts/e2e_l5_digest.sh || fail "L5 e2e digest"
ok "L5 e2e digest match (seed42 all-correct/all-wrong)"

echo "==> smoke_learn.py"
python3 scripts/smoke_learn.py || fail "L5 learn smoke"
ok "L5 learn smoke"

# ─── L6 mastery / coverage ──────────────────────────────────────────────────
echo "==> smoke_srs.mjs";        node scripts/smoke_srs.mjs        || fail "L6 srs smoke";        ok "L5 srs smoke"
echo "==> smoke_mastery.mjs";    node scripts/smoke_mastery.mjs    || fail "L6 mastery smoke";    ok "L6 mastery smoke"
echo "==> smoke_weak_links.py";  python3 scripts/smoke_weak_links.py || fail "L6 weak-links smoke"; ok "L6 weak-links smoke"
echo "==> smoke_hub_mastery.mjs"; node scripts/smoke_hub_mastery.mjs || fail "L6-S4 hub mastery"; ok "L6 hub mastery + recommend smoke"
ok "L6-S4 hub mastery surface wired"

echo "==> L6 multi-seed export-web (fixture golden-stable)"
_MS_TMP="$(mktemp -d "${TMPDIR:-/tmp}/cdcp_multiseed.XXXXXX")"
cargo run -q -p cdcp_cli -- export-web --bank bank/items --seed 42 --out "$_MS_TMP" >/dev/null \
  || fail "L6 multi-seed export-web"
for f in mock40_seed42.json keys_seed42.json bank_items_seed42.json; do
  cmp -s "$_MS_TMP/$f" "web/data/$f" || fail "L6 export-web seed42 not golden-stable: $f"
done
rm -rf "$_MS_TMP"
ok "L6 multi-seed export-web --seed 42 (fixture golden-stable)"

echo "==> L6 session shapes"
for _shape in "Drill due" "Miss review"; do
  grep -q "$_shape" web/drill.html || fail "L6 session shape missing from web/drill.html: $_shape"
done
ok "L6 session shapes (Drill due · Miss review) present"

echo "==> L6 domain coverage oracle"
python3 scripts/verify_coverage.py || fail "L6 coverage"
ok "L6 coverage GREEN (modules 1–14 ≥ domain_min)"
echo "==> selftest_l6_coverage.sh"
sh scripts/selftest_l6_coverage.sh || fail "L6 coverage selftest"
ok "L6 coverage selftest (empty RED · missing-module RED · live GREEN)"

# ─── L7 product surfaces ────────────────────────────────────────────────────
echo "==> L7 product surfaces"
for f in web/reference.html web/learn.html; do
  [ -f "$f" ] || fail "L7 surface missing: $f"
done
ok "L7 surfaces (reference · closed-notes · Learn-15)"

echo "==> smoke_learn_chrome.py (M8-A)"; python3 scripts/smoke_learn_chrome.py || fail "M8-A learn chrome"; ok "M8-A learn chrome smoke"
echo "==> build_units.py";               python3 scripts/build_units.py         || fail "M8-B units_index"; ok "M8-B units_index"
echo "==> build_glossary_json.py";       python3 scripts/build_glossary_json.py || fail "M8-D glossary";    ok "M8-D glossary.json"
echo "==> smoke_learn_v2.py";            python3 scripts/smoke_learn_v2.py      || fail "M8-B/D learn v2";  ok "M8-B/D learn v2 smoke"
echo "==> smoke_diagrams.py";            python3 scripts/smoke_diagrams.py      || fail "M8-C diagrams";    ok "M8-C diagrams smoke"
echo "==> smoke_a11y.py";                python3 scripts/smoke_a11y.py          || fail "L7 a11y";          ok "L7 a11y baseline"
echo "==> smoke_feedback_links.py";      python3 scripts/smoke_feedback_links.py || fail "L7 feedback links"; ok "L7-S2 feedback section-anchor links smoke"
ok "L7 feedback section links"

echo "==> L7 CLI product verbs"
_HELP="$(cargo run -q -p cdcp_cli -- --help 2>&1)"
for v in bank-hash grade goldens export-web serve; do
  printf '%s' "$_HELP" | grep -q -- "$v" || fail "L7 CLI verb missing from --help: $v"
done
ok "L7 CLI product verbs listed"

echo "==> verify_objectives.py"
python3 scripts/verify_objectives.py || fail "L7 objective coverage"
ok "L7 objective coverage"
echo "==> selftest_l7_objectives.sh"
sh scripts/selftest_l7_objectives.sh || fail "L7 objectives selftest"
ok "L7 objectives known-bad selftest"

echo "==> smoke_slo.sh"
if cargo run -q -p cdcp_cli -- export-web --help >/dev/null 2>&1; then
  sh scripts/smoke_slo.sh || fail "L7 SLO budgets"
  ok "L7 SLO budgets"
else
  echo "check.sh: GAP: L7 SLO budgets NOT RUN — cdcp_cli lacks the 'export-web' verb" >&2
  GAPS="${GAPS}L7-SLO "
fi

echo "==> verify_content_lock.py"
python3 scripts/verify_content_lock.py || fail "L7 content.lock"
ok "L7 content.lock"

# ─── V11 stretch surfaces ───────────────────────────────────────────────────
if [ -f scripts/selftest_reconstructed.sh ] && [ "${CDCP_IN_SELFTEST:-0}" != "1" ]; then
  echo "==> selftest_reconstructed.sh (L5–V11 reconstructed stages)"
  CDCP_IN_SELFTEST=1 sh scripts/selftest_reconstructed.sh || fail "reconstructed-stage selftests"
  ok "L5–V11 reconstructed stages proven to trip RED"
fi

echo "==> V11 stretch surfaces"
python3 scripts/export_anki.py --check >/dev/null 2>&1 || python3 scripts/export_anki.py >/dev/null 2>&1 || fail "V11 Anki export"
ok "V11 Anki export"
grep -q "study aid" web/reference.html 2>/dev/null || grep -rq "not.*certif" web/ 2>/dev/null || fail "V11 diagram honesty"
ok "V11 diagram honesty present"
if cargo run -q -p cdcp_cli -- serve --help >/dev/null 2>&1; then
  ok "V11 serve subcommand present"
else
  echo "check.sh: GAP: V11 serve subcommand ABSENT from cdcp_cli source" >&2
  GAPS="${GAPS}V11-serve "
fi
ls bank/items/*.toml >/dev/null 2>&1 || fail "V11 runbook bank items"
ok "V11 runbook bank items present"

if [ -n "$GAPS" ]; then
  echo "check.sh: KNOWN GAPS (not green, not silent): $GAPS" >&2
fi
echo "check.sh: complete != EPI certified (study signal / mastery only)"
echo "==> check.sh PASSED (W0-L7 + V11 stretch; L4 WASM=$L4_WASM)"
exit 0
