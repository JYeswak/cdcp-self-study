//! W1 (`bd-installability-sm4g.1`): the release binary resolves an
//! INSTALLED bundle. Measured here with `cdcp serve --no-open` (the
//! `study` verb is N.2 and is blocked on this bead).
//!
//! Proof is a relocated temp tree, not a container:
//!   1. copy `web/` out of the repo into `$XDG_DATA_HOME/cdcp/web`
//!   2. run the `cdcp` binary from `/tmp` with an isolated HOME
//!   3. (a) HTTP 200 + `<title>`
//!   4. (c) bundle removed → exit 4, stderr names the absolute path
//!   5. (d) the binary does not contain `CARGO_MANIFEST_DIR` of the
//!      former fallback sites; release builds contain zero `/Users/`
//!
//! An empty proof list is ERROR. Skipping (a)/(c)/(d) is ERROR.

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// Named proofs this file must run. Dropping a row is a vacuous pass.
const PROOFS: &[&str] = &[
    "a_relocated_serve_200",
    "c_missing_bundle_exit_4",
    "d_no_baked_users",
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
        "cdcp-w1-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    )
}

fn copy_dir(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap_or_else(|e| panic!("mkdir {}: {e}", dst.display()));
    for entry in fs::read_dir(src).unwrap_or_else(|e| panic!("read {}: {e}", src.display())) {
        let entry = entry.unwrap();
        let to = dst.join(entry.file_name());
        let ty = entry.file_type().unwrap();
        if ty.is_dir() {
            copy_dir(&entry.path(), &to);
        } else if ty.is_file() {
            fs::copy(entry.path(), &to).unwrap_or_else(|e| {
                panic!("copy {} -> {}: {e}", entry.path().display(), to.display())
            });
        }
    }
}

fn isolate_cmd(cmd: &mut Command, xdg: &Path, home: &Path) {
    cmd.env("XDG_DATA_HOME", xdg);
    cmd.env("HOME", home);
    cmd.env_remove("CDCP_HOME");
    cmd.env_remove("CDCP_REPO_ROOT");
    // Run from /tmp so a cwd-walk cannot find the source checkout.
    cmd.current_dir("/tmp");
}

fn http_get(hostport: &str, path: &str) -> (u16, String) {
    let mut stream =
        TcpStream::connect(hostport).unwrap_or_else(|e| panic!("connect {hostport}: {e}"));
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: {hostport}\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    let mut buf = String::new();
    stream.read_to_string(&mut buf).unwrap();
    let status = buf
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    (status, buf)
}

fn wait_for_listen_line(child: &mut std::process::Child) -> Result<String, String> {
    let stdout = child.stdout.take().ok_or("no stdout")?;
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        let mut acc = String::new();
        for line in reader.lines() {
            match line {
                Ok(l) => {
                    acc.push_str(&l);
                    acc.push('\n');
                    if l.contains("cdcp serve: http://") {
                        let _ = tx.send(Ok(acc));
                        return;
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(format!("read serve stdout: {e}")));
                    return;
                }
            }
        }
        let _ = tx.send(Err(format!(
            "serve exited before printing a listen URL\n{acc}"
        )));
    });
    match rx.recv_timeout(Duration::from_secs(10)) {
        Ok(v) => v,
        Err(_) => {
            let _ = child.kill();
            let err = child
                .stderr
                .take()
                .map(|mut s| {
                    let mut b = String::new();
                    let _ = s.read_to_string(&mut b);
                    b
                })
                .unwrap_or_default();
            Err(format!(
                "timeout waiting for serve listen line; stderr={err}"
            ))
        }
    }
}

fn parse_hostport(listen: &str) -> String {
    for line in listen.lines() {
        if let Some(rest) = line.strip_prefix("cdcp serve: http://") {
            let addr = rest.split('/').next().unwrap_or("");
            assert!(
                !addr.is_empty() && addr != "127.0.0.1:0",
                "listen line must name the bound addr: {line}"
            );
            return addr.to_string();
        }
    }
    panic!("no listen URL in:\n{listen}");
}

#[test]
fn proof_list_is_not_empty() {
    assert!(
        !PROOFS.is_empty(),
        "empty W1 proof list is ERROR — (a)(c)(d) unmeasured"
    );
    assert!(PROOFS.contains(&"a_relocated_serve_200"));
    assert!(PROOFS.contains(&"c_missing_bundle_exit_4"));
    assert!(PROOFS.contains(&"d_no_baked_users"));
}

