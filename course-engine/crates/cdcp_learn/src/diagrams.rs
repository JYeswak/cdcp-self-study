//! Diagram smoke — bind every PRESENT registry row to its artifact.
//!
//! Extracted from `scripts/smoke_diagrams.py` by
//! `bd-substrate-rust-migration-jhd.14` as product, not a gate file. A
//! learner opens `web/diagrams/<id>.html`. If they can see it, it is not a
//! `cdcp_gate` concern.
//!
//! # Contract
//!
//! * the inventory table of [`REGISTRY_REL`] is parsed structurally
//! * every `present` row is bound to exactly `web/diagrams/<id>.html`
//! * that file is HTML (at least one element)
//! * it carries a `class` token `honesty-banner` whose OWN text disclaims
//!   certification (`\bnot\b` AND `certif`)
//! * it carries `data-diagram="<id>"`
//! * present count is pinned ([`EXPECTED_PRESENT`])
//! * zero present rows is an ERROR
//! * an empty honesty-banner is a FAIL
//! * an unclosed honesty-banner is a FAIL — the retired script swallowed the
//!   rest of the page and scored GREEN
//!   (`bd-smoke-diagrams-unclosed-banner-swallows-page-61v0`). Closed here.
//!
//! This is NOT a CPython `html.parser` replica and not a stdout match of the
//! retired script. `.parked-wave8/` stays parked.
//!
//! # What this cannot decide
//!
//! Registry completeness (a shipped file with no row is unchecked). Pedagogy.
//! That the diagram works (no JS runs). The pin's correctness.

#![forbid(unsafe_code)]

use crate::{join_rel, BuildOutcome};
use std::collections::BTreeSet;
use std::path::Path;

pub const NAME: &str = "smoke-diagrams";
pub const SUMMARY: &str =
    "M8-C: bind every PRESENT diagram-registry row to its artifact and its two landmarks";

pub const REGISTRY_REL: &str = "docs/DIAGRAM-REGISTRY.md";
pub const DIAGRAM_DIR: &str = "web/diagrams";
pub const EXPECTED_PRESENT: usize = 7;
pub const INVENTORY_HEADING: &str = "## Inventory";
pub const EXPECTED_COLUMNS: &[&str] = &["ID", "Title", "Modules", "Priority", "Status", "Path"];
pub const STATUS_PRESENT: &str = "present";
pub const STATUS_PLANNED: &str = "planned";

const VOID_TAGS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

/// Run the diagram smoke against `root` (the course-engine directory).
///
/// Reader: writes nothing. `code` is 0 PASS, 1 FAIL (artifact), 2 ERROR
/// (registry untrustworthy). `artifact` is always `None`.
pub fn run(root: &Path) -> BuildOutcome {
    let mut out = String::from("==> smoke_diagrams\n");
    match parse_registry(root) {
        RegistryRead::Unreadable { message, code } => {
            out.push_str(&message);
            if !message.ends_with('\n') {
                out.push('\n');
            }
            return outcome(code, out);
        }
        RegistryRead::Parsed { rows, errors } => {
            for e in &errors {
                out.push_str("  ERROR: ");
                out.push_str(e);
                out.push('\n');
            }
            let present: Vec<&str> = rows
                .iter()
                .filter(|r| r.status == STATUS_PRESENT)
                .map(|r| r.id.as_str())
                .collect();
            let mut errors = errors;
            if present.is_empty() {
                out.push_str(
                    "smoke_diagrams: ERROR: zero present diagrams parsed from the registry\n",
                );
                return outcome(2, out);
            }
            if present.len() != EXPECTED_PRESENT {
                out.push_str(&format!(
                    "  ERROR: present count {} != pinned {EXPECTED_PRESENT} ({})\n",
                    present.len(),
                    present.join(", ")
                ));
                errors.push("count".into());
            }
            if !errors.is_empty() {
                out.push_str(&format!(
                    "smoke_diagrams: ERROR ({}) \u{2014} registry not trustworthy\n",
                    errors.len()
                ));
                return outcome(2, out);
            }

            let mut fails: Vec<String> = Vec::new();
            for id in &present {
                let bad = check_diagram(root, id);
                if bad.is_empty() {
                    out.push_str("  ok: ");
                    out.push_str(id);
                    out.push('\n');
                } else {
                    for msg in bad {
                        out.push_str("  FAIL: ");
                        out.push_str(&msg);
                        out.push('\n');
                        fails.push(msg);
                    }
                }
            }
            if !fails.is_empty() {
                out.push_str(&format!("smoke_diagrams: FAIL ({})\n", fails.len()));
                return outcome(1, out);
            }
            out.push_str(&format!(
                "smoke_diagrams: PASS ({} present diagrams from the registry)\n",
                present.len()
            ));
            outcome(0, out)
        }
    }
}

