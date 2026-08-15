/**
 * CDCP grade bridge — browser + headless glue for WASM GradeExact.
 *
 * API
 * ---
 *   import {
 *     loadWasm, gradeDigest, isWasmReady, ENGINE_IDENTITY_SUBJECT,
 *   } from "./grade_bridge.js";
 *
 *   await loadWasm("/assets/wasm/cdcp_wasm.wasm"); // optional path
 *   const digest = await gradeDigest(bankJson, attemptJson);
 *
 * bankJson  — JSON array of BankItem, or {"items":[...]} (same as cdcp_wasm::grade_digest_json).
 *             Prefer the export-web file: web/data/bank_items_seed42.json (full bank so
 *             bank_hash matches `cdcp bank-hash` and grade goldens).
 * attemptJson — ExamAttempt: { exam_id, seed, bank_hash, answers: [{item_id, chosen}, ...] }
 *               bank_hash MUST equal the bank loaded from bankJson.
 *
 * Answer-key policy
 * -----------------
 * - Learner pack (web/data/mock40_seed42.json): NO correct letters — UI only.
 * - Keys (web/data/keys_seed42.json): e2e/harness + post-grade explanations only.
 * - bank_items_*.json: full BankItem rows (includes correct) required for client-side grade.
 *
 * Native / headless e2e without WASM
 * ---------------------------------
 * If WASM is not built or loadWasm fails, gradeDigest rejects. Headless CI should use
 * the Rust oracle instead:
 *
 *   cargo run -q -p cdcp_cli -- grade \
 *     --fixture goldens/fixtures/mock40_seed42.json --mode all-correct
 *
 *   cargo run -q -p cdcp_cli -- goldens check
 *
 * Build + install WASM artifact
 * -----------------------------
 *   ./scripts/build_web_wasm.sh          # release → web/assets/wasm/
 *   ./scripts/build_web_wasm.sh --debug  # faster compile
 *
 * Or manually:
 *   rustup target add wasm32-unknown-unknown
 *   cargo build -p cdcp_wasm --target wasm32-unknown-unknown --release
 *   mkdir -p web/assets/wasm
 *   cp target/wasm32-unknown-unknown/release/cdcp_wasm.wasm web/assets/wasm/
 *
 * Headless digest smoke (Node 18+):
 *   node scripts/smoke_results_wasm.mjs
 *
 * Linear-memory ABI (cdcp_wasm wasm32):
 *   cdcp_alloc(len) -> ptr
 *   write UTF-8 into linear memory at ptr
 *   rc = cdcp_grade_digest(bank_ptr, bank_len, att_ptr, att_len)
 *   rc >= 0  → hex digest length; bytes at cdcp_last_ptr()
 *   rc <  0  → error UTF-8 length is -rc; bytes at cdcp_last_ptr()
 *   cdcp_free(ptr, len) for input buffers
 *
 * @module grade_bridge
 */

/** Subject identity label (pairs with native `cdcp_grade-native`). */
export const ENGINE_IDENTITY_SUBJECT = "cdcp_wasm-wasm32";

/** Default relative path for the shipped wasm artifact under the static web root. */
export const DEFAULT_WASM_URL = "assets/wasm/cdcp_wasm.wasm";

/** @type {WebAssembly.Instance | null} */
let _instance = null;
/** @type {WebAssembly.Memory | null} */
let _memory = null;

/**
 * @returns {boolean} true when loadWasm succeeded and exports are present.
 */
export function isWasmReady() {
  return !!(
    _instance &&
    _memory &&
    typeof _instance.exports.cdcp_grade_digest === "function" &&
    typeof _instance.exports.cdcp_next_interval_days === "function" &&
    typeof _instance.exports.cdcp_is_mastered === "function"
  );
}

/** Live wasm exports. Throws if not loaded. */
export function wasmExports() {
  if (!isWasmReady()) {
    throw new Error("WASM not loaded. Call await loadWasm() first.");
  }
  return _instance.exports;
}

/** Live linear memory. Throws if not loaded. */
export function wasmMemory() {
  if (!_memory) {
    throw new Error("WASM not loaded. Call await loadWasm() first.");
  }
  return _memory;
}

/**
 * Drop a previously loaded instance (tests / hot reload).
 */
export function resetWasm() {
  _instance = null;
  _memory = null;
}

/**
 * Load and instantiate the cdcp_wasm module.
 *
 * @param {string} [url] — path or URL to cdcp_wasm.wasm
 * @returns {Promise<WebAssembly.Instance>}
 */
export async function loadWasm(url = DEFAULT_WASM_URL) {
  if (isWasmReady()) {
    return _instance;
  }

  const imports = {
    // Current cdcp_wasm guest needs no host imports; keep empty env for forward-compat.
    env: {},
  };

  let result;
  if (typeof fetch === "function") {
    try {
      const resp = await fetch(url);
      if (!resp.ok) {
        throw new Error(`fetch wasm ${url}: HTTP ${resp.status}`);
      }
      if (typeof WebAssembly.instantiateStreaming === "function") {
        try {
          result = await WebAssembly.instantiateStreaming(resp, imports);
        } catch {
          // MIME/type or streaming issues → fall back to ArrayBuffer path.
          const buf = await resp.arrayBuffer();
          result = await WebAssembly.instantiate(buf, imports);
        }
      } else {
        const buf = await resp.arrayBuffer();
        result = await WebAssembly.instantiate(buf, imports);
      }
    } catch (e) {
      // Node / file path fallback when fetch is missing or failed.
      result = await instantiateFromPath(url, imports, e);
    }
  } else {
    result = await instantiateFromPath(url, imports, null);
  }

  _instance = result.instance;
  _memory = /** @type {WebAssembly.Memory} */ (_instance.exports.memory);
  if (!_memory) {
    resetWasm();
    throw new Error("cdcp_wasm missing export `memory`");
  }
  assertExports(_instance);
  return _instance;
}

