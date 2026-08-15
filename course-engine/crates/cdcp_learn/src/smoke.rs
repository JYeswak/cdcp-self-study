//! smoke_learn — the offline Learn surface the learner actually opens.
//!
//! Extracted from `scripts/smoke_learn.py` by bd-substrate-rust-migration-jhd.17
//! as product, not a gate file. A learner walks `web/learn.html` and the
//! per-module pages. If they can see it, it is not a `cdcp_gate` concern.
//!
//! # Contract
//!
//! Every domain `knowledge/domains.toml` declares has a real `primary_notes`
//! file (or is an empty-ok row with `exam_weight_unknown = true`), the Learn
//! index agrees with that id set, every navigable module has a relative
//! `learn/{id}.html` page with the non-grant honesty banner and relative
//! assets, and the hub lists those modules without linking empty-ok ids.
//!
//! # Empty input is an ERROR
//!
//! The retired Python PASSED a registry whose every row was empty-ok
//! (`primary_notes_checked=0`, `PASS: smoke_learn`). That hole is
//! `bd-smoke-learn-vacuous-empty-ok-9d3n`. This smoke closes it: zero
//! resolved notes, zero domain rows, or zero navigable modules is RED.
//!
//! # What this cannot decide
//!
//! It does not open a browser, fetch assets, or read prose. A page whose
//! banner is present and whose body is a stub clears the same as a correct
//! page. A content copy that exists and is ≥ [`MIN_CONTENT_BYTES`] is not
//! checked for being the right module. Completeness of the corpus (a notes
//! file with no `[[domain]]` row) is invisible here.

#![forbid(unsafe_code)]

use crate::{join_rel, BuildOutcome};
use serde_json::Value as Json;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};
use toml::Value as Toml;

pub const NAME: &str = "smoke-learn";
pub const SUMMARY: &str =
    "L5 Learn surface: every declared module has a relative, honest, on-disk page";

pub const DOMAINS_REL: &str = "knowledge/domains.toml";
pub const INDEX_REL: &str = "web/data/modules_index.json";
pub const LEARN_HUB_REL: &str = "web/learn.html";
pub const LEARN_DIR_REL: &str = "web/learn";
pub const CONTENT_DIR_REL: &str = "web/content/modules";

/// Byte floor a shipped content copy must clear. The retired script used
/// `st_size < 32`. A byte count on one file, not a module bound.
pub const MIN_CONTENT_BYTES: u64 = 32;

