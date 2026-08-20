//! Operator surface: `doctor`, `health --robot`, `repair`.
//!
//! These are product commands (bd-engine-not-gate-ar39.4). They are not gates.
//! Learner `repair` (bd-installability-sm4g.12) is receipt-driven: default
//! `--dry-run` hashes every `files[]` entry in `install-receipt.json` and
//! writes nothing. `--apply` is idempotent on a matching receipt and REFUSES
//! to invent bytes when a hash drifted (an installed learner has no `bank/`
//! to export-web from). Missing receipt is refuse, not a path guess.
//! `goldens/` is never a repair target — UPDATE_GOLDENS stays a human/env
//! gate. A repair verb that silently re-freezes is the B2 hole.
//!
//! Learner `doctor` (PLAN-N W12, bd-installability-sm4g.11) probes the
//! INSTALLED layer only: `web/`, shipped wasm `\0asm`, install receipt if
//! present, bindable port. Authoring probes (bank / goldens / content.lock /
//! python3) stay behind `CDCP_DEV=1`. `CDCP_DEV=1 repair --apply` MAY still
//! rebuild units/glossary/slugs/export-web from `bank/` on a source checkout
//! that has no receipt.

use cdcp_bank::Bank;
use cdcp_core::sha256_hex;
use cdcp_learn::join_rel;
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Version of the `health --robot` envelope. Bump only with a consumer change.
/// v3 lists installed facts only (web / wasm / receipt / identities / attempts).
/// bank_hash / approved_n / manifest_n / goldens / unit_count were v2 authoring
/// facts and made every bundle-only tree RED-by-construction.
pub(crate) const HEALTH_SCHEMA_VERSION: u64 = 3;

/// Top-level keys of the health envelope, in emit order. A test pins these
/// names so a consumer can rely on them; adding or renaming one is a
/// deliberate schema bump.
pub(crate) const HEALTH_ROBOT_FIELDS: &[&str] = &[
    "schema_version",
    "web",
    "wasm",
    "receipt",
    "engine_identities",
    "attempts_store",
];

/// Version of `cdcp doctor --json`. Bump when the probe object shape changes.
pub(crate) const DOCTOR_SCHEMA_VERSION: u64 = 1;

/// Learner-path doctor probes. Empty is compile-fail AND runtime ERROR.
/// An empty tree still RUNS every row and names what is missing — it must
/// never report the way a tree that passed everything does.
pub(crate) const DOCTOR_CHECKS: &[&str] = &["web", "wasm", "receipt", "port"];

/// Authoring probes. Reached only when `CDCP_DEV=1`. Empty is ERROR on that path.
pub(crate) const DOCTOR_AUTHORING_CHECKS: &[&str] = &["bank", "goldens", "content.lock", "python3"];

const _: () = assert!(
    !DOCTOR_CHECKS.is_empty(),
    "empty DOCTOR_CHECKS certifies nothing"
);
const _: () = assert!(
    !DOCTOR_AUTHORING_CHECKS.is_empty(),
    "empty DOCTOR_AUTHORING_CHECKS certifies nothing"
);
const _: () = assert!(
    !HEALTH_ROBOT_FIELDS.is_empty(),
    "empty HEALTH_ROBOT_FIELDS is an unversioned envelope"
);
const _: () = assert!(
    HEALTH_SCHEMA_VERSION > 0,
    "HEALTH_SCHEMA_VERSION 0 is unversioned"
);
const _: () = assert!(
    DOCTOR_SCHEMA_VERSION > 0,
    "DOCTOR_SCHEMA_VERSION 0 is unversioned"
);
const _: () = assert!(
    !REPAIR_JSON_FIELDS.is_empty(),
    "empty REPAIR_JSON_FIELDS is an unversioned envelope"
);
const _: () = assert!(
    REPAIR_SCHEMA_VERSION > 0,
    "REPAIR_SCHEMA_VERSION 0 is unversioned"
);

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
pub(crate) const UNITS_REL: &str = "web/data/units_index.json";
pub(crate) const GLOSSARY_REL: &str = "web/data/glossary.json";
pub(crate) const SLUGS_REL: &str = "web/data/module_learn_slugs.js";
pub(crate) const BANK_REL: &str = "bank/items";
pub(crate) const EXPORT_OUT_REL: &str = "web/data";
pub(crate) const RECEIPT_REL: &str = "install-receipt.json";
pub(crate) const DEFAULT_BIND: &str = "127.0.0.1:8766";
pub(crate) const EPHEMERAL_BIND: &str = "127.0.0.1:0";
pub(crate) const DEV_ENV: &str = "CDCP_DEV";

