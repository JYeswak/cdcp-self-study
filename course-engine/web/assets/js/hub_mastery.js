/**
 * Hub mastery dashboard + next-module recommend (L6-S4 / bd-qyi).
 *
 * Surfaces practiced / mastered badges for modules 1–14 and a recommend card
 * that prefers: last-attempt weak module → first unpracticed → first unmastered
 * → "all practiced" message. Links always resolve to learn/*.html or quiz.html.
 *
 * Study signal only — never a CDCP / EPI credential claim.
 *
 * localStorage:
 *   cdcp.mastery.v1     — owned by mastery.js
 *   cdcp.last_weak.v1   — weak module list from last graded mock (results.js)
 *
 * @module hub_mastery
 */

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
 * Bank module 1–14 catalog (matches modules_index.json order → id / href).
 * Module 15 is empty-ok and intentionally omitted from the dashboard.
 */
export const MODULE_CATALOG = Object.freeze([
  {
    order: 1,
    id: "01-mission-critical",
    title: "The Mission Critical Site",
    learnHref: "learn/01-mission-critical.html",
    quizHref: "quiz.html?module=1",
  },
  {
    order: 2,
    id: "02-standards",
    title: "Data Centre Standards",
    learnHref: "learn/02-standards.html",
    quizHref: "quiz.html?module=2",
  },
  {
    order: 3,
    id: "03-site-building",
    title: "Data Centre Location, Building and Construction",
    learnHref: "learn/03-site-building.html",
    quizHref: "quiz.html?module=3",
  },
  {
    order: 4,
    id: "04-floor-ceiling",
    title: "Raised Access Flooring and Suspended Ceiling",
    learnHref: "learn/04-floor-ceiling.html",
    quizHref: "quiz.html?module=4",
  },
  {
    order: 5,
    id: "05-lighting",
    title: "Light",
    learnHref: "learn/05-lighting.html",
    quizHref: "quiz.html?module=5",
  },
  {
    order: 6,
    id: "06-power",
    title: "Power Infrastructure",
    learnHref: "learn/06-power.html",
    quizHref: "quiz.html?module=6",
  },
  {
    order: 7,
    id: "07-emf",
    title: "Electro Magnetic Fields (EMF)",
    learnHref: "learn/07-emf.html",
    quizHref: "quiz.html?module=7",
  },
  {
    order: 8,
    id: "08-racks",
    title: "Equipment Racks",
    learnHref: "learn/08-racks.html",
    quizHref: "quiz.html?module=8",
  },
  {
    order: 9,
    id: "09-cooling",
    title: "Cooling Infrastructure",
    learnHref: "learn/09-cooling.html",
    quizHref: "quiz.html?module=9",
  },
  {
    order: 10,
    id: "10-water",
    title: "Water Supply",
    learnHref: "learn/10-water.html",
    quizHref: "quiz.html?module=10",
  },
  {
    order: 11,
    id: "11-network",
    title: "Designing a Scalable Network Infrastructure",
    learnHref: "learn/11-network.html",
    quizHref: "quiz.html?module=11",
  },
  {
    order: 12,
    id: "12-fire",
    title: "Fire Protection",
    learnHref: "learn/12-fire.html",
    quizHref: "quiz.html?module=12",
  },
  {
    order: 13,
    id: "13-security",
    title: "Physical Security and Safety",
    learnHref: "learn/13-security.html",
    quizHref: "quiz.html?module=13",
  },
  {
    order: 14,
    id: "14-auxiliary",
    title: "Auxiliary Systems",
    learnHref: "learn/14-auxiliary.html",
    quizHref: "quiz.html?module=14",
  },
]);

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
    if (!isFinite(n) || n < 1 || n > 14) continue;
    if (seen[n]) continue;
    // Only catalog modules (1–14 with learn pages).
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
 *  2. First unpracticed module (order 1→14)
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

  // 4) All practiced (and mastered under our laws).
  return {
    kind: "all_practiced",
    module: null,
    href: null,
    title: "All modules practiced",
    reason:
      "Every curriculum module (1-14) is practiced and mastered in this browser. Keep drilling or take another mock — study signal only, not a credential.",
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
 */
export function boot() {
  if (typeof document === "undefined") return;
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
    document.addEventListener("DOMContentLoaded", boot);
  } else {
    boot();
  }
}
