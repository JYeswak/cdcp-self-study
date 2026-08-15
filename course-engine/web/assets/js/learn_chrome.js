/**
 * learn_chrome.js — M8-A1/A4 Learn chrome: sticky TOC, read progress, continue.
 * Offline only. Study signals — not a credential.
 */
(function (global) {
  "use strict";

  var CONTINUE_KEY = "cdcp.learn.continue.v1";

  function setContinue(moduleId, href, extra) {
    if (!moduleId) return;
    extra = extra || {};
    var unit = parseInt(extra.unit, 10);
    if (!(unit >= 1)) unit = null;
    var mode = extra.mode === "full" ? "full" : "unit";
    var unitId =
      typeof extra.unitId === "string" && extra.unitId ? extra.unitId : null;
    var path = href || null;
    if (path) {
      var q = path.indexOf("?");
      if (q !== -1) path = path.slice(0, q);
    }
    try {
      global.localStorage.setItem(
        CONTINUE_KEY,
        JSON.stringify({
          moduleId: moduleId,
          href: path,
          unit: unit,
          unitId: unitId,
          mode: mode,
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

  /**
   * Restore URL: unit offset inside the module page, never the catalog.
   * Default path is unit mode (`?unit=N`). Full article is the appendix.
   */
  function continueHref(c) {
    if (!c || typeof c.moduleId !== "string" || !c.moduleId) return null;
    var base = c.href || "learn/" + c.moduleId + ".html";
    var q = base.indexOf("?");
    if (q !== -1) base = base.slice(0, q);
    if (c.mode === "full") return base + "?full=1";
    var unit = parseInt(c.unit, 10);
    if (!(unit >= 1)) unit = 1;
    return base + "?unit=" + unit;
  }

  function firstUnitHref(index) {
    if (index && Array.isArray(index.modules)) {
      for (var i = 0; i < index.modules.length; i++) {
        var m = index.modules[i];
        if (!m || m.empty) continue;
        if (typeof m.href === "string" && m.href) {
          var h = m.href;
          var q = h.indexOf("?");
          if (q !== -1) h = h.slice(0, q);
          return h + "?unit=1";
        }
        if (typeof m.id === "string" && m.id) {
          return "learn/" + m.id + ".html?unit=1";
        }
      }
    }
    return "learn/01-mission-critical.html?unit=1";
  }

  /**
   * First-run Learn opens unit 1, not the module list / full article.
   * Stay on the catalog when `?catalog=1` (appendix) or when Continue exists
   * so the chip can restore the unit offset.
   * @returns {boolean} true when a redirect was issued
   */
  function maybeOpenUnitPath() {
    if (!global.document || !global.document.getElementById("module-list")) {
      return false;
    }
    var search = (global.location && global.location.search) || "";
    var params = new URLSearchParams(search);
    if (params.get("catalog") === "1" || params.get("full") === "1") {
      return false;
    }
    var c = getContinue();
    if (c && c.moduleId) return false;
    var href = firstUnitHref(loadHubIndex());
    if (global.location && typeof global.location.replace === "function") {
      global.location.replace(href);
    }
    return true;
  }

  function loadHubIndex() {
    var el = global.document && global.document.getElementById("modules-index");
    if (!el) return null;
    try {
      return JSON.parse(el.textContent || "{}");
    } catch (e) {
      return null;
    }
  }

  /** Historical key name; law is the 1d/3d ladder, not SRS. */
  var REVIEW_KEY = "cdcp.srs.v1";
  var DRILL10_LIMIT = 10;

  function countDueCards(nowMs) {
    var now = typeof nowMs === "number" ? nowMs : Date.now();
    try {
      var raw = global.localStorage.getItem(REVIEW_KEY);
      if (!raw) return 0;
      var o = JSON.parse(raw);
      var cards = o && o.cards && typeof o.cards === "object" ? o.cards : null;
      if (!cards) return 0;
      var n = 0;
      var ids = Object.keys(cards);
      for (var i = 0; i < ids.length; i++) {
        var card = cards[ids[i]];
        if (card && typeof card.due_at === "number" && card.due_at <= now) {
          n += 1;
        }
      }
      return n;
    } catch (e) {
      return 0;
    }
  }

  function paintHubDue() {
    var el = global.document && global.document.getElementById("hub-due");
    if (!el) return 0;
    var n = countDueCards();
    var cap = n > DRILL10_LIMIT ? DRILL10_LIMIT : n;
    if (n <= 0) {
      el.textContent =
        "No cards due. Take a mock or quiz, then come back for a 90-second loop.";
    } else if (n === 1) {
      el.textContent = "You're due 1 card — 90-second loop + one diagram.";
    } else if (n <= DRILL10_LIMIT) {
      el.textContent =
        "You're due " + n + " cards — Drill-10 is a 90-second loop.";
    } else {
      el.textContent =
        "You're due " +
        n +
        " cards — Drill-10 takes the first " +
        cap +
        ".";
    }
    el.hidden = false;
    return n;
  }

  function paintHubLearnCta() {
    var card = global.document && global.document.getElementById("hub-learn");
    if (!card) return;
    var c = getContinue();
    if (c && c.moduleId) {
      var href = continueHref(c);
      if (href) card.setAttribute("href", href);
      var title = card.querySelector(".card__title");
      if (title) title.textContent = "Continue unit";
      var desc = card.querySelector(".card__desc");
      if (desc) {
        desc.textContent =
          "Resume the unit you left — not the module list. Study signal only.";
      }
      return;
    }
    if (!card.getAttribute("href") || card.getAttribute("href") === "learn.html") {
      card.setAttribute("href", firstUnitHref(null));
    }
  }

  function rewriteCatalogUnitLinks() {
    var links =
      global.document &&
      global.document.querySelectorAll(".module-list__link[href]");
    if (!links) return;
    for (var i = 0; i < links.length; i++) {
      var a = links[i];
      var href = a.getAttribute("href") || "";
      if (!href || href.indexOf("?") !== -1) continue;
      if (href.indexOf("learn/") === -1) continue;
      a.setAttribute("href", href + "?unit=1");
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
      var params = new URLSearchParams(
        (global.location && global.location.search) || ""
      );
      var unit = parseInt(params.get("unit") || "", 10);
      var prev = getContinue();
      var extra = {
        mode: params.get("full") === "1" ? "full" : "unit",
      };
      if (unit >= 1) {
        extra.unit = unit;
      } else if (
        prev &&
        prev.moduleId === moduleId &&
        parseInt(prev.unit, 10) >= 1
      ) {
        extra.unit = parseInt(prev.unit, 10);
        if (params.get("full") !== "1" && prev.mode === "full") {
          extra.mode = "full";
        }
      }
      setContinue(moduleId, href, extra);
    }
    var catalog = global.document.querySelectorAll(".mod-nav__hub a[href]");
    for (var i = 0; i < catalog.length; i++) {
      var a = catalog[i];
      var dest = a.getAttribute("href") || "";
      if (dest.indexOf("learn.html") !== -1 && dest.indexOf("catalog=") === -1) {
        var sep = dest.indexOf("?") === -1 ? "?" : "&";
        a.setAttribute("href", dest + sep + "catalog=1");
      }
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
    var href = continueHref(c);
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
    var unit = parseInt(c.unit, 10);
    var label = "Continue · " + title;
    if (unit >= 1 && c.mode !== "full") {
      label += " · unit " + unit;
    }
    if (link) {
      link.href = href;
      link.textContent = label;
    }
    var meta = el.querySelector(".meta");
    if (meta) {
      meta.textContent = " · last unit in this browser (study signal only)";
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
    if (maybeOpenUnitPath()) return;
    paintContinueOnHub();
    paintHubDue();
    paintHubLearnCta();
    rewriteCatalogUnitLinks();
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
    continueHref: continueHref,
    firstUnitHref: firstUnitHref,
    maybeOpenUnitPath: maybeOpenUnitPath,
    countDueCards: countDueCards,
    paintHubDue: paintHubDue,
    paintHubLearnCta: paintHubLearnCta,
    paintContinueOnHub: paintContinueOnHub,
    loadHubExtras: loadHubExtras,
    estimateMinutes: estimateMinutes,
    buildToc: buildToc,
    CONTINUE_KEY: CONTINUE_KEY,
  };

  function bootChrome() {
    if (!global.document) return;
    if (global.document.getElementById("module-list")) {
      loadHubExtras();
      return;
    }
    if (
      global.document.getElementById("hub-due") ||
      global.document.getElementById("hub-learn")
    ) {
      paintHubDue();
      paintHubLearnCta();
    }
  }

  if (global.document) {
    if (global.document.readyState === "loading") {
      global.document.addEventListener("DOMContentLoaded", bootChrome);
    } else {
      bootChrome();
    }
  }
})(typeof window !== "undefined" ? window : globalThis);
