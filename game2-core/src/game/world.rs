//! World: map grid, regions, settlements.

use crate::engine::ecs::Entity;
use crate::game::terrain::Terrain;

/// A named region of the map
#[derive(Debug)]
pub struct Region {
    pub name: String,
    pub climate_tag: String,
    pub resources: Vec<String>,
}

/// A settlement in the world
#[derive(Debug)]
pub struct Settlement {
    pub name: String,
    pub population: u32,
    pub region: String,
    pub ruler: Option<Entity>,
}

/// The game world state
#[derive(Debug)]
pub struct GameWorld {
    pub terrain: Option<Terrain>,
    pub regions: Vec<Region>,
    pub settlements: Vec<Settlement>,
}