/// Version of `cdcp repair --json`. Bump when the planned/actual shape changes.
pub(crate) const REPAIR_SCHEMA_VERSION: u64 = 1;

/// Top-level keys of the repair envelope. A test pins these names so a
/// consumer can rely on them; adding or renaming one is a schema bump.
pub(crate) const REPAIR_JSON_FIELDS: &[&str] = &[
    "schema_version",
    "mode",
    "ok",
    "receipt",
    "planned",
    "actual",
    "planned_restore",
    "actual_restore",
];

const WASM_MAGIC: &[u8] = b"\0asm";

/// Versioned health envelope. Field order IS the contract: serde emits these
/// keys in declaration order, and the test pins the names.
#[derive(Serialize)]
struct HealthEnvelope {
    schema_version: u64,
    web: PathFact,
    wasm: WasmFact,
    receipt: PathFact,
    engine_identities: EngineIdentities,
    attempts_store: AttemptsStoreState,
}

#[derive(Serialize)]
struct PathFact {
    state: String,
    path: String,
}

#[derive(Serialize)]
struct WasmFact {
    state: String,
    path: String,
    bytes: u64,
}

#[derive(Serialize)]
struct DoctorEnvelope {
    schema_version: u64,
    ok: bool,
    layer: &'static str,
    passed: usize,
    failed: usize,
    probes: Vec<DoctorProbe>,
}

#[derive(Serialize)]
struct DoctorProbe {
    name: &'static str,
    ok: bool,
    detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
}

/// Mention of the local-first attempt store. Absence is the default
/// (export stays OFF). `n == 0` here is not a health ERROR — empty is
/// ERROR on list/export, not on mention.
#[derive(Serialize)]
struct AttemptsStoreState {
    state: String,
    path: String,
    n: u64,
    export_policy: String,
}

#[derive(Serialize)]
struct EngineIdentities {
    oracle: &'static str,
    subject: &'static str,
}

/// Versioned `cdcp repair --json` envelope. Field order IS the contract.
#[derive(Serialize)]
struct RepairEnvelope {
    schema_version: u64,
    mode: &'static str,
    ok: bool,
    receipt: String,
    planned: Vec<RepairAction>,
    actual: Vec<RepairAction>,
    planned_restore: usize,
    actual_restore: usize,
}

#[derive(Serialize, Clone)]
struct RepairAction {
    path: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    expected_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    actual_sha256: Option<String>,
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
        // Tests and operators pass `--root` at trees that are not yet a
        // classified bundle (empty dirs, planted known-bad). Do not fail
        // closed at resolve — the probes name the missing installed files.
        Some(p) => Ok(p.to_path_buf()),
        None => {
            let resolved = cdcp_root::resolve_from_env(None).map_err(|e| e.to_string())?;
            Ok(resolved.path)
        }
    }
}

fn authoring_mode() -> bool {
    std::env::var(DEV_ENV).ok().as_deref() == Some("1")
}

fn json_flag_from_args() -> bool {
    // clap keeps `--json` on argv even after parsing. Reading argv (not a
    // third parameter) keeps the doctor() signature stable while N.2 edits
    // main.rs study/serve dispatch.
    std::env::args().any(|a| a == "--json")
}

/// Directory the learner bundle lives in.
///
/// `--root` may be the install home (`web/index.html`) or the web directory
/// itself (`index.html`). A missing web/ still returns the *expected* path so
/// RED names it.
fn installed_web_dir(root: &Path) -> PathBuf {
    let nested = join_rel(root, "web");
    if nested.is_dir() {
        nested
    } else if root.join("index.html").is_file() {
        root.to_path_buf()
    } else {
        nested
    }
}

fn wasm_path(root: &Path) -> PathBuf {
    if root.join("index.html").is_file() && !join_rel(root, "web").is_dir() {
        root.join("assets/wasm/cdcp_wasm.wasm")
    } else {
        join_rel(root, WASM_REL)
    }
}

fn receipt_path(root: &Path) -> PathBuf {
    if root.join("index.html").is_file() && !join_rel(root, "web").is_dir() {
        match root.parent() {
            Some(p) if !p.as_os_str().is_empty() => p.join(RECEIPT_REL),
            _ => root.join(RECEIPT_REL),
        }
    } else {
        root.join(RECEIPT_REL)
    }
}

