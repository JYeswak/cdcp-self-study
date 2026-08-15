//! A11y marker smoke — skip-link, honesty, landmark, course.css.
//!
//! Extracted from `scripts/smoke_a11y.py` by
//! `bd-substrate-rust-migration-jhd.19` as product, not a gate file. A
//! learner opens the hub / mock / results / learn / drill / quiz shells. If
//! they can see it, it is not a `cdcp_gate` concern.
//!
//! # Contract
//!
//! Every required primary shell under `web/` carries four markers, and
//! `course.css` carries two:
//!
//! * a skip link (`.skip-link` class token, or `href="#main"` + `Skip`, or
//!   the copy "Skip to content" / "Skip to main content")
//! * an honesty signal: `.honesty-banner` **with** non-grant / meta language,
//!   or the language alone (banner class with no copy is a hollow shell)
//! * a `<main>` / `role=main` landmark (`id=main` alone is the other message)
//! * an `href` that **ends** with `assets/css/course.css`
//! * `:focus-visible` (case-sensitive) in the stylesheet
//! * `--touch-min` as a token (case-sensitive; `--touch-minimum` is not it)
//!
//! # Anti-vacuous
//!
//! A missing `web/`, a missing required page, an empty page, an empty
//! stylesheet, a scan that checked zero pages, and an undecodable or
//! unreadable file are each a named FAIL. The retired script **raised**
//! `UnicodeDecodeError` on invalid UTF-8 and printed a traceback
//! (`bd-a11y-undecodable-raises-6jmi`). Closed here: that is a FAIL row.
//!
//! This is a READER: it writes nothing. It is a marker census over
//! comment-stripped source, not a tree walk and not an axe run.
//! `.parked-wave8/` stays parked.
//!
//! # What this cannot decide
//!
//! Accessibility. Position, contrast, labels, heading order, alt text, tab
//! order, ARIA correctness, motion. HTML comments and `<script>` / `<style>`
//! bodies are stripped before the census (`bd-a11y-comment-blind-udze`): a
//! marker that exists only there does not satisfy. A marker in an attribute
//! string or a `display:none` element still would. Only the shells named
//! below are scanned — `web/learn/*.html` is outside.

#![forbid(unsafe_code)]

use crate::{join_rel, BuildOutcome};
use std::path::Path;

pub const NAME: &str = "smoke-a11y";
pub const SUMMARY: &str =
    "L7-S5 a11y marker baseline over the primary web/ page shells and course.css";

/// Required primary shells, in report order.
pub const REQUIRED_PAGES: &[&str] = &[
    "index.html",
    "mock.html",
    "results.html",
    "learn.html",
    "drill.html",
    "quiz.html",
];

/// Optional primary shells — checked only when present (L7-S4 reference).
pub const OPTIONAL_PAGES: &[&str] = &["reference.html"];

pub const CSS_REL: &str = "web/assets/css/course.css";
pub const CSS_HREF_TAIL: &str = "assets/css/course.css";

/// Compiled-in floor so emptying the check list cannot go green.
///
/// Green fixture: CSS presence + 2 CSS tokens + 6 required pages × 4 markers.
pub const MIN_CHECKS: usize = 1 + 2 + REQUIRED_PAGES.len() * 4;

// ── character classes ──────────────────────────────────────────────────────

/// Python `re` `\s` on `str` patterns: Unicode whitespace plus the four
/// ASCII information separators (`str.isspace()` includes them).
fn py_space(c: char) -> bool {
    c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c)
}

/// `\w`: alphanumeric or `_`. Used only for `\b`.
fn py_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn boundary(ch: &[char], i: usize) -> bool {
    let before = i > 0 && py_word(ch[i - 1]);
    let after = i < ch.len() && py_word(ch[i]);
    before != after
}

/// ASCII case-insensitive compare. Product contract: the live shells are
/// ASCII. We do not reproduce Python `re.I` Unicode folding (long-s / kelvin).
fn ci_char(pat: char, c: char) -> bool {
    if c == pat {
        return true;
    }
    let l = pat.to_ascii_lowercase();
    if l.is_ascii_lowercase() {
        c == l || c == l.to_ascii_uppercase()
    } else {
        false
    }
}

