/**
 * Hub mastery dashboard + next-module recommend (L6-S4 / bd-qyi, bd-61ey).
 *
 * Surfaces practiced / mastered badges for EVERY module the Learn registry
 * declares, plus a recommend card that prefers: last-attempt weak module →
 * first unpracticed → first unmastered → "all practiced" message. Links always
 * resolve to learn/&#42;.html or quiz.html.
 *
 * Study signal only — never a CDCP / EPI credential claim.
 *
 * ## Why the catalog is DERIVED, and not a literal (bd-61ey / bd-lt7 class)
 *
 * Until 2026-08-14 `MODULE_CATALOG` was a hand-maintained array of fourteen
 * frozen objects, and `scripts/smoke_hub_mastery.mjs` asserted its length was
 * fourteen. Module 15 (`15-ops-adjacent`) is assessed and, since C5, taught —
 * it has a Learn page and an approved bank — yet a learner could not see any
 * module-15 progress, because this file did not know the module existed. The
 * defect was never "someone typed 14"; it was that the catalog was an
 * OBSERVATION of the curriculum rather than a PROJECTION of it. Bumping the
 * literal to fifteen would re-encode the same defect one module later.
 *
 * The registry is `web/data/modules_index.json`, imported statically so the
 * catalog exists synchronously with no fetch and no network:
 *
 *   - `scripts/build_learn.py` generates it from `knowledge/domains.toml`, and
 *     `cdcp smoke-learn` gates the two against each other, so it cannot
 *     drift from the authoring registry.
 *   - It is what the Learn surface itself is built from: every
 *     `web/learn/{id}.html`, the `#module-list` on `web/learn.html`, and the
 *     `#modules-index` blob that page embeds all come out of the same rows.
 *   - It already ships inside `web/`, unlike `knowledge/domains.toml`, and it
 *     already carries the fields this catalog needs (`order`, `id`,
 *     `epi_heading`, `href`) — `domains.toml` carries authoring paths instead.
 *   - `scripts/build_units.py` and `crates/cdcp_assemble/tests/learn_surface_coverage.rs`
 *     were rebased onto this same file, so there is one registry for the Learn
 *     surface rather than one per consumer.
 *
 * A module declared in that registry is ALWAYS in this catalog. There is no
 * filter to be wrong about, and `scripts/smoke_hub_mastery.mjs` asserts the
 * two agree by id — naming any module the catalog dropped.
 *
 * localStorage:
 *   cdcp.mastery.v1     — owned by mastery.js
 *   cdcp.last_weak.v1   — weak module list from last graded mock (results.js)
 *
 * @module hub_mastery
 */

import MODULES_INDEX from "../../data/modules_index.json" with { type: "json" };

import { loadWasm } from "./grade_bridge.js";
import {
  isPracticed,
  isMastered,
  getState,
  loadState,
} from "./mastery.js";

/** Weak modules from last graded mock attempt. */
export const WEAK_STORAGE_KEY = "cdcp.last_weak.v1";
export const WEAK_SCHEMA_VERSION = 1;

/**
 * Projection of one Learn-registry row onto a mastery-dashboard entry.
 *
 * @typedef {{
 *   order: number,
 *   id: string,
 *   title: string,
 *   learnHref: string,
 *   quizHref: string
 * }} CatalogEntry
 */

/**
 * Derive the module catalog from a Learn registry document.
 *
 * EVERY declared module becomes an entry — there is deliberately no filter, no
 * skip and no upper bound here, because a filter is exactly what hid module 15
 * from the learner. If a module must not be surfaced, it must not be declared.
 *
 * Anti-vacuous: a registry with no `modules` array, or with zero modules, is an
 * ERROR and throws. A dashboard that silently renders nothing looks the same as
 * one whose learner has simply not started, which is how a missing catalog goes
 * unnoticed for a whole wave.
 *
 * @param {unknown} indexDoc parsed `web/data/modules_index.json`
 * @returns {readonly CatalogEntry[]} frozen, ordered by `order`
 * @throws {Error} on a malformed, empty, or self-contradictory registry
 */
