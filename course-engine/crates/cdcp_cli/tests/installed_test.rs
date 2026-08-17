//! N.14 (`bd-installability-sm4g.14`): `cdcp test` smokes the installed tree.
//!
//! Proofs this file must run. An empty list is ERROR.
//!   1. suite list is compiled-in and non-empty
//!   2. bundle-only tree (web/ + seed-42 assets + wasm) exits 0
//!   3. wasm deleted → non-zero, names the absolute wasm path
//!   4. does not invoke cargo, python3, or goldens check
//!   5. `--help` lists `test`
//!
//! What a green result does not prove: the forty stems are the published
//! draw; the wasm is bit-identical to a fresh `--release` rebuild; keys
//! match the pack item-for-item; HTTP 200 against a live listener.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Named proofs. Dropping a row is a vacuous pass.
const PROOFS: &[&str] = &[
    "suite_list_is_compiled_in_and_nonempty",
    "bundle_only_tree_exits_0",
    "wasm_deleted_exits_nonzero_naming_abs_path",
    "does_not_invoke_cargo_python_goldens",
    "help_lists_test",
];

/// Files a bundle-only tree must carry. Emptying this makes the helper
/// construct nothing and the pass becomes vacuous.
const BUNDLE_FILES: &[&str] = &[
    "web/index.html",
    "web/assets/wasm/cdcp_wasm.wasm",
    "web/data/mock40_seed42.json",
    "web/data/bank_items_seed42.json",
    "web/data/keys_seed42.json",
];

const EXPECTED_SUITE: &[&str] = &[
    "learner-pack",
    "wasm",
    "seed42-pack",
    "seed42-bank",
    "seed42-keys",
];

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("course-engine workspace root")
}

fn cdcp_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cdcp"))
}

fn stamp() -> String {
    format!(
        "cdcp-n14-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    )
}

fn isolate_cmd(cmd: &mut Command, xdg: &Path, home: &Path) {
    cmd.env("XDG_DATA_HOME", xdg);
    cmd.env("HOME", home);
    cmd.env_remove("CDCP_HOME");
    cmd.env_remove("CDCP_REPO_ROOT");
    cmd.current_dir("/tmp");
}

fn copy_file(src: &Path, dst: &Path) {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|e| panic!("mkdir {}: {e}", parent.display()));
    }
    fs::copy(src, dst)
        .unwrap_or_else(|e| panic!("copy {} -> {}: {e}", src.display(), dst.display()));
}

/// INSTALLED layer only: `web/` + shipped wasm + seed-42 pack/bank/keys.
/// No bank/, no goldens/, no content.lock. A helper that cannot copy those
/// files fails the test — it must not skip and look green.
fn bundle_only_tree(tag: &str) -> PathBuf {
    assert!(
        !BUNDLE_FILES.is_empty(),
        "empty BUNDLE_FILES is ERROR — nothing to plant"
    );
    let live = workspace_root();
    for rel in BUNDLE_FILES {
        let src = live.join(rel);
        assert!(
            src.is_file(),
            "cannot construct bundle-only tree: {} missing — this test is vacuous without a live web/",
            src.display()
        );
    }
    let dst = std::env::temp_dir().join(format!("{}-{tag}", stamp()));
    for rel in BUNDLE_FILES {
        copy_file(&live.join(rel), &dst.join(rel));
    }
    assert!(
        dst.join("web/index.html").is_file()
            && dst.join("web/assets/wasm/cdcp_wasm.wasm").is_file()
            && dst.join("web/data/mock40_seed42.json").is_file()
            && dst.join("web/data/bank_items_seed42.json").is_file()
            && dst.join("web/data/keys_seed42.json").is_file(),
        "bundle-only tree failed to materialize under {}",
        dst.display()
    );
    assert!(
        !dst.join("bank").exists()
            && !dst.join("goldens").exists()
            && !dst.join("content.lock").exists(),
        "bundle-only tree must not contain bank/goldens/content.lock"
    );
    dst
}

fn run_test(root: &Path) -> std::process::Output {
    let base = std::env::temp_dir().join(format!("{}-iso", stamp()));
    let xdg = base.join("xdg");
    let home = base.join("home");
    fs::create_dir_all(&xdg).unwrap();
    fs::create_dir_all(&home).unwrap();
    let mut cmd = Command::new(cdcp_bin());
    isolate_cmd(&mut cmd, &xdg, &home);
    cmd.args(["test", "--root"]);
    cmd.arg(root);
    let out = cmd
        .output()
        .unwrap_or_else(|e| panic!("spawn cdcp test: {e}"));
    let _ = fs::remove_dir_all(&base);
    out
}

