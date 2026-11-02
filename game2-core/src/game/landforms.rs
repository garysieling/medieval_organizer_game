//! Game2's small, data-driven landform layer.
//!
//! RimWorld's Geological Landforms keeps landforms independent from biome
//! selection and filters them against tile requirements. This module keeps
//! that boundary, using Game2 terrain fractions instead of RimWorld Defs.
//! Profile data is held separately from the generation code. The curated
//! source-derived subset and its attribution are documented in
//! `data/landforms/README.md`.

use serde::Deserialize;
use std::sync::OnceLock;

use crate::game::terrain::Terrain;
use crate::game::tilemap::{smooth_noise, ROCK_LEVEL, WATER_LEVEL};

/// The broad shape to apply after a profile has been selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
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
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LandformRequirements {
    pub min_buildable_fraction: f32,
    pub min_dry_fraction: f32,
    pub max_rock_fraction: f32,
    pub min_water_fraction: f32,
    pub max_water_fraction: f32,
}

/// A landform is a separate terrain-generation choice, not a biome.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LandformProfile {
    pub id: String,
    pub display_name: String,
    pub shape: LandformShape,
    pub requirements: LandformRequirements,
    pub selection_weight: f32,
    pub source_profile: Option<SourceProfile>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceProfile {
    pub id: String,
    pub source_file: String,
    pub topology: String,
    pub commonness: f32,
    pub hilliness_requirement: Option<[f32; 2]>,
    pub river_requirement: Option<[f32; 2]>,
    pub allows_settlements: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LandformCatalog {
    #[serde(rename = "source")]
    _source: SourceAttribution,
    profiles: Vec<LandformProfile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceAttribution {
    #[serde(rename = "title")]
    _title: String,
    #[serde(rename = "author")]
    _author: String,
    #[serde(rename = "url")]
    _url: String,
    #[serde(rename = "version")]
    _version: String,
    #[serde(rename = "commit")]
    _commit: String,
    #[serde(rename = "license")]
    _license: String,
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

/// Read the bundled profile data once. Keeping it in JSON makes landform
/// tuning independent from terrain-generation code and preserves provenance.
pub fn profiles() -> &'static [LandformProfile] {
    static PROFILES: OnceLock<Vec<LandformProfile>> = OnceLock::new();
    PROFILES.get_or_init(|| {
        let catalog: LandformCatalog = serde_json::from_str(include_str!("../../../data/landforms/geological-landforms.json"))
            .expect("bundled geological landform data must be valid");
        catalog.profiles
    })
}

/// Select an eligible profile using the seed. `None` means the terrain does
/// not meet any profile's settlement requirements and generation must retry
/// or use its separate safe fallback instead of mislabeling hostile terrain.
pub fn select_landform(terrain: &Terrain, seed: u64) -> Option<&'static LandformProfile> {
    let availability = TerrainAvailability::measure(terrain);
    let eligible: Vec<&LandformProfile> = profiles()
        .iter()
        .filter(|profile| availability.satisfies(profile.requirements))
        .collect();
    if eligible.is_empty() { return None; }
    let total_weight: f64 = eligible.iter().map(|profile| profile.selection_weight as f64).sum();
    if total_weight <= 0.0 { return None; }
    let mut choice = (mixed_seed(seed) as f64 / u64::MAX as f64) * total_weight;
    for profile in &eligible {
        if choice < profile.selection_weight as f64 { return Some(profile); }
        choice -= profile.selection_weight as f64;
    }
    eligible.last().copied()
}

/// Apply a seeded profile only when its resulting terrain still meets the
/// profile's usable-land requirements. Failure restores the source heights.
pub fn apply_landform(terrain: &mut Terrain, profile: &LandformProfile, seed: u64) -> bool {
    let original = terrain.heights.clone();
    let (width, height) = (terrain.width, terrain.height);
    if width < 2 || height < 2 { return false; }
    let rotation = (mixed_seed(seed) % 4) as u8;
    let salt = mixed_seed(seed ^ 0x4C414E44464F524D) as u32;
    for y in 0..height {
        for x in 0..width {
            let mut u = x as f32 / (width - 1) as f32;
            let mut v = y as f32 / (height - 1) as f32;
            match rotation {
                1 => (u, v) = (1.0 - v, u),
                2 => (u, v) = (1.0 - u, 1.0 - v),
                3 => (u, v) = (v, 1.0 - u),
                _ => {}
            }
            let height_value = &mut terrain.heights[y][x];
            match profile.shape {
                LandformShape::CoastalPlain => {
                    // A broad, one-sided rise keeps a clear coast and lowers
                    // the sharp relief of the source country terrain.
                    let coastal = 0.82 - 0.72 * v;
                    *height_value = (*height_value * 0.45 + coastal * 0.55).clamp(0.0, 1.0);
                }
                LandformShape::OpenValley => {
                    let ridge_a = gaussian(u, 0.22, 0.12);
                    let ridge_b = gaussian(u, 0.78, 0.12);
                    let valley = gaussian(u, 0.50, 0.20);
                    *height_value = (*height_value + 0.14 * (ridge_a + ridge_b) - 0.08 * valley).clamp(0.0, 1.0);
                }
                LandformShape::LakeBasin => {
                    let lake = ((u - 0.50) / 0.17).powi(2) + ((v - 0.48) / 0.14).powi(2);
                    let dry_approach = (u - 0.72).abs() < 0.035 && v > 0.44;
                    if !dry_approach {
                        *height_value = (*height_value - 0.75 * (-2.0 * lake).exp()).clamp(0.0, 1.0);
                    } else {
                        *height_value = (*height_value).max(WATER_LEVEL + 0.05);
                    }
                }
                LandformShape::RollingUplands => {
                    let scale = (width.min(height) as i32 / 8).max(4);
                    let relief = smooth_noise(x as i32, y as i32, scale, salt) - 0.5;
                    *height_value = (*height_value + relief * 0.24).clamp(0.0, 1.0);
                }
            }
        }
    }
    if TerrainAvailability::measure(terrain).satisfies(profile.requirements) { true }
    else { terrain.heights = original; false }
}

/// Choose and apply a profile, trying remaining eligible profiles in a
/// seeded order when shaping would leave too little usable land.
pub fn shape_terrain(terrain: &mut Terrain, seed: u64) -> Option<&'static LandformProfile> {
    let availability = TerrainAvailability::measure(terrain);
    let eligible: Vec<&LandformProfile> = profiles().iter()
        .filter(|profile| availability.satisfies(profile.requirements))
        .collect();
    if eligible.is_empty() { return None; }
    let first = select_landform(terrain, seed)?;
    let first_index = eligible.iter().position(|profile| profile.id == first.id)?;
    let original = terrain.heights.clone();
    for offset in 0..eligible.len() {
        terrain.heights.clone_from(&original);
        let profile = eligible[(first_index + offset) % eligible.len()];
        if apply_landform(terrain, profile, seed) { return Some(profile); }
    }
    terrain.heights = original;
    None
}

