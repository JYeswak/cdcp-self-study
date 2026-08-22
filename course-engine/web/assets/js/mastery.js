/**
 * Mastery state machine (L6-S2 / bd-iwp) — practiced / mastered from quiz scores.
 *
 * Schema (localStorage key `cdcp.mastery.v1`):
 * {
 *   schema_version: 1,
 *   modules: {
 *     [moduleKey]: {
 *       module: number | string,
 *       attempts: [
 *         { correct: number, total: number, ratio: number, at_ms: number }
 *       ],
 *       best_ratio: number
 *     }
 *   }
 * }
 *
 * Laws live in `cdcp_schedule` and are decided by WASM (schedule_bridge).
 * This module persists attempts and renders. JS does not hold the bars.
 *
 * Study signal only — never a CDCP / EPI credential claim.
 *
 * @module mastery
 */

import {
  dayMs as wasmDayMs,
  practicedRatio as wasmPracticedRatio,
  masteredRatio as wasmMasteredRatio,
  masteredMinGapMs as wasmMasteredMinGapMs,
  isPracticedRatio,
  isMasteredAttempts,
} from "./schedule_bridge.js";

export const STORAGE_KEY = "cdcp.mastery.v1";
export const SCHEMA_VERSION = 1;

/** Day length in ms from `cdcp_schedule` (no DST). */
export function dayMs() {
  return wasmDayMs();
}

export function practicedRatio() {
  return wasmPracticedRatio();
}

export function masteredRatio() {
  return wasmMasteredRatio();
}

export function masteredMinGapMs() {
  return wasmMasteredMinGapMs();
}

/**
 * Normalize a module identifier to a stable string map key.
 * @param {unknown} module
 * @returns {string | null}
 */
export function moduleKey(module) {
  if (typeof module === "number" && isFinite(module)) {
    return String(module);
  }
  if (typeof module === "string" && module.length > 0) {
    return module;
  }
  return null;
}

/**
 * @param {number} correct
 * @param {number} total
 * @returns {number} ratio in [0, 1], or 0 if total invalid
 */
export function ratioOf(correct, total) {
  const c = typeof correct === "number" && isFinite(correct) ? correct : 0;
  const t = typeof total === "number" && isFinite(total) ? total : 0;
  if (t <= 0) return 0;
  const r = c / t;
  if (!isFinite(r)) return 0;
  if (r < 0) return 0;
  if (r > 1) return 1;
  return r;
}

/**
 * Mastery evidence contract mirrored by `cdcp_schedule::attempt_is_complete`.
 * A caller may omit `answered` for the historical complete-result API; an
 * explicit shortfall is always rejected.
 * @param {number} answered
 * @param {number} total
 * @returns {boolean}
 */
export function attemptIsComplete(answered, total) {
  return (
    Number.isFinite(answered) &&
    Number.isFinite(total) &&
    total > 0 &&
    answered === total
  );
}

/**
 * @param {unknown} raw
 * @returns {{ schema_version: number, modules: Record<string, object> }}
 */
export function normalizeState(raw) {
  const empty = {
    schema_version: SCHEMA_VERSION,
    modules: Object.create(null),
  };
  if (!raw || typeof raw !== "object") return empty;
  const modsIn = /** @type {{modules?: unknown}} */ (raw).modules;
  if (!modsIn || typeof modsIn !== "object") return empty;

  /** @type {Record<string, object>} */
  const modules = Object.create(null);
  const keys = Object.keys(/** @type {object} */ (modsIn));
  for (let i = 0; i < keys.length; i++) {
    const k = keys[i];
    const m = /** @type {Record<string, unknown>} */ (
      /** @type {Record<string, unknown>} */ (modsIn)[k]
    );
    if (!m || typeof m !== "object") continue;

    const modId =
      typeof m.module === "number" || typeof m.module === "string"
        ? m.module
        : isFinite(Number(k))
          ? Number(k)
          : k;

    const attemptsIn = Array.isArray(m.attempts) ? m.attempts : [];
    /** @type {Array<{correct:number,total:number,ratio:number,at_ms:number}>} */
    const attempts = [];
    let best = 0;
    for (let j = 0; j < attemptsIn.length; j++) {
      const a = attemptsIn[j];
      if (!a || typeof a !== "object") continue;
      const correct =
        typeof a.correct === "number" && isFinite(a.correct) ? a.correct : 0;
      const total =
        typeof a.total === "number" && isFinite(a.total) ? a.total : 0;
      const atMs =
        typeof a.at_ms === "number" && isFinite(a.at_ms)
          ? a.at_ms
          : typeof a.atMs === "number" && isFinite(a.atMs)
            ? a.atMs
            : 0;
      const ratio =
        typeof a.ratio === "number" && isFinite(a.ratio)
          ? Math.max(0, Math.min(1, a.ratio))
          : ratioOf(correct, total);
      attempts.push({
        correct: correct,
        total: total,
        ratio: ratio,
        at_ms: atMs,
      });
      if (ratio > best) best = ratio;
    }

    if (
      typeof m.best_ratio === "number" &&
      isFinite(m.best_ratio) &&
      m.best_ratio > best
    ) {
      best = Math.max(0, Math.min(1, m.best_ratio));
    }

    const key = moduleKey(modId) || k;
    modules[key] = {
      module: modId,
      attempts: attempts,
      best_ratio: best,
    };
  }
  return { schema_version: SCHEMA_VERSION, modules: modules };
}

