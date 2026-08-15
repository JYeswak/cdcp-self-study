//! `cdcp_evidence` — `Evidence<T>`: certificates that travel INSIDE values.
//!
//! Adapted from frankensim `fs-evidence`. Without model-form evidence a
//! solver can produce beautifully certified WRONG answers. Attaching the
//! certificate to the value makes assumptions visible and non-bypassable.
//!
//! Four uncertainty slices travel with every value:
//! - [`NumericalCertificate`] — `kind` + `[lo, hi]` enclosure
//! - [`StatisticalCertificate`] — e-values / confidence half-widths
//! - [`ModelEvidence`] — cards, assumptions, validity, discrepancy, `in_domain`
//! - [`SensitivitySummary`] — `d_qoi: {param → d(qoi)/d(param)}`
//!
//! [`Evidence::combine`] is CONSERVATIVE by construction:
//! - numerical enclosures take the **hull** then round **outward** one ulp
//!   (never a tighter enclosure than either input);
//! - validity domains **intersect** (never a wider domain than either input);
//! - `discrepancy_rel` **adds** (first-order conservative);
//! - `in_domain` is AND (false the moment any constituent was out of domain).
//!
//! Composition logic never uses floating-point equality (`==` / `!=` on
//! `f64`). Bounds are ordered with `<` / `>` / `<=` / `>=` and classified
//! with `is_nan` / `is_finite`.
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use thiserror::Error;

/// How strong the numerical bound is. Composition takes the weakest
/// (`PartialOrd`: later variants are weaker) and float composition never
/// claims [`NumericalKind::Exact`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NumericalKind {
    /// Exact (integer-representable results, by-construction identities).
    Exact,
    /// Rigorous outward-rounded enclosure.
    Enclosure,
    /// Reported band without rigor.
    Estimate,
    /// No numerical claim: the band is the whole line.
    NoClaim,
}

/// Canonical `(lo ≤ hi)` that PROPAGATES NaN. `f64::min` / `f64::max`
/// silently discard a NaN operand — normalizing `(NaN, 1.0)` to `(1.0, 1.0)`
/// would mint razor-thin false precision from a garbage bound.
fn ordered_bounds(lo: f64, hi: f64) -> (f64, f64) {
    if lo.is_nan() || hi.is_nan() {
        (f64::NAN, f64::NAN)
    } else if lo <= hi {
        (lo, hi)
    } else {
        (hi, lo)
    }
}

/// The numerical slice: `[lo, hi]` encloses (or estimates) the scalar QoI.
#[derive(Debug, Clone, Copy)]
pub struct NumericalCertificate {
    /// Bound strength.
    pub kind: NumericalKind,
    /// Lower bound on the QoI.
    pub lo: f64,
    /// Upper bound on the QoI.
    pub hi: f64,
}

impl NumericalCertificate {
    /// An exact value (degenerate enclosure).
    #[must_use]
    pub fn exact(v: f64) -> Self {
        NumericalCertificate {
            kind: NumericalKind::Exact,
            lo: v,
            hi: v,
        }
    }

    /// A rigorous enclosure. Reversed finite endpoints normalize; a NaN
    /// endpoint is preserved as an unusable bound.
    #[must_use]
    pub fn enclosure(lo: f64, hi: f64) -> Self {
        let (lo, hi) = ordered_bounds(lo, hi);
        NumericalCertificate {
            kind: NumericalKind::Enclosure,
            lo,
            hi,
        }
    }

    /// A non-rigorous band.
    #[must_use]
    pub fn estimate(lo: f64, hi: f64) -> Self {
        let (lo, hi) = ordered_bounds(lo, hi);
        NumericalCertificate {
            kind: NumericalKind::Estimate,
            lo,
            hi,
        }
    }

    /// The explicit refusal: the whole line, no numerical claim.
    #[must_use]
    pub fn no_claim() -> Self {
        NumericalCertificate {
            kind: NumericalKind::NoClaim,
            lo: f64::NEG_INFINITY,
            hi: f64::INFINITY,
        }
    }

    fn whole_line(kind: NumericalKind) -> Self {
        NumericalCertificate {
            kind,
            lo: f64::NEG_INFINITY,
            hi: f64::INFINITY,
        }
    }

