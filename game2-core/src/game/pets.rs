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

#[derive(Debug, Clone, Copy, PartialEq)]
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
    e
}

/// Bond a pet to an owner
pub fn adopt(world: &mut World, pet: Entity, owner: Entity) {
    if let Some(p) = world.get_component_mut::<Pet>(pet) {
        p.owner = Some(owner);
    }
}

/// System: bonds strengthen with care over time
pub struct PetBondSystem;
impl crate::engine::ecs::System for PetBondSystem {
    fn run(&mut self, world: &mut World) {
        for e in world.query::<Pet>() {
            if let Some(p) = world.get_component_mut::<Pet>(e) {
                if p.owner.is_some() {
                    p.bond_strength = (p.bond_strength + 0.005).min(1.0);
                }
            }
        }
    }
}
