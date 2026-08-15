//! Learn surface compiler — pages, hub, content copies, modules_index, topic_anchors.
//!
//! Extracted from `scripts/build_learn.py` by
//! `bd-substrate-rust-migration-jhd.28`. The Python is DELETED. This is
//! product, not a gate file: a learner opens `web/learn.html` and the
//! per-module pages this compiler writes. If they can see it, it is not
//! a `cdcp_gate` concern.
//!
//! # Contract
//!
//! * every `[[domain]]` with a usable id is emitted into `modules_index.json`
//! * empty `primary_notes` is allowed only when `exam_weight_unknown = true`
//! * a missing notes file is RED, naming the path
//! * zero domain rows, or a missing/unreadable/unparseable registry, is RED
//! * write-after-verdict: a RED compile writes nothing and unlinks nothing
//! * leftover generated `{NN-slug}.md` copies are swept; tracked docs stay
//! * an empty content-dir sweep (zero `.md` after apply) is an ERROR
//!
//! # What this cannot decide
//!
//! It does not open a browser. A page whose banner is present and whose
//! body is a stub is the same write as a correct page. It does not check
//! that a topic-anchor is the RIGHT heading — only that the matcher ran.
//! Completeness of the parent corpus (a notes file with no `[[domain]]`
//! row) is invisible here.

#![forbid(unsafe_code)]