    /// True when `self` is a **strictly tighter** enclosure than `other`:
    /// a strictly higher lower bound or a strictly lower upper bound.
    ///
    /// NaN on either side is unusable, not "tighter". An inverted
    /// candidate (`lo > hi`) that raised the floor or cut the ceiling
    /// still counts as tighter — that is the inward-rounding plant.
    #[must_use]
    pub fn is_tighter_than(&self, other: &Self) -> bool {
        if self.lo.is_nan() || self.hi.is_nan() || other.lo.is_nan() || other.hi.is_nan() {
            return false;
        }
        self.lo > other.lo || self.hi < other.hi
    }

    /// Conservative composition of two numerical certificates.
    ///
    /// COMPOSITION LOGIC: hull of the two intervals, then one-ulp outward
    /// rounding. `NoClaim` absorbs. NaN or inverted inputs fail closed to
    /// the whole line. Float composition never claims `Exact`.
    ///
    /// This function is the load-bearing rounding site. It must not use
    /// floating-point equality.
    #[must_use]
    pub fn combine(a: &Self, b: &Self) -> Self {
        if matches!(a.kind, NumericalKind::NoClaim) || matches!(b.kind, NumericalKind::NoClaim) {
            return Self::no_claim();
        }
        let kind = a.kind.max(b.kind).max(NumericalKind::Enclosure);
        if a.lo.is_nan() || a.hi.is_nan() || b.lo.is_nan() || b.hi.is_nan() {
            return Self::whole_line(kind);
        }
        if a.lo > a.hi || b.lo > b.hi {
            return Self::whole_line(kind);
        }
        // Hull via total-order comparisons — not f64::min/max (those
        // discard NaN; we already failed closed above).
        let lo = if a.lo < b.lo { a.lo } else { b.lo };
        let hi = if a.hi > b.hi { a.hi } else { b.hi };
        // Outward rounding: one ulp each way. Inward (`next_up` on lo,
        // `next_down` on hi) is the known-bad the property test must trip.
        NumericalCertificate {
            kind,
            lo: lo.next_down(),
            hi: hi.next_up(),
        }
    }
}

/// The statistical slice. Composition is conservative-weakest: half-widths
/// add and confidence takes the min; mixed kinds keep the width-bearing
/// certificate; `None` is identity.
#[derive(Debug, Clone, Copy)]
pub enum StatisticalCertificate {
    /// No stochastic component.
    None,
    /// An anytime-valid e-value against the stated null level.
    EValue {
        /// Finite, non-negative e-value.
        e: f64,
        /// Finite design level in `(0, 1)`.
        alpha: f64,
    },
    /// A confidence half-width around the QoI.
    HalfWidth {
        /// Finite, non-negative absolute half-width.
        half_width: f64,
        /// Finite confidence level in `(0, 1)`.
        confidence: f64,
    },
}

impl StatisticalCertificate {
    fn invalid(&self) -> bool {
        match *self {
            StatisticalCertificate::None => false,
            StatisticalCertificate::EValue { e, alpha } => {
                !e.is_finite() || e < 0.0 || !alpha.is_finite() || alpha <= 0.0 || alpha >= 1.0
            }
            StatisticalCertificate::HalfWidth {
                half_width,
                confidence,
            } => {
                !half_width.is_finite()
                    || half_width < 0.0
                    || !confidence.is_finite()
                    || confidence <= 0.0
                    || confidence >= 1.0
            }
        }
    }

    /// Conservative-weakest composition. Invalid operands degrade to an
    /// unbounded half-width (never a silent zero).
    #[must_use]
    pub fn combine(a: &Self, b: &Self) -> Self {
        if a.invalid() || b.invalid() {
            return StatisticalCertificate::HalfWidth {
                half_width: f64::INFINITY,
                confidence: f64::MIN_POSITIVE,
            };
        }
        match (*a, *b) {
            (StatisticalCertificate::None, x) | (x, StatisticalCertificate::None) => x,
            (
                StatisticalCertificate::HalfWidth {
                    half_width: w1,
                    confidence: c1,
                },
                StatisticalCertificate::HalfWidth {
                    half_width: w2,
                    confidence: c2,
                },
            ) => StatisticalCertificate::HalfWidth {
                half_width: w1 + w2,
                confidence: if c1 < c2 { c1 } else { c2 },
            },
            (
                StatisticalCertificate::EValue { e: e1, alpha: a1 },
                StatisticalCertificate::EValue { e: e2, alpha: a2 },
            ) => StatisticalCertificate::EValue {
                e: if e1 < e2 { e1 } else { e2 },
                alpha: if a1 > a2 { a1 } else { a2 },
            },
            (
                w @ StatisticalCertificate::HalfWidth { .. },
                StatisticalCertificate::EValue { .. },
            )
            | (
                StatisticalCertificate::EValue { .. },
                w @ StatisticalCertificate::HalfWidth { .. },
            ) => w,
        }
    }
}

