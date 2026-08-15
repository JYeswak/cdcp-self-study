//! Offline Reference surface — glossary + power/redundancy cheatsheet.
//!
//! Extracted from `scripts/build_reference.py` by
//! `bd-substrate-rust-migration-jhd.29`. The Python is DELETED. Product,
//! not a gate: a learner opens `web/reference.html`. If they can see it,
//! it is not a `cdcp_gate` concern.
//!
//! # Contract
//!
//! * both declared parent-corpus docs exist and are copied (link-rewritten)
//! * `web/reference.html` carries the honesty banner, relative CSS, and
//!   a panel per doc
//! * a missing parent `reference/` dir or a missing source file is RED
//! * write-after-verdict: a RED compile writes nothing
//! * leftover `web/content/reference/*.md` not in the declared set are swept
//!
//! # What this cannot decide
//!
//! It does not open a browser. A page whose banner is present and whose
//! body is a stub is the same write as a correct page. Completeness of
//! the parent glossary is invisible here.

#![forbid(unsafe_code)]

use crate::{join_rel, BuildOutcome, LearnError};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub const NAME: &str = "build-reference";
pub const SUMMARY: &str = "compile the offline Reference surface (glossary + power cheatsheet)";

pub const CONTENT_DIR_REL: &str = "web/content/reference";
pub const PAGE_REL: &str = "web/reference.html";
pub const PARENT_REFERENCE_REL: &str = "../reference";

const HONESTY: &str = concat!(
    "<strong>Study tool only.</strong>\n",
    "      This tool does <strong>not</strong> grant EPI/EXIN certification.\n",
    "      Completing practice here is not a CDCP credential."
);

/// Declared docs. A compile-in list so emptying it is a source edit, not a
/// silent green of zero copies.
pub const DOCS: &[Doc] = &[
    Doc {
        id: "glossary",
        title: "Glossary",
        src_name: "GLOSSARY.md",
        dest_name: "GLOSSARY.md",
    },
    Doc {
        id: "power",
        title: "Power & redundancy",
        src_name: "POWER-AND-REDUNDANCY-CHEATSHEET.md",
        dest_name: "POWER-AND-REDUNDANCY-CHEATSHEET.md",
    },
];

#[derive(Debug, Clone, Copy)]
pub struct Doc {
    pub id: &'static str,
    pub title: &'static str,
    pub src_name: &'static str,
    pub dest_name: &'static str,
}

/// Parent markdown → in-app paths. Order matters for overlapping prefixes.
pub const LINK_REWRITES: &[(&str, &str)] = &[
    ("../practice/DRILL-CARDS.md", "drill.html"),
    ("../practice/PRACTICE-EXAM.md", "mock.html"),
    ("../modules/06-power.md", "learn/06-power.html"),
    ("../modules/09-cooling.md", "learn/09-cooling.html"),
    ("./GLOSSARY.md", "#glossary"),
    ("GLOSSARY.md", "#glossary"),
    ("./POWER-AND-REDUNDANCY-CHEATSHEET.md", "#power"),
    ("POWER-AND-REDUNDANCY-CHEATSHEET.md", "#power"),
];

#[derive(Debug, Clone)]
pub struct RefPlan {
    pub stdout: String,
    pub code: i32,
    pub writes: Vec<(PathBuf, Vec<u8>)>,
    pub unlinks: Vec<PathBuf>,
}

impl RefPlan {
    pub fn outcome(&self) -> BuildOutcome {
        let artifact = self
            .writes
            .iter()
            .find(|(p, _)| p.ends_with("reference.html"))
            .map(|(p, b)| (p.clone(), String::from_utf8_lossy(b).into_owned()));
        BuildOutcome {
            stdout: self.stdout.clone(),
            code: self.code,
            artifact: if self.code == 0 { artifact } else { None },
        }
    }
}

/// Rewrite parent-corpus markdown links to in-app hrefs.
pub fn rewrite_links(text: &str) -> String {
    let mut out = text.to_string();
    for (old, new) in LINK_REWRITES {
        let needle = format!("]({old})");
        let repl = format!("]({new})");
        out = out.replace(&needle, &repl);
        // ](old "title") — space after the path
        let prefix = format!("]({old} ");
        let new_prefix = format!("]({new} ");
        out = out.replace(&prefix, &new_prefix);
    }
    out
}

