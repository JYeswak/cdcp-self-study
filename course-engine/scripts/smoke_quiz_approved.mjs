#!/usr/bin/env node
/**
 * NO LEARNER-FACING SURFACE MAY DRAW A NON-APPROVED BANK ITEM (bd-7big P0 / bd-qqwc P1).
 *
 * Usage (from course-engine/):
 *   node scripts/smoke_quiz_approved.mjs
 *
 * # Why this gate is in the browser layer and not in Rust
 *
 * The defect was a browser-layer draw: `quiz.js filterByModule()` filtered
 * `web/data/bank_items_seed42.json` by the numeric `module` field and never read
 * `status`. Every Rust gate in this repo was GREEN while it happened, and could
 * not have been otherwise — no Rust code path executes `filterByModule`.
 *
 * A Rust test could only re-implement the draw, and a re-implementation is
 * exactly what produced the original measurement of this bug: faithful today,
 * silently divergent the first time somebody changes `sampleItems`. So this gate
 * IMPORTS THE SHIPPED MODULES and runs THE COMMITTED PACK through them. What it
 * asserts is what a learner is actually served.
 *
 * # Why the pack itself is not the fix (decided; do not re-litigate)
 *
 * `bank_items_seed42.json` is the content-addressed MANIFEST of the whole bank —
 * 804 rows, 779 `approved`, 25 `retired` — and it cannot be filtered at the
 * source: `cdcp_wasm::grade_digest_json` feeds those exact bytes to
 * `Bank::from_json_str`, which recomputes `bank_hash`, and `cdcp_grade::grade`
 * hard-fails on a mismatch. An approved-only pack would make EVERY client-side
 * grade fail. The manifest ships whole; the CONSUMERS filter. Recorded in
 * `web/data/README.md` and the `web.bank-items-pack` row of
 * `registries/goldens-couplings.toml`.
 *
 * # Anti-vacuous discipline
 *
 * Every leg below can fail; none can pass by finding nothing.
 *   * A pack with ZERO retired rows makes the whole sweep vacuous — it would
 *     report identically against an unfiltered `filterByModule`. That is a
 *     FAILURE here, not a pass.
 *   * A quiz that drew ZERO items is a FAILURE, not a clean sweep.
 *   * The KNOWN-BAD leg proves the sweep can go red: it retires an item the
 *     draw currently serves and asserts (a) the module-only predicate still
 *     picks it up, so the plant landed, and (b) the shipped filter drops it.
 *   * A filter that removes EVERYTHING is an ERROR, never an empty quiz; and a
 *     pool below the requested size is an ERROR NAMING THE MODULE, never a
 *     silently shorter quiz.
 *
 * Exit 0 only if every leg passes.
 */
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, "..");
const PACK_REL = "web/data/bank_items_seed42.json";
const UNITS_REL = "web/data/units_index.json";
const APPROVED = "approved";

/** Bank module numbers a module quiz can be deep-linked to. */
const MODULES = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

const quiz = await import(
  pathToFileURL(join(ROOT, "web/assets/js/quiz.js")).href
);
await import(pathToFileURL(join(ROOT, "web/assets/js/learn_units.js")).href);
const learnUnits = globalThis.CdcpLearnUnits;

let failed = 0;

function assert(cond, msg) {
  if (!cond) {
    console.error("FAIL:", msg);
    failed += 1;
  } else {
    console.log("ok:", msg);
  }
}

function readJson(rel) {
  return JSON.parse(readFileSync(join(ROOT, rel), "utf8"));
}

function rows(pack) {
  return Array.isArray(pack) ? pack : (pack && pack.items) || [];
}

/** The predicate the defect shipped: module only, status ignored. */
function moduleOnly(pack, moduleNum) {
  return rows(pack).filter(
    (it) => it && typeof it.module === "number" && it.module === moduleNum
  );
}

/**
 * Every quiz the deterministic draw can produce for one pack, as
 * `{ label, module, items }`. Mirrors `startModule`'s seed derivation exactly:
 * `42 + module*1000`, plus 15 for `?mode=learn15`.
 */
function sweep(pack) {
  const out = [];
  for (const m of MODULES) {
    const pool = quiz.filterByModule(pack, m);
    if (pool.length === 0) continue;
    for (const [label, opts] of [
      ["default", { min: 8, max: 12, learn15: false }],
      ["learn15", { min: 5, max: 5, learn15: true }],
    ]) {
      const seed = 42 + m * 1000 + (opts.learn15 ? 15 : 0);
      out.push({
        label: `m${String(m).padStart(2, "0")} ${label}`,
        module: m,
        items: quiz.sampleItems(pool, seed, opts.min, opts.max),
      });
    }
  }
  return out;
}

