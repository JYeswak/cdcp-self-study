//! Local-only static server shared by `cdcp serve` and `cdcp study`.
//!
//! `serve` binds the requested address and fails if it is taken (that is the
//! measured operator surface — do not give it silent fallback).
//! `study` retries nearby ports and then `:0` so an occupied 8766 is not a
//! hard stop. Both print the address `local_addr()` actually bound.

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Documented learner default. `study` prefers this, then retries, then `:0`.
pub(crate) const DEFAULT_BIND: &str = "127.0.0.1:8766";

/// Ports tried above the preferred port before falling back to `:0`.
const FALLBACK_SPAN: u16 = 16;

/// Whether to open a browser after the listen URL is printed.
pub(crate) enum OpenMode {
    /// `cdcp serve`: never opens. `--no-open` is accepted as a no-op at the
    /// clap layer so W1 measurements stay valid.
    Serve,
    /// `cdcp study`: opens unless `--no-open`.
    Study { no_open: bool },
}

/// Resolve the bundle, bind, announce, optionally open, then serve until Ctrl-C.
pub(crate) fn run(explicit: Option<&Path>, preferred: &str, mode: OpenMode) -> Result<(), String> {
    let resolved = cdcp_root::resolve_from_env(explicit).map_err(|e| e.to_string())?;
    // Source-checkout vs installed must PRINT the chosen root. Silent
    // precedence is the next fooled certificate.
    println!("cdcp: {}", resolved.announce());
    let root = resolve_web_dir(&resolved)?;
    let fallback = matches!(mode, OpenMode::Study { .. });
    let listener = bind_listener(preferred, fallback)?;
    let addr = listener
        .local_addr()
        .map_err(|e| format!("local_addr after bind {preferred}: {e}"))?;
    let cmd = match mode {
        OpenMode::Serve => "serve",
        OpenMode::Study { .. } => "study",
    };
    let url = format!("http://{addr}/");
    println!("cdcp {cmd}: {url}  (root {})", root.display());
    println!("cdcp {cmd}: Ctrl-C to stop");
    if let OpenMode::Study { no_open: false } = mode {
        open_browser(&url);
    }
    serve_loop(listener, &root)
}

/// Bind `preferred`. When `fallback` is true and that address is taken, try
/// the next [`FALLBACK_SPAN`] ports on the same host, then host:0.
///
/// `fallback = false` is the serve contract: an explicit `--bind` that cannot
/// be taken is an error, not a silent move.
pub(crate) fn bind_listener(preferred: &str, fallback: bool) -> Result<TcpListener, String> {
    match TcpListener::bind(preferred) {
        Ok(listener) => Ok(listener),
        Err(first) if !fallback => Err(format!("bind {preferred}: {first}")),
        Err(first) => bind_fallback(preferred, first),
    }
}

fn bind_fallback(preferred: &str, first: std::io::Error) -> Result<TcpListener, String> {
    let (ip, preferred_port) = match preferred.parse::<SocketAddr>() {
        Ok(sa) => (sa.ip(), sa.port()),
        Err(_) => (IpAddr::V4(Ipv4Addr::LOCALHOST), 8766),
    };
    if preferred_port > 0 {
        let start = preferred_port.saturating_add(1);
        let end = start.saturating_add(FALLBACK_SPAN);
        for port in start..end {
            if let Ok(listener) = TcpListener::bind(SocketAddr::new(ip, port)) {
                return Ok(listener);
            }
        }
    }
    TcpListener::bind(SocketAddr::new(ip, 0)).map_err(|second| {
        format!("bind {preferred} failed ({first}); fallback :0 failed ({second})")
    })
}

fn resolve_web_dir(resolved: &cdcp_root::ResolvedRoot) -> Result<PathBuf, String> {
    let root = resolved.web_dir();
    if !root.is_dir() {
        return Err(format!(
            "{}: {}",
            cdcp_root::BUNDLE_NOT_FOUND,
            root.display()
        ));
    }
    root.canonicalize()
        .map_err(|e| format!("{}: {} ({e})", cdcp_root::BUNDLE_NOT_FOUND, root.display()))
}

/// `open` on macOS, `xdg-open` elsewhere. A missing opener is a warning, not
/// a failed command — the URL is already on stdout.
///
/// Resolved via PATH (first executable hit) so a test can plant `open` /
/// `xdg-open` ahead of `/usr/bin`. `Command::new("open")` on macOS can
/// otherwise skip a prepended PATH entry.
pub(crate) fn open_browser(url: &str) {
    let bin = resolve_opener();
    match Command::new(&bin)
        .arg(url)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(_) => {}
        Err(e) => {
            eprintln!(
                "cdcp study: warning: could not open a browser ({}: {e}); URL is {url}",
                bin.display()
            );
        }
    }
}

