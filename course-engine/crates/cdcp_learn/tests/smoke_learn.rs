//! Verdict suite for `cdcp_learn::smoke` (bd-substrate-rust-migration-jhd.17).
//!
//! EXTRACT-THEN-DELETE: this is NOT a differential against
//! `scripts/smoke_learn.py`. The Python is deleted in the same commit. Every
//! case asserts WHAT THE CORRECT ANSWER IS against the Rust alone.
//!
//! The parked 1,435-line `cdcp_gate` transcription (.parked-wave8/smoke_learn.rs)
//! stays parked. Empty input is an ERROR — the inherited all-empty-ok vacuous
//! PASS (bd-smoke-learn-vacuous-empty-ok-9d3n) is closed, not reproduced.
//!
//! This smoke is a READER. Fixtures live under $TMPDIR. The live-tree claim
//! is bought by copying the tracked surface and asserting the copy matches
//! before the smoke runs.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Cases actually run, so "the suite ran" is itself checked.
static RAN: AtomicUsize = AtomicUsize::new(0);

/// Raise when you add a `#[test]`. A DROP means a case was deleted.
const EXPECTED_CASES: usize = 28;

fn engine_root() -> PathBuf {
    cdcp_learn::resolve_engine_root(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
}

fn tick() {
    RAN.fetch_add(1, Ordering::SeqCst);
}

fn run(root: &Path) -> cdcp_learn::BuildOutcome {
    cdcp_learn::smoke::run(root)
}

// ── fixture plumbing ───────────────────────────────────────────────────────

/// `<tmp>/study/engine` is the engine ROOT; `<tmp>/study/modules` is the
/// parent corpus `../modules/...` resolves against.
struct Tree {
    _dir: tempfile::TempDir,
    root: PathBuf,
    corpus: PathBuf,
}

impl Tree {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let base = dir.path().canonicalize().expect("canonicalize");
        let root = base.join("study/engine");
        let corpus = base.join("study/modules");
        for d in [
            "knowledge",
            "scripts",
            "registries",
            "web/data",
            "web/learn",
            "web/content/modules",
        ] {
            std::fs::create_dir_all(root.join(d)).unwrap();
        }
        std::fs::create_dir_all(&corpus).unwrap();
        std::fs::write(root.join("registries/claims.toml"), "schema_version = 1\n").unwrap();
        Tree {
            _dir: dir,
            root,
            corpus,
        }
    }

    fn write(&self, rel: &str, body: &str) -> &Self {
        let p = self.root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
        self
    }

    fn remove(&self, rel: &str) -> &Self {
        let p = self.root.join(rel);
        if p.is_file() {
            std::fs::remove_file(p).unwrap();
        }
        self
    }

    fn note(&self, name: &str) -> &Self {
        let p = self.corpus.join(name);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, "# corpus notes\n\nEnough bytes to clear the floor.\n").unwrap();
        self
    }
}

fn page(id: &str) -> String {
    format!(
        "<!doctype html>\n<html><head>\n\
         <link rel=\"stylesheet\" href=\"../assets/css/course.css\">\n\
         </head><body>\n\
         <p>Study tool only. This tool does <strong>not</strong> grant EPI/EXIN certification.</p>\n\
         <div id=\"module-md\"></div>\n\
         <script src=\"../assets/js/learn_progress.js\"></script>\n\
         <script src=\"../assets/js/learn_md.js\"></script>\n\
         <script src=\"../assets/js/learn_reader.js\"></script>\n\
         <!-- content/modules/{id}.md -->\n\
         </body></html>\n"
    )
}

fn hub(ids: &[&str]) -> String {
    let links: String = ids
        .iter()
        .map(|i| format!("  <a href=\"learn/{i}.html\">{i}</a>\n"))
        .collect();
    format!(
        "<!doctype html>\n<html><head>\n\
         <link rel=\"stylesheet\" href=\"assets/css/course.css\">\n\
         </head><body>\n\
         <p>Study tool only. This tool does <strong>not</strong> grant EPI/EXIN certification.</p>\n\
         {links}</body></html>\n"
    )
}