fn abs_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .unwrap_or_else(|_| path.to_path_buf())
    }
}

// ── doctor ────────────────────────────────────────────────────────────────

pub(crate) fn doctor(root: Option<&Path>, bind: &str) -> Result<(), String> {
    doctor_run(root, bind, json_flag_from_args())
}

fn doctor_run(root: Option<&Path>, bind: &str, json: bool) -> Result<(), String> {
    if DOCTOR_CHECKS.is_empty() {
        return Err(
            "DOCTOR_CHECKS is empty — a doctor that requires nothing certifies nothing".into(),
        );
    }
    let authoring = authoring_mode();
    if authoring && DOCTOR_AUTHORING_CHECKS.is_empty() {
        return Err(
            "DOCTOR_AUTHORING_CHECKS is empty — a doctor that requires nothing certifies nothing"
                .into(),
        );
    }
    let root = resolve_doctor_root(root, authoring, json)?;
    let mut names: Vec<&'static str> = DOCTOR_CHECKS.to_vec();
    if authoring {
        names.extend_from_slice(DOCTOR_AUTHORING_CHECKS);
    }

    let mut probes: Vec<DoctorProbe> = Vec::new();
    let mut fails: Vec<String> = Vec::new();

    for name in names.iter().copied() {
        let path = probe_path(name, &root);
        match run_probe(name, &root, bind) {
            Ok(detail) => {
                if !json {
                    println!("ok doctor {name} {detail}");
                }
                probes.push(DoctorProbe {
                    name,
                    ok: true,
                    detail,
                    path,
                });
            }
            Err(e) => {
                if !json {
                    println!("FAIL doctor {name}: {e}");
                }
                fails.push(e.clone());
                probes.push(DoctorProbe {
                    name,
                    ok: false,
                    detail: e,
                    path,
                });
            }
        }
    }

    // Mention, not a compiled-in check: the store is opt-in. Absence is
    // the default, not a defect. Empty is ERROR on list/export, not here.
    if !json {
        println!("{}", crate::attempts::doctor_line(&root));
    }

    let ran = probes.len();
    if ran == 0 {
        return Err("doctor ran 0 checks — an empty input set is a FAILURE, not a pass".into());
    }
    if ran != names.len() {
        return Err(format!(
            "doctor ran {ran} check(s), expected {} — a leg was dropped",
            names.len()
        ));
    }

    let ok = fails.is_empty();
    if json {
        let envelope = DoctorEnvelope {
            schema_version: DOCTOR_SCHEMA_VERSION,
            ok,
            layer: if authoring { "authoring" } else { "installed" },
            passed: ran - fails.len(),
            failed: fails.len(),
            probes,
        };
        let line = serde_json::to_string(&envelope)
            .map_err(|e| format!("doctor --json envelope unparseable: {e}"))?;
        println!("{line}");
    }

    if !ok {
        return Err(format!(
            "doctor: {}/{} check(s) failed: {}",
            fails.len(),
            ran,
            fails.join("; ")
        ));
    }
    if !json {
        println!("doctor: {ran} check(s) passed");
    }
    Ok(())
}

/// Learner doctor must certify the installed layer, never a nearby source
/// checkout discovered by the cwd walk. Authoring mode is the explicit escape
/// hatch for source-tree probes; an implicit source fallback would turn a
/// missing installed bundle into a confident green result.
fn resolve_doctor_root(
    explicit: Option<&Path>,
    authoring: bool,
    json: bool,
) -> Result<PathBuf, String> {
    let Some(_) = explicit else {
        let resolved = cdcp_root::resolve_from_env(None).map_err(|e| e.to_string())?;
        if !json {
            println!("cdcp doctor: {}", resolved.announce());
        }
        if resolved.kind == cdcp_root::RootKind::SourceCheckout && !authoring {
            return Err(format!(
                "doctor resolved source-checkout root {} via {}; installed bundle root was not found",
                resolved.path.display(),
                resolved.via.as_str()
            ));
        }
        return Ok(resolved.path);
    };
    Ok(explicit.expect("explicit root checked above").to_path_buf())
}

fn run_probe(name: &str, root: &Path, bind: &str) -> Result<String, String> {
    match name {
        "web" => check_web(root),
        "wasm" => check_wasm(root),
        "receipt" => check_receipt(root),
        "port" => check_port(bind),
        "bank" => check_bank(root),
        "goldens" => check_goldens(root),
        "content.lock" => check_content_lock(root),
        "python3" => check_python3(),
        other => Err(format!(
            "unknown doctor probe {other} — a name in the list that has no implementation is a dropped leg"
        )),
    }
}

