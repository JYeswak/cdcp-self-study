#!/usr/bin/env node
/**
 * Pure-function smoke for hub mastery dashboard + recommend (L6-S4 / bd-qyi).
 *
 * Usage (from course-engine/):
 *   node scripts/smoke_hub_mastery.mjs
 *
 * Exit 0 only if:
 *   - MODULE_CATALOG agrees, module for module, with the Learn registry
 *     (web/data/modules_index.json) — same count, same ids, nothing dropped
 *   - recordQuizResult module 3 @ 80%+ -> practiced badge state true
 *   - recommend prefers weak -> unpracticed -> unmastered -> all_practiced
 *   - every recommend href maps to an existing learn/*.html or quiz.html pattern
 *   - saveLastWeak writes cdcp.last_weak.v1
 *   - no cert language in recommend copy
 *
 * Exit 2 means the smoke could not be run honestly: no registry, an unparseable
 * registry, zero declared modules, or an empty catalog. Those are ERRORS and
 * never a pass — a suite that checked nothing reports exactly like one that
 * checked everything and found it clean.
 *
 * ## Why there is no module count in this file (bd-61ey / bd-lt7 class)
 *
 * Until 2026-08-14 this gate asserted `MODULE_CATALOG.length === 14` and swept
 * modules with two `m <= 14` loops. Module 15 is assessed AND taught, so those
 * three lines had written the product defect down as a requirement: adding the
 * module to the hub made the gate go RED for being correct, and a fixer reading
 * the red gate would revert the fix. Raising the literal to fifteen would have
 * re-encoded the same defect one module later. Every count below is now read
 * off the registry at run time, so the gate tracks the curriculum instead of
 * remembering a snapshot of it.
 *
 * No browser required.
 */
import { existsSync, readFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { pathToFileURL } from "node:url";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, "..");
const WEB = join(ROOT, "web");
const INDEX_PATH = join(WEB, "data/modules_index.json");

/** Structural ERROR: the smoke cannot be run honestly. Never a pass. */
function abort(msg) {
  console.error("ERROR: " + msg);
  console.error("\nsmoke_hub_mastery: NOT RUN (structural)");
  process.exit(2);
}

// --- the registry, read independently of the module under test -------------
// This gate must not learn the module set from the same object hub_mastery.js
// built its catalog out of, or the comparison below would be a tautology.
if (!existsSync(INDEX_PATH)) {
  abort("no Learn registry at web/data/modules_index.json — run scripts/build_learn.py");
}
let declared = null;
try {
  const doc = JSON.parse(readFileSync(INDEX_PATH, "utf8"));
  declared = doc && Array.isArray(doc.modules) ? doc.modules.slice() : null;
} catch (e) {
  abort("web/data/modules_index.json does not parse: " + e.message);
}
if (!declared) {
  abort("web/data/modules_index.json has no `modules` array");
}
if (declared.length === 0) {
  abort("web/data/modules_index.json declares zero modules — a vacuous sweep is an ERROR");
}
declared.sort(function (a, b) {
  return Number(a.order) - Number(b.order);
});

function findWasm() {
  const candidates = [
    join(ROOT, "web/assets/wasm/cdcp_wasm.wasm"),
    join(ROOT, "target/wasm32-unknown-unknown/release/cdcp_wasm.wasm"),
    join(ROOT, "target/wasm32-unknown-unknown/debug/cdcp_wasm.wasm"),
  ];
  for (const p of candidates) {
    if (existsSync(p)) return p;
  }
  abort("no cdcp_wasm.wasm — cargo build -p cdcp_wasm --target wasm32-unknown-unknown");
}
const { loadWasm } = await import(
  pathToFileURL(join(WEB, "assets/js/grade_bridge.js")).href
);
try {
  await loadWasm(findWasm());
} catch (e) {
  abort("WASM required for mastery law: " + (e && e.message));
}

const masteryPath = pathToFileURL(join(WEB, "assets/js/mastery.js")).href;
const hubPath = pathToFileURL(join(WEB, "assets/js/hub_mastery.js")).href;

const { recordQuizResult, isPracticed, dayMs } = await import(masteryPath);
const DAY_MS = dayMs();
const {
  WEAK_STORAGE_KEY,
  MODULE_CATALOG,
  buildCatalog,
  saveLastWeak,
  loadLastWeak,
  recommendNext,
  moduleBadgeState,
  badgeHtml,
} = await import(hubPath);

if (typeof buildCatalog !== "function") {
  abort("hub_mastery.js exports no buildCatalog — the catalog is not derived");
}
if (!Array.isArray(MODULE_CATALOG) || MODULE_CATALOG.length === 0) {
  abort("MODULE_CATALOG is empty — an empty catalog is an ERROR, not a blank dashboard");
}

let failed = 0;

