//! What's in and what's out. A metric cannot exist without this.

use crate::error::MetricsError;
use crate::kind::MetricKind;
use std::collections::BTreeSet;
use std::fmt;

/// Where IT energy is metered. Relocates the same losses between
/// numerator and denominator (Green Grid WP#49 / ISO/IEC 30134-2).
/// A PUE without this is Unrecognized by its own authoring body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ItMeter {
    /// Category 1: UPS output.
    UpsOutput,
    /// Category 2: PDU output.
    PduOutput,
    /// Category 3: IT equipment input.
    ItInput,
}

/// Site water (Scope-1 analog) vs source water (site + energy-water).
/// The figure 1.8 L/kWh names both; they are not the same quantity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WaterScope {
    /// On-site water only.
    Site,
    /// On-site plus water used to produce the electricity consumed.
    Source,
}

/// Whether open-reservoir evaporation from hydropower is attributed
/// to electricity consumption. Must be declared for any source WUE.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HydroReservoir {
    /// Reservoir evaporation is in the EWIF.
    Included,
    /// Reservoir evaporation is out. Honest only when the mix is
    /// thermoelectric-only — never as a national average.
    Excluded,
}

/// What mix an EWIF claims to represent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EnergyWaterMix {
    /// Thermoelectric plants only. The NREL 2003 1.8 L/kWh figure.
    ThermoelectricOnly,
    /// Thermoelectric plus hydro reservoir evaporation.
    ThermoelectricAndHydro,
    /// Claimed national average (must include hydro to be honest).
    NationalAverage,
    /// TGG WP#35 "unknown" bucket. Combined with 1.8 L/kWh + hydro
    /// excluded this is [`MetricsError::EwifExcludesHydroTwice`].
    NationalUnknown,
    /// Named balancing-authority mix (the id lives on [`Boundary`]).
    BalancingAuthority,
}

/// Carbon control volume. Location-based vs market-based is a
/// different number, not a rounding error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CarbonAccounting {
    /// On-site combustion only.
    Scope1,
    /// Scope 1 + 2, location-based.
    Scope1And2LocationBased,
    /// Scope 1 + 2, market-based.
    Scope1And2MarketBased,
    /// Scope 1 + 2 + 3.
    Scope123,
}

/// What ERE is allowed to subtract from facility energy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReusePolicy {
    /// Energy actually consumed by the reuse sink.
    ActuallyConsumed,
    /// Recovered at the plant but not shown to be consumed.
    /// Cannot compute ERE — that is the game.
    RecoveredNotConsumed,
}

/// One named slice of a control volume.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ScopeItem {
    /// IT equipment energy (must sit inside a PUE/ERE facility total).
    ItEnergy,
    /// Mechanical cooling plant and CRAH/CRAC.
    Cooling,
    /// Lighting.
    Lighting,
    /// UPS conversion losses.
    UpsLosses,
    /// Switchgear / transformer losses.
    SwitchgearLosses,
    /// PDU / busway losses.
    PduLosses,
    /// Generator load-bank / test fuel and electricity.
    GeneratorTesting,
    /// Office / admin HVAC.
    OfficeHvac,
    /// Security and BMS parasitism.
    Security,
    /// Cooling-tower evaporation.
    CoolingTowerEvaporation,
    /// Cooling-tower blowdown.
    Blowdown,
    /// Cooling-tower drift.
    Drift,
    /// Humidification water.
    Humidification,
    /// Water used to produce the electricity consumed (EWIF term).
    EnergyWater,
    /// Other process water.
    ProcessWater,
    /// Fire-suppression water (almost always out).
    FireWater,
    /// Hydro reservoir evaporation attributed to electricity.
    HydroReservoirEvaporation,
    /// On-site combustion CO2.
    Scope1OnSite,
    /// Purchased-energy CO2.
    Scope2PurchasedEnergy,
    /// Upstream CO2.
    Scope3Upstream,
    /// Heat exported to a district network.
    DistrictHeat,
    /// Heat exported to an adjacent building.
    AdjacentBuilding,
    /// Heat exported to an industrial process.
    IndustrialProcess,
}

/// Declared control volume. Includes and excludes are both required
/// and must be disjoint. A bare number cannot carry this, so it cannot
/// be a [`crate::Metric`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Boundary {
    it_meter: ItMeter,
    includes: BTreeSet<ScopeItem>,
    excludes: BTreeSet<ScopeItem>,
    water_scope: Option<WaterScope>,
    hydro: Option<HydroReservoir>,
    energy_water_mix: Option<EnergyWaterMix>,
    mix_id: Option<String>,
    carbon: Option<CarbonAccounting>,
    reuse: Option<ReusePolicy>,
}

