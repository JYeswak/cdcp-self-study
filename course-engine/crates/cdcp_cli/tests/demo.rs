//! `cdcp demo` (`bd-installability-sm4g.13`): planted grade + URL on an
//! INSTALLED tree. An empty proof list is ERROR.
//!
//! Proofs this file must run:
//!   1. bundle-only tree (web/ + wasm + seed-42 JSON; no bank/goldens/python)
//!      exits 0 and prints a URL + two distinct grade digests
//!   2. missing wasm → non-zero, names the absolute path
//!   3. missing mock seed-42 asset → non-zero, names the absolute path
//!   4. empty planted set → non-zero, names the file
//!   5. `--no-open` does not spawn a browser
//!   6. `cdcp --help` lists `demo`

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Named proofs. Dropping a row is a vacuous pass.
const PROOFS: &[&str] = &[
    "bundle_only_prints_url_and_two_distinct_digests",
    "missing_wasm_names_path",
    "missing_seed42_asset_names_path",
    "empty_planted_set_is_error",
    "no_open_does_not_spawn_browser",
    "help_lists_demo",
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

fn stamp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "cdcp-demo-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ))
}

/// INSTALLED layer only: `web/index.html` + shipped wasm + seed-42 JSON.
/// No bank/, no goldens/, no content.lock. A helper that cannot copy those
/// files fails the test — it must not skip and look green.
fn bundle_only_tree(tag: &str) -> PathBuf {
    let live = workspace_root();
    let required = [
        "web/index.html",
        "web/assets/wasm/cdcp_wasm.wasm",
        "web/data/mock40_seed42.json",
        "web/data/keys_seed42.json",
        "web/data/bank_items_seed42.json",
    ];
    for rel in required {
        let src = live.join(rel);
        assert!(
            src.is_file(),
            "cannot construct bundle-only tree: {} missing — this test is vacuous without the shipped asset",
            src.display()
        );
    }
    let dst = stamp(tag);
    for rel in required {
        let src = live.join(rel);
        let dest = dst.join(rel);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).unwrap_or_else(|e| {
                panic!("mkdir {}: {e}", parent.display());
            });
        }
        fs::copy(&src, &dest).unwrap_or_else(|e| {
            panic!("copy {} -> {}: {e}", src.display(), dest.display());
        });
    }
    assert!(
        !dst.join("bank").exists()
            && !dst.join("goldens").exists()
            && !dst.join("content.lock").exists()
            && !dst.join("registries").exists(),
        "bundle-only tree must not contain bank/goldens/content.lock/registries"
    );
    dst
}

fn isolate_cmd(cmd: &mut Command, tree: &Path) {
    let xdg = tree.join("xdg-empty");
    let home = tree.join("home-empty");
    let _ = fs::create_dir_all(&xdg);
    let _ = fs::create_dir_all(&home);
    cmd.env("XDG_DATA_HOME", &xdg);
    cmd.env("HOME", &home);
    cmd.env_remove("CDCP_HOME");
    cmd.env_remove("CDCP_REPO_ROOT");
    cmd.current_dir("/tmp");
}

fn run_demo(tree: &Path, extra: &[&str]) -> std::process::Output {
    let mut cmd = Command::new(cdcp_bin());
    isolate_cmd(&mut cmd, tree);
    cmd.args(["demo", "--no-open", "--root"]);
    cmd.arg(tree);
    cmd.args(extra);
    cmd.output()
        .unwrap_or_else(|e| panic!("spawn cdcp demo: {e}"))
}

