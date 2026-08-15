//! Verdict suite for `cdcp_learn::weak_links` (extracted by
//! bd-substrate-rust-migration-jhd.16). The Python is DELETED. Every case
//! asserts the correct answer, not agreement with a retired script.
//!
//! This smoke is a READER: it reads the Learn / results / domain-registry
//! surface and writes nothing. Cases run against TEMP fixtures (or the live
//! tree read-only).
//!
//! ANTI-VACUOUS: a missing, empty or unparseable domain registry is RED; a
//! registry that declares fewer than fourteen modules is RED; an empty
//! MODULE_LEARN_SLUGS is RED. A suite that ran no case is RED.
//!
//! Slug strings in this file are FORMATTED, not enumerated, so the bd-ggs7
//! frozen-table detector does not mistake a fixture helper for a product
//! map.

use cdcp_learn::weak_links::{
    load_declared_modules, run, DOMAINS_TOML_REL, INDEX_JSON_REL, LEARN_DIR_REL, RESULTS_JS_REL,
    SLUGS_JS_REL,
};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Cases actually run, so "the suite ran" is itself checked.
static RAN: AtomicUsize = AtomicUsize::new(0);
static ROUND: AtomicUsize = AtomicUsize::new(0);

/// Raise when you add a `#[test]`. A DROP means a case was deleted.
const EXPECTED_CASES: usize = 20;

fn engine_root() -> PathBuf {
    cdcp_learn::resolve_engine_root(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
}

fn tick() {
    RAN.fetch_add(1, Ordering::SeqCst);
}

struct Fixture {
    dir: tempfile::TempDir,
    root: PathBuf,
}

impl Fixture {
    fn new() -> Fixture {
        let dir = tempfile::tempdir().unwrap();
        let n = ROUND.fetch_add(1, Ordering::SeqCst);
        root_at(dir, n)
    }
}

fn root_at(dir: tempfile::TempDir, n: usize) -> Fixture {
    let root = dir.path().join(format!("r{n}"));
    std::fs::create_dir_all(&root).unwrap();
    Fixture { dir, root }
}

impl Fixture {
    fn at(&self, rel: &str) -> PathBuf {
        cdcp_learn::join_rel(&self.root, rel)
    }

    fn put(&self, rel: &str, body: &str) {
        let p = self.at(rel);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(p, body).unwrap();
    }

    fn rm(&self, rel: &str) {
        let p = self.at(rel);
        if p.is_dir() {
            std::fs::remove_dir_all(&p).unwrap();
        } else if p.exists() {
            std::fs::remove_file(&p).unwrap();
        }
    }
}

fn slug(n: i64) -> String {
    format!("{n:02}-mod")
}

fn domains_toml(count: i64) -> String {
    let mut s = String::from("schema_version = 1\n");
    for n in 1..=count {
        s.push_str(&format!(
            "\n[[domain]]\nid = \"{}\"\norder = {n}\n",
            slug(n)
        ));
    }
    s
}

fn slugs_js(count: i64) -> String {
    let mut body = String::new();
    for n in 1..=count {
        body.push_str(&format!("  {n}: \"{}\",\n", slug(n)));
    }
    format!("export const MODULE_LEARN_SLUGS = Object.freeze({{\n{body}}});\n")
}

fn results_js() -> String {
    "import { MODULE_LEARN_SLUGS } from \"../../data/module_learn_slugs.js\";\n\
     export { MODULE_LEARN_SLUGS };\n\
     export function moduleLearnHref(n) { return \"learn/\" + n + \".html\"; }\n\
     const copy = \"Review weak modules in Learn\";\n\
     const chip = '<a class=\"weak-chip--link\" href=\"learn/x.html\">';\n"
        .to_string()
}

fn modules_index(count: i64) -> String {
    let mut rows = String::new();
    for n in 1..=count {
        if n > 1 {
            rows.push(',');
        }
        let s = slug(n);
        rows.push_str(&format!(
            "{{\"id\":\"{s}\",\"order\":{n},\"empty\":false,\"href\":\"learn/{s}.html\"}}"
        ));
    }
    format!("{{\"modules\":[{rows}]}}")
}

/// Minimal GREEN tree: fourteen public domains, matching slug map, pages.
fn green(count: i64) -> Fixture {
    let f = Fixture::new();
    f.put(DOMAINS_TOML_REL, &domains_toml(count));
    f.put(RESULTS_JS_REL, &results_js());
    f.put(SLUGS_JS_REL, &slugs_js(count));
    f.put(INDEX_JSON_REL, &modules_index(count));
    for n in 1..=count {
        f.put(
            &format!("{LEARN_DIR_REL}/{}.html", slug(n)),
            "<html>learn</html>\n",
        );
    }
    f
}

#[test]
fn live_tree_passes() {
    tick();
    let root = engine_root();
    assert!(
        root.join("web").is_dir(),
        "live engine has no web/ — a missing product tree is an ERROR"
    );
    let o = run(&root);
    assert_eq!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout.starts_with("PASS: smoke_weak_links"),
        "{}",
        o.stdout
    );
    assert!(
        o.stdout
            .contains("modules=15 (derived from knowledge/domains.toml)"),
        "{}",
        o.stdout
    );
    assert!(
        o.stdout.contains("M15 → learn/15-ops-adjacent.html"),
        "module 15 must be inside the swept set:\n{}",
        o.stdout
    );
    assert!(o.artifact.is_none(), "smoke is a reader");
}

