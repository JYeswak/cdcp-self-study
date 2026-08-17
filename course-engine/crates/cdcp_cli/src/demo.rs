//! `cdcp demo` — one command that proves an installed tree is the product.
//!
//! Resolves the bundle, prints the study URL (does not block like `serve`),
//! grades a planted all-correct and all-wrong attempt against the installed
//! seed-42 JSON pack, and prints the 2-minute path. The shipped wasm must
//! be present at the N.4 path (`web/assets/wasm/cdcp_wasm.wasm`); missing
//! or not-a-wasm is RED naming the path. Grade itself is native
//! `cdcp_grade` over `web/data/bank_items_seed42.json` + keys — the same
//! payloads the guest wasm grades in-page. Instantiating wasmtime is the
//! dual-path oracle's job, not this command's.
//!
//! Empty planted set is ERROR. bank/, goldens/, python3, and a source
//! checkout are not consulted.

use cdcp_bank::Bank;
use cdcp_core::{AnsweredItem, ChoiceLetter, ExamAttempt};
use cdcp_grade::grade_digest;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::http_serve;

/// N.4 shipped-wasm path, relative to the HTTP document root.
pub(crate) const WASM_REL: &str = "assets/wasm/cdcp_wasm.wasm";

/// Mock seed-42 assets, relative to the HTTP document root.
/// Emptying this list is compile-fail — a demo that requires nothing
/// cannot fail when the pack is missing.
pub(crate) const SEED42_ASSETS: &[&str] = &[
    "data/mock40_seed42.json",
    "data/keys_seed42.json",
    "data/bank_items_seed42.json",
];

const _: () = assert!(
    !SEED42_ASSETS.is_empty(),
    "empty SEED42_ASSETS certifies nothing"
);

const WASM_MAGIC: &[u8] = b"\0asm";

/// Printed after a green demo. The stranger's next three steps.
pub(crate) const TWO_MINUTE_PATH: &str = "\
cdcp demo: 2-minute path
  1. cdcp study
  2. open Mock exam (seed 42)
  3. submit — in-page wasm grades this same pack";

#[derive(Debug, Deserialize)]
struct MockPack {
    exam_id: String,
    seed: u64,
    items: Vec<MockItem>,
}

#[derive(Debug, Deserialize)]
struct MockItem {
    id: String,
}

#[derive(Debug, Deserialize)]
struct KeysFile {
    keys: Vec<KeyRow>,
}

#[derive(Debug, Deserialize)]
struct KeyRow {
    item_id: String,
    correct: String,
}

/// Resolve the bundle, print URL + planted digests + 2-minute path, exit.
pub(crate) fn run(explicit: Option<&Path>, bind: &str, no_open: bool) -> Result<(), String> {
    let resolved = cdcp_root::resolve_from_env(explicit).map_err(|e| e.to_string())?;
    println!("cdcp: {}", resolved.announce());

    let web = resolved.web_dir();
    let index = require_file(&web.join("index.html"), "web/ is not a serveable bundle")?;
    let wasm = require_wasm(&web.join(WASM_REL))?;
    let mut assets = BTreeMap::new();
    for rel in SEED42_ASSETS {
        let path = require_file(&web.join(rel), "missing mock seed-42 asset")?;
        assets.insert(*rel, path);
    }

    let mock_path = &assets["data/mock40_seed42.json"];
    let keys_path = &assets["data/keys_seed42.json"];
    let bank_path = &assets["data/bank_items_seed42.json"];

    let (exam_id, seed, item_ids) = load_planted_ids(mock_path)?;
    let keys = load_keys(keys_path)?;
    if keys.is_empty() {
        return Err(format!(
            "{}: keys is empty — an empty planted set is an ERROR, not a pass",
            keys_path.display()
        ));
    }

    let bank_json =
        fs::read_to_string(bank_path).map_err(|e| format!("read {}: {e}", bank_path.display()))?;
    if bank_json.trim().is_empty() {
        return Err(format!(
            "{} is 0 bytes — an empty bank grades nothing",
            bank_path.display()
        ));
    }
    let bank =
        Bank::from_json_str(&bank_json).map_err(|e| format!("{}: {e}", bank_path.display()))?;
    if bank.items.is_empty() {
        return Err(format!(
            "{}: bank loaded 0 items — an empty planted set is an ERROR",
            bank_path.display()
        ));
    }

    let ac = attempt_from_keys(&bank, &exam_id, seed, &item_ids, &keys, true, keys_path)?;
    let aw = attempt_from_keys(&bank, &exam_id, seed, &item_ids, &keys, false, keys_path)?;
    let ac_digest = grade_digest(&bank, &ac).map_err(|e| e.to_string())?;
    let aw_digest = grade_digest(&bank, &aw).map_err(|e| e.to_string())?;
    if ac_digest == aw_digest {
        return Err(format!(
            "planted all-correct digest equals all-wrong ({ac_digest}) — a grader that cannot tell them apart certifies nothing"
        ));
    }

    let study_url = study_url(bind);
    let file_url = file_url(&index);
    println!("cdcp demo: {study_url}");
    println!("cdcp demo: {file_url}");
    println!("cdcp demo: wasm {}", wasm.display());
    println!("cdcp demo: planted n={}", item_ids.len());
    println!("cdcp demo: all-correct digest={ac_digest}");
    println!("cdcp demo: all-wrong digest={aw_digest}");
    println!("{TWO_MINUTE_PATH}");

    if !no_open {
        http_serve::open_browser(&file_url);
    }
    Ok(())
}

