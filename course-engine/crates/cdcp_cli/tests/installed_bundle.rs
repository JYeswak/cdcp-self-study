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
use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

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

fn parse_http_status(buf: &str) -> u16 {
    buf.lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

fn retryable_io(kind: ErrorKind) -> bool {
    matches!(
        kind,
        ErrorKind::ConnectionReset
            | ErrorKind::ConnectionRefused
            | ErrorKind::ConnectionAborted
            | ErrorKind::BrokenPipe
            | ErrorKind::UnexpectedEof
            | ErrorKind::TimedOut
            | ErrorKind::WouldBlock
            | ErrorKind::Interrupted
            | ErrorKind::NotConnected
    )
}

/// One GET. Transport errors are returned so the caller can retry.
/// A status line already in `buf` after RST still counts — the 200 was sent.
fn http_get_once(hostport: &str, path: &str) -> std::io::Result<(u16, String)> {
    let mut stream = TcpStream::connect(hostport)?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    let _ = stream.set_nodelay(true);
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: {hostport}\r\nConnection: close\r\n\r\n"
    )?;
    // Half-close the write side so the server sees EOF after headers.
    let _ = stream.shutdown(std::net::Shutdown::Write);
    let mut buf = String::new();
    let read_res = stream.read_to_string(&mut buf);
    let status = parse_http_status(&buf);
    match read_res {
        Ok(_) => Ok((status, buf)),
        Err(_) if status != 0 => Ok((status, buf)),
        Err(e) => Err(e),
    }
}

fn child_stderr(child: &mut std::process::Child) -> String {
    let mut err = String::new();
    if let Some(mut s) = child.stderr.take() {
        let _ = s.read_to_string(&mut err);
    }
    format!("stderr={err:?}")
}

fn give_up(
    child: Option<&mut std::process::Child>,
    attempts: u32,
    deadline: Duration,
    last_err: &str,
) -> String {
    let ms = deadline.as_millis();
    if let Some(c) = child {
        match c.try_wait() {
            Ok(Some(st)) => {
                let code = st
                    .code()
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| st.to_string());
                let stderr = child_stderr(c);
                format!(
                    "child exited with status {code} (attempts={attempts}; last_err={last_err}); {stderr}"
                )
            }
            _ => format!(
                "listener never came up within {ms}ms (attempts={attempts}; last_err={last_err})"
            ),
        }
    } else {
        format!("listener never came up within {ms}ms (attempts={attempts}; last_err={last_err})")
    }
}

/// Bounded GET retry. ConnectionReset/Refused sleep-and-retry to `deadline`.
/// Timeout names exactly one cause: child-exited vs listener-never-came-up.
fn http_get_until(
    hostport: &str,
    path: &str,
    deadline: Duration,
    mut child: Option<&mut std::process::Child>,
) -> Result<(u16, String, u32), String> {
    let start = Instant::now();
    let mut attempts = 0u32;
    loop {
        attempts += 1;
        let last_err = match http_get_once(hostport, path) {
            Ok(pair) => return Ok((pair.0, pair.1, attempts)),
            Err(e) => {
                let last_err = e.to_string();
                if !retryable_io(e.kind()) && start.elapsed() >= deadline {
                    return Err(give_up(child, attempts, deadline, &last_err));
                }
                last_err
            }
        };

        if let Some(c) = child.as_deref_mut() {
            if let Ok(Some(st)) = c.try_wait() {
                let code = st
                    .code()
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| st.to_string());
                let stderr = child_stderr(c);
                return Err(format!(
                    "child exited with status {code} (attempts={attempts}; last_err={last_err}); {stderr}"
                ));
            }
        }

        if start.elapsed() >= deadline {
            return Err(give_up(child, attempts, deadline, &last_err));
        }
        let remain = deadline.saturating_sub(start.elapsed());
        thread::sleep(Duration::from_millis(20).min(remain));
    }
}

fn unused_loopback() -> String {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap_or_else(|e| panic!("bind ephemeral for dead-port probe: {e}"));
    let addr = listener.local_addr().unwrap();
    drop(listener);
    addr.to_string()
}

fn attempts_in(msg: &str) -> u32 {
    msg.split("attempts=")
        .nth(1)
        .and_then(|rest| {
            rest.chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse()
                .ok()
        })
        .unwrap_or(0)
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
                    // URL first, Ctrl-C second (after stdout flush). Wait for
                    // both so GET does not race a server still in println!.
                    if acc.contains("cdcp serve: http://") && acc.contains("Ctrl-C to stop") {
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
    let (status, body, attempts) =
        match http_get_until(&hostport, "/", Duration::from_secs(8), Some(&mut child)) {
            Ok(v) => v,
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = fs::remove_dir_all(&base);
                panic!("{e}");
            }
        };
    let _ = child.kill();
    let _ = child.wait();
    let _ = fs::remove_dir_all(&base);

    eprintln!("a_relocated_serve_200: GET / ok attempts={attempts} hostport={hostport}");
    assert_eq!(status, 200, "GET / against relocated bundle: {body}");
    assert!(
        body.contains("<title>"),
        "200 body must contain <title>: {body}"
    );
    assert!(
        attempts >= 1,
        "retry helper must report attempts used, got {attempts}"
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

/// L4: a dead port with a still-running child must name "listener never came up".
#[test]
fn retry_dead_port_names_listener_never_came_up() {
    let hostport = unused_loopback();
    let mut child = Command::new("sleep")
        .arg("30")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|e| panic!("spawn sleep: {e}"));
    let err = match http_get_until(&hostport, "/", Duration::from_millis(250), Some(&mut child)) {
        Ok(v) => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("dead port must not serve: {v:?}");
        }
        Err(e) => e,
    };
    let _ = child.kill();
    let _ = child.wait();
    assert!(
        err.contains("listener never came up"),
        "must name the no-listener cause: {err}"
    );
    assert!(
        !err.contains("child exited with status"),
        "live child must not be reported as exited: {err}"
    );
    let n = attempts_in(&err);
    assert!(n >= 2, "retry loop must execute (attempts>=2): {err}");
}

/// L4: a dead port after the child has already exited must name that exit.
#[test]
fn retry_dead_port_names_child_exited() {
    let hostport = unused_loopback();
    let mut child = Command::new("sh")
        .args(["-c", "exit 7"])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("spawn sh: {e}"));
    let waited = child.wait().unwrap_or_else(|e| panic!("wait sh: {e}"));
    assert_eq!(waited.code(), Some(7), "planted child must exit 7");
    let err = match http_get_until(&hostport, "/", Duration::from_millis(250), Some(&mut child)) {
        Ok(v) => panic!("dead port must not serve: {v:?}"),
        Err(e) => e,
    };
    assert!(
        err.contains("child exited with status 7"),
        "must name the child-exited cause: {err}"
    );
    assert!(
        !err.contains("listener never came up"),
        "dead child must not be reported as a missing listener: {err}"
    );
    assert!(
        attempts_in(&err) >= 1,
        "receipt must state attempts used: {err}"
    );
}