/// A named-parameter validity box. Missing parameters are unconstrained.
/// Composition intersects (never wider than either input).
#[derive(Debug, Clone, Default)]
pub struct ValidityDomain {
    bounds: BTreeMap<String, (f64, f64)>,
}

impl ValidityDomain {
    /// The unconstrained domain.
    #[must_use]
    pub fn unconstrained() -> Self {
        ValidityDomain::default()
    }

    /// Declared axis bounds (read-only).
    #[must_use]
    pub fn bounds(&self) -> &BTreeMap<String, (f64, f64)> {
        &self.bounds
    }

    /// Constrain one parameter to `[lo, hi]`. Reversed finite endpoints
    /// normalize; a NaN endpoint is preserved as an unusable domain.
    #[must_use]
    pub fn with(mut self, param: impl Into<String>, lo: f64, hi: f64) -> Self {
        self.bounds.insert(param.into(), ordered_bounds(lo, hi));
        self
    }

    /// Constraint bounds for a parameter, if any.
    #[must_use]
    pub fn bound(&self, param: &str) -> Option<(f64, f64)> {
        self.bounds.get(param).copied()
    }

    /// True when the point satisfies every constraint.
    #[must_use]
    pub fn contains(&self, point: &BTreeMap<String, f64>) -> bool {
        self.bounds.iter().all(|(k, &(lo, hi))| {
            lo.is_finite()
                && hi.is_finite()
                && lo <= hi
                && point
                    .get(k)
                    .is_some_and(|&v| v.is_finite() && v >= lo && v <= hi)
        })
    }

    /// Per-parameter intersection. Composed validity is never wider than
    /// either input. COMPOSITION LOGIC: no floating-point equality.
    #[must_use]
    pub fn intersect(&self, other: &Self) -> Self {
        let mut out = self.bounds.clone();
        for (k, &(lo2, hi2)) in &other.bounds {
            out.entry(k.clone())
                .and_modify(|(lo, hi)| {
                    if lo.is_nan() || hi.is_nan() || lo2.is_nan() || hi2.is_nan() {
                        *lo = f64::NAN;
                        *hi = f64::NAN;
                    } else {
                        if lo2 > *lo {
                            *lo = lo2;
                        }
                        if hi2 < *hi {
                            *hi = hi2;
                        }
                    }
                })
                .or_insert((lo2, hi2));
        }
        ValidityDomain { bounds: out }
    }

    /// True when some parameter's interval is empty or unusable.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bounds
            .values()
            .any(|&(lo, hi)| lo.is_nan() || hi.is_nan() || lo > hi)
    }

    /// True when `self` is **wider** than `other`: it drops a constraint
    /// `other` holds, or loosens an axis (`lo` strictly lower or `hi`
    /// strictly higher). Unusable (NaN / empty) bounds on `other` do not
    /// count as constraints.
    #[must_use]
    pub fn is_wider_than(&self, other: &Self) -> bool {
        for (k, &(olo, ohi)) in &other.bounds {
            if olo.is_nan() || ohi.is_nan() || olo > ohi {
                continue;
            }
            match self.bounds.get(k) {
                None => return true,
                Some(&(slo, shi)) => {
                    if slo.is_nan() || shi.is_nan() {
                        return true;
                    }
                    if slo < olo || shi > ohi {
                        return true;
                    }
                }
            }
        }
        false
    }
}

/// Headline sensitivities `d(qoi)/d(param)`. Merging keeps the larger
/// magnitude per parameter (conservative).
#[derive(Debug, Clone, Default)]
pub struct SensitivitySummary {
    /// Parameter → d(qoi)/d(param).
    pub d_qoi: BTreeMap<String, f64>,
}

impl SensitivitySummary {
    /// Conservative merge: larger absolute derivative wins. COMPOSITION
    /// LOGIC: magnitude compare with `>`, never `==`.
    #[must_use]
    pub fn combine(a: &Self, b: &Self) -> Self {
        let mut out = a.d_qoi.clone();
        for (k, &v) in &b.d_qoi {
            out.entry(k.clone())
                .and_modify(|cur| {
                    if v.abs() > cur.abs() {
                        *cur = v;
                    }
                })
                .or_insert(v);
        }
        SensitivitySummary { d_qoi: out }
    }
}

