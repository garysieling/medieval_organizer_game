//! The fold's animals: sheep and goats as individual entities that graze
//! the meadow, get milked and shorn, and - now and then - get taken by
//! wolves. They're not pets (`Pet` is owner-bond companions); livestock
//! belong to a fold and keep themselves fed off the grass.

use crate::engine::ecs::{Entity, World};
use crate::game::villager::Pos;

/// What's in the fold
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum LivestockKind {
    Sheep,
    Goat,
    /// A yard hen: attached to a Home, lays eggs
    Hen,
}

/// One animal belonging to a fold (`BuildingKind::Pasture` entity)
#[derive(Debug)]
pub struct Livestock {
    pub kind: LivestockKind,
    pub fold: Entity,
}

/// Put an animal on the map near its fold
pub fn spawn_animal(world: &mut World, kind: LivestockKind, fold: Entity, x: f32, y: f32) -> Entity {
    let e = world.create_entity();
    world.add_component(e, Pos { x, y });
    world.add_component(e, Livestock { kind, fold });
    e
}

/// The animals of one fold, alive and grazing
pub fn flock(world: &World, fold: Entity) -> Vec<Entity> {
    world.query::<Livestock>().iter().copied()
        .filter(|&e| world.get_component::<Livestock>(e).map(|l| l.fold == fold).unwrap_or(false))
        .collect()
}

/// (sheep, goats) alive in the fold
pub fn counts(world: &World, fold: Entity) -> (usize, usize) {
    let mut out = (0, 0);
    for e in flock(world, fold) {
        match world.get_component::<Livestock>(e).unwrap().kind {
            LivestockKind::Sheep => out.0 += 1,
            LivestockKind::Goat => out.1 += 1,
            LivestockKind::Hen => {}
        }
    }
    out
}

/// The middle of the flock, for drifting and drawing the grazing point
pub fn centroid(world: &World, fold: Entity) -> Option<(f32, f32)> {
    let (mut sx, mut sy, mut n) = (0.0, 0.0, 0usize);
    for e in flock(world, fold) {
        if let Some(p) = world.get_component::<Pos>(e) { sx += p.x; sy += p.y; n += 1; }
    }
    if n == 0 { None } else { Some((sx / n as f32, sy / n as f32)) }
}
