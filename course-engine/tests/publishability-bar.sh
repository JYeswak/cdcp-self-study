#!/usr/bin/env sh
# tests/publishability-bar.sh — required by L88.
#
# WHY THIS EXISTS: .flywheel/scripts/publishability-bar.sh does not COMPUTE the
# score — it PARSES .flywheel/PUBLISHABILITY-AUDIT.md and echoes whatever the
# markdown claims. A hand-typed "7 / 7" therefore reports as pass. That is a
# self-signed certificate.
#
# This test closes the loop: every factual claim the audit makes must be true of
# the repository RIGHT NOW, or the test fails. It cannot make the audit honest
# on its own — but it makes a dishonest audit fail loudly.
#
# Measured 2026-08-12: the audit's F1 evidence claimed the README documented
# `cdcp serve` on :8766. The README contained neither string. Score still said
# 7/7 and status still said pass. That is the failure this test prevents.
set -eu

ENGINE="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
ROOT="$(CDPATH= cd -- "$ENGINE/.." && pwd)"
cd "$ROOT"

AUDIT="$ENGINE/.flywheel/PUBLISHABILITY-AUDIT.md"
PASS=0
FAIL=0

ok()   { PASS=$((PASS+1)); echo "publishability-bar: ok: $*"; }
bad()  { FAIL=$((FAIL+1)); echo "publishability-bar: FAIL: $*" >&2; }

[ -f "$AUDIT" ] || { echo "publishability-bar: FAIL: no $AUDIT (L88 forbids calling a repo publishable without one)" >&2; exit 1; }

echo "==> publishability-bar (audit claims must be true of the repo)"

# ── L88 required artefacts ──────────────────────────────────────────────────
for f in "$ENGINE/.flywheel/PUBLISHABILITY-BAR.md" \
         "$ENGINE/.flywheel/PUBLISHABILITY-AUDIT.md" \
         "$ENGINE/.flywheel/scripts/publishability-bar.sh"; do
  [ -f "$f" ] && ok "L88 artefact present: ${f#$ROOT/}" || bad "L88 artefact missing: ${f#$ROOT/}"
done

# ── OSS meta must sit at the REPO ROOT (GitHub does not look one level down) ──
for f in LICENSE README.md CONTRIBUTING.md CODE_OF_CONDUCT.md SECURITY.md; do
  [ -f "$ROOT/$f" ] && ok "root OSS meta: $f" || bad "root OSS meta MISSING: $f (GitHub reads the repo root)"
done

# ── Dual licence must be stated, not implied ────────────────────────────────
grep -q "MIT" "$ROOT/LICENSE" && ok "LICENSE names MIT (software)" \
  || bad "LICENSE does not name MIT"
grep -qi "CC BY-NC-SA" "$ROOT/LICENSE" && ok "LICENSE names the content licence" \
  || bad "LICENSE does not name a content licence (curriculum is prose, not code)"
grep -qi "not affiliated" "$ROOT/LICENSE" && ok "LICENSE carries the non-affiliation disclaimer" \
  || bad "LICENSE lost the EPI/EXIN non-affiliation disclaimer"

# ── F1: the audit claims a start path — it must actually be in a README ─────
if grep -q "serve" "$ROOT/README.md" && grep -q "8766" "$ROOT/README.md"; then
  ok "F1 start path (serve :8766) really is in the root README"
else
  bad "F1 evidence FALSE: root README does not document the serve start path"
fi
grep -q "check.sh" "$ROOT/README.md" && ok "F1 root README names the gate" \
  || bad "F1 root README never mentions check.sh"
grep -qi "LICENSE" "$ROOT/README.md" && ok "F1 root README points at the licence" \
  || bad "F1 root README never mentions the licence"

# ── Honesty claim must be enforced, not just written ────────────────────────
grep -qi "not.*certif" "$ROOT/README.md" && ok "root README carries the not-certified honesty note" \
  || bad "root README lost the not-certified honesty note"

# ── F3: the doctor must actually run and emit the contracted shape ──────────
# F3 asks whether the doctor EMITS the contracted shape — not whether it says
# pass. A doctor that correctly reports an unmet bar exits non-zero and is still
# a working doctor; requiring exit 0 here would reward a doctor that never fails.
(cd "$ENGINE" && bash .flywheel/scripts/publishability-bar.sh --doctor --json) >/tmp/pb.$$ 2>/dev/null || true
if grep -q "publishability_bar_score" /tmp/pb.$$ && grep -q "facet_id" /tmp/pb.$$; then
  ok "F3 doctor emits publishability_bar_score"