export function buildCatalog(indexDoc) {
  if (!indexDoc || typeof indexDoc !== "object") {
    throw new Error(
      "hub_mastery: Learn registry is missing or not an object — refusing to " +
        "build a module catalog from nothing"
    );
  }
  const rows = /** @type {Record<string, unknown>} */ (indexDoc).modules;
  if (!Array.isArray(rows)) {
    throw new Error(
      "hub_mastery: Learn registry has no `modules` array — refusing to build " +
        "a module catalog from nothing"
    );
  }
  if (rows.length === 0) {
    throw new Error(
      "hub_mastery: Learn registry declares zero modules — an empty catalog is " +
        "an ERROR, not an empty dashboard"
    );
  }

  /** @type {CatalogEntry[]} */
  const out = [];
  const seenOrder = new Set();
  const seenId = new Set();
  for (let i = 0; i < rows.length; i++) {
    const r = /** @type {Record<string, unknown>} */ (rows[i] || {});
    const id = typeof r.id === "string" ? r.id : "";
    const order = Number(r.order);
    const href = typeof r.href === "string" ? r.href : "";
    const title = typeof r.epi_heading === "string" ? r.epi_heading : "";
    if (!id) {
      throw new Error("hub_mastery: Learn registry row " + i + " has no `id`");
    }
    if (!isFinite(order) || order < 1 || Math.floor(order) !== order) {
      throw new Error(
        "hub_mastery: Learn registry row " + id + " has no usable `order`"
      );
    }
    if (!href) {
      throw new Error(
        "hub_mastery: Learn registry row " + id + " has no `href`"
      );
    }
    if (seenOrder.has(order)) {
      throw new Error(
        "hub_mastery: Learn registry declares order " + order + " twice"
      );
    }
    if (seenId.has(id)) {
      throw new Error("hub_mastery: Learn registry declares id " + id + " twice");
    }
    seenOrder.add(order);
    seenId.add(id);
    out.push({
      order: order,
      id: id,
      title: title || id,
      learnHref: href,
      quizHref: "quiz.html?module=" + order,
    });
  }
  out.sort(function (a, b) {
    return a.order - b.order;
  });
  if (out.length !== rows.length) {
    throw new Error(
      "hub_mastery: derived " +
        out.length +
        " entries from " +
        rows.length +
        " declared modules — the catalog must project every declared module"
    );
  }
  return Object.freeze(out);
}

/**
 * The live catalog: one entry per module declared by the Learn registry.
 *
 * DERIVED — see the module header. Never edit this by hand; edit
 * `knowledge/domains.toml` and re-run `scripts/build_learn.py`.
 *
 * @type {readonly CatalogEntry[]}
 */
export const MODULE_CATALOG = buildCatalog(MODULES_INDEX);

/** @type {Map<number, typeof MODULE_CATALOG[0]>} */
const BY_ORDER = (function () {
  const m = new Map();
  for (let i = 0; i < MODULE_CATALOG.length; i++) {
    m.set(MODULE_CATALOG[i].order, MODULE_CATALOG[i]);
  }
  return m;
})();

/**
 * @param {number|string} order
 * @returns {typeof MODULE_CATALOG[0] | null}
 */
