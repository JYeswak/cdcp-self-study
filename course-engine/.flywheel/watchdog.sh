#!/usr/bin/env bash
# Loop WATCHDOG — detects a stalled tick loop and alerts. NEVER dispatches work.
#
# loop-engineering forbids a clock-paced pulse driving the loop ("work-paced, not
# clock-paced"; "do not create a daemon, timer, watcher, or persistent pulse to CONTINUE
# the loop"). This honours that: the loop is continued by tick completion, and this script
# only observes. If it ever dispatches work, it has become the thing the doctrine forbids.
#
# Exit 0 = healthy or idle-by-design. Exit 3 = STALLED (alert).
set -euo pipefail
REPO="${1:-$HOME/cdcp-self-study/course-engine}"
STALL_MIN="${STALL_MIN:-45}"
LEDGER="$REPO/.flywheel/tick-ledger.jsonl"
STATE="$REPO/.flywheel/STATE.md"

fail() { echo "watchdog: STALLED: $*" >&2; exit 3; }

[ -f "$LEDGER" ] || fail "no tick ledger at $LEDGER"

# 1. Pause switch honoured (loop-engineering: halt exit 3 means stop loop work).
if [ -f "$REPO/.flywheel/PAUSE" ]; then
  echo "watchdog: PAUSED by .flywheel/PAUSE — not a stall."; exit 0
fi

# 2. Is there anything to do at all? An empty ready queue is a refill point, not a stall.
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
ready=$(cd "$REPO" && br ready --json 2>/dev/null | python3 -c 'import json,sys
try:
    d=json.load(sys.stdin); r=d["issues"] if isinstance(d,dict) else d; print(len(r))
except Exception: print(-1)' 2>/dev/null || echo -1)
if [ "$ready" = "0" ]; then
  echo "watchdog: ready queue empty — refill point, not a stall."; exit 0
fi

# 3. Staleness: how long since the last ledger row changed?
age_min=$(( ( $(date +%s) - $(stat -f %m "$LEDGER") ) / 60 ))
if [ "$age_min" -gt "$STALL_MIN" ]; then
  fail "no tick recorded in ${age_min}m (threshold ${STALL_MIN}m) while ${ready} bead(s) are ready"
fi

# 4. RED streak — the charter's own pause condition, observed not enforced.
red=$(tail -5 "$LEDGER" | grep -c '"verdict":"RED"' || true)
if [ "$red" -ge 3 ]; then
  fail "RED streak: ${red} of the last 5 ticks are RED (charter red_streak_pause = 3)"
fi

echo "watchdog: healthy — last tick ${age_min}m ago, ${ready} ready, ${red}/5 RED"
[ -f "$STATE" ] || echo "watchdog: WARN missing STATE.md" >&2
exit 0
