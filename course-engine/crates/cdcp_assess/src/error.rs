//! Named failures. A missing field or an empty set is an error, never a zero.
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AssessError {
    #[error("empty id is a schema ERROR")]
    EmptyId,
    #[error("duplicate id: {0}")]
    DuplicateId(String),
    #[error("empty option/element list is a schema ERROR")]
    EmptyOptions,
    #[error("single-select requires at least two options (one option is vacuous)")]
    TooFewOptions,
    #[error("correct id {0} is not among the options")]
    CorrectNotInOptions(String),
    #[error("empty correct set is a schema ERROR")]
    EmptyCorrectSet,
    #[error("empty sequence is a schema ERROR")]
    EmptySequence,
    #[error(
        "adjacent-pairs credit is a schema ERROR on a sequence of length < 2 (out_of would be 0)"
    )]
    AdjacentPairsTooShort,
    #[error("empty units is a schema ERROR — a bare number cannot be scored")]
    BareNumber,
    #[error("tolerance magnitude must be >= 0")]
    NegativeTolerance,
    #[error("kind mismatch: item is {item}, response is {response}")]
    KindMismatch {
        item: &'static str,
        response: &'static str,
    },
    #[error("response id {0} is not in the item universe")]
    UnknownId(String),
    #[error("response contains a duplicate id: {0}")]
    DuplicateResponseId(String),
    #[error("unit mismatch: item has {expected}, response has {got}")]
    UnitMismatch { expected: String, got: String },
    #[error("ratio denominator must be > 0")]
    ZeroDenominator,
    #[error("integer overflow in rational arithmetic")]
    Overflow,
    #[error("score denominator must be > 0")]
    ZeroScoreDenominator,
    #[error("json: {0}")]
    Json(String),
}