/// Run the Learn-surface smoke against `root` (the course-engine directory).
///
/// This is a reader: it writes nothing. `BuildOutcome.artifact` is always
/// `None`. `code != 0` is RED.
pub fn run(root: &Path) -> BuildOutcome {
    let domains_path = join_rel(root, DOMAINS_REL);
    if !domains_path.is_file() {
        return outcome(1, "FAIL: knowledge/domains.toml missing\n");
    }

    let text = match std::fs::read_to_string(&domains_path) {
        Ok(t) => t,
        Err(e) => {
            return fail(vec![format!("knowledge/domains.toml unreadable: {e}")]);
        }
    };
    let parsed: Toml = match text.parse() {
        Ok(v) => v,
        Err(e) => {
            return fail(vec![format!("knowledge/domains.toml invalid TOML: {e}")]);
        }
    };

    let mut errors: Vec<String> = Vec::new();
    let domains = match parsed.get("domain") {
        Some(Toml::Array(rows)) => rows.clone(),
        None => Vec::new(),
        Some(_) => {
            errors.push("domains.toml `domain` is not an array of tables".into());
            Vec::new()
        }
    };
    if domains.is_empty() && errors.is_empty() {
        errors.push("domains.toml has zero [[domain]] rows".into());
    }

    let mut checked = 0usize;
    let mut empty_ok = 0usize;
    let mut domain_by_id: BTreeMap<String, Toml> = BTreeMap::new();
    for dom in &domains {
        let Some(table) = dom.as_table() else {
            errors.push("domains.toml has a [[domain]] row that is not a table".into());
            continue;
        };
        let did = table
            .get("id")
            .map(toml_as_string)
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "<missing-id>".to_string());
        domain_by_id.insert(did.clone(), dom.clone());
        match table.get("primary_notes") {
            None => {
                errors.push(format!("{did}: primary_notes field missing"));
            }
            Some(pn_v) => {
                let pn_s = toml_as_string(pn_v);
                let pn_stripped = pn_s.trim();
                if pn_stripped.is_empty() {
                    if matches!(table.get("exam_weight_unknown"), Some(Toml::Boolean(true))) {
                        empty_ok += 1;
                    } else {
                        errors.push(format!(
                            "{did}: empty primary_notes without exam_weight_unknown=true"
                        ));
                    }
                    continue;
                }
                let candidate = resolve_notes(root, pn_stripped);
                checked += 1;
                if !candidate.is_file() {
                    errors.push(format!(
                        "{did}: primary_notes does not resolve: {pn_stripped} → {}",
                        candidate.display()
                    ));
                }
            }
        }
    }

    // Zero resolved notes is the inherited vacuous-PASS hole. Closed here.
    if checked == 0 {
        errors.push(
            "empty input: zero primary_notes resolved — a scan that checked nothing is an ERROR, not a pass"
                .into(),
        );
    }

    let index_path = join_rel(root, INDEX_REL);
    let index: Option<Json> = if !index_path.is_file() {
        errors.push(format!("missing {INDEX_REL} — run `cdcp build-learn`"));
        None
    } else {
        match std::fs::read_to_string(&index_path) {
            Ok(body) => match serde_json::from_str::<Json>(&body) {
                Ok(v) => Some(v),
                Err(e) => {
                    errors.push(format!("modules_index.json invalid JSON: {e}"));
                    None
                }
            },
            Err(e) => {
                errors.push(format!("modules_index.json unreadable: {e}"));
                None
            }
        }
    };

    let hub_path = join_rel(root, LEARN_HUB_REL);
    if !hub_path.is_file() {
        errors.push("missing web/learn.html".into());
    }

    let mut index_module_count: Option<usize> = None;
    let mut navigable_reported: Option<Json> = None;

    if let Some(index) = index.as_ref() {
        let mods = index
            .get("modules")
            .and_then(Json::as_array)
            .cloned()
            .unwrap_or_default();
        index_module_count = Some(mods.len());
        navigable_reported = index.get("navigable_count").cloned();
        if mods.is_empty() {
            errors.push("modules_index.json has zero modules".into());
        }

        let index_ids: BTreeSet<String> = mods.iter().map(|m| json_id(m)).collect();
        let domain_ids: BTreeSet<String> = domain_by_id.keys().cloned().collect();
        if index_ids != domain_ids {
            let missing: Vec<&String> = domain_ids.difference(&index_ids).collect();
            let extra: Vec<&String> = index_ids.difference(&domain_ids).collect();
            if !missing.is_empty() {
                errors.push(format!("index missing domain ids: {}", join_ids(&missing)));
            }
            if !extra.is_empty() {
                errors.push(format!(
                    "index has unknown domain ids: {}",
                    join_ids(&extra)
                ));
            }
        }

        let mut navigable_n = 0usize;
        for m in &mods {
            let mid = json_id(m);
            let empty = matches!(m.get("empty"), Some(Json::Bool(true)));
            if empty {
                if let Some(h) = json_href(m) {
                    if !h.is_empty() {
                        errors.push(format!(
                            "{mid}: empty-ok domain must not have href (got {h})"
                        ));
                    }
                }
                continue;
            }
            navigable_n += 1;

            let href = json_href(m).unwrap_or_default();
            if href.is_empty() {
                errors.push(format!("{mid}: navigable module missing href"));
            } else if href.starts_with("http://")
                || href.starts_with("https://")
                || href.starts_with('/')
            {
                errors.push(format!(
                    "{mid}: href must be relative offline path, got {href}"
                ));
            } else if !href.starts_with("learn/") || !href.ends_with(".html") {
                errors.push(format!("{mid}: unexpected href shape {href}"));
            }

            let page = join_rel(root, LEARN_DIR_REL).join(format!("{mid}.html"));
            if !page.is_file() {
                errors.push(format!(
                    "{mid}: missing learn page {LEARN_DIR_REL}/{mid}.html"
                ));
            } else {
                match std::fs::read_to_string(&page) {
                    Ok(text) => check_page(&mid, &text, &mut errors),
                    Err(e) => errors.push(format!("{mid}: learn page unreadable: {e}")),
                }
            }

            let content = join_rel(root, CONTENT_DIR_REL).join(format!("{mid}.md"));
            if content.is_file() {
                match content.metadata() {
                    Ok(meta) if meta.len() < MIN_CONTENT_BYTES => {
                        errors.push(format!(
                            "{mid}: content copy is empty/tiny: {CONTENT_DIR_REL}/{mid}.md"
                        ));
                    }
                    Ok(_) => {}
                    Err(e) => errors.push(format!("{mid}: content copy unreadable: {e}")),
                }
            } else {
                let pn = domain_by_id
                    .get(&mid)
                    .and_then(Toml::as_table)
                    .and_then(|t| t.get("primary_notes"))
                    .map(toml_as_string)
                    .unwrap_or_default();
                let pn = pn.trim();
                let src_ok = !pn.is_empty() && resolve_notes(root, pn).is_file();
                if !src_ok {
                    errors.push(format!(
                        "{mid}: missing content copy and primary_notes source \
                         (run `cdcp build-learn`)"
                    ));
                }
            }
        }

        if !mods.is_empty() && navigable_n == 0 {
            errors.push(
                "modules_index.json has zero navigable modules — an empty Learn surface is an ERROR"
                    .into(),
            );
        }
    }

    if hub_path.is_file() {
        match std::fs::read_to_string(&hub_path) {
            Ok(hub) => check_hub(&hub, index.as_ref(), &domain_by_id, &mut errors),
            Err(e) => errors.push(format!("web/learn.html unreadable: {e}")),
        }
    }

    if !errors.is_empty() {
        return fail(errors);
    }

    let mut out = String::from("PASS: smoke_learn\n");
    out.push_str(&format!("  primary_notes_checked={checked}\n"));
    out.push_str(&format!("  empty_allowed={empty_ok}\n"));
    if let Some(n) = index_module_count {
        out.push_str(&format!("  index_modules={n}\n"));
        match navigable_reported {
            Some(v) => out.push_str(&format!("  navigable={v}\n")),
            None => out.push_str("  navigable=none\n"),
        }
    }
    out.push_str(&format!("  hub={LEARN_HUB_REL}\n"));
    outcome(0, out)
}

