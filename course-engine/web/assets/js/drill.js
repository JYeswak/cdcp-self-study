/**
 * Drill + short-interval review surface (L5-S7 / L6-S6).
 *
 * Modes via ?mode= query (one surface, three learner-visible modes):
 *   due    — Drill-10: review cards with due_at ≤ now, first 10 only
 *   missed — items from cdcp.drill.missed.v1 only
 *   module — approved items from one module (?m=NN)
 *   (default / omitted) — full dashboard: missed + due + all cards
 *
 * Explanations from keys/bank export — never from an LLM grader.
 * Interval ladder is 1d → 3d (cap) via cdcp_schedule / WASM — not SRS.
 *
 * @module drill
 */

import { loadWasm } from "./grade_bridge.js";
import {
  loadMissed,
  listDue,
  listDueDrill,
  listAllCards,
  reviewCard,
  selectDueOnly,
  DRILL10_LIMIT,
  MISSED_STORAGE_KEY,
  REVIEW_STORAGE_KEY,
  pruneNonApprovedFromStorage,
  formatPruneNotice,
} from "./review.js";

const BANK_URL = "data/bank_items_seed42.json";
const KEYS_URL = "data/keys_seed42.json";

/** Honest empty copy for Drill-10 when nothing is due. */
export const EMPTY_DUE_MESSAGE = "No cards due — take a mock or quiz";

/** @type {Record<string, {id:string, stem:string, choices:string[], correct:string, explanation:string, module:number}>} */
let byId = Object.create(null);
/** @type {Record<string, {correct:string, explanation:string}>} */
let keyById = Object.create(null);

/** @type {"due" | "miss" | "module" | "default"} */
let pageMode = "default";
let pageModule = null;

function $(id) {
  return document.getElementById(id);
}

function errMsg(e) {
  if (e instanceof Error) return e.message;
  return String(e);
}

