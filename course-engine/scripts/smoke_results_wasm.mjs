#!/usr/bin/env node
/**
 * Headless smoke: grade_bridge + cdcp_wasm vs goldens (L5 dual-path check).
 *
 * Usage (from course-engine/):
 *   ./scripts/build_web_wasm.sh --debug   # or release
 *   node scripts/smoke_results_wasm.mjs
 *   node scripts/smoke_results_wasm.mjs --golden-dir /tmp/alt_goldens
 *
 * Optional env:
 *   CDCP_WASM_PATH=path/to/cdcp_wasm.wasm
 *   CDCP_GOLDEN_DIR=path/to/dir containing mock40_seed42_{all_correct,all_wrong}.sha256
 *   CDCP_BANK_JSON / CDCP_KEYS_JSON  — override fixture paths
 *
 * Exit 0 only if all-correct and all-wrong digests match goldens.
 * On mismatch prints "GOLDEN MISMATCH" (selftest needle).
 * Zero / missing fixtures → non-zero (no vacuous green).
 */
import { readFileSync, existsSync } from "node:fs";
import { fileURLToPath, pathToFileURL } from "node:url";
import { dirname, join, resolve } from "node:path";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, "..");

function usage() {
  console.error(
    "usage: node scripts/smoke_results_wasm.mjs [--golden-dir DIR] [--help]"
  );
}

function parseArgs(argv) {
  const out = { goldenDir: null };
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === "--help" || a === "-h") {
      usage();
      process.exit(0);
    }
    if (a === "--golden-dir") {
      const v = argv[++i];
      if (!v) {
        console.error("smoke_results_wasm: --golden-dir requires a path");
        process.exit(2);
      }
      out.goldenDir = v;
      continue;
    }
    if (a.startsWith("--golden-dir=")) {
      out.goldenDir = a.slice("--golden-dir=".length);
      continue;
    }
    console.error(`smoke_results_wasm: unknown arg: ${a}`);
    usage();
    process.exit(2);
  }
  return out;
}

const args = parseArgs(process.argv.slice(2));

// Dynamic import of grade_bridge + results helpers (ESM under web/)
const bridgePath = pathToFileURL(join(ROOT, "web/assets/js/grade_bridge.js")).href;
const { loadWasm, gradeDigest, resetWasm } = await import(bridgePath);

const resultsPath = pathToFileURL(join(ROOT, "web/assets/js/results.js")).href;
const { buildAllCorrectAttempt, buildAllWrongAttempt } = await import(resultsPath);

function read(p) {
  return readFileSync(p, "utf8").trim();
}

function die(msg, code = 2) {
  console.error(`smoke_results_wasm: ERROR: ${msg}`);
  process.exit(code);
}

function resolveWasm() {
  if (process.env.CDCP_WASM_PATH) return process.env.CDCP_WASM_PATH;
  const candidates = [
    join(ROOT, "web/assets/wasm/cdcp_wasm.wasm"),
    join(ROOT, "target/wasm32-unknown-unknown/release/cdcp_wasm.wasm"),
    join(ROOT, "target/wasm32-unknown-unknown/debug/cdcp_wasm.wasm"),
  ];
  for (const c of candidates) {
    if (existsSync(c)) return c;
  }
  die(
    "No cdcp_wasm.wasm found. Run ./scripts/build_web_wasm.sh first. " +
      "(missing wasm is ERROR, not vacuous skip)"
  );
}

const goldenDir = resolve(
  args.goldenDir ||
    process.env.CDCP_GOLDEN_DIR ||
    join(ROOT, "goldens")
);

const bankPath =
  process.env.CDCP_BANK_JSON || join(ROOT, "web/data/bank_items_seed42.json");
const keysPath =
  process.env.CDCP_KEYS_JSON || join(ROOT, "web/data/keys_seed42.json");
const goldenCorrectPath = join(goldenDir, "mock40_seed42_all_correct.sha256");
const goldenWrongPath = join(goldenDir, "mock40_seed42_all_wrong.sha256");

// Anti-vacuous: every required fixture must exist (zero fixtures = ERROR).
const required = [
  ["bank_json", bankPath],
  ["keys_json", keysPath],
  ["golden_all_correct", goldenCorrectPath],
  ["golden_all_wrong", goldenWrongPath],
];
const missing = required.filter(([, p]) => !existsSync(p));
if (missing.length > 0) {
  for (const [label, p] of missing) {
    console.error(`smoke_results_wasm: missing fixture ${label}: ${p}`);
  }
  die(
    `zero/missing fixtures (${missing.length}/${required.length} absent) — refusing vacuous green`
  );
}

const bankJson = readFileSync(bankPath, "utf8");
const keysPack = JSON.parse(readFileSync(keysPath, "utf8"));
if (!keysPack || !Array.isArray(keysPack.keys) || keysPack.keys.length === 0) {
  die("keys fixture empty or missing keys[] — refusing vacuous green");
}

const goldenCorrect = read(goldenCorrectPath);
const goldenWrong = read(goldenWrongPath);
if (!/^[0-9a-f]{64}$/i.test(goldenCorrect) || !/^[0-9a-f]{64}$/i.test(goldenWrong)) {
  die("golden digest pin must be 64 hex chars");
}

const wasmPath = resolveWasm();
console.log("smoke_results_wasm: wasm =", wasmPath);
console.log("smoke_results_wasm: golden_dir =", goldenDir);
console.log("smoke_results_wasm: bank =", bankPath);
console.log("smoke_results_wasm: keys =", keysPath, `(n=${keysPack.keys.length})`);

resetWasm();
await loadWasm(wasmPath);

const allCorrect = buildAllCorrectAttempt(keysPack);
const allWrong = buildAllWrongAttempt(keysPack);

if (!allCorrect.answers || allCorrect.answers.length === 0) {
  die("built attempt has zero answers — refusing vacuous green");
}

const digCorrect = await gradeDigest(bankJson, JSON.stringify(allCorrect));
const digWrong = await gradeDigest(bankJson, JSON.stringify(allWrong));

console.log("all-correct digest:", digCorrect);
console.log("all-correct golden:", goldenCorrect);
console.log("all-wrong  digest:", digWrong);
console.log("all-wrong  golden:", goldenWrong);
console.log(
  "attempt bank_hash:",
  allCorrect.bank_hash,
  "answers:",
  allCorrect.answers.length
);

let failed = 0;
if (digCorrect !== goldenCorrect) {
  console.error("GOLDEN MISMATCH: all-correct digest does not match pin");
  console.error("  got:      ", digCorrect);
  console.error("  expected: ", goldenCorrect);
  failed++;
} else {
  console.log("ok: all-correct matches golden");
}
if (digWrong !== goldenWrong) {
  console.error("GOLDEN MISMATCH: all-wrong digest does not match pin");
  console.error("  got:      ", digWrong);
  console.error("  expected: ", goldenWrong);
  failed++;
} else {
  console.log("ok: all-wrong matches golden");
}

if (failed) {
  console.error(`smoke_results_wasm: FAIL (${failed} GOLDEN MISMATCH)`);
  process.exit(1);
}
console.log("smoke_results_wasm: PASS");
console.log("matched digests: all-correct=" + digCorrect);
console.log("matched digests: all-wrong=" + digWrong);
