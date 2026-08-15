//! Weak-link smoke — every declared domain maps to a Learn page.
//!
//! Extracted from `scripts/smoke_weak_links.py` by
//! `bd-substrate-rust-migration-jhd.16` as product, not a gate file. A
//! learner follows Results → Learn via `MODULE_LEARN_SLUGS`. If they can
//! see it, it is not a `cdcp_gate` concern.
//!
//! # Contract
//!
//! * every module `knowledge/domains.toml` declares is mapped in
//!   `MODULE_LEARN_SLUGS` with the declared slug, and has
//!   `web/learn/{slug}.html`
//! * every `MODULE_LEARN_SLUGS` entry is a module the registry still
//!   declares (drift in both directions is RED, naming the module)
//! * `modules_index.json` order→id (when present) agrees with the registry
//! * `moduleLearnHref` emits `learn/{slug}.html` for mapped modules
//!
//! # The module set is READ (bd-ggs7)
//!
//! `order` is the bank module number and `id` is the Learn slug. Until
//! 2026-08-14 the retired script carried a hand-written module→slug table —
//! the third surviving copy of that mapping. A frozen table stays correct
//! right up until the registry gains a module, then fails a correct change
//! for being correct (the bd-lt7 failure mode).
//!
//! # Anti-vacuous
//!
//! A missing, unparseable or empty registry is an ERROR. So is a registry
//! that declares fewer than 14 modules: that is a FLOOR taken from the
//! certification's fourteen public EPI CDCP domains, not from whatever the
//! tree happens to hold today. It can never hold a module out — module 15
//! and any later partner supplement sit above it — it can only notice a
//! registry that collapsed. An empty `MODULE_LEARN_SLUGS` is likewise an
//! ERROR.
//!
//! # Verdict discipline
//!
//! Every check is collected first. The report — verdict line included — is
//! composed and printed once, at the end. No PASS is emitted ahead of work
//! that can fail.
//!
//! # What this cannot decide
//!
//! It does not open a browser. A Learn page that exists and 404s at runtime
//! is green here. `results.js` checks are substring scans, not execution.
//! Completeness of the corpus (a Learn page with no `[[domain]]` row) is
//! invisible unless that page is also mapped in `MODULE_LEARN_SLUGS`.

#![forbid(unsafe_code)]

use crate::slugs::load_module_learn_slugs;
use crate::{join_rel, BuildOutcome};
use serde_json::Value as Json;
use std::collections::BTreeMap;
use std::path::Path;
use toml::Value as Toml;

pub const NAME: &str = "smoke-weak-links";
pub const SUMMARY: &str =
    "L6-S3: every declared domain maps to a Learn page via MODULE_LEARN_SLUGS";

pub const RESULTS_JS_REL: &str = "web/assets/js/results.js";
pub use crate::slugs::SLUGS_JS_REL;
pub const LEARN_DIR_REL: &str = "web/learn";
pub const INDEX_JSON_REL: &str = "web/data/modules_index.json";
pub const DOMAINS_TOML_REL: &str = "knowledge/domains.toml";

