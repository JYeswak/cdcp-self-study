//! Licence three-field split: four rules, known-bad plants, meta-tests.
//!
//! In-tree fixtures, no git-apply patches. Deleting a check those tokens
//! sit inside makes the matching selftest non-zero.

use cdcp_evidence::{
    audit_index, build_agent_reachable_index, evaluate_artifact, may_load, parse_meta_toml,
    resolve_engine_root, scan_engine, AiIngestion, CorpusIndex, LicenceFault, ANTI_VACUOUS,
    R1_PUBLISHED_UNLICENSED, R2_MISSING_RIGHTS, R3_THIRD_PARTY_PUBLIC_DOMAIN, R4_PROHIBITED_INDEX,
};
use std::path::{Path, PathBuf};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/licence")
}

fn scan_fix(name: &str) -> cdcp_evidence::LicenceReport {
    scan_engine(&fixtures().join(name))
}

fn production_licence_src() -> &'static str {
    include_str!("../src/licence.rs")
}

fn fn_body<'a>(src: &'a str, name: &str) -> &'a str {
    let needle = format!("fn {name}");
    let after = src
        .split(&needle)
        .nth(1)
        .unwrap_or_else(|| panic!("{name} is missing from licence.rs"));
    after
}

#[test]
fn tokens_are_the_acceptance_sentences() {
    assert!(R1_PUBLISHED_UNLICENSED.contains("redistribution != permitted"));
    assert!(R2_MISSING_RIGHTS.contains("never default-permissive"));
    assert!(R3_THIRD_PARTY_PUBLIC_DOMAIN.contains("third_party_figures"));
    assert!(R3_THIRD_PARTY_PUBLIC_DOMAIN.contains("redistribution=public-domain"));
    assert!(R4_PROHIBITED_INDEX.contains("PROHIBITED"));
    assert!(ANTI_VACUOUS.contains("zero artifacts"));
}

#[test]
fn known_good_fixture_is_clean() {
    let report = scan_fix("good");
    assert!(
        report.is_clean(),
        "known-good must stay green so the suite is not attack-only:\n{report}"
    );
    assert!(report.scanned >= 3, "scanned={}", report.scanned);
}

#[test]
fn known_bad_r1_unlicensed_body_under_published_path_is_red_and_names_the_file() {
    let report = scan_fix("r1_unlicensed_body");
    assert!(!report.is_clean(), "R1 plant must be RED:\n{report}");
    let named = report.faults.iter().any(|f| match f {
        LicenceFault::PublishedUnlicensed {
            path,
            redistribution,
        } => path.contains("leak.pdf") && redistribution == "not-licensed",
        _ => false,
    });
    assert!(named, "R1 must name the published file:\n{report}");
    let text = report.to_string();
    assert!(
        text.contains("leak.pdf"),
        "finding text must name the file:\n{text}"
    );
    assert!(text.contains(R1_PUBLISHED_UNLICENSED));
}

#[test]
fn known_bad_r2_missing_rights_is_error_never_default_permissive() {
    let report = scan_fix("r2_missing_rights");
    assert!(!report.is_clean(), "R2 plant must be RED:\n{report}");
    assert!(
        report.faults.iter().any(|f| matches!(
            f,
            LicenceFault::MissingRights {
                field: "rights",
                ..
            }
        )),
        "missing rights must be a fault, not permitted-by-default:\n{report}"
    );
    assert!(report.to_string().contains(R2_MISSING_RIGHTS));

    let sidecar =
        fixtures().join("r2_missing_rights/knowledge/corpus/free-pdfs/norights.meta.toml");
    let text = std::fs::read_to_string(&sidecar).unwrap();
    let meta = parse_meta_toml(&text, "norights.meta.toml").unwrap();
    assert!(!meta.has_licence_or_rights());
    assert!(
        may_load(&meta).is_err(),
        "E1 may_load must refuse a sidecar with no licence line"
    );
}

#[test]
fn known_bad_r3_third_party_figures_may_not_be_public_domain() {
    let report = scan_fix("r3_third_party_pd");
    assert!(!report.is_clean(), "R3 plant must be RED:\n{report}");
    assert!(
        report.faults.iter().any(|f| matches!(
            f,
            LicenceFault::ThirdPartyPublicDomain { marked, .. }
                if marked.contains("public-domain")
        )),
        "third_party_figures + redistribution=public-domain must be RED:\n{report}"
    );
    assert!(report.to_string().contains(R3_THIRD_PARTY_PUBLIC_DOMAIN));
}

