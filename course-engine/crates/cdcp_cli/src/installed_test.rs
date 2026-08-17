//! `cdcp test` — installed-tree smoke (bd-installability-sm4g.14).
//!
//! One command on the installed home: learner-pack shape (`n=40`, no leaked
//! `correct`), shipped wasm `\0asm` + non-empty, mock seed-42 pack/bank/keys
//! present. An empty suite is ERROR. Resolves via `cdcp_root` (`--root` /
//! `CDCP_HOME` / XDG / `~/.local/share/cdcp` / cwd walk), same as serve/study.

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

/// Compiled-in suite. Emptying this is a product defect: a test that ships
/// zero cases certifies nothing. Named so a dropped row is a dropped check.
pub(crate) const SUITE: &[&str] = &[
    "learner-pack",
    "wasm",
    "seed42-pack",
    "seed42-bank",
    "seed42-keys",
];

const _: () = assert!(!SUITE.is_empty(), "empty SUITE certifies nothing");
const _: () = assert!(
    SUITE.len() >= 5,
    "SUITE shrank below the planted floor (pack + wasm + three seed-42 assets)"
);

const WASM_REL: &str = "assets/wasm/cdcp_wasm.wasm";
const PACK_REL: &str = "data/mock40_seed42.json";
const BANK_REL: &str = "data/bank_items_seed42.json";
const KEYS_REL: &str = "data/keys_seed42.json";
const WASM_MAGIC: &[u8] = b"\0asm";

/// Resolve the installed home and run every compiled-in case.
pub(crate) fn run(explicit: Option<&Path>) -> Result<(), String> {
    if SUITE.is_empty() {
        return Err("SUITE is empty — a test that ships zero cases is RED".into());
    }
    let resolved = cdcp_root::resolve_from_env(explicit).map_err(|e| e.to_string())?;
    println!("cdcp: {}", resolved.announce());
    let web = resolved.web_dir();

    let mut fails: Vec<String> = Vec::new();
    let mut ran = 0usize;
    for name in SUITE {
        ran += 1;
        match run_case(name, web) {
            Ok(detail) => println!("ok test {name} {detail}"),
            Err(e) => {
                println!("FAIL test {name}: {e}");
                fails.push(format!("{name}: {e}"));
            }
        }
    }
    if ran == 0 {
        return Err("test ran 0 cases — an empty suite is ERROR, not a pass".into());
    }
    if ran != SUITE.len() {
        return Err(format!(
            "test ran {ran} case(s), expected {} — a leg was dropped",
            SUITE.len()
        ));
    }
    if !fails.is_empty() {
        return Err(format!(
            "test: {}/{} case(s) failed: {}",
            fails.len(),
            ran,
            fails.join("; ")
        ));
    }
    println!("test: {ran} case(s) passed");
    Ok(())
}

fn run_case(name: &str, web: &Path) -> Result<String, String> {
    match name {
        "learner-pack" => check_learner_pack(web),
        "wasm" => check_wasm(web),
        "seed42-pack" => check_seed42_pack(web),
        "seed42-bank" => check_seed42_bank(web),
        "seed42-keys" => check_seed42_keys(web),
        other => Err(format!(
            "unknown suite case {other} — a name in SUITE with no implementation is a dropped leg"
        )),
    }
}

fn check_learner_pack(web: &Path) -> Result<String, String> {
    let path = shown(&web.join(PACK_REL));
    let out = cdcp_learn::learner_pack::check_path(&path);
    if out.code == 0 {
        Ok(format!(
            "(n={} items={}, no leaked correct, {})",
            cdcp_learn::learner_pack::EXPECTED_N_ITEMS,
            cdcp_learn::learner_pack::EXPECTED_N_ITEMS,
            path.display()
        ))
    } else {
        Err(out.stdout.trim().to_string())
    }
}

fn check_wasm(web: &Path) -> Result<String, String> {
    let path = web.join(WASM_REL);
    let label = shown(&path);
    if !path.is_file() {
        return Err(format!(
            "missing {} — the browser grade artifact is absent",
            label.display()
        ));
    }
    let bytes = fs::read(&path).map_err(|e| format!("read {}: {e}", label.display()))?;
    if bytes.is_empty() {
        return Err(format!(
            "{} is 0 bytes — a present-but-empty artifact is not a wasm module",
            label.display()
        ));
    }
    if bytes.len() < WASM_MAGIC.len() || !bytes.starts_with(WASM_MAGIC) {
        return Err(format!(
            "{} is not a wasm module (missing \\0asm magic)",
            label.display()
        ));
    }
    Ok(format!("({}, {} bytes)", label.display(), bytes.len()))
}

