//! L1 claims constitution checker (Assessment-System).
//!
//! Loads `registries/`, validates the strength lattice, ensures
//! `knowledge/claims.toml` rows resolve, and runs claims-lint over prose.
//! Empty registry / empty claim set is always an ERROR (never vacuous green).
#![forbid(unsafe_code)]

use regex::Regex;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub mod anti_vacuous;
pub mod count_pins;
pub mod doc_facts;
mod gate_shrink;
pub mod scratch;
pub mod shell_walk;
pub mod track;
pub mod verify_doc_consistency;

/// Rank lattice: invariant(6) > proof(5) > bounded_model(4) > statistical(3) > slo(2) > benchmark(1).
pub const CANONICAL_CLASSES: &[(&str, u8)] = &[
    ("invariant", 6),
    ("proof", 5),
    ("bounded_model", 4),
    ("statistical", 3),
    ("slo", 2),
    ("benchmark", 1),
];

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CheckError {
    #[error("{0}")]
    Msg(String),
}

impl CheckError {
    fn msg(s: impl Into<String>) -> Self {
        Self::Msg(s.into())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClaimsFile {
    pub schema_version: u32,
    #[serde(default)]
    pub claim_class: Vec<ClaimClass>,
    #[serde(default)]
    pub claim: Vec<ClaimRow>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClaimClass {
    pub name: String,
    pub rank: u8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClaimRow {
    pub id: String,
    pub strength: String,
    pub text: String,
    #[serde(default)]
    pub justified_by: Vec<String>,
    #[serde(default)]
    pub enforcement: Option<String>,
    #[serde(default)]
    pub evidence: Option<String>,
    #[serde(default)]
    pub forbidden_surface: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KnowledgeClaimsFile {
    pub schema_version: u32,
    #[serde(default)]
    pub claim: Vec<KnowledgeClaimRow>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KnowledgeClaimRow {
    pub id: String,
    pub strength: String,
    pub text: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ObjectivesFile {
    pub schema_version: u32,
    #[serde(default)]
    pub objective: Vec<ObjectiveRow>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ObjectiveRow {
    pub id: String,
    pub text: String,
    #[serde(default)]
    pub claim_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClaimsLintFile {
    pub schema_version: u32,
    pub scan: ScanConfig,
    #[serde(default)]
    pub exclude: Vec<ExcludeRow>,
    #[serde(default)]
    pub load_bearing: Vec<LoadBearing>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScanConfig {
    pub marker_prefix: String,
    pub marker_suffix: String,
    pub roots: Vec<String>,
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExcludeRow {
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoadBearing {
    pub id: String,
    pub pattern: String,
    pub must_cite: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Violation {
    pub code: String,
    pub message: String,
}

impl Violation {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

/// Resolve repo root: walk up from `start` until `registries/claims.toml` exists.
///
/// No env-overridable repo root (that was D6). No compile-time crate-directory
/// fallback. Walk budget is the unified [`cdcp_root::WALK_LEVELS`] (12), not
/// the old 8-level cut.
pub fn resolve_repo_root(start: &Path) -> Result<PathBuf, CheckError> {
    cdcp_root::walk_engine_root(start).map_err(|e| CheckError::msg(e.to_string()))
}

pub fn load_toml<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, CheckError> {
    let text = fs::read_to_string(path)
        .map_err(|e| CheckError::msg(format!("read {}: {e}", path.display())))?;
    toml::from_str(&text).map_err(|e| CheckError::msg(format!("parse {}: {e}", path.display())))
}

pub fn class_rank(classes: &BTreeMap<String, u8>, name: &str) -> Result<u8, CheckError> {
    classes
        .get(name)
        .copied()
        .ok_or_else(|| CheckError::msg(format!("unknown claim strength/class: {name}")))
}

/// Validate claim classes against the canonical lattice.
pub fn validate_claim_classes(classes: &[ClaimClass]) -> Vec<Violation> {
    let mut v = Vec::new();
    if classes.is_empty() {
        v.push(Violation::new(
            "empty_claim_classes",
            "registries/claims.toml has zero [[claim_class]] rows",
        ));
        return v;
    }
    let mut by_name: BTreeMap<String, u8> = BTreeMap::new();
    for c in classes {
        if by_name.insert(c.name.clone(), c.rank).is_some() {
            v.push(Violation::new(
                "duplicate_claim_class",
                format!("duplicate claim_class name {}", c.name),
            ));
        }
    }
    for &(name, rank) in CANONICAL_CLASSES {
        match by_name.get(name) {
            None => v.push(Violation::new(
                "missing_claim_class",
                format!("required claim_class {name} (rank {rank}) missing"),
            )),
            Some(&r) if r != rank => v.push(Violation::new(
                "claim_class_rank_mismatch",
                format!("claim_class {name} rank {r} != canonical {rank}"),
            )),
            _ => {}
        }
    }
    v
}

/// Validate claim rows: non-empty, unique ids, known strength, lattice on justified_by.
pub fn validate_claims(claims: &[ClaimRow], classes: &BTreeMap<String, u8>) -> Vec<Violation> {
    let mut v = Vec::new();
    if claims.is_empty() {
        v.push(Violation::new(
            "empty_claims",
            "registries/claims.toml has zero [[claim]] rows (empty registry = ERROR)",
        ));
        return v;
    }
    let mut ids: BTreeSet<String> = BTreeSet::new();
    for c in claims {
        if c.id.trim().is_empty() {
            v.push(Violation::new("empty_claim_id", "claim with empty id"));
            continue;
        }
        if !ids.insert(c.id.clone()) {
            v.push(Violation::new(
                "duplicate_claim_id",
                format!("duplicate claim id {}", c.id),
            ));
        }
        if c.text.trim().is_empty() {
            v.push(Violation::new(
                "empty_claim_text",
                format!("claim {} has empty text", c.id),
            ));
        }
        if class_rank(classes, &c.strength).is_err() {
            v.push(Violation::new(
                "unknown_strength",
                format!("claim {} strength {:?} not in lattice", c.id, c.strength),
            ));
        }
    }
    let by_id: BTreeMap<&str, &ClaimRow> = claims.iter().map(|c| (c.id.as_str(), c)).collect();
    for c in claims {
        let Ok(claim_rank) = class_rank(classes, &c.strength) else {
            continue;
        };
        for jid in &c.justified_by {
            match by_id.get(jid.as_str()) {
                None => v.push(Violation::new(
                    "justifier_missing",
                    format!("claim {} justified_by unknown id {jid}", c.id),
                )),
                Some(j) => {
                    if let Ok(jr) = class_rank(classes, &j.strength) {
                        if jr < claim_rank {
                            v.push(Violation::new(
                                "lattice_violation",
                                format!(
                                    "claim {} (rank {claim_rank}) justified_by {} (rank {jr}); rank(justifier) must be >= rank(claim)",
                                    c.id, jid
                                ),
                            ));
                        }
                    }
                }
            }
        }
    }
    // Required honesty spine
    for required in [
        "claim-not-epi-certified",
        "claim-study-signal-27",
        "claim-interview-ready",
        "claim-domain-covered",
        "claim-forbidden-dump-bank",
    ] {
        if !ids.contains(required) {
            v.push(Violation::new(
                "missing_required_claim",
                format!("required honesty/coverage claim {required} missing from registries/claims.toml"),
            ));
        }
    }
    v
}

/// knowledge/claims.toml rows must resolve to registry claims with matching strength.
pub fn validate_knowledge_claims(
    knowledge: &[KnowledgeClaimRow],
    registry: &[ClaimRow],
    classes: &BTreeMap<String, u8>,
) -> Vec<Violation> {
    let mut v = Vec::new();
    if knowledge.is_empty() {
        v.push(Violation::new(
            "empty_knowledge_claims",
            "knowledge/claims.toml has zero [[claim]] rows",
        ));
        return v;
    }
    let reg: BTreeMap<&str, &ClaimRow> = registry.iter().map(|c| (c.id.as_str(), c)).collect();
    for k in knowledge {
        if class_rank(classes, &k.strength).is_err() {
            v.push(Violation::new(
                "knowledge_unknown_strength",
                format!("knowledge claim {} strength {:?}", k.id, k.strength),
            ));
        }
        match reg.get(k.id.as_str()) {
            None => v.push(Violation::new(
                "knowledge_unresolved",
                format!(
                    "knowledge/claims.toml id {} does not resolve to registries/claims.toml",
                    k.id
                ),
            )),
            Some(r) if r.strength != k.strength => v.push(Violation::new(
                "knowledge_strength_mismatch",
                format!(
                    "knowledge claim {} strength {:?} != registry {:?}",
                    k.id, k.strength, r.strength
                ),
            )),
            _ => {}
        }
    }
    v
}

pub fn validate_objectives(
    objectives: &[ObjectiveRow],
    claim_ids: &BTreeSet<String>,
) -> Vec<Violation> {
    let mut v = Vec::new();
    if objectives.is_empty() {
        v.push(Violation::new(
            "empty_objectives",
            "registries/objectives.toml has zero [[objective]] rows",
        ));
        return v;
    }
    let mut seen = BTreeSet::new();
    for o in objectives {
        if !seen.insert(o.id.clone()) {
            v.push(Violation::new(
                "duplicate_objective",
                format!("duplicate objective {}", o.id),
            ));
        }
        if o.claim_ids.is_empty() {
            v.push(Violation::new(
                "objective_no_claims",
                format!("objective {} cites zero claims", o.id),
            ));
        }
        for cid in &o.claim_ids {
            if !claim_ids.contains(cid) {
                v.push(Violation::new(
                    "objective_unresolved_claim",
                    format!("objective {} cites unknown claim {cid}", o.id),
                ));
            }
        }
    }
    v
}

fn collect_markdown_files(root: &Path, scan: &ScanConfig) -> Result<Vec<PathBuf>, CheckError> {
    let mut out = Vec::new();
    for r in &scan.roots {
        let p = root.join(r);
        if p.is_file() {
            out.push(p);
            continue;
        }
        if p.is_dir() {
            walk_md(&p, &scan.extensions, &mut out)?;
            continue;
        }
        // Parent-corpus roots (`../README.md`, `../CHARTER.md`) exist in a
        // full checkout. The prove-wired probe materialises only the engine
        // prefix, so those paths are absent there. Skip them. A missing
        // in-engine root is still an ERROR. Zero collected files is ERROR.
        if r.starts_with("../") || r.starts_with("..\\") {
            continue;
        }
        return Err(CheckError::msg(format!(
            "claims_lint scan root missing: {r}"
        )));
    }
    out.sort();
    out.dedup();
    if out.is_empty() {
        return Err(CheckError::msg(
            "claims_lint scan collected 0 files — empty scan is an ERROR, not a pass",
        ));
    }
    Ok(out)
}

fn walk_md(dir: &Path, exts: &[String], out: &mut Vec<PathBuf>) -> Result<(), CheckError> {
    let entries = fs::read_dir(dir)
        .map_err(|e| CheckError::msg(format!("read_dir {}: {e}", dir.display())))?;
    for ent in entries {
        let ent = ent.map_err(|e| CheckError::msg(format!("dirent: {e}")))?;
        let path = ent.path();
        if path.is_dir() {
            walk_md(&path, exts, out)?;
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let dotted = format!(".{ext}");
            if exts.iter().any(|e| e == &dotted || e == ext) {
                out.push(path);
            }
        }
    }
    Ok(())
}

/// Extract claim markers from text using configured prefix/suffix.
pub fn extract_markers(text: &str, prefix: &str, suffix: &str) -> Vec<String> {
    let mut ids = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find(prefix) {
        let after = &rest[start + prefix.len()..];
        if let Some(end) = after.find(suffix) {
            let id = after[..end].trim();
            if !id.is_empty() {
                ids.push(id.to_string());
            }
            rest = &after[end + suffix.len()..];
        } else {
            break;
        }
    }
    ids
}

fn markdown_table_row(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.matches('|').count() >= 2 && (trimmed.starts_with('|') || trimmed.ends_with('|'))
}

fn markdown_table_separator(line: &str) -> bool {
    let cells = line
        .split('|')
        .map(str::trim)
        .filter(|cell| !cell.is_empty())
        .collect::<Vec<_>>();
    cells.len() >= 2
        && cells.iter().all(|cell| {
            let cell = cell.trim_matches(':').trim();
            cell.len() >= 3 && cell.chars().all(|ch| ch == '-' || ch.is_whitespace())
        })
}

fn strip_inline_quoted_content(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut closing_quote = None;
    let mut escaped = false;
    for ch in line.chars() {
        if let Some(closing) = closing_quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == closing {
                closing_quote = None;
            }
            continue;
        }

        let opening = match ch {
            '"' | '`' => Some(ch),
            '“' => Some('”'),
            '‘' => Some('’'),
            '\'' if out
                .chars()
                .last()
                .is_none_or(|previous| !previous.is_alphanumeric()) =>
            {
                Some('\'')
            }
            _ => None,
        };
        if let Some(closing) = opening {
            closing_quote = Some(closing);
        } else {
            out.push(ch);
        }
    }
    out
}

/// Return the authored-prose view used for load-bearing phrase matching.
///
/// Markdown tables, block quotes, fenced code, and inline quoted/code spans
/// commonly carry copied curriculum data rather than claims made by the
/// receipt author. Markers still scan the original text; only phrase-class
/// matching uses this narrower view.
pub fn authored_prose(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut fenced = false;
    let lines = text.lines().collect::<Vec<_>>();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            fenced = !fenced;
            index += 1;
            continue;
        }
        if !fenced
            && index + 1 < lines.len()
            && line.contains('|')
            && markdown_table_separator(lines[index + 1])
        {
            index += 2;
            while index < lines.len() && lines[index].contains('|') {
                index += 1;
            }
            continue;
        }
        if fenced
            || trimmed.starts_with('>')
            || markdown_table_row(line)
            || markdown_table_separator(line)
        {
            index += 1;
            continue;
        }
        out.push_str(&strip_inline_quoted_content(line));
        out.push('\n');
        index += 1;
    }
    out
}

pub fn validate_claims_lint(
    root: &Path,
    lint: &ClaimsLintFile,
    claim_ids: &BTreeSet<String>,
) -> Result<Vec<Violation>, CheckError> {
    let mut v = Vec::new();

    // Exclusions must have non-empty reasons; missing reason = schema error
    let mut excluded: BTreeSet<String> = BTreeSet::new();
    for ex in &lint.exclude {
        if ex.reason.trim().is_empty() {
            v.push(Violation::new(
                "exclude_no_reason",
                format!("exclude path {} has empty reason", ex.path),
            ));
        }
        let full = root.join(&ex.path);
        if !full.exists() {
            v.push(Violation::new(
                "exclude_missing_path",
                format!("exclude path {} does not exist", ex.path),
            ));
        }
        excluded.insert(ex.path.replace('\\', "/"));
    }

    if lint.load_bearing.is_empty() {
        v.push(Violation::new(
            "empty_load_bearing",
            "claims_lint.toml has zero [[load_bearing]] rows",
        ));
    }

    let files = collect_markdown_files(root, &lint.scan)?;
    if files.is_empty() {
        v.push(Violation::new(
            "empty_scan_set",
            "claims-lint scan set is empty (never vacuous green)",
        ));
        return Ok(v);
    }

    // Compile load-bearing patterns
    let mut patterns: Vec<(&LoadBearing, Regex)> = Vec::new();
    for lb in &lint.load_bearing {
        match Regex::new(&lb.pattern) {
            Ok(re) => patterns.push((lb, re)),
            Err(e) => v.push(Violation::new(
                "bad_load_bearing_pattern",
                format!("load_bearing {} pattern: {e}", lb.id),
            )),
        }
        for cid in &lb.must_cite {
            if !claim_ids.contains(cid) {
                v.push(Violation::new(
                    "load_bearing_unknown_claim",
                    format!("load_bearing {} must_cite unknown claim {cid}", lb.id),
                ));
            }
        }
    }

    let prefix = &lint.scan.marker_prefix;
    let suffix = &lint.scan.marker_suffix;

    for file in &files {
        let rel = file
            .strip_prefix(root)
            .unwrap_or(file)
            .to_string_lossy()
            .replace('\\', "/");
        let text = fs::read_to_string(file)
            .map_err(|e| CheckError::msg(format!("read {}: {e}", file.display())))?;

        // Direction 1: every marker must resolve
        for mid in extract_markers(&text, prefix, suffix) {
            if !claim_ids.contains(&mid) {
                v.push(Violation::new(
                    "marker_unresolved",
                    format!("{rel}: marker [[claim:{mid}]] does not resolve to a registered claim"),
                ));
            }
        }

        if excluded.contains(&rel) {
            continue;
        }

        // Direction 2: load-bearing phrases need a must_cite marker in the same file
        let file_markers: BTreeSet<String> =
            extract_markers(&text, prefix, suffix).into_iter().collect();
        let prose = authored_prose(&text);
        for (lb, re) in &patterns {
            if re.is_match(&prose) {
                let ok = lb.must_cite.iter().any(|c| file_markers.contains(c));
                if !ok {
                    v.push(Violation::new(
                        "load_bearing_uncited",
                        format!(
                            "{rel}: load-bearing phrase class '{}' matched but file lacks markers for {:?}",
                            lb.id, lb.must_cite
                        ),
                    ));
                }
            }
        }
    }

    Ok(v)
}

/// Full L1 check against a repo root.
pub fn check_repo(root: &Path) -> Result<Vec<Violation>, CheckError> {
    let mut v = Vec::new();

    let claims_path = root.join("registries/claims.toml");
    if !claims_path.is_file() {
        return Ok(vec![Violation::new(
            "missing_registry",
            "registries/claims.toml missing (empty/deleted registry = ERROR)",
        )]);
    }

    let claims_file: ClaimsFile = load_toml(&claims_path)?;
    if claims_file.schema_version != 1 {
        v.push(Violation::new(
            "schema_version",
            format!(
                "registries/claims.toml schema_version {} (expected 1)",
                claims_file.schema_version
            ),
        ));
    }

    v.extend(validate_claim_classes(&claims_file.claim_class));

    let class_map: BTreeMap<String, u8> = claims_file
        .claim_class
        .iter()
        .map(|c| (c.name.clone(), c.rank))
        .collect();

    v.extend(validate_claims(&claims_file.claim, &class_map));

    let claim_ids: BTreeSet<String> = claims_file.claim.iter().map(|c| c.id.clone()).collect();

    // knowledge/claims.toml
    let knowledge_path = root.join("knowledge/claims.toml");
    if !knowledge_path.is_file() {
        v.push(Violation::new(
            "missing_knowledge_claims",
            "knowledge/claims.toml missing",
        ));
    } else {
        let knowledge: KnowledgeClaimsFile = load_toml(&knowledge_path)?;
        v.extend(validate_knowledge_claims(
            &knowledge.claim,
            &claims_file.claim,
            &class_map,
        ));
    }

    // objectives
    let obj_path = root.join("registries/objectives.toml");
    if !obj_path.is_file() {
        v.push(Violation::new(
            "missing_objectives",
            "registries/objectives.toml missing",
        ));
    } else {
        let objectives: ObjectivesFile = load_toml(&obj_path)?;
        v.extend(validate_objectives(&objectives.objective, &claim_ids));
    }

    // claims-lint
    let lint_path = root.join("registries/claims_lint.toml");
    if !lint_path.is_file() {
        v.push(Violation::new(
            "missing_claims_lint",
            "registries/claims_lint.toml missing",
        ));
    } else {
        let lint: ClaimsLintFile = load_toml(&lint_path)?;
        // exclusion reason non-empty already checked inside
        for ex in &lint.exclude {
            if ex.reason.trim().is_empty() {
                v.push(Violation::new(
                    "exclude_no_reason",
                    format!("exclude {} missing reason", ex.path),
                ));
            }
        }
        v.extend(validate_claims_lint(root, &lint, &claim_ids)?);
    }

    Ok(v)
}

/// Run check and return process-style result.
pub fn run(root: &Path) -> Result<(), CheckError> {
    let violations = check_repo(root)?;
    let shrink = gate_shrink::check_gate_shrink(root);
    if let Err(ref e) = shrink {
        eprintln!("cdcp_registry_check: {e}");
    }
    if violations.is_empty() && shrink.is_ok() {
        println!("cdcp_registry_check: OK (L1 claims constitution green)");
        Ok(())
    } else if violations.is_empty() {
        Err(shrink.unwrap_err())
    } else {
        for viol in &violations {
            eprintln!("cdcp_registry_check: {} — {}", viol.code, viol.message);
        }
        eprintln!(
            "cdcp_registry_check: FAIL ({} violation(s))",
            violations.len()
        );
        Err(CheckError::msg(format!(
            "{} claim-registry violation(s)",
            violations.len()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn sample_classes() -> Vec<ClaimClass> {
        CANONICAL_CLASSES
            .iter()
            .map(|(n, r)| ClaimClass {
                name: (*n).to_string(),
                rank: *r,
            })
            .collect()
    }

    fn class_map() -> BTreeMap<String, u8> {
        CANONICAL_CLASSES
            .iter()
            .map(|(n, r)| ((*n).to_string(), *r))
            .collect()
    }

    fn required_claims() -> Vec<ClaimRow> {
        vec![
            ClaimRow {
                id: "claim-not-epi-certified".into(),
                strength: "invariant".into(),
                text: "not certified".into(),
                justified_by: vec![],
                enforcement: None,
                evidence: None,
                forbidden_surface: Some("epi_certified".into()),
            },
            ClaimRow {
                id: "claim-study-signal-27".into(),
                strength: "benchmark".into(),
                text: "27/40 study signal".into(),
                justified_by: vec![],
                enforcement: None,
                evidence: None,
                forbidden_surface: None,
            },
            ClaimRow {
                id: "claim-interview-ready".into(),
                strength: "benchmark".into(),
                text: "interview ready".into(),
                justified_by: vec!["claim-study-signal-27".into()],
                enforcement: None,
                evidence: None,
                forbidden_surface: None,
            },
            ClaimRow {
                id: "claim-domain-covered".into(),
                strength: "slo".into(),
                text: "domain covered".into(),
                justified_by: vec![],
                enforcement: None,
                evidence: None,
                forbidden_surface: None,
            },
            ClaimRow {
                id: "claim-forbidden-dump-bank".into(),
                strength: "invariant".into(),
                text: "no dumps".into(),
                justified_by: vec![],
                enforcement: None,
                evidence: None,
                forbidden_surface: None,
            },
        ]
    }

    #[test]
    fn empty_claims_is_error() {
        let v = validate_claims(&[], &class_map());
        assert!(
            v.iter().any(|x| x.code == "empty_claims"),
            "empty registry must ERROR: {v:?}"
        );
    }

    #[test]
    fn empty_classes_is_error() {
        let v = validate_claim_classes(&[]);
        assert!(v.iter().any(|x| x.code == "empty_claim_classes"));
    }

    #[test]
    fn lattice_rejects_weaker_justifier() {
        let mut claims = required_claims();
        // invariant justified by benchmark — forbidden
        claims[0].justified_by = vec!["claim-study-signal-27".into()];
        let v = validate_claims(&claims, &class_map());
        assert!(
            v.iter().any(|x| x.code == "lattice_violation"),
            "expected lattice_violation, got {v:?}"
        );
    }

    #[test]
    fn lattice_allows_equal_or_stronger_justifier() {
        let mut claims = required_claims();
        // benchmark justified by invariant — OK
        claims.push(ClaimRow {
            id: "claim-extra".into(),
            strength: "benchmark".into(),
            text: "extra".into(),
            justified_by: vec!["claim-not-epi-certified".into()],
            enforcement: None,
            evidence: None,
            forbidden_surface: None,
        });
        let v = validate_claims(&claims, &class_map());
        assert!(
            !v.iter().any(|x| x.code == "lattice_violation"),
            "unexpected lattice violations: {v:?}"
        );
    }

    #[test]
    fn knowledge_unresolved_is_error() {
        let reg = required_claims();
        let knowledge = vec![KnowledgeClaimRow {
            id: "claim-does-not-exist".into(),
            strength: "slo".into(),
            text: "ghost".into(),
        }];
        let v = validate_knowledge_claims(&knowledge, &reg, &class_map());
        assert!(v.iter().any(|x| x.code == "knowledge_unresolved"));
    }

    #[test]
    fn knowledge_strength_mismatch() {
        let reg = required_claims();
        let knowledge = vec![KnowledgeClaimRow {
            id: "claim-not-epi-certified".into(),
            strength: "benchmark".into(), // registry says invariant
            text: "not certified".into(),
        }];
        let v = validate_knowledge_claims(&knowledge, &reg, &class_map());
        assert!(v.iter().any(|x| x.code == "knowledge_strength_mismatch"));
    }

    #[test]
    fn extract_markers_basic() {
        let t = "hello [[claim:claim-not-epi-certified]] and [[claim:claim-domain-covered]] end";
        let m = extract_markers(t, "[[claim:", "]]");
        assert_eq!(
            m,
            vec![
                "claim-not-epi-certified".to_string(),
                "claim-domain-covered".to_string()
            ]
        );
    }

    fn grade_exact_lint(roots: Vec<String>) -> ClaimsLintFile {
        ClaimsLintFile {
            schema_version: 1,
            scan: ScanConfig {
                marker_prefix: "[[claim:".into(),
                marker_suffix: "]]".into(),
                roots,
                extensions: vec![".md".into()],
            },
            exclude: vec![],
            load_bearing: vec![LoadBearing {
                id: "grade-exact".into(),
                pattern: "(?i)byte-exact|GradeExact|grade digest|dual-path".into(),
                must_cite: vec!["claim-grade-byte-exact".into()],
            }],
        }
    }

    fn grade_claim_ids() -> BTreeSet<String> {
        ["claim-grade-byte-exact".to_string()].into_iter().collect()
    }

    #[test]
    fn quoted_stem_in_markdown_table_is_not_a_grade_exact_claim() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::write(
            root.join("receipt.md"),
            "id | stem\n--- | ---\nm06-q048 | \"A single-corded critical device in a dual-path hall often needs an STS because:\"\n",
        )
        .unwrap();
        let lint = grade_exact_lint(vec!["receipt.md".into()]);
        let violations = validate_claims_lint(root, &lint, &grade_claim_ids()).unwrap();
        assert!(
            violations.is_empty(),
            "quoted table data was treated as authored prose: {violations:?}"
        );
    }

    #[test]
    fn authored_grade_exact_claim_without_marker_still_goes_red() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::write(
            root.join("README.md"),
            "The grader emits byte-exact native and WASM output.\n",
        )
        .unwrap();
        let lint = grade_exact_lint(vec!["README.md".into()]);
        let violations = validate_claims_lint(root, &lint, &grade_claim_ids()).unwrap();
        assert!(
            violations.iter().any(|v| v.code == "load_bearing_uncited"),
            "authored grade claim was not caught: {violations:?}"
        );
    }

    #[test]
    fn claims_lint_empty_scan_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir(root.join("empty")).unwrap();
        let lint = grade_exact_lint(vec!["empty".into()]);
        let error = validate_claims_lint(root, &lint, &grade_claim_ids()).unwrap_err();
        assert!(error.to_string().contains("0 files"), "{error}");
    }

    #[test]
    fn exclude_without_reason_caught_in_lint_schema() {
        // validate via claims_lint path: empty reason
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("registries")).unwrap();
        fs::create_dir_all(root.join("docs")).unwrap();
        fs::create_dir_all(root.join("knowledge")).unwrap();

        // Minimal registries so check_repo gets to lint
        write_minimal_repo(root);

        let lint = r#"
schema_version = 1
[scan]
marker_prefix = "[[claim:"
marker_suffix = "]]"
roots = ["README.md"]
extensions = [".md"]
[[exclude]]
path = "README.md"
reason = ""
[[load_bearing]]
id = "x"
pattern = "never-match-zzz"
must_cite = ["claim-not-epi-certified"]
"#;
        fs::write(root.join("registries/claims_lint.toml"), lint).unwrap();
        let v = check_repo(root).unwrap();
        assert!(
            v.iter().any(|x| x.code == "exclude_no_reason"),
            "expected exclude_no_reason: {v:?}"
        );
    }

    #[test]
    fn marker_unresolved_is_error() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_minimal_repo(root);
        fs::write(
            root.join("README.md"),
            "We are honest [[claim:claim-totally-fake]]\n",
        )
        .unwrap();
        // claims_lint already written by write_minimal_repo to scan README
        let v = check_repo(root).unwrap();
        assert!(
            v.iter().any(|x| x.code == "marker_unresolved"),
            "expected marker_unresolved: {v:?}"
        );
    }

    #[test]
    fn load_bearing_uncited_is_error() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_minimal_repo(root);
        fs::write(
            root.join("README.md"),
            "This product makes you interview-ready for DC jobs.\n",
        )
        .unwrap();
        let v = check_repo(root).unwrap();
        assert!(
            v.iter().any(|x| x.code == "load_bearing_uncited"),
            "expected load_bearing_uncited: {v:?}"
        );
    }

    #[test]
    fn missing_registry_file_is_error() {
        let dir = tempfile::tempdir().unwrap();
        let v = check_repo(dir.path()).unwrap();
        assert!(v.iter().any(|x| x.code == "missing_registry"));
    }

    #[test]
    fn real_repo_is_clean() {
        let root = resolve_repo_root(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("repo root");
        let v = check_repo(&root).expect("check runs");
        assert!(
            v.is_empty(),
            "shipped registries/docs must be clean, found: {v:?}"
        );
    }

    #[test]
    fn canonical_classes_cover_lattice() {
        let v = validate_claim_classes(&sample_classes());
        assert!(v.is_empty(), "{v:?}");
    }

    fn write_minimal_repo(root: &Path) {
        fs::create_dir_all(root.join("registries")).unwrap();
        fs::create_dir_all(root.join("knowledge")).unwrap();
        let claims = r#"
schema_version = 1
[[claim_class]]
name = "invariant"
rank = 6
[[claim_class]]
name = "proof"
rank = 5
[[claim_class]]
name = "bounded_model"
rank = 4
[[claim_class]]
name = "statistical"
rank = 3
[[claim_class]]
name = "slo"
rank = 2
[[claim_class]]
name = "benchmark"
rank = 1
[[claim]]
id = "claim-not-epi-certified"
strength = "invariant"
text = "not certified"
[[claim]]
id = "claim-study-signal-27"
strength = "benchmark"
text = "27 study"
[[claim]]
id = "claim-interview-ready"
strength = "benchmark"
text = "interview"
justified_by = ["claim-study-signal-27"]
[[claim]]
id = "claim-domain-covered"
strength = "slo"
text = "domains"
[[claim]]
id = "claim-forbidden-dump-bank"
strength = "invariant"
text = "no dumps"
"#;
        fs::write(root.join("registries/claims.toml"), claims).unwrap();
        let knowledge = r#"
schema_version = 1
[[claim]]
id = "claim-not-epi-certified"
strength = "invariant"
text = "not certified"
"#;
        fs::write(root.join("knowledge/claims.toml"), knowledge).unwrap();
        let objectives = r#"
schema_version = 1
[[objective]]
id = "obj-honesty"
text = "honesty"
claim_ids = ["claim-not-epi-certified"]
"#;
        fs::write(root.join("registries/objectives.toml"), objectives).unwrap();
        let lint = r#"
schema_version = 1
[scan]
marker_prefix = "[[claim:"
marker_suffix = "]]"
roots = ["README.md"]
extensions = [".md"]
[[load_bearing]]
id = "interview-ready"
pattern = "(?i)interview[- ]ready"
must_cite = ["claim-interview-ready"]
"#;
        fs::write(root.join("registries/claims_lint.toml"), lint).unwrap();
        let mut f = fs::File::create(root.join("README.md")).unwrap();
        writeln!(f, "# test").unwrap();
    }
}