fn lit_at(ch: &[char], i: usize, pat: &str, ci: bool) -> Option<usize> {
    let mut k = i;
    for p in pat.chars() {
        let c = *ch.get(k)?;
        let hit = if ci { ci_char(p, c) } else { c == p };
        if !hit {
            return None;
        }
        k += 1;
    }
    Some(k)
}

fn is_quote(c: char) -> bool {
    c == '"' || c == '\''
}

fn skip_space(ch: &[char], i: usize) -> usize {
    let mut k = i;
    while k < ch.len() && py_space(ch[k]) {
        k += 1;
    }
    k
}

fn skip_space_plus(ch: &[char], i: usize) -> Option<usize> {
    let k = skip_space(ch, i);
    (k > i).then_some(k)
}

/// End of a `[^"']*` run starting at `i`. `None` if no closing quote.
fn quoted_run(ch: &[char], i: usize) -> Option<usize> {
    let mut e = i;
    while e < ch.len() && !is_quote(ch[e]) {
        e += 1;
    }
    (e < ch.len()).then_some(e)
}

fn positions(ch: &[char], pat: &str, ci: bool) -> Vec<usize> {
    (0..=ch.len())
        .filter(|&i| lit_at(ch, i, pat, ci).is_some())
        .collect()
}

// ── the six markers ────────────────────────────────────────────────────────

/// `class=["'][^"']*\bNEEDLE\b[^"']*["']` under ASCII `re.I`.
pub fn class_attr_has_word(ch: &[char], needle: &str) -> bool {
    let nlen = needle.chars().count();
    for i in positions(ch, "class=", true) {
        let j = i + 6;
        if !matches!(ch.get(j), Some(c) if is_quote(*c)) {
            continue;
        }
        let k = j + 1;
        let Some(e) = quoted_run(ch, k) else { continue };
        let mut p = k;
        while p + nlen <= e {
            if lit_at(ch, p, needle, true).is_some() && boundary(ch, p) && boundary(ch, p + nlen) {
                return true;
            }
            p += 1;
        }
    }
    false
}

fn has_skip_href(ch: &[char]) -> bool {
    for i in positions(ch, "href=", true) {
        let j = i + 5;
        if !matches!(ch.get(j), Some(c) if is_quote(*c)) {
            continue;
        }
        let Some(m) = lit_at(ch, j + 1, "#main", true) else {
            continue;
        };
        if !matches!(ch.get(m), Some(c) if is_quote(*c)) {
            continue;
        }
        let mut q = m + 1;
        while q < ch.len() && ch[q] != '>' {
            q += 1;
        }
        if q >= ch.len() {
            continue;
        }
        let r = skip_space(ch, q + 1);
        if let Some(s) = lit_at(ch, r, "Skip", true) {
            if boundary(ch, s) {
                return true;
            }
        }
    }
    false
}

fn has_skip_copy(ch: &[char]) -> bool {
    for (i, c) in ch.iter().enumerate() {
        if *c != '>' {
            continue;
        }
        let p = skip_space(ch, i + 1);
        let Some(q) = lit_at(ch, p, "Skip to ", true) else {
            continue;
        };
        let mut starts = Vec::new();
        if let Some(m) = lit_at(ch, q, "main ", true) {
            starts.push(m);
        }
        starts.push(q);
        for s in starts {
            if let Some(e) = lit_at(ch, s, "content", true) {
                let z = skip_space(ch, e);
                if ch.get(z) == Some(&'<') {
                    return true;
                }
            }
        }
    }
    false
}

pub fn has_skip_link(ch: &[char]) -> bool {
    class_attr_has_word(ch, "skip-link") || has_skip_href(ch) || has_skip_copy(ch)
}

pub fn has_honesty_banner(ch: &[char]) -> bool {
    class_attr_has_word(ch, "honesty-banner")
}