use crate::content::{should_unlink_content_copy, CONTENT_DIR_REL, PROTECTED_CONTENT_DOCS};
use crate::feedback::{
    build_topic_map, extract_headings, render_topic_anchors, Heading,
    CONTENT_DIR_REL as FEEDBACK_CONTENT, TOPIC_ANCHORS_JSON_REL,
};
use crate::{join_rel, BuildOutcome, LearnError, GENERATED_BY};
use serde_json::{Map, Value as Json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use toml::Value as Toml;

pub const NAME: &str = "build-learn";
pub const SUMMARY: &str =
    "compile the offline Learn surface (pages, hub, modules_index, topic_anchors)";

pub const DOMAINS_REL: &str = "knowledge/domains.toml";
pub const TOPICS_REL: &str = "knowledge/topics.toml";
pub const LEARN_DIR_REL: &str = "web/learn";
pub const INDEX_REL: &str = "web/data/modules_index.json";
pub const HUB_REL: &str = "web/learn.html";
pub const TOPIC_ANCHORS_REL: &str = TOPIC_ANCHORS_JSON_REL;

const HONESTY: &str = concat!(
    "<strong>Study tool only.</strong>\n",
    "      This tool does <strong>not</strong> grant EPI/EXIN certification.\n",
    "      Completing practice here is not a CDCP credential."
);

/// Planned writes + unlinks. `writes`/`unlinks` are empty on every RED path.
#[derive(Debug, Clone)]
pub struct LearnPlan {
    pub stdout: String,
    pub code: i32,
    pub writes: Vec<(PathBuf, Vec<u8>)>,
    pub unlinks: Vec<PathBuf>,
}

impl LearnPlan {
    pub fn outcome(&self) -> BuildOutcome {
        let artifact = self
            .writes
            .iter()
            .find(|(p, _)| p.ends_with("modules_index.json"))
            .map(|(p, b)| (p.clone(), String::from_utf8_lossy(b).into_owned()));
        BuildOutcome {
            stdout: self.stdout.clone(),
            code: self.code,
            artifact: if self.code == 0 { artifact } else { None },
        }
    }
}

/// Compile. Does not write. A RED compile carries no writes and no unlinks.
pub fn evaluate(root: &Path) -> Result<LearnPlan, LearnError> {
    let (domains, load_errors) = load_domains(&join_rel(root, DOMAINS_REL));
    let mut errors = load_errors;
    let mut modules: Vec<Module> = Vec::new();
    let mut copies: Vec<(PathBuf, Vec<u8>, String)> = Vec::new();
    let mut headings_by_domain: BTreeMap<String, Vec<Heading>> = BTreeMap::new();

    for dom in &domains {
        let did = toml_string(dom.get("id"));
        if did.is_empty() {
            errors.push("domain missing id".into());
            continue;
        }
        let order = toml_int(dom.get("order")).unwrap_or(0);
        let heading = {
            let h = toml_string(dom.get("epi_heading"));
            if h.is_empty() {
                did.clone()
            } else {
                h
            }
        };
        let pn = match dom.get("primary_notes") {
            None => String::new(),
            Some(v) => toml_as_string(v),
        };
        let pn_s = pn.trim().to_string();
        let exam_unknown = matches!(dom.get("exam_weight_unknown"), Some(Toml::Boolean(true)));

        let mut entry = Module {
            id: did.clone(),
            order,
            epi_heading: heading,
            primary_notes: pn_s.clone(),
            exam_weight_unknown: exam_unknown,
            empty: false,
            href: None,
            content_path: None,
            source_path: None,
            word_count: None,
            estimate_minutes: None,
        };

        if pn_s.is_empty() {
            if !exam_unknown {
                errors.push(format!(
                    "{did}: empty primary_notes without exam_weight_unknown=true"
                ));
                continue;
            }
            entry.empty = true;
            modules.push(entry);
            continue;
        }

        let src = resolve_notes(root, &pn_s);
        if !src.is_file() {
            errors.push(format!(
                "{did}: primary_notes missing: {pn_s} → {}",
                src.display()
            ));
            continue;
        }
        let bytes = match std::fs::read(&src) {
            Ok(b) => b,
            Err(e) => {
                errors.push(format!("{did}: primary_notes unreadable: {e}"));
                continue;
            }
        };
        let text = match std::str::from_utf8(&bytes) {
            Ok(t) => t,
            Err(e) => {
                errors.push(format!("{did}: primary_notes not UTF-8: {e}"));
                continue;
            }
        };
        let words = word_count(text);
        let eta = estimate_minutes(words);
        headings_by_domain.insert(did.clone(), extract_headings(text));

        let dest = join_rel(root, CONTENT_DIR_REL).join(format!("{did}.md"));
        copies.push((dest, bytes, did.clone()));

        entry.empty = false;
        entry.href = Some(format!("learn/{did}.html"));
        entry.content_path = Some(format!("content/modules/{did}.md"));
        entry.source_path = Some(pn_s);
        entry.word_count = Some(words);
        entry.estimate_minutes = Some(eta);
        modules.push(entry);
    }

    if !errors.is_empty() {
        return Ok(fail(errors));
    }

    let navigable: Vec<&Module> = modules.iter().filter(|m| !m.empty).collect();
    let mut writes: Vec<(PathBuf, Vec<u8>)> = Vec::new();

    for (dest, bytes, _) in &copies {
        writes.push((dest.clone(), bytes.clone()));
    }

    for (idx, m) in navigable.iter().enumerate() {
        let prev = if idx > 0 {
            Some(navigable[idx - 1])
        } else {
            None
        };
        let next = if idx + 1 < navigable.len() {
            Some(navigable[idx + 1])
        } else {
            None
        };
        let page = render_module_page(m, prev, next);
        writes.push((
            join_rel(root, LEARN_DIR_REL).join(format!("{}.html", m.id)),
            page.into_bytes(),
        ));
    }

    let index = render_modules_index(&modules, navigable.len());
    writes.push((join_rel(root, INDEX_REL), index.into_bytes()));

    let topics = load_topics_python_shape(&join_rel(root, TOPICS_REL), &mut errors);
    if !errors.is_empty() {
        return Ok(fail(errors));
    }
    let nav_pairs: Vec<(i64, String)> = navigable.iter().map(|m| (m.order, m.id.clone())).collect();
    let (topic_map, matched) = build_topic_map(&topics, &nav_pairs, &headings_by_domain);
    let anchors = render_topic_anchors(&topic_map, matched, &nav_pairs, &headings_by_domain);
    writes.push((join_rel(root, TOPIC_ANCHORS_REL), anchors.into_bytes()));

    writes.push((join_rel(root, HUB_REL), render_hub(&modules).into_bytes()));

    let keep_html: BTreeSet<String> = navigable.iter().map(|m| format!("{}.html", m.id)).collect();
    let keep_md: BTreeSet<String> = navigable.iter().map(|m| format!("{}.md", m.id)).collect();

    let mut unlinks = Vec::new();
    let learn_dir = join_rel(root, LEARN_DIR_REL);
    if learn_dir.is_dir() {
        let rd = std::fs::read_dir(&learn_dir)
            .map_err(|e| LearnError::io(format!("read {LEARN_DIR_REL}: {e}")))?;
        for ent in rd {
            let ent = ent.map_err(|e| LearnError::io(format!("dirent: {e}")))?;
            let path = ent.path();
            if !path.is_file() {
                continue;
            }
            let name = match path.file_name().and_then(|s| s.to_str()) {
                Some(n) if n.ends_with(".html") => n.to_string(),
                _ => continue,
            };
            if !keep_html.contains(&name) {
                unlinks.push(path);
            }
        }
    }

    let content_dir = join_rel(root, CONTENT_DIR_REL);
    let mut after_md: BTreeSet<String> = BTreeSet::new();
    if content_dir.is_dir() {
        let rd = std::fs::read_dir(&content_dir)
            .map_err(|e| LearnError::io(format!("read {CONTENT_DIR_REL}: {e}")))?;
        for ent in rd {
            let ent = ent.map_err(|e| LearnError::io(format!("dirent: {e}")))?;
            let path = ent.path();
            if !path.is_file() {
                continue;
            }
            let name = match path.file_name().and_then(|s| s.to_str()) {
                Some(n) if n.ends_with(".md") => n.to_string(),
                _ => continue,
            };
            if should_unlink_content_copy(&name, &keep_md) {
                unlinks.push(path);
            } else {
                after_md.insert(name);
            }
        }
    }
    for name in keep_md {
        after_md.insert(name);
    }
    for name in PROTECTED_CONTENT_DOCS {
        if content_dir.join(name).is_file() {
            after_md.insert((*name).to_string());
        }
    }
    if after_md.is_empty() {
        return Ok(fail(vec![
            "content dir matched zero .md files (empty sweep is ERROR)".into(),
        ]));
    }

    debug_assert_eq!(FEEDBACK_CONTENT, CONTENT_DIR_REL);

    let empty_ok = modules.iter().filter(|m| m.empty).count();
    let stdout = format!(
        "PASS: build_learn\n  modules={} navigable={} empty_ok={empty_ok}\n  index={INDEX_REL}\n  topic_anchors={TOPIC_ANCHORS_REL}\n  topics_with_anchor={matched}/{}\n  learn_pages={LEARN_DIR_REL}/\n  content={CONTENT_DIR_REL}/\n",
        modules.len(),
        navigable.len(),
        topic_map.len(),
    );
    Ok(LearnPlan {
        stdout,
        code: 0,
        writes,
        unlinks,
    })
}

/// Compile and write on the GREEN path only.
pub fn write_learn(root: &Path) -> Result<BuildOutcome, LearnError> {
    let plan = evaluate(root)?;
    debug_assert!(
        plan.code == 0 || (plan.writes.is_empty() && plan.unlinks.is_empty()),
        "a failing run must not carry writes"
    );
    if plan.code == 0 {
        apply(root, &plan)?;
    }
    Ok(plan.outcome())
}

/// Apply a GREEN plan. Creates dest dirs, writes, then unlinks.
pub fn apply(root: &Path, plan: &LearnPlan) -> Result<(), LearnError> {
    if plan.code != 0 {
        return Err(LearnError::io(
            "refusing to apply a RED Learn plan — write-after-verdict".to_string(),
        ));
    }
    for rel in [CONTENT_DIR_REL, LEARN_DIR_REL, "web/data"] {
        let dir = join_rel(root, rel);
        std::fs::create_dir_all(&dir)
            .map_err(|e| LearnError::io(format!("mkdir {}: {e}", dir.display())))?;
    }
    for (path, bytes) in &plan.writes {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| LearnError::io(format!("mkdir {}: {e}", parent.display())))?;
        }
        std::fs::write(path, bytes)
            .map_err(|e| LearnError::io(format!("write {}: {e}", path.display())))?;
    }
    for path in &plan.unlinks {
        if path.is_file() {
            std::fs::remove_file(path)
                .map_err(|e| LearnError::io(format!("unlink {}: {e}", path.display())))?;
        }
    }
    Ok(())
}

