//! Gossip/rumor propagation through relationships.
//! Information spreads along bonds, may distort, reaches interested parties.

use crate::engine::ecs::{Entity, World};
use crate::game::relationships::Relationship;

/// A piece of gossip
#[derive(Debug)]
pub struct Rumor {
    pub subject: String,
    pub content: String,
    pub truth_value: f32,   // 1.0 = true, distorts over hops
    pub spread: u32,        // how many heard it
    pub hops: u32,
}

/// Spread a rumor from one villager to their contacts
pub fn spread_rumor(world: &mut World, rumor_entity: Entity) -> Vec<Entity> {
    let mut informed = vec![];
    // Find who the subject is connected to via relationships
    for e in world.query::<Relationship>() {
        if let Some(_r) = world.get_component::<Relationship>(e) {
            informed.push(e); // would tell their connections
        }
    }
    if let Some(r) = world.get_component_mut::<Rumor>(rumor_entity) {
        r.spread += informed.len() as u32;
        r.hops += 1;
        r.truth_value *= 0.9; // distort slightly each hop
    }
    informed
}

/// Spawn a rumor
pub fn spawn_rumor(world: &mut World, subject: &str, content: &str) -> Entity {
    let e = world.create_entity();
    world.add_component(e, Rumor {
        subject: subject.into(), content: content.into(),
        truth_value: 1.0, spread: 0, hops: 0,
    });
    e
}

/// System: rumors spread each tick
pub struct GossipSystem;
impl crate::engine::ecs::System for GossipSystem {
    fn run(&mut self, world: &mut World) {
        for r in world.query::<Rumor>() {
            spread_rumor(world, r);
        }
    }
}
