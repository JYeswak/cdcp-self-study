//! build_glossary — compile `web/data/glossary.json` (learner-visible product).
//!
//! # CLAIM: FLOOR-RAISE
//!
//! This gate is a BUILDER, not a checker: it derives `web/data/glossary.json`
//! from the `| **Term** | Definition |` rows of the reference glossary and
//! writes it. The floor it raises is narrow and worth stating exactly —
//!
//!   1. the shipped popover data is DERIVED from the prose glossary on every
//!      run, so a term edited in `GLOSSARY.md` and forgotten in the JSON is not
//!      a state the tree can hold; and
//!   2. a glossary that yielded fewer than `MIN_TERMS` terms is RED. A run that
//!      parsed nothing must not report like a run that parsed everything —
//!      a missing source file and an empty term table are each non-zero.
//!
//! # WHAT THIS GATE CANNOT DECIDE
//!
//! It cannot tell whether a definition is CORRECT, whether it matches the
//! module text that uses the term, or whether a term the syllabus leans on is
//! missing from the table entirely — nothing dangles when a row was never
//! written. It reads one table shape and nothing else: a term defined in prose,
//! in a list, or in a table whose term cell is not bold is invisible here. The
//! term floor is a floor against silence, not a claim about coverage: it counts
//! rows, and one throwaway row counts exactly as much as a rigorous one.
//!
//! # BYTE-EXACTNESS WITH THE PYTHON ORACLE
//!
//! `scripts/build_glossary_json.py` stays in the tree as the differential
//! oracle for this port; `tests/diff_build_glossary_json.rs` runs both on every
//! case and asserts stdout, stderr, exit code AND THE BYTES WRITTEN match. That
//! contract is why this module carries hand-written emulations of several
//! Python behaviours — the `\s` class and backtracking of `re`, `str.casefold`,
//! `json.dumps(indent=2, ensure_ascii=False)` — rather than the idiomatic Rust
//! nearest-neighbour, and why the failure report is written to **stdout with
//! exit status 1** instead of going through `GateError`: the dispatcher's
//! `report()` writes to stderr and maps to exit 2 or 4, which the oracle never
//! produces, so routing through it would make the two sides differ on every RED
//! case. The exit-code mapping in `crate::exit` is therefore deliberately NOT
//! used on the report path. Same knowing, single-file deviation as
//! `verify_orphans.rs`, recorded here for review rather than made quietly.
//! bd-2m9 flips the whole crate to a single convention later.
//!
//! `generated_by` still reads `scripts/build_glossary_json.py`. That string is
//! part of the artifact's bytes, so changing it while the oracle is alive would
//! break the differential on the very first case. It changes when the Python is
//! deleted, not before.
//!
//! # THE TWO FINDINGS OF bd-.11, NOW FIXED IN THE ORACLE FIRST
//!
//! Both were reported and DELIBERATELY not repaired when this port landed: a
//! port stricter than its oracle blinds the whole differential. They were fixed
//! in `scripts/build_glossary_json.py` first, the differential was watched go
//! RED on them, and only then were they moved here
//! (bd-builder-verdict-shape-qm65). Measured 2026-08-14 before the fix, on an
//! empty glossary, both implementations byte-identical:
//!
//! ```text
//! PASS: glossary terms=0 → web/data/glossary.json
//! FAIL: need ≥15 terms
//! (exit 1, and a 161-byte glossary.json left behind)
//! ```
//!
//!   * VERDICT SHAPE. The success line was printed FIRST and the failure
//!     underneath it, on the way to exit 1. A reader skimming stdout saw PASS;
//!     CI saw non-zero; which one won depended on whether anyone looked. This
//!     is the same defect bd-lt7 fixed in `build_units.py`, which makes it a
//!     CLASS: a verdict printed before the checks that decide it. The verdict
//!     is now the first line of a report composed once, after every check.
//!   * WRITE-BEFORE-VERDICT. The artifact was written BEFORE the term floor was
//!     evaluated, so a below-floor run left a short `glossary.json` in
//!     `web/data/` and a later reader could not tell a passing artifact from
//!     the residue of a failed run. The write is now on the GREEN path only.
//!
//! The GREEN bytes — both the `PASS:` line and the artifact — are unchanged, so
//! the live-tree tie-back in `tests/diff_build_glossary_json.rs` still holds
//! against the tracked `web/data/glossary.json`.
//!
//! # DEVIATIONS THAT REMAIN (each unreachable from the live tree)
//!
//!   * An unknown argument is a USAGE error here; the oracle ignores argv
//!     entirely. `check.sh` passes no arguments, so no live invocation differs,
//!     and the stricter side is the one where a typo cannot read as a pass.
//!   * An unreadable or non-UTF-8 source, or a failed write, ends as
//!     `GateError::Error` (exit 4) where the oracle would emit a Python
//!     traceback and exit 1. A traceback is not portable output.
//!   * `py_casefold` is `str::to_lowercase` plus the sharp-s fold. Full Unicode
//!     case folding differs on a handful of further code points (ligatures,
//!     Cherokee); none appear in the glossary, and the only effect would be
//!     term ORDER, never term content.