fn honesty_copy_at(ch: &[char], i: usize) -> bool {
    if let Some(a) = lit_at(ch, i, "does", true) {
        if let Some(b) = skip_space_plus(ch, a) {
            let mut opens = Vec::new();
            if let Some(o) = lit_at(ch, b, "<strong>", true) {
                opens.push(o);
            }
            opens.push(b);
            for o in opens {
                let Some(n) = lit_at(ch, o, "not", true) else {
                    continue;
                };
                let mut closes = Vec::new();
                if let Some(c) = lit_at(ch, n, "</strong>", true) {
                    closes.push(c);
                }
                closes.push(n);
                for c in closes {
                    let Some(d) = skip_space_plus(ch, c) else {
                        continue;
                    };
                    let Some(e) = lit_at(ch, d, "grant", true) else {
                        continue;
                    };
                    let Some(f) = skip_space_plus(ch, e) else {
                        continue;
                    };
                    let Some(g) = lit_at(ch, f, "EPI/EXIN", true) else {
                        continue;
                    };
                    let Some(h) = skip_space_plus(ch, g) else {
                        continue;
                    };
                    if lit_at(ch, h, "certification", true).is_some() {
                        return true;
                    }
                }
            }
        }
    }
    if let Some(a) = lit_at(ch, i, "not", true) {
        if let Some(b) = skip_space_plus(ch, a) {
            if let Some(c) = lit_at(ch, b, "EPI/EXIN", true) {
                if let Some(d) = skip_space_plus(ch, c) {
                    if lit_at(ch, d, "certification", true).is_some() {
                        return true;
                    }
                }
            }
        }
    }
    if let Some(a) = lit_at(ch, i, "study", true) {
        if let Some(b) = skip_space_plus(ch, a) {
            for word in ["tool", "signal"] {
                let Some(c) = lit_at(ch, b, word, true) else {
                    continue;
                };
                let Some(d) = skip_space_plus(ch, c) else {
                    continue;
                };
                if lit_at(ch, d, "only", true).is_some() {
                    return true;
                }
            }
        }
    }
    false
}

pub fn has_honesty_copy(ch: &[char]) -> bool {
    (0..=ch.len()).any(|i| honesty_copy_at(ch, i))
}

fn honesty_meta_tail_at(ch: &[char], i: usize) -> bool {
    if let Some(a) = lit_at(ch, i, "does", true) {
        if let Some(b) = skip_space_plus(ch, a) {
            if let Some(c) = lit_at(ch, b, "not", true) {
                if let Some(d) = skip_space_plus(ch, c) {
                    if lit_at(ch, d, "grant", true).is_some() {
                        return true;
                    }
                }
            }
        }
    }
    if let Some(a) = lit_at(ch, i, "not", true) {
        if let Some(b) = skip_space_plus(ch, a) {
            if lit_at(ch, b, "EPI/EXIN", true).is_some() {
                return true;
            }
        }
    }
    if let Some(a) = lit_at(ch, i, "study", true) {
        if let Some(b) = skip_space_plus(ch, a) {
            for word in ["tool", "signal"] {
                let Some(c) = lit_at(ch, b, word, true) else {
                    continue;
                };
                let Some(d) = skip_space_plus(ch, c) else {
                    continue;
                };
                if lit_at(ch, d, "only", true).is_some() {
                    return true;
                }
            }
        }
    }
    false
}

