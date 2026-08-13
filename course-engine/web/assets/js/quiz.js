/**
 * Module quiz (L5-S7 / bd-ca8).
 *
 * Samples 8–12 items for one bank module (BankItem.module number), grades via
 * WASM grade_bridge when available (same GradeExact law as mock). Presentation
 * and explanations come from bank rows / key-compare — never LLM.
 *
 * Honesty: quiz score is a study signal only. Not a CDCP credential. Module
 * quiz digests are NOT the mock40 golden cert path; they share the same
 * grade_digest engine when WASM loads.
 *
 * URL: quiz.html?module=6  (module number 1–15)
 * Storage: session draft `cdcp_quiz_draft_v1`; wrongs → srs.recordGradedWrongs;
 * score → mastery.recordQuizResult (`cdcp.mastery.v1`).
 *
 * @module quiz
 */

import {
  loadWasm,
  gradeDigest,
  isWasmReady,
  DEFAULT_WASM_URL,
  ENGINE_IDENTITY_SUBJECT,
} from "./grade_bridge.js";
import { recordGradedWrongs } from "./srs.js";
import { recordQuizResult } from "./mastery.js";

const BANK_URL = "data/bank_items_seed42.json";
const KEYS_URL = "data/keys_seed42.json";
const WASM_URL = DEFAULT_WASM_URL;
const STORAGE_DRAFT = "cdcp_quiz_draft_v1";
const QUIZ_MIN = 8;
const QUIZ_MAX = 12;
const LETTERS = ["A", "B", "C", "D"];

/** @type {{ module: number, bank_hash: string, seed: number, items: Array, exam_id: string } | null} */
let pack = null;
/** @type {Record<string, "A"|"B"|"C"|"D">} */
let answers = Object.create(null);
let index = 0;
/** @type {string | null} */
let lastDigest = null;
let gradeMode = ""; // "wasm" | "key-compare"

const el = {
  status: null,
  picker: null,
  exam: null,
  results: null,
  progress: null,
  card: null,
  stem: null,
  choices: null,
  prev: null,
  next: null,
  submit: null,
  unanswered: null,
  moduleSelect: null,
  startBtn: null,
  scoreLine: null,
  digestLine: null,
  modeLine: null,
  itemList: null,
};

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
 * Parse ?module=N from location.
 * @returns {number | null}
 */
export function parseModuleParam(search) {
  const q = new URLSearchParams(
    typeof search === "string" ? search : window.location.search
  );
  const raw = q.get("module");
  if (raw == null || raw === "") return null;
  const n = parseInt(raw, 10);
  if (!isFinite(n) || n < 1 || n > 99) return null;
  return n;
}

/**
 * Parse sample size from ?count=N or Learn-15 shortcut ?mode=learn15 (count=5).
 * Returns { min, max } for sampleItems.
 */
export function parseCountParams(search) {
  const q = new URLSearchParams(
    typeof search === "string" ? search : window.location.search
  );
  const mode = (q.get("mode") || "").toLowerCase();
  if (mode === "learn15" || mode === "learn-15") {
    return { min: 5, max: 5, learn15: true };
  }
  const raw = q.get("count");
  if (raw == null || raw === "") {
    return { min: QUIZ_MIN, max: QUIZ_MAX, learn15: false };
  }
  const n = parseInt(raw, 10);
  if (!isFinite(n) || n < 1 || n > 40) {
    return { min: QUIZ_MIN, max: QUIZ_MAX, learn15: false };
  }
  return { min: n, max: n, learn15: n === 5 };
}

/**
 * Filter bank items by module number. BankItem schema uses numeric `module`.
 * @param {Array|{items:Array}} bank
 * @param {number} moduleNum
 */
export function filterByModule(bank, moduleNum) {
  const arr = Array.isArray(bank) ? bank : bank && bank.items ? bank.items : [];
  return arr.filter(function (it) {
    return it && typeof it.module === "number" && it.module === moduleNum;
  });
}

/**
 * Deterministic sample of k items (mulberry32 from seed).
 * Prefer 8–12; if pool smaller than 8, take all.
 *
 * @param {Array} items
 * @param {number} seed
 * @param {number} [minN]
 * @param {number} [maxN]
 */
