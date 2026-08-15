//! Operator surface: `doctor`, `health --robot`, `repair`.
//!
//! These are product commands (bd-engine-not-gate-ar39.4). They are not gates.
//! `repair` rebuilds learner artifacts and MUST NOT re-freeze anything under
//! `goldens/` — UPDATE_GOLDENS stays a human/env gate. A repair verb that
//! silently re-freezes is the B2 hole.

use cdcp_bank::Bank;
use cdcp_learn::join_rel;
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Version of the `health --robot` envelope. Bump only with a consumer change.
pub(crate) const HEALTH_SCHEMA_VERSION: u64 = 1;

/// Top-level keys of the health envelope, in emit order. A test pins these
/// names so a consumer can rely on them; adding or renaming one is a
/// deliberate schema bump.
pub(crate) const HEALTH_ROBOT_FIELDS: &[&str] = &[
    "schema_version",
    "bank_hash",
    "approved_n",
    "manifest_n",
    "unit_count",
    "engine_identities",
    "goldens",
];

/// Doctor checks, compiled in so emptying the list is a RED run rather than a
/// silently vacuous one. An empty tree still RUNS every row and names what is
/// missing — it must never report the way a tree that passed everything does.
pub(crate) const DOCTOR_CHECKS: &[&str] =
    &["bank", "wasm", "goldens", "content.lock", "port", "python3"];

/// Required goldens for the operator surface. Same four files `goldens check`
/// requires; duplicated so this module does not reach into that function
/// (its body is a coupling-ledger region pin).
pub(crate) const OPERATOR_REQUIRED_GOLDENS: &[&str] = &[
    "fixtures/mock40_seed42.json",
    "mock40_seed42_all_correct.sha256",
    "mock40_seed42_all_wrong.sha256",
    "bank_hash.txt",
];

pub(crate) const WASM_REL: &str = "web/assets/wasm/cdcp_wasm.wasm";
pub(crate) const LOCK_REL: &str = "content.lock";
pub(crate) const MANIFEST_REL: &str = "bank/MANIFEST.toml";
pub(crate) const UNITS_REL: &str = "web/data/units_index.json";
pub(crate) const GLOSSARY_REL: &str = "web/data/glossary.json";
pub(crate) const SLUGS_REL: &str = "web/data/module_learn_slugs.js";
pub(crate) const BANK_REL: &str = "bank/items";
pub(crate) const EXPORT_OUT_REL: &str = "web/data";
pub(crate) const DEFAULT_BIND: &str = "127.0.0.1:8766";

const WASM_MAGIC: &[u8] = b"\0asm";

/// Versioned health envelope. Field order IS the contract: serde emits these
/// keys in declaration order, and the test pins the names.
#[derive(Serialize)]
struct HealthEnvelope {
    schema_version: u64,
    bank_hash: String,
    approved_n: u64,
    manifest_n: u64,
    unit_count: u64,
    engine_identities: EngineIdentities,
    goldens: GoldensState,
}

#[derive(Serialize)]
struct EngineIdentities {
    oracle: &'static str,
    subject: &'static str,
}

#[derive(Serialize)]
struct GoldensState {
    state: String,
    required_n: usize,
    present_n: usize,
}

/// Write `bytes` to `path` only when the on-disk content differs.
///
/// Returns `true` if a write happened. Comparing bytes (not mtime, not exit
/// code) is what makes `repair` idempotent: a second run against a matching
/// tree must leave every mtime untouched.
pub(crate) fn write_bytes_if_changed(path: &Path, bytes: &[u8]) -> Result<bool, String> {
    if path.is_file() {
        match fs::read(path) {
            Ok(existing) if existing == bytes => return Ok(false),
            Ok(_) | Err(_) => {}
        }
    }
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
        }
    }
    fs::write(path, bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(true)
}

/// THE HARD RULE: nothing on the operator write path may land under goldens/.
fn refuse_goldens_write(path: &Path) -> Result<(), String> {
    for comp in path.components() {
        if comp.as_os_str() == "goldens" {
            return Err(format!(
                "repair refused to write {} — goldens/ is a human/UPDATE_GOLDENS gate, not a repair target",
                path.display()
            ));
        }
    }
    Ok(())
}

