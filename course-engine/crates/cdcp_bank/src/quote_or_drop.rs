//! Receipt-backed truth checks for citations in bank item comments.
//!
//! The network sweep is deliberately separate from the hermetic gate.  The
//! `cdcp_gate quote-or-drop --refresh` authoring command fetches public pages,
//! records the HTTP result, response digest, and exact supporting excerpt, and
//! writes a receipt. The ordinary gate only validates that receipt against the
//! current bank and fails closed on DEAD, NON_SUPPORTING, and UNVERIFIABLE
//! citations. Known bot blocks remain visible but are not source failures.
//! It therefore proves neither pedagogical value nor source authority/currentness.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use toml::Value;

pub const NAME: &str = "quote-or-drop";
pub const SUMMARY: &str = "verify citation receipts resolve and carry exact supporting text";
pub const RECEIPT: &str = "docs/receipts/quote-or-drop.json";
const POLICY: &str = "registries/quote_or_drop.toml";

#[derive(Debug, Clone)]
struct Policy {
    item_files: usize,
    citation_rows: usize,
    bot_block_hosts: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Status {
    Dead,
    BotBlocked,
    Supporting,
    NonSupporting,
    Unverifiable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationRow {
    pub item_id: String,
    pub url: String,
    pub status: Status,
    pub http_status: u16,
    pub error: Option<String>,
    pub claim: Option<String>,
    pub supporting_text: Option<String>,
    pub supporting_text_sha256: Option<String>,
    pub response_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub schema: String,
    pub consumer: String,
    pub observed_defect: String,
    pub deletion_condition: String,
    pub generated_at_unix: u64,
    pub bank_sha256: String,
    pub item_file_denominator: usize,
    pub citation_denominator: usize,
    pub rows: Vec<CitationRow>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counts {
    pub cited: usize,
    pub http_resolved: usize,
    pub resolved: usize,
    pub dead: usize,
    pub bot_blocked: usize,
    pub supporting: usize,
    pub non_supporting: usize,
    pub unverifiable: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Eval {
    Ok(String),
    Violation(Vec<String>),
    Error(String),
}

#[derive(Debug, Clone)]
struct CitationTarget {
    item_id: String,
    url: String,
    claim: Option<String>,
}

fn sha256(bytes: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(bytes);
    hex::encode(digest.finalize())
}

fn bank_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let dir = root.join("bank/items");
    let mut files = fs::read_dir(&dir)
        .map_err(|e| format!("read {}: {e}", dir.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|v| v.to_str()) == Some("toml"))
        .collect::<Vec<_>>();
    files.sort();
    if files.is_empty() {
        return Err("zero bank item files (vacuous citation scan)".to_string());
    }
    Ok(files)
}

fn policy(root: &Path) -> Result<Policy, String> {
    let path = root.join(POLICY);
    let raw = fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let table: Value = raw
        .parse()
        .map_err(|e| format!("parse {}: {e}", path.display()))?;
    let item_files = table
        .get("expected_item_files")
        .and_then(Value::as_integer)
        .ok_or_else(|| format!("{}: expected_item_files is required", path.display()))?;
    let citation_rows = table
        .get("expected_citation_rows")
        .and_then(Value::as_integer)
        .ok_or_else(|| format!("{}: expected_citation_rows is required", path.display()))?;
    if item_files <= 0 || citation_rows <= 0 {
        return Err(format!("{}: denominators must be positive", path.display()));
    }
    let bot_block_hosts = table
        .get("bot_block_hosts")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{}: bot_block_hosts is required", path.display()))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_ascii_lowercase)
                .ok_or_else(|| format!("{}: bot_block_hosts must contain strings", path.display()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if bot_block_hosts.is_empty() {
        return Err(format!(
            "{}: bot_block_hosts must not be empty",
            path.display()
        ));
    }
    Ok(Policy {
        item_files: item_files as usize,
        citation_rows: citation_rows as usize,
        bot_block_hosts,
    })
}

fn bank_sha256(root: &Path, files: &[PathBuf]) -> Result<String, String> {
    let mut bytes = Vec::new();
    for file in files {
        let relative = file
            .strip_prefix(root)
            .map_err(|_| format!("{} is outside repository root", file.display()))?;
        bytes.extend_from_slice(relative.to_string_lossy().as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(
            &fs::read(file).map_err(|e| format!("read {}: {e}", file.display()))?,
        );
        bytes.push(0);
    }
    Ok(sha256(&bytes))
}

fn clean_url(token: &str) -> Option<String> {
    let mut url = token.trim_matches(|c: char| matches!(c, '"' | '\'' | '`' | '<' | '>'));
    while matches!(url.chars().last(), Some('.' | ',' | ';' | ')' | ']' | '}')) {
        url = &url[..url.len() - 1];
    }
    (url.starts_with("http://") || url.starts_with("https://")).then(|| url.to_string())
}

fn comment_claim(contents: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let trimmed = line.trim_start();
        for prefix in ["# Source quote:", "# Supporting text:", "# Quote:"] {
            if let Some(value) = trimmed.strip_prefix(prefix) {
                let value = value.trim();
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
        None
    })
}

fn targets(_root: &Path, files: &[PathBuf]) -> Result<Vec<CitationTarget>, String> {
    let mut targets = Vec::new();
    for file in files {
        let item_id = file
            .file_stem()
            .and_then(|v| v.to_str())
            .ok_or_else(|| format!("invalid item filename {}", file.display()))?;
        let contents = fs::read_to_string(file)
            .map_err(|e| format!("read {} as UTF-8: {e}", file.display()))?;
        let claim = comment_claim(&contents);
        let mut seen = BTreeSet::new();
        for token in contents.split_whitespace() {
            if let Some(url) = clean_url(token) {
                if seen.insert(url.clone()) {
                    targets.push(CitationTarget {
                        item_id: item_id.to_string(),
                        url,
                        claim: claim.clone(),
                    });
                }
            }
        }
    }
    if targets.is_empty() {
        return Err("zero cited URLs (vacuous citation scan)".to_string());
    }
    Ok(targets)
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

fn host(url: &str) -> Option<&str> {
    url.split_once("://")?.1.split(['/', '?', '#']).next()
}

fn is_bot_block_host(url: &str, hosts: &[String]) -> bool {
    host(url).is_some_and(|value| {
        let value = value.to_ascii_lowercase();
        hosts
            .iter()
            .any(|known| value == *known || value.ends_with(&format!(".{known}")))
    })
}

fn status_for_http(url: &str, http_status: u16, bot_block_hosts: &[String]) -> Status {
    if matches!(http_status, 403 | 429) && is_bot_block_host(url, bot_block_hosts) {
        Status::BotBlocked
    } else {
        Status::Dead
    }
}

fn fetch(url: &str, claim: Option<&str>, bot_block_hosts: &[String]) -> CitationRow {
    if url.to_ascii_lowercase().contains(".pdf") {
        return CitationRow {
            item_id: String::new(),
            url: url.to_string(),
            status: Status::Unverifiable,
            http_status: 0,
            error: Some("PDF source is excluded by the public-source policy".to_string()),
            claim: claim.map(str::to_string),
            supporting_text: None,
            supporting_text_sha256: None,
            response_sha256: None,
        };
    }
    let output = Command::new("curl")
        .args([
            "-L",
            "--silent",
            "--show-error",
            "--connect-timeout",
            "3",
            "--max-time",
            "5",
            "--write-out",
            "\n__CDCP_HTTP_STATUS__:%{http_code}",
            url,
        ])
        .output();
    let Ok(output) = output else {
        return CitationRow {
            item_id: String::new(),
            url: url.to_string(),
            status: Status::Dead,
            http_status: 0,
            error: Some("curl could not be started".to_string()),
            claim: claim.map(str::to_string),
            supporting_text: None,
            supporting_text_sha256: None,
            response_sha256: None,
        };
    };
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let marker = b"\n__CDCP_HTTP_STATUS__:";
    let Some(marker_at) = output
        .stdout
        .windows(marker.len())
        .position(|w| w == marker)
    else {
        return CitationRow {
            item_id: String::new(),
            url: url.to_string(),
            status: Status::Dead,
            http_status: 0,
            error: Some(if stderr.is_empty() {
                "no HTTP status".to_string()
            } else {
                stderr
            }),
            claim: claim.map(str::to_string),
            supporting_text: None,
            supporting_text_sha256: None,
            response_sha256: None,
        };
    };
    let body = &output.stdout[..marker_at];
    let status_text = String::from_utf8_lossy(&output.stdout[marker_at + marker.len()..]);
    let http_status = status_text.trim().parse::<u16>().unwrap_or_default();
    let response_sha256 = Some(sha256(body));
    if !(200..300).contains(&http_status) {
        let status = status_for_http(url, http_status, bot_block_hosts);
        return CitationRow {
            item_id: String::new(),
            url: url.to_string(),
            status,
            http_status,
            error: Some(if stderr.is_empty() {
                format!("HTTP status {http_status}")
            } else {
                stderr
            }),
            claim: claim.map(str::to_string),
            supporting_text: None,
            supporting_text_sha256: None,
            response_sha256,
        };
    }
    let Some(claim) = claim else {
        return CitationRow {
            item_id: String::new(),
            url: url.to_string(),
            status: Status::Unverifiable,
            http_status,
            error: Some("no exact source quote was recorded in the item".to_string()),
            claim: None,
            supporting_text: None,
            supporting_text_sha256: None,
            response_sha256,
        };
    };
    let status = if body
        .windows(claim.len())
        .any(|window| window == claim.as_bytes())
    {
        Status::Supporting
    } else {
        Status::NonSupporting
    };
    CitationRow {
        item_id: String::new(),
        url: url.to_string(),
        status,
        http_status,
        error: None,
        claim: Some(claim.to_string()),
        supporting_text: (status == Status::Supporting).then(|| claim.to_string()),
        supporting_text_sha256: (status == Status::Supporting).then(|| sha256(claim.as_bytes())),
        response_sha256,
    }
}

fn write_receipt(root: &Path, receipt: &Receipt) -> Result<(), String> {
    let path = root.join(RECEIPT);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
    }
    let json = serde_json::to_vec_pretty(receipt).map_err(|e| format!("encode receipt: {e}"))?;
    fs::write(&path, json).map_err(|e| format!("write {}: {e}", path.display()))
}

/// Fetch all cited URLs and write the periodic, non-hermetic evidence receipt.
/// This is intentionally not used by the ordinary gate/check chain.
pub fn refresh(root: &Path) -> Result<(Receipt, Counts), String> {
    let policy = policy(root)?;
    let files = bank_files(root)?;
    if files.len() != policy.item_files {
        return Err(format!(
            "predeclared item denominator {} does not match current bank {}",
            policy.item_files,
            files.len()
        ));
    }
    let targets = targets(root, &files)?;
    if targets.len() != policy.citation_rows {
        return Err(format!(
            "predeclared citation denominator {} does not match current bank {}",
            policy.citation_rows,
            targets.len()
        ));
    }
    let bank_sha256 = bank_sha256(root, &files)?;
    let mut unique = BTreeMap::<(String, Option<String>), CitationTarget>::new();
    for target in &targets {
        unique
            .entry((target.url.clone(), target.claim.clone()))
            .or_insert_with(|| target.clone());
    }
    let unique_targets = unique.into_values().collect::<Vec<_>>();
    let worker_count = 32usize.min(unique_targets.len());
    let chunk_size = unique_targets.len().div_ceil(worker_count);
    let mut workers = Vec::new();
    for chunk in unique_targets.chunks(chunk_size) {
        let jobs = chunk.to_vec();
        let bot_block_hosts = policy.bot_block_hosts.clone();
        workers.push(std::thread::spawn(move || {
            jobs.into_iter()
                .map(|target| {
                    let mut row = fetch(&target.url, target.claim.as_deref(), &bot_block_hosts);
                    row.item_id = target.item_id;
                    row.claim = target.claim.or(row.claim);
                    row
                })
                .collect::<Vec<_>>()
        }));
    }
    let mut fetched = BTreeMap::<(String, Option<String>), CitationRow>::new();
    for worker in workers {
        for row in worker
            .join()
            .map_err(|_| "citation refresh worker panicked".to_string())?
        {
            fetched.insert((row.url.clone(), row.claim.clone()), row);
        }
    }
    let mut rows = Vec::with_capacity(targets.len());
    for target in targets {
        let key = (target.url.clone(), target.claim.clone());
        let mut row = fetched
            .get(&key)
            .cloned()
            .ok_or_else(|| format!("refresh lost fetched URL {}", target.url))?;
        row.item_id = target.item_id;
        row.claim = target.claim.or(row.claim);
        rows.push(row);
    }
    rows.sort_by(|left, right| {
        left.item_id
            .cmp(&right.item_id)
            .then(left.url.cmp(&right.url))
    });
    let receipt = Receipt {
        schema: "zs.quote-or-drop.v1".to_string(),
        consumer: "quote-or-drop gate; prevents unsupported citations from being treated as grounding".to_string(),
        observed_defect: "917 bank files carried URLs that no code had opened or checked for claim support".to_string(),
        deletion_condition: "delete only when every bank citation is covered by a successor truth oracle with the same fail-closed and causal legs".to_string(),
        generated_at_unix: now_unix(),
        bank_sha256,
        item_file_denominator: files.len(),
        citation_denominator: rows.len(),
        rows,
    };
    write_receipt(root, &receipt)?;
    Ok((receipt.clone(), counts(&receipt.rows)))
}

fn counts(rows: &[CitationRow]) -> Counts {
    let mut counts = Counts {
        cited: rows.len(),
        ..Counts::default()
    };
    for row in rows {
        match row.status {
            Status::Dead => counts.dead += 1,
            Status::BotBlocked => counts.bot_blocked += 1,
            Status::Supporting => {
                counts.http_resolved += 1;
                counts.resolved += 1;
                counts.supporting += 1;
            }
            Status::NonSupporting => {
                counts.http_resolved += 1;
                counts.non_supporting += 1;
            }
            Status::Unverifiable => {
                if (200..300).contains(&row.http_status) {
                    counts.http_resolved += 1;
                }
                counts.unverifiable += 1;
            }
        }
    }
    counts
}

fn load_receipt(root: &Path) -> Result<Receipt, String> {
    let path = root.join(RECEIPT);
    let raw = fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    serde_json::from_str(&raw).map_err(|e| format!("parse {}: {e}", path.display()))
}

fn validate_row(row: &CitationRow, failures: &mut Vec<String>) {
    match row.status {
        Status::Supporting => {
            if !(200..300).contains(&row.http_status) {
                failures.push(format!(
                    "{} {} marked SUPPORTING with HTTP {}",
                    row.item_id, row.url, row.http_status
                ));
            }
            let Some(claim) = row.claim.as_deref() else {
                failures.push(format!(
                    "{} {} SUPPORTING row has no claim",
                    row.item_id, row.url
                ));
                return;
            };
            let Some(text) = row.supporting_text.as_deref() else {
                failures.push(format!(
                    "{} {} SUPPORTING row has no supporting text",
                    row.item_id, row.url
                ));
                return;
            };
            if !text.contains(claim) {
                failures.push(format!(
                    "{} {} supporting text does not contain the claim",
                    row.item_id, row.url
                ));
            }
            if row.supporting_text_sha256.as_deref() != Some(sha256(text.as_bytes()).as_str()) {
                failures.push(format!(
                    "{} {} supporting-text digest does not match the excerpt",
                    row.item_id, row.url
                ));
            }
            if row.response_sha256.as_deref().is_none() {
                failures.push(format!(
                    "{} {} SUPPORTING row has no response digest",
                    row.item_id, row.url
                ));
            }
        }
        Status::Dead => {
            if (200..300).contains(&row.http_status) {
                failures.push(format!(
                    "{} {} marked DEAD with HTTP {}",
                    row.item_id, row.url, row.http_status
                ));
            }
        }
        Status::BotBlocked => {
            if !matches!(row.http_status, 403 | 429) {
                failures.push(format!(
                    "{} {} marked BOT_BLOCKED with HTTP {}",
                    row.item_id, row.url, row.http_status
                ));
            }
        }
        Status::NonSupporting => {
            if !(200..300).contains(&row.http_status) {
                failures.push(format!(
                    "{} {} NON_SUPPORTING without a resolved HTTP response",
                    row.item_id, row.url
                ));
            }
            if row.claim.as_deref().unwrap_or_default().is_empty() {
                failures.push(format!(
                    "{} {} NON_SUPPORTING row has no claim",
                    row.item_id, row.url
                ));
            }
        }
        Status::Unverifiable => {
            failures.push(format!(
                "{} {} is UNVERIFIABLE: {}",
                row.item_id,
                row.url,
                row.error.as_deref().unwrap_or("no reason recorded")
            ));
        }
    }
}

/// Validate the committed receipt against the current bank. This path performs
/// no network I/O and is suitable for a deterministic gate.
pub fn evaluate(root: &Path) -> Eval {
    let policy = match policy(root) {
        Ok(policy) => policy,
        Err(message) => return Eval::Error(message),
    };
    let files = match bank_files(root) {
        Ok(files) => files,
        Err(message) => return Eval::Error(message),
    };
    let targets = match targets(root, &files) {
        Ok(targets) => targets,
        Err(message) => return Eval::Error(message),
    };
    let receipt = match load_receipt(root) {
        Ok(receipt) => receipt,
        Err(message) => return Eval::Error(message),
    };
    if receipt.item_file_denominator != policy.item_files || files.len() != policy.item_files {
        return Eval::Error(format!(
            "item denominator must be predeclared as {}; bank={} receipt={}",
            policy.item_files,
            files.len(),
            receipt.item_file_denominator
        ));
    }
    if receipt.citation_denominator != policy.citation_rows
        || receipt.citation_denominator != targets.len()
        || receipt.rows.len() != targets.len()
    {
        return Eval::Error(format!(
            "citation denominator drift: predeclared={} current={} receipt={} rows={}",
            policy.citation_rows,
            targets.len(),
            receipt.citation_denominator,
            receipt.rows.len()
        ));
    }
    let expected_hash = match bank_sha256(root, &files) {
        Ok(hash) => hash,
        Err(message) => return Eval::Error(message),
    };
    if receipt.bank_sha256 != expected_hash {
        return Eval::Violation(vec![format!(
            "receipt bank SHA {} does not match current bank {}",
            receipt.bank_sha256, expected_hash
        )]);
    }
    let expected = targets
        .into_iter()
        .map(|target| ((target.item_id.clone(), target.url.clone()), target))
        .collect::<BTreeMap<_, _>>();
    let mut actual = BTreeSet::new();
    let mut failures = Vec::new();
    for row in &receipt.rows {
        let key = (row.item_id.clone(), row.url.clone());
        if !actual.insert(key.clone()) {
            failures.push(format!("duplicate receipt row {} {}", row.item_id, row.url));
        }
        let Some(target) = expected.get(&key) else {
            failures.push(format!(
                "receipt row {} {} is not in current bank",
                row.item_id, row.url
            ));
            continue;
        };
        if row.claim != target.claim {
            failures.push(format!(
                "{} {} claim changed without a new sweep",
                row.item_id, row.url
            ));
        }
        validate_row(row, &mut failures);
        if matches!(
            row.status,
            Status::Dead | Status::NonSupporting | Status::Unverifiable
        ) {
            failures.push(format!(
                "{} {} is not admissible: status={:?}; unresolved citations fail closed",
                row.item_id, row.url, row.status
            ));
        }
    }
    if actual.len() != expected.len() {
        failures.push(format!(
            "receipt coverage {} of {} citations",
            actual.len(),
            expected.len()
        ));
    }
    let counts = counts(&receipt.rows);
    let mut report = format!(
        "{NAME}: cited={} http_resolved={} resolved_for_grounding={} dead={} bot_blocked={} supporting={} non_supporting={} unverifiable={} item_files={}",
        counts.cited,
        counts.http_resolved,
        counts.resolved,
        counts.dead,
        counts.bot_blocked,
        counts.supporting,
        counts.non_supporting,
        counts.unverifiable,
        files.len()
    );
    let _ = write!(
        report,
        "; green means only that the citation receipt resolves and its exact excerpt supports the claim; it does not prove pedagogical usefulness, authority for a jurisdiction, or currentness"
    );
    if failures.is_empty() {
        Eval::Ok(report)
    } else {
        failures.push(report);
        Eval::Violation(failures)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn fixture(
        status: Status,
        http_status: u16,
        claim: Option<&str>,
        text: Option<&str>,
    ) -> Receipt {
        Receipt {
            schema: "zs.quote-or-drop.v1".to_string(),
            consumer: "test consumer".to_string(),
            observed_defect: "test defect".to_string(),
            deletion_condition: "test successor".to_string(),
            generated_at_unix: 1,
            bank_sha256: String::new(),
            item_file_denominator: 1,
            citation_denominator: 1,
            rows: vec![CitationRow {
                item_id: "m01-q001".to_string(),
                url: "https://example.test/source".to_string(),
                status,
                http_status,
                error: (status != Status::Supporting).then(|| "fixture reason".to_string()),
                claim: claim.map(str::to_string),
                supporting_text: text.map(str::to_string),
                supporting_text_sha256: text.map(|value| sha256(value.as_bytes())),
                response_sha256: Some("digest".to_string()),
            }],
        }
    }

    #[test]
    fn supporting_receipt_passes_row_detector() {
        let row = fixture(
            Status::Supporting,
            200,
            Some("exact claim"),
            Some("exact claim"),
        );
        let mut failures = Vec::new();
        validate_row(&row.rows[0], &mut failures);
        assert!(
            failures.is_empty(),
            "intact supporting fixture: {failures:?}"
        );
    }

    #[test]
    fn committed_causal_fixtures_are_read_by_the_product_detector() {
        for name in [
            "supporting.json",
            "unreachable_bypassed.json",
            "non_supporting_bypassed.json",
            "bot_blocked.json",
        ] {
            let path = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/quote_or_drop")
                .join(name);
            let receipt: Receipt =
                serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
            let mut failures = Vec::new();
            validate_row(&receipt.rows[0], &mut failures);
            if matches!(name, "supporting.json" | "bot_blocked.json") {
                assert!(
                    failures.is_empty(),
                    "supporting fixture failed: {failures:?}"
                );
            } else {
                assert!(
                    !failures.is_empty(),
                    "bypassed fixture did not fail: {name}"
                );
            }
        }
    }

    #[test]
    fn unreachable_branch_is_causal() {
        let row = fixture(
            Status::Supporting,
            404,
            Some("exact claim"),
            Some("exact claim"),
        );
        let mut failures = Vec::new();
        validate_row(&row.rows[0], &mut failures);
        assert!(failures.iter().any(|f| f.contains("HTTP 404")));
    }

    #[test]
    fn non_supporting_branch_is_causal() {
        let row = fixture(
            Status::NonSupporting,
            200,
            Some("missing claim"),
            Some("other text"),
        );
        let mut failures = Vec::new();
        validate_row(&row.rows[0], &mut failures);
        assert!(
            failures.is_empty(),
            "a recorded non-supporting row is a valid finding"
        );

        let bypassed = fixture(
            Status::Supporting,
            200,
            Some("missing claim"),
            Some("other text"),
        );
        let mut bypass_failures = Vec::new();
        validate_row(&bypassed.rows[0], &mut bypass_failures);
        assert!(
            bypass_failures
                .iter()
                .any(|f| f.contains("does not contain"))
        );
    }

    #[test]
    fn pdf_is_unverifiable_and_not_fetched() {
        let row = fetch("https://example.test/standard.pdf", Some("claim"), &[]);
        assert_eq!(row.status, Status::Unverifiable);
        assert!(row.error.unwrap().contains("PDF"));
    }

    #[test]
    fn bot_blocked_is_not_a_citation_failure_but_bypass_is() {
        let hosts = vec!["iso.org".to_string()];
        assert_eq!(
            status_for_http("https://www.iso.org/standard/1", 403, &hosts),
            Status::BotBlocked
        );
        assert_eq!(
            status_for_http("https://www.iso.org/standard/1", 404, &hosts),
            Status::Dead
        );

        let blocked = CitationRow {
            item_id: "m01-q001".to_string(),
            url: "https://www.iso.org/standard/78550.html?browse=tc".to_string(),
            status: Status::BotBlocked,
            http_status: 403,
            error: Some("known standards host blocks automation".to_string()),
            claim: None,
            supporting_text: None,
            supporting_text_sha256: None,
            response_sha256: Some("digest".to_string()),
        };
        let mut failures = Vec::new();
        validate_row(&blocked, &mut failures);
        assert!(
            failures.is_empty(),
            "bot-blocked citation failed: {failures:?}"
        );

        let bypassed = CitationRow {
            status: Status::Supporting,
            ..blocked
        };
        let mut bypass_failures = Vec::new();
        validate_row(&bypassed, &mut bypass_failures);
        assert!(!bypass_failures.is_empty(), "bot-block branch was bypassed");
    }

    #[test]
    fn missing_receipt_is_an_error_not_a_pass() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("bank/items")).unwrap();
        fs::write(
            dir.path().join("bank/items/m01-q001.toml"),
            "# https://example.test/source\nid = \"m01-q001\"\n",
        )
        .unwrap();
        assert!(matches!(evaluate(dir.path()), Eval::Error(_)));
    }
}
