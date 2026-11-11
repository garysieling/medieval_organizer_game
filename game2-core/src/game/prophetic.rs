//! The prophetic movement: a villager takes the calling and the
//! village's grievances gather to them - fast, fervent, and thin. The
//! crowd it raises can march and riot but can't hold a seat or a
//! campaign; it runs on a millenarian deadline, and killing the prophet
//! usually ends it - unless the death converts into the recruiting
//! event. (#878, after the #777 lit review's mechanics 1-3)

use crate::engine::ecs::{Entity, World};
use crate::game::desires::{Domination, InnerLife};
use crate::game::lifecycle::{Age, LifeStage};
use crate::game::villager::{self, Pos};

/// A villager swept up by the prophet: still themselves, still fed and
/// housed - but the calling thins what they can hold. They march, they
/// process, they riot; they don't keep seats, campaigns, institutions.
#[derive(Debug)]
pub struct Follower {
    /// The day the calling took them
    pub since: u64,
    /// A remnant who stayed after the deadline's disconfirmation: the
    /// hardened few who went deeper when the day came and went
    pub remnant: bool,
}

/// The movement itself: one prophet, a deadline the vision sets, and
/// the crowd that gathers till the day comes
#[derive(Debug)]
pub struct Movement {
    /// The villager who took the calling
    pub prophet: Entity,
    /// The day it began
    pub since: u64,
    /// The millenarian deadline: before it, maximal commitment; after
    /// it, the disconfirmation - mass demobilization or the remnant
    pub deadline: u64,
    /// Post-deadline remnant mode: the few who hardened; no growth left
    pub remnant: bool,
    /// The death already converted once - the grave took over from
    /// then on, and the SPOF branch doesn't fire twice
    pub martyred: bool,
}

/// The most a crowd this village can gather - crowds are broad, not deep
pub const CROWD_CAP: usize = 20;
/// Days from the calling to the day it all comes due
pub const DEADLINE_DAYS: u64 = 90;
/// The grievance density a calling needs to take root (0-1)
pub const EMERGE_POOL: f32 = 0.3;
/// The rare roll a dead prophet becomes the recruiting event (Hus,
/// Boukman, Biko): the grave does the growing the man no longer can
pub const MARTYRDOM_CHANCE: f32 = 0.12;
/// The hardened few a disconfirmation leaves when it takes the remnant
/// branch (Munster) instead of dissolving
pub const REMNANT_KEEP: usize = 3;
/// The surge a martyred prophet's death pulls in at once
pub const MARTYR_SURGE: usize = 8;

/// The vision's pull: the agency the prophet's own raising grew -
/// someone who never found their voice can't carry one to others
pub fn vision_power(world: &World, prophet: Entity) -> f32 {
    let agency = world.get_component::<Domination>(prophet)
        .map(|d| d.critical_agency).unwrap_or(0.0);
    0.3 + agency.clamp(0.0, 1.0) * 0.7
}

/// The pool the vision drinks: the share of live grievances across the
/// living villagers, 0-1 - an unwronged village can't be prophesied to
pub fn grievance_pool(world: &World, villagers: &[Entity], tick: u64) -> f32 {
    let (mut total, mut n) = (0.0f32, 0.0f32);
    for &v in villagers {
        if villager::is_dead(world, v) { continue; }
        n += 1.0;
        total += world.get_component::<InnerLife>(v)
            .map(|l| l.grievances.iter().filter(|(_, until)| *until > tick).count() as f32)
            .unwrap_or(0.0);
    }
    if n == 0.0 { 0.0 } else { (total / n / 2.0).min(1.0) }
}

/// How far the word carries: the settlement's own density - a wider
/// village hears more of a voice
pub fn rumor_reach(villagers: &[Entity]) -> f32 {
    (villagers.len() as f32 / 40.0).min(1.0)
}

/// Expected recruits a day under the formula: vision x grievance x reach
/// - vastly faster than organizing when the pool runs deep
pub fn daily_growth(world: &World, m: &Movement, villagers: &[Entity], tick: u64) -> f32 {
    vision_power(world, m.prophet) * grievance_pool(world, villagers, tick) * rumor_reach(villagers) * 4.0
}

/// The crowd's roll: living followers of the movement
pub fn followers(world: &World, villagers: &[Entity]) -> Vec<Entity> {
    villagers.iter().copied()
        .filter(|&v| world.get_component::<Follower>(v).is_some() && !villager::is_dead(world, v))
        .collect()
}

/// An adult fit for the calling: the highest critical agency in the
/// living village, ties broken by roster order. `excluded` keeps a
/// seat's own wanderer out - the calling takes a villager, not a player
pub fn pick_prophet(world: &World, villagers: &[Entity], excluded: &[Entity]) -> Option<Entity> {
    let mut best: Option<(Entity, f32)> = None;
    for &v in villagers {
        if excluded.contains(&v) { continue; }
        if villager::is_dead(world, v) { continue; }
        let adult = world.get_component::<Age>(v)
            .map(|a| a.stage == LifeStage::Adult || a.stage == LifeStage::Elder)
            .unwrap_or(false);
        if !adult { continue; }
        if world.get_component::<Follower>(v).is_some() { continue; }
        let agency = world.get_component::<Domination>(v)
            .map(|d| d.critical_agency).unwrap_or(0.0);
        if best.map(|(_, b)| agency > b).unwrap_or(true) { best = Some((v, agency)); }
    }
    best.map(|(v, _)| v)
}

/// The prophet's own tile, for the gather and the grave
pub fn prophet_tile(world: &World, m: &Movement) -> Option<(i32, i32)> {
    world.get_component::<Pos>(m.prophet)
        .map(|p| (p.x.round() as i32, p.y.round() as i32))
}