pub(crate) fn resolve_root(explicit: Option<&Path>) -> Result<PathBuf, String> {
    match explicit {
        Some(p) => Ok(p.to_path_buf()),
        None => {
            let cwd = std::env::current_dir().map_err(|e| format!("cwd: {e}"))?;
            cdcp_learn::resolve_engine_root(&cwd).map_err(|e| e.to_string())
        }
    }
}

// ── doctor ────────────────────────────────────────────────────────────────

pub(crate) fn doctor(root: Option<&Path>, bind: &str) -> Result<(), String> {
    if DOCTOR_CHECKS.is_empty() {
        return Err(
            "DOCTOR_CHECKS is empty — a doctor that requires nothing certifies nothing".into(),
        );
    }
    let root = resolve_root(root)?;
    let mut fails: Vec<String> = Vec::new();
    let mut ran = 0usize;

    ran += 1;
    match check_bank(&root) {
        Ok(msg) => println!("ok doctor bank {msg}"),
        Err(e) => {
            println!("FAIL doctor bank: {e}");
            fails.push(e);
        }
    }

    ran += 1;
    match check_wasm(&root) {
        Ok(msg) => println!("ok doctor wasm {msg}"),
        Err(e) => {
            println!("FAIL doctor wasm: {e}");
            fails.push(e);
        }
    }

    ran += 1;
    match check_goldens(&root) {
        Ok(msg) => println!("ok doctor goldens {msg}"),
        Err(e) => {
            println!("FAIL doctor goldens: {e}");
            fails.push(e);
        }
    }

    ran += 1;
    match check_content_lock(&root) {
        Ok(msg) => println!("ok doctor content.lock {msg}"),
        Err(e) => {
            println!("FAIL doctor content.lock: {e}");
            fails.push(e);
        }
    }

    ran += 1;
    match check_port(bind) {
        Ok(msg) => println!("ok doctor port {msg}"),
        Err(e) => {
            println!("FAIL doctor port: {e}");
            fails.push(e);
        }
    }

    ran += 1;
    match check_python3() {
        Ok(msg) => println!("ok doctor python3 {msg}"),
        Err(e) => {
            println!("FAIL doctor python3: {e}");
            fails.push(e);
        }
    }

    if ran == 0 {
        return Err("doctor ran 0 checks — an empty input set is a FAILURE, not a pass".into());
    }
    if ran != DOCTOR_CHECKS.len() {
        return Err(format!(
            "doctor ran {ran} check(s), expected {} — a leg was dropped",
            DOCTOR_CHECKS.len()
        ));
    }
    if !fails.is_empty() {
        return Err(format!(
            "doctor: {}/{} check(s) failed: {}",
            fails.len(),
            ran,
            fails.join("; ")
        ));
    }
    println!("doctor: {ran} check(s) passed");
    Ok(())
}

fn check_bank(root: &Path) -> Result<String, String> {
    let dir = join_rel(root, BANK_REL);
    if !dir.is_dir() {
        return Err(format!(
            "missing {BANK_REL} — an empty tree is an ERROR, not a green 'nothing to check'"
        ));
    }
    let bank =
        Bank::load_dir(&dir).map_err(|e| format!("bank failed to load from {BANK_REL}: {e}"))?;
    let n = bank.items.len();
    if n == 0 {
        return Err("bank loaded 0 items — an empty scan is an ERROR, not a pass".into());
    }
    let approved = bank.items.values().filter(|i| i.is_approved()).count();
    Ok(format!(
        "({n} items, {approved} approved, hash={})",
        bank.bank_hash
    ))
}

fn check_wasm(root: &Path) -> Result<String, String> {
    let path = join_rel(root, WASM_REL);
    if !path.is_file() {
        return Err(format!(
            "missing {WASM_REL} — the browser grade artifact is absent"
        ));
    }
    let bytes = fs::read(&path).map_err(|e| format!("read {WASM_REL}: {e}"))?;
    if bytes.is_empty() {
        return Err(format!(
            "{WASM_REL} is 0 bytes — a present-but-empty artifact is not fresh"
        ));
    }
    if bytes.len() < WASM_MAGIC.len() || !bytes.starts_with(WASM_MAGIC) {
        return Err(format!(
            "{WASM_REL} is not a wasm module (missing \\0asm magic) — not fresh"
        ));
    }
    Ok(format!("({WASM_REL}, {} bytes)", bytes.len()))
}