#[test]
fn a_relocated_serve_200() {
    let root = workspace_root();
    let src_web = root.join("web");
    assert!(
        src_web.join("index.html").is_file(),
        "empty source web/ is ERROR — nothing to relocate"
    );
    let base = std::env::temp_dir().join(stamp());
    let xdg = base.join("xdg");
    let home = base.join("home");
    let dest_web = xdg.join("cdcp/web");
    copy_dir(&src_web, &dest_web);
    assert!(
        dest_web.join("index.html").is_file(),
        "relocated copy lost index.html"
    );

    let mut child = {
        let mut cmd = Command::new(cdcp_bin());
        isolate_cmd(&mut cmd, &xdg, &home);
        cmd.args(["serve", "--no-open", "--bind", "127.0.0.1:0"]);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.spawn()
            .unwrap_or_else(|e| panic!("spawn cdcp serve: {e}"))
    };

    let listen = match wait_for_listen_line(&mut child) {
        Ok(s) => s,
        Err(e) => {
            let _ = child.kill();
            let _ = child.wait();
            let _ = fs::remove_dir_all(&base);
            panic!("{e}");
        }
    };
    assert!(
        listen.contains("using installed root"),
        "must PRINT installed vs source-checkout: {listen}"
    );
    assert!(
        listen.contains("via XDG_DATA_HOME"),
        "must PRINT the precedence slot: {listen}"
    );
    assert!(
        listen.contains(&xdg.join("cdcp").display().to_string()),
        "announce must name the XDG home: {listen}"
    );

    let hostport = parse_hostport(&listen);
    let (status, body) = http_get(&hostport, "/");
    let _ = child.kill();
    let _ = child.wait();
    let _ = fs::remove_dir_all(&base);

    assert_eq!(status, 200, "GET / against relocated bundle: {body}");
    assert!(
        body.contains("<title>"),
        "200 body must contain <title>: {body}"
    );
}

#[test]
fn c_missing_bundle_exit_4() {
    let base = std::env::temp_dir().join(format!("{}-missing", stamp()));
    let xdg = base.join("xdg");
    let home = base.join("home");
    fs::create_dir_all(&xdg).unwrap();
    fs::create_dir_all(&home).unwrap();
    // Deliberately do NOT create $XDG_DATA_HOME/cdcp/web.

    let mut cmd = Command::new(cdcp_bin());
    isolate_cmd(&mut cmd, &xdg, &home);
    cmd.args(["serve", "--no-open", "--bind", "127.0.0.1:0"]);
    let out = cmd.output().unwrap_or_else(|e| panic!("spawn: {e}"));
    let stderr = String::from_utf8_lossy(&out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let looked = xdg.join("cdcp/web");
    let _ = fs::remove_dir_all(&base);

    assert_eq!(
        out.status.code(),
        Some(4),
        "missing bundle must exit 4, got {:?}\nstdout={stdout}\nstderr={stderr}",
        out.status.code()
    );
    assert!(
        stderr.contains("bundle not found") || stdout.contains("bundle not found"),
        "must name the miss: stdout={stdout} stderr={stderr}"
    );
    assert!(
        stderr.contains(&looked.display().to_string()),
        "stderr must contain the absolute path looked for ({})\nstderr={stderr}",
        looked.display()
    );
    assert!(
        looked.is_absolute(),
        "looked-for path must be absolute: {}",
        looked.display()
    );
}

#[test]
fn d_no_baked_users() {
    let bytes = fs::read(cdcp_bin()).unwrap_or_else(|e| panic!("read cdcp bin: {e}"));
    let hay = String::from_utf8_lossy(&bytes);

    // The five former fallbacks expanded CARGO_MANIFEST_DIR at compile time.
    // Those exact crate directories must not appear as baked literals.
    // (DWARF in debug builds may still mention src/ files under the crate.)
    for crate_name in [
        "cdcp_gate",
        "cdcp_learn",
        "cdcp_evidence",
        "cdcp_anki",
        "cdcp_registry_check",
    ] {
        let baked = format!("{}/crates/{crate_name}", workspace_root().display());
        // A DWARF path is `.../crates/cdcp_learn/src/lib.rs`. The old fallback
        // was the crate dir itself joined with `../..`. Look for the join
        // operand as a C-string: the crate directory followed by NUL or
        // `/../..` — not a src/ suffix.
        let as_c = format!("{baked}\0");
        let as_join = format!("{baked}/../..");
        assert!(
            !hay.contains(&as_c) && !hay.contains(&as_join),
            "cdcp binary still contains compile-time {crate_name} manifest path"
        );
        let _ = baked;
    }

    if !cfg!(debug_assertions) {
        let count = hay.matches("/Users/").count();
        assert_eq!(
            count, 0,
            "release cdcp contains {count} '/Users/' strings — CARGO_MANIFEST_DIR leaked"
        );
    }
}