fn opener_bin() -> &'static str {
    if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    }
}

fn resolve_opener() -> PathBuf {
    let name = opener_bin();
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            let cand = dir.join(name);
            if opener_is_executable(&cand) {
                return cand;
            }
        }
    }
    PathBuf::from(name)
}

fn opener_is_executable(path: &Path) -> bool {
    let Ok(meta) = fs::metadata(path) else {
        return false;
    };
    if !meta.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        meta.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

fn serve_loop(listener: TcpListener, root: &Path) -> Result<(), String> {
    // The guards in this loop are per-CONNECTION liveness, not verdicts about an
    // artifact: a dropped socket or an unreadable request line must not take the
    // server down, and neither one grants access to anything. The access verdict
    // is the traversal guard below, which is fail-closed.
    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(s) => s,
            Err(_) => continue,
        };
        let mut line = String::new();
        if BufReader::new(&stream).read_line(&mut line).is_err() {
            continue;
        }
        let mut parts = line.split_whitespace();
        // Fail-closed defaults: a request line with no verb yields "", which is
        // neither GET nor HEAD and is answered 405; a request line with no target
        // yields "/", which is served as index.html or 404s. Neither default can
        // widen what is reachable.
        let method = parts.next().unwrap_or("");
        let raw = parts.next().unwrap_or("/");
        if method != "GET" && method != "HEAD" {
            let _ =
                stream.write_all(b"HTTP/1.1 405 Method Not Allowed\r\nContent-Length: 0\r\n\r\n");
            continue;
        }
        let path = raw.split('?').next().unwrap_or("/");
        let rel = if path == "/" {
            "index.html"
        } else {
            path.trim_start_matches('/')
        };

        // Path traversal guard: resolve, then require the result stay under root.
        // This IS a verdict, and it is fail-CLOSED in both legs: a canonicalize
        // failure becomes None via `.ok()` (404, never "assume it is fine"), and
        // the `starts_with` filter turns any escape into None as well. A file
        // that cannot be resolved is refused, not served.
        let candidate = root.join(rel);
        let resolved = candidate
            .canonicalize()
            .ok()
            .filter(|p| p.starts_with(root));
        let (status, body, ctype) = match resolved {
            Some(p) if p.is_file() => match fs::read(&p) {
                Ok(bytes) => ("200 OK", bytes, content_type(&p)),
                Err(_) => (
                    "500 Internal Server Error",
                    b"read error".to_vec(),
                    "text/plain",
                ),
            },
            _ => ("404 Not Found", b"not found".to_vec(), "text/plain"),
        };
        let head = format!(
            "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\n\
             X-Content-Type-Options: nosniff\r\nConnection: close\r\n\r\n",
            body.len()
        );
        let _ = stream.write_all(head.as_bytes());
        if method == "GET" {
            let _ = stream.write_all(&body);
        }
    }
    Ok(())
}

/// Content-Type for a served file. The `unwrap_or("")` fallback is a labelling
/// decision, not a verdict: an unknown or absent extension becomes
/// `application/octet-stream`, which (with the `nosniff` header above) is the
/// conservative answer. It cannot make an unreachable file reachable.
fn content_type(p: &Path) -> &'static str {
    match p.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "wasm" => "application/wasm",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_bind_is_the_documented_8766() {
        assert_eq!(DEFAULT_BIND, "127.0.0.1:8766");
        let sa: SocketAddr = DEFAULT_BIND.parse().unwrap();
        assert_eq!(sa.port(), 8766);
    }

    #[test]
    fn bind_without_fallback_fails_when_occupied() {
        let held = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = held.local_addr().unwrap().to_string();
        let err = bind_listener(&addr, false).unwrap_err();
        assert!(
            err.contains("bind") && err.contains(&addr),
            "serve-shaped error must name the requested address: {err}"
        );
    }

    #[test]
    fn bind_with_fallback_picks_a_different_port() {
        let held = TcpListener::bind("127.0.0.1:0").unwrap();
        let occupied = held.local_addr().unwrap();
        let listener = bind_listener(&occupied.to_string(), true)
            .unwrap_or_else(|e| panic!("study-shaped fallback must bind: {e}"));
        let got = listener.local_addr().unwrap();
        assert_ne!(
            got.port(),
            occupied.port(),
            "fallback must not report the occupied port"
        );
        assert_eq!(got.ip(), occupied.ip());
        assert_ne!(got.port(), 0, "local_addr after :0 must be a real port");
    }

    #[test]
    fn opener_bin_is_platform_native() {
        if cfg!(target_os = "macos") {
            assert_eq!(opener_bin(), "open");
        } else {
            assert_eq!(opener_bin(), "xdg-open");
        }
    }
}