/// `\b\w+\b` over Unicode word chars (letter / digit / `_`).
pub fn word_count(text: &str) -> usize {
    let mut n = 0usize;
    let mut in_word = false;
    for c in text.chars() {
        let word = c.is_alphanumeric() || c == '_';
        if word && !in_word {
            n += 1;
        }
        in_word = word;
    }
    n
}

/// `max(15, min(55, round(words / 200 * 1.35)))` with Python 3 half-to-even.
pub fn estimate_minutes(words: usize) -> i64 {
    let raw = (words as f64) / 200.0 * 1.35;
    py_round_half_even(raw).clamp(15, 55)
}

fn py_round_half_even(x: f64) -> i64 {
    if !x.is_finite() {
        return 0;
    }
    let f = x.floor();
    let frac = x - f;
    if frac < 0.5 {
        f as i64
    } else if frac > 0.5 {
        f as i64 + 1
    } else {
        let n = f as i64;
        if n % 2 == 0 {
            n
        } else {
            n + 1
        }
    }
}

// ── domains / topics ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct Module {
    id: String,
    order: i64,
    epi_heading: String,
    primary_notes: String,
    exam_weight_unknown: bool,
    empty: bool,
    href: Option<String>,
    content_path: Option<String>,
    source_path: Option<String>,
    word_count: Option<usize>,
    estimate_minutes: Option<i64>,
}

