#!/usr/bin/env sh
# selftest_orphan.sh — the "orphan item" known-bad from ORACLE-GAUNTLET.md,
# proven to trip.
#
# Contract (mirrors scripts/selftest_l6_coverage.sh):
#   inject an in-tree known-bad specimen into TEMP, assert verify_orphans.py
#   goes RED with the expected signal, restore. The live bank/ and knowledge/
#   are NEVER mutated — the specimens are real TOML files this script writes,
#   not a patch applied to the working tree.
#
# Cases:
#   a) empty bank dir                  → ERROR (anti-vacuous)
#   b) empty topics registry           → ERROR (anti-vacuous)
#   c) item referencing unknown topic  → RED (orphan item, forward direction)
#   d) item with empty topic_ids       → RED (unanchored item)
#   e) topic referenced by zero items  → RED (orphan topic, reverse direction)
#   f) live tree                       → GREEN and un-dirtied
#
# A gate that cannot fail is not a gate: if any injection stays GREEN this
# script exits non-zero and check.sh fails.
set -eu

ROOT="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() { echo "selftest_orphan: FAIL: $*" >&2; exit 2; }
ok() { echo "selftest_orphan: ok: $*"; }

# -- L4 drift guard: self-reported RED-injection count ----------------------
# INJ counts the injections this run actually asserted RED (green controls are
# NOT counted). Emitted once, on the success path only, as a machine-readable
# line that scripts/verify_injection_count.py aggregates. A suite that stops
# emitting the line is an ERROR to that gate, never a silent zero.
INJ=0
SUITE_NAME="selftest_orphan"
inject_counted() { INJ=$((INJ + 1)); }

TMP_ROOT=""
restore_all() {
  if [ -n "${TMP_ROOT:-}" ] && [ -d "${TMP_ROOT}" ]; then
    rm -rf "${TMP_ROOT}" 2>/dev/null || true
  fi
}
trap restore_all EXIT INT TERM HUP

# assert_fails_with <label> <needle> cmd...
assert_fails_with() {
  label="$1"
  needle="$2"
  shift 2
  rc=0
  out="$("$@" 2>&1)" || rc=$?
  if [ "$rc" -eq 0 ]; then
    printf '%s\n' "$out" >&2
    fail "expected RED for $label but command exited 0"
  fi
  case "$out" in
    *"$needle"*)
      inject_counted
      ok "$label trips RED (rc=$rc, saw: $needle)"
      ;;
    *)
      printf '%s\n' "$out" >&2
      fail "$label exited $rc but missing expected signal '$needle'"
      ;;
  esac
}

echo "==> selftest_orphan (ORACLE-GAUNTLET known-bad: orphan item)"

[ -f scripts/verify_orphans.py ] || fail "missing scripts/verify_orphans.py"
[ -d bank/items ] || fail "missing bank/items"
[ -f knowledge/topics.toml ] || fail "missing knowledge/topics.toml"
command -v python3 >/dev/null 2>&1 || fail "python3 required"

TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/selftest_orphan.XXXXXX")"

# A known-good topic id, taken from the live registry (never hard-coded blind).
GOOD_TOPIC="$(python3 - <<'PY'
import re, pathlib
text = pathlib.Path("knowledge/topics.toml").read_text(encoding="utf-8")
ids = re.findall(r'(?m)^\s*id\s*=\s*"([^"]+)"', text)
print(ids[0] if ids else "")
PY
)"
[ -n "$GOOD_TOPIC" ] || fail "could not read a topic id from knowledge/topics.toml"
ok "anchor topic for specimens: $GOOD_TOPIC"

# ── (a) empty bank directory → anti-vacuous ERROR ───────────────────────────
echo "==> (a) empty bank → ERROR"
empty_bank="$TMP_ROOT/empty_bank"
mkdir -p "$empty_bank"
assert_fails_with "empty-bank" "empty bank" \
  python3 scripts/verify_orphans.py --bank "$empty_bank"

# ── (b) empty topics registry → anti-vacuous ERROR ──────────────────────────
echo "==> (b) empty topics registry → ERROR"
empty_topics="$TMP_ROOT/empty_topics.toml"
: >"$empty_topics"
assert_fails_with "empty-topics" "empty topic registry" \
  python3 scripts/verify_orphans.py --topics "$empty_topics"