fn check_page(mid: &str, text: &str, errors: &mut Vec<String>) {
    if !has_honesty_banner(text) {
        errors.push(format!(
            "{mid}: learn page missing honesty non-grant banner"
        ));
    }
    if !has_quoted_attr(text, "href", "../assets/css/course.css") {
        errors.push(format!(
            "{mid}: learn page css must be relative ../assets/css/course.css"
        ));
    }
    if !text.contains("src=\"../assets/js/learn_progress.js\"") {
        errors.push(format!(
            "{mid}: learn page must load relative learn_progress.js"
        ));
    }
    if !text.contains("src=\"../assets/js/learn_md.js\"") {
        errors.push(format!("{mid}: learn page must load relative learn_md.js"));
    }
    let has_reader = text.contains("src=\"../assets/js/learn_reader.js\"");
    let has_embed = text.contains("id=\"module-md\"");
    if !has_reader && !has_embed {
        errors.push(format!(
            "{mid}: learn page must load learn_reader.js or embed #module-md"
        ));
    }
    let has_fetch = text.contains(&format!("content/modules/{mid}.md"));
    if !has_embed && !has_fetch {
        errors.push(format!(
            "{mid}: learn page must embed #module-md or fetch content/modules/{mid}.md"
        ));
    }
}

fn check_hub(
    hub: &str,
    index: Option<&Json>,
    domain_by_id: &BTreeMap<String, Toml>,
    errors: &mut Vec<String>,
) {
    if !has_honesty_banner(hub) {
        errors.push("web/learn.html missing honesty non-grant banner".into());
    }
    if !has_quoted_attr(hub, "href", "assets/css/course.css") {
        errors.push("web/learn.html css must be relative assets/css/course.css".into());
    }

    let empty_ids: BTreeSet<String> = index
        .and_then(|i| i.get("modules"))
        .and_then(Json::as_array)
        .into_iter()
        .flatten()
        .filter(|m| matches!(m.get("empty"), Some(Json::Bool(true))))
        .map(json_id)
        .collect();
    for eid in &empty_ids {
        let target = format!("learn/{eid}.html");
        if has_quoted_attr(hub, "href", &target) {
            errors.push(format!("hub must not link to empty-ok module page {eid}"));
        }
    }

    for (mid, dom) in domain_by_id {
        let pn = dom
            .as_table()
            .and_then(|t| t.get("primary_notes"))
            .map(toml_as_string)
            .unwrap_or_default();
        if pn.trim().is_empty() {
            continue;
        }
        let listed = hub.contains(&format!("learn/{mid}.html"))
            || hub.contains(&format!("data-module-id=\"{mid}\""));
        if !listed {
            errors.push(format!("hub does not list navigable module {mid}"));
        }
    }
}

