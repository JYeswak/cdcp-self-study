/**
 * Minimal SRS (L5-S7 / bd-ca8) — 1d / 3d interval steps in localStorage.
 *
 * Schema (localStorage key `cdcp.srs.v1`):
 * {
 *   schema_version: 1,
 *   cards: {
 *     [item_id]: {
 *       item_id: string,
 *       interval_days: number,  // 0 | 1 | 3 (minimal ladder)
 *       due_at: number,         // epoch ms when card is due
 *       reps: number,           // successful reviews
 *       lapses: number,         // failed reviews after first schedule
 *       updated_at: number      // epoch ms
 *     }
 *   }
 * }
 *
 * Interval law (pure, unit-tested in scripts/smoke_srs.mjs):
 *   nextIntervalDays(current, correct):
 *     wrong  → 1
 *     correct && current < 1 → 1   (first success after new / reset)
 *     correct && current < 3 → 3
 *     correct && current >= 3 → 3  (cap; re-schedule at 3d)
 *
 * Drill-10 (L6-S6): selectDueOnly(cards, nowMs, limit=10) /
 * listDueDrill — due_at ≤ now, earliest first, cap 10.
 *
 * Missed-item feed (localStorage key `cdcp.drill.missed.v1`):
 * {
 *   schema_version: 1,
 *   source: "mock" | "quiz",
 *   exam_id: string,
 *   seed: number | null,
 *   bank_hash: string,
 *   saved_at: number,
 *   item_ids: string[]
 * }
 *
 * No LLM. Pedagogy-only — never a cert claim.
 *
 * @module srs
 */

export const SRS_STORAGE_KEY = "cdcp.srs.v1";
export const MISSED_STORAGE_KEY = "cdcp.drill.missed.v1";
export const SRS_SCHEMA_VERSION = 1;
export const MISSED_SCHEMA_VERSION = 1;

/** Day length in ms (fixed 86400000 — no DST logic for study tool). */
export const DAY_MS = 24 * 60 * 60 * 1000;

/** Allowed interval steps for the minimal ladder. */
export const INTERVAL_STEPS = Object.freeze([1, 3]);

/**
 * Pure interval step function.
 *
 * @param {number} currentIntervalDays — last scheduled interval (0 if new)
 * @param {boolean} correct — review outcome
 * @returns {number} next interval in days (1 or 3)
 */
export function nextIntervalDays(currentIntervalDays, correct) {
  const cur =
    typeof currentIntervalDays === "number" &&
    isFinite(currentIntervalDays) &&
    currentIntervalDays > 0
      ? currentIntervalDays
      : 0;
  if (!correct) return 1;
  if (cur < 1) return 1;
  if (cur < 3) return 3;
  return 3;
}

/**
 * Compute due_at from now + interval_days.
 *
 * @param {number} intervalDays
 * @param {number} [nowMs]
 * @returns {number}
 */
export function dueAtFromInterval(intervalDays, nowMs) {
  const now = typeof nowMs === "number" ? nowMs : Date.now();
  const d =
    typeof intervalDays === "number" && intervalDays > 0 ? intervalDays : 0;
  return now + d * DAY_MS;
}

/**
 * @param {unknown} raw
 * @returns {{ schema_version: number, cards: Record<string, object> }}
 */
export function normalizeSrsState(raw) {
  const empty = { schema_version: SRS_SCHEMA_VERSION, cards: Object.create(null) };
  if (!raw || typeof raw !== "object") return empty;
  const cardsIn = /** @type {{cards?: unknown}} */ (raw).cards;
  if (!cardsIn || typeof cardsIn !== "object") return empty;
  /** @type {Record<string, object>} */
  const cards = Object.create(null);
  const ids = Object.keys(/** @type {object} */ (cardsIn));
  for (let i = 0; i < ids.length; i++) {
    const id = ids[i];
    const c = /** @type {Record<string, unknown>} */ (
      /** @type {Record<string, unknown>} */ (cardsIn)[id]
    );
    if (!c || typeof c !== "object") continue;
    const itemId = typeof c.item_id === "string" ? c.item_id : id;
    if (!itemId) continue;
    cards[itemId] = {
      item_id: itemId,
      interval_days:
        typeof c.interval_days === "number" && c.interval_days >= 0
          ? c.interval_days
          : 0,
      due_at: typeof c.due_at === "number" ? c.due_at : 0,
      reps: typeof c.reps === "number" && c.reps >= 0 ? c.reps : 0,
      lapses: typeof c.lapses === "number" && c.lapses >= 0 ? c.lapses : 0,
      updated_at: typeof c.updated_at === "number" ? c.updated_at : 0,
    };
  }
  return { schema_version: SRS_SCHEMA_VERSION, cards: cards };
}