impl Boundary {
    /// Assemble a boundary. Empty includes or excludes is
    /// [`MetricsError::EmptyBoundary`]. Overlap is
    /// [`MetricsError::BoundaryOverlap`].
    pub fn new(spec: BoundarySpec) -> Result<Self, MetricsError> {
        let includes: BTreeSet<ScopeItem> = spec.includes.into_iter().collect();
        let excludes: BTreeSet<ScopeItem> = spec.excludes.into_iter().collect();
        if includes.is_empty() || excludes.is_empty() {
            let _ = crate::error::EMPTY_BOUNDARY;
            return Err(MetricsError::EmptyBoundary);
        }
        if let Some(item) = includes.intersection(&excludes).next() {
            return Err(MetricsError::BoundaryOverlap(item.as_str().to_string()));
        }
        let mix_id = spec
            .mix_id
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        Ok(Self {
            it_meter: spec.it_meter,
            includes,
            excludes,
            water_scope: spec.water_scope,
            hydro: spec.hydro,
            energy_water_mix: spec.energy_water_mix,
            mix_id,
            carbon: spec.carbon,
            reuse: spec.reuse,
        })
    }

    /// PUE/ERE energy boundary. `includes` must contain [`ScopeItem::ItEnergy`].
    pub fn pue(
        it_meter: ItMeter,
        includes: impl IntoIterator<Item = ScopeItem>,
        excludes: impl IntoIterator<Item = ScopeItem>,
    ) -> Result<Self, MetricsError> {
        let b = Self::new(BoundarySpec {
            it_meter,
            includes: includes.into_iter().collect(),
            excludes: excludes.into_iter().collect(),
            water_scope: None,
            hydro: None,
            energy_water_mix: None,
            mix_id: None,
            carbon: None,
            reuse: None,
        })?;
        b.require_it_in_facility()?;
        Ok(b)
    }

    /// WUE boundary. Source form requires a hydro declaration.
    pub fn wue(
        it_meter: ItMeter,
        water_scope: WaterScope,
        hydro: Option<HydroReservoir>,
        includes: impl IntoIterator<Item = ScopeItem>,
        excludes: impl IntoIterator<Item = ScopeItem>,
    ) -> Result<Self, MetricsError> {
        let b = Self::new(BoundarySpec {
            it_meter,
            includes: includes.into_iter().collect(),
            excludes: excludes.into_iter().collect(),
            water_scope: Some(water_scope),
            hydro,
            energy_water_mix: None,
            mix_id: None,
            carbon: None,
            reuse: None,
        })?;
        b.validate_for(MetricKind::Wue)?;
        Ok(b)
    }

    /// CUE boundary.
    pub fn cue(
        it_meter: ItMeter,
        carbon: CarbonAccounting,
        includes: impl IntoIterator<Item = ScopeItem>,
        excludes: impl IntoIterator<Item = ScopeItem>,
    ) -> Result<Self, MetricsError> {
        let b = Self::new(BoundarySpec {
            it_meter,
            includes: includes.into_iter().collect(),
            excludes: excludes.into_iter().collect(),
            water_scope: None,
            hydro: None,
            energy_water_mix: None,
            mix_id: None,
            carbon: Some(carbon),
            reuse: None,
        })?;
        b.validate_for(MetricKind::Cue)?;
        Ok(b)
    }

    /// ERE boundary. `includes` must contain [`ScopeItem::ItEnergy`].
    pub fn ere(
        it_meter: ItMeter,
        reuse: ReusePolicy,
        includes: impl IntoIterator<Item = ScopeItem>,
        excludes: impl IntoIterator<Item = ScopeItem>,
    ) -> Result<Self, MetricsError> {
        let b = Self::new(BoundarySpec {
            it_meter,
            includes: includes.into_iter().collect(),
            excludes: excludes.into_iter().collect(),
            water_scope: None,
            hydro: None,
            energy_water_mix: None,
            mix_id: None,
            carbon: None,
            reuse: Some(reuse),
        })?;
        b.validate_for(MetricKind::Ere)?;
        Ok(b)
    }

    /// Kind-specific required declarations.
    pub fn validate_for(&self, kind: MetricKind) -> Result<(), MetricsError> {
        match kind {
            MetricKind::Pue => self.require_it_in_facility(),
            MetricKind::Wue => {
                if self.water_scope.is_none() {
                    return Err(MetricsError::MissingDeclaration {
                        kind: kind.as_str(),
                        field: "water_scope",
                    });
                }
                if self.water_scope == Some(WaterScope::Source) && self.hydro.is_none() {
                    return Err(MetricsError::MissingDeclaration {
                        kind: kind.as_str(),
                        field: "hydro",
                    });
                }
                Ok(())
            }
            MetricKind::Cue => {
                if self.carbon.is_none() {
                    return Err(MetricsError::MissingDeclaration {
                        kind: kind.as_str(),
                        field: "carbon",
                    });
                }
                Ok(())
            }
            MetricKind::Ere => {
                self.require_it_in_facility()?;
                if self.reuse.is_none() {
                    return Err(MetricsError::MissingDeclaration {
                        kind: kind.as_str(),
                        field: "reuse",
                    });
                }
                Ok(())
            }
        }
    }

