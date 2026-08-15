//! EXTRACT-THEN-DELETE: this is NOT a differential against
//! `scripts/fetch_public_corpus.py`. The Python is deleted in the same
//! commit. Identity is the access policy + no-socket contract.
//! Known-bad: paid source is RED / skipped and never written.

use cdcp_data::{
    fetch_corpus, parse_sources, plan_sources, AccessKind, CorpusError, FetchRequest, PlanAction,
    ALLOWED_ACCESS, ANTI_VACUOUS_NONE_ALLOWED, NEVER_WRITTEN, NO_SOCKET, REFUSED_PAID, SOURCES_REL,
};
use std::fs;
use std::path::{Path, PathBuf};
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
        "cdcp-fetch-corpus-{}-{}-{}-{}",
        tag,
        std::process::id(),
        n,
        nanos
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("scratch");
    dir
}

fn write_rel(root: &Path, rel: &str, bytes: &[u8]) {
    let p = root.join(rel);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(p, bytes).unwrap();
}

#[test]
fn production_corpus_module_refuses_paid_and_has_no_socket() {
    let src = include_str!("../src/corpus.rs");
    assert!(src.contains("REFUSED_PAID"));
    assert!(src.contains("NEVER_WRITTEN"));
    assert!(src.contains("NO_SOCKET"));
    assert!(src.contains("AccessKind::Paid"));
    assert!(
        src.contains("PlanAction::RefusePaid"),
        "paid must classify as RefusePaid"
    );
    for needle in [
        "TcpStream",
        "UdpSocket",
        "TcpListener",
        "std::net",
        "::net::",
        "ToSocketAddrs",
        "reqwest",
        "ureq",
        "hyper",
        "urllib",
    ] {
        assert!(!src.contains(needle), "corpus.rs mentions {needle}");
    }
}

#[test]
fn allowed_set_is_the_retired_three() {
    assert_eq!(ALLOWED_ACCESS, &["public_summary", "free", "local"]);
    assert!(!ALLOWED_ACCESS.contains(&"paid"));
    assert!(AccessKind::PublicSummary.is_allowed());
    assert!(AccessKind::Free.is_allowed());
    assert!(AccessKind::Local.is_allowed());
    assert!(!AccessKind::Paid.is_allowed());
    assert!(!AccessKind::Unknown("restricted".into()).is_allowed());
}

#[test]
fn parse_sources_empty_is_error() {
    let err = parse_sources("schema_version = 1\n", "empty.toml").expect_err("empty");
    assert!(matches!(err, CorpusError::EmptySources), "{err:?}");
    let err = parse_sources("source = []\n", "arr.toml").expect_err("empty arr");
    assert!(matches!(err, CorpusError::EmptySources), "{err:?}");
}

#[test]
fn path_escape_id_is_refused() {
    let text = r#"
[[source]]
id = "../escape"
url = "file://notes/a.txt"
access = "local"
"#;
    let err = parse_sources(text, "bad.toml").expect_err("escape id");
    assert!(matches!(err, CorpusError::UnsafeId { .. }), "{err:?}");
}

#[test]
fn python_fetcher_is_gone() {
    assert!(
        !engine().join("scripts/fetch_public_corpus.py").exists(),
        "EXTRACT-THEN-DELETE: scripts/fetch_public_corpus.py must stay gone"
    );
}

#[test]
fn live_sources_dry_run_refuses_paid_and_does_not_write() {
    let root = engine();
    let req = FetchRequest {
        root: root.clone(),
        sources: root.join(SOURCES_REL),
        out_dir: root.join("target/cdcp-fetch-corpus-must-not-write"),
        dry_run: true,
        fetched: "2026-08-15".into(),
    };
    let before = if req.out_dir.exists() {
        fs::read_dir(&req.out_dir).map(|rd| rd.count()).unwrap_or(0)
    } else {
        0
    };
    let report = fetch_corpus(&req).expect("live dry-run");
    assert!(report.dry_run);
    assert_eq!(report.wrote, 0);
    assert!(
        report.plan.refused_paid >= 1,
        "live ledger must still carry the NFPA paid catalog row: {:?}",
        report.plan
    );
    assert!(
        report.plan.allowed >= 1,
        "live ledger must still have allowed rows"
    );
    let text = report.to_string();
    assert!(text.contains("DRY-RUN"), "{text}");
    assert!(text.contains("refuse-paid"), "{text}");
    assert!(text.contains("src-nfpa-75"), "{text}");
    assert!(
        text.contains(REFUSED_PAID.split('=').next().unwrap()) || text.contains("paid"),
        "{text}"
    );
    assert!(text.contains(NO_SOCKET), "{text}");
    assert!(
        !req.out_dir.exists()
            || fs::read_dir(&req.out_dir).map(|rd| rd.count()).unwrap_or(0) == before,
        "dry-run must not create dest files"
    );
}

#[test]
fn live_sources_parse_classifies_nfpa_paid() {
    let root = engine();
    let text = fs::read_to_string(root.join(SOURCES_REL)).expect("sources.toml");
    let sources = parse_sources(&text, SOURCES_REL).expect("parse live");
    assert!(
        !sources.is_empty(),
        "{ANTI_VACUOUS_NONE_ALLOWED}: live sources empty"
    );
    let nfpa = sources
        .iter()
        .find(|s| s.id == "src-nfpa-75")
        .expect("src-nfpa-75 must stay in the ledger as the paid known-bad");
    assert_eq!(nfpa.access, AccessKind::Paid);
    assert!(!nfpa.access.is_allowed());
}

