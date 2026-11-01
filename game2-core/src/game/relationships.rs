//! Relationships: attachment-informed bonds between villagers.
//! Emergent: bonds form from interactions, not assigned.

use crate::engine::ecs::{Entity, World};
use std::collections::HashMap;

/// Pair index: unordered entity pair -> the earliest Relationship
/// entity between them. find_relationship consults it only while the
/// index is provably complete: `bonds_indexed` counts every bond folded
/// in, and once it trails the live component count a bare add_component
/// has slipped a bond past the index - lookups then fall back to a scan
/// and the next add_relationship rebuilds in one pass.
#[derive(Default)]
struct RelIndex {
    by_pair: HashMap<(Entity, Entity), Entity>,
    /// Bonds folded into by_pair so far (== live count while complete)
    bonds_indexed: usize,
}

fn pair_key(a: Entity, b: Entity) -> (Entity, Entity) {
    (a.min(b), a.max(b))
}

/// One pass over every live bond -> earliest entity per pair, marked
/// complete. Runs when a bare add_component put bonds the index never
/// saw into the world - rare outside hand-built test setups.
fn rebuild_index(world: &mut World) {
    let mut by_pair: HashMap<(Entity, Entity), Entity> = HashMap::new();
    let mut n = 0usize;
    world.iter::<Relationship>(|e, r| {
        n += 1;
        by_pair.entry(pair_key(r.from, r.other))
            .and_modify(|cur| { if e < *cur { *cur = e; } })
            .or_insert(e);
    });
    let idx = world.resource_or_default::<RelIndex>();
    idx.by_pair = by_pair;
    idx.bonds_indexed = n;
}

/// The lowest-id live Relationship matching `pred` - the answer a sorted
/// query() + first-match always gave, without the alloc or the sort.
fn lowest_matching(world: &World, matches: impl Fn(&Relationship) -> bool) -> Option<Entity> {
    let mut best: Option<Entity> = None;
    world.iter::<Relationship>(|e, r| {
        if matches(r) && best.map(|b| e < b).unwrap_or(true) { best = Some(e); }
    });
    best
}

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
            if let Some(r) = world.get_component::<Relationship>(e) {
                let kind = infer_kind(r);
                world.get_component_mut::<Relationship>(e).unwrap().kind = kind;
            }
        }
    }
}

/// Create the entity a Relationship lives on and register it in the pair
/// index. Bonds that come in through here resolve in O(1) from then on -
/// prefer it over bare create_entity + add_component.
pub fn add_relationship(world: &mut World, r: Relationship) -> Entity {
    let e = world.create_entity();
    let key = pair_key(r.from, r.other);
    world.add_component(e, r);
    let live = world.component_count::<Relationship>();
    let indexed = world.resource::<RelIndex>().map(|i| i.bonds_indexed).unwrap_or(0);
    if indexed + 1 == live {
        // the index was complete before this bond - fold it in. An
        // existing entry already holds the pair's earliest bond, and e
        // is the newest id, so or_insert keeps the ordering a sorted
        // scan always favoured
        let idx = world.resource_or_default::<RelIndex>();
        idx.by_pair.entry(key).or_insert(e);
        idx.bonds_indexed = live;
    } else {
        // a bare add_component slipped bonds past the index - rebuild
        // once rather than scan per insert
        rebuild_index(world);
    }
    e
}

/// The bond entities `v` is a side of, in creation order - ties settle
/// the way a sorted query() always did, so picks stay reproducible.
/// For a walk over many villagers' bonds at once, build a `bond_map`
/// instead of calling this per villager.
pub fn bonds_of(world: &World, v: Entity) -> Vec<Entity> {
    let mut out = world.query::<Relationship>();
    out.retain(|&e| world.get_component::<Relationship>(e)
        .map(|r| r.from == v || r.other == v).unwrap_or(false));
    out
}

/// One pass over every bond -> who holds bonds to whom (each list in
/// creation order). For code walking many villagers' bonds in one go,
/// building this once beats a full scan per villager; strength and kind
/// still come off the components.
pub fn bond_map(world: &World) -> HashMap<Entity, Vec<Entity>> {
    let mut m: HashMap<Entity, Vec<Entity>> = HashMap::new();
    for e in world.query::<Relationship>() {
        let Some(r) = world.get_component::<Relationship>(e) else { continue };
        m.entry(r.from).or_default().push(e);
        if r.other != r.from { m.entry(r.other).or_default().push(e); }
    }
    m
}

/// Find the relationship entity between `from` and `other`, if any exists
/// (checked in both directions, since either side may have recorded the
/// first interaction).
pub fn find_relationship(world: &World, from: Entity, other: Entity) -> Option<Entity> {
    let matches = |r: &Relationship|
        (r.from == from && r.other == other) || (r.from == other && r.other == from);
    if let Some(idx) = world.resource::<RelIndex>() {
        // a complete index answers in O(1); a stale one defers to the
        // scan, since an unindexed bond could predate any indexed entry
        if idx.bonds_indexed == world.component_count::<Relationship>() {
            return match idx.by_pair.get(&pair_key(from, other)) {
                // the entry only counts if the entity still holds this
                // bond - a destroyed relationship leaves a stale key
                Some(&e) if world.get_component::<Relationship>(e)
                    .map(&matches).unwrap_or(false) => Some(e),
                Some(_) => lowest_matching(world, &matches),
                None => None,
            };
        }
    }
    lowest_matching(world, matches)
}