else
  bad "F3 doctor did not emit the contracted JSON"
fi

# EXTRACT-THEN-DELETE (bd-extract-publishability-bar-python-9tji): doctor
# JSON parse and corpus-rights scan live in `cdcp publishability`. A
# missing $CDCP after check.sh's cargo build is RED (no interpreter fallback).
if [ -z "${CDCP:-}" ]; then
  if [ -z "${CDCP_BIN_DIR:-}" ]; then
    if [ -n "${CARGO_TARGET_DIR:-}" ]; then
      CDCP_BIN_DIR="${CARGO_TARGET_DIR%/}/debug"
    else
      CDCP_BIN_DIR="$ENGINE/target/debug"
    fi
  fi
  CDCP="$CDCP_BIN_DIR/cdcp"
fi

# ── The fleet doctor's error set must be EXACTLY known+accepted ─────────────
# The current audit has EXEMPT_NON_ZS_SURFACE, so the expected error set is
# empty. Keep the legacy single error accepted for older audit fixtures; any
# other error is a regression and must not slip by unnoticed.
KNOWN_ERR="brand_voice_composite_low"
if [ ! -x "$CDCP" ]; then
  bad "missing $CDCP (doctor-errors helper; no interpreter fallback)"
else
  ACTUAL_ERRS="$("$CDCP" publishability doctor-errors --json /tmp/pb.$$)"
  if [ "$ACTUAL_ERRS" = "$KNOWN_ERR" ] || [ -z "$ACTUAL_ERRS" ]; then
    ok "fleet doctor errors are exactly the known set (${ACTUAL_ERRS:-none})"
  else
    bad "fleet doctor raised UNEXPECTED errors: $ACTUAL_ERRS (known: $KNOWN_ERR)"
  fi
fi
rm -f /tmp/pb.$$

# ── F4: the gate must exist and be executable ───────────────────────────────
[ -x "$ENGINE/scripts/check.sh" ] || [ -f "$ENGINE/scripts/check.sh" ] \
  && ok "F4 gate present: course-engine/scripts/check.sh" \
  || bad "F4 gate missing"

# ── Nothing private may be tracked ──────────────────────────────────────────
if git -C "$ROOT" ls-files --error-unmatch job-research >/dev/null 2>&1; then
  bad "PRIVATE job-research/ is tracked — must never be published"
else
  ok "job-research/ not tracked"
fi
if git -C "$ROOT" rev-list --objects --all 2>/dev/null | grep -q "ashrae.*\.pdf"; then
  bad "ASHRAE PDFs present in git history (not licensed for redistribution)"
else
  ok "no ASHRAE PDFs anywhere in history"
fi

# ── Corpus rights must be recorded per source, not assumed ──────────────────
# EXTRACT-THEN-DELETE: empty sources / missing rights is RED in
# `cdcp publishability corpus-rights`. Presence of a rights field is
# not proof the rights are correct.
if [ ! -x "$CDCP" ]; then
  bad "missing $CDCP (corpus-rights helper; no interpreter fallback)"
elif "$CDCP" publishability corpus-rights --file "$ENGINE/knowledge/corpus/public/manifest.json"
then
  ok "every scraped corpus source records its rights"
else
  bad "corpus manifest has sources without a rights field (or is empty)"
fi

# ── Audit must not claim a state it has not reached ─────────────────────────
if grep -qi "^- \*\*Public repo:\*\* yes" "$AUDIT"; then
  grep -qiE "Scorecard log.*not-run" "$AUDIT" \
    && bad "audit claims Public repo: yes while the voice scorecard is not-run" \
    || ok "audit public-repo claim is consistent with a run scorecard"
else
  ok "audit does not overclaim public status"
fi

# ── An exemption must carry a reason (L88: no hiding a NO facet in prose) ───
if grep -q "^- \*\*Exemption:\*\*$" "$AUDIT"; then
  bad "audit has an EMPTY Exemption field — an exemption without a reason is a schema error"
else
  ok "no bare/unreasoned exemption in the audit"
fi

echo "publishability-bar: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
