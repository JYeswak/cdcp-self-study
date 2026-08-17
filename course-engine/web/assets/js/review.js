/**
 * Short-interval review (L5-S7 / bd-ca8 · bd-engine-not-gate-ar39.5).
 *
 * NOT spaced repetition. The interval ladder lives in `cdcp_schedule`
 * (`INTERVAL_STEPS = [1, 3]`, 3-day cap) and is decided by WASM. This module
 * renders and persists. Calling it SRS overstates it.
 *
 * Schema (localStorage key `cdcp.srs.v1` — historical name; law is not SRS).
 * `schema_version` is accepted/migrated by WASM (`cdcp_migrate_state_version`).
 * Unversioned (`0` / missing) → 1. Unknown version is an ERROR (not rewritten).
 * {
 *   schema_version: 1,
 *   cards: {
 *     [item_id]: {
 *       item_id: string,
 *       interval_days: number,
 *       due_at: number,
 *       reps: number,
 *       lapses: number,
 *       updated_at: number
 *     }
 *   }
 * }
 *
 * Interval law: `cdcp_schedule::next_interval_days` via schedule_bridge.
 * Drill-10: selectDueOnly / listDueDrill — due_at ≤ now, earliest first, cap 10.
 *
 * Residue sweep (bd-srs-residue-retired-ids-k3vs): pre-bd-7big quiz draws wrote
 * retired ids into these stores. On load, `pruneNonApprovedFromStorage` drops
 * any id whose pack row is not status=approved. Do not filter drill's byId
 * map — that map resolves already-answered ids so a withdrawn card can still
 * be rendered; it must not stay due on the 1d/3d ladder.
 *
 * No LLM. Pedagogy-only — never a cert claim.
 *
 * @module review
 */

import {
  nextIntervalDays as wasmNextIntervalDays,
  dayMs as wasmDayMs,
  firstStepDays,
  migrateStateVersion,
} from "./schedule_bridge.js";

/** Historical localStorage key. Do not treat the name as an algorithm claim. */
export const REVIEW_STORAGE_KEY = "cdcp.srs.v1";
export const MISSED_STORAGE_KEY = "cdcp.drill.missed.v1";
export const REVIEW_SCHEMA_VERSION = 1;
export const MISSED_SCHEMA_VERSION = 1;

/**
 * The one BankItem.status that may stay on the 1d/3d ladder or in the
 * missed feed. Same rule as quiz.js `isApproved`: absent status is
 * WITHHELD. This module cannot import quiz.js (quiz imports us).
 */
export const APPROVED_STATUS = "approved";

/**
 * Next interval in days. WASM decides (`cdcp_next_interval_days`).
 *
 * @param {number} currentIntervalDays
 * @param {boolean} correct
 * @returns {number}
 */
export function nextIntervalDays(currentIntervalDays, correct) {
  return wasmNextIntervalDays(currentIntervalDays, correct);
}

/**
 * Day length in ms from the schedule crate (no DST).
 * @returns {number}
 */
export function dayMs() {
  return wasmDayMs();
}

/**
 * Compute due_at from now + interval_days. Day length comes from WASM.
 *
 * @param {number} intervalDays
 * @param {number} [nowMs]
 * @returns {number}
 */
export function dueAtFromInterval(intervalDays, nowMs) {
  const now = typeof nowMs === "number" ? nowMs : Date.now();
  const d =
    typeof intervalDays === "number" && intervalDays > 0 ? intervalDays : 0;
  return now + d * dayMs();
}

/**
 * True when `err` is the fail-closed unknown-version path.
 * `loadReviewState` must rethrow this — swallowing it would wipe the store.
 *
 * @param {unknown} err
 * @returns {boolean}
 */
export function isUnknownScheduleVersionError(err) {
  return !!(
    err &&
    typeof /** @type {{message?: unknown}} */ (err).message === "string" &&
    String(/** @type {{message: string}} */ (err).message).indexOf(
      "unknown schedule state version"
    ) !== -1
  );
}

