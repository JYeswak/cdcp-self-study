//! Subcommand registry and the shared gate contract.
//!
//! The dispatch table itself is generated (see `build.rs`) from the files in
//! `src/gates/`, so a new gate never edits a shared file.

use std::fmt;
use std::path::PathBuf;

/// The three ways a gate can end other than success. See `crate::exit` for the
/// codes these map to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateError {
    /// One or more assertions failed. Each string names a file/row and why.
    Violation(Vec<String>),
    /// The gate was invoked wrongly.
    Usage(String),
    /// The gate could not honestly evaluate (bad registry, no git, vacuous scan).
    Error(String),
}

impl GateError {
    pub fn violation(items: impl IntoIterator<Item = String>) -> Self {
        Self::Violation(items.into_iter().collect())
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self::Error(msg.into())
    }

    pub fn usage(msg: impl Into<String>) -> Self {
        Self::Usage(msg.into())
    }

    pub fn code(&self) -> u8 {
        match self {
            Self::Violation(_) => crate::exit::VIOLATION,
            Self::Usage(_) => crate::exit::USAGE,
            Self::Error(_) => crate::exit::ERROR,
        }
    }

    /// Print the failure the way `check.sh` output expects: one line per finding,
    /// prefixed with the gate name so a 400-line check.sh log stays greppable.
    pub fn report(&self, gate: &str) {
        match self {
            Self::Violation(items) => {
                for item in items {
                    eprintln!("{gate}: FAIL: {item}");
                }
                eprintln!("{gate}: FAIL: {} violation(s)", items.len());
            }
            Self::Usage(m) => eprintln!("{gate}: USAGE: {m}"),
            Self::Error(m) => eprintln!("{gate}: ERROR: {m}"),
        }
    }
}

impl fmt::Display for GateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Violation(items) => {
                write!(f, "{} violation(s): {}", items.len(), items.join("; "))
            }
            Self::Usage(m) => write!(f, "usage: {m}"),
            Self::Error(m) => write!(f, "error: {m}"),
        }
    }
}

/// Everything a gate is handed. Deliberately small: repo root plus the argv tail.
#[derive(Debug, Clone)]
pub struct GateCtx {
    /// Absolute path to the course-engine project root (the dir holding `registries/`).
    pub root: PathBuf,
    /// Arguments after the subcommand name.
    pub args: Vec<String>,
}

impl GateCtx {
    pub fn new(root: PathBuf, args: Vec<String>) -> Self {
        Self { root, args }
    }

    pub fn has_flag(&self, flag: &str) -> bool {
        self.args.iter().any(|a| a == flag)
    }

    /// Reject anything we do not recognise instead of silently ignoring it. A
    /// typo'd `--stagd` must not read as "the staged leg passed".
    pub fn reject_unknown_flags(&self, known: &[&str]) -> Result<(), GateError> {
        for a in &self.args {
            if !known.contains(&a.as_str()) {
                return Err(GateError::usage(format!(
                    "unknown argument {a:?}; known: {}",
                    known.join(" ")
                )));
            }
        }
        Ok(())
    }
}

pub type GateFn = fn(&GateCtx) -> Result<(), GateError>;

/// One registered subcommand.
pub struct Gate {
    pub name: &'static str,
    pub summary: &'static str,
    pub run: GateFn,
}

/// Every gate compiled into this binary.
pub fn all() -> &'static [Gate] {
    crate::gates::GATES
}

pub fn find(name: &str) -> Option<&'static Gate> {
    all().iter().find(|g| g.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_distinct_and_nonzero_for_failure() {
        assert_eq!(crate::exit::OK, 0);
        assert_ne!(crate::exit::VIOLATION, crate::exit::ERROR);
        assert_ne!(crate::exit::VIOLATION, crate::exit::USAGE);
        for c in [
            crate::exit::VIOLATION,
            crate::exit::USAGE,
            crate::exit::ERROR,
        ] {
            assert_ne!(c, crate::exit::OK, "failure code must not be 0");
        }
    }

    #[test]
    fn error_kinds_map_to_codes() {
        assert_eq!(
            GateError::violation(["x".to_string()]).code(),
            crate::exit::VIOLATION
        );
        assert_eq!(GateError::usage("x").code(), crate::exit::USAGE);
        assert_eq!(GateError::error("x").code(), crate::exit::ERROR);
    }

    #[test]
    fn unknown_flag_is_usage_not_silence() {
        let ctx = GateCtx::new(PathBuf::from("/tmp"), vec!["--stagd".into()]);
        let err = ctx.reject_unknown_flags(&["--staged"]).unwrap_err();
        assert_eq!(err.code(), crate::exit::USAGE);
    }

    #[test]
    fn registry_is_not_empty() {
        assert!(!all().is_empty(), "empty gate registry is an ERROR");
    }
}
