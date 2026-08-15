//! Item-review → Learn module/section links (L7-S2).
//!
//! Extracted from `scripts/smoke_feedback_links.py` by
//! `bd-substrate-rust-migration-jhd.18`. The Python is DELETED. This is
//! product, not a gate file: a learner follows Review → Learn from
//! `results.js`. If they can see it, it is not a `cdcp_gate` concern.
//!
//! # Contract
//!
//! * every module `knowledge/domains.toml` declares is mapped in
//!   `MODULE_LEARN_SLUGS` and has an on-disk Learn page + content copy
//! * every `MODULE_LEARN_SLUGS` entry is a module the registry still declares
//! * an item on the seed42 form whose module has no Learn surface is the
//!   C5 "assessed but untaught" defect and is an ERROR, named, never skipped
//! * `knowledge/topics.toml` exists and declares topics (the registry is
//!   not optional)
//! * `web/data/topic_anchors.json` exists, has topics, and has at least one
//!   resolved section anchor when modules are declared
//! * those anchors still exist as heading ids in the shipped markdown
//!
//! # The smoke is a READER. The builder lives here.
//!
//! `run` writes nothing. The retired script imported `build_learn.py` by
//! path and regenerated `topic_anchors.json` on every run — including RED.
//! That write-before-verdict shape is `bd-feedback-links-builder-red-write-2vqx`.
//! `evaluate_topic_anchors` is the rust builder (`match_topic_to_heading` +
//! the topic map). It does not stat `scripts/build_learn.py` and it does
//! not emit an import-failure note. The smoke still *reads* the committed
//! artifact so a missing learner-visible file stays RED.
//!
//! # Empty / absent topics is an ERROR
//!
//! The retired script derived its only section-anchor guard from the same
//! `topics_with_anchor` number the builder produced. Delete or empty
//! `knowledge/topics.toml` and the builder emitted `topics_with_anchor=0`,
//! which switched the guard OFF, and the run printed
//! `section_anchor_hit_rate=0.0%` and PASSED. That hole is
//! `bd-feedback-links-vacuous-topics-ilad`. Closed here: a missing or empty
//! topic registry is RED, a missing or empty anchors file is RED, and
//! `topics_with_anchor == 0` with declared modules present is RED. The
//! floor is not computed by the thing it guards.
//!
//! # What this cannot decide
//!
//! It does not open a browser. A Learn page that exists and 404s at runtime
//! is green here. It does not check that an anchor is the RIGHT section —
//! only that a heading with that id exists. A hit rate that silently halves
//! is invisible; only the degenerate zero case is an error once anchors
//! exist. `results.js` checks are substring scans, not execution.

#![forbid(unsafe_code)]

use crate::slugs::load_module_learn_slugs;
use crate::{join_rel, BuildOutcome, LearnError, GENERATED_BY};
use serde_json::{Map, Value as Json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use toml::Value as Toml;

pub use crate::slugs::{parse_module_learn_slugs, SLUGS_JS_REL};

pub const NAME: &str = "smoke-feedback-links";
pub const SUMMARY: &str =
    "L7-S2: item review → Learn module/section links resolve in both directions";

pub const RESULTS_JS_REL: &str = "web/assets/js/results.js";
pub const LEARN_MD_JS_REL: &str = "web/assets/js/learn_md.js";
pub const LEARN_DIR_REL: &str = "web/learn";
pub const CONTENT_DIR_REL: &str = "web/content/modules";
pub const KEYS_JSON_REL: &str = "web/data/keys_seed42.json";
pub const BANK_JSON_REL: &str = "web/data/bank_items_seed42.json";
pub const TOPIC_ANCHORS_JSON_REL: &str = "web/data/topic_anchors.json";
pub const DOMAINS_TOML_REL: &str = "knowledge/domains.toml";
pub const TOPICS_TOML_REL: &str = "knowledge/topics.toml";

/// How many failure rows of one class the report prints before it summarises
/// the rest. Ten is a report window, not a module bound.
pub const MAX_REPORT_ROWS: usize = 10;

/// Provenance + matcher label written into a rust-built `topic_anchors.json`.
pub const SLUG_ALGORITHM: &str = "learn_md.js CdcpLearnMd.slugify / slugify_heading";

/// Stop words ignored when fuzzy-matching topic labels to headings.
/// Same set as `scripts/build_learn.py` `_STOP`.
pub const TOPIC_MATCH_STOP: &[&str] = &[
    "a", "an", "and", "as", "at", "for", "in", "of", "on", "or", "the", "to", "vs", "with",
];

/// One ATX heading after slug collision-resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    pub level: usize,
    pub text: String,
    pub id: String,
}

