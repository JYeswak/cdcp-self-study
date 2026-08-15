#!/usr/bin/env node
/**
 * Origin-contract smoke (bd-hop9).
 *
 * Usage (from course-engine/):
 *   node scripts/smoke_file_origin.mjs
 *
 * Product contract: the hub is local HTTP (`cdcp_cli serve`).
 * file:// is not a supported origin.
 *
 * Exit 0 only if:
 *   - origin_guard.js is a classic script that names CDCP_FILE_ORIGIN
 *   - executing it under file:// paints a banner with that code
 *   - executing it under http:/https: paints nothing
 *   - every learner-facing HTML page includes the guard as a classic
 *     (NOT type=module) script — a module script would not run on file://
 *   - the hub lede, web/README.md, and CHARTER shipped_means state the
 *     contract in words a stranger cannot misread
 *
 * Exit 2 = structural ERROR (nothing to check). Never a pass.
 * Exit 1 = the contract is stated or wired incorrectly.
 *
 * Planted known-bad (must go RED inside this process):
 *   - HTML that omits the guard
 *   - HTML that loads the guard as type=module
 */
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { createContext, runInContext } from "node:vm";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, "..");
const WEB = join(ROOT, "web");
const GUARD = join(WEB, "assets/js/origin_guard.js");
const CHARTER = join(ROOT, "..", "CHARTER.md");
const WEB_README = join(WEB, "README.md");
const CODE = "CDCP_FILE_ORIGIN";

let failed = 0;
let checks = 0;

function abort(msg) {
  console.error("ERROR: " + msg);
  console.error("\nsmoke_file_origin: NOT RUN (structural)");
  process.exit(2);
}

function fail(msg) {
  failed += 1;
  checks += 1;
  console.error("  FAIL: " + msg);
}

function ok(msg) {
  checks += 1;
  console.log("  ok: " + msg);
}

if (!existsSync(GUARD)) {
  abort("missing web/assets/js/origin_guard.js — the named-error script is the product");
}
if (!existsSync(join(WEB, "index.html"))) {
  abort("missing web/index.html — empty scan set is an ERROR");
}

const guardSrc = readFileSync(GUARD, "utf8");
if (/^\s*(import|export)\b/m.test(guardSrc)) {
  abort("origin_guard.js must not be an ES module — file:// would block it");
}
if (!guardSrc.includes(CODE)) {
  abort("origin_guard.js does not name " + CODE);
}
if (!guardSrc.includes("diagnoseOrigin")) {
  abort("origin_guard.js missing diagnoseOrigin");
}
ok("origin_guard.js present, classic, names " + CODE);

function mockDocument() {
  const children = [];
  const htmlAttrs = Object.create(null);
  const body = {
    firstChild: null,
    insertBefore: function (el, _ref) {
      children.unshift(el);
      this.firstChild = el;
      return el;
    },
    appendChild: function (el) {
      children.push(el);
      if (!this.firstChild) this.firstChild = el;
      return el;
    },
  };
  const doc = {
    documentElement: {
      setAttribute: function (k, v) {
        htmlAttrs[k] = String(v);
      },
      removeAttribute: function (k) {
        delete htmlAttrs[k];
      },
    },
    body: body,
    getElementById: function (id) {
      for (let i = 0; i < children.length; i++) {
        if (children[i].id === id) return children[i];
      }
      return null;
    },
    createElement: function (_tag) {
      return {
        id: "",
        className: "",
        innerHTML: "",
        style: { cssText: "" },
        attrs: Object.create(null),
        setAttribute: function (k, v) {
          this.attrs[k] = String(v);
        },
      };
    },
    addEventListener: function () {},
  };
  return { doc: doc, children: children, htmlAttrs: htmlAttrs };
}

function runGuard(protocol) {
  const mock = mockDocument();
  const sandbox = {
    location: { protocol: protocol },
    document: mock.doc,
  };
  createContext(sandbox);
  runInContext(guardSrc, sandbox, { filename: "origin_guard.js" });
  return { sandbox: sandbox, mock: mock };
}

