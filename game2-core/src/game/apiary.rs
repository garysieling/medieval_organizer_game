//! The apiary's hives: skeps at the meadow edge that the beekeeper tends.
//! Honey for the table, wax for candles and seals. A hive isn't an
//! animal - it doesn't wander - it's a working thing that yields in season.

use crate::engine::ecs::{Entity, World};
use crate::game::villager::Pos;

/// How the bees are kept
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HiveKind {
    Skep,
    Log,
}

/// One hive standing in the apiary's yard
#[derive(Debug)]
pub struct Hive {
    pub kind: HiveKind,
    pub yard: Entity,
}

/// Stand a hive near its apiary
pub fn spawn_hive(world: &mut World, kind: HiveKind, yard: Entity, x: f32, y: f32) -> Entity {
    let e = world.create_entity();
    world.add_component(e, Pos { x, y });
    world.add_component(e, Hive { kind, yard });
    e
}

/// The hives of one apiary
pub fn hives_of(world: &World, yard: Entity) -> Vec<Entity> {
    world.query::<Hive>().iter().copied()
        .filter(|&e| world.get_component::<Hive>(e).map(|h| h.yard == yard).unwrap_or(false))
        .collect()
}
