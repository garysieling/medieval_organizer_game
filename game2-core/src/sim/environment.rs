//! The land keeps score: slow environmental counters that turn the
//! village's appetite into grievances. Not scenery - organizing content:
//! each pressure crosses a threshold that seeds problems, complaints,
//! and discoverable issues through the Heard -> Issue machinery.
//!
//! Three pressures, all commons - the land's grievances are the ones
//! everybody shares, so they're the organizer's bridge issues:
//! - the shrinking wood: gathers take canopy; only undisturbed ground
//!   regrows
//! - the drowned pit: stone digs below the water table; yield drops,
//!   then stops
//! - the stinking reach: the town's leavings settle in downstream water

use std::collections::HashMap;

/// The land's memory: what the village has taken and dumped
#[derive(Debug, Default)]
pub struct LandScore {
    /// tree tile -> remaining canopy (1.0 = full)
    pub tree_cover: HashMap<(i32, i32), f32>,
    /// the worked pit fills as it's dug (0 = dry, 1.0 = drowned)
    pub pit_water: f32,
    /// river tile -> the stink carried to it (0 = clean, 1.0 = foul)
    pub effluent: HashMap<(i32, i32), f32>,
}

/// A wood tile yields while there's canopy on it; stumps give little
pub fn wood_yield(cover: f32) -> u32 {
    if cover < 0.3 { 1 } else { 2 }
}

/// The pit yields until the water wins
pub fn stone_yield(water: f32) -> u32 {
    if water >= 1.0 { 0 } else if water >= 0.7 { 1 } else { 2 }
}
