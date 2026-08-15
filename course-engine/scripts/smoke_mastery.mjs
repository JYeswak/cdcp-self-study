#!/usr/bin/env node
/**
 * Smoke for mastery state machine (L6-S2 / bd-iwp).
 *
 * Thresholds come from WASM (`cdcp_schedule`). This smoke loads wasm first.
 *
 * Usage (from course-engine/):
 *   node scripts/smoke_mastery.mjs
 */
import { existsSync } from "node:fs";
import { pathToFileURL } from "node:url";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, "..");

function findWasm() {
  const candidates = [
    join(ROOT, "web/assets/wasm/cdcp_wasm.wasm"),
    join(ROOT, "target/wasm32-unknown-unknown/release/cdcp_wasm.wasm"),
    join(ROOT, "target/wasm32-unknown-unknown/debug/cdcp_wasm.wasm"),
  ];
  for (const p of candidates) {
    if (existsSync(p)) return p;
  }
  throw new Error(
    "no cdcp_wasm.wasm — cargo build -p cdcp_wasm --target wasm32-unknown-unknown"
  );
}

const { loadWasm } = await import(
  pathToFileURL(join(ROOT, "web/assets/js/grade_bridge.js")).href
);
await loadWasm(findWasm());

const masteryPath = pathToFileURL(join(ROOT, "web/assets/js/mastery.js")).href;

const {
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
  getState,
  recordQuizResult,
  bestRatio,
  isPracticed,
  isMastered,
  listPracticed,
  listMastered,
} = await import(masteryPath);

const DAY_MS = dayMs();
const PRACTICED_RATIO = practicedRatio();
const MASTERED_RATIO = masteredRatio();
const MASTERED_MIN_GAP_MS = masteredMinGapMs();

let failed = 0;

function assert(cond, msg) {
  if (!cond) {
    console.error("FAIL:", msg);
    failed += 1;
  } else {
    console.log("ok:", msg);
  }
}

// --- in-memory Storage mock ---
function makeStore() {
  /** @type {Map<string,string>} */
  const m = new Map();
  return {
    getItem(k) {
      return m.has(k) ? m.get(k) : null;
    },
    setItem(k, v) {
      m.set(k, String(v));
    },
    removeItem(k) {
      m.delete(k);
    },
    clear() {
      m.clear();
    },
    key(i) {
      return Array.from(m.keys())[i] ?? null;
    },
    get length() {
      return m.size;
    },
  };
}

// --- constants / pure helpers ---
assert(STORAGE_KEY === "cdcp.mastery.v1", "STORAGE_KEY = cdcp.mastery.v1");
assert(typeof SCHEMA_VERSION === "number", "SCHEMA_VERSION is number");
assert(DAY_MS === 24 * 60 * 60 * 1000, "DAY_MS = 86400000");
assert(PRACTICED_RATIO === 0.8, "PRACTICED_RATIO = 0.80");
assert(MASTERED_RATIO === 0.9, "MASTERED_RATIO = 0.90");
assert(MASTERED_MIN_GAP_MS === DAY_MS, "MASTERED_MIN_GAP_MS = 24h");
assert(moduleKey(6) === "6", "moduleKey(6)");
assert(moduleKey("06") === "06", "moduleKey string");
assert(moduleKey(null) === null, "moduleKey(null)");
assert(ratioOf(8, 10) === 0.8, "ratioOf 8/10 = 0.8");
assert(ratioOf(9, 10) === 0.9, "ratioOf 9/10 = 0.9");
assert(ratioOf(0, 0) === 0, "ratioOf 0/0 = 0");
assert(ratioOf(11, 10) === 1, "ratioOf clamps >1");

// normalize rejects garbage
const empty = normalizeState({ modules: "nope" });
assert(Object.keys(empty.modules).length === 0, "normalize rejects bad modules");
assert(normalizeState(null).schema_version === SCHEMA_VERSION, "normalize null");

// --- acceptance cases ---
const t0 = 1_700_000_000_000;
const store = makeStore();

// Fresh module: neither practiced nor mastered
assert(isPracticed(1, { store }) === false, "empty → not practiced");
assert(isMastered(1, { store }) === false, "empty → not mastered");
assert(listPracticed({ store }).length === 0, "listPracticed empty");
assert(listMastered({ store }).length === 0, "listMastered empty");

// <80% → not practiced
// 7/10 = 0.70
recordQuizResult(
  { module: 1, correct: 7, total: 10, atMs: t0 },
  { store }
);
assert(bestRatio(1, { store }) === 0.7, "best ratio 0.70");
assert(isPracticed(1, { store }) === false, "<80% → not practiced");
assert(isMastered(1, { store }) === false, "<80% → not mastered");
assert(listPracticed({ store }).indexOf(1) === -1, "listPracticed excludes <80%");

// ≥80% → practiced (exact 80%)
recordQuizResult(
  { module: 2, correct: 8, total: 10, atMs: t0 },
  { store }
);
assert(bestRatio(2, { store }) === 0.8, "best ratio 0.80");
assert(isPracticed(2, { store }) === true, "≥80% → practiced");
assert(isMastered(2, { store }) === false, "80% only → not mastered");
assert(listPracticed({ store }).indexOf(2) !== -1, "listPracticed includes 80%");

// Higher best wins practiced
recordQuizResult(
  { module: 1, correct: 9, total: 10, atMs: t0 + 1000 },
  { store }
);
assert(bestRatio(1, { store }) === 0.9, "best updates to 0.90");
assert(isPracticed(1, { store }) === true, "after 90% attempt → practiced");