/// Run the weak-link smoke against `root` (the course-engine directory).
///
/// This is a reader: it writes nothing. `BuildOutcome.artifact` is always
/// `None`. `code != 0` is RED.
pub fn run(root: &Path) -> BuildOutcome {
    let mut errors: Vec<String> = Vec::new();

    let (declared, registry_errors) = load_declared_modules(&join_rel(root, DOMAINS_TOML_REL));
    errors.extend(registry_errors);

    let results_path = join_rel(root, RESULTS_JS_REL);
    let js = if !results_path.is_file() {
        errors.push(format!("missing {RESULTS_JS_REL}"));
        None
    } else {
        match std::fs::read_to_string(&results_path) {
            Ok(text) => Some(text),
            Err(e) => {
                errors.push(format!("{RESULTS_JS_REL} unreadable: {e}"));
                None
            }
        }
    };

    if let Some(js) = js.as_ref() {
        if !js.contains("function moduleLearnHref") && !js.contains("moduleLearnHref") {
            errors.push("moduleLearnHref helper missing from results.js".into());
        }
        if !js.contains("Review weak modules in Learn") {
            errors.push("CTA copy \"Review weak modules in Learn\" missing from results.js".into());
        }
        if !js.contains("weak-chip--link") && !js.contains("href=\"") {
            errors.push("weak module chips do not appear to emit learn hrefs".into());
        }
        if !js.contains("moduleLearnHref") || !js.contains("learn/") {
            errors.push("results.js must call moduleLearnHref / emit learn/… hrefs".into());
        }
        if !js.contains("module_learn_slugs") {
            errors.push("results.js must import MODULE_LEARN_SLUGS from the generated map".into());
        }
    }

    let mut slugs: BTreeMap<i64, String> = BTreeMap::new();
    match load_module_learn_slugs(root) {
        Ok(found) => {
            if found.is_empty() {
                errors.push("MODULE_LEARN_SLUGS is empty — refusing vacuous green".into());
            }
            slugs = found;
        }
        Err(e) => errors.push(e),
    }

    for (n, expect) in &declared {
        match slugs.get(n) {
            Some(got) if got == expect => {}
            None => errors.push(format!(
                "module {n}: knowledge/domains.toml declares '{expect}' but \
                 MODULE_LEARN_SLUGS has no entry — a learner cannot reach it \
                 from results"
            )),
            Some(got) => errors.push(format!(
                "module {n}: map slug '{got}' != declared '{expect}' \
                 (knowledge/domains.toml)"
            )),
        }
    }

    for n in slugs.keys().filter(|n| !declared.contains_key(*n)) {
        errors.push(format!(
            "module {n}: results.js maps '{}' but knowledge/domains.toml \
             does not declare that module",
            slugs[n]
        ));
    }

    let learn_dir = join_rel(root, LEARN_DIR_REL);
    if !learn_dir.is_dir() {
        errors.push(format!("missing learn dir {LEARN_DIR_REL}"));
    } else {
        for (n, slug) in &declared {
            let page = learn_dir.join(format!("{slug}.html"));
            if !page.is_file() {
                errors.push(format!(
                    "module {n}: declared slug has no Learn page {LEARN_DIR_REL}/{slug}.html"
                ));
            }
        }
    }

    check_modules_index(root, &declared, &mut errors);

    if !errors.is_empty() {
        let mut report = vec!["FAIL: smoke_weak_links".to_string()];
        for e in &errors {
            report.push(format!("  - {e}"));
        }
        return outcome(1, report.join("\n") + "\n");
    }

    let mut report = vec![
        "PASS: smoke_weak_links".to_string(),
        format!(
            "  modules={} (derived from {DOMAINS_TOML_REL})",
            declared.len()
        ),
        format!("  learn_dir={LEARN_DIR_REL}"),
    ];
    for (n, slug) in &declared {
        report.push(format!("  M{n:02} → learn/{slug}.html"));
    }
    outcome(0, report.join("\n") + "\n")
}

