//! verify-knowledge-paths — Rust port of `scripts/verify_knowledge_paths.py`
//! (bd-substrate-rust-migration-jhd.5).
//!
//! # CLAIM: FLOOR-RAISE
//!
//! This gate raises one floor: **every `primary_notes` pointer the knowledge
//! registries declare must land on a real file.** Concretely it goes RED when any
//! of these is true —
//!
//!   1. `knowledge/domains.toml` is missing, or declares zero `[[domain]]` rows.
//!   2. A `[[domain]]` row has no `primary_notes` key at all.
//!   3. A row's `primary_notes` is empty (or whitespace only) without
//!      `exam_weight_unknown = true` to license the hole.
//!   4. A non-empty `primary_notes` does not resolve to an existing file, where
//!      resolution is relative to the **engine root** — the corpus lives at
//!      `../modules/`, one level above `course-engine/`.
//!   5. A `primary_notes` resolves *under* `course-engine/modules/`, the wrong
//!      tree — a path that would silently be satisfied by a stray local copy.
//!   6. Any other top-level `knowledge/*.toml` carries a `primary_notes = ...`
//!      line whose value does not resolve. This leg is a textual line scan, not a
//!      TOML parse, so it also catches the key in files with no schema for it.
//!
//! # WHAT THIS GATE CANNOT DO
//!
//! It reads no note content. A `primary_notes` that resolves to an empty file, to
//! the wrong module's notes, or to a stub with a heading and nothing under it
//! clears this gate exactly as a complete set of notes does — the check is that
//! the pointer lands, never that what it lands on is worth reading, current, or
//! about the domain that names it. It does not decide that the domain registry is
//! *complete*: a module with no `[[domain]]` row at all is invisible here, because
//! nothing dangles. It says nothing about the reverse direction either — a file
//! under `../modules/` that no domain references is not its business.
//!
//! Two vacuity holes are inherited from the retired oracle deliberately and are
//! NOT closed here (see `evaluate`): a registry whose every row is an
//! `exam_weight_unknown` empty passes with `primary_notes_checked=0`, and the
//! second-leg line scan is silent when it scans zero files. The rust module
//! tests pin both holes so closing them later is a visible, reviewed change.
//!
//! The floor moves from *silence* to *every declared pointer resolves*. That is
//! the whole claim, and this header will not stretch it.
//!
//! # BYTE-EXACTNESS WITH THE RETIRED PYTHON ORACLE
//!
//! `scripts/verify_knowledge_paths.py` and `tests/diff_verify_knowledge_paths.rs`
//! were deleted (bd-substrate-rust-migration-jhd.33). check.sh never invoked
//! the `.py`; CI is `cdcp_gate verify-knowledge-paths`; known-bad lives here.
//! This module still carries hand-written emulations of CPython behaviour
//! (`repr()` of a `str`, `str.strip()`, `str.splitlines()`, `posixpath.realpath`,
//! `PurePath.relative_to`) rather than the idiomatic Rust nearest-neighbour,
//! and the failure report still goes to **stdout with exit status 1** instead
//! of through `GateError`: the dispatcher's `report()` writes to stderr and
//! maps to exit 2 or 4. `crate::exit`'s codes are therefore deliberately not
//! used on the RED path. Same knowing, single-file deviation `verify_orphans`
//! records, recorded here rather than made quietly.
//!
//! Two further deviations, both outside the oracle's agreeing surface:
//!
//! - **argv.** The oracle ignores `sys.argv` entirely, so `--anything` is a silent
//!   full-tree run there. This gate rejects any argument as USAGE (exit 3). A
//!   typo'd flag reading as "the gate passed" is the one thing this crate's
//!   contract will not have; `scripts/check.sh` passes no arguments, so the
//!   divergence is unreachable from the wiring. Flagged for review.
//! - **crash-shaped inputs** (unreadable/non-UTF-8 `domains.toml`, malformed TOML,
//!   a `domain` key that is not an array of tables, a *directory* named `*.toml`
//!   in `knowledge/`). The oracle raises and prints a Python traceback with exit
//!   1; this gate returns `GateError::Error` (exit 4, stderr). Both are non-zero
//!   and neither is a pass; the bytes differ, and reproducing a traceback would be
//!   worse than naming the gap. Enumerated on `evaluate`.
//!
//! # bd-lt7
//!
//! bd-lt7 tracks gates that hardcode a module bound (`range(1, 15)`, `<= 14`) and
//! so silently exempt module 15. **This script contains no such bound** — it is
//! driven entirely by the rows present in `knowledge/domains.toml`, so the
//! `15-ops-adjacent` row is checked like any other (the live tree reports
//! `primary_notes_checked=15`). Nothing was fixed here because nothing was broken
//! here; the note exists so the next reader does not have to re-derive it.

