//! build-glossary-json — Rust port of `scripts/build_glossary_json.py`
//! (bd-substrate-rust-migration-jhd.11).
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
//! # DEFECTS IN THE ORACLE THAT ARE REPRODUCED, NOT FIXED (findings, bd-.11)
//!
//!   * VERDICT SHAPE. On the below-floor path the oracle prints
//!     `PASS: glossary terms=N → …` FIRST and only then `FAIL: need ≥…`, on its
//!     way to returning 1. A reader skimming stdout sees PASS; CI sees non-zero.
//!     This is the same defect bd-lt7 fixed in `build_units.py`, still live
//!     here. It is REPRODUCED byte for byte: a port that fixes a bug is an
//!     unreviewed behaviour change that blinds the differential.
//!   * WRITE-BEFORE-VERDICT. The artifact is written BEFORE the term floor is
//!     evaluated, so a below-floor run still leaves a short `glossary.json` in
//!     `web/data/`. Also reproduced.
//!
//! Neither is repaired here. Both are reported on the bead.
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

use crate::registry::{GateCtx, GateError};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

pub const NAME: &str = "build-glossary-json";
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
    s.push_str("  \"generated_by\": \"scripts/build_glossary_json.py\",\n");
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

fn join_rel(root: &Path, rel: &str) -> PathBuf {
    let mut p = root.to_path_buf();
    for part in rel.split('/') {
        p.push(part);
    }
    p
}

/// `str(SRC.relative_to(ROOT)) if SRC.is_relative_to(ROOT) else str(SRC)`.
pub fn source_field(root: &Path, src: &Path) -> String {
    match src.strip_prefix(root) {
        Ok(rel) => rel.to_string_lossy().into_owned(),
        Err(_) => src.to_string_lossy().into_owned(),
    }
}

pub fn run(ctx: &GateCtx) -> Result<(), GateError> {
    // The oracle ignores argv; this side refuses to. See the header.
    ctx.reject_unknown_flags(&[])?;

    // The Python resolves its own location (`Path(__file__).resolve()`), so
    // every path it prints is symlink-free. Do the same to the engine root.
    let root = ctx.root.canonicalize().unwrap_or_else(|_| ctx.root.clone());
    let src = resolve_source(&root);
    let out = join_rel(&root, OUT_REL);

    if !src.is_file() {
        println!("FAIL: missing glossary at {}", src.display());
        let _ = std::io::stdout().flush();
        std::process::exit(1);
    }

    let bytes = std::fs::read(&src)
        .map_err(|e| GateError::error(format!("read {}: {e}", src.display())))?;
    let text = String::from_utf8(bytes)
        .map_err(|e| GateError::error(format!("{} is not valid UTF-8: {e}", src.display())))?;

    let terms = extract_terms(&text);
    let body = render(&source_field(&root, &src), &terms.sorted());

    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| GateError::error(format!("mkdir {}: {e}", parent.display())))?;
    }
    std::fs::write(&out, body.as_bytes())
        .map_err(|e| GateError::error(format!("write {}: {e}", out.display())))?;

    // The oracle prints the PASS line unconditionally, BEFORE the floor is
    // evaluated. See the header: reproduced, not repaired.
    println!("PASS: glossary terms={} → {OUT_REL}", terms.len());
    let _ = std::io::stdout().flush();
    if terms.len() < MIN_TERMS {
        println!("FAIL: need ≥{MIN_TERMS} terms");
        let _ = std::io::stdout().flush();
        std::process::exit(1);
    }
    Ok(())
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
            "{\n  \"schema_version\": 1,\n  \"generated_by\": \"scripts/build_glossary_json.py\",\n  \"source\": \"s.md\",\n  \"term_count\": 1,\n  \"terms\": {\n    \"A\": \"one\"\n  }\n}\n"
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
            parsed_nothing.len() < MIN_TERMS,
            "an empty glossary would report GREEN"
        );
        // and the artifact it would still write is the empty one.
        let body = render("x.md", &parsed_nothing.sorted());
        assert!(body.contains("\"term_count\": 0,"), "{body}");
    }
}
