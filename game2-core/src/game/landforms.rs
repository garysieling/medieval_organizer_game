//! Game2's small, data-driven landform layer.
//!
//! RimWorld's Geological Landforms keeps landforms independent from biome
//! selection and filters them against tile requirements. This module keeps
//! that boundary, using Game2 terrain fractions instead of RimWorld Defs.
//! The profiles below are original Game2 parameters; no upstream code or XML
//! is copied here. See docs/plans/geological-landforms-integration.md for
//! provenance and the planned attribution rules for any future imported data.

use crate::game::terrain::Terrain;
use crate::game::tilemap::{ROCK_LEVEL, WATER_LEVEL};

/// The broad shape to apply after a profile has been selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LandformShape {
    /// Gently lift the interior and let the edges fall toward the coast.
    CoastalPlain,
    /// Raise two sides and preserve an open corridor through the middle.
    OpenValley,
    /// Create a broad low basin while keeping a dry route to the map edge.
    LakeBasin,
    /// Add low-frequency relief while preserving a generous lowland share.
    RollingUplands,
}

/// Measurable terrain limits that keep a landform useful for settlements.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LandformRequirements {
    pub min_buildable_fraction: f32,
    pub min_dry_fraction: f32,
    pub max_rock_fraction: f32,
    pub min_water_fraction: f32,
    pub max_water_fraction: f32,
}

/// A landform is a separate terrain-generation choice, not a biome.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LandformProfile {
    pub id: &'static str,
    pub display_name: &'static str,
    pub shape: LandformShape,
    pub requirements: LandformRequirements,
}

/// Fractions of the height field that map to useful and obstructive terrain.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TerrainAvailability {
    pub buildable_fraction: f32,
    pub dry_fraction: f32,
    pub rock_fraction: f32,
    pub water_fraction: f32,
}

impl TerrainAvailability {
    pub fn measure(terrain: &Terrain) -> Self {
        let total = (terrain.width * terrain.height).max(1) as f32;
        let mut buildable = 0usize;
        let mut dry = 0usize;
        let mut rock = 0usize;
        let mut water = 0usize;
        for row in &terrain.heights {
            for &height in row {
                if height < WATER_LEVEL {
                    water += 1;
                } else {
                    dry += 1;
                    if height < ROCK_LEVEL { buildable += 1; }
                    else { rock += 1; }
                }
            }
        }
        Self {
            buildable_fraction: buildable as f32 / total,
            dry_fraction: dry as f32 / total,
            rock_fraction: rock as f32 / total,
            water_fraction: water as f32 / total,
        }
    }

    pub fn satisfies(self, requirements: LandformRequirements) -> bool {
        self.buildable_fraction >= requirements.min_buildable_fraction
            && self.dry_fraction >= requirements.min_dry_fraction
            && self.rock_fraction <= requirements.max_rock_fraction
            && self.water_fraction >= requirements.min_water_fraction
            && self.water_fraction <= requirements.max_water_fraction
    }
}

const OPEN_TERRAIN: LandformRequirements = LandformRequirements {
    min_buildable_fraction: 0.38,
    min_dry_fraction: 0.55,
    max_rock_fraction: 0.20,
    min_water_fraction: 0.02,
    max_water_fraction: 0.45,
};

/// Original Game2 starter profiles inspired by broad geological forms.
pub const LANDFORMS: &[LandformProfile] = &[
    LandformProfile {
        id: "coastal_plain",
        display_name: "Coastal Plain",
        shape: LandformShape::CoastalPlain,
        requirements: LandformRequirements { max_water_fraction: 0.48, ..OPEN_TERRAIN },
    },
    LandformProfile {
        id: "open_valley",
        display_name: "Open Valley",
        shape: LandformShape::OpenValley,
        requirements: LandformRequirements { min_buildable_fraction: 0.45, max_rock_fraction: 0.16, ..OPEN_TERRAIN },
    },
    LandformProfile {
        id: "lake_basin",
        display_name: "Lake Basin",
        shape: LandformShape::LakeBasin,
        requirements: LandformRequirements {
            min_buildable_fraction: 0.40,
            min_dry_fraction: 0.52,
            max_rock_fraction: 0.18,
            min_water_fraction: 0.10,
            max_water_fraction: 0.40,
        },
    },
    LandformProfile {
        id: "rolling_uplands",
        display_name: "Rolling Uplands",
        shape: LandformShape::RollingUplands,
        requirements: LandformRequirements { min_buildable_fraction: 0.42, max_rock_fraction: 0.24, ..OPEN_TERRAIN },
    },
];

/// Select an eligible profile using the seed, with an open valley fallback.
/// The eligibility check happens against the generated terrain so maps that
/// are mostly water or high rock cannot select a settlement-hostile profile.
pub fn select_landform<'a>(terrain: &Terrain, seed: u64) -> &'a LandformProfile {
    let availability = TerrainAvailability::measure(terrain);
    let eligible: Vec<&LandformProfile> = LANDFORMS
        .iter()
        .filter(|profile| availability.satisfies(profile.requirements))
        .collect();
    if eligible.is_empty() {
        return LANDFORMS.iter().find(|profile| profile.id == "open_valley").unwrap();
    }
    let index = mixed_seed(seed) as usize % eligible.len();
    eligible[index]
}

fn mixed_seed(seed: u64) -> u64 {
    let mut value = seed.wrapping_add(0x9E3779B97F4A7C15);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D049BB133111EB);
    value ^ (value >> 31)
}