fn probe_path(name: &str, root: &Path) -> Option<String> {
    match name {
        "web" => Some(abs_path(&installed_web_dir(root)).display().to_string()),
        "wasm" => Some(abs_path(&wasm_path(root)).display().to_string()),
        "receipt" => Some(abs_path(&receipt_path(root)).display().to_string()),
        _ => None,
    }
}

fn check_web(root: &Path) -> Result<String, String> {
    let web = installed_web_dir(root);
    let shown = abs_path(&web);
    if !web.is_dir() {
        return Err(format!(
            "missing {} — an installed tree without web/ cannot serve",
            shown.display()
        ));
    }
    let index = web.join("index.html");
    if !index.is_file() {
        return Err(format!(
            "missing {} — web/ is not a serveable bundle",
            abs_path(&index).display()
        ));
    }
    if file_is_empty(&index) {
        return Err(format!(
            "{} is 0 bytes — a present-but-empty index is not a bundle",
            abs_path(&index).display()
        ));
    }
    Ok(format!("({})", shown.display()))
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
    let path = wasm_path(root);
    let shown = abs_path(&path);
    let label = shown.display();
    if !path.is_file() {
        return Err(format!(
            "missing {label} — the browser grade artifact is absent"
        ));
    }
    let bytes = fs::read(&path).map_err(|e| format!("read {label}: {e}"))?;
    if bytes.is_empty() {
        return Err(format!(
            "{label} is 0 bytes — a present-but-empty artifact is not a wasm module"
        ));
    }
    if bytes.len() < WASM_MAGIC.len() || !bytes.starts_with(WASM_MAGIC) {
        return Err(format!(
            "{label} is not a wasm module (missing \\0asm magic)"
        ));
    }
    Ok(format!("({label}, {} bytes)", bytes.len()))
}