/// Run the feedback-link smoke against `root` (the course-engine directory).
///
/// This is a reader: it writes nothing. `BuildOutcome.artifact` is always
/// `None`. `code != 0` is RED.
pub fn run(root: &Path) -> BuildOutcome {
    let mut errors: Vec<String> = Vec::new();

    let (module_slugs, registry_errors) = load_declared_modules(&join_rel(root, DOMAINS_TOML_REL));
    errors.extend(registry_errors);

    check_topic_registry(&join_rel(root, TOPICS_TOML_REL), &mut errors);

    let results_path = join_rel(root, RESULTS_JS_REL);
    if !results_path.is_file() {
        errors.push(format!("missing {RESULTS_JS_REL}"));
    } else {
        match std::fs::read_to_string(&results_path) {
            Ok(text) => {
                if !text.contains("itemLearnHref") {
                    errors.push("results.js missing itemLearnHref".into());
                }
                if !text.contains("learn_href") {
                    errors.push("results.js must set learn_href on item rows".into());
                }
                if !text.contains("Review section in Learn")
                    && !text.contains("Review module in Learn")
                {
                    errors.push("results.js missing \"Review … in Learn\" link copy".into());
                }
                if !text.contains("topic_anchors.json") {
                    errors.push("results.js should load data/topic_anchors.json".into());
                }
                if !text.contains("module_learn_slugs") {
                    errors.push(
                        "results.js must import MODULE_LEARN_SLUGS from the generated map".into(),
                    );
                }
            }
            Err(e) => {
                errors.push(format!("{RESULTS_JS_REL} unreadable: {e}"));
            }
        }
    }

    let mdjs_path = join_rel(root, LEARN_MD_JS_REL);
    if !mdjs_path.is_file() {
        errors.push(format!("missing {LEARN_MD_JS_REL}"));
    } else {
        match std::fs::read_to_string(&mdjs_path) {
            Ok(mdjs) => {
                if !mdjs.contains("function slugify") && !mdjs.contains("slugify:") {
                    errors.push("learn_md.js missing slugify (stable heading anchors)".into());
                }
                if !mdjs.contains("uniqueSlug")
                    && !mdjs.contains("id=\"")
                    && !mdjs.contains("id=\\\"")
                {
                    errors.push("learn_md.js does not appear to emit heading id attributes".into());
                }
            }
            Err(e) => errors.push(format!("{LEARN_MD_JS_REL} unreadable: {e}")),
        }
    }

    let mut slugs: BTreeMap<i64, String> = BTreeMap::new();
    match load_module_learn_slugs(root) {
        Ok(found) => {
            if found.is_empty() {
                errors.push("MODULE_LEARN_SLUGS empty — refusing vacuous green".into());
            }
            slugs = found;
        }
        Err(e) => errors.push(e),
    }

    for (n, expect) in &module_slugs {
        match slugs.get(n) {
            Some(got) if got == expect => {}
            other => {
                let got = other
                    .map(|s| format!("'{s}'"))
                    .unwrap_or_else(|| "None".into());
                errors.push(format!(
                    "module {n}: results.js slug map {got} != '{expect}' ({DOMAINS_TOML_REL})"
                ));
            }
        }
        let page = join_rel(root, LEARN_DIR_REL).join(format!("{expect}.html"));
        if !page.is_file() {
            errors.push(format!("missing learn page {LEARN_DIR_REL}/{expect}.html"));
        }
        let content = join_rel(root, CONTENT_DIR_REL).join(format!("{expect}.md"));
        if !content.is_file() {
            errors.push(format!("missing content {CONTENT_DIR_REL}/{expect}.md"));
        }
    }

    for n in slugs.keys().filter(|n| !module_slugs.contains_key(*n)) {
        errors.push(format!(
            "module {n}: results.js maps '{}' but {DOMAINS_TOML_REL} does not declare that module",
            slugs[n]
        ));
    }

    let keys_path = join_rel(root, KEYS_JSON_REL);
    let bank_path = join_rel(root, BANK_JSON_REL);
    let keys = load_keys(&keys_path, &mut errors);
    let bank_by_id = load_bank(&bank_path, &mut errors);

    let topic_anchors = load_topic_anchors(
        &join_rel(root, TOPIC_ANCHORS_JSON_REL),
        !module_slugs.is_empty(),
        &mut errors,
    );

    let mut heading_ids_by_slug: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for slug in module_slugs.values() {
        let md_path = join_rel(root, CONTENT_DIR_REL).join(format!("{slug}.md"));
        if md_path.is_file() {
            match std::fs::read_to_string(&md_path) {
                Ok(text) => {
                    heading_ids_by_slug.insert(slug.clone(), extract_heading_ids(&text));
                }
                Err(e) => {
                    errors.push(format!("{CONTENT_DIR_REL}/{slug}.md unreadable: {e}"));
                    heading_ids_by_slug.insert(slug.clone(), BTreeSet::new());
                }
            }
        } else {
            heading_ids_by_slug.insert(slug.clone(), BTreeSet::new());
        }
    }

    if let Some(anchors) = topic_anchors.as_ref() {
        if let Some(topics) = anchors.get("topics").and_then(Json::as_object) {
            for (tid, row) in topics {
                let Some(row) = row.as_object() else {
                    continue;
                };
                let Some(anchor) = row
                    .get("anchor")
                    .and_then(Json::as_str)
                    .filter(|s| !s.is_empty())
                else {
                    continue;
                };
                let Some(slug) = row
                    .get("slug")
                    .and_then(Json::as_str)
                    .filter(|s| !s.is_empty())
                else {
                    continue;
                };
                let ids = heading_ids_by_slug.get(slug);
                if ids.map(|s| !s.contains(anchor)).unwrap_or(true) {
                    errors.push(format!(
                        "topic {tid}: anchor '{anchor}' not in headings of {slug}"
                    ));
                }
            }
        }
    }

    let mut total = 0usize;
    let mut module_linked = 0usize;
    let mut section_linked = 0usize;
    let mut missing_module: Vec<String> = Vec::new();
    let mut no_bank: Vec<String> = Vec::new();
    let mut unmapped_modules: Vec<String> = Vec::new();

    let topics_map = topic_anchors
        .as_ref()
        .and_then(|a| a.get("topics"))
        .cloned()
        .unwrap_or(Json::Object(Default::default()));

    if let Some(keys) = keys.as_ref() {
        for k in keys {
            let iid = k
                .get("item_id")
                .and_then(Json::as_str)
                .unwrap_or("")
                .to_string();
            if iid.is_empty() {
                errors.push("key entry missing item_id".into());
                continue;
            }
            total += 1;
            let Some(item) = bank_by_id.as_ref().and_then(|b| b.get(&iid)) else {
                no_bank.push(iid);
                continue;
            };
            let mod_n = json_module(item.get("module"));
            let Some(n) = mod_n.filter(|n| module_slugs.contains_key(n)) else {
                unmapped_modules.push(format!(
                    "{iid}: module {} is not declared in {DOMAINS_TOML_REL} \
                     — assessed with no Learn surface",
                    match mod_n {
                        Some(n) => n.to_string(),
                        None => "None".into(),
                    }
                ));
                continue;
            };
            let slug = &module_slugs[&n];
            let page = join_rel(root, LEARN_DIR_REL).join(format!("{slug}.html"));
            if !page.is_file() {
                missing_module.push(format!("{iid}: 404 {LEARN_DIR_REL}/{slug}.html"));
                continue;
            }
            module_linked += 1;

            let topic_ids = item
                .get("topic_ids")
                .and_then(Json::as_array)
                .cloned()
                .unwrap_or_default();
            let mut anchor: Option<String> = None;
            for tid in &topic_ids {
                let tid = match tid {
                    Json::String(s) => s.clone(),
                    other => other.to_string(),
                };
                let Some(row) = topics_map.get(&tid).and_then(Json::as_object) else {
                    continue;
                };
                let Some(a) = row
                    .get("anchor")
                    .and_then(Json::as_str)
                    .filter(|s| !s.is_empty())
                else {
                    continue;
                };
                let row_mod = json_module(row.get("module"));
                if row_mod.is_some() && row_mod != Some(n) {
                    continue;
                }
                if heading_ids_by_slug
                    .get(slug)
                    .map(|s| s.contains(a))
                    .unwrap_or(false)
                {
                    anchor = Some(a.to_string());
                    break;
                }
            }
            if anchor.is_some() {
                section_linked += 1;
            }
        }
    }

    push_truncated(
        &mut errors,
        &no_bank,
        |iid| format!("key item_id not in bank_items_seed42: {iid}"),
        "more missing bank rows",
    );
    push_truncated(
        &mut errors,
        &missing_module,
        |msg| format!("module link: {msg}"),
        "more module-link failures",
    );
    push_truncated(
        &mut errors,
        &unmapped_modules,
        |msg| format!("assessed but untaught: {msg}"),
        "more items in untaught modules",
    );

    let navigable_keys = total.saturating_sub(unmapped_modules.len() + no_bank.len());
    if keys.is_some() && total == 0 {
        errors.push("zero keys — vacuous".into());
    }
    if module_linked == 0 && navigable_keys > 0 {
        errors.push("zero module-level links resolved — refusing vacuous green".into());
    }

    let topics_with_anchor = topic_anchors
        .as_ref()
        .and_then(|a| a.get("topics_with_anchor"))
        .and_then(json_u64)
        .unwrap_or(0);
    if topics_with_anchor > 0 && module_linked > 0 && section_linked == 0 {
        errors
            .push("section-anchor hit rate 0% despite topics_with_anchor>0 — check matcher".into());
    }

    let hit_rate = if module_linked == 0 {
        0.0
    } else {
        100.0 * section_linked as f64 / module_linked as f64
    };

    if !errors.is_empty() {
        let mut report = vec!["FAIL: smoke_feedback_links".to_string()];
        for e in &errors {
            report.push(format!("  - {e}"));
        }
        report.push(format!(
            "  stats: keys={total} module_linked={module_linked} \
             section_linked={section_linked} hit_rate={hit_rate:.1}% \
             unmapped_mod={}",
            unmapped_modules.len()
        ));
        return outcome(1, report.join("\n") + "\n");
    }

    let mut report = vec![
        "PASS: smoke_feedback_links".to_string(),
        format!(
            "  modules={} (derived from {DOMAINS_TOML_REL})",
            module_slugs.len()
        ),
        format!("  keys_seed42={total}"),
        format!("  module_level_links={module_linked} (non-404 learn/{{slug}}.html)"),
        format!("  section_anchor_links={section_linked}"),
        format!("  section_anchor_hit_rate={hit_rate:.1}% ({section_linked}/{module_linked})"),
        format!(
            "  untaught_module_items={} (must be 0)",
            unmapped_modules.len()
        ),
    ];
    if let Some(anchors) = topic_anchors.as_ref() {
        report.push(format!(
            "  topic_anchors topics_with_anchor={}/{}",
            anchors
                .get("topics_with_anchor")
                .and_then(json_u64)
                .unwrap_or(0),
            anchors.get("topic_count").and_then(json_u64).unwrap_or(0)
        ));
    }
    for (n, slug) in &module_slugs {
        report.push(format!("  M{n:02} → learn/{slug}.html"));
    }
    outcome(0, report.join("\n") + "\n")
}

