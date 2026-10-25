//! Tea history: trade good + psychometric effects.
//! From backlog: economic trade AND psychometric effects (caffeine/ritual).

use crate::engine::ecs::{Entity, World};

/// A consumable tea
#[derive(Debug)]
pub struct Tea {
    pub variety: TeaVariety,
    pub caffeine: f32,     // 0-1 stimulation
    pub ritual_value: f32, // social/ceremonial value
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TeaVariety {
    Green,
    Black,
    Herbal,
    Oolong,
    PuErh,
}

/// Effect of drinking tea on a character
#[derive(Debug)]
pub struct TeaEffect {
    pub alertness: f32,    // caffeine boost
    pub sociability: f32,  // ritual/tea ceremony bonding
    pub duration: u32,     // ticks
}

/// Drinking tea produces psychometric effects
pub fn drink_tea(tea: &Tea) -> TeaEffect {
    TeaEffect {
        alertness: tea.caffeine * 0.5,
        sociability: tea.ritual_value * 0.4,
        duration: 12, // effects last 12 ticks
    }
}

/// Tea as a trade good (affects prices via ritual demand)
#[derive(Debug)]
pub struct TeaTrade {
    pub supply: u32,
    pub demand: u32,
    pub cultural_premium: f32, // ritual value adds to price
}

/// Tea ceremony - social bonding event
#[derive(Debug)]
pub struct TeaCeremony {
    pub participants: Vec<Entity>,
    pub cohesion_boost: f32,
}

/// Spawn tea variety
pub fn spawn_tea(world: &mut World, variety: TeaVariety) -> Entity {
    let e = world.create_entity();
    world.add_component(e, Tea {
        variety,
        caffeine: match variety {
            TeaVariety::Green => 0.3,
            TeaVariety::Black => 0.6,
            TeaVariety::Herbal => 0.0,
            TeaVariety::Oolong => 0.4,
            TeaVariety::PuErh => 0.5,
        },
        ritual_value: match variety {
            TeaVariety::Green => 0.7,
            TeaVariety::Black => 0.4,
            TeaVariety::Herbal => 0.8,
            TeaVariety::Oolong => 0.6,
            TeaVariety::PuErh => 0.9,
        },
    });
    e
}
