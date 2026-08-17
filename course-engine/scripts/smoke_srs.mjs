#!/usr/bin/env node
/**
 * Smoke for short-interval review + Drill-10 due filter
 * (L5-S7 / bd-ca8 · L6-S6 / bd-3dd · bd-engine-not-gate-ar39.5).
 *
 * The interval law is WASM (`cdcp_schedule`). This smoke loads wasm, then
 * exercises the JS persist/render path.
 *
 * Usage (from course-engine/):
 *   node scripts/smoke_srs.mjs
 */
import { existsSync, readFileSync } from "node:fs";
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

const reviewPath = pathToFileURL(join(ROOT, "web/assets/js/review.js")).href;

const {
  nextIntervalDays,
  dueAtFromInterval,
  dayMs,
  DRILL10_LIMIT,
  normalizeReviewState,
  scheduleMissed,
  scheduleMissedMany,
  reviewCard,
  selectDueOnly,
  listDue,
  listDueDrill,
  recordGradedWrongs,
  loadMissed,
  loadReviewState,
  saveReviewState,
  saveMissed,
  listAllCards,
  isApprovedRow,
  packRows,
  formatPruneNotice,
  pruneNonApprovedFromStorage,
  APPROVED_STATUS,
  REVIEW_STORAGE_KEY,
  MISSED_STORAGE_KEY,
  isUnknownScheduleVersionError,
} = await import(reviewPath);

const DAY_MS = dayMs();

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
const srs = loadReviewState(store);
assert(srs.cards["w1"] && srs.cards["w2"], "wrong cards scheduled in review");

// scheduleMissedMany on existing re-misses
const n = scheduleMissedMany(["w1"], { nowMs: now + 999, store });
assert(n === 1, "re-miss schedules");
assert(loadReviewState(store).cards["w1"].interval_days === 1, "re-miss resets to 1d");

// normalize rejects garbage
const empty = normalizeReviewState({ cards: "nope" });
assert(Object.keys(empty.cards).length === 0, "normalize rejects bad cards");
assert(empty.schema_version === 1, "unversioned garbage migrates to v1");

// v0 (missing version) → v1, fields identity
const v0 = normalizeReviewState({
  cards: {
    x: {
      item_id: "x",
      interval_days: 3,
      due_at: 42,
      reps: 1,
      lapses: 2,
      updated_at: 41,
    },
  },
});
assert(v0.schema_version === 1, "unversioned record migrates to v1");
assert(v0.cards.x && v0.cards.x.interval_days === 3, "v0→v1 identity on fields");

// unknown version is ERROR — must not coerce to v1 or wipe
let unknownThrew = false;
try {
  normalizeReviewState({ schema_version: 99, cards: {} });
} catch (err) {
  unknownThrew = isUnknownScheduleVersionError(err);
}
assert(unknownThrew, "unknown schema_version is ERROR");

const storeUnknown = makeStore();
storeUnknown.setItem(
  REVIEW_STORAGE_KEY,
  JSON.stringify({ schema_version: 2, cards: { y: { item_id: "y" } } })
);
let loadUnknownThrew = false;
try {
  loadReviewState(storeUnknown);
} catch (err) {
  loadUnknownThrew = isUnknownScheduleVersionError(err);
}
assert(loadUnknownThrew, "loadReviewState does not swallow unknown version");
assert(
  storeUnknown.getItem(REVIEW_STORAGE_KEY).indexOf('"schema_version":2') !== -1,
  "unknown version must not wipe storage"
);

// Keys exist (schema documentation surface)
assert(typeof REVIEW_STORAGE_KEY === "string" && REVIEW_STORAGE_KEY.length > 0, "review key");
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
saveReviewState({ schema_version: 1, cards: cardsMap }, store2);
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

// --- bd-srs-residue-retired-ids-k3vs: drop retired residue from stores ---

function cardAt(id, dueAt) {
  return {
    item_id: id,
    interval_days: 1,
    due_at: dueAt,
    reps: 0,
    lapses: 1,
    updated_at: t0,
  };
}