/**
 * @param {Storage | null | undefined} [store]
 * @returns {Storage | null}
 */
function resolveStore(store) {
  if (store) return store;
  if (typeof localStorage !== "undefined") return localStorage;
  return null;
}

/**
 * Load mastery state from localStorage (or injectable store).
 * @param {Storage} [store]
 * @returns {{ schema_version: number, modules: Record<string, object> }}
 */
export function loadState(store) {
  const s = resolveStore(store);
  if (!s) return normalizeState(null);
  try {
    const raw = s.getItem(STORAGE_KEY);
    if (!raw) return normalizeState(null);
    return normalizeState(JSON.parse(raw));
  } catch (_) {
    return normalizeState(null);
  }
}

/**
 * Persist mastery state.
 * @param {{ schema_version?: number, modules: Record<string, object> }} state
 * @param {Storage} [store]
 * @returns {boolean}
 */
export function saveState(state, store) {
  const s = resolveStore(store);
  if (!s) return false;
  try {
    const payload = {
      schema_version: SCHEMA_VERSION,
      modules: state && state.modules ? state.modules : Object.create(null),
    };
    s.setItem(STORAGE_KEY, JSON.stringify(payload));
    return true;
  } catch (_) {
    return false;
  }
}

/**
 * Full state snapshot (alias of loadState for the public API).
 * @param {{ store?: Storage }} [opts]
 * @returns {{ schema_version: number, modules: Record<string, object> }}
 */
export function getState(opts) {
  const o = opts || {};
  return loadState(o.store);
}

/**
 * Append a quiz result and recompute best_ratio.
 *
 * @param {{ module: number|string, correct: number, total: number, atMs?: number }} result
 * @param {{ store?: Storage, nowMs?: number }} [opts]
 * @returns {{ module: *, attempts: object[], best_ratio: number } | null}
 */
export function recordQuizResult(result, opts) {
  if (!result || result.module == null) return null;
  const key = moduleKey(result.module);
  if (!key) return null;

  const o = opts || {};
  const now =
    typeof result.atMs === "number" && isFinite(result.atMs)
      ? result.atMs
      : typeof o.nowMs === "number" && isFinite(o.nowMs)
        ? o.nowMs
        : Date.now();

  const correct =
    typeof result.correct === "number" && isFinite(result.correct)
      ? result.correct
      : 0;
  const total =
    typeof result.total === "number" && isFinite(result.total)
      ? result.total
      : 0;
  const answered =
    typeof result.answered === "number" && isFinite(result.answered)
      ? result.answered
      : total;
  const complete =
    result.complete !== false && attemptIsComplete(answered, total);
  // Partial attempts remain visible in Results, but never become evidence for
  // practiced/mastered module state. This is the product-level call site for
  // cdcp_schedule::attempt_is_complete.
  if (!complete) return null;
  const ratio = ratioOf(correct, total);

  const state = loadState(o.store);
  const prev = state.modules[key] || {
    module: result.module,
    attempts: [],
    best_ratio: 0,
  };
  const attempts = Array.isArray(prev.attempts) ? prev.attempts.slice() : [];
  attempts.push({
    correct: correct,
    total: total,
    ratio: ratio,
    at_ms: now,
  });
  const best = Math.max(
    typeof prev.best_ratio === "number" ? prev.best_ratio : 0,
    ratio
  );
  const entry = {
    module: prev.module != null ? prev.module : result.module,
    attempts: attempts,
    best_ratio: best,
  };
  state.modules[key] = entry;
  saveState(state, o.store);
  return entry;
}

