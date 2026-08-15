/**
 * learn_reader.js — load module markdown and render offline.
 *
 * Resolution order:
 *   1. #module-md embed (no fetch; still requires local HTTP for sibling JS)
 *   2. data-content-href on #module-prose (web/content/modules/{id}.md)
 *   3. Fallback relative paths to parent corpus when serving repo root
 *
 * Progress: CdcpLearn.markVisited(moduleId) via localStorage.
 */
(function (global) {
  "use strict";

  function candidates(moduleId, preferred) {
    var list = [];
    if (preferred) list.push(preferred);
    if (moduleId) {
      list.push("../content/modules/" + moduleId + ".md");
      // Serving cdcp-self-study/ as static root:
      //   /course-engine/web/learn/{id}.html → ../../../modules/{id}.md
      list.push("../../../modules/" + moduleId + ".md");
      // Serving course-engine/ as static root:
      //   /web/learn/{id}.html → ../../modules/{id}.md
      list.push("../../modules/" + moduleId + ".md");
    }
    var seen = Object.create(null);
    var out = [];
    for (var i = 0; i < list.length; i++) {
      var h = list[i];
      if (!h || seen[h]) continue;
      seen[h] = true;
      out.push(h);
    }
    return out;
  }

  function fetchFirst(urls) {
    if (!urls.length) {
      return Promise.reject(new Error("no urls"));
    }
    if (typeof fetch !== "function") {
      return Promise.reject(new Error("fetch unavailable"));
    }
    var i = 0;
    function next() {
      if (i >= urls.length) {
        return Promise.reject(new Error("all fetch paths failed"));
      }
      var href = urls[i++];
      return fetch(href).then(function (r) {
        if (!r.ok) return next();
        return r.text().then(function (text) {
          if (!text || !String(text).trim()) return next();
          return text;
        });
      }).catch(function () {
        return next();
      });
    }
    return next();
  }

  function loadAndRender(moduleId) {
    var prose = document.getElementById("module-prose");
    var srcEl = document.getElementById("module-md");
    var md = srcEl ? String(srcEl.textContent || "") : "";
    // Strip common leading newline from pretty-printed embeds
    if (md.charAt(0) === "\n") md = md.slice(1);

    function scrollToHash() {
      // Deep links from results: learn/{slug}.html#section-id (L7-S2).
      // Content is async; re-apply location.hash after headings exist.
      var raw = (global.location && global.location.hash) || "";
      if (!raw || raw.length < 2) return;
      var id = decodeURIComponent(raw.slice(1));
      if (!id) return;
      var target =
        (global.document && global.document.getElementById(id)) || null;
      if (!target) return;
      try {
        target.scrollIntoView({ block: "start", behavior: "auto" });
      } catch (_) {
        target.scrollIntoView(true);
      }
      // Prefer focus for a11y without stealing when already focused.
      if (typeof target.focus === "function") {
        try {
          target.setAttribute("tabindex", "-1");
          target.focus({ preventScroll: true });
        } catch (_) {
          /* ignore */
        }
      }
    }

    function done(text) {
      if (global.CdcpLearnMd && prose) {
        global.CdcpLearnMd.renderInto(prose, text);
        prose.setAttribute("aria-busy", "false");
      }
      if (global.CdcpLearn && moduleId) {
        global.CdcpLearn.markVisited(moduleId);
      }
      // Defer so layout has ids painted.
      function afterPaint() {
        scrollToHash();
        if (global.CdcpLearnChrome && global.CdcpLearnChrome.afterRender) {
          global.CdcpLearnChrome.afterRender();
        }
        if (global.CdcpLearnUnits && global.CdcpLearnUnits.mount && moduleId) {
          global.CdcpLearnUnits.mount(moduleId);
        }
        if (global.CdcpLearnGlossary && global.CdcpLearnGlossary.enhance) {
          global.CdcpLearnGlossary.enhance(
            global.document.getElementById("module-prose")
          );
        }
      }
      if (typeof global.requestAnimationFrame === "function") {
        global.requestAnimationFrame(afterPaint);
      } else {
        setTimeout(afterPaint, 0);
      }
    }

    function fail(msg) {
      if (prose) {
        prose.innerHTML =
          '<p class="lede">' +
          msg +
          "</p><p class=\"meta\">Run <span class=\"mono\">cdcp build-learn</span> " +
          "from course-engine to copy notes into <span class=\"mono\">web/content/modules/</span>, " +
          "then serve <span class=\"mono\">web/</span> (see web/README.md).</p>";
        prose.setAttribute("aria-busy", "false");
      }
    }

    if (md && md.trim().length > 0) {
      done(md);
      return;
    }

    var preferred =
      (prose && prose.getAttribute("data-content-href")) || null;
    var urls = candidates(moduleId, preferred);

    fetchFirst(urls)
      .then(done)
      .catch(function () {
        var loc = global.location;
        if (loc && loc.protocol === "file:") {
          fail(
            "CDCP_FILE_ORIGIN: cannot fetch module notes over file://. " +
              "Serve with <span class=\"mono\">cdcp serve</span> " +
              "and open <span class=\"mono\">http://127.0.0.1:8766/</span>."
          );
          return;
        }
        fail(
          "Could not load module notes for " +
            (moduleId || "unknown") +
            " via relative paths."
        );
      });
  }

  global.CdcpLearnReader = { loadAndRender: loadAndRender };
})(typeof window !== "undefined" ? window : globalThis);