function seedStores(store, ids) {
  const cards = Object.create(null);
  for (let i = 0; i < ids.length; i++) {
    cards[ids[i]] = cardAt(ids[i], t0);
  }
  saveReviewState({ schema_version: 1, cards: cards }, store);
  saveMissed(
    { source: "quiz", exam_id: "m05", seed: 42, item_ids: ids.slice() },
    store
  );
}

const plantPack = [
  { id: "keep-me", status: "approved", stem: "live" },
  { id: "m05-q200", status: "retired", stem: "withdrawn" },
  { id: "planted-retired-xyz", status: "retired", stem: "not in the 8-id list" },
  { id: "drafty", status: "draft", stem: "draft" },
  { id: "nostatus-row", stem: "no status field" },
];

assert(APPROVED_STATUS === "approved", "APPROVED_STATUS is approved");
assert(isApprovedRow({ status: "approved" }) === true, "isApprovedRow approved");
assert(isApprovedRow({ status: "retired" }) === false, "isApprovedRow retired");
assert(isApprovedRow({ status: "draft" }) === false, "isApprovedRow draft");
assert(isApprovedRow({}) === false, "isApprovedRow absent status is withheld");
assert(isApprovedRow(null) === false, "isApprovedRow null");
assert(packRows(plantPack).length === 5, "packRows array");
assert(packRows({ items: plantPack }).length === 5, "packRows {items}");
assert(packRows(null).length === 0, "packRows null → empty");
assert(packRows({}).length === 0, "packRows {} → empty");

assert(formatPruneNotice(null) === "", "notice null → empty");
assert(formatPruneNotice({ dropped: [] }) === "", "notice nothing → empty");
assert(
  formatPruneNotice({ dropped: [], emptied: true }) === "",
  "notice emptied-but-dropped-empty is still empty (no-op must not look like a prune)"
);
const noticePartial = formatPruneNotice({
  dropped: ["planted-retired-xyz"],
  emptied: false,
});
const noticeEmptyQ = formatPruneNotice({
  dropped: ["m05-q200", "planted-retired-xyz"],
  emptied: true,
});
assert(noticePartial.length > 0, "notice that pruned is non-empty");
assert(
  noticePartial.indexOf("planted-retired-xyz") >= 0,
  "notice names the withdrawn id"
);
assert(
  noticePartial.indexOf("1 card removed") >= 0,
  "notice singular card copy"
);
assert(
  noticePartial.indexOf("empty") < 0,
  "partial prune must not claim the queue is empty"
);
assert(
  noticeEmptyQ.indexOf("2 cards removed") >= 0,
  "notice plural cards copy"
);
assert(
  noticeEmptyQ.indexOf("empty") >= 0,
  "a prune that empties the whole queue must say so"
);
assert(
  formatPruneNotice({ dropped: [] }) !== noticePartial,
  "a prune that finds nothing must not report like one that pruned"
);

// Empty pack must not wipe a store that happens to hold residue.
const storeEmptyPack = makeStore();
seedStores(storeEmptyPack, ["m05-q200", "keep-me"]);
const skipEmpty = pruneNonApprovedFromStorage([], storeEmptyPack);
assert(skipEmpty.skipped === "empty-pack", "empty pack is skipped");
assert(skipEmpty.dropped.length === 0, "empty pack drops nothing");
assert(skipEmpty.persisted === false, "empty pack does not persist");
assert(
  loadMissed(storeEmptyPack).item_ids.join(",") === "m05-q200,keep-me",
  "empty pack leaves missed untouched"
);
assert(
  !!loadReviewState(storeEmptyPack).cards["m05-q200"],
  "empty pack leaves SRS untouched"
);

// Learner mock40 pack has no status field — must not wipe the queue.
const storeNoStatus = makeStore();
seedStores(storeNoStatus, ["m05-q200", "keep-me"]);
const skipNoStatus = pruneNonApprovedFromStorage(
  { items: [{ id: "m05-q200", stem: "x", choices: [] }] },
  storeNoStatus
);
assert(
  skipNoStatus.skipped === "pack-has-no-status",
  "pack without status is skipped"
);
assert(
  loadMissed(storeNoStatus).item_ids.join(",") === "m05-q200,keep-me",
  "no-status pack leaves missed untouched"
);