export function sampleItems(items, seed, minN, maxN) {
  const min = minN != null ? minN : QUIZ_MIN;
  const max = maxN != null ? maxN : QUIZ_MAX;
  if (!Array.isArray(items) || items.length === 0) return [];
  if (items.length <= min) return items.slice();
  const target = Math.min(max, items.length);
  // mulberry32
  let t = (seed >>> 0) + 0x6d2b79f5;
  function rand() {
    t |= 0;
    t = (t + 0x6d2b79f5) | 0;
    let r = Math.imul(t ^ (t >>> 15), 1 | t);
    r = (r + Math.imul(r ^ (r >>> 7), 61 | r)) ^ r;
    return ((r ^ (r >>> 14)) >>> 0) / 4294967296;
  }
  const idx = items.map(function (_, i) {
    return i;
  });
  for (let i = idx.length - 1; i > 0; i--) {
    const j = Math.floor(rand() * (i + 1));
    const tmp = idx[i];
    idx[i] = idx[j];
    idx[j] = tmp;
  }
  const picked = idx.slice(0, target).sort(function (a, b) {
    return a - b;
  });
  return picked.map(function (i) {
    return items[i];
  });
}

/**
 * Key-compare presentation (same letter law as GradeExact item check).
 * @param {Array<{id:string, correct:string, explanation?:string, stem?:string, module?:number}>} items
 * @param {Record<string,string>} answerMap
 */
export function gradeByKeys(items, answerMap) {
  const rows = [];
  let correct = 0;
  for (let i = 0; i < items.length; i++) {
    const it = items[i];
    const chosen = answerMap[it.id] || null;
    const ok = !!(chosen && it.correct && chosen === it.correct);
    if (ok) correct += 1;
    rows.push({
      item_id: it.id,
      chosen: chosen || "—",
      correct: it.correct || "—",
      is_correct: ok,
      explanation: it.explanation || "",
      stem: it.stem || "",
      module: typeof it.module === "number" ? it.module : null,
    });
  }
  return {
    score_correct: correct,
    score_total: items.length,
    item_results: rows,
  };
}

function learnerItems(fullItems) {
  // Strip correct/explanation from on-screen pack (bank still holds keys for grade).
  return fullItems.map(function (it) {
    return {
      id: it.id,
      stem: it.stem,
      choices: it.choices.slice(0, 4),
      module: it.module,
      correct: it.correct,
      explanation: it.explanation || "",
    };
  });
}

function answeredCount() {
  if (!pack) return 0;
  let n = 0;
  for (let i = 0; i < pack.items.length; i++) {
    if (answers[pack.items[i].id]) n += 1;
  }
  return n;
}

function saveDraft() {
  if (!pack) return;
  try {
    sessionStorage.setItem(
      STORAGE_DRAFT,
      JSON.stringify({
        exam_id: pack.exam_id,
        module: pack.module,
        seed: pack.seed,
        bank_hash: pack.bank_hash,
        item_ids: pack.items.map(function (it) {
          return it.id;
        }),
        answers: answers,
        index: index,
        saved_at: Date.now(),
      })
    );
  } catch (_) {
    /* quota */
  }
}

function loadDraft() {
  try {
    const raw = sessionStorage.getItem(STORAGE_DRAFT);
    if (!raw) return null;
    return JSON.parse(raw);
  } catch (_) {
    return null;
  }
}

function buildAttempt() {
  const list = [];
  for (let i = 0; i < pack.items.length; i++) {
    const it = pack.items[i];
    const chosen = answers[it.id];
    if (chosen) list.push({ item_id: it.id, chosen: chosen });
  }
  return {
    exam_id: pack.exam_id,
    seed: pack.seed,
    bank_hash: pack.bank_hash,
    answers: list,
  };
}

function updateChrome() {
  if (!pack) return;
  const total = pack.items.length;
  const answered = answeredCount();
  const n = index + 1;
  el.progress.textContent = n + " / " + total;
  el.progress.setAttribute(
    "aria-label",
    "Question " + n + " of " + total + ", " + answered + " answered"
  );
  const allDone = answered === total;
  el.submit.disabled = !allDone;
  el.unanswered.textContent = allDone
    ? "All " + total + " answered — ready to grade."
    : answered +
      " of " +
      total +
      " answered. Grade unlocks when every item has a choice.";
  el.prev.disabled = index <= 0;
  el.next.disabled = index >= total - 1;
}

