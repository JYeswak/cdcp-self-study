//! Bounded metric values. A number without a [`Boundary`] cannot be built.

use crate::boundary::{
    Boundary, EnergyWaterMix, HydroReservoir, ReusePolicy, ScopeItem, WaterScope,
};
use crate::error::MetricsError;
use crate::kind::MetricKind;
use crate::ratio::Ratio;
use std::fmt;

/// Energy-water intensity factor used by source WUE.
///
/// `WUE_source = EWIF × PUE + WUE_site` (Green Grid WP#35 eq. 4).
/// Constructing the TGG "unknown" default of 9/5 L/kWh with hydro
/// excluded is [`MetricsError::EwifExcludesHydroTwice`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ewif {
    value: Ratio,
    hydro: HydroReservoir,
    mix: EnergyWaterMix,
    mix_id: Option<String>,
}

impl Ewif {
    /// Declare an EWIF. The 9/5 (1.8 L/kWh) national-unknown default
    /// with hydro excluded is refused — that figure is thermoelectric-only
    /// *and* TGG set hydro to 0.
    pub fn new(
        liters_per_kwh: Ratio,
        hydro: HydroReservoir,
        mix: EnergyWaterMix,
    ) -> Result<Self, MetricsError> {
        Self::new_named(liters_per_kwh, hydro, mix, None)
    }

    /// EWIF with a named balancing-authority (or other mix) id.
    pub fn new_named(
        liters_per_kwh: Ratio,
        hydro: HydroReservoir,
        mix: EnergyWaterMix,
        mix_id: Option<String>,
    ) -> Result<Self, MetricsError> {
        let _ = crate::error::EWIF_EXCLUDES_HYDRO_TWICE;
        if liters_per_kwh.is_negative() {
            return Err(MetricsError::NegativeValue);
        }
        if is_tgg_unknown_18(liters_per_kwh, hydro, mix) {
            return Err(MetricsError::EwifExcludesHydroTwice);
        }
        let mix_id = mix_id
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        Ok(Self {
            value: liters_per_kwh,
            hydro,
            mix,
            mix_id,
        })
    }

    /// The TGG WP#35 "unknown" default. Always [`MetricsError::EwifExcludesHydroTwice`].
    pub fn tgg_unknown_default() -> Result<Self, MetricsError> {
        Self::new(
            tgg_unknown_18(),
            HydroReservoir::Excluded,
            EnergyWaterMix::NationalUnknown,
        )
    }

    /// L/kWh as a rational.
    #[must_use]
    pub fn value(&self) -> Ratio {
        self.value
    }

    /// Hydro-reservoir treatment.
    #[must_use]
    pub fn hydro(&self) -> HydroReservoir {
        self.hydro
    }

    /// Mix claim.
    #[must_use]
    pub fn mix(&self) -> EnergyWaterMix {
        self.mix
    }

    /// Named mix id, if any.
    #[must_use]
    pub fn mix_id(&self) -> Option<&str> {
        self.mix_id.as_deref()
    }
}

fn tgg_unknown_18() -> Ratio {
    Ratio::new(9, 5).expect("9/5 is a valid ratio")
}

fn is_tgg_unknown_18(value: Ratio, hydro: HydroReservoir, mix: EnergyWaterMix) -> bool {
    value == tgg_unknown_18()
        && hydro == HydroReservoir::Excluded
        && matches!(
            mix,
            EnergyWaterMix::NationalUnknown | EnergyWaterMix::NationalAverage
        )
}

/// A KPI that cannot exist without its control volume.
///
/// Display always prints the boundary. There is no method that returns
/// a floating-point marketing number.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Metric {
    kind: MetricKind,
    value: Ratio,
    boundary: Boundary,
}

impl Metric {
    /// Report an already-measured value. Boundary is required.
    pub fn declared(
        kind: MetricKind,
        value: Ratio,
        boundary: Boundary,
    ) -> Result<Self, MetricsError> {
        if value.is_negative() {
            return Err(MetricsError::NegativeValue);
        }
        boundary.validate_for(kind)?;
        if kind == MetricKind::Pue {
            require_pue_ge_one(value)?;
        }
        Ok(Self {
            kind,
            value,
            boundary,
        })
    }

