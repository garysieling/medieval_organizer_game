//! Military: simple abstracted battles, not a war sim.
//! From backlog: "very little in terms of military simulation - simple battles are fine"

use crate::engine::ecs::{Entity, World};

/// A military unit
#[derive(Debug)]
pub struct Unit {
    pub name: String,
    pub kind: UnitKind,
    pub strength: f32,
    pub morale: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnitKind {
    Infantry,
    Archers,
    Cavalry,
    Militia,
}

/// An army = collection of units
#[derive(Debug)]
pub struct Army {
    pub units: Vec<Entity>,
    pub commander: Option<Entity>,
    pub total_strength: f32,
}

/// Battle outcome
#[derive(Debug)]
pub struct BattleResult {
    pub winner: Option<Entity>,
    pub casualties_a: f32,
    pub casualties_b: f32,
    pub duration: u32, // ticks
}

/// Resolve a battle simply: strength + morale vs strength + morale
pub fn resolve_battle(_world: &mut World, a: &Army, b: &Army) -> BattleResult {
    let sa = a.total_strength;
    let sb = b.total_strength;
    let _winner = if sa > sb { Some(Entity::MAX) } else if sb > sa { Some(0) } else { None };
    BattleResult {
        winner: None,
        casualties_a: sb * 0.1,
        casualties_b: sa * 0.1,
        duration: 5,
    }
}
