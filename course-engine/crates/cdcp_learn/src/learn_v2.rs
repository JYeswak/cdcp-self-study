//! Learn v2 smoke — units index + unit shell + glossary + concept card.
//!
//! Extracted from `scripts/smoke_learn_v2.py` by
//! `bd-substrate-rust-migration-jhd.20` as product, not a gate file. A
//! learner is scored against `units_index.json` check items and reads the
//! glossary / concept card. If they can see it, it is not a `cdcp_gate`
//! concern.
//!
//! # Contract
//!
//! * `web/data/units_index.json` is a JSON object whose `units` array is
//!   non-empty, whose declared `unit_count` is at least 50, whose M01 and
//!   M06 lists clear the floors (4 and 3), whose check-item coverage is
//!   at least 80%, and whose every M01 unit carries ≥2 check ids
//! * `web/data/glossary.json` is a JSON object whose declared `term_count`
//!   is at least 15
//! * the three JS assets, the M01 Learn page (with the unit-shell needles),
//!   and `drill.html` (with `concept_card.js`) are on disk
//!
//! # Anti-vacuous (bd-learnv2-vacuous-coverage-dad8)
//!
//! An empty `units` array or an empty M01 list is ERROR, never
//! `ok: check_item_ids coverage 0/0` / a pass of the M01 check-item floor
//! over nothing. A missing file, malformed JSON, a JSON array where an
//! object is required, or a non-UTF-8 file is a FAIL row + verdict, never
//! a panic (bd-learnv2-unguarded-reads-0f7g closed, not reproduced).
//!
//! Type mismatches inside a parsed object FAIL-close too (stricter than
//! the retired script, which still raised).
//!
//! This is a READER: it writes nothing. The verdict line is always printed.
//!
//! # What this cannot decide
//!
//! It does not open a browser or execute JS. `unit_count` is a DECLARED
//! field, not `len(units)` — this smoke does not cross-check them. A
//! page whose needles are present in an HTML comment clears the same as
//! a correct page.

#![forbid(unsafe_code)]

use crate::{join_rel, BuildOutcome};
use serde_json::Value as Json;
use std::path::Path;

pub const NAME: &str = "smoke-learn-v2";
pub const SUMMARY: &str = "M8-B/D: units index + unit shell + glossary + concept card assets";

pub const UNITS_INDEX_REL: &str = "web/data/units_index.json";
pub const GLOSSARY_REL: &str = "web/data/glossary.json";
pub const M01_PAGE_REL: &str = "web/learn/01-mission-critical.html";
pub const DRILL_REL: &str = "web/drill.html";

/// JS assets required under `web/`, reported relative to `web/`.
pub const JS_ASSETS: &[&str] = &[
    "assets/js/learn_units.js",
    "assets/js/learn_glossary.js",
    "assets/js/concept_card.js",
];

pub const M01_NEEDLES: &[&str] = &["learn-unit-shell", "learn_units.js", "learn_glossary.js"];

/// Run the Learn-v2 smoke against `root` (the course-engine directory).
///
/// Reader: writes nothing. `code != 0` is RED. `artifact` is always `None`.
/// The verdict line is printed on every path.
pub fn run(root: &Path) -> BuildOutcome {
    evaluate(&join_rel(root, "web"))
}

fn evaluate(web: &Path) -> BuildOutcome {
    let mut r = Report {
        out: String::from("==> smoke_learn_v2 (M8 B/D assets)\n"),
        errs: Vec::new(),
    };

    let units_path = web.join("data/units_index.json");
    if !units_path.is_file() {
        r.fail("missing units_index.json — run `cdcp build-units`");
    } else {
        match load_json_object(&units_path, "units_index.json") {
            Err(row) => r.fail(row),
            Ok(d) => grade_units(&d, &mut r),
        }
    }

    let gloss = web.join("data/glossary.json");
    if !gloss.is_file() {
        r.fail("missing glossary.json");
    } else {
        match load_json_object(&gloss, "glossary.json") {
            Err(row) => r.fail(row),
            Ok(g) => grade_glossary(&g, &mut r),
        }
    }

    for rel in JS_ASSETS {
        if !web.join(rel).is_file() {
            r.fail(format!("missing {rel}"));
        } else {
            r.ok(*rel);
        }
    }

    let m01_path = web.join("learn/01-mission-critical.html");
    if !m01_path.is_file() {
        r.fail("missing learn/01-mission-critical.html");
    } else {
        match read_utf8(&m01_path, "learn/01-mission-critical.html") {
            Err(row) => r.fail(row),
            Ok(m01) => {
                for needle in M01_NEEDLES {
                    if !m01.contains(needle) {
                        r.fail(format!("M01 missing {needle}"));
                    } else {
                        r.ok(format!("M01 {needle}"));
                    }
                }
            }
        }
    }

    let drill_path = web.join("drill.html");
    if !drill_path.is_file() {
        r.fail("missing drill.html");
    } else {
        match read_utf8(&drill_path, "drill.html") {
            Err(row) => r.fail(row),
            Ok(drill) => {
                if !drill.contains("concept_card.js") {
                    r.fail("drill.html missing concept_card.js");
                } else {
                    r.ok("drill concept_card");
                }
            }
        }
    }

    if r.errs.is_empty() {
        r.out.push_str("smoke_learn_v2: PASS\n");
        outcome(0, r.out)
    } else {
        r.out.push_str("smoke_learn_v2: FAIL\n");
        outcome(1, r.out)
    }
}

