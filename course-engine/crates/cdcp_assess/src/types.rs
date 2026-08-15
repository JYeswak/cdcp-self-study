//! Typed items and responses. Kinds are not four letters.
use crate::error::AssessError;
use crate::ratio::Ratio;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeSet;
use std::fmt;

/// Kind names as they appear on the wire (`kebab-case` tags).
pub const KINDS: &[&str] = &[
    "single-select",
    "multi-select",
    "ordering",
    "numeric-range",
    "topology-selection",
    "procedural-sequence",
];

/// Non-empty identifier. Not trimmed — `"A"` and `" A "` are different ids.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(String);

impl Id {
    pub fn new(s: impl Into<String>) -> Result<Self, AssessError> {
        let s = s.into();
        if s.is_empty() || s.chars().all(char::is_whitespace) {
            return Err(AssessError::EmptyId);
        }
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for Id {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Id::new(s).map_err(serde::de::Error::custom)
    }
}

/// Non-empty unit label. Empty is a bare number, which is a schema ERROR.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Units(String);

impl Units {
    pub fn new(s: impl Into<String>) -> Result<Self, AssessError> {
        let s = s.into();
        if s.is_empty() || s.chars().all(char::is_whitespace) {
            return Err(AssessError::BareNumber);
        }
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Serialize for Units {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Units {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Units::new(s).map_err(serde::de::Error::custom)
    }
}

/// A measured quantity. Units are required; a bare number cannot be constructed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Quantity {
    pub value: Ratio,
    pub units: Units,
}

impl Quantity {
    pub fn new(value: Ratio, units: impl Into<String>) -> Result<Self, AssessError> {
        Ok(Self {
            value,
            units: Units::new(units)?,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ToleranceKind {
    Absolute,
    Relative,
}

/// Declared tolerance. Magnitude is a non-negative rational. There is no default.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tolerance {
    pub kind: ToleranceKind,
    pub magnitude: Ratio,
}

impl Tolerance {
    pub fn new(kind: ToleranceKind, magnitude: Ratio) -> Result<Self, AssessError> {
        if magnitude.is_negative() {
            return Err(AssessError::NegativeTolerance);
        }
        Ok(Self { kind, magnitude })
    }

    pub fn validate(self) -> Result<(), AssessError> {
        if self.magnitude.is_negative() {
            Err(AssessError::NegativeTolerance)
        } else {
            Ok(())
        }
    }
}

/// How a set-valued answer (multi-select, topology) is scored. No implicit default.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SetCredit {
    /// 1/1 iff the chosen set equals the key; else 0/1.
    AllOrNothing,
    /// `|chosen ∩ correct| / |chosen ∪ correct|`. `correct` is required non-empty.
    Jaccard,
}

/// How an ordered answer (ordering, procedural-sequence) is scored.
///
/// Partial credit is never implicit: the item must name one of these policies.
/// `AllOrNothing` refuses partial credit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SequenceCredit {
    /// 1/1 iff the response equals the key; else 0/1. Swaps, omissions, extras → 0.
    AllOrNothing,
    /// `earned = count of i where response[i] == key[i]`; `out_of = key.len()`.
    /// A shorter response cannot match later positions. Extra tail ids earn nothing.
    PositionMatches,
    /// `earned =` number of adjacent pairs `(key[i], key[i+1])` that appear
    /// consecutively in that order in the response; `out_of = key.len() - 1`.
    /// Forbidden on a key of length &lt; 2 (`out_of` would be 0).
    AdjacentPairs,
}

/// Assessment item. Wire tag is `kind`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Item {
    SingleSelect {
        options: Vec<Id>,
        correct: Id,
    },
    MultiSelect {
        options: Vec<Id>,
        correct: Vec<Id>,
        credit: SetCredit,
    },
    Ordering {
        elements: Vec<Id>,
        credit: SequenceCredit,
    },
    NumericRange {
        expected: Quantity,
        tolerance: Tolerance,
    },
    TopologySelection {
        elements: Vec<Id>,
        correct: Vec<Id>,
        credit: SetCredit,
    },
    ProceduralSequence {
        steps: Vec<Id>,
        credit: SequenceCredit,
    },
}

/// Learner response. Wire tag is `kind` and must match the item.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Response {
    SingleSelect { chosen: Id },
    MultiSelect { chosen: Vec<Id> },
    Ordering { order: Vec<Id> },
    NumericRange { submitted: Quantity },
    TopologySelection { chosen: Vec<Id> },
    ProceduralSequence { steps: Vec<Id> },
}

/// Reduced non-negative rational `earned/out_of` with `out_of > 0`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Score {
    earned: u64,
    out_of: u64,
}

impl Score {
    pub fn new(earned: u64, out_of: u64) -> Result<Self, AssessError> {
        if out_of == 0 {
            return Err(AssessError::ZeroScoreDenominator);
        }
        let g = gcd_u64(earned, out_of);
        Ok(Self {
            earned: earned / g,
            out_of: out_of / g,
        })
    }

    pub fn zero() -> Self {
        Self {
            earned: 0,
            out_of: 1,
        }
    }

