//! Water on the map: a slow inundation over the real TileMap.
//! The fire's twin in shape - the spread's cellular over the terrain's
//! own heights, low ground drinking first while the hills stay dry -
//! but the wetting's patient: a spell's flood or a season's rain raises
//! the level, the cells fill where the land lies low, the water goes
//! back down when the weather does. What's drowned stays drowned: the
//! crops under the water are gone when it goes.

use crate::engine::det::HashMap;
use super::tilemap::{noise, Tile, TileMap, NEIGHBOURS};

/// The level a spell's flood reaches: low ground drinks first
pub const FLOOD_LEVEL: f32 = 0.30;
/// Per-tick chance a wetting tile soaks a lower neighbour
pub const SOAK_CHANCE: f32 = 0.15;
/// Days a dried tile stays waterlogged - the ground's slow to shed the wet
pub const DRY_DAYS: u64 = 10;
/// The level rain can raise the flood to on a wet enough season
pub const RAIN_LEVEL: f32 = 0.25;

/// The flood: the level the water's reached, and the cells it's taken.
/// Cells hold their depth band (how far under the water's face they lie);
/// a tile's drowned is a tile's wetness, and the recede drains it back.
#[derive(Debug, Default)]
pub struct Flood {
    pub level: f32,
    pub cells: HashMap<(i32, i32), u8>,
}

impl Flood {
    pub fn new() -> Self { Self::default() }

    /// Whether the water's up anywhere
    pub fn active(&self) -> bool { !self.cells.is_empty() }

    /// Whether a tile lies under the water
    pub fn flooded(&self, x: i32, y: i32) -> bool { self.cells.contains_key(&(x, y)) }

    /// Raise the water: a spell's surge or the rain's long patience.
    /// The level only climbs - the weather takes it back down.
    pub fn raise(&mut self, to: f32) {
        self.level = self.level.max(to);
    }

    /// Let the water down: a tile band drains at a time, the shallowest
    /// first - the ground's slow to shed the wet
    pub fn recede(&mut self, by: f32) {
        self.level = (self.level - by).max(0.0);
        // the shallowest cells dry first - the deep ones stay
        self.cells.retain(|_, &mut band| band > 1);
    }

    /// The wetting: land tiles lower than the level, beside water or a
    /// cell already drowned, soak in - the wetting front moves as the
    /// land lies low
    pub fn wet(&mut self, map: &TileMap, heights: &[Vec<f32>], tick: u64) {
        let mut new_cells = self.cells.clone();
        for (&(x, y), _) in self.cells.iter() {
            for (dx, dy) in NEIGHBOURS {
                let (nx, ny) = (x + dx, y + dy);
                if new_cells.contains_key(&(nx, ny)) { continue; }
                let Some(t) = map.get(nx, ny) else { continue };
                if t == Tile::Water { continue; }
                let Some(h) = heights.get(ny as usize).and_then(|r| r.get(nx as usize)) else { continue };
                if *h >= self.level { continue; }
                if noise(nx, ny, tick as u32) < SOAK_CHANCE {
                    let band = if *h < self.level - 0.10 { 2 } else { 1 };
                    new_cells.insert((nx, ny), band);
                }
            }
        }
        // the water's edge: beside the river's own water, the low ground
        // drinks first - find tiles next to water under the level
        for y in 0..map.height as i32 {
            for x in 0..map.width as i32 {
                if new_cells.contains_key(&(x, y)) { continue; }
                let Some(t) = map.get(x, y) else { continue };
                if t == Tile::Water { continue; }
                let Some(h) = heights.get(y as usize).and_then(|r| r.get(x as usize)) else { continue };
                if *h >= self.level { continue; }
                let near_water = NEIGHBOURS.iter().any(|(dx, dy)|
                    map.get(x + dx, y + dy).map(|t| t == Tile::Water).unwrap_or(false));
                if near_water && noise(x, y, tick as u32) < SOAK_CHANCE {
                    let band = if *h < self.level - 0.10 { 2 } else { 1 };
                    new_cells.insert((x, y), band);
                }
            }
        }
        self.cells = new_cells;
    }
}