fn check_goldens(root: &Path) -> Result<String, String> {
    if OPERATOR_REQUIRED_GOLDENS.is_empty() {
        return Err(
            "OPERATOR_REQUIRED_GOLDENS is empty — a check that requires no goldens certifies nothing"
                .into(),
        );
    }
    let dir = join_rel(root, "goldens");
    if !dir.is_dir() {
        return Err(format!(
            "missing goldens/ — an empty tree is an ERROR, not a green 'nothing to check'"
        ));
    }
    let mut missing = Vec::new();
    let mut empty = Vec::new();
    for rel in OPERATOR_REQUIRED_GOLDENS {
        let p = dir.join(rel);
        if !p.is_file() {
            missing.push(rel.to_string());
            continue;
        }
        let meta = fs::metadata(&p).map_err(|e| format!("stat goldens/{rel}: {e}"))?;
        if meta.len() == 0 {
            empty.push(rel.to_string());
        }
    }
    if !missing.is_empty() || !empty.is_empty() {
        let mut parts = Vec::new();
        if !missing.is_empty() {
            parts.push(format!("missing {}", missing.join(", ")));
        }
        if !empty.is_empty() {
            parts.push(format!("empty {}", empty.join(", ")));
        }
        return Err(format!(
            "goldens present-but-unusable ({}) — absence is an ERROR here, not a skipped check",
            parts.join("; ")
        ));
    }
    Ok(format!(
        "({}/{} required)",
        OPERATOR_REQUIRED_GOLDENS.len(),
        OPERATOR_REQUIRED_GOLDENS.len()
    ))
}

fn check_content_lock(root: &Path) -> Result<String, String> {
    let path = join_rel(root, LOCK_REL);
    if !path.is_file() {
        return Err(format!(
            "missing {LOCK_REL} — an empty tree is an ERROR, not a green 'nothing to check'"
        ));
    }
    let text =
        fs::read_to_string(&path).map_err(|e| format!("{LOCK_REL} unreadable (corrupt): {e}"))?;
    if text.trim().is_empty() {
        return Err(format!(
            "{LOCK_REL} is empty — a 0-byte lock pins nothing and is corrupt"
        ));
    }
    let table: toml::Value = text
        .parse()
        .map_err(|e| format!("{LOCK_REL} is corrupt (not valid TOML): {e}"))?;
    let Some(tbl) = table.as_table() else {
        return Err(format!("{LOCK_REL} is corrupt: root is not a table"));
    };
    match tbl.get("schema_version").and_then(|v| v.as_integer()) {
        Some(1) => {}
        other => {
            return Err(format!(
                "{LOCK_REL} is corrupt: schema_version={other:?} (want 1)"
            ))
        }
    }
    let hash = match tbl.get("bank_hash").and_then(|v| v.as_str()) {
        Some(h) if h.len() == 64 && h.chars().all(|c| c.is_ascii_hexdigit()) => h,
        Some(h) => {
            return Err(format!(
                "{LOCK_REL} is corrupt: bank_hash is not 64 hex chars (got {:?})",
                &h.chars().take(16).collect::<String>()
            ))
        }
        None => return Err(format!("{LOCK_REL} is corrupt: missing bank_hash")),
    };
    let knowledge_n = table_len(tbl.get("knowledge"));
    let modules_n = table_len(tbl.get("modules"));
    if knowledge_n == 0 {
        return Err(format!(
            "{LOCK_REL} is corrupt: [knowledge] empty (vacuous pin)"
        ));
    }
    if modules_n == 0 {
        return Err(format!(
            "{LOCK_REL} is corrupt: [modules] empty (vacuous pin)"
        ));
    }
    Ok(format!(
        "(schema=1 bank_hash={}… knowledge={knowledge_n} modules={modules_n})",
        &hash[..12]
    ))
}

fn table_len(v: Option<&toml::Value>) -> usize {
    v.and_then(|x| x.as_table()).map(|t| t.len()).unwrap_or(0)
}

fn check_port(bind: &str) -> Result<String, String> {
    match TcpListener::bind(bind) {
        Ok(listener) => {
            let addr = listener
                .local_addr()
                .map(|a| a.to_string())
                .unwrap_or_else(|_| bind.to_string());
            drop(listener);
            Ok(format!("({addr} bindable)"))
        }
        Err(e) => Err(format!(
            "port {bind} is not bindable — this tree cannot serve: {e}"
        )),
    }
}

