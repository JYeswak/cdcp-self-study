/**
 * CDCP results page (L5-S4 / bd-1an) — browser WASM grade dual-path.
 *
 * Flow
 * ----
 * 1. Read ExamAttempt from sessionStorage key `cdcp_mock_attempt_v1`.
 * 2. Load full bank JSON + keys for attempt.seed (`bank_items_seed{N}.json`,
 *    `keys_seed{N}.json`; default N=42). Only seed42 is golden-pinned.
 * 3. Prefer WASM GradeExact via grade_bridge.gradeDigest — digest is grade-of-record.
 * 4. On WASM success: show score / study signal / weak modules / digest / per-item
 *    explanations from keys (not the learner pack).
 * 5. On WASM failure: clear error only — do NOT invent scores in JS.
 *
 * PedagogySignal (study bar ≥27) is display language only; it never alters
 * score_correct / score_total numbers shown from the key comparison.
 *
 * Headless / offline without wasm
 * --------------------------------
 *   cargo run -q -p cdcp_cli -- grade \
 *     --fixture goldens/fixtures/mock40_seed42.json --mode all-correct
 *
 * E2e hook: window.__cdcp_last_digest (lowercase hex, 64 chars) after success.
 *
 * @module results
 */

import {
  loadWasm,
  gradeDigest,
  isWasmReady,
  DEFAULT_WASM_URL,
  ENGINE_IDENTITY_SUBJECT,
} from "./grade_bridge.js";
import { recordGradedWrongs } from "./review.js";
import { saveLastWeak } from "./hub_mastery.js";

const STORAGE_ATTEMPT = "cdcp_mock_attempt_v1";
/** Fallback when attempt has no seed (legacy drafts). */
const DEFAULT_SEED = 42;
const WASM_URL = DEFAULT_WASM_URL;

function bankUrl(seed) {
  return "data/bank_items_seed" + seed + ".json";
}

function keysUrl(seed) {
  return "data/keys_seed" + seed + ".json";
}

/** Study bar threshold (exam_form pass_correct / STUDY_PASS_CORRECT). Display only. */
const STUDY_PASS_CORRECT = 27;

/**
 * Bank module number (1–15) → learn page slug (basename without .html).
 * Matches web/data/modules_index.json `order` → `id` and web/learn/*.html.
 * Module 15 (ops-adjacent) is TAUGHT as of 2026-08-15 (CHARTER §11 row 8) — it
 * was previously empty-ok with no learn page, which meant a learner who missed
 * an ops-adjacent item got no "Review in Learn" link at all. Any module that can
 * appear on a form must be mapped here; see
 * crates/cdcp_assemble/tests/learn_surface_coverage.rs.
 */
export const MODULE_LEARN_SLUGS = Object.freeze({
  1: "01-mission-critical",
  2: "02-standards",
  3: "03-site-building",
  4: "04-floor-ceiling",
  5: "05-lighting",
  6: "06-power",
  7: "07-emf",
  8: "08-racks",
  9: "09-cooling",
  10: "10-water",
  11: "11-network",
  12: "12-fire",
  13: "13-security",
  14: "14-auxiliary",
  15: "15-ops-adjacent",
});

/**
 * Relative href from results.html to a Learn module page.
 * @param {number} moduleNum bank module 1–15
 * @returns {string|null} e.g. "learn/06-power.html" or null if unmapped
 */
export function moduleLearnHref(moduleNum) {
  const n = Number(moduleNum);
  const slug = MODULE_LEARN_SLUGS[n];
  if (!slug) return null;
  return "learn/" + slug + ".html";
}

/**
 * Relative href from results.html to a Learn page, optionally with section anchor.
 * Prefers topic_anchors map (web/data/topic_anchors.json from build_learn.py).
 * Falls back to module page when no section match.
 *
 * @param {number|null|undefined} moduleNum bank module 1–14
 * @param {string[]|null|undefined} topicIds item topic_ids
 * @param {{topics?: Record<string, {href?: string, anchor?: string|null, module?: number, slug?: string}>}|null|undefined} topicAnchors
 * @returns {string|null} e.g. "learn/06-power.html#transformers" or module page or null
 */
