/**
 * reference.js — offline Glossary + Power cheatsheet panel (L7-S4).
 * Loads web/content/reference/*.md and renders via CdcpLearnMd.
 * Hash routing: #glossary | #power (default glossary).
 */
(function (global) {
  "use strict";

  var VALID = { glossary: true, power: true };
  var loaded = Object.create(null);

  function activeId() {
    var h = (global.location && global.location.hash
      ? String(global.location.hash).replace(/^#/, "")
      : ""
    ).toLowerCase();
    if (h === "cheatsheet" || h === "power-and-redundancy") h = "power";
    if (VALID[h]) return h;
    return "glossary";
  }

  function setHash(id) {
    if (!global.history || !global.location) return;
    var next = "#" + id;
    if (global.location.hash === next) return;
    try {
      global.history.replaceState(null, "", next);
    } catch (_) {
      global.location.hash = next;
    }
  }

  function selectTab(id) {
    if (!VALID[id]) id = "glossary";
    var tabs = document.querySelectorAll('[role="tab"][data-ref-id]');
    var panels = document.querySelectorAll('[role="tabpanel"][data-ref-id]');
    var i, el, match;
    for (i = 0; i < tabs.length; i++) {
      el = tabs[i];
      match = el.getAttribute("data-ref-id") === id;
      el.setAttribute("aria-selected", match ? "true" : "false");
      el.tabIndex = match ? 0 : -1;
    }
    for (i = 0; i < panels.length; i++) {
      el = panels[i];
      match = el.getAttribute("data-ref-id") === id;
      if (match) {
        el.removeAttribute("hidden");
      } else {
        el.setAttribute("hidden", "");
      }
    }
    setHash(id);
    loadPanel(id);
  }

  function failProse(prose, msg) {
    if (!prose) return;
    prose.setAttribute("aria-busy", "false");
    prose.innerHTML =
      '<p class="lede">Could not load reference markdown. ' +
      String(msg || "") +
      " Serve <span class=\"mono\">web/</span> over HTTP and re-run " +
      "<span class=\"mono\">python3 scripts/build_reference.py</span>.</p>";
  }

  function loadPanel(id) {
    var panel = document.getElementById("panel-" + id);
    if (!panel) return;
    var prose = document.getElementById("prose-" + id);
    if (!prose) return;

    if (loaded[id]) {
      prose.setAttribute("aria-busy", "false");
      return;
    }

    var href = panel.getAttribute("data-content-href");
    if (!href) {
      failProse(prose, "Missing data-content-href.");
      return;
    }
    if (typeof fetch !== "function") {
      failProse(prose, "fetch unavailable.");
      return;
    }

    prose.setAttribute("aria-busy", "true");
    fetch(href)
      .then(function (r) {
        if (!r.ok) throw new Error("HTTP " + r.status);
        return r.text();
      })
      .then(function (text) {
        if (!text || !String(text).trim()) throw new Error("empty document");
        if (global.CdcpLearnMd && global.CdcpLearnMd.renderInto) {
          global.CdcpLearnMd.renderInto(prose, text);
        } else {
          prose.textContent = text;
        }
        prose.setAttribute("aria-busy", "false");
        loaded[id] = true;
      })
      .catch(function (err) {
        failProse(prose, err && err.message ? err.message : "fetch failed");
      });
  }

  function onTabClick(ev) {
    var btn = ev.currentTarget;
    var id = btn && btn.getAttribute("data-ref-id");
    if (id) selectTab(id);
  }

  function onTabKey(ev) {
    var key = ev.key;
    if (key !== "ArrowLeft" && key !== "ArrowRight" && key !== "Home" && key !== "End") {
      return;
    }
    var tabs = Array.prototype.slice.call(
      document.querySelectorAll('[role="tab"][data-ref-id]')
    );
    if (!tabs.length) return;
    var idx = tabs.indexOf(ev.currentTarget);
    if (idx < 0) idx = 0;
    var next = idx;
    if (key === "ArrowLeft") next = (idx - 1 + tabs.length) % tabs.length;
    if (key === "ArrowRight") next = (idx + 1) % tabs.length;
    if (key === "Home") next = 0;
    if (key === "End") next = tabs.length - 1;
    ev.preventDefault();
    tabs[next].focus();
    selectTab(tabs[next].getAttribute("data-ref-id"));
  }

  function init() {
    var tabs = document.querySelectorAll('[role="tab"][data-ref-id]');
    var i;
    for (i = 0; i < tabs.length; i++) {
      tabs[i].addEventListener("click", onTabClick);
      tabs[i].addEventListener("keydown", onTabKey);
    }
    global.addEventListener("hashchange", function () {
      selectTab(activeId());
    });
    selectTab(activeId());
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }

  global.CdcpReference = {
    selectTab: selectTab,
    activeId: activeId,
  };
})(typeof window !== "undefined" ? window : globalThis);