/// Record an interaction - relationships evolve emergently. Finds the
/// relationship entity between `from` and the interaction's target (or
/// creates one), applies the interaction's effect on strength, and
/// re-infers the relationship kind from the new state.
pub fn record_interaction(world: &mut World, from: Entity, interaction: Interaction) -> Entity {
    let base_delta = match interaction.kind {
        InteractionKind::Conversation => 0.05,
        InteractionKind::SharedMeal => 0.08,
        InteractionKind::Conflict => -0.1,
        InteractionKind::Aid => 0.12,
        InteractionKind::Betrayal => -0.3,
        InteractionKind::Romance => 0.15,
    };
    let delta = base_delta * interaction.intensity.clamp(0.0, 1.0);
    let other = interaction.other;

    let rel_entity = find_relationship(world, from, other).unwrap_or_else(|| {
        add_relationship(world, Relationship {
            from, other, kind: RelKind::Stranger, strength: 0.0, interactions: 0,
        })
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

/// Infer relationship kind from history (emergent). Explicitly assigned
/// roles (Family, Lover, Mentor, Apprentice) are fixed and are never
/// overwritten by the strength/interaction-based inference below.
pub fn infer_kind(r: &Relationship) -> RelKind {
    if matches!(r.kind, RelKind::Family | RelKind::Lover | RelKind::Mentor | RelKind::Apprentice) {
        return r.kind;
    }
    if r.interactions == 0 {
        // Untouched pairs stay Stranger; neutral strength alone shouldn't age into Acquaintance.
        return RelKind::Stranger;
    }
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
    use crate::engine::ecs::System;

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

    #[test]
    fn fixed_kinds_survive_interactions() {
        let mut world = World::new();
        let a = world.create_entity();
        let b = world.create_entity();
        let e = record_interaction(&mut world, a, Interaction { other: b, kind: InteractionKind::Conversation, intensity: 1.0 });
        world.get_component_mut::<Relationship>(e).unwrap().kind = RelKind::Family;

        record_interaction(&mut world, a, Interaction { other: b, kind: InteractionKind::Conversation, intensity: 1.0 });

        let r = world.get_component::<Relationship>(e).unwrap();
        assert_eq!(r.kind, RelKind::Family, "a Family bond should not be reclassified as Acquaintance by an ordinary interaction");
    }

    #[test]
    fn low_intensity_interactions_affect_bonds_less() {
        let mut world = World::new();
        let a = world.create_entity();
        let b = world.create_entity();
        let weak = record_interaction(&mut world, a, Interaction { other: b, kind: InteractionKind::Conversation, intensity: 0.1 });
        let r = world.get_component::<Relationship>(weak).unwrap();
        assert!((r.strength - 0.005).abs() < 1e-6, "a low-intensity interaction should change strength proportionally less than a full-intensity one");
    }

    #[test]
    fn rival_label_clears_once_strength_decays_back_to_neutral() {
        let mut world = World::new();
        let a = world.create_entity();
        let b = world.create_entity();
        let mut e = Entity::default();
        for _ in 0..5 {
            e = record_interaction(&mut world, a, Interaction { other: b, kind: InteractionKind::Betrayal, intensity: 1.0 });
        }
        assert_eq!(world.get_component::<Relationship>(e).unwrap().kind, RelKind::Rival);

        let mut system = RelationshipSystem;
        for _ in 0..200 {
            system.run(&mut world);
        }

        let r = world.get_component::<Relationship>(e).unwrap();
        assert!(r.strength > -0.3, "strength should have decayed back above the rivalry threshold");
        assert_ne!(r.kind, RelKind::Rival, "kind should be re-inferred once strength decays out of rivalry range");
    }

    #[test]
    fn a_handmade_bond_keeps_its_place_when_a_bond_is_indexed_later() {
        let mut world = World::new();
        let a = world.create_entity();
        let b = world.create_entity();
        // a bond added bare - the index never sees it made
        let early = world.create_entity();
        world.add_component(early, Relationship {
            from: a, other: b, kind: RelKind::Family, strength: 0.8, interactions: 0,
        });
        // then an indexed bond lands on the same pair - a duplicate, the
        // way a second Family or a stray record can be
        let later = add_relationship(&mut world, Relationship {
            from: b, other: a, kind: RelKind::Acquaintance, strength: 0.1, interactions: 1,
        });
        assert_eq!(find_relationship(&world, a, b), Some(early),
            "the earliest record still wins the pair, indexed or not");
        assert_eq!(find_relationship(&world, b, a), Some(early));
        assert_ne!(later, early, "the new bond still exists, it just doesn't own the pair");
    }

    #[test]
    fn untouched_stranger_stays_a_stranger_through_decay_ticks() {
        let mut world = World::new();
        let a = world.create_entity();
        let b = world.create_entity();
        let e = world.create_entity();
        world.add_component(e, Relationship {
            from: a, other: b, kind: RelKind::Stranger, strength: 0.0, interactions: 0,
        });

        let mut system = RelationshipSystem;
        for _ in 0..5 {
            system.run(&mut world);
        }

        let r = world.get_component::<Relationship>(e).unwrap();
        assert_eq!(r.interactions, 0);
        assert_eq!(r.kind, RelKind::Stranger, "a pair with zero interactions should not age into Acquaintance on its own");
    }
}
