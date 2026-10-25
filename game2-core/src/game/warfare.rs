//! Warfare: simple military conflict (per Gary - diplomacy/econ > combat).
//! Villages fight for resources/territory; abstracted, not deep combat.

use crate::engine::ecs::{Entity, World};

/// A military force
#[derive(Debug)]
pub struct Force {
    pub size: usize,
    pub morale: f32,
    pub strength: f32,
    pub supply: f32,     // logistics matter more than combat
}

/// A conflict between two villages
#[derive(Debug)]
pub struct Conflict {
    pub attacker: Entity,
    pub defender: Entity,
    pub cause: ConflictCause,
    pub intensity: f32,
    pub tick: u64,
}

#[derive(Debug)]
pub enum ConflictCause {
    Resources,     // food, water, land
    Territory,     // borders, control
    Honor,         // slight, vengeance
    Conquest,      // expansion
    Defense,       // protect
}

/// Battle resolution (simple - not deep combat)
pub fn resolve_battle(attacker: &Force, defender: &Force) -> BattleResult {
    let attack_power = attacker.strength * attacker.morale * attacker.supply;
    let defense_power = defender.strength * defender.morale * defender.supply;
    if attack_power > defense_power * 1.3 {
        BattleResult::DecisiveVictory
    } else if attack_power > defense_power {
        BattleResult::Victory
    } else if attack_power > defense_power * 0.7 {
        BattleResult::PyrrhicVictory
    } else {
        BattleResult::Defeat
    }
}

#[derive(Debug)]
pub enum BattleResult {
    DecisiveVictory,
    Victory,
    PyrrhicVictory,
    Defeat,
}

/// Conflict consequences (simple - affects territory/resources)
pub fn conflict_effects(result: &BattleResult) -> Vec<&'static str> {
    match result {
        BattleResult::DecisiveVictory => vec!["territory gained", "morale boost", "power growth"],
        BattleResult::Victory => vec!["objective achieved", "small gains"],
        BattleResult::PyrrhicVictory => vec!["won but costly", "weakness exposed"],
        BattleResult::Defeat => vec!["retreat", "morale loss", "vulnerability"],
    }
}

/// Spawn a conflict
pub fn spawn_conflict(world: &mut World, attacker: Entity, defender: Entity, cause: ConflictCause) -> Entity {
    let e = world.create_entity();
    world.add_component(e, Conflict {
        attacker, defender, cause, intensity: 0.5, tick: 0,
    });
    e
}