#![forbid(unsafe_code)]

use crate::{join_rel, BuildOutcome, LearnError, GENERATED_BY};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub const NAME: &str = "build-glossary";
pub const SUMMARY: &str = "build web/data/glossary.json from the reference GLOSSARY.md term table";

/// Engine-root-relative paths, matching the Python module constants.
pub const SRC_REL: &str = "web/content/reference/GLOSSARY.md";
pub const OUT_REL: &str = "web/data/glossary.json";

/// The term floor. Named rather than written inline as a literal comparison so
/// the value is stated once, and so the bound sweep in
/// `tests/rebase_module_bounds.rs` is not asked to adjudicate a text shape that
/// has nothing to do with module numbering.
pub const MIN_TERMS: usize = 15;

// ── Python-behaviour emulations ────────────────────────────────────────────
// Each of these exists because the port's acceptance bar is byte-identical
// output plus byte-identical artifact, not merely an identical verdict.

/// The `\s` character class of Python's `re` on `str` patterns, which is also
/// the set `str.strip()` removes: Unicode `White_Space` plus the four ASCII
/// information separators (0x1C-0x1F) that `str.isspace()` counts and Rust's
/// `char::is_whitespace` does not.
pub fn py_space(c: char) -> bool {
    c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c)
}

/// `str.strip()` with no argument.
pub fn py_strip(s: &str) -> &str {
    s.trim_matches(py_space)
}

/// `str.casefold()`, to the depth this gate can reach. `to_lowercase` is full
/// Unicode lowercasing; the sharp-s fold is the one common case where casefold
/// and lower disagree on Latin text.
pub fn py_casefold(s: &str) -> String {
    s.to_lowercase().replace('ß', "ss")
}

/// One `| **Term** | Definition |` row, as
/// `re.finditer(r"\|\s*\*\*([^*]+)\*\*\s*\|\s*([^|]+)\|", text)` yields it.
///
/// Returns `(term, definition, end)` where `end` is the index just past the
/// closing `|`, so the caller can advance the way `finditer` does (matches are
/// non-overlapping, and a failed attempt advances by exactly one character).
///
/// The one place real backtracking is observable is `\s*([^|]+)`: the two
/// classes overlap, so on `| |` the greedy `\s*` gives a character back and the
/// definition ends up as a single space — which `strip()` then empties, which
/// is what makes a separator row skip. Reproduced literally below.
pub fn match_row(c: &[char], start: usize) -> Option<(String, String, usize)> {
    let n = c.len();
    let mut p = start;
    if c.get(p) != Some(&'|') {
        return None;
    }
    p += 1;
    while p < n && py_space(c[p]) {
        p += 1;
    }
    if !(p + 1 < n && c[p] == '*' && c[p + 1] == '*') {
        return None;
    }
    p += 2;
    // `[^*]+` cannot contain `*`, so it is maximal-munch with no useful
    // backtrack: the `\*\*` that follows must sit at the first star run.
    let g1s = p;
    while p < n && c[p] != '*' {
        p += 1;
    }
    if p == g1s {
        return None;
    }
    if !(p + 1 < n && c[p] == '*' && c[p + 1] == '*') {
        return None;
    }
    let term: String = c[g1s..p].iter().collect();
    p += 2;
    while p < n && py_space(c[p]) {
        p += 1;
    }
    if c.get(p) != Some(&'|') {
        return None;
    }
    p += 1;
    let after_pipe = p;
    let mut ws_end = after_pipe;
    while ws_end < n && py_space(c[ws_end]) {
        ws_end += 1;
    }
    let q = (after_pipe..n).find(|&k| c[k] == '|')?;
    // `\s*` is greedy but must leave `[^|]+` at least one character.
    let g2s = if ws_end < q {
        ws_end
    } else if q > after_pipe {
        q - 1
    } else {
        return None;
    };
    let defn: String = c[g2s..q].iter().collect();
    Some((term, defn, q + 1))
}