/// `{module_number: learn_slug}` from the domain registry.
///
/// A missing, malformed or empty registry yields zero modules AND an error —
/// never a silent empty set.
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
            errors.push("domains.toml: [[domain]] row is not a table".into());
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
                if did.is_empty() { "<missing-id>" } else { &did }
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
        errors.push("domain registry declares zero modules (vacuous link check is ERROR)".into());
    }
    (declared, errors)
}

fn check_topic_registry(path: &Path, errors: &mut Vec<String>) {
    if !path.is_file() {
        errors.push(format!(
            "topic registry missing: {TOPICS_TOML_REL} — the section-anchor registry is not optional"
        ));
        return;
    }
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            errors.push(format!("{TOPICS_TOML_REL} unreadable: {e}"));
            return;
        }
    };
    let parsed: Toml = match text.parse() {
        Ok(v) => v,
        Err(e) => {
            errors.push(format!("{TOPICS_TOML_REL} invalid TOML: {e}"));
            return;
        }
    };
    let n = match parsed.get("topic") {
        Some(Toml::Array(rows)) => rows
            .iter()
            .filter(|r| {
                r.get("id")
                    .map(toml_as_string)
                    .map(|s| !s.trim().is_empty())
                    .unwrap_or(false)
            })
            .count(),
        _ => 0,
    };
    if n == 0 {
        errors.push(
            "topic registry declares zero topics (vacuous section-anchor check is ERROR)".into(),
        );
    }
}

