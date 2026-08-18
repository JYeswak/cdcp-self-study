//! Shell script invocation walking.
//!
//! Extracts the transitive set of scripts that `scripts/check.sh` invokes,
//! following `source`, `.`, `sh`, `bash`, `python3`, etc. Used by
//! substrate-guard to verify that reason claims match actual invocations.
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

/// Executors whose next token is treated as an invoked path.
///
/// Longer names first so `python3` is not read as `python`, `nodejs` is not
/// read as `node`, and `bash`/`zsh` are not read as `sh`.
pub const INVOKE_EXECS: &[&str] = &["python3", "python", "nodejs", "node", "bash", "zsh", "sh"];

/// Everything before an unquoted `#` — the part of the line the shell executes.
pub fn code_part(line: &str) -> &str {
    let b = line.as_bytes();
    let mut in_single = false;
    let mut in_double = false;
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'\\' if !in_single => i += 1,
            b'\'' if !in_double => in_single = !in_single,
            b'"' if !in_single => in_double = !in_double,
            b'#' if !in_single && !in_double && (i == 0 || b[i - 1].is_ascii_whitespace()) => {
                return &line[..i];
            }
            _ => {}
        }
        i += 1;
    }
    line
}

pub fn is_word_start(text: &str, i: usize) -> bool {
    if i == 0 {
        return true;
    }
    let b = text.as_bytes()[i - 1];
    // `.` is not a word start: `voice-slop.sh >/dev/null` must not be
    // parsed as an `sh` invoke of `>/dev/null`.
    !b.is_ascii_alphanumeric() && b != b'_' && b != b'.'
}

pub fn next_shell_token(after: &str) -> Option<&str> {
    let s = after.trim_start();
    if s.is_empty() {
        return None;
    }
    if let Some(q) = s.chars().next().filter(|c| *c == '"' || *c == '\'') {
        let rest = &s[q.len_utf8()..];
        let end = rest.find(q)?;
        Some(&rest[..end])
    } else {
        let end = s
            .find(|c: char| c.is_ascii_whitespace() || matches!(c, ';' | '|' | '&' | ')'))
            .unwrap_or(s.len());
        Some(&s[..end])
    }
}

/// What `scripts/check.sh` transitively reaches.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InvocationWalk {
    /// Repo-relative script paths (`scripts/foo.py`, `tests/voice-slop.sh`, …).
    pub paths: BTreeSet<String>,
    /// `[ -f path ]` / `test -f path` targets. Distinct from invoke.
    pub presence: BTreeSet<String>,
    /// `cargo run -p <pkg> -- <cmd>` records. Not followed into Rust.
    pub cargo: BTreeSet<String>,
    /// Shell files whose bodies were opened. Cycle-breaking, not an inventory.
    pub followed: BTreeSet<String>,
}

impl InvocationWalk {
    pub fn is_empty(&self) -> bool {
        self.paths.is_empty() && self.cargo.is_empty()
    }

    pub fn python(&self) -> Vec<&str> {
        self.paths
            .iter()
            .filter(|p| p.ends_with(".py"))
            .map(String::as_str)
            .collect()
    }
}

/// Empty inventory is an ERROR, not a pass. Check.sh always runs something.
pub fn require_nonempty_inventory(walk: &InvocationWalk) -> Result<(), String> {
    if walk.is_empty() {
        return Err(
            "transitive invocation inventory is empty — a scan that found nothing is an ERROR, not a pass"
                .into(),
        );
    }
    Ok(())
}

/// Floor derived from the tree: at least one walked path (invoke or presence)
/// must exist on disk. A walk whose every target is a ghost is an ERROR.
pub fn require_tree_derived_floor(
    walk: &InvocationWalk,
    exists: impl Fn(&str) -> bool,
) -> Result<usize, String> {
    let n = walk
        .paths
        .iter()
        .chain(walk.presence.iter())
        .filter(|p| exists(p))
        .count();
    if n == 0 {
        return Err(
            "tree-derived invocation/presence floor is 0 — a walk whose targets do not exist on disk is an ERROR, not a pass"
                .into(),
        );
    }
    Ok(n)
}