fn check_receipt(root: &Path) -> Result<String, String> {
    // Absence is not a defect: a source checkout or a copied web/ tree has
    // no install.sh receipt. A present-but-unusable receipt is RED.
    let path = receipt_path(root);
    let shown = abs_path(&path);
    if !path.is_file() {
        return Ok(format!("(absent {})", shown.display()));
    }
    let bytes = fs::read(&path).map_err(|e| format!("read {}: {e}", shown.display()))?;
    if bytes.is_empty() {
        return Err(format!(
            "{} is 0 bytes — a present-but-empty receipt pins nothing",
            shown.display()
        ));
    }
    let v: Value = serde_json::from_slice(&bytes).map_err(|e| {
        format!(
            "{} is not JSON — a present-but-unusable receipt is RED: {e}",
            shown.display()
        )
    })?;
    if !v.is_object() {
        return Err(format!(
            "{} is not a JSON object — a present-but-unusable receipt is RED",
            shown.display()
        ));
    }
    Ok(format!("({}, {} bytes)", shown.display(), bytes.len()))
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
        return Err(
            "missing goldens/ — an empty tree is an ERROR, not a green 'nothing to check'".into(),
        );
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
                h.chars().take(16).collect::<String>()
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
        Err(e) if bind == DEFAULT_BIND => {
            // Occupied default is not a missing wasm. Prove *some* port is
            // bindable; do not report the tool broken because 8766 is busy.
            match TcpListener::bind(EPHEMERAL_BIND) {
                Ok(listener) => {
                    let addr = listener
                        .local_addr()
                        .map(|a| a.to_string())
                        .unwrap_or_else(|_| EPHEMERAL_BIND.to_string());
                    drop(listener);
                    Ok(format!(
                        "({DEFAULT_BIND} occupied, ephemeral {addr} bindable)"
                    ))
                }
                Err(e2) => Err(format!(
                    "no bindable port (default {DEFAULT_BIND}: {e}; ephemeral: {e2})"
                )),
            }
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
    let web = installed_web_dir(root);
    if !web.is_dir() {
        return Err(format!(
            "health: missing {} — bundle-only health requires the installed web/",
            abs_path(&web).display()
        ));
    }
    let (oracle, subject) = cdcp_wasm::engine_identities();
    if oracle.is_empty() || subject.is_empty() {
        return Err("health: engine identities are empty".into());
    }
    let attempts = crate::attempts::mention(root);
    Ok(HealthEnvelope {
        schema_version: HEALTH_SCHEMA_VERSION,
        web: path_fact(&web, web.is_dir()),
        wasm: wasm_fact(root),
        receipt: receipt_fact(root),
        engine_identities: EngineIdentities { oracle, subject },
        attempts_store: AttemptsStoreState {
            state: attempts.state.to_string(),
            path: attempts.rel_path.to_string(),
            n: attempts.n,
            export_policy: attempts.export_policy.to_string(),
        },
    })
}

fn path_fact(path: &Path, present: bool) -> PathFact {
    PathFact {
        state: if present {
            "present".into()
        } else {
            "missing".into()
        },
        path: abs_path(path).display().to_string(),
    }
}

fn wasm_fact(root: &Path) -> WasmFact {
    let path = wasm_path(root);
    let shown = abs_path(&path).display().to_string();
    match fs::read(&path) {
        Ok(bytes) if bytes.is_empty() => WasmFact {
            state: "empty".into(),
            path: shown,
            bytes: 0,
        },
        Ok(bytes) if bytes.starts_with(WASM_MAGIC) => WasmFact {
            state: "present".into(),
            path: shown,
            bytes: bytes.len() as u64,
        },
        Ok(bytes) => WasmFact {
            state: "not_wasm".into(),
            path: shown,
            bytes: bytes.len() as u64,
        },
        Err(_) if !path.is_file() => WasmFact {
            state: "missing".into(),
            path: shown,
            bytes: 0,
        },
        Err(_) => WasmFact {
            state: "unreadable".into(),
            path: shown,
            bytes: 0,
        },
    }
}

fn receipt_fact(root: &Path) -> PathFact {
    let path = receipt_path(root);
    let shown = abs_path(&path).display().to_string();
    if !path.is_file() {
        return PathFact {
            state: "absent".into(),
            path: shown,
        };
    }
    match fs::read(&path) {
        Ok(bytes) if bytes.is_empty() => PathFact {
            state: "empty".into(),
            path: shown,
        },
        Ok(bytes) => match serde_json::from_slice::<Value>(&bytes) {
            Ok(v) if v.is_object() => PathFact {
                state: "present".into(),
                path: shown,
            },
            _ => PathFact {
                state: "corrupt".into(),
                path: shown,
            },
        },
        Err(_) => PathFact {
            state: "unreadable".into(),
            path: shown,
        },
    }
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
    Ok(())
}

fn emit_human(envelope: &Value) -> Result<(), String> {
    println!("schema_version={}", envelope["schema_version"]);
    println!(
        "web.state={}",
        envelope["web"]["state"].as_str().unwrap_or("")
    );
    println!(
        "web.path={}",
        envelope["web"]["path"].as_str().unwrap_or("")
    );
    println!(
        "wasm.state={}",
        envelope["wasm"]["state"].as_str().unwrap_or("")
    );
    println!(
        "wasm.path={}",
        envelope["wasm"]["path"].as_str().unwrap_or("")
    );
    println!("wasm.bytes={}", envelope["wasm"]["bytes"]);
    println!(
        "receipt.state={}",
        envelope["receipt"]["state"].as_str().unwrap_or("")
    );
    println!(
        "receipt.path={}",
        envelope["receipt"]["path"].as_str().unwrap_or("")
    );
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
        "attempts_store.state={}",
        envelope["attempts_store"]["state"].as_str().unwrap_or("")
    );
    println!(
        "attempts_store.path={}",
        envelope["attempts_store"]["path"].as_str().unwrap_or("")
    );
    println!("attempts_store.n={}", envelope["attempts_store"]["n"]);
    println!(
        "attempts_store.export_policy={}",
        envelope["attempts_store"]["export_policy"]
            .as_str()
            .unwrap_or("")
    );
    Ok(())
}

// ── repair ────────────────────────────────────────────────────────────────