    fn require_it_in_facility(&self) -> Result<(), MetricsError> {
        if !self.includes.contains(&ScopeItem::ItEnergy) {
            return Err(MetricsError::PueWithoutItInBoundary);
        }
        Ok(())
    }

    /// IT energy meter point.
    #[must_use]
    pub fn it_meter(&self) -> ItMeter {
        self.it_meter
    }

    /// What's in.
    #[must_use]
    pub fn includes(&self) -> &BTreeSet<ScopeItem> {
        &self.includes
    }

    /// What's out.
    #[must_use]
    pub fn excludes(&self) -> &BTreeSet<ScopeItem> {
        &self.excludes
    }

    /// Site vs source water. Present on WUE.
    #[must_use]
    pub fn water_scope(&self) -> Option<WaterScope> {
        self.water_scope
    }

    /// Hydro-reservoir treatment. Required for source WUE.
    #[must_use]
    pub fn hydro(&self) -> Option<HydroReservoir> {
        self.hydro
    }

    /// EWIF mix claim.
    #[must_use]
    pub fn energy_water_mix(&self) -> Option<EnergyWaterMix> {
        self.energy_water_mix
    }

    /// Balancing-authority (or other mix) id, if named.
    #[must_use]
    pub fn mix_id(&self) -> Option<&str> {
        self.mix_id.as_deref()
    }

    /// Carbon accounting method.
    #[must_use]
    pub fn carbon(&self) -> Option<CarbonAccounting> {
        self.carbon
    }

    /// ERE reuse policy.
    #[must_use]
    pub fn reuse(&self) -> Option<ReusePolicy> {
        self.reuse
    }

    /// Attach mix metadata used by source WUE.
    pub(crate) fn with_source_water(
        mut self,
        hydro: HydroReservoir,
        mix: EnergyWaterMix,
        mix_id: Option<String>,
    ) -> Self {
        self.water_scope = Some(WaterScope::Source);
        self.hydro = Some(hydro);
        self.energy_water_mix = Some(mix);
        self.mix_id = mix_id;
        self.includes.insert(ScopeItem::EnergyWater);
        if hydro == HydroReservoir::Included {
            self.includes.insert(ScopeItem::HydroReservoirEvaporation);
            self.excludes.remove(&ScopeItem::HydroReservoirEvaporation);
        } else {
            self.excludes.insert(ScopeItem::HydroReservoirEvaporation);
            self.includes.remove(&ScopeItem::HydroReservoirEvaporation);
        }
        self
    }
}

/// Constructor input for [`Boundary::new`].
#[derive(Debug, Clone)]
pub struct BoundarySpec {
    /// IT energy meter point.
    pub it_meter: ItMeter,
    /// What's in.
    pub includes: BTreeSet<ScopeItem>,
    /// What's out.
    pub excludes: BTreeSet<ScopeItem>,
    /// Site vs source. Required for WUE.
    pub water_scope: Option<WaterScope>,
    /// Hydro reservoir. Required for source WUE.
    pub hydro: Option<HydroReservoir>,
    /// EWIF mix claim.
    pub energy_water_mix: Option<EnergyWaterMix>,
    /// Named mix (balancing authority id, …).
    pub mix_id: Option<String>,
    /// Carbon accounting. Required for CUE.
    pub carbon: Option<CarbonAccounting>,
    /// Reuse policy. Required for ERE.
    pub reuse: Option<ReusePolicy>,
}

impl fmt::Display for Boundary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "it_meter={}", self.it_meter.as_str())?;
        write!(f, " includes=")?;
        write_set(f, &self.includes)?;
        write!(f, " excludes=")?;
        write_set(f, &self.excludes)?;
        if let Some(w) = self.water_scope {
            write!(f, " water_scope={}", w.as_str())?;
        }
        if let Some(h) = self.hydro {
            write!(f, " hydro={}", h.as_str())?;
        }
        if let Some(m) = self.energy_water_mix {
            write!(f, " mix={}", m.as_str())?;
        }
        if let Some(id) = &self.mix_id {
            write!(f, " mix_id={id}")?;
        }
        if let Some(c) = self.carbon {
            write!(f, " carbon={}", c.as_str())?;
        }
        if let Some(r) = self.reuse {
            write!(f, " reuse={}", r.as_str())?;
        }
        Ok(())
    }
}

