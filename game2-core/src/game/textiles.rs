//! Textiles/fabric: trade good + sensory/texture sensitivity.
//! From backlog: fabric affects trade AND texture sensitivity (autism).

use crate::engine::ecs::{Entity, World};

/// A fabric/textile good
#[derive(Debug)]
pub struct Textile {
    pub name: String,
    pub fiber: FiberType,
    pub quality: f32,      // 0-1 craftsmanship
    pub texture: f32,      // 0-1 smoothness (sensory)
    pub trade_value: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FiberType {
    Wool,
    Linen,
    Cotton,
    Silk,
    Hemp,
}

/// Texture sensitivity effect on a character
#[derive(Debug)]
pub struct TextureComfort {
    pub comfort: f32,      // -1 to 1 (rough = negative, smooth = positive)
    pub sensitivity: f32,  // how strongly they feel it
}

/// Some characters are texture-sensitive (autism spectrum)
#[derive(Debug)]
pub struct TextureSensitive {
    pub sensitivity: f32,  // 0-1, higher = more sensitive
}

/// Check texture comfort for a sensitive character
pub fn comfort_level(textile: &Textile, sensitivity: f32) -> f32 {
    let base = textile.texture - 0.5; // -0.5 to 0.5
    base * (0.5 + sensitivity) // sensitive characters feel it more
}

/// Textile production chain
#[derive(Debug)]
pub struct Weaver {
    pub skill: f32,
    pub specialty: FiberType,
}

/// Spawn a textile
pub fn spawn_textile(world: &mut World, name: &str, fiber: FiberType, quality: f32) -> Entity {
    let e = world.create_entity();
    let texture = match fiber {
        FiberType::Wool => 0.4,
        FiberType::Linen => 0.6,
        FiberType::Cotton => 0.7,
        FiberType::Silk => 0.95,
        FiberType::Hemp => 0.3,
    };
    world.add_component(e, Textile {
        name: name.into(),
        fiber,
        quality,
        texture,
        trade_value: quality * texture * 20.0,
    });
    e
}
