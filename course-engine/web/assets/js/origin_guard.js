/**
 * origin_guard.js — fail-closed file:// diagnosis (bd-hop9).
 *
 * Classic script (not an ES module). Chrome blocks every ES module on
 * file:// (CORS, origin "null"), so the named error cannot live in the
 * module graph. This file is the only JS that is guaranteed to run when
 * a learner double-clicks web/index.html.
 *
 * Product contract: local HTTP via `cdcp_cli serve`.
 * file:// is not a supported origin.
 *
 * @file
 */
(function (root) {
  "use strict";

  var CODE = "CDCP_FILE_ORIGIN";
  var SERVE_HINT =
    "cargo run -p cdcp_cli -- serve   # then open http://127.0.0.1:8766/";

  function normalizeProtocol(protocol) {
    var p = String(protocol == null ? "" : protocol).toLowerCase();
    if (p && p.charAt(p.length - 1) !== ":") p += ":";
    return p;
  }

  /**
   * Classify an origin protocol against the hub contract.
   *
   * @param {string} protocol location.protocol-style ("file:", "http:", …)
   * @returns {{ ok: boolean, code: string|null, message: string }}
   */
  function diagnoseOrigin(protocol) {
    var p = normalizeProtocol(protocol);
    if (p === "http:" || p === "https:") {
      return { ok: true, code: null, message: "" };
    }
    if (p === "file:") {
      return {
        ok: false,
        code: CODE,
        message:
          CODE +
          ": this hub does not run over file://. Browsers block ES modules, " +
          "fetch(), and WASM from a double-clicked HTML file. Serve locally " +
          "with no network: " +
          SERVE_HINT,
      };
    }
    if (!p) {
      return {
        ok: false,
        code: CODE,
        message:
          CODE +
          ": origin protocol is missing; refusing to boot. Serve locally: " +
          SERVE_HINT,
      };
    }
    return {
      ok: false,
      code: CODE,
      message:
        CODE +
        ": unsupported origin protocol " +
        p +
        ". Supported: local HTTP via `cdcp_cli serve` (http://127.0.0.1:8766/).",
    };
  }

  function escapeHtml(s) {
    return String(s)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }

  /**
   * Paint or clear the named-error banner. Safe to call before <body> exists.
   *
   * @param {Document} doc
   * @param {{ protocol?: string }} loc
   * @returns {{ ok: boolean, code: string|null, message: string } | null}
   */
  function apply(doc, loc) {
    if (!doc) return null;
    var protocol = loc && loc.protocol != null ? loc.protocol : "";
    var diag = diagnoseOrigin(protocol);
    var rootEl = doc.documentElement;

    if (diag.ok) {
      if (rootEl && rootEl.removeAttribute) {
        rootEl.removeAttribute("data-cdcp-origin-error");
      }
      var stale =
        typeof doc.getElementById === "function"
          ? doc.getElementById("cdcp-file-origin")
          : null;
      if (stale && stale.parentNode) stale.parentNode.removeChild(stale);
      return diag;
    }

    if (rootEl && rootEl.setAttribute) {
      rootEl.setAttribute("data-cdcp-origin-error", diag.code);
    }

    function inject() {
      if (!doc.body) return false;
      if (
        typeof doc.getElementById === "function" &&
        doc.getElementById("cdcp-file-origin")
      ) {
        return true;
      }
      if (typeof doc.createElement !== "function") return false;
      var el = doc.createElement("div");
      el.id = "cdcp-file-origin";
      el.className = "origin-banner";
      if (el.setAttribute) {
        el.setAttribute("role", "alert");
        el.setAttribute("data-error", diag.code);
      }
      // Inline styles: diagnosis must remain readable if course.css never
      // loads (the whole point of a file:// fail-closed).
      if (el.style) {
        el.style.cssText =
          "background:#3f1d1d;color:#fecaca;border-bottom:2px solid #f87171;" +
          "padding:0.85rem 1.25rem;font:0.95rem/1.45 system-ui,sans-serif;";
      }
      el.innerHTML =
        "<strong>" +
        escapeHtml(diag.code) +
        "</strong> " +
        escapeHtml(diag.message);
      if (doc.body.insertBefore) {
        doc.body.insertBefore(el, doc.body.firstChild || null);
      } else if (doc.body.appendChild) {
        doc.body.appendChild(el);
      }
      return true;
    }

    if (!inject() && doc.addEventListener) {
      doc.addEventListener("DOMContentLoaded", inject);
    }
    return diag;
  }

  var api = {
    CODE: CODE,
    SERVE_HINT: SERVE_HINT,
    diagnoseOrigin: diagnoseOrigin,
    apply: apply,
  };

  root.CdcpOriginGuard = api;

  if (typeof document !== "undefined") {
    apply(
      document,
      typeof location !== "undefined" ? location : { protocol: "" }
    );
  }
})(typeof globalThis !== "undefined" ? globalThis : this);
