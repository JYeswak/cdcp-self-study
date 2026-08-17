//! Learn UI chrome smoke — TOC / math / continue / power-path wiring.
//!
//! Static file checks on the learner-visible Learn surface. Does **not** drive
//! a browser. Extracted from `scripts/smoke_learn_chrome.py` by
//! `bd-substrate-rust-migration-jhd.15`. The Python is DELETED; this module
//! asserts the product contract, not a stdout replica of a retired script.
//!
//! # Contract
//!
//! * `learn_chrome.js` is present
//! * `learn_md.js` is present and carries the latex / `math-block` path
//! * the M01 page hosts TOC + progress bar + chrome script
//! * the M06 page embeds the power-path diagram CTA
//! * the Learn hub hosts the continue chip and chrome script
//! * `modules_index.json` has at least one navigable module, each with a
//!   present `estimate_minutes` or `word_count` (zero / missing = missing)
//! * the power-path diagram file exists
//! * `course.css` carries the four chrome hooks
//!
//! # Anti-vacuous
//!
//! A run that performed fewer than `MIN_CHECKS` checks cannot PASS. An empty
//! `web/` is RED. A modules index with no navigable modules is RED. A missing
//! CSS file is a named FAIL, not a panic. Unparseable JSON is a named FAIL.
//! Empty is an ERROR, never a pass.

#![forbid(unsafe_code)]

use crate::join_rel;
use serde_json::Value;
use std::path::Path;

pub const NAME: &str = "smoke-learn-chrome";
pub const SUMMARY: &str = "static smoke: Learn UI chrome (TOC/math/continue/power embed)";

pub const CHROME_JS: &str = "web/assets/js/learn_chrome.js";
pub const MD_JS: &str = "web/assets/js/learn_md.js";
pub const M01: &str = "web/learn/01-mission-critical.html";
pub const M06: &str = "web/learn/06-power.html";
pub const HUB: &str = "web/learn.html";
pub const INDEX: &str = "web/data/modules_index.json";
pub const POWER_PATH: &str = "web/diagrams/power-path.html";
pub const CSS: &str = "web/assets/css/course.css";

pub const M01_NEEDLES: &[&str] = &[
    "id=\"learn-toc\"",
    "id=\"learn-progress-bar\"",
    "learn_chrome.js",
];
pub const M06_NEEDLES: &[&str] = &["diagrams/power-path.html", "diagram-cta"];
pub const HUB_NEEDLES: &[&str] = &["id=\"learn-continue\"", "learn_chrome.js"];
pub const CSS_HOOKS: &[&str] = &[
    ".learn-toc",
    ".math-block",
    ".learn-continue",
    ".diagram-cta",
];

/// Compiled-in floor so emptying the check list cannot go green.
///
/// Live / green fixture count:
///   chrome.js + md latex path + M01 needles + M06 needles + hub needles
///   + index eta + power-path + CSS hooks
///
/// = 1 + 1 + 3 + 2 + 2 + 1 + 1 + 4 = 15
pub const MIN_CHECKS: usize = 15;

/// The four presence legs that are not needle/hook lists.
pub const PRESENCE_LEGS: usize = 4;

#[derive(Debug, Clone)]
pub struct SmokeReport {
    pub stdout: String,
    pub code: i32,
    pub checks: usize,
    pub errors: usize,
}

struct Run {
    out: String,
    errs: Vec<String>,
    checks: usize,
}

impl Run {
    fn ok(&mut self, msg: &str) {
        self.checks += 1;
        self.out.push_str("  ok: ");
        self.out.push_str(msg);
        self.out.push('\n');
    }

    fn fail(&mut self, msg: &str) {
        self.checks += 1;
        self.errs.push(msg.to_string());
        self.out.push_str("  FAIL: ");
        self.out.push_str(msg);
        self.out.push('\n');
    }
}

/// Python-ish truthiness for a JSON value. `0`, `""`, `[]`, `{}`, `null`,
/// missing, and `false` are absent — matching `if not m.get("estimate_minutes")`.
pub fn json_present(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64().map(|x| x != 0.0).unwrap_or(false),
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Array(a)) => !a.is_empty(),
        Some(Value::Object(o)) => !o.is_empty(),
    }
}