use crate::registry::{GateCtx, GateError};
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use toml::Value;

pub const NAME: &str = "verify-knowledge-paths";
pub const SUMMARY: &str =
    "every non-empty primary_notes in knowledge/ resolves to a real file under the parent corpus";

/// Engine-root-relative registry the first leg parses. Matches `DOMAINS`.
pub const DOMAINS_REL: &str = "knowledge/domains.toml";
/// Engine-root-relative directory the second leg line-scans. Matches `KNOWLEDGE_DIR`.
pub const KNOWLEDGE_REL: &str = "knowledge";

// ── Python-behaviour emulations ────────────────────────────────────────────
// Each exists because the acceptance bar is byte-identical output, not merely an
// identical verdict.

/// The character set `str.strip()` removes and `str.isspace()` reports: Unicode
/// `White_Space` plus the four ASCII information separators (0x1C-0x1F) that Rust's
/// `char::is_whitespace` does not count.
pub fn py_space(c: char) -> bool {
    c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c)
}

/// `str.strip()` with no argument.
pub fn py_strip(s: &str) -> &str {
    s.trim_matches(py_space)
}

/// `repr()` of a Python `str`.
///
/// Single quotes unless the value contains `'` and no `"`. Backslash, the active
/// quote, and the three named escapes are escaped; C0 controls and DEL become
/// `\xNN`. Non-ASCII characters pass through, which matches CPython for printable
/// code points and differs for unprintable ones (CPython would emit
/// `\xNN`/`\uNNNN`); reaching that needs an unprintable non-ASCII character inside
/// a `primary_notes` value.
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

/// `str.splitlines()`. Splits on far more than `\n`: CR, CRLF, VT, FF, the three
/// ASCII separators 0x1C-0x1E, NEL, LS, and PS. Line *numbers* in the second leg's
/// findings depend on this, so a `lines()` stand-in would mis-number a file that
/// happens to contain a form feed.
pub fn py_splitlines(s: &str) -> Vec<&str> {
    const BREAKS: [char; 9] = [
        '\n', '\r', '\u{0b}', '\u{0c}', '\u{1c}', '\u{1d}', '\u{1e}', '\u{2028}', '\u{2029}',
    ];
    let mut out = Vec::new();
    let chars: Vec<(usize, char)> = s.char_indices().collect();
    let mut start = 0usize;
    let mut k = 0usize;
    while k < chars.len() {
        let (idx, c) = chars[k];
        if BREAKS.contains(&c) || c == '\u{85}' {
            out.push(&s[start..idx]);
            let mut next = idx + c.len_utf8();
            if c == '\r' && k + 1 < chars.len() && chars[k + 1].1 == '\n' {
                next = chars[k + 1].0 + 1;
                k += 1;
            }
            start = next;
        }
        k += 1;
    }
    if start < s.len() {
        out.push(&s[start..]);
    }
    out
}

/// Python truthiness of a value `tomllib` would have produced.
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
///
/// Reachable only through a non-string `id` or `primary_notes`, which no shipped
/// registry has. The `Datetime` arm is a known approximation: `tomllib` yields a
/// `datetime` whose `str()` uses a space separator, the `toml` crate's `Display`
/// keeps the `T`.
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

/// `PurePosixPath.is_absolute()`.
pub fn is_absolute_posix(s: &str) -> bool {
    s.starts_with('/')
}

/// `str(PurePosixPath(s))`: empty and `.` components drop, `..` is kept, duplicate
/// separators collapse, a trailing separator drops, and exactly two leading
/// separators survive as a root.
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

/// `root / rel` with `PurePosixPath` semantics: an absolute `rel` replaces `root`.
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

