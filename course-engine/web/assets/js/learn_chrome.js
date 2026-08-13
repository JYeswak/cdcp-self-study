/**
 * learn_chrome.js — M8-A1/A4 Learn chrome: sticky TOC, read progress, continue.
 * Offline only. Study signals — not a credential.
 */
(function (global) {
  "use strict";

  var CONTINUE_KEY = "cdcp.learn.continue.v1";

  function setContinue(moduleId, href) {
    if (!moduleId) return;
    try {
      global.localStorage.setItem(
        CONTINUE_KEY,
        JSON.stringify({
          moduleId: moduleId,
          href: href || null,
          ts: Date.now(),
        })
      );
    } catch (e) {
      /* ignore */
    }
  }

  function getContinue() {
    try {
      var raw = global.localStorage.getItem(CONTINUE_KEY);
      if (!raw) return null;
      var o = JSON.parse(raw);
      if (!o || typeof o.moduleId !== "string") return null;
      return o;
    } catch (e) {
      return null;
    }
  }

  function buildToc(prose, nav) {
    if (!prose || !nav) return 0;
    var heads = prose.querySelectorAll("h2, h3");
    if (!heads.length) {
      nav.hidden = true;
      return 0;
    }
    var ul = document.createElement("ul");
    ul.className = "learn-toc__list";
    var count = 0;
    for (var i = 0; i < heads.length; i++) {
      var h = heads[i];
      if (!h.id) continue;
      var li = document.createElement("li");
      li.className =
        "learn-toc__item learn-toc__item--" + h.tagName.toLowerCase();
      var a = document.createElement("a");
      a.href = "#" + h.id;
      a.className = "learn-toc__link";
      a.textContent = (h.textContent || "").trim() || h.id;
      li.appendChild(a);
      ul.appendChild(li);
      count++;
    }
    nav.innerHTML = "";
    var title = document.createElement("p");
    title.className = "learn-toc__title mono";
    title.textContent = "On this page";
    nav.appendChild(title);
    nav.appendChild(ul);
    nav.hidden = count === 0;
    return count;
  }

  function bindProgress(prose, bar) {
    if (!prose || !bar) return;
    var fill = bar.querySelector(".learn-progress-bar__fill");
    var label = bar.querySelector(".learn-progress-bar__label");
    function tick() {
      var rect = prose.getBoundingClientRect();
      var total = prose.scrollHeight - global.innerHeight;
      if (total <= 0) {
        if (fill) fill.style.width = "100%";
        if (label) label.textContent = "100%";
        return;
      }
      var scrolled = global.scrollY - (prose.offsetTop || 0);
      var pct = Math.max(0, Math.min(100, Math.round((scrolled / total) * 100)));
      // Prefer document scroll through article
      var docMax =
        (global.document.documentElement.scrollHeight || 0) -
        (global.innerHeight || 1);
      if (docMax > 0) {
        pct = Math.max(
          0,
          Math.min(100, Math.round((global.scrollY / docMax) * 100))
        );
      }
      if (fill) fill.style.width = pct + "%";
      if (label) label.textContent = pct + "%";
      bar.setAttribute("aria-valuenow", String(pct));
    }
    global.addEventListener("scroll", tick, { passive: true });
    global.addEventListener("resize", tick, { passive: true });
    tick();
  }

  function enhanceModulePage() {
    var body = global.document.body;
    var moduleId =
      (body && body.getAttribute("data-module-id")) ||
      (global.document.getElementById("module-prose") &&
        global.document
          .getElementById("module-prose")
          .getAttribute("data-module-id"));
    var prose = global.document.getElementById("module-prose");
    var toc = global.document.getElementById("learn-toc");
    var bar = global.document.getElementById("learn-progress-bar");

    if (moduleId) {
      var path =
        (global.location &&
          global.location.pathname &&
          global.location.pathname.split("/").pop()) ||
        "";
      var href = path ? "learn/" + path : null;
      setContinue(moduleId, href);
    }

    if (prose && toc) buildToc(prose, toc);
    if (prose && bar) bindProgress(prose, bar);
  }

  /** Call after async markdown render. */
  function afterRender() {
    enhanceModulePage();
  }

  function paintContinueOnHub() {
    var el = global.document.getElementById("learn-continue");
    if (!el) return;
    var c = getContinue();
    if (!c || !c.moduleId) {
      el.hidden = true;
      return;
    }
    var href = c.href || "learn/" + c.moduleId + ".html";
    // titles from list
    var link = el.querySelector("a");
    var titleEl = global.document.querySelector(
      '.module-list__item[data-module-id="' +
        c.moduleId +
        '"] .module-list__title'
    );
    var title = titleEl
      ? titleEl.textContent
      : c.moduleId.replace(/-/g, " ");
    if (link) {
      link.href = href;
      link.textContent = "Continue · " + title;
    }
    el.hidden = false;
  }

  /**
   * Rough study minutes from word count (200 wpm reading + 30% drill buffer).
   * @param {number} words
   */
  function estimateMinutes(words) {
    var w = Math.max(0, Number(words) || 0);
    var min = Math.round((w / 200) * 1.35);
    if (min < 15) min = 15;
    if (min > 55) min = 55;
    return min;
  }

  function paintTimeEstimates(index) {
    if (!index || !index.modules) return;
    for (var i = 0; i < index.modules.length; i++) {
      var m = index.modules[i];
      if (m.empty) continue;
      var li = global.document.querySelector(
        '.module-list__item[data-module-id="' + m.id + '"]'
      );
      if (!li) continue;
      var body = li.querySelector(".module-list__body");
      if (!body) continue;
      if (body.querySelector(".module-list__eta")) continue;
      var words = m.word_count || 0;
      var mins = m.estimate_minutes || estimateMinutes(words);
      var span = global.document.createElement("span");
      span.className = "module-list__eta mono";
      span.textContent = "~" + mins + " min";
      body.appendChild(span);
    }
  }

  function loadHubExtras() {
    paintContinueOnHub();
    if (typeof fetch !== "function") return;
    fetch("data/modules_index.json")
      .then(function (r) {
        if (!r.ok) throw new Error("no index");
        return r.json();
      })
      .then(paintTimeEstimates)
      .catch(function () {
        /* estimates optional if index old */
      });
  }

  global.CdcpLearnChrome = {
    afterRender: afterRender,
    setContinue: setContinue,
    getContinue: getContinue,
    paintContinueOnHub: paintContinueOnHub,
    loadHubExtras: loadHubExtras,
    estimateMinutes: estimateMinutes,
    buildToc: buildToc,
  };

  if (global.document) {
    if (global.document.readyState === "loading") {
      global.document.addEventListener("DOMContentLoaded", function () {
        if (global.document.getElementById("module-list")) loadHubExtras();
      });
    } else if (global.document.getElementById("module-list")) {
      loadHubExtras();
    }
  }
})(typeof window !== "undefined" ? window : globalThis);