export function itemLearnHref(moduleNum, topicIds, topicAnchors) {
  const base = moduleNum != null ? moduleLearnHref(moduleNum) : null;
  const topics = topicAnchors && topicAnchors.topics ? topicAnchors.topics : null;
  const ids = Array.isArray(topicIds) ? topicIds : [];

  if (topics) {
    // Pass 1: prefer a topic that resolves to a section anchor.
    for (let i = 0; i < ids.length; i++) {
      const tid = ids[i];
      if (!tid || !topics[tid]) continue;
      const row = topics[tid];
      if (
        moduleNum != null &&
        row.module != null &&
        Number(row.module) !== Number(moduleNum)
      ) {
        continue;
      }
      if (row.anchor && base) {
        return base + "#" + row.anchor;
      }
      if (row.href && typeof row.href === "string" && row.href.indexOf("#") !== -1) {
        return row.href;
      }
    }
    // Pass 2: any topic-mapped module href (still better than null).
    for (let i = 0; i < ids.length; i++) {
      const tid = ids[i];
      if (!tid || !topics[tid]) continue;
      const row = topics[tid];
      if (
        moduleNum != null &&
        row.module != null &&
        Number(row.module) !== Number(moduleNum)
      ) {
        continue;
      }
      if (row.href && typeof row.href === "string") {
        return row.href;
      }
    }
  }

  return base;
}

/** Weak module: correctness rate strictly below 3/5 (integer: 5*c < 3*t). Matches cdcp_grade. */
function isWeakModule(correct, total) {
  if (total === 0) return false;
  return 5 * correct < 3 * total;
}

/**
 * Build all-correct ExamAttempt from keys pack (console / e2e helper).
 * @param {{ exam_id?: string, seed?: number, bank_hash?: string, keys: Array<{item_id:string, correct:string}> }} keysPack
 * @param {{ exam_id?: string, seed?: number, bank_hash?: string }} [overrides]
 */
export function buildAllCorrectAttempt(keysPack, overrides) {
  if (!keysPack || !Array.isArray(keysPack.keys)) {
    throw new TypeError("buildAllCorrectAttempt: keysPack.keys required");
  }
  const o = overrides || {};
  return {
    exam_id: o.exam_id != null ? o.exam_id : keysPack.exam_id || "mock40",
    seed: o.seed != null ? o.seed : keysPack.seed != null ? keysPack.seed : 42,
    bank_hash: o.bank_hash != null ? o.bank_hash : keysPack.bank_hash || "",
    answers: keysPack.keys.map(function (k) {
      return { item_id: k.item_id, chosen: k.correct };
    }),
  };
}

/**
 * Build all-wrong ExamAttempt (cycles A→B→C→D→A). Matches cdcp_grade::all_wrong_attempt.
 * @param {{ exam_id?: string, seed?: number, bank_hash?: string, keys: Array<{item_id:string, correct:string}> }} keysPack
 * @param {{ exam_id?: string, seed?: number, bank_hash?: string }} [overrides]
 */
export function buildAllWrongAttempt(keysPack, overrides) {
  const correct = buildAllCorrectAttempt(keysPack, overrides);
  const cycle = { A: "B", B: "C", C: "D", D: "A" };
  return {
    exam_id: correct.exam_id,
    seed: correct.seed,
    bank_hash: correct.bank_hash,
    answers: correct.answers.map(function (a) {
      const w = cycle[a.chosen] || "A";
      return { item_id: a.item_id, chosen: w };
    }),
  };
}

/**
 * Per-item + aggregate presentation from keys + bank items (display only).
 * Grade-of-record remains the WASM digest; this never runs when WASM failed.
 *
 * @param {{ answers: Array<{item_id:string, chosen:string}> }} attempt
 * @param {{ keys: Array<{item_id:string, correct:string, explanation?:string}> }} keysPack
 * @param {Array<{id:string, module:number, stem?:string, choices?:string[], topic_ids?:string[]}>| {items:Array}} bank
 * @param {{topics?: Record<string, object>}|null|undefined} [topicAnchors] optional L7-S2 map
 */
