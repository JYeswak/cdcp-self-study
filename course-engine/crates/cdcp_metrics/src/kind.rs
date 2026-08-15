//! The four Green Grid / ISO 30134 KPIs this crate knows how to carry.

use crate::error::MetricsError;

/// Kind names as they appear on the wire (`kebab-case` tags).
pub const KINDS: &[&str] = &["pue", "wue", "cue", "ere"];

/// PUE / WUE / CUE / ERE.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MetricKind {
    /// Power Usage Effectiveness: facility energy / IT energy.
    Pue,
    /// Water Usage Effectiveness: water / IT energy (L/kWh class).
    Wue,
    /// Carbon Usage Effectiveness: CO2 / IT energy.
    Cue,
    /// Energy Reuse Effectiveness: (facility − reuse) / IT energy.
    Ere,
}

impl MetricKind {
    /// Stable kebab-case name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pue => "pue",
            Self::Wue => "wue",
            Self::Cue => "cue",
            Self::Ere => "ere",
        }
    }

    /// Parse a kebab-case name.
    pub fn parse(s: &str) -> Result<Self, MetricsError> {
        match s {
            "pue" => Ok(Self::Pue),
            "wue" => Ok(Self::Wue),
            "cue" => Ok(Self::Cue),
            "ere" => Ok(Self::Ere),
            other => Err(MetricsError::UnknownKind(other.to_string())),
        }
    }
}