fn read_utf8(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

fn needles(run: &mut Run, body: &str, rel: &str, want: &[&str]) {
    for needle in want {
        if body.contains(needle) {
            run.ok(&format!("{rel} has {needle}"));
        } else {
            run.fail(&format!("{rel} missing {needle}"));
        }
    }
}

/// Run the chrome smoke against an engine root (the directory that holds `web/`).
pub fn smoke(root: &Path) -> SmokeReport {
    let mut run = Run {
        out: String::from("==> smoke_learn_chrome (M8-A)\n"),
        errs: Vec::new(),
        checks: 0,
    };

    let chrome = join_rel(root, CHROME_JS);
    if chrome.is_file() {
        run.ok("learn_chrome.js");
    } else {
        run.fail("missing learn_chrome.js");
    }

    let mdjs = join_rel(root, MD_JS);
    if !mdjs.is_file() {
        run.fail("missing learn_md.js");
    } else {
        match read_utf8(&mdjs) {
            Ok(text) => {
                if text.contains("latexToHtml") && text.contains("math-block") {
                    run.ok("learn_md formula path");
                } else {
                    run.fail("learn_md.js missing latex/math-block path");
                }
            }
            Err(e) => run.fail(&format!("unreadable learn_md.js: {e}")),
        }
    }

    let m01 = join_rel(root, M01);
    if !m01.is_file() {
        run.fail("missing learn/01-mission-critical.html — run `cdcp build-learn`");
    } else {
        match read_utf8(&m01) {
            Ok(h) => needles(&mut run, &h, "M01 page", M01_NEEDLES),
            Err(e) => run.fail(&format!("unreadable {M01}: {e}")),
        }
    }

    let m06 = join_rel(root, M06);
    if !m06.is_file() {
        run.fail("missing learn/06-power.html");
    } else {
        match read_utf8(&m06) {
            Ok(h) => needles(&mut run, &h, "M06 page", M06_NEEDLES),
            Err(e) => run.fail(&format!("unreadable {M06}: {e}")),
        }
    }

    let hub = join_rel(root, HUB);
    if !hub.is_file() {
        run.fail("missing learn.html");
    } else {
        match read_utf8(&hub) {
            Ok(h) => needles(&mut run, &h, "learn.html", HUB_NEEDLES),
            Err(e) => run.fail(&format!("unreadable {HUB}: {e}")),
        }
    }

    let idx = join_rel(root, INDEX);
    if !idx.is_file() {
        run.fail("missing modules_index.json");
    } else {
        match read_utf8(&idx) {
            Ok(raw) => match serde_json::from_str::<Value>(&raw) {
                Ok(data) => check_modules_index(&mut run, &data),
                Err(e) => run.fail(&format!("modules_index.json is not JSON: {e}")),
            },
            Err(e) => run.fail(&format!("unreadable {INDEX}: {e}")),
        }
    }

    let pp = join_rel(root, POWER_PATH);
    if pp.is_file() {
        run.ok("power-path diagram file");
    } else {
        run.fail("missing diagrams/power-path.html");
    }

    let css_path = join_rel(root, CSS);
    if !css_path.is_file() {
        run.fail("missing course.css");
    } else {
        match read_utf8(&css_path) {
            Ok(css) => {
                for cls in CSS_HOOKS {
                    if css.contains(cls) {
                        run.ok(&format!("css {cls}"));
                    } else {
                        run.fail(&format!("course.css missing {cls}"));
                    }
                }
            }
            Err(e) => run.fail(&format!("unreadable {CSS}: {e}")),
        }
    }

    if run.errs.is_empty() && run.checks < MIN_CHECKS {
        run.fail(&format!(
            "performed {} check(s) < MIN_CHECKS={MIN_CHECKS} — a smoke that checked nothing cannot PASS",
            run.checks
        ));
    }

    let errors = run.errs.len();
    if errors > 0 {
        run.out
            .push_str(&format!("smoke_learn_chrome: FAIL ({errors} errors)\n"));
        SmokeReport {
            stdout: run.out,
            code: 1,
            checks: run.checks,
            errors,
        }
    } else {
        run.out.push_str("smoke_learn_chrome: PASS\n");
        SmokeReport {
            stdout: run.out,
            code: 0,
            checks: run.checks,
            errors: 0,
        }
    }
}

fn check_modules_index(run: &mut Run, data: &Value) {
    let Some(mods) = data.get("modules").and_then(|v| v.as_array()) else {
        run.fail("modules_index.json missing modules array");
        return;
    };
    let mut navigable: Vec<&Value> = Vec::new();
    for (i, m) in mods.iter().enumerate() {
        let Some(obj) = m.as_object() else {
            run.fail(&format!("modules[{i}] is not an object"));
            continue;
        };
        if json_present(obj.get("empty")) {
            continue;
        }
        navigable.push(m);
    }
    if navigable.is_empty() {
        run.fail("no navigable modules in index");
        return;
    }
    let mut missing_eta: Vec<String> = Vec::new();
    for m in &navigable {
        let obj = m.as_object().expect("filtered to objects");
        let id = obj
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if id.is_empty() {
            run.fail("module missing id");
            continue;
        }
        if !json_present(obj.get("estimate_minutes")) && !json_present(obj.get("word_count")) {
            missing_eta.push(id);
        }
    }
    if !missing_eta.is_empty() {
        let shown: Vec<&str> = missing_eta.iter().take(3).map(String::as_str).collect();
        run.fail(&format!("modules missing word_count/eta: {shown:?}"));
    } else {
        run.ok(&format!(
            "modules_index eta fields on {} modules",
            navigable.len()
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::assertions_on_constants)] // anti-vacuous: MIN_CHECKS=0 is a deleted floor
    fn min_checks_matches_compiled_lists() {
        assert_eq!(
            MIN_CHECKS,
            PRESENCE_LEGS
                + M01_NEEDLES.len()
                + M06_NEEDLES.len()
                + HUB_NEEDLES.len()
                + CSS_HOOKS.len()
        );
        assert!(
            MIN_CHECKS > 0,
            "MIN_CHECKS=0 would make a deleted check list a pass"
        );
        assert!(!M01_NEEDLES.is_empty());
        assert!(!M06_NEEDLES.is_empty());
        assert!(!HUB_NEEDLES.is_empty());
        assert!(!CSS_HOOKS.is_empty());
    }

    #[test]
    fn json_present_matches_python_truthiness() {
        assert!(!json_present(None));
        assert!(!json_present(Some(&Value::Null)));
        assert!(!json_present(Some(&Value::Bool(false))));
        assert!(json_present(Some(&Value::Bool(true))));
        assert!(!json_present(Some(&Value::from(0))));
        assert!(json_present(Some(&Value::from(24))));
        assert!(!json_present(Some(&Value::from(""))));
        assert!(json_present(Some(&Value::from("24"))));
        assert!(!json_present(Some(&Value::Array(vec![]))));
        assert!(!json_present(Some(&Value::Object(Default::default()))));
    }
}