/// `Path.resolve()` — a port of CPython's `posixpath.realpath(..., strict=False)`.
///
/// Not `std::fs::canonicalize`: that fails outright on a missing path, and the
/// oracle's whole RED path is *printing the resolved form of a path that does not
/// exist*. `..` is applied to the already-symlink-resolved prefix (not lexically
/// to the input), missing components are appended as-is, and a symlink loop
/// degrades to the unresolved path instead of erroring — all three are the
/// oracle's observable behaviour.
pub fn py_resolve(filename: &str) -> String {
    // The stack of unresolved parts. `None` marks "a symlink target finished
    // resolving"; the entry under it is the symlink whose result to cache.
    let mut rest: Vec<Option<String>> = filename
        .split('/')
        .rev()
        .map(|s| Some(s.to_string()))
        .collect();
    let mut part_count = rest.len();
    let mut path = if filename.starts_with('/') {
        "/".to_string()
    } else {
        std::env::current_dir()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "/".to_string())
    };
    let mut seen: HashMap<String, Option<String>> = HashMap::new();

    while part_count > 0 {
        let Some(popped) = rest.pop() else { break };
        let Some(name) = popped else {
            // A symlink target just finished resolving.
            if let Some(Some(key)) = rest.pop() {
                seen.insert(key, Some(path.clone()));
            }
            continue;
        };
        part_count -= 1;
        if name.is_empty() || name == "." {
            continue;
        }
        if name == ".." {
            path = match path.rfind('/') {
                Some(0) | None => "/".to_string(),
                Some(i) => path[..i].to_string(),
            };
            continue;
        }
        let newpath = if path == "/" {
            format!("/{name}")
        } else {
            format!("{path}/{name}")
        };

        let target = match std::fs::symlink_metadata(&newpath) {
            Ok(md) => {
                if !md.file_type().is_symlink() {
                    path = newpath;
                    continue;
                }
                if let Some(cached) = seen.get(&newpath) {
                    match cached {
                        Some(resolved) => {
                            path = resolved.clone();
                        }
                        // Already-seen-but-unresolved is a symlink loop; non-strict
                        // resolution keeps the unresolved path.
                        None => path = newpath,
                    }
                    continue;
                }
                std::fs::read_link(&newpath)
                    .ok()
                    .map(|t| t.to_string_lossy().into_owned())
            }
            Err(_) => None,
        };

        match target {
            Some(t) => {
                if t.starts_with('/') {
                    path = "/".to_string();
                }
                seen.insert(newpath.clone(), None);
                rest.push(Some(newpath));
                rest.push(None);
                let parts: Vec<&str> = t.split('/').collect();
                part_count += parts.len();
                for p in parts.into_iter().rev() {
                    rest.push(Some(p.to_string()));
                }
            }
            // lstat or readlink failed: the component does not exist (or raced).
            None => path = newpath,
        }
    }
    path
}

/// The components `PurePath.parts` yields, with a leading `/` kept as its own
/// element. (A `//`-rooted path renders its root as `/` here rather than `//`;
/// only a `primary_notes` beginning with exactly two slashes could tell.)
fn path_parts(p: &str) -> Vec<&str> {
    let mut v = Vec::new();
    if p.starts_with('/') {
        v.push("/");
    }
    v.extend(p.split('/').filter(|c| !c.is_empty() && *c != "."));
    v
}

/// `cand.relative_to(base)` without raising: true when `base` is `cand` or one of
/// its ancestors. Lexical, as `PurePath.relative_to` is — the oracle compares the
/// *resolved* candidate against an *unresolved* `ROOT / "modules"`, and that
/// asymmetry is preserved.
pub fn is_relative_to(cand: &str, base: &str) -> bool {
    let c = path_parts(cand);
    let b = path_parts(base);
    b.len() <= c.len() && c[..b.len()] == b[..]
}

// ── the gate ───────────────────────────────────────────────────────────────

/// Exactly what the oracle writes to stdout, and the status it exits with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub stdout: String,
    pub code: i32,
}

impl Outcome {
    fn line(msg: &str) -> Self {
        Outcome {
            stdout: format!("{msg}\n"),
            code: 1,
        }
    }
}

