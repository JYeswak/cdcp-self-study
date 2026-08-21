#!/usr/bin/env node
/**
 * Rendered-output contract smoke for Q22 / bd-std-rendered-output-5moj.
 *
 * This deliberately exercises the learner JavaScript entry points, not copies
 * of their formatting logic.  The small DOM below is only a DOM adapter for
 * Node: the strings are produced by web/assets/js/{mock,results,quiz}.js and
 * results.js grades through the shipped WASM bridge.
 *
 * Inventory denominator: 43 named presentation sites.  The inventory is
 * grouped by renderer, not by item instance, and is kept explicit so deleting
 * an assertion is an anti-vacuous failure.
 */

const { readFile, mkdtemp, rm, writeFile } = require("node:fs/promises");
const { existsSync } = require("node:fs");
const { tmpdir } = require("node:os");
const { dirname, join, resolve } = require("node:path");
const { pathToFileURL } = require("node:url");

const ROOT = resolve(dirname(__filename), "../..", "..");
const INVENTORY = [
  ["mock.exam-form-meta", "web/mock.html", "40Q · 60:00 · study bar 27/40"],
  ["mock.seed-menu", "web/mock.html", "seed-select"],
  ["mock.progress", "web/assets/js/mock.js", 'n + " / " + total'],
  ["mock.timer", "web/assets/js/mock.js", 'formatTime(rem)'],
  ["mock.submit-label", "web/mock.html", "Submit attempt"],
  ["mock.unanswered", "web/assets/js/mock.js", 'answered + " of " + total'],
  ["mock.question-meta", "web/assets/js/mock.js", '"Item " + n + " of " + total'],
  ["mock.option-letters", "web/assets/js/mock.js", 'badge.textContent = letter'],
  ["mock.jump-labels", "web/assets/js/mock.js", 'btn.textContent = String(i + 1)'],
  ["mock.pack-identity", "web/assets/js/mock.js", 'mock40_seed" + activeSeed'],
  ["results.exam", "web/assets/js/results.js", '$("r-exam").textContent'],
  ["results.seed", "web/assets/js/results.js", '$("r-seed").textContent'],
  ["results.bank-hash", "web/assets/js/results.js", '$("r-hash").textContent'],
  ["results.answer-count", "web/assets/js/results.js", '$("r-count").textContent'],
  ["results.score", "web/assets/js/results.js", 'scoreEl.textContent'],
  ["results.digest", "web/assets/js/results.js", 'digestEl.textContent'],
  ["results.engine", "web/assets/js/results.js", 'engineEl.textContent'],
  ["results.study-signal", "web/assets/js/results.js", "correct meets the practice bar"],
  ["results.weak-module-heading", "web/assets/js/results.js", "Weak modules"],
  ["results.weak-module-chip", "web/assets/js/results.js", 'String(m).padStart(2, "0")'],
  ["results.item-status", "web/assets/js/results.js", 'const mark = ok ? "Correct"'],
  ["results.item-letters", "web/assets/js/results.js", "chosen "],
  ["results.learn-link", "web/assets/js/results.js", "Review section in Learn"],
  ["quiz.module-picker", "web/assets/js/quiz.js", '"Module " + String(m).padStart'],
  ["quiz.progress", "web/assets/js/quiz.js", 'el.progress.textContent = n + " / " + total'],
  ["quiz.status", "web/assets/js/quiz.js", '" quiz: "'],
  ["quiz.question-meta", "web/assets/js/quiz.js", " · Item "],
  ["quiz.option-letters", "web/assets/js/quiz.js", 'badge.textContent = letter'],
  ["quiz.unanswered", "web/assets/js/quiz.js", "Grade unlocks when every item"],
  ["quiz.score", "web/assets/js/quiz.js", 'el.scoreLine.textContent ='],
  ["quiz.digest", "web/assets/js/quiz.js", 'el.digestLine.textContent = digest'],
  ["quiz.mode", "web/assets/js/quiz.js", 'el.modeLine.textContent = modeNote'],
  ["quiz.item-review", "web/assets/js/quiz.js", 'results-item-list'],
  ["learn.unit-status", "web/assets/js/learn_units.js", '"Unit " +'],
  ["learn.here-bar", "web/assets/js/learn_units.js", '"You are here · unit "'],
  ["learn.quick-check", "web/assets/js/learn_units.js", "Quick check (study only)"],
  ["learn.check-completion", "web/assets/js/learn_units.js", "Check complete · "],
  ["learn.visited-summary", "web/assets/js/learn_progress.js", '"Visited " + done'],
  ["drill.mode-heading", "web/assets/js/drill.js", '"Drill / short-interval review"'],
  ["drill.missed-count", "web/assets/js/drill.js", 'missed.item_ids.length +'],
  ["drill.item-module", "web/assets/js/drill.js", 'String(it.module).padStart'],
  ["drill.correct-label", "web/assets/js/drill.js", "correct "],
  ["hub.module-row", "web/assets/js/hub_mastery.js", 'orderLabel'],
  ["hub.badges", "web/assets/js/hub_mastery.js", "badgeHtml(badges)"],
  ["hub.recommendation", "web/assets/js/hub_mastery.js", "Next up"],
];