/**
 * @param {Storage} [store]
 * @returns {{ schema_version: number, cards: Record<string, object> }}
 */
export function loadSrsState(store) {
  const s = store || (typeof localStorage !== "undefined" ? localStorage : null);
  if (!s) return normalizeSrsState(null);
  try {
    const raw = s.getItem(SRS_STORAGE_KEY);
    if (!raw) return normalizeSrsState(null);
    return normalizeSrsState(JSON.parse(raw));
  } catch (_) {
    return normalizeSrsState(null);
  }
}

/**
 * @param {{ schema_version?: number, cards: Record<string, object> }} state
 * @param {Storage} [store]
 * @returns {boolean}
 */
export function saveSrsState(state, store) {
  const s = store || (typeof localStorage !== "undefined" ? localStorage : null);
  if (!s) return false;
  try {
    const payload = {
      schema_version: SRS_SCHEMA_VERSION,
      cards: state && state.cards ? state.cards : Object.create(null),
    };
    s.setItem(SRS_STORAGE_KEY, JSON.stringify(payload));
    return true;
  } catch (_) {
    return false;
  }
}

/**
 * Schedule or re-schedule a missed item (default interval = 1d from now).
 * Does not advance reps; use reviewCard for outcomes.
 *
 * @param {string} itemId
 * @param {{ nowMs?: number, intervalDays?: number, store?: Storage }} [opts]
 * @returns {object | null} card
 */
export function scheduleMissed(itemId, opts) {
  if (typeof itemId !== "string" || !itemId) return null;
  const o = opts || {};
  const now = typeof o.nowMs === "number" ? o.nowMs : Date.now();
  const interval =
    typeof o.intervalDays === "number" && o.intervalDays > 0
      ? o.intervalDays
      : 1;
  const state = loadSrsState(o.store);
  const prev = state.cards[itemId];
  const card = {
    item_id: itemId,
    interval_days: interval,
    due_at: dueAtFromInterval(interval, now),
    reps: prev && typeof prev.reps === "number" ? prev.reps : 0,
    lapses: prev && typeof prev.lapses === "number" ? prev.lapses : 0,
    updated_at: now,
  };
  state.cards[itemId] = card;
  saveSrsState(state, o.store);
  return card;
}

/**
 * Schedule many missed ids (idempotent re-schedule to 1d when newly missed).
 *
 * @param {string[]} itemIds
 * @param {{ nowMs?: number, store?: Storage }} [opts]
 * @returns {number} count scheduled
 */
export function scheduleMissedMany(itemIds, opts) {
  if (!Array.isArray(itemIds)) return 0;
  const o = opts || {};
  const now = typeof o.nowMs === "number" ? o.nowMs : Date.now();
  const state = loadSrsState(o.store);
  let n = 0;
  for (let i = 0; i < itemIds.length; i++) {
    const id = itemIds[i];
    if (typeof id !== "string" || !id) continue;
    const prev = state.cards[id];
    // New miss: schedule 1d. Already scheduled: leave due_at unless overdue / no card.
    if (!prev) {
      state.cards[id] = {
        item_id: id,
        interval_days: 1,
        due_at: dueAtFromInterval(1, now),
        reps: 0,
        lapses: 0,
        updated_at: now,
      };
      n += 1;
    } else {
      // Re-missed: bump lapses and reset to 1d from now.
      const next = nextIntervalDays(prev.interval_days, false);
      state.cards[id] = {
        item_id: id,
        interval_days: next,
        due_at: dueAtFromInterval(next, now),
        reps: typeof prev.reps === "number" ? prev.reps : 0,
        lapses: (typeof prev.lapses === "number" ? prev.lapses : 0) + 1,
        updated_at: now,
      };
      n += 1;
    }
  }
  saveSrsState(state, o.store);
  return n;
}