enum RegistryRead {
    Unreadable {
        message: String,
        code: i32,
    },
    Parsed {
        rows: Vec<RegRow>,
        errors: Vec<String>,
    },
}

struct RegRow {
    id: String,
    status: String,
}

fn parse_registry(root: &Path) -> RegistryRead {
    let path = join_rel(root, REGISTRY_REL);
    if !path.is_file() {
        return RegistryRead::Unreadable {
            message: format!("smoke_diagrams: ERROR: missing registry {REGISTRY_REL}\n"),
            code: 2,
        };
    }
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t.replace('\r', ""),
        Err(e) => {
            return RegistryRead::Unreadable {
                message: format!(
                    "smoke_diagrams: ERROR: unreadable registry {REGISTRY_REL}: {e}\n"
                ),
                code: 2,
            };
        }
    };
    let lines: Vec<&str> = text.split('\n').collect();

    let start = lines.iter().position(|ln| ln.trim() == INVENTORY_HEADING);
    let Some(start) = start else {
        return RegistryRead::Unreadable {
            message: format!(
                "smoke_diagrams: ERROR: no '{INVENTORY_HEADING}' heading in {REGISTRY_REL}\n"
            ),
            code: 2,
        };
    };

    let head = (start + 1..lines.len()).find(|&i| split_row(lines[i]).is_some());
    let Some(head) = head else {
        return RegistryRead::Unreadable {
            message: format!("smoke_diagrams: ERROR: no table under '{INVENTORY_HEADING}'\n"),
            code: 2,
        };
    };

    let columns = split_row(lines[head]).unwrap_or_default();
    if columns != EXPECTED_COLUMNS {
        return RegistryRead::Unreadable {
            message: format!(
                "smoke_diagrams: ERROR: inventory columns changed: expected {}, found {}\n",
                join_cols(EXPECTED_COLUMNS),
                join_cols(&columns.iter().map(String::as_str).collect::<Vec<_>>())
            ),
            code: 2,
        };
    }

    let body = head + 1;
    let sep = if body < lines.len() {
        split_row(lines[body])
    } else {
        None
    };
    if sep
        .as_ref()
        .is_none_or(|s| !s.iter().all(|c| is_sep_cell(c)))
    {
        return RegistryRead::Unreadable {
            message: "smoke_diagrams: ERROR: inventory header not followed by a separator row\n"
                .into(),
            code: 2,
        };
    }

    let mut rows = Vec::new();
    let mut errors = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let fname = "DIAGRAM-REGISTRY.md";
    let mut lineno = body + 1;
    while lineno < lines.len() {
        let Some(cells) = split_row(lines[lineno]) else {
            break;
        };
        let where_ = format!("{fname}:{}", lineno + 1);
        if cells.len() != EXPECTED_COLUMNS.len() {
            errors.push(format!(
                "{where_}: expected {} cells, got {}",
                EXPECTED_COLUMNS.len(),
                cells.len()
            ));
            lineno += 1;
            continue;
        }
        let raw_id = &cells[0];
        let raw_status = &cells[4];
        let raw_path = &cells[5];

        let Some(did) = parse_id_cell(raw_id) else {
            errors.push(format!(
                "{where_}: malformed ID cell {raw_id:?} (want `lower-kebab-id`)"
            ));
            lineno += 1;
            continue;
        };
        if !seen.insert(did.to_string()) {
            errors.push(format!("{where_}: duplicate ID `{did}`"));
            lineno += 1;
            continue;
        }

        let status = normalize_status(raw_status);
        if status != STATUS_PRESENT && status != STATUS_PLANNED {
            errors.push(format!(
                "{where_}: `{did}` unrecognised status {raw_status:?} (known: {STATUS_PRESENT}, {STATUS_PLANNED})"
            ));
            lineno += 1;
            continue;
        }

        if status == STATUS_PLANNED {
            let stripped = raw_path.replace('`', "");
            let stripped = stripped.trim();
            if !is_no_path(stripped) {
                errors.push(format!(
                    "{where_}: `{did}` is planned but names path {raw_path:?}"
                ));
            } else {
                rows.push(RegRow {
                    id: did.to_string(),
                    status,
                });
            }
            lineno += 1;
            continue;
        }

        let Some(got) = parse_path_cell(raw_path) else {
            errors.push(format!(
                "{where_}: `{did}` malformed path cell {raw_path:?} (want `path`)"
            ));
            lineno += 1;
            continue;
        };
        let want = format!("{DIAGRAM_DIR}/{did}.html");
        if got != want {
            errors.push(format!("{where_}: `{did}` path {got:?} is not {want:?}"));
            lineno += 1;
            continue;
        }
        rows.push(RegRow {
            id: did.to_string(),
            status,
        });
        lineno += 1;
    }

    RegistryRead::Parsed { rows, errors }
}

