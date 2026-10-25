//! Social stratification: class emerges from wealth + influence + birth.
//! Village hierarchy is economic - who has wealth/power has status.

use crate::engine::ecs::{Entity, World};

/// Social class (emergent from wealth/influence)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Class {
    Poor,      // subsistence
    Laborer,   // works for others
    Artisan,   // skilled, some wealth
    Merchant,  // trade wealth
    Gentry,    // land + influence
    Noble,     // hereditary power
}

/// An entity's social position
#[derive(Debug)]
pub struct SocialPosition {
    pub class: Class,
    pub wealth: f32,
    pub influence: f32,
    pub birth: f32,      // hereditary advantage
    pub status: f32,     // overall standing
}

/// Compute class from wealth + influence + birth
pub fn infer_class(wealth: f32, influence: f32, birth: f32) -> Class {
    let score = wealth * 0.5 + influence * 0.4 + birth * 0.1;
    if score < 0.2 { Class::Poor }
    else if score < 0.4 { Class::Laborer }
    else if score < 0.6 { Class::Artisan }
    else if score < 0.8 { Class::Merchant }
    else if score < 0.9 { Class::Gentry }
    else { Class::Noble }
}

/// Update status from class + current state
pub fn update_status(pos: &mut SocialPosition) {
    let class_val = match pos.class {
        Class::Poor => 0.1, Class::Laborer => 0.3, Class::Artisan => 0.5,
        Class::Merchant => 0.7, Class::Gentry => 0.85, Class::Noble => 1.0,
    };
    pos.status = class_val * 0.6 + pos.influence * 0.4;
    pos.class = infer_class(pos.wealth, pos.influence, pos.birth);
}

/// Spawn a social position
pub fn spawn_position(world: &mut World, wealth: f32, influence: f32, birth: f32) -> Entity {
    let e = world.create_entity();
    let mut pos = SocialPosition { class: Class::Poor, wealth, influence, birth, status: 0.0 };
    update_status(&mut pos);
    world.add_component(e, pos);
    e
}
