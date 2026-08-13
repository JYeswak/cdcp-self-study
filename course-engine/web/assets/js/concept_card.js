/**
 * concept_card.js — M8-D1 miss → definition + optional diagram + similar item.
 */
(function (global) {
  "use strict";

  var CONCEPTS = {
    MTBF: {
      title: "MTBF — Mean Time Between Failures",
      body: "Average operating time between failures. Higher MTBF means failures are rarer (reliability), not the same as availability (which also needs fast restore / MTTR).",
      diagram: null,
      topics: ["m15-mtbf-mttr", "m01-unavailability"],
    },
    availability: {
      title: "Availability vs reliability",
      body: "Availability is uptime fraction when needed. You can improve it by rarer failures (MTBF) or faster restore (MTTR) — or both. Redundancy often targets availability under single failure.",
      diagram: null,
      topics: ["m01-unavailability"],
    },
    spof: {
      title: "Path SPOF + change error",
      body: "Systemic unavailability often comes from single points of failure in design combined with human/process error during change — not only component MTBF.",
      diagram: "../diagrams/site-stack.html",
      topics: ["m01-unavailability"],
    },
    "n+1": {
      title: "N+1 redundancy",
      body: "Capacity for the load (N) plus one spare unit. Supports concurrent maintainability when paths are independent.",
      diagram: "../diagrams/power-path.html",
      topics: ["m06-redundancy"],
    },
    "2n": {
      title: "2N dual path",
      body: "Two full independent paths each able to carry the load. Dual cords only help if they land on independent upstream sources.",
      diagram: "../diagrams/power-path.html",
      topics: ["m06-redundancy"],
    },
  };

  function pickConcept(item) {
    if (!item) return CONCEPTS.spof;
    var blob = (
      (item.stem || "") +
      " " +
      (item.explanation || "") +
      " " +
      (item.id || "")
    ).toLowerCase();
    if (blob.indexOf("mtbf") !== -1) return CONCEPTS.MTBF;
    if (blob.indexOf("availability") !== -1 || blob.indexOf("mttr") !== -1)
      return CONCEPTS.availability;
    if (blob.indexOf("2n") !== -1 || blob.indexOf("dual") !== -1)
      return CONCEPTS["2n"];
    if (blob.indexOf("n+1") !== -1 || blob.indexOf("redundan") !== -1)
      return CONCEPTS["n+1"];
    if (
      blob.indexOf("spof") !== -1 ||
      blob.indexOf("systemic") !== -1 ||
      blob.indexOf("unavailability") !== -1
    )
      return CONCEPTS.spof;
    return {
      title: "Concept coach",
      body:
        item.explanation ||
        "Review the explanation and related Learn section for this topic.",
      diagram: null,
      topics: item.topic_ids || [],
    };
  }

  function render(host, item, similar) {
    if (!host) return;
    var c = pickConcept(item);
    var html =
      '<div class="concept-card" role="region" aria-label="Concept coach">' +
      '<p class="concept-card__tag mono">CONCEPT</p>' +
      "<h3 class=\"concept-card__title\">" +
      escapeHtml(c.title) +
      "</h3>" +
      '<p class="concept-card__body">' +
      escapeHtml(c.body) +
      "</p>";
    if (c.diagram) {
      html +=
        '<p><a class="concept-card__link" href="' +
        escapeHtml(c.diagram) +
        '">Open related diagram →</a></p>';
    }
    if (similar && similar.stem) {
      html +=
        '<p class="meta"><strong>Similar item:</strong> ' +
        escapeHtml(similar.stem.slice(0, 160)) +
        (similar.stem.length > 160 ? "…" : "") +
        "</p>";
    }
    html +=
      '<p class="meta">Study signal only — not a CDCP credential.</p></div>';
    host.innerHTML = html;
  }

  function escapeHtml(s) {
    return String(s)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }

  global.CdcpConceptCard = {
    pickConcept: pickConcept,
    render: render,
  };
})(typeof window !== "undefined" ? window : globalThis);
