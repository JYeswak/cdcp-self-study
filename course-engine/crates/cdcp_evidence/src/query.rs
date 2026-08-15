//! Query-time abstention: out of domain is a refusal, not a caveat.
//!
//! [`ValidityDomain::check`] is the load-bearing predicate. A query that
//! fails it returns [`Abstention`] — never [`crate::Evidence`] with
//! `in_domain: false`. That flag is only a latch on values that already
//! exist; it cannot mint a number the domain refused.

use std::fmt;

/// Token the live query body must interpolate. The selftest keys on this
/// identifier appearing *inside* [`crate::Correlation::query`].
pub const DOMAIN_CHECK: &str = "query outside a declared bound returns Abstention, not a value";

/// Why a declared bound rejected a query point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViolationKind {
    /// A NaN or non-finite endpoint — the axis is UNUSABLE, not unconstrained.
    Unusable,
    /// Finite `lo > hi` (empty box, typically after intersection).
    Empty,
    /// The query omitted a constrained parameter.
    Missing,
    /// The query value is non-finite or strictly outside `[lo, hi]`.
    OutOfRange,
    /// A constituent already carried `in_domain = false` (composition latch).
    Propagated,
}

impl ViolationKind {
    fn as_str(self) -> &'static str {
        match self {
            ViolationKind::Unusable => "unusable-domain",
            ViolationKind::Empty => "empty-domain",
            ViolationKind::Missing => "missing-parameter",
            ViolationKind::OutOfRange => "out-of-range",
            ViolationKind::Propagated => "in-domain-false-propagated",
        }
    }
}

/// One axis the query failed.
#[derive(Debug, Clone, PartialEq)]
pub struct DomainViolation {
    /// Parameter name (`"*"` for a propagated latch with no single axis).
    pub param: String,
    /// Queried value (`None` = omitted, or no value on a propagated latch).
    pub value: Option<f64>,
    /// Declared `[lo, hi]` when one exists.
    pub bound: Option<(f64, f64)>,
    /// How the axis failed.
    pub kind: ViolationKind,
}

/// Explicit scope refusal. This is the result, not a warning on a number.
#[derive(Debug, Clone, PartialEq)]
pub struct Abstention {
    /// Every violated axis (BTreeMap order from the domain — deterministic).
    pub violations: Vec<DomainViolation>,
}

impl Abstention {
    pub(crate) fn from_violations(violations: Vec<DomainViolation>) -> Self {
        Abstention { violations }
    }

    /// Composition latch: a constituent was already out of domain, so the
    /// composed query refuses rather than emitting a value with a caveat.
    #[must_use]
    pub fn propagated() -> Self {
        Abstention {
            violations: vec![DomainViolation {
                param: "*".into(),
                value: None,
                bound: None,
                kind: ViolationKind::Propagated,
            }],
        }
    }

    /// Merge two refusals (union of violations, first-then-second order).
    #[must_use]
    pub fn merge(mut self, other: Self) -> Self {
        self.violations.extend(other.violations);
        self
    }

    /// True when any violation is a range/missing/unusable domain failure
    /// (i.e. a bound check, not only a propagated latch).
    #[must_use]
    pub fn is_domain_refusal(&self) -> bool {
        self.violations.iter().any(|v| {
            matches!(
                v.kind,
                ViolationKind::Unusable
                    | ViolationKind::Empty
                    | ViolationKind::Missing
                    | ViolationKind::OutOfRange
            )
        })
    }
}

impl fmt::Display for Abstention {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{DOMAIN_CHECK}")?;
        for v in &self.violations {
            write!(f, "; {} `{}`", v.kind.as_str(), v.param)?;
            if let Some(val) = v.value {
                write!(f, " = {val}")?;
            }
            if let Some((lo, hi)) = v.bound {
                write!(f, " vs [{lo}, {hi}]")?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for Abstention {}