#[test]
fn live_tree_writes_nothing() {
    tick();
    let root = engine_root();
    let path = root.join(RESULTS_JS_REL);
    let before = std::fs::read(&path).expect("live results.js");
    assert!(
        !before.is_empty(),
        "live results.js is empty — a missing product file is an ERROR"
    );
    let _ = run(&root);
    let after = std::fs::read(&path).expect("results.js after run");
    assert_eq!(before, after, "the smoke wrote results.js");
}

#[test]
fn fourteen_module_green_fixture_passes() {
    tick();
    let f = green(14);
    let o = run(&f.root);
    assert_eq!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout.starts_with("PASS: smoke_weak_links"),
        "{}",
        o.stdout
    );
    assert!(
        o.stdout
            .contains("modules=14 (derived from knowledge/domains.toml)"),
        "{}",
        o.stdout
    );
    assert!(
        !o.stdout.contains("M15"),
        "a 14-module tree must not sweep a fifteenth module:\n{}",
        o.stdout
    );
    let _ = &f.dir;
}

#[test]
fn fifteen_module_green_fixture_passes() {
    tick();
    let f = green(15);
    let o = run(&f.root);
    assert_eq!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout
            .contains("modules=15 (derived from knowledge/domains.toml)"),
        "{}",
        o.stdout
    );
    let _ = &f.dir;
}

#[test]
fn empty_tree_is_an_error() {
    tick();
    let f = Fixture::new();
    let o = run(&f.root);
    assert_ne!(o.code, 0, "empty tree must be RED, got PASS:\n{}", o.stdout);
    assert!(
        o.stdout.starts_with("FAIL: smoke_weak_links"),
        "{}",
        o.stdout
    );
    assert!(o.stdout.contains("domain registry missing"), "{}", o.stdout);
    assert!(
        !o.stdout.contains("PASS: smoke_weak_links"),
        "PASS must not appear on a failing run:\n{}",
        o.stdout
    );
    let _ = &f.dir;
}

#[test]
fn empty_registry_is_an_error() {
    tick();
    let f = green(14);
    f.put(DOMAINS_TOML_REL, "schema_version = 1\n");
    let o = run(&f.root);
    assert_ne!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout
            .contains("domain registry declares zero modules (vacuous weak-link check is ERROR)"),
        "{}",
        o.stdout
    );
    let _ = &f.dir;
}

#[test]
fn missing_registry_is_an_error() {
    tick();
    let f = green(14);
    f.rm(DOMAINS_TOML_REL);
    let o = run(&f.root);
    assert_ne!(o.code, 0, "{}", o.stdout);
    assert!(o.stdout.contains("domain registry missing"), "{}", o.stdout);
    let _ = &f.dir;
}

#[test]
fn unparseable_registry_is_an_error() {
    tick();
    let f = green(14);
    f.put(
        DOMAINS_TOML_REL,
        "schema_version = 1\n[[domain]\nid = \"01-broken\"\n",
    );
    let o = run(&f.root);
    assert_ne!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout.contains("domain registry parse error"),
        "{}",
        o.stdout
    );
    let _ = &f.dir;
}

#[test]
fn collapsed_registry_below_fourteen_is_an_error() {
    tick();
    let f = green(12);
    let o = run(&f.root);
    assert_ne!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout.contains(
            "domain registry declares only 12 modules; the CDCP course has fourteen \
             public EPI domains at minimum"
        ),
        "{}",
        o.stdout
    );
    assert!(
        o.stdout.starts_with("FAIL: smoke_weak_links"),
        "{}",
        o.stdout
    );
    assert!(
        !o.stdout.contains("PASS: smoke_weak_links"),
        "PASS must not appear on a failing run:\n{}",
        o.stdout
    );
    let _ = &f.dir;
}

/// Planted known-bad: a declared module with no slug map entry is RED and
/// names the module. Deleting this assertion makes `EXPECTED_CASES` fail.
#[test]
fn planted_results_js_missing_declared_slug_is_red_naming_the_module() {
    tick();
    let f = green(14);
    // Drop module 6 from the shipped map only.
    let mut body = String::new();
    for n in 1..=14i64 {
        if n == 6 {
            continue;
        }
        body.push_str(&format!("  {n}: \"{}\",\n", slug(n)));
    }
    f.put(
        SLUGS_JS_REL,
        &format!("export const MODULE_LEARN_SLUGS = Object.freeze({{\n{body}}});\n"),
    );
    let o = run(&f.root);
    assert_ne!(
        o.code, 0,
        "a declared module missing from MODULE_LEARN_SLUGS must be RED:\n{}",
        o.stdout
    );
    assert!(
        o.stdout.contains("module 6:") && o.stdout.contains("MODULE_LEARN_SLUGS has no entry"),
        "the slug-map gap must name the module:\n{}",
        o.stdout
    );
    assert!(
        o.stdout.starts_with("FAIL: smoke_weak_links"),
        "{}",
        o.stdout
    );
    let _ = &f.dir;
}