pub fn has_honesty_meta(ch: &[char]) -> bool {
    let n = ch.len();
    for i in positions(ch, "<meta", true) {
        let after = i + 5;
        let mut gt = after;
        while gt < n && ch[gt] != '>' {
            gt += 1;
        }
        for j in (after + 1)..=gt.min(n) {
            let Some(a) = lit_at(ch, j, "name=", true) else {
                continue;
            };
            if !matches!(ch.get(a), Some(c) if is_quote(*c)) {
                continue;
            }
            let mut names = Vec::new();
            if let Some(v) = lit_at(ch, a + 1, "description", true) {
                names.push(v);
            }
            if let Some(v) = lit_at(ch, a + 1, "honesty", true) {
                names.push(v);
            }
            for v in names {
                if !matches!(ch.get(v), Some(c) if is_quote(*c)) {
                    continue;
                }
                let k = v + 1;
                let mut gt2 = k;
                while gt2 < n && ch[gt2] != '>' {
                    gt2 += 1;
                }
                for m in k..=gt2.min(n) {
                    let Some(c) = lit_at(ch, m, "content=", true) else {
                        continue;
                    };
                    if !matches!(ch.get(c), Some(q) if is_quote(*q)) {
                        continue;
                    }
                    let start = c + 1;
                    let end = quoted_run(ch, start).unwrap_or(n);
                    if (start..=end).any(|p| honesty_meta_tail_at(ch, p)) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

pub fn has_main_landmark(ch: &[char]) -> bool {
    for i in positions(ch, "<main", true) {
        if boundary(ch, i + 5) {
            return true;
        }
    }
    for i in positions(ch, "role=", true) {
        let j = i + 5;
        if !matches!(ch.get(j), Some(c) if is_quote(*c)) {
            continue;
        }
        let Some(m) = lit_at(ch, j + 1, "main", true) else {
            continue;
        };
        if matches!(ch.get(m), Some(c) if is_quote(*c)) {
            return true;
        }
    }
    false
}

pub fn has_id_main(ch: &[char]) -> bool {
    for i in positions(ch, "id=", true) {
        let j = i + 3;
        if !matches!(ch.get(j), Some(c) if is_quote(*c)) {
            continue;
        }
        let Some(m) = lit_at(ch, j + 1, "main", true) else {
            continue;
        };
        if matches!(ch.get(m), Some(c) if is_quote(*c)) {
            return true;
        }
    }
    false
}

/// `href=["'][^"']*assets/css/course.css["']` — the literal must END the
/// quote-free run. `course.css?v=2` does not count.
pub fn has_course_css(ch: &[char]) -> bool {
    let tail = CSS_HREF_TAIL.chars().count();
    for i in positions(ch, "href=", true) {
        let j = i + 5;
        if !matches!(ch.get(j), Some(c) if is_quote(*c)) {
            continue;
        }
        let k = j + 1;
        let Some(e) = quoted_run(ch, k) else { continue };
        if e >= k + tail && lit_at(ch, e - tail, CSS_HREF_TAIL, true) == Some(e) {
            return true;
        }
    }
    false
}

/// `:focus-visible\b` — case-sensitive.
pub fn has_focus_visible(ch: &[char]) -> bool {
    let n = ":focus-visible".chars().count();
    (0..=ch.len())
        .filter(|&i| lit_at(ch, i, ":focus-visible", false).is_some())
        .any(|i| boundary(ch, i + n))
}

/// `--touch-min\b` — case-sensitive.
pub fn has_touch_min(ch: &[char]) -> bool {
    let n = "--touch-min".chars().count();
    (0..=ch.len())
        .filter(|&i| lit_at(ch, i, "--touch-min", false).is_some())
        .any(|i| boundary(ch, i + n))
}

// ── ignore comments / hidden hosts before the census ───────────────────────

/// Drop `<!-- ... -->` and `<script>` / `<style>` element bodies.
///
/// A marker that exists only in those hosts is not in the DOM the learner
/// tabs through. Unclosed comment / script / style consumes the rest of
/// the file (fail-closed: hidden text cannot go green).
pub fn strip_ignored_markup(text: &str) -> String {
    let ch: Vec<char> = text.chars().collect();
    let n = ch.len();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < n {
        if lit_at(&ch, i, "<!--", false).is_some() {
            i += 4;
            while i < n && lit_at(&ch, i, "-->", false).is_none() {
                i += 1;
            }
            if i < n {
                i += 3;
            }
            continue;
        }
        if let Some(open_end) = tag_open_end(&ch, i, "script") {
            i = tag_close_end(&ch, open_end, "script").unwrap_or(n);
            continue;
        }
        if let Some(open_end) = tag_open_end(&ch, i, "style") {
            i = tag_close_end(&ch, open_end, "style").unwrap_or(n);
            continue;
        }
        out.push(ch[i]);
        i += 1;
    }
    out
}

/// `<name` + (space | `/` | `>`) … `>`. Quotes hide a `>` in an attribute.
fn tag_open_end(ch: &[char], i: usize, name: &str) -> Option<usize> {
    if ch.get(i) != Some(&'<') {
        return None;
    }
    let after = lit_at(ch, i + 1, name, true)?;
    match ch.get(after) {
        Some(c) if py_space(*c) || *c == '/' || *c == '>' => {}
        _ => return None,
    }
    let mut k = after;
    let mut quote: Option<char> = None;
    while k < ch.len() {
        let c = ch[k];
        if let Some(q) = quote {
            if c == q {
                quote = None;
            }
        } else if is_quote(c) {
            quote = Some(c);
        } else if c == '>' {
            return Some(k + 1);
        }
        k += 1;
    }
    None
}

/// First `</name` … `>` at or after `start`.
fn tag_close_end(ch: &[char], start: usize, name: &str) -> Option<usize> {
    let mut k = start;
    while k < ch.len() {
        if ch[k] == '<' && ch.get(k + 1) == Some(&'/') {
            if let Some(after) = lit_at(ch, k + 2, name, true) {
                if matches!(ch.get(after), Some(c) if py_space(*c) || *c == '>') {
                    let mut e = after;
                    while e < ch.len() && ch[e] != '>' {
                        e += 1;
                    }
                    if e < ch.len() {
                        return Some(e + 1);
                    }
                    return None;
                }
            }
        }
        k += 1;
    }
    None
}

// ── the smoke ──────────────────────────────────────────────────────────────

pub fn check_page(rel: &str, text: &str) -> Vec<String> {
    let visible = strip_ignored_markup(text);
    let ch: Vec<char> = visible.chars().collect();
    let mut errors = Vec::new();

    if !has_skip_link(&ch) {
        errors.push(format!(
            "{rel}: missing skip link (.skip-link or Skip to main content)"
        ));
    }

    let banner = has_honesty_banner(&ch);
    let meta = has_honesty_meta(&ch);
    let copy = has_honesty_copy(&ch);
    if !(banner || meta || copy) {
        errors.push(format!(
            "{rel}: missing honesty banner (.honesty-banner) and meta honesty language"
        ));
    } else if banner && !copy && !meta {
        errors.push(format!(
            "{rel}: honesty-banner present but no non-grant / meta honesty language"
        ));
    }

    if !has_main_landmark(&ch) {
        if has_id_main(&ch) {
            errors.push(format!(
                "{rel}: #main present but missing landmark element (<main> or role=main)"
            ));
        } else {
            errors.push(format!(
                "{rel}: missing main/content landmark (<main> or role=main)"
            ));
        }
    }

    if !has_course_css(&ch) {
        errors.push(format!("{rel}: missing course.css stylesheet link"));
    }

    errors
}

pub fn check_css(text: &str) -> Vec<String> {
    let ch: Vec<char> = text.chars().collect();
    let mut errors = Vec::new();
    if !has_focus_visible(&ch) {
        errors.push("course.css: missing :focus-visible rule".to_string());
    }
    if !has_touch_min(&ch) {
        errors.push("course.css: missing --touch-min token".to_string());
    }
    errors
}

fn py_strip(s: &str) -> &str {
    s.trim_matches(py_space)
}

/// Read as UTF-8. Invalid UTF-8 and IO errors are named, never a panic.
fn read_utf8(p: &Path) -> Result<String, ReadFail> {
    let bytes = std::fs::read(p).map_err(ReadFail::Io)?;
    String::from_utf8(bytes).map_err(|_| ReadFail::NotUtf8)
}

enum ReadFail {
    Io(std::io::Error),
    NotUtf8,
}

impl ReadFail {
    fn row(&self, rel: &str) -> String {
        match self {
            ReadFail::NotUtf8 => {
                format!("{rel}: not valid UTF-8 \u{2014} refusing vacuous green")
            }
            ReadFail::Io(e) => {
                format!("{rel}: unreadable ({e}) \u{2014} refusing vacuous green")
            }
        }
    }
}

fn outcome(code: i32, stdout: impl Into<String>) -> BuildOutcome {
    BuildOutcome {
        stdout: stdout.into(),
        code,
        artifact: None,
    }
}

/// Run the a11y marker smoke against `root` (the course-engine directory).
///
/// Reader: writes nothing. `code != 0` is RED. `artifact` is always `None`.
pub fn run(root: &Path) -> BuildOutcome {
    let web = join_rel(root, "web");
    let css = join_rel(root, CSS_REL);

    if !web.is_dir() {
        return outcome(1, "FAIL: smoke_a11y \u{2014} missing web/\n");
    }

    let mut errors: Vec<String> = Vec::new();
    let mut pages_checked = 0usize;
    let mut checks = 0usize;

    if !css.is_file() {
        errors.push(format!("missing {CSS_REL}"));
    } else {
        checks += 1;
        match read_utf8(&css) {
            Err(e) => errors.push(e.row(CSS_REL)),
            Ok(text) => {
                if py_strip(&text).is_empty() {
                    errors.push("course.css is empty \u{2014} refusing vacuous green".into());
                } else {
                    let css_errs = check_css(&text);
                    checks += 2;
                    errors.extend(css_errs);
                }
            }
        }
    }

    for name in REQUIRED_PAGES {
        let path = web.join(name);
        let rel = format!("web/{name}");
        if !path.is_file() {
            errors.push(format!("missing required primary page {rel}"));
            continue;
        }
        match read_utf8(&path) {
            Err(e) => errors.push(e.row(&rel)),
            Ok(text) => {
                if py_strip(&text).is_empty() {
                    errors.push(format!("{rel}: empty file \u{2014} refusing vacuous green"));
                    continue;
                }
                pages_checked += 1;
                let page_errs = check_page(&rel, &text);
                checks += 4;
                errors.extend(page_errs);
            }
        }
    }

    let mut optional_checked = 0usize;
    for name in OPTIONAL_PAGES {
        let path = web.join(name);
        if !path.is_file() {
            continue;
        }
        let rel = format!("web/{name}");
        match read_utf8(&path) {
            Err(e) => errors.push(e.row(&rel)),
            Ok(text) => {
                if py_strip(&text).is_empty() {
                    errors.push(format!("{rel}: empty file \u{2014} refusing vacuous green"));
                    continue;
                }
                pages_checked += 1;
                optional_checked += 1;
                let page_errs = check_page(&rel, &text);
                checks += 4;
                errors.extend(page_errs);
            }
        }
    }

    if pages_checked == 0 {
        errors.push(format!(
            "zero primary HTML pages checked \u{2014} refusing vacuous green \
             (expected at least {} required pages)",
            REQUIRED_PAGES.len()
        ));
    }

    if errors.is_empty() && checks < MIN_CHECKS {
        errors.push(format!(
            "performed {checks} check(s) < MIN_CHECKS={MIN_CHECKS} \u{2014} \
             a smoke that checked nothing cannot PASS"
        ));
    }

    if !errors.is_empty() {
        let mut out = String::from("FAIL: smoke_a11y\n");
        for e in &errors {
            out.push_str("  - ");
            out.push_str(e);
            out.push('\n');
        }
        return outcome(1, out);
    }

    let mut out = String::from("PASS: smoke_a11y\n");
    out.push_str(&format!("  pages_checked={pages_checked}\n"));
    out.push_str(&format!("  required={}\n", REQUIRED_PAGES.len()));
    out.push_str(&format!("  optional_present={optional_checked}\n"));
    out.push_str(&format!("  css={CSS_REL}\n"));
    out.push_str(
        "  checks=skip-link \u{b7} honesty \u{b7} main landmark \u{b7} course.css \u{b7} :focus-visible \u{b7} --touch-min\n",
    );
    for name in REQUIRED_PAGES {
        out.push_str(&format!("  ok web/{name}\n"));
    }
    for name in OPTIONAL_PAGES {
        if web.join(name).is_file() {
            out.push_str(&format!("  ok web/{name} (optional)\n"));
        }
    }
    outcome(0, out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> Vec<char> {
        s.chars().collect()
    }

    #[test]
    fn min_checks_matches_compiled_lists() {
        assert_eq!(MIN_CHECKS, 1 + 2 + REQUIRED_PAGES.len() * 4);
        assert!(MIN_CHECKS > 0);
        assert_eq!(REQUIRED_PAGES.len(), 6);
        assert_eq!(OPTIONAL_PAGES.len(), 1);
        assert!(!CSS_HREF_TAIL.is_empty());
    }

    #[test]
    fn skip_link_class_needs_word_boundaries() {
        assert!(has_skip_link(&v(r#"<a class="skip-link">x</a>"#)));
        assert!(has_skip_link(&v(r#"<a class="btn skip-link big">x</a>"#)));
        assert!(has_skip_link(&v(r#"<a class="my-skip-link">x</a>"#)));
        assert!(!has_skip_link(&v(r#"<a class="skip-links">x</a>"#)));
        assert!(!has_skip_link(&v(r#"<a class="askip-link">x</a>"#)));
    }

    #[test]
    fn mainframe_is_not_a_landmark() {
        assert!(has_main_landmark(&v("<main>y</main>")));
        assert!(has_main_landmark(&v("<MAIN>y</MAIN>")));
        assert!(!has_main_landmark(&v("<mainframe>y</mainframe>")));
        assert!(has_main_landmark(&v(r#"<div role="main">y</div>"#)));
        assert!(has_main_landmark(&v(r#"<div role='main'>y</div>"#)));
    }

    #[test]
    fn css_href_must_end_the_attribute() {
        assert!(has_course_css(&v(r#"<link href="assets/css/course.css">"#)));
        assert!(has_course_css(&v(
            r#"<link href="../assets/css/course.css">"#
        )));
        assert!(!has_course_css(&v(
            r#"<link href="assets/css/course.css?v=2">"#
        )));
    }

    #[test]
    fn css_tokens_are_case_sensitive() {
        assert!(has_focus_visible(&v("a:focus-visible{outline:2px}")));
        assert!(!has_focus_visible(&v("a:FOCUS-VISIBLE{outline:2px}")));
        assert!(has_touch_min(&v(":root{--touch-min:44px}")));
        assert!(!has_touch_min(&v(":root{--TOUCH-MIN:44px}")));
        assert!(!has_touch_min(&v(":root{--touch-minimum:44px}")));
    }

    #[test]
    fn hollow_banner_is_a_distinct_finding() {
        let errs = check_page(
            "web/x.html",
            r#"<link href="assets/css/course.css"><a class="skip-link">x</a>
               <div class="honesty-banner">Welcome</div><main>y</main>"#,
        );
        assert_eq!(errs.len(), 1, "{errs:?}");
        assert!(errs[0].contains("honesty-banner present but no non-grant"));
    }

    #[test]
    fn comment_only_markers_do_not_satisfy_check_page() {
        let html = concat!(
            "<!--",
            r#"<link href="assets/css/course.css"><a class="skip-link">x</a>"#,
            r#"<div class="honesty-banner">study tool only</div><main>y</main>"#,
            "-->",
            "<html><body>visible but bare</body></html>",
        );
        let errs = check_page("web/x.html", html);
        assert_eq!(errs.len(), 4, "{errs:?}");
        assert!(html.contains("skip-link") && html.contains("honesty-banner"));
        assert!(!strip_ignored_markup(html).contains("skip-link"));
    }

    #[test]
    fn script_and_style_bodies_do_not_satisfy_check_page() {
        let scripted = concat!(
            "<script>",
            r#"<link href="assets/css/course.css"><a class="skip-link">x</a>"#,
            r#"<div class="honesty-banner">study tool only</div><main>y</main>"#,
            "</script>",
            "<p>bare</p>",
        );
        assert_eq!(check_page("web/x.html", scripted).len(), 4);
        let styled = concat!(
            "<style>",
            r#"<link href="assets/css/course.css"><a class="skip-link">x</a>"#,
            r#"<div class="honesty-banner">study tool only</div><main>y</main>"#,
            "</style>",
            "<p>bare</p>",
        );
        assert_eq!(check_page("web/x.html", styled).len(), 4);
    }

    #[test]
    fn comment_beside_live_markers_does_not_hide_them() {
        let html = concat!(
            r#"<link href="assets/css/course.css">"#,
            r#"<!-- <a class="skip-link">dead</a> -->"#,
            r#"<a class="skip-link">x</a>"#,
            r#"<div class="honesty-banner">study tool only</div><main>y</main>"#,
        );
        assert!(check_page("web/x.html", html).is_empty());
    }
}