// ── 0. the pack, and the precondition that makes this suite non-vacuous ────

const pack = readJson(PACK_REL);
const all = rows(pack);
const approved = all.filter((it) => it && it.status === APPROVED);
const withheld = all.filter((it) => !it || it.status !== APPROVED);

assert(all.length > 0, `${PACK_REL} carries ${all.length} manifest rows`);
assert(
  withheld.length > 0,
  `${PACK_REL} carries ${withheld.length} NON-APPROVED row(s) — without at ` +
    `least one, every sweep below would report identically against an ` +
    `unfiltered filter and this whole file would be vacuous`
);
assert(
  approved.length > 0 && approved.length < all.length,
  `approved pool ${approved.length} of ${all.length} manifest rows ` +
    `(the two numbers are different things and must not blur)`
);

const withheldIds = new Set(withheld.map((it) => it.id));

// ── 1. the shipped draw serves nothing withheld, computed not asserted ─────

const quizzes = sweep(pack);
assert(
  quizzes.length === MODULES.length * 2,
  `swept ${quizzes.length} deterministic quizzes (expected ${MODULES.length * 2}: ` +
    `every module in default and learn15 mode)`
);

let served = 0;
const offenders = [];
for (const q of quizzes) {
  assert(
    q.items.length > 0,
    `${q.label} drew ${q.items.length} items — a quiz that drew nothing is a FAILURE`
  );
  for (const it of q.items) {
    served += 1;
    if (withheldIds.has(it.id)) offenders.push(`${q.label} -> ${it.id}`);
  }
}
assert(served > 0, `the sweep served ${served} item slots in total`);
assert(
  offenders.length === 0,
  `0 non-approved items served across ${quizzes.length} deterministic quizzes` +
    (offenders.length ? ` — served: ${offenders.join(", ")}` : "")
);

// The same sweep under the DEFECTIVE predicate, to record what was being served
// before the filter landed. Not an assertion about today's behaviour — a
// measurement that keeps the number in the gate rather than in a report nobody
// re-runs.
let legacyOffenders = 0;
for (const m of MODULES) {
  const pool = moduleOnly(pack, m);
  if (!pool.length) continue;
  for (const [min, max, l15] of [
    [8, 12, false],
    [5, 5, true],
  ]) {
    const seed = 42 + m * 1000 + (l15 ? 15 : 0);
    for (const it of quiz.sampleItems(pool, seed, min, max)) {
      if (withheldIds.has(it.id)) legacyOffenders += 1;
    }
  }
}
console.log(
  `note: the module-only predicate this gate replaced would serve ` +
    `${legacyOffenders} non-approved item(s) on the same pack`
);

// ── 2. KNOWN-BAD: retiring a served item must remove it from the draw ──────
//
// Proof the sweep can go RED. Take an item the draw currently serves, withdraw
// it in a copy of the pack, and assert both directions: the module-only
// predicate still picks it (so the plant is real and reachable) and the shipped
// filter drops it (so the filter is what removed it).

const victimQuiz = quizzes.find((q) => q.items.length > 0);
assert(!!victimQuiz, "the known-bad leg needs at least one non-empty quiz");
if (victimQuiz) {
  const victim = victimQuiz.items[0].id;
  const planted = all.map((it) =>
    it.id === victim ? { ...it, status: "retired" } : it
  );
  assert(
    moduleOnly(planted, victimQuiz.module).some((it) => it.id === victim),
    `known-bad plant is reachable: ${victim} is still in module ` +
      `${victimQuiz.module} under the module-only predicate`
  );
  assert(
    !quiz
      .filterByModule(planted, victimQuiz.module)
      .some((it) => it.id === victim),
    `known-bad: ${victim} retired -> excluded from the module ${victimQuiz.module} pool`
  );
  const after = sweep(planted);
  assert(
    !after.some((q) => q.items.some((it) => it.id === victim)),
    `known-bad: ${victim} retired -> served by NO quiz in the whole sweep`
  );
  assert(
    after.length === quizzes.length,
    "retiring one item must not silently delete a whole quiz"
  );
}

// ── 3. ANTI-VACUOUS: an emptied or under-supplied pool is an ERROR ─────────