fn load_domains(path: &Path) -> (Vec<Toml>, Vec<String>) {
    if !path.is_file() {
        return (Vec::new(), vec![format!("missing {DOMAINS_REL}")]);
    }
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => return (Vec::new(), vec![format!("{DOMAINS_REL} unreadable: {e}")]),
    };
    let parsed: Toml = match text.parse() {
        Ok(v) => v,
        Err(e) => return (Vec::new(), vec![format!("{DOMAINS_REL} invalid TOML: {e}")]),
    };
    let rows = match parsed.get("domain") {
        Some(Toml::Array(rows)) => rows.clone(),
        None => Vec::new(),
        Some(_) => {
            return (
                Vec::new(),
                vec!["domains.toml `domain` is not an array of tables".into()],
            );
        }
    };
    if rows.is_empty() {
        return (
            Vec::new(),
            vec!["domains.toml has zero [[domain]] rows".into()],
        );
    }
    let mut indexed: Vec<(i64, usize, Toml)> = rows
        .into_iter()
        .enumerate()
        .map(|(i, row)| {
            let order = row
                .as_table()
                .and_then(|t| toml_int(t.get("order")))
                .unwrap_or(0);
            (order, i, row)
        })
        .collect();
    indexed.sort_by_key(|(order, i, _)| (*order, *i));
    (
        indexed.into_iter().map(|(_, _, row)| row).collect(),
        Vec::new(),
    )
}

/// Missing topics.toml → empty map (Python builder). Invalid TOML is RED.
fn load_topics_python_shape(
    path: &Path,
    errors: &mut Vec<String>,
) -> Vec<(String, String, String)> {
    if !path.is_file() {
        return Vec::new();
    }
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            errors.push(format!("{TOPICS_REL} unreadable: {e}"));
            return Vec::new();
        }
    };
    let parsed: Toml = match text.parse() {
        Ok(v) => v,
        Err(e) => {
            errors.push(format!("{TOPICS_REL} invalid TOML: {e}"));
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
        let tid = toml_string(table.get("id"));
        if tid.is_empty() {
            continue;
        }
        let domain = toml_string(table.get("domain"));
        let label = {
            let l = toml_string(table.get("label"));
            if l.is_empty() {
                tid.clone()
            } else {
                l
            }
        };
        out.push((tid, domain, label));
    }
    out
}

fn resolve_notes(root: &Path, pn: &str) -> PathBuf {
    let p = Path::new(pn);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        root.join(p)
    }
}