class Storage {
  constructor() { this.m = new Map(); }
  getItem(k) { return this.m.has(k) ? this.m.get(k) : null; }
  setItem(k, v) { this.m.set(String(k), String(v)); }
  removeItem(k) { this.m.delete(String(k)); }
  clear() { this.m.clear(); }
}

class ClassList {
  constructor(owner) { this.owner = owner; this.s = new Set(); }
  add(...xs) { xs.forEach((x) => this.s.add(x)); this.sync(); }
  remove(...xs) { xs.forEach((x) => this.s.delete(x)); this.sync(); }
  toggle(x, force) {
    const on = force === undefined ? !this.s.has(x) : !!force;
    if (on) this.s.add(x); else this.s.delete(x);
    this.sync(); return on;
  }
  contains(x) { return this.s.has(x) || this.owner.className.split(/\s+/).includes(x); }
  sync() { this.owner.className = [...this.s].join(" "); }
}

class Element {
  constructor(tag = "div", id = "", className = "") {
    this.tagName = tag.toUpperCase(); this.id = id; this.className = className;
    this.classList = new ClassList(this);
    for (const c of className.split(/\s+/).filter(Boolean)) this.classList.s.add(c);
    this.children = []; this.parentNode = null; this.attributes = new Map();
    this.hidden = false; this.disabled = false; this.value = ""; this.checked = false;
    this.type = ""; this.textContent = ""; this._innerHTML = ""; this.dataset = {};
  }
  set innerHTML(v) { this._innerHTML = String(v); this.children = []; }
  get innerHTML() { return this._innerHTML; }
  appendChild(c) { c.parentNode = this; this.children.push(c); return c; }
  insertBefore(c, before) {
    c.parentNode = this; const i = this.children.indexOf(before);
    if (i < 0) this.children.push(c); else this.children.splice(i, 0, c); return c;
  }
  addEventListener() {}
  focus() {}
  setAttribute(k, v) {
    this.attributes.set(String(k), String(v));
    if (k === "id") this.id = String(v);
    if (k === "class") { this.className = String(v); this.classList = new ClassList(this); }
    if (String(k).startsWith("data-")) this.dataset[String(k).slice(5).replace(/-([a-z])/g, (_, c) => c.toUpperCase())] = String(v);
  }
  getAttribute(k) { return this.attributes.has(k) ? this.attributes.get(k) : null; }
  querySelector(sel) { return this.querySelectorAll(sel)[0] || null; }
  querySelectorAll(sel) {
    const out = []; const match = (e) => {
      if (sel === "*") return true;
      if (sel.startsWith("#")) return e.id === sel.slice(1);
      if (sel.startsWith(".")) return e.classList.contains(sel.slice(1));
      const data = sel.match(/^\[([^=\]]+)(?:="([^"]*)")?\]$/);
      if (data) return e.attributes.has(data[1]) && (data[2] === undefined || e.attributes.get(data[1]) === data[2]);
      const cls = sel.match(/^\.([^\[]+)\[([^=]+)(?:="([^"]*)")?\]$/);
      if (cls) return e.classList.contains(cls[1]) && e.attributes.has(cls[2]) && (cls[3] === undefined || e.attributes.get(cls[2]) === cls[3]);
      return e.tagName.toLowerCase() === sel.toLowerCase();
    };
    const walk = (e) => { for (const c of e.children) { if (match(c)) out.push(c); walk(c); } };
    walk(this); return out;
  }
  get options() { return this.children; }
}

class Document extends Element {
  constructor() { super("document"); this.readyState = "complete"; this.body = new Element("body"); this.appendChild(this.body); }
  getElementById(id) { return this.querySelectorAll("#" + id)[0] || null; }
  createElement(tag) { return new Element(tag); }
}

function add(doc, id, tag = "div", cls = "", parent = doc.body) {
  const e = new Element(tag, id, cls); e.setAttribute("id", id); parent.appendChild(e); return e;
}

function mockDocument(kind) {
  const d = new Document();
  if (kind === "mock") {
    for (const id of ["exam-status", "exam-progress", "exam-timer", "q-stem", "q-choices", "btn-prev", "btn-next", "jump-strip", "btn-submit", "unanswered-hint", "seed-select", "pack-meta", "closed-notes-toggle", "closed-notes-hint"]) add(d, id, id === "seed-select" ? "select" : "div");
    const card = add(d, "question-card", "section"); card.appendChild(new Element("div", "", "question-card__meta"));
  } else if (kind === "quiz") {
    for (const id of ["quiz-status", "quiz-picker", "quiz-exam", "quiz-results", "quiz-progress", "q-stem", "q-choices", "btn-prev", "btn-next", "btn-submit", "unanswered-hint", "module-select", "btn-start-quiz", "quiz-score", "quiz-digest", "quiz-mode", "quiz-item-list", "btn-quiz-again"]) add(d, id, id === "module-select" ? "select" : "div");
    const card = d.getElementById("question-card") || add(d, "question-card", "section"); card.appendChild(new Element("div", "", "question-card__meta"));
  } else {
    for (const id of ["results-status", "results-summary", "results-score", "results-study-signal", "results-weak", "results-items", "r-exam", "r-seed", "r-hash", "r-count", "r-score", "r-digest", "r-engine", "results-drill-link"]) add(d, id);
  }
  return d;
}

function makeWindow(search = "") {
  return { location: { href: "http://127.0.0.1:8766/", origin: "http://127.0.0.1:8766", pathname: "/mock.html", search }, addEventListener() {}, confirm: () => true };
}

function installGlobals(doc, win, store) {
  globalThis.document = doc; globalThis.window = win; win.document = doc;
  globalThis.location = win.location; globalThis.sessionStorage = store; globalThis.localStorage = store;
  globalThis.confirm = () => true;
  globalThis.setInterval = () => 1; globalThis.clearInterval = () => {};
}

function installFetch(root) {
  globalThis.fetch = async (url) => {
    const raw = String(url);
    const rel = raw.replace(/^https?:\/\/[^/]+\//, "");
    const p = join(root, "web", rel);
    if (!existsSync(p)) return { ok: false, status: 404, text: async () => "", json: async () => { throw new Error("404 " + rel); }, arrayBuffer: async () => new ArrayBuffer(0) };
    const bytes = await readFile(p);
    return { ok: true, status: 200, text: async () => bytes.toString("utf8"), json: async () => JSON.parse(bytes.toString("utf8")), arrayBuffer: async () => bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength) };
  };
}

function wait(ms = 20) { return new Promise((r) => setTimeout(r, ms)); }
function check(condition, message) { if (!condition) throw new Error(message); }

async function inventoryCheck(root, deletedId = null) {
  const assertions = new Set();
  for (const [id, rel, marker] of INVENTORY) {
    if (id === deletedId) continue;
    const text = await readFile(join(root, rel), "utf8");
    check(text.includes(marker), `${id}: source marker missing (${rel}: ${marker})`);
    assertions.add(id);
  }
  check(assertions.size === INVENTORY.length, `rendered-output inventory incomplete: ${assertions.size}/${INVENTORY.length} assertions`);
  return assertions.size;
}

async function runMock(root, store, sourceOverride = null) {
  const doc = mockDocument("mock"); const win = makeWindow("?seed=42"); installGlobals(doc, win, store); installFetch(root);
  const source = sourceOverride || join(root, "web/assets/js/mock.js");
  await import(pathToFileURL(source).href + `?render=${Date.now()}-${Math.random()}`); await wait();
  const progress = doc.getElementById("exam-progress").textContent;
  const meta = doc.querySelector(".question-card__meta").textContent;
  const letters = doc.getElementById("q-choices").querySelectorAll(".choice__letter").map((e) => e.textContent).join("");
  check(progress === "1 / 40", `mock progress rendered ${JSON.stringify(progress)}`);
  check(/^Item 1 of 40 · /.test(meta), `mock question meta rendered ${JSON.stringify(meta)}`);
  check(letters === "ABCD", `mock option labels rendered ${JSON.stringify(letters)}`);
  check(doc.getElementById("exam-timer").textContent === "60:00", "mock timer did not render 60:00");
  return { progress, meta, letters, timer: doc.getElementById("exam-timer").textContent };
}

async function runResults(root, store, wrong) {
  const keys = JSON.parse(await readFile(join(root, "web/data/keys_seed42.json"), "utf8"));
  const answers = keys.keys.map((k) => ({ item_id: k.item_id, chosen: wrong ? ({ A: "B", B: "C", C: "D", D: "A" })[k.correct] : k.correct }));
  store.setItem("cdcp_mock_attempt_v1", JSON.stringify({ exam_id: "mock40", seed: 42, bank_hash: keys.bank_hash, answers }));
  const doc = mockDocument("results"); const win = makeWindow(); win.location.pathname = "/results.html"; installGlobals(doc, win, store); installFetch(root);
  const p = join(root, "web/assets/js/results.js");
  await import(pathToFileURL(p).href + `?render=${wrong ? "wrong" : "correct"}-${Date.now()}`); await wait(40);
  const score = doc.getElementById("r-score").textContent;
  const study = doc.getElementById("results-study-signal").innerHTML;
  const weak = doc.getElementById("results-weak").innerHTML;
  const items = doc.getElementById("results-items").innerHTML;
  const engine = doc.getElementById("r-engine").textContent;
  check(score === (wrong ? "0 / 40" : "40 / 40"), `results score rendered ${JSON.stringify(score)}`);
  check(study.includes(wrong ? "0 / 40 is below the practice bar of 27" : "40 / 40 correct meets the practice bar of 27"), "results study signal lost denominator or bar");
  check(weak.includes("Weak modules"), "results weak-module heading missing");
  check(items.includes(wrong ? "Incorrect" : "Correct") && items.includes("chosen "), "results item review missing status/letters");
  check(engine === "cdcp_wasm-wasm32", `results engine rendered ${JSON.stringify(engine)}`);
  return { score, study: study.replace(/<[^>]+>/g, ""), engine };
}

async function runQuiz(root, store) {
  const doc = mockDocument("quiz"); const win = makeWindow("?module=6&count=8"); win.location.pathname = "/quiz.html"; installGlobals(doc, win, store); installFetch(root);
  const p = join(root, "web/assets/js/quiz.js");
  await import(pathToFileURL(p).href + `?render=${Date.now()}-${Math.random()}`); await wait(50);
  const progress = doc.getElementById("quiz-progress").textContent;
  const status = doc.getElementById("quiz-status").textContent;
  const meta = doc.querySelector(".question-card__meta").textContent;
  const letters = doc.getElementById("q-choices").querySelectorAll(".choice__letter").map((e) => e.textContent).join("");
  check(/^1 \/ 8$/.test(progress), `quiz progress rendered ${JSON.stringify(progress)}`);
  check(status.includes("Module 06 quiz: 8 items") && status.includes("Study only — not a credential"), "quiz status lost count/honesty label");
  check(/^Module 06 · Item 1 of 8 · /.test(meta), `quiz question meta rendered ${JSON.stringify(meta)}`);
  check(letters === "ABCD", `quiz option labels rendered ${JSON.stringify(letters)}`);
  return { progress, status, meta, letters };
}

async function runKnownBad(root, store) {
  const dir = await mkdtemp(join(tmpdir(), "cdcp-rendered-output-"));
  const source = join(dir, "mock-bad.js");
  let js = await readFile(join(root, "web/assets/js/mock.js"), "utf8");
  js = js.replace("badge.textContent = letter;", 'badge.textContent = letter === "A" ? "B" : letter;');
  check(js !== await readFile(join(root, "web/assets/js/mock.js"), "utf8"), "known-bad mutation did not change named component");
  await writeFile(source, js);
  let caught = false;
  try { await runMock(root, store, source); } catch (e) { caught = true; console.log("known-bad option-letter: exit=2; only mock.option-letters RED:", e.message); }
  await rm(dir, { recursive: true, force: true });
  check(caught, "known-bad option-letter mutation did not turn exact assertion RED");
}

async function main() {
  const args = new Set(process.argv.slice(2));
  const deleted = args.has("--delete-assertion") ? "results.score" : null;
  const count = await inventoryCheck(ROOT, deleted);
  console.log(`rendered-output inventory: ${INVENTORY.length} named sites`);
  if (deleted) { console.log("known-bad assertion deletion: exit=2 (anti-vacuous inventory count)"); return; }
  const store = new Storage();
  const mock = await runMock(ROOT, store);
  const correct = await runResults(ROOT, store, false);
  const wrong = await runResults(ROOT, store, true);
  const quiz = await runQuiz(ROOT, store);
  await runKnownBad(ROOT, store);
  console.log("known-good: exit=0; mock/results/quiz production renderers and WASM path passed");
  console.log("mock:", JSON.stringify(mock));
  console.log("results all-correct:", JSON.stringify(correct));
  console.log("results all-wrong:", JSON.stringify(wrong));
  console.log("quiz:", JSON.stringify(quiz));
  console.log("rendered-output limitation: DOM strings and WASM are covered; CSS/layout/pixel rendering and unenumerated sites require a real browser review");
}

main().catch((err) => { console.error("smoke_rendered_output: ERROR:", err.stack || err.message || err); process.exitCode = 2; });