fn check_diagram(root: &Path, did: &str) -> Vec<String> {
    let rel = format!("{DIAGRAM_DIR}/{did}.html");
    let path = join_rel(root, &rel);
    if !path.is_file() {
        return vec![format!("{did}: missing {rel}")];
    }
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => return vec![format!("{did}: unreadable {rel}: {e}")],
    };
    let parsed = parse_html(&text);
    if parsed.tags == 0 {
        return vec![format!("{did}: not HTML (zero elements parsed)")];
    }
    let mut bad = Vec::new();
    if !parsed.saw_banner {
        bad.push(format!("{did}: no element with class=\"honesty-banner\""));
    } else {
        if parsed.unclosed_banner {
            bad.push(format!("{did}: honesty-banner is never closed"));
        }
        let banner = normalize_ws(&parsed.banner_text).to_lowercase();
        if banner.is_empty() {
            bad.push(format!("{did}: honesty-banner element is empty"));
        } else if !(has_word_not(&banner) && banner.contains("certif")) {
            bad.push(format!(
                "{did}: honesty-banner does not disclaim certification"
            ));
        }
    }
    if !parsed.markers.iter().any(|m| m == did) {
        bad.push(format!("{did}: no element with data-diagram=\"{did}\""));
    }
    bad
}

/// Structural landmarks collected from one diagram page.
#[derive(Debug, Default)]
pub struct DiagramParse {
    pub tags: usize,
    pub markers: BTreeSet<String>,
    pub saw_banner: bool,
    pub banner_text: String,
    pub unclosed_banner: bool,
}

/// Product HTML scan. Not `html.parser`. An unclosed banner does NOT swallow
/// the rest of the page — that hole is closed.
pub fn parse_html(input: &str) -> DiagramParse {
    let mut p = DiagramParse::default();
    let bytes = input.as_bytes();
    let mut i = 0;
    let mut banner_depth: usize = 0;
    let mut text = String::new();

    while i < bytes.len() {
        if bytes[i] == b'<' {
            if starts_with_ci(&bytes[i..], b"<!--") {
                i += 4;
                if let Some(end) = find_bytes(&bytes[i..], b"-->") {
                    i += end + 3;
                } else {
                    i = bytes.len();
                }
                continue;
            }
            if starts_with_ci(&bytes[i..], b"<![cdata[") {
                i += 9;
                if let Some(end) = find_bytes(&bytes[i..], b"]]>") {
                    i += end + 3;
                } else {
                    i = bytes.len();
                }
                continue;
            }
            if starts_with_ci(&bytes[i..], b"<!") {
                // doctype / declaration
                i += 2;
                while i < bytes.len() && bytes[i] != b'>' {
                    i += 1;
                }
                if i < bytes.len() {
                    i += 1;
                }
                continue;
            }
            if starts_with_ci(&bytes[i..], b"</") {
                let (name, next) = parse_end_tag(&bytes[i..]);
                i += next;
                if name.is_empty() {
                    continue;
                }
                p.tags += 1;
                if is_void(&name) {
                    continue;
                }
                if banner_depth > 0 {
                    banner_depth -= 1;
                }
                continue;
            }
            if i + 1 < bytes.len() && is_name_start(bytes[i + 1]) {
                let (tag, attrs, self_close, next) = parse_start_tag(&bytes[i..]);
                i += next;
                if tag.is_empty() {
                    continue;
                }
                p.tags += 1;
                let void = self_close || is_void(&tag);
                apply_attrs(&mut p, &attrs, &mut banner_depth, void);
                if tag == "script" || tag == "style" {
                    let closer = format!("</{tag}");
                    if let Some(end) = find_end_tag_ci(&bytes[i..], closer.as_bytes()) {
                        i += end;
                    } else {
                        i = bytes.len();
                    }
                }
                continue;
            }
            // lone '<' is data
            if banner_depth > 0 {
                text.push('<');
            }
            i += 1;
            continue;
        }
        // text run
        let start = i;
        while i < bytes.len() && bytes[i] != b'<' {
            i += 1;
        }
        if banner_depth > 0 {
            if let Ok(s) = std::str::from_utf8(&bytes[start..i]) {
                text.push_str(s);
            }
        }
    }

    p.unclosed_banner = banner_depth > 0;
    p.banner_text = decode_entities(&text);
    p
}