fn fail(errors: Vec<String>) -> LearnPlan {
    let mut report = vec!["FAIL: build_learn".to_string()];
    for e in &errors {
        report.push(format!("  - {e}"));
    }
    LearnPlan {
        stdout: format!("{}\n", report.join("\n")),
        code: 1,
        writes: Vec::new(),
        unlinks: Vec::new(),
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

fn toml_string(v: Option<&Toml>) -> String {
    v.map(toml_as_string).unwrap_or_default().trim().to_string()
}

fn toml_int(v: Option<&Toml>) -> Option<i64> {
    match v {
        Some(Toml::Integer(i)) => Some(*i),
        Some(Toml::String(s)) => s.trim().parse().ok(),
        Some(Toml::Float(f)) if f.is_finite() => Some(*f as i64),
        _ => None,
    }
}

// ── JSON ──────────────────────────────────────────────────────────────────

fn render_modules_index(modules: &[Module], navigable: usize) -> String {
    let empty_ok = modules.iter().filter(|m| m.empty).count();
    let rows: Vec<Json> = modules.iter().map(module_to_json).collect();
    let mut root = Map::new();
    root.insert("schema_version".into(), Json::from(1u64));
    root.insert("generated_by".into(), Json::String(GENERATED_BY.into()));
    root.insert("module_count".into(), Json::from(modules.len() as u64));
    root.insert("navigable_count".into(), Json::from(navigable as u64));
    root.insert("empty_ok_count".into(), Json::from(empty_ok as u64));
    root.insert("modules".into(), Json::Array(rows));
    let mut body =
        serde_json::to_string_pretty(&Json::Object(root)).unwrap_or_else(|_| "{}".into());
    body.push('\n');
    body
}

fn module_to_json(m: &Module) -> Json {
    let mut row = Map::new();
    row.insert("id".into(), Json::String(m.id.clone()));
    row.insert("order".into(), Json::from(m.order));
    row.insert("epi_heading".into(), Json::String(m.epi_heading.clone()));
    row.insert(
        "primary_notes".into(),
        Json::String(m.primary_notes.clone()),
    );
    row.insert(
        "exam_weight_unknown".into(),
        Json::from(m.exam_weight_unknown),
    );
    row.insert("empty".into(), Json::from(m.empty));
    row.insert(
        "href".into(),
        match &m.href {
            Some(h) => Json::String(h.clone()),
            None => Json::Null,
        },
    );
    row.insert(
        "content_path".into(),
        match &m.content_path {
            Some(p) => Json::String(p.clone()),
            None => Json::Null,
        },
    );
    if let Some(p) = &m.source_path {
        row.insert("source_path".into(), Json::String(p.clone()));
    }
    if let Some(n) = m.word_count {
        row.insert("word_count".into(), Json::from(n as u64));
    }
    if let Some(n) = m.estimate_minutes {
        row.insert("estimate_minutes".into(), Json::from(n));
    }
    Json::Object(row)
}

// ── HTML ──────────────────────────────────────────────────────────────────

fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            c => out.push(c),
        }
    }
    out
}

fn json_string(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".into())
}

fn origin_script(depth: usize) -> String {
    let prefix = if depth > 0 { "../" } else { "" };
    format!("  <script src=\"{prefix}assets/js/origin_guard.js\"></script>")
}

fn header(depth: usize) -> String {
    let prefix = if depth > 0 { "../" } else { "" };
    format!(
        "  <header class=\"site-header\">\n    <div class=\"honesty-banner\" role=\"status\">\n      {HONESTY}\n    </div>\n    <div class=\"site-header__inner\">\n      <a class=\"brand\" href=\"{prefix}index.html\">\n        <span class=\"brand__title\">CDCP Study</span>\n        <span class=\"brand__sub\">Self-study · local HTTP</span>\n      </a>\n      <nav aria-label=\"Hub\">\n        <ul class=\"hub-nav\">\n          <li><a href=\"{prefix}learn.html\" aria-current=\"page\">Learn</a></li>\n          <li><a href=\"{prefix}drill.html\">Drill</a></li>\n          <li><a href=\"{prefix}mock.html\">Mock</a></li>\n          <li><a href=\"{prefix}reference.html\">Reference</a></li>\n        </ul>\n      </nav>\n    </div>\n  </header>"
    )
}