/// Why a model-form certificate could not be constructed.
#[derive(Debug, Error)]
pub enum EvidenceError {
    /// `discrepancy_rel` is NaN or negative. `+inf` is the explicit
    /// unbounded claim and is accepted.
    #[error(
        "discrepancy_rel {discrepancy_rel} is NaN or negative; \
         +inf is a valid explicit unbounded claim"
    )]
    InvalidDiscrepancy {
        /// The rejected relative discrepancy.
        discrepancy_rel: f64,
    },
}

/// The model-form slice: which cards, whose assumptions, what validity,
/// how big the discrepancy band.
#[derive(Debug, Clone)]
pub struct ModelEvidence {
    /// Names of the model cards in play (sorted, deduplicated).
    pub cards: Vec<String>,
    /// Union of stated assumptions (sorted, deduplicated).
    pub assumptions: Vec<String>,
    /// Validity domain (composition intersects).
    pub validity: ValidityDomain,
    /// Non-negative relative model-form discrepancy band (composition
    /// adds). `+inf` is an explicit unbounded claim; NaN and negatives
    /// are invalid.
    pub discrepancy_rel: f64,
    /// False the moment any constituent was queried outside its domain.
    pub in_domain: bool,
}

impl ModelEvidence {
    /// No model-form claims (pure numerics). `discrepancy_rel = 0`,
    /// `in_domain = true`, unconstrained validity.
    #[must_use]
    pub fn none() -> Self {
        ModelEvidence {
            cards: Vec::new(),
            assumptions: Vec::new(),
            validity: ValidityDomain::unconstrained(),
            discrepancy_rel: 0.0,
            in_domain: true,
        }
    }

    /// True when `discrepancy_rel` is an admissible claim: not NaN, and
    /// `>= 0` (so `+inf` is valid and negatives are not).
    #[must_use]
    pub fn valid_discrepancy(discrepancy_rel: f64) -> bool {
        !discrepancy_rel.is_nan() && discrepancy_rel >= 0.0
    }

    /// Construct a model-form certificate, rejecting NaN / negative
    /// `discrepancy_rel`. Cards and assumptions are sorted and deduped.
    pub fn try_new(
        mut cards: Vec<String>,
        mut assumptions: Vec<String>,
        validity: ValidityDomain,
        discrepancy_rel: f64,
        in_domain: bool,
    ) -> Result<Self, EvidenceError> {
        if !Self::valid_discrepancy(discrepancy_rel) {
            return Err(EvidenceError::InvalidDiscrepancy { discrepancy_rel });
        }
        cards.sort_unstable();
        cards.dedup();
        assumptions.sort_unstable();
        assumptions.dedup();
        Ok(ModelEvidence {
            cards,
            assumptions,
            validity,
            discrepancy_rel,
            in_domain,
        })
    }

    /// Conservative composition. Validity intersects; discrepancy adds
    /// (invalid operands become `+inf` so a NaN cannot be laundered into
    /// a finite band); `in_domain` is AND; cards/assumptions union.
    /// COMPOSITION LOGIC: no floating-point equality.
    #[must_use]
    pub fn combine(a: &Self, b: &Self) -> Self {
        let mut cards = [a.cards.clone(), b.cards.clone()].concat();
        cards.sort_unstable();
        cards.dedup();
        let mut assumptions = [a.assumptions.clone(), b.assumptions.clone()].concat();
        assumptions.sort_unstable();
        assumptions.dedup();
        let discrepancy_rel = if Self::valid_discrepancy(a.discrepancy_rel)
            && Self::valid_discrepancy(b.discrepancy_rel)
        {
            a.discrepancy_rel + b.discrepancy_rel
        } else {
            f64::INFINITY
        };
        ModelEvidence {
            cards,
            assumptions,
            validity: a.validity.intersect(&b.validity),
            discrepancy_rel,
            in_domain: a.in_domain && b.in_domain,
        }
    }
}

/// A value with its full evidence: the noun that travels through every layer.
#[derive(Debug, Clone)]
pub struct Evidence<T> {
    /// The carried value.
    pub value: T,
    /// The scalar quantity of interest the certificates describe (equals
    /// `value` for scalar evidence).
    pub qoi: f64,
    /// Numerical slice.
    pub numerical: NumericalCertificate,
    /// Statistical slice.
    pub statistical: StatisticalCertificate,
    /// Model-form slice.
    pub model: ModelEvidence,
    /// Sensitivity headline.
    pub sensitivity: SensitivitySummary,
}