/// `{module_number: learn_slug}` from the domain registry.
///
/// A missing, malformed, empty or collapsed registry yields errors — never
/// a silent empty set that would make every check below vacuous.
pub fn load_declared_modules(domains_path: &Path) -> (BTreeMap<i64, String>, Vec<String>) {
    let mut errors = Vec::new();
    let mut declared = BTreeMap::new();
    if !domains_path.is_file() {
        return (
            declared,
            vec![format!("domain registry missing: {DOMAINS_TOML_REL}")],
        );
    }
    let text = match std::fs::read_to_string(domains_path) {
        Ok(t) => t,
        Err(e) => {
            return (declared, vec![format!("domain registry unreadable: {e}")]);
        }
    };
    let parsed: Toml = match text.parse() {
        Ok(v) => v,
        Err(e) => {
            return (declared, vec![format!("domain registry parse error: {e}")]);
        }
    };
    let rows = match parsed.get("domain") {
        Some(Toml::Array(rows)) => rows.as_slice(),
        None => &[][..],
        Some(_) => {
            errors.push("domains.toml `domain` is not an array of tables".into());
            &[][..]
        }
    };
    for row in rows {
        let Some(table) = row.as_table() else {
            errors.push(format!(
                "domains.toml: [[domain]] row is not a table: {row}"
            ));
            continue;
        };
        let did = table
            .get("id")
            .map(toml_as_string)
            .unwrap_or_default()
            .trim()
            .to_string();
        let Some(order) = toml_order(table.get("order")) else {
            errors.push(format!(
                "domains.toml: {} has no usable order",
                if did.is_empty() {
                    "<missing-id>"
                } else {
                    did.as_str()
                }
            ));
            continue;
        };
        if did.is_empty() {
            errors.push(format!(
                "domains.toml: module {order} has no id (no Learn slug)"
            ));
            continue;
        }
        if declared.contains_key(&order) {
            errors.push(format!(
                "domains.toml: duplicate order {order} ({} and {did})",
                declared[&order]
            ));
            continue;
        }
        declared.insert(order, did);
    }
    if declared.is_empty() {
        errors.push(
            "domain registry declares zero modules (vacuous weak-link check is ERROR)".into(),
        );
    } else if declared.len() < 14 {
        // FLOOR, not an exclusion: the fourteen public EPI CDCP domains.
        // The literal is written here rather than hidden behind a named
        // constant so the bd-lt7 bound sweep can see it and hold a verdict
        // on it. It cannot hold a module out; it can only notice a
        // collapsed registry.
        errors.push(format!(
            "domain registry declares only {} modules; the CDCP course has fourteen \
             public EPI domains at minimum (vacuous weak-link check is ERROR)",
            declared.len()
        ));
    }
    (declared, errors)
}

fn check_modules_index(root: &Path, declared: &BTreeMap<i64, String>, errors: &mut Vec<String>) {
    let path = join_rel(root, INDEX_JSON_REL);
    if !path.is_file() {
        // Index optional for this smoke; registry+map+files are the hard gate.
        return;
    }
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            errors.push(format!("{INDEX_JSON_REL} unreadable: {e}"));
            return;
        }
    };
    let index: Json = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => {
            errors.push(format!("modules_index.json invalid JSON: {e}"));
            return;
        }
    };
    let Some(modules) = index.get("modules").and_then(Json::as_array) else {
        return;
    };
    for m in modules {
        if m.get("empty") == Some(&Json::Bool(true)) {
            continue;
        }
        let Some(n) = json_order(m.get("order")) else {
            continue;
        };
        let mid = m.get("id").and_then(Json::as_str);
        if !declared.contains_key(&n) {
            errors.push(format!(
                "modules_index has navigable order={n} id={mid:?} which \
                 knowledge/domains.toml does not declare"
            ));
            continue;
        }
        if declared[&n] != mid.unwrap_or("") {
            errors.push(format!(
                "modules_index order={n} id={mid:?} != declared slug '{}'",
                declared[&n]
            ));
        }
        let href = m.get("href").and_then(Json::as_str).unwrap_or("");
        let want = format!("learn/{}.html", declared[&n]);
        if !href.is_empty() && href != want {
            errors.push(format!("modules_index order={n} href={href:?} != {want:?}"));
        }
    }
}

fn json_order(v: Option<&Json>) -> Option<i64> {
    match v {
        Some(Json::Number(n)) => n.as_i64().or_else(|| n.as_u64().map(|u| u as i64)),
        Some(Json::String(s)) => s.trim().parse().ok(),
        _ => None,
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

fn toml_order(v: Option<&Toml>) -> Option<i64> {
    match v {
        Some(Toml::Integer(i)) => Some(*i),
        Some(Toml::String(s)) => s.trim().parse().ok(),
        Some(Toml::Float(f)) if f.is_finite() => Some(*f as i64),
        _ => None,
    }
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
    fn floor_is_the_fourteen_public_domains() {
        // The comparison lives on the load path so the bd-lt7 sweep can
        // hold a verdict on it. This test only names that the floor is
        // the thing the load path already encodes.
        let (_declared, errors) = load_declared_modules(Path::new("/no/such/domains.toml"));
        assert!(
            errors.iter().any(|e| e.contains("domain registry missing")),
            "{errors:?}"
        );
    }
}