const allRetired = all.map((it) => ({ ...it, status: "retired" }));
assert(
  quiz.approvedModules(allRetired).length === 0,
  "a pack with no approved rows offers NO modules in the picker"
);
assert(
  quiz.filterByModule(allRetired, 6).length === 0,
  "a pack with no approved rows yields an empty module 6 pool"
);
assert(
  typeof quiz.poolShortfall([], 6, 8, 136) === "string" &&
    quiz.poolShortfall([], 6, 8, 136).includes("Module 06") &&
    quiz.poolShortfall([], 6, 8, 136).includes("NO approved items"),
  "an emptied pool is an ERROR NAMING THE MODULE, not an empty quiz"
);
const thin = quiz.filterByModule(pack, 5).slice(0, 4);
const thinMsg = quiz.poolShortfall(thin, 5, 8, 31);
assert(
  typeof thinMsg === "string" &&
    thinMsg.includes("Module 05") &&
    thinMsg.includes("4 approved item(s)") &&
    thinMsg.includes("asks for 8"),
  "a pool below the quiz size is an ERROR NAMING THE MODULE, never a shorter quiz"
);
assert(
  quiz.poolShortfall(quiz.filterByModule(pack, 5), 5, 8, 31) === null,
  "a pool that clears the floor is not an error"
);

// Headroom: how close the thinnest module is to the refusal line. Reported so a
// future retirement wave cannot quietly walk a module under the quiz size.
const headroom = [];
for (const m of MODULES) {
  const a = quiz.filterByModule(pack, m).length;
  const t = moduleOnly(pack, m).length;
  headroom.push({ module: m, approved: a, manifest: t, marginToMax: a - 12 });
}
headroom.sort((x, y) => x.approved - y.approved);
console.log(
  "note: thinnest approved pools — " +
    headroom
      .slice(0, 3)
      .map(
        (h) =>
          `m${String(h.module).padStart(2, "0")} ${h.approved}/${h.manifest} ` +
          `(+${h.marginToMax} over the 12-item max)`
      )
      .join(", ")
);
assert(
  headroom[0].approved >= 12,
  `thinnest approved pool is m${headroom[0].module} with ${headroom[0].approved} ` +
    `items; every module must clear the 12-item quiz max or the draw refuses`
);

// ── 4. the learn-page surfaces: units_index.json and pickItems ─────────────

const units = readJson(UNITS_REL);
const unitRows = units.units || [];
assert(unitRows.length > 0, `${UNITS_REL} carries ${unitRows.length} units`);
const badChecks = [];
let checkIds = 0;
for (const u of unitRows) {
  for (const id of u.check_item_ids || []) {
    checkIds += 1;
    if (withheldIds.has(id)) badChecks.push(`${u.id} -> ${id}`);
  }
}
assert(checkIds > 0, `${UNITS_REL} carries ${checkIds} check_item_ids in total`);
assert(
  badChecks.length === 0,
  `0 non-approved ids among ${checkIds} unit check_item_ids` +
    (badChecks.length ? ` — found: ${badChecks.join(", ")}` : "")
);

assert(
  !!learnUnits && typeof learnUnits.bankList === "function",
  "learn_units.js exposes bankList for inspection"
);
if (learnUnits && learnUnits.bankList) {
  assert(
    learnUnits.bankList(pack).length === approved.length,
    `learn_units.bankList drops every withheld row ` +
      `(${learnUnits.bankList(pack).length} of ${all.length})`
  );
  // pickItems branch 2 and 3 draw from the whole module, not from
  // check_item_ids — the paths a units_index fix alone would leave open.
  const unit = unitRows.find(
    (u) => u.module_num === 5 && (u.topic_ids || []).length
  );
  assert(!!unit, "found a module-5 unit to exercise pickItems against");
  if (unit) {
    const bare = { ...unit, check_item_ids: [] };
    const picked = learnUnits.pickItems(pack, bare, 3);
    assert(
      picked.length > 0,
      `pickItems fell through to topic/module fill and drew ${picked.length} items`
    );
    assert(
      !picked.some((it) => withheldIds.has(it.id)),
      "pickItems topic/module fill draws no withheld item"
    );
  }
}

// ── verdict ────────────────────────────────────────────────────────────────

if (failed) {
  console.error(`\nFAIL: smoke_quiz_approved — ${failed} check(s) failed`);
  process.exit(1);
}
console.log(
  `\nPASS: smoke_quiz_approved — manifest=${all.length} approved=${approved.length} ` +
    `withheld=${withheld.length} quizzes=${quizzes.length} served=${served} ` +
    `unit_checks=${checkIds}; 0 non-approved reachable`
);
