//! Single root resolver for an installed `cdcp` tree and a source checkout.
//!
//! Resolution order (product):
//! `--root` > `CDCP_HOME` > `$XDG_DATA_HOME/cdcp` > `~/.local/share/cdcp` > cwd walk.
//!
//! A source checkout is the directory holding [`ENGINE_ANCHOR`]. An installed
//! home is the directory holding [`BUNDLE_ANCHOR`]. There is no compile-time
//! crate-directory fallback: a binary that cannot find a live tree must fail,
//! naming the absolute path it looked for.
//!
//! Authoring/gates keep [`walk_engine_root`]: walk up from a start path for
//! the source-checkout anchor only. No env override (that was D6). No
//! compile-time escape.
#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

/// Presence of this file defines a source-checkout engine root.
pub const ENGINE_ANCHOR: &str = "registries/claims.toml";

/// Presence of this file defines an installed home (`$CDCP_HOME/web/index.html`).
/// `registries/claims.toml` is deliberately not installed.
pub const BUNDLE_ANCHOR: &str = "web/index.html";

/// Upward walk budget. Unified at 12; the registry-check copy used to stop at 8.
pub const WALK_LEVELS: usize = 12;

/// Process exit when the learner bundle cannot be found. Named so a bind
/// failure (1) cannot be confused with a missing tree.
pub const EXIT_BUNDLE_MISSING: u8 = 4;

/// Token interpolated into every missing-bundle error. Deleting it makes
/// the matching selftest non-zero.
pub const BUNDLE_NOT_FOUND: &str = "bundle not found";

/// How the resolver classified the tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootKind {
    /// `registries/claims.toml` is present. `web/` is the learner bundle.
    SourceCheckout,
    /// Installed home (`web/index.html`) or a web directory itself (`index.html`).
    Installed,
}

impl RootKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SourceCheckout => "source-checkout",
            Self::Installed => "installed",
        }
    }
}

/// Which slot in the precedence list produced the root.
///
/// Silent precedence is a fooled certificate: the caller must print this.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Via {
    Explicit,
    CdcpHome,
    XdgDataHome,
    LocalShare,
    CwdWalk,
}

impl Via {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Explicit => "--root",
            Self::CdcpHome => "CDCP_HOME",
            Self::XdgDataHome => "XDG_DATA_HOME",
            Self::LocalShare => "~/.local/share/cdcp",
            Self::CwdWalk => "cwd-walk",
        }
    }
}

/// A resolved CDCP home / engine root, plus the directory `cdcp serve` should bind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRoot {
    /// Engine root (source) or CDCP home (installed). When `--root` pointed at
    /// the web directory itself, this is that directory.
    pub path: PathBuf,
    /// Directory containing `index.html` — the HTTP document root.
    pub web: PathBuf,
    pub kind: RootKind,
    pub via: Via,
}

impl ResolvedRoot {
    /// One line naming kind, path, and via. Must be printed by the product path.
    pub fn announce(&self) -> String {
        format!(
            "using {} root {} (via {})",
            self.kind.as_str(),
            self.path.display(),
            self.via.as_str()
        )
    }

    pub fn web_dir(&self) -> &Path {
        &self.web
    }
}

/// Inputs the resolver reads. Tests construct this directly so they do not
/// race on process-global `CDCP_HOME` / `XDG_DATA_HOME` / `HOME`.
#[derive(Debug, Clone)]
pub struct ResolveEnv {
    pub explicit: Option<PathBuf>,
    pub cdcp_home: Option<PathBuf>,
    pub xdg_data_home: Option<PathBuf>,
    pub home: Option<PathBuf>,
    pub cwd: PathBuf,
}

impl ResolveEnv {
    /// Snapshot the process environment. `explicit` starts unset.
    pub fn from_process() -> Result<Self, RootError> {
        Ok(Self {
            explicit: None,
            cdcp_home: nonempty_env("CDCP_HOME"),
            xdg_data_home: nonempty_env("XDG_DATA_HOME"),
            home: nonempty_env("HOME"),
            cwd: std::env::current_dir().map_err(|e| RootError::Io {
                detail: format!("cwd: {e}"),
            })?,
        })
    }
}