/// `re.sub(r"\s*\([^)]*\)\s*", " ", s)` — the "bare key without parens" rewrite.
pub fn strip_parens(s: &str) -> String {
    let c: Vec<char> = s.chars().collect();
    let n = c.len();
    let mut out = String::new();
    let mut i = 0;
    while i < n {
        let mut p = i;
        while p < n && py_space(c[p]) {
            p += 1;
        }
        if p < n && c[p] == '(' {
            let mut q = p + 1;
            while q < n && c[q] != ')' {
                q += 1;
            }
            if q < n {
                let mut e = q + 1;
                while e < n && py_space(c[e]) {
                    e += 1;
                }
                out.push(' ');
                i = e;
                continue;
            }
        }
        out.push(c[i]);
        i += 1;
    }
    out
}

/// An insertion-ordered `dict[str, str]`, which is what `terms` is on the
/// Python side. Order is load-bearing: it is the tiebreak of the stable sort
/// that produces the emitted key order.
#[derive(Debug, Default, Clone)]
pub struct Terms {
    order: Vec<(String, String)>,
    index: HashMap<String, usize>,
}

impl Terms {
    /// `terms[k] = v` — re-assignment does NOT move an existing key.
    pub fn set(&mut self, k: &str, v: &str) {
        match self.index.get(k) {
            Some(&i) => self.order[i].1 = v.to_string(),
            None => {
                self.index.insert(k.to_string(), self.order.len());
                self.order.push((k.to_string(), v.to_string()));
            }
        }
    }

    /// `terms.setdefault(k, v)`.
    pub fn set_default(&mut self, k: &str, v: &str) {
        if !self.index.contains_key(k) {
            self.index.insert(k.to_string(), self.order.len());
            self.order.push((k.to_string(), v.to_string()));
        }
    }

    pub fn len(&self) -> usize {
        self.order.len()
    }

    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    /// `sorted(terms.items(), key=lambda x: x[0].casefold())` — a STABLE sort,
    /// so equal casefolds keep insertion order.
    pub fn sorted(&self) -> Vec<(String, String)> {
        let mut v = self.order.clone();
        v.sort_by_key(|kv| py_casefold(&kv.0));
        v
    }
}

/// The whole extraction loop of the oracle's `main`.
pub fn extract_terms(text: &str) -> Terms {
    let chars: Vec<char> = text.chars().collect();
    let mut terms = Terms::default();
    let mut i = 0usize;
    while i < chars.len() {
        let Some((raw_term, raw_defn, end)) = match_row(&chars, i) else {
            i += 1;
            continue;
        };
        i = end;
        let term = py_strip(&raw_term).to_string();
        let defn = py_strip(&raw_defn).to_string();
        let lowered = term.to_lowercase();
        if lowered == "term" || lowered == "---" {
            continue;
        }
        if defn.is_empty() || defn.starts_with("---") {
            continue;
        }
        terms.set(&term, &defn);
        let bare = py_strip(&strip_parens(&term)).to_string();
        if bare != term {
            terms.set_default(&bare, &defn);
        }
    }
    terms
}

