//! EXTRACT-THEN-DELETE: corpus-rights checker (rights-policy.toml CORPUS-R*).
//!
//! Known-bad CORPUS-R7 / CORPUS-R8 must TRIP. Known-good public-domain
//! body-retained must PASS. The checker reads metadata + tree names only;
//! it never opens a capture body.

use cdcp_data::{
    check_corpus_rights, parse_json, records_from_manifest, RightsError, CR_R7, CR_R8,
    MANIFEST_PATH, NEVER_OPENS_CAPTURE_BODIES,
};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static SEQ: AtomicU64 = AtomicU64::new(1);

fn engine() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("engine")
}

fn scratch(tag: &str) -> PathBuf {
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!(
        "cdcp-corpus-rights-{}-{}-{}-{}",
        tag,
        std::process::id(),
        n,
        nanos
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("scratch");
    dir
}

const POLICY: &str = r#"
schema = "cdcp.corpus.rights-policy.v1"

[vocabulary]
capture = ["body-retained", "citation-only", "not-vendored"]
rights = [
  "public-domain", "open-licence", "own-work-this-repo",
  "publisher-retains-copyright", "publisher-copyright",
  "mixed-us-government-work-with-third-party-material", "unknown",
]
redistribution = ["permitted", "not-licensed", "NOT-licensed", "unknown"]
ai_ingestion = ["permitted", "PROHIBITED", "unknown"]
self_evidencing_rights = ["public-domain", "own-work-this-repo"]

[published_tree]
roots = [
  "course-engine/knowledge/corpus/public",
  "course-engine/knowledge/corpus/free-pdfs",
]
"#;

const CITATION_ROW: &str = r#"{
      "id": "src-cited",
      "capture": "citation-only",
      "rights": "publisher-retains-copyright",
      "redistribution": "not-licensed",
      "ai_ingestion": "unknown"
    }"#;

const OWN_WORK_ROW: &str = r#"{
      "id": "src-own",
      "capture": "body-retained",
      "rights": "own-work-this-repo",
      "redistribution": "permitted",
      "ai_ingestion": "permitted",
      "path": "knowledge/corpus/public/src-own.txt"
    }"#;

const SIDECAR: &str = r#"
source_id = "src-pdf"
rights = "public-domain"
redistribution = "permitted"
ai_ingestion = "permitted"
capture = "body-retained"
path = "knowledge/corpus/free-pdfs/gov.pdf"
"#;

struct Repo {
    root: PathBuf,
}

impl Repo {
    fn new() -> Self {
        let root = scratch("repo");
        let r = Repo { root };
        r.write("registries/claims.toml", "schema_version = 1\n");
        r.write("knowledge/corpus/rights-policy.toml", POLICY);
        r.set_sources(&[CITATION_ROW, OWN_WORK_ROW]);
        r.write("knowledge/corpus/free-pdfs/gov.meta.toml", SIDECAR);
        r.write("knowledge/corpus/public/src-own.txt", "local objectives\n");
        r.write("knowledge/corpus/free-pdfs/gov.pdf", "%PDF-1.4 fixture\n");
        r
    }

    fn path(&self, rel: &str) -> PathBuf {
        self.root.join(rel)
    }

    fn write(&self, rel: &str, body: &str) {
        let p = self.path(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, body).unwrap();
    }

    fn remove(&self, rel: &str) {
        fs::remove_file(self.path(rel)).unwrap();
    }

    fn set_sources(&self, rows: &[&str]) {
        self.write(
            "knowledge/corpus/public/manifest.json",
            &format!(
                "{{\n  \"schema\": \"cdcp.corpus.manifest.v2\",\n  \"sources\": [\n    {}\n  ]\n}}\n",
                rows.join(",\n    ")
            ),
        );
    }