fn diagram_cta(order: i64, mod_id: &str) -> String {
    if order == 6 || mod_id == "06-power" {
        return String::from(
            "\n        <aside class=\"diagram-cta\" aria-label=\"Interactive diagram\">\n          <p class=\"diagram-cta__tag mono\">DIAGRAM</p>\n          <h2 class=\"diagram-cta__title\">Power path N vs 2N</h2>\n          <p class=\"diagram-cta__body\">Interactive label quiz for single-path vs dual-path topology.\n          Interview one-liner: dual cords only protect you if upstream paths are independent.</p>\n          <p><a class=\"diagram-cta__link\" href=\"../diagrams/power-path.html\">Open power-path self-check →</a></p>\n        </aside>",
        );
    }
    if order == 1 || mod_id == "01-mission-critical" {
        return String::from(
            "\n        <aside class=\"diagram-cta\" aria-label=\"Interactive diagram\">\n          <p class=\"diagram-cta__tag mono\">DIAGRAM</p>\n          <h2 class=\"diagram-cta__title\">Site dependency stack</h2>\n          <p class=\"diagram-cta__body\">Click layers from business impact down to MEP.\n          Interview one-liner: white space is not enough — availability is manufactured in grey space.</p>\n          <p><a class=\"diagram-cta__link\" href=\"../diagrams/site-stack.html\">Open site-stack →</a></p>\n        </aside>",
        );
    }
    if order == 9 || mod_id == "09-cooling" {
        return String::from(
            "\n        <aside class=\"diagram-cta\" aria-label=\"Interactive diagram\">\n          <p class=\"diagram-cta__tag mono\">DIAGRAM</p>\n          <h2 class=\"diagram-cta__title\">Heat path chip → outdoors</h2>\n          <p class=\"diagram-cta__body\">Stepper: IT load → rack → room → plant → outdoors.\n          Interview one-liner: every watt to IT becomes heat that must leave the building.</p>\n          <p><a class=\"diagram-cta__link\" href=\"../diagrams/heat-path.html\">Open heat-path →</a></p>\n        </aside>",
        );
    }
    // Shipped after the retired Python (bd-1sd.9.2). The compiler must keep
    // the learner-visible CTA; a rebuild that drops it is a product regression.
    if order == 13 || mod_id == "13-security" {
        return String::from(
            "\n        <aside class=\"diagram-cta\" aria-label=\"Interactive diagram\">\n          <p class=\"diagram-cta__tag mono\">DIAGRAM</p>\n          <h2 class=\"diagram-cta__title\">Security layers perimeter → white space</h2>\n          <p class=\"diagram-cta__body\">Click layers from the fence to the cage. Toggle egress vs perimeter vs EPO.\n          Interview one-liner: one strong door is not a program — grey space is as sensitive as white space.</p>\n          <p><a class=\"diagram-cta__link\" href=\"../diagrams/security-layers.html\">Open security-layers →</a></p>\n        </aside>",
        );
    }
    String::new()
}

