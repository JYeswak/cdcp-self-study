//! build-learn-slugs — compile `web/data/module_learn_slugs.js`.
//!
//! The Results page has no TOML reader, so the shipped `MODULE_LEARN_SLUGS`
//! map has to be a literal. Until bd-we5a that literal lived hand-frozen in
//! `web/assets/js/results.js`. The Learn build now derives it from
//! `knowledge/domains.toml` and writes this artifact; `results.js` imports
//! it. The map is PRODUCT (a learner follows Results → Learn through it).
//!
//! # Contract
//!
//! * every `[[domain]]` with a usable `order` and `id` is emitted
//! * a registry module with no id is RED, naming the module — there is
//!   no slug to emit
//! * an empty emitted map is an ERROR (anti-vacuous)
//! * a missing, unreadable or unparseable registry is an ERROR
//! * write-after-verdict: a RED compile writes nothing
//! * emitted bytes are stable across runs
//!
//! The fourteen-public-domain FLOOR lives on the weak-link smoke, not here.
//! This is a BUILDER: it emits what the registry declares. A legitimate
//! fourteen-module tree is GREEN. A collapsed registry that still has rows
//! is still emitted; the smoke is what refuses to agree with a silent
//! three-module set.
//!
//! Slug strings in this file are FORMATTED, not enumerated, so the bd-ggs7
//! frozen-table detector does not mistake the compiler for a product map.

#![forbid(unsafe_code)]

use crate::{join_rel, BuildOutcome, LearnError};
use std::collections::BTreeMap;
use std::path::Path;
use toml::Value as Toml;

pub const NAME: &str = "build-learn-slugs";
pub const SUMMARY: &str = "build web/data/module_learn_slugs.js from knowledge/domains.toml";

/// Engine-root-relative paths.
pub const DOMAINS_REL: &str = "knowledge/domains.toml";
pub const OUT_REL: &str = "web/data/module_learn_slugs.js";

/// The product file `results.js` imports. Same bytes as `OUT_REL`.
pub const SLUGS_JS_REL: &str = OUT_REL;

pub type Outcome = BuildOutcome;

/// Compile the slug map. Does not write. A RED compile carries no artifact.
pub fn evaluate(root: &Path) -> Result<Outcome, LearnError> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let out = join_rel(&root, OUT_REL);

    let (declared, mut errors) = load_declared_slugs(&join_rel(&root, DOMAINS_REL));

    if declared.is_empty() && errors.is_empty() {
        errors.push("MODULE_LEARN_SLUGS empty — refusing vacuous green".into());
    }

    let body = if errors.is_empty() {
        let rendered = render(&declared);
        match check_emitted(&declared, &rendered) {
            Ok(()) => Some(rendered),
            Err(more) => {
                errors.extend(more);
                None
            }
        }
    } else {
        None
    };

    if !errors.is_empty() {
        let mut report = vec![format!(
            "FAIL: build_learn_slugs modules={}",
            declared.len()
        )];
        for e in &errors {
            report.push(format!("  - {e}"));
        }
        report.push(format!(
            "  out={OUT_REL} NOT WRITTEN (a failing build leaves no artifact)"
        ));
        return Ok(Outcome {
            stdout: format!("{}\n", report.join("\n")),
            code: 1,
            artifact: None,
        });
    }

    let body = body.expect("green path carries rendered bytes");
    Ok(Outcome {
        stdout: format!(
            "PASS: build_learn_slugs modules={} → {OUT_REL}\n",
            declared.len()
        ),
        code: 0,
        artifact: Some((out, body)),
    })
}

/// Compile and write the artifact on the GREEN path only.
pub fn write_slugs(root: &Path) -> Result<Outcome, LearnError> {
    let outcome = evaluate(root)?;
    debug_assert!(
        outcome.code == 0 || outcome.artifact.is_none(),
        "a failing run must not carry an artifact"
    );
    if outcome.code == 0 {
        if let Some((path, body)) = &outcome.artifact {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| LearnError::io(format!("mkdir {}: {e}", parent.display())))?;
            }
            std::fs::write(path, body.as_bytes())
                .map_err(|e| LearnError::io(format!("write {}: {e}", path.display())))?;
        }
    }
    Ok(outcome)
}

