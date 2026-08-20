//! Source-derived count pins.
//!
//! A count assertion buried in a product test fails too late and too opaquely
//! when the bank legitimately changes.  This module puts those pins in one
//! registry, recomputes them from the named source, and reports
//! `DRIFT expected=… actual=…` before the downstream suites run.
//!
//! The registry is not a source of truth: `expected` is only a checked-in
//! observation of the source.  A pin that was wrong from its first commit is
//! still wrong; this guard can only detect movement after the pin was recorded.
//!
//! Exact pins fail on any movement.  A floor pin is deliberately narrower: it
//! is allowed only for an anti-vacuity scan-domain size, and fails when that
//! domain falls below its positive `min`.  Passing the numeric-claims floor at
//! 345 therefore proves only that the scan did not collapse to zero; it does
//! not establish that 345 is the correct number of numeric claims.
#![forbid(unsafe_code)]

use cdcp_bank::key_contradiction;
use cdcp_bank::Bank;
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub const REGISTRY_PATH: &str = "registries/count_pins.toml";

fn default_kind() -> String {
    "exact".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct Registry {
    pub schema_version: u32,
    #[serde(default)]
    pub derived: Vec<DerivedCount>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DerivedCount {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub metric: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub module: Option<u32>,
    #[serde(default)]
    pub field: Option<String>,
    #[serde(default)]
    pub denominator: Option<u64>,
    /// `exact` is the fail-closed default.  `floor` is restricted by schema
    /// validation to scan-domain size metrics.
    #[serde(default = "default_kind")]
    pub kind: String,
    #[serde(default)]
    pub min: Option<u64>,
    #[serde(default)]
    pub expected: Option<u64>,
    #[serde(default)]
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drift {
    pub id: String,
    pub source: String,
    pub metric: String,
    pub kind: String,
    pub expected: Option<u64>,
    pub min: Option<u64>,
    pub actual: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CountPinReport {
    pub pins: usize,
    pub observations: Vec<(String, u64)>,
    pub drifts: Vec<Drift>,
}

pub fn parse_registry(text: &str) -> Result<Registry, String> {
    toml::from_str(text).map_err(|error| format!("parse {REGISTRY_PATH}: {error}"))
}

/// Schema errors are distinct from a source count changing.  In particular,
/// an empty registry is not a clean scan and a blank reason does not grant
/// permission for an unreviewed pin.
pub fn schema_errors(registry: &Registry) -> Vec<String> {
    let mut errors = Vec::new();
    if registry.schema_version != 1 {
        errors.push(format!(
            "{REGISTRY_PATH}: schema_version {} unsupported (expected 1)",
            registry.schema_version
        ));
    }
    if registry.derived.is_empty() {
        errors.push(format!(
            "{REGISTRY_PATH}: zero [[derived]] rows — empty registry is SCHEMA ERROR, never a pass"
        ));
        return errors;
    }

    let mut ids = BTreeSet::new();
    for (index, row) in registry.derived.iter().enumerate() {
        let where_ = if row.id.trim().is_empty() {
            format!("[[derived]] #{}", index + 1)
        } else {
            format!("[[derived]] {}", row.id.trim())
        };
        if row.id.trim().is_empty() {
            errors.push(format!("{where_}: missing id"));
        } else if !ids.insert(row.id.trim().to_string()) {
            errors.push(format!("{where_}: duplicate id"));
        }
        if row.source.trim().is_empty() {
            errors.push(format!("{where_}: missing source"));
        }
        if row.metric.trim().is_empty() {
            errors.push(format!("{where_}: missing metric"));
        }
        if row.reason.trim().is_empty() {
            errors.push(format!(
                "{where_}: blank reason is SCHEMA ERROR, not permission for a count pin"
            ));
        }
        match row.kind.trim() {
            "exact" => {
                if row.expected.is_none() {
                    errors.push(format!(
                        "{where_}: exact pin is missing expected value — it must state the checked-in observation"
                    ));
                }
                if row.min.is_some() {
                    errors.push(format!(
                        "{where_}: exact pin must not define min; use kind=\"floor\" only for an anti-vacuity scan domain"
                    ));
                }
            }
            "floor" => {
                if row.expected.is_some() {
                    errors.push(format!(
                        "{where_}: floor pin must not define expected; its fail-closed policy is min"
                    ));
                }
                match row.min {
                    None => errors.push(format!(
                        "{where_}: floor pin is missing min — this is SCHEMA ERROR, not an exact-count default"
                    )),
                    Some(0) => errors.push(format!(
                        "{where_}: floor min must be > 0 — a floor of zero is vacuity wearing a badge"
                    )),
                    Some(_) => {}
                }
                let reason = row.reason.to_ascii_lowercase();
                if !reason.contains("vacu")
                    || !(reason.contains("zero")
                        || reason.contains("empty")
                        || reason.contains("collapse"))
                {
                    errors.push(format!(
                        "{where_}: floor reason must name the vacuity it prevents (zero, empty, or collapse)"
                    ));
                }
                if !is_scan_domain_metric(row.metric.trim()) {
                    errors.push(format!(
                        "{where_}: kind=\"floor\" is only allowed on a scan-domain size metric"
                    ));
                }
            }
            other => errors.push(format!(
                "{where_}: unknown kind {other:?} (expected \"exact\" or \"floor\")"
            )),
        }
        let metric = row.metric.trim();
        match metric {
            "file_count" => {
                if row.source.trim() != "bank/items" {
                    errors.push(format!("{where_}: file_count source must be bank/items"));
                }
            }
            "status_count" => {
                if row.source.trim() != "bank/items"
                    || row.status.as_deref().unwrap_or("").trim().is_empty()
                {
                    errors.push(format!(
                        "{where_}: status_count needs source=bank/items and a status"
                    ));
                }
            }
            "module_status_count" => {
                if row.source.trim() != "bank/items"
                    || row.module.is_none()
                    || row.status.as_deref().unwrap_or("").trim().is_empty()
                {
                    errors.push(format!(
                        "{where_}: module_status_count needs source=bank/items, module, and status"
                    ));
                }
            }
            "module_file_count" => {
                if row.source.trim() != "bank/items" || row.module.is_none() {
                    errors.push(format!(
                        "{where_}: module_file_count needs source=bank/items and module"
                    ));
                }
            }
            "units_index_field" => {
                if row.source.trim() != "web/data/units_index.json"
                    || row.field.as_deref().unwrap_or("").trim().is_empty()
                {
                    errors.push(format!(
                        "{where_}: units_index_field needs source=web/data/units_index.json and field"
                    ));
                }
            }
            "key_contradiction_numeric_claims" | "key_contradiction_numeric_contradictions" => {
                if row.source.trim() != "bank/items" {
                    errors.push(format!("{where_}: {metric} source must be bank/items"));
                }
            }
            "approved_pool_multiplier_tenths" => {
                if row.source.trim() != "bank/items"
                    || row.denominator.is_none_or(|value| value == 0)
                {
                    errors.push(format!(
                        "{where_}: approved_pool_multiplier_tenths needs source=bank/items and a positive denominator"
                    ));
                }
            }
            other => errors.push(format!("{where_}: unknown metric {other:?}")),
        }
    }
    errors
}

fn is_scan_domain_metric(metric: &str) -> bool {
    matches!(metric, "key_contradiction_numeric_claims")
}

fn load_registry(root: &Path) -> Result<Registry, String> {
    let path = root.join(REGISTRY_PATH);
    let text =
        fs::read_to_string(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
    let registry = parse_registry(&text)?;
    let errors = schema_errors(&registry);
    if errors.is_empty() {
        Ok(registry)
    } else {
        Err(format!("SCHEMA ERROR: {}", errors.join("; ")))
    }
}

/// Read one checked-in expected value for a consumer test.  The count gate is
/// still the authority that compares it with the source; consumers use this
/// helper so they do not repeat the literal and fail later with an opaque
/// assertion.
pub fn expected_value(root: &Path, id: &str) -> Result<u64, String> {
    let registry = load_registry(root)?;
    let id = id.trim();
    registry
        .derived
        .iter()
        .find(|row| row.id.trim() == id)
        .and_then(|row| row.expected)
        .ok_or_else(|| format!("{REGISTRY_PATH}: no expected value for pin {id:?}"))
}

fn bank(root: &Path) -> Result<Bank, String> {
    Bank::load_dir(&root.join("bank/items")).map_err(|error| format!("load bank/items: {error}"))
}

fn file_count(root: &Path) -> Result<u64, String> {
    let dir = root.join("bank/items");
    // Validate the same source before counting it.  An invalid TOML file is
    // not a legitimate new count that this guard should certify.
    let _ = bank(root)?;
    let mut count = 0u64;
    for entry in fs::read_dir(&dir).map_err(|error| format!("read {}: {error}", dir.display()))? {
        let entry = entry.map_err(|error| format!("read bank/items entry: {error}"))?;
        if entry.path().extension().and_then(|value| value.to_str()) == Some("toml") {
            count += 1;
        }
    }
    if count == 0 {
        return Err(
            "bank/items has zero TOML files — source count scan is ERROR, not a pass".into(),
        );
    }
    Ok(count)
}

fn bank_status_count(bank: &Bank, status: &str) -> u64 {
    bank.items
        .values()
        .filter(|item| item.status.as_str() == status)
        .count() as u64
}

fn bank_module_count(bank: &Bank, module: u32, status: Option<&str>) -> u64 {
    bank.items
        .values()
        .filter(|item| item.module == module)
        .filter(|item| status.is_none_or(|wanted| item.status.as_str() == wanted))
        .count() as u64
}

fn units_index_field(root: &Path, field: &str) -> Result<u64, String> {
    let path = root.join("web/data/units_index.json");
    let text =
        fs::read_to_string(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
    let value: Value = serde_json::from_str(&text)
        .map_err(|error| format!("parse {}: {error}", path.display()))?;
    value.get(field).and_then(Value::as_u64).ok_or_else(|| {
        format!("web/data/units_index.json: field {field:?} is not an unsigned integer")
    })
}

fn actual_value(root: &Path, row: &DerivedCount) -> Result<u64, String> {
    match row.metric.trim() {
        "file_count" => file_count(root),
        "status_count" => {
            let bank = bank(root)?;
            Ok(bank_status_count(
                &bank,
                row.status.as_deref().unwrap_or(""),
            ))
        }
        "module_status_count" => {
            let bank = bank(root)?;
            Ok(bank_module_count(
                &bank,
                row.module.ok_or("module_status_count missing module")?,
                Some(row.status.as_deref().unwrap_or("")),
            ))
        }
        "module_file_count" => {
            let bank = bank(root)?;
            Ok(bank_module_count(
                &bank,
                row.module.ok_or("module_file_count missing module")?,
                None,
            ))
        }
        "units_index_field" => units_index_field(root, row.field.as_deref().unwrap_or("")),
        "key_contradiction_numeric_claims" => {
            Ok(key_contradiction::measure(root)?.numeric_claims as u64)
        }
        "key_contradiction_numeric_contradictions" => {
            Ok(key_contradiction::measure(root)?.numeric_contradictions as u64)
        }
        "approved_pool_multiplier_tenths" => {
            let denominator = row
                .denominator
                .ok_or("approved_pool_multiplier_tenths missing denominator")?;
            let bank = bank(root)?;
            let approved = bank_status_count(&bank, "approved");
            Ok((approved * 10 + denominator / 2) / denominator)
        }
        metric => Err(format!("unknown metric {metric:?}")),
    }
}

pub fn evaluate(root: &Path) -> Result<CountPinReport, String> {
    let registry = load_registry(root)?;
    let mut observations = Vec::with_capacity(registry.derived.len());
    let mut drifts = Vec::new();
    for row in &registry.derived {
        let actual = actual_value(root, row).map_err(|error| {
            format!(
                "{REGISTRY_PATH}: pin {} could not compute from {}: {error}",
                row.id.trim(),
                row.source.trim()
            )
        })?;
        observations.push((row.id.trim().to_string(), actual));
        match row.kind.trim() {
            "exact" => {
                let expected = row.expected.ok_or_else(|| {
                    format!(
                        "{REGISTRY_PATH}: exact pin {} has no expected value",
                        row.id
                    )
                })?;
                if expected != actual {
                    drifts.push(Drift {
                        id: row.id.trim().to_string(),
                        source: row.source.trim().to_string(),
                        metric: row.metric.trim().to_string(),
                        kind: row.kind.trim().to_string(),
                        expected: Some(expected),
                        min: None,
                        actual,
                    });
                }
            }
            "floor" => {
                let min = row
                    .min
                    .ok_or_else(|| format!("{REGISTRY_PATH}: floor pin {} has no min", row.id))?;
                if actual < min {
                    drifts.push(Drift {
                        id: row.id.trim().to_string(),
                        source: row.source.trim().to_string(),
                        metric: row.metric.trim().to_string(),
                        kind: row.kind.trim().to_string(),
                        expected: None,
                        min: Some(min),
                        actual,
                    });
                }
            }
            kind => {
                return Err(format!(
                    "{REGISTRY_PATH}: pin {} has unsupported kind {kind:?}",
                    row.id
                ));
            }
        }
    }
    Ok(CountPinReport {
        pins: registry.derived.len(),
        observations,
        drifts,
    })
}

pub fn render(report: &CountPinReport, tree: &str) -> String {
    let mut out = format!(
        "count-pin-drift: tree={tree} pins={} source-derived=bank/items,web/data/units_index.json\n",
        report.pins
    );
    for (id, actual) in &report.observations {
        out.push_str(&format!(
            "count-pin-drift: observed id={id} actual={actual}\n"
        ));
    }
    for drift in &report.drifts {
        match drift.kind.as_str() {
            "exact" => out.push_str(&format!(
                "count-pin-drift: DRIFT kind=exact id={} expected={} actual={} source={} metric={}\n",
                drift.id,
                drift.expected
                    .expect("schema/evaluation guarantees exact pins have expected"),
                drift.actual,
                drift.source,
                drift.metric
            )),
            "floor" => out.push_str(&format!(
                "count-pin-drift: DRIFT kind=floor id={} min={} actual={} source={} metric={}\n",
                drift.id,
                drift.min
                    .expect("schema/evaluation guarantees floor pins have min"),
                drift.actual,
                drift.source,
                drift.metric
            )),
            kind => out.push_str(&format!(
                "count-pin-drift: DRIFT kind={kind} id={} actual={} source={} metric={}\n",
                drift.id, drift.actual, drift.source, drift.metric
            )),
        }
    }
    if report.drifts.is_empty() {
        out.push_str("count-pin-drift: PASS (all registry pins match their sources)\n");
    } else {
        out.push_str(&format!(
            "count-pin-drift: RED drifts={} (downstream count assertions were not run)\n",
            report.drifts.len()
        ));
    }
    out
}

pub fn run(root: &Path) -> Result<(), crate::CheckError> {
    let report = evaluate(root).map_err(crate::CheckError::msg)?;
    print!("{}", render(&report, "worktree"));
    if report.drifts.is_empty() {
        Ok(())
    } else {
        Err(crate::CheckError::msg(format!(
            "{} count pin(s) drifted",
            report.drifts.len()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn fixture_root(expected: u64) -> (tempfile::TempDir, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().to_path_buf();
        fs::create_dir_all(root.join("bank/items")).unwrap();
        fs::create_dir_all(root.join("registries")).unwrap();
        fs::write(
            root.join("bank/items/one.toml"),
            r#"id = "one"
module = 1
stem = "A question"
choices = ["one", "two", "three", "four"]
correct = "A"
explanation = "A sufficiently long reason for this fixture item."
topic_ids = ["topic"]
bloom = "remember"
source_class = "original"
quantity_evidence = "qualitative_only"
status = "approved"
kind = "single-select"
"#,
        )
        .unwrap();
        fs::write(
            root.join(REGISTRY_PATH),
            format!(
                "schema_version = 1\n\n[[derived]]\nid = \"bank.item-count\"\nsource = \"bank/items\"\nmetric = \"file_count\"\nexpected = {expected}\nreason = \"The fixture proves expected-versus-actual drift is named before tests.\"\n"
            ),
        )
        .unwrap();
        (temp, root)
    }

    fn numeric_floor_fixture(
        min: Option<u64>,
        reason: &str,
        metric: &str,
    ) -> (tempfile::TempDir, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().to_path_buf();
        fs::create_dir_all(root.join("bank/items")).unwrap();
        fs::create_dir_all(root.join("registries")).unwrap();

        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../cdcp_bank/tests/fixtures/key_contradiction/good");
        for entry in fs::read_dir(fixture.join("bank/items")).unwrap() {
            let entry = entry.unwrap();
            fs::copy(
                entry.path(),
                root.join("bank/items").join(entry.file_name()),
            )
            .unwrap();
        }
        fs::copy(
            fixture.join("registries/key_contradiction.toml"),
            root.join("registries/key_contradiction.toml"),
        )
        .unwrap();

        let min = min.map_or_else(String::new, |value| format!("min = {value}\n"));
        fs::write(
            root.join(REGISTRY_PATH),
            format!(
                "schema_version = 1\n\n[[derived]]\nid = \"key-contradiction.numeric-claims\"\nsource = \"bank/items\"\nmetric = \"{metric}\"\nkind = \"floor\"\n{min}reason = \"{reason}\"\n"
            ),
        )
        .unwrap();
        (temp, root)
    }

    #[test]
    fn source_change_is_reported_as_expected_vs_actual_drift() {
        let (_temp, root) = fixture_root(1);
        let second = fs::read_to_string(root.join("bank/items/one.toml")).unwrap();
        fs::write(
            root.join("bank/items/two.toml"),
            second.replace(r#"id = "one""#, r#"id = "two""#),
        )
        .unwrap();
        let report = evaluate(&root).unwrap();
        assert_eq!(report.drifts.len(), 1);
        let text = render(&report, "fixture");
        assert!(
            text.contains("DRIFT kind=exact id=bank.item-count expected=1 actual=2"),
            "{text}"
        );
        assert!(
            text.contains("downstream count assertions were not run"),
            "{text}"
        );
    }

    #[test]
    fn expected_value_is_registry_backed() {
        let (_temp, root) = fixture_root(1);
        assert_eq!(expected_value(&root, "bank.item-count").unwrap(), 1);
    }

    #[test]
    fn empty_registry_and_blank_reason_are_schema_errors() {
        let (_temp, root) = fixture_root(1);
        fs::write(root.join(REGISTRY_PATH), "schema_version = 1\n").unwrap();
        let empty = evaluate(&root).unwrap_err();
        assert!(empty.contains("empty registry is SCHEMA ERROR"), "{empty}");

        fs::write(
            root.join(REGISTRY_PATH),
            "schema_version = 1\n\n[[derived]]\nid = \"x\"\nsource = \"bank/items\"\nmetric = \"file_count\"\nexpected = 1\nreason = \"\"\n",
        )
        .unwrap();
        let blank = evaluate(&root).unwrap_err();
        assert!(blank.contains("blank reason is SCHEMA ERROR"), "{blank}");
    }

    #[test]
    fn floor_below_min_is_a_distinct_expected_floor_drift() {
        let (_temp, root) = numeric_floor_fixture(
            Some(999),
            "Anti-vacuity: prevents zero numeric claims from certifying an unrun scan.",
            "key_contradiction_numeric_claims",
        );
        let report = evaluate(&root).unwrap();
        assert_eq!(report.drifts.len(), 1);
        let text = render(&report, "fixture");
        assert!(
            text.contains("DRIFT kind=floor id=key-contradiction.numeric-claims min=999 actual="),
            "{text}"
        );
        assert!(!text.contains("expected="), "{text}");
    }

    #[test]
    fn floor_at_min_is_green_without_an_exact_numeric_pin() {
        let (_temp, root) = numeric_floor_fixture(
            Some(1),
            "Anti-vacuity: prevents zero numeric claims from certifying an unrun scan.",
            "key_contradiction_numeric_claims",
        );
        let report = evaluate(&root).unwrap();
        assert!(report.drifts.is_empty(), "{}", render(&report, "fixture"));
        let actual = report
            .observations
            .iter()
            .find(|(id, _)| id == "key-contradiction.numeric-claims")
            .map(|(_, value)| *value)
            .unwrap();
        assert!(actual >= 1, "numeric scan domain unexpectedly collapsed");
    }

    #[test]
    fn floor_without_min_is_schema_error() {
        let (_temp, root) = numeric_floor_fixture(
            None,
            "Anti-vacuity: prevents zero numeric claims from certifying an unrun scan.",
            "key_contradiction_numeric_claims",
        );
        let error = evaluate(&root).unwrap_err();
        assert!(error.contains("floor pin is missing min"), "{error}");
        assert!(error.contains("SCHEMA ERROR"), "{error}");
    }

    #[test]
    fn zero_floor_is_schema_error_not_a_vacuous_pass() {
        let (_temp, root) = numeric_floor_fixture(
            Some(0),
            "Anti-vacuity: prevents zero numeric claims from certifying an unrun scan.",
            "key_contradiction_numeric_claims",
        );
        let error = evaluate(&root).unwrap_err();
        assert!(error.contains("floor min must be > 0"), "{error}");
        assert!(error.contains("SCHEMA ERROR"), "{error}");
    }

    #[test]
    fn floor_is_restricted_to_scan_domain_sizes() {
        let (_temp, root) = numeric_floor_fixture(
            Some(1),
            "Anti-vacuity: prevents zero counts from certifying an unrun scan.",
            "file_count",
        );
        let error = evaluate(&root).unwrap_err();
        assert!(
            error.contains("only allowed on a scan-domain size metric"),
            "{error}"
        );
        assert!(error.contains("SCHEMA ERROR"), "{error}");
    }
}