function escapeHtml(s) {
  return String(s)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

/**
 * Parse ?mode=due | ?mode=missed | ?mode=module | default.
 * @param {string} [search] — location.search (injectable for tests)
 * @returns {"due" | "miss" | "module" | "default"}
 */
export function parseDrillMode(search) {
  const q =
    typeof search === "string"
      ? search
      : typeof window !== "undefined" && window.location
        ? window.location.search
        : "";
  let mode = "";
  try {
    const params = new URLSearchParams(
      q.charAt(0) === "?" ? q.slice(1) : q
    );
    mode = (params.get("mode") || "").toLowerCase().trim();
  } catch (_) {
    mode = "";
  }
  if (mode === "due") return mode;
  if (mode === "missed" || mode === "miss") return "miss";
  if (mode === "module") return "module";
  return "default";
}

/** Parse the module number for the module drill mode. */
export function parseDrillModule(search) {
  const q =
    typeof search === "string"
      ? search
      : typeof window !== "undefined" && window.location
        ? window.location.search
        : "";
  try {
    const n = parseInt(new URLSearchParams(q).get("m") || "", 10);
    return Number.isFinite(n) && n >= 1 && n <= 99 ? n : null;
  } catch (_) {
    return null;
  }
}

function setStatus(kind, text) {
  const el = $("drill-status");
  if (!el) return;
  el.hidden = false;
  el.className =
    "exam-status" +
    (kind === "error"
      ? " exam-status--error"
      : kind === "ok"
        ? " exam-status--ok"
        : "");
  el.textContent = text;
}

/**
 * Surface a withdrawal sweep. A no-op (nothing dropped) must not
 * paint a "cards removed" banner.
 * @param {{ dropped?: string[], emptied?: boolean }} report
 */
function showPruneNotice(report) {
  const text = formatPruneNotice(report);
  const existing = $("drill-prune-notice");
  if (!text) {
    if (existing) existing.remove();
    return;
  }
  let el = existing;
  if (!el) {
    el = document.createElement("div");
    el.id = "drill-prune-notice";
    el.className = "exam-status exam-status--ok";
    el.setAttribute("role", "status");
    const status = $("drill-status");
    if (!status || !status.parentNode) return;
    status.parentNode.insertBefore(el, status.nextSibling);
  }
  el.hidden = false;
  el.textContent = text;
}

function lookupItem(itemId) {
  const bank = byId[itemId];
  const key = keyById[itemId];
  const correct =
    (key && key.correct) || (bank && bank.correct) || "—";
  const explanation =
    (key && key.explanation) || (bank && bank.explanation) || "";
  return {
    item_id: itemId,
    stem: (bank && bank.stem) || "(stem not in local bank export)",
    choices: (bank && bank.choices) || [],
    correct: correct,
    explanation: explanation,
    module: bank && typeof bank.module === "number" ? bank.module : null,
  };
}

function formatDue(dueAt, now) {
  const ms = dueAt - now;
  if (ms <= 0) return "due now";
  const days = ms / (24 * 60 * 60 * 1000);
  if (days < 1) {
    const h = Math.ceil(ms / (60 * 60 * 1000));
    return "in ~" + h + "h";
  }
  return "in " + days.toFixed(1) + "d";
}

/**
 * Show/hide dashboard sections and retitle for session shape.
 * @param {"due" | "miss" | "module" | "default"} mode
 */
function applyModeChrome(mode) {
  const missedSec = $("section-missed");
  const dueSec = $("section-srs-due");
  const allSec = $("section-srs-all");
  const moduleSec = $("section-module");
  const title = document.querySelector("main h1");
  const lede = document.querySelector("main .lede");

  if (mode === "due") {
    if (missedSec) missedSec.hidden = true;
    if (dueSec) dueSec.hidden = false;
    if (allSec) allSec.hidden = true;
    if (moduleSec) moduleSec.hidden = true;
    if (title) title.textContent = "Drill-10 (due only)";
    if (lede) {
      lede.textContent =
        "Due queue only — up to " +
        DRILL10_LIMIT +
        " cards with due_at ≤ now. Study signal only; not a credential.";
    }
    const dueTitle = $("srs-due-title");
    if (dueTitle) dueTitle.textContent = "Due now (max " + DRILL10_LIMIT + ")";
  } else if (mode === "miss") {
    if (missedSec) missedSec.hidden = false;
    if (dueSec) dueSec.hidden = true;
    if (allSec) allSec.hidden = true;
    if (moduleSec) moduleSec.hidden = true;
    if (title) title.textContent = "Miss review";
    if (lede) {
      lede.textContent =
        "Only items you missed on the last mock or module quiz. Explanations from keys/bank — never an LLM grader.";
    }
  } else if (mode === "module") {
    if (missedSec) missedSec.hidden = true;
    if (dueSec) dueSec.hidden = true;
    if (allSec) allSec.hidden = true;
    if (moduleSec) moduleSec.hidden = false;
    if (title) title.textContent = "Module drill";
    if (lede) {
      lede.textContent =
        "Practice approved items from one module. Explanations come from the keys/bank export — never an LLM grader.";
    }
  } else {
    if (missedSec) missedSec.hidden = false;
    if (dueSec) dueSec.hidden = false;
    if (allSec) allSec.hidden = false;
    if (moduleSec) moduleSec.hidden = true;
    if (title) title.textContent = "Drill / short-interval review";
    if (lede) {
      lede.textContent =
        "Review items you missed on the last mock or module quiz. Explanations come from the keys/bank export — never from an LLM grader. Short-interval ladder is 1d → 3d (cap) — not spaced repetition.";
    }
    const dueTitle = $("srs-due-title");
    if (dueTitle) dueTitle.textContent = "Due now";
  }
}

function populateModuleSelect() {
  const select = $("drill-module-select");
  if (!select) return;
  const modules = Object.keys(byId)
    .map((id) => byId[id])
    .filter((it) => it && it.status === "approved" && Number.isFinite(it.module))
    .map((it) => it.module)
    .filter((value, index, arr) => arr.indexOf(value) === index)
    .sort((a, b) => a - b);
  for (let i = 0; i < modules.length; i++) {
    const option = document.createElement("option");
    option.value = String(modules[i]);
    option.textContent = "Module " + String(modules[i]).padStart(2, "0");
    select.appendChild(option);
  }
  if (pageModule != null && modules.indexOf(pageModule) !== -1) {
    select.value = String(pageModule);
  }
  select.addEventListener("change", function () {
    const n = parseInt(select.value, 10);
    if (!Number.isFinite(n)) return;
    window.location.href = "drill.html?mode=module&m=" + n;
  });
}

function renderModule() {
  const host = $("module-list");
  const meta = $("module-meta");
  if (!host) return;
  const moduleItems = Object.keys(byId)
    .map((id) => byId[id])
    .filter((it) => it && it.status === "approved" && it.module === pageModule);
  if (meta) {
    meta.textContent =
      pageModule == null
        ? "Choose a module above to begin."
        : "Module " + String(pageModule).padStart(2, "0") + " · " + moduleItems.length + " approved item(s)";
  }
  if (!moduleItems.length) {
    host.innerHTML =
      '<p class="meta" style="margin:0;border:0;padding:0">No approved items are available for this module.</p>';
    return;
  }
  const parts = ['<ul class="results-item-list">'];
  for (let i = 0; i < moduleItems.length; i++) {
    const it = lookupItem(moduleItems[i].id);
    const panel = "module-expl-" + i;
    parts.push(
      '<li class="results-item drill-card" data-item-id="' + escapeHtml(it.item_id) + '">' +
        '<div class="results-item__head"><span class="results-item__id mono">' + escapeHtml(it.item_id) +
        '</span><span class="results-item__mod mono">M' + String(pageModule).padStart(2, "0") + '</span></div>' +
        '<p class="results-item__stem">' + escapeHtml(it.stem) + '</p>' +
        '<button type="button" class="btn btn--ghost drill-flip" data-expl="' + panel +
        '" aria-expanded="false">Show explanation</button>' +
        '<div id="' + panel + '" class="drill-expl" hidden><p class="results-item__letters mono">correct ' +
        escapeHtml(it.correct) + '</p>' + (it.explanation ? '<p class="results-item__expl">' + escapeHtml(it.explanation) + '</p>' : '') + '</div>' +
      '</li>'
    );
  }
  parts.push("</ul>");
  host.innerHTML = parts.join("");
  const flips = host.querySelectorAll(".drill-flip");
  for (let i = 0; i < flips.length; i++) flips[i].addEventListener("click", onFlip);
}

function renderMissed() {
  const host = $("missed-list");
  const meta = $("missed-meta");
  const missed = loadMissed();
  if (!host) return;

  if (!missed || !missed.item_ids.length) {
    if (meta) {
      meta.textContent =
        "No missed items stored yet. Take a mock or module quiz and answer some incorrectly.";
    }
    host.innerHTML =
      '<p class="meta" style="margin:0;border:0;padding:0">Empty. Storage key <span class="mono">' +
      escapeHtml(MISSED_STORAGE_KEY) +
      "</span>.</p>";
    return;
  }

  if (meta) {
    const when = missed.saved_at
      ? new Date(missed.saved_at).toLocaleString()
      : "—";
    meta.textContent =
      missed.item_ids.length +
      " missed from " +
      (missed.source || "unknown") +
      (missed.exam_id ? " · " + missed.exam_id : "") +
      " · saved " +
      when;
  }

  const parts = ['<ul class="results-item-list">'];
  for (let i = 0; i < missed.item_ids.length; i++) {
    const id = missed.item_ids[i];
    const it = lookupItem(id);
    const explId = "expl-" + i;
    parts.push(
      '<li class="results-item results-item--bad drill-card" data-item-id="' +
        escapeHtml(id) +
        '">' +
        '<div class="results-item__head">' +
        '<span class="results-item__id mono">' +
        escapeHtml(id) +
        "</span>" +
        (it.module != null
          ? '<span class="results-item__mod mono">M' +
            String(it.module).padStart(2, "0") +
            "</span>"
          : "") +
        "</div>" +
        '<p class="results-item__stem">' +
        escapeHtml(it.stem) +
        "</p>" +
        '<button type="button" class="btn btn--ghost drill-flip" data-expl="' +
        explId +
        '" aria-expanded="false">Show explanation</button>' +
        '<div id="' +
        explId +
        '" class="drill-expl" hidden>' +
        '<p class="results-item__letters mono">correct ' +
        escapeHtml(it.correct) +
        "</p>" +
        (it.explanation
          ? '<p class="results-item__expl">' +
            escapeHtml(it.explanation) +
            "</p>"
          : '<p class="results-item__expl">No explanation in keys/bank for this item.</p>') +
        '<div class="concept-card-host" data-concept-for="' +
        escapeHtml(id) +
        '"></div>' +
        "</div></li>"
    );
  }
  parts.push("</ul>");
  host.innerHTML = parts.join("");

  const flips = host.querySelectorAll(".drill-flip");
  for (let f = 0; f < flips.length; f++) {
    flips[f].addEventListener("click", onFlip);
  }
  // M8-D1 concept cards
  if (globalThis.CdcpConceptCard) {
    const hosts = host.querySelectorAll(".concept-card-host");
    for (let c = 0; c < hosts.length; c++) {
      const hid = hosts[c].getAttribute("data-concept-for");
      const it = lookupItem(hid);
      globalThis.CdcpConceptCard.render(hosts[c], it, null);
    }
  }
}

function onFlip(ev) {
  const btn = ev.currentTarget;
  const id = btn.getAttribute("data-expl");
  const panel = id ? document.getElementById(id) : null;
  if (!panel) return;
  const open = panel.hidden;
  panel.hidden = !open;
  btn.setAttribute("aria-expanded", open ? "true" : "false");
  if (open) {
    if (!btn.dataset.closedLabel) {
      btn.dataset.closedLabel = btn.textContent || "Show explanation";
    }
    const closed = btn.dataset.closedLabel;
    btn.textContent =
      closed.indexOf("answer") >= 0 ? "Hide answer" : "Hide explanation";
  } else {
    btn.textContent = btn.dataset.closedLabel || "Show explanation";
  }
}

function renderSrs(mode) {
  const dueHost = $("srs-due-list");
  const allHost = $("srs-all-meta");
  if (!dueHost) return;
  const now = Date.now();
  const all = listAllCards();
  const allDue = listDue({ nowMs: now });
  // Drill-10 (mode=due): cap at 10. Default dashboard: show full due list.
  const due =
    mode === "due" ? selectDueOnly(allDue, now, DRILL10_LIMIT) : allDue;

  if (allHost) {
    allHost.textContent =
      all.length +
      " card(s) in " +
      REVIEW_STORAGE_KEY +
      " · " +
      due.length +
      (mode === "due" ? " in this Drill-10" : " due now") +
      (mode === "due" && allDue.length > DRILL10_LIMIT
        ? " (of " + allDue.length + " total due)"
        : "");
  }

  if (due.length === 0) {
    dueHost.innerHTML =
      '<p class="meta" style="margin:0;border:0;padding:0">' +
      escapeHtml(EMPTY_DUE_MESSAGE) +
      "</p>";
    return;
  }

  const parts = ['<ul class="results-item-list">'];
  for (let i = 0; i < due.length; i++) {
    const card = due[i];
    const it = lookupItem(card.item_id);
    const rid = "srs-expl-" + i;
    parts.push(
      '<li class="results-item drill-card" data-item-id="' +
        escapeHtml(card.item_id) +
        '">' +
        '<div class="results-item__head">' +
        '<span class="results-item__id mono">' +
        escapeHtml(card.item_id) +
        "</span>" +
        '<span class="results-item__mod mono">ivl ' +
        card.interval_days +
        "d · " +
        formatDue(card.due_at, now) +
        "</span></div>" +
        '<p class="results-item__stem">' +
        escapeHtml(it.stem) +
        "</p>" +
        '<div class="drill-srs-actions">' +
        '<button type="button" class="btn btn--ghost drill-flip" data-expl="' +
        rid +
        '" aria-expanded="false">Show answer</button>' +
        '<button type="button" class="btn btn--ghost srs-again" data-item="' +
        escapeHtml(card.item_id) +
        '">Again (1d)</button>' +
        '<button type="button" class="btn btn--primary srs-good" data-item="' +
        escapeHtml(card.item_id) +
        '">Good (next step)</button>' +
        "</div>" +
        '<div id="' +
        rid +
        '" class="drill-expl" hidden>' +
        '<p class="results-item__letters mono">correct ' +
        escapeHtml(it.correct) +
        "</p>" +
        (it.explanation
          ? '<p class="results-item__expl">' +
            escapeHtml(it.explanation) +
            "</p>"
          : "") +
        "</div></li>"
    );
  }
  parts.push("</ul>");
  dueHost.innerHTML = parts.join("");

  const flips = dueHost.querySelectorAll(".drill-flip");
  for (let f = 0; f < flips.length; f++) {
    flips[f].addEventListener("click", onFlip);
  }
  const goods = dueHost.querySelectorAll(".srs-good");
  for (let g = 0; g < goods.length; g++) {
    goods[g].addEventListener("click", function (ev) {
      const id = ev.currentTarget.getAttribute("data-item");
      reviewCard(id, true);
      renderSrs(pageMode);
      renderAllSrsTable();
      setStatus("ok", "Marked good — interval stepped (1d→3d cap).");
    });
  }
  const agains = dueHost.querySelectorAll(".srs-again");
  for (let a = 0; a < agains.length; a++) {
    agains[a].addEventListener("click", function (ev) {
      const id = ev.currentTarget.getAttribute("data-item");
      reviewCard(id, false);
      renderSrs(pageMode);
      renderAllSrsTable();
      setStatus("ok", "Marked again — back to 1d.");
    });
  }
}

function renderAllSrsTable() {
  const host = $("srs-all-list");
  if (!host) return;
  const all = listAllCards();
  const now = Date.now();
  if (!all.length) {
    host.innerHTML = "";
    return;
  }
  const parts = [
    '<table class="srs-table"><thead><tr>' +
      "<th>item_id</th><th>interval</th><th>due</th><th>reps</th><th>lapses</th>" +
      "</tr></thead><tbody>",
  ];
  for (let i = 0; i < all.length; i++) {
    const c = all[i];
    parts.push(
      "<tr><td class=\"mono\">" +
        escapeHtml(c.item_id) +
        "</td><td class=\"mono\">" +
        c.interval_days +
        "d</td><td class=\"mono\">" +
        formatDue(c.due_at, now) +
        "</td><td class=\"mono\">" +
        c.reps +
        "</td><td class=\"mono\">" +
        c.lapses +
        "</td></tr>"
    );
  }
  parts.push("</tbody></table>");
  host.innerHTML = parts.join("");
}

async function loadBank() {
  const [bankRes, keysRes] = await Promise.all([
    fetch(BANK_URL, { cache: "no-store" }),
    fetch(KEYS_URL, { cache: "no-store" }),
  ]);
  if (!bankRes.ok) throw new Error("HTTP " + bankRes.status + " " + BANK_URL);
  if (!keysRes.ok) throw new Error("HTTP " + keysRes.status + " " + KEYS_URL);
  const bank = await bankRes.json();
  const keysPack = await keysRes.json();
  const arr = Array.isArray(bank) ? bank : bank.items || [];
  // Index EVERY manifest row, including retired. Filtering byId by
  // status would blank the stem of a legitimately-answered withdrawn
  // card instead of removing it from the schedule
  // (bd-srs-residue-retired-ids-k3vs).
  byId = Object.create(null);
  for (let i = 0; i < arr.length; i++) {
    if (arr[i] && arr[i].id) byId[arr[i].id] = arr[i];
  }
  keyById = Object.create(null);
  const keys = (keysPack && keysPack.keys) || [];
  for (let j = 0; j < keys.length; j++) {
    if (keys[j] && keys[j].item_id) keyById[keys[j].item_id] = keys[j];
  }
}

async function init() {
  pageMode = parseDrillMode();
  pageModule = parseDrillModule();
  applyModeChrome(pageMode);

  try {
    await loadWasm();
  } catch (err) {
    setStatus(
      "error",
      "Schedule law requires WASM (cdcp_schedule). " + errMsg(err)
    );
    return;
  }

  setStatus("", "Loading bank + keys for stems/explanations…");
  try {
    await loadBank();
    populateModuleSelect();
    // Sweep residue AFTER the pack is in byId. Do not filter byId —
    // that map resolves already-answered ids, including withdrawn ones
    // if we ever render a "withdrawn" card. The schedule/missed stores
    // are what must not keep a retired id due.
    const rows = [];
    const ids = Object.keys(byId);
    for (let i = 0; i < ids.length; i++) rows.push(byId[ids[i]]);
    showPruneNotice(pruneNonApprovedFromStorage(rows));
  } catch (err) {
    setStatus(
      "error",
      "Failed to load bank/keys. Serve web/ over HTTP. " + errMsg(err)
    );
    // Still render storage-backed lists without stems.
    // Do NOT prune against an empty pack — that would wipe a live queue.
  }

  if (pageMode === "due") {
    renderSrs("due");
    const due = listDueDrill();
    setStatus(
      "ok",
      "Drill-10 · " +
        due.length +
        " due card(s) (cap " +
        DRILL10_LIMIT +
        "). Study only — not a credential."
    );
  } else if (pageMode === "miss") {
    renderMissed();
    const missed = loadMissed();
    setStatus(
      "ok",
      "Miss review · " +
        (missed ? missed.item_ids.length : 0) +
        " item(s). Study only — not a credential."
    );
  } else if (pageMode === "module") {
    renderModule();
    setStatus(
      "ok",
      "Module drill · " +
        (pageModule == null ? "choose a module" : "M" + String(pageModule).padStart(2, "0")) +
        ". Study only — not a credential."
    );
  } else {
    renderMissed();
    renderSrs("default");
    renderAllSrsTable();
    const missed = loadMissed();
    const due = listDue();
    setStatus(
      "ok",
      "Drill ready · missed " +
        (missed ? missed.item_ids.length : 0) +
        " · due " +
        due.length +
        ". Study only — not a credential."
    );
  }
}

if (typeof window !== "undefined") {
  window.CdcpDrill = {
    loadMissed,
    listDue,
    listDueDrill,
    listAllCards,
    parseDrillModule,
    reviewCard,
    selectDueOnly,
    parseDrillMode,
    pruneNonApprovedFromStorage,
    formatPruneNotice,
    DRILL10_LIMIT,
    EMPTY_DUE_MESSAGE,
    MISSED_STORAGE_KEY,
    REVIEW_STORAGE_KEY,
  };
}

if (typeof document !== "undefined") {
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function () {
      init().catch(function (err) {
        setStatus("error", "Drill init failed: " + errMsg(err));
      });
    });
  } else {
    init().catch(function (err) {
      setStatus("error", "Drill init failed: " + errMsg(err));
    });
  }
}