/**
 * Best attempt ratio for a module (0 if none).
 * @param {number|string} module
 * @param {{ store?: Storage, state?: object }} [opts]
 * @returns {number}
 */
export function bestRatio(module, opts) {
  const o = opts || {};
  const key = moduleKey(module);
  if (!key) return 0;
  const state = o.state || loadState(o.store);
  const entry = state.modules[key];
  if (!entry) return 0;
  if (typeof entry.best_ratio === "number" && isFinite(entry.best_ratio)) {
    return entry.best_ratio;
  }
  const attempts = Array.isArray(entry.attempts) ? entry.attempts : [];
  let best = 0;
  for (let i = 0; i < attempts.length; i++) {
    const r = attempts[i] && attempts[i].ratio;
    if (typeof r === "number" && r > best) best = r;
  }
  return best;
}

/**
 * practiced: WASM `cdcp_is_practiced` on best ratio.
 * @param {number|string} module
 * @param {{ store?: Storage, state?: object }} [opts]
 * @returns {boolean}
 */
export function isPracticed(module, opts) {
  return isPracticedRatio(bestRatio(module, opts));
}

/**
 * mastered: WASM `cdcp_is_mastered` on stored attempts.
 * @param {number|string} module
 * @param {{ store?: Storage, state?: object }} [opts]
 * @returns {boolean}
 */
export function isMastered(module, opts) {
  const o = opts || {};
  const key = moduleKey(module);
  if (!key) return false;
  const state = o.state || loadState(o.store);
  const entry = state.modules[key];
  if (!entry || !Array.isArray(entry.attempts)) return false;
  return isMasteredAttempts(entry.attempts);
}

/**
 * Modules that meet practiced threshold, sorted by module key.
 * @param {{ store?: Storage, state?: object }} [opts]
 * @returns {(number|string)[]}
 */
export function listPracticed(opts) {
  const o = opts || {};
  const state = o.state || loadState(o.store);
  const out = [];
  const keys = Object.keys(state.modules);
  for (let i = 0; i < keys.length; i++) {
    const entry = state.modules[keys[i]];
    if (!entry) continue;
    if (isPracticed(entry.module != null ? entry.module : keys[i], { state: state })) {
      out.push(entry.module != null ? entry.module : keys[i]);
    }
  }
  out.sort(function (a, b) {
    const na = Number(a);
    const nb = Number(b);
    if (isFinite(na) && isFinite(nb)) return na - nb;
    return String(a).localeCompare(String(b));
  });
  return out;
}

/**
 * Modules that meet mastered law, sorted by module key.
 * @param {{ store?: Storage, state?: object }} [opts]
 * @returns {(number|string)[]}
 */
export function listMastered(opts) {
  const o = opts || {};
  const state = o.state || loadState(o.store);
  const out = [];
  const keys = Object.keys(state.modules);
  for (let i = 0; i < keys.length; i++) {
    const entry = state.modules[keys[i]];
    if (!entry) continue;
    if (isMastered(entry.module != null ? entry.module : keys[i], { state: state })) {
      out.push(entry.module != null ? entry.module : keys[i]);
    }
  }
  out.sort(function (a, b) {
    const na = Number(a);
    const nb = Number(b);
    if (isFinite(na) && isFinite(nb)) return na - nb;
    return String(a).localeCompare(String(b));
  });
  return out;
}

// Browser global for console / non-module consumers.
if (typeof globalThis !== "undefined") {
  globalThis.CdcpMastery = {
    STORAGE_KEY,
    SCHEMA_VERSION,
    dayMs,
    practicedRatio,
    masteredRatio,
    masteredMinGapMs,
    moduleKey,
    ratioOf,
    normalizeState,
    loadState,
    saveState,
    getState,
    recordQuizResult,
    attemptIsComplete,
    bestRatio,
    isPracticed,
    isMastered,
    listPracticed,
    listMastered,
  };
}