function assert(cond, msg) {
  if (!cond) {
    console.error("FAIL:", msg);
    failed += 1;
  } else {
    console.log("ok:", msg);
  }
}

function threw(fn) {
  try {
    fn();
    return false;
  } catch (_) {
    return true;
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

function label(order, id) {
  return "M" + String(order).padStart(2, "0") + " " + id;
}

/**
 * The detector this gate exists for: which modules does the registry declare
 * that the catalog does not carry (and vice versa)? Returns human-readable
 * findings that NAME each module, so a red gate says which one went missing.
 */
function agreementReport(declaredRows, catalog) {
  const declaredIds = new Set(declaredRows.map((r) => String(r.id)));
  const catalogIds = new Set(catalog.map((e) => String(e.id)));
  const findings = [];
  for (const r of declaredRows) {
    if (!catalogIds.has(String(r.id))) {
      findings.push(label(r.order, r.id) + " declared by the registry, ABSENT from the catalog");
    }
  }
  for (const e of catalog) {
    if (!declaredIds.has(String(e.id))) {
      findings.push(label(e.order, e.id) + " in the catalog, NOT declared by the registry");
    }
  }
  return findings;
}

/** Local projection of a registry row, independent of buildCatalog. */
function projectRow(r) {
  return {
    order: Number(r.order),
    id: String(r.id),
    title: String(r.epi_heading || r.id),
    learnHref: String(r.href),
    quizHref: "quiz.html?module=" + Number(r.order),
  };
}

// --- catalog is DERIVED from the registry ----------------------------------
assert(
  MODULE_CATALOG.length === declared.length,
  "catalog carries " +
    MODULE_CATALOG.length +
    " modules, matching the " +
    declared.length +
    " the Learn registry declares"
);
{
  // THE ASSERTION THIS BEAD EXISTS FOR. A module that is taught and assessed
  // but missing from the catalog cannot be seen by a learner anywhere on the
  // hub, and nothing else in this suite would notice.
  const drift = agreementReport(declared, MODULE_CATALOG);
  assert(
    drift.length === 0,
    "catalog and Learn registry agree module for module" +
      (drift.length ? " — " + drift.join("; ") : "")
  );
}

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

// --- known-bad: a declared module missing from the catalog is RED, by name --
{
  const dropped = declared[declared.length - 1];
  const crippled = declared.slice(0, -1).map(projectRow);
  const report = agreementReport(declared, crippled);
  assert(
    report.length === 1,
    "known-bad: a catalog missing one declared module produces exactly one finding (got " +
      report.length +
      ")"
  );
  assert(
    report.length > 0 && report[0].indexOf(dropped.id) !== -1,
    "known-bad: the finding NAMES the missing module (" +
      dropped.id +
      "): " +
      (report[0] || "<no finding>")
  );
  assert(
    report.length > 0 && /ABSENT from the catalog/.test(report[0]),
    "known-bad: the finding says the module is absent, not merely different"
  );
}

// --- known-GOOD: a smaller but legitimate curriculum still passes -----------
// declared.slice(0, -1) is a real, self-consistent registry one module shorter
// than the live one. It must derive cleanly, or the rebase is only a new
// literal in place of the old one and the next module hits the same wall.
{
  const smaller = declared.slice(0, -1);
  if (smaller.length === 0) {
    abort("only one module is declared — the smaller-tree known-GOOD leg cannot run");
  }
  const smallerCatalog = buildCatalog({ modules: smaller });
  assert(
    smallerCatalog.length === smaller.length,
    "known-GOOD: a " +
      smaller.length +
      "-module registry derives a " +
      smallerCatalog.length +
      "-module catalog"
  );
  assert(
    agreementReport(smaller, smallerCatalog).length === 0,
    "known-GOOD: the smaller registry and its catalog agree module for module"
  );
  const dropped = declared[declared.length - 1];
  assert(
    !smallerCatalog.some((e) => String(e.id) === String(dropped.id)),
    "known-GOOD: the smaller catalog invents no module its registry does not declare"
  );
}

// --- anti-vacuous: an empty or absent registry is an ERROR, never a pass ----
{
  assert(
    threw(() => buildCatalog({ modules: [] })),
    "anti-vacuous: a registry declaring zero modules throws, it does not yield an empty catalog"
  );
  assert(
    threw(() => buildCatalog({})),
    "anti-vacuous: a registry with no `modules` array throws"
  );
  assert(
    threw(() => buildCatalog(null)),
    "anti-vacuous: a missing registry throws"
  );
  assert(
    threw(() => buildCatalog({ modules: [{ order: 1, href: "learn/x.html" }] })),
    "anti-vacuous: a registry row with no id throws"
  );
}

// --- weak storage ----------------------------------------------------------
{
  const first = MODULE_CATALOG[0];
  const mid = MODULE_CATALOG[Math.floor(MODULE_CATALOG.length / 2)];
  // An order no declared module has: the filter must be catalog membership,
  // not a numeric ceiling that would also silently clip the last real module.
  const undeclared = MODULE_CATALOG[MODULE_CATALOG.length - 1].order + 900;
  const store = makeStore();
  const saved = saveLastWeak([mid.order, first.order, undeclared, first.order], {
    store,
    atMs: 1000,
    source: "mock",
  });
  assert(WEAK_STORAGE_KEY === "cdcp.last_weak.v1", "WEAK_STORAGE_KEY");
  assert(
    JSON.stringify(saved.weak_modules) ===
      JSON.stringify([first.order, mid.order]),
    "weak modules sorted + deduped + filtered to declared catalog orders"
  );
  assert(
    saved.weak_modules.indexOf(undeclared) === -1,
    "weak module " + undeclared + " is dropped: no such module is declared"
  );
  const loaded = loadLastWeak({ store });
  assert(
    JSON.stringify(loaded.weak_modules) ===
      JSON.stringify([first.order, mid.order]),
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
  const firstEntry = MODULE_CATALOG[0];
  const weakEntry = MODULE_CATALOG[Math.floor(MODULE_CATALOG.length / 2)];

  // Empty mastery + one weak module → recommend that module's learn link
  saveLastWeak([weakEntry.order], { store, source: "mock" });
  let rec = recommendNext({ store });
  assert(rec.kind === "weak", "recommend kind=weak when last_weak set");
  assert(rec.module === weakEntry.order, "recommend first weak module " + weakEntry.order);
  assert(rec.href === weakEntry.learnHref, "weak → learn href");
  assert(existsSync(join(WEB, rec.href)), "weak recommend href exists on disk");
  assert(!/certif/i.test(rec.reason + rec.label), "weak reason no cert language");

  // Clear weak; nothing practiced → first unpracticed is the first catalog entry
  saveLastWeak([], { store });
  rec = recommendNext({ store });
  assert(rec.kind === "unpracticed", "recommend unpracticed when no weak");
  assert(rec.module === firstEntry.order, "first unpracticed is module " + firstEntry.order);
  assert(rec.href === firstEntry.quizHref, "unpracticed → quiz href");

  // Practice every declared module once at 80% → unmastered (need 90%×2 spaced)
  for (const e of MODULE_CATALOG) {
    recordQuizResult(
      { module: e.order, correct: 8, total: 10, atMs: 2_000_000 + e.order },
      { store }
    );
  }
  rec = recommendNext({ store });
  assert(rec.kind === "unmastered", "recommend unmastered when all practiced");
  assert(rec.module === firstEntry.order, "first unmastered is module " + firstEntry.order);
  assert(/^quiz\.html\?module=\d+$/.test(rec.href), "unmastered → quiz");

  // Master every declared module (two 90%+ attempts ≥24h apart). If any module
  // were missing from the catalog this sweep would skip it and the recommend
  // below would never reach all_practiced.
  const t0 = 10_000_000;
  for (const e of MODULE_CATALOG) {
    recordQuizResult(
      { module: e.order, correct: 9, total: 10, atMs: t0 + e.order },
      { store }
    );
    recordQuizResult(
      { module: e.order, correct: 10, total: 10, atMs: t0 + DAY_MS + e.order },
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
  // The copy states the real curriculum range, not a remembered one.
  assert(
    rec.reason.indexOf(
      "(" +
        MODULE_CATALOG[0].order +
        "-" +
        MODULE_CATALOG[MODULE_CATALOG.length - 1].order +
        ")"
    ) !== -1,
    "all_practiced copy names the derived module range, not a literal: " + rec.reason
  );
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

// Origin contract (bd-hop9): hub is local HTTP; file:// is CDCP_FILE_ORIGIN.
// Wired here so the existing check.sh hub-mastery step exercises it without
// adding a new counted check.sh step while bd-1sd.13 owns the step ledger.
{
  const originSmoke = join(ROOT, "scripts/smoke_file_origin.mjs");
  if (!existsSync(originSmoke)) {
    abort("missing scripts/smoke_file_origin.mjs — origin contract unenforced");
  }
  const spawned = spawnSync(process.execPath, [originSmoke], {
    encoding: "utf8",
    cwd: ROOT,
  });
  if (spawned.status !== 0) {
    if (spawned.stdout) process.stdout.write(spawned.stdout);
    if (spawned.stderr) process.stderr.write(spawned.stderr);
    assert(
      false,
      "origin contract (smoke_file_origin.mjs) failed with status " +
        spawned.status
    );
  } else {
    assert(true, "origin contract (smoke_file_origin.mjs)");
  }
}

if (failed > 0) {
  console.error("\nsmoke_hub_mastery: " + failed + " failure(s)");
  process.exit(1);
}
console.log(
  "\nsmoke_hub_mastery: PASS (" + MODULE_CATALOG.length + " declared modules)"
);
process.exit(0);