fn is_followable_shell(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.ends_with(".sh") || lower.ends_with(".bash") || lower.ends_with(".zsh")
}

pub fn normalize_repo_path(tok: &str) -> Option<String> {
    let t = tok.trim();
    let t = t.strip_prefix("./").unwrap_or(t);
    let t = t
        .strip_prefix("$ROOT/")
        .or_else(|| t.strip_prefix("${ROOT}/"))
        .unwrap_or(t);
    if t.is_empty() || t.starts_with('$') || t.starts_with('/') || t.contains("..") {
        return None;
    }
    if !(t.contains('/')
        || t.ends_with(".py")
        || t.ends_with(".sh")
        || t.ends_with(".mjs")
        || t.ends_with(".js")
        || t.ends_with(".bash")
        || t.ends_with(".zsh"))
    {
        return None;
    }
    Some(t.to_string())
}

fn var_name(tok: &str) -> Option<&str> {
    let t = tok
        .strip_prefix("${")
        .and_then(|s| s.strip_suffix('}'))
        .or_else(|| tok.strip_prefix('$'))?;
    if !t.is_empty() && t.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        Some(t)
    } else {
        None
    }
}

fn resolve_invoke_token(tok: &str, vars: &BTreeMap<String, BTreeSet<String>>) -> Vec<String> {
    if let Some(name) = var_name(tok) {
        return vars
            .get(name)
            .into_iter()
            .flatten()
            .filter_map(|v| normalize_repo_path(v))
            .collect();
    }
    normalize_repo_path(tok).into_iter().collect()
}

pub fn collect_assignments(text: &str) -> BTreeMap<String, BTreeSet<String>> {
    let mut vars: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.starts_with('#') {
            continue;
        }
        let code = code_part(line);
        for (i, _) in code.char_indices() {
            if !is_word_start(code, i) {
                continue;
            }
            let rest = &code[i..];
            let Some(eq) = rest.find('=') else {
                continue;
            };
            let name = &rest[..eq];
            if name.is_empty()
                || !name
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
                || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            {
                continue;
            }
            let after = &rest[eq + 1..];
            if after.starts_with('$') || after.starts_with('`') {
                continue;
            }
            if let Some(val) = next_shell_token(after) {
                if !val.is_empty() && !val.contains('$') && !val.contains('`') {
                    vars.entry(name.to_string())
                        .or_default()
                        .insert(val.to_string());
                }
            }
        }
    }
    vars
}

fn collect_sourced_tokens(code: &str, out: &mut Vec<String>) {
    for (i, _) in code.char_indices() {
        if !is_word_start(code, i) {
            continue;
        }
        let rest = &code[i..];
        let after = rest
            .strip_prefix("source")
            .or_else(|| rest.strip_prefix('.'))
            .filter(|tail| tail.chars().next().is_some_and(|c| c.is_ascii_whitespace()));
        if let Some(tail) = after {
            if let Some(tok) = next_shell_token(tail) {
                if !tok.is_empty() && !tok.starts_with('-') {
                    out.push(tok.to_string());
                }
            }
        }
    }
}

fn collect_exec_tokens(code: &str, out: &mut Vec<String>) {
    for (i, _) in code.char_indices() {
        if !is_word_start(code, i) {
            continue;
        }
        let rest = &code[i..];
        let Some(exec) = INVOKE_EXECS.iter().copied().find(|e| rest.starts_with(e)) else {
            continue;
        };
        let after = &rest[exec.len()..];
        let boundary = match after.chars().next() {
            None => true,
            Some(c) => c.is_ascii_whitespace() || c == '"' || c == '\'',
        };
        if !boundary {
            continue;
        }
        if let Some(tok) = next_shell_token(after) {
            let tok = tok.trim();
            if !tok.is_empty()
                && !tok.starts_with('-')
                && !tok.starts_with('>')
                && !tok.starts_with('<')
            {
                out.push(tok.to_string());
            }
        }
    }
}