fn apply_attrs(
    p: &mut DiagramParse,
    attrs: &[(String, String)],
    banner_depth: &mut usize,
    void: bool,
) {
    let mut class = None;
    let mut marker = None;
    for (k, v) in attrs {
        if k == "class" {
            class = Some(v.as_str());
        }
        if k == "data-diagram" {
            marker = Some(v.trim().to_string());
        }
    }
    if let Some(m) = marker {
        p.markers.insert(m);
    }
    let is_banner = class.is_some_and(|c| class_has_token(c, "honesty-banner"));
    if *banner_depth > 0 {
        if !void {
            *banner_depth += 1;
        }
    } else if is_banner {
        p.saw_banner = true;
        if !void {
            *banner_depth = 1;
        }
    }
}

fn parse_start_tag(src: &[u8]) -> (String, Vec<(String, String)>, bool, usize) {
    // src starts with '<'
    if src.len() < 2 {
        return (String::new(), Vec::new(), false, src.len());
    }
    let mut i = 1;
    let name_start = i;
    while i < src.len() && is_name_char(src[i]) {
        i += 1;
    }
    let name = ascii_lower(&src[name_start..i]);
    let mut attrs = Vec::new();
    let mut self_close = false;
    loop {
        while i < src.len() && src[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= src.len() {
            // unterminated tag: no element
            return (String::new(), Vec::new(), false, src.len());
        }
        if src[i] == b'>' {
            i += 1;
            break;
        }
        if src[i] == b'/' && i + 1 < src.len() && src[i + 1] == b'>' {
            self_close = true;
            i += 2;
            break;
        }
        if !is_name_start(src[i]) {
            // junk until '>' or end
            while i < src.len() && src[i] != b'>' {
                i += 1;
            }
            if i < src.len() {
                i += 1;
            }
            break;
        }
        let an_start = i;
        while i < src.len() && (is_name_char(src[i]) || src[i] == b':') {
            i += 1;
        }
        let aname = ascii_lower(&src[an_start..i]);
        while i < src.len() && src[i].is_ascii_whitespace() {
            i += 1;
        }
        let aval = if i < src.len() && src[i] == b'=' {
            i += 1;
            while i < src.len() && src[i].is_ascii_whitespace() {
                i += 1;
            }
            if i >= src.len() {
                return (String::new(), Vec::new(), false, src.len());
            }
            if src[i] == b'"' || src[i] == b'\'' {
                let q = src[i];
                i += 1;
                let vstart = i;
                while i < src.len() && src[i] != q {
                    i += 1;
                }
                let raw = std::str::from_utf8(&src[vstart..i]).unwrap_or("");
                if i < src.len() {
                    i += 1;
                }
                decode_entities(raw)
            } else {
                let vstart = i;
                while i < src.len()
                    && !src[i].is_ascii_whitespace()
                    && src[i] != b'>'
                    && src[i] != b'/'
                {
                    i += 1;
                }
                let raw = std::str::from_utf8(&src[vstart..i]).unwrap_or("");
                decode_entities(raw)
            }
        } else {
            String::new()
        };
        // last duplicate wins
        if let Some(existing) = attrs.iter_mut().find(|(k, _)| *k == aname) {
            existing.1 = aval;
        } else {
            attrs.push((aname, aval));
        }
    }
    (name, attrs, self_close, i)
}

fn parse_end_tag(src: &[u8]) -> (String, usize) {
    // src starts with "</"
    let mut i = 2;
    while i < src.len() && src[i].is_ascii_whitespace() {
        i += 1;
    }
    let name_start = i;
    while i < src.len() && is_name_char(src[i]) {
        i += 1;
    }
    let name = ascii_lower(&src[name_start..i]);
    while i < src.len() && src[i] != b'>' {
        i += 1;
    }
    if i < src.len() {
        i += 1;
    }
    (name, i)
}

fn find_end_tag_ci(src: &[u8], closer: &[u8]) -> Option<usize> {
    let mut i = 0;
    while i + closer.len() <= src.len() {
        if starts_with_ci(&src[i..], closer) {
            let mut j = i + closer.len();
            while j < src.len() && src[j] != b'>' {
                j += 1;
            }
            if j < src.len() {
                return Some(j + 1);
            }
            return Some(src.len());
        }
        i += 1;
    }
    None
}

fn is_void(name: &str) -> bool {
    VOID_TAGS.contains(&name)
}

fn is_name_start(b: u8) -> bool {
    b.is_ascii_alphabetic()
}

fn is_name_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-'
}