    pub fn full() -> Self {
        Self {
            earned: 1,
            out_of: 1,
        }
    }

    pub fn earned(self) -> u64 {
        self.earned
    }

    pub fn out_of(self) -> u64 {
        self.out_of
    }

    pub fn is_full(self) -> bool {
        self.earned == self.out_of
    }

    pub fn is_zero(self) -> bool {
        self.earned == 0
    }
}

/// Canonical result used for digest / dual-path comparison.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScoreReport {
    pub kind: String,
    pub earned: u64,
    pub out_of: u64,
    pub full_credit: bool,
}

impl Item {
    pub fn kind_name(&self) -> &'static str {
        match self {
            Item::SingleSelect { .. } => "single-select",
            Item::MultiSelect { .. } => "multi-select",
            Item::Ordering { .. } => "ordering",
            Item::NumericRange { .. } => "numeric-range",
            Item::TopologySelection { .. } => "topology-selection",
            Item::ProceduralSequence { .. } => "procedural-sequence",
        }
    }

    pub fn from_json(s: &str) -> Result<Self, AssessError> {
        let item: Self = serde_json::from_str(s).map_err(|e| AssessError::Json(e.to_string()))?;
        item.validate()?;
        Ok(item)
    }

    pub fn single_select<I, S>(options: I, correct: S) -> Result<Self, AssessError>
    where
        I: IntoIterator,
        I::Item: Into<String>,
        S: Into<String>,
    {
        let options = map_ids(options)?;
        let correct = Id::new(correct.into())?;
        let item = Item::SingleSelect { options, correct };
        item.validate()?;
        Ok(item)
    }

    pub fn multi_select<I, C>(
        options: I,
        correct: C,
        credit: SetCredit,
    ) -> Result<Self, AssessError>
    where
        I: IntoIterator,
        I::Item: Into<String>,
        C: IntoIterator,
        C::Item: Into<String>,
    {
        let item = Item::MultiSelect {
            options: map_ids(options)?,
            correct: map_ids(correct)?,
            credit,
        };
        item.validate()?;
        Ok(item)
    }

    pub fn ordering<I>(elements: I, credit: SequenceCredit) -> Result<Self, AssessError>
    where
        I: IntoIterator,
        I::Item: Into<String>,
    {
        let item = Item::Ordering {
            elements: map_ids(elements)?,
            credit,
        };
        item.validate()?;
        Ok(item)
    }

    pub fn numeric_range(expected: Quantity, tolerance: Tolerance) -> Result<Self, AssessError> {
        let item = Item::NumericRange {
            expected,
            tolerance,
        };
        item.validate()?;
        Ok(item)
    }

    pub fn topology_selection<I, C>(
        elements: I,
        correct: C,
        credit: SetCredit,
    ) -> Result<Self, AssessError>
    where
        I: IntoIterator,
        I::Item: Into<String>,
        C: IntoIterator,
        C::Item: Into<String>,
    {
        let item = Item::TopologySelection {
            elements: map_ids(elements)?,
            correct: map_ids(correct)?,
            credit,
        };
        item.validate()?;
        Ok(item)
    }

    pub fn procedural_sequence<I>(steps: I, credit: SequenceCredit) -> Result<Self, AssessError>
    where
        I: IntoIterator,
        I::Item: Into<String>,
    {
        let item = Item::ProceduralSequence {
            steps: map_ids(steps)?,
            credit,
        };
        item.validate()?;
        Ok(item)
    }

    pub fn validate(&self) -> Result<(), AssessError> {
        match self {
            Item::SingleSelect { options, correct } => {
                require_unique(options)?;
                if options.len() < 2 {
                    return Err(if options.is_empty() {
                        AssessError::EmptyOptions
                    } else {
                        AssessError::TooFewOptions
                    });
                }
                if !options.contains(correct) {
                    return Err(AssessError::CorrectNotInOptions(correct.to_string()));
                }
            }
            Item::MultiSelect {
                options, correct, ..
            } => {
                validate_set_item(options, correct)?;
            }
            Item::Ordering { elements, credit } => {
                validate_sequence(elements, *credit)?;
            }
            Item::NumericRange {
                expected,
                tolerance,
            } => {
                // Units::new already rejected empty; belt for direct struct build.
                if expected.units.as_str().is_empty() {
                    return Err(AssessError::BareNumber);
                }
                tolerance.validate()?;
            }
            Item::TopologySelection {
                elements, correct, ..
            } => {
                validate_set_item(elements, correct)?;
            }
            Item::ProceduralSequence { steps, credit } => {
                validate_sequence(steps, *credit)?;
            }
        }
        Ok(())
    }
}