{
  const fileRun = runGuard("file:");
  const api = fileRun.sandbox.CdcpOriginGuard;
  if (!api || typeof api.diagnoseOrigin !== "function") {
    fail("file:// boot did not install CdcpOriginGuard");
  } else {
    const d = api.diagnoseOrigin("file:");
    if (!d || d.ok || d.code !== CODE) {
      fail("diagnoseOrigin('file:') must be { ok:false, code:" + CODE + " }");
    } else {
      ok("diagnoseOrigin(file:) is " + CODE);
    }
    if (!String(d && d.message).includes(CODE)) {
      fail("file:// message must name " + CODE);
    } else {
      ok("file:// message names " + CODE);
    }
    if (!/cdcp_cli -- serve|127\.0\.0\.1:8766/.test(String(d && d.message))) {
      fail("file:// message must name the serve path");
    } else {
      ok("file:// message names cdcp_cli serve");
    }
  }
  const banner = fileRun.mock.doc.getElementById("cdcp-file-origin");
  if (!banner) {
    fail("file:// must paint #cdcp-file-origin (not a silent blank dashboard)");
  } else if (!String(banner.innerHTML).includes(CODE)) {
    fail("painted banner must contain " + CODE);
  } else {
    ok("file:// paints visible " + CODE + " banner");
  }
  if (fileRun.mock.htmlAttrs["data-cdcp-origin-error"] !== CODE) {
    fail("file:// must set data-cdcp-origin-error=" + CODE);
  } else {
    ok("file:// sets data-cdcp-origin-error");
  }
}

{
  const httpRun = runGuard("http:");
  if (httpRun.mock.doc.getElementById("cdcp-file-origin")) {
    fail("http: must not paint the file-origin banner");
  } else {
    ok("http: paints no file-origin banner");
  }
  const d = httpRun.sandbox.CdcpOriginGuard.diagnoseOrigin("http:");
  if (!d || !d.ok) {
    fail("diagnoseOrigin('http:') must be ok");
  } else {
    ok("diagnoseOrigin(http:) is ok");
  }
}

{
  const httpsRun = runGuard("https:");
  if (httpsRun.mock.doc.getElementById("cdcp-file-origin")) {
    fail("https: must not paint the file-origin banner");
  } else {
    ok("https: paints no file-origin banner");
  }
}

function hasClassicGuard(html, src) {
  if (!html || typeof html !== "string") return false;
  const escaped = src.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  if (
    new RegExp(
      "<script[^>]*\\btype\\s*=\\s*[\"']module[\"'][^>]*\\bsrc\\s*=\\s*[\"']" +
        escaped +
        "[\"']",
      "i"
    ).test(html)
  ) {
    return false;
  }
  if (
    new RegExp(
      "<script[^>]*\\bsrc\\s*=\\s*[\"']" +
        escaped +
        "[\"'][^>]*\\btype\\s*=\\s*[\"']module[\"']",
      "i"
    ).test(html)
  ) {
    return false;
  }
  return new RegExp(
    "<script(?![^>]*\\btype\\s*=\\s*[\"']module[\"'])[^>]*\\bsrc\\s*=\\s*[\"']" +
      escaped +
      "[\"']",
    "i"
  ).test(html);
}

function assertClassicGuard(html, src, label) {
  if (!hasClassicGuard(html, src)) {
    fail(label + " missing classic <script src=\"" + src + "\"> (not type=module)");
  } else {
    ok(label + " includes classic origin_guard.js");
  }
}

// Planted known-bad: omitting the guard, or loading it as a module, must RED.
{
  const missing = "<!doctype html><html><head></head><body>hub</body></html>";
  if (hasClassicGuard(missing, "assets/js/origin_guard.js")) {
    fail("planted HTML without the guard was accepted — detector is vacuous");
  } else {
    ok("planted HTML without origin_guard.js is RED");
  }
  const asModule =
    '<script type="module" src="assets/js/origin_guard.js"></script>';
  if (hasClassicGuard(asModule, "assets/js/origin_guard.js")) {
    fail("planted type=module guard was accepted — that would not run on file://");
  } else {
    ok("planted type=module origin_guard.js is RED");
  }
}

