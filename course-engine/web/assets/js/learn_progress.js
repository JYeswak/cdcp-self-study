/**
 * learn_progress.js — localStorage module-visited progress (no server).
 *
 * Storage key: cdcp.learn.visited.v1 → JSON string[] of module ids.
 * Offline / file:// safe. Never claims certification.
 */
(function (global) {
  "use strict";

  var STORAGE_KEY = "cdcp.learn.visited.v1";

  function readVisited() {
    try {
      var raw = global.localStorage.getItem(STORAGE_KEY);
      if (!raw) return [];
      var parsed = JSON.parse(raw);
      if (!Array.isArray(parsed)) return [];
      return parsed.filter(function (x) {
        return typeof x === "string" && x.length > 0;
      });
    } catch (e) {
      return [];
    }
  }

  function writeVisited(ids) {
    try {
      var uniq = [];
      var seen = Object.create(null);
      for (var i = 0; i < ids.length; i++) {
        var id = ids[i];
        if (typeof id !== "string" || !id || seen[id]) continue;
        seen[id] = true;
        uniq.push(id);
      }
      uniq.sort();
      global.localStorage.setItem(STORAGE_KEY, JSON.stringify(uniq));
      return uniq;
    } catch (e) {
      return ids;
    }
  }

  function markVisited(moduleId) {
    if (typeof moduleId !== "string" || !moduleId) return readVisited();
    var cur = readVisited();
    if (cur.indexOf(moduleId) === -1) {
      cur.push(moduleId);
      writeVisited(cur);
    }
    paintBadges(cur);
    return cur;
  }

  function isVisited(moduleId) {
    return readVisited().indexOf(moduleId) !== -1;
  }

  function paintBadges(visited) {
    visited = visited || readVisited();
    var set = Object.create(null);
    for (var i = 0; i < visited.length; i++) set[visited[i]] = true;

    var badges = document.querySelectorAll("[data-progress-for]");
    for (var b = 0; b < badges.length; b++) {
      var el = badges[b];
      var mid = el.getAttribute("data-progress-for");
      if (mid && set[mid]) {
        el.hidden = false;
        el.classList.add("module-list__badge--on");
      } else {
        el.hidden = true;
        el.classList.remove("module-list__badge--on");
      }
    }

    var items = document.querySelectorAll(".module-list__item[data-module-id]");
    for (var j = 0; j < items.length; j++) {
      var item = items[j];
      var id = item.getAttribute("data-module-id");
      if (id && set[id] && !item.classList.contains("module-list__item--empty")) {
        item.classList.add("module-list__item--visited");
      } else {
        item.classList.remove("module-list__item--visited");
      }
    }
  }

  function loadIndex() {
    var el = document.getElementById("modules-index");
    if (!el) return null;
    try {
      return JSON.parse(el.textContent || "{}");
    } catch (e) {
      return null;
    }
  }

  function paintHub() {
    var visited = readVisited();
    paintBadges(visited);
    var summary = document.getElementById("learn-progress-summary");
    if (!summary) return;
    var index = loadIndex();
    var navCount = 0;
    if (index && Array.isArray(index.modules)) {
      for (var i = 0; i < index.modules.length; i++) {
        if (!index.modules[i].empty) navCount += 1;
      }
    }
    var done = 0;
    if (index && Array.isArray(index.modules)) {
      for (var j = 0; j < index.modules.length; j++) {
        var m = index.modules[j];
        if (!m.empty && visited.indexOf(m.id) !== -1) done += 1;
      }
    } else {
      done = visited.length;
    }
    if (navCount > 0) {
      summary.textContent =
        "Visited " + done + " of " + navCount + " modules (this browser only).";
    } else {
      summary.textContent = "";
    }
  }

  global.CdcpLearn = {
    STORAGE_KEY: STORAGE_KEY,
    readVisited: readVisited,
    markVisited: markVisited,
    isVisited: isVisited,
    paintHub: paintHub,
    paintBadges: paintBadges,
  };
})(typeof window !== "undefined" ? window : globalThis);