fn index_json(rows: &[(&str, bool)]) -> String {
    let navigable = rows.iter().filter(|(_, e)| !*e).count();
    let mods: Vec<String> = rows
        .iter()
        .map(|(id, empty)| {
            if *empty {
                format!("    {{\"id\": \"{id}\", \"empty\": true, \"href\": null}}")
            } else {
                format!(
                    "    {{\"id\": \"{id}\", \"empty\": false, \"href\": \"learn/{id}.html\"}}"
                )
            }
        })
        .collect();
    format!(
        "{{\n  \"navigable_count\": {navigable},\n  \"modules\": [\n{}\n  ]\n}}\n",
        mods.join(",\n")
    )
}

fn green_tree() -> Tree {
    let t = Tree::new();
    let ids = ["alpha", "beta", "gamma"];
    let mut doms = String::new();
    for (n, id) in ids.iter().enumerate() {
        t.note(&format!("{id}.md"));
        t.write(&format!("web/learn/{id}.html"), &page(id));
        t.write(
            &format!("web/content/modules/{id}.md"),
            "# heading\n\nBody text long enough to clear the size floor.\n",
        );
        doms.push_str(&format!(
            "[[domain]]\nid = \"{id}\"\norder = {}\nprimary_notes = \"../modules/{id}.md\"\n\n",
            n + 1
        ));
    }
    t.write("knowledge/domains.toml", &doms);
    t.write(
        "web/data/modules_index.json",
        &index_json(&[("alpha", false), ("beta", false), ("gamma", false)]),
    );
    t.write("web/learn.html", &hub(&ids));
    t
}

const LIVE_FILES: &[&str] = &[
    "knowledge/domains.toml",
    "web/data/modules_index.json",
    "web/learn.html",
];
const LIVE_DIRS: &[&str] = &["web/learn", "web/content/modules"];

fn live_copy() -> Tree {
    let t = Tree::new();
    let src = engine_root();
    for rel in LIVE_FILES {
        let body = std::fs::read(src.join(rel))
            .unwrap_or_else(|e| panic!("live tree must have {rel} ({e})"));
        let dst = t.root.join(rel);
        std::fs::create_dir_all(dst.parent().unwrap()).unwrap();
        std::fs::write(dst, body).unwrap();
    }
    for d in LIVE_DIRS {
        for e in std::fs::read_dir(src.join(d))
            .unwrap_or_else(|e| panic!("live tree must have {d}/ ({e})"))
            .flatten()
        {
            if e.path().is_file() {
                let dst = t.root.join(d).join(e.file_name());
                std::fs::create_dir_all(dst.parent().unwrap()).unwrap();
                std::fs::copy(e.path(), dst).unwrap();
            }
        }
    }
    let corpus_src = src.parent().expect("engine parent").join("modules");
    for e in std::fs::read_dir(&corpus_src)
        .unwrap_or_else(|e| panic!("parent corpus must exist ({e})"))
        .flatten()
    {
        if e.path().is_file() {
            std::fs::copy(e.path(), t.corpus.join(e.file_name())).unwrap();
        }
    }
    t
}

