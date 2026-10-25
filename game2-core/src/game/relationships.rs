//! Relationships: attachment-informed bonds between villagers.
//! Emergent: bonds form from interactions, not assigned.

use crate::engine::ecs::{Entity, World};

/// A relationship between two entities
#[derive(Debug)]
pub struct Relationship {
    pub other: Entity,
    pub kind: RelKind,
    pub strength: f32,      // -1 to 1
    pub interactions: u32,  // how many times they've interacted
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RelKind {
    Stranger,
    Acquaintance,
    Friend,
    Rival,
    Family,
    Lover,
    Mentor,
    Apprentice,
}

/// An interaction event between two entities
#[derive(Debug)]
pub struct Interaction {
    pub other: Entity,
    pub kind: InteractionKind,
    pub intensity: f32,
}

#[derive(Debug)]
pub enum InteractionKind {
    Conversation,
    SharedMeal,
    Conflict,
    Aid,
    Betrayal,
    Romance,
}

/// System: interactions deepen relationships, kinds evolve
pub struct RelationshipSystem;
impl crate::engine::ecs::System for RelationshipSystem {
    fn run(&mut self, world: &mut World) {
        for e in world.query::<Relationship>() {
            if let Some(r) = world.get_component_mut::<Relationship>(e) {
                // Decay toward neutral if no recent interactions
                r.strength *= 0.99;
            }
        }
    }
}

/// Record an interaction - relationships evolve emergently
pub fn record_interaction(world: &mut World, from: Entity, interaction: Interaction) {
    let delta = match interaction.kind {
        InteractionKind::Conversation => 0.05,
        InteractionKind::SharedMeal => 0.08,
        InteractionKind::Conflict => -0.1,
        InteractionKind::Aid => 0.12,
        InteractionKind::Betrayal => -0.3,
        InteractionKind::Romance => 0.15,
    };
    let _ = delta; // would find/create Relationship and update
    let _ = from;
}

/// Infer relationship kind from history (emergent)
pub fn infer_kind(r: &Relationship) -> RelKind {
    if r.strength > 0.6 {
        match r.interactions {
            0..=5 => RelKind::Acquaintance,
            _ => RelKind::Friend,
        }
    } else if r.strength < -0.3 {
        RelKind::Rival
    } else {
        RelKind::Acquaintance
    }
}