    /// `PUE = facility / it`. Both in the same energy unit.
    pub fn pue(facility: u64, it: u64, boundary: Boundary) -> Result<Self, MetricsError> {
        boundary.validate_for(MetricKind::Pue)?;
        let value = ratio_quot(facility, it)?;
        require_pue_ge_one(value)?;
        Ok(Self {
            kind: MetricKind::Pue,
            value,
            boundary,
        })
    }

    /// Site `WUE = water_liters / it_kwh`.
    pub fn wue(water_liters: u64, it_kwh: u64, boundary: Boundary) -> Result<Self, MetricsError> {
        if boundary.water_scope() != Some(WaterScope::Site)
            && boundary.water_scope() != Some(WaterScope::Source)
        {
            // validate_for catches missing; source form should go through
            // [`Metric::wue_source`] so the EWIF is declared.
        }
        boundary.validate_for(MetricKind::Wue)?;
        if boundary.water_scope() == Some(WaterScope::Source) {
            return Err(MetricsError::KindMismatch(
                "source WUE must be computed via wue_source (EWIF × PUE + site WUE)".into(),
            ));
        }
        let value = ratio_quot(water_liters, it_kwh)?;
        Ok(Self {
            kind: MetricKind::Wue,
            value,
            boundary,
        })
    }

    /// `WUE_source = EWIF × PUE + WUE_site`.
    ///
    /// IT meter points must match. The site WUE must be a site-scope WUE.
    /// The EWIF constructor has already refused the TGG 1.8 lie.
    pub fn wue_source(ewif: &Ewif, pue: &Metric, site: &Metric) -> Result<Self, MetricsError> {
        if pue.kind != MetricKind::Pue {
            return Err(MetricsError::KindMismatch("wue_source needs a PUE".into()));
        }
        if site.kind != MetricKind::Wue {
            return Err(MetricsError::KindMismatch(
                "wue_source needs a site WUE".into(),
            ));
        }
        if site.boundary.water_scope() != Some(WaterScope::Site) {
            return Err(MetricsError::KindMismatch(
                "wue_source site term is not water_scope=site".into(),
            ));
        }
        if pue.boundary.it_meter() != site.boundary.it_meter() {
            return Err(MetricsError::IncomparableBoundaries { kind: "wue" });
        }
        let value = ewif.value.checked_mul(pue.value)?.checked_add(site.value)?;
        let mut includes = site.boundary.includes().clone();
        includes.insert(ScopeItem::EnergyWater);
        let mut excludes = site.boundary.excludes().clone();
        excludes.remove(&ScopeItem::EnergyWater);
        let mut spec_boundary = Boundary::wue(
            site.boundary.it_meter(),
            WaterScope::Source,
            Some(ewif.hydro),
            includes,
            excludes,
        )?;
        spec_boundary = spec_boundary.with_source_water(ewif.hydro, ewif.mix, ewif.mix_id.clone());
        spec_boundary.validate_for(MetricKind::Wue)?;
        Ok(Self {
            kind: MetricKind::Wue,
            value,
            boundary: spec_boundary,
        })
    }

    /// `CUE = co2 / it`.
    pub fn cue(co2: u64, it: u64, boundary: Boundary) -> Result<Self, MetricsError> {
        boundary.validate_for(MetricKind::Cue)?;
        let value = ratio_quot(co2, it)?;
        Ok(Self {
            kind: MetricKind::Cue,
            value,
            boundary,
        })
    }

    /// `ERE = (facility − reuse) / it`. Reuse must be actually consumed.
    pub fn ere(
        facility: u64,
        reuse: u64,
        it: u64,
        boundary: Boundary,
    ) -> Result<Self, MetricsError> {
        boundary.validate_for(MetricKind::Ere)?;
        if boundary.reuse() != Some(ReusePolicy::ActuallyConsumed) {
            return Err(MetricsError::ReuseNotConsumed);
        }
        if reuse > facility {
            return Err(MetricsError::ReuseExceedsFacility);
        }
        let net = facility - reuse;
        let value = ratio_quot(net, it)?;
        Ok(Self {
            kind: MetricKind::Ere,
            value,
            boundary,
        })
    }

    /// Kind.
    #[must_use]
    pub fn kind(&self) -> MetricKind {
        self.kind
    }

