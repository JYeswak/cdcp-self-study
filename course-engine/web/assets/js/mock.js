/**
 * CDCP mock exam take flow (L5-S3 / bd-38o; multi-seed L6-S5 / bd-wkh;
 * closed-notes L7-S1 / bd-30j).
 *
 * Loads web/data/mock40_seed{N}.json (default seed 42), one question at a time,
 * A–D choices, progress N/40, 60:00 countdown (soft — does not block submit).
 * Answers persist in sessionStorage until submit; submit writes ExamAttempt JSON
 * and navigates to results.html for S4 grading.
 *
 * Closed-notes mode (optional): during an active attempt, hides/disables nav to
 * Learn / Drill / Reference (and soft-warns on other leave paths). Preference in
 * sessionStorage. Submit → results.html still works (leave allowed).
 *
 * Seed selection (priority): URL ?seed=N → #seed-select → default 42.
 * The Seed menu lists only seeds this copy ships (pack + bank + keys).
 * A missing pack is a product gap — tell the learner to pick a listed seed.
 * Never instruct `export-web`; an installed learner has no source checkout.
 *
 * Attempt shape (cdcp_core::ExamAttempt):
 *   { exam_id, seed, bank_hash, answers: [{ item_id, chosen: "A"|"B"|"C"|"D" }] }
 */
