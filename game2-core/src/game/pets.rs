//! Pets: companion animals for villagers.
//! From backlog: RimWorld + Stardew Valley models - bonding, utility, mood effects.

use crate::engine::ecs::{Entity, World};

/// A pet animal
#[derive(Debug)]
pub struct Pet {
    pub name: String,
    pub species: Species,
    pub owner: Option<Entity>,
    pub bond_strength: f32,   // 0-1, grows with care
    pub mood_effect: f32,     // how much they improve owner's mood
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Species {
    Dog,
    Cat,
    Bird,
    Horse,
    Chicken,
    Goat,
}

/// Pet utility: what they help with
#[derive(Debug)]
pub enum PetRole {
    Companion,   // mood boost
    Guard,       // protects home
    Hunting,     // helps find food
    Herding,     // manages livestock
    Labor,       // works (horses)
}

/// Pet effects on owner
pub fn pet_mood_boost(pet: &Pet) -> f32 {
    pet.bond_strength * pet.mood_effect
}

/// Spawn a pet
pub fn spawn_pet(world: &mut World, name: &str, species: Species, role: PetRole) -> Entity {
    let e = world.create_entity();
    world.add_component(e, Pet {
        name: name.into(),
        species,
        owner: None,
        bond_strength: 0.0,
        mood_effect: match role {
            PetRole::Companion => 0.3,
            PetRole::Guard => 0.1,
            PetRole::Hunting => 0.15,
            PetRole::Herding => 0.1,
            PetRole::Labor => 0.0,
        },
    });
    world.add_component(e, role);
    e
}

/// Bond a pet to an owner
pub fn adopt(world: &mut World, pet: Entity, owner: Entity) {
    if let Some(p) = world.get_component_mut::<Pet>(pet) {
        p.owner = Some(owner);
    }
}

/// Bond gained per tick while the owner is able to care for the pet
const CARE_RATE: f32 = 0.005;
/// Bond lost per tick while the owner is starving or sick and can't
const NEGLECT_RATE: f32 = 0.002;
/// Bond a companion needs before it can comfort its owner
pub const COMFORT_BOND: f32 = 0.5;

/// Can this owner look after a pet right now? Not if they're starving or
/// sick in bed (owners without those needs tracked are assumed fine).
fn can_care(world: &World, owner: Entity) -> bool {
    let fed = world.get_component::<crate::game::villager::Hunger>(owner)
        .map(|h| h.value >= 30.0).unwrap_or(true);
    let well = world.get_component::<crate::sim::illness::Health>(owner)
        .map(|h| h.condition >= 0.8).unwrap_or(true);
    fed && well
}

use crate::game::villager::is_dead; // health 0, or marked Dead by the windowed game

/// Who takes in a pet when its owner dies: the living person the owner was
/// closest to (strongest positive relationship), if anyone
fn next_keeper(world: &World, dead_owner: Entity) -> Option<Entity> {
    use crate::game::relationships::Relationship;
    world.query::<Relationship>().into_iter()
        .filter_map(|r| world.get_component::<Relationship>(r))
        .filter_map(|r| {
            let other = if r.from == dead_owner { r.other } else if r.other == dead_owner { r.from } else { return None };
            (r.strength > 0.0 && !is_dead(world, other)).then_some((other, r.strength))
        })
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(e, _)| e)
}

/// System: bonds strengthen with care over time, and fray when the owner
/// is too hungry or sick to look after the pet. When the owner dies, the
/// person they were closest to takes the pet in and the bond starts over;
/// with no one to take it, the pet becomes a stray.
pub struct PetBondSystem;
impl crate::engine::ecs::System for PetBondSystem {
    fn run(&mut self, world: &mut World) {
        for e in world.query::<Pet>() {
            let Some(owner) = world.get_component::<Pet>(e).and_then(|p| p.owner) else { continue };
            if is_dead(world, owner) {
                let keeper = next_keeper(world, owner);
                if let Some(p) = world.get_component_mut::<Pet>(e) {
                    p.owner = keeper;
                    p.bond_strength = 0.0;
                }
                continue;
            }
            let cared_for = can_care(world, owner);
            if let Some(p) = world.get_component_mut::<Pet>(e) {
                p.bond_strength = if cared_for {
                    (p.bond_strength + CARE_RATE).min(1.0)
                } else {
                    (p.bond_strength - NEGLECT_RATE).max(0.0)
                };
            }
        }
    }
}

