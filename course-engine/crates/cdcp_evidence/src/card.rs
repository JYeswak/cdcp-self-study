//! Model cards: named assumptions + a validity box. No card, no query.
//!
//! Analytic / empirical rows must name their idealizing assumptions
//! explicitly. An empty assumption list is the invisible-axiom failure
//! mode review round 3 killed.

use crate::{
    Abstention, Evidence, EvidenceError, ModelEvidence, NumericalCertificate, SensitivitySummary,
    StatisticalCertificate, ValidityDomain, DOMAIN_CHECK,
};
use std::collections::BTreeMap;
use thiserror::Error;

/// Why a model card could not be constructed.
#[derive(Debug, Error, PartialEq)]
pub enum CardError {
    /// A card without a name cannot be cited.
    #[error("model card name is empty")]
    EmptyName,
    /// Idealizing assumptions must be named; silence is an invisible axiom.
    #[error(
        "model card must name its idealizing assumptions explicitly; \
         an empty list is the invisible-axiom failure mode"
    )]
    EmptyAssumptions,
    /// A blank (whitespace-only) assumption is not a name.
    #[error("model card assumption is blank")]
    BlankAssumption,
    /// Same lattice as [`EvidenceError::InvalidDiscrepancy`].
    #[error(transparent)]
    InvalidDiscrepancy(#[from] EvidenceError),
}

/// A named model: assumptions, validity, discrepancy. The query door
/// refuses out-of-domain points instead of emitting a number.
#[derive(Debug, Clone)]
pub struct ModelCard {
    name: String,
    assumptions: Vec<String>,
    validity: ValidityDomain,
    discrepancy_rel: f64,
}

impl ModelCard {
    /// Construct a card. Assumptions are sorted and deduplicated.
    /// An empty name or an empty/blank assumption list is rejected.
    pub fn try_new(
        name: impl Into<String>,
        mut assumptions: Vec<String>,
        validity: ValidityDomain,
        discrepancy_rel: f64,
    ) -> Result<Self, CardError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(CardError::EmptyName);
        }
        if assumptions.is_empty() {
            return Err(CardError::EmptyAssumptions);
        }
        if assumptions.iter().any(|a| a.trim().is_empty()) {
            return Err(CardError::BlankAssumption);
        }
        if !ModelEvidence::valid_discrepancy(discrepancy_rel) {
            return Err(CardError::InvalidDiscrepancy(
                EvidenceError::InvalidDiscrepancy { discrepancy_rel },
            ));
        }
        assumptions.sort_unstable();
        assumptions.dedup();
        Ok(ModelCard {
            name,
            assumptions,
            validity,
            discrepancy_rel,
        })
    }

    /// Stable card name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Idealizing assumptions (sorted, non-empty).
    #[must_use]
    pub fn assumptions(&self) -> &[String] {
        &self.assumptions
    }

    /// Declared validity box.
    #[must_use]
    pub fn validity(&self) -> &ValidityDomain {
        &self.validity
    }

    /// Relative model-form discrepancy inside the box.
    #[must_use]
    pub fn discrepancy_rel(&self) -> f64 {
        self.discrepancy_rel
    }

    /// Query-time model slice. Out of domain is [`Abstention`], never
    /// [`ModelEvidence`] with `in_domain: false`.
    pub fn query(&self, point: &BTreeMap<String, f64>) -> Result<ModelEvidence, Abstention> {
        self.validity.check(point)?;
        Ok(ModelEvidence {
            cards: vec![self.name.clone()],
            assumptions: self.assumptions.clone(),
            validity: self.validity.clone(),
            discrepancy_rel: self.discrepancy_rel,
            in_domain: true,
        })
    }
}

/// An empirical correlation bound to a [`ModelCard`]. The formula runs
/// only after the domain check; an out-of-domain point never becomes a
/// value with a caveat.
#[derive(Debug, Clone)]
pub struct Correlation {
    card: ModelCard,
}

impl Correlation {
    /// Bind a card as a queryable correlation.
    #[must_use]
    pub fn new(card: ModelCard) -> Self {
        Correlation { card }
    }

    /// The bound card.
    #[must_use]
    pub fn card(&self) -> &ModelCard {
        &self.card
    }

    /// Evaluate `formula` only if `point` is inside the card's validity.
    ///
    /// COMPOSITION / QUERY LOGIC: the bound check runs first. Deleting
    /// it is what the meta-test is for. `{DOMAIN_CHECK}` is interpolated
    /// here so a deleted check cannot hide behind a comment.
    pub fn query<F>(
        &self,
        point: &BTreeMap<String, f64>,
        formula: F,
    ) -> Result<Evidence<f64>, Abstention>
    where
        F: FnOnce(&BTreeMap<String, f64>) -> f64,
    {
        match self.card.query(point) {
            Err(abs) => {
                let _ = DOMAIN_CHECK;
                Err(abs)
            }
            Ok(model) => {
                let _ = DOMAIN_CHECK;
                let value = formula(point);
                Ok(Evidence {
                    value,
                    qoi: value,
                    numerical: NumericalCertificate::estimate(value, value),
                    statistical: StatisticalCertificate::None,
                    model,
                    sensitivity: SensitivitySummary::default(),
                })
            }
        }
    }
}
