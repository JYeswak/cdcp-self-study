#!/usr/bin/env bash
# Loop watchdog ESCALATION wrapper — the reader the ALERT file never had.
#
# WHY THIS EXISTS (measured 2026-08-18): .flywheel/ALERT held 145 consecutive
# `STALL` rows written every 10 minutes across 19 hours while 419 commits landed
# on main. The watchdog was correct and enforced by no one. An alarm nobody
# consumes is a heartbeat with no reader — CLAUDE.md §6, "3 identical
# consecutive blockers => escalation required ... you cannot silence it."
#
# WHAT IT ENFORCES (floor-raise, stated plainly): after N consecutive STALL
# observations it writes URGENT_JOSH.md and fires a desktop notification.
# WHAT IT CANNOT DECIDE: whether the work happening outside the ledger is good.
# It observes that the tick choke-point is being bypassed, nothing more.
#
# SILENCE CONTRACT: emits NOTHING on the healthy path. A monitor that speaks on
# every run trains its reader to ignore it.
#
# NEVER DISPATCHES WORK. loop-engineering forbids a clock-paced pulse driving the
# loop; this only observes and surfaces. If it ever sends a prompt, it has become
# the thing the doctrine forbids.
set -euo pipefail

# cron/launchd start WITHOUT the login-shell env (CLAUDE.md §0). Everything this
# script needs is passed explicitly; nothing is inherited by convenience.
export PATH="/Users/josh/.local/bin:/Users/josh/.cargo/bin:/opt/homebrew/bin:/usr/bin:/bin"
export GIT_PAGER=cat PAGER=cat

REPO="${1:-/Users/josh/cdcp-self-study/course-engine}"
FW="$REPO/.flywheel"
ALERT="$FW/ALERT"
URGENT="$FW/URGENT_JOSH.md"
STREAK_FILE="$FW/.stall-streak"
ESCALATE_AFTER="${ESCALATE_AFTER:-3}"
TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

# Fail closed: a watchdog that cannot find its own subject is BLOCKED, not green.
if [ ! -x "$FW/watchdog.sh" ]; then
  echo "watchdog-escalate: BLOCKED: no watchdog at $FW/watchdog.sh" >&2
  exit 78
fi

set +e
OUT="$("$FW/watchdog.sh" "$REPO" 2>&1)"
RC=$?
set -e

if [ "$RC" -eq 0 ]; then
  # Healthy. Reset the streak and say nothing (silence contract).
  : > "$STREAK_FILE"
  [ -f "$URGENT" ] && mv "$URGENT" "$FW/URGENT_JOSH.resolved-$TS"
  exit 0
fi

# STALLED. Append the observation, then count how deep the streak is.
echo "STALL $TS" >> "$ALERT"
STREAK=$(( $(cat "$STREAK_FILE" 2>/dev/null || echo 0) + 1 ))
echo "$STREAK" > "$STREAK_FILE"

if [ "$STREAK" -lt "$ESCALATE_AFTER" ]; then
  exit 3
fi

# Escalation. This is the leg the old watchdog was missing entirely.
LAST_TICK=$(tail -1 "$FW/tick-ledger.jsonl" 2>/dev/null | sed 's/.*"tick": *"\([^"]*\)".*/\1/' || echo "unknown")
LEDGER_AGE=$(( ( $(date +%s) - $(stat -f %m "$FW/tick-ledger.jsonl" 2>/dev/null || date +%s) ) / 3600 ))
COMMITS=$(cd "$REPO" && git log --oneline --since="${LEDGER_AGE} hours ago" 2>/dev/null | wc -l | tr -d ' ')

cat > "$URGENT" <<EOF
# URGENT — the tick loop is being bypassed

**Raised:** $TS · **consecutive STALL observations:** $STREAK

## What was observed

- Last ledger row: \`$LAST_TICK\`, written **${LEDGER_AGE}h ago**.
- Commits on main in that same window: **$COMMITS**.
- Watchdog verdict: $OUT

## What that means

Work is landing without passing the tick choke-point. The ledger is therefore not
a record of what happened — it is a record of what happened *until it stopped
being written*. Every metric computed from it (product-move rate, RED ratio,
value density) is measuring a window that closed ${LEDGER_AGE}h ago.

This is not a claim that the work is bad. It is a claim that the loop cannot see it.

## The single decision this needs

Either (a) route the active work through \`emit_tick\` so the ledger resumes, or
(b) record — in the CHARTER, in place — that this repo's work is deliberately
outside the loop for now, and say until when.

Silence is option (c), and option (c) is what produced 145 unread STALL rows.

*Written by \`.flywheel/watchdog-escalate.sh\`. Resolved automatically on the next
healthy observation (this file is renamed, never deleted).*
EOF

/usr/bin/osascript -e 'display notification "Tick loop bypassed — see .flywheel/URGENT_JOSH.md" with title "cdcp loop STALLED"' 2>/dev/null || true

echo "watchdog-escalate: ESCALATED after $STREAK consecutive stalls -> $URGENT" >&2
exit 3