export function buildPresentation(attempt, keysPack, bank, topicAnchors) {
  const itemsArr = Array.isArray(bank) ? bank : bank && bank.items ? bank.items : [];
  const byId = Object.create(null);
  for (let i = 0; i < itemsArr.length; i++) {
    byId[itemsArr[i].id] = itemsArr[i];
  }
  const keyById = Object.create(null);
  const keys = keysPack.keys || [];
  for (let i = 0; i < keys.length; i++) {
    keyById[keys[i].item_id] = keys[i];
  }

  const itemRows = [];
  let scoreCorrect = 0;
  /** @type {Record<number, {correct:number, total:number}>} */
  const modMap = Object.create(null);

  for (let i = 0; i < attempt.answers.length; i++) {
    const ans = attempt.answers[i];
    const key = keyById[ans.item_id];
    const bankItem = byId[ans.item_id];
    const correctLetter = key ? key.correct : null;
    const isCorrect = !!(correctLetter && ans.chosen === correctLetter);
    if (isCorrect) scoreCorrect += 1;

    const module =
      bankItem && typeof bankItem.module === "number" ? bankItem.module : null;
    if (module != null) {
      if (!modMap[module]) modMap[module] = { correct: 0, total: 0 };
      modMap[module].total += 1;
      if (isCorrect) modMap[module].correct += 1;
    }

    const topicIds =
      bankItem && Array.isArray(bankItem.topic_ids) ? bankItem.topic_ids : [];
    const learnHref = itemLearnHref(module, topicIds, topicAnchors);

    itemRows.push({
      item_id: ans.item_id,
      chosen: ans.chosen,
      correct: correctLetter || "—",
      is_correct: isCorrect,
      explanation: key && key.explanation ? key.explanation : "",
      module: module,
      stem: bankItem && bankItem.stem ? bankItem.stem : "",
      topic_ids: topicIds,
      learn_href: learnHref,
    });
  }

  const scoreTotal = attempt.answers.length;
  const weakModules = [];
  const modNums = Object.keys(modMap)
    .map(Number)
    .sort(function (a, b) {
      return a - b;
    });
  for (let i = 0; i < modNums.length; i++) {
    const m = modNums[i];
    const st = modMap[m];
    if (isWeakModule(st.correct, st.total)) weakModules.push(m);
  }

  // Study language only — does not rewrite score_correct / score_total.
  const metStudyBar = scoreCorrect >= STUDY_PASS_CORRECT;

  return {
    score_correct: scoreCorrect,
    score_total: scoreTotal,
    weak_modules: weakModules,
    by_module: modNums.map(function (m) {
      return {
        module: m,
        correct: modMap[m].correct,
        total: modMap[m].total,
      };
    }),
    item_results: itemRows,
    met_study_bar: metStudyBar,
    study_pass_correct: STUDY_PASS_CORRECT,
  };
}

function $(id) {
  return document.getElementById(id);
}

function shortHash(h) {
  if (!h) return "—";
  return h.length > 16 ? h.slice(0, 12) + "…" : h;
}