fn grade_units(d: &Json, r: &mut Report) {
    let by = match d.get("by_module") {
        None | Some(Json::Null) => None,
        Some(Json::Object(map)) => Some(map),
        Some(_) => {
            r.fail("units_index.json by_module is not a JSON object");
            None
        }
    };

    // Floors are written as literals so the bd-lt7 bound sweep can see them.
    // 4 and 3 sit outside 13–16; they are not module-count bounds.
    for (mid, need) in [("01-mission-critical", 4usize), ("06-power", 3usize)] {
        let n = match by.and_then(|m| m.get(mid)) {
            None | Some(Json::Null) => 0,
            Some(Json::Array(rows)) => rows.len(),
            Some(_) => {
                r.fail(format!(
                    "units_index.json by_module.{mid} is not a JSON array"
                ));
                0
            }
        };
        if n < need {
            r.fail(format!("{mid} units={n} need ≥{need}"));
        } else {
            r.ok(format!("{mid} units={n}"));
        }
    }

    match json_num(d.get("unit_count")) {
        Err(msg) => r.fail(format!("units_index.json unit_count {msg}")),
        Ok(n) => {
            if n < 50.0 {
                r.fail("unit_count too low");
            } else if n.fract() == 0.0 {
                r.ok(format!("unit_count={}", n as i64));
            } else {
                r.ok(format!("unit_count={n}"));
            }
        }
    }

    let units = match d.get("units") {
        None | Some(Json::Null) => Some(&[][..]),
        Some(Json::Array(rows)) => Some(rows.as_slice()),
        Some(_) => {
            r.fail("units_index.json units is not a JSON array");
            None
        }
    };

    if let Some(units) = units {
        let mut with_checks = 0usize;
        let mut thin: Vec<String> = Vec::new();
        let mut row_ok = true;
        for u in units {
            match check_id_count(u) {
                None => {
                    r.fail("units_index.json units[] entry is not a JSON object");
                    row_ok = false;
                    break;
                }
                Some(Err(msg)) => {
                    r.fail(format!("units_index.json {msg}"));
                    row_ok = false;
                    break;
                }
                Some(Ok(n)) => {
                    if n >= 2 {
                        with_checks += 1;
                    } else {
                        thin.push(unit_id(u));
                    }
                }
            }
        }
        if row_ok {
            if units.is_empty() {
                r.fail("zero units in units_index.json (vacuous coverage is ERROR)");
            } else if (with_checks as f64) / (units.len() as f64) < 0.8 {
                let sample: Vec<&str> = thin.iter().take(5).map(String::as_str).collect();
                r.fail(format!(
                    "check_item_ids coverage {with_checks}/{} < 80% (sample thin: {sample:?})",
                    units.len()
                ));
            } else {
                r.ok(format!(
                    "check_item_ids coverage {with_checks}/{}",
                    units.len()
                ));
            }
        }
    }

    let m01 = match by.and_then(|m| m.get("01-mission-critical")) {
        None | Some(Json::Null) => Some(&[][..]),
        Some(Json::Array(rows)) => Some(rows.as_slice()),
        Some(_) => None,
    };
    if let Some(m01) = m01 {
        if m01.is_empty() {
            r.fail("zero M01 units in units_index.json (vacuous M01 check-item floor is ERROR)");
        } else {
            let mut min_checks: Option<usize> = None;
            let mut row_ok = true;
            for u in m01 {
                match check_id_count(u) {
                    None => {
                        r.fail(
                            "units_index.json by_module.01-mission-critical[] entry is not a JSON object",
                        );
                        row_ok = false;
                        break;
                    }
                    Some(Err(msg)) => {
                        r.fail(format!("units_index.json {msg}"));
                        row_ok = false;
                        break;
                    }
                    Some(Ok(n)) => {
                        min_checks = Some(min_checks.map_or(n, |m| m.min(n)));
                    }
                }
            }
            if row_ok {
                if min_checks.unwrap_or(0) < 2 {
                    r.fail("M01 unit missing ≥2 check_item_ids");
                } else {
                    r.ok("M01 every unit has ≥2 check items");
                }
            }
        }
    }
}