/**
 * @param {unknown} raw
 * @returns {{ schema_version: number, cards: Record<string, object> }}
 */
export function normalizeReviewState(raw) {
  const empty = {
    schema_version: REVIEW_SCHEMA_VERSION,
    cards: Object.create(null),
  };
  if (!raw || typeof raw !== "object") return empty;
  const incoming = /** @type {{schema_version?: unknown}} */ (raw).schema_version;
  const from =
    typeof incoming === "number" && isFinite(incoming)
      ? Math.floor(incoming)
      : 0;
  // WASM decides. Unknown version throws — do not stamp v1 and continue.
  const version = migrateStateVersion(from);
  const cardsIn = /** @type {{cards?: unknown}} */ (raw).cards;
  if (!cardsIn || typeof cardsIn !== "object") {
    return { schema_version: version, cards: Object.create(null) };
  }
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
  return { schema_version: version, cards: cards };
}

/**
 * @param {Storage} [store]
 * @returns {{ schema_version: number, cards: Record<string, object> }}
 */
export function loadReviewState(store) {
  const s = store || (typeof localStorage !== "undefined" ? localStorage : null);
  if (!s) return normalizeReviewState(null);
  try {
    const raw = s.getItem(REVIEW_STORAGE_KEY);
    if (!raw) return normalizeReviewState(null);
    return normalizeReviewState(JSON.parse(raw));
  } catch (err) {
    if (isUnknownScheduleVersionError(err)) throw err;
    return normalizeReviewState(null);
  }
}

/**
 * @param {{ schema_version?: number, cards: Record<string, object> }} state
 * @param {Storage} [store]
 * @returns {boolean}
 */
export function saveReviewState(state, store) {
  const s = store || (typeof localStorage !== "undefined" ? localStorage : null);
  if (!s) return false;
  try {
    const payload = {
      schema_version: REVIEW_SCHEMA_VERSION,
      cards: state && state.cards ? state.cards : Object.create(null),
    };
    s.setItem(REVIEW_STORAGE_KEY, JSON.stringify(payload));
    return true;
  } catch (_) {
    return false;
  }
}

/**
 * Schedule or re-schedule a missed item (default = first ladder step).
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
      : firstStepDays();
  const state = loadReviewState(o.store);
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
  saveReviewState(state, o.store);
  return card;
}

/**
 * Schedule many missed ids (new miss → first step; re-miss → wrong-step).
 *
 * @param {string[]} itemIds
 * @param {{ nowMs?: number, store?: Storage }} [opts]
 * @returns {number} count scheduled
 */