fn snapshot(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(base: &Path, cur: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        let Ok(rd) = std::fs::read_dir(cur) else {
            return;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(base, &p, out);
            } else if p.is_file() {
                let rel = p.strip_prefix(base).unwrap().to_string_lossy().into_owned();
                out.insert(rel, std::fs::read(&p).unwrap_or_default());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(dir, dir, &mut out);
    out
}

// ── live tree ──────────────────────────────────────────────────────────────

#[test]
fn live_copy_is_green_and_checked_something() {
    tick();
    let t = live_copy();
    let o = run(&t.root);
    assert_eq!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout.starts_with("PASS: smoke_learn\n"),
        "{}",
        o.stdout
    );
    let n: u32 = o
        .stdout
        .lines()
        .find_map(|l| l.trim().strip_prefix("primary_notes_checked="))
        .expect("checked count")
        .parse()
        .expect("number");
    assert!(n > 0, "a live run that checked zero notes is an ERROR");
    assert!(o.artifact.is_none(), "smoke is a reader");
}

#[test]
fn smoke_writes_nothing() {
    tick();
    let t = live_copy();
    let before = snapshot(&t.root);
    let _ = run(&t.root);
    let after = snapshot(&t.root);
    assert_eq!(before, after, "smoke_learn wrote to the tree");
    assert!(
        before.len() > 10,
        "an empty snapshot is an ERROR, not a pass ({} files)",
        before.len()
    );
}

#[test]
fn synthetic_green_tree_passes() {
    tick();
    let t = green_tree();
    let o = run(&t.root);
    assert_eq!(o.code, 0, "{}", o.stdout);
    assert!(o.stdout.contains("primary_notes_checked=3"), "{}", o.stdout);
    assert!(o.stdout.contains("index_modules=3"), "{}", o.stdout);
}

// ── registry legs ──────────────────────────────────────────────────────────

#[test]
fn missing_domains_registry_is_the_early_exit() {
    tick();
    let t = green_tree();
    t.remove("knowledge/domains.toml");
    let o = run(&t.root);
    assert_eq!(o.code, 1);
    assert_eq!(o.stdout, "FAIL: knowledge/domains.toml missing\n");
}

#[test]
fn zero_domain_rows_is_red() {
    tick();
    let t = green_tree();
    t.write("knowledge/domains.toml", "schema_version = 1\n");
    let o = run(&t.root);
    assert_eq!(o.code, 1);
    assert!(o.stdout.contains("zero [[domain]] rows"), "{}", o.stdout);
    assert!(
        o.stdout.contains("zero primary_notes resolved"),
        "empty input must also trip: {}",
        o.stdout
    );
}

#[test]
fn primary_notes_field_absent_is_red() {
    tick();
    let t = green_tree();
    t.write(
        "knowledge/domains.toml",
        "[[domain]]\nid = \"alpha\"\norder = 1\n",
    );
    let o = run(&t.root);
    assert_eq!(o.code, 1);
    assert!(
        o.stdout.contains("primary_notes field missing"),
        "{}",
        o.stdout
    );
}

#[test]
fn empty_primary_notes_without_the_licence_is_red() {
    tick();
    let t = green_tree();
    t.write(
        "knowledge/domains.toml",
        "[[domain]]\nid = \"alpha\"\norder = 1\nprimary_notes = \"   \"\n",
    );
    let o = run(&t.root);
    assert_eq!(o.code, 1);
    assert!(
        o.stdout.contains("without exam_weight_unknown=true"),
        "{}",
        o.stdout
    );
}

#[test]
fn unresolvable_primary_notes_names_the_path() {
    tick();
    let t = green_tree();
    t.write(
        "knowledge/domains.toml",
        "[[domain]]\nid = \"alpha\"\norder = 1\nprimary_notes = \"../modules/nope.md\"\n",
    );
    let o = run(&t.root);
    assert_eq!(o.code, 1);
    assert!(
        o.stdout.contains("primary_notes does not resolve: ../modules/nope.md"),
        "{}",
        o.stdout
    );
}

// ── index legs ─────────────────────────────────────────────────────────────

#[test]
fn missing_index_is_red_and_names_the_builder() {
    tick();
    let t = green_tree();
    t.remove("web/data/modules_index.json");
    let o = run(&t.root);
    assert_eq!(o.code, 1);
    assert!(
        o.stdout.contains("run scripts/build_learn.py"),
        "{}",
        o.stdout
    );
}

#[test]
fn invalid_index_json_is_red() {
    tick();
    for (body, case) in [
        ("not json at all\n", "not-json"),
        ("{\"modules\": [}\n", "bad-value"),
        ("{\"modules\": []\n", "truncated"),
        ("{} extra\n", "extra-data"),
    ] {
        let t = green_tree();
        t.write("web/data/modules_index.json", body);
        let o = run(&t.root);
        assert_eq!(o.code, 1, "[{case}] {}", o.stdout);
        assert!(
            o.stdout.contains("modules_index.json invalid JSON:"),
            "[{case}] {}",
            o.stdout
        );
    }
}

#[test]
fn zero_modules_in_the_index_is_red() {
    tick();
    let t = green_tree();
    t.write("web/data/modules_index.json", "{\"modules\": []}\n");
    let o = run(&t.root);
    assert_eq!(o.code, 1);
    assert!(
        o.stdout.contains("modules_index.json has zero modules"),
        "{}",
        o.stdout
    );
}

#[test]
fn id_set_disagreement_reports_both_directions() {
    tick();
    let t = green_tree();
    t.write(
        "web/data/modules_index.json",
        &index_json(&[("alpha", false), ("delta", false)]),
    );
    let o = run(&t.root);
    assert_eq!(o.code, 1);
    assert!(
        o.stdout.contains("index missing domain ids: beta, gamma"),
        "{}",
        o.stdout
    );
    assert!(
        o.stdout.contains("index has unknown domain ids: delta"),
        "{}",
        o.stdout
    );
}

#[test]
fn an_empty_ok_module_carrying_an_href_is_red() {
    tick();
    let t = green_tree();
    t.write(
        "web/data/modules_index.json",
        "{\"navigable_count\": 0, \"modules\": [\
         {\"id\": \"alpha\", \"empty\": true, \"href\": \"learn/alpha.html\"},\
         {\"id\": \"beta\", \"empty\": true, \"href\": null},\
         {\"id\": \"gamma\", \"empty\": true, \"href\": \"\"}]}\n",
    );
    let o = run(&t.root);
    assert_eq!(o.code, 1);
    assert!(
        o.stdout.contains("empty-ok domain must not have href (got learn/alpha.html)"),
        "{}",
        o.stdout
    );
    assert!(!o.stdout.contains("beta: empty-ok"), "null href is allowed: {}", o.stdout);
    assert!(!o.stdout.contains("gamma: empty-ok"), "empty href is allowed: {}", o.stdout);
}

#[test]
fn href_shape_legs() {
    tick();
    for (href, needle, case) in [
        ("", "navigable module missing href", "no-href"),
        (
            "https://cdn.example/alpha.html",
            "href must be relative offline path",
            "absolute-https",
        ),
        ("/learn/alpha.html", "href must be relative offline path", "root-relative"),
        ("pages/alpha.html", "unexpected href shape", "wrong-prefix"),
        ("learn/alpha.htm", "unexpected href shape", "wrong-suffix"),
    ] {
        let t = green_tree();
        t.write(
            "web/data/modules_index.json",
            &format!(
                "{{\"navigable_count\": 1, \"modules\": [{{\"id\": \"alpha\", \"empty\": false, \"href\": \"{href}\"}}]}}\n"
            ),
        );
        let o = run(&t.root);
        assert_eq!(o.code, 1, "[{case}] {}", o.stdout);
        assert!(
            o.stdout.contains(needle),
            "[{case}] {}",
            o.stdout
        );
    }
}

// ── page / content / hub ───────────────────────────────────────────────────

#[test]
fn a_missing_learn_page_is_red() {
    tick();
    let t = green_tree();
    t.remove("web/learn/alpha.html");
    let o = run(&t.root);
    assert_eq!(o.code, 1);
    assert!(
        o.stdout.contains("missing learn page web/learn/alpha.html"),
        "{}",
        o.stdout
    );
}

#[test]
fn page_shape_legs_one_perturbation_at_a_time() {
    tick();
    let base = page("alpha");
    for (from, to, needle, case) in [
        (
            "does <strong>not</strong> grant",
            "does grant",
            "missing honesty non-grant banner",
            "no-honesty",
        ),
        (
            "href=\"../assets/css/course.css\"",
            "href=\"/assets/css/course.css\"",
            "css must be relative",
            "absolute-css",
        ),
        (
            "src=\"../assets/js/learn_progress.js\"",
            "src=\"/assets/js/learn_progress.js\"",
            "must load relative learn_progress.js",
            "absolute-progress",
        ),
        (
            "src=\"../assets/js/learn_md.js\"",
            "src=\"/assets/js/learn_md.js\"",
            "must load relative learn_md.js",
            "absolute-md",
        ),
    ] {
        let t = green_tree();
        t.write("web/learn/alpha.html", &base.replace(from, to));
        let o = run(&t.root);
        assert_eq!(o.code, 1, "[{case}] {}", o.stdout);
        assert!(o.stdout.contains(needle), "[{case}] {}", o.stdout);
    }
}

#[test]
fn a_page_with_neither_reader_nor_embed_reports_both_legs() {
    tick();
    let t = green_tree();
    let body = page("alpha")
        .replace("<div id=\"module-md\"></div>\n", "")
        .replace("<script src=\"../assets/js/learn_reader.js\"></script>\n", "")
        .replace("<!-- content/modules/alpha.md -->\n", "");
    t.write("web/learn/alpha.html", &body);
    let o = run(&t.root);
    assert_eq!(o.code, 1);
    assert!(
        o.stdout.contains("must load learn_reader.js or embed #module-md"),
        "{}",
        o.stdout
    );
    assert!(
        o.stdout.contains("must embed #module-md or fetch content/modules/alpha.md"),
        "{}",
        o.stdout
    );
}

#[test]
fn a_tiny_content_copy_is_red() {
    tick();
    let t = green_tree();
    t.write("web/content/modules/alpha.md", "# x\n");
    let o = run(&t.root);
    assert_eq!(o.code, 1);
    assert!(
        o.stdout.contains("content copy is empty/tiny"),
        "{}",
        o.stdout
    );
}

#[test]
fn a_missing_content_copy_is_green_while_the_corpus_source_resolves() {
    tick();
    let t = green_tree();
    t.remove("web/content/modules/alpha.md");
    let o = run(&t.root);
    assert_eq!(o.code, 0, "fallback to corpus source: {}", o.stdout);
}

#[test]
fn a_missing_content_copy_with_no_source_is_red() {
    tick();
    let t = green_tree();
    t.remove("web/content/modules/alpha.md");
    std::fs::remove_file(t.corpus.join("alpha.md")).unwrap();
    let o = run(&t.root);
    assert_eq!(o.code, 1);
    assert!(
        o.stdout.contains("missing content copy and primary_notes source"),
        "{}",
        o.stdout
    );
}

#[test]
fn a_missing_hub_is_red() {
    tick();
    let t = green_tree();
    t.remove("web/learn.html");
    let o = run(&t.root);
    assert_eq!(o.code, 1);
    assert!(o.stdout.contains("missing web/learn.html"), "{}", o.stdout);
}

#[test]
fn hub_shape_legs() {
    tick();
    for (from, to, needle, case) in [
        (
            "does <strong>not</strong> grant",
            "does grant",
            "web/learn.html missing honesty non-grant banner",
            "hub-no-honesty",
        ),
        (
            "href=\"assets/css/course.css\"",
            "href=\"/assets/css/course.css\"",
            "web/learn.html css must be relative",
            "hub-absolute-css",
        ),
    ] {
        let t = green_tree();
        let body = hub(&["alpha", "beta", "gamma"]).replace(from, to);
        t.write("web/learn.html", &body);
        let o = run(&t.root);
        assert_eq!(o.code, 1, "[{case}] {}", o.stdout);
        assert!(o.stdout.contains(needle), "[{case}] {}", o.stdout);
    }
}

#[test]
fn single_quoted_hub_css_is_accepted() {
    tick();
    let t = green_tree();
    let body = hub(&["alpha", "beta", "gamma"])
        .replace("href=\"assets/css/course.css\"", "href='assets/css/course.css'");
    t.write("web/learn.html", &body);
    let o = run(&t.root);
    assert_eq!(o.code, 0, "{}", o.stdout);
}

#[test]
fn the_hub_must_not_link_an_empty_ok_module() {
    tick();
    let t = Tree::new();
    t.note("alpha.md");
    t.write("web/learn/alpha.html", &page("alpha"));
    t.write(
        "web/content/modules/alpha.md",
        "# heading\n\nBody text long enough to clear the size floor.\n",
    );
    t.write(
        "knowledge/domains.toml",
        "[[domain]]\nid = \"alpha\"\norder = 1\nprimary_notes = \"../modules/alpha.md\"\n\n\
         [[domain]]\nid = \"omega\"\norder = 2\nprimary_notes = \"\"\nexam_weight_unknown = true\n",
    );
    t.write(
        "web/data/modules_index.json",
        &index_json(&[("alpha", false), ("omega", true)]),
    );
    t.write("web/learn.html", &hub(&["alpha", "omega"]));
    let o = run(&t.root);
    assert_eq!(o.code, 1);
    assert!(
        o.stdout.contains("hub must not link to empty-ok module page omega"),
        "{}",
        o.stdout
    );
}

#[test]
fn a_navigable_module_absent_from_the_hub_is_red() {
    tick();
    let t = green_tree();
    t.write("web/learn.html", &hub(&["alpha", "beta"]));
    let o = run(&t.root);
    assert_eq!(o.code, 1);
    assert!(
        o.stdout.contains("hub does not list navigable module gamma"),
        "{}",
        o.stdout
    );
}

#[test]
fn a_data_module_id_attribute_satisfies_the_hub_listing() {
    tick();
    let t = green_tree();
    let body = hub(&["alpha", "beta"]).replace(
        "</body>",
        "  <li data-module-id=\"gamma\"></li>\n</body>",
    );
    t.write("web/learn.html", &body);
    let o = run(&t.root);
    assert_eq!(o.code, 0, "{}", o.stdout);
}

// ── empty input ERROR (the hole the Python left open) ──────────────────────

/// bd-smoke-learn-vacuous-empty-ok-9d3n: the Python PASSED this tree.
/// EXTRACT-THEN-DELETE closes the hole. A scan that checked nothing is RED.
#[test]
fn an_all_empty_ok_registry_is_an_error_not_a_vacuous_pass() {
    tick();
    let t = Tree::new();
    t.write(
        "knowledge/domains.toml",
        "[[domain]]\nid = \"omega\"\norder = 1\nprimary_notes = \"\"\nexam_weight_unknown = true\n",
    );
    t.write("web/data/modules_index.json", &index_json(&[("omega", true)]));
    t.write("web/learn.html", &hub(&[]));
    let o = run(&t.root);
    assert_ne!(o.code, 0, "vacuous PASS is forbidden: {}", o.stdout);
    assert!(o.stdout.starts_with("FAIL: smoke_learn\n"), "{}", o.stdout);
    assert!(
        o.stdout.contains("zero primary_notes resolved"),
        "{}",
        o.stdout
    );
    assert!(
        o.stdout.contains("zero navigable modules"),
        "{}",
        o.stdout
    );
    assert!(
        !o.stdout.contains("PASS: smoke_learn"),
        "PASS must not appear on the empty-input path: {}",
        o.stdout
    );
}

#[test]
fn a_tree_with_no_learn_inputs_at_all_is_an_error() {
    tick();
    let t = Tree::new();
    // claims.toml only — no domains, no index, no hub
    let o = run(&t.root);
    assert_ne!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout.contains("knowledge/domains.toml missing"),
        "{}",
        o.stdout
    );
}

// ── anti-vacuous meta ──────────────────────────────────────────────────────

#[test]
fn this_suite_has_not_shrunk() {
    tick();
    let this = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/smoke_learn.rs"),
    )
    .expect("this test file");
    let cases = this.matches("#[test]").count();
    assert!(
        cases >= EXPECTED_CASES,
        "case count fell to {cases}; EXPECTED_CASES is {EXPECTED_CASES}. \
         A suite that quietly shrank reports exactly like one that passed."
    );
    let ran = RAN.load(Ordering::SeqCst);
    // This test itself has ticked; others may have run in parallel.
    assert!(
        ran >= 1,
        "this file's own tick must have fired"
    );
}