fn flag_value<'a>(s: &'a str, flag: &str) -> Option<&'a str> {
    let mut search = s;
    loop {
        let i = search.find(flag)?;
        if !is_word_start(search, i) {
            search = &search[i + flag.len()..];
            continue;
        }
        let after = &search[i + flag.len()..];
        if !after
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_whitespace())
        {
            search = &search[i + flag.len()..];
            continue;
        }
        return next_shell_token(after);
    }
}

fn extract_cargo_runs(code: &str, out: &mut BTreeSet<String>) {
    for (i, _) in code.char_indices() {
        if !is_word_start(code, i) {
            continue;
        }
        let rest = &code[i..];
        if !rest.starts_with("cargo") {
            continue;
        }
        let after = &rest["cargo".len()..];
        if !after
            .chars()
            .next()
            .map(|c| c.is_ascii_whitespace())
            .unwrap_or(true)
        {
            continue;
        }
        if !after.split_whitespace().any(|w| w == "run") {
            continue;
        }
        let pkg = flag_value(after, "-p").unwrap_or("");
        let cmd = after
            .split_once(" -- ")
            .and_then(|(_, c)| c.split_whitespace().next())
            .unwrap_or("");
        let rec = match (pkg.is_empty(), cmd.is_empty()) {
            (false, false) => format!("cargo run -p {pkg} -- {cmd}"),
            (false, true) => format!("cargo run -p {pkg}"),
            (true, false) => format!("cargo run -- {cmd}"),
            (true, true) => "cargo run".into(),
        };
        out.insert(rec);
    }
}

fn preceding_token(code: &str, at: usize) -> &str {
    let before = code[..at].trim_end();
    let bytes = before.as_bytes();
    let mut i = bytes.len();
    while i > 0 {
        let c = bytes[i - 1] as char;
        if c.is_ascii_whitespace() || matches!(c, ';' | '|' | '&' | '(' | ')' | '`') {
            break;
        }
        i -= 1;
    }
    &before[i..]
}

/// `-f` is a presence test only inside `[` / `[[` / `test`. `rm -f` / `git add -f`
/// are force flags and must not mint a presence row.
fn collect_presence_tokens(code: &str, out: &mut Vec<String>) {
    for (i, _) in code.char_indices() {
        if !is_word_start(code, i) || !code[i..].starts_with("-f") {
            continue;
        }
        let after = &code[i + 2..];
        let boundary = match after.chars().next() {
            None => true,
            Some(c) => c.is_ascii_whitespace() || c == '"' || c == '\'',
        };
        if !boundary {
            continue;
        }
        if !matches!(
            preceding_token(code, i),
            "[" | "[[" | "test" | "!" | "-a" | "-o"
        ) {
            continue;
        }
        if let Some(tok) = next_shell_token(after) {
            let tok = tok.trim();
            if !tok.is_empty() && tok != "]" && tok != "]]" && !tok.starts_with('-') {
                out.push(tok.to_string());
            }
        }
    }
}

