//! Installed self-doc (`bd-installability-sm4g.15`).
//!
//! Proofs this file must run. An empty list is ERROR.
//!   1. `--info` prints version + root + via (`--root` / CDCP_HOME / XDG / cwd-walk)
//!   2. `--info --json` is a versioned envelope (schema_version required)
//!   3. `quickstart` is ≥200 words and names demo, doctor, test, study/serve
//!   4. `help install|doctor|study` are topics, not "unrecognized subcommand"
//!   5. `completion bash | bash -n -` exits 0; zsh and fish emit too
//!   6. learner `--help` lists quickstart/completion and `--info`, not authoring
//!
//! Green does not prove: that install.sh writes the receipt `--info` names,
//! or that a stranger's first machine has bash-completion wired.

use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command as Proc, Stdio};

/// Named proofs. Dropping a row is a vacuous pass.
const PROOFS: &[&str] = &[
    "info_via_explicit_root",
    "info_via_cdcp_home",
    "info_via_xdg",
    "info_via_cwd_walk",
    "info_json_is_versioned_envelope",
    "info_json_unresolved_still_versioned",
    "quickstart_word_count_and_named_verbs",
    "help_install_is_topic_not_subcommand",
    "help_doctor_is_topic_not_clap_usage",
    "help_study_is_topic_not_clap_usage",
    "completion_bash_passes_bash_n",
    "completion_zsh_and_fish_emit",
    "learner_help_lists_selfdoc_hides_authoring",
];

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("course-engine workspace root")
}

fn cdcp() -> Command {
    let mut cmd = Command::cargo_bin("cdcp").expect("cdcp binary");
    cmd.env_remove("CDCP_DEV");
    cmd
}

fn stamp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "cdcp-selfdoc-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ))
}

fn plant_installed(dir: &Path) {
    let index = dir.join("web/index.html");
    if let Some(parent) = index.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|e| panic!("mkdir {}: {e}", parent.display()));
    }
    fs::write(&index, "<title>cdcp-selfdoc-plant</title>\n")
        .unwrap_or_else(|e| panic!("write {}: {e}", index.display()));
    assert!(
        index.is_file(),
        "plant failed — --info via tests are vacuous without a bundle: {}",
        index.display()
    );
}

fn isolate(cmd: &mut Command, cwd: &Path, home: &Path, xdg: &Path) {
    cmd.current_dir(cwd);
    cmd.env("HOME", home);
    cmd.env("XDG_DATA_HOME", xdg);
    cmd.env_remove("CDCP_HOME");
    cmd.env_remove("CDCP_REPO_ROOT");
}

fn listed_commands(help: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_cmds = false;
    for line in help.lines() {
        if line.starts_with("Commands:") {
            in_cmds = true;
            continue;
        }
        if in_cmds && (line.starts_with("Options:") || line.starts_with("Arguments:")) {
            break;
        }
        if in_cmds {
            let t = line.trim_start();
            if let Some(name) = t.split_whitespace().next() {
                if name.chars().all(|c| c.is_ascii_lowercase() || c == '-') {
                    out.push(name.to_string());
                }
            }
        }
    }
    out
}

#[test]
fn proofs_list_is_not_empty() {
    assert!(
        !PROOFS.is_empty(),
        "empty self-doc proof list is ERROR — nothing was checked"
    );
    assert!(
        PROOFS.len() >= 13,
        "self-doc proof list shrank: {}",
        PROOFS.len()
    );
    assert!(PROOFS.contains(&"info_via_explicit_root"));
    assert!(PROOFS.contains(&"info_via_cdcp_home"));
    assert!(PROOFS.contains(&"info_via_xdg"));
    assert!(PROOFS.contains(&"info_via_cwd_walk"));
    assert!(PROOFS.contains(&"completion_bash_passes_bash_n"));
}

