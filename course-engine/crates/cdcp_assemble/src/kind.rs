//! Kind gate: assemble will not flatten typed assess items back to A–D.
//!
//! The live bank path ([`crate::assemble`] / [`crate::assemble_with`])
//! samples letter-MCQ [`cdcp_bank::BankItem`]s and presents them through
//! [`assemble_input`]. Extra typed rows can also carry a
//! [`cdcp_assess::Item`]. A multi-select, numeric-range, ordering,
//! topology-selection, or procedural-sequence offer is
//! [`crate::AssembleError::NotLetterMcq`] — never four shuffled strings.

use crate::{rng_from_seed, shuffle_choices, AssembleConfig, AssembleError, AssembledItem};
use cdcp_assess::Item;
use cdcp_bank::BankItem;
use cdcp_core::ChoiceLetter;
use rand::seq::SliceRandom;
use rand::Rng;

/// Kinds assemble will present as a letter / single-select form.
///
/// `letter-mcq` is the implicit kind of a [`BankItem`]. `single-select` is
/// the `cdcp_assess` lift ([`cdcp_assess::lift_letter_mcq`] and semantic
/// single-select). Every other kind is a named refuse.
pub const LETTER_ASSEMBLE_KINDS: &[&str] = &["letter-mcq", "single-select"];

/// One row offered to assemble. The 804-item bank stays on [`crate::assemble`].
#[derive(Debug, Clone, Copy)]
pub enum AssembleInput<'a> {
    /// Existing four-letter bank item. Implicit kind `letter-mcq`.
    LetterMcq(&'a BankItem),
    /// Typed assess item. Only `single-select` is admitted.
    Assess {
        id: &'a str,
        module: u32,
        stem: &'a str,
        item: &'a Item,
    },
}

impl<'a> AssembleInput<'a> {
    pub fn id(self) -> &'a str {
        match self {
            Self::LetterMcq(item) => item.id.as_str(),
            Self::Assess { id, .. } => id,
        }
    }

    /// Wire kind name. Bank items are `letter-mcq`; assess items use the
    /// `cdcp_assess` tag (`single-select`, `multi-select`, …).
    pub fn kind_name(self) -> &'a str {
        match self {
            Self::LetterMcq(_) => "letter-mcq",
            Self::Assess { item, .. } => item.kind_name(),
        }
    }
}

/// Admit a kind for letter-form assembly, or refuse.
///
/// Unknown kinds fail closed. An empty kind string is not letter-mcq.
pub fn admit_assemble_kind(id: impl Into<String>, kind: &str) -> Result<(), AssembleError> {
    if LETTER_ASSEMBLE_KINDS.contains(&kind) {
        Ok(())
    } else {
        Err(AssembleError::NotLetterMcq {
            id: id.into(),
            kind: kind.to_string(),
        })
    }
}