fn check_seed42_pack(web: &Path) -> Result<String, String> {
    let path = web.join(PACK_REL);
    let v = read_json(&path, "seed-42 pack")?;
    if !v.is_object() {
        return Err(format!(
            "{} is not a JSON object — seed-42 pack is unusable",
            shown(&path).display()
        ));
    }
    Ok(format!("({})", shown(&path).display()))
}

fn check_seed42_bank(web: &Path) -> Result<String, String> {
    let path = web.join(BANK_REL);
    let v = read_json(&path, "seed-42 bank")?;
    let Some(items) = v.as_array() else {
        return Err(format!(
            "{} is not a JSON array — seed-42 bank is unusable",
            shown(&path).display()
        ));
    };
    if items.is_empty() {
        return Err(format!(
            "{} items=0 — an empty seed-42 bank is ERROR, not a pass",
            shown(&path).display()
        ));
    }
    Ok(format!(
        "({} items, {})",
        items.len(),
        shown(&path).display()
    ))
}

fn check_seed42_keys(web: &Path) -> Result<String, String> {
    let path = web.join(KEYS_REL);
    let v = read_json(&path, "seed-42 keys")?;
    let Some(obj) = v.as_object() else {
        return Err(format!(
            "{} is not a JSON object — seed-42 keys is unusable",
            shown(&path).display()
        ));
    };
    match obj.get("keys") {
        None => Err(format!("{}: keys missing", shown(&path).display())),
        Some(Value::Array(keys)) if keys.is_empty() => Err(format!(
            "{} keys=0 — an empty seed-42 key list is ERROR, not a pass",
            shown(&path).display()
        )),
        Some(Value::Array(keys)) => {
            Ok(format!("({} keys, {})", keys.len(), shown(&path).display()))
        }
        Some(_) => Err(format!("{}: keys is not an array", shown(&path).display())),
    }
}

fn read_json(path: &Path, label: &str) -> Result<Value, String> {
    let shown = shown(path);
    if !path.is_file() {
        return Err(format!(
            "missing {} — {label} is absent from the installed tree",
            shown.display()
        ));
    }
    let bytes = fs::read(path).map_err(|e| format!("read {}: {e}", shown.display()))?;
    if bytes.is_empty() {
        return Err(format!(
            "{} is 0 bytes — a present-but-empty {label} pins nothing",
            shown.display()
        ));
    }
    serde_json::from_slice(&bytes).map_err(|e| {
        format!(
            "{} is not JSON — a present-but-unusable {label} is RED: {e}",
            shown.display()
        )
    })
}