/**
 * Apply a review outcome and persist.
 *
 * @param {string} itemId
 * @param {boolean} correct
 * @param {{ nowMs?: number, store?: Storage }} [opts]
 * @returns {object | null}
 */
export function reviewCard(itemId, correct, opts) {
  if (typeof itemId !== "string" || !itemId) return null;
  const o = opts || {};
  const now = typeof o.nowMs === "number" ? o.nowMs : Date.now();
  const state = loadSrsState(o.store);
  const prev = state.cards[itemId] || {
    item_id: itemId,
    interval_days: 0,
    due_at: now,
    reps: 0,
    lapses: 0,
    updated_at: now,
  };
  const nextDays = nextIntervalDays(prev.interval_days, !!correct);
  const card = {
    item_id: itemId,
    interval_days: nextDays,
    due_at: dueAtFromInterval(nextDays, now),
    reps: (typeof prev.reps === "number" ? prev.reps : 0) + (correct ? 1 : 0),
    lapses:
      (typeof prev.lapses === "number" ? prev.lapses : 0) + (correct ? 0 : 1),
    updated_at: now,
  };
  state.cards[itemId] = card;
  saveSrsState(state, o.store);
  return card;
}

/** Drill-10 session shape: at most this many due cards per session. */
export const DRILL10_LIMIT = 10;

/**
 * Pure due-only filter for Drill-10 (charter session shape).
 * Cards with due_at ≤ nowMs, sorted earliest-due first, capped at `limit`.
 *
 * @param {Array<{ due_at?: number, item_id?: string } | null | undefined>} cards
 * @param {number} [nowMs]
 * @param {number} [limit] — default DRILL10_LIMIT (10)
 * @returns {object[]}
 */
export function selectDueOnly(cards, nowMs, limit) {
  const now = typeof nowMs === "number" ? nowMs : Date.now();
  const lim =
    typeof limit === "number" && isFinite(limit) && limit > 0
      ? Math.floor(limit)
      : DRILL10_LIMIT;
  const arr = Array.isArray(cards) ? cards : [];
  const out = [];
  for (let i = 0; i < arr.length; i++) {
    const c = arr[i];
    if (c && typeof c.due_at === "number" && c.due_at <= now) {
      out.push(c);
    }
  }
  out.sort(function (a, b) {
    return a.due_at - b.due_at;
  });
  return out.slice(0, lim);
}

/**
 * Cards due at or before nowMs (all due; no Drill-10 cap).
 *
 * @param {{ nowMs?: number, store?: Storage }} [opts]
 * @returns {object[]}
 */
export function listDue(opts) {
  const o = opts || {};
  const now = typeof o.nowMs === "number" ? o.nowMs : Date.now();
  const state = loadSrsState(o.store);
  const out = [];
  const ids = Object.keys(state.cards);
  for (let i = 0; i < ids.length; i++) {
    const c = state.cards[ids[i]];
    if (c && typeof c.due_at === "number" && c.due_at <= now) {
      out.push(c);
    }
  }
  out.sort(function (a, b) {
    return a.due_at - b.due_at;
  });
  return out;
}

/**
 * Drill-10 queue: due cards from storage, earliest first, max 10.
 *
 * @param {{ nowMs?: number, store?: Storage, limit?: number }} [opts]
 * @returns {object[]}
 */
export function listDueDrill(opts) {
  const o = opts || {};
  const limit =
    typeof o.limit === "number" && isFinite(o.limit) && o.limit > 0
      ? Math.floor(o.limit)
      : DRILL10_LIMIT;
  return selectDueOnly(listDue(o), o.nowMs, limit);
}

/**
 * All SRS cards.
 *
 * @param {{ store?: Storage }} [opts]
 * @returns {object[]}
 */
export function listAllCards(opts) {
  const o = opts || {};
  const state = loadSrsState(o.store);
  const out = [];
  const ids = Object.keys(state.cards);
  for (let i = 0; i < ids.length; i++) {
    out.push(state.cards[ids[i]]);
  }
  out.sort(function (a, b) {
    return a.due_at - b.due_at;
  });
  return out;
}