fn check_python3() -> Result<String, String> {
    let out = Command::new("python3")
        .arg("-c")
        .arg("import sys; print(f'{sys.version_info[0]}.{sys.version_info[1]}')")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();
    match out {
        Ok(o) if o.status.success() => {
            let ver = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if ver.is_empty() {
                return Err(
                    "python3 present but printed no version — surviving oracles cannot run".into(),
                );
            }
            Ok(format!("({ver})"))
        }
        Ok(o) => Err(format!(
            "python3 exited {} — surviving oracles cannot run",
            o.status
        )),
        Err(e) => Err(format!(
            "python3 not present ({e}) — surviving oracles cannot run"
        )),
    }
}

// ── health ────────────────────────────────────────────────────────────────

pub(crate) fn health(root: Option<&Path>, robot: bool) -> Result<(), String> {
    if HEALTH_ROBOT_FIELDS.is_empty() {
        return Err("HEALTH_ROBOT_FIELDS is empty — an unversioned envelope is an ERROR".into());
    }
    if HEALTH_SCHEMA_VERSION == 0 {
        return Err("HEALTH_SCHEMA_VERSION is 0 — an unversioned envelope is an ERROR".into());
    }
    let root = resolve_root(root)?;
    let envelope = health_envelope(&root)?;
    let line = serde_json::to_string(&envelope)
        .map_err(|e| format!("health --robot envelope unparseable: {e}"))?;
    let value: Value = serde_json::from_str(&line)
        .map_err(|e| format!("health --robot envelope unparseable: {e}"))?;
    validate_envelope(&value)?;
    if robot {
        println!("{line}");
        Ok(())
    } else {
        emit_human(&value)
    }
}

fn health_envelope(root: &Path) -> Result<HealthEnvelope, String> {
    let bank_dir = join_rel(root, BANK_REL);
    if !bank_dir.is_dir() {
        return Err(format!(
            "health: missing {BANK_REL} — zero items is an ERROR, not a pass"
        ));
    }
    let bank =
        Bank::load_dir(&bank_dir).map_err(|e| format!("health: bank failed to load: {e}"))?;
    let n = bank.items.len();
    if n == 0 {
        return Err("health: bank has zero items — an empty scan is an ERROR, not a pass".into());
    }
    let approved_n = bank.items.values().filter(|i| i.is_approved()).count();
    if approved_n == 0 {
        return Err(
            "health: approved_n is 0 — a bank with nothing drawable is an ERROR, not a pass".into(),
        );
    }
    let manifest_n = load_manifest_n(root)?;
    let unit_count = load_unit_count(root)?;
    let (oracle, subject) = cdcp_wasm::engine_identities();
    if oracle.is_empty() || subject.is_empty() {
        return Err("health: engine identities are empty".into());
    }
    let (state, required_n, present_n) = goldens_snapshot(root);

    Ok(HealthEnvelope {
        schema_version: HEALTH_SCHEMA_VERSION,
        bank_hash: bank.bank_hash,
        approved_n: approved_n as u64,
        manifest_n,
        unit_count,
        engine_identities: EngineIdentities { oracle, subject },
        goldens: GoldensState {
            state,
            required_n,
            present_n,
        },
    })
}

fn validate_envelope(v: &Value) -> Result<(), String> {
    let obj = v
        .as_object()
        .ok_or_else(|| "health --robot envelope is not a JSON object — unparseable".to_string())?;
    let ver = obj
        .get("schema_version")
        .and_then(|x| x.as_u64())
        .ok_or_else(|| "health --robot envelope is unversioned — refusing to emit".to_string())?;
    if ver != HEALTH_SCHEMA_VERSION {
        return Err(format!(
            "health --robot schema_version={ver} != {HEALTH_SCHEMA_VERSION}"
        ));
    }
    for key in HEALTH_ROBOT_FIELDS {
        if !obj.contains_key(*key) {
            return Err(format!(
                "health --robot envelope missing field {key} — refusing to emit"
            ));
        }
    }
    let approved = obj.get("approved_n").and_then(|x| x.as_u64()).unwrap_or(0);
    let manifest = obj.get("manifest_n").and_then(|x| x.as_u64()).unwrap_or(0);
    let units = obj.get("unit_count").and_then(|x| x.as_u64()).unwrap_or(0);
    if approved == 0 || manifest == 0 || units == 0 {
        return Err(
            "health: zero items (approved_n, manifest_n, or unit_count) is an ERROR, not a pass"
                .into(),
        );
    }
    Ok(())
}

