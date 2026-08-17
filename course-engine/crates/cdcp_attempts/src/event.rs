//! Attempt-event record. Capture only — no analysis fields.

use crate::error::AttemptError;
use serde::{Deserialize, Serialize};

/// Schema version written on every new event. Bump when the on-disk shape
/// changes; loaders must name the version they understand.
pub const SCHEMA_VERSION: u32 = 1;

/// Product surfaces that can emit an attempt. Unknown tags are a schema ERROR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AttemptMode {
    Learn,
    Quiz,
    Drill,
    Mock,
}

impl AttemptMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Learn => "learn",
            Self::Quiz => "quiz",
            Self::Drill => "drill",
            Self::Mock => "mock",
        }
    }

    pub fn parse(s: &str) -> Result<Self, AttemptError> {
        match s {
            "learn" => Ok(Self::Learn),
            "quiz" => Ok(Self::Quiz),
            "drill" => Ok(Self::Drill),
            "mock" => Ok(Self::Mock),
            other => Err(AttemptError::UnknownMode(other.to_string())),
        }
    }
}

/// One recorded attempt. The field set is EPIC L plus [`SCHEMA_VERSION`].
///
/// There is no item-difficulty field, no discrimination field, and no
/// ability/theta field. Those quantities are sample-dependent and are
/// refused by this crate at any N.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttemptEvent {
    pub schema_version: u32,
    pub item_version: String,
    pub bank_hash: String,
    pub learner_pseudonym: String,
    pub mode: AttemptMode,
    pub exposure_count: u32,
    pub chosen_option: String,
    pub correctness: bool,
    pub latency_ms: u64,
    pub timestamp_unix_ms: u64,
    pub prior_attempts: u32,
}

impl AttemptEvent {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        item_version: impl Into<String>,
        bank_hash: impl Into<String>,
        learner_pseudonym: impl Into<String>,
        mode: AttemptMode,
        exposure_count: u32,
        chosen_option: impl Into<String>,
        correctness: bool,
        latency_ms: u64,
        timestamp_unix_ms: u64,
        prior_attempts: u32,
    ) -> Result<Self, AttemptError> {
        let event = Self {
            schema_version: SCHEMA_VERSION,
            item_version: item_version.into(),
            bank_hash: bank_hash.into(),
            learner_pseudonym: learner_pseudonym.into(),
            mode,
            exposure_count,
            chosen_option: chosen_option.into(),
            correctness,
            latency_ms,
            timestamp_unix_ms,
            prior_attempts,
        };
        event.validate()?;
        Ok(event)
    }

    pub fn validate(&self) -> Result<(), AttemptError> {
        require_nonempty("item_version", &self.item_version)?;
        require_nonempty("bank_hash", &self.bank_hash)?;
        require_nonempty("learner_pseudonym", &self.learner_pseudonym)?;
        require_nonempty("chosen_option", &self.chosen_option)?;
        if self.timestamp_unix_ms == 0 {
            return Err(AttemptError::TimestampUnset);
        }
        Ok(())
    }
}

fn require_nonempty(name: &'static str, value: &str) -> Result<(), AttemptError> {
    if value.is_empty() || value.chars().all(char::is_whitespace) {
        return Err(AttemptError::EmptyField(name));
    }
    Ok(())
}