    fn check(&self) -> Result<String, String> {
        match check_corpus_rights(&self.root) {
            Ok(rep) => Ok(rep.to_string()),
            Err(e) => Err(e.to_string()),
        }
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

// ── known-GOOD ─────────────────────────────────────────────────────────────

#[test]
fn known_good_the_baseline_tree_passes() {
    let r = Repo::new();
    let out = r.check().expect("a clean corpus must pass");
    assert!(out.contains("records=3"), "{out}");
    assert!(out.contains("bodies_retained=2"), "{out}");
}

#[test]
fn known_good_a_recorded_licence_justification_keeps_the_body() {
    let r = Repo::new();
    r.set_sources(&[
        CITATION_ROW,
        OWN_WORK_ROW,
        r#"{
      "id": "src-open",
      "capture": "body-retained",
      "rights": "open-licence",
      "redistribution": "permitted",
      "ai_ingestion": "permitted",
      "path": "knowledge/corpus/public/src-open.txt",
      "redistribution_evidence": {
        "licence": "CC-BY-4.0",
        "url": "https://creativecommons.org/licenses/by/4.0/legalcode",
        "clause": "Section 2(a)(1)(A)"
      }
    }"#,
    ]);
    r.write("knowledge/corpus/public/src-open.txt", "openly licensed\n");
    let out = r
        .check()
        .expect("a licence justification is the sanctioned way to keep a body");
    assert!(out.contains("bodies_retained=3"), "{out}");
}

#[test]
fn known_good_a_public_domain_pdf_needs_no_licence_citation() {
    let r = Repo::new();
    r.set_sources(&[CITATION_ROW]);
    r.remove("knowledge/corpus/public/src-own.txt");
    r.check().expect("public-domain body-retained must PASS");
}

// ── known-bad ──────────────────────────────────────────────────────────────

#[test]
fn known_bad_not_licensed_body_in_the_published_tree_is_red() {
    let r = Repo::new();
    r.set_sources(&[
        CITATION_ROW,
        OWN_WORK_ROW,
        r#"{
      "id": "src-fixture-bad",
      "capture": "body-retained",
      "rights": "publisher-retains-copyright",
      "redistribution": "not-licensed",
      "ai_ingestion": "unknown",
      "path": "knowledge/corpus/public/src-fixture-bad.txt"
    }"#,
    ]);
    r.write(
        "knowledge/corpus/public/src-fixture-bad.txt",
        "publisher prose stand-in\n",
    );
    let err = r.check().expect_err("planted unlicensed body must be RED");
    assert!(err.contains("CR-R1"), "names the invariant: {err}");
    assert!(err.contains("src-fixture-bad"), "names the record: {err}");
    assert!(
        err.contains("knowledge/corpus/public/src-fixture-bad.txt"),
        "names the FILE: {err}"
    );
    assert!(err.contains("redistribution"), "names the FIELD: {err}");
}

#[test]
fn known_bad_the_legacy_spelling_is_the_same_violation() {
    let r = Repo::new();
    r.set_sources(&[
        OWN_WORK_ROW,
        r#"{
      "id": "src-legacy",
      "capture": "body-retained",
      "rights": "publisher-copyright",
      "redistribution": "NOT-licensed",
      "ai_ingestion": "unknown",
      "path": "knowledge/corpus/public/src-legacy.txt"
    }"#,
    ]);
    r.write("knowledge/corpus/public/src-legacy.txt", "x\n");
    let err = r.check().expect_err("legacy spelling must RED");
    assert!(err.contains("CR-R1") && err.contains("src-legacy"), "{err}");
}

#[test]
fn known_bad_missing_redistribution_is_an_error_never_a_pass() {
    let r = Repo::new();
    r.set_sources(&[
        OWN_WORK_ROW,
        r#"{
      "id": "src-fixture-norights",
      "capture": "citation-only",
      "rights": "publisher-retains-copyright"
    }"#,
    ]);
    let err = r
        .check()
        .expect_err("absence must never read as permission");
    assert!(err.contains("CR-R2"), "{err}");
    assert!(err.contains("src-fixture-norights"), "{err}");
    assert!(err.contains("redistribution"), "names the FIELD: {err}");
}

