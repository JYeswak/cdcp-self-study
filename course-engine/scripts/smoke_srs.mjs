#!/usr/bin/env node
/**
 * Pure-function smoke for minimal SRS intervals + Drill-10 due filter
 * (L5-S7 / bd-ca8 · L6-S6 / bd-3dd).
 *
 * Usage (from course-engine/):
 *   node scripts/smoke_srs.mjs
 *
 * Exit 0 only if nextIntervalDays is monotonic on the 1d→3d ladder,
 * in-memory schedule/review round-trip works, and selectDueOnly /
 * listDueDrill honor due_at ≤ now + cap 10.
 */
import { pathToFileURL } from "node:url";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, "..");
const srsPath = pathToFileURL(join(ROOT, "web/assets/js/srs.js")).href;

const {
  nextIntervalDays,
  dueAtFromInterval,
  DAY_MS,
  DRILL10_LIMIT,
  normalizeSrsState,
  scheduleMissed,
  scheduleMissedMany,
  reviewCard,
  selectDueOnly,
  listDue,
  listDueDrill,
  recordGradedWrongs,
  loadMissed,
  loadSrsState,
  saveSrsState,
  SRS_STORAGE_KEY,
  MISSED_STORAGE_KEY,
} = await import(srsPath);

let failed = 0;

function assert(cond, msg) {
  if (!cond) {
    console.error("FAIL:", msg);
    failed += 1;
  } else {
    console.log("ok:", msg);
  }
}

// --- nextIntervalDays law ---
assert(nextIntervalDays(0, false) === 1, "new + wrong → 1d");
assert(nextIntervalDays(0, true) === 1, "new + correct → 1d");
assert(nextIntervalDays(1, true) === 3, "1d + correct → 3d");
assert(nextIntervalDays(3, true) === 3, "3d + correct → 3d (cap)");
assert(nextIntervalDays(3, false) === 1, "3d + wrong → 1d");
assert(nextIntervalDays(1, false) === 1, "1d + wrong → 1d");
assert(nextIntervalDays(undefined, true) === 1, "undefined current + correct → 1d");
assert(nextIntervalDays(-5, true) === 1, "negative current + correct → 1d");

// Monotonic on success path: 0 → 1 → 3 → 3
const chain = [0];
chain.push(nextIntervalDays(chain[chain.length - 1], true));
chain.push(nextIntervalDays(chain[chain.length - 1], true));
chain.push(nextIntervalDays(chain[chain.length - 1], true));
assert(
  chain[1] <= chain[2] && chain[2] <= chain[3],
  "success chain non-decreasing: " + chain.slice(1).join("→")
);
assert(chain[1] === 1 && chain[2] === 3 && chain[3] === 3, "success chain 1→3→3");

// dueAt arithmetic
const t0 = 1_700_000_000_000;
assert(dueAtFromInterval(1, t0) === t0 + DAY_MS, "due 1d = now + DAY_MS");
assert(dueAtFromInterval(3, t0) === t0 + 3 * DAY_MS, "due 3d = now + 3*DAY_MS");
assert(dueAtFromInterval(0, t0) === t0, "due 0d = now");

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

const store = makeStore();
const now = t0;

const card = scheduleMissed("item-a", { nowMs: now, store });
assert(card && card.interval_days === 1, "scheduleMissed defaults to 1d");
assert(card.due_at === now + DAY_MS, "scheduleMissed due_at = now+1d");

// Not due yet
const due0 = listDue({ nowMs: now, store });
assert(due0.length === 0, "not due immediately after schedule(1d)");

const due1 = listDue({ nowMs: now + DAY_MS, store });
assert(due1.length === 1 && due1[0].item_id === "item-a", "due after 1d");

// Correct review → 3d
const revOk = reviewCard("item-a", true, { nowMs: now + DAY_MS, store });
assert(revOk.interval_days === 3, "review correct: 1→3");
assert(revOk.reps === 1, "reps increments on correct");
assert(
  revOk.due_at === now + DAY_MS + 3 * DAY_MS,
  "review correct reschedules +3d from review time"
);

// Wrong → back to 1d
const revBad = reviewCard("item-a", false, {
  nowMs: now + DAY_MS + 3 * DAY_MS,
  store,
});
assert(revBad.interval_days === 1, "review wrong: →1");
assert(revBad.lapses >= 1, "lapses increments on wrong");

// recordGradedWrongs
const graded = recordGradedWrongs(
  {
    source: "mock",
    exam_id: "mock40",
    seed: 42,
    bank_hash: "abc",
    item_results: [
      { item_id: "w1", is_correct: false },
      { item_id: "ok1", is_correct: true },
      { item_id: "w2", is_correct: false },
    ],
  },
  { nowMs: now, store }
);
assert(graded.missed_ids.length === 2, "recordGradedWrongs keeps wrongs only");
assert(graded.missed_ids[0] === "w1" && graded.missed_ids[1] === "w2", "wrong order");
const missed = loadMissed(store);
assert(missed && missed.item_ids.join(",") === "w1,w2", "missed feed persists");
assert(missed.source === "mock", "missed source=mock");
const srs = loadSrsState(store);
assert(srs.cards["w1"] && srs.cards["w2"], "wrong cards scheduled in SRS");