impl Evidence<f64> {
    /// Scalar evidence with an exact numerical certificate.
    #[must_use]
    pub fn exact(value: f64) -> Self {
        Evidence {
            value,
            qoi: value,
            numerical: NumericalCertificate::exact(value),
            statistical: StatisticalCertificate::None,
            model: ModelEvidence::none(),
            sensitivity: SensitivitySummary::default(),
        }
    }

    /// Scalar evidence with a rigorous enclosure.
    #[must_use]
    pub fn enclosed(value: f64, lo: f64, hi: f64) -> Self {
        Evidence {
            numerical: NumericalCertificate::enclosure(lo, hi),
            ..Evidence::exact(value)
        }
    }
}

impl<T> Evidence<T> {
    /// Attach a statistical certificate.
    #[must_use]
    pub fn with_statistical(mut self, s: StatisticalCertificate) -> Self {
        self.statistical = s;
        self
    }

    /// Attach a model-form certificate.
    #[must_use]
    pub fn with_model(mut self, m: ModelEvidence) -> Self {
        self.model = m;
        self
    }

    /// Attach a sensitivity headline.
    #[must_use]
    pub fn with_sensitivity(mut self, s: SensitivitySummary) -> Self {
        self.sensitivity = s;
        self
    }

    /// Conservative composition of two evidence values.
    ///
    /// Certificates merge by the module-level laws (hull+outward,
    /// intersect, add). The carried `value` is supplied by the caller —
    /// combine does not invent a QoI from the operands.
    #[must_use]
    pub fn combine<U, V>(a: &Evidence<T>, b: &Evidence<U>, value: V) -> Evidence<V> {
        Evidence {
            value,
            qoi: a.qoi,
            numerical: NumericalCertificate::combine(&a.numerical, &b.numerical),
            statistical: StatisticalCertificate::combine(&a.statistical, &b.statistical),
            model: ModelEvidence::combine(&a.model, &b.model),
            sensitivity: SensitivitySummary::combine(&a.sensitivity, &b.sensitivity),
        }
    }
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn crate_forbids_unsafe() {
        let src = include_str!("lib.rs");
        assert!(
            src.contains("#![forbid(unsafe_code)]"),
            "the crate root must forbid unsafe_code"
        );
    }

    #[test]
    fn composition_logic_has_no_float_equality() {
        // Production only — the test module below may mention the banned
        // spellings as needles. Split on the test cfg, not a grep of self.
        let src = include_str!("lib.rs");
        let prod = src
            .split("#[cfg(test)]")
            .next()
            .expect("production source precedes tests");
        let banned = [
            [".lo", " =="].concat(),
            [".hi", " =="].concat(),
            ["discrepancy_rel", " =="].concat(),
            ["== ", "0.0"].concat(),
            ["== ", "f64"].concat(),
            ["!= ", "0.0"].concat(),
            [".lo", " !="].concat(),
            [".hi", " !="].concat(),
        ];
        for needle in &banned {
            assert!(
                !prod.contains(needle),
                "composition logic must not use floating-point equality ({needle})"
            );
        }
    }

    #[test]
    fn exact_combine_rounds_outward_one_ulp() {
        let a = NumericalCertificate::exact(1.0);
        let c = NumericalCertificate::combine(&a, &a);
        assert_eq!(c.kind, NumericalKind::Enclosure);
        assert!(c.lo < 1.0 && c.hi > 1.0, "{c:?}");
        assert!(c.lo.to_bits() == 1.0_f64.next_down().to_bits());
        assert!(c.hi.to_bits() == 1.0_f64.next_up().to_bits());
        assert!(!c.is_tighter_than(&a));
    }

    #[test]
    fn hull_is_never_tighter_than_either_input() {
        let a = NumericalCertificate::enclosure(0.0, 10.0);
        let b = NumericalCertificate::enclosure(4.0, 6.0);
        let c = NumericalCertificate::combine(&a, &b);
        assert!(!c.is_tighter_than(&a), "{c:?} tighter than {a:?}");
        assert!(!c.is_tighter_than(&b), "{c:?} tighter than {b:?}");
        assert!(c.lo < 0.0 && c.hi > 10.0, "outward around the hull: {c:?}");
    }