fn stdout_of(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr_of(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn combined(out: &std::process::Output) -> String {
    format!("{}{}", stdout_of(out), stderr_of(out))
}

fn digest_after(stdout: &str, label: &str) -> String {
    for line in stdout.lines() {
        if let Some(rest) = line.strip_prefix(label) {
            let hex = rest.trim();
            assert!(
                hex.len() == 64 && hex.chars().all(|c| c.is_ascii_hexdigit()),
                "{label} must be 64 hex, got {hex:?}\n{stdout}"
            );
            return hex.to_string();
        }
    }
    panic!("missing {label} in:\n{stdout}");
}

fn plant_fake_opener(bin_dir: &Path, marker: &Path) {
    fs::create_dir_all(bin_dir).unwrap();
    let script = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$1\" >> \"{}\"\nexit 0\n",
        marker.display()
    );
    for name in ["open", "xdg-open"] {
        let path = bin_dir.join(name);
        fs::write(&path, &script).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&path, perms).unwrap();
        }
    }
}

fn prepend_path(bin_dir: &Path) -> std::ffi::OsString {
    let mut path = bin_dir.as_os_str().to_os_string();
    path.push(":");
    path.push(std::env::var_os("PATH").unwrap_or_default());
    path
}

#[test]
fn proof_list_is_not_empty() {
    assert!(
        !PROOFS.is_empty(),
        "empty demo proof list is ERROR — demo unmeasured"
    );
    assert!(PROOFS.contains(&"bundle_only_prints_url_and_two_distinct_digests"));
    assert!(PROOFS.contains(&"missing_wasm_names_path"));
    assert!(PROOFS.contains(&"missing_seed42_asset_names_path"));
    assert!(PROOFS.contains(&"empty_planted_set_is_error"));
    assert!(PROOFS.contains(&"no_open_does_not_spawn_browser"));
    assert!(PROOFS.contains(&"help_lists_demo"));
}

#[test]
fn bundle_only_prints_url_and_two_distinct_digests() {
    let tree = bundle_only_tree("ok");
    let out = run_demo(&tree, &[]);
    let stdout = stdout_of(&out);
    let stderr = stderr_of(&out);
    let code = out.status.code();
    if code != Some(0) {
        let _ = fs::remove_dir_all(&tree);
        panic!("bundle-only demo must exit 0, got {code:?}\nstdout={stdout}\nstderr={stderr}");
    }
    assert!(
        stdout.contains("http://") || stdout.contains("file://"),
        "demo must print a URL:\n{stdout}"
    );
    let ac = digest_after(&stdout, "cdcp demo: all-correct digest=");
    let aw = digest_after(&stdout, "cdcp demo: all-wrong digest=");
    assert_ne!(
        ac, aw,
        "all-correct digest must differ from all-wrong\n{stdout}"
    );
    assert!(
        stdout.contains("2-minute path"),
        "demo must print the 2-minute path:\n{stdout}"
    );
    assert!(
        stdout.contains("cdcp study"),
        "2-minute path must name cdcp study:\n{stdout}"
    );
    assert!(
        !tree.join("bank").exists() && !tree.join("goldens").exists(),
        "success must not have required bank/ or goldens/"
    );
    let _ = fs::remove_dir_all(&tree);
}

#[test]
fn missing_wasm_names_path() {
    let tree = bundle_only_tree("nowasm");
    let wasm = tree.join("web/assets/wasm/cdcp_wasm.wasm");
    let abs = wasm.canonicalize().unwrap_or(wasm.clone());
    fs::remove_file(&wasm).expect("delete wasm");
    assert!(!wasm.is_file(), "wasm must actually be gone");
    let out = run_demo(&tree, &[]);
    let text = combined(&out);
    let _ = fs::remove_dir_all(&tree);
    assert_ne!(
        out.status.code(),
        Some(0),
        "missing wasm must be RED\n{text}"
    );
    assert!(
        text.contains(&abs.display().to_string()) || text.contains(&wasm.display().to_string()),
        "missing wasm must name the path {}\n{text}",
        abs.display()
    );
}

