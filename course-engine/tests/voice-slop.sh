#!/usr/bin/env sh
# tests/voice-slop.sh — the portion of the ZestStream voice gate that DOES apply
# to this repo, enforced mechanically.
#
# The full ZS public-voice gate is EXEMPT here (see .flywheel/PUBLISHABILITY-AUDIT.md
# "Exemption"): this is a personal educational repo, not a ZS-branded product
# surface, and the ZS banned-word list is marketing vocabulary — it flags
# "mission-critical", which is the name of syllabus Module 01.
#
# What still applies to ANY public copy, and is checked here:
#   1. no marketing slop (the ZS banned list MINUS legitimate domain terms)
#   2. the honesty note survives on the front door
#   3. no certification overclaim
#
# An exemption that is merely declared is not a gate. This is the part that bites.
set -eu

ROOT="$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); echo "voice-slop: ok: $*"; }
bad() { FAIL=$((FAIL+1)); echo "voice-slop: FAIL: $*" >&2; }

# ZS banned marketing vocabulary. Domain terms deliberately excluded with reason:
#   mission-critical — EPI syllabus Module 01 ("The Mission Critical Site")
SLOP="best-in-class
empower
enterprise-grade
harness
holistic
in today's world
operationalize
paradigm
revolutionize
synergy
transform the way
cutting-edge
game-changing
seamlessly
unlock the power"

echo "==> voice-slop (applicable slice of the ZS voice gate)"

for f in README.md course-engine/README.md CONTRIBUTING.md SECURITY.md; do
  [ -f "$f" ] || { bad "missing public copy: $f"; continue; }
  hits=""
  # shellcheck disable=SC2086
  printf '%s\n' "$SLOP" | while IFS= read -r term; do
    [ -n "$term" ] || continue
    if grep -qi -- "$term" "$f"; then echo "$term"; fi
  done > /tmp/slop.$$ || true
  hits="$(cat /tmp/slop.$$)"; rm -f /tmp/slop.$$
  if [ -n "$hits" ]; then
    bad "$f contains marketing slop: $(printf '%s' "$hits" | tr '\n' ' ')"
  else
    ok "$f free of marketing slop"
  fi
done

# Honesty note must survive on the front door.
grep -qi "not.*official\|not.*certif" README.md \
  && ok "root README keeps the honesty note" \
  || bad "root README lost the honesty note"

# No certification overclaim anywhere in public copy.
if grep -rniE "you (are|will be) (now )?(CDCP|EPI) certified|grants? (you )?certification" \
     README.md course-engine/README.md 2>/dev/null | grep -viE "not |never |does not"; then
  bad "certification overclaim in public copy"
else
  ok "no certification overclaim"
fi

echo "voice-slop: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