/// Run both legs and render the oracle's report.
///
/// `root_str` is the engine root as an already-resolved POSIX string.
///
/// INHERITED VACUITY HOLES (ported deliberately, reported as findings, not fixed):
///
/// - A registry whose every row is an empty `primary_notes` with
///   `exam_weight_unknown = true` prints `PASS` / `primary_notes_checked=0`. Zero
///   pointers checked is indistinguishable here from every pointer landing.
/// - The second leg is silent when `knowledge/` holds no other `*.toml`, and when
///   those files hold no `primary_notes` line. Only the *first* leg has an
///   anti-vacuous check, and it covers only "zero `[[domain]]` rows".
///
/// CRASH-SHAPED INPUTS return `Err` here and a traceback there (see the module
/// header): unreadable or non-UTF-8 `domains.toml`; malformed TOML; a `domain` key
/// that is truthy but not an array of tables; a directory named `*.toml` inside
/// `knowledge/`.
pub fn evaluate(root_str: &str) -> Result<Outcome, GateError> {
    let domains_path = join_posix(root_str, DOMAINS_REL);
    if !Path::new(&domains_path).is_file() {
        return Ok(Outcome::line("FAIL: knowledge/domains.toml missing"));
    }

    let text = std::fs::read_to_string(&domains_path).map_err(|e| {
        GateError::error(format!(
            "{domains_path}: unreadable or not UTF-8: {e} (the oracle raises here)"
        ))
    })?;
    let data: toml::Table = text.parse().map_err(|e| {
        GateError::error(format!(
            "{domains_path}: TOML parse error: {e} (the oracle raises here)"
        ))
    })?;

    let domains: &Vec<Value> = match data.get("domain") {
        Some(Value::Array(a)) if !a.is_empty() => a,
        Some(v) if py_truthy(v) => {
            return Err(GateError::error(format!(
                "{domains_path}: `domain` is {}, not an array of tables (the oracle raises here)",
                kind_of(v)
            )))
        }
        _ => return Ok(Outcome::line("FAIL: domains.toml has zero [[domain]] rows")),
    };

    let mut errors: Vec<String> = Vec::new();
    let mut checked: usize = 0;
    let mut empty_ok: usize = 0;
    let modules_base = join_posix(root_str, "modules");

    for dom in domains {
        let Some(dom) = dom.as_table() else {
            return Err(GateError::error(format!(
                "{domains_path}: a [[domain]] entry is {}, not a table (the oracle raises here)",
                kind_of(dom)
            )));
        };
        let did = match dom.get("id") {
            Some(v) if py_truthy(v) => py_str(v),
            _ => "<missing-id>".to_string(),
        };
        let Some(pn) = dom.get("primary_notes") else {
            errors.push(format!("{did}: primary_notes field missing"));
            continue;
        };
        let rendered = py_str(pn);
        let pn_s = py_strip(&rendered);
        if pn_s.is_empty() {
            // `is True` in the oracle: a truthy 1 does not license the hole.
            if matches!(dom.get("exam_weight_unknown"), Some(Value::Boolean(true))) {
                empty_ok += 1;
                continue;
            }
            errors.push(format!(
                "{did}: empty primary_notes without exam_weight_unknown=true"
            ));
            continue;
        }

        // Resolve relative to the engine ROOT (not knowledge/).
        let candidate = if is_absolute_posix(pn_s) {
            py_resolve(&norm_posix(pn_s))
        } else {
            py_resolve(&join_posix(root_str, pn_s))
        };

        checked += 1;
        if !Path::new(&candidate).is_file() {
            errors.push(format!(
                "{did}: primary_notes does not resolve to a file: {} (resolved {candidate})",
                py_repr(pn_s)
            ));
            continue;
        }

        if is_relative_to(&candidate, &modules_base) {
            errors.push(format!(
                "{did}: primary_notes resolves under course-engine/modules/ ({candidate}); parent corpus is ../modules/ relative to ROOT"
            ));
        }
    }

    // Second leg: line-scan every other top-level knowledge/*.toml.
    let knowledge_dir = join_posix(root_str, KNOWLEDGE_REL);
    for name in glob_toml_sorted(&knowledge_dir) {
        if name == "domains.toml" {
            continue;
        }
        let full = format!("{knowledge_dir}/{name}");
        let text = std::fs::read_to_string(&full).map_err(|e| {
            GateError::error(format!(
                "{full}: unreadable, a directory, or not UTF-8: {e} (the oracle raises here)"
            ))
        })?;
        for (i, line) in py_splitlines(&text).iter().enumerate() {
            let lineno = i + 1;
            let stripped = py_strip(line);
            if !stripped.starts_with("primary_notes") {
                continue;
            }
            let Some((_, rhs)) = stripped.split_once('=') else {
                continue;
            };
            let raw = py_strip(rhs).trim_matches('"').trim_matches('\'');
            if raw.is_empty() {
                continue;
            }
            let cand = if is_absolute_posix(raw) {
                norm_posix(raw)
            } else {
                py_resolve(&join_posix(root_str, raw))
            };
            if !Path::new(&cand).is_file() {
                errors.push(format!(
                    "{name}:{lineno}: primary_notes {} does not resolve",
                    py_repr(raw)
                ));
            }
        }
    }

    if !errors.is_empty() {
        let mut out = String::from("FAIL\n");
        for e in &errors {
            out.push_str(&format!("  - {e}\n"));
        }
        return Ok(Outcome {
            stdout: out,
            code: 1,
        });
    }

    Ok(Outcome {
        stdout: format!(
            "PASS\n  primary_notes_checked={checked}\n  empty_allowed={empty_ok}\n  root={root_str}\n"
        ),
        code: 0,
    })
}

fn kind_of(v: &Value) -> &'static str {
    match v {
        Value::String(_) => "a string",
        Value::Integer(_) => "an integer",
        Value::Float(_) => "a float",
        Value::Boolean(_) => "a boolean",
        Value::Datetime(_) => "a datetime",
        Value::Array(_) => "an array",
        Value::Table(_) => "a table",
    }
}