/// Assemble a caller-supplied input list, or refuse non-letter kinds.
///
/// Kind is checked on **every** row **before** any choice shuffle, so a
/// planted multi-select or numeric-range cannot leak out as four shuffled
/// strings. Stratified bank sampling remains [`crate::assemble`].
///
/// An empty input is an ERROR, not an empty exam.
pub fn assemble_input(
    input: &[AssembleInput<'_>],
    seed: u64,
    cfg: AssembleConfig,
) -> Result<Vec<AssembledItem>, AssembleError> {
    if input.is_empty() {
        return Err(AssembleError::EmptyInput);
    }
    // Fail closed on kind first. Shuffle never sees a refused row.
    for offer in input {
        admit_assemble_kind(offer.id(), offer.kind_name())?;
    }

    let mut rng = rng_from_seed(seed ^ 0xCDC5_FF1E_u64);
    let mut out = Vec::with_capacity(input.len());
    for offer in input {
        out.push(present_input(*offer, cfg.shuffle_choices, &mut rng)?);
    }
    Ok(out)
}

fn present_input(
    offer: AssembleInput<'_>,
    shuffle: bool,
    rng: &mut impl Rng,
) -> Result<AssembledItem, AssembleError> {
    match offer {
        AssembleInput::LetterMcq(item) => present_letter_mcq(item, shuffle, rng),
        AssembleInput::Assess {
            id,
            module,
            stem,
            item,
        } => present_assess(id, module, stem, item, shuffle, rng),
    }
}

fn present_letter_mcq(
    bank_item: &BankItem,
    shuffle: bool,
    rng: &mut impl Rng,
) -> Result<AssembledItem, AssembleError> {
    if bank_item.choices.len() != 4 {
        return Err(AssembleError::BadChoices(bank_item.id.clone()));
    }
    let original = bank_item
        .correct_letter()
        .map_err(|e| AssembleError::Item(bank_item.id.clone(), e.to_string()))?;
    let (choices, correct) = if shuffle {
        shuffle_choices(&bank_item.choices, original, rng)?
    } else {
        (bank_item.choices.clone(), original)
    };
    Ok(AssembledItem {
        id: bank_item.id.clone(),
        module: bank_item.module,
        stem: bank_item.stem.clone(),
        choices,
        correct: correct.as_str().to_string(),
        original_correct: original.as_str().to_string(),
    })
}

fn present_assess(
    id: &str,
    module: u32,
    stem: &str,
    item: &Item,
    shuffle: bool,
    rng: &mut impl Rng,
) -> Result<AssembledItem, AssembleError> {
    // Belt: assemble_input already admitted; refuse again so a direct call
    // cannot flatten.
    admit_assemble_kind(id, item.kind_name())?;
    match item {
        Item::SingleSelect { options, correct } => {
            if is_lift_letter_mcq(options, correct) {
                present_lift_letter(id, module, stem, options, correct, shuffle, rng)
            } else {
                present_semantic_single(id, module, stem, options, correct, shuffle, rng)
            }
        }
        other => Err(AssembleError::NotLetterMcq {
            id: id.to_string(),
            kind: other.kind_name().to_string(),
        }),
    }
}

/// `lift_letter_mcq` shape: options are exactly A–D in order and the key is a letter.
fn is_lift_letter_mcq(options: &[cdcp_assess::Id], correct: &cdcp_assess::Id) -> bool {
    let labels: Vec<&str> = options.iter().map(|o| o.as_str()).collect();
    labels.as_slice() == ["A", "B", "C", "D"] && ChoiceLetter::parse(correct.as_str()).is_ok()
}

fn present_lift_letter(
    id: &str,
    module: u32,
    stem: &str,
    options: &[cdcp_assess::Id],
    correct: &cdcp_assess::Id,
    shuffle: bool,
    rng: &mut impl Rng,
) -> Result<AssembledItem, AssembleError> {
    let choices: Vec<String> = options.iter().map(|o| o.as_str().to_string()).collect();
    let original = ChoiceLetter::parse(correct.as_str())
        .map_err(|e| AssembleError::Item(id.to_string(), e.to_string()))?;
    let (choices, new_correct) = if shuffle {
        shuffle_choices(&choices, original, rng)?
    } else {
        (choices, original)
    };
    Ok(AssembledItem {
        id: id.to_string(),
        module,
        stem: stem.to_string(),
        choices,
        correct: new_correct.as_str().to_string(),
        original_correct: original.as_str().to_string(),
    })
}

/// Semantic single-select: keep option ids. Do **not** rewrite them to A–D.
fn present_semantic_single(
    id: &str,
    module: u32,
    stem: &str,
    options: &[cdcp_assess::Id],
    correct: &cdcp_assess::Id,
    shuffle: bool,
    rng: &mut impl Rng,
) -> Result<AssembledItem, AssembleError> {
    let mut choices: Vec<String> = options.iter().map(|o| o.as_str().to_string()).collect();
    let original = correct.as_str().to_string();
    if shuffle {
        choices.shuffle(rng);
    }
    Ok(AssembledItem {
        id: id.to_string(),
        module,
        stem: stem.to_string(),
        choices,
        correct: original.clone(),
        original_correct: original,
    })
}