#[test]
fn paid_only_fixture_is_red_and_writes_nothing() {
    let tmp = scratch("paid-only");
    let root = tmp.join("engine");
    fs::create_dir_all(&root).unwrap();
    write_rel(
        &root,
        SOURCES_REL,
        br#"schema_version = 1

[[source]]
id = "src-nfpa-plant"
org = "NFPA"
title = "paid catalog"
url = "https://example.invalid/nfpa-75"
access = "paid"
"#,
    );
    let out = root.join("knowledge/corpus/public");
    let req = FetchRequest {
        root: root.clone(),
        sources: root.join(SOURCES_REL),
        out_dir: out.clone(),
        dry_run: true,
        fetched: "2026-08-15".into(),
    };
    let err = fetch_corpus(&req).expect_err("paid-only dry-run is RED");
    match err {
        CorpusError::NoneAllowed {
            refused_paid,
            skipped,
        } => {
            assert_eq!(refused_paid, 1);
            assert_eq!(skipped, 0);
        }
        other => panic!("expected NoneAllowed, got {other:?}"),
    }
    let msg = err.to_string();
    assert!(msg.contains(REFUSED_PAID), "{msg}");
    assert!(msg.contains(NEVER_WRITTEN), "{msg}");
    assert!(msg.contains(ANTI_VACUOUS_NONE_ALLOWED), "{msg}");

    let write_req = FetchRequest {
        dry_run: false,
        ..req
    };
    let err = fetch_corpus(&write_req).expect_err("paid-only write is RED");
    assert!(matches!(err, CorpusError::NoneAllowed { .. }), "{err:?}");
    assert!(
        !out.join("src-nfpa-plant.txt").exists(),
        "paid dest must never be written"
    );
    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn mixed_fixture_writes_local_skips_paid_and_http() {
    let tmp = scratch("mixed");
    let root = tmp.join("engine");
    fs::create_dir_all(&root).unwrap();
    write_rel(&root, "notes/local.txt", b"local body\n\n\nextra\n");
    write_rel(
        &root,
        SOURCES_REL,
        br#"schema_version = 1

[[source]]
id = "src-local-ok"
title = "local note"
url = "file://notes/local.txt"
access = "local"

[[source]]
id = "src-paid-skip"
title = "paid catalog"
url = "https://example.invalid/paid.pdf"
access = "paid"

[[source]]
id = "src-http-skip"
title = "public page"
url = "https://example.invalid/page"
access = "public_summary"
"#,
    );
    let out = root.join("knowledge/corpus/public");
    let req = FetchRequest {
        root: root.clone(),
        sources: root.join(SOURCES_REL),
        out_dir: out.clone(),
        dry_run: false,
        fetched: "2026-08-15".into(),
    };
    let report = fetch_corpus(&req).expect("mixed write");
    assert_eq!(report.wrote, 1);
    assert_eq!(report.plan.refused_paid, 1);
    assert_eq!(report.plan.allowed, 2);
    assert!(out.join("src-local-ok.txt").is_file());
    assert!(
        !out.join("src-paid-skip.txt").exists(),
        "paid dest must never be written"
    );
    assert!(
        !out.join("src-http-skip.txt").exists(),
        "HTTP dest must not be written ({NO_SOCKET})"
    );
    let body = fs::read_to_string(out.join("src-local-ok.txt")).unwrap();
    assert!(body.contains("# source_id: src-local-ok"));
    assert!(body.contains("# access: local"));
    assert!(body.contains("# fetched: 2026-08-15"));
    assert!(body.contains("local body"));
    assert!(
        !body.contains("https://example.invalid/paid"),
        "paid url must not leak into the local snapshot"
    );
    assert!(
        !out.join("manifest.json").exists(),
        "must not clobber v2 manifest"
    );

    let sources = parse_sources(
        &fs::read_to_string(root.join(SOURCES_REL)).unwrap(),
        SOURCES_REL,
    )
    .unwrap();
    let plan = plan_sources(&root, &sources, &out).unwrap();
    let paid = plan
        .rows
        .iter()
        .find(|r| r.source.id == "src-paid-skip")
        .unwrap();
    assert_eq!(paid.action, PlanAction::RefusePaid);
    assert_eq!(paid.source.access, AccessKind::Paid);
    let http = plan
        .rows
        .iter()
        .find(|r| r.source.id == "src-http-skip")
        .unwrap();
    assert_eq!(http.action, PlanAction::WouldFetch);

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn write_does_not_fetch_http_only_ledger() {
    let tmp = scratch("http-only");
    let root = tmp.join("engine");
    fs::create_dir_all(&root).unwrap();
    write_rel(
        &root,
        SOURCES_REL,
        br#"
[[source]]
id = "src-http-only"
url = "https://example.invalid/page"
access = "free"
"#,
    );
    let out = root.join("out");
    let req = FetchRequest {
        root: root.clone(),
        sources: root.join(SOURCES_REL),
        out_dir: out.clone(),
        dry_run: false,
        fetched: "2026-08-15".into(),
    };
    let err = fetch_corpus(&req).expect_err("http-only write is RED");
    assert!(matches!(err, CorpusError::NothingWritten), "{err:?}");
    assert!(err.to_string().contains(NO_SOCKET), "{err}");
    assert!(!out.join("src-http-only.txt").exists());
    let _ = fs::remove_dir_all(&tmp);
}