fn load_keys(path: &Path, errors: &mut Vec<String>) -> Option<Vec<Json>> {
    if !path.is_file() {
        errors.push(format!("missing {KEYS_JSON_REL}"));
        return None;
    }
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            errors.push(format!("{KEYS_JSON_REL} unreadable: {e}"));
            return None;
        }
    };
    let pack: Json = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => {
            errors.push(format!("keys JSON: {e}"));
            return None;
        }
    };
    let keys = pack.get("keys").and_then(Json::as_array).cloned();
    match keys {
        Some(k) if !k.is_empty() => Some(k),
        _ => {
            errors.push("keys_seed42 has zero keys (vacuous)".into());
            None
        }
    }
}

fn load_bank(path: &Path, errors: &mut Vec<String>) -> Option<BTreeMap<String, Json>> {
    if !path.is_file() {
        errors.push(format!("missing {BANK_JSON_REL}"));
        return None;
    }
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            errors.push(format!("{BANK_JSON_REL} unreadable: {e}"));
            return None;
        }
    };
    let raw: Json = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => {
            errors.push(format!("bank_items_seed42 JSON: {e}"));
            return None;
        }
    };
    let items: Vec<Json> = if let Some(arr) = raw.as_array() {
        arr.clone()
    } else {
        raw.get("items")
            .and_then(Json::as_array)
            .cloned()
            .unwrap_or_default()
    };
    let mut out = BTreeMap::new();
    for it in items {
        if let Some(id) = it
            .get("id")
            .and_then(Json::as_str)
            .filter(|s| !s.is_empty())
        {
            out.insert(id.to_string(), it);
        }
    }
    if out.is_empty() {
        errors.push("bank_items_seed42 empty".into());
        None
    } else {
        Some(out)
    }
}

fn load_topic_anchors(
    path: &Path,
    modules_declared: bool,
    errors: &mut Vec<String>,
) -> Option<Json> {
    if !path.is_file() {
        errors.push(format!("topic_anchors missing: {TOPIC_ANCHORS_JSON_REL}"));
        return None;
    }
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            errors.push(format!("{TOPIC_ANCHORS_JSON_REL} unreadable: {e}"));
            return None;
        }
    };
    let parsed: Json = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => {
            errors.push(format!("topic_anchors.json invalid: {e}"));
            return None;
        }
    };
    let topics = parsed.get("topics").and_then(Json::as_object);
    let topic_n = topics.map(|t| t.len()).unwrap_or(0);
    if topic_n == 0 {
        errors.push(
            "topic_anchors.json has zero topics (vacuous section-anchor check is ERROR)".into(),
        );
    }
    let with_anchor = parsed
        .get("topics_with_anchor")
        .and_then(json_u64)
        .unwrap_or(0);
    if modules_declared && with_anchor == 0 {
        errors.push(
            "topics_with_anchor=0 with declared modules present — a dead section-anchor matcher is an ERROR"
                .into(),
        );
    }
    Some(parsed)
}