fn ascii_lower(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| b.to_ascii_lowercase() as char)
        .collect()
}

fn starts_with_ci(hay: &[u8], needle: &[u8]) -> bool {
    if hay.len() < needle.len() {
        return false;
    }
    hay[..needle.len()]
        .iter()
        .zip(needle.iter())
        .all(|(a, b)| a.to_ascii_lowercase() == b.to_ascii_lowercase())
}

fn find_bytes(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

pub fn class_has_token(class: &str, token: &str) -> bool {
    class.split_whitespace().any(|t| t == token)
}

/// `\bnot\b` on Unicode word characters (alphanumeric + `_`).
pub fn has_word_not(s: &str) -> bool {
    let lower = s.to_lowercase();
    let chars: Vec<char> = lower.chars().collect();
    let is_word = |c: char| c.is_alphanumeric() || c == '_';
    let needle = ['n', 'o', 't'];
    for i in 0..chars.len() {
        if chars[i..].starts_with(&needle) {
            let before_ok = i == 0 || !is_word(chars[i - 1]);
            let after = i + 3;
            let after_ok = after >= chars.len() || !is_word(chars[after]);
            if before_ok && after_ok {
                return true;
            }
        }
    }
    false
}

pub fn decode_entities(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '&' {
            if let Some((ch, consumed)) = take_entity(&chars[i..]) {
                out.push(ch);
                i += consumed;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

fn take_entity(chars: &[char]) -> Option<(char, usize)> {
    if chars.len() < 3 || chars[0] != '&' {
        return None;
    }
    if chars[1] == '#' {
        let hex = chars.get(2) == Some(&'x') || chars.get(2) == Some(&'X');
        let start = if hex { 3 } else { 2 };
        let mut j = start;
        while j < chars.len() && chars[j] != ';' {
            let ok = if hex {
                chars[j].is_ascii_hexdigit()
            } else {
                chars[j].is_ascii_digit()
            };
            if !ok {
                return None;
            }
            j += 1;
        }
        if j >= chars.len() || j == start {
            return None;
        }
        let digits: String = chars[start..j].iter().collect();
        let n = if hex {
            u32::from_str_radix(&digits, 16).ok()?
        } else {
            digits.parse::<u32>().ok()?
        };
        let ch = char::from_u32(n)?;
        return Some((ch, j + 1));
    }
    let mut j = 1;
    while j < chars.len() && chars[j].is_ascii_alphanumeric() {
        j += 1;
    }
    if j >= chars.len() || chars[j] != ';' || j == 1 {
        return None;
    }
    let name: String = chars[1..j].iter().collect();
    let ch = match name.as_str() {
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "quot" => '"',
        "apos" => '\'',
        "nbsp" => '\u{00a0}',
        "not" => '\u{00ac}',
        _ => return None,
    };
    Some((ch, j + 1))
}

fn normalize_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn split_row(line: &str) -> Option<Vec<String>> {
    let s = line.trim();
    if !s.starts_with('|') || !s.ends_with('|') || s.len() < 2 {
        return None;
    }
    Some(
        s[1..s.len() - 1]
            .split('|')
            .map(|c| c.trim().to_string())
            .collect(),
    )
}

fn parse_id_cell(raw: &str) -> Option<&str> {
    let s = raw.trim();
    if s.len() < 3 || !s.starts_with('`') || !s.ends_with('`') {
        return None;
    }
    let inner = &s[1..s.len() - 1];
    if inner.is_empty() {
        return None;
    }
    if inner
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        Some(inner)
    } else {
        None
    }
}

fn parse_path_cell(raw: &str) -> Option<String> {
    let s = raw.trim();
    if s.len() < 3 || !s.starts_with('`') || !s.ends_with('`') {
        return None;
    }
    let inner = s[1..s.len() - 1].trim();
    if inner.is_empty() {
        None
    } else {
        Some(inner.to_string())
    }
}

fn normalize_status(raw: &str) -> String {
    raw.replace(['*', '`'], "").trim().to_lowercase()
}

fn is_no_path(s: &str) -> bool {
    matches!(s, "\u{2014}" | "\u{2013}" | "-" | "")
}

fn is_sep_cell(c: &str) -> bool {
    let t = c.trim();
    let t = t.strip_prefix(':').unwrap_or(t);
    let t = t.strip_suffix(':').unwrap_or(t);
    t.len() >= 3 && t.chars().all(|ch| ch == '-')
}

fn join_cols(cols: &[&str]) -> String {
    format!("[{}]", cols.join(", "))
}

fn outcome(code: i32, stdout: impl Into<String>) -> BuildOutcome {
    BuildOutcome {
        stdout: stdout.into(),
        code,
        artifact: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_not_is_a_word_boundary() {
        assert!(has_word_not("does not grant"));
        assert!(has_word_not("is-not-certification"));
        assert!(!has_word_not("notice: certification"));
        assert!(!has_word_not("cannot certification"));
        assert!(!has_word_not(""));
    }

    #[test]
    fn class_token_is_exact() {
        assert!(class_has_token("honesty-banner", "honesty-banner"));
        assert!(class_has_token("foo honesty-banner bar", "honesty-banner"));
        assert!(!class_has_token("HONESTY-BANNER", "honesty-banner"));
        assert!(!class_has_token("my-honesty-banner", "honesty-banner"));
    }

    #[test]
    fn entity_not_is_not_the_word() {
        let s = decode_entities("&not; certification");
        assert!(!has_word_not(&s), "{s:?}");
        assert!(s.contains('\u{00ac}'));
    }

    #[test]
    fn nbsp_decodes_to_whitespace() {
        let s = decode_entities("does&nbsp;not&nbsp;grant");
        assert_eq!(normalize_ws(&s), "does not grant");
        assert!(has_word_not(&normalize_ws(&s)));
    }

    #[test]
    fn unclosed_banner_is_flagged_even_if_a_footer_would_disclaim() {
        // An unclosed <div> really does contain the rest of the file. The
        // retired script treated that as GREEN. The product verdict is RED
        // because the element is never closed — not because we pretend the
        // footer sat outside the div.
        let p = parse_html(
            "<div class=\"honesty-banner\">Welcome<i data-diagram=\"a1\"></i>\
             <footer>This tool does not grant EPI/EXIN certification.</footer>",
        );
        assert!(p.saw_banner);
        assert!(p.unclosed_banner, "depth>0 at EOF must be named");
        assert!(p.markers.contains("a1"));
    }

    #[test]
    fn closed_hollow_banner_ignores_footer() {
        let p = parse_html(
            "<div class=\"honesty-banner\">Welcome</div><i data-diagram=\"a1\"></i>\
             <footer>This tool does not grant EPI/EXIN certification.</footer>",
        );
        assert!(p.saw_banner);
        assert!(!p.unclosed_banner);
        assert_eq!(normalize_ws(&p.banner_text), "Welcome");
    }

    #[test]
    fn comment_is_not_a_banner() {
        let p = parse_html(
            "<!--<div class=\"honesty-banner\">does not grant certification</div>-->\
             <i data-diagram=\"a1\"></i>",
        );
        assert!(!p.saw_banner);
        assert!(p.markers.contains("a1"));
    }

    #[test]
    fn nested_markup_collects_banner_text() {
        let p = parse_html(
            "<div class=\"honesty-banner\"><p>does <strong>not</strong> grant</p> \
             <em>certification</em></div><i data-diagram=\"x\"></i>",
        );
        assert!(p.saw_banner);
        assert!(!p.unclosed_banner);
        let banner = normalize_ws(&p.banner_text).to_lowercase();
        assert!(
            has_word_not(&banner) && banner.contains("certif"),
            "{banner:?}"
        );
    }

    #[test]
    fn plaintext_file_is_zero_tags() {
        let p = parse_html("not certif a1 data-diagram honesty-banner");
        assert_eq!(p.tags, 0);
        assert!(!p.saw_banner);
    }

    #[test]
    fn pin_is_seven() {
        assert_eq!(EXPECTED_PRESENT, 7);
        assert!(EXPECTED_PRESENT > 0);
    }
}
