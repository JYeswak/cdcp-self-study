/**
 * Browser glue for `cdcp_schedule` via cdcp_wasm.
 *
 * JS renders and persists. WASM decides the interval ladder and mastery
 * thresholds. There is no JS copy of that law — a second implementation is
 * how learners got scored on withdrawn items while every Rust gate was green.
 *
 * Requires `await loadWasm()` first (same instance as grade_bridge).
 *
 * @module schedule_bridge
 */

import { loadWasm, isWasmReady, wasmExports, wasmMemory } from "./grade_bridge.js";

function requireExports() {
  if (!isWasmReady()) {
    throw new Error(
      "WASM not loaded — schedule law lives in cdcp_schedule. Call await loadWasm() first."
    );
  }
  const ex = wasmExports();
  const ok = ex.cdcp_schedule_ok();
  if (ok !== 1) {
    throw new Error(
      "cdcp_schedule_ok != 1 — empty ladder or zero mastery threshold is an ERROR"
    );
  }
  return ex;
}

function normalizeCurrent(currentIntervalDays) {
  return typeof currentIntervalDays === "number" &&
    isFinite(currentIntervalDays) &&
    currentIntervalDays > 0
    ? Math.floor(currentIntervalDays)
    : 0;
}

/** Compiled interval steps (days). Throws if the wasm ladder is empty. */
export function intervalSteps() {
  const ex = requireExports();
  const n = ex.cdcp_interval_step_count();
  if (n <= 0) {
    throw new Error("empty interval ladder is an ERROR");
  }
  const out = [];
  for (let i = 0; i < n; i++) {
    const s = ex.cdcp_interval_step(i);
    if (s <= 0) {
      throw new Error("interval step <= 0 is an ERROR");
    }
    out.push(s);
  }
  return out;
}

/**
 * Next interval in days. WASM decides.
 *
 * @param {number} currentIntervalDays
 * @param {boolean} correct
 * @returns {number}
 */
export function nextIntervalDays(currentIntervalDays, correct) {
  const ex = requireExports();
  return ex.cdcp_next_interval_days(normalizeCurrent(currentIntervalDays), correct ? 1 : 0);
}

/** Fixed day length in ms (no DST). */
export function dayMs() {
  return requireExports().cdcp_day_ms();
}

/** Practiced bar as 0..=1 (800 milli → 0.8). */
export function practicedRatio() {
  const m = requireExports().cdcp_practiced_milli();
  if (m <= 0) throw new Error("practiced threshold 0 is an ERROR");
  return m / 1000;
}

/** Mastered per-attempt bar as 0..=1 (900 milli → 0.9). */
export function masteredRatio() {
  const m = requireExports().cdcp_mastered_milli();
  if (m <= 0) throw new Error("mastered threshold 0 is an ERROR");
  return m / 1000;
}

export function practicedMilli() {
  const m = requireExports().cdcp_practiced_milli();
  if (m <= 0) throw new Error("practiced threshold 0 is an ERROR");
  return m;
}

export function masteredMilli() {
  const m = requireExports().cdcp_mastered_milli();
  if (m <= 0) throw new Error("mastered threshold 0 is an ERROR");
  return m;
}

export function masteredMinGapMs() {
  return requireExports().cdcp_mastered_min_gap_ms();
}

export function firstStepDays() {
  return intervalSteps()[0];
}

export function capDays() {
  const steps = intervalSteps();
  return steps[steps.length - 1];
}

/**
 * @param {number} ratio 0..=1
 * @returns {number} parts per thousand
 */
export function ratioToMilli(ratio) {
  if (typeof ratio !== "number" || !isFinite(ratio) || ratio <= 0) return 0;
  if (ratio >= 1) return 1000;
  return Math.round(ratio * 1000);
}

/** @param {number} ratioMilli */
export function isPracticedMilli(ratioMilli) {
  const m =
    typeof ratioMilli === "number" && isFinite(ratioMilli) ? Math.floor(ratioMilli) : 0;
  return requireExports().cdcp_is_practiced(m) === 1;
}

/** @param {number} ratio 0..=1 */
export function isPracticedRatio(ratio) {
  return isPracticedMilli(ratioToMilli(ratio));
}

/**
 * @param {Array<{ratio?: number, ratio_milli?: number, at_ms?: number}>} attempts
 * @returns {boolean}
 */
export function isMasteredAttempts(attempts) {
  const ex = requireExports();
  const arr = Array.isArray(attempts) ? attempts : [];
  const payload = JSON.stringify(
    arr.map(function (a) {
      const row = {
        at_ms: a && typeof a.at_ms === "number" && isFinite(a.at_ms) ? a.at_ms : 0,
      };
      if (a && typeof a.ratio_milli === "number") {
        row.ratio_milli = a.ratio_milli;
      } else if (a && typeof a.ratio === "number") {
        row.ratio = a.ratio;
      }
      return row;
    })
  );
  const bytes = new TextEncoder().encode(payload);
  const ptr = ex.cdcp_alloc(bytes.length);
  if (!ptr && bytes.length > 0) {
    throw new Error("cdcp_alloc returned 0");
  }
  try {
    const mem = new Uint8Array(wasmMemory().buffer);
    mem.set(bytes, ptr);
    const rc = ex.cdcp_is_mastered(ptr, bytes.length);
    if (rc < 0) {
      const mem2 = new Uint8Array(wasmMemory().buffer);
      const lastPtr = ex.cdcp_last_ptr();
      const errLen = -rc;
      const msg = new TextDecoder().decode(mem2.slice(lastPtr, lastPtr + errLen));
      throw new Error("is_mastered: " + msg);
    }
    return rc === 1;
  } finally {
    try {
      ex.cdcp_free(ptr, bytes.length);
    } catch {
      /* ignore */
    }
  }
}

export { loadWasm, isWasmReady };

if (typeof globalThis !== "undefined") {
  globalThis.CdcpSchedule = {
    loadWasm,
    isWasmReady,
    intervalSteps,
    nextIntervalDays,
    dayMs,
    practicedRatio,
    masteredRatio,
    practicedMilli,
    masteredMilli,
    masteredMinGapMs,
    firstStepDays,
    capDays,
    ratioToMilli,
    isPracticedMilli,
    isPracticedRatio,
    isMasteredAttempts,
  };
}