(function () {
  "use strict";

  var DEFAULT_SEED = 42;
  var STORAGE_DRAFT = "cdcp_mock_draft_v1";
  var STORAGE_ATTEMPT = "cdcp_mock_attempt_v1";
  /** Preference only — not tied to a single draft attempt. */
  var STORAGE_CLOSED_NOTES = "cdcp_mock_closed_notes_v1";
  var TIMER_SECONDS = 60 * 60; // 60:00
  var LETTERS = ["A", "B", "C", "D"];
  var LEAVE_WARN =
    "Closed-notes mode is on. Leave this mock attempt?\n\n" +
    "Draft answers stay in this browser tab (sessionStorage) until you submit or clear storage.";

  /** @type {{ exam_id: string, seed: number, bank_hash: string, items: Array<{id:string,stem:string,choices:string[]}> } | null} */
  var pack = null;
  /** @type {Record<string, "A"|"B"|"C"|"D">} */
  var answers = Object.create(null);
  var index = 0;
  var timerStartedAt = null; // epoch ms when countdown started
  var timerInterval = null;
  var expiredAnnounced = false;
  /** Active pack seed (resolved at init). */
  var activeSeed = DEFAULT_SEED;
  /** Items marked for review in the current draft. */
  var flags = Object.create(null);
  /** True when this page starts a fresh attempt after a prior submission. */
  var priorAttemptFound = false;
  /** Closed-notes preference (sessionStorage). */
  var closedNotes = false;
  /**
   * When true, navigation away is intentional (submit → results, confirmed leave).
   * Suppresses beforeunload soft-warn for that navigation.
   */
  var allowLeave = false;

  var el = {
    status: null,
    progress: null,
    timer: null,
    card: null,
    stem: null,
    choices: null,
    prev: null,
    next: null,
    jump: null,
    submit: null,
    unanswered: null,
    reviewUnanswered: null,
    flag: null,
    flagStatus: null,
    seedSelect: null,
    packMeta: null,
    closedNotesToggle: null,
    closedNotesHint: null,
    submitConfirm: null,
    submitConfirmCopy: null,
    submitConfirmGaps: null,
    submitConfirmReview: null,
    submitConfirmCancel: null,
    submitConfirmAccept: null,
  };

  function $(id) {
    return document.getElementById(id);
  }

  function packUrl(seed) {
    return "data/mock40_seed" + seed + ".json";
  }

  /**
   * Parse seed from URL ?seed=N, else select value, else default 42.
   * Non-finite / negative → default.
   */
  function resolveSeed() {
    var fromUrl = null;
    try {
      var params = new URLSearchParams(window.location.search);
      if (params.has("seed")) {
        var n = parseInt(params.get("seed"), 10);
        if (Number.isFinite(n) && n >= 0) fromUrl = n;
      }
    } catch (_) {
      /* ignore */
    }
    if (fromUrl != null) return fromUrl;
    if (el.seedSelect) {
      var sel = parseInt(el.seedSelect.value, 10);
      if (Number.isFinite(sel) && sel >= 0) return sel;
    }
    return DEFAULT_SEED;
  }

  function missingPackHint(seed) {
    return (
      "This copy of CDCP Study does not include a mock exam for seed " +
      seed +
      ". Use a seed listed in the Seed menu on this page."
    );
  }

  function loadClosedNotesPref() {
    try {
      closedNotes = sessionStorage.getItem(STORAGE_CLOSED_NOTES) === "1";
    } catch (_) {
      closedNotes = false;
    }
  }

  function saveClosedNotesPref() {
    try {
      sessionStorage.setItem(STORAGE_CLOSED_NOTES, closedNotes ? "1" : "0");
    } catch (_) {
      /* quota / private mode */
    }
  }

  /** True while a pack is loaded and we have not intentionally left for results. */
  function isAttemptActive() {
    return !!pack && !allowLeave;
  }

  function isClosedNotesActive() {
    return closedNotes && isAttemptActive();
  }

  /**
   * Path-ish match for study surfaces blocked under closed notes.
   * Covers Learn, Drill, Reference (future), module readers, quiz.
   */
  function isStudyNavUrl(href) {
    if (!href || typeof href !== "string") return false;
    var path = href;
    try {
      var u = new URL(href, window.location.href);
      // Same-origin only; external links soft-warn via leave path.
      if (u.origin !== window.location.origin) return false;
      path = u.pathname;
    } catch (_) {
      /* relative string */
    }
    path = path.replace(/\\/g, "/").toLowerCase();
    // Strip trailing slash for matching
    if (path.length > 1 && path.charAt(path.length - 1) === "/") {
      path = path.slice(0, -1);
    }
    var base = path.split("/").pop() || path;
    if (base === "learn.html" || base === "drill.html" || base === "quiz.html") {
      return true;
    }
    if (base === "reference.html" || base.indexOf("reference") === 0) {
      return true;
    }
    // Module readers live under learn/
    if (path.indexOf("/learn/") !== -1 || path.indexOf("learn/") === 0) {
      return true;
    }
    // Cheatsheet / reference panel pages if added later
    if (path.indexOf("/reference") !== -1 || path.indexOf("cheatsheet") !== -1) {
      return true;
    }
    return false;
  }

  function isResultsUrl(href) {
    if (!href || typeof href !== "string") return false;
    try {
      var u = new URL(href, window.location.href);
      var base = (u.pathname.split("/").pop() || "").toLowerCase();
      return base === "results.html";
    } catch (_) {
      return /results\.html/i.test(href);
    }
  }

  function isSamePageNav(href) {
    if (!href || href.charAt(0) === "#") return true;
    try {
      var u = new URL(href, window.location.href);
      return (
        u.origin === window.location.origin &&
        u.pathname === window.location.pathname &&
        u.search === window.location.search
      );
    } catch (_) {
      return false;
    }
  }

  function confirmLeave() {
    // Soft warning — browser confirm; Cancel keeps the learner on the mock.
    return window.confirm(LEAVE_WARN);
  }

  function applyClosedNotesChrome() {
    var on = isClosedNotesActive();
    document.body.classList.toggle("closed-notes-on", on);

    if (el.closedNotesToggle) {
      el.closedNotesToggle.checked = closedNotes;
      el.closedNotesToggle.setAttribute("aria-checked", closedNotes ? "true" : "false");
    }
    if (el.closedNotesHint) {
      el.closedNotesHint.hidden = !on;
    }

    // Hide/disable study nav links (Learn / Drill / Reference + data-marked).
    var nodes = document.querySelectorAll(
      "a[data-closed-notes-block], .hub-nav a[href]"
    );
    for (var i = 0; i < nodes.length; i++) {
      var a = nodes[i];
      var href = a.getAttribute("href") || "";
      var marked = a.hasAttribute("data-closed-notes-block");
      var study = marked || isStudyNavUrl(href);
      // Never lock the current Mock page link
      var isMockSelf =
        a.getAttribute("aria-current") === "page" || /mock\.html/i.test(href);
      if (!study || isMockSelf) {
        // Restore if previously locked
        if (a.getAttribute("data-cn-locked") === "1") {
          a.removeAttribute("data-cn-locked");
          a.removeAttribute("aria-disabled");
          a.removeAttribute("tabindex");
          a.classList.remove("hub-nav__locked");
          var prevHref = a.getAttribute("data-cn-href");
          if (prevHref != null) {
            a.setAttribute("href", prevHref);
            a.removeAttribute("data-cn-href");
          }
        }
        continue;
      }
      if (on) {
        if (a.getAttribute("data-cn-locked") !== "1") {
          a.setAttribute("data-cn-locked", "1");
          if (a.hasAttribute("href")) {
            a.setAttribute("data-cn-href", href);
          }
        }
        a.setAttribute("aria-disabled", "true");
        a.setAttribute("tabindex", "-1");
        a.classList.add("hub-nav__locked");
        // Keep href for screen-reader context but navigation is blocked in click handler.
        // Visually treated as disabled; click prevented when closed-notes active.
      } else if (a.getAttribute("data-cn-locked") === "1") {
        a.removeAttribute("data-cn-locked");
        a.removeAttribute("aria-disabled");
        a.removeAttribute("tabindex");
        a.classList.remove("hub-nav__locked");
        var restore = a.getAttribute("data-cn-href");
        if (restore != null) {
          a.setAttribute("href", restore);
          a.removeAttribute("data-cn-href");
        }
      }
    }
  }

  function onClosedNotesToggle() {
    if (!el.closedNotesToggle) return;
    closedNotes = !!el.closedNotesToggle.checked;
    saveClosedNotesPref();
    applyClosedNotesChrome();
  }

  function onDocumentClick(ev) {
    if (!isClosedNotesActive()) return;
    // Find nearest anchor
    var t = ev.target;
    if (!t || !t.closest) return;
    var a = t.closest("a[href]");
    if (!a) return;
    var href = a.getAttribute("href") || "";
    if (isSamePageNav(href)) return;

    // Study surfaces: hard-disable (no leave even with confirm — use toggle off first).
    // Soft-warn path is for Hub / other pages; study nav is locked outright.
    var marked = a.hasAttribute("data-closed-notes-block");
    if (marked || isStudyNavUrl(href) || a.getAttribute("data-cn-locked") === "1") {
      ev.preventDefault();
      ev.stopPropagation();
      // Brief status cue if available
      if (el.status) {
        var wasHidden = el.status.hidden;
        var prevClass = el.status.className;
        var prevText = el.status.textContent;
        el.status.hidden = false;
        el.status.className = "exam-status exam-status--error";
        el.status.textContent =
          "Closed-notes mode: Learn / Drill / Reference are locked. Turn off Closed notes to open study surfaces.";
        window.setTimeout(function () {
          if (!el.status) return;
          // Only restore if we still own the message
          if (
            el.status.textContent &&
            el.status.textContent.indexOf("Closed-notes mode:") === 0
          ) {
            el.status.hidden = wasHidden;
            el.status.className = prevClass;
            el.status.textContent = prevText;
          }
        }, 4000);
      }
      return;
    }

    // Results is allowed only when submit set allowLeave (handled before navigation).
    if (isResultsUrl(href) && allowLeave) return;

    // Soft warning for Hub / seed reload / other leave
    if (!confirmLeave()) {
      ev.preventDefault();
      ev.stopPropagation();
      return;
    }
    allowLeave = true;
  }

  function onBeforeUnload(ev) {
    if (!isClosedNotesActive()) return;
    if (allowLeave) return;
    // Soft browser dialog (message text ignored by modern browsers).
    ev.preventDefault();
    ev.returnValue = LEAVE_WARN;
    return LEAVE_WARN;
  }

  function loadDraft() {
    try {
      var raw = sessionStorage.getItem(STORAGE_DRAFT);
      if (!raw) return;
      var draft = JSON.parse(raw);
      if (!draft || typeof draft !== "object") return;
      if (draft.answers && typeof draft.answers === "object") {
        answers = draft.answers;
      }
      if (draft.flags && typeof draft.flags === "object") {
        flags = draft.flags;
      }
      if (typeof draft.index === "number" && draft.index >= 0) {
        index = draft.index;
      }
      if (typeof draft.timerStartedAt === "number") {
        timerStartedAt = draft.timerStartedAt;
      }
    } catch (_) {
      /* ignore corrupt draft */
    }
  }

  function saveDraft() {
    try {
      sessionStorage.setItem(
        STORAGE_DRAFT,
        JSON.stringify({
          exam_id: pack ? pack.exam_id : null,
          seed: pack ? pack.seed : null,
          bank_hash: pack ? pack.bank_hash : null,
          answers: answers,
          flags: flags,
          index: index,
          timerStartedAt: timerStartedAt,
          saved_at: Date.now(),
        })
      );
    } catch (_) {
      /* quota / private mode */
    }
  }

  function answeredCount() {
    if (!pack) return 0;
    var n = 0;
    for (var i = 0; i < pack.items.length; i++) {
      if (answers[pack.items[i].id]) n++;
    }
    return n;
  }

  function buildAttempt() {
    var list = [];
    for (var i = 0; i < pack.items.length; i++) {
      var item = pack.items[i];
      var chosen = answers[item.id];
      if (chosen) {
        list.push({ item_id: item.id, chosen: chosen });
      }
    }
    return {
      exam_id: pack.exam_id,
      seed: pack.seed,
      bank_hash: pack.bank_hash,
      item_ids: pack.items.map(function (item) { return item.id; }),
      total_items: pack.items.length,
      answered_count: list.length,
      answers: list,
    };
  }

  function missingIndices() {
    var missing = [];
    if (!pack) return missing;
    for (var i = 0; i < pack.items.length; i++) {
      if (!answers[pack.items[i].id]) missing.push(i);
    }
    return missing;
  }

  function hideSubmitConfirm() {
    if (!el.submitConfirm) return;
    el.submitConfirm.hidden = true;
    el.submitConfirm.style.display = "none";
  }

  function showSubmitConfirm() {
    if (!el.submitConfirm || !pack) return;
    var missing = missingIndices();
    var answered = pack.items.length - missing.length;
    el.submitConfirmCopy.textContent = missing.length
      ? "You answered " + answered + " of " + pack.items.length + ". Review the unanswered items, or submit this partial attempt as-is."
      : "All " + pack.items.length + " items are answered. Submit this attempt for grading?";
    el.submitConfirmGaps.innerHTML = "";
    for (var i = 0; i < missing.length; i++) {
      var link = document.createElement("a");
      link.href = "#question-card";
      link.className = "submit-confirm__gap-link";
      link.setAttribute("data-jump", String(missing[i]));
      link.textContent = "Item " + String(missing[i] + 1);
      link.addEventListener("click", function (ev) {
        ev.preventDefault();
        hideSubmitConfirm();
        goTo(parseInt(ev.currentTarget.getAttribute("data-jump"), 10));
      });
      var li = document.createElement("li");
      li.appendChild(link);
      el.submitConfirmGaps.appendChild(li);
    }
    el.submitConfirmReview.hidden = missing.length === 0;
    el.submitConfirmGaps.hidden = missing.length === 0;
    el.submitConfirm.hidden = false;
    el.submitConfirm.style.display = "block";
    el.submitConfirmCancel.focus();
  }

  function formatTime(totalSec) {
    var s = Math.max(0, Math.floor(totalSec));
    var m = Math.floor(s / 60);
    var r = s % 60;
    return (m < 10 ? "0" : "") + m + ":" + (r < 10 ? "0" : "") + r;
  }

  function elapsedSeconds() {
    if (timerStartedAt == null) return 0;
    return (Date.now() - timerStartedAt) / 1000;
  }

  function remainingSeconds() {
    return TIMER_SECONDS - elapsedSeconds();
  }

  function updateTimer() {
    if (!el.timer) return;
    var rem = remainingSeconds();
    el.timer.textContent = formatTime(rem);
    el.timer.setAttribute("datetime", "PT" + Math.max(0, Math.floor(rem)) + "S");

    if (rem <= 0) {
      el.timer.classList.add("exam-timer--expired");
      el.timer.setAttribute("aria-label", "Timer expired at 00:00 — you may still submit");
      if (!expiredAnnounced) {
        expiredAnnounced = true;
        el.timer.setAttribute("aria-live", "assertive");
      }
    } else if (rem <= 5 * 60) {
      el.timer.classList.add("exam-timer--warn");
      el.timer.classList.remove("exam-timer--expired");
      el.timer.setAttribute(
        "aria-label",
        "Time remaining " + formatTime(rem)
      );
    } else {
      el.timer.classList.remove("exam-timer--warn", "exam-timer--expired");
      el.timer.setAttribute(
        "aria-label",
        "Time remaining " + formatTime(rem)
      );
    }
  }

  function startTimer() {
    if (timerStartedAt == null) {
      timerStartedAt = Date.now();
      saveDraft();
    }
    updateTimer();
    if (timerInterval) clearInterval(timerInterval);
    timerInterval = setInterval(updateTimer, 1000);
  }

  function updateChrome() {
    if (!pack) return;
    var total = pack.items.length;
    var answered = answeredCount();
    var n = index + 1;
    var remaining = total - answered;

    el.progress.textContent =
      n + " / " + total + " · " + answered + " answered · " + remaining + " left";
    el.progress.setAttribute(
      "aria-label",
      "Question " + n + " of " + total + ", " + answered + " answered, " + remaining + " remaining"
    );

    var allDone = answered === total;
    var remaining = total - answered;
    el.submit.disabled = false;
    el.submit.setAttribute("aria-disabled", "false");
    el.submit.textContent = allDone
      ? "Submit · " + total + " of " + total
      : "Submit · " + answered + " of " + total + " — " + remaining + " unanswered";
    el.unanswered.textContent = allDone
      ? "All " + total + " answered — ready to submit."
      : answered +
        " of " +
        total +
        " answered · " +
        remaining +
        " unanswered. Submit remains available.";
    if (el.reviewUnanswered) {
      var missing = [];
      for (var mi = 0; mi < pack.items.length; mi++) {
        if (!answers[pack.items[mi].id]) missing.push(mi + 1);
      }
      el.reviewUnanswered.hidden = missing.length === 0;
      // `.btn` supplies a display rule, so do not rely on the UA [hidden]
      // stylesheet rule to remove this control from the visual and tab trees.
      el.reviewUnanswered.style.display = missing.length === 0 ? "none" : "";
      if (missing.length > 0) {
        el.reviewUnanswered.textContent =
          "Review unanswered (Q" + missing.join(", Q") + ")";
        el.reviewUnanswered.setAttribute(
          "aria-label",
          "Review unanswered questions: " + missing.join(", ")
        );
      } else {
        el.reviewUnanswered.textContent = "Review unanswered";
        el.reviewUnanswered.removeAttribute("aria-label");
      }
    }

    el.prev.disabled = index <= 0;
    el.next.disabled = index >= total - 1;

    // Jump chips
    if (el.jump) {
      var chips = el.jump.querySelectorAll("[data-jump]");
      for (var i = 0; i < chips.length; i++) {
        var chip = chips[i];
        var ji = parseInt(chip.getAttribute("data-jump"), 10);
        var id = pack.items[ji].id;
        var isCurrent = ji === index;
        var isAnswered = !!answers[id];
        var isFlagged = !!flags[id];
        chip.classList.toggle("jump-chip--current", isCurrent);
        chip.classList.toggle("jump-chip--answered", isAnswered && !isCurrent);
        chip.classList.toggle("jump-chip--flagged", isFlagged && !isCurrent);
        chip.setAttribute("aria-current", isCurrent ? "true" : "false");
        chip.setAttribute(
          "aria-label",
          "Question " +
            (ji + 1) +
            (isAnswered ? ", answered" : ", unanswered") +
            (isFlagged ? ", flagged for review" : "") +
            (isCurrent ? ", current" : "")
        );
      }
    }
  }

  function renderQuestion() {
    if (!pack) return;
    var item = pack.items[index];
    var total = pack.items.length;
    var n = index + 1;

    el.stem.id = "q-stem";
    el.stem.textContent = item.stem;

    el.card.setAttribute("aria-labelledby", "q-stem");
    el.card.setAttribute("data-item-id", item.id);

    // Rebuild choices as radio group for keyboard a11y
    el.choices.innerHTML = "";
    el.choices.setAttribute("role", "radiogroup");
    el.choices.setAttribute("aria-labelledby", "q-stem");

    var selected = answers[item.id] || null;

    for (var i = 0; i < LETTERS.length; i++) {
      var letter = LETTERS[i];
      var text = item.choices[i] != null ? item.choices[i] : "";
      var inputId = "choice-" + letter;

      var label = document.createElement("label");
      label.className = "choice";
      if (selected === letter) label.classList.add("choice--selected");
      label.setAttribute("for", inputId);

      var input = document.createElement("input");
      input.type = "radio";
      input.name = "mock-choice";
      input.id = inputId;
      input.value = letter;
      input.checked = selected === letter;
      input.setAttribute("data-letter", letter);

      var badge = document.createElement("span");
      badge.className = "choice__letter";
      badge.setAttribute("aria-hidden", "true");
      badge.textContent = letter;

      var body = document.createElement("span");
      body.className = "choice__text";
      body.textContent = text;

      label.appendChild(input);
      label.appendChild(badge);
      label.appendChild(body);
      el.choices.appendChild(label);

      input.addEventListener("change", onChoiceChange);
    }

    el.card.querySelector(".question-card__meta").textContent =
      "Item " + n + " of " + total + " · " + item.id;

    if (el.flag) {
      var isFlagged = !!flags[item.id];
      el.flag.textContent = isFlagged ? "Remove flag" : "Flag for review";
      el.flag.setAttribute("aria-pressed", isFlagged ? "true" : "false");
      if (el.flagStatus) {
        el.flagStatus.hidden = !isFlagged;
        el.flagStatus.textContent = isFlagged
          ? "Flagged for review — use the question map to return here."
          : "";
      }
    }

    updateChrome();
  }

  function onChoiceChange(ev) {
    var letter = ev.target.value;
    if (LETTERS.indexOf(letter) === -1) return;
    var item = pack.items[index];
    answers[item.id] = letter;
    saveDraft();
    renderQuestion();
  }

  function selectLetter(letter) {
    if (LETTERS.indexOf(letter) === -1 || !pack) return;
    var item = pack.items[index];
    answers[item.id] = letter;
    saveDraft();
    renderQuestion();
  }

  function goTo(i) {
    if (!pack) return;
    if (i < 0 || i >= pack.items.length) return;
    index = i;
    saveDraft();
    renderQuestion();
    // Move focus to stem for screen-reader context
    el.stem.setAttribute("tabindex", "-1");
    el.stem.focus({ preventScroll: false });
  }

  function toggleFlag() {
    if (!pack) return;
    var item = pack.items[index];
    if (flags[item.id]) {
      delete flags[item.id];
    } else {
      flags[item.id] = true;
    }
    saveDraft();
    renderQuestion();
  }

  function commitAttempt() {
    var attempt = buildAttempt();
    try {
      sessionStorage.setItem(STORAGE_ATTEMPT, JSON.stringify(attempt));
      // Keep draft until results grades; clear timer interval
      if (timerInterval) clearInterval(timerInterval);
    } catch (err) {
      el.status.textContent =
        "Could not save attempt to sessionStorage: " + (err && err.message ? err.message : err);
      el.status.hidden = false;
      return;
    }
    hideSubmitConfirm();
    // Closed-notes must not block submit → results (results still work).
    allowLeave = true;
    applyClosedNotesChrome();
    window.location.href = "results.html";
  }

  function onSubmit(ev) {
    ev.preventDefault();
    if (!pack) return;
    showSubmitConfirm();
  }

  function onKeydown(ev) {
    // Ignore when typing in inputs we don't own, or with modifiers (except shift for letters)
    if (ev.altKey || ev.ctrlKey || ev.metaKey) return;
    var t = ev.target;
    if (t && (t.tagName === "INPUT" || t.tagName === "TEXTAREA" || t.tagName === "SELECT")) {
      // Allow radio group arrow navigation default; still handle letter keys if radio focused
      if (t.type === "radio") {
        /* fall through for A–D / 1–4 */
      } else {
        return;
      }
    }

    var key = ev.key;
    if (key === "a" || key === "A" || key === "1") {
      ev.preventDefault();
      selectLetter("A");
      return;
    }
    if (key === "b" || key === "B" || key === "2") {
      ev.preventDefault();
      selectLetter("B");
      return;
    }
    if (key === "c" || key === "C" || key === "3") {
      ev.preventDefault();
      selectLetter("C");
      return;
    }
    if (key === "d" || key === "D" || key === "4") {
      ev.preventDefault();
      selectLetter("D");
      return;
    }
    if (key === "ArrowLeft" || key === "p" || key === "P") {
      ev.preventDefault();
      goTo(index - 1);
      return;
    }
    if (key === "ArrowRight" || key === "n" || key === "N") {
      ev.preventDefault();
      goTo(index + 1);
      return;
    }
  }

  function buildJumpStrip() {
    el.jump.innerHTML = "";
    for (var i = 0; i < pack.items.length; i++) {
      var btn = document.createElement("button");
      btn.type = "button";
      btn.className = "jump-chip";
      btn.setAttribute("data-jump", String(i));
      btn.textContent = String(i + 1);
      btn.addEventListener(
        "click",
        (function (ji) {
          return function () {
            goTo(ji);
          };
        })(i)
      );
      el.jump.appendChild(btn);
    }
  }

  function showError(msg) {
    el.status.hidden = false;
    el.status.className = "exam-status exam-status--error";
    el.status.textContent = msg;
    if (el.card) el.card.hidden = true;
    // Submit is intentionally never disabled by answer completeness (or by
    // an error branch). The button remains the stable, always-available
    // control; with no pack loaded onSubmit simply has no attempt to commit.
    if (el.submit) {
      el.submit.disabled = false;
      el.submit.setAttribute("aria-disabled", "false");
    }
  }

  function updatePackMeta() {
    if (!el.packMeta) return;
    el.packMeta.textContent = "data/mock40_seed" + activeSeed + ".json";
  }

  function onSeedChange() {
    if (!el.seedSelect) return;
    var next = parseInt(el.seedSelect.value, 10);
    if (!Number.isFinite(next) || next < 0) next = DEFAULT_SEED;
    // Soft-warn when closed-notes is locking the attempt.
    if (isClosedNotesActive()) {
      if (!confirmLeave()) {
        // Restore select to active seed
        el.seedSelect.value = String(activeSeed);
        return;
      }
      allowLeave = true;
    }
    // Navigate so URL is shareable and draft/pack re-init cleanly.
    var url = new URL(window.location.href);
    url.searchParams.set("seed", String(next));
    window.location.href = url.toString();
  }

  function bind() {
    el.status = $("exam-status");
    el.progress = $("exam-progress");
    el.timer = $("exam-timer");
    el.card = $("question-card");
    el.stem = $("q-stem");
    el.choices = $("q-choices");
    el.prev = $("btn-prev");
    el.next = $("btn-next");
    el.jump = $("jump-strip");
    el.submit = $("btn-submit");
    el.unanswered = $("unanswered-hint");
    el.reviewUnanswered = $("btn-review-unanswered");
    el.flag = $("btn-flag");
    el.flagStatus = $("flag-status");
    el.seedSelect = $("seed-select");
    el.packMeta = $("pack-meta");
    el.closedNotesToggle = $("closed-notes-toggle");
    el.closedNotesHint = $("closed-notes-hint");
    el.submitConfirm = $("submit-confirm");
    el.submitConfirmCopy = $("submit-confirm-copy");
    el.submitConfirmGaps = $("submit-confirm-gaps");
    el.submitConfirmReview = $("submit-confirm-review");
    el.submitConfirmCancel = $("submit-confirm-cancel");
    el.submitConfirmAccept = $("submit-confirm-accept");

    el.prev.addEventListener("click", function () {
      goTo(index - 1);
    });
    el.next.addEventListener("click", function () {
      goTo(index + 1);
    });
    el.submit.addEventListener("click", onSubmit);
    el.submitConfirmAccept.addEventListener("click", commitAttempt);
    el.submitConfirmCancel.addEventListener("click", hideSubmitConfirm);
    el.submitConfirmReview.addEventListener("click", function () {
      var missing = missingIndices();
      hideSubmitConfirm();
      if (missing.length) goTo(missing[0]);
    });
    el.flag.addEventListener("click", toggleFlag);
    el.reviewUnanswered.addEventListener("click", function () {
      if (!pack) return;
      for (var i = 0; i < pack.items.length; i++) {
        if (!answers[pack.items[i].id]) {
          goTo(i);
          return;
        }
      }
    });
    document.addEventListener("keydown", onKeydown);
    document.addEventListener("click", onDocumentClick, true);
    window.addEventListener("beforeunload", onBeforeUnload);
    if (el.seedSelect) {
      el.seedSelect.addEventListener("change", onSeedChange);
    }
    if (el.closedNotesToggle) {
      el.closedNotesToggle.addEventListener("change", onClosedNotesToggle);
    }
  }

  function loadPack(seed) {
    var url = packUrl(seed);
    activeSeed = seed;
    updatePackMeta();
    if (el.seedSelect) {
      // Keep select in sync; add custom option if seed not in preset list.
      var found = false;
      for (var i = 0; i < el.seedSelect.options.length; i++) {
        if (el.seedSelect.options[i].value === String(seed)) {
          found = true;
          break;
        }
      }
      if (!found) {
        var opt = document.createElement("option");
        opt.value = String(seed);
        opt.textContent = String(seed) + " (custom)";
        el.seedSelect.appendChild(opt);
      }
      el.seedSelect.value = String(seed);
    }

    el.status.hidden = false;
    el.status.className = "exam-status";
    el.status.textContent = priorAttemptFound
      ? "Starting a fresh attempt after the previous submission (seed " + seed + ")…"
      : "Loading exam pack (seed " + seed + ")…";
    if (el.card) el.card.hidden = true;

    fetch(url, { cache: "no-store" })
      .then(function (res) {
        if (res.status === 404) {
          throw new Error(missingPackHint(seed));
        }
        if (!res.ok) {
          throw new Error("HTTP " + res.status + " loading " + url);
        }
        return res.json();
      })
      .then(function (data) {
        if (!data || !Array.isArray(data.items) || data.items.length === 0) {
          throw new Error("Pack missing items[]");
        }
        if (data.items.length !== 40) {
          console.warn(
            "Expected 40 items, got " + data.items.length + " — continuing with pack size"
          );
        }
        // Validate each item has 4 choices
        for (var i = 0; i < data.items.length; i++) {
          var it = data.items[i];
          if (!it.id || !it.stem || !Array.isArray(it.choices) || it.choices.length < 4) {
            throw new Error("Invalid item at index " + i);
          }
        }
        var packSeed = typeof data.seed === "number" ? data.seed : seed;
        pack = {
          exam_id: data.exam_id || "mock40",
          seed: packSeed,
          bank_hash: data.bank_hash || "",
          items: data.items,
        };

        // Drop draft if pack identity changed
        try {
          var raw = sessionStorage.getItem(STORAGE_DRAFT);
          if (raw) {
            var draft = JSON.parse(raw);
            if (
              draft &&
              (draft.exam_id !== pack.exam_id ||
                draft.seed !== pack.seed ||
                draft.bank_hash !== pack.bank_hash)
            ) {
              answers = Object.create(null);
              flags = Object.create(null);
              index = 0;
              timerStartedAt = null;
            }
          }
        } catch (_) {
          /* ignore */
        }

        if (index >= pack.items.length) index = 0;

        el.status.hidden = true;
        el.card.hidden = false;
        buildJumpStrip();
        startTimer();
        renderQuestion();
        // Attempt is active — apply closed-notes lock if preferred on.
        applyClosedNotesChrome();
      })
      .catch(function (err) {
        var msg = err && err.message ? err.message : String(err);
        if (msg.indexOf("does not include a mock exam") !== -1) {
          showError(msg);
          return;
        }
        showError(
          "Could not load the mock exam (" +
            url +
            "). Open the study site over HTTP (the URL the app printed), not as a file:// page. " +
            msg
        );
      });
  }

  function init() {
    bind();
    loadClosedNotesPref();
    if (el.closedNotesToggle) {
      el.closedNotesToggle.checked = closedNotes;
    }
    // Chrome before pack load: toggle reflects preference; lock applies once attempt active.
    applyClosedNotesChrome();
    try {
      priorAttemptFound = !!sessionStorage.getItem(STORAGE_ATTEMPT);
      if (priorAttemptFound) sessionStorage.removeItem(STORAGE_DRAFT);
    } catch (_) {
      priorAttemptFound = false;
    }
    if (!priorAttemptFound) loadDraft();
    var seed = resolveSeed();
    loadPack(seed);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
