//! Workspace Cargo.lock must not grow an HTTP client. Offline-first.

use std::collections::BTreeSet;
use std::path::PathBuf;

/// Package names that mean "this crate talks to the network".
const BANNED: &[&str] = &[
    "reqwest",
    "reqwest-middleware",
    "ureq",
    "hyper",
    "hyper-util",
    "hyper-tls",
    "hyper-rustls",
    "attohttpc",
    "isahc",
    "surf",
    "minreq",
    "awc",
    "ehttp",
    "curl",
    "curl-sys",
    "http",
    "http-body",
    "http-body-util",
];

fn cargo_lock() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.lock")
}

fn package_names(text: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let mut in_package = false;
    for line in text.lines() {
        let t = line.trim();
        if t == "[[package]]" {
            in_package = true;
            continue;
        }
        if t.starts_with('[') {
            in_package = false;
            continue;
        }
        if in_package {
            if let Some(rest) = t.strip_prefix("name = \"") {
                if let Some(name) = rest.strip_suffix('"') {
                    names.insert(name.to_string());
                }
            }
        }
    }
    names
}

#[test]
fn cargo_lock_has_no_http_client() {
    let path = cargo_lock();
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let names = package_names(&text);
    assert!(
        !names.is_empty(),
        "Cargo.lock listed zero packages — a vacuous lock read is an ERROR"
    );
    let hits: Vec<&str> = BANNED
        .iter()
        .copied()
        .filter(|b| names.contains(*b))
        .collect();
    assert!(
        hits.is_empty(),
        "HTTP dependency entered Cargo.lock: {hits:?}"
    );
}

#[test]
fn cdcp_site_manifest_has_no_http_dep() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("crate Cargo.toml");
    for needle in ["reqwest", "ureq", "hyper", "attohttpc", "isahc"] {
        assert!(
            !text.contains(needle),
            "cdcp_site Cargo.toml mentions {needle}"
        );
    }
}

#[test]
fn banned_list_is_non_empty() {
    assert!(!BANNED.is_empty(), "an empty ban list certifies nothing");
}
