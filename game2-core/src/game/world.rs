//! World: map grid, regions, settlements.

use crate::engine::ecs::{Entity, World};
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

/// Spawn a world with regions and settlements
pub fn spawn_world(world: &mut World, _name: &str) -> Entity {
    let e = world.create_entity();
    world.add_component(e, GameWorld {
        terrain: Some(Terrain::generate(33, 0.7, 42)),
        regions: vec![
            Region { name: "Northvale".into(), climate_tag: "cold".into(), resources: vec!["timber".into(), "iron".into()] },
            Region { name: "Southfield".into(), climate_tag: "temperate".into(), resources: vec!["grain".into(), "wool".into()] },
        ],
        settlements: vec![
            Settlement { name: "Millbrook".into(), population: 120, region: "Southfield".into(), ruler: None },
            Settlement { name: "Stonekeep".into(), population: 450, region: "Northvale".into(), ruler: None },
        ],
    });
    e
}
