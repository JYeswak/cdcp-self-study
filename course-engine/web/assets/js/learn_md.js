/**
 * learn_md.js — minimal offline Markdown → HTML for Learn modules.
 * No CDN. Covers ATX headings, lists, tables, fences, hr, blockquote, inline.
 *
 * h2/h3 (and other ATX headings) get stable `id` attributes so results feedback
 * can deep-link to learn/{slug}.html#section (L7-S2 / bd-cgx).
 * Slug algorithm is shared with scripts/build_learn.py and smoke_feedback_links.py.
 */
(function (global) {
  "use strict";

  function esc(s) {
    return String(s)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }

  /**
   * Stable heading slug (ASCII-oriented, curriculum English).
   * Matches Python `slugify_heading` in build_learn / smoke_feedback_links.
   * @param {string} text heading text (markdown stripped of trailing #)
   * @returns {string}
   */
  function slugify(text) {
    var s = String(text || "")
      .toLowerCase()
      // strip common markdown emphasis leftovers
      .replace(/[*_`]/g, "")
      // drop punctuation except spaces/hyphens
      .replace(/[^\w\s-]/g, "")
      .trim()
      .replace(/[\s_]+/g, "-")
      .replace(/-+/g, "-")
      .replace(/^-+|-+$/g, "");
    return s || "section";
  }

  function uniqueSlug(base, used) {
    var slug = base || "section";
    if (!used[slug]) {
      used[slug] = 1;
      return slug;
    }
    var n = used[slug] + 1;
    while (used[slug + "-" + n]) n++;
    used[slug] = n;
    used[slug + "-" + n] = 1;
    return slug + "-" + n;
  }

  /** Offline LaTeX → HTML (syllabus subset: frac, text). No CDN. */
  function latexToHtml(src) {
    var s = String(src || "").trim();
    var m = /^\\\[([\s\S]*?)\\\]$/.exec(s) || /^\$\$([\s\S]*?)\$\$$/.exec(s);
    if (m) s = m[1].trim();
    var texts = [];
    var prev;
    s = s.replace(/\\text\{([^}]*)\}/g, function (_, t) {
      texts.push(esc(t));
      return "\0T" + (texts.length - 1) + "\0";
    });
    do {
      prev = s;
      s = s.replace(/\\frac\{([^{}]*)\}\{([^{}]*)\}/g, function (_, a, b) {
        return (
          '<span class="math-frac" role="math">' +
          '<span class="math-frac__num">' +
          a +
          '</span><span class="math-frac__bar" aria-hidden="true"></span>' +
          '<span class="math-frac__den">' +
          b +
          "</span></span>"
        );
      });
    } while (s !== prev);
    s = s.replace(/\\\\/g, "<br>");
    s = s.replace(/\\,/g, " ");
    s = s.replace(/\\ /g, " ");
    s = s.replace(/\\left|\\right/g, "");
    s = s.replace(/\\approx/g, "≈");
    s = s.replace(/\\times/g, "×");
    s = s.replace(/\\cdot/g, "·");
    s = s.replace(/\\ge|\\geq/g, "≥");
    s = s.replace(/\\le|\\leq/g, "≤");
    s = s.replace(/\\pm/g, "±");
    s = s.replace(/=/g, " = ");
    s = s.replace(/\+/g, " + ");
    s = s
      .split(/(<[^>]+>)/g)
      .map(function (part) {
        if (part.charAt(0) === "<") return part;
        return esc(part);
      })
      .join("");
    var ti;
    for (ti = 0; ti < texts.length; ti++) {
      s = s.split("\0T" + ti + "\0").join(texts[ti]);
    }
    return s;
  }

  function inline(text) {
    var codes = [];
    var links = [];
    var bolds = [];
    var italics = [];
    var s = String(text);

    // inline $...$ math (not $$)
    s = s.replace(/\$([^$\n]+)\$/g, function (_, tex) {
      return (
        '<span class="math-inline" role="math">' +
        latexToHtml("\\[" + tex + "\\]") +
        "</span>"
      );
    });

    s = s.replace(/`([^`]+)`/g, function (_, c) {
      codes.push(c);
      return "\0C" + (codes.length - 1) + "\0";
    });
    s = s.replace(/\[([^\]]+)\]\(([^)]+)\)/g, function (_, label, href) {
      links.push([label, href.trim()]);
      return "\0L" + (links.length - 1) + "\0";
    });
    s = s.replace(/\*\*(.+?)\*\*/g, function (_, b) {
      bolds.push(b);
      return "\0B" + (bolds.length - 1) + "\0";
    });
    // Italic *text* without lookbehind (broader Safari support)
    s = s.replace(/(^|[^*])\*([^*]+)\*(?!\*)/g, function (_, pre, i) {
      italics.push(i);
      return pre + "\0I" + (italics.length - 1) + "\0";
    });

    s = esc(s);

    var i, body, href, label, safeHref;
    for (i = 0; i < italics.length; i++) {
      s = s.split("\0I" + i + "\0").join("<em>" + inline(italics[i]) + "</em>");
    }
    for (i = 0; i < bolds.length; i++) {
      s = s.split("\0B" + i + "\0").join("<strong>" + inline(bolds[i]) + "</strong>");
    }
    for (i = 0; i < links.length; i++) {
      label = inline(links[i][0]);
      href = links[i][1];
      safeHref = esc(href);
      if (/^https?:\/\//i.test(href) || href.indexOf("//") === 0) {
        s = s
          .split("\0L" + i + "\0")
          .join(
            '<a href="' +
              safeHref +
              '" rel="noopener noreferrer">' +
              label +
              "</a>"
          );
      } else {
        s = s
          .split("\0L" + i + "\0")
          .join('<a href="' + safeHref + '">' + label + "</a>");
      }
    }
    for (i = 0; i < codes.length; i++) {
      s = s
        .split("\0C" + i + "\0")
        .join("<code>" + esc(codes[i]) + "</code>");
    }
    return s;
  }

  function splitRow(row) {
    var r = row.trim();
    if (r.charAt(0) === "|") r = r.slice(1);
    if (r.charAt(r.length - 1) === "|") r = r.slice(0, -1);
    return r.split("|").map(function (c) {
      return c.trim();
    });
  }

  function mdToHtml(src) {
    var lines = String(src).replace(/\r\n/g, "\n").replace(/\r/g, "\n").split("\n");
    var out = [];
    var i = 0;
    var n = lines.length;
    var para = [];
    /** @type {Record<string, number>} */
    var usedIds = Object.create(null);

    function flushPara() {
      if (!para.length) return;
      var text = para
        .map(function (x) {
          return x.trim();
        })
        .filter(Boolean)
        .join(" ");
      if (text) out.push("<p>" + inline(text) + "</p>");
      para = [];
    }

    while (i < n) {
      var line = lines[i];
      var stripped = line.trim();

      if (!stripped) {
        flushPara();
        i++;
        continue;
      }

      if (/^```/.test(stripped)) {
        flushPara();
        var lang = stripped.slice(3).trim();
        i++;
        var codeLines = [];
        while (i < n && !/^```/.test(lines[i].trim())) {
          codeLines.push(lines[i]);
          i++;
        }
        if (i < n) i++;
        var cls = lang ? ' class="language-' + esc(lang) + '"' : "";
        out.push(
          "<pre><code" + cls + ">" + esc(codeLines.join("\n")) + "</code></pre>"
        );
        continue;
      }

      // Display math \[ ... \] (possibly multi-line)
      if (/^\\\[/.test(stripped) || /^\$\$/.test(stripped)) {
        flushPara();
        var mathBuf = [stripped];
        var mathEnd = /\\\]\s*$/.test(stripped) || (/^\$\$/.test(stripped) && /\$\$\s*$/.test(stripped) && stripped.length > 3);
        i++;
        while (!mathEnd && i < n) {
          mathBuf.push(lines[i]);
          if (/\\\]\s*$/.test(lines[i].trim()) || /\$\$\s*$/.test(lines[i].trim())) {
            mathEnd = true;
          }
          i++;
        }
        var mathSrc = mathBuf.join("\n");
        out.push(
          '<div class="math-block" role="math" aria-label="formula">' +
            latexToHtml(mathSrc) +
            "</div>"
        );
        continue;
      }

      var hm = /^(#{1,6})\s+(.*)$/.exec(stripped);
      if (hm) {
        flushPara();
        var level = hm[1].length;
        // Plain text for slug (no inline HTML); strip simple emphasis markers.
        var title = hm[2].replace(/\s+#*\s*$/, "").trim();
        var plainTitle = title
          .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1")
          .replace(/[*_`]/g, "");
        var id = uniqueSlug(slugify(plainTitle), usedIds);
        out.push(
          "<h" +
            level +
            ' id="' +
            esc(id) +
            '">' +
            inline(title) +
            "</h" +
            level +
            ">"
        );
        i++;
        continue;
      }

      if (/^(-{3,}|\*{3,}|_{3,})$/.test(stripped)) {
        flushPara();
        out.push("<hr>");
        i++;
        continue;
      }

      if (
        stripped.indexOf("|") !== -1 &&
        i + 1 < n &&
        /^\|?\s*:?-{3,}:?\s*(\|\s*:?-{3,}:?\s*)+\|?\s*$/.test(lines[i + 1].trim())
      ) {
        flushPara();
        var headers = splitRow(stripped);
        i += 2;
        var bodyRows = [];
        while (i < n && lines[i].indexOf("|") !== -1 && lines[i].trim()) {
          bodyRows.push(splitRow(lines[i]));
          i++;
        }
        var parts = ['<div class="table-wrap"><table><thead><tr>'];
        var h;
        for (h = 0; h < headers.length; h++) {
          parts.push("<th>" + inline(headers[h]) + "</th>");
        }
        parts.push("</tr></thead><tbody>");
        var r, c, row;
        for (r = 0; r < bodyRows.length; r++) {
          row = bodyRows[r].slice();
          while (row.length < headers.length) row.push("");
          parts.push("<tr>");
          for (c = 0; c < headers.length; c++) {
            parts.push("<td>" + inline(row[c] || "") + "</td>");
          }
          parts.push("</tr>");
        }
        parts.push("</tbody></table></div>");
        out.push(parts.join(""));
        continue;
      }

      if (stripped.charAt(0) === ">") {
        flushPara();
        var bq = [];
        while (i < n && lines[i].trim().charAt(0) === ">") {
          bq.push(lines[i].trim().replace(/^>\s?/, ""));
          i++;
        }
        out.push("<blockquote>" + inline(bq.join(" ")) + "</blockquote>");
        continue;
      }

      if (/^[-*+]\s+/.test(stripped)) {
        flushPara();
        out.push("<ul>");
        while (i < n && /^[-*+]\s+/.test(lines[i].trim())) {
          var item = lines[i].trim().replace(/^[-*+]\s+/, "");
          i++;
          while (
            i < n &&
            lines[i].trim() &&
            !/^[-*+]\s+/.test(lines[i].trim()) &&
            !/^\d+\.\s+/.test(lines[i].trim()) &&
            lines[i].trim().charAt(0) !== "#" &&
            lines[i].trim().charAt(0) !== "|" &&
            !/^(-{3,}|\*{3,})$/.test(lines[i].trim())
          ) {
            if (/^[ \t]/.test(lines[i])) {
              item += " " + lines[i].trim();
              i++;
            } else break;
          }
          out.push("<li>" + inline(item) + "</li>");
        }
        out.push("</ul>");
        continue;
      }

      if (/^\d+\.\s+/.test(stripped)) {
        flushPara();
        out.push("<ol>");
        while (i < n && /^\d+\.\s+/.test(lines[i].trim())) {
          var oitem = lines[i].trim().replace(/^\d+\.\s+/, "");
          i++;
          while (
            i < n &&
            lines[i].trim() &&
            !/^[-*+]\s+/.test(lines[i].trim()) &&
            !/^\d+\.\s+/.test(lines[i].trim()) &&
            lines[i].trim().charAt(0) !== "#" &&
            lines[i].trim().charAt(0) !== "|" &&
            !/^(-{3,}|\*{3,})$/.test(lines[i].trim())
          ) {
            if (/^[ \t]/.test(lines[i])) {
              oitem += " " + lines[i].trim();
              i++;
            } else break;
          }
          out.push("<li>" + inline(oitem) + "</li>");
        }
        out.push("</ol>");
        continue;
      }

      para.push(stripped);
      i++;
    }
    flushPara();
    return out.join("\n");
  }

  function renderInto(el, src) {
    if (!el) return;
    el.innerHTML = mdToHtml(src);
  }

  global.CdcpLearnMd = {
    mdToHtml: mdToHtml,
    renderInto: renderInto,
    inline: inline,
    slugify: slugify,
    latexToHtml: latexToHtml,
  };
})(typeof window !== "undefined" ? window : globalThis);