#[test]
fn known_bad_r4_prohibited_id_in_agent_index_is_red() {
    let report = scan_fix("r4_prohibited_index");
    assert!(!report.is_clean(), "R4 plant must be RED:\n{report}");
    assert!(
        report.faults.iter().any(|f| matches!(
            f,
            LicenceFault::ProhibitedInAgentIndex { id, index }
                if id == "src-r4-ashrae" && index.contains("agent-index.toml")
        )),
        "PROHIBITED id in an agent index must be RED:\n{report}"
    );
    assert!(report.to_string().contains(R4_PROHIBITED_INDEX));

    let sidecar =
        fixtures().join("r4_prohibited_index/knowledge/corpus/free-pdfs/ashrae.meta.toml");
    let text = std::fs::read_to_string(&sidecar).unwrap();
    let meta = parse_meta_toml(&text, "ashrae.meta.toml").unwrap();
    assert_eq!(meta.ai_ingestion(), Some(AiIngestion::Prohibited));
    assert!(!meta.eligible_for_agent_index());
    let built = build_agent_reachable_index(std::slice::from_ref(&meta));
    assert!(
        !built.contains("src-r4-ashrae"),
        "the product index builder must exclude PROHIBITED"
    );
}

#[test]
fn anti_vacuous_zero_artifacts_scanned_is_error() {
    let report = scan_fix("empty");
    assert!(!report.is_clean(), "empty scan must be RED:\n{report}");
    assert_eq!(report.scanned, 0);
    assert!(
        report
            .faults
            .iter()
            .any(|f| matches!(f, LicenceFault::VacuousScan)),
        "zero artifacts scanned is an ERROR, not a pass:\n{report}"
    );
    assert!(report.to_string().contains(ANTI_VACUOUS));
}

#[test]
fn live_published_tree_passes() {
    let engine = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let engine = engine.canonicalize().expect("engine root");
    let report = scan_engine(&engine);
    assert!(
        report.is_clean(),
        "live published corpus must satisfy the four rules:\n{report}"
    );
    assert!(
        report.scanned >= 5,
        "live tree has five free-pdfs sidecars; scanned={}",
        report.scanned
    );
}

#[test]
fn r3_also_trips_on_rights_public_domain_and_permitted() {
    let pd = parse_meta_toml(
        r#"
source_id = "src-plant-rights-pd"
rights = "public-domain"
redistribution = "not-licensed"
ai_ingestion = "unknown"
third_party_figures = ["Table 3-1"]
"#,
        "plant-rights-pd.meta.toml",
    )
    .unwrap();
    let faults = evaluate_artifact(&pd, None);
    assert!(
        faults.iter().any(|f| matches!(
            f,
            LicenceFault::ThirdPartyPublicDomain { marked, .. } if marked.contains("rights=public-domain")
        )),
        "{faults:?}"
    );

    let permitted = parse_meta_toml(
        r#"
source_id = "src-plant-permitted"
rights = "mixed-us-government-work-with-third-party-material"
redistribution = "permitted"
ai_ingestion = "permitted"
third_party_figures = ["Table 3-1"]
"#,
        "plant-permitted.meta.toml",
    )
    .unwrap();
    let faults = evaluate_artifact(&permitted, None);
    assert!(
        faults.iter().any(|f| matches!(
            f,
            LicenceFault::ThirdPartyPublicDomain { marked, .. } if marked.contains("redistribution=permitted")
        )),
        "{faults:?}"
    );
}

#[test]
fn planted_index_is_red_even_when_builder_excludes() {
    let meta = parse_meta_toml(
        r#"
source_id = "src-ashrae"
rights = "publisher-copyright"
redistribution = "NOT-licensed"
ai_ingestion = "PROHIBITED"
"#,
        "ashrae.meta.toml",
    )
    .unwrap();
    let built = build_agent_reachable_index(std::slice::from_ref(&meta));
    assert!(audit_index(&built, std::slice::from_ref(&meta), "built").is_empty());
    let planted = CorpusIndex::from_ids(["src-ashrae"]);
    let faults = audit_index(&planted, &[meta], "planted-index");
    assert!(
        faults.iter().any(|f| matches!(
            f,
            LicenceFault::ProhibitedInAgentIndex { id, index }
                if id == "src-ashrae" && index == "planted-index"
        )),
        "{faults:?}"
    );
}

#[test]
fn missing_redistribution_or_ai_ingestion_is_error() {
    let no_redist = parse_meta_toml(
        r#"
source_id = "src-x"
rights = "public-domain"
ai_ingestion = "permitted"
"#,
        "x.meta.toml",
    )
    .unwrap();
    let faults = evaluate_artifact(&no_redist, None);
    assert!(faults.iter().any(|f| matches!(
        f,
        LicenceFault::MissingRights {
            field: "redistribution",
            ..
        }
    )));

    let no_ai = parse_meta_toml(
        r#"
source_id = "src-y"
rights = "public-domain"
redistribution = "permitted"
"#,
        "y.meta.toml",
    )
    .unwrap();
    let faults = evaluate_artifact(&no_ai, None);
    assert!(faults.iter().any(|f| matches!(
        f,
        LicenceFault::MissingRights {
            field: "ai_ingestion",
            ..
        }
    )));
}