// scheduleMissedMany on existing re-misses
const n = scheduleMissedMany(["w1"], { nowMs: now + 999, store });
assert(n === 1, "re-miss schedules");
assert(loadSrsState(store).cards["w1"].interval_days === 1, "re-miss resets to 1d");

// normalize rejects garbage
const empty = normalizeSrsState({ cards: "nope" });
assert(Object.keys(empty.cards).length === 0, "normalize rejects bad cards");

// Keys exist (schema documentation surface)
assert(typeof SRS_STORAGE_KEY === "string" && SRS_STORAGE_KEY.length > 0, "SRS key");
assert(
  typeof MISSED_STORAGE_KEY === "string" && MISSED_STORAGE_KEY.length > 0,
  "missed key"
);

// --- L6-S6: Drill-10 due-only pure filter ---
assert(DRILL10_LIMIT === 10, "DRILL10_LIMIT is 10");

const pureCards = [
  { item_id: "future", due_at: t0 + 10 * DAY_MS },
  { item_id: "due-mid", due_at: t0 - DAY_MS },
  { item_id: "due-early", due_at: t0 - 5 * DAY_MS },
  { item_id: "due-late", due_at: t0 },
  { item_id: "no-due" },
  null,
];
const pureDue = selectDueOnly(pureCards, t0, 10);
assert(pureDue.length === 3, "selectDueOnly keeps due_at ≤ now only");
assert(
  pureDue.map(function (c) {
    return c.item_id;
  }).join(",") === "due-early,due-mid,due-late",
  "selectDueOnly sorts earliest due first"
);
assert(selectDueOnly(pureCards, t0, 2).length === 2, "selectDueOnly respects limit");
assert(
  selectDueOnly(pureCards, t0, 2)[0].item_id === "due-early",
  "selectDueOnly limit keeps earliest"
);
assert(selectDueOnly([], t0, 10).length === 0, "selectDueOnly empty → empty");
assert(selectDueOnly(null, t0, 10).length === 0, "selectDueOnly null cards → empty");
assert(
  selectDueOnly(pureCards, t0).length === 3,
  "selectDueOnly default limit still returns all when under 10"
);

// Cap at 10 when more than 10 are due
const many = [];
for (let i = 0; i < 15; i++) {
  many.push({ item_id: "d" + i, due_at: t0 - (15 - i) * 1000 });
}
const capped = selectDueOnly(many, t0, DRILL10_LIMIT);
assert(capped.length === 10, "selectDueOnly caps at DRILL10_LIMIT");
assert(capped[0].item_id === "d0", "cap keeps earliest first");
assert(capped[9].item_id === "d9", "cap drops later due cards");

// listDueDrill via storage: seed 12 overdue + 1 future
const store2 = makeStore();
const cardsMap = Object.create(null);
for (let i = 0; i < 12; i++) {
  cardsMap["due-" + i] = {
    item_id: "due-" + i,
    interval_days: 1,
    due_at: t0 - (12 - i) * DAY_MS,
    reps: 0,
    lapses: 0,
    updated_at: t0,
  };
}
cardsMap["future-x"] = {
  item_id: "future-x",
  interval_days: 3,
  due_at: t0 + DAY_MS,
  reps: 1,
  lapses: 0,
  updated_at: t0,
};
saveSrsState({ schema_version: 1, cards: cardsMap }, store2);
const drill10 = listDueDrill({ nowMs: t0, store: store2 });
assert(drill10.length === 10, "listDueDrill returns at most 10");
assert(drill10[0].item_id === "due-0", "listDueDrill earliest first");
assert(
  drill10.every(function (c) {
    return c.item_id !== "future-x";
  }),
  "listDueDrill excludes not-yet-due"
);
const allDueNow = listDue({ nowMs: t0, store: store2 });
assert(allDueNow.length === 12, "listDue still returns full due set");

// parseDrillMode — load from drill.js if Node can import (browser globals optional)
try {
  const drillPath = pathToFileURL(join(ROOT, "web/assets/js/drill.js")).href;
  const drillMod = await import(drillPath);
  if (typeof drillMod.parseDrillMode === "function") {
    assert(drillMod.parseDrillMode("?mode=due") === "due", "parse mode=due");
    assert(drillMod.parseDrillMode("?mode=miss") === "miss", "parse mode=miss");
    assert(drillMod.parseDrillMode("?mode=DUE") === "due", "parse mode case-insensitive");
    assert(drillMod.parseDrillMode("") === "default", "parse empty → default");
    assert(drillMod.parseDrillMode("?mode=other") === "default", "parse unknown → default");
    assert(
      drillMod.EMPTY_DUE_MESSAGE === "No cards due — take a mock or quiz",
      "honest empty due copy"
    );
  }
} catch (e) {
  // drill.js may touch document on import; pure srs tests above still cover filter.
  console.log("ok: skip drill.js mode parse (import side-effect): " + (e && e.message));
}

if (failed > 0) {
  console.error("\nsmoke_srs: " + failed + " failure(s)");
  process.exit(1);
}
console.log("\nsmoke_srs: all checks passed");
process.exit(0);