// Mixed store: keep approved, drop retired / draft / missing-status / unknown.
const storeMix = makeStore();
const mixIds = [
  "keep-me",
  "m05-q200",
  "planted-retired-xyz",
  "drafty",
  "nostatus-row",
  "ghost-not-in-pack",
];
seedStores(storeMix, mixIds);
const rMix = pruneNonApprovedFromStorage(plantPack, storeMix);
assert(rMix.skipped === null, "mixed prune is not skipped");
assert(rMix.persisted === true, "mixed prune persists");
assert(
  rMix.dropped.join(",") ===
    "drafty,ghost-not-in-pack,m05-q200,nostatus-row,planted-retired-xyz",
  "mixed prune drops every non-approved id (status-driven, not a hardcoded list)"
);
assert(
  rMix.dropped.indexOf("planted-retired-xyz") >= 0,
  "a later retirement wave id is covered without a code change"
);
assert(rMix.emptied === false, "mixed prune leaves the approved card");
assert(rMix.emptied_missed === false, "missed still has keep-me");
assert(rMix.emptied_srs === false, "SRS still has keep-me");
const missedMix = loadMissed(storeMix);
const srsMix = loadReviewState(storeMix);
assert(
  missedMix.item_ids.join(",") === "keep-me",
  "missed keeps only approved"
);
assert(
  Object.keys(srsMix.cards).join(",") === "keep-me",
  "SRS keeps only approved"
);
assert(
  formatPruneNotice(rMix).indexOf("planted-retired-xyz") >= 0,
  "mixed notice names a dropped id"
);
assert(
  formatPruneNotice(rMix).indexOf("empty") < 0,
  "mixed notice does not claim empty"
);

// Second load is a no-op — pruned state persisted, not re-pruned every load.
const rMix2 = pruneNonApprovedFromStorage(plantPack, storeMix);
assert(rMix2.dropped.length === 0, "second load drops nothing");
assert(rMix2.persisted === false, "second load does not persist");
assert(
  formatPruneNotice(rMix2) === "",
  "second load must not report like a prune"
);
assert(
  loadMissed(storeMix).item_ids.join(",") === "keep-me",
  "second load leaves the approved remainder"
);

// ANTI-VACUOUS: a store that contains ONLY retired ids must end empty,
// not look like a live review queue.
const storeOnlyRetired = makeStore();
const onlyRetired = ["m05-q200", "planted-retired-xyz"];
seedStores(storeOnlyRetired, onlyRetired);
const rOnly = pruneNonApprovedFromStorage(plantPack, storeOnlyRetired);
assert(rOnly.dropped.length === 2, "only-retired drops both");
assert(rOnly.emptied === true, "only-retired empties the whole queue");
assert(rOnly.emptied_missed === true, "only-retired empties missed");
assert(rOnly.emptied_srs === true, "only-retired empties SRS");
assert(rOnly.persisted === true, "only-retired persists the empty stores");
assert(
  loadMissed(storeOnlyRetired).item_ids.length === 0,
  "only-retired missed ends empty"
);
assert(
  Object.keys(loadReviewState(storeOnlyRetired).cards).length === 0,
  "only-retired SRS cards end empty"
);
assert(
  listDue({ nowMs: t0, store: storeOnlyRetired }).length === 0,
  "only-retired listDue is not a live queue"
);
assert(
  listAllCards({ store: storeOnlyRetired }).length === 0,
  "only-retired listAllCards is empty"
);
assert(
  listDueDrill({ nowMs: t0, store: storeOnlyRetired }).length === 0,
  "only-retired Drill-10 is empty"
);
const onlyNotice = formatPruneNotice(rOnly);
assert(onlyNotice.indexOf("empty") >= 0, "only-retired notice says empty");
assert(onlyNotice.indexOf("m05-q200") >= 0, "only-retired notice names an id");

