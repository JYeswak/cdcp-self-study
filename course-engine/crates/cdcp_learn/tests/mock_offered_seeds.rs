//! bd-installability-sm4g.3 — every seed the mock UI offers must resolve.
//!
//! The seed list is parsed from `web/mock.html` itself (`#seed-select`).
//! A hardcoded `[42]` would stay green if the menu grew a seed with no
//! pack/bank/keys. An empty menu is an ERROR, not a pass.
//!
//! "Resolves" means the three browser assets the take + grade path fetch
//! (`mock40_seed{N}.json`, `bank_items_seed{N}.json`, `keys_seed{N}.json`)
//! exist under `web/data/` and return HTTP 200 when served as the learner
//! server maps them (`/data/...`). Offering a seed with no data is RED.
//!
//! This is product, not a gate. Deleting this file is how the original
//! 404-for-seeds-7-1-99 hole comes back.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Raise when you add a `#[test]`. A DROP means a case was deleted.
const EXPECTED_CASES: usize = 7;

fn engine_root() -> PathBuf {
    cdcp_learn::resolve_engine_root(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
}

fn read_required(path: &Path) -> String {
    assert!(
        path.is_file(),
        "required file missing: {} — an empty scan is an ERROR, not a pass",
        path.display()
    );
    let text =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    assert!(
        !text.trim().is_empty(),
        "{} is empty — an empty scan is an ERROR, not a pass",
        path.display()
    );
    text
}

/// Pack / bank / keys the mock take + results grade path fetch for `seed`.
fn seed_asset_rels(seed: u64) -> [String; 3] {
    [
        format!("data/mock40_seed{seed}.json"),
        format!("data/bank_items_seed{seed}.json"),
        format!("data/keys_seed{seed}.json"),
    ]
}

fn strip_html_comments(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(start) = rest.find("<!--") {
        out.push_str(&rest[..start]);
        match rest[start + 4..].find("-->") {
            Some(end) => rest = &rest[start + 4 + end + 3..],
            None => return out,
        }
    }
    out.push_str(rest);
    out
}

fn seed_select_inner(html: &str) -> Result<String, String> {
    let cleaned = strip_html_comments(html);
    let id_at = cleaned
        .find("id=\"seed-select\"")
        .or_else(|| cleaned.find("id='seed-select'"))
        .ok_or_else(|| {
            "mock.html has no #seed-select — an empty seed list is an ERROR, not a pass".to_string()
        })?;
    let after_id = &cleaned[id_at..];
    let gt = after_id.find('>').ok_or_else(|| {
        "mock.html #seed-select opening tag is unterminated — cannot read the seed list".to_string()
    })?;
    let inner_start = id_at + gt + 1;
    let rest = &cleaned[inner_start..];
    let rest_l = rest.to_ascii_lowercase();
    let end = rest_l.find("</select>").ok_or_else(|| {
        "mock.html #seed-select is unclosed — cannot read the seed list".to_string()
    })?;
    Ok(rest[..end].to_string())
}

fn option_value(tag: &str) -> Option<u64> {
    let lower = tag.to_ascii_lowercase();
    let idx = lower.find("value")?;
    let after = tag[idx + 5..].trim_start();
    let after = after.strip_prefix('=').unwrap_or(after).trim_start();
    let raw = if let Some(rest) = after.strip_prefix('"') {
        rest.split('"').next()?
    } else if let Some(rest) = after.strip_prefix('\'') {
        rest.split('\'').next()?
    } else {
        after
            .split(|c: char| c.is_whitespace() || c == '>')
            .next()?
    };
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    raw.parse().ok()
}

/// Seeds offered by a mock.html body. Empty / missing menu is Err.
fn parse_offered_seeds(html: &str) -> Result<Vec<u64>, String> {
    let inner = seed_select_inner(html)?;
    let inner_l = inner.to_ascii_lowercase();
    let mut seeds = Vec::new();
    let mut i = 0usize;
    while let Some(rel) = inner_l[i..].find("<option") {
        let start = i + rel;
        let tag_end = inner_l[start..]
            .find('>')
            .map(|n| start + n)
            .unwrap_or(inner.len());
        let tag = &inner[start..tag_end];
        if let Some(v) = option_value(tag) {
            seeds.push(v);
        }
        i = tag_end.saturating_add(1);
        if i >= inner.len() {
            break;
        }
    }
    if seeds.is_empty() {
        return Err(
            "mock.html #seed-select offers no seeds — an empty seed list is an ERROR, not a pass"
                .into(),
        );
    }
    Ok(seeds)
}

fn resolve_offered_seeds(html: &str, web_root: &Path) -> Result<Vec<u64>, String> {
    let seeds = parse_offered_seeds(html)?;
    let mut missing = Vec::new();
    for seed in &seeds {
        for rel in seed_asset_rels(*seed) {
            let path = cdcp_learn::join_rel(web_root, &rel);
            if !path.is_file() {
                missing.push(format!("seed {seed}: missing web/{rel}"));
                continue;
            }
            let len = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            if len == 0 {
                missing.push(format!("seed {seed}: empty web/{rel}"));
            }
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "offered seed has no pack+bank+keys: {}",
            missing.join("; ")
        ));
    }
    Ok(seeds)
}