    /// Rational value. Compare through [`Metric::compare`], never by extracting
    /// this and ignoring the boundary.
    #[must_use]
    pub fn value(&self) -> Ratio {
        self.value
    }

    /// Control volume. Always present.
    #[must_use]
    pub fn boundary(&self) -> &Boundary {
        &self.boundary
    }

    /// Compare two metrics. Different kinds or different boundaries
    /// are [`MetricsError::IncomparableBoundaries`] — not a number.
    ///
    /// Not `Ord::cmp`: unequal boundaries are an error, not an order.
    pub fn compare(&self, other: &Self) -> Result<std::cmp::Ordering, MetricsError> {
        let _ = crate::error::INCOMPARABLE;
        if self.kind != other.kind || self.boundary != other.boundary {
            return Err(MetricsError::IncomparableBoundaries {
                kind: self.kind.as_str(),
            });
        }
        self.value.cmp_ratio(other.value)
    }
}

impl fmt::Display for Metric {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.kind.as_str(), self.value, self.boundary)
    }
}

fn ratio_quot(num: u64, den: u64) -> Result<Ratio, MetricsError> {
    if den == 0 {
        return Err(MetricsError::ZeroItEnergy);
    }
    let n = i64::try_from(num).map_err(|_| MetricsError::Overflow)?;
    let d = i64::try_from(den).map_err(|_| MetricsError::Overflow)?;
    Ratio::new(n, d)
}