/// `does … not … grant EPI/EXIN certification`, case-insensitive.
/// Matches the retired script's `HONESTY_RE` including the
/// `does <strong>not</strong> grant` form.
pub fn has_honesty_banner(text: &str) -> bool {
    let hay = text.to_lowercase();
    let Some(i) = hay.find("does") else {
        return false;
    };
    let rest = &hay[i + 4..];
    let Some(j) = rest.find("not") else {
        return false;
    };
    rest[j + 3..].contains("grant epi/exin certification")
}

fn has_quoted_attr(text: &str, attr: &str, value: &str) -> bool {
    text.contains(&format!("{attr}=\"{value}\"")) || text.contains(&format!("{attr}='{value}'"))
}

fn json_id(m: &Json) -> String {
    m.get("id")
        .and_then(Json::as_str)
        .filter(|s| !s.is_empty())
        .unwrap_or("<missing>")
        .to_string()
}

fn json_href(m: &Json) -> Option<String> {
    match m.get("href") {
        None | Some(Json::Null) => None,
        Some(Json::String(s)) => Some(s.clone()),
        Some(other) => Some(other.to_string()),
    }
}

fn toml_as_string(v: &Toml) -> String {
    match v {
        Toml::String(s) => s.clone(),
        Toml::Integer(i) => i.to_string(),
        Toml::Float(f) => f.to_string(),
        Toml::Boolean(b) => b.to_string(),
        Toml::Datetime(d) => d.to_string(),
        other => other.to_string(),
    }
}

fn join_ids(ids: &[&String]) -> String {
    ids.iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Python `Path.resolve(strict=False)`: absolute, `.`/`..` collapsed.
/// Uses `canonicalize` when the path exists so symlinks match the filesystem.
fn resolve_notes(root: &Path, pn: &str) -> PathBuf {
    let raw = PathBuf::from(pn);
    let abs = if raw.is_absolute() {
        raw
    } else {
        root.join(raw)
    };
    if let Ok(c) = abs.canonicalize() {
        return c;
    }
    lexical_normalize(&abs)
}

fn lexical_normalize(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                let _ = out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

fn fail(errors: Vec<String>) -> BuildOutcome {
    let mut out = String::from("FAIL: smoke_learn\n");
    for e in errors {
        out.push_str("  - ");
        out.push_str(&e);
        out.push('\n');
    }
    outcome(1, out)
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
    fn honesty_matches_live_banner_and_plain_prose() {
        assert!(has_honesty_banner(
            "This tool does <strong>not</strong> grant EPI/EXIN certification."
        ));
        assert!(has_honesty_banner("does not grant EPI/EXIN certification"));
        assert!(has_honesty_banner("DOES NOT GRANT epi/exin CERTIFICATION"));
        assert!(!has_honesty_banner("does grant EPI/EXIN certification"));
        assert!(!has_honesty_banner("not a credential"));
        assert!(!has_honesty_banner(""));
    }

    #[test]
    fn quoted_attr_accepts_both_quote_styles() {
        assert!(has_quoted_attr(
            "href=\"assets/css/course.css\"",
            "href",
            "assets/css/course.css"
        ));
        assert!(has_quoted_attr(
            "href='assets/css/course.css'",
            "href",
            "assets/css/course.css"
        ));
        assert!(!has_quoted_attr(
            "href=\"/assets/css/course.css\"",
            "href",
            "assets/css/course.css"
        ));
    }

    #[test]
    fn min_content_floor_is_the_retired_scripts_32() {
        assert_eq!(MIN_CONTENT_BYTES, 32);
    }
}