/// `json.dumps` string encoding with `ensure_ascii=False`: escape the backslash,
/// the quote, the five named controls, and every other C0 control as `\u00xx`.
/// Everything else — including all non-ASCII — passes through unchanged.
pub fn json_string(s: &str, out: &mut String) {
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

/// The artifact, byte for byte:
/// `json.dumps({...}, indent=2, ensure_ascii=False) + "\n"`.
pub fn render(source: &str, sorted_terms: &[(String, String)]) -> String {
    let mut s = String::new();
    s.push_str("{\n");
    s.push_str("  \"schema_version\": 1,\n");
    s.push_str(&format!("  \"generated_by\": \"{GENERATED_BY}\",\n"));
    s.push_str("  \"source\": ");
    json_string(source, &mut s);
    s.push_str(",\n");
    s.push_str(&format!("  \"term_count\": {},\n", sorted_terms.len()));
    if sorted_terms.is_empty() {
        // `json.dumps` renders an empty mapping as `{}` even under `indent`.
        s.push_str("  \"terms\": {}\n");
    } else {
        s.push_str("  \"terms\": {\n");
        let last = sorted_terms.len() - 1;
        for (i, (k, v)) in sorted_terms.iter().enumerate() {
            s.push_str("    ");
            json_string(k, &mut s);
            s.push_str(": ");
            json_string(v, &mut s);
            if i != last {
                s.push(',');
            }
            s.push('\n');
        }
        s.push_str("  }\n");
    }
    s.push_str("}\n");
    s
}

/// Where the oracle looks, in order: under the engine root first, then the
/// sibling `reference/` directory one level up.
pub fn resolve_source(root: &Path) -> PathBuf {
    let primary = join_rel(root, SRC_REL);
    if primary.is_file() {
        return primary;
    }
    let parent = root
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| root.to_path_buf());
    parent.join("reference").join("GLOSSARY.md")
}

/// `str(SRC.relative_to(ROOT)) if SRC.is_relative_to(ROOT) else str(SRC)`.
pub fn source_field(root: &Path, src: &Path) -> String {
    match src.strip_prefix(root) {
        Ok(rel) => rel.to_string_lossy().into_owned(),
        Err(_) => src.to_string_lossy().into_owned(),
    }
}

pub type Outcome = BuildOutcome;

/// Compile the glossary. Does not write. A RED compile carries no artifact.
pub fn evaluate(root: &Path) -> Result<Outcome, LearnError> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let src = resolve_source(&root);
    let out = join_rel(&root, OUT_REL);

    if !src.is_file() {
        return Ok(Outcome {
            stdout: format!("FAIL: missing glossary at {}\n", src.display()),
            code: 1,
            artifact: None,
        });
    }

    let bytes = std::fs::read(&src)
        .map_err(|e| LearnError::io(format!("read {}: {e}", src.display())))?;
    let text = String::from_utf8(bytes)
        .map_err(|e| LearnError::parse(format!("{} is not valid UTF-8: {e}", src.display())))?;

    let terms = extract_terms(&text);
    let body = render(&source_field(&root, &src), &terms.sorted());

    // The verdict is decided BEFORE anything is written.
    let failures = failures_for(terms.len());
    if !failures.is_empty() {
        let mut report = vec![format!("FAIL: glossary terms={}", terms.len())];
        report.extend(failures.iter().map(|f| format!("  - {f}")));
        report.push(format!(
            "  out={OUT_REL} NOT WRITTEN (a failing build leaves no artifact)"
        ));
        return Ok(Outcome {
            stdout: format!("{}\n", report.join("\n")),
            code: 1,
            artifact: None,
        });
    }

    Ok(Outcome {
        stdout: format!("PASS: glossary terms={} → {OUT_REL}\n", terms.len()),
        code: 0,
        artifact: Some((out, body)),
    })
}

/// Compile the glossary and write the artifact on the GREEN path only.
pub fn write_glossary(root: &Path) -> Result<Outcome, LearnError> {
    let outcome = evaluate(root)?;
    debug_assert!(
        outcome.code == 0 || outcome.artifact.is_none(),
        "a failing run must not carry an artifact"
    );
    if outcome.code == 0 {
        if let Some((path, body)) = &outcome.artifact {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| LearnError::io(format!("mkdir {}: {e}", parent.display())))?;
            }
            std::fs::write(path, body.as_bytes())
                .map_err(|e| LearnError::io(format!("write {}: {e}", path.display())))?;
        }
    }
    Ok(outcome)
}