function setStatus(el, kind, text) {
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

function errMsg(e) {
  if (e instanceof Error) return e.message;
  return String(e);
}

/**
 * Fetch text (not parsed) so bank JSON bytes match what WASM grades.
 * @param {string} url
 * @returns {Promise<string>}
 */
async function fetchText(url) {
  const res = await fetch(url, { cache: "no-store" });
  if (!res.ok) {
    throw new Error("HTTP " + res.status + " loading " + url);
  }
  return res.text();
}

function renderStudySignal(el, presentation) {
  el.hidden = false;
  const n = presentation.score_correct;
  const total = presentation.score_total;
  const bar = presentation.study_pass_correct;

  // Honesty: never claim certification. Amber study language only.
  if (presentation.met_study_bar) {
    el.className = "study-signal study-signal--met";
    el.innerHTML =
      "<strong>Study signal:</strong> " +
      n +
      " / " +
      total +
      " correct meets the practice bar of " +
      bar +
      ". " +
      "<em>This is not EPI/EXIN certification and is never a CDCP credential.</em> " +
      "Treat it as readiness practice only.";
  } else {
    el.className = "study-signal study-signal--below";
    el.innerHTML =
      "<strong>Study signal:</strong> " +
      n +
      " / " +
      total +
      " is below the practice bar of " +
      bar +
      ". " +
      "Review weak modules below. " +
      "<em>This tool never grants a CDCP credential.</em>";
  }
}

/**
 * Render weak-module chips as Learn deep links (L6-S3).
 * Each mapped module is <a href="learn/XX-slug.html">; unmapped stays a span.
 * Non-empty list gets CTA: "Review weak modules in Learn".
 *
 * @param {HTMLElement} el
 * @param {number[]} weak
 */
function renderWeakModules(el, weak) {
  el.hidden = false;
  if (!weak.length) {
    el.innerHTML =
      "<h2 class=\"results-section-title\">Weak modules</h2>" +
      "<p class=\"meta\" style=\"margin:0;border:0;padding:0\">" +
      "None flagged (module rate ≥ 3/5 on attempted items in that module)." +
      "</p>";
    return;
  }
  const chips = weak
    .map(function (m) {
      const label = "M" + String(m).padStart(2, "0");
      const href = moduleLearnHref(m);
      const title =
        "Module " + m + " correctness rate &lt; 3/5 — open Learn";
      if (href) {
        return (
          '<a class="weak-chip weak-chip--link mono" role="listitem" href="' +
          escapeHtml(href) +
          '" title="' +
          title +
          '">' +
          label +
          "</a>"
        );
      }
      return (
        '<span class="weak-chip mono" role="listitem" title="Module ' +
        m +
        ' correctness rate &lt; 3/5 (no Learn page)">' +
        label +
        "</span>"
      );
    })
    .join("");
  el.innerHTML =
    "<h2 class=\"results-section-title\">Weak modules</h2>" +
    "<p class=\"results-weak-cta\">Review weak modules in Learn</p>" +
    "<p class=\"meta\" style=\"margin:0 0 0.65rem;border:0;padding:0\">" +
    "Modules with correctness rate strictly below 3/5 on items you answered. " +
    "Tap a module to open its Learn page." +
    "</p>" +
    '<div class="weak-chip-row" role="list">' +
    chips +
    "</div>";
}

function renderItemList(el, rows) {
  el.hidden = false;
  const parts = [
    '<h2 class="results-section-title">Item review</h2>',
    '<ol class="results-item-list">',
  ];
  for (let i = 0; i < rows.length; i++) {
    const r = rows[i];
    const ok = r.is_correct;
    const cls = ok ? "results-item results-item--ok" : "results-item results-item--bad";
    const mark = ok ? "Correct" : "Incorrect";
    const markCls = ok ? "results-item__mark results-item__mark--ok" : "results-item__mark results-item__mark--bad";
    const stem = r.stem
      ? '<p class="results-item__stem">' + escapeHtml(r.stem) + "</p>"
      : "";
    const expl = r.explanation
      ? '<p class="results-item__expl">' + escapeHtml(r.explanation) + "</p>"
      : "";
    // L7-S2: link to learn/{slug}.html#section when possible, else module page.
    let learn = "";
    if (r.learn_href) {
      const hasAnchor = r.learn_href.indexOf("#") !== -1;
      const linkLabel = hasAnchor
        ? "Review section in Learn"
        : "Review module in Learn";
      learn =
        '<p class="results-item__learn">' +
        '<a class="results-item__learn-link" href="' +
        escapeHtml(r.learn_href) +
        '">' +
        linkLabel +
        " →</a>" +
        (r.module != null
          ? ' <span class="meta mono" style="margin:0;border:0;padding:0">M' +
            String(r.module).padStart(2, "0") +
            "</span>"
          : "") +
        "</p>";
    }
    parts.push(
      '<li class="' +
        cls +
        '">' +
        '<div class="results-item__head">' +
        '<span class="' +
        markCls +
        '">' +
        mark +
        "</span>" +
        '<span class="results-item__id mono">' +
        escapeHtml(r.item_id) +
        "</span>" +
        (r.module != null
          ? '<span class="results-item__mod mono">M' +
            String(r.module).padStart(2, "0") +
            "</span>"
          : "") +
        "</div>" +
        stem +
        '<p class="results-item__letters mono">chosen ' +
        escapeHtml(r.chosen) +
        " · correct " +
        escapeHtml(r.correct) +
        "</p>" +
        expl +
        learn +
        "</li>"
    );
  }
  parts.push("</ol>");
  el.innerHTML = parts.join("");
}

function escapeHtml(s) {
  return String(s)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

async function run() {
  const status = $("results-status");
  const summary = $("results-summary");
  const scorePanel = $("results-score");
  const studyEl = $("results-study-signal");
  const weakEl = $("results-weak");
  const itemsEl = $("results-items");
  const digestEl = $("r-digest");
  const scoreEl = $("r-score");
  const engineEl = $("r-engine");

  // Clear e2e hook until graded.
  window.__cdcp_last_digest = null;

  let attempt;
  try {
    const raw = sessionStorage.getItem(STORAGE_ATTEMPT);
    if (!raw) {
      setStatus(
        status,
        "error",
        "No attempt found in sessionStorage. Take the mock exam first, then submit."
      );
      return;
    }
    attempt = JSON.parse(raw);
    if (!attempt || !Array.isArray(attempt.answers) || attempt.answers.length === 0) {
      setStatus(status, "error", "Attempt JSON is invalid or has no answers.");
      return;
    }
  } catch (err) {
    setStatus(status, "error", "Could not read attempt: " + errMsg(err));
    return;
  }

  // Show attempt identity immediately (pre-grade).
  summary.hidden = false;
  $("r-exam").textContent = attempt.exam_id || "—";
  $("r-seed").textContent = attempt.seed != null ? String(attempt.seed) : "—";
  const h = attempt.bank_hash || "";
  $("r-hash").textContent = shortHash(h);
  $("r-hash").title = h;
  $("r-count").textContent = String(attempt.answers.length) + " item(s) recorded";

  const seed =
    attempt.seed != null && Number.isFinite(Number(attempt.seed))
      ? Number(attempt.seed)
      : DEFAULT_SEED;
  const BANK_URL = bankUrl(seed);
  const KEYS_URL = keysUrl(seed);

  setStatus(status, "", "Loading bank + keys and grading via WASM…");

  let bankJson;
  let keysPack;
  let bankParsed;
  /** @type {object|null} */
  let topicAnchors = null;
  try {
    const fetches = [
      fetchText(BANK_URL),
      fetchText(KEYS_URL),
      fetch("data/topic_anchors.json", { cache: "no-store" })
        .then(function (r) {
          if (!r.ok) return null;
          return r.json();
        })
        .catch(function () {
          return null;
        }),
    ];
    const [bankText, keysText, anchors] = await Promise.all(fetches);
    bankJson = bankText;
    keysPack = JSON.parse(keysText);
    bankParsed = JSON.parse(bankText);
    topicAnchors = anchors;
    if (!keysPack || !Array.isArray(keysPack.keys)) {
      throw new Error("keys pack missing keys[]");
    }
  } catch (err) {
    setStatus(
      status,
      "error",
      "Failed to load grade data (" +
        BANK_URL +
        " / " +
        KEYS_URL +
        "). Serve web/ over HTTP. If the pack is missing: cargo run -q -p cdcp_cli -- export-web --seed " +
        seed +
        " --out web/data. " +
        errMsg(err)
    );
    return;
  }

  // bank_hash on attempt must match keys/pack (full-bank hash).
  if (keysPack.bank_hash && attempt.bank_hash && keysPack.bank_hash !== attempt.bank_hash) {
    setStatus(
      status,
      "error",
      "bank_hash mismatch: attempt has " +
        shortHash(attempt.bank_hash) +
        ", keys pack has " +
        shortHash(keysPack.bank_hash) +
        ". Re-take the mock with the current pack."
    );
    return;
  }

  // --- WASM grade path (required for score display) ---
  let digest;
  try {
    await loadWasm(WASM_URL);
    if (!isWasmReady()) {
      throw new Error("WASM loaded but exports missing");
    }
    const attemptJson = JSON.stringify(attempt);
    digest = await gradeDigest(bankJson, attemptJson);
    if (typeof digest !== "string" || !/^[0-9a-f]{64}$/.test(digest)) {
      throw new Error("unexpected digest shape: " + String(digest));
    }
  } catch (err) {
    // Fail closed: no invented scores.
    window.__cdcp_last_digest = null;
    if (scorePanel) scorePanel.hidden = true;
    if (studyEl) studyEl.hidden = true;
    if (weakEl) weakEl.hidden = true;
    if (itemsEl) itemsEl.hidden = true;
    setStatus(
      status,
      "error",
      "WASM grade failed — scores withheld (no JS fallback grader). " +
        errMsg(err) +
        " Build with ./scripts/build_web_wasm.sh and ensure " +
        WASM_URL +
        " is served. CLI oracle: cargo run -q -p cdcp_cli -- grade " +
        "--fixture goldens/fixtures/mock40_seed42.json --mode all-correct"
    );
    return;
  }

  window.__cdcp_last_digest = digest;

  // Presentation from keys + bank (display only; digest is grade-of-record).
  // topicAnchors enables L7-S2 section deep links when present.
  const presentation = buildPresentation(
    attempt,
    keysPack,
    bankParsed,
    topicAnchors
  );

  scorePanel.hidden = false;
  scoreEl.textContent =
    String(presentation.score_correct) + " / " + String(presentation.score_total);
  digestEl.textContent = digest;
  digestEl.title = digest;
  if (engineEl) {
    engineEl.textContent = ENGINE_IDENTITY_SUBJECT;
  }

  renderStudySignal(studyEl, presentation);
  renderWeakModules(weakEl, presentation.weak_modules);
  renderItemList(itemsEl, presentation.item_results);

  // L6-S4: persist weak modules for hub recommend (cdcp.last_weak.v1).
  try {
    saveLastWeak(presentation.weak_modules, {
      source: "mock",
      atMs: Date.now(),
    });
  } catch (weakErr) {
    console.warn("last_weak persist failed:", errMsg(weakErr));
  }

  // L5-S7: feed Drill / short-interval review with missed item_ids.
  try {
    const rec = recordGradedWrongs({
      source: "mock",
      exam_id: attempt.exam_id,
      seed: attempt.seed,
      bank_hash: attempt.bank_hash,
      item_results: presentation.item_results,
    });
    const drillLink = document.getElementById("results-drill-link");
    if (drillLink) {
      drillLink.hidden = false;
      drillLink.textContent =
        rec.missed_ids.length > 0
          ? "Drill " + rec.missed_ids.length + " missed item(s) →"
          : "Open Drill →";
    }
  } catch (srsErr) {
    console.warn("review/missed record failed:", errMsg(srsErr));
  }

  setStatus(
    status,
    "ok",
    "Graded via WASM (" +
      ENGINE_IDENTITY_SUBJECT +
      "). Digest is GradeExact-of-record; study language is separate."
  );
}

// Console / e2e helpers (also available as ES exports above).
if (typeof window !== "undefined") {
  window.CdcpResults = {
    buildAllCorrectAttempt,
    buildAllWrongAttempt,
    buildPresentation,
    moduleLearnHref,
    itemLearnHref,
    MODULE_LEARN_SLUGS,
    STUDY_PASS_CORRECT,
    STORAGE_ATTEMPT,
    saveLastWeak,
  };
}

// Browser auto-run only (Node smoke imports helpers without DOM).
if (typeof document !== "undefined") {
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function () {
      run().catch(function (err) {
        const status = $("results-status");
        if (status) setStatus(status, "error", "Results failed: " + errMsg(err));
      });
    });
  } else {
    run().catch(function (err) {
      const status = $("results-status");
      if (status) setStatus(status, "error", "Results failed: " + errMsg(err));
    });
  }
}