fn grade_glossary(g: &Json, r: &mut Report) {
    match json_num(g.get("term_count")) {
        Err(msg) => r.fail(format!("glossary.json term_count {msg}")),
        Ok(n) => {
            // The 15 is a TERM floor, not a module bound. Written as a
            // comparison so the bd-lt7 NumericBound sweep can hold a
            // verdict on it (see rebase_module_bounds.rs).
            if n < 15.0 {
                r.fail("glossary term_count < 15");
            } else if n.fract() == 0.0 {
                r.ok(format!("glossary terms={}", n as i64));
            } else {
                r.ok(format!("glossary terms={n}"));
            }
        }
    }
}

/// Number of `check_item_ids` on one unit row.
///
/// `None` — the row is not an object (caller FAIL-closes).
/// `Some(Err)` — `check_item_ids` is present and not an array.
fn check_id_count(u: &Json) -> Option<Result<usize, &'static str>> {
    let obj = u.as_object()?;
    match obj.get("check_item_ids") {
        None | Some(Json::Null) => Some(Ok(0)),
        Some(Json::Array(ids)) => Some(Ok(ids.len())),
        Some(_) => Some(Err("check_item_ids is not a JSON array")),
    }
}

fn unit_id(u: &Json) -> String {
    match u.get("id") {
        None | Some(Json::Null) => "None".into(),
        Some(Json::String(s)) => s.clone(),
        Some(other) => other.to_string(),
    }
}

/// Missing / null → 0 (the retired script's `or 0`). Anything else that is
/// not a number is a named FAIL, not a raise.
fn json_num(v: Option<&Json>) -> Result<f64, &'static str> {
    match v {
        None | Some(Json::Null) => Ok(0.0),
        Some(Json::Bool(false)) => Ok(0.0),
        Some(Json::Bool(true)) => Ok(1.0),
        Some(Json::Number(n)) => n.as_f64().ok_or("is not a finite number"),
        Some(Json::String(s)) if s.is_empty() => Ok(0.0),
        Some(_) => Err("is not a number"),
    }
}

fn load_json_object(path: &Path, label: &str) -> Result<Json, String> {
    let text = read_utf8(path, label)?;
    let data: Json =
        serde_json::from_str(&text).map_err(|_| format!("{label} is not valid JSON"))?;
    if !data.is_object() {
        return Err(format!("{label} is not a JSON object"));
    }
    Ok(data)
}

fn read_utf8(path: &Path, label: &str) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{label} unreadable ({e})"))?;
    String::from_utf8(bytes).map_err(|_| format!("{label} is not valid UTF-8"))
}

struct Report {
    out: String,
    errs: Vec<String>,
}

impl Report {
    fn fail(&mut self, m: impl Into<String>) {
        let m = m.into();
        self.errs.push(m.clone());
        self.out.push_str("  FAIL: ");
        self.out.push_str(&m);
        self.out.push('\n');
    }

    fn ok(&mut self, m: impl Into<String>) {
        self.out.push_str("  ok: ");
        self.out.push_str(&m.into());
        self.out.push('\n');
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
    fn lists_are_not_empty() {
        assert!(!JS_ASSETS.is_empty());
        assert!(!M01_NEEDLES.is_empty());
        assert_eq!(NAME, "smoke-learn-v2");
    }

    #[test]
    fn json_num_treats_missing_and_null_as_zero() {
        assert_eq!(json_num(None).unwrap(), 0.0);
        assert_eq!(json_num(Some(&Json::Null)).unwrap(), 0.0);
        assert!(json_num(Some(&Json::String("60".into()))).is_err());
    }
}