function renderQuestion() {
  if (!pack) return;
  const item = pack.items[index];
  const total = pack.items.length;
  const n = index + 1;
  el.stem.textContent = item.stem;
  el.card.setAttribute("data-item-id", item.id);
  el.choices.innerHTML = "";
  el.choices.setAttribute("role", "radiogroup");
  el.choices.setAttribute("aria-labelledby", "q-stem");
  const selected = answers[item.id] || null;
  for (let i = 0; i < LETTERS.length; i++) {
    const letter = LETTERS[i];
    const text = item.choices[i] != null ? item.choices[i] : "";
    const inputId = "quiz-choice-" + letter;
    const label = document.createElement("label");
    label.className = "choice";
    if (selected === letter) label.classList.add("choice--selected");
    label.setAttribute("for", inputId);
    const input = document.createElement("input");
    input.type = "radio";
    input.name = "quiz-choice";
    input.id = inputId;
    input.value = letter;
    input.checked = selected === letter;
    const badge = document.createElement("span");
    badge.className = "choice__letter";
    badge.setAttribute("aria-hidden", "true");
    badge.textContent = letter;
    const body = document.createElement("span");
    body.className = "choice__text";
    body.textContent = text;
    label.appendChild(input);
    label.appendChild(badge);
    label.appendChild(body);
    el.choices.appendChild(label);
    input.addEventListener("change", onChoiceChange);
  }
  el.card.querySelector(".question-card__meta").textContent =
    "Module " +
    String(pack.module).padStart(2, "0") +
    " · Item " +
    n +
    " of " +
    total +
    " · " +
    item.id;
  updateChrome();
}

function onChoiceChange(ev) {
  const letter = ev.target.value;
  if (LETTERS.indexOf(letter) === -1) return;
  answers[pack.items[index].id] = letter;
  saveDraft();
  renderQuestion();
}

function selectLetter(letter) {
  if (LETTERS.indexOf(letter) === -1 || !pack) return;
  if (el.results && !el.results.hidden) return;
  answers[pack.items[index].id] = letter;
  saveDraft();
  renderQuestion();
}

function goTo(i) {
  if (!pack) return;
  if (i < 0 || i >= pack.items.length) return;
  index = i;
  saveDraft();
  renderQuestion();
  el.stem.setAttribute("tabindex", "-1");
  el.stem.focus({ preventScroll: false });
}

function setStatus(kind, text) {
  el.status.hidden = false;
  el.status.className =
    "exam-status" +
    (kind === "error"
      ? " exam-status--error"
      : kind === "ok"
        ? " exam-status--ok"
        : "");
  el.status.textContent = text;
}

function renderResults(presentation, digest, modeNote) {
  el.exam.hidden = true;
  el.picker.hidden = true;
  el.results.hidden = false;
  lastDigest = digest;
  window.__cdcp_quiz_last_digest = digest;
  el.scoreLine.textContent =
    presentation.score_correct + " / " + presentation.score_total;
  el.digestLine.textContent = digest || "— (no WASM digest; key-compare only)";
  el.modeLine.textContent = modeNote;
  const parts = ['<ol class="results-item-list">'];
  for (let i = 0; i < presentation.item_results.length; i++) {
    const r = presentation.item_results[i];
    const ok = r.is_correct;
    const cls = ok
      ? "results-item results-item--ok"
      : "results-item results-item--bad";
    const mark = ok ? "Correct" : "Incorrect";
    const markCls = ok
      ? "results-item__mark results-item__mark--ok"
      : "results-item__mark results-item__mark--bad";
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
        "</span></div>" +
        (r.stem
          ? '<p class="results-item__stem">' + escapeHtml(r.stem) + "</p>"
          : "") +
        '<p class="results-item__letters mono">chosen ' +
        escapeHtml(r.chosen) +
        " · correct " +
        escapeHtml(r.correct) +
        "</p>" +
        (r.explanation
          ? '<p class="results-item__expl">' +
            escapeHtml(r.explanation) +
            "</p>"
          : "") +
        "</li>"
    );
  }
  parts.push("</ol>");
  el.itemList.innerHTML = parts.join("");
}