    #[test]
    fn no_claim_absorbs() {
        let a = NumericalCertificate::exact(1.0);
        let c = NumericalCertificate::combine(&a, &NumericalCertificate::no_claim());
        assert!(matches!(c.kind, NumericalKind::NoClaim));
        assert!(c.lo.is_infinite() && c.hi.is_infinite());
    }

    #[test]
    fn nan_bounds_fail_closed_to_whole_line() {
        let a = NumericalCertificate {
            kind: NumericalKind::Enclosure,
            lo: f64::NAN,
            hi: 1.0,
        };
        let b = NumericalCertificate::exact(0.0);
        let c = NumericalCertificate::combine(&a, &b);
        assert!(c.lo.is_infinite() && c.hi.is_infinite());
    }

    #[test]
    fn discrepancy_rejects_nan_and_negatives_accepts_inf() {
        assert!(ModelEvidence::try_new(
            vec![],
            vec![],
            ValidityDomain::unconstrained(),
            f64::NAN,
            true
        )
        .is_err());
        assert!(ModelEvidence::try_new(
            vec![],
            vec![],
            ValidityDomain::unconstrained(),
            -0.1,
            true
        )
        .is_err());
        assert!(ModelEvidence::try_new(
            vec![],
            vec![],
            ValidityDomain::unconstrained(),
            f64::NEG_INFINITY,
            true
        )
        .is_err());
        let inf = ModelEvidence::try_new(
            vec![],
            vec![],
            ValidityDomain::unconstrained(),
            f64::INFINITY,
            true,
        )
        .expect("+inf is a valid unbounded claim");
        assert!(inf.discrepancy_rel.is_infinite() && inf.discrepancy_rel > 0.0);
        let zero =
            ModelEvidence::try_new(vec![], vec![], ValidityDomain::unconstrained(), 0.0, true)
                .expect("zero is a valid (no-band) claim");
        assert!(zero.discrepancy_rel >= 0.0 && zero.discrepancy_rel < f64::MIN_POSITIVE);
    }

    #[test]
    fn discrepancy_adds_and_invalid_becomes_unbounded() {
        let a = ModelEvidence::try_new(
            vec!["a".into()],
            vec!["assume-a".into()],
            ValidityDomain::unconstrained(),
            0.02,
            true,
        )
        .unwrap();
        let b = ModelEvidence::try_new(
            vec!["b".into()],
            vec!["assume-b".into()],
            ValidityDomain::unconstrained(),
            0.03,
            true,
        )
        .unwrap();
        let c = ModelEvidence::combine(&a, &b);
        let expected = 0.02 + 0.03;
        assert!(c.discrepancy_rel >= expected);
        // first-order add, no shrinkage: composed is at least the sum
        assert!(!(c.discrepancy_rel < expected));
        assert_eq!(c.cards, vec!["a".to_string(), "b".to_string()]);
        assert_eq!(
            c.assumptions,
            vec!["assume-a".to_string(), "assume-b".to_string()]
        );

        let nan = ModelEvidence {
            discrepancy_rel: f64::NAN,
            ..ModelEvidence::none()
        };
        let poisoned = ModelEvidence::combine(&a, &nan);
        assert!(poisoned.discrepancy_rel.is_infinite());
    }

    #[test]
    fn validity_intersects_and_is_never_wider() {
        let a = ValidityDomain::unconstrained().with("Re", 1e4, 1e5);
        let b = ValidityDomain::unconstrained()
            .with("Re", 5e4, 2e5)
            .with("Ma", 0.0, 0.3);
        let i = a.intersect(&b);
        let (lo, hi) = i.bound("Re").expect("Re");
        assert!(lo >= 5e4 && hi <= 1e5);
        assert!(i.bound("Ma").is_some());
        assert!(!i.is_wider_than(&a), "intersect wider than A");
        assert!(!i.is_wider_than(&b), "intersect wider than B");
        let empty = a.intersect(&ValidityDomain::unconstrained().with("Re", 2e5, 3e5));
        assert!(empty.is_empty());
        assert!(!empty.is_wider_than(&a));
    }

    #[test]
    fn in_domain_false_propagates() {
        let a = ModelEvidence {
            in_domain: false,
            ..ModelEvidence::none()
        };
        let b = ModelEvidence::none();
        let c = ModelEvidence::combine(&a, &b);
        assert!(!c.in_domain);
        assert!(ModelEvidence::combine(&b, &b).in_domain);
    }
}