/// Compile `web/data/topic_anchors.json` from domains + topics + module markdown.
///
/// Write-after-verdict: a RED compile carries no artifact. Does not stat or
/// import `scripts/build_learn.py` — that presence-check coupling is
/// `bd-feedback-links-build-learn-presence-coupling-jz55`.
pub fn evaluate_topic_anchors(root: &Path) -> BuildOutcome {
    let mut errors: Vec<String> = Vec::new();
    let (module_slugs, registry_errors) = load_declared_modules(&join_rel(root, DOMAINS_TOML_REL));
    errors.extend(registry_errors);

    let topics = load_topics_for_builder(&join_rel(root, TOPICS_TOML_REL), &mut errors);

    let mut navigable: Vec<(i64, String)> = Vec::new();
    let mut headings_by_domain: BTreeMap<String, Vec<Heading>> = BTreeMap::new();
    for (order, slug) in &module_slugs {
        let md_path = join_rel(root, CONTENT_DIR_REL).join(format!("{slug}.md"));
        if !md_path.is_file() {
            continue;
        }
        match std::fs::read_to_string(&md_path) {
            Ok(text) => {
                headings_by_domain.insert(slug.clone(), extract_headings(&text));
            }
            Err(e) => {
                errors.push(format!("{CONTENT_DIR_REL}/{slug}.md unreadable: {e}"));
                headings_by_domain.insert(slug.clone(), Vec::new());
            }
        }
        navigable.push((*order, slug.clone()));
    }

    if errors.is_empty() && topics.is_empty() {
        errors.push(
            "topic registry declares zero topics (vacuous section-anchor check is ERROR)".into(),
        );
    }

    let (topic_map, matched) = if errors.is_empty() {
        build_topic_map(&topics, &navigable, &headings_by_domain)
    } else {
        (BTreeMap::new(), 0)
    };

    if errors.is_empty() && topic_map.is_empty() {
        errors.push(
            "topic_anchors.json has zero topics (vacuous section-anchor check is ERROR)".into(),
        );
    }
    if errors.is_empty() && !module_slugs.is_empty() && matched == 0 {
        errors.push(
            "topics_with_anchor=0 with declared modules present — a dead section-anchor matcher is an ERROR"
                .into(),
        );
    }

    if !errors.is_empty() {
        let mut report = vec!["FAIL: build_topic_anchors".to_string()];
        for e in &errors {
            report.push(format!("  - {e}"));
        }
        report.push(format!(
            "  out={TOPIC_ANCHORS_JSON_REL} NOT WRITTEN (a failing build leaves no artifact)"
        ));
        return outcome(1, report.join("\n") + "\n");
    }

    let body = render_topic_anchors(&topic_map, matched, &navigable, &headings_by_domain);
    let out = join_rel(root, TOPIC_ANCHORS_JSON_REL);
    BuildOutcome {
        stdout: format!(
            "PASS: build_topic_anchors topics_with_anchor={matched}/{} → {TOPIC_ANCHORS_JSON_REL}\n",
            topic_map.len()
        ),
        code: 0,
        artifact: Some((out, body)),
    }
}

/// Compile and write `topic_anchors.json` on the GREEN path only.
pub fn write_topic_anchors(root: &Path) -> Result<BuildOutcome, LearnError> {
    let built = evaluate_topic_anchors(root);
    debug_assert!(
        built.code == 0 || built.artifact.is_none(),
        "a failing run must not carry an artifact"
    );
    if built.code == 0 {
        if let Some((path, body)) = &built.artifact {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| LearnError::io(format!("mkdir {}: {e}", parent.display())))?;
            }
            std::fs::write(path, body.as_bytes())
                .map_err(|e| LearnError::io(format!("write {}: {e}", path.display())))?;
        }
    }
    Ok(built)
}