fn render_module_page(m: &Module, prev: Option<&Module>, next: Option<&Module>) -> String {
    let title = escape_html(&m.epi_heading);
    let mid = escape_html(&m.id);
    let prev_link = match prev {
        Some(p) => format!(
            "<a class=\"mod-nav__link\" href=\"{}.html\">← {:02}. {}</a>",
            escape_html(&p.id),
            p.order,
            escape_html(&p.epi_heading)
        ),
        None => String::new(),
    };
    let next_link = match next {
        Some(n) => format!(
            "<a class=\"mod-nav__link\" href=\"{}.html\">{:02}. {} →</a>",
            escape_html(&n.id),
            n.order,
            escape_html(&n.epi_heading)
        ),
        None => String::new(),
    };
    let cta = diagram_cta(m.order, &m.id);
    let js_id = json_string(&m.id);
    format!(
        r###"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <meta name="description" content="CDCP Learn — {title}. Does not grant EPI/EXIN certification.">
  <title>CDCP Study — {title}</title>
  <link rel="stylesheet" href="../assets/css/course.css">
{origin}
</head>
<body data-module-id="{mid}">
  <a class="skip-link" href="#main">Skip to main content</a>
  <div id="learn-progress-bar" class="learn-progress-bar" role="progressbar"
       aria-valuemin="0" aria-valuemax="100" aria-valuenow="0" aria-label="Reading progress">
    <div class="learn-progress-bar__fill"></div>
    <span class="learn-progress-bar__label mono">0%</span>
  </div>

{header}

  <main id="main" class="wrap wrap-learn" tabindex="-1">
    <p class="breadcrumb">
      <a href="../learn.html">Learn</a>
      <span aria-hidden="true"> / </span>
      <span>Module {order:02}</span>
    </p>
    <noscript>
      <p class="lede">JavaScript is required to render module markdown. Source:
      <span class="mono">web/content/modules/{mid}.md</span>.</p>
    </noscript>
    <div class="learn-layout">
      <nav id="learn-toc" class="learn-toc" aria-label="On this page" hidden></nav>
      <div class="learn-layout__main">
        <div id="learn-unit-shell" class="learn-unit-shell" hidden>
          <p class="learn-unit-shell__status mono"></p>
          <h2 class="learn-unit-shell__title"></h2>
          <div class="learn-unit-shell__controls">
            <button type="button" data-unit-prev>← Prev unit</button>
            <button type="button" data-unit-next>Next unit →</button>
            <button type="button" data-unit-full>Full article</button>
            <button type="button" data-unit-mode>Unit mode</button>
          </div>
          <p class="meta">Unit mode shows one section + quick check. Full article = entire module. Study signals only.</p>
        </div>
        <article
          class="prose"
          id="module-prose"
          data-module-id="{mid}"
          data-content-href="../content/modules/{mid}.md"
          aria-busy="true"
        >
          <p class="lede">Loading module…</p>
        </article>
        <div id="learn-unit-check" class="unit-check" hidden></div>
{cta}
        <nav class="mod-nav" aria-label="Module sequence">
          <div class="mod-nav__prev">{prev_link}</div>
          <div class="mod-nav__hub"><a href="../learn.html">All modules</a></div>
          <div class="mod-nav__next">{next_link}</div>
        </nav>
        <p class="meta">
          <a href="../quiz.html?module={order}&amp;mode=learn15">Learn-15 (5 check Q)</a>
          · <a href="../quiz.html?module={order}">Module {order:02} quiz (8–12)</a>
          · study notes only · not an EPI/EXIN credential · progress in this browser.
        </p>
      </div>
    </div>
  </main>
  <script src="../assets/js/learn_md.js"></script>
  <script src="../assets/js/learn_progress.js"></script>
  <script src="../assets/js/learn_chrome.js"></script>
  <script src="../assets/js/learn_units.js"></script>
  <script src="../assets/js/learn_glossary.js"></script>
  <script src="../assets/js/learn_reader.js"></script>
  <script>
    if (window.CdcpLearnReader) {{
      CdcpLearnReader.loadAndRender({js_id});
    }}
    document.addEventListener("DOMContentLoaded", function () {{
      /* units/glossary mount after async render via learn_reader hooks */
    }});
  </script>
</body>
</html>
"###,
        title = title,
        mid = mid,
        origin = origin_script(1),
        header = header(1),
        order = m.order,
        cta = cta,
        prev_link = prev_link,
        next_link = next_link,
        js_id = js_id,
    )
}