#[test]
fn known_bad_an_invented_redistribution_value_is_not_permission() {
    let r = Repo::new();
    r.set_sources(&[
        OWN_WORK_ROW,
        r#"{
      "id": "src-invented",
      "capture": "body-retained",
      "rights": "publisher-retains-copyright",
      "redistribution": "probably-fine",
      "path": "knowledge/corpus/public/src-invented.txt"
    }"#,
    ]);
    r.write("knowledge/corpus/public/src-invented.txt", "x\n");
    let err = r.check().expect_err("invented value must ERROR");
    assert!(err.contains("probably-fine"), "quotes the bad value: {err}");
}

#[test]
fn known_bad_ai_prohibited_body_is_red() {
    let r = Repo::new();
    r.set_sources(&[
        OWN_WORK_ROW,
        r#"{
      "id": "src-ai-prohibited",
      "capture": "body-retained",
      "rights": "publisher-retains-copyright",
      "redistribution": "permitted",
      "ai_ingestion": "PROHIBITED",
      "path": "knowledge/corpus/public/src-ai-prohibited.txt",
      "redistribution_evidence": {"licence": "L", "url": "U", "clause": "C"}
    }"#,
    ]);
    r.write("knowledge/corpus/public/src-ai-prohibited.txt", "x\n");
    let err = r.check().expect_err("AI-prohibited body must RED");
    assert!(err.contains("CR-R6"), "{err}");
    assert!(err.contains("src-ai-prohibited"), "{err}");
}

#[test]
fn known_bad_corpus_r7_bare_permitted_over_publisher_copyright_is_red() {
    let r = Repo::new();
    r.set_sources(&[
        OWN_WORK_ROW,
        r#"{
      "id": "src-asserted",
      "capture": "body-retained",
      "rights": "publisher-retains-copyright",
      "redistribution": "permitted",
      "ai_ingestion": "permitted",
      "path": "knowledge/corpus/public/src-asserted.txt"
    }"#,
    ]);
    r.write("knowledge/corpus/public/src-asserted.txt", "x\n");
    let err = r
        .check()
        .expect_err("typing \"permitted\" is not a licence");
    assert!(err.contains(CR_R7), "{err}");
    assert!(err.contains("src-asserted"), "{err}");
}

#[test]
fn known_bad_an_incomplete_licence_citation_does_not_count() {
    let r = Repo::new();
    r.set_sources(&[
        OWN_WORK_ROW,
        r#"{
      "id": "src-halfcited",
      "capture": "body-retained",
      "rights": "open-licence",
      "redistribution": "permitted",
      "path": "knowledge/corpus/public/src-halfcited.txt",
      "redistribution_evidence": {"licence": "CC-BY-4.0", "url": "", "clause": ""}
    }"#,
    ]);
    r.write("knowledge/corpus/public/src-halfcited.txt", "x\n");
    let err = r.check().expect_err("incomplete citation must RED");
    assert!(err.contains(CR_R7), "{err}");
}

#[test]
fn known_bad_a_bare_exemption_is_a_schema_error() {
    let r = Repo::new();
    r.set_sources(&[
        OWN_WORK_ROW,
        r#"{
      "id": "src-exempt",
      "capture": "citation-only",
      "rights": "publisher-retains-copyright",
      "redistribution": "not-licensed",
      "rights_review": "OPEN",
      "rights_review_reason": ""
    }"#,
    ]);
    let err = r.check().expect_err("bare exemption must ERROR");
    assert!(err.contains("CR-R4") && err.contains("src-exempt"), "{err}");
}

#[test]
fn known_bad_corpus_r8_a_file_with_no_metadata_at_all_is_an_error() {
    let r = Repo::new();
    r.write(
        "knowledge/corpus/public/src-stowaway.txt",
        "arrived with no record\n",
    );
    let err = r
        .check()
        .expect_err("an unregistered capture must not pass");
    assert!(err.contains(CR_R8), "{err}");
    assert!(
        err.contains("knowledge/corpus/public/src-stowaway.txt"),
        "names the FILE: {err}"
    );
}