/// Every reason this build is RED, as the report prints them. Factored out of
/// `run` so the floor can be asserted without running the process — `run` calls
/// `std::process::exit`, which a unit test cannot survive, and a floor that is
/// only reachable through a process boundary tends to end up untested.
pub fn failures_for(term_count: usize) -> Vec<String> {
    let mut failures: Vec<String> = Vec::new();
    if term_count < MIN_TERMS {
        failures.push(format!("need ≥{MIN_TERMS} terms, got {term_count}"));
    }
    failures
}

// ── tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(text: &str) -> Vec<(String, String)> {
        extract_terms(text).sorted()
    }

    #[test]
    fn a_plain_row_yields_term_and_definition() {
        let t = extract_terms("| **CRAH** | Computer Room Air Handler. |\n");
        assert_eq!(t.len(), 1);
        assert_eq!(
            rows("| **CRAH** | Computer Room Air Handler. |\n")[0],
            ("CRAH".to_string(), "Computer Room Air Handler.".to_string())
        );
    }

    #[test]
    fn the_header_and_separator_rows_are_skipped() {
        // The header cell is not bold, so it never matches at all; the
        // separator row matches with a definition that strips to empty.
        let text = "| Term | Definition |\n|---|---|\n| **A** | one |\n";
        assert_eq!(rows(text).len(), 1);
    }

    #[test]
    fn a_bold_term_literally_named_term_is_still_skipped() {
        assert!(rows("| **Term** | header-ish |\n").is_empty());
        assert!(rows("| **---** | header-ish |\n").is_empty());
    }

    #[test]
    fn a_definition_that_is_a_rule_is_skipped() {
        assert!(rows("| **A** | --- |\n").is_empty());
    }

    #[test]
    fn an_empty_definition_cell_is_skipped_via_the_backtrack() {
        // `\s*([^|]+)` gives a space back so the cell parses, then strips empty.
        assert!(rows("| **A** |  |\n").is_empty());
        // and with nothing at all between the pipes there is no match to make.
        assert!(rows("| **A** ||\n").is_empty());
    }

    #[test]
    fn a_parenthesised_term_also_registers_its_bare_form() {
        let out = rows("| **ASHRAE (TC 9.9)** | Thermal guidelines. |\n");
        let keys: Vec<&str> = out.iter().map(|(k, _)| k.as_str()).collect();
        assert!(keys.contains(&"ASHRAE (TC 9.9)"), "{keys:?}");
        assert!(keys.contains(&"ASHRAE"), "{keys:?}");
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn the_bare_alias_never_overwrites_an_existing_entry() {
        let t = extract_terms("| **UPS** | first |\n| **UPS (static)** | second |\n");
        let m: HashMap<String, String> = t.sorted().into_iter().collect();
        assert_eq!(m["UPS"], "first", "setdefault must not clobber");
        assert_eq!(m["UPS (static)"], "second");
    }

    #[test]
    fn a_redefined_term_keeps_its_original_position() {
        let t = extract_terms("| **B** | one |\n| **A** | two |\n| **B** | three |\n");
        // Insertion order is B, A; re-assignment of B does not move it.
        assert_eq!(t.order[0].0, "B");
        assert_eq!(t.order[0].1, "three");
        assert_eq!(t.order[1].0, "A");
    }

    #[test]
    fn sorting_is_casefolded_and_stable() {
        let t = extract_terms("| **beta** | b |\n| **Alpha** | a |\n| **ALPHA2** | c |\n");
        let keys: Vec<String> = t.sorted().into_iter().map(|(k, _)| k).collect();
        assert_eq!(keys, vec!["Alpha", "ALPHA2", "beta"]);
    }

    #[test]
    fn a_single_asterisk_inside_a_term_does_not_match() {
        assert!(rows("| **a*b** | x |\n").is_empty());
    }

    #[test]
    fn a_row_can_span_lines_because_the_space_class_matches_newlines() {
        let out = rows("|\n  **A**\n  | one |\n");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].0, "A");
    }

    #[test]
    fn json_escaping_matches_python_with_ensure_ascii_false() {
        let mut s = String::new();
        json_string("a\"b\\c\td\u{1}e — é", &mut s);
        assert_eq!(s, "\"a\\\"b\\\\c\\td\\u0001e — é\"");
    }

    #[test]
    fn an_empty_term_table_renders_an_empty_object_not_a_newline() {
        let body = render("web/content/reference/GLOSSARY.md", &[]);
        assert!(body.contains("\"terms\": {}\n"), "{body}");
        assert!(body.ends_with("}\n"), "{body}");
        assert!(body.contains("\"term_count\": 0,"), "{body}");
    }

    #[test]
    fn the_rendered_artifact_has_the_python_key_order_and_indent() {
        let terms = vec![("A".to_string(), "one".to_string())];
        let body = render("s.md", &terms);
        assert_eq!(
            body,
            "{\n  \"schema_version\": 1,\n  \"generated_by\": \"cdcp_learn\",\n  \"source\": \"s.md\",\n  \"term_count\": 1,\n  \"terms\": {\n    \"A\": \"one\"\n  }\n}\n"
        );
    }

    #[test]
    fn strip_parens_collapses_to_a_single_space_and_handles_no_close() {
        assert_eq!(py_strip(&strip_parens("A (b) C")), "A C");
        assert_eq!(py_strip(&strip_parens("A (b")), "A (b");
        assert_eq!(py_strip(&strip_parens("A (b) (c)")), "A");
    }

    #[test]
    fn casefold_folds_the_sharp_s() {
        assert_eq!(py_casefold("STRAßE"), "strasse");
    }

    #[test]
    fn the_source_field_falls_back_to_an_absolute_path() {
        let root = Path::new("/x/engine");
        assert_eq!(
            source_field(
                root,
                Path::new("/x/engine/web/content/reference/GLOSSARY.md")
            ),
            "web/content/reference/GLOSSARY.md"
        );
        assert_eq!(
            source_field(root, Path::new("/x/reference/GLOSSARY.md")),
            "/x/reference/GLOSSARY.md"
        );
    }

    #[test]
    fn an_empty_glossary_cannot_clear_the_term_floor() {
        // The anti-vacuous leg, asserted through the same expression `run`
        // uses: a build that parsed nothing must not be able to report a pass.
        let parsed_nothing = extract_terms("");
        assert!(parsed_nothing.is_empty());
        assert!(
            !failures_for(parsed_nothing.len()).is_empty(),
            "an empty glossary would report GREEN"
        );
    }

    #[test]
    fn the_floor_is_the_only_thing_that_can_turn_this_build_red() {
        // `failures_for` is the whole verdict, so the report `run` composes
        // cannot disagree with the exit code — there is no second source of
        // truth to fall out of step with the first.
        assert!(
            failures_for(MIN_TERMS).is_empty(),
            "the floor itself is GREEN"
        );
        assert!(failures_for(MIN_TERMS + 1).is_empty());
        for n in 0..MIN_TERMS {
            let f = failures_for(n);
            assert_eq!(f.len(), 1, "{n} terms: {f:?}");
            assert!(
                f[0].contains(&format!("{MIN_TERMS}")) && f[0].contains(&format!("{n}")),
                "the finding must name both the floor and what was measured: {f:?}"
            );
        }
    }

    #[test]
    fn a_red_verdict_is_decided_before_anything_is_written_or_printed() {
        // The shape bd-lt7 fixed in build_units.py and
        // bd-builder-verdict-shape-qm65 fixed here: the success token may not be
        // emitted on a path that returns non-zero. `run` calls
        // `std::process::exit`, so this asserts the predicate `run` branches on
        // rather than the process; `tests/diff_build_glossary_json.rs` asserts
        // the stdout bytes and the absent artifact across the process boundary.
        let below = extract_terms("| **A** | one |\n");
        assert!(
            !failures_for(below.len()).is_empty(),
            "a one-term glossary must be RED"
        );
        // The artifact this run would have written is computable, and is
        // exactly what must NOT reach the disk.
        let body = render("x.md", &below.sorted());
        assert!(body.contains("\"term_count\": 1,"), "{body}");
    }
}
