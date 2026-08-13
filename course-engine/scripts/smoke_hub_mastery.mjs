#!/usr/bin/env node
/**
 * Pure-function smoke for hub mastery dashboard + recommend (L6-S4 / bd-qyi).
 *
 * Usage (from course-engine/):
 *   node scripts/smoke_hub_mastery.mjs
 *
 * Exit 0 only if:
 *   - recordQuizResult module 3 @ 80%+ → practiced badge state true
 *   - recommend prefers weak → unpracticed → unmastered → all_practiced
 *   - every recommend href maps to an existing learn/*.html or quiz.html pattern
 *   - saveLastWeak writes cdcp.last_weak.v1
 *   - no cert language in recommend copy
 *
 * No browser required.
 */
import { existsSync } from "node:fs";
import { pathToFileURL } from "node:url";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, "..");
const WEB = join(ROOT, "web");

const masteryPath = pathToFileURL(join(WEB, "assets/js/mastery.js")).href;
const hubPath = pathToFileURL(join(WEB, "assets/js/hub_mastery.js")).href;

const { recordQuizResult, isPracticed, DAY_MS } = await import(masteryPath);
const {
  WEAK_STORAGE_KEY,
  MODULE_CATALOG,
  saveLastWeak,
  loadLastWeak,
  recommendNext,
  moduleBadgeState,
  badgeHtml,
} = await import(hubPath);

let failed = 0;

function assert(cond, msg) {
  if (!cond) {
    console.error("FAIL:", msg);
    failed += 1;
  } else {
    console.log("ok:", msg);
  }
}

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

// --- catalog integrity ---
assert(MODULE_CATALOG.length === 14, "MODULE_CATALOG has 14 modules");
for (let i = 0; i < MODULE_CATALOG.length; i++) {
  const e = MODULE_CATALOG[i];
  assert(e.order === i + 1, "order " + e.order + " sequential");
  const learnPath = join(WEB, e.learnHref);
  assert(existsSync(learnPath), "learn page exists: " + e.learnHref);
  assert(
    /^quiz\.html\?module=\d+$/.test(e.quizHref),
    "quiz href shape: " + e.quizHref
  );
}

// --- weak storage ---
{
  const store = makeStore();
  const saved = saveLastWeak([6, 3, 99, 3], { store, atMs: 1000, source: "mock" });
  assert(WEAK_STORAGE_KEY === "cdcp.last_weak.v1", "WEAK_STORAGE_KEY");
  assert(
    JSON.stringify(saved.weak_modules) === JSON.stringify([3, 6]),
    "weak modules sorted+deduped+clamped to 1–14"
  );
  const loaded = loadLastWeak({ store });
  assert(
    JSON.stringify(loaded.weak_modules) === JSON.stringify([3, 6]),
    "loadLastWeak round-trips"
  );
}

// --- practiced after recordQuizResult module 3 @ 80%+ ---
{
  const store = makeStore();
  recordQuizResult(
    { module: 3, correct: 8, total: 10, atMs: 1_000_000 },
    { store }
  );
  assert(isPracticed(3, { store }), "module 3 practiced after 80% quiz");
  const badges = moduleBadgeState(3, { store });
  assert(badges.practiced === true, "moduleBadgeState practiced for 3");
  assert(badges.mastered === false, "module 3 not mastered from single attempt");
  const html = badgeHtml(badges);
  assert(/Practiced/.test(html), "badgeHtml shows Practiced");
  assert(!/certif/i.test(html), "badgeHtml has no cert language");
}

// --- recommend priority: weak > unpracticed > unmastered > all_practiced ---
{
  const store = makeStore();
  // Empty mastery + weak [9] → recommend weak learn link
  saveLastWeak([9], { store, source: "mock" });
  let rec = recommendNext({ store });
  assert(rec.kind === "weak", "recommend kind=weak when last_weak set");
  assert(rec.module === 9, "recommend first weak module 9");
  assert(rec.href === "learn/09-cooling.html", "weak → learn href");
  assert(existsSync(join(WEB, rec.href)), "weak recommend href exists on disk");
  assert(!/certif/i.test(rec.reason + rec.label), "weak reason no cert language");

  // Clear weak; nothing practiced → first unpracticed = 1
  saveLastWeak([], { store });
  rec = recommendNext({ store });
  assert(rec.kind === "unpracticed", "recommend unpracticed when no weak");
  assert(rec.module === 1, "first unpracticed is module 1");
  assert(rec.href === "quiz.html?module=1", "unpracticed → quiz href");

  // Practice all 1–14 once at 80% → unmastered (need 90%×2 spaced)
  for (let m = 1; m <= 14; m++) {
    recordQuizResult(
      { module: m, correct: 8, total: 10, atMs: 2_000_000 + m },
      { store }
    );
  }
  rec = recommendNext({ store });
  assert(rec.kind === "unmastered", "recommend unmastered when all practiced");
  assert(rec.module === 1, "first unmastered is module 1");
  assert(/^quiz\.html\?module=\d+$/.test(rec.href), "unmastered → quiz");

  // Master all 1–14 (two 90%+ attempts ≥24h apart)
  const t0 = 10_000_000;
  for (let m = 1; m <= 14; m++) {
    recordQuizResult(
      { module: m, correct: 9, total: 10, atMs: t0 + m },
      { store }
    );
    recordQuizResult(
      { module: m, correct: 10, total: 10, atMs: t0 + DAY_MS + m },
      { store }
    );
  }
  rec = recommendNext({ store });
  assert(rec.kind === "all_practiced", "all_practiced when every module mastered");
  assert(rec.href === null, "all_practiced has no required href");
  assert(
    !/certif/i.test(rec.reason) || /not a credential/i.test(rec.reason),
    "all_practiced denies credential claim"
  );
  // Soft: must not claim certified
  assert(!/\bcertified\b/i.test(rec.reason), "no 'certified' claim in reason");
}

// --- recommend never 404: every catalog learn path exists ---
{
  for (let i = 0; i < MODULE_CATALOG.length; i++) {
    const e = MODULE_CATALOG[i];
    const store = makeStore();
    saveLastWeak([e.order], { store });
    const rec = recommendNext({ store });
    assert(rec.href === e.learnHref, "weak M" + e.order + " href matches catalog");
    assert(existsSync(join(WEB, rec.href)), "no-404: " + rec.href);
  }
}

if (failed > 0) {
  console.error("\nsmoke_hub_mastery: " + failed + " failure(s)");
  process.exit(1);
}
console.log("\nsmoke_hub_mastery: PASS");
process.exit(0);