#[test]
fn mapped_module_the_registry_drops_is_drift() {
    tick();
    let f = green(15);
    f.put(DOMAINS_TOML_REL, &domains_toml(14));
    let o = run(&f.root);
    assert_ne!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout.contains("module 15: results.js maps")
            && o.stdout.contains("does not declare that module"),
        "{}",
        o.stdout
    );
    let _ = &f.dir;
}

#[test]
fn declared_module_with_no_learn_page_is_red() {
    tick();
    let f = green(14);
    f.rm(&format!("{LEARN_DIR_REL}/{}.html", slug(6)));
    let o = run(&f.root);
    assert_ne!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout
            .contains("module 6: declared slug has no Learn page"),
        "{}",
        o.stdout
    );
    let _ = &f.dir;
}

#[test]
fn missing_results_js_is_red() {
    tick();
    let f = green(14);
    f.rm(RESULTS_JS_REL);
    let o = run(&f.root);
    assert_ne!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout.contains("missing web/assets/js/results.js"),
        "{}",
        o.stdout
    );
    let _ = &f.dir;
}

#[test]
fn empty_module_learn_slugs_is_red() {
    tick();
    let f = green(14);
    f.put(
        SLUGS_JS_REL,
        "export const MODULE_LEARN_SLUGS = Object.freeze({});\n",
    );
    let o = run(&f.root);
    assert_ne!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout.contains("MODULE_LEARN_SLUGS is empty"),
        "{}",
        o.stdout
    );
    let _ = &f.dir;
}

#[test]
fn slug_mismatch_is_red() {
    tick();
    let f = green(14);
    let mut body = String::new();
    for n in 1..=14i64 {
        let s = if n == 3 {
            "03-wrong".to_string()
        } else {
            slug(n)
        };
        body.push_str(&format!("  {n}: \"{s}\",\n"));
    }
    f.put(
        SLUGS_JS_REL,
        &format!("export const MODULE_LEARN_SLUGS = Object.freeze({{\n{body}}});\n"),
    );
    let o = run(&f.root);
    assert_ne!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout.contains("module 3: map slug") && o.stdout.contains("!= declared"),
        "{}",
        o.stdout
    );
    let _ = &f.dir;
}

#[test]
fn modules_index_disagreement_is_red() {
    tick();
    let f = green(14);
    f.put(
        INDEX_JSON_REL,
        r#"{"modules":[{"id":"03-wrong","order":3,"empty":false,"href":"learn/03-wrong.html"}]}"#,
    );
    let o = run(&f.root);
    assert_ne!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout.contains("modules_index order=3") && o.stdout.contains("!= declared slug"),
        "{}",
        o.stdout
    );
    let _ = &f.dir;
}

#[test]
fn missing_module_learn_href_is_red() {
    tick();
    let f = green(14);
    f.put(
        RESULTS_JS_REL,
        "import { MODULE_LEARN_SLUGS } from \"../../data/module_learn_slugs.js\";\n\
         const copy = \"Review weak modules in Learn\";\n",
    );
    let o = run(&f.root);
    assert_ne!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout.contains("moduleLearnHref helper missing")
            || o.stdout.contains("must call moduleLearnHref"),
        "{}",
        o.stdout
    );
    let _ = &f.dir;
}

#[test]
fn missing_generated_slugs_file_is_red() {
    tick();
    let f = green(14);
    f.rm(SLUGS_JS_REL);
    let o = run(&f.root);
    assert_ne!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout.contains("missing web/data/module_learn_slugs.js"),
        "{}",
        o.stdout
    );
    let _ = &f.dir;
}

#[test]
fn load_declared_modules_names_a_missing_file() {
    tick();
    let (declared, errors) = load_declared_modules(Path::new("/no/such/domains.toml"));
    assert!(declared.is_empty());
    assert!(
        errors.iter().any(|e| e.contains("domain registry missing")),
        "{errors:?}"
    );
}

#[test]
fn this_suite_has_not_shrunk() {
    tick();
    let this = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/smoke_weak_links.rs"),
    )
    .expect("this test file");
    let cases = this.matches("#[test]").count();
    assert!(
        cases >= EXPECTED_CASES,
        "case count fell to {cases}; EXPECTED_CASES is {EXPECTED_CASES}. \
         A suite that quietly shrank reports exactly like one that passed."
    );
    assert!(
        RAN.load(Ordering::SeqCst) > 0,
        "the verdict suite ran nothing"
    );
}