/// Receipt-driven bundle integrity. Default is dry-run (writes nothing).
///
/// Learner path (no `CDCP_DEV`): read `install-receipt.json`, hash each
/// `files[]` path, report ok / missing / hash-mismatch. Never writes. Never
/// calls export-web. Never touches `goldens/`. Missing receipt is REFUSE.
/// `--apply` on a matching receipt is a no-op (`planned_restore=0`).
/// `--apply` on drift refuses to invent content and names the paths.
///
/// Authoring (`CDCP_DEV=1 --apply` on a source checkout with `bank/` and no
/// receipt) may still rebuild units/glossary/slugs/export-web. That path is
/// not the installed product.
pub(crate) fn repair(
    root: Option<&Path>,
    apply: bool,
    json: bool,
    seed: u64,
) -> Result<(), String> {
    if REPAIR_JSON_FIELDS.is_empty() {
        return Err("REPAIR_JSON_FIELDS is empty — an unversioned envelope is an ERROR".into());
    }
    if REPAIR_SCHEMA_VERSION == 0 {
        return Err("REPAIR_SCHEMA_VERSION is 0 — an unversioned envelope is an ERROR".into());
    }
    let root = resolve_root(root)?;
    let receipt = receipt_path(&root);
    let shown = abs_path(&receipt);
    if receipt.is_file() {
        return repair_from_receipt(&root, &receipt, &shown, apply, json);
    }
    // No receipt. Authoring rebuild is opt-in and still never writes goldens/.
    if authoring_mode() && join_rel(&root, BANK_REL).is_dir() {
        if apply {
            return repair_authoring(&root, seed);
        }
        return repair_authoring_dry_run(&root, &shown, seed, json);
    }
    let msg = format!(
        "repair: no receipt at {} — refusing to guess (will not invent content from bank/; reinstall via install.sh --verify)",
        shown.display()
    );
    if json {
        emit_repair_json(&RepairEnvelope {
            schema_version: REPAIR_SCHEMA_VERSION,
            mode: repair_mode(apply),
            ok: false,
            receipt: shown.display().to_string(),
            planned: Vec::new(),
            actual: Vec::new(),
            planned_restore: 0,
            actual_restore: 0,
        })?;
    }
    Err(msg)
}

fn repair_mode(apply: bool) -> &'static str {
    if apply {
        "apply"
    } else {
        "dry-run"
    }
}

fn repair_from_receipt(
    root: &Path,
    receipt: &Path,
    shown: &Path,
    apply: bool,
    json: bool,
) -> Result<(), String> {
    let bytes = fs::read(receipt).map_err(|e| {
        format!(
            "repair: receipt {} unreadable — refusing to guess: {e}",
            shown.display()
        )
    })?;
    if bytes.is_empty() {
        return refuse_unusable_receipt(
            shown,
            apply,
            json,
            "is 0 bytes — a receipt that pins nothing is an ERROR",
        );
    }
    let files = load_receipt_files(&bytes, shown)?;
    let mut planned = Vec::with_capacity(files.len());
    for (path, expected) in &files {
        planned.push(receipt_file_status(root, path, expected));
    }
    if planned.len() != files.len() {
        return Err(format!(
            "repair planned {} action(s), expected {} — a leg was dropped",
            planned.len(),
            files.len()
        ));
    }
    if planned.is_empty() {
        return Err(format!(
            "repair: receipt {} planned 0 actions — a receipt that certifies nothing is an ERROR",
            shown.display()
        ));
    }
    let planned_restore = planned.iter().filter(|a| a.status != "ok").count();
    let envelope = RepairEnvelope {
        schema_version: REPAIR_SCHEMA_VERSION,
        mode: repair_mode(apply),
        ok: planned_restore == 0,
        receipt: shown.display().to_string(),
        planned: planned.clone(),
        actual: Vec::new(),
        planned_restore,
        actual_restore: 0,
    };
    if json {
        emit_repair_json(&envelope)?;
    } else {
        emit_repair_human(apply, shown, &planned, planned_restore);
    }
    if planned_restore == 0 {
        return Ok(());
    }
    // Dry-run reports; apply still writes nothing — hashes are not copies.
    Err(drift_summary(&planned))
}

fn refuse_unusable_receipt(shown: &Path, apply: bool, json: bool, why: &str) -> Result<(), String> {
    let msg = format!(
        "repair: receipt {} {why} — refusing to guess",
        shown.display()
    );
    if json {
        emit_repair_json(&RepairEnvelope {
            schema_version: REPAIR_SCHEMA_VERSION,
            mode: repair_mode(apply),
            ok: false,
            receipt: shown.display().to_string(),
            planned: Vec::new(),
            actual: Vec::new(),
            planned_restore: 0,
            actual_restore: 0,
        })?;
    }
    Err(msg)
}