fn emit_human(envelope: &Value) -> Result<(), String> {
    println!("schema_version={}", envelope["schema_version"]);
    println!("bank_hash={}", envelope["bank_hash"].as_str().unwrap_or(""));
    println!("approved_n={}", envelope["approved_n"]);
    println!("manifest_n={}", envelope["manifest_n"]);
    println!("unit_count={}", envelope["unit_count"]);
    println!(
        "engine_identities.oracle={}",
        envelope["engine_identities"]["oracle"]
            .as_str()
            .unwrap_or("")
    );
    println!(
        "engine_identities.subject={}",
        envelope["engine_identities"]["subject"]
            .as_str()
            .unwrap_or("")
    );
    println!(
        "goldens.state={}",
        envelope["goldens"]["state"].as_str().unwrap_or("")
    );
    println!("goldens.required_n={}", envelope["goldens"]["required_n"]);
    println!("goldens.present_n={}", envelope["goldens"]["present_n"]);
    Ok(())
}

fn load_manifest_n(root: &Path) -> Result<u64, String> {
    let path = join_rel(root, MANIFEST_REL);
    if !path.is_file() {
        return Err(format!(
            "health: missing {MANIFEST_REL} — cannot report manifest_n"
        ));
    }
    let text =
        fs::read_to_string(&path).map_err(|e| format!("health: read {MANIFEST_REL}: {e}"))?;
    let v: toml::Value = text
        .parse()
        .map_err(|e| format!("health: {MANIFEST_REL} unparseable: {e}"))?;
    let n = v
        .get("item_count")
        .and_then(|x| x.as_integer())
        .ok_or_else(|| format!("health: {MANIFEST_REL} missing item_count"))?;
    if n <= 0 {
        return Err("health: manifest_n is 0 — an empty manifest is an ERROR, not a pass".into());
    }
    Ok(n as u64)
}

fn load_unit_count(root: &Path) -> Result<u64, String> {
    let path = join_rel(root, UNITS_REL);
    if !path.is_file() {
        return Err(format!(
            "health: missing {UNITS_REL} — unit_count cannot be 0 by omission"
        ));
    }
    let text = fs::read_to_string(&path).map_err(|e| format!("health: read {UNITS_REL}: {e}"))?;
    let v: Value =
        serde_json::from_str(&text).map_err(|e| format!("health: {UNITS_REL} unparseable: {e}"))?;
    let n = v
        .get("unit_count")
        .and_then(|x| x.as_u64())
        .ok_or_else(|| format!("health: {UNITS_REL} missing unit_count"))?;
    if n == 0 {
        return Err("health: unit_count is 0 — zero units is an ERROR, not a pass".into());
    }
    Ok(n)
}

fn goldens_snapshot(root: &Path) -> (String, usize, usize) {
    let required_n = OPERATOR_REQUIRED_GOLDENS.len();
    let dir = join_rel(root, "goldens");
    if !dir.is_dir() {
        return ("missing".into(), required_n, 0);
    }
    let present_n = OPERATOR_REQUIRED_GOLDENS
        .iter()
        .filter(|rel| dir.join(rel).is_file())
        .count();
    let state = if present_n == 0 {
        "missing"
    } else if present_n < required_n {
        "incomplete"
    } else {
        "present"
    };
    (state.into(), required_n, present_n)
}

// ── repair ────────────────────────────────────────────────────────────────