const TOP_PAGES = [
  "index.html",
  "learn.html",
  "drill.html",
  "mock.html",
  "quiz.html",
  "results.html",
  "reference.html",
  "runbooks.html",
];

for (let i = 0; i < TOP_PAGES.length; i++) {
  const rel = TOP_PAGES[i];
  const path = join(WEB, rel);
  if (!existsSync(path)) {
    fail("missing web/" + rel);
    continue;
  }
  assertClassicGuard(
    readFileSync(path, "utf8"),
    "assets/js/origin_guard.js",
    "web/" + rel
  );
}

const learnDir = join(WEB, "learn");
if (!existsSync(learnDir)) {
  abort("missing web/learn/ — empty scan set is an ERROR");
}
const learnPages = readdirSync(learnDir)
  .filter(function (n) {
    return n.endsWith(".html");
  })
  .sort();
if (learnPages.length === 0) {
  abort("web/learn/ has zero HTML pages — empty scan set is an ERROR");
}
for (let j = 0; j < learnPages.length; j++) {
  const name = learnPages[j];
  assertClassicGuard(
    readFileSync(join(learnDir, name), "utf8"),
    "../assets/js/origin_guard.js",
    "web/learn/" + name
  );
}
ok("scanned " + learnPages.length + " learn pages");

{
  const hub = readFileSync(join(WEB, "index.html"), "utf8");
  if (!hub.includes(CODE)) {
    fail("web/index.html lede/copy must name " + CODE + " at first contact");
  } else {
    ok("web/index.html names " + CODE);
  }
  if (!/cdcp_cli -- serve|127\.0\.0\.1:8766/.test(hub)) {
    fail("web/index.html must name the serve path at first contact");
  } else {
    ok("web/index.html names the serve path");
  }
  if (/Self-study · offline/.test(hub)) {
    fail("web/index.html still says 'Self-study · offline' — that is the ambiguity");
  } else {
    ok("web/index.html brand no longer says bare 'offline'");
  }
}

{
  if (!existsSync(WEB_README)) {
    fail("missing web/README.md");
  } else {
    const readme = readFileSync(WEB_README, "utf8");
    const head = readme.slice(0, 1200);
    if (!head.includes(CODE)) {
      fail("web/README.md must name " + CODE + " at first contact");
    } else {
      ok("web/README.md names " + CODE + " at first contact");
    }
    if (!/cdcp_cli -- serve/.test(head)) {
      fail("web/README.md must document `cdcp_cli serve` at first contact");
    } else {
      ok("web/README.md documents cdcp_cli serve at first contact");
    }
    if (!/file:\/\//.test(head)) {
      fail("web/README.md must say file:// is unsupported at first contact");
    } else {
      ok("web/README.md says file:// is unsupported at first contact");
    }
  }
}

{
  if (!existsSync(CHARTER)) {
    fail("missing parent CHARTER.md — shipped_means is the contract");
  } else {
    const charter = readFileSync(CHARTER, "utf8");
    if (!charter.includes(CODE)) {
      fail("CHARTER.md shipped_means must name " + CODE);
    } else {
      ok("CHARTER.md names " + CODE);
    }
    if (!/cdcp_cli -- serve|local-server hub|local HTTP/.test(charter)) {
      fail("CHARTER.md shipped_means must say local-server / cdcp_cli serve");
    } else {
      ok("CHARTER.md shipped_means is local-server, not file://");
    }
    if (/Open offline hub/.test(charter)) {
      fail("CHARTER.md still says 'Open offline hub' — stranger can still misread it");
    } else {
      ok("CHARTER.md no longer says bare 'Open offline hub'");
    }
  }
}

if (checks < 20) {
  abort(
    "performed only " +
      checks +
      " checks — a short run is an ERROR, not a pass"
  );
}

if (failed > 0) {
  console.error("\nsmoke_file_origin: " + failed + " failure(s) / " + checks + " checks");
  process.exit(1);
}
console.log(
  "\nsmoke_file_origin: PASS (" +
    checks +
    " checks · " +
    learnPages.length +
    " learn pages · contract=local-http · file://=" +
    CODE +
    ")"
);
process.exit(0);