#[test]
fn missing_seed42_asset_names_path() {
    // One planted known-bad per required JSON. Dropping a row is vacuous.
    let assets = [
        "web/data/mock40_seed42.json",
        "web/data/keys_seed42.json",
        "web/data/bank_items_seed42.json",
    ];
    #[allow(clippy::const_is_empty)] // anti-vacuous: empty planted list is ERROR
    {
        assert!(
            !assets.is_empty(),
            "empty missing-asset list is ERROR — nothing was planted"
        );
    }
    for rel in assets {
        let tree = bundle_only_tree(&format!("no-{}", rel.replace('/', "_")));
        let path = tree.join(rel);
        let abs = path.canonicalize().unwrap_or(path.clone());
        fs::remove_file(&path).unwrap_or_else(|e| panic!("delete {rel}: {e}"));
        let out = run_demo(&tree, &[]);
        let text = combined(&out);
        let _ = fs::remove_dir_all(&tree);
        assert_ne!(
            out.status.code(),
            Some(0),
            "missing {rel} must be RED\n{text}"
        );
        assert!(
            text.contains(&abs.display().to_string()) || text.contains(&path.display().to_string()),
            "missing {rel} must name the path {}\n{text}",
            abs.display()
        );
    }
}

#[test]
fn empty_planted_set_is_error() {
    let tree = bundle_only_tree("empty-plant");
    let mock = tree.join("web/data/mock40_seed42.json");
    fs::write(
        &mock,
        r#"{"exam_id":"mock40","seed":42,"n_items":0,"items":[]}"#,
    )
    .expect("plant empty mock");
    let keys = tree.join("web/data/keys_seed42.json");
    fs::write(&keys, r#"{"exam_id":"mock40","keys":[]}"#).expect("plant empty keys");
    let out = run_demo(&tree, &[]);
    let text = combined(&out);
    let _ = fs::remove_dir_all(&tree);
    assert_ne!(
        out.status.code(),
        Some(0),
        "empty planted set must be RED\n{text}"
    );
    assert!(
        text.contains("empty") || text.contains("planted"),
        "empty planted set must say so:\n{text}"
    );
    assert!(
        text.contains(&mock.display().to_string())
            || text.contains("mock40_seed42")
            || text.contains(&keys.display().to_string())
            || text.contains("keys_seed42"),
        "empty planted set must name the file:\n{text}"
    );
}

#[test]
fn no_open_does_not_spawn_browser() {
    let tree = bundle_only_tree("noopen");
    let bin_dir = tree.join("bin");
    let marker = tree.join("opener-marker");
    plant_fake_opener(&bin_dir, &marker);

    let mut cmd = Command::new(cdcp_bin());
    isolate_cmd(&mut cmd, &tree);
    cmd.env("PATH", prepend_path(&bin_dir));
    cmd.args(["demo", "--no-open", "--root"]);
    cmd.arg(&tree);
    let out = cmd.output().unwrap_or_else(|e| panic!("spawn: {e}"));
    let stdout = stdout_of(&out);
    let stderr = stderr_of(&out);
    let marker_exists = marker.exists();
    let _ = fs::remove_dir_all(&tree);
    assert_eq!(
        out.status.code(),
        Some(0),
        "demo --no-open must exit 0\nstdout={stdout}\nstderr={stderr}"
    );
    assert!(
        !marker_exists,
        "--no-open must not spawn a browser; opener marker exists"
    );
}

#[test]
fn help_lists_demo() {
    let out = Command::new(cdcp_bin())
        .arg("--help")
        .output()
        .unwrap_or_else(|e| panic!("spawn --help: {e}"));
    assert_eq!(out.status.code(), Some(0));
    let stdout = stdout_of(&out);
    assert!(
        stdout.contains("demo"),
        "cdcp --help must list demo: {stdout}"
    );
    let demo_help = Command::new(cdcp_bin())
        .args(["demo", "--help"])
        .output()
        .unwrap_or_else(|e| panic!("spawn demo --help: {e}"));
    let demo_out = stdout_of(&demo_help);
    assert!(
        demo_out.contains("--no-open"),
        "demo --help must list --no-open: {demo_out}"
    );
}