fn load_receipt_files(bytes: &[u8], shown: &Path) -> Result<Vec<(String, String)>, String> {
    let v: Value = serde_json::from_slice(bytes).map_err(|e| {
        format!(
            "repair: receipt {} is not JSON — refusing to guess: {e}",
            shown.display()
        )
    })?;
    let obj = v.as_object().ok_or_else(|| {
        format!(
            "repair: receipt {} is not a JSON object — refusing to guess",
            shown.display()
        )
    })?;
    let files = obj.get("files").ok_or_else(|| {
        format!(
            "repair: receipt {} has no files[] — a receipt that pins nothing is an ERROR",
            shown.display()
        )
    })?;
    let arr = files.as_array().ok_or_else(|| {
        format!(
            "repair: receipt {} files is not an array — refusing to guess",
            shown.display()
        )
    })?;
    if arr.is_empty() {
        return Err(format!(
            "repair: receipt {} pins 0 files — a receipt that certifies nothing is an ERROR",
            shown.display()
        ));
    }
    let mut out = Vec::with_capacity(arr.len());
    for (i, row) in arr.iter().enumerate() {
        let path = row
            .get("path")
            .and_then(|p| p.as_str())
            .unwrap_or("")
            .trim();
        let sha = row
            .get("sha256")
            .and_then(|p| p.as_str())
            .unwrap_or("")
            .trim();
        if path.is_empty() {
            return Err(format!(
                "repair: receipt {} files[{i}] has empty path — refusing to guess",
                shown.display()
            ));
        }
        if !is_sha256_hex(sha) {
            return Err(format!(
                "repair: receipt {} files[{i}] sha256 is not 64 hex — refusing to guess",
                shown.display()
            ));
        }
        out.push((path.to_string(), sha.to_ascii_lowercase()));
    }
    Ok(out)
}

fn is_sha256_hex(s: &str) -> bool {
    s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit())
}

fn resolve_recorded_path(root: &Path, recorded: &str) -> PathBuf {
    let p = PathBuf::from(recorded);
    if p.is_absolute() {
        p
    } else {
        root.join(p)
    }
}

fn receipt_file_status(root: &Path, recorded: &str, expected: &str) -> RepairAction {
    let path = resolve_recorded_path(root, recorded);
    let shown = abs_path(&path).display().to_string();
    if !path.is_file() {
        return RepairAction {
            path: shown,
            status: "missing".into(),
            expected_sha256: Some(expected.to_string()),
            actual_sha256: None,
        };
    }
    match fs::read(&path) {
        Ok(bytes) => {
            let actual = sha256_hex(&bytes);
            let status = if actual == expected {
                "ok"
            } else {
                "hash-mismatch"
            };
            RepairAction {
                path: shown,
                status: status.into(),
                expected_sha256: Some(expected.to_string()),
                actual_sha256: Some(actual),
            }
        }
        Err(_) => RepairAction {
            path: shown,
            status: "unreadable".into(),
            expected_sha256: Some(expected.to_string()),
            actual_sha256: None,
        },
    }
}

fn drift_summary(actions: &[RepairAction]) -> String {
    let bad: Vec<String> = actions
        .iter()
        .filter(|a| a.status != "ok")
        .map(|a| {
            format!(
                "{} {} expected={}",
                a.status,
                a.path,
                a.expected_sha256.as_deref().unwrap_or("")
            )
        })
        .collect();
    format!(
        "repair: {} file(s) drifted ({}) — refusing to invent content; reinstall via install.sh --verify",
        bad.len(),
        bad.join("; ")
    )
}

fn emit_repair_human(
    apply: bool,
    receipt: &Path,
    planned: &[RepairAction],
    planned_restore: usize,
) {
    if apply {
        println!("repair: apply");
    } else {
        println!("repair: dry-run (writes nothing)");
    }
    println!("repair: receipt {}", receipt.display());
    for a in planned {
        match a.status.as_str() {
            "ok" => println!("ok repair {} (sha256 match)", a.path),
            "missing" => println!(
                "FAIL repair {}: missing (expected sha256={})",
                a.path,
                a.expected_sha256.as_deref().unwrap_or("")
            ),
            "hash-mismatch" => println!(
                "FAIL repair {}: hash-mismatch expected={} actual={}",
                a.path,
                a.expected_sha256.as_deref().unwrap_or(""),
                a.actual_sha256.as_deref().unwrap_or("")
            ),
            other => println!(
                "FAIL repair {}: {other} (expected sha256={})",
                a.path,
                a.expected_sha256.as_deref().unwrap_or("")
            ),
        }
    }
    println!("repair: goldens/ not touched (UPDATE_GOLDENS is a human gate)");
    if planned_restore == 0 {
        if apply {
            println!("repair: planned_restore=0 (idempotent no-op)");
        } else {
            println!("repair: {} file(s) ok, planned_restore=0", planned.len());
        }
    }
}