/// Derive the transitive invocation set from `entry_text` (`scripts/check.sh`).
///
/// `read` opens a child script. An invoked followable shell that cannot be
/// read is an ERROR — an incomplete walk must not report like a complete one.
/// Empty walk is `Ok` (fixtures); `require_nonempty_inventory` is the live gate.
pub fn walk_invocations(
    entry_text: &str,
    mut read: impl FnMut(&str) -> Option<String>,
) -> Result<InvocationWalk, String> {
    let mut walk = InvocationWalk::default();
    let mut queue: Vec<(String, String)> =
        vec![("scripts/check.sh".into(), entry_text.to_string())];
    let mut seen = BTreeSet::new();

    while let Some((from, text)) = queue.pop() {
        if !seen.insert(from.clone()) {
            continue;
        }
        walk.followed.insert(from.clone());
        let vars = collect_assignments(&text);
        for raw in text.lines() {
            let line = raw.trim();
            if line.starts_with('#') {
                continue;
            }
            let code = code_part(line);
            extract_cargo_runs(code, &mut walk.cargo);
            let mut tokens = Vec::new();
            collect_exec_tokens(code, &mut tokens);
            collect_sourced_tokens(code, &mut tokens);
            for tok in tokens {
                for path in resolve_invoke_token(&tok, &vars) {
                    walk.paths.insert(path.clone());
                    if is_followable_shell(&path) && !seen.contains(&path) {
                        match read(&path) {
                            Some(body) => queue.push((path, body)),
                            None => {
                                return Err(format!(
                                    "invoked {path} from {from} but could not read it — transitive inventory is incomplete. ERROR, not a pass"
                                ));
                            }
                        }
                    }
                }
            }
            let mut pres = Vec::new();
            collect_presence_tokens(code, &mut pres);
            for tok in pres {
                for path in resolve_invoke_token(&tok, &vars) {
                    walk.presence.insert(path);
                }
            }
        }
    }
    Ok(walk)
}

/// Non-comment body still names `python3 <path>` or assigns that path to a
/// variable later used as `python3 "$VAR"`. Used by the live tripwire so a
/// gna0 deletion of the call does not hard-fail this bead.
pub fn script_still_invokes_py(text: &str, path: &str) -> bool {
    let vars = collect_assignments(text);
    let names: BTreeSet<&str> = vars
        .iter()
        .filter(|(_, vs)| {
            vs.iter()
                .any(|v| v == path || normalize_repo_path(v).as_deref() == Some(path))
        })
        .map(|(k, _)| k.as_str())
        .collect();
    for raw in text.lines() {
        let line = raw.trim();
        if line.starts_with('#') {
            continue;
        }
        let code = code_part(line);
        let mut tokens = Vec::new();
        collect_exec_tokens(code, &mut tokens);
        for tok in tokens {
            if normalize_repo_path(&tok).as_deref() == Some(path) {
                return true;
            }
            if let Some(name) = var_name(&tok) {
                if names.contains(name) {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_part_strips_trailing_comment() {
        assert_eq!(code_part("true # cargo run"), "true ");
        assert_eq!(code_part("echo 'a#b' # tail"), "echo 'a#b' ");
    }

    #[test]
    fn walk_empty_is_ok() {
        let w = walk_invocations("#!/bin/sh\ntrue\n", |_| None).expect("walk");
        assert!(w.paths.is_empty());
    }

    #[test]
    fn walk_missing_child_is_error() {
        let e = walk_invocations("#!/bin/sh\nsh scripts/missing.sh\n", |_| None).unwrap_err();
        assert!(e.contains("missing.sh"), "{e}");
    }

    #[test]
    fn walk_finds_python() {
        let walk = walk_invocations("#!/bin/sh\npython3 scripts/foo.py\n", |_| None).expect("walk");
        assert!(walk.paths.contains("scripts/foo.py"));
    }

    #[test]
    fn walk_finds_cargo_run() {
        let walk = walk_invocations("#!/bin/sh\ncargo run -p cdcp_gate -- substrate-guard\n", |_| None).expect("walk");
        assert!(walk.cargo.contains("cargo run -p cdcp_gate -- substrate-guard"));
    }

    #[test]
    fn presence_test_recognized() {
        let walk = walk_invocations("#!/bin/sh\n[ -f scripts/check.py ] && echo exists\n", |_| None).expect("walk");
        assert!(walk.presence.contains("scripts/check.py"));
    }

    #[test]
    fn script_still_invokes_py_direct() {
        assert!(script_still_invokes_py("#!/bin/sh\npython3 scripts/foo.py\n", "scripts/foo.py"));
        assert!(!script_still_invokes_py("#!/bin/sh\npython3 scripts/bar.py\n", "scripts/foo.py"));
    }
}
