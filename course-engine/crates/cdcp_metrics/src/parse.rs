//! Document parser. A bare number or a missing `[boundary]` is a schema ERROR.

use crate::boundary::{
    Boundary, BoundarySpec, CarbonAccounting, EnergyWaterMix, HydroReservoir, ItMeter, ReusePolicy,
    ScopeItem, WaterScope,
};
use crate::error::{MetricsError, BARE_NUMBER, MISSING_BOUNDARY};
use crate::kind::MetricKind;
use crate::metric::Metric;
use crate::ratio::Ratio;
use std::collections::BTreeSet;

/// Refuse a document that has no `boundary` table.
///
/// This is the check the known-bad meta-test names. Deleting the body
/// (or the `doc.get("boundary")` lookup) makes
/// `selftest_delete_boundary_check_is_red` non-zero.
pub fn require_boundary(doc: &toml::Value) -> Result<(), MetricsError> {
    let _ = MISSING_BOUNDARY;
    match doc.get("boundary") {
        None => Err(MetricsError::MissingBoundary),
        Some(toml::Value::Table(t)) if t.is_empty() => Err(MetricsError::EmptyBoundary),
        Some(toml::Value::Table(_)) => Ok(()),
        Some(_) => Err(MetricsError::Unparseable("boundary must be a table".into())),
    }
}

/// Parse a metric document.
///
/// A lone integer, float, or `num/den` token is [`MetricsError::BareNumber`].
/// A table without `boundary` is [`MetricsError::MissingBoundary`].
/// `value` must be `{ num, den }` — a float is [`MetricsError::FloatForbidden`].
pub fn parse_metric(text: &str) -> Result<Metric, MetricsError> {
    let _ = BARE_NUMBER;
    let _ = MISSING_BOUNDARY;
    let t = text.trim();
    if t.is_empty() {
        return Err(MetricsError::EmptyDocument);
    }
    if is_bare_number(t) {
        return Err(MetricsError::BareNumber);
    }
    let doc: toml::Value =
        toml::from_str(t).map_err(|e| MetricsError::Unparseable(e.to_string()))?;
    match &doc {
        toml::Value::Integer(_) | toml::Value::Float(_) | toml::Value::String(_) => {
            return Err(MetricsError::BareNumber);
        }
        toml::Value::Table(_) => {}
        _ => return Err(MetricsError::BareNumber),
    }
    require_boundary(&doc)?;
    let kind = match doc.get("kind").and_then(|v| v.as_str()) {
        Some(s) => MetricKind::parse(s)?,
        None => {
            return Err(MetricsError::Unparseable("missing kind".into()));
        }
    };
    let value = match doc.get("value") {
        Some(v) => parse_ratio(v)?,
        None => return Err(MetricsError::Unparseable("missing value".into())),
    };
    let boundary = parse_boundary(doc.get("boundary").expect("require_boundary held"))?;
    Metric::declared(kind, value, boundary)
}

fn is_bare_number(t: &str) -> bool {
    if t.is_empty() {
        return false;
    }
    let compact: String = t.chars().filter(|c| !c.is_whitespace()).collect();
    if compact
        .chars()
        .all(|c| c.is_ascii_digit() || c == '-' || c == '+' || c == '.')
        && compact.chars().any(|c| c.is_ascii_digit())
        && !compact.contains('\n')
        && t.lines().count() == 1
        && !t.contains('=')
        && !t.contains('[')
    {
        return true;
    }
    let parts: Vec<&str> = compact.split('/').collect();
    parts.len() == 2
        && parts.iter().all(|p| {
            !p.is_empty()
                && p.chars()
                    .enumerate()
                    .all(|(i, c)| c.is_ascii_digit() || (i == 0 && (c == '-' || c == '+')))
        })
        && t.lines().count() == 1
        && !t.contains('=')
}

fn parse_ratio(v: &toml::Value) -> Result<Ratio, MetricsError> {
    match v {
        toml::Value::Float(_) => Err(MetricsError::FloatForbidden),
        toml::Value::Integer(_) | toml::Value::String(_) => Err(MetricsError::BareNumber),
        toml::Value::Table(t) => {
            let num = int_field(t, "num")?;
            let den = int_field(t, "den")?;
            Ratio::new(num, den)
        }
        _ => Err(MetricsError::Unparseable(
            "value must be { num, den }".into(),
        )),
    }
}