export function scheduleMissedMany(itemIds, opts) {
  if (!Array.isArray(itemIds)) return 0;
  const o = opts || {};
  const now = typeof o.nowMs === "number" ? o.nowMs : Date.now();
  const state = loadReviewState(o.store);
  const first = firstStepDays();
  let n = 0;
  for (let i = 0; i < itemIds.length; i++) {
    const id = itemIds[i];
    if (typeof id !== "string" || !id) continue;
    const prev = state.cards[id];
    if (!prev) {
      state.cards[id] = {
        item_id: id,
        interval_days: first,
        due_at: dueAtFromInterval(first, now),
        reps: 0,
        lapses: 0,
        updated_at: now,
      };
      n += 1;
    } else {
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
  saveReviewState(state, o.store);
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
  const state = loadReviewState(o.store);
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
  saveReviewState(state, o.store);
  return card;
}

/** Drill-10 session shape: at most this many due cards per session. */
export const DRILL10_LIMIT = 10;

/**
 * Pure due-only filter for Drill-10 (charter session shape).
 *
 * @param {Array<{ due_at?: number, item_id?: string } | null | undefined>} cards
 * @param {number} [nowMs]
 * @param {number} [limit]
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
 * @param {{ nowMs?: number, store?: Storage }} [opts]
 * @returns {object[]}
 */
export function listDue(opts) {
  const o = opts || {};
  const now = typeof o.nowMs === "number" ? o.nowMs : Date.now();
  const state = loadReviewState(o.store);
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
 * All review cards.
 *
 * @param {{ store?: Storage }} [opts]
 * @returns {object[]}
 */
export function listAllCards(opts) {
  const o = opts || {};
  const state = loadReviewState(o.store);
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
 * True when a pack row may remain in missed / short-interval storage.
 * Absent status is withheld — never guessed in the learner's favour.
 *
 * @param {{status?: string, id?: string}|null|undefined} it
 * @returns {boolean}
 */
export function isApprovedRow(it) {
  return !!it && it.status === APPROVED_STATUS;
}

/**
 * @param {unknown} pack
 * @returns {Array}
 */
export function packRows(pack) {
  if (Array.isArray(pack)) return pack;
  if (pack && typeof pack === "object") {
    const items = /** @type {{items?: unknown}} */ (pack).items;
    if (Array.isArray(items)) return items;
  }
  return [];
}

/**
 * @param {string | null} [skipped]
 * @returns {{
 *   dropped: string[],
 *   dropped_missed: string[],
 *   dropped_srs: string[],
 *   emptied: boolean,
 *   emptied_missed: boolean,
 *   emptied_srs: boolean,
 *   persisted: boolean,
 *   skipped: string | null
 * }}
 */
function emptyPruneReport(skipped) {
  return {
    dropped: [],
    dropped_missed: [],
    dropped_srs: [],
    emptied: false,
    emptied_missed: false,
    emptied_srs: false,
    persisted: false,
    skipped: skipped || null,
  };
}

/**
 * Learner-facing copy. Empty string iff nothing was pruned — a no-op
 * must not read like a withdrawal (bd-srs-residue-retired-ids-k3vs).
 *
 * @param {{ dropped?: string[], emptied?: boolean } | null | undefined} report
 * @returns {string}
 */
export function formatPruneNotice(report) {
  if (!report || !Array.isArray(report.dropped) || report.dropped.length === 0) {
    return "";
  }
  const n = report.dropped.length;
  const noun = n === 1 ? "card" : "cards";
  const head =
    n +
    " " +
    noun +
    " removed: these questions were withdrawn from the bank (" +
    report.dropped.join(", ") +
    ").";
  if (report.emptied) {
    return head + " The review queue is now empty.";
  }
  return head;
}

/**
 * Drop ids that are not status=approved from missed + SRS storage.
 *
 * Driven by the pack's `status` field, never by a hardcoded id list.
 * An id missing from the pack is not in the approved pool and is dropped.
 *
 * Fail-closed on a pack we cannot trust:
 *   - empty pack / failed load → do not touch storage
 *   - pack rows carry no `status` (e.g. the learner mock40 pack) → do not
 *     treat every id as withdrawn and wipe the queue
 *
 * Persist only when something was removed, so a second load is a no-op
 * (not re-pruned every load).
 *
 * @param {unknown} pack bank_items rows (array or {items})
 * @param {Storage} [store]
 * @returns {{
 *   dropped: string[],
 *   dropped_missed: string[],
 *   dropped_srs: string[],
 *   emptied: boolean,
 *   emptied_missed: boolean,
 *   emptied_srs: boolean,
 *   persisted: boolean,
 *   skipped: string | null
 * }}
 */
export function pruneNonApprovedFromStorage(pack, store) {
  const rows = packRows(pack);
  if (!rows.length) return emptyPruneReport("empty-pack");

  /** @type {Record<string, boolean>} */
  const approved = Object.create(null);
  let withStatus = 0;
  for (let i = 0; i < rows.length; i++) {
    const it = rows[i];
    if (!it || typeof it.id !== "string" || !it.id) continue;
    if (typeof it.status === "string") withStatus += 1;
    if (isApprovedRow(it)) approved[it.id] = true;
  }
  if (withStatus === 0) return emptyPruneReport("pack-has-no-status");

  const s = store || (typeof localStorage !== "undefined" ? localStorage : null);
  const missed = loadMissed(s);
  const state = loadReviewState(s);

  const droppedMissed = [];
  let nextMissedIds = null;
  if (missed && Array.isArray(missed.item_ids) && missed.item_ids.length) {
    nextMissedIds = [];
    for (let i = 0; i < missed.item_ids.length; i++) {
      const id = missed.item_ids[i];
      if (approved[id]) nextMissedIds.push(id);
      else droppedMissed.push(id);
    }
  }

  const droppedSrs = [];
  const nextCards = Object.create(null);
  const cardIds = Object.keys(state.cards);
  for (let i = 0; i < cardIds.length; i++) {
    const id = cardIds[i];
    const card = state.cards[id];
    const itemId =
      card && typeof card.item_id === "string" && card.item_id
        ? card.item_id
        : id;
    if (approved[itemId]) nextCards[itemId] = card;
    else droppedSrs.push(itemId);
  }

  /** @type {Record<string, boolean>} */
  const seen = Object.create(null);
  for (let i = 0; i < droppedMissed.length; i++) seen[droppedMissed[i]] = true;
  for (let i = 0; i < droppedSrs.length; i++) seen[droppedSrs[i]] = true;
  const dropped = Object.keys(seen).sort();

  if (dropped.length === 0) return emptyPruneReport(null);

  const hadMissed = !!(missed && missed.item_ids.length);
  const hadSrs = cardIds.length > 0;
  const remainMissed = nextMissedIds
    ? nextMissedIds.length
    : missed
      ? missed.item_ids.length
      : 0;
  const remainSrs = Object.keys(nextCards).length;
  const emptiedMissed = hadMissed && remainMissed === 0;
  const emptiedSrs = hadSrs && remainSrs === 0;
  const emptied =
    (hadMissed || hadSrs) && remainMissed === 0 && remainSrs === 0;

  let persisted = false;
  if (missed && droppedMissed.length > 0) {
    const savedMissed = saveMissed(
      {
        source: missed.source,
        exam_id: missed.exam_id,
        seed: missed.seed,
        bank_hash: missed.bank_hash,
        saved_at: missed.saved_at,
        item_ids: nextMissedIds || [],
      },
      s
    );
    persisted = persisted || savedMissed;
  }
  if (droppedSrs.length > 0) {
    const savedSrs = saveReviewState(
      { schema_version: REVIEW_SCHEMA_VERSION, cards: nextCards },
      s
    );
    persisted = persisted || savedSrs;
  }

  return {
    dropped: dropped,
    dropped_missed: droppedMissed,
    dropped_srs: droppedSrs,
    emptied: emptied,
    emptied_missed: emptiedMissed,
    emptied_srs: emptiedSrs,
    persisted: persisted,
    skipped: null,
  };
}

/**
 * After a graded attempt: store wrongs + schedule short-interval review.
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

if (typeof globalThis !== "undefined") {
  globalThis.CdcpReview = {
    REVIEW_STORAGE_KEY,
    MISSED_STORAGE_KEY,
    REVIEW_SCHEMA_VERSION,
    MISSED_SCHEMA_VERSION,
    APPROVED_STATUS,
    dayMs,
    DRILL10_LIMIT,
    nextIntervalDays,
    dueAtFromInterval,
    normalizeReviewState,
    loadReviewState,
    saveReviewState,
    scheduleMissed,
    scheduleMissedMany,
    reviewCard,
    selectDueOnly,
    listDue,
    listDueDrill,
    listAllCards,
    saveMissed,
    loadMissed,
    isApprovedRow,
    packRows,
    formatPruneNotice,
    pruneNonApprovedFromStorage,
    recordGradedWrongs,
    isUnknownScheduleVersionError,
  };
}
