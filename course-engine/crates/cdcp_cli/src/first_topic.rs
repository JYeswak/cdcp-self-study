//! `cdcp first-topic-id` — first `id = "..."` in a topics.toml.
//!
//! EXTRACT-THEN-DELETE (`bd-extract-orphan-topic-python-9lj5`).
//! `scripts/selftest_orphan.sh` used to spawn an interpreter one-liner
//! to pull one live topic id as the specimen anchor. The extract lives
//! here so that L4 selftest has no live interpreter. Not a gate: this
//! does not decide whether the id is assessed, approved, or the right
//! topic. It prints the first match of the retired regex
//! `(?m)^\s*id\s*=\s*"([^"]+)"`.
//!
//! Fail-closed vs the retired one-liner: an unreadable file, a 0-byte
//! file, or a document with zero matches is RED. The one-liner printed
//! an empty line and left the shell to notice.

use std::fs;
use std::path::Path;

/// Error token for a 0-byte / whitespace-only document.
pub(crate) const EMPTY_DOCUMENT: &str = "empty document";
/// Error token when the regex finds no `id = "..."` rows.
pub(crate) const NO_TOPIC_ID: &str = "no topic id";

/// `cdcp first-topic-id --file <path>`.
pub(crate) fn emit(path: &Path) -> Result<(), String> {
    if path.as_os_str().is_empty() {
        return Err("first-topic-id: --file is empty".into());
    }
    let raw = fs::read_to_string(path)
        .map_err(|e| format!("first-topic-id: read {}: {e}", path.display()))?;
    let id = first_topic_id(&raw)?;
    println!("{id}");
    Ok(())
}

/// First match of the retired python `re.findall`, or a named RED.
pub(crate) fn first_topic_id(text: &str) -> Result<String, String> {
    if text.trim().is_empty() {
        return Err(format!(
            "{EMPTY_DOCUMENT} — a 0-byte topics file pins no anchor"
        ));
    }
    find_topic_ids(text).into_iter().next().ok_or_else(|| {
        format!("{NO_TOPIC_ID} — a topics file with zero id= rows certifies nothing")
    })
}

/// `re.findall(r'(?m)^\s*id\s*=\s*"([^"]+)"', text)`.
///
/// Written as an explicit scanner rather than a line loop because the
/// retired pattern's `\s*` and `[^"]+` both match newlines, so a match
/// may legally span lines. Matching is non-overlapping and left-to-right.
pub(crate) fn find_topic_ids(text: &str) -> Vec<String> {
    let ch: Vec<char> = text.chars().collect();
    let n = ch.len();
    let mut out = Vec::new();
    let mut p = 0usize;
    while p <= n {
        // `^` under re.MULTILINE.
        if p != 0 && ch[p - 1] != '\n' {
            p += 1;
            continue;
        }
        match match_id_at(&ch, p) {
            Some((end, id)) => {
                out.push(id);
                p = end;
            }
            None => p += 1,
        }
    }
    out
}

/// Python `re` `\s` on a `str` pattern: Unicode White_Space plus the
/// four ASCII information separators that `str.isspace()` counts.
fn py_space(c: char) -> bool {
    c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c)
}

/// One anchored attempt of the id pattern. No backtracking is needed:
/// every `\s*` is followed by a non-space literal, and `[^"]+` is
/// followed by `"`, so the greedy run is the only run that can succeed.
fn match_id_at(ch: &[char], start: usize) -> Option<(usize, String)> {
    let n = ch.len();
    let mut i = start;
    while i < n && py_space(ch[i]) {
        i += 1;
    }
    if i + 1 >= n || ch[i] != 'i' || ch[i + 1] != 'd' {
        return None;
    }
    i += 2;
    while i < n && py_space(ch[i]) {
        i += 1;
    }
    if i >= n || ch[i] != '=' {
        return None;
    }
    i += 1;
    while i < n && py_space(ch[i]) {
        i += 1;
    }
    if i >= n || ch[i] != '"' {
        return None;
    }
    i += 1;
    let s = i;
    while i < n && ch[i] != '"' {
        i += 1;
    }
    if i == s || i >= n {
        return None;
    }
    Some((i + 1, ch[s..i].iter().collect()))
}

#[cfg(test)]
mod unit {
    use super::*;

    fn production_src() -> &'static str {
        include_str!("first_topic.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source precedes tests")
    }

    #[test]
    fn first_of_several_is_the_declaration_order_head() {
        let text = "[[topic]]\nid = \"a\"\n\n[[topic]]\n  id=\"b\"\n";
        assert_eq!(first_topic_id(text).unwrap(), "a");
        assert_eq!(find_topic_ids(text), vec!["a", "b"]);
    }

    #[test]
    fn last_id_is_not_returned_when_two_exist() {
        let text = "id = \"first\"\nid = \"last\"\n";
        assert_ne!(first_topic_id(text).unwrap(), "last");
        assert_eq!(first_topic_id(text).unwrap(), "first");
    }

    #[test]
    fn id_pattern_ignores_non_line_start_and_nested_keys() {
        // `^` is anchored, so `topic_id = "x"` and a trailing `id` on a
        // line that already has content are not matches.
        let text = "topic_id = \"x\"\nlabel = \"y\" id = \"z\"\nid = \"real\"\n";
        assert_eq!(first_topic_id(text).unwrap(), "real");
        assert_eq!(find_topic_ids(text), vec!["real"]);
    }

    #[test]
    fn id_pattern_spans_lines_like_the_python_does() {
        assert_eq!(first_topic_id("id\n=\n\"spanning\"\n").unwrap(), "spanning");
    }

    #[test]
    fn empty_quoted_id_is_not_a_match() {
        let err = first_topic_id("id = \"\"\n").unwrap_err();
        assert!(err.contains(NO_TOPIC_ID), "{err}");
        assert!(find_topic_ids("id = \"\"\n").is_empty());
    }

    #[test]
    fn unterminated_quote_is_not_a_match() {
        let err = first_topic_id("id = \"oops\n").unwrap_err();
        assert!(err.contains(NO_TOPIC_ID), "{err}");
    }

    #[test]
    fn first_line_needs_no_preceding_newline() {
        assert_eq!(first_topic_id("id = \"first\"").unwrap(), "first");
    }

    #[test]
    fn comment_line_is_not_a_match() {
        let err = first_topic_id("# id = \"nope\"\nschema_version = 1\n").unwrap_err();
        assert!(err.contains(NO_TOPIC_ID), "{err}");
    }

    #[test]
    fn empty_document_is_red() {
        for raw in ["", "   \n\t  "] {
            let err = first_topic_id(raw).unwrap_err();
            assert!(err.contains(EMPTY_DOCUMENT), "{err}");
        }
    }

    #[test]
    fn schema_only_document_is_red() {
        let err = first_topic_id("schema_version = 1\nmin_items_default = 1\n").unwrap_err();
        assert!(err.contains(NO_TOPIC_ID), "{err}");
    }

    #[test]
    fn production_has_no_python_and_no_network() {
        let src = production_src();
        for needle in [
            "python3",
            "tomllib",
            "tomli",
            "TcpStream",
            "UdpSocket",
            "TcpListener",
            "std::net",
            "reqwest",
            "ureq",
            "cdcp_gate",
        ] {
            assert!(!src.contains(needle), "production mentions {needle}");
        }
        assert!(
            src.contains("match_id_at"),
            "delete the id scanner → selftest non-zero"
        );
        assert!(
            src.contains("re.findall"),
            "delete the retired-regex contract → selftest non-zero"
        );
    }
}