pub fn evaluate(root: &Path) -> Result<RefPlan, LearnError> {
    if DOCS.is_empty() {
        return Ok(fail(vec![
            "DOCS is empty — a reference compile that copies nothing certifies nothing".into(),
        ]));
    }
    let src_dir = join_rel(root, PARENT_REFERENCE_REL);
    if !src_dir.is_dir() {
        return Ok(fail(vec![format!(
            "missing parent reference dir {}",
            src_dir.display()
        )]));
    }

    let mut errors = Vec::new();
    let mut copies: Vec<(PathBuf, Vec<u8>, &'static Doc)> = Vec::new();
    for doc in DOCS {
        let src = src_dir.join(doc.src_name);
        if !src.is_file() {
            errors.push(format!("missing {}", src.display()));
            continue;
        }
        let raw = match std::fs::read_to_string(&src) {
            Ok(t) => t,
            Err(e) => {
                errors.push(format!("{} unreadable: {e}", src.display()));
                continue;
            }
        };
        let text = rewrite_links(&raw);
        let dest = join_rel(root, CONTENT_DIR_REL).join(doc.dest_name);
        copies.push((dest, text.into_bytes(), doc));
    }
    if !errors.is_empty() {
        return Ok(fail(errors));
    }

    let html = render_page();
    let mut check_errors = Vec::new();
    if !html.contains("does <strong>not</strong> grant EPI/EXIN certification") {
        check_errors.push("reference.html missing honesty non-grant language".into());
    }
    if !html.contains("href=\"assets/css/course.css\"") {
        check_errors.push("reference.html missing course.css".into());
    }
    let html_lc = html.to_ascii_lowercase();
    if html_lc.contains("cdn.") || html_lc.contains("https://cdn") {
        check_errors.push("reference.html must not pull a CDN".into());
    }
    for doc in DOCS {
        if !html.contains(&format!("content/reference/{}", doc.dest_name)) {
            check_errors.push(format!(
                "reference.html missing panel for {}",
                doc.dest_name
            ));
        }
    }
    if !check_errors.is_empty() {
        return Ok(fail(check_errors));
    }

    let mut writes: Vec<(PathBuf, Vec<u8>)> = copies
        .iter()
        .map(|(p, b, _)| (p.clone(), b.clone()))
        .collect();
    writes.push((join_rel(root, PAGE_REL), html.into_bytes()));

    let keep: BTreeSet<&str> = DOCS.iter().map(|d| d.dest_name).collect();
    let mut unlinks = Vec::new();
    let content_dir = join_rel(root, CONTENT_DIR_REL);
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
            if !keep.contains(name.as_str()) {
                unlinks.push(path);
            }
        }
    }

    let mut stdout = String::from("PASS: build_reference\n");
    for (_, bytes, doc) in &copies {
        stdout.push_str(&format!(
            "  {}: content/reference/{} ({} bytes)\n",
            doc.id,
            doc.dest_name,
            bytes.len()
        ));
    }
    stdout.push_str(&format!("  page={PAGE_REL}\n"));
    Ok(RefPlan {
        stdout,
        code: 0,
        writes,
        unlinks,
    })
}

pub fn write_reference(root: &Path) -> Result<BuildOutcome, LearnError> {
    let plan = evaluate(root)?;
    debug_assert!(
        plan.code == 0 || (plan.writes.is_empty() && plan.unlinks.is_empty()),
        "a failing run must not carry writes"
    );
    if plan.code == 0 {
        apply(&plan)?;
    }
    Ok(plan.outcome())
}