async function onSubmit(ev) {
  ev.preventDefault();
  if (!pack || answeredCount() !== pack.items.length) {
    el.unanswered.focus();
    return;
  }
  setStatus("", "Grading module quiz…");
  const attempt = buildAttempt();
  const presentation = gradeByKeys(pack.items, answers);

  // Prefer WASM GradeExact (same law as mock). Fail open to key-compare for
  // pedagogy-only score — never claim cert; never invent a fake digest.
  let digest = null;
  gradeMode = "key-compare";
  try {
    await loadWasm(WASM_URL);
    if (isWasmReady()) {
      // Full bank JSON bytes required so bank_hash matches.
      const bankText = pack._bankJson;
      digest = await gradeDigest(bankText, JSON.stringify(attempt));
      if (typeof digest === "string" && /^[0-9a-f]{64}$/.test(digest)) {
        gradeMode = "wasm";
      } else {
        digest = null;
      }
    }
  } catch (err) {
    console.warn("quiz WASM grade failed; key-compare only:", errMsg(err));
    digest = null;
    gradeMode = "key-compare";
  }

  // Persist wrongs + SRS regardless of grade path (letter law is the same).
  recordGradedWrongs({
    source: "quiz",
    exam_id: pack.exam_id,
    seed: pack.seed,
    bank_hash: pack.bank_hash,
    item_results: presentation.item_results,
  });

  // Mastery state (L6-S2): practiced ≥80% / mastered 90%×2≥24h — study only.
  recordQuizResult({
    module: pack.module,
    correct: presentation.score_correct,
    total: presentation.score_total,
    atMs: Date.now(),
  });

  const modeNote =
    gradeMode === "wasm"
      ? "Graded via WASM (" +
        ENGINE_IDENTITY_SUBJECT +
        "). Same GradeExact letter law as mock. Study signal only — not a CDCP credential."
      : "WASM unavailable — pedagogy score via key-compare only (no GradeExact digest). " +
        "Study signal only — not a CDCP credential. Never claims certification.";

  renderResults(presentation, digest, modeNote);
  setStatus(
    "ok",
    gradeMode === "wasm"
      ? "Quiz graded (WASM). Missed items sent to Drill / SRS."
      : "Quiz scored (key-compare). Missed items sent to Drill / SRS."
  );
  try {
    sessionStorage.removeItem(STORAGE_DRAFT);
  } catch (_) {
    /* ignore */
  }
}

function onKeydown(ev) {
  if (ev.altKey || ev.ctrlKey || ev.metaKey) return;
  if (!pack || (el.results && !el.results.hidden)) return;
  const t = ev.target;
  if (
    t &&
    (t.tagName === "INPUT" || t.tagName === "TEXTAREA" || t.tagName === "SELECT")
  ) {
    if (t.type !== "radio") return;
  }
  const key = ev.key;
  if (key === "a" || key === "A" || key === "1") {
    ev.preventDefault();
    selectLetter("A");
    return;
  }
  if (key === "b" || key === "B" || key === "2") {
    ev.preventDefault();
    selectLetter("B");
    return;
  }
  if (key === "c" || key === "C" || key === "3") {
    ev.preventDefault();
    selectLetter("C");
    return;
  }
  if (key === "d" || key === "D" || key === "4") {
    ev.preventDefault();
    selectLetter("D");
    return;
  }
  if (key === "ArrowLeft" || key === "p" || key === "P") {
    ev.preventDefault();
    goTo(index - 1);
    return;
  }
  if (key === "ArrowRight" || key === "n" || key === "N") {
    ev.preventDefault();
    goTo(index + 1);
  }
}

function showExam() {
  el.picker.hidden = true;
  el.results.hidden = true;
  el.exam.hidden = false;
  renderQuestion();
}

async function startModule(moduleNum) {
  setStatus("", "Loading bank for module " + moduleNum + "…");
  el.results.hidden = true;
  try {
    const [bankRes, keysRes] = await Promise.all([
      fetch(BANK_URL, { cache: "no-store" }),
      fetch(KEYS_URL, { cache: "no-store" }),
    ]);
    if (!bankRes.ok) throw new Error("HTTP " + bankRes.status + " " + BANK_URL);
    if (!keysRes.ok) throw new Error("HTTP " + keysRes.status + " " + KEYS_URL);
    const bankText = await bankRes.text();
    const bankParsed = JSON.parse(bankText);
    const keysPack = await keysRes.json();
    const bankHash =
      (keysPack && keysPack.bank_hash) ||
      (Array.isArray(bankParsed) ? "" : bankParsed.bank_hash) ||
      "";
    const pool = filterByModule(bankParsed, moduleNum);
    if (pool.length === 0) {
      setStatus(
        "error",
        "No bank items for module " +
          moduleNum +
          ". Pick another module (BankItem.module field)."
      );
      return;
    }
    const countOpts = parseCountParams(
      typeof window !== "undefined" ? window.location.search : ""
    );
    const seed = 42 + moduleNum * 1000 + (countOpts.learn15 ? 15 : 0);
    const sampled = sampleItems(pool, seed, countOpts.min, countOpts.max);
    const items = learnerItems(sampled);

    // Restore draft if same module + same item set
    const draft = loadDraft();
    answers = Object.create(null);
    index = 0;
    if (
      draft &&
      draft.module === moduleNum &&
      Array.isArray(draft.item_ids) &&
      draft.item_ids.length === items.length &&
      draft.item_ids.every(function (id, i) {
        return id === items[i].id;
      })
    ) {
      if (draft.answers && typeof draft.answers === "object") {
        answers = draft.answers;
      }
      if (typeof draft.index === "number" && draft.index >= 0) {
        index = Math.min(draft.index, items.length - 1);
      }
    }

    pack = {
      module: moduleNum,
      bank_hash: bankHash,
      seed: seed,
      exam_id: "module-quiz-m" + String(moduleNum).padStart(2, "0"),
      items: items,
      _bankJson: bankText,
    };
    saveDraft();
    setStatus(
      "ok",
      "Module " +
        String(moduleNum).padStart(2, "0") +
        " quiz: " +
        items.length +
        " items (pool " +
        pool.length +
        "). Study only — not a credential."
    );
    showExam();
  } catch (err) {
    setStatus(
      "error",
      "Failed to load quiz data. Serve web/ over HTTP. " + errMsg(err)
    );
  }
}

