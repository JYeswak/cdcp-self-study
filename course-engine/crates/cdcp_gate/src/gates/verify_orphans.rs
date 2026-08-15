//! verify-orphans — Rust port of `scripts/verify_orphans.py`
//! (bd-substrate-rust-migration-jhd.2).
//!
//! # CLAIM: FLOOR-RAISE
//!
//! Syllabus and bank must point at each other, measured on the **approved**
//! pool (`status == "approved"`), never the file set. C1 draws only approved
//! items, so a topic whose only refs are retired is unreachable
//! (bd-orphans-counts-retired-items-farl). RED: orphan topic (0 approved refs
//! of N referencing), orphan ref, unanchored item. Anti-vacuous: zero topics,
//! zero items, zero approved, missing bank/registry. Empty-bank stays a
//! scan-set property. File granularity (bd-2kr): empty `items[]` is named RED.
//!
//! # WHAT THIS GATE CANNOT DO
//!
//! It cannot tell whether a topic is assessed well, or whether a resolving
//! `topic_id` is the right topic. A syllabus that omits a topic is invisible.
//!
//! # BYTE-EXACTNESS WITH THE PYTHON ORACLE
//!
//! Oracle: `scripts/verify_orphans.py`. Differential asserts stdout/stderr/exit
//! byte for byte. RED reports go to stdout with exit 1, not `GateError`.

use crate::registry::{GateCtx, GateError};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::Path;
use toml::Value;

pub const NAME: &str = "verify-orphans";
pub const SUMMARY: &str =
    "topic<->bank referential integrity: no orphan topics, orphan refs, or unanchored items";

const APPROVED: &str = "approved";
fn is_approved(it: &toml::Table) -> bool {
    matches!(it.get("status"), Some(Value::String(s)) if s == APPROVED)
}
fn status_is_known(it: &toml::Table) -> bool {
    match it.get("status") {
        None => true,
        Some(Value::String(s)) => matches!(s.as_str(), "approved" | "draft" | "retired"),
        Some(_) => false,
    }
}

/// Engine-root-relative defaults, matching the Python module constants.
pub const DEFAULT_BANK: &str = "bank/items";
pub const DEFAULT_TOPICS: &str = "knowledge/topics.toml";

/// How many failures the report prints before it truncates. Mirrors `MAX_REPORT`.
pub const MAX_REPORT: usize = 40;

/// The long options this gate accepts, for the abbreviation resolver.
const OPTIONS: &[&str] = &["--bank", "--topics"];

// ── Python-behaviour emulations ────────────────────────────────────────────
// Each of these exists because the port's acceptance bar is byte-identical
// output, not merely an identical verdict.

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