/// A pet close enough to its owner, and suited to it, to comfort them when
/// they're distressed: the owner's most comforting pet whose bond is at
/// least COMFORT_BOND and that offers a real mood boost
pub fn comforting_pet(world: &World, owner: Entity) -> Option<Entity> {
    world.query::<Pet>().into_iter()
        .filter(|e| world.get_component::<Pet>(*e)
            .map(|p| p.owner == Some(owner) && p.bond_strength >= COMFORT_BOND && pet_mood_boost(p) >= 0.1)
            .unwrap_or(false))
        .max_by(|a, b| {
            let boost = |e: &Entity| world.get_component::<Pet>(*e).map(pet_mood_boost).unwrap_or(0.0);
            boost(a).total_cmp(&boost(b))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::ecs::System;
    use crate::game::villager::Hunger;

    fn owner_with_dog(w: &mut World, hunger: f32, role: PetRole) -> (Entity, Entity) {
        let owner = w.create_entity();
        w.add_component(owner, Hunger { value: hunger });
        let dog = spawn_pet(w, "Rex", Species::Dog, role);
        adopt(w, dog, owner);
        (owner, dog)
    }

    fn bond(w: &World, pet: Entity) -> f32 {
        w.get_component::<Pet>(pet).unwrap().bond_strength
    }

    #[test]
    fn test_cared_for_pet_bonds() {
        let mut w = World::new();
        let (_, dog) = owner_with_dog(&mut w, 80.0, PetRole::Companion);
        for _ in 0..20 { PetBondSystem.run(&mut w); }
        assert!(bond(&w, dog) > 0.0);
    }

    #[test]
    fn test_starving_owner_neglects_pet() {
        let mut w = World::new();
        let (_, dog) = owner_with_dog(&mut w, 10.0, PetRole::Companion);
        w.get_component_mut::<Pet>(dog).unwrap().bond_strength = 0.6;
        for _ in 0..20 { PetBondSystem.run(&mut w); }
        assert!(bond(&w, dog) < 0.6, "a starving owner can't look after their pet");
    }

    #[test]
    fn test_unowned_pet_bond_unchanged() {
        let mut w = World::new();
        let stray = spawn_pet(&mut w, "Stray", Species::Cat, PetRole::Companion);
        for _ in 0..20 { PetBondSystem.run(&mut w); }
        assert_eq!(bond(&w, stray), 0.0);
    }

    #[test]
    fn test_only_a_well_bonded_companion_comforts() {
        let mut w = World::new();
        let (owner, dog) = owner_with_dog(&mut w, 80.0, PetRole::Companion);
        assert_eq!(comforting_pet(&w, owner), None, "a new pet isn't bonded enough yet");
        w.get_component_mut::<Pet>(dog).unwrap().bond_strength = 0.8;
        assert_eq!(comforting_pet(&w, owner), Some(dog));
        let (farmer, horse) = owner_with_dog(&mut w, 80.0, PetRole::Labor);
        w.get_component_mut::<Pet>(horse).unwrap().bond_strength = 0.8;
        assert_eq!(comforting_pet(&w, farmer), None, "a work animal offers no comfort");
    }

    #[test]
    fn test_pet_goes_to_closest_survivor_when_owner_dies() {
        use crate::game::relationships::{Relationship, RelKind};
        use crate::sim::illness::Health;
        let mut w = World::new();
        let (owner, dog) = owner_with_dog(&mut w, 80.0, PetRole::Companion);
        w.add_component(owner, Health { condition: 0.9, immunity: 0.5 });
        let rel = |w: &mut World, other: Entity, strength: f32| {
            let r = w.create_entity();
            w.add_component(r, Relationship { from: owner, other, kind: RelKind::Friend, strength, interactions: 5 });
        };
        let (friend, best_friend, rival) = (w.create_entity(), w.create_entity(), w.create_entity());
        rel(&mut w, friend, 0.4);
        rel(&mut w, best_friend, 0.8);
        rel(&mut w, rival, -0.6);
        for _ in 0..40 { PetBondSystem.run(&mut w); }
        assert!(bond(&w, dog) > 0.1);

        w.get_component_mut::<Health>(owner).unwrap().condition = 0.0;
        PetBondSystem.run(&mut w);
        let p = w.get_component::<Pet>(dog).unwrap();
        assert_eq!(p.owner, Some(best_friend), "the closest survivor takes the dog in");
        assert_eq!(p.bond_strength, 0.0, "and has to build the bond from scratch");
    }

    #[test]
    fn test_pet_with_no_one_left_becomes_a_stray() {
        use crate::sim::illness::Health;
        let mut w = World::new();
        let (owner, dog) = owner_with_dog(&mut w, 80.0, PetRole::Companion);
        w.add_component(owner, Health { condition: 0.0, immunity: 0.5 });
        PetBondSystem.run(&mut w);
        assert_eq!(w.get_component::<Pet>(dog).unwrap().owner, None);
        PetBondSystem.run(&mut w); // strays are left alone
    }

    #[test]
    fn test_pet_of_an_owner_marked_dead_is_rehomed() {
        // the windowed game removes Health on death and marks the villager Dead
        let mut w = World::new();
        let (owner, dog) = owner_with_dog(&mut w, 80.0, PetRole::Companion);
        w.add_component(owner, crate::game::villager::Dead);
        PetBondSystem.run(&mut w);
        assert_eq!(w.get_component::<Pet>(dog).unwrap().owner, None, "a corpse can't keep a pet");
    }
}