/// `sorted(dir.glob("*.toml"))` — names only.
///
/// `pathlib.glob` matches dotfiles and is case-sensitive, and it yields
/// directories too (a directory named `x.toml` makes the oracle raise, which is
/// why the caller propagates the read error rather than skipping). `sorted()` on
/// same-directory paths orders by name; Rust's `sort` on `String` is byte order,
/// which for UTF-8 is code-point order, so the two agree.
fn glob_toml_sorted(dir: &str) -> Vec<String> {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = rd
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".toml"))
        .collect();
    names.sort();
    names
}

pub fn run(ctx: &GateCtx) -> Result<(), GateError> {
    // The oracle ignores sys.argv; this gate does not (module header). A typo'd
    // flag must not read as a clean full-tree run.
    ctx.reject_unknown_flags(&[])?;

    // The oracle resolves its own location, so the printed root is symlink-free.
    let root = ctx.root.canonicalize().unwrap_or_else(|_| ctx.root.clone());
    let root_str = norm_posix(&root.to_string_lossy());

    let outcome = evaluate(&root_str)?;
    print!("{}", outcome.stdout);
    let _ = std::io::stdout().flush();

    if outcome.code != 0 {
        // See the header: the oracle exits 1 with an empty stderr, and this port's
        // acceptance bar is byte-identical output. `GateError` would write to
        // stderr and exit 2 instead.
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
    fn repr_matches_cpython_quoting_rules() {
        assert_eq!(py_repr("plain"), "'plain'");
        assert_eq!(py_repr("it's"), "\"it's\"");
        assert_eq!(py_repr("say \"hi\""), "'say \"hi\"'");
        assert_eq!(py_repr("both ' and \""), "'both \\' and \"'");
        assert_eq!(py_repr("a\\b"), "'a\\\\b'");
        assert_eq!(py_repr("a\nb\tc"), "'a\\nb\\tc'");
        assert_eq!(py_repr("\u{7}"), "'\\x07'");
        assert_eq!(py_repr("../modules/x.md"), "'../modules/x.md'");
    }

    #[test]
    fn strip_covers_the_information_separators() {
        assert!(py_strip(" \t\n\u{1c}\u{1f}").is_empty());
        assert_eq!(py_strip("  x  "), "x");
    }

    #[test]
    fn splitlines_breaks_on_more_than_newline() {
        assert_eq!(
            py_splitlines("a\nb\r\nc\rd\u{0c}e"),
            vec!["a", "b", "c", "d", "e"]
        );
        // A trailing break yields no extra empty line, as Python's does.
        assert_eq!(py_splitlines("a\n"), vec!["a"]);
        assert_eq!(py_splitlines("a\n\nb"), vec!["a", "", "b"]);
        assert!(py_splitlines("").is_empty());
    }

    #[test]
    fn truthiness_follows_python_not_rust() {
        assert!(!py_truthy(&Value::String(String::new())));
        assert!(!py_truthy(&Value::Array(vec![])));
        assert!(!py_truthy(&Value::Integer(0)));
        assert!(!py_truthy(&Value::Boolean(false)));
        assert!(py_truthy(&Value::String("x".into())));
    }

    #[test]
    fn posix_path_normalisation_and_joining() {
        assert_eq!(norm_posix("/tmp//a/./b/"), "/tmp/a/b");
        assert_eq!(norm_posix("a/../b"), "a/../b");
        assert_eq!(norm_posix(""), ".");
        assert_eq!(join_posix("/r", "../modules/x.md"), "/r/../modules/x.md");
        assert_eq!(join_posix("/r", "/abs/x"), "/abs/x");
    }

    #[test]
    fn resolve_eliminates_dotdot_on_a_missing_path() {
        // canonicalize() cannot do this: the target does not exist.
        let td = tempfile::tempdir().unwrap();
        let root = td.path().canonicalize().unwrap();
        let root_s = root.to_string_lossy().into_owned();
        let got = py_resolve(&join_posix(&root_s, "../modules/nope.md"));
        let parent = root.parent().unwrap().to_string_lossy().into_owned();
        assert_eq!(got, format!("{parent}/modules/nope.md"));
    }

    #[test]
    fn resolve_follows_a_symlink_then_applies_dotdot_to_the_target() {
        let td = tempfile::tempdir().unwrap();
        let root = td.path().canonicalize().unwrap();
        std::fs::create_dir_all(root.join("real/inner")).unwrap();
        std::fs::create_dir_all(root.join("here")).unwrap();
        std::os::unix::fs::symlink(root.join("real/inner"), root.join("here/link")).unwrap();
        let root_s = root.to_string_lossy().into_owned();
        let got = py_resolve(&format!("{root_s}/here/link/.."));
        assert_eq!(
            got,
            format!("{root_s}/real"),
            "`..` must apply after the link"
        );
    }

    #[test]
    fn resolve_survives_a_symlink_loop_instead_of_hanging() {
        let td = tempfile::tempdir().unwrap();
        let root = td.path().canonicalize().unwrap();
        std::os::unix::fs::symlink(root.join("b"), root.join("a")).unwrap();
        std::os::unix::fs::symlink(root.join("a"), root.join("b")).unwrap();
        let root_s = root.to_string_lossy().into_owned();
        let got = py_resolve(&format!("{root_s}/a"));
        assert!(got.starts_with(&root_s), "{got}");
    }

    #[test]
    fn relative_to_is_lexical_and_component_wise() {
        assert!(is_relative_to("/r/modules/a.md", "/r/modules"));
        assert!(is_relative_to("/r/modules", "/r/modules"));
        assert!(!is_relative_to("/r/modules-extra/a.md", "/r/modules"));
        assert!(!is_relative_to("/other/modules/a.md", "/r/modules"));
    }

    // ── the assertions this gate exists for ───────────────────────────────

    struct Tree {
        dir: tempfile::TempDir,
    }

    impl Tree {
        /// `<tmp>/study/engine` is ROOT; `<tmp>/study/modules` is the parent corpus.
        fn new() -> Self {
            let t = Tree {
                dir: tempfile::tempdir().unwrap(),
            };
            std::fs::create_dir_all(t.root().join("knowledge")).unwrap();
            std::fs::create_dir_all(t.corpus()).unwrap();
            t
        }
        fn root(&self) -> PathBuf {
            self.dir.path().canonicalize().unwrap().join("study/engine")
        }
        fn corpus(&self) -> PathBuf {
            self.dir
                .path()
                .canonicalize()
                .unwrap()
                .join("study/modules")
        }
        fn root_str(&self) -> String {
            norm_posix(&self.root().to_string_lossy())
        }
        fn domains(&self, body: &str) {
            std::fs::write(self.root().join(DOMAINS_REL), body).unwrap();
        }
        fn other(&self, name: &str, body: &str) {
            std::fs::write(self.root().join("knowledge").join(name), body).unwrap();
        }
        fn note(&self, name: &str) {
            std::fs::write(self.corpus().join(name), "# notes\n").unwrap();
        }
        fn run(&self) -> Outcome {
            evaluate(&self.root_str()).expect("evaluate")
        }
    }

    fn good_tree() -> Tree {
        let t = Tree::new();
        t.note("01-a.md");
        t.note("02-b.md");
        t.domains(
            "[[domain]]\nid = \"01-a\"\nprimary_notes = \"../modules/01-a.md\"\n\n\
             [[domain]]\nid = \"02-b\"\nprimary_notes = \"../modules/02-b.md\"\n",
        );
        t
    }

    #[test]
    fn clean_tree_is_green_and_counts_what_it_checked() {
        let out = good_tree().run();
        assert_eq!(out.code, 0, "{}", out.stdout);
        assert!(out.stdout.starts_with("PASS\n"), "{}", out.stdout);
        assert!(
            out.stdout.contains("  primary_notes_checked=2\n"),
            "{}",
            out.stdout
        );
        assert!(out.stdout.contains("  empty_allowed=0\n"), "{}", out.stdout);
    }

    #[test]
    fn a_missing_target_is_red_and_prints_both_forms() {
        let t = good_tree();
        t.domains("[[domain]]\nid = \"01-a\"\nprimary_notes = \"../modules/gone.md\"\n");
        let out = t.run();
        assert_eq!(out.code, 1);
        assert!(
            out.stdout.contains(
                "01-a: primary_notes does not resolve to a file: '../modules/gone.md' (resolved "
            ),
            "{}",
            out.stdout
        );
        // The resolved form is `..`-free, which is the point of emulating resolve().
        assert!(!out.stdout.contains("/.."), "{}", out.stdout);
    }

    #[test]
    fn a_missing_domains_toml_is_its_own_one_line_fail() {
        let t = Tree::new();
        let out = t.run();
        assert_eq!(out.code, 1);
        assert_eq!(out.stdout, "FAIL: knowledge/domains.toml missing\n");
    }

    #[test]
    fn zero_domain_rows_is_an_error_not_a_pass() {
        let t = Tree::new();
        t.domains("schema_version = 1\n");
        let out = t.run();
        assert_eq!(out.code, 1);
        assert_eq!(out.stdout, "FAIL: domains.toml has zero [[domain]] rows\n");
    }

    #[test]
    fn an_empty_domain_array_is_also_zero_rows() {
        let t = Tree::new();
        t.domains("domain = []\n");
        let out = t.run();
        assert_eq!(out.code, 1);
        assert_eq!(out.stdout, "FAIL: domains.toml has zero [[domain]] rows\n");
    }

    #[test]
    fn a_missing_primary_notes_key_is_red() {
        let t = Tree::new();
        t.domains("[[domain]]\nid = \"01-a\"\n");
        let out = t.run();
        assert_eq!(out.code, 1);
        assert!(
            out.stdout
                .contains("  - 01-a: primary_notes field missing\n"),
            "{}",
            out.stdout
        );
    }

    #[test]
    fn an_empty_primary_notes_needs_the_exam_weight_licence() {
        let t = Tree::new();
        t.domains("[[domain]]\nid = \"01-a\"\nprimary_notes = \"   \"\n");
        let out = t.run();
        assert_eq!(out.code, 1);
        assert!(
            out.stdout
                .contains("  - 01-a: empty primary_notes without exam_weight_unknown=true\n"),
            "{}",
            out.stdout
        );
    }

    #[test]
    fn the_licence_must_be_boolean_true_not_merely_truthy() {
        let t = Tree::new();
        t.domains("[[domain]]\nid = \"01-a\"\nprimary_notes = \"\"\nexam_weight_unknown = 1\n");
        let out = t.run();
        assert_eq!(out.code, 1, "`is True` in the oracle rejects 1");
    }

    #[test]
    fn a_licensed_empty_is_counted_not_reported() {
        let t = good_tree();
        t.domains(
            "[[domain]]\nid = \"01-a\"\nprimary_notes = \"../modules/01-a.md\"\n\n\
             [[domain]]\nid = \"15-ops\"\nprimary_notes = \"\"\nexam_weight_unknown = true\n",
        );
        let out = t.run();
        assert_eq!(out.code, 0, "{}", out.stdout);
        assert!(
            out.stdout.contains("  primary_notes_checked=1\n"),
            "{}",
            out.stdout
        );
        assert!(out.stdout.contains("  empty_allowed=1\n"), "{}", out.stdout);
    }

    #[test]
    fn a_note_under_the_engines_own_modules_dir_is_the_wrong_tree() {
        let t = Tree::new();
        std::fs::create_dir_all(t.root().join("modules")).unwrap();
        std::fs::write(t.root().join("modules/01-a.md"), "# stray\n").unwrap();
        t.domains("[[domain]]\nid = \"01-a\"\nprimary_notes = \"modules/01-a.md\"\n");
        let out = t.run();
        assert_eq!(out.code, 1);
        assert!(
            out.stdout
                .contains("01-a: primary_notes resolves under course-engine/modules/ ("),
            "{}",
            out.stdout
        );
        assert!(
            out.stdout
                .contains("); parent corpus is ../modules/ relative to ROOT\n"),
            "{}",
            out.stdout
        );
        // It resolved to a real file, so it still counted as checked.
    }

    #[test]
    fn a_row_with_no_id_reports_as_missing_id() {
        let t = Tree::new();
        t.domains("[[domain]]\nprimary_notes = \"../modules/gone.md\"\n");
        let out = t.run();
        assert_eq!(out.code, 1);
        assert!(out.stdout.contains("  - <missing-id>: "), "{}", out.stdout);
    }

    #[test]
    fn a_non_string_primary_notes_is_stringified_not_skipped() {
        let t = Tree::new();
        t.domains("[[domain]]\nid = \"01-a\"\nprimary_notes = 123\n");
        let out = t.run();
        assert_eq!(out.code, 1);
        assert!(
            out.stdout.contains("does not resolve to a file: '123' "),
            "{}",
            out.stdout
        );
    }

    // ── second leg: the line scan over other knowledge/*.toml ─────────────

    #[test]
    fn other_toml_files_are_line_scanned_with_line_numbers() {
        let t = good_tree();
        t.other(
            "topics.toml",
            "# header\n\n[[topic]]\nprimary_notes = \"../modules/nope.md\"\n",
        );
        let out = t.run();
        assert_eq!(out.code, 1);
        assert!(
            out.stdout.contains(
                "  - topics.toml:4: primary_notes '../modules/nope.md' does not resolve\n"
            ),
            "{}",
            out.stdout
        );
    }

    #[test]
    fn the_line_scan_is_prefix_matched_and_quote_stripped_exactly() {
        let t = good_tree();
        // `startswith` also fires on a longer key; a trailing comment stays in the
        // value because only ONE leading/trailing quote layer is stripped.
        t.other(
            "a.toml",
            "primary_notes_extra = \"../modules/nope.md\"\nprimary_notes = '../modules/01-a.md'\nprimary_notes\nprimary_notes = \"../modules/01-a.md\"  # ok?\n",
        );
        let out = t.run();
        assert_eq!(out.code, 1);
        assert!(
            out.stdout
                .contains("a.toml:1: primary_notes '../modules/nope.md' does not resolve"),
            "{}",
            out.stdout
        );
        // Line 2 is single-quoted and resolves; line 3 has no `=`; line 4 keeps the
        // trailing comment in the value and therefore does not resolve.
        assert!(
            out.stdout
                .contains("a.toml:4: primary_notes '../modules/01-a.md\"  # ok?' does not resolve"),
            "{}",
            out.stdout
        );
        assert_eq!(
            out.stdout.lines().filter(|l| l.starts_with("  - ")).count(),
            2,
            "{}",
            out.stdout
        );
    }

    #[test]
    fn domains_toml_is_not_line_scanned_twice() {
        let t = good_tree();
        // domains.toml is skipped by name in the second leg; a resolving tree stays
        // green even though its lines start with `primary_notes`.
        assert_eq!(t.run().code, 0);
    }

    #[test]
    fn second_leg_findings_come_in_sorted_file_order() {
        let t = good_tree();
        t.other("zz.toml", "primary_notes = \"../modules/z.md\"\n");
        t.other("aa.toml", "primary_notes = \"../modules/a.md\"\n");
        let out = t.run();
        assert_eq!(out.code, 1);
        let first = out.stdout.find("aa.toml:1").expect("aa reported");
        let second = out.stdout.find("zz.toml:1").expect("zz reported");
        assert!(
            first < second,
            "sorted() order not reproduced:\n{}",
            out.stdout
        );
    }

    #[test]
    fn non_toml_neighbours_are_not_scanned() {
        let t = good_tree();
        t.other("notes.md", "primary_notes = \"../modules/nope.md\"\n");
        t.other("case.TOML", "primary_notes = \"../modules/nope.md\"\n");
        assert_eq!(t.run().code, 0);
    }

    // ── the inherited vacuity holes, pinned so a silent change is loud ────

    #[test]
    fn all_rows_licensed_empty_passes_with_zero_checked_which_is_the_oracles_hole() {
        let t = Tree::new();
        t.domains("[[domain]]\nid = \"x\"\nprimary_notes = \"\"\nexam_weight_unknown = true\n");
        let out = t.run();
        assert_eq!(
            out.code, 0,
            "the oracle passes here; the port must too, and the hole is reported, not fixed"
        );
        assert!(
            out.stdout.contains("  primary_notes_checked=0\n"),
            "{}",
            out.stdout
        );
    }

    #[test]
    fn zero_other_knowledge_files_is_silent_which_is_the_oracles_other_hole() {
        let t = good_tree();
        // Only domains.toml exists; the second leg scans nothing and says nothing.
        assert_eq!(t.run().code, 0);
    }

    // ── crash-shaped inputs are ERROR, never OK ───────────────────────────

    #[test]
    fn malformed_domains_toml_is_an_error_not_a_pass() {
        let t = Tree::new();
        t.domains("[[domain]\nid = \"broken\"\n");
        let err = evaluate(&t.root_str()).unwrap_err();
        assert_eq!(err.code(), crate::exit::ERROR);
    }

    #[test]
    fn a_domain_key_that_is_not_an_array_is_an_error() {
        let t = Tree::new();
        t.domains("domain = \"nope\"\n");
        let err = evaluate(&t.root_str()).unwrap_err();
        assert_eq!(err.code(), crate::exit::ERROR);
    }

    // ── argv ──────────────────────────────────────────────────────────────

    #[test]
    fn any_argument_is_usage_never_a_silent_full_tree_run() {
        let ctx = GateCtx::new(PathBuf::from("/"), vec!["--staged".into()]);
        assert_eq!(run(&ctx).unwrap_err().code(), crate::exit::USAGE);
    }

    // ── the header's own honesty ──────────────────────────────────────────

    #[test]
    fn header_states_a_floor_raise_and_overclaims_nothing() {
        let src = include_str!("verify_knowledge_paths.rs");
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
        assert!(
            header.contains("bd-lt7"),
            "header must record the bd-lt7 module-bound finding"
        );
        for banned in ["guarantee", "proves", "makes impossible", "impossible"] {
            assert!(
                !header.to_lowercase().contains(banned),
                "header overclaims with {banned:?}"
            );
        }
    }
}