// single 90% → not mastered
const storeB = makeStore();
recordQuizResult(
  { module: 3, correct: 9, total: 10, atMs: t0 },
  { store: storeB }
);
assert(isPracticed(3, { store: storeB }) === true, "single 90% → practiced");
assert(isMastered(3, { store: storeB }) === false, "single 90% → not mastered");
assert(
  listMastered({ store: storeB }).indexOf(3) === -1,
  "listMastered excludes single 90%"
);

// two 90% same day (<24h) → not mastered
recordQuizResult(
  { module: 3, correct: 10, total: 10, atMs: t0 + 12 * 60 * 60 * 1000 },
  { store: storeB }
);
assert(
  isMastered(3, { store: storeB }) === false,
  "two 90% same day (<24h) → not mastered"
);
assert(
  listMastered({ store: storeB }).indexOf(3) === -1,
  "listMastered excludes <24h pair"
);

// two 90% ≥24h apart → mastered
const storeC = makeStore();
recordQuizResult(
  { module: 4, correct: 9, total: 10, atMs: t0 },
  { store: storeC }
);
recordQuizResult(
  { module: 4, correct: 9, total: 10, atMs: t0 + DAY_MS },
  { store: storeC }
);
assert(isPracticed(4, { store: storeC }) === true, "≥24h pair → practiced");
assert(isMastered(4, { store: storeC }) === true, "two 90% ≥24h apart → mastered");
assert(
  listMastered({ store: storeC }).indexOf(4) !== -1,
  "listMastered includes spaced pair"
);

// Exactly 24h boundary
const storeD = makeStore();
recordQuizResult(
  { module: 5, correct: 9, total: 10, atMs: t0 },
  { store: storeD }
);
recordQuizResult(
  { module: 5, correct: 9, total: 10, atMs: t0 + MASTERED_MIN_GAP_MS },
  { store: storeD }
);
assert(
  isMastered(5, { store: storeD }) === true,
  "exactly 24h gap → mastered"
);

// Just under 24h
const storeE = makeStore();
recordQuizResult(
  { module: 6, correct: 9, total: 10, atMs: t0 },
  { store: storeE }
);
recordQuizResult(
  { module: 6, correct: 9, total: 10, atMs: t0 + DAY_MS - 1 },
  { store: storeE }
);
assert(
  isMastered(6, { store: storeE }) === false,
  "24h - 1ms → not mastered"
);

// Two high scores with a low score between them still masters if spaced
const storeF = makeStore();
recordQuizResult(
  { module: 7, correct: 9, total: 10, atMs: t0 },
  { store: storeF }
);
recordQuizResult(
  { module: 7, correct: 5, total: 10, atMs: t0 + 1000 },
  { store: storeF }
);
recordQuizResult(
  { module: 7, correct: 10, total: 10, atMs: t0 + DAY_MS + 1000 },
  { store: storeF }
);
assert(
  isMastered(7, { store: storeF }) === true,
  "qualifying pair with intervening low score → mastered"
);

// Two 89% never master (below 0.90)
const storeG = makeStore();
recordQuizResult(
  { module: 8, correct: 89, total: 100, atMs: t0 },
  { store: storeG }
);
recordQuizResult(
  { module: 8, correct: 89, total: 100, atMs: t0 + DAY_MS },
  { store: storeG }
);
assert(isPracticed(8, { store: storeG }) === true, "89% → practiced");
assert(
  isMastered(8, { store: storeG }) === false,
  "two 89% spaced → not mastered"
);

// Persistence round-trip via getState / loadState
const snap = getState({ store: storeC });
assert(snap.modules["4"] && snap.modules["4"].attempts.length === 2, "getState has 2 attempts");
assert(loadState(storeC).modules["4"].best_ratio === 0.9, "loadState best_ratio");

// No "certified" / credential language in exported API surface
const apiNames = [
  "STORAGE_KEY",
  "isPracticed",
  "isMastered",
  "listPracticed",
  "listMastered",
  "recordQuizResult",
  "getState",
].join(" ");
assert(
  !/certif/i.test(apiNames),
  "API names contain no 'certif*' language"
);

// recordQuizResult ignores bad module
assert(
  recordQuizResult({ module: null, correct: 10, total: 10 }, { store }) === null,
  "null module → null"
);

// listPracticed / listMastered sorting with multiple modules
const storeH = makeStore();
recordQuizResult(
  { module: 12, correct: 9, total: 10, atMs: t0 },
  { store: storeH }
);
recordQuizResult(
  { module: 12, correct: 9, total: 10, atMs: t0 + DAY_MS },
  { store: storeH }
);
recordQuizResult(
  { module: 3, correct: 8, total: 10, atMs: t0 },
  { store: storeH }
);
const practiced = listPracticed({ store: storeH });
const mastered = listMastered({ store: storeH });
assert(
  practiced[0] === 3 && practiced[1] === 12,
  "listPracticed sorted: " + practiced.join(",")
);
assert(
  mastered.length === 1 && mastered[0] === 12,
  "listMastered only 12: " + mastered.join(",")
);

if (failed > 0) {
  console.error("\nsmoke_mastery: " + failed + " failure(s)");
  process.exit(1);
}
console.log("\nsmoke_mastery: all checks passed");
process.exit(0);