function populatePicker(modulesPresent) {
  if (!el.moduleSelect) return;
  el.moduleSelect.innerHTML = "";
  const nums = modulesPresent.slice().sort(function (a, b) {
    return a - b;
  });
  for (let i = 0; i < nums.length; i++) {
    const m = nums[i];
    const opt = document.createElement("option");
    opt.value = String(m);
    opt.textContent = "Module " + String(m).padStart(2, "0");
    el.moduleSelect.appendChild(opt);
  }
  const pref = parseModuleParam();
  if (pref != null && nums.indexOf(pref) !== -1) {
    el.moduleSelect.value = String(pref);
  }
}

async function initPicker() {
  try {
    const res = await fetch(BANK_URL, { cache: "no-store" });
    if (!res.ok) throw new Error("HTTP " + res.status);
    const bank = await res.json();
    const arr = Array.isArray(bank) ? bank : bank.items || [];
    const set = Object.create(null);
    for (let i = 0; i < arr.length; i++) {
      if (arr[i] && typeof arr[i].module === "number") set[arr[i].module] = true;
    }
    const mods = Object.keys(set).map(Number);
    if (mods.length === 0) throw new Error("bank has no module fields");
    populatePicker(mods);
    el.picker.hidden = false;
    setStatus(
      "",
      "Pick a module and start. Quiz samples 8–12 items (or all if fewer)."
    );

    const pref = parseModuleParam();
    if (pref != null && mods.indexOf(pref) !== -1) {
      // Auto-start when deep-linked from learn pages.
      await startModule(pref);
    }
  } catch (err) {
    setStatus(
      "error",
      "Could not load bank for module list. Serve web/ over HTTP. " + errMsg(err)
    );
  }
}

function bind() {
  el.status = $("quiz-status");
  el.picker = $("quiz-picker");
  el.exam = $("quiz-exam");
  el.results = $("quiz-results");
  el.progress = $("quiz-progress");
  el.card = $("question-card");
  el.stem = $("q-stem");
  el.choices = $("q-choices");
  el.prev = $("btn-prev");
  el.next = $("btn-next");
  el.submit = $("btn-submit");
  el.unanswered = $("unanswered-hint");
  el.moduleSelect = $("module-select");
  el.startBtn = $("btn-start-quiz");
  el.scoreLine = $("quiz-score");
  el.digestLine = $("quiz-digest");
  el.modeLine = $("quiz-mode");
  el.itemList = $("quiz-item-list");

  el.prev.addEventListener("click", function () {
    goTo(index - 1);
  });
  el.next.addEventListener("click", function () {
    goTo(index + 1);
  });
  el.submit.addEventListener("click", onSubmit);
  el.startBtn.addEventListener("click", function () {
    const m = parseInt(el.moduleSelect.value, 10);
    if (!isFinite(m)) return;
    // Update URL without reload
    try {
      const u = new URL(window.location.href);
      u.searchParams.set("module", String(m));
      history.replaceState(null, "", u.pathname + u.search);
    } catch (_) {
      /* ignore */
    }
    startModule(m);
  });
  const again = $("btn-quiz-again");
  if (again) {
    again.addEventListener("click", function () {
      answers = Object.create(null);
      index = 0;
      if (pack) startModule(pack.module);
      else {
        el.results.hidden = true;
        el.picker.hidden = false;
        el.exam.hidden = true;
      }
    });
  }
  document.addEventListener("keydown", onKeydown);
}

function init() {
  bind();
  window.__cdcp_quiz_last_digest = null;
  initPicker();
}

// Console helpers
if (typeof window !== "undefined") {
  window.CdcpQuiz = {
    parseModuleParam,
    filterByModule,
    sampleItems,
    gradeByKeys,
    QUIZ_MIN,
    QUIZ_MAX,
    STORAGE_DRAFT,
  };
}

if (typeof document !== "undefined") {
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
}