#[test]
fn info_via_explicit_root() {
    assert!(PROOFS.contains(&"info_via_explicit_root"));
    let base = stamp("root");
    let expl = base.join("explicit");
    let home = base.join("home");
    let xdg = base.join("xdg");
    let decoy = base.join("decoy");
    plant_installed(&expl);
    plant_installed(&decoy);
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&xdg).unwrap();
    let assert = {
        let mut cmd = cdcp();
        isolate(&mut cmd, &base, &home, &xdg);
        cmd.env("CDCP_HOME", &decoy);
        cmd.args(["--info", "--root"]).arg(&expl);
        cmd.assert().success()
    };
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stdout.contains(env!("CARGO_PKG_VERSION")),
        "--info must print the workspace version: {stdout}"
    );
    assert!(
        stdout.contains(&expl.display().to_string()),
        "--info --root must name the explicit root: {stdout}"
    );
    assert!(
        stdout.contains("via: --root"),
        "--info --root must name the precedence step --root: {stdout}"
    );
    assert!(
        !stdout.contains(&decoy.display().to_string()),
        "--root must beat CDCP_HOME: {stdout}"
    );
    assert!(
        stderr.is_empty(),
        "--info success must not write stderr: {stderr:?}"
    );
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn info_via_cdcp_home() {
    assert!(PROOFS.contains(&"info_via_cdcp_home"));
    let base = stamp("home");
    let cdcp_home = base.join("cdcp-home");
    let xdg = base.join("xdg");
    let home = base.join("home");
    plant_installed(&cdcp_home);
    plant_installed(&xdg.join("cdcp"));
    fs::create_dir_all(&home).unwrap();
    let assert = {
        let mut cmd = cdcp();
        isolate(&mut cmd, &base, &home, &xdg);
        cmd.env("CDCP_HOME", &cdcp_home);
        cmd.arg("--info");
        cmd.assert().success()
    };
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains(env!("CARGO_PKG_VERSION")),
        "--info must print version: {stdout}"
    );
    assert!(
        stdout.contains(&cdcp_home.display().to_string()),
        "--info CDCP_HOME must name the home: {stdout}"
    );
    assert!(
        stdout.contains("via: CDCP_HOME"),
        "--info must name the precedence step CDCP_HOME: {stdout}"
    );
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn info_via_xdg() {
    assert!(PROOFS.contains(&"info_via_xdg"));
    let base = stamp("xdg");
    let xdg = base.join("xdg");
    let home = base.join("home");
    let cwd = base.join("cwd");
    plant_installed(&xdg.join("cdcp"));
    plant_installed(&cwd);
    fs::create_dir_all(&home).unwrap();
    let assert = {
        let mut cmd = cdcp();
        isolate(&mut cmd, &cwd, &home, &xdg);
        cmd.arg("--info");
        cmd.assert().success()
    };
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let want = xdg.join("cdcp");
    assert!(
        stdout.contains(&want.display().to_string()),
        "--info XDG must name $XDG_DATA_HOME/cdcp: {stdout}"
    );
    assert!(
        stdout.contains("via: XDG"),
        "--info must name the precedence step XDG: {stdout}"
    );
    assert!(
        !stdout.contains(&cwd.display().to_string()) || stdout.contains("via: XDG"),
        "XDG must beat cwd-walk: {stdout}"
    );
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn info_via_cwd_walk() {
    assert!(PROOFS.contains(&"info_via_cwd_walk"));
    let base = stamp("cwd");
    let root = base.join("tree");
    let nested = root.join("a/b");
    let xdg = base.join("xdg-empty");
    let home = base.join("home-empty");
    plant_installed(&root);
    fs::create_dir_all(&nested).unwrap();
    fs::create_dir_all(&xdg).unwrap();
    fs::create_dir_all(&home).unwrap();
    let assert = {
        let mut cmd = cdcp();
        isolate(&mut cmd, &nested, &home, &xdg);
        cmd.arg("--info");
        cmd.assert().success()
    };
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains(&root.display().to_string()),
        "--info cwd-walk must name the walked root: {stdout}"
    );
    assert!(
        stdout.contains("via: cwd-walk"),
        "--info must name the precedence step cwd-walk: {stdout}"
    );
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn info_json_is_versioned_envelope() {
    assert!(PROOFS.contains(&"info_json_is_versioned_envelope"));
    let base = stamp("json");
    let root = base.join("tree");
    let xdg = base.join("xdg");
    let home = base.join("home");
    plant_installed(&root);
    fs::create_dir_all(&xdg).unwrap();
    fs::create_dir_all(&home).unwrap();
    let assert = {
        let mut cmd = cdcp();
        isolate(&mut cmd, &base, &home, &xdg);
        cmd.args(["--info", "--json", "--root"]).arg(&root);
        cmd.assert().success()
    };
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let v: Value = serde_json::from_str(stdout.trim()).unwrap_or_else(|e| {
        panic!("--info --json must be JSON: {e}: {stdout}");
    });
    let obj = v.as_object().expect("--info --json must be an object");
    for key in ["schema_version", "version", "root", "kind", "via", "web"] {
        assert!(
            obj.contains_key(key),
            "--info --json missing {key}: {stdout}"
        );
    }
    assert_eq!(
        obj.get("schema_version").and_then(Value::as_u64),
        Some(1),
        "--info --json schema_version must be 1: {stdout}"
    );
    assert_eq!(
        obj.get("version").and_then(Value::as_str),
        Some(env!("CARGO_PKG_VERSION")),
        "--info --json version must be the workspace pin: {stdout}"
    );
    assert_eq!(
        obj.get("via").and_then(Value::as_str),
        Some("--root"),
        "--info --json via must be --root: {stdout}"
    );
    assert_eq!(
        obj.get("root").and_then(Value::as_str),
        Some(root.to_str().expect("utf-8 root")),
        "--info --json root path: {stdout}"
    );
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn info_json_unresolved_still_versioned() {
    assert!(PROOFS.contains(&"info_json_unresolved_still_versioned"));
    let base = stamp("missing");
    let cwd = base.join("nowhere");
    let xdg = base.join("xdg-empty");
    let home = base.join("home-empty");
    fs::create_dir_all(&cwd).unwrap();
    fs::create_dir_all(&xdg).unwrap();
    fs::create_dir_all(&home).unwrap();
    let output = {
        let mut cmd = cdcp();
        isolate(&mut cmd, &cwd, &home, &xdg);
        cmd.args(["--info", "--json"]);
        cmd.output().expect("spawn --info --json")
    };
    assert_eq!(
        output.status.code(),
        Some(4),
        "unresolved --info must exit 4 (bundle missing), got {:?}",
        output.status.code()
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let v: Value = serde_json::from_str(stdout.trim()).unwrap_or_else(|e| {
        panic!("unresolved --info --json must still be JSON: {e}: {stdout}");
    });
    assert_eq!(
        v.get("schema_version").and_then(Value::as_u64),
        Some(1),
        "unresolved envelope must stay versioned: {stdout}"
    );
    assert!(
        v.get("error")
            .and_then(Value::as_str)
            .is_some_and(|e| e.contains("bundle not found")),
        "unresolved --info --json must name bundle not found: {stdout}"
    );
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn quickstart_word_count_and_named_verbs() {
    assert!(PROOFS.contains(&"quickstart_word_count_and_named_verbs"));
    let assert = cdcp().arg("quickstart").assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let n = stdout.split_whitespace().count();
    assert!(n >= 200, "quickstart is {n} words (floor 200): {stdout}");
    for needle in ["demo", "doctor", "test"] {
        assert!(
            stdout.contains(needle),
            "quickstart must name {needle}: {stdout}"
        );
    }
    assert!(
        stdout.contains("study") || stdout.contains("serve"),
        "quickstart must name study/serve: {stdout}"
    );
}

#[test]
fn help_install_is_topic_not_subcommand() {
    assert!(PROOFS.contains(&"help_install_is_topic_not_subcommand"));
    let assert = cdcp().args(["help", "install"]).assert().success();
    let out = assert.get_output();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let both = format!("{stdout}{stderr}");
    assert!(
        !both.contains("unrecognized subcommand"),
        "help install must be a topic, not unrecognized: {both}"
    );
    assert!(
        !stdout.contains("Usage: cdcp install"),
        "help install must not invent a clap subcommand: {stdout}"
    );
    assert!(
        stdout.contains("install.sh") || stdout.contains("curl"),
        "help install topic must name the installer: {stdout}"
    );
}

#[test]
fn help_doctor_is_topic_not_clap_usage() {
    assert!(PROOFS.contains(&"help_doctor_is_topic_not_clap_usage"));
    let assert = cdcp().args(["help", "doctor"]).assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    let both = format!("{stdout}{stderr}");
    assert!(
        !both.contains("unrecognized subcommand"),
        "help doctor must be a topic: {both}"
    );
    assert!(
        !stdout.contains("Usage: cdcp doctor [OPTIONS]"),
        "help doctor must be topic-help, not clap subcommand help: {stdout}"
    );
    assert!(
        stdout.to_ascii_lowercase().contains("installed"),
        "help doctor topic must describe the installed layer: {stdout}"
    );
}

#[test]
fn help_study_is_topic_not_clap_usage() {
    assert!(PROOFS.contains(&"help_study_is_topic_not_clap_usage"));
    let assert = cdcp().args(["help", "study"]).assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    let both = format!("{stdout}{stderr}");
    assert!(
        !both.contains("unrecognized subcommand"),
        "help study must be a topic: {both}"
    );
    assert!(
        !stdout.contains("Usage: cdcp study [OPTIONS]"),
        "help study must be topic-help, not clap subcommand help: {stdout}"
    );
    assert!(
        stdout.contains("study"),
        "help study topic must name study: {stdout}"
    );
}

#[test]
fn unknown_help_topic_is_still_unrecognized() {
    let output = cdcp()
        .args(["help", "definitely-not-a-cdcp-topic"])
        .output()
        .expect("spawn help unknown");
    assert!(
        !output.status.success(),
        "unknown help topic must not exit 0 (that would mint topics)"
    );
    let both = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        both.contains("unrecognized subcommand"),
        "unknown topic must stay clap-unrecognized: {both}"
    );
}

#[test]
fn completion_bash_passes_bash_n() {
    assert!(PROOFS.contains(&"completion_bash_passes_bash_n"));
    let assert = cdcp().args(["completion", "bash"]).assert().success();
    let script = &assert.get_output().stdout;
    assert!(
        !script.is_empty(),
        "completion bash emitted zero bytes — bash -n on empty is a vacuous pass"
    );
    let mut child = Proc::new("bash")
        .args(["-n", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("spawn bash -n - (required, not skippable): {e}"));
    {
        use std::io::Write;
        child
            .stdin
            .as_mut()
            .expect("bash stdin")
            .write_all(script)
            .expect("pipe completion into bash -n");
    }
    let out = child.wait_with_output().expect("wait bash -n");
    assert!(
        out.status.success(),
        "completion bash | bash -n - failed ({}):\n{}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn completion_zsh_and_fish_emit() {
    assert!(PROOFS.contains(&"completion_zsh_and_fish_emit"));
    for shell in ["zsh", "fish"] {
        let assert = cdcp().args(["completion", shell]).assert().success();
        let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
        assert!(
            !stdout.trim().is_empty(),
            "completion {shell} emitted nothing"
        );
        assert!(
            stdout.contains("cdcp"),
            "completion {shell} must name cdcp: {stdout}"
        );
    }
}

#[test]
fn completion_unknown_shell_is_red() {
    let output = cdcp()
        .args(["completion", "csh"])
        .output()
        .expect("spawn completion csh");
    assert!(
        !output.status.success(),
        "completion csh must be RED (unknown shell is not a pass)"
    );
}

#[test]
fn learner_help_lists_selfdoc_hides_authoring() {
    assert!(PROOFS.contains(&"learner_help_lists_selfdoc_hides_authoring"));
    let assert = cdcp()
        .current_dir(workspace_root())
        .arg("--help")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let cmds = listed_commands(&stdout);
    for verb in [
        "quickstart",
        "completion",
        "study",
        "doctor",
        "demo",
        "test",
        "repair",
    ] {
        assert!(
            cmds.iter().any(|c| c == verb),
            "learner --help missing {verb}: {cmds:?}\n{stdout}"
        );
    }
    assert!(
        stdout.contains("--info"),
        "learner --help must list --info: {stdout}"
    );
    for authoring in ["bank-hash", "build-learn", "goldens", "export-web", "serve"] {
        assert!(
            !cmds.iter().any(|c| c == authoring),
            "learner --help still lists authoring verb {authoring}: {cmds:?}"
        );
    }
}

#[test]
fn completion_bash_does_not_list_authoring() {
    let assert = cdcp().args(["completion", "bash"]).assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    for authoring in ["bank-hash", "build-learn", "goldens", "export-web"] {
        assert!(
            !stdout.contains(authoring),
            "learner completion listed authoring verb {authoring}: {stdout}"
        );
    }
}