/// Rebuild units, glossary, learn slugs, and export-web. Never writes goldens/.
///
/// Idempotent: a second run against an already-rebuilt tree writes nothing.
/// Asserted by mtime in tests, not by exit code.
pub(crate) fn repair(root: Option<&Path>, seed: u64) -> Result<(), String> {
    let root = resolve_root(root)?;
    if !join_rel(&root, BANK_REL).is_dir() {
        return Err(format!(
            "repair: missing {BANK_REL} — nothing to rebuild (an empty tree is an ERROR)"
        ));
    }

    let mut wrote = 0usize;
    let mut planned = 0usize;

    planned += 1;
    wrote += repair_learn(&root, LearnTarget::Units)?;
    planned += 1;
    wrote += repair_learn(&root, LearnTarget::Glossary)?;
    planned += 1;
    wrote += repair_learn(&root, LearnTarget::Slugs)?;

    let export_paths = [
        join_rel(&root, EXPORT_OUT_REL).join(format!("mock40_seed{seed}.json")),
        join_rel(&root, EXPORT_OUT_REL).join(format!("keys_seed{seed}.json")),
        join_rel(&root, EXPORT_OUT_REL).join(format!("bank_items_seed{seed}.json")),
    ];
    planned += export_paths.len();
    let before = snapshot_mtimes(&export_paths);
    let bank = join_rel(&root, BANK_REL);
    let out = join_rel(&root, EXPORT_OUT_REL);
    crate::export_web(&bank, seed, &out, None)?;
    for p in &export_paths {
        if !p.is_file() {
            return Err(format!(
                "repair: export-web did not produce {} — a silent no-op is not a rebuild",
                p.display()
            ));
        }
        if file_is_empty(p) {
            return Err(format!(
                "repair: {} is 0 bytes after export-web — refusing to call that a rebuild",
                p.display()
            ));
        }
    }
    let after = snapshot_mtimes(&export_paths);
    let export_wrote = count_mtime_moves(&before, &after);
    wrote += export_wrote;
    if export_wrote == 0 {
        println!("repair: export-web seed={seed} unchanged (3 pack(s))");
    } else {
        println!("repair: export-web seed={seed} wrote {export_wrote} pack(s)");
    }

    if planned == 0 {
        return Err(
            "repair planned 0 writes — a rebuild that targets nothing certifies nothing".into(),
        );
    }
    println!("repair: goldens/ not touched (UPDATE_GOLDENS is a human gate)");
    println!(
        "repair: {wrote} file(s) written, {} unchanged",
        planned - wrote
    );
    Ok(())
}

enum LearnTarget {
    Units,
    Glossary,
    Slugs,
}

fn repair_learn(root: &Path, kind: LearnTarget) -> Result<usize, String> {
    let (label, rel, outcome) = match kind {
        LearnTarget::Units => (
            "units",
            UNITS_REL,
            cdcp_learn::units::evaluate(root).map_err(|e| e.to_string())?,
        ),
        LearnTarget::Glossary => (
            "glossary",
            GLOSSARY_REL,
            cdcp_learn::glossary::evaluate(root).map_err(|e| e.to_string())?,
        ),
        LearnTarget::Slugs => (
            "learn-slugs",
            SLUGS_REL,
            cdcp_learn::slugs::evaluate(root).map_err(|e| e.to_string())?,
        ),
    };
    print!("{}", outcome.stdout);
    if outcome.code != 0 {
        return Err(format!(
            "repair: {label} compile failed (exit {})",
            outcome.code
        ));
    }
    let Some((path, body)) = outcome.artifact else {
        return Err(format!(
            "repair: {label} compile was green but carried no artifact — refusing to call that a rebuild"
        ));
    };
    // evaluate() returns the path it *would* write; prefer the declared rel
    // under this root so a canonicalize inside glossary cannot escape.
    let dest = join_rel(root, rel);
    let dest = if path.ends_with(rel) { path } else { dest };
    if body.is_empty() {
        return Err(format!(
            "repair: {label} artifact is empty — refusing to write {rel}"
        ));
    }
    refuse_goldens_write(&dest)?;
    let changed = write_bytes_if_changed(&dest, body.as_bytes())?;
    if changed {
        println!("repair: wrote {rel}");
        Ok(1)
    } else {
        println!("repair: {rel} unchanged");
        Ok(0)
    }
}

fn file_is_empty(path: &Path) -> bool {
    fs::metadata(path).map(|m| m.len() == 0).unwrap_or(true)
}

fn snapshot_mtimes(paths: &[PathBuf]) -> Vec<Option<std::time::SystemTime>> {
    paths
        .iter()
        .map(|p| fs::metadata(p).and_then(|m| m.modified()).ok())
        .collect()
}

fn count_mtime_moves(
    before: &[Option<std::time::SystemTime>],
    after: &[Option<std::time::SystemTime>],
) -> usize {
    before
        .iter()
        .zip(after.iter())
        .filter(|(b, a)| match (b, a) {
            (Some(b), Some(a)) => a != b,
            (None, Some(_)) => true,
            _ => false,
        })
        .count()
}