/// topic_id → learn href fragment map. `navigable` is `(order, domain_id)`.
pub fn build_topic_map(
    topics: &[(String, String, String)],
    navigable: &[(i64, String)],
    headings_by_domain: &BTreeMap<String, Vec<Heading>>,
) -> (BTreeMap<String, Map<String, Json>>, usize) {
    let domain_by_id: BTreeMap<&str, i64> = navigable
        .iter()
        .map(|(order, id)| (id.as_str(), *order))
        .collect();
    let mut topic_map: BTreeMap<String, Map<String, Json>> = BTreeMap::new();
    let mut matched = 0usize;
    for (tid, domain, label) in topics {
        let mut row = Map::new();
        row.insert("topic_id".into(), Json::String(tid.clone()));
        row.insert("domain".into(), Json::String(domain.clone()));
        row.insert("label".into(), Json::String(label.clone()));
        match domain_by_id.get(domain.as_str()) {
            None => {
                row.insert("module".into(), Json::Null);
                row.insert("slug".into(), Json::Null);
                row.insert("anchor".into(), Json::Null);
                row.insert("href".into(), Json::Null);
            }
            Some(order) => {
                let slug = domain.as_str();
                let heads = headings_by_domain
                    .get(slug)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]);
                let anchor = match_topic_to_heading(label, tid, heads);
                let mut href = format!("learn/{slug}.html");
                if let Some(a) = anchor.as_ref() {
                    href = format!("{href}#{a}");
                    matched += 1;
                }
                row.insert("module".into(), Json::from(*order));
                row.insert("slug".into(), Json::String(slug.to_string()));
                row.insert(
                    "anchor".into(),
                    match anchor {
                        Some(a) => Json::String(a),
                        None => Json::Null,
                    },
                );
                row.insert("href".into(), Json::String(href));
            }
        }
        topic_map.insert(tid.clone(), row);
    }
    (topic_map, matched)
}

/// Best-effort map topic label/id → heading id. `None` if no credible match.
///
/// Port of `scripts/build_learn.py` `match_topic_to_heading`. Prefer h2/h3;
/// exact slug or casefold title wins; otherwise a scored overlap with a
/// floor of 50.
pub fn match_topic_to_heading(label: &str, topic_id: &str, headings: &[Heading]) -> Option<String> {
    if headings.is_empty() {
        return None;
    }
    let section: Vec<&Heading> = {
        let h23: Vec<&Heading> = headings
            .iter()
            .filter(|h| h.level == 2 || h.level == 3)
            .collect();
        if h23.is_empty() {
            headings.iter().collect()
        } else {
            h23
        }
    };
    let label_slug = slugify_heading(label);
    if section.iter().any(|h| h.id == label_slug) {
        return Some(label_slug);
    }
    let label_cf = label.to_lowercase();
    let label_cf = label_cf.trim();
    for h in &section {
        if h.text.to_lowercase().trim() == label_cf {
            return Some(h.id.clone());
        }
    }

    let tail = if topic_id.starts_with('m') {
        topic_id
            .split_once('-')
            .map(|(_, rest)| rest)
            .unwrap_or(topic_id)
    } else {
        topic_id
    };
    let tail_tokens: Vec<&str> = tail
        .split(['-', '_'])
        .filter(|t| !t.is_empty() && !TOPIC_MATCH_STOP.contains(t))
        .collect();
    let label_words = significant_words(label);

    let mut candidates: Vec<(i32, i32, String)> = Vec::new();
    for h in &section {
        let hid = &h.id;
        let htext = h.text.to_lowercase();
        let hslug = hid.as_str();
        let mut score = 0i32;
        if !label_slug.is_empty() && (hslug.contains(&label_slug) || label_slug.contains(hslug)) {
            score = score.max(if label_slug == hslug { 80 } else { 55 });
        }
        if !label_words.is_empty() {
            let hits = label_words
                .iter()
                .filter(|w| htext.contains(w.as_str()) || hslug.contains(w.as_str()))
                .count();
            if hits == label_words.len() && hits > 0 {
                score = score.max(70 + (hits.min(5) as i32));
            } else if hits >= 2.max(label_words.len().div_ceil(2)) {
                score = score.max(40 + hits as i32);
            }
        }
        if !tail_tokens.is_empty() {
            let t_hits = tail_tokens
                .iter()
                .filter(|t| hslug.contains(*t) || htext.contains(*t))
                .count();
            if t_hits == tail_tokens.len() && t_hits > 0 {
                score = score.max(60 + t_hits as i32);
            } else if t_hits >= 1 && tail_tokens.len() == 1 {
                score = score.max(50);
            }
        }
        if score > 0 {
            let level_bonus = if h.level == 3 { 1 } else { 0 };
            candidates.push((score + level_bonus, -(hid.len() as i32), hid.clone()));
        }
    }
    if candidates.is_empty() {
        return None;
    }
    candidates.sort_by(|a, b| b.cmp(a));
    let (best_score, _, best_id) = candidates.remove(0);
    if best_score < 50 {
        None
    } else {
        Some(best_id)
    }
}