/// Why a root could not be resolved.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum RootError {
    #[error("{BUNDLE_NOT_FOUND}: {looked_for}")]
    BundleMissing { looked_for: String },
    #[error("could not locate the course-engine root (no {ENGINE_ANCHOR} at or above {start})")]
    EngineMissing { start: String },
    #[error("{detail}")]
    Io { detail: String },
}

impl RootError {
    pub fn exit_code(&self) -> u8 {
        match self {
            Self::BundleMissing { .. } => EXIT_BUNDLE_MISSING,
            Self::EngineMissing { .. } => EXIT_BUNDLE_MISSING,
            Self::Io { .. } => EXIT_BUNDLE_MISSING,
        }
    }

    fn bundle(looked_for: PathBuf) -> Self {
        Self::BundleMissing {
            looked_for: looked_for.display().to_string(),
        }
    }
}

/// Product resolver. `--root` / `CDCP_HOME` fail closed when set but unusable;
/// XDG and `~/.local/share/cdcp` are optional slots (skip if absent) then cwd walk.
pub fn resolve(env: &ResolveEnv) -> Result<ResolvedRoot, RootError> {
    if let Some(raw) = env.explicit.as_ref() {
        let path = absolute(raw, &env.cwd);
        return classify(&path)
            .map(|mut r| {
                r.via = Via::Explicit;
                r
            })
            .ok_or_else(|| RootError::bundle(looked_for_bundle(&path)));
    }

    if let Some(raw) = env.cdcp_home.as_ref() {
        let path = absolute(raw, &env.cwd);
        return classify(&path)
            .map(|mut r| {
                r.via = Via::CdcpHome;
                r
            })
            .ok_or_else(|| RootError::bundle(looked_for_bundle(&path)));
    }

    let mut named: Vec<PathBuf> = Vec::new();

    if let Some(xdg) = env.xdg_data_home.as_ref() {
        let home = absolute(xdg, &env.cwd).join("cdcp");
        named.push(home.join("web"));
        if let Some(mut r) = classify(&home) {
            r.via = Via::XdgDataHome;
            return Ok(r);
        }
    }

    if let Some(home) = env.home.as_ref() {
        let local = absolute(home, &env.cwd).join(".local/share/cdcp");
        named.push(local.join("web"));
        if let Some(mut r) = classify(&local) {
            r.via = Via::LocalShare;
            return Ok(r);
        }
    }

    if let Some(mut r) = walk_any(&env.cwd) {
        r.via = Via::CwdWalk;
        return Ok(r);
    }

    let looked = named
        .into_iter()
        .next()
        .unwrap_or_else(|| env.cwd.join("web"));
    Err(RootError::bundle(looked))
}

/// Snapshot process env, apply `explicit`, resolve.
pub fn resolve_from_env(explicit: Option<&Path>) -> Result<ResolvedRoot, RootError> {
    let mut env = ResolveEnv::from_process()?;
    env.explicit = explicit.map(Path::to_path_buf);
    resolve(&env)
}

/// Walk up from `start` for [`ENGINE_ANCHOR`] only. No env. No compile-time path.
pub fn walk_engine_root(start: &Path) -> Result<PathBuf, RootError> {
    walk_up(start, |dir| dir.join(ENGINE_ANCHOR).is_file()).ok_or_else(|| {
        RootError::EngineMissing {
            start: start.display().to_string(),
        }
    })
}

fn classify(dir: &Path) -> Option<ResolvedRoot> {
    if dir.join(ENGINE_ANCHOR).is_file() {
        return Some(ResolvedRoot {
            path: dir.to_path_buf(),
            web: dir.join("web"),
            kind: RootKind::SourceCheckout,
            via: Via::CwdWalk,
        });
    }
    if dir.join(BUNDLE_ANCHOR).is_file() {
        return Some(ResolvedRoot {
            path: dir.to_path_buf(),
            web: dir.join("web"),
            kind: RootKind::Installed,
            via: Via::CwdWalk,
        });
    }
    if dir.join("index.html").is_file() {
        return Some(ResolvedRoot {
            path: dir.to_path_buf(),
            web: dir.to_path_buf(),
            kind: RootKind::Installed,
            via: Via::CwdWalk,
        });
    }
    None
}

fn walk_any(start: &Path) -> Option<ResolvedRoot> {
    let found = walk_up(start, |dir| classify(dir).is_some())?;
    classify(&found)
}