fn gaussian(value: f32, center: f32, width: f32) -> f32 {
    (-((value - center) / width).powi(2)).exp()
}

fn mixed_seed(seed: u64) -> u64 {
    let mut value = seed.wrapping_add(0x9E3779B97F4A7C15);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D049BB133111EB);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn terrain_with(shares: (usize, usize, usize), size: usize) -> Terrain {
        let (water, rock, buildable) = shares;
        let mut heights = vec![vec![0.0; size]; size];
        for (index, height) in heights.iter_mut().flatten().enumerate() {
            *height = if index < water { WATER_LEVEL - 0.01 }
                else if index < water + rock { ROCK_LEVEL + 0.01 }
                else if index < water + rock + buildable { (WATER_LEVEL + ROCK_LEVEL) / 2.0 }
                else { WATER_LEVEL - 0.01 };
        }
        Terrain { width: size, height: size, heights, seed: 1 }
    }

    #[test]
    fn availability_measures_buildable_water_and_rock_fractions() {
        let terrain = terrain_with((10, 10, 80), 10);
        let availability = TerrainAvailability::measure(&terrain);
        assert!((availability.buildable_fraction - 0.8).abs() < f32::EPSILON);
        assert!((availability.water_fraction - 0.1).abs() < f32::EPSILON);
        assert!((availability.rock_fraction - 0.1).abs() < f32::EPSILON);
        assert!((availability.dry_fraction - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn water_and_rock_dominated_maps_have_no_eligible_profile() {
        let all_water = terrain_with((100, 0, 0), 10);
        let all_rock = terrain_with((0, 100, 0), 10);
        assert!(select_landform(&all_water, 17).is_none());
        assert!(select_landform(&all_rock, 17).is_none());
    }

    #[test]
    fn coastal_plain_accepts_its_declared_water_limit() {
        let availability = TerrainAvailability {
            buildable_fraction: 0.54,
            dry_fraction: 0.54,
            rock_fraction: 0.0,
            water_fraction: 0.46,
        };
        assert!(availability.satisfies(profiles()[0].requirements));
    }

    #[test]
    fn selection_is_repeatable_for_the_same_seed() {
        let terrain = terrain_with((10, 5, 85), 10);
        let first = select_landform(&terrain, 231);
        let second = select_landform(&terrain, 231);
        assert_eq!(first.map(|profile| profile.id.as_str()), second.map(|profile| profile.id.as_str()));
    }

    #[test]
    fn shaping_keeps_the_selected_profiles_usable() {
        let mut shaped_count = 0;
        let mut profile_ids = std::collections::HashSet::new();
        for seed in 0..24 {
            let mut terrain = Terrain::generate_country(49, 0.7, seed);
            if let Some(profile) = shape_terrain(&mut terrain, seed) {
                shaped_count += 1;
                profile_ids.insert(profile.id.as_str());
                assert!(TerrainAvailability::measure(&terrain).satisfies(profile.requirements));
            }
        }
        assert!(shaped_count > 0, "the country generator should admit at least one landform");
        assert!(profile_ids.len() > 1, "seeded generation should use more than one landform profile");
    }
}