fn load_topics_for_builder(path: &Path, errors: &mut Vec<String>) -> Vec<(String, String, String)> {
    if !path.is_file() {
        errors.push(format!(
            "topic registry missing: {TOPICS_TOML_REL} — the section-anchor registry is not optional"
        ));
        return Vec::new();
    }
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            errors.push(format!("{TOPICS_TOML_REL} unreadable: {e}"));
            return Vec::new();
        }
    };
    let parsed: Toml = match text.parse() {
        Ok(v) => v,
        Err(e) => {
            errors.push(format!("{TOPICS_TOML_REL} invalid TOML: {e}"));
            return Vec::new();
        }
    };
    let rows = match parsed.get("topic") {
        Some(Toml::Array(rows)) => rows.as_slice(),
        _ => return Vec::new(),
    };
    let mut out = Vec::new();
    for row in rows {
        let Some(table) = row.as_table() else {
            continue;
        };
        let tid = table
            .get("id")
            .map(toml_as_string)
            .unwrap_or_default()
            .trim()
            .to_string();
        if tid.is_empty() {
            continue;
        }
        let domain = table
            .get("domain")
            .map(toml_as_string)
            .unwrap_or_default()
            .trim()
            .to_string();
        let label = table
            .get("label")
            .map(toml_as_string)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| tid.clone());
        out.push((tid, domain, label));
    }
    out
}

fn render_topic_anchors(
    topic_map: &BTreeMap<String, Map<String, Json>>,
    matched: usize,
    navigable: &[(i64, String)],
    headings_by_domain: &BTreeMap<String, Vec<Heading>>,
) -> String {
    let mut modules = Map::new();
    for (order, slug) in navigable {
        let headings: Vec<Json> = headings_by_domain
            .get(slug)
            .into_iter()
            .flatten()
            .filter(|h| h.level == 2 || h.level == 3)
            .map(|h| {
                let mut m = Map::new();
                m.insert("id".into(), Json::String(h.id.clone()));
                m.insert("level".into(), Json::from(h.level as u64));
                m.insert("text".into(), Json::String(h.text.clone()));
                Json::Object(m)
            })
            .collect();
        let mut row = Map::new();
        row.insert("order".into(), Json::from(*order));
        row.insert("headings".into(), Json::Array(headings));
        modules.insert(slug.clone(), Json::Object(row));
    }
    let topics = topic_map
        .iter()
        .map(|(k, v)| (k.clone(), Json::Object(v.clone())))
        .collect();
    let mut root = Map::new();
    root.insert("schema_version".into(), Json::from(1u64));
    root.insert("generated_by".into(), Json::String(GENERATED_BY.into()));
    root.insert("slug_algorithm".into(), Json::String(SLUG_ALGORITHM.into()));
    root.insert("topic_count".into(), Json::from(topic_map.len() as u64));
    root.insert("topics_with_anchor".into(), Json::from(matched as u64));
    root.insert("modules".into(), Json::Object(modules));
    root.insert("topics".into(), Json::Object(topics));
    let mut body =
        serde_json::to_string_pretty(&Json::Object(root)).unwrap_or_else(|_| "{}".into());
    body.push('\n');
    body
}

fn significant_words(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    let mut words = Vec::new();
    let mut cur = String::new();
    let flush = |cur: &mut String, words: &mut Vec<String>| {
        if cur.is_empty() {
            return;
        }
        let keep = !TOPIC_MATCH_STOP.contains(&cur.as_str()) && cur.len() > 1;
        let taken = std::mem::take(cur);
        if keep {
            words.push(taken);
        }
    };
    for c in lower.chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            cur.push(c);
        } else {
            flush(&mut cur, &mut words);
        }
    }
    flush(&mut cur, &mut words);
    words
}

/// Must match `learn_md.js` `CdcpLearnMd.slugify` / `build_learn.slugify_heading`.
pub fn slugify_heading(text: &str) -> String {
    let lower = text.to_lowercase();
    let no_md: String = lower
        .chars()
        .filter(|c| !matches!(c, '*' | '_' | '`'))
        .collect();
    let kept: String = no_md
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-')
        .collect();
    let trimmed = kept.trim();
    let mut out = String::new();
    let mut last_dash = false;
    for c in trimmed.chars() {
        if c.is_whitespace() || c == '-' {
            if !last_dash {
                out.push('-');
                last_dash = true;
            }
        } else {
            out.push(c);
            last_dash = false;
        }
    }
    let out = out.trim_matches('-').to_string();
    if out.is_empty() {
        "section".into()
    } else {
        out
    }
}

/// ATX headings for a markdown body (fences skipped, unique suffixes).
pub fn extract_headings(md_text: &str) -> Vec<Heading> {
    let mut used: BTreeMap<String, u32> = BTreeMap::new();
    let mut out = Vec::new();
    let mut in_fence = false;
    let norm = md_text.replace("\r\n", "\n").replace('\r', "\n");
    for raw in norm.lines() {
        let stripped = raw.trim();
        if stripped.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        let Some((level, title)) = parse_atx(stripped) else {
            continue;
        };
        let title = strip_closing_hashes(&title).to_string();
        let plain = unwrap_md_links(&title);
        let plain: String = plain
            .chars()
            .filter(|c| !matches!(c, '*' | '_' | '`'))
            .collect();
        let base = slugify_heading(&plain);
        let hid = unique_slug(&base, &mut used);
        out.push(Heading {
            level,
            text: title,
            id: hid,
        });
    }
    out
}

/// Heading ids for a markdown body (ATX, fences skipped, unique suffixes).
pub fn extract_heading_ids(md_text: &str) -> BTreeSet<String> {
    extract_headings(md_text)
        .into_iter()
        .map(|h| h.id)
        .collect()
}