fn emit_repair_json(envelope: &RepairEnvelope) -> Result<(), String> {
    let line = serde_json::to_string(envelope)
        .map_err(|e| format!("repair --json envelope unparseable: {e}"))?;
    let value: Value = serde_json::from_str(&line)
        .map_err(|e| format!("repair --json envelope unparseable: {e}"))?;
    validate_repair_envelope(&value)?;
    println!("{line}");
    Ok(())
}

fn validate_repair_envelope(v: &Value) -> Result<(), String> {
    let obj = v
        .as_object()
        .ok_or_else(|| "repair --json envelope is not a JSON object — unparseable".to_string())?;
    let ver = obj
        .get("schema_version")
        .and_then(|x| x.as_u64())
        .ok_or_else(|| "repair --json envelope is unversioned — refusing to emit".to_string())?;
    if ver != REPAIR_SCHEMA_VERSION {
        return Err(format!(
            "repair --json schema_version={ver} != {REPAIR_SCHEMA_VERSION}"
        ));
    }
    for key in REPAIR_JSON_FIELDS {
        if !obj.contains_key(*key) {
            return Err(format!(
                "repair --json envelope missing field {key} — refusing to emit"
            ));
        }
    }
    Ok(())
}

/// Source-checkout rebuild. Reached only when `CDCP_DEV=1 --apply`, `bank/`
/// is present, and there is no install receipt. Never writes goldens/.
fn repair_authoring(root: &Path, seed: u64) -> Result<(), String> {
    let mut wrote = 0usize;
    let mut planned = 0usize;

    planned += 1;
    wrote += repair_learn(root, LearnTarget::Units)?;
    planned += 1;
    wrote += repair_learn(root, LearnTarget::Glossary)?;
    planned += 1;
    wrote += repair_learn(root, LearnTarget::Slugs)?;

    let export_paths = [
        join_rel(root, EXPORT_OUT_REL).join(format!("mock40_seed{seed}.json")),
        join_rel(root, EXPORT_OUT_REL).join(format!("keys_seed{seed}.json")),
        join_rel(root, EXPORT_OUT_REL).join(format!("bank_items_seed{seed}.json")),
    ];
    planned += export_paths.len();
    let before = snapshot_mtimes(&export_paths);
    let bank = join_rel(root, BANK_REL);
    let out = join_rel(root, EXPORT_OUT_REL);
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

fn repair_authoring_dry_run(
    root: &Path,
    shown: &Path,
    seed: u64,
    json: bool,
) -> Result<(), String> {
    let rels = [
        UNITS_REL.to_string(),
        GLOSSARY_REL.to_string(),
        SLUGS_REL.to_string(),
        format!("{EXPORT_OUT_REL}/mock40_seed{seed}.json"),
        format!("{EXPORT_OUT_REL}/keys_seed{seed}.json"),
        format!("{EXPORT_OUT_REL}/bank_items_seed{seed}.json"),
    ];
    if rels.is_empty() {
        return Err(
            "repair planned 0 writes — a rebuild that targets nothing certifies nothing".into(),
        );
    }
    let planned: Vec<RepairAction> = rels
        .iter()
        .map(|rel| RepairAction {
            path: abs_path(&join_rel(root, rel)).display().to_string(),
            status: "rebuild".into(),
            expected_sha256: None,
            actual_sha256: None,
        })
        .collect();
    let planned_restore = planned.len();
    if json {
        emit_repair_json(&RepairEnvelope {
            schema_version: REPAIR_SCHEMA_VERSION,
            mode: "dry-run",
            ok: true,
            receipt: shown.display().to_string(),
            planned,
            actual: Vec::new(),
            planned_restore,
            actual_restore: 0,
        })?;
    } else {
        println!("repair: dry-run (writes nothing)");
        println!(
            "repair: no receipt at {} — authoring rebuild planned (pass --apply to write)",
            shown.display()
        );
        for rel in &rels {
            println!("repair: would rebuild {rel}");
        }
        println!("repair: goldens/ not touched (UPDATE_GOLDENS is a human gate)");
        println!("repair: planned_restore={planned_restore} (writes nothing)");
    }
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
