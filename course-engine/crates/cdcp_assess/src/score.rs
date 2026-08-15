//! Pure scoring. Integer and rational comparisons only.
use crate::error::AssessError;
use crate::ratio::Ratio;
use crate::types::{Item, Response, Score, SequenceCredit, SetCredit, Tolerance, ToleranceKind};
use std::collections::BTreeSet;

pub fn score(item: &Item, response: &Response) -> Result<Score, AssessError> {
    item.validate()?;
    response.validate()?;
    match (item, response) {
        (Item::SingleSelect { options, correct }, Response::SingleSelect { chosen }) => {
            score_single(options, correct, chosen)
        }
        (
            Item::MultiSelect {
                options,
                correct,
                credit,
            },
            Response::MultiSelect { chosen },
        ) => score_set(options, correct, chosen, *credit),
        (Item::Ordering { elements, credit }, Response::Ordering { order }) => {
            score_sequence(elements, order, *credit)
        }
        (
            Item::NumericRange {
                expected,
                tolerance,
            },
            Response::NumericRange { submitted },
        ) => {
            if expected.units != submitted.units {
                return Err(AssessError::UnitMismatch {
                    expected: expected.units.as_str().to_string(),
                    got: submitted.units.as_str().to_string(),
                });
            }
            if in_tolerance(submitted.value, expected.value, *tolerance)? {
                Ok(Score::full())
            } else {
                Ok(Score::zero())
            }
        }
        (
            Item::TopologySelection {
                elements,
                correct,
                credit,
            },
            Response::TopologySelection { chosen },
        ) => score_set(elements, correct, chosen, *credit),
        (
            Item::ProceduralSequence { steps, credit },
            Response::ProceduralSequence { steps: got },
        ) => score_sequence(steps, got, *credit),
        (item, response) => Err(AssessError::KindMismatch {
            item: item.kind_name(),
            response: response.kind_name(),
        }),
    }
}

fn score_single(
    options: &[crate::types::Id],
    correct: &crate::types::Id,
    chosen: &crate::types::Id,
) -> Result<Score, AssessError> {
    if !options.iter().any(|o| o == chosen) {
        return Err(AssessError::UnknownId(chosen.to_string()));
    }
    if chosen == correct {
        Ok(Score::full())
    } else {
        Ok(Score::zero())
    }
}

fn score_set(
    universe: &[crate::types::Id],
    correct: &[crate::types::Id],
    chosen: &[crate::types::Id],
    credit: SetCredit,
) -> Result<Score, AssessError> {
    for c in chosen {
        if !universe.iter().any(|u| u == c) {
            return Err(AssessError::UnknownId(c.to_string()));
        }
    }
    let correct: BTreeSet<_> = correct.iter().collect();
    let chosen: BTreeSet<_> = chosen.iter().collect();
    match credit {
        SetCredit::AllOrNothing => {
            if chosen == correct {
                Ok(Score::full())
            } else {
                Ok(Score::zero())
            }
        }
        SetCredit::Jaccard => {
            let inter = chosen.intersection(&correct).count() as u64;
            let union = chosen.union(&correct).count() as u64;
            // `correct` is non-empty (schema), so union > 0.
            Score::new(inter, union)
        }
    }
}

fn score_sequence(
    key: &[crate::types::Id],
    got: &[crate::types::Id],
    credit: SequenceCredit,
) -> Result<Score, AssessError> {
    for g in got {
        if !key.iter().any(|k| k == g) {
            return Err(AssessError::UnknownId(g.to_string()));
        }
    }
    match credit {
        SequenceCredit::AllOrNothing => {
            if got == key {
                Ok(Score::full())
            } else {
                Ok(Score::zero())
            }
        }
        SequenceCredit::PositionMatches => {
            let earned = key
                .iter()
                .enumerate()
                .filter(|(i, k)| got.get(*i) == Some(*k))
                .count() as u64;
            Score::new(earned, key.len() as u64)
        }
        SequenceCredit::AdjacentPairs => {
            // Schema already rejected key.len() < 2.
            let mut earned = 0u64;
            for pair in key.windows(2) {
                if got.windows(2).any(|w| w == pair) {
                    earned += 1;
                }
            }
            Score::new(earned, (key.len() - 1) as u64)
        }
    }
}

/// `|submitted - expected| <= tolerance` using only integer compares.
fn in_tolerance(submitted: Ratio, expected: Ratio, tol: Tolerance) -> Result<bool, AssessError> {
    let sn = i128::from(submitted.num());
    let sd = i128::from(submitted.den());
    let en = i128::from(expected.num());
    let ed = i128::from(expected.den());
    let tn = i128::from(tol.magnitude.num());
    let td = i128::from(tol.magnitude.den());
    // dens and |tolerance| are non-negative by construction.
    let sd_u = u128::try_from(sd).map_err(|_| AssessError::Overflow)?;
    let ed_u = u128::try_from(ed).map_err(|_| AssessError::Overflow)?;
    let td_u = u128::try_from(td).map_err(|_| AssessError::Overflow)?;
    let tn_u = u128::try_from(tn).map_err(|_| AssessError::Overflow)?;

    // |sn/sd - en/ed| = |sn*ed - en*sd| / (sd*ed)
    let cross = sn
        .checked_mul(ed)
        .and_then(|a| en.checked_mul(sd).and_then(|b| a.checked_sub(b)))
        .ok_or(AssessError::Overflow)?;
    let abs_cross = cross.unsigned_abs();

    let lhs = abs_cross.checked_mul(td_u).ok_or(AssessError::Overflow)?;
    let rhs = match tol.kind {
        // abs_cross / (sd*ed) <= tn/td  ⇒  abs_cross * td <= tn * sd * ed
        ToleranceKind::Absolute => tn_u
            .checked_mul(sd_u)
            .and_then(|v| v.checked_mul(ed_u))
            .ok_or(AssessError::Overflow)?,
        // abs_cross / (sd*ed) <= |en/ed| * (tn/td)
        // ⇒ abs_cross * td <= |en| * tn * sd
        ToleranceKind::Relative => en
            .unsigned_abs()
            .checked_mul(tn_u)
            .and_then(|v| v.checked_mul(sd_u))
            .ok_or(AssessError::Overflow)?,
    };
    Ok(lhs <= rhs)
}