fn study_url(bind: &str) -> String {
    if bind.starts_with("http://") || bind.starts_with("https://") {
        if bind.ends_with('/') {
            bind.to_string()
        } else {
            format!("{bind}/")
        }
    } else {
        format!("http://{bind}/")
    }
}

fn file_url(index: &Path) -> String {
    format!("file://{}", index.display())
}

fn require_file(path: &Path, why: &str) -> Result<PathBuf, String> {
    let shown = abs_path(path);
    if !path.is_file() {
        return Err(format!("missing {} — {why}", shown.display()));
    }
    let meta = fs::metadata(path).map_err(|e| format!("stat {}: {e}", shown.display()))?;
    if meta.len() == 0 {
        return Err(format!(
            "{} is 0 bytes — a present-but-empty file is not an asset",
            shown.display()
        ));
    }
    match path.canonicalize() {
        Ok(p) => Ok(p),
        Err(_) => Ok(shown),
    }
}

fn require_wasm(path: &Path) -> Result<PathBuf, String> {
    let shown = require_file(path, "the browser grade artifact is absent")?;
    let bytes = fs::read(&shown).map_err(|e| format!("read {}: {e}", shown.display()))?;
    if bytes.len() < WASM_MAGIC.len() || !bytes.starts_with(WASM_MAGIC) {
        return Err(format!(
            "{} is not a wasm module (missing \\0asm magic)",
            shown.display()
        ));
    }
    Ok(shown)
}

fn load_planted_ids(path: &Path) -> Result<(String, u64, Vec<String>), String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let pack: MockPack =
        serde_json::from_str(&raw).map_err(|e| format!("parse {}: {e}", path.display()))?;
    let item_ids: Vec<String> = pack
        .items
        .into_iter()
        .map(|it| it.id)
        .filter(|id| !id.is_empty())
        .collect();
    if item_ids.is_empty() {
        return Err(format!(
            "{}: planted item set is empty — an empty planted set is an ERROR, not a pass",
            path.display()
        ));
    }
    Ok((pack.exam_id, pack.seed, item_ids))
}

fn load_keys(path: &Path) -> Result<BTreeMap<String, ChoiceLetter>, String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let file: KeysFile =
        serde_json::from_str(&raw).map_err(|e| format!("parse {}: {e}", path.display()))?;
    let mut map = BTreeMap::new();
    for row in file.keys {
        if row.item_id.is_empty() {
            continue;
        }
        let letter = ChoiceLetter::parse(&row.correct).map_err(|e| {
            format!(
                "{}: key {} has invalid correct letter: {e}",
                path.display(),
                row.item_id
            )
        })?;
        map.insert(row.item_id, letter);
    }
    Ok(map)
}

fn attempt_from_keys(
    bank: &Bank,
    exam_id: &str,
    seed: u64,
    item_ids: &[String],
    keys: &BTreeMap<String, ChoiceLetter>,
    all_correct: bool,
    keys_path: &Path,
) -> Result<ExamAttempt, String> {
    let mut answers = Vec::with_capacity(item_ids.len());
    for id in item_ids {
        if bank.get(id).is_none() {
            return Err(format!("unknown planted item_id: {id}"));
        }
        let letter = keys.get(id).copied().ok_or_else(|| {
            format!(
                "{}: planted {id} has no key — an incomplete planted set is an ERROR",
                keys_path.display()
            )
        })?;
        let chosen = if all_correct {
            letter
        } else {
            letter.wrong_letter()
        };
        answers.push(AnsweredItem {
            item_id: id.clone(),
            chosen,
        });
    }
    if answers.is_empty() {
        return Err("planted item set is empty — an empty planted set is an ERROR".into());
    }
    Ok(ExamAttempt {
        exam_id: exam_id.into(),
        seed,
        bank_hash: bank.bank_hash.clone(),
        answers,
    })
}

fn abs_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|c| c.join(path))
            .unwrap_or_else(|_| path.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed42_asset_list_is_not_empty() {
        assert!(
            !SEED42_ASSETS.is_empty(),
            "empty SEED42_ASSETS is ERROR — demo would require nothing"
        );
        assert!(SEED42_ASSETS.contains(&"data/mock40_seed42.json"));
        assert!(SEED42_ASSETS.contains(&"data/keys_seed42.json"));
        assert!(SEED42_ASSETS.contains(&"data/bank_items_seed42.json"));
    }

    #[test]
    fn wasm_rel_is_the_n4_shipped_path() {
        assert_eq!(WASM_REL, "assets/wasm/cdcp_wasm.wasm");
    }

    #[test]
    fn study_url_prefixes_http() {
        assert_eq!(study_url("127.0.0.1:8766"), "http://127.0.0.1:8766/");
        assert_eq!(study_url("http://127.0.0.1:9000"), "http://127.0.0.1:9000/");
    }

    #[test]
    fn two_minute_path_names_study_and_seed_42() {
        assert!(TWO_MINUTE_PATH.contains("cdcp study"));
        assert!(TWO_MINUTE_PATH.contains("seed 42"));
        assert!(TWO_MINUTE_PATH.contains("wasm"));
    }
}
