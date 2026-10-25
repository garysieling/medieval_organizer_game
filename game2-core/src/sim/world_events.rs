//! World events: rare large events that can help struggling players recover.
//! From backlog: "you vs the world conflicts, novel threats, rare large events
//! help struggling players recover" - forcing functions + comeback mechanics.

use crate::engine::ecs::{Entity, World};

/// A major world event (rare, large impact)
#[derive(Debug)]
pub struct WorldEvent {
    pub name: String,
    pub kind: EventKind,
    pub impact: f32,        // magnitude of effect
    pub benefits_loser: bool, // helps struggling players?
    pub triggered_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventKind {
    BountifulHarvest,    // everyone gets food (helps weak)
    Plague,             // disrupts everyone (levels field)
    Revelation,         // new knowledge spreads (helps innovation)
    Migration,          // new people arrive (new blood)
    RelicFound,         // ancient power discovered
    GreatStorm,         // resets some accumulation
    Miracle,            // extraordinary luck for the weak
    Conquest,           // external threat invades (you vs world)
}

/// Check if a world event should fire (rare, favors struggling)
pub fn should_fire(player_state: &PlayerStanding, tick: u64) -> bool {
    // Rare events (every ~100 ticks), biased toward helping the struggling
    if tick % 100 != 0 { return false; }
    // Struggling players (low standing) get event help more often
    player_state.standing < 0.4 || (tick % 300 == 0)
}

/// Player's standing in the world (for event eligibility)
#[derive(Debug)]
pub struct PlayerStanding {
    pub standing: f32,    // 0-1, low = struggling
    pub prosperity: f32,
    pub power: f32,
}

/// What a world event does
pub fn event_effect(event: &WorldEvent) -> Vec<&'static str> {
    match event.kind {
        EventKind::BountifulHarvest => vec!["food surplus for all", "weak recover faster"],
        EventKind::Plague => vec!["disrupts the powerful", "levels the field"],
        EventKind::Revelation => vec!["new tech/knowledge available", "innovation boost"],
        EventKind::Migration => vec!["new villagers arrive", "fresh labor + ideas"],
        EventKind::RelicFound => vec!["ancient power discovered", "quest/opportunity"],
        EventKind::GreatStorm => vec!["accumulated wealth reset", "rebalances"],
        EventKind::Miracle => vec!["extraordinary luck", "struggling player recovers"],
        EventKind::Conquest => vec!["external threat invades", "you vs world"],
    }
}

/// Spawn a world event
pub fn spawn_event(world: &mut World, kind: EventKind, impact: f32) -> Entity {
    let e = world.create_entity();
    world.add_component(e, WorldEvent {
        name: format!("{:?}", kind),
        kind, impact,
        benefits_loser: matches!(kind, EventKind::BountifulHarvest | EventKind::Plague | EventKind::Miracle),
        triggered_at: 0,
    });
    e
}