fn unique_slug(base: &str, used: &mut BTreeMap<String, u32>) -> String {
    if !used.contains_key(base) {
        used.insert(base.to_string(), 1);
        return base.to_string();
    }
    let mut n = used.get(base).copied().unwrap_or(1) + 1;
    while used.contains_key(&format!("{base}-{n}")) {
        n += 1;
    }
    used.insert(base.to_string(), n);
    used.insert(format!("{base}-{n}"), 1);
    format!("{base}-{n}")
}

fn parse_atx(stripped: &str) -> Option<(usize, String)> {
    if !stripped.starts_with('#') {
        return None;
    }
    let mut level = 0usize;
    for c in stripped.chars() {
        if c == '#' && level < 6 {
            level += 1;
        } else {
            break;
        }
    }
    if level == 0 {
        return None;
    }
    let rest = &stripped[level..];
    if !rest.starts_with(|c: char| c.is_whitespace()) {
        return None;
    }
    Some((level, rest.trim().to_string()))
}

fn strip_closing_hashes(s: &str) -> &str {
    let t = s.trim_end();
    let without = t.trim_end_matches('#');
    if without.len() == t.len() {
        return t;
    }
    let trimmed = without.trim_end();
    if trimmed.len() == without.len() {
        t
    } else {
        trimmed
    }
}

fn unwrap_md_links(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '[' {
            if let Some((text, next)) = parse_md_link(&chars, i) {
                out.push_str(&text);
                i = next;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

fn parse_md_link(chars: &[char], start: usize) -> Option<(String, usize)> {
    let mut i = start + 1;
    let mut text = String::new();
    while i < chars.len() && chars[i] != ']' {
        text.push(chars[i]);
        i += 1;
    }
    if text.is_empty() || i >= chars.len() || chars[i] != ']' {
        return None;
    }
    i += 1;
    if i >= chars.len() || chars[i] != '(' {
        return None;
    }
    i += 1;
    let url_at = i;
    while i < chars.len() && chars[i] != ')' {
        i += 1;
    }
    if i >= chars.len() || i == url_at {
        return None;
    }
    Some((text, i + 1))
}

fn push_truncated(
    errors: &mut Vec<String>,
    rows: &[String],
    fmt: impl Fn(&str) -> String,
    more: &str,
) {
    for msg in rows.iter().take(MAX_REPORT_ROWS) {
        errors.push(fmt(msg));
    }
    if rows.len() > MAX_REPORT_ROWS {
        errors.push(format!("… and {} {more}", rows.len() - MAX_REPORT_ROWS));
    }
}

fn json_module(v: Option<&Json>) -> Option<i64> {
    match v {
        Some(Json::Number(n)) => n.as_i64().or_else(|| n.as_u64().map(|u| u as i64)),
        Some(Json::String(s)) => s.trim().parse().ok(),
        _ => None,
    }
}

fn json_u64(v: &Json) -> Option<u64> {
    match v {
        Json::Number(n) => n
            .as_u64()
            .or_else(|| n.as_i64().and_then(|i| u64::try_from(i).ok())),
        Json::String(s) => s.trim().parse().ok(),
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
    fn slugify_matches_curriculum_english() {
        assert_eq!(
            slugify_heading("Types of data centres"),
            "types-of-data-centres"
        );
        assert_eq!(
            slugify_heading("Elements of a data centre"),
            "elements-of-a-data-centre"
        );
        assert_eq!(
            slugify_heading("Availability “nines” and annual downtime"),
            "availability-nines-and-annual-downtime"
        );
        assert_eq!(slugify_heading("***"), "section");
        assert_eq!(slugify_heading(""), "section");
        assert_eq!(slugify_heading("  Foo   Bar  "), "foo-bar");
    }

    #[test]
    fn heading_ids_skip_fences_and_suffix_dupes() {
        let ids = extract_heading_ids("# Title\n```\n# not a heading\n```\n## Title\n## Title\n");
        assert!(ids.contains("title"));
        assert!(ids.contains("title-2"));
        assert!(!ids.contains("not-a-heading"));
    }

    #[test]
    fn heading_ids_unwrap_markdown_links() {
        let ids = extract_heading_ids("## See [Types of data centres](https://example)\n");
        assert!(ids.contains("see-types-of-data-centres"));
    }

    #[test]
    fn report_window_is_not_a_module_bound() {
        assert_eq!(MAX_REPORT_ROWS, 10);
        assert!(MAX_REPORT_ROWS > 0);
    }

    #[test]
    fn matcher_exact_slug_and_miss() {
        let heads = extract_headings("## Types of data centres\n\nBody.\n");
        assert_eq!(
            match_topic_to_heading("Types of data centres", "m01-dc-types", &heads).as_deref(),
            Some("types-of-data-centres")
        );
        assert_eq!(
            match_topic_to_heading("No such heading anywhere", "m99-zzz", &heads),
            None
        );
    }
}