// Clean store of only approved: find-nothing must not look like a prune.
const storeClean = makeStore();
seedStores(storeClean, ["keep-me"]);
const rClean = pruneNonApprovedFromStorage(plantPack, storeClean);
assert(rClean.dropped.length === 0, "clean store drops nothing");
assert(rClean.persisted === false, "clean store does not persist");
assert(rClean.emptied === false, "clean store is not reported emptied");
assert(formatPruneNotice(rClean) === "", "clean store has no notice");
assert(
  loadMissed(storeClean).item_ids.join(",") === "keep-me",
  "clean store missed untouched"
);

// Committed pack: the 8 residue ids 7big named, plus one live approved.
const bankPack = JSON.parse(
  readFileSync(join(ROOT, "web/data/bank_items_seed42.json"), "utf8")
);
const bankRows = packRows(bankPack);
const bankRetired = bankRows.filter(function (r) {
  return r && r.status === "retired";
});
const bankApproved = bankRows.filter(function (r) {
  return r && r.status === APPROVED_STATUS;
});
assert(bankRows.length > 0, "committed bank pack is non-empty");
assert(
  bankRetired.length > 0,
  "committed pack has retired rows — else this residue sweep is vacuous"
);
assert(
  bankApproved.length > 0 && bankApproved.length < bankRows.length,
  "committed pack has an approved pool distinct from the file set"
);
const residueNamed = [
  "m05-q200",
  "mock40-q18",
  "mock40-q24",
  "mock40-q22",
  "m12-q219",
  "mock40-q37",
  "mock40-q40",
];
for (let i = 0; i < residueNamed.length; i++) {
  const row = bankRows.find(function (r) {
    return r && r.id === residueNamed[i];
  });
  assert(
    !!row && row.status === "retired",
    residueNamed[i] + " is retired in the committed pack"
  );
}
const liveId = bankApproved[0].id;
const storePack = makeStore();
seedStores(storePack, residueNamed.concat([liveId]));
const rPack = pruneNonApprovedFromStorage(bankPack, storePack);
assert(
  rPack.dropped.length === residueNamed.length,
  "committed-pack prune drops the 7 named residue ids"
);
assert(
  residueNamed.every(function (id) {
    return rPack.dropped.indexOf(id) >= 0;
  }),
  "committed-pack prune names every 7big residue id"
);
assert(
  loadMissed(storePack).item_ids.join(",") === liveId,
  "committed-pack prune keeps a live approved id"
);
assert(
  !!loadReviewState(storePack).cards[liveId] &&
    !loadReviewState(storePack).cards["m05-q200"],
  "committed-pack SRS keeps live and drops retired"
);
assert(
  listDue({ nowMs: t0, store: storePack }).every(function (c) {
    return c.item_id === liveId;
  }),
  "committed-pack due queue is only the live id"
);

// The prune implementation is not a hardcoded id list.
const reviewSrc = readFileSync(join(ROOT, "web/assets/js/review.js"), "utf8");
assert(
  reviewSrc.indexOf("m05-q200") < 0 &&
    reviewSrc.indexOf("mock40-q18") < 0 &&
    reviewSrc.indexOf("planted-retired-xyz") < 0,
  "review.js prune is not a hardcoded residue id list"
);
const drillSrc = readFileSync(join(ROOT, "web/assets/js/drill.js"), "utf8");
const loadBankSlice = drillSrc.slice(
  drillSrc.indexOf("async function loadBank"),
  drillSrc.indexOf("async function init")
);
assert(
  loadBankSlice.indexOf("status ===") < 0 &&
    loadBankSlice.indexOf("status ==") < 0 &&
    loadBankSlice.indexOf("status !==") < 0 &&
    loadBankSlice.indexOf("isApproved") < 0,
  "loadBank must not filter byId on status"
);
assert(
  loadBankSlice.indexOf("byId[arr[i].id] = arr[i]") >= 0,
  "loadBank indexes every manifest row"
);
assert(
  drillSrc.indexOf("pruneNonApprovedFromStorage") >= 0,
  "drill.js calls the storage sweep on load"
);

if (failed > 0) {
  console.error("\nsmoke_srs: " + failed + " failure(s)");
  process.exit(1);
}
console.log("\nsmoke_srs: all checks passed");
process.exit(0);