fn walk_up(start: &Path, pred: impl Fn(&Path) -> bool) -> Option<PathBuf> {
    let mut cur = start.to_path_buf();
    if cur.is_file() {
        cur.pop();
    }
    for _ in 0..WALK_LEVELS {
        if pred(&cur) {
            return Some(cur);
        }
        if !cur.pop() {
            break;
        }
    }
    None
}

fn absolute(p: &Path, cwd: &Path) -> PathBuf {
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        cwd.join(p)
    }
}

fn looked_for_bundle(path: &Path) -> PathBuf {
    if path.file_name().is_some_and(|n| n == "web" || n == "web/") {
        path.to_path_buf()
    } else {
        path.join("web")
    }
}

fn nonempty_env(key: &str) -> Option<PathBuf> {
    std::env::var_os(key)
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static CASE_HITS: AtomicUsize = AtomicUsize::new(0);

    /// Named precedence / failure cases. An empty list is ERROR.
    const CASES: &[&str] = &[
        "explicit_beats_cdcp_home",
        "cdcp_home_beats_xdg",
        "xdg_beats_local_share",
        "local_share_beats_cwd_walk",
        "cwd_walk_source_checkout",
        "cwd_walk_installed",
        "missing_names_absolute_xdg_web",
        "cdcp_home_invalid_does_not_fall_through",
        "walk_engine_root_rejects_installed_only",
        "walk_engine_root_finds_nested_source",
        "cdcp_repo_root_is_not_an_override",
        "announce_names_kind_and_via",
        "production_has_no_cargo_manifest_dir",
        "delegates_dropped_compile_time_fallback",
    ];

    fn hit(name: &str) {
        assert!(
            CASES.contains(&name),
            "test {name} is not in CASES — the anti-vacuous list drifted"
        );
        CASE_HITS.fetch_add(1, Ordering::SeqCst);
    }

    fn write(path: &Path, body: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, body).unwrap();
    }

    fn plant_source(dir: &Path) {
        write(&dir.join(ENGINE_ANCHOR), "schema_version = 1\n");
        write(&dir.join(BUNDLE_ANCHOR), "<title>source</title>\n");
    }

    fn plant_installed(dir: &Path) {
        write(&dir.join(BUNDLE_ANCHOR), "<title>installed</title>\n");
    }

    fn env_at(cwd: &Path) -> ResolveEnv {
        ResolveEnv {
            explicit: None,
            cdcp_home: None,
            xdg_data_home: None,
            home: None,
            cwd: cwd.to_path_buf(),
        }
    }

    #[test]
    fn cases_list_is_not_empty() {
        assert!(
            !CASES.is_empty(),
            "empty resolver case list is ERROR — nothing was checked"
        );
        assert!(
            CASES.len() >= 12,
            "resolver case list shrank below the planted floor: {}",
            CASES.len()
        );
    }

    #[test]
    fn explicit_beats_cdcp_home() {
        hit("explicit_beats_cdcp_home");
        let td = tempfile::tempdir().unwrap();
        let expl = td.path().join("explicit");
        let home = td.path().join("home");
        plant_installed(&expl);
        plant_source(&home);
        let mut env = env_at(td.path());
        env.explicit = Some(expl.clone());
        env.cdcp_home = Some(home);
        let r = resolve(&env).unwrap();
        assert_eq!(r.kind, RootKind::Installed);
        assert_eq!(r.via, Via::Explicit);
        assert_eq!(r.path, expl);
        assert_eq!(r.web, expl.join("web"));
    }

    #[test]
    fn cdcp_home_beats_xdg() {
        hit("cdcp_home_beats_xdg");
        let td = tempfile::tempdir().unwrap();
        let home = td.path().join("cdcp-home");
        let xdg = td.path().join("xdg");
        plant_installed(&home);
        plant_source(&xdg.join("cdcp"));
        let mut env = env_at(td.path());
        env.cdcp_home = Some(home.clone());
        env.xdg_data_home = Some(xdg);
        let r = resolve(&env).unwrap();
        assert_eq!(r.via, Via::CdcpHome);
        assert_eq!(r.path, home);
        assert_eq!(r.kind, RootKind::Installed);
    }

    #[test]
    fn xdg_beats_local_share() {
        hit("xdg_beats_local_share");
        let td = tempfile::tempdir().unwrap();
        let xdg = td.path().join("xdg");
        let home = td.path().join("home");
        plant_installed(&xdg.join("cdcp"));
        plant_source(&home.join(".local/share/cdcp"));
        let mut env = env_at(td.path());
        env.xdg_data_home = Some(xdg.clone());
        env.home = Some(home);
        let r = resolve(&env).unwrap();
        assert_eq!(r.via, Via::XdgDataHome);
        assert_eq!(r.path, xdg.join("cdcp"));
        assert_eq!(r.kind, RootKind::Installed);
    }

    #[test]
    fn local_share_beats_cwd_walk() {
        hit("local_share_beats_cwd_walk");
        let td = tempfile::tempdir().unwrap();
        let home = td.path().join("home");
        let cwd = td.path().join("checkout");
        plant_installed(&home.join(".local/share/cdcp"));
        plant_source(&cwd);
        let mut env = env_at(&cwd);
        env.home = Some(home.clone());
        let r = resolve(&env).unwrap();
        assert_eq!(r.via, Via::LocalShare);
        assert_eq!(r.path, home.join(".local/share/cdcp"));
        assert_eq!(r.kind, RootKind::Installed);
    }

    #[test]
    fn cwd_walk_source_checkout() {
        hit("cwd_walk_source_checkout");
        let td = tempfile::tempdir().unwrap();
        let root = td.path().join("engine");
        let nested = root.join("a/b/c");
        plant_source(&root);
        fs::create_dir_all(&nested).unwrap();
        let r = resolve(&env_at(&nested)).unwrap();
        assert_eq!(r.via, Via::CwdWalk);
        assert_eq!(r.kind, RootKind::SourceCheckout);
        assert_eq!(r.path, root);
        assert_eq!(r.web, root.join("web"));
    }

    #[test]
    fn cwd_walk_installed() {
        hit("cwd_walk_installed");
        let td = tempfile::tempdir().unwrap();
        let home = td.path().join("opt/cdcp");
        plant_installed(&home);
        let r = resolve(&env_at(&home.join("web"))).unwrap();
        assert_eq!(r.via, Via::CwdWalk);
        assert_eq!(r.kind, RootKind::Installed);
        // Standing in web/ classifies that directory as the bundle itself.
        assert_eq!(r.web, home.join("web"));
        assert_eq!(r.path, home.join("web"));
    }

    #[test]
    fn missing_names_absolute_xdg_web() {
        hit("missing_names_absolute_xdg_web");
        let td = tempfile::tempdir().unwrap();
        let xdg = td.path().join("xdg-empty");
        fs::create_dir_all(&xdg).unwrap();
        let mut env = env_at(&td.path().join("nowhere"));
        env.xdg_data_home = Some(xdg.clone());
        let err = resolve(&env).unwrap_err();
        match err {
            RootError::BundleMissing { ref looked_for } => {
                let want = xdg.join("cdcp/web");
                assert_eq!(looked_for.as_str(), want.display().to_string());
                assert!(
                    Path::new(&looked_for).is_absolute(),
                    "missing-bundle path must be absolute: {looked_for}"
                );
            }
            other => panic!("expected BundleMissing, got {other:?}"),
        }
        assert_eq!(err.exit_code(), EXIT_BUNDLE_MISSING);
        assert!(err.to_string().contains(BUNDLE_NOT_FOUND));
        assert!(err
            .to_string()
            .contains(&xdg.join("cdcp/web").display().to_string()));
    }

    #[test]
    fn cdcp_home_invalid_does_not_fall_through() {
        hit("cdcp_home_invalid_does_not_fall_through");
        let td = tempfile::tempdir().unwrap();
        let bad = td.path().join("bad-home");
        let xdg = td.path().join("xdg");
        plant_installed(&xdg.join("cdcp"));
        let mut env = env_at(td.path());
        env.cdcp_home = Some(bad.clone());
        env.xdg_data_home = Some(xdg);
        let err = resolve(&env).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains(BUNDLE_NOT_FOUND), "{msg}");
        assert!(
            msg.contains(&bad.join("web").display().to_string()),
            "must name CDCP_HOME/web, not fall through to XDG: {msg}"
        );
    }

    #[test]
    fn walk_engine_root_rejects_installed_only() {
        hit("walk_engine_root_rejects_installed_only");
        let td = tempfile::tempdir().unwrap();
        plant_installed(td.path());
        let err = walk_engine_root(td.path()).unwrap_err();
        match err {
            RootError::EngineMissing { .. } => {}
            other => panic!("expected EngineMissing, got {other:?}"),
        }
    }

    #[test]
    fn walk_engine_root_finds_nested_source() {
        hit("walk_engine_root_finds_nested_source");
        let td = tempfile::tempdir().unwrap();
        let root = td.path().join("engine");
        let nested = root.join("crates/cdcp_gate");
        plant_source(&root);
        fs::create_dir_all(&nested).unwrap();
        assert_eq!(walk_engine_root(&nested).unwrap(), root);
    }

    #[test]
    fn cdcp_repo_root_is_not_an_override() {
        hit("cdcp_repo_root_is_not_an_override");
        // D6: an env-overridable trust anchor is a defect. CDCP_REPO_ROOT
        // is not a ResolveEnv field and is not read. A decoy source tree
        // sitting next to a valid XDG home must lose.
        let td = tempfile::tempdir().unwrap();
        let decoy = td.path().join("decoy");
        let xdg = td.path().join("xdg");
        plant_source(&decoy);
        plant_installed(&xdg.join("cdcp"));
        let mut env = env_at(td.path());
        env.xdg_data_home = Some(xdg.clone());
        let r = resolve(&env).unwrap();
        assert_eq!(r.path, xdg.join("cdcp"), "only XDG/CDCP_HOME/--root win");
        assert_eq!(r.via, Via::XdgDataHome);
        assert_ne!(r.path, decoy);
    }

    #[test]
    fn announce_names_kind_and_via() {
        hit("announce_names_kind_and_via");
        let td = tempfile::tempdir().unwrap();
        plant_source(td.path());
        let r = resolve(&env_at(td.path())).unwrap();
        let line = r.announce();
        assert!(line.contains("source-checkout"), "{line}");
        assert!(line.contains("cwd-walk"), "{line}");
        assert!(line.contains(&td.path().display().to_string()), "{line}");
    }

    #[test]
    fn production_has_no_cargo_manifest_dir() {
        hit("production_has_no_cargo_manifest_dir");
        let src = include_str!("lib.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production precedes tests");
        assert!(
            !src.contains("env!(\"CARGO_MANIFEST_DIR\")"),
            "cdcp_root production baked a compile-time path"
        );
        assert!(
            !src.contains("var(\"CDCP_REPO_ROOT\")"),
            "cdcp_root must not revive the D6 env override"
        );
        assert!(src.contains("BUNDLE_NOT_FOUND"));
        assert!(src.contains("WALK_LEVELS"));
    }

    #[test]
    fn delegates_dropped_compile_time_fallback() {
        hit("delegates_dropped_compile_time_fallback");
        let engine = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let files = [
            "crates/cdcp_gate/src/root.rs",
            "crates/cdcp_learn/src/lib.rs",
            "crates/cdcp_evidence/src/licence.rs",
            "crates/cdcp_anki/src/lib.rs",
            "crates/cdcp_registry_check/src/lib.rs",
            "crates/cdcp_data/src/lib.rs",
        ];
        assert_eq!(
            files.len(),
            6,
            "the five copies + one delegate must all be scanned"
        );
        let mut scanned = 0usize;
        for rel in files {
            let path = engine.join(rel);
            let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {rel}: {e}"));
            let prod = text.split("#[cfg(test)]").next().unwrap();
            assert!(
                !prod.contains("env!(\"CARGO_MANIFEST_DIR\")"),
                "{rel} production still expands a compile-time crate directory"
            );
            assert!(
                !prod.contains("var(\"CDCP_REPO_ROOT\")"),
                "{rel} production still reads CDCP_REPO_ROOT"
            );
            if rel.ends_with("cdcp_data/src/lib.rs") {
                assert!(
                    prod.contains("resolve_engine_root(start)"),
                    "{rel} must stay a delegate"
                );
            } else {
                assert!(
                    prod.contains("cdcp_root::walk_engine_root"),
                    "{rel} is not a walk_engine_root delegate"
                );
            }
            scanned += 1;
        }
        assert_eq!(scanned, files.len());
    }

    #[test]
    fn walk_levels_is_twelve_not_eight() {
        assert_eq!(WALK_LEVELS, 12);
    }
}