# ── shared specimen bank: a faithful copy of the live bank ──────────────────
# Copying (rather than filtering) means the ONLY defect in each case below is
# the one we planted, so the needle proves the specific detector fired.
plant_bank="$TMP_ROOT/bank_items"
mkdir -p "$plant_bank"
cp bank/items/*.toml "$plant_bank/" || fail "could not copy live bank into TEMP"
n_copied="$(ls "$plant_bank"/*.toml 2>/dev/null | wc -l | tr -d ' ')"
[ "$n_copied" -gt 0 ] || fail "copied zero bank items into TEMP (vacuous specimen)"
ok "specimen bank copied ($n_copied files)"

# ── (c) orphan item: topic_ids points at a topic that does not exist ────────
echo "==> (c) item referencing unknown topic → RED"
cat >"$plant_bank/zz-selftest-orphan-ref.toml" <<EOF
id = "selftest-orphan-ref"
module = 1
stem = "selftest planted item — orphan reference specimen, not for exam use"
choices = ["A", "B", "C", "D"]
correct = "A"
explanation = "planted for the orphan-item selftest only"
topic_ids = ["zz-topic-that-does-not-exist"]
bloom = "remember"
source_class = "original"
quantity_evidence = "qualitative_only"
EOF
assert_fails_with "orphan-item-ref" "unknown topic_id" \
  python3 scripts/verify_orphans.py --bank "$plant_bank"
rm -f "$plant_bank/zz-selftest-orphan-ref.toml"

# ── (d) unanchored item: topic_ids present but empty ────────────────────────
echo "==> (d) item with empty topic_ids → RED"
cat >"$plant_bank/zz-selftest-unanchored.toml" <<EOF
id = "selftest-unanchored"
module = 1
stem = "selftest planted item — unanchored specimen, not for exam use"
choices = ["A", "B", "C", "D"]
correct = "A"
explanation = "planted for the orphan-item selftest only"
topic_ids = []
bloom = "remember"
source_class = "original"
quantity_evidence = "qualitative_only"
EOF
assert_fails_with "unanchored-item" "missing/empty topic_ids" \
  python3 scripts/verify_orphans.py --bank "$plant_bank"
rm -f "$plant_bank/zz-selftest-unanchored.toml"

# specimen bank must now be back to a clean copy → GREEN against live topics
rc=0
python3 scripts/verify_orphans.py --bank "$plant_bank" >/dev/null 2>&1 || rc=$?
[ "$rc" -eq 0 ] || fail "specimen bank not clean after removing planted items (rc=$rc)"
ok "specimen bank clean after (c)(d) removal"

# ── (e) orphan topic: declared in the registry, referenced by zero items ────
echo "==> (e) topic referenced by zero items → RED"
plant_topics="$TMP_ROOT/topics_plus_orphan.toml"
cp knowledge/topics.toml "$plant_topics"
cat >>"$plant_topics" <<'EOF'

[[topic]]
id = "zz-selftest-orphan-topic"
domain = "01-mission-critical"
label = "selftest planted orphan topic — assessed by zero bank items"
source = "src-epi-cdcp-page"
EOF
assert_fails_with "orphan-topic" "orphan topic 'zz-selftest-orphan-topic'" \
  python3 scripts/verify_orphans.py --topics "$plant_topics"

# ── (f) live tree still GREEN, and nothing planted leaked into it ───────────
echo "==> (f) live tree GREEN"
rc=0
live_out="$(python3 scripts/verify_orphans.py 2>&1)" || rc=$?
printf '%s\n' "$live_out"
[ "$rc" -eq 0 ] || fail "live orphan check exited $rc (selftest must not dirty the tree)"
printf '%s\n' "$live_out" | grep -q 'orphan integrity GREEN' \
  || fail "live output missing 'orphan integrity GREEN'"

for leaked in bank/items/zz-selftest-orphan-ref.toml \
              bank/items/zz-selftest-unanchored.toml; do
  [ -f "$leaked" ] && fail "specimen leaked into the live tree: $leaked"
done
grep -q 'zz-selftest-orphan-topic' knowledge/topics.toml \
  && fail "specimen topic leaked into knowledge/topics.toml"
ok "live tree clean (no specimen leaked)"

echo "INJECTIONS=$INJ SUITE=$SUITE_NAME"
echo "selftest_orphan: PASSED (a empty bank · b empty topics · c orphan ref · d unanchored · e orphan topic · f live GREEN)"
exit 0