/// `{module_number: learn_slug}` the compiler will emit.
///
/// A missing, malformed or empty registry yields errors — never a silent
/// empty set that would make an empty map look like a clean compile.
pub fn load_declared_slugs(domains_path: &Path) -> (BTreeMap<i64, String>, Vec<String>) {
    let mut errors = Vec::new();
    let mut declared = BTreeMap::new();
    if !domains_path.is_file() {
        return (
            declared,
            vec![format!("domain registry missing: {DOMAINS_REL}")],
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
                "module {}: knowledge/domains.toml has no usable order — no slug to emit",
                if did.is_empty() {
                    "<missing-id>"
                } else {
                    did.as_str()
                }
            ));
            continue;
        };
        if did.is_empty() {
            // THE KNOWN-BAD: a registry module with no emitted slug.
            errors.push(format!(
                "module {order}: knowledge/domains.toml has no id — no slug to emit"
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
        errors.push("MODULE_LEARN_SLUGS empty — refusing vacuous green".into());
    }
    (declared, errors)
}

/// The shipped ES module. Stable: BTreeMap order, trailing comma, one
/// trailing newline. Header names the source; it does not enumerate slugs.
pub fn render(declared: &BTreeMap<i64, String>) -> String {
    let mut s = String::from(concat!(
        "/**\n",
        " * @generated by cdcp_learn from knowledge/domains.toml — do not edit.\n",
        " * Results → Learn slug map. Regenerated by `cdcp build-learn-slugs`.\n",
        " */\n",
        "export const MODULE_LEARN_SLUGS = Object.freeze({\n",
    ));
    for (n, slug) in declared {
        s.push_str("  ");
        s.push_str(&n.to_string());
        s.push_str(": ");
        push_js_string(&mut s, slug);
        s.push_str(",\n");
    }
    s.push_str("});\n");
    s
}

/// Read `MODULE_LEARN_SLUGS` from the generated artifact under `root`.
pub fn load_module_learn_slugs(root: &Path) -> Result<BTreeMap<i64, String>, String> {
    let path = join_rel(root, SLUGS_JS_REL);
    if !path.is_file() {
        return Err(format!(
            "missing {SLUGS_JS_REL} — run `cdcp build-learn-slugs`"
        ));
    }
    let text =
        std::fs::read_to_string(&path).map_err(|e| format!("{SLUGS_JS_REL} unreadable: {e}"))?;
    parse_module_learn_slugs(&text)
}

/// Parse `MODULE_LEARN_SLUGS = Object.freeze({ N: "slug", ... })`.
pub fn parse_module_learn_slugs(js_text: &str) -> Result<BTreeMap<i64, String>, String> {
    let Some(idx) = js_text.find("MODULE_LEARN_SLUGS") else {
        return Err("MODULE_LEARN_SLUGS not found".into());
    };
    let rest = &js_text[idx..];
    let Some(brace) = rest.find('{') else {
        return Err("MODULE_LEARN_SLUGS not found".into());
    };
    let body = &rest[brace + 1..];
    let Some(end) = body.find('}') else {
        return Err("MODULE_LEARN_SLUGS not found".into());
    };
    let inner = &body[..end];
    let mut found = BTreeMap::new();
    let chars: Vec<char> = inner.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        while i < chars.len() && !chars[i].is_ascii_digit() {
            i += 1;
        }
        if i >= chars.len() {
            break;
        }
        let start = i;
        while i < chars.len() && chars[i].is_ascii_digit() {
            i += 1;
        }
        let n: i64 = inner[start..start + (i - start)]
            .parse()
            .map_err(|_| "MODULE_LEARN_SLUGS has a non-integer key".to_string())?;
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        if i >= chars.len() || chars[i] != ':' {
            continue;
        }
        i += 1;
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        if i >= chars.len() {
            break;
        }
        let quote = chars[i];
        if quote != '"' && quote != '\'' {
            continue;
        }
        i += 1;
        let mut slug = String::new();
        while i < chars.len() && chars[i] != quote {
            slug.push(chars[i]);
            i += 1;
        }
        if i < chars.len() {
            i += 1;
        }
        found.insert(n, slug);
    }
    Ok(found)
}

fn check_emitted(declared: &BTreeMap<i64, String>, body: &str) -> Result<(), Vec<String>> {
    let parsed = match parse_module_learn_slugs(body) {
        Ok(m) => m,
        Err(e) => return Err(vec![e]),
    };
    let mut errors = Vec::new();
    if parsed.is_empty() {
        errors.push("MODULE_LEARN_SLUGS empty — refusing vacuous green".into());
    }
    for (n, expect) in declared {
        match parsed.get(n) {
            Some(got) if got == expect => {}
            None => errors.push(format!(
                "module {n}: knowledge/domains.toml declares '{expect}' but \
                 emitted MODULE_LEARN_SLUGS has no entry"
            )),
            Some(got) => errors.push(format!(
                "module {n}: emitted slug '{got}' != declared '{expect}'"
            )),
        }
    }
    for n in parsed.keys().filter(|n| !declared.contains_key(*n)) {
        errors.push(format!(
            "module {n}: emitted slug '{}' is not declared in {DOMAINS_REL}",
            parsed[n]
        ));
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn push_js_string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            c => out.push(c),
        }
    }
    out.push('"');
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_slugs_reads_the_frozen_object() {
        let js = r#"
export const MODULE_LEARN_SLUGS = Object.freeze({
  1: "01-mission-critical",
  6: '06-power',
});
"#;
        let slugs = parse_module_learn_slugs(js).unwrap();
        assert_eq!(
            slugs.get(&1).map(String::as_str),
            Some("01-mission-critical")
        );
        assert_eq!(slugs.get(&6).map(String::as_str), Some("06-power"));
        assert!(parse_module_learn_slugs("no map here").is_err());
    }

    #[test]
    fn render_round_trips_through_the_parser() {
        let mut declared = BTreeMap::new();
        declared.insert(1, format!("{:02}-mod", 1));
        declared.insert(2, format!("{:02}-mod", 2));
        let body = render(&declared);
        assert!(body.contains("@generated by cdcp_learn"));
        assert!(body.contains("do not edit"));
        let parsed = parse_module_learn_slugs(&body).unwrap();
        assert_eq!(parsed, declared);
        check_emitted(&declared, &body).unwrap();
    }

    #[test]
    fn an_empty_render_fails_the_emitted_check() {
        let declared = BTreeMap::new();
        let body = render(&declared);
        let err = check_emitted(&declared, &body).unwrap_err();
        assert!(
            err.iter().any(|e| e.contains("MODULE_LEARN_SLUGS empty")),
            "{err:?}"
        );
    }
}
