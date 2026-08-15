//! Named failures. A missing boundary is a schema ERROR, never a bare number.

use thiserror::Error;

/// Token interpolated inside the missing-boundary path. Deleting the
/// [`crate::parse::require_boundary`] check makes the matching selftest
/// non-zero.
pub const MISSING_BOUNDARY: &str = "missing boundary is a schema ERROR";

/// Token interpolated inside the bare-number path.
pub const BARE_NUMBER: &str = "a bare number is a schema ERROR";

/// Token interpolated inside the empty-boundary path.
pub const EMPTY_BOUNDARY: &str = "empty boundary is a schema ERROR";

/// Token interpolated inside the TGG 1.8 L/kWh refusal.
///
/// NREL/TP-550-33905 (2003) gives 1.8 L/kWh as thermoelectric-only. The
/// Green Grid WP#35 took that figure as the US "unknown" default *and*
/// assigned hydro an EWIF of 0 — excluding reservoir evaporation twice.
pub const EWIF_EXCLUDES_HYDRO_TWICE: &str = "EWIF excludes hydro reservoir evaporation twice";

/// Token interpolated when two values cannot be compared.
pub const INCOMPARABLE: &str = "incomparable metrics: boundaries differ";

/// Why a metric could not be constructed, parsed, or compared.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum MetricsError {
    /// Document or constructor omitted the boundary table.
    #[error("{MISSING_BOUNDARY}")]
    MissingBoundary,
    /// A scalar / ratio with no kind and no boundary.
    #[error("{BARE_NUMBER}")]
    BareNumber,
    /// `[boundary]` present but names nothing in or out.
    #[error("{EMPTY_BOUNDARY}")]
    EmptyBoundary,
    /// The infamous 1.8 L/kWh "unknown" EWIF.
    #[error("{EWIF_EXCLUDES_HYDRO_TWICE}")]
    EwifExcludesHydroTwice,
    /// Comparison across unequal control volumes.
    #[error("{INCOMPARABLE} ({kind})")]
    IncomparableBoundaries {
        /// Metric kind that was compared.
        kind: &'static str,
    },
    /// Includes and excludes share an item.
    #[error("boundary overlap: {0} is both in and out")]
    BoundaryOverlap(String),
    /// Kind-specific declaration is missing.
    #[error("boundary missing {field} for {kind}")]
    MissingDeclaration {
        /// Metric kind.
        kind: &'static str,
        /// Required field name.
        field: &'static str,
    },
    /// PUE facility total does not include IT energy.
    #[error("PUE/ERE facility total must include it-energy (otherwise PUE < 1 is a lie)")]
    PueWithoutItInBoundary,
    /// PUE numerator smaller than IT energy.
    #[error("PUE < 1: facility energy is below IT energy")]
    PueLessThanOne,
    /// ERE reuse exceeds facility energy.
    #[error("reuse energy exceeds facility energy")]
    ReuseExceedsFacility,
    /// ERE counted recovered energy that was not consumed.
    #[error("ERE cannot count recovered-not-consumed energy as reuse")]
    ReuseNotConsumed,
    /// IT energy (the shared denominator) is zero.
    #[error("IT energy is zero")]
    ZeroItEnergy,
    /// Ratio denominator is zero.
    #[error("ratio denominator must be > 0")]
    ZeroDenominator,
    /// Intermediate arithmetic does not fit i64.
    #[error("integer overflow in rational arithmetic")]
    Overflow,
    /// Declared metric value is negative.
    #[error("metric value must be >= 0")]
    NegativeValue,
    /// Unknown kind tag.
    #[error("unknown metric kind: {0}")]
    UnknownKind(String),
    /// Unknown scope / enum tag.
    #[error("unknown boundary token: {0}")]
    UnknownToken(String),
    /// Kind and boundary disagree.
    #[error("kind mismatch: {0}")]
    KindMismatch(String),
    /// Input document was empty.
    #[error("empty metric document is a schema ERROR")]
    EmptyDocument,
    /// A float appeared where a rational is required.
    #[error("floating-point value is a schema ERROR — use {{num, den}}")]
    FloatForbidden,
    /// TOML could not be read as a metric document.
    #[error("unparseable metric: {0}")]
    Unparseable(String),
    /// Free-cooling hours from `cdcp_data` / `cdcp_site` are not an integer count.
    #[error("free-cooling hours must be a non-negative integer count, got {0}")]
    NonIntegerHours(String),
    /// Upstream quantity failure (missing location, parse).
    #[error("free-cooling: {0}")]
    FreeCooling(String),
}