impl Response {
    pub fn kind_name(&self) -> &'static str {
        match self {
            Response::SingleSelect { .. } => "single-select",
            Response::MultiSelect { .. } => "multi-select",
            Response::Ordering { .. } => "ordering",
            Response::NumericRange { .. } => "numeric-range",
            Response::TopologySelection { .. } => "topology-selection",
            Response::ProceduralSequence { .. } => "procedural-sequence",
        }
    }

    pub fn from_json(s: &str) -> Result<Self, AssessError> {
        let resp: Self = serde_json::from_str(s).map_err(|e| AssessError::Json(e.to_string()))?;
        resp.validate()?;
        Ok(resp)
    }

    pub fn single_select(chosen: impl Into<String>) -> Result<Self, AssessError> {
        Ok(Response::SingleSelect {
            chosen: Id::new(chosen.into())?,
        })
    }

    pub fn multi_select<I>(chosen: I) -> Result<Self, AssessError>
    where
        I: IntoIterator,
        I::Item: Into<String>,
    {
        let chosen = map_ids(chosen)?;
        require_unique(&chosen).map_err(|e| match e {
            AssessError::DuplicateId(id) => AssessError::DuplicateResponseId(id),
            other => other,
        })?;
        Ok(Response::MultiSelect { chosen })
    }

    pub fn ordering<I>(order: I) -> Result<Self, AssessError>
    where
        I: IntoIterator,
        I::Item: Into<String>,
    {
        let order = map_ids(order)?;
        require_unique(&order).map_err(|e| match e {
            AssessError::DuplicateId(id) => AssessError::DuplicateResponseId(id),
            other => other,
        })?;
        Ok(Response::Ordering { order })
    }

    pub fn numeric_range(submitted: Quantity) -> Result<Self, AssessError> {
        if submitted.units.as_str().is_empty() {
            return Err(AssessError::BareNumber);
        }
        Ok(Response::NumericRange { submitted })
    }

    pub fn topology_selection<I>(chosen: I) -> Result<Self, AssessError>
    where
        I: IntoIterator,
        I::Item: Into<String>,
    {
        let chosen = map_ids(chosen)?;
        require_unique(&chosen).map_err(|e| match e {
            AssessError::DuplicateId(id) => AssessError::DuplicateResponseId(id),
            other => other,
        })?;
        Ok(Response::TopologySelection { chosen })
    }

    pub fn procedural_sequence<I>(steps: I) -> Result<Self, AssessError>
    where
        I: IntoIterator,
        I::Item: Into<String>,
    {
        let steps = map_ids(steps)?;
        require_unique(&steps).map_err(|e| match e {
            AssessError::DuplicateId(id) => AssessError::DuplicateResponseId(id),
            other => other,
        })?;
        Ok(Response::ProceduralSequence { steps })
    }

    pub fn validate(&self) -> Result<(), AssessError> {
        match self {
            Response::SingleSelect { .. } => Ok(()),
            Response::MultiSelect { chosen } => require_unique(chosen).map_err(dup_resp),
            Response::Ordering { order } => require_unique(order).map_err(dup_resp),
            Response::NumericRange { submitted } => {
                if submitted.units.as_str().is_empty() {
                    Err(AssessError::BareNumber)
                } else {
                    Ok(())
                }
            }
            Response::TopologySelection { chosen } => require_unique(chosen).map_err(dup_resp),
            Response::ProceduralSequence { steps } => require_unique(steps).map_err(dup_resp),
        }
    }
}

fn dup_resp(e: AssessError) -> AssessError {
    match e {
        AssessError::DuplicateId(id) => AssessError::DuplicateResponseId(id),
        other => other,
    }
}

fn map_ids<I>(ids: I) -> Result<Vec<Id>, AssessError>
where
    I: IntoIterator,
    I::Item: Into<String>,
{
    ids.into_iter().map(|s| Id::new(s.into())).collect()
}

fn require_unique(ids: &[Id]) -> Result<(), AssessError> {
    let mut seen = BTreeSet::new();
    for id in ids {
        if !seen.insert(id) {
            return Err(AssessError::DuplicateId(id.to_string()));
        }
    }
    Ok(())
}

fn validate_set_item(universe: &[Id], correct: &[Id]) -> Result<(), AssessError> {
    if universe.is_empty() {
        return Err(AssessError::EmptyOptions);
    }
    require_unique(universe)?;
    if correct.is_empty() {
        return Err(AssessError::EmptyCorrectSet);
    }
    require_unique(correct)?;
    for c in correct {
        if !universe.contains(c) {
            return Err(AssessError::CorrectNotInOptions(c.to_string()));
        }
    }
    Ok(())
}

fn validate_sequence(steps: &[Id], credit: SequenceCredit) -> Result<(), AssessError> {
    if steps.is_empty() {
        return Err(AssessError::EmptySequence);
    }
    require_unique(steps)?;
    if matches!(credit, SequenceCredit::AdjacentPairs) && steps.len() < 2 {
        return Err(AssessError::AdjacentPairsTooShort);
    }
    Ok(())
}

fn gcd_u64(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

/// Lift a four-letter MCQ into [`Item::SingleSelect`].
///
/// This is a one-way lift so an existing A–D item can be scored here without
/// changing its grade. New kinds must not be encoded as letters.
pub fn lift_letter_mcq(correct: &str) -> Result<Item, AssessError> {
    Item::single_select(["A", "B", "C", "D"], correct)
}