#[test]
fn known_bad_a_declared_body_that_is_not_there_is_red() {
    let r = Repo::new();
    r.remove("knowledge/corpus/public/src-own.txt");
    let err = r.check().expect_err("missing declared body must RED");
    assert!(err.contains("disagree"), "{err}");
}

#[test]
fn known_bad_dropping_a_published_root_is_an_error_not_a_smaller_scan() {
    let r = Repo::new();
    r.write(
        "knowledge/corpus/rights-policy.toml",
        &POLICY.replace("  \"course-engine/knowledge/corpus/free-pdfs\",\n", ""),
    );
    let err = r.check().expect_err("dropped root must ERROR");
    assert!(err.contains("free-pdfs"), "{err}");
}

#[test]
fn known_bad_widening_self_evidencing_rights_is_an_error() {
    let r = Repo::new();
    r.write(
        "knowledge/corpus/rights-policy.toml",
        &POLICY.replace(
            r#"self_evidencing_rights = ["public-domain", "own-work-this-repo"]"#,
            r#"self_evidencing_rights = ["public-domain", "own-work-this-repo", "publisher-retains-copyright"]"#,
        ),
    );
    let err = r.check().expect_err("the policy may narrow, never widen");
    assert!(err.contains("publisher-retains-copyright"), "{err}");
}

#[test]
fn known_bad_a_stale_open_violation_entry_is_an_error() {
    let r = Repo::new();
    r.write(
        "knowledge/corpus/rights-policy.toml",
        &format!(
            "{POLICY}\n[[open_violation]]\ninvariant = \"CORPUS-R1\"\nbead = \"bd-ghost\"\nrecords = [\"src-vanished\"]\n"
        ),
    );
    let err = r.check().expect_err("stale amnesty must ERROR");
    assert!(
        err.contains("src-vanished") && err.contains("rotted"),
        "{err}"
    );
}

// ── anti-vacuous ───────────────────────────────────────────────────────────

#[test]
fn anti_vacuous_zero_sources_is_an_error() {
    let r = Repo::new();
    r.set_sources(&[]);
    let err = r.check().expect_err("empty manifest must not pass");
    assert!(err.contains("zero sources"), "{err}");
}

#[test]
fn anti_vacuous_zero_sidecars_is_an_error() {
    let r = Repo::new();
    r.remove("knowledge/corpus/free-pdfs/gov.meta.toml");
    r.remove("knowledge/corpus/free-pdfs/gov.pdf");
    let err = r.check().expect_err("zero sidecars must ERROR");
    assert!(err.contains("zero"), "{err}");
}

#[test]
fn anti_vacuous_a_missing_manifest_is_an_error() {
    let r = Repo::new();
    r.remove("knowledge/corpus/public/manifest.json");
    let err = r.check().expect_err("missing manifest must ERROR");
    assert!(err.contains("manifest.json"), "{err}");
}

#[test]
fn anti_vacuous_a_truncated_manifest_is_an_error_not_a_short_one() {
    let r = Repo::new();
    r.write(
        "knowledge/corpus/public/manifest.json",
        "{\"sources\": [{\"id\": \"a\"",
    );
    r.check().expect_err("truncated manifest must ERROR");
}

#[test]
fn anti_vacuous_a_missing_published_root_is_an_error() {
    let r = Repo::new();
    fs::remove_dir_all(r.path("knowledge/corpus/free-pdfs")).unwrap();
    r.check()
        .expect_err("a root that is not there is not an empty one");
}

#[test]
fn anti_vacuous_a_missing_policy_is_an_error() {
    let r = Repo::new();
    r.remove("knowledge/corpus/rights-policy.toml");
    let err = r.check().expect_err("missing policy must ERROR");
    assert!(err.contains("rights-policy.toml"), "{err}");
}

// ── the live tree ──────────────────────────────────────────────────────────