fn require_pue_ge_one(value: Ratio) -> Result<(), MetricsError> {
    if value.cmp_ratio(Ratio::from_int(1))? == std::cmp::Ordering::Less {
        return Err(MetricsError::PueLessThanOne);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boundary::{CarbonAccounting, ItMeter, ScopeItem};

    fn test_pue_boundary() -> Boundary {
        Boundary::pue(
            ItMeter::UpsOutput,
            [
                ScopeItem::ItEnergy,
                ScopeItem::Cooling,
                ScopeItem::Lighting,
                ScopeItem::UpsLosses,
            ],
            [ScopeItem::GeneratorTesting, ScopeItem::OfficeHvac],
        )
        .expect("test pue boundary")
    }

    fn site_wue_boundary() -> Boundary {
        Boundary::wue(
            ItMeter::UpsOutput,
            WaterScope::Site,
            None,
            [
                ScopeItem::CoolingTowerEvaporation,
                ScopeItem::Blowdown,
                ScopeItem::Humidification,
            ],
            [ScopeItem::FireWater, ScopeItem::EnergyWater],
        )
        .expect("site wue")
    }

    #[test]
    fn pue_is_facility_over_it() {
        let m = Metric::pue(120, 100, test_pue_boundary()).unwrap();
        assert_eq!(m.kind(), MetricKind::Pue);
        assert_eq!(m.value(), Ratio::new(6, 5).unwrap());
        assert!(m.to_string().contains("it_meter=ups-output"));
        assert!(m.to_string().contains("includes="));
    }

    #[test]
    fn pue_below_one_is_error() {
        let err = Metric::pue(99, 100, test_pue_boundary()).unwrap_err();
        assert_eq!(err, MetricsError::PueLessThanOne);
    }

    #[test]
    fn zero_it_is_error() {
        let err = Metric::pue(100, 0, test_pue_boundary()).unwrap_err();
        assert_eq!(err, MetricsError::ZeroItEnergy);
    }

    #[test]
    fn same_boundary_compares_by_ratio() {
        let a = Metric::pue(120, 100, test_pue_boundary()).unwrap();
        let b = Metric::pue(150, 100, test_pue_boundary()).unwrap();
        assert_eq!(a.compare(&b).unwrap(), std::cmp::Ordering::Less);
    }

    #[test]
    fn different_it_meter_is_incomparable() {
        let a = Metric::pue(120, 100, test_pue_boundary()).unwrap();
        let other = Boundary::pue(
            ItMeter::ItInput,
            [
                ScopeItem::ItEnergy,
                ScopeItem::Cooling,
                ScopeItem::Lighting,
                ScopeItem::UpsLosses,
            ],
            [ScopeItem::GeneratorTesting, ScopeItem::OfficeHvac],
        )
        .unwrap();
        let b = Metric::pue(120, 100, other).unwrap();
        let err = a.compare(&b).unwrap_err();
        assert!(matches!(err, MetricsError::IncomparableBoundaries { .. }));
        assert!(err.to_string().contains(crate::error::INCOMPARABLE));
    }

    #[test]
    fn tgg_unknown_ewif_is_refused() {
        let err = Ewif::tgg_unknown_default().unwrap_err();
        assert_eq!(err, MetricsError::EwifExcludesHydroTwice);
        assert!(err
            .to_string()
            .contains(crate::error::EWIF_EXCLUDES_HYDRO_TWICE));
    }

    #[test]
    fn thermoelectric_only_18_is_allowed() {
        let e = Ewif::new(
            Ratio::new(9, 5).unwrap(),
            HydroReservoir::Excluded,
            EnergyWaterMix::ThermoelectricOnly,
        )
        .unwrap();
        assert_eq!(e.value(), Ratio::new(9, 5).unwrap());
    }

    #[test]
    fn wue_source_is_ewif_times_pue_plus_site() {
        // Paper: EWIF = 87/20 (4.35), PUE = 6/5 (1.2), site = 9/25 (0.36)
        // source = (87/20)*(6/5) + 9/25 = 261/50 + 18/50 = 279/50
        let ewif = Ewif::new(
            Ratio::new(87, 20).unwrap(),
            HydroReservoir::Included,
            EnergyWaterMix::NationalAverage,
        )
        .unwrap();
        let pue = Metric::pue(120, 100, test_pue_boundary()).unwrap();
        let site = Metric::wue(36, 100, site_wue_boundary()).unwrap();
        assert_eq!(site.value(), Ratio::new(9, 25).unwrap());
        let src = Metric::wue_source(&ewif, &pue, &site).unwrap();
        assert_eq!(src.kind(), MetricKind::Wue);
        assert_eq!(src.boundary().water_scope(), Some(WaterScope::Source));
        assert_eq!(src.boundary().hydro(), Some(HydroReservoir::Included));
        assert_eq!(src.value(), Ratio::new(279, 50).unwrap());
    }

    #[test]
    fn site_and_source_wue_are_incomparable() {
        let ewif = Ewif::new(
            Ratio::new(87, 20).unwrap(),
            HydroReservoir::Included,
            EnergyWaterMix::NationalAverage,
        )
        .unwrap();
        let pue = Metric::pue(120, 100, test_pue_boundary()).unwrap();
        let site = Metric::wue(36, 100, site_wue_boundary()).unwrap();
        let source = Metric::wue_source(&ewif, &pue, &site).unwrap();
        let err = site.compare(&source).unwrap_err();
        assert!(matches!(err, MetricsError::IncomparableBoundaries { .. }));
    }

    #[test]
    fn cue_and_ere_compute() {
        let cue_b = Boundary::cue(
            ItMeter::UpsOutput,
            CarbonAccounting::Scope1And2LocationBased,
            [ScopeItem::Scope1OnSite, ScopeItem::Scope2PurchasedEnergy],
            [ScopeItem::Scope3Upstream],
        )
        .unwrap();
        let c = Metric::cue(34, 100, cue_b).unwrap();
        assert_eq!(c.value(), Ratio::new(17, 50).unwrap());

        let ere_b = Boundary::ere(
            ItMeter::UpsOutput,
            ReusePolicy::ActuallyConsumed,
            [
                ScopeItem::ItEnergy,
                ScopeItem::Cooling,
                ScopeItem::DistrictHeat,
            ],
            [ScopeItem::OfficeHvac],
        )
        .unwrap();
        let e = Metric::ere(120, 20, 100, ere_b).unwrap();
        assert_eq!(e.value(), Ratio::new(1, 1).unwrap());
    }

    #[test]
    fn recovered_not_consumed_cannot_compute_ere() {
        let ere_b = Boundary::ere(
            ItMeter::UpsOutput,
            ReusePolicy::RecoveredNotConsumed,
            [ScopeItem::ItEnergy, ScopeItem::DistrictHeat],
            [ScopeItem::OfficeHvac],
        )
        .unwrap();
        let err = Metric::ere(120, 20, 100, ere_b).unwrap_err();
        assert_eq!(err, MetricsError::ReuseNotConsumed);
    }
}