export function catalogEntry(order) {
  const n = Number(order);
  if (!isFinite(n)) return null;
  return BY_ORDER.get(n) || null;
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
 * Normalize weak_modules payload.
 * @param {unknown} raw
 * @returns {{ schema_version: number, weak_modules: number[], at_ms: number, source: string }}
 */
export function normalizeLastWeak(raw) {
  const empty = {
    schema_version: WEAK_SCHEMA_VERSION,
    weak_modules: [],
    at_ms: 0,
    source: "",
  };
  if (!raw || typeof raw !== "object") return empty;
  const o = /** @type {Record<string, unknown>} */ (raw);
  const arr = Array.isArray(o.weak_modules)
    ? o.weak_modules
    : Array.isArray(o.weakModules)
      ? o.weakModules
      : [];
  /** @type {number[]} */
  const weak = [];
  const seen = Object.create(null);
  for (let i = 0; i < arr.length; i++) {
    const n = Number(arr[i]);
    if (!isFinite(n)) continue;
    if (seen[n]) continue;
    // Catalog membership IS the bound. A numeric ceiling here used to hold
    // module 15 out of the weak list even once the catalog knew about it, so
    // the only test is "does the registry declare a module at this order".
    if (!BY_ORDER.has(n)) continue;
    seen[n] = true;
    weak.push(n);
  }
  weak.sort(function (a, b) {
    return a - b;
  });
  const atMs =
    typeof o.at_ms === "number" && isFinite(o.at_ms)
      ? o.at_ms
      : typeof o.atMs === "number" && isFinite(o.atMs)
        ? o.atMs
        : 0;
  const source = typeof o.source === "string" ? o.source : "";
  return {
    schema_version: WEAK_SCHEMA_VERSION,
    weak_modules: weak,
    at_ms: atMs,
    source: source,
  };
}

/**
 * Load last-attempt weak modules.
 * @param {{ store?: Storage }} [opts]
 */
export function loadLastWeak(opts) {
  const o = opts || {};
  const s = resolveStore(o.store);
  if (!s) return normalizeLastWeak(null);
  try {
    const raw = s.getItem(WEAK_STORAGE_KEY);
    if (!raw) return normalizeLastWeak(null);
    return normalizeLastWeak(JSON.parse(raw));
  } catch (_) {
    return normalizeLastWeak(null);
  }
}

/**
 * Persist weak modules from a graded mock (called by results.js).
 *
 * @param {number[]|unknown} weakModules
 * @param {{ store?: Storage, atMs?: number, source?: string, nowMs?: number }} [opts]
 * @returns {{ schema_version: number, weak_modules: number[], at_ms: number, source: string }}
 */
export function saveLastWeak(weakModules, opts) {
  const o = opts || {};
  const atMs =
    typeof o.atMs === "number" && isFinite(o.atMs)
      ? o.atMs
      : typeof o.nowMs === "number" && isFinite(o.nowMs)
        ? o.nowMs
        : Date.now();
  const payload = normalizeLastWeak({
    weak_modules: weakModules,
    at_ms: atMs,
    source: o.source || "mock",
  });
  const s = resolveStore(o.store);
  if (s) {
    try {
      s.setItem(WEAK_STORAGE_KEY, JSON.stringify(payload));
    } catch (_) {
      /* quota / private mode */
    }
  }
  return payload;
}

/**
 * Recommend next study action.
 *
 * Priority:
 *  1. First weak module from last mock attempt (if still in catalog)
 *  2. First unpracticed module, in registry order
 *  3. First unmastered module
 *  4. All practiced (and mastered) message — no required link
 *
 * @param {{ store?: Storage, state?: object, weak?: number[]|object }} [opts]
 * @returns {{
 *   kind: "weak"|"unpracticed"|"unmastered"|"all_practiced",
 *   module: number|null,
 *   href: string|null,
 *   title: string,
 *   reason: string,
 *   label: string
 * }}
 */
export function recommendNext(opts) {
  const o = opts || {};
  const state = o.state || loadState(o.store);
  const masteryOpts = { state: state, store: o.store };

  let weakList = [];
  if (Array.isArray(o.weak)) {
    weakList = normalizeLastWeak({ weak_modules: o.weak }).weak_modules;
  } else if (o.weak && typeof o.weak === "object") {
    weakList = normalizeLastWeak(o.weak).weak_modules;
  } else {
    weakList = loadLastWeak({ store: o.store }).weak_modules;
  }

  // 1) First weak from last attempt that still maps to a learn page.
  for (let i = 0; i < weakList.length; i++) {
    const entry = catalogEntry(weakList[i]);
    if (!entry) continue;
    return {
      kind: "weak",
      module: entry.order,
      href: entry.learnHref,
      title: entry.title,
      reason: "Flagged weak on your last mock (rate below 3/5).",
      label: "Review M" + String(entry.order).padStart(2, "0"),
    };
  }

  // 2) First unpracticed.
  for (let i = 0; i < MODULE_CATALOG.length; i++) {
    const entry = MODULE_CATALOG[i];
    if (!isPracticed(entry.order, masteryOpts)) {
      return {
        kind: "unpracticed",
        module: entry.order,
        href: entry.quizHref,
        title: entry.title,
        reason: "Not yet practiced (80%+ on a module quiz).",
        label: "Quiz M" + String(entry.order).padStart(2, "0"),
      };
    }
  }

  // 3) First unmastered (practiced but not mastered).
  for (let i = 0; i < MODULE_CATALOG.length; i++) {
    const entry = MODULE_CATALOG[i];
    if (!isMastered(entry.order, masteryOpts)) {
      return {
        kind: "unmastered",
        module: entry.order,
        href: entry.quizHref,
        title: entry.title,
        reason: "Practiced but not yet mastered (90% x2, 24h apart).",
        label: "Retake M" + String(entry.order).padStart(2, "0"),
      };
    }
  }

  // 4) All practiced (and mastered under our laws). The module range is read
  // off the catalog, so this copy can never advertise a curriculum smaller
  // than the one the learner was actually assessed on.
  const first = MODULE_CATALOG[0];
  const last = MODULE_CATALOG[MODULE_CATALOG.length - 1];
  return {
    kind: "all_practiced",
    module: null,
    href: null,
    title: "All modules practiced",
    reason:
      "Every curriculum module (" +
      first.order +
      "-" +
      last.order +
      ") is practiced and mastered in this browser. Keep drilling or take " +
      "another mock — study signal only, not a credential.",
    label: "All practiced",
  };
}

/**
 * Badge state for one module order.
 * @param {number|string} order
 * @param {{ store?: Storage, state?: object }} [opts]
 * @returns {{ practiced: boolean, mastered: boolean }}
 */
export function moduleBadgeState(order, opts) {
  const o = opts || {};
  const state = o.state || loadState(o.store);
  const mo = { state: state, store: o.store };
  return {
    practiced: isPracticed(order, mo),
    mastered: isMastered(order, mo),
  };
}

function escapeHtml(s) {
  return String(s)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

/**
 * HTML for practiced/mastered pill badges (shared hub + learn).
 * @param {{ practiced: boolean, mastered: boolean }} state
 * @returns {string}
 */
export function badgeHtml(state) {
  if (!state) return "";
  const parts = [];
  if (state.mastered) {
    parts.push(
      '<span class="mastery-pill mastery-pill--mastered" title="Mastered: two ≥90% attempts ≥24h apart (study signal)">Mastered</span>'
    );
  } else if (state.practiced) {
    parts.push(
      '<span class="mastery-pill mastery-pill--practiced" title="Practiced: best module quiz ≥80% (study signal)">Practiced</span>'
    );
  }
  return parts.join("");
}

/**
 * Paint recommend card into #mastery-recommend (or provided el).
 * @param {HTMLElement | null} [el]
 * @param {{ store?: Storage, state?: object }} [opts]
 */
export function paintRecommend(el, opts) {
  const target =
    el ||
    (typeof document !== "undefined"
      ? document.getElementById("mastery-recommend")
      : null);
  if (!target) return null;

  const rec = recommendNext(opts);
  target.hidden = false;
  target.className =
    "recommend-card" +
    (rec.kind === "all_practiced" ? " recommend-card--done" : "");

  if (rec.href) {
    target.innerHTML =
      '<p class="recommend-card__label mono">Next up</p>' +
      '<a class="recommend-card__link" href="' +
      escapeHtml(rec.href) +
      '">' +
      '<span class="recommend-card__cta">' +
      escapeHtml(rec.label) +
      "</span>" +
      '<span class="recommend-card__title">' +
      escapeHtml(rec.title) +
      "</span>" +
      "</a>" +
      '<p class="recommend-card__reason">' +
      escapeHtml(rec.reason) +
      "</p>" +
      '<p class="recommend-card__honesty">Study path only — not EPI/EXIN certification.</p>';
  } else {
    target.innerHTML =
      '<p class="recommend-card__label mono">Next up</p>' +
      '<p class="recommend-card__title">' +
      escapeHtml(rec.title) +
      "</p>" +
      '<p class="recommend-card__reason">' +
      escapeHtml(rec.reason) +
      "</p>" +
      '<p class="recommend-card__honesty">Study path only — not EPI/EXIN certification.</p>';
  }
  return rec;
}

/**
 * Paint hub mastery grid (#mastery-grid) + recommend.
 * @param {{ store?: Storage, root?: Document|HTMLElement }} [opts]
 */
export function paintHub(opts) {
  const o = opts || {};
  const root = o.root || (typeof document !== "undefined" ? document : null);
  if (!root) return;

  const state = o.state || getState({ store: o.store });
  const masteryOpts = { state: state, store: o.store };

  paintRecommend(
    root.getElementById
      ? root.getElementById("mastery-recommend")
      : null,
    masteryOpts
  );

  const grid = root.getElementById
    ? root.getElementById("mastery-grid")
    : null;
  if (!grid) return;

  const parts = [];
  for (let i = 0; i < MODULE_CATALOG.length; i++) {
    const entry = MODULE_CATALOG[i];
    const badges = moduleBadgeState(entry.order, masteryOpts);
    const statusClass = badges.mastered
      ? " mastery-row--mastered"
      : badges.practiced
        ? " mastery-row--practiced"
        : "";
    const orderLabel = String(entry.order).padStart(2, "0");
    parts.push(
      '<li class="mastery-row' +
        statusClass +
        '" data-mastery-module="' +
        entry.order +
        '" data-module-id="' +
        escapeHtml(entry.id) +
        '">' +
        '<a class="mastery-row__link" href="' +
        escapeHtml(entry.learnHref) +
        '">' +
        '<span class="mastery-row__order mono">' +
        orderLabel +
        "</span>" +
        '<span class="mastery-row__title">' +
        escapeHtml(entry.title) +
        "</span>" +
        '<span class="mastery-row__badges">' +
        badgeHtml(badges) +
        (badges.practiced || badges.mastered
          ? ""
          : '<span class="mastery-pill mastery-pill--open" title="Not yet practiced">Open</span>') +
        "</span>" +
        "</a>" +
        '<a class="mastery-row__quiz mono" href="' +
        escapeHtml(entry.quizHref) +
        '" title="Module quiz">Quiz</a>' +
        "</li>"
    );
  }
  grid.innerHTML = parts.join("");
}

/**
 * Paint mastery badges onto learn.html module list.
 * Looks for [data-mastery-for="<order>"] or .module-list__item[data-module-id].
 * @param {{ store?: Storage, root?: Document|HTMLElement }} [opts]
 */
export function paintLearnBadges(opts) {
  const o = opts || {};
  const root = o.root || (typeof document !== "undefined" ? document : null);
  if (!root) return;

  const state = o.state || getState({ store: o.store });
  const masteryOpts = { state: state, store: o.store };

  // Explicit badge slots (preferred).
  const slots =
    typeof root.querySelectorAll === "function"
      ? root.querySelectorAll("[data-mastery-for]")
      : [];
  for (let i = 0; i < slots.length; i++) {
    const el = slots[i];
    const order = Number(el.getAttribute("data-mastery-for"));
    if (!isFinite(order)) continue;
    const badges = moduleBadgeState(order, masteryOpts);
    const html = badgeHtml(badges);
    if (html) {
      el.innerHTML = html;
      el.hidden = false;
    } else {
      el.innerHTML = "";
      el.hidden = true;
    }
  }

  // Fallback: inject into module-list items by data-module-id.
  const items =
    typeof root.querySelectorAll === "function"
      ? root.querySelectorAll(".module-list__item[data-module-id]")
      : [];
  for (let j = 0; j < items.length; j++) {
    const item = items[j];
    const mid = item.getAttribute("data-module-id");
    if (!mid) continue;
    let entry = null;
    for (let k = 0; k < MODULE_CATALOG.length; k++) {
      if (MODULE_CATALOG[k].id === mid) {
        entry = MODULE_CATALOG[k];
        break;
      }
    }
    if (!entry) continue;

    const badges = moduleBadgeState(entry.order, masteryOpts);
    item.classList.toggle("module-list__item--practiced", badges.practiced);
    item.classList.toggle("module-list__item--mastered", badges.mastered);

    // Prefer dedicated slot inside the item.
    let slot = item.querySelector("[data-mastery-for]");
    if (!slot) {
      // Create a badge host after progress badge / at end of link.
      const link = item.querySelector(".module-list__link") || item;
      slot = item.querySelector(".mastery-badge-host");
      if (!slot) {
        slot = (root.ownerDocument || document).createElement("span");
        slot.className = "mastery-badge-host";
        slot.setAttribute("data-mastery-for", String(entry.order));
        link.appendChild(slot);
      }
    }
    const html = badgeHtml(badges);
    if (html) {
      slot.innerHTML = html;
      slot.hidden = false;
    } else {
      slot.innerHTML = "";
      slot.hidden = true;
    }
  }
}

/**
 * Boot: paint hub and/or learn surfaces present on the page.
 * WASM must load first — mastery bars live in cdcp_schedule.
 */
export async function boot() {
  if (typeof document === "undefined") return;
  try {
    await loadWasm();
  } catch (e) {
    console.warn(
      "hub_mastery: WASM required for mastery law (cdcp_schedule):",
      e
    );
    return;
  }
  const hasHub =
    document.getElementById("mastery-grid") ||
    document.getElementById("mastery-recommend") ||
    document.getElementById("mastery-dashboard");
  if (hasHub) paintHub();
  if (document.getElementById("module-list")) paintLearnBadges();
}

// Browser global for console / non-module consumers.
if (typeof globalThis !== "undefined") {
  globalThis.CdcpHubMastery = {
    WEAK_STORAGE_KEY,
    WEAK_SCHEMA_VERSION,
    MODULE_CATALOG,
    buildCatalog,
    catalogEntry,
    normalizeLastWeak,
    loadLastWeak,
    saveLastWeak,
    recommendNext,
    moduleBadgeState,
    badgeHtml,
    paintRecommend,
    paintHub,
    paintLearnBadges,
    boot,
  };
}

// Auto-paint when loaded as a module on a real page.
if (typeof document !== "undefined") {
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function () {
      boot().catch(function (e) {
        console.warn("hub_mastery boot failed:", e);
      });
    });
  } else {
    boot().catch(function (e) {
      console.warn("hub_mastery boot failed:", e);
    });
  }
}