fn combined(out: &std::process::Output) -> String {
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

#[test]
fn proof_list_is_not_empty() {
    assert!(
        !PROOFS.is_empty(),
        "empty N.14 proof list is ERROR — acceptance unmeasured"
    );
    assert!(PROOFS.contains(&"suite_list_is_compiled_in_and_nonempty"));
    assert!(PROOFS.contains(&"bundle_only_tree_exits_0"));
    assert!(PROOFS.contains(&"wasm_deleted_exits_nonzero_naming_abs_path"));
    assert!(PROOFS.contains(&"does_not_invoke_cargo_python_goldens"));
    assert!(PROOFS.contains(&"help_lists_test"));
    assert!(
        !BUNDLE_FILES.is_empty(),
        "empty BUNDLE_FILES is ERROR — the helper would plant nothing"
    );
    assert!(
        !EXPECTED_SUITE.is_empty(),
        "empty EXPECTED_SUITE is ERROR — a test that ships zero cases is RED"
    );
}

#[test]
fn suite_list_is_compiled_in_and_nonempty() {
    let src = include_str!("../src/installed_test.rs");
    let prod = src
        .split("#[cfg(test)]")
        .next()
        .expect("production precedes tests");
    let start = prod
        .find("const SUITE:")
        .expect("SUITE must be a compiled-in list");
    let rest = &prod[start..];
    let open = rest
        .find("= &[")
        .expect("SUITE must be a compiled-in array");
    let body = &rest[open + 4..];
    let end = body.find(']').expect("SUITE list must close");
    let chunk = &body[..end];
    for name in EXPECTED_SUITE {
        assert!(
            chunk.contains(&format!("\"{name}\"")),
            "compiled-in SUITE missing {name}: {chunk}"
        );
    }
    assert!(
        EXPECTED_SUITE.len() >= 5,
        "suite floor shrank: {}",
        EXPECTED_SUITE.len()
    );
}

#[test]
fn help_lists_test() {
    let out = Command::new(cdcp_bin())
        .arg("--help")
        .output()
        .unwrap_or_else(|e| panic!("spawn cdcp --help: {e}"));
    assert!(out.status.success(), "cdcp --help must exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("test"),
        "cdcp --help must list test: {stdout}"
    );
    let bare = Command::new(cdcp_bin())
        .output()
        .unwrap_or_else(|e| panic!("spawn bare cdcp: {e}"));
    let orientation = String::from_utf8_lossy(&bare.stdout);
    assert!(
        orientation.contains("cdcp test"),
        "bare cdcp must advertise test: {orientation}"
    );
}

#[test]
fn bundle_only_tree_exits_0() {
    let tree = bundle_only_tree("ok");
    let out = run_test(&tree);
    let text = combined(&out);
    let _ = fs::remove_dir_all(&tree);
    assert_eq!(
        out.status.code(),
        Some(0),
        "cdcp test on a bundle-only tree must exit 0, got {:?}\n{text}",
        out.status.code()
    );
    assert!(
        text.contains("using installed root"),
        "must PRINT installed vs source-checkout: {text}"
    );
    assert!(
        text.contains("via --root"),
        "must PRINT the precedence slot: {text}"
    );
    for name in EXPECTED_SUITE {
        assert!(
            text.contains(&format!("ok test {name}")),
            "suite case {name} did not run: {text}"
        );
    }
    assert!(
        text.contains(&format!("test: {} case(s) passed", EXPECTED_SUITE.len())),
        "must report the compiled-in count: {text}"
    );
    assert!(
        !text.contains("ok golden")
            && !text.contains("goldens check")
            && !text.contains("UPDATE_GOLDENS"),
        "installed-tree smoke must not run goldens check: {text}"
    );
}

#[test]
fn wasm_deleted_exits_nonzero_naming_abs_path() {
    let tree = bundle_only_tree("nowasm");
    let wasm = tree.join("web/assets/wasm/cdcp_wasm.wasm");
    assert!(wasm.is_file(), "fixture must start with a wasm artifact");
    let abs = wasm
        .canonicalize()
        .unwrap_or_else(|_| wasm.clone())
        .display()
        .to_string();
    fs::remove_file(&wasm).expect("delete wasm");
    assert!(!wasm.is_file(), "wasm must actually be gone");

    let out = run_test(&tree);
    let text = combined(&out);
    let _ = fs::remove_dir_all(&tree);

    assert_ne!(
        out.status.code(),
        Some(0),
        "wasm-deleted tree must be RED, got {:?}\n{text}",
        out.status.code()
    );
    assert!(
        text.contains(&abs),
        "deleting the wasm artifact must name the absolute path {abs}, got: {text}"
    );
    assert!(
        Path::new(&abs).is_absolute(),
        "named wasm path must be absolute: {abs}"
    );
    assert!(
        text.contains("FAIL test wasm") || text.contains("wasm:"),
        "must name the wasm case: {text}"
    );
}

#[test]
fn does_not_invoke_cargo_python_goldens() {
    let tree = bundle_only_tree("nospawn");
    let base = std::env::temp_dir().join(format!("{}-spy", stamp()));
    let spy = base.join("spy");
    let xdg = base.join("xdg");
    let home = base.join("home");
    let log = base.join("invoked.log");
    fs::create_dir_all(&spy).unwrap();
    fs::create_dir_all(&xdg).unwrap();
    fs::create_dir_all(&home).unwrap();

    // Planted spies: if `cdcp test` execs these, the log is written and the
    // test is RED. An empty spy list would be a vacuous pass.
    let spies = ["cargo", "python3", "python", "check.sh"];
    #[allow(clippy::const_is_empty)] // anti-vacuous: empty spy list is ERROR
    {
        assert!(!spies.is_empty(), "empty spy list is ERROR");
    }
    let script = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$0\" >> \"{}\"\nexit 0\n",
        log.display()
    );
    for name in spies {
        let path = spy.join(name);
        fs::write(&path, &script).unwrap();
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).unwrap();
    }

    let path_now = std::env::var("PATH").unwrap_or_default();
    let mut cmd = Command::new(cdcp_bin());
    isolate_cmd(&mut cmd, &xdg, &home);
    cmd.env("PATH", format!("{}:{path_now}", spy.display()));
    cmd.args(["test", "--root"]);
    cmd.arg(&tree);
    let out = cmd
        .output()
        .unwrap_or_else(|e| panic!("spawn cdcp test under spies: {e}"));
    let text = combined(&out);
    let invoked = fs::read_to_string(&log).unwrap_or_default();
    let _ = fs::remove_dir_all(&tree);
    let _ = fs::remove_dir_all(&base);

    assert_eq!(
        out.status.code(),
        Some(0),
        "bundle-only tree under spy PATH must still exit 0\n{text}"
    );
    assert!(
        invoked.is_empty(),
        "cdcp test invoked a planted authoring tool:\n{invoked}\n{text}"
    );
    assert!(
        !text.contains("ok golden") && !text.contains("goldens check"),
        "stdout must not show a goldens check: {text}"
    );

    let src = include_str!("../src/installed_test.rs");
    let prod = src
        .split("#[cfg(test)]")
        .next()
        .expect("production precedes tests");
    for needle in [
        "Command::new",
        "std::process",
        "python3",
        "goldens",
        "cargo",
    ] {
        assert!(
            !prod.contains(needle),
            "production installed_test.rs mentions {needle}"
        );
    }
}