/**
 * Persist missed item feed for Drill surface.
 *
 * @param {{
 *   source: string,
 *   exam_id?: string,
 *   seed?: number | null,
 *   bank_hash?: string,
 *   item_ids: string[],
 *   saved_at?: number
 * }} payload
 * @param {Storage} [store]
 * @returns {boolean}
 */
export function saveMissed(payload, store) {
  const s = store || (typeof localStorage !== "undefined" ? localStorage : null);
  if (!s || !payload || !Array.isArray(payload.item_ids)) return false;
  try {
    const body = {
      schema_version: MISSED_SCHEMA_VERSION,
      source: payload.source || "unknown",
      exam_id: payload.exam_id || "",
      seed: payload.seed != null ? payload.seed : null,
      bank_hash: payload.bank_hash || "",
      saved_at:
        typeof payload.saved_at === "number" ? payload.saved_at : Date.now(),
      item_ids: payload.item_ids.filter(function (id) {
        return typeof id === "string" && id.length > 0;
      }),
    };
    s.setItem(MISSED_STORAGE_KEY, JSON.stringify(body));
    return true;
  } catch (_) {
    return false;
  }
}

/**
 * @param {Storage} [store]
 * @returns {{
 *   schema_version: number,
 *   source: string,
 *   exam_id: string,
 *   seed: number | null,
 *   bank_hash: string,
 *   saved_at: number,
 *   item_ids: string[]
 * } | null}
 */
export function loadMissed(store) {
  const s = store || (typeof localStorage !== "undefined" ? localStorage : null);
  if (!s) return null;
  try {
    const raw = s.getItem(MISSED_STORAGE_KEY);
    if (!raw) return null;
    const p = JSON.parse(raw);
    if (!p || !Array.isArray(p.item_ids)) return null;
    return {
      schema_version: MISSED_SCHEMA_VERSION,
      source: typeof p.source === "string" ? p.source : "unknown",
      exam_id: typeof p.exam_id === "string" ? p.exam_id : "",
      seed: typeof p.seed === "number" ? p.seed : null,
      bank_hash: typeof p.bank_hash === "string" ? p.bank_hash : "",
      saved_at: typeof p.saved_at === "number" ? p.saved_at : 0,
      item_ids: p.item_ids.filter(function (id) {
        return typeof id === "string" && id.length > 0;
      }),
    };
  } catch (_) {
    return null;
  }
}

/**
 * After a graded attempt: store wrongs + schedule SRS.
 *
 * @param {{
 *   source: string,
 *   exam_id?: string,
 *   seed?: number | null,
 *   bank_hash?: string,
 *   item_results: Array<{ item_id: string, is_correct: boolean }>
 * }} graded
 * @param {{ nowMs?: number, store?: Storage }} [opts]
 * @returns {{ missed_ids: string[], scheduled: number }}
 */
export function recordGradedWrongs(graded, opts) {
  const o = opts || {};
  const results = graded && Array.isArray(graded.item_results)
    ? graded.item_results
    : [];
  const missed = [];
  for (let i = 0; i < results.length; i++) {
    const r = results[i];
    if (r && r.is_correct === false && typeof r.item_id === "string") {
      missed.push(r.item_id);
    }
  }
  saveMissed(
    {
      source: graded.source || "unknown",
      exam_id: graded.exam_id,
      seed: graded.seed,
      bank_hash: graded.bank_hash,
      item_ids: missed,
    },
    o.store
  );
  const scheduled = scheduleMissedMany(missed, o);
  return { missed_ids: missed, scheduled: scheduled };
}

// Browser global for non-module script tags / console.
if (typeof globalThis !== "undefined") {
  globalThis.CdcpSrs = {
    SRS_STORAGE_KEY,
    MISSED_STORAGE_KEY,
    SRS_SCHEMA_VERSION,
    MISSED_SCHEMA_VERSION,
    DAY_MS,
    INTERVAL_STEPS,
    DRILL10_LIMIT,
    nextIntervalDays,
    dueAtFromInterval,
    normalizeSrsState,
    loadSrsState,
    saveSrsState,
    scheduleMissed,
    scheduleMissedMany,
    reviewCard,
    selectDueOnly,
    listDue,
    listDueDrill,
    listAllCards,
    saveMissed,
    loadMissed,
    recordGradedWrongs,
  };
}