/// `repr()` of a Python `str`.
///
/// Single quotes unless the value contains `'` and no `"`. Backslash, the
/// active quote, and the three named escapes are escaped; C0 controls and DEL
/// become `\xNN`. Non-ASCII characters are passed through, which matches
/// CPython for printable code points and differs for unprintable ones (it would
/// emit `\xNN`/`\uNNNN`); reaching that needs an unprintable non-ASCII
/// character inside a topic id.
pub fn py_repr(s: &str) -> String {
    let quote = if s.contains('\'') && !s.contains('"') {
        '"'
    } else {
        '\''
    };
    let mut out = String::with_capacity(s.len() + 2);
    out.push(quote);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c if (c as u32) < 0x20 || (c as u32) == 0x7f => {
                out.push_str(&format!("\\x{:02x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push(quote);
    out
}

/// Is this the POSIX-absolute path `PurePosixPath.is_absolute()` reports?
pub fn is_absolute_posix(s: &str) -> bool {
    s.starts_with('/')
}

/// `str(PurePosixPath(s))`: empty and `.` components are dropped, `..` is kept,
/// duplicate separators collapse, a trailing separator is dropped, and exactly
/// two leading separators are preserved as a root (three or more collapse).
pub fn norm_posix(s: &str) -> String {
    let root = if s.starts_with("//") && !s.starts_with("///") {
        "//"
    } else if s.starts_with('/') {
        "/"
    } else {
        ""
    };
    let body = s
        .split('/')
        .filter(|c| !c.is_empty() && *c != ".")
        .collect::<Vec<_>>()
        .join("/");
    let joined = format!("{root}{body}");
    if joined.is_empty() {
        ".".to_string()
    } else {
        joined
    }
}

/// `root / rel` with `PurePosixPath` semantics: an absolute `rel` replaces the
/// root outright.
pub fn join_posix(root: &str, rel: &str) -> String {
    if is_absolute_posix(rel) {
        return norm_posix(rel);
    }
    let combined = if root.ends_with('/') {
        format!("{root}{rel}")
    } else {
        format!("{root}/{rel}")
    };
    norm_posix(&combined)
}

/// `re.findall(r'(?m)^\s*id\s*=\s*"([^"]+)"', text)`.
///
/// Written as an explicit scanner rather than a line loop because the Python
/// pattern's `\s*` and `[^"]+` both match newlines, so a match may legally span
/// lines; a per-line scan would silently find fewer ids and a gate that under-
/// counts its registry is the failure mode this whole file exists to prevent.
/// Matching is non-overlapping and left-to-right, as `findall` is.
pub fn find_topic_ids(text: &str) -> Vec<String> {
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

/// One anchored attempt of the id pattern. No backtracking is needed: every
/// `\s*` is followed by a non-space literal, and `[^"]+` is followed by `"`, so
/// the greedy run is the only run that can succeed.
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

/// Python truthiness of a value tomllib would have produced.
pub fn py_truthy(v: &Value) -> bool {
    match v {
        Value::String(s) => !s.is_empty(),
        Value::Integer(i) => *i != 0,
        Value::Float(f) => *f != 0.0,
        Value::Boolean(b) => *b,
        Value::Datetime(_) => true,
        Value::Array(a) => !a.is_empty(),
        Value::Table(t) => !t.is_empty(),
    }
}

/// `str(value)` for the scalar cases, `repr`-style rendering for containers.
/// Only reachable through a non-string `id` key, which no shipped item has.
pub fn py_str(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => format!("{f:?}"),
        Value::Boolean(b) => if *b { "True" } else { "False" }.to_string(),
        Value::Datetime(d) => d.to_string(),
        Value::Array(a) => format!(
            "[{}]",
            a.iter().map(py_inner_repr).collect::<Vec<_>>().join(", ")
        ),
        Value::Table(t) => format!(
            "{{{}}}",
            t.iter()
                .map(|(k, v)| format!("{}: {}", py_repr(k), py_inner_repr(v)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn py_inner_repr(v: &Value) -> String {
    match v {
        Value::String(s) => py_repr(s),
        other => py_str(other),
    }
}

// ── the gate ───────────────────────────────────────────────────────────────

/// Exactly what the Python writes to stdout, and the status it exits with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub stdout: String,
    pub code: i32,
}

/// One bank item as the Python loop sees it: the file it came from, and its table.
type Item = (String, toml::Table);

/// Run the whole check and render the oracle's report.
///
/// `root_str` is the engine root as an already-normalised POSIX string;
/// `bank_arg` / `topics_arg` are the raw option values (engine-root-relative
/// unless absolute), exactly as argparse would hand them over.
///
/// KNOWN RESIDUAL DEVIATIONS from the Python, none of them reachable from the
/// live tree or from any `selftest_orphan.sh` case, and all of them affecting
/// the wording of a failure line rather than whether the line appears:
///
/// - A malformed bank item produces `<file>: parse error: <msg>`, where `<msg>`
///   comes from the `toml` crate rather than from `tomllib`. Both go RED on the
///   same file; only the explanation text differs.
/// - An unreadable or non-UTF-8 `topics.toml` makes the Python raise and print
///   a traceback; here the file reads as empty, which lands as the anti-vacuous
///   "empty topic registry" ERROR. Both are non-zero; the bytes differ.
/// - A non-string, non-scalar `id` renders through `py_str`'s container branch,
///   which approximates CPython's `str()` of a nested `dict`/`list`.
pub fn evaluate(root_str: &str, bank_arg: &str, topics_arg: &str) -> Outcome {
    let bank_disp = join_posix(root_str, bank_arg);
    let topics_disp = join_posix(root_str, topics_arg);

    let mut errors: Vec<String> = Vec::new();

    let (declared, topic_errors) = read_topic_ids(Path::new(&topics_disp), &topics_disp);
    errors.extend(topic_errors);
    let known: BTreeSet<&str> = declared.iter().map(String::as_str).collect();

    let (loaded, load_errors) = read_items(Path::new(&bank_disp), &bank_disp);
    errors.extend(load_errors);

    let mut approved_n = 0usize;
    for (fname, it) in &loaded {
        if is_approved(it) {
            approved_n += 1;
        } else if !status_is_known(it) {
            let iid = match it.get("id") {
                Some(v) if py_truthy(v) => py_str(v),
                _ => fname.clone(),
            };
            let sr = match it.get("status") {
                Some(Value::String(s)) => py_repr(s),
                Some(v) => py_str(v),
                None => py_repr("draft"),
            };
            errors.push(format!("{iid}: unknown status {sr}"));
        }
    }

    // ── anti-vacuous: an empty scan set is an ERROR, never a pass ───────────
    if known.is_empty() {
        errors.push(
            "empty topic registry: zero topic ids (vacuous referential integrity is ERROR)"
                .to_string(),
        );
    }
    if loaded.is_empty() {
        errors.push("empty bank: zero items loaded (vacuous orphan scan is ERROR)".to_string());
    } else if approved_n == 0 {
        errors.push(format!("zero approved items ({} scanned): the orphan predicate measures a pool no learner can be assessed from (vacuous orphan scan is ERROR)", loaded.len()));
    }

    let mut referenced: BTreeSet<String> = BTreeSet::new();
    let mut approved_referenced: BTreeSet<String> = BTreeSet::new();
    let mut ref_count: BTreeMap<String, usize> = BTreeMap::new();
    let mut orphan_refs: Vec<String> = Vec::new();
    let mut unanchored: Vec<String> = Vec::new();

    for (fname, it) in &loaded {
        let iid = match it.get("id") {
            Some(v) if py_truthy(v) => py_str(v),
            _ => fname.clone(),
        };
        let approved = is_approved(it);
        let tids = it
            .get("topic_ids")
            .filter(|v| py_truthy(v))
            .and_then(Value::as_array);
        let Some(tids) = tids else {
            unanchored.push(format!("{iid}: missing/empty topic_ids (orphan item)"));
            continue;
        };
        for t in tids {
            match t.as_str() {
                Some(s) if !py_strip(s).is_empty() => {
                    referenced.insert(s.to_string());
                    *ref_count.entry(s.to_string()).or_insert(0) += 1;
                    if approved {
                        approved_referenced.insert(s.to_string());
                    }
                    if !known.contains(s) {
                        orphan_refs.push(format!(
                            "{iid}: unknown topic_id {} (orphan item)",
                            py_repr(s)
                        ));
                    }
                }
                _ => unanchored.push(format!("{iid}: blank topic_id entry (orphan item)")),
            }
        }
    }

    // Iterating `declared` (not `known`) is deliberate: a duplicated orphan id
    // is reported once per declaration, as the Python does.
    let orphan_topics: Vec<&String> = declared
        .iter()
        .filter(|t| !approved_referenced.contains(*t))
        .collect();

    errors.extend(unanchored.iter().cloned());
    errors.extend(orphan_refs.iter().cloned());
    errors.extend(orphan_topics.iter().map(|t| {
        format!("orphan topic {}: declared in topics.toml, referenced by 0 approved items of {} referencing", py_repr(t), ref_count.get(*t).copied().unwrap_or(0))
    }));

    let referenced_known = referenced
        .iter()
        .filter(|t| known.contains(t.as_str()))
        .count();
    let approved_known = approved_referenced
        .iter()
        .filter(|t| known.contains(t.as_str()))
        .count();

    let mut out = String::new();
    out.push_str(if errors.is_empty() {
        "PASS\n"
    } else {
        "FAIL\n"
    });
    out.push_str(&format!("  topics={topics_disp}\n"));
    out.push_str(&format!("  bank={bank_disp}\n"));
    out.push_str(&format!("  topics_declared={}\n", known.len()));
    out.push_str(&format!("  items={} scanned, {approved_n} approved (orphan predicate counts the approved pool only)\n", loaded.len()));
    out.push_str(&format!(
        "  topics_referenced={approved_known} approved of {referenced_known} referencing\n"
    ));
    out.push_str(&format!("  orphan_topics={}\n", orphan_topics.len()));
    out.push_str(&format!("  orphan_item_refs={}\n", orphan_refs.len()));
    out.push_str(&format!("  unanchored_items={}\n", unanchored.len()));

    if !errors.is_empty() {
        out.push_str("  failures:\n");
        for e in errors.iter().take(MAX_REPORT) {
            out.push_str(&format!("    - {e}\n"));
        }
        if errors.len() > MAX_REPORT {
            out.push_str(&format!("    ... +{} more\n", errors.len() - MAX_REPORT));
        }
        return Outcome {
            stdout: out,
            code: 1,
        };
    }

    out.push_str(
        "  orphan integrity GREEN (every topic assessed by an approved item; every ref resolves)\n",
    );
    Outcome {
        stdout: out,
        code: 0,
    }
}

/// `topic_ids()` — ids in declaration order, plus registry-level errors.
fn read_topic_ids(path: &Path, disp: &str) -> (Vec<String>, Vec<String>) {
    if !path.is_file() {
        return (Vec::new(), vec![format!("topics registry missing: {disp}")]);
    }
    let text = std::fs::read_to_string(path).unwrap_or_default();
    let ids = find_topic_ids(&text);

    let mut errors = Vec::new();
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut dupes: BTreeSet<&str> = BTreeSet::new();
    for t in &ids {
        if !seen.insert(t.as_str()) {
            dupes.insert(t.as_str());
        }
    }
    if !dupes.is_empty() {
        let shown = dupes
            .iter()
            .take(10)
            .map(|d| py_repr(d))
            .collect::<Vec<_>>()
            .join(", ");
        errors.push(format!("duplicate topic ids in registry: [{shown}]"));
    }
    (ids, errors)
}

/// `load_items()` — every `*.toml` under the bank dir, in `sorted()` order.
///
/// A file must contribute at least one item or produce an error line naming it;
/// there is no third outcome, which is what keeps a never-checked file from
/// reporting like a checked one.
fn read_items(dir: &Path, disp: &str) -> (Vec<Item>, Vec<String>) {
    let mut errors = Vec::new();
    let mut loaded: Vec<Item> = Vec::new();
    if !dir.is_dir() {
        return (loaded, vec![format!("bank dir missing: {disp}")]);
    }
    let Ok(rd) = std::fs::read_dir(dir) else {
        // `glob()` on an unreadable directory yields nothing; the anti-vacuous
        // leg then turns that into the ERROR it is.
        return (loaded, errors);
    };
    // `pathlib.glob("*.toml")` matches dotfiles and is case-sensitive, and
    // `sorted()` orders by name; both are reproduced here rather than by
    // `Path::extension`, which does not treat ".toml" as an extension.
    let mut names: Vec<String> = rd
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".toml"))
        .collect();
    names.sort();

    for name in names {
        let path = dir.join(&name);
        let parsed = std::fs::read_to_string(&path)
            .map_err(|e| e.to_string())
            .and_then(|t| t.parse::<toml::Table>().map_err(|e| e.to_string()));
        let data = match parsed {
            Ok(d) => d,
            Err(e) => {
                errors.push(format!("{name}: parse error: {e}"));
                continue;
            }
        };
        // `if "items" in data and isinstance(data["items"], list)` — an `items`
        // key of any other type falls through to the `id` leg, as it does there.
        let nested: Option<Vec<toml::Table>> = match data.get("items") {
            Some(Value::Array(items)) => Some(
                items
                    .iter()
                    .filter_map(|it| match it {
                        Value::Table(t) => Some(t.clone()),
                        _ => None,
                    })
                    .collect(),
            ),
            _ => None,
        };
        match nested {
            // Anti-vacuous at FILE granularity (bd-2kr). An `items[]` that
            // yields nothing took this branch and can never reach the
            // `no id or items[]` leg — Python's `elif` cannot run once the `if`
            // has — so without this arm the file is scanned, contributes zero
            // items, and is never named.
            Some(tables) if tables.is_empty() => errors.push(format!(
                "{name}: items[] yielded zero items (vacuous file scan is ERROR)"
            )),
            Some(tables) => loaded.extend(tables.into_iter().map(|t| (name.clone(), t))),
            None if data.contains_key("id") => loaded.push((name.clone(), data)),
            None => errors.push(format!("{name}: no id or items[]")),
        }
    }
    (loaded, errors)
}

/// argparse's unambiguous-prefix matching for long options.
fn resolve_option(given: &str) -> Result<&'static str, GateError> {
    let hits: Vec<&&str> = OPTIONS.iter().filter(|o| o.starts_with(given)).collect();
    match hits.len() {
        1 => Ok(hits[0]),
        0 => Err(GateError::usage(format!(
            "unrecognized argument {given:?}; known: {}",
            OPTIONS.join(" ")
        ))),
        _ => Err(GateError::usage(format!(
            "ambiguous option {given:?}; matches: {}",
            hits.iter().map(|h| **h).collect::<Vec<_>>().join(" ")
        ))),
    }
}

/// `(bank, topics)` from the argv tail, accepting `--opt v`, `--opt=v`, and the
/// unambiguous prefixes argparse allows.
pub fn parse_args(args: &[String]) -> Result<(String, String), GateError> {
    let mut bank = DEFAULT_BANK.to_string();
    let mut topics = DEFAULT_TOPICS.to_string();
    let mut i = 0;
    while i < args.len() {
        let (name, inline) = match args[i].split_once('=') {
            Some((n, v)) => (n, Some(v.to_string())),
            None => (args[i].as_str(), None),
        };
        let opt = resolve_option(name)?;
        let value = match inline {
            Some(v) => v,
            None => {
                i += 1;
                args.get(i)
                    .cloned()
                    .ok_or_else(|| GateError::usage(format!("{opt}: expected one argument")))?
            }
        };
        match opt {
            "--bank" => bank = value,
            _ => topics = value,
        }
        i += 1;
    }
    Ok((bank, topics))
}

pub fn run(ctx: &GateCtx) -> Result<(), GateError> {
    let (bank, topics) = parse_args(&ctx.args)?;

    // The Python resolves its own location (`Path(__file__).resolve()`), so the
    // printed default paths are symlink-free. Do the same to the engine root —
    // and only to the root: an absolute `--bank`/`--topics` is printed exactly
    // as `PurePosixPath` normalises it, never canonicalised.
    let root = ctx.root.canonicalize().unwrap_or_else(|_| ctx.root.clone());
    let root_str = norm_posix(&root.to_string_lossy());

    let outcome = evaluate(&root_str, &bank, &topics);
    print!("{}", outcome.stdout);
    let _ = std::io::stdout().flush();

    if outcome.code != 0 {
        // See the module header: the oracle exits 1 with an empty stderr, and
        // this port's acceptance bar is byte-identical output. Routing through
        // `GateError` would write to stderr and exit 2/4 instead.
        std::process::exit(outcome.code);
    }
    Ok(())
}

// ── tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // ── the Python emulations ─────────────────────────────────────────────

    #[test]
    fn finds_ids_in_declaration_order_with_duplicates_kept() {
        let text = "[[topic]]\nid = \"a\"\n\n[[topic]]\n  id=\"b\"\n[[topic]]\nid = \"a\"\n";
        assert_eq!(find_topic_ids(text), vec!["a", "b", "a"]);
    }

    #[test]
    fn id_pattern_ignores_non_line_start_and_nested_keys() {
        // `^` is anchored, so `topic_id = "x"` and a trailing `id` on a line
        // that already has content are not matches.
        let text = "topic_id = \"x\"\nlabel = \"y\" id = \"z\"\nid = \"real\"\n";
        assert_eq!(find_topic_ids(text), vec!["real"]);
    }

    #[test]
    fn id_pattern_spans_lines_like_the_python_does() {
        // `\s*` matches newlines in the oracle; a line-by-line port would miss
        // this and silently under-count the registry.
        assert_eq!(find_topic_ids("id\n=\n\"spanning\"\n"), vec!["spanning"]);
    }

    #[test]
    fn empty_quoted_id_is_not_a_match() {
        assert!(find_topic_ids("id = \"\"\n").is_empty());
    }

    #[test]
    fn unterminated_quote_is_not_a_match() {
        assert!(find_topic_ids("id = \"oops\n").is_empty());
    }

    #[test]
    fn first_line_needs_no_preceding_newline() {
        assert_eq!(find_topic_ids("id = \"first\""), vec!["first"]);
    }

    #[test]
    fn repr_matches_cpython_quoting_rules() {
        assert_eq!(py_repr("plain"), "'plain'");
        assert_eq!(py_repr("it's"), "\"it's\"");
        assert_eq!(py_repr("say \"hi\""), "'say \"hi\"'");
        assert_eq!(py_repr("both ' and \""), "'both \\' and \"'");
        assert_eq!(py_repr("a\\b"), "'a\\\\b'");
        assert_eq!(py_repr("a\nb\tc"), "'a\\nb\\tc'");
        assert_eq!(py_repr("\u{7}"), "'\\x07'");
    }

    #[test]
    fn posix_path_normalisation() {
        assert_eq!(norm_posix("/tmp//a/./b/"), "/tmp/a/b");
        assert_eq!(norm_posix("a/../b"), "a/../b");
        assert_eq!(norm_posix(""), ".");
        assert_eq!(norm_posix("."), ".");
        assert_eq!(norm_posix("/"), "/");
        assert_eq!(norm_posix("//a"), "//a");
        assert_eq!(norm_posix("///a"), "/a");
    }

    #[test]
    fn joining_respects_absolute_overrides() {
        assert_eq!(join_posix("/r", "bank/items"), "/r/bank/items");
        assert_eq!(join_posix("/r", "/abs/x"), "/abs/x");
        assert_eq!(join_posix("/", "bank"), "/bank");
        assert_eq!(join_posix("/r", ""), "/r");
    }

    #[test]
    fn strip_covers_the_information_separators() {
        assert!(py_strip(" \t\n\u{1c}\u{1f}").is_empty());
        assert_eq!(py_strip("  x  "), "x");
    }

    #[test]
    fn truthiness_follows_python_not_rust() {
        assert!(!py_truthy(&Value::String(String::new())));
        assert!(!py_truthy(&Value::Array(vec![])));
        assert!(!py_truthy(&Value::Integer(0)));
        assert!(!py_truthy(&Value::Boolean(false)));
        assert!(py_truthy(&Value::Integer(1)));
        assert!(py_truthy(&Value::String("x".into())));
    }

    // ── the assertions this gate exists for ───────────────────────────────

    struct Tree {
        dir: tempfile::TempDir,
    }

    impl Tree {
        fn new() -> Self {
            let t = Tree {
                dir: tempfile::tempdir().unwrap(),
            };
            std::fs::create_dir_all(t.root().join("bank/items")).unwrap();
            std::fs::create_dir_all(t.root().join("knowledge")).unwrap();
            t
        }
        fn root(&self) -> PathBuf {
            self.dir.path().to_path_buf()
        }
        fn root_str(&self) -> String {
            norm_posix(&self.root().to_string_lossy())
        }
        fn topics(&self, body: &str) {
            std::fs::write(self.root().join(DEFAULT_TOPICS), body).unwrap();
        }
        fn item(&self, name: &str, body: &str) {
            std::fs::write(self.root().join("bank/items").join(name), body).unwrap();
        }
        fn run(&self) -> Outcome {
            evaluate(&self.root_str(), DEFAULT_BANK, DEFAULT_TOPICS)
        }
    }

    fn good_tree() -> Tree {
        let t = Tree::new();
        t.topics("[[topic]]\nid = \"t-one\"\n\n[[topic]]\nid = \"t-two\"\n");
        t.item(
            "a.toml",
            "id = \"i-a\"\ntopic_ids = [\"t-one\"]\nstatus = \"approved\"\n",
        );
        t.item(
            "b.toml",
            "id = \"i-b\"\ntopic_ids = [\"t-two\"]\nstatus = \"approved\"\n",
        );
        t
    }

    #[test]
    fn clean_tree_is_green() {
        let out = good_tree().run();
        assert_eq!(out.code, 0, "{}", out.stdout);
        assert!(out.stdout.starts_with("PASS\n"), "{}", out.stdout);
        assert!(
            out.stdout.contains("orphan integrity GREEN"),
            "{}",
            out.stdout
        );
        assert!(out.stdout.contains("topics_declared=2"), "{}", out.stdout);
        assert!(
            out.stdout.contains("items=2 scanned, 2 approved"),
            "{}",
            out.stdout
        );
    }

    #[test]
    fn unknown_topic_ref_is_red() {
        let t = good_tree();
        t.item("c.toml", "id = \"i-c\"\ntopic_ids = [\"t-nope\"]\n");
        let out = t.run();
        assert_eq!(out.code, 1);
        assert!(
            out.stdout
                .contains("i-c: unknown topic_id 't-nope' (orphan item)"),
            "{}",
            out.stdout
        );
    }

    #[test]
    fn empty_topic_ids_is_unanchored() {
        let t = good_tree();
        t.item("c.toml", "id = \"i-c\"\ntopic_ids = []\n");
        let out = t.run();
        assert_eq!(out.code, 1);
        assert!(
            out.stdout
                .contains("i-c: missing/empty topic_ids (orphan item)"),
            "{}",
            out.stdout
        );
    }

    #[test]
    fn missing_topic_ids_key_is_unanchored() {
        let t = good_tree();
        t.item("c.toml", "id = \"i-c\"\n");
        let out = t.run();
        assert_eq!(out.code, 1);
        assert!(out.stdout.contains("unanchored_items=1"), "{}", out.stdout);
    }

    #[test]
    fn blank_topic_id_entry_is_unanchored() {
        let t = good_tree();
        t.item("c.toml", "id = \"i-c\"\ntopic_ids = [\"  \"]\n");
        let out = t.run();
        assert_eq!(out.code, 1);
        assert!(
            out.stdout
                .contains("i-c: blank topic_id entry (orphan item)"),
            "{}",
            out.stdout
        );
    }

    #[test]
    fn topic_referenced_by_zero_items_is_red() {
        let t = good_tree();
        t.topics(
            "[[topic]]\nid = \"t-one\"\n[[topic]]\nid = \"t-two\"\n[[topic]]\nid = \"t-lonely\"\n",
        );
        let out = t.run();
        assert_eq!(out.code, 1);
        assert!(
            out.stdout.contains(
                "orphan topic 't-lonely': declared in topics.toml, referenced by 0 approved items of 0 referencing"
            ),
            "{}",
            out.stdout
        );
    }

    // ── anti-vacuous: never green on nothing ──────────────────────────────

    #[test]
    fn empty_bank_is_an_error_not_a_pass() {
        let t = Tree::new();
        t.topics("[[topic]]\nid = \"t-one\"\n");
        let out = t.run();
        assert_eq!(out.code, 1);
        assert!(
            out.stdout
                .contains("empty bank: zero items loaded (vacuous orphan scan is ERROR)"),
            "{}",
            out.stdout
        );
    }

    #[test]
    fn empty_topics_registry_is_an_error_not_a_pass() {
        let t = Tree::new();
        t.topics("");
        t.item("a.toml", "id = \"i-a\"\ntopic_ids = [\"t-one\"]\n");
        let out = t.run();
        assert_eq!(out.code, 1);
        assert!(
            out.stdout.contains(
                "empty topic registry: zero topic ids (vacuous referential integrity is ERROR)"
            ),
            "{}",
            out.stdout
        );
    }

    #[test]
    fn missing_bank_dir_is_an_error() {
        let t = Tree::new();
        t.topics("[[topic]]\nid = \"t-one\"\n");
        let out = evaluate(&t.root_str(), "bank/nowhere", DEFAULT_TOPICS);
        assert_eq!(out.code, 1);
        assert!(out.stdout.contains("bank dir missing:"), "{}", out.stdout);
    }

    #[test]
    fn missing_topics_registry_is_an_error() {
        let t = Tree::new();
        t.item("a.toml", "id = \"i-a\"\ntopic_ids = [\"t-one\"]\n");
        let out = evaluate(&t.root_str(), DEFAULT_BANK, "knowledge/nowhere.toml");
        assert_eq!(out.code, 1);
        assert!(
            out.stdout.contains("topics registry missing:"),
            "{}",
            out.stdout
        );
    }

    #[test]
    fn both_empty_reports_both_errors() {
        let t = Tree::new();
        t.topics("");
        let out = t.run();
        assert_eq!(out.code, 1);
        assert!(
            out.stdout.contains("empty topic registry"),
            "{}",
            out.stdout
        );
        assert!(out.stdout.contains("empty bank"), "{}", out.stdout);
    }

    // ── report shape ──────────────────────────────────────────────────────

    #[test]
    fn duplicate_topic_ids_are_reported_as_a_python_list() {
        let t = good_tree();
        t.topics(
            "[[topic]]\nid = \"t-one\"\n[[topic]]\nid = \"t-one\"\n[[topic]]\nid = \"t-two\"\n",
        );
        let out = t.run();
        assert_eq!(out.code, 1);
        assert!(
            out.stdout
                .contains("duplicate topic ids in registry: ['t-one']"),
            "{}",
            out.stdout
        );
        // set() de-duplication means one declared id, and it is still counted once.
        assert!(out.stdout.contains("topics_declared=2"), "{}", out.stdout);
    }

    #[test]
    fn report_truncates_at_max_report() {
        let t = Tree::new();
        let mut topics = String::new();
        for i in 0..(MAX_REPORT + 5) {
            topics.push_str(&format!("[[topic]]\nid = \"t{i:03}\"\n"));
        }
        t.topics(&topics);
        t.item(
            "a.toml",
            "id = \"i-a\"\ntopic_ids = [\"t000\"]\nstatus = \"approved\"\n",
        );
        let out = t.run();
        assert_eq!(out.code, 1);
        let shown = out
            .stdout
            .lines()
            .filter(|l| l.starts_with("    - "))
            .count();
        assert_eq!(shown, MAX_REPORT);
        assert!(out.stdout.contains("    ... +4 more"), "{}", out.stdout);
    }

    #[test]
    fn items_array_files_are_expanded() {
        let t = Tree::new();
        t.topics("[[topic]]\nid = \"t-one\"\n");
        t.item(
            "multi.toml",
            "[[items]]\nid = \"i-1\"\ntopic_ids = [\"t-one\"]\nstatus = \"approved\"\n\n[[items]]\nid = \"i-2\"\ntopic_ids = [\"t-one\"]\nstatus = \"approved\"\n",
        );
        let out = t.run();
        assert_eq!(out.code, 0, "{}", out.stdout);
        assert!(out.stdout.contains("items=2"), "{}", out.stdout);
    }

    #[test]
    fn a_file_with_neither_id_nor_items_is_reported() {
        let t = good_tree();
        t.item("junk.toml", "label = \"nothing useful\"\n");
        let out = t.run();
        assert_eq!(out.code, 1);
        assert!(
            out.stdout.contains("junk.toml: no id or items[]"),
            "{}",
            out.stdout
        );
    }

    #[test]
    fn a_file_yielding_zero_items_is_named_and_red() {
        // bd-2kr: `items = []` used to take the list branch, add nothing, and
        // never reach the `no id or items[]` leg. It reported exactly like a
        // file that passed, because the two clean items carried the count.
        let t = good_tree();
        t.item("silently-empty.toml", "items = []\n");
        let out = t.run();
        assert_eq!(out.code, 1, "{}", out.stdout);
        assert!(
            out.stdout.contains(
                "silently-empty.toml: items[] yielded zero items (vacuous file scan is ERROR)"
            ),
            "the file that contributed nothing must be named: {}",
            out.stdout
        );
        // the aggregate stayed healthy on its own — which is the whole point
        assert!(out.stdout.contains("items=2"), "{}", out.stdout);
    }

    #[test]
    fn an_items_array_of_non_tables_yields_zero_and_is_red() {
        // Same branch, reached without an empty literal: the entries exist but
        // none of them is a table, so the loop still loads nothing.
        let t = good_tree();
        t.item("scalars.toml", "items = [1, 2, 3]\n");
        let out = t.run();
        assert_eq!(out.code, 1, "{}", out.stdout);
        assert!(
            out.stdout
                .contains("scalars.toml: items[] yielded zero items"),
            "{}",
            out.stdout
        );
    }

    #[test]
    fn the_single_item_id_form_still_passes() {
        // Known-GOOD leg. The `elif "id" in data` branch must survive the fix:
        // a legitimate one-item file has no `items` key at all and is not empty.
        let t = Tree::new();
        t.topics("[[topic]]\nid = \"t-one\"\n");
        t.item(
            "solo.toml",
            "id = \"i-solo\"\ntopic_ids = [\"t-one\"]\nstatus = \"approved\"\n",
        );
        let out = t.run();
        assert_eq!(out.code, 0, "{}", out.stdout);
        assert!(out.stdout.starts_with("PASS\n"), "{}", out.stdout);
        assert!(out.stdout.contains("items=1"), "{}", out.stdout);
    }

    #[test]
    fn a_non_empty_items_array_is_not_flagged_as_vacuous() {
        let t = Tree::new();
        t.topics("[[topic]]\nid = \"t-one\"\n");
        t.item(
            "multi.toml",
            "[[items]]\nid = \"i-1\"\ntopic_ids = [\"t-one\"]\nstatus = \"approved\"\n",
        );
        let out = t.run();
        assert_eq!(out.code, 0, "{}", out.stdout);
        assert!(
            !out.stdout.contains("yielded zero items"),
            "a file that did contribute must not be flagged: {}",
            out.stdout
        );
    }

    #[test]
    fn an_item_without_an_id_falls_back_to_its_filename() {
        let t = good_tree();
        t.item("noid.toml", "[[items]]\ntopic_ids = []\n");
        let out = t.run();
        assert_eq!(out.code, 1);
        assert!(
            out.stdout
                .contains("noid.toml: missing/empty topic_ids (orphan item)"),
            "{}",
            out.stdout
        );
    }

    #[test]
    fn a_malformed_item_is_a_parse_error_not_a_skip() {
        let t = good_tree();
        t.item("bad.toml", "id = \"unterminated\n");
        let out = t.run();
        assert_eq!(out.code, 1);
        assert!(
            out.stdout.contains("bad.toml: parse error:"),
            "{}",
            out.stdout
        );
    }

    #[test]
    fn non_toml_files_are_not_scanned() {
        let t = good_tree();
        t.item("notes.md", "not a bank item\n");
        t.item("case.TOML", "id = \"i-x\"\n");
        let out = t.run();
        assert_eq!(out.code, 0, "{}", out.stdout);
        assert!(out.stdout.contains("items=2"), "{}", out.stdout);
    }

    // ── argument handling ─────────────────────────────────────────────────

    #[test]
    fn defaults_apply_when_no_flags_are_given() {
        let (b, t) = parse_args(&[]).unwrap();
        assert_eq!(b, DEFAULT_BANK);
        assert_eq!(t, DEFAULT_TOPICS);
    }

    #[test]
    fn both_flag_spellings_and_prefixes_parse() {
        let args: Vec<String> = ["--bank", "/x", "--topics=/y"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(parse_args(&args).unwrap(), ("/x".into(), "/y".into()));
        let abbr: Vec<String> = ["--ba", "/x"].iter().map(|s| s.to_string()).collect();
        assert_eq!(parse_args(&abbr).unwrap().0, "/x");
    }

    #[test]
    fn an_unknown_flag_is_usage_never_a_silent_pass() {
        let args = vec!["--bnak".to_string(), "/x".to_string()];
        assert_eq!(
            parse_args(&args).unwrap_err().code(),
            crate::exit::USAGE,
            "a typo must not read as a default-path run"
        );
    }

    #[test]
    fn a_flag_without_its_value_is_usage() {
        let args = vec!["--bank".to_string()];
        assert_eq!(parse_args(&args).unwrap_err().code(), crate::exit::USAGE);
    }

    // ── the header's own honesty ──────────────────────────────────────────

    #[test]
    fn header_states_a_floor_raise_and_overclaims_nothing() {
        let src = include_str!("verify_orphans.rs");
        let header: String = src
            .lines()
            .take_while(|l| l.starts_with("//!"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            header.contains("FLOOR-RAISE"),
            "header must state the claim class"
        );
        assert!(
            header.contains("CANNOT"),
            "header must state what the gate cannot decide"
        );
        for banned in ["guarantee", "proves", "makes impossible", "impossible"] {
            assert!(
                !header.to_lowercase().contains(banned),
                "header overclaims with {banned:?}"
            );
        }
    }
}