#[test]
fn leaked_correct_is_red() {
    let tree = bundle_only_tree("leak");
    let pack = tree.join("web/data/mock40_seed42.json");
    let raw = fs::read_to_string(&pack).expect("read pack");
    let mut v: serde_json::Value = serde_json::from_str(&raw).expect("pack json");
    v["items"][0]["correct"] = serde_json::json!("A");
    fs::write(&pack, serde_json::to_vec(&v).unwrap()).unwrap();

    let out = run_test(&tree);
    let text = combined(&out);
    let _ = fs::remove_dir_all(&tree);
    assert_ne!(
        out.status.code(),
        Some(0),
        "leaked correct must RED\n{text}"
    );
    assert!(
        text.contains("leaks correct") || text.contains("learner-pack"),
        "must name the leak: {text}"
    );
}

#[test]
fn missing_keys_is_red_naming_path() {
    let tree = bundle_only_tree("nokeys");
    let keys = tree.join("web/data/keys_seed42.json");
    let abs = keys
        .canonicalize()
        .unwrap_or_else(|_| keys.clone())
        .display()
        .to_string();
    fs::remove_file(&keys).expect("delete keys");

    let out = run_test(&tree);
    let text = combined(&out);
    let _ = fs::remove_dir_all(&tree);
    assert_ne!(out.status.code(), Some(0), "missing keys must RED\n{text}");
    assert!(
        text.contains(&abs),
        "missing keys must name the absolute path {abs}, got: {text}"
    );
}

#[test]
fn cdcp_home_resolves_the_same_tree() {
    let tree = bundle_only_tree("home");
    let base = std::env::temp_dir().join(format!("{}-homeiso", stamp()));
    let xdg = base.join("xdg");
    let home = base.join("home");
    fs::create_dir_all(&xdg).unwrap();
    fs::create_dir_all(&home).unwrap();
    let mut cmd = Command::new(cdcp_bin());
    cmd.env("XDG_DATA_HOME", &xdg);
    cmd.env("HOME", &home);
    cmd.env("CDCP_HOME", &tree);
    cmd.env_remove("CDCP_REPO_ROOT");
    cmd.current_dir("/tmp");
    cmd.arg("test");
    let out = cmd
        .output()
        .unwrap_or_else(|e| panic!("spawn cdcp test via CDCP_HOME: {e}"));
    let text = combined(&out);
    let _ = fs::remove_dir_all(&tree);
    let _ = fs::remove_dir_all(&base);
    assert_eq!(
        out.status.code(),
        Some(0),
        "CDCP_HOME must resolve the bundle-only tree\n{text}"
    );
    assert!(
        text.contains("via CDCP_HOME"),
        "must PRINT the CDCP_HOME slot: {text}"
    );
}