fn shown(path: &Path) -> PathBuf {
    if let Ok(c) = path.canonicalize() {
        return c;
    }
    // File is missing: still name the real parent so a symlink temp dir
    // (`/var` → `/private/var` on macOS) is reported as an absolute path.
    if let Some(parent) = path.parent() {
        if let (Ok(parent), Some(name)) = (parent.canonicalize(), path.file_name()) {
            return parent.join(name);
        }
    }
    if path.is_absolute() {
        return path.to_path_buf();
    }
    std::env::current_dir()
        .map(|cwd| cwd.join(path))
        .unwrap_or_else(|_| path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static CASE_HITS: AtomicUsize = AtomicUsize::new(0);

    const CASES: &[&str] = &[
        "suite_list_is_compiled_in_and_nonempty",
        "production_does_not_spawn",
        "wasm_magic_is_zero_asm",
        "wasm_deleted_names_absolute_path",
        "zero_byte_wasm_is_red",
        "empty_bank_cannot_pass",
        "empty_keys_cannot_pass",
        "unknown_case_is_dropped_leg",
    ];

    fn hit(name: &str) {
        assert!(
            CASES.contains(&name),
            "test {name} is not in CASES — the anti-vacuous list drifted"
        );
        CASE_HITS.fetch_add(1, Ordering::SeqCst);
    }

    fn scratch(tag: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!(
            "cdcp-n14-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(&p).unwrap();
        p
    }

    fn write(path: &Path, bytes: &[u8]) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, bytes).unwrap();
    }

    #[test]
    fn cases_list_is_not_empty() {
        assert!(
            !CASES.is_empty(),
            "empty unit-test case list is ERROR — nothing was checked"
        );
        assert!(
            !SUITE.is_empty(),
            "empty SUITE is ERROR — a test that ships zero cases is RED"
        );
        assert!(SUITE.len() >= 5, "SUITE shrank: {}", SUITE.len());
        for name in [
            "learner-pack",
            "wasm",
            "seed42-pack",
            "seed42-bank",
            "seed42-keys",
        ] {
            assert!(SUITE.contains(&name), "SUITE missing {name}: {SUITE:?}");
        }
    }

    #[test]
    fn suite_list_is_compiled_in_and_nonempty() {
        hit("suite_list_is_compiled_in_and_nonempty");
        let src = include_str!("installed_test.rs");
        let prod = src
            .split("#[cfg(test)]")
            .next()
            .expect("prod precedes tests");
        assert!(
            prod.contains("const SUITE: &[&str] = &["),
            "SUITE must be a compiled-in list, not discovered at runtime"
        );
        assert!(
            !prod.contains("std::process"),
            "installed-tree smoke must not spawn a process"
        );
    }

    #[test]
    fn production_does_not_spawn() {
        hit("production_does_not_spawn");
        let src = include_str!("installed_test.rs");
        let prod = src
            .split("#[cfg(test)]")
            .next()
            .expect("prod precedes tests");
        for needle in [
            "Command::new",
            "std::process",
            "python3",
            "goldens",
            "check.sh",
        ] {
            assert!(
                !prod.contains(needle),
                "production installed_test.rs mentions {needle} — the smoke must not reach authoring tools"
            );
        }
        // The word "cargo" must not appear as an invocation. A comment that
        // merely says we do not call it would still trip this; keep prod silent.
        assert!(
            !prod.contains("cargo"),
            "production installed_test.rs mentions cargo"
        );
    }

    #[test]
    fn wasm_magic_is_zero_asm() {
        hit("wasm_magic_is_zero_asm");
        assert_eq!(WASM_MAGIC, b"\0asm");
        let dir = scratch("magic");
        write(&dir.join(WASM_REL), b"\0asm\x01\x00\x00\x00rest");
        let out = check_wasm(&dir).unwrap();
        assert!(out.contains("bytes"), "{out}");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn wasm_deleted_names_absolute_path() {
        hit("wasm_deleted_names_absolute_path");
        let dir = scratch("nowasm");
        fs::create_dir_all(dir.join("assets/wasm")).unwrap();
        let wasm = dir.join(WASM_REL);
        let err = check_wasm(&dir).unwrap_err();
        let abs = shown(&wasm);
        assert!(
            err.contains(&abs.display().to_string()),
            "missing wasm must name the absolute path {}, got: {err}",
            abs.display()
        );
        assert!(abs.is_absolute(), "shown path must be absolute: {abs:?}");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn zero_byte_wasm_is_red() {
        hit("zero_byte_wasm_is_red");
        let dir = scratch("empty-wasm");
        write(&dir.join(WASM_REL), b"");
        let err = check_wasm(&dir).unwrap_err();
        assert!(err.contains("0 bytes"), "{err}");
        write(&dir.join(WASM_REL), b"not-a-module");
        let err = check_wasm(&dir).unwrap_err();
        assert!(
            err.contains("\\0asm") || err.contains("not a wasm"),
            "{err}"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn empty_bank_cannot_pass() {
        hit("empty_bank_cannot_pass");
        let dir = scratch("empty-bank");
        write(&dir.join(BANK_REL), b"[]");
        let err = check_seed42_bank(&dir).unwrap_err();
        assert!(err.contains("items=0"), "{err}");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn empty_keys_cannot_pass() {
        hit("empty_keys_cannot_pass");
        let dir = scratch("empty-keys");
        write(&dir.join(KEYS_REL), br#"{"keys":[]}"#);
        let err = check_seed42_keys(&dir).unwrap_err();
        assert!(err.contains("keys=0"), "{err}");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn unknown_case_is_dropped_leg() {
        hit("unknown_case_is_dropped_leg");
        let err = run_case("not-a-real-case", Path::new("/tmp")).unwrap_err();
        assert!(err.contains("dropped leg"), "{err}");
    }
}
