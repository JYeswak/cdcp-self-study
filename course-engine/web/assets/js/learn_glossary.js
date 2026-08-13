/**
 * learn_glossary.js — M8-D2 term popovers from glossary.json (offline).
 */
(function (global) {
  "use strict";

  var cache = null;
  var PRIORITY = [
    "MTBF",
    "MTTR",
    "Availability",
    "Reliability",
    "N+1",
    "2N",
    "CRAC",
    "CRAH",
    "UPS",
    "ATS",
    "STS",
    "AHJ",
    "White space",
    "Grey space",
    "PDU",
    "Containment",
    "BMS",
    "DCIM",
    "Colocation",
    "Concurrent maintainability",
  ];

  function loadGlossary() {
    if (cache) return Promise.resolve(cache);
    var base =
      global.location.pathname.indexOf("/learn/") !== -1
        ? "../data/glossary.json"
        : "data/glossary.json";
    return fetch(base)
      .then(function (r) {
        if (!r.ok) throw new Error("no glossary");
        return r.json();
      })
      .then(function (j) {
        cache = j.terms || {};
        return cache;
      })
      .catch(function () {
        cache = {};
        return cache;
      });
  }

  function findDef(terms, key) {
    if (terms[key]) return terms[key];
    var ck = Object.keys(terms);
    for (var i = 0; i < ck.length; i++) {
      if (ck[i].casefold ? ck[i].casefold() === key.casefold() : ck[i].toLowerCase() === key.toLowerCase()) {
        return terms[ck[i]];
      }
    }
    return null;
  }

  function enhance(prose) {
    if (!prose) return;
    loadGlossary().then(function (terms) {
      if (!terms || !Object.keys(terms).length) return;
      var keys = PRIORITY.filter(function (k) {
        return findDef(terms, k);
      });
      // also include long glossary keys that appear
      Object.keys(terms).forEach(function (k) {
        if (k.length >= 4 && PRIORITY.indexOf(k) === -1) keys.push(k);
      });
      keys.sort(function (a, b) {
        return b.length - a.length;
      });
      if (!keys.length) return;

      var walker = global.document.createTreeWalker(
        prose,
        NodeFilter.SHOW_TEXT,
        null
      );
      var nodes = [];
      while (walker.nextNode()) nodes.push(walker.currentNode);

      nodes.forEach(function (textNode) {
        var parent = textNode.parentNode;
        if (!parent || parent.closest("a, button, .math-block, .math-inline, code, pre")) {
          return;
        }
        var text = textNode.nodeValue;
        if (!text || !text.trim()) return;
        var frag = global.document.createDocumentFragment();
        var remaining = text;
        var safety = 0;
        while (remaining && safety++ < 50) {
          var best = null;
          var bestAt = -1;
          for (var i = 0; i < keys.length; i++) {
            var k = keys[i];
            var re = new RegExp(
              "\\b" + k.replace(/[.*+?^${}()|[\]\\]/g, "\\$&") + "\\b",
              "i"
            );
            var m = re.exec(remaining);
            if (m && (bestAt < 0 || m.index < bestAt)) {
              bestAt = m.index;
              best = { key: k, match: m[0], index: m.index };
            }
          }
          if (!best) {
            frag.appendChild(global.document.createTextNode(remaining));
            break;
          }
          if (best.index > 0) {
            frag.appendChild(
              global.document.createTextNode(remaining.slice(0, best.index))
            );
          }
          var btn = global.document.createElement("button");
          btn.type = "button";
          btn.className = "term-pop";
          btn.textContent = best.match;
          btn.setAttribute("aria-expanded", "false");
          var def = findDef(terms, best.key) || findDef(terms, best.match);
          btn.setAttribute("data-def", def || "");
          btn.title = "Glossary: " + best.key;
          btn.addEventListener("click", function (ev) {
            ev.preventDefault();
            ev.stopPropagation();
            var wasOpen = btn.getAttribute("aria-expanded") === "true";

            function tipFor(el) {
              var tip = el.nextElementSibling;
              if (tip && tip.classList && tip.classList.contains("term-pop__tip")) {
                return tip;
              }
              return null;
            }

            function closeTip(el) {
              el.setAttribute("aria-expanded", "false");
              var tip = tipFor(el);
              if (tip) tip.hidden = true;
            }

            // Close every open term (including this one) first.
            prose.querySelectorAll(".term-pop[aria-expanded='true']").forEach(closeTip);

            // Second click on the same term = close only (already done above).
            if (wasOpen) return;

            btn.setAttribute("aria-expanded", "true");
            var tipEl = tipFor(btn);
            if (!tipEl) {
              tipEl = global.document.createElement("span");
              tipEl.className = "term-pop__tip";
              tipEl.setAttribute("role", "note");
              tipEl.textContent = btn.getAttribute("data-def") || "";
              btn.parentNode.insertBefore(tipEl, btn.nextSibling);
            }
            tipEl.hidden = false;
          });
          frag.appendChild(btn);
          remaining = remaining.slice(best.index + best.match.length);
        }
        parent.replaceChild(frag, textNode);
      });
    });
  }

  global.CdcpLearnGlossary = { enhance: enhance };
})(typeof window !== "undefined" ? window : globalThis);