pub fn apply(plan: &RefPlan) -> Result<(), LearnError> {
    if plan.code != 0 {
        return Err(LearnError::io(
            "refusing to apply a RED Reference plan — write-after-verdict",
        ));
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

fn fail(errors: Vec<String>) -> RefPlan {
    let mut report = vec!["FAIL: build_reference".to_string()];
    for e in &errors {
        report.push(format!("  - {e}"));
    }
    RefPlan {
        stdout: format!("{}\n", report.join("\n")),
        code: 1,
        writes: Vec::new(),
        unlinks: Vec::new(),
    }
}

fn render_page() -> String {
    let mut tab_buttons = Vec::new();
    let mut panels = Vec::new();
    for (i, doc) in DOCS.iter().enumerate() {
        let selected = if i == 0 { "true" } else { "false" };
        let hidden = if i == 0 { "" } else { " hidden" };
        tab_buttons.push(format!(
            "        <button type=\"button\" class=\"ref-tabs__btn\" role=\"tab\"\n          id=\"tab-{id}\" data-ref-id=\"{id}\"\n          aria-controls=\"panel-{id}\" aria-selected=\"{selected}\">\n          {title}\n        </button>",
            id = doc.id,
            title = doc.title,
            selected = selected,
        ));
        panels.push(format!(
            "      <section class=\"ref-panel\" role=\"tabpanel\"\n        id=\"panel-{id}\" data-ref-id=\"{id}\"\n        aria-labelledby=\"tab-{id}\"{hidden}\n        data-content-href=\"content/reference/{dest}\">\n        <article class=\"prose\" id=\"prose-{id}\" aria-busy=\"true\">\n          <p class=\"lede\">Loading {title}…</p>\n        </article>\n      </section>",
            id = doc.id,
            dest = doc.dest_name,
            hidden = hidden,
            title = doc.title.to_lowercase(),
        ));
    }
    let tabs_html = tab_buttons.join("\n");
    let panels_html = panels.join("\n");
    format!(
        r###"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <meta name="description" content="CDCP reference — glossary and power/redundancy cheatsheet. Study tool only; does not grant EPI/EXIN certification.">
  <title>CDCP Study — Reference</title>
  <link rel="stylesheet" href="assets/css/course.css">
  <script src="assets/js/origin_guard.js"></script>
</head>
<body>
  <a class="skip-link" href="#main">Skip to main content</a>

  <header class="site-header">
    <div class="honesty-banner" role="status">
      {HONESTY}
    </div>
    <div class="site-header__inner">
      <a class="brand" href="index.html">
        <span class="brand__title">CDCP Study</span>
        <span class="brand__sub">Self-study · local HTTP</span>
      </a>
      <nav aria-label="Hub">
        <ul class="hub-nav">
          <li><a href="learn.html">Learn</a></li>
          <li><a href="drill.html">Drill</a></li>
          <li><a href="mock.html">Mock</a></li>
          <li><a href="reference.html" aria-current="page">Reference</a></li>
        </ul>
      </nav>
    </div>
  </header>

  <main id="main" class="wrap wrap-learn" tabindex="-1">
    <h1>Reference</h1>
    <p class="lede">
      Local glossary and power/redundancy cheatsheet from the parent study corpus.
      Interview-ready shorthand only — not an official EPI dictionary and not a design manual.
    </p>

    <div class="ref-tabs" role="tablist" aria-label="Reference documents">
{tabs_html}
    </div>

{panels_html}

    <p class="meta">
      <a href="index.html">← Hub</a>
      · <a href="learn.html">Learn</a>
      · <a href="learn/06-power.html">Module 06 power</a>
      · source <span class="mono">web/content/reference/</span>
      · study notes only · not an EPI/EXIN credential.
    </p>
  </main>

  <script src="assets/js/learn_md.js"></script>
  <script src="assets/js/reference.js"></script>
</body>
</html>
"###
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrite_replaces_markdown_and_titled_links() {
        let src = "[g](GLOSSARY.md) [p](../modules/06-power.md \"Power\")";
        let got = rewrite_links(src);
        assert!(got.contains("](#glossary)"), "{got}");
        assert!(got.contains("](learn/06-power.html \"Power\")"), "{got}");
    }

    #[test]
    fn docs_inventory_is_non_empty() {
        assert!(
            !DOCS.is_empty(),
            "DOCS is empty — a compile that copies nothing certifies nothing"
        );
    }
}