fn int_field(t: &toml::map::Map<String, toml::Value>, key: &str) -> Result<i64, MetricsError> {
    match t.get(key) {
        Some(toml::Value::Integer(n)) => Ok(*n),
        Some(toml::Value::Float(_)) => Err(MetricsError::FloatForbidden),
        Some(_) => Err(MetricsError::Unparseable(format!(
            "{key} must be an integer"
        ))),
        None => Err(MetricsError::Unparseable(format!("missing {key}"))),
    }
}

fn parse_boundary(v: &toml::Value) -> Result<Boundary, MetricsError> {
    let t = v
        .as_table()
        .ok_or_else(|| MetricsError::Unparseable("boundary must be a table".into()))?;
    let it_meter = match t.get("it_meter").and_then(|x| x.as_str()) {
        Some(s) => ItMeter::parse(s)?,
        None => {
            return Err(MetricsError::MissingDeclaration {
                kind: "boundary",
                field: "it_meter",
            });
        }
    };
    let includes = parse_items(t.get("includes"))?;
    let excludes = parse_items(t.get("excludes"))?;
    let water_scope = opt_parse(t, "water_scope", WaterScope::parse)?;
    let hydro = opt_parse(t, "hydro", HydroReservoir::parse)?;
    let energy_water_mix = opt_parse(t, "mix", EnergyWaterMix::parse)?;
    let mix_id = t.get("mix_id").and_then(|x| x.as_str()).map(str::to_string);
    let carbon = opt_parse(t, "carbon", CarbonAccounting::parse)?;
    let reuse = opt_parse(t, "reuse", ReusePolicy::parse)?;
    Boundary::new(BoundarySpec {
        it_meter,
        includes,
        excludes,
        water_scope,
        hydro,
        energy_water_mix,
        mix_id,
        carbon,
        reuse,
    })
}

fn opt_parse<T>(
    t: &toml::map::Map<String, toml::Value>,
    key: &str,
    parse: fn(&str) -> Result<T, MetricsError>,
) -> Result<Option<T>, MetricsError> {
    match t.get(key) {
        None => Ok(None),
        Some(v) => match v.as_str() {
            Some(s) => parse(s).map(Some),
            None => Err(MetricsError::Unparseable(format!("{key} must be a string"))),
        },
    }
}

fn parse_items(v: Option<&toml::Value>) -> Result<BTreeSet<ScopeItem>, MetricsError> {
    match v {
        None => Ok(BTreeSet::new()),
        Some(toml::Value::Array(arr)) => {
            let mut out = BTreeSet::new();
            for item in arr {
                let s = item.as_str().ok_or_else(|| {
                    MetricsError::Unparseable("scope item must be a string".into())
                })?;
                out.insert(ScopeItem::parse(s)?);
            }
            Ok(out)
        }
        Some(_) => Err(MetricsError::Unparseable(
            "includes/excludes must be arrays".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_number_is_schema_error() {
        for raw in ["1.8", "18", "9/5", "-2", "1.54"] {
            let err = parse_metric(raw).unwrap_err();
            assert_eq!(err, MetricsError::BareNumber, "{raw}");
            assert!(err.to_string().contains(BARE_NUMBER));
        }
    }

    #[test]
    fn omit_boundary_is_schema_error() {
        let err = parse_metric("kind = \"pue\"\nvalue = { num = 6, den = 5 }\n").unwrap_err();
        assert_eq!(err, MetricsError::MissingBoundary);
        assert!(err.to_string().contains(MISSING_BOUNDARY));
    }

    #[test]
    fn empty_boundary_table_is_error() {
        let err =
            parse_metric("kind = \"pue\"\nvalue = { num = 6, den = 5 }\n[boundary]\n").unwrap_err();
        assert_eq!(err, MetricsError::EmptyBoundary);
    }

    #[test]
    fn float_value_is_forbidden() {
        let text = "\
kind = \"pue\"
value = 1.2
[boundary]
it_meter = \"ups-output\"
includes = [\"it-energy\", \"cooling\"]
excludes = [\"office-hvac\"]
";
        let err = parse_metric(text).unwrap_err();
        assert_eq!(err, MetricsError::FloatForbidden);
    }

    #[test]
    fn good_pue_parses() {
        let text = "\
kind = \"pue\"
value = { num = 6, den = 5 }
[boundary]
it_meter = \"ups-output\"
includes = [\"it-energy\", \"cooling\", \"lighting\", \"ups-losses\"]
excludes = [\"generator-testing\", \"office-hvac\"]
";
        let m = parse_metric(text).unwrap();
        assert_eq!(m.kind(), MetricKind::Pue);
        assert_eq!(m.value(), Ratio::new(6, 5).unwrap());
        assert_eq!(m.boundary().it_meter(), ItMeter::UpsOutput);
    }
}