fn write_set(f: &mut fmt::Formatter<'_>, set: &BTreeSet<ScopeItem>) -> fmt::Result {
    write!(f, "{{")?;
    for (i, item) in set.iter().enumerate() {
        if i > 0 {
            write!(f, ",")?;
        }
        write!(f, "{}", item.as_str())?;
    }
    write!(f, "}}")
}

macro_rules! kebab {
    ($ty:ident, $($var:ident => $s:literal),+ $(,)?) => {
        impl $ty {
            /// Stable kebab-case name.
            #[must_use]
            pub fn as_str(self) -> &'static str {
                match self {
                    $(Self::$var => $s,)+
                }
            }
            /// Parse a kebab-case name.
            pub fn parse(s: &str) -> Result<Self, MetricsError> {
                match s {
                    $($s => Ok(Self::$var),)+
                    other => Err(MetricsError::UnknownToken(other.to_string())),
                }
            }
        }
    };
}

kebab!(ItMeter, UpsOutput => "ups-output", PduOutput => "pdu-output", ItInput => "it-input");
kebab!(WaterScope, Site => "site", Source => "source");
kebab!(HydroReservoir, Included => "included", Excluded => "excluded");
kebab!(
    EnergyWaterMix,
    ThermoelectricOnly => "thermoelectric-only",
    ThermoelectricAndHydro => "thermoelectric-and-hydro",
    NationalAverage => "national-average",
    NationalUnknown => "national-unknown",
    BalancingAuthority => "balancing-authority",
);
kebab!(
    CarbonAccounting,
    Scope1 => "scope-1",
    Scope1And2LocationBased => "scope-1-2-location",
    Scope1And2MarketBased => "scope-1-2-market",
    Scope123 => "scope-1-2-3",
);
kebab!(
    ReusePolicy,
    ActuallyConsumed => "actually-consumed",
    RecoveredNotConsumed => "recovered-not-consumed",
);
kebab!(
    ScopeItem,
    ItEnergy => "it-energy",
    Cooling => "cooling",
    Lighting => "lighting",
    UpsLosses => "ups-losses",
    SwitchgearLosses => "switchgear-losses",
    PduLosses => "pdu-losses",
    GeneratorTesting => "generator-testing",
    OfficeHvac => "office-hvac",
    Security => "security",
    CoolingTowerEvaporation => "cooling-tower-evaporation",
    Blowdown => "blowdown",
    Drift => "drift",
    Humidification => "humidification",
    EnergyWater => "energy-water",
    ProcessWater => "process-water",
    FireWater => "fire-water",
    HydroReservoirEvaporation => "hydro-reservoir-evaporation",
    Scope1OnSite => "scope-1-on-site",
    Scope2PurchasedEnergy => "scope-2-purchased-energy",
    Scope3Upstream => "scope-3-upstream",
    DistrictHeat => "district-heat",
    AdjacentBuilding => "adjacent-building",
    IndustrialProcess => "industrial-process",
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_includes_or_excludes_is_error() {
        let err = Boundary::new(BoundarySpec {
            it_meter: ItMeter::UpsOutput,
            includes: BTreeSet::new(),
            excludes: [ScopeItem::OfficeHvac].into_iter().collect(),
            water_scope: None,
            hydro: None,
            energy_water_mix: None,
            mix_id: None,
            carbon: None,
            reuse: None,
        })
        .unwrap_err();
        assert_eq!(err, MetricsError::EmptyBoundary);
    }

    #[test]
    fn overlap_is_error() {
        let err = Boundary::pue(
            ItMeter::UpsOutput,
            [ScopeItem::ItEnergy, ScopeItem::Cooling],
            [ScopeItem::Cooling],
        )
        .unwrap_err();
        assert!(matches!(err, MetricsError::BoundaryOverlap(_)));
    }

    #[test]
    fn pue_without_it_energy_is_error() {
        let err = Boundary::pue(
            ItMeter::UpsOutput,
            [ScopeItem::Cooling],
            [ScopeItem::OfficeHvac],
        )
        .unwrap_err();
        assert_eq!(err, MetricsError::PueWithoutItInBoundary);
    }

    #[test]
    fn source_wue_without_hydro_is_error() {
        let err = Boundary::wue(
            ItMeter::ItInput,
            WaterScope::Source,
            None,
            [ScopeItem::CoolingTowerEvaporation],
            [ScopeItem::FireWater],
        )
        .unwrap_err();
        assert!(matches!(
            err,
            MetricsError::MissingDeclaration { field: "hydro", .. }
        ));
    }
}