#[test]
fn citation_only_not_licensed_sidecar_is_not_r1() {
    let meta = parse_meta_toml(
        r#"
source_id = "src-cite"
rights = "publisher-copyright"
redistribution = "not-licensed"
ai_ingestion = "PROHIBITED"
capture = "not-vendored"
"#,
        "cite.meta.toml",
    )
    .unwrap();
    let faults = evaluate_artifact(&meta, None);
    assert!(
        !faults
            .iter()
            .any(|f| matches!(f, LicenceFault::PublishedUnlicensed { .. })),
        "a sidecar with no published body is not R1: {faults:?}"
    );
}

#[test]
fn resolve_engine_root_finds_claims_toml() {
    let start = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = resolve_engine_root(&start).expect("engine root");
    assert!(root.join("registries/claims.toml").is_file());
    assert!(root.join("knowledge/corpus/rights-policy.toml").is_file());
}

/// Meta-test: delete the R1 published-unlicensed check → this selftest is non-zero.
#[test]
fn selftest_delete_r1_is_nonzero() {
    let body = fn_body(production_licence_src(), "evaluate_artifact");
    assert!(
        body.contains("R1_PUBLISHED_UNLICENSED"),
        "delete the R1 token interpolation → selftest non-zero"
    );
    assert!(
        body.contains("PublishedUnlicensed"),
        "delete the R1 fault → selftest non-zero"
    );
}

/// Meta-test: delete the R2 missing-rights check → this selftest is non-zero.
#[test]
fn selftest_delete_r2_is_nonzero() {
    let body = fn_body(production_licence_src(), "evaluate_artifact");
    assert!(
        body.contains("R2_MISSING_RIGHTS"),
        "delete the R2 token interpolation → selftest non-zero"
    );
    assert!(
        body.contains("has_licence_or_rights") && body.contains("MissingRights"),
        "delete the R2 missing-field check → selftest non-zero"
    );
}

/// Meta-test: delete the R3 third-party check → this selftest is non-zero.
#[test]
fn selftest_delete_r3_is_nonzero() {
    let body = fn_body(production_licence_src(), "evaluate_artifact");
    assert!(
        body.contains("R3_THIRD_PARTY_PUBLIC_DOMAIN"),
        "delete the R3 token interpolation → selftest non-zero"
    );
    assert!(
        body.contains("third_party_figures") && body.contains("PUBLIC_DOMAIN"),
        "delete the R3 third-party check → selftest non-zero"
    );
}

/// Meta-test: delete the R4 index exclusion → this selftest is non-zero.
#[test]
fn selftest_delete_r4_is_nonzero() {
    let src = production_licence_src();
    let audit = fn_body(src, "audit_index");
    assert!(
        audit.contains("R4_PROHIBITED_INDEX"),
        "delete the R4 token interpolation → selftest non-zero"
    );
    assert!(
        audit.contains("ProhibitedInAgentIndex") && audit.contains("Prohibited"),
        "delete the R4 index check → selftest non-zero"
    );
    let eligible = fn_body(src, "eligible_for_agent_index");
    assert!(
        eligible.contains("AiIngestion::Prohibited"),
        "delete the PROHIBITED exclusion from the index builder → selftest non-zero"
    );
}

/// Meta-test: delete the anti-vacuous empty-scan check → this selftest is non-zero.
#[test]
fn selftest_delete_anti_vacuous_is_nonzero() {
    let body = fn_body(production_licence_src(), "scan");
    assert!(
        body.contains("ANTI_VACUOUS"),
        "delete the anti-vacuous token interpolation → selftest non-zero"
    );
    assert!(
        body.contains("VacuousScan") && body.contains("scanned == 0"),
        "delete the zero-scan ERROR → selftest non-zero"
    );
}

#[test]
fn production_does_not_default_missing_rights_to_permitted() {
    let src = production_licence_src();
    assert!(
        !src.contains("unwrap_or(\"permitted\")")
            && !src.contains("unwrap_or_else(|| \"permitted\")"),
        "a missing field must not collapse to permitted"
    );
}

/// Silence an unused import if a future edit drops the Path use.
#[test]
fn fixture_root_is_a_directory() {
    assert!(Path::new(&fixtures()).is_dir());
}
