//! G1: a stdout-only capture must not turn a failed `check.sh` run into a
//! green-looking transcript. The probe invokes check.sh's real `fail()` path;
//! it is opt-in and exits before the ordinary chain, so it cannot weaken CI.

use std::process::{Command, Stdio};

#[test]
fn check_failure_is_visible_on_stdout_and_stderr() {
    let engine = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new("sh")
        .arg(engine.join("scripts/check.sh"))
        .arg("--selftest-failure-visibility")
        .current_dir(&engine)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run check.sh failure-visibility probe");

    assert_eq!(output.status.code(), Some(2), "probe status: {output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let marker = "check.sh: FAIL: intentional failure-visibility probe";
    assert!(stdout.contains(marker), "stdout lost failure marker: {stdout}");
    assert!(stderr.contains(marker), "stderr lost failure marker: {stderr}");
}