/**
 * @param {string} url
 * @param {object} imports
 * @param {unknown} priorErr
 */
async function instantiateFromPath(url, imports, priorErr) {
  // Node 18+: read file when `fs` is available (headless smoke without a browser).
  try {
    const fs = await import("node:fs/promises");
    const buf = await fs.readFile(url);
    return await WebAssembly.instantiate(buf, imports);
  } catch (e) {
    const detail = priorErr ? `; fetch error: ${errMsg(priorErr)}` : "";
    throw new Error(
      `loadWasm failed for ${url}: ${errMsg(e)}${detail}. ` +
        `Build with: cargo build -p cdcp_wasm --target wasm32-unknown-unknown ` +
        `and copy cdcp_wasm.wasm to web/assets/wasm/. ` +
        `Headless e2e without wasm: cargo run -p cdcp_cli -- grade …`
    );
  }
}

/** @param {WebAssembly.Instance} instance */
function assertExports(instance) {
  const need = [
    "cdcp_alloc",
    "cdcp_free",
    "cdcp_last_ptr",
    "cdcp_last_len",
    "cdcp_grade_digest",
    "cdcp_schedule_ok",
    "cdcp_interval_step_count",
    "cdcp_interval_step",
    "cdcp_next_interval_days",
    "cdcp_day_ms",
    "cdcp_practiced_milli",
    "cdcp_mastered_milli",
    "cdcp_mastered_min_gap_ms",
    "cdcp_is_practiced",
    "cdcp_is_mastered",
    "memory",
  ];
  for (const name of need) {
    if (!(name in instance.exports)) {
      throw new Error(`cdcp_wasm missing export \`${name}\``);
    }
  }
}

/**
 * Grade via WASM: SHA-256 hex digest of canonical GradeReport JSON.
 * Same contract as cdcp_wasm::grade_digest_json / native grade_digest.
 *
 * @param {string} bankJson
 * @param {string} attemptJson
 * @returns {Promise<string>} lowercase hex digest (64 chars)
 */
export async function gradeDigest(bankJson, attemptJson) {
  if (typeof bankJson !== "string" || typeof attemptJson !== "string") {
    throw new TypeError("gradeDigest(bankJson, attemptJson) expects two strings");
  }
  if (!isWasmReady()) {
    throw new Error(
      "WASM not loaded. Call await loadWasm() first, or use CLI oracle: " +
        "`cargo run -p cdcp_cli -- grade --fixture … --mode all-correct`"
    );
  }

  const ex = _instance.exports;
  const bankBytes = new TextEncoder().encode(bankJson);
  const attBytes = new TextEncoder().encode(attemptJson);

  const bankPtr = ex.cdcp_alloc(bankBytes.length);
  const attPtr = ex.cdcp_alloc(attBytes.length);
  if (!bankPtr || !attPtr) {
    throw new Error("cdcp_alloc returned null/0");
  }

  try {
    const mem = new Uint8Array(_memory.buffer);
    mem.set(bankBytes, bankPtr);
    mem.set(attBytes, attPtr);

    const rc = ex.cdcp_grade_digest(
      bankPtr,
      bankBytes.length,
      attPtr,
      attBytes.length
    );

    // Re-bind memory view in case the guest grew linear memory.
    const mem2 = new Uint8Array(_memory.buffer);
    const lastPtr = ex.cdcp_last_ptr();
    if (rc >= 0) {
      const len = rc;
      const slice = mem2.slice(lastPtr, lastPtr + len);
      return new TextDecoder().decode(slice);
    }
    const errLen = -rc;
    const slice = mem2.slice(lastPtr, lastPtr + errLen);
    const msg = new TextDecoder().decode(slice);
    throw new Error(`grade_digest: ${msg}`);
  } finally {
    try {
      ex.cdcp_free(bankPtr, bankBytes.length);
    } catch {
      /* ignore */
    }
    try {
      ex.cdcp_free(attPtr, attBytes.length);
    } catch {
      /* ignore */
    }
  }
}

/**
 * Convenience: parse JSON values then grade.
 *
 * @param {object|Array} bank
 * @param {object} attempt
 * @returns {Promise<string>}
 */
export async function gradeDigestValue(bank, attempt) {
  return gradeDigest(JSON.stringify(bank), JSON.stringify(attempt));
}

/** @param {unknown} e */
function errMsg(e) {
  if (e instanceof Error) return e.message;
  return String(e);
}

// UMD-ish global for non-module script tags (optional).
if (typeof globalThis !== "undefined") {
  globalThis.CdcpGradeBridge = {
    loadWasm,
    gradeDigest,
    gradeDigestValue,
    isWasmReady,
    resetWasm,
    wasmExports,
    wasmMemory,
    ENGINE_IDENTITY_SUBJECT,
    DEFAULT_WASM_URL,
  };
}
