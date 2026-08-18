//! W2 (`bd-installability-sm4g.2`): `cdcp study` is the product command.
//!
//! Proofs this file must run. An empty list is ERROR.
//!   1. default 8766 occupied → study still binds, prints a *different* port,
//!      and that port is the live listener (printed-URL-only is vacuous)
//!   2. `--no-open` does not spawn a browser
//!   3. bundle absent → exit 4, stderr names the absolute path
//!   4. `serve` still fails on an occupied `--bind` (do not inherit fallback)
//!   5. bare `cdcp` advertises `study`

use std::fs;
use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

/// Named proofs. Dropping a row is a vacuous pass.
const PROOFS: &[&str] = &[
    "occupied_default_port_binds_different_listener",
    "no_open_does_not_spawn_browser",
    "missing_bundle_exits_4_naming_path",
    "serve_still_fails_when_bind_is_occupied",
    "orientation_advertises_study",
];

const DEFAULT_BIND: &str = "127.0.0.1:8766";
const DEFAULT_PORT: u16 = 8766;
const OCCUPIER_TOKEN: &str = "cdcp-study-occupier-token-sm4g2";
const STUDY_TOKEN: &str = "cdcp-study-listener-token-sm4g2";

fn cdcp_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cdcp"))
}

fn stamp() -> String {
    format!(
        "cdcp-w2-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    )
}

fn plant_bundle(base: &Path) -> PathBuf {
    let root = base.join("bundle");
    let web = root.join("web");
    fs::create_dir_all(&web).unwrap();
    fs::write(
        web.join("index.html"),
        format!("<!doctype html><title>{STUDY_TOKEN}</title>\n"),
    )
    .unwrap();
    root
}

fn isolate_cmd(cmd: &mut Command, xdg: &Path, home: &Path) {
    cmd.env("XDG_DATA_HOME", xdg);
    cmd.env("HOME", home);
    cmd.env_remove("CDCP_HOME");
    cmd.env_remove("CDCP_REPO_ROOT");
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
/// Same helper as `installed_bundle` (330d28d / sm4g.21).
///
/// `half_close` is for *our* drain-aware listener. Do not SHUT_WR against a
/// foreign :8766 occupant — measured: Python-style serve answers curl but
/// returns 0 bytes after client write-shutdown.
fn http_get_once(hostport: &str, path: &str, half_close: bool) -> std::io::Result<(u16, String)> {
    let mut stream = TcpStream::connect(hostport)?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    let _ = stream.set_nodelay(true);
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: {hostport}\r\nConnection: close\r\n\r\n"
    )?;
    if half_close {
        let _ = stream.shutdown(Shutdown::Write);
    }
    let mut buf = String::new();
    let read_res = stream.read_to_string(&mut buf);
    let status = parse_http_status(&buf);
    match read_res {
        Ok(_) if status != 0 => Ok((status, buf)),
        Err(_) if status != 0 => Ok((status, buf)),
        Ok(_) => Err(std::io::Error::new(
            ErrorKind::UnexpectedEof,
            "no HTTP status before EOF",
        )),
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
    half_close: bool,
) -> Result<(u16, String, u32), String> {
    let start = Instant::now();
    let mut attempts = 0u32;
    loop {
        attempts += 1;
        let last_err = match http_get_once(hostport, path, half_close) {
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

fn http_get(hostport: &str, path: &str) -> (u16, String) {
    // Default: no SHUT_WR. Occupier / foreign :8766 may not be our serve.
    match http_get_until(hostport, path, Duration::from_secs(8), None, false) {
        Ok((status, body, _)) => (status, body),
        Err(e) => panic!("GET {hostport}{path}: {e}"),
    }
}

fn wait_for_listen_line(child: &mut std::process::Child, prefix: &str) -> Result<String, String> {
    let stdout = child.stdout.take().ok_or("no stdout")?;
    let needle = format!("{prefix} http://");
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        let mut acc = String::new();
        for line in reader.lines() {
            match line {
                Ok(l) => {
                    acc.push_str(&l);
                    acc.push('\n');
                    // URL is printed first, Ctrl-C second. Return after both
                    // so a Ctrl-C assertion is not a race on the first line.
                    if acc.contains(&needle) && acc.contains("Ctrl-C to stop") {
                        let _ = tx.send(Ok(acc));
                        return;
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(format!("read stdout: {e}")));
                    return;
                }
            }
        }
        let _ = tx.send(Err(format!("exited before printing a listen URL\n{acc}")));
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
                "timeout waiting for {prefix} listen line; stderr={err}"
            ))
        }
    }
}

fn parse_hostport(listen: &str, prefix: &str) -> String {
    let needle = format!("{prefix} http://");
    for line in listen.lines() {
        if let Some(rest) = line.find(&needle).map(|i| &line[i + needle.len()..]) {
            let addr = rest.split('/').next().unwrap_or("");
            assert!(
                !addr.is_empty() && !addr.ends_with(":0"),
                "listen line must name the bound addr, not :0: {line}"
            );
            return addr.to_string();
        }
    }
    panic!("no listen URL in:\n{listen}");
}

fn parse_port(hostport: &str) -> u16 {
    hostport
        .rsplit_once(':')
        .and_then(|(_, p)| p.parse().ok())
        .unwrap_or_else(|| panic!("no port in {hostport}"))
}

/// Make sure 127.0.0.1:8766 is occupied. If this test can bind it, serve a
/// unique token. If something else already holds it (the live product case),
/// keep that occupant and snapshot its body so we can prove study did not
/// steal the port. Killing a foreign listener is not our job.
fn occupy_default_port() -> Occupier {
    match TcpListener::bind(DEFAULT_BIND) {
        Ok(listener) => {
            let addr = listener.local_addr().unwrap();
            assert_eq!(
                addr.port(),
                DEFAULT_PORT,
                "occupier must hold the documented default, got {addr}"
            );
            thread::spawn(move || {
                for stream in listener.incoming().flatten() {
                    occupier_answer(stream);
                }
            });
            let (status, body) = http_get(DEFAULT_BIND, "/");
            assert_eq!(status, 200, "occupier must answer GET /: {body}");
            assert!(
                body.contains(OCCUPIER_TOKEN),
                "occupier body missing token: {body}"
            );
            Occupier { ours: true }
        }
        Err(e) => {
            let (_status, snapshot) = http_get(DEFAULT_BIND, "/");
            assert!(
                !snapshot.contains(STUDY_TOKEN),
                "pre-existing {DEFAULT_BIND} occupant already serves this test's study token ({e})"
            );
            Occupier { ours: false }
        }
    }
}

struct Occupier {
    ours: bool,
}

/// Occupier must drain the request the same way `http_serve` does. POSIX
/// `close()` on unread recv data is RST — W2's first GET of 8766 then
/// panics at `read_to_string` even when the 200 was fully sent.
fn occupier_drain_request(stream: &TcpStream) -> bool {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    match reader.read_line(&mut line) {
        Ok(0) | Err(_) => return false,
        Ok(_) => {}
    }
    loop {
        let mut hdr = String::new();
        match reader.read_line(&mut hdr) {
            Ok(0) | Err(_) => break,
            Ok(_) if hdr == "\r\n" || hdr == "\n" || hdr.is_empty() => break,
            Ok(_) => {}
        }
    }
    true
}

fn occupier_answer(mut stream: TcpStream) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));
    let _ = stream.set_nodelay(true);
    if !occupier_drain_request(&stream) {
        occupier_finish(&mut stream);
        return;
    }
    let body = OCCUPIER_TOKEN.as_bytes();
    let _ = write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(body);
    occupier_finish(&mut stream);
}