const REMOVED_UNDER_THIS_BEAD: &[&str] = &[
    "src-epi-cdcp-page",
    "src-exin-cdcp-page",
    "src-nh-cdcp",
    "src-tuv-22237",
    "src-en-50600-overview",
    "src-tia-942",
    "src-tia-942-c-fotc",
    "src-uptime-tiers",
    "src-ocp-ready",
];

#[test]
fn the_live_corpus_passes() {
    let root = engine();
    let report = check_corpus_rights(&root).unwrap_or_else(|e| {
        panic!("the shipped corpus must be clean:\n{e}");
    });
    assert!(report.is_clean(), "{report}");
    assert!(
        report.to_string().starts_with("corpus-rights: ok:"),
        "{report}"
    );
}

#[test]
fn the_nine_removed_captures_have_no_body_in_the_tree() {
    let root = engine();
    let text = fs::read_to_string(root.join(MANIFEST_PATH)).expect("manifest");
    let doc = parse_json(&text).expect("manifest parses");
    let recs = records_from_manifest(&doc, MANIFEST_PATH).expect("records");
    assert!(
        recs.len() >= REMOVED_UNDER_THIS_BEAD.len(),
        "a vacuous manifest scan is an ERROR"
    );
    for id in REMOVED_UNDER_THIS_BEAD {
        let r = recs
            .iter()
            .find(|r| r.id == *id)
            .unwrap_or_else(|| panic!("{id} must remain as a citation row, not vanish"));
        assert!(
            !r.is_body_retained(),
            "{id} is body-retained again — the removal was undone"
        );
        assert!(r.path.is_none(), "{id} declares a path again: {:?}", r.path);
        assert!(
            !root
                .join(format!("knowledge/corpus/public/{id}.txt"))
                .exists(),
            "{id}.txt is back in the published tree"
        );
    }
}

#[test]
fn every_removal_is_recorded_with_a_prior_digest_and_a_bead() {
    let root = engine();
    let text = fs::read_to_string(root.join(MANIFEST_PATH)).expect("manifest");
    let doc = parse_json(&text).expect("parses");
    let Some(cdcp_data::Json::Arr(sources)) = doc.get("sources") else {
        panic!("sources array");
    };
    let mut checked = 0usize;
    for s in sources {
        let Some(id) = s.get("id").and_then(|v| v.as_str()) else {
            continue;
        };
        if !REMOVED_UNDER_THIS_BEAD.contains(&id) {
            continue;
        }
        for key in [
            "body_removed_at",
            "body_removed_bead",
            "body_removed_sha256_of_prior_capture",
        ] {
            let v = s.get(key).and_then(|v| v.as_str()).unwrap_or("");
            assert!(!v.trim().is_empty(), "{id}: missing {key}");
        }
        checked += 1;
    }
    assert_eq!(
        checked,
        REMOVED_UNDER_THIS_BEAD.len(),
        "a vacuous provenance scan is an ERROR"
    );
}

#[test]
fn production_module_never_opens_capture_bodies() {
    let src = include_str!("../src/corpus_rights.rs");
    assert!(src.contains(NEVER_OPENS_CAPTURE_BODIES));
    assert!(src.contains(CR_R7));
    assert!(src.contains(CR_R8));
    let prod = src.split("#[cfg(test)]").next().expect("prod");
    assert!(
        !prod.contains("std::fs::read("),
        "production must not open capture bytes"
    );
    assert!(!prod.contains("File::open"));
}

#[test]
fn check_on_a_non_tree_is_an_error_not_a_pass() {
    let dir = scratch("empty");
    let err = check_corpus_rights(&dir).expect_err("empty root must ERROR");
    match err {
        RightsError::Error(s) => assert!(s.contains("rights-policy.toml"), "{s}"),
        RightsError::Violation(v) => panic!("empty root is an ERROR, not a violation: {v:?}"),
    }
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn engine_helper_is_the_live_root() {
    assert!(engine().join(MANIFEST_PATH).is_file());
}
