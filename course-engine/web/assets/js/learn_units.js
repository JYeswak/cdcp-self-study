/**
 * learn_units.js — M8-B unit shell + mid-unit micro-checks (offline).
 * Study signal only — not a credential.
 */
(function (global) {
  "use strict";

  var UNIT_DONE_KEY = "cdcp.learn.units_done.v1";
  var unitsCache = null;
  var bankCache = null;

  function readDone() {
    try {
      var o = JSON.parse(global.localStorage.getItem(UNIT_DONE_KEY) || "{}");
      return o && typeof o === "object" ? o : {};
    } catch (e) {
      return {};
    }
  }

  function markDone(unitId) {
    var o = readDone();
    o[unitId] = Date.now();
    try {
      global.localStorage.setItem(UNIT_DONE_KEY, JSON.stringify(o));
    } catch (e) {
      /* ignore */
    }
  }

  function isDone(unitId) {
    return !!readDone()[unitId];
  }

  function fetchJson(url) {
    return fetch(url).then(function (r) {
      if (!r.ok) throw new Error(url);
      return r.json();
    });
  }

  function moduleUnits(index, moduleId) {
    if (!index) return [];
    if (index.by_module && index.by_module[moduleId]) {
      return index.by_module[moduleId];
    }
    return (index.units || []).filter(function (u) {
      return u.module_id === moduleId;
    });
  }

  function bankList(bank) {
    if (!bank) return [];
    if (Array.isArray(bank)) return bank;
    if (bank.items && Array.isArray(bank.items)) return bank.items;
    return [];
  }

  function byId(bank) {
    var map = {};
    var list = bankList(bank);
    for (var i = 0; i < list.length; i++) {
      if (list[i] && list[i].id) map[list[i].id] = list[i];
    }
    return map;
  }

  function qualityScore(it) {
    if (!it) return 0;
    var s = 0;
    if (it.explanation && String(it.explanation).length >= 20) s += 50;
    if ((it.choices || []).length >= 4) s += 20;
    var stem = String(it.stem || "");
    if (stem.length >= 40 && stem.length <= 280) s += 15;
    if ((it.topic_ids || []).length) s += 10;
    return s;
  }

  /**
   * Resolve 2–3 study Quick-check items for a unit.
   * Prefers precomputed unit.check_item_ids (build_units), then topic match,
   * then whole-module bank fill — never claims covered when empty.
   */
  function pickItems(bank, unit, n) {
    n = n || 3;
    var list = bankList(bank);
    if (!list.length) return [];

    var tids = (unit && unit.topic_ids) || [];
    var mnum =
      unit && (unit.module_num != null
        ? unit.module_num
        : moduleNumFromId(unit && unit.module_id));

    var pool = [];
    var seen = {};
    function add(it) {
      if (!it || !it.id || seen[it.id]) return;
      if (!it.stem || !(it.choices || []).length) return;
      seen[it.id] = true;
      pool.push(it);
    }

    // 1) Precomputed ids from units_index (best — curated + diversified)
    if (unit && unit.check_item_ids && unit.check_item_ids.length) {
      var map = byId(list);
      for (var i = 0; i < unit.check_item_ids.length && pool.length < n; i++) {
        add(map[unit.check_item_ids[i]]);
      }
      if (pool.length >= 2) return pool.slice(0, n);
    }

    // 2) Topic match within module
    for (var a = 0; a < list.length; a++) {
      var item = list[a];
      if (mnum != null && Number(item.module) !== Number(mnum)) continue;
      var tops = item.topic_ids || [];
      for (var j = 0; j < tops.length; j++) {
        if (tids.indexOf(tops[j]) !== -1) {
          add(item);
          break;
        }
      }
    }

    // 3) Module fill
    if (pool.length < n && mnum != null) {
      for (var b = 0; b < list.length; b++) {
        if (Number(list[b].module) === Number(mnum)) add(list[b]);
      }
    }

    pool.sort(function (x, y) {
      var dq = qualityScore(y) - qualityScore(x);
      if (dq) return dq;
      return String(x.id).localeCompare(String(y.id));
    });

    // Offset by unit order so units don't all show the same top-3
    var order = (unit && unit.order) || 1;
    if (pool.length > n) {
      var start = ((order - 1) * n) % pool.length;
      pool = pool.slice(start).concat(pool.slice(0, start));
    }
    return pool.slice(0, n);
  }

  function moduleNumFromId(moduleId) {
    if (!moduleId) return null;
    var m = String(moduleId).match(/^(\d{2})-/);
    return m ? parseInt(m[1], 10) : null;
  }

  function showUnit(prose, units, idx) {
    if (!prose || !units.length) return;
    var unit = units[idx];
    if (!unit) return;
    var heads = prose.querySelectorAll("h2");
    var start = null;
    var end = null;
    for (var i = 0; i < heads.length; i++) {
      if (heads[i].id === unit.heading_id) {
        start = heads[i];
        end = heads[i + 1] || null;
        break;
      }
    }
    // hide all children of prose, then show range
    var kids = Array.prototype.slice.call(prose.children);
    var showing = !start;
    for (var k = 0; k < kids.length; k++) {
      var el = kids[k];
      if (start && el === start) showing = true;
      if (end && el === end) showing = false;
      el.hidden = !showing;
    }
    // always show if no match
    if (!start) {
      for (k = 0; k < kids.length; k++) kids[k].hidden = false;
    }
  }

  function showFull(prose) {
    if (!prose) return;
    var kids = prose.children;
    for (var i = 0; i < kids.length; i++) kids[i].hidden = false;
  }

  function letterOf(item, choiceText) {
    var choices = item.choices || [];
    var letters = "ABCD";
    for (var i = 0; i < choices.length; i++) {
      if (choices[i] === choiceText) return letters.charAt(i);
    }
    // choice may already be "A. foo"
    return null;
  }

  function correctLetter(item) {
    var c = String(item.correct || "").trim().toUpperCase();
    if (/^[ABCD]$/.test(c)) return c;
    // correct is full choice text
    var choices = item.choices || [];
    var letters = "ABCD";
    for (var i = 0; i < choices.length; i++) {
      if (choices[i] === item.correct) return letters.charAt(i);
    }
    return c.charAt(0);
  }

  function renderMicro(host, items, unitId) {
    host.innerHTML = "";
    host.hidden = false;
    var h = document.createElement("h3");
    h.className = "unit-check__title";
    h.textContent = "Quick check (study only)";
    host.appendChild(h);
    if (!items.length) {
      var empty = document.createElement("p");
      empty.className = "meta";
      empty.textContent =
        "No study questions available for this unit yet — open Module quiz for practice.";
      host.appendChild(empty);
      return;
    }
    var state = { answered: 0, correct: 0, total: items.length };
    items.forEach(function (item, qi) {
      var card = document.createElement("div");
      card.className = "unit-check__q";
      var stem = document.createElement("p");
      stem.className = "unit-check__stem";
      stem.textContent = item.stem || item.id;
      card.appendChild(stem);
      var choices = item.choices || [];
      var letters = "ABCD";
      var right = correctLetter(item);
      for (var i = 0; i < choices.length; i++) {
        var btn = document.createElement("button");
        btn.type = "button";
        btn.className = "unit-check__choice";
        btn.textContent = letters.charAt(i) + ". " + choices[i];
        btn.setAttribute("data-letter", letters.charAt(i));
        (function (button, letter) {
          button.addEventListener("click", function () {
            if (card.getAttribute("data-done")) return;
            card.setAttribute("data-done", "1");
            state.answered++;
            var ok = letter === right;
            if (ok) state.correct++;
            button.classList.add(ok ? "is-correct" : "is-wrong");
            // mark right answer
            var all = card.querySelectorAll(".unit-check__choice");
            for (var j = 0; j < all.length; j++) {
              all[j].disabled = true;
              if (all[j].getAttribute("data-letter") === right) {
                all[j].classList.add("is-correct");
              }
            }
            var exp = document.createElement("p");
            exp.className = "unit-check__exp meta";
            exp.textContent = item.explanation || (ok ? "Correct." : "See explanation in bank.");
            card.appendChild(exp);
            if (state.answered >= state.total) {
              markDone(unitId);
              var done = document.createElement("p");
              done.className = "unit-check__done";
              done.textContent =
                "Check complete · " +
                state.correct +
                "/" +
                state.total +
                " (study signal only — not a credential)";
              host.appendChild(done);
            }
          });
        })(btn, letters.charAt(i));
        card.appendChild(btn);
      }
      host.appendChild(card);
    });
  }

  function mount(moduleId) {
    var shell = global.document.getElementById("learn-unit-shell");
    var prose = global.document.getElementById("module-prose");
    var checkHost = global.document.getElementById("learn-unit-check");
    if (!shell || !prose || !moduleId) return;

    var base = "../data/";
    // when on learn.html root vs learn/
    if (global.location.pathname.indexOf("/learn/") === -1) {
      base = "data/";
    }

    Promise.all([
      fetchJson(base + "units_index.json"),
      fetchJson(base + "bank_items_seed42.json").catch(function () {
        return [];
      }),
    ])
      .then(function (pair) {
        unitsCache = pair[0];
        bankCache = pair[1];
        var units = moduleUnits(unitsCache, moduleId);
        if (units.length < 1) {
          shell.hidden = true;
          return;
        }
        shell.hidden = false;
        var idx = 0;
        var params = new URLSearchParams(global.location.search || "");
        var q = parseInt(params.get("unit") || "1", 10);
        if (q >= 1 && q <= units.length) idx = q - 1;

        var modeFull = false;
        var status = shell.querySelector(".learn-unit-shell__status");
        var title = shell.querySelector(".learn-unit-shell__title");
        var prev = shell.querySelector("[data-unit-prev]");
        var next = shell.querySelector("[data-unit-next]");
        var fullBtn = shell.querySelector("[data-unit-full]");
        var unitBtn = shell.querySelector("[data-unit-mode]");

        function paint() {
          var u = units[idx];
          if (status) {
            status.textContent =
              "Unit " + (idx + 1) + " / " + units.length + (isDone(u.id) ? " · done" : "");
          }
          if (title) title.textContent = u.title;
          if (modeFull) {
            showFull(prose);
            if (checkHost) checkHost.hidden = true;
          } else {
            showUnit(prose, units, idx);
            if (checkHost) {
              var items = pickItems(bankCache, u, 3);
              renderMicro(checkHost, items, u.id);
            }
          }
          if (prev) prev.disabled = idx <= 0 || modeFull;
          if (next) next.disabled = idx >= units.length - 1 || modeFull;
        }

        if (prev) {
          prev.onclick = function () {
            if (idx > 0) {
              idx--;
              paint();
            }
          };
        }
        if (next) {
          next.onclick = function () {
            if (idx < units.length - 1) {
              idx++;
              paint();
            }
          };
        }
        if (fullBtn) {
          fullBtn.onclick = function () {
            modeFull = true;
            paint();
          };
        }
        if (unitBtn) {
          unitBtn.onclick = function () {
            modeFull = false;
            paint();
          };
        }
        paint();
      })
      .catch(function () {
        shell.hidden = true;
      });
  }

  global.CdcpLearnUnits = {
    mount: mount,
    pickItems: pickItems,
    markDone: markDone,
  };
})(typeof window !== "undefined" ? window : globalThis);