fn occupier_finish(stream: &mut TcpStream) {
    let _ = stream.shutdown(Shutdown::Write);
    let _ = stream.set_nonblocking(true);
    let mut sink = [0u8; 256];
    loop {
        match stream.read(&mut sink) {
            Ok(0) => break,
            Ok(_) => {}
            Err(e)
                if matches!(
                    e.kind(),
                    ErrorKind::WouldBlock | ErrorKind::TimedOut | ErrorKind::Interrupted
                ) =>
            {
                break;
            }
            Err(_) => break,
        }
    }
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
        "empty W2 proof list is ERROR — study unmeasured"
    );
    assert!(PROOFS.contains(&"occupied_default_port_binds_different_listener"));
    assert!(PROOFS.contains(&"no_open_does_not_spawn_browser"));
    assert!(PROOFS.contains(&"missing_bundle_exits_4_naming_path"));
    assert!(PROOFS.contains(&"serve_still_fails_when_bind_is_occupied"));
    assert!(PROOFS.contains(&"orientation_advertises_study"));
}

#[test]
fn occupied_default_port_binds_different_listener() {
    let occupier = occupy_default_port();
    let base = std::env::temp_dir().join(stamp());
    let bundle = plant_bundle(&base);
    let xdg = base.join("xdg");
    let home = base.join("home");
    fs::create_dir_all(&xdg).unwrap();
    fs::create_dir_all(&home).unwrap();

    let mut child = {
        let mut cmd = Command::new(cdcp_bin());
        isolate_cmd(&mut cmd, &xdg, &home);
        cmd.args(["study", "--no-open", "--root"]);
        cmd.arg(&bundle);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.spawn()
            .unwrap_or_else(|e| panic!("spawn cdcp study: {e}"))
    };

    let listen = match wait_for_listen_line(&mut child, "cdcp study:") {
        Ok(s) => s,
        Err(e) => {
            let _ = child.kill();
            let _ = child.wait();
            let _ = fs::remove_dir_all(&base);
            panic!("{e}");
        }
    };
    assert!(
        listen.contains("Ctrl-C to stop"),
        "Ctrl-C must be discoverable: {listen}"
    );

    let hostport = parse_hostport(&listen, "cdcp study:");
    let printed_port = parse_port(&hostport);
    assert_ne!(
        printed_port, DEFAULT_PORT,
        "printed URL must not claim the occupied default {DEFAULT_BIND}: {listen}"
    );

    // Listener proof: GET the printed address. A printed-only claim is vacuous.
    // Retry + accept-status-after-RST matches installed_bundle (330d28d).
    let (status, body, attempts) = match http_get_until(
        &hostport,
        "/",
        Duration::from_secs(8),
        Some(&mut child),
        true,
    ) {
        Ok(v) => v,
        Err(e) => {
            let _ = child.kill();
            let _ = child.wait();
            let _ = fs::remove_dir_all(&base);
            panic!("{e}");
        }
    };
    eprintln!("W2 occupied-port GET printed URL attempts={attempts} hostport={hostport}");
    assert_eq!(
        status, 200,
        "GET printed URL {hostport} must hit study's listener: {body}"
    );
    assert!(
        body.contains(STUDY_TOKEN),
        "printed URL listener is not study (missing {STUDY_TOKEN}): {body}"
    );
    assert!(
        !body.contains(OCCUPIER_TOKEN),
        "printed URL hit the occupier, not study: {body}"
    );

    // Occupier is still the listener on 8766 — study did not steal it.
    let (occ_status, occ_body) = http_get(DEFAULT_BIND, "/");
    assert_eq!(
        occ_status, 200,
        "occupier on {DEFAULT_BIND} must still answer: {occ_body}"
    );
    if occupier.ours {
        assert!(
            occ_body.contains(OCCUPIER_TOKEN),
            "8766 is no longer the occupier — study stole the default port: {occ_body}"
        );
    }
    assert!(
        !occ_body.contains(STUDY_TOKEN),
        "8766 is serving study; fallback did not move: {occ_body}"
    );

    eprintln!(
        "W2 occupied-port: default={DEFAULT_PORT} printed={printed_port} ours={} study_url=http://{hostport}/",
        occupier.ours
    );

    let _ = child.kill();
    let _ = child.wait();
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn no_open_does_not_spawn_browser() {
    let base = std::env::temp_dir().join(format!("{}-noopen", stamp()));
    let bundle = plant_bundle(&base);
    let xdg = base.join("xdg");
    let home = base.join("home");
    let bin_dir = base.join("bin");
    let marker = base.join("opener-marker");
    fs::create_dir_all(&xdg).unwrap();
    fs::create_dir_all(&home).unwrap();
    plant_fake_opener(&bin_dir, &marker);

    let mut child = {
        let mut cmd = Command::new(cdcp_bin());
        isolate_cmd(&mut cmd, &xdg, &home);
        cmd.env("PATH", prepend_path(&bin_dir));
        cmd.args(["study", "--no-open", "--bind", "127.0.0.1:0", "--root"]);
        cmd.arg(&bundle);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.spawn()
            .unwrap_or_else(|e| panic!("spawn cdcp study --no-open: {e}"))
    };

    let listen = match wait_for_listen_line(&mut child, "cdcp study:") {
        Ok(s) => s,
        Err(e) => {
            let _ = child.kill();
            let _ = child.wait();
            let _ = fs::remove_dir_all(&base);
            panic!("{e}");
        }
    };
    let hostport = parse_hostport(&listen, "cdcp study:");
    // Server is up ⇒ the opener decision has already been made.
    let (status, _) = http_get(&hostport, "/");
    assert_eq!(status, 200, "study --no-open must still serve");
    assert!(
        !marker.exists(),
        "--no-open must not spawn a browser; opener marker exists at {}",
        marker.display()
    );

    let _ = child.kill();
    let _ = child.wait();
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn open_without_no_open_invokes_opener() {
    let base = std::env::temp_dir().join(format!("{}-open", stamp()));
    let bundle = plant_bundle(&base);
    let xdg = base.join("xdg");
    let home = base.join("home");
    let bin_dir = base.join("bin");
    let marker = base.join("opener-marker");
    fs::create_dir_all(&xdg).unwrap();
    fs::create_dir_all(&home).unwrap();
    plant_fake_opener(&bin_dir, &marker);

    let mut child = {
        let mut cmd = Command::new(cdcp_bin());
        isolate_cmd(&mut cmd, &xdg, &home);
        cmd.env("PATH", prepend_path(&bin_dir));
        cmd.args(["study", "--bind", "127.0.0.1:0", "--root"]);
        cmd.arg(&bundle);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.spawn()
            .unwrap_or_else(|e| panic!("spawn cdcp study: {e}"))
    };

    let listen = match wait_for_listen_line(&mut child, "cdcp study:") {
        Ok(s) => s,
        Err(e) => {
            let _ = child.kill();
            let _ = child.wait();
            let _ = fs::remove_dir_all(&base);
            panic!("{e}");
        }
    };
    let hostport = parse_hostport(&listen, "cdcp study:");
    let (status, _) = http_get(&hostport, "/");
    assert_eq!(status, 200);

    // Opener is spawned, not waited. Give the shell script a beat to write.
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    while !marker.exists() && std::time::Instant::now() < deadline {
        thread::sleep(Duration::from_millis(20));
    }
    let recorded = fs::read_to_string(&marker).unwrap_or_default();
    let _ = child.kill();
    let _ = child.wait();
    let _ = fs::remove_dir_all(&base);
    assert!(
        recorded.contains(&format!("http://{hostport}/")),
        "unset --no-open must invoke the platform opener with the bound URL\nmarker={recorded:?}\nlisten={listen}"
    );
}

#[test]
fn missing_bundle_exits_4_naming_path() {
    let base = std::env::temp_dir().join(format!("{}-missing", stamp()));
    let xdg = base.join("xdg");
    let home = base.join("home");
    fs::create_dir_all(&xdg).unwrap();
    fs::create_dir_all(&home).unwrap();

    let mut cmd = Command::new(cdcp_bin());
    isolate_cmd(&mut cmd, &xdg, &home);
    cmd.current_dir("/tmp");
    cmd.args(["study", "--no-open"]);
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
}

#[test]
fn serve_still_fails_when_bind_is_occupied() {
    // Do not give serve the study fallback. An explicit --bind that is taken
    // is still a bind error (the measured serve defect; study is the fix).
    let held = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = held.local_addr().unwrap().to_string();
    let base = std::env::temp_dir().join(format!("{}-serve", stamp()));
    let bundle = plant_bundle(&base);
    let xdg = base.join("xdg");
    let home = base.join("home");
    fs::create_dir_all(&xdg).unwrap();
    fs::create_dir_all(&home).unwrap();

    let mut cmd = Command::new(cdcp_bin());
    isolate_cmd(&mut cmd, &xdg, &home);
    cmd.args(["serve", "--no-open", "--root"]);
    cmd.arg(&bundle);
    cmd.args(["--bind", &addr]);
    let out = cmd.output().unwrap_or_else(|e| panic!("spawn serve: {e}"));
    let stderr = String::from_utf8_lossy(&out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let _ = fs::remove_dir_all(&base);
    drop(held);

    assert_ne!(
        out.status.code(),
        Some(0),
        "serve must not silently move off an occupied --bind\nstdout={stdout}\nstderr={stderr}"
    );
    assert!(
        stderr.contains("bind") || stdout.contains("bind"),
        "serve occupied-bind error must mention bind: stdout={stdout} stderr={stderr}"
    );
}

#[test]
fn orientation_advertises_study() {
    let out = Command::new(cdcp_bin())
        .output()
        .unwrap_or_else(|e| panic!("spawn bare cdcp: {e}"));
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("cdcp study"),
        "bare cdcp must advertise the product command: {stdout}"
    );
}