fn render_hub(modules: &[Module]) -> String {
    let mut rows = Vec::new();
    for m in modules {
        let heading = escape_html(&m.epi_heading);
        let mid = escape_html(&m.id);
        if m.empty {
            rows.push(format!(
                "      <li class=\"module-list__item module-list__item--empty\" data-module-id=\"{mid}\">\n        <span class=\"module-list__order mono\">{:02}</span>\n        <span class=\"module-list__body\">\n          <span class=\"module-list__title\">{heading}</span>\n          <span class=\"module-list__status\">Notes not shipped (exam weight unknown)</span>\n        </span>\n        <span class=\"module-list__badge\" data-progress-for=\"{mid}\" hidden>Visited</span>\n      </li>",
                m.order
            ));
        } else {
            let href = escape_html(m.href.as_deref().unwrap_or(""));
            rows.push(format!(
                "      <li class=\"module-list__item\" data-module-id=\"{mid}\">\n        <a class=\"module-list__link\" href=\"{href}\">\n          <span class=\"module-list__order mono\">{:02}</span>\n          <span class=\"module-list__body\">\n            <span class=\"module-list__title\">{heading}</span>\n            <span class=\"module-list__status mono\">{mid}</span>\n          </span>\n          <span class=\"module-list__badge\" data-progress-for=\"{mid}\" hidden>Visited</span>\n          <span class=\"module-list__mastery\" data-mastery-for=\"{}\" hidden></span>\n        </a>\n      </li>",
                m.order, m.order
            ));
        }
    }
    let list_html = rows.join("\n");
    let slim: Vec<Json> = modules
        .iter()
        .map(|m| {
            let mut row = Map::new();
            row.insert("id".into(), Json::String(m.id.clone()));
            row.insert("order".into(), Json::from(m.order));
            row.insert("epi_heading".into(), Json::String(m.epi_heading.clone()));
            row.insert("empty".into(), Json::from(m.empty));
            row.insert(
                "href".into(),
                match &m.href {
                    Some(h) => Json::String(h.clone()),
                    None => Json::Null,
                },
            );
            row.insert(
                "word_count".into(),
                Json::from(m.word_count.unwrap_or(0) as u64),
            );
            row.insert(
                "estimate_minutes".into(),
                Json::from(m.estimate_minutes.unwrap_or(0)),
            );
            Json::Object(row)
        })
        .collect();
    let mut embed_root = Map::new();
    embed_root.insert("schema_version".into(), Json::from(1u64));
    embed_root.insert("modules".into(), Json::Array(slim));
    let index_embed = serde_json::to_string_pretty(&Json::Object(embed_root))
        .unwrap_or_else(|_| "{\n  \"schema_version\": 1,\n  \"modules\": []\n}".into());

    format!(
        r###"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <meta name="description" content="CDCP Learn — study modules. Does not grant EPI/EXIN certification.">
  <title>CDCP Study — Learn</title>
  <link rel="stylesheet" href="assets/css/course.css">
{origin}
</head>
<body>
  <a class="skip-link" href="#main">Skip to main content</a>

{header}

  <main id="main" class="wrap" tabindex="-1">
    <h1>Learn</h1>
    <p class="lede">
      Fourteen EPI CDCP curriculum domains plus partner ops expansions.
      Open a module to study over local HTTP. Progress is stored only in this browser.
      Completing modules here is a study signal — not a CDCP credential.
    </p>

    <p class="learn-progress-summary" id="learn-progress-summary" aria-live="polite"></p>

    <p id="learn-continue" class="learn-continue" hidden>
      <a class="learn-continue__link" href="#">Continue</a>
      <span class="meta"> · last module in this browser (study signal only)</span>
    </p>

    <ol class="module-list" id="module-list">
{list_html}
    </ol>

    <p class="meta">
      <a href="index.html">← Hub</a>
      · <a href="quiz.html">Module quiz</a>
      · <a href="drill.html">Drill</a>
      · Generated from <span class="mono">knowledge/domains.toml</span>
    </p>
  </main>

  <script type="application/json" id="modules-index">
{index_embed}
  </script>
  <script src="assets/js/learn_progress.js"></script>
  <script src="assets/js/learn_chrome.js"></script>
  <script type="module" src="assets/js/hub_mastery.js"></script>
  <script>
    if (window.CdcpLearn) {{
      CdcpLearn.paintHub();
    }}
    if (window.CdcpLearnChrome) {{
      CdcpLearnChrome.loadHubExtras();
    }}
  </script>
</body>
</html>
"###,
        origin = origin_script(0),
        header = header(0),
        list_html = list_html,
        index_embed = index_embed,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_count_splits_on_non_word() {
        assert_eq!(word_count("one two  three"), 3);
        assert_eq!(word_count("N+1 / 2N"), 3);
        assert_eq!(word_count(""), 0);
    }

    #[test]
    fn estimate_minutes_clamps_and_rounds() {
        assert_eq!(estimate_minutes(0), 15);
        assert_eq!(estimate_minutes(3484), 24);
        assert_eq!(estimate_minutes(20_000), 55);
    }

    #[test]
    fn html_escape_matches_python_quote_true() {
        assert_eq!(escape_html(r#"a&b<c>"d""#), "a&amp;b&lt;c&gt;&quot;d&quot;");
    }
}
