//! Relationships: attachment-informed bonds between villagers.
//! Emergent: bonds form from interactions, not assigned.

use crate::engine::ecs::{Entity, World};

/// A relationship between two entities, stored as its own entity (like
/// Organization or HouseMeeting) so each person can hold many of them.
#[derive(Debug)]
pub struct Relationship {
    pub from: Entity,
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

/// Find the relationship entity between `from` and `other`, if any exists
/// (checked in both directions, since either side may have recorded the
/// first interaction).
pub fn find_relationship(world: &World, from: Entity, other: Entity) -> Option<Entity> {
    world.query::<Relationship>().into_iter().find(|&e| {
        world.get_component::<Relationship>(e)
            .map(|r| (r.from == from && r.other == other) || (r.from == other && r.other == from))
            .unwrap_or(false)
    })
}

/// Record an interaction - relationships evolve emergently. Finds the
/// relationship entity between `from` and the interaction's target (or
/// creates one), applies the interaction's effect on strength, and
/// re-infers the relationship kind from the new state.
pub fn record_interaction(world: &mut World, from: Entity, interaction: Interaction) -> Entity {
    let delta = match interaction.kind {
        InteractionKind::Conversation => 0.05,
        InteractionKind::SharedMeal => 0.08,
        InteractionKind::Conflict => -0.1,
        InteractionKind::Aid => 0.12,
        InteractionKind::Betrayal => -0.3,
        InteractionKind::Romance => 0.15,
    };
    let other = interaction.other;

    let rel_entity = find_relationship(world, from, other).unwrap_or_else(|| {
        let e = world.create_entity();
        world.add_component(e, Relationship {
            from, other, kind: RelKind::Stranger, strength: 0.0, interactions: 0,
        });
        e
    });

    if let Some(r) = world.get_component_mut::<Relationship>(rel_entity) {
        r.strength = (r.strength + delta).clamp(-1.0, 1.0);
        r.interactions += 1;
    }
    if let Some(r) = world.get_component::<Relationship>(rel_entity) {
        let kind = infer_kind(r);
        world.get_component_mut::<Relationship>(rel_entity).unwrap().kind = kind;
    }
    rel_entity
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_interaction_creates_a_relationship() {
        let mut world = World::new();
        let a = world.create_entity();
        let b = world.create_entity();

        let e = record_interaction(&mut world, a, Interaction { other: b, kind: InteractionKind::Conversation, intensity: 1.0 });

        let r = world.get_component::<Relationship>(e).expect("relationship should exist");
        assert_eq!(r.from, a);
        assert_eq!(r.other, b);
        assert_eq!(r.interactions, 1);
        assert!((r.strength - 0.05).abs() < 1e-6);
    }

    #[test]
    fn repeated_interactions_accumulate_on_the_same_relationship() {
        let mut world = World::new();
        let a = world.create_entity();
        let b = world.create_entity();

        let e1 = record_interaction(&mut world, a, Interaction { other: b, kind: InteractionKind::Conversation, intensity: 1.0 });
        let e2 = record_interaction(&mut world, a, Interaction { other: b, kind: InteractionKind::Conversation, intensity: 1.0 });

        assert_eq!(e1, e2, "a second interaction with the same person should update the existing relationship, not create a new one");
        let r = world.get_component::<Relationship>(e1).unwrap();
        assert_eq!(r.interactions, 2);
        assert!((r.strength - 0.10).abs() < 1e-5);
    }

    #[test]
    fn relationship_lookup_is_bidirectional() {
        let mut world = World::new();
        let a = world.create_entity();
        let b = world.create_entity();
        record_interaction(&mut world, a, Interaction { other: b, kind: InteractionKind::Aid, intensity: 1.0 });

        assert!(find_relationship(&world, a, b).is_some());
        assert!(find_relationship(&world, b, a).is_some(), "recording from A should also be visible when looking up from B's side");
    }

    #[test]
    fn strength_is_clamped_to_valid_range() {
        let mut world = World::new();
        let a = world.create_entity();
        let b = world.create_entity();
        let mut last = Entity::default();
        for _ in 0..50 {
            last = record_interaction(&mut world, a, Interaction { other: b, kind: InteractionKind::Romance, intensity: 1.0 });
        }
        let r = world.get_component::<Relationship>(last).unwrap();
        assert!(r.strength <= 1.0 && r.strength >= -1.0);
    }

    #[test]
    fn betrayals_can_turn_a_relationship_into_a_rivalry() {
        let mut world = World::new();
        let a = world.create_entity();
        let b = world.create_entity();
        let mut e = Entity::default();
        for _ in 0..5 {
            e = record_interaction(&mut world, a, Interaction { other: b, kind: InteractionKind::Betrayal, intensity: 1.0 });
        }
        let r = world.get_component::<Relationship>(e).unwrap();
        assert_eq!(r.kind, RelKind::Rival);
    }
}