/// One-shot static GET against `web_root`. Mirrors `cdcp serve` path mapping
/// (`/data/foo.json` → `<web_root>/data/foo.json`): file present → 200, else 404.
fn http_get_status(web_root: &Path, url_path: &str) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral test listener");
    let addr = listener.local_addr().expect("local_addr");
    let web_root = web_root.to_path_buf();
    let server = std::thread::spawn(move || -> std::io::Result<()> {
        let (mut stream, _) = listener.accept()?;
        let mut line = String::new();
        BufReader::new(&stream).read_line(&mut line)?;
        let rel = line
            .split_whitespace()
            .nth(1)
            .unwrap_or("/")
            .split('?')
            .next()
            .unwrap_or("/")
            .trim_start_matches('/');
        let candidate = cdcp_learn::join_rel(&web_root, rel);
        let (code, reason, body): (&str, &str, Vec<u8>) = if candidate.is_file() {
            match std::fs::read(&candidate) {
                Ok(bytes) => ("200", "OK", bytes),
                Err(_) => ("500", "Internal Server Error", b"read error".to_vec()),
            }
        } else {
            ("404", "Not Found", b"not found".to_vec())
        };
        let head = format!(
            "HTTP/1.1 {code} {reason}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        stream.write_all(head.as_bytes())?;
        stream.write_all(&body)?;
        Ok(())
    });

    let mut last = None;
    let mut client = None;
    for _ in 0..40 {
        match TcpStream::connect_timeout(&addr, Duration::from_millis(50)) {
            Ok(s) => {
                client = Some(s);
                break;
            }
            Err(e) => {
                last = Some(e);
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }
    let mut client = client.unwrap_or_else(|| {
        panic!("test client could not connect to in-process static server: {last:?}")
    });
    client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("read timeout");
    client
        .set_write_timeout(Some(Duration::from_secs(2)))
        .expect("write timeout");
    let req = format!("GET {url_path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n");
    client.write_all(req.as_bytes()).expect("write request");
    let mut status_line = String::new();
    BufReader::new(client)
        .read_line(&mut status_line)
        .expect("read status line");
    let _ = server.join();
    status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| panic!("no HTTP status in {status_line:?}"))
}

#[test]
fn empty_seed_list_is_error() {
    let err = parse_offered_seeds(r#"<select id="seed-select"></select>"#)
        .expect_err("empty #seed-select must be ERROR");
    assert!(
        err.contains("empty seed list"),
        "empty menu must name the vacuous case: {err}"
    );
}

#[test]
fn missing_seed_select_is_error() {
    let err = parse_offered_seeds("<html><body>no menu</body></html>")
        .expect_err("missing #seed-select must be ERROR");
    assert!(
        err.contains("no #seed-select") || err.contains("empty seed list"),
        "missing menu must not parse as a seed list: {err}"
    );
}

#[test]
fn commented_out_options_are_not_a_seed_list() {
    let html = r#"<select id="seed-select"><!-- <option value="42">42</option> --></select>"#;
    let err = parse_offered_seeds(html).expect_err("comment-only menu must be ERROR");
    assert!(
        err.contains("empty seed list"),
        "commented options must not count as offered seeds: {err}"
    );
}

#[test]
fn offered_seed_without_data_is_error() {
    let html = r#"<select id="seed-select"><option value="7">7</option></select>"#;
    let dir = tempfile::tempdir().expect("tempdir");
    let err = resolve_offered_seeds(html, dir.path())
        .expect_err("offering seed 7 with no pack+bank+keys must be RED");
    assert!(
        err.contains("7"),
        "missing-data error must name the offered seed: {err}"
    );
    assert!(
        err.contains("pack+bank+keys") || err.contains("missing"),
        "missing-data error must say the assets are absent: {err}"
    );
    let status = http_get_status(dir.path(), "/data/mock40_seed7.json");
    assert_eq!(
        status, 404,
        "a seed with no committed pack must HTTP 404, not {status}"
    );
}

#[test]
fn mock_html_offered_seeds_resolve_pack_bank_keys() {
    let root = engine_root();
    let mock_html = root.join("web/mock.html");
    let html = read_required(&mock_html);
    let web = root.join("web");
    assert!(
        web.is_dir(),
        "web/ missing — nothing to resolve seeds against"
    );

    let seeds = resolve_offered_seeds(&html, &web)
        .unwrap_or_else(|e| panic!("every seed mock.html offers must resolve pack+bank+keys: {e}"));
    assert!(
        !seeds.is_empty(),
        "mock.html #seed-select is empty — an empty seed list is an ERROR, not a pass"
    );

    for seed in &seeds {
        for rel in seed_asset_rels(*seed) {
            let url = format!("/{rel}");
            let status = http_get_status(&web, &url);
            assert_eq!(
                status, 200,
                "seed {seed} {url} must HTTP 200 (got {status}) — offering a seed with no data is RED"
            );
        }
    }
}

#[test]
fn mock_js_does_not_instruct_export_web() {
    let root = engine_root();
    let js = read_required(&root.join("web/assets/js/mock.js"));
    assert!(
        !js.contains("export-web --seed") && !js.contains("cdcp export-web"),
        "mock.js must not tell an installed learner to run export-web"
    );
    assert!(
        js.contains("does not include a mock exam")
            || js.contains("Use a seed listed in the Seed menu"),
        "mock.js missing-pack path must be actionable without a source checkout"
    );
    let page = read_required(&root.join("web/mock.html"));
    assert!(
        !page.contains("export-web --seed"),
        "mock.html must not tell a learner to run export-web"
    );
}

#[test]
fn suite_declares_its_cases() {
    let src = include_str!("mock_offered_seeds.rs");
    let n = src.lines().filter(|l| l.trim() == "#[test]").count();
    assert_eq!(
        n, EXPECTED_CASES,
        "mock_offered_seeds declares {n} test functions, expected {EXPECTED_CASES} — a deleted case is not a pass"
    );
}
