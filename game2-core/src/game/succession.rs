//! Succession/inheritance: when a ruler dies, power transfers.
//! Emergent: who inherits affects governance style + stability.

use crate::engine::ecs::{Entity, World};

/// A ruling lineage
#[derive(Debug)]
pub struct Lineage {
    pub name: String,
    pub current_ruler: Entity,
    pub heirs: Vec<Entity>,       // succession order
    pub stability: f32,           // 0-1, drops on contested succession
    pub governance: Governance,
}

#[derive(Debug)]
pub enum Governance {
    Primogeniture,   // eldest child inherits
    Election,        // council chooses
    Strongest,       // most powerful takes it
    Sacred,          // religious/dynastic right
}

/// Ruler dies -> succession happens
pub fn succeed(world: &mut World, lineage: Entity) -> Option<Entity> {
    if let Some(l) = world.get_component::<Lineage>(lineage) {
        let heirs = l.heirs.clone();
        let _governance = &l.governance;
        if heirs.is_empty() {
            return None; // no heir - succession crisis
        }
        return Some(heirs[0]); // primogeniture default
    }
    None
}

/// Succession crisis: no clear heir, factions fight
pub fn succession_crisis(lineage: &Lineage) -> f32 {
    // Stability drops based on contested succession
    if lineage.heirs.is_empty() { 0.8 }
    else if lineage.heirs.len() == 1 { 0.1 }
    else { 0.3 + (lineage.heirs.len() as f32 * 0.1) }
}

/// Spawn a lineage
pub fn spawn_lineage(world: &mut World, name: &str, ruler: Entity, heirs: Vec<Entity>, gov: Governance) -> Entity {
    let e = world.create_entity();
    world.add_component(e, Lineage {
        name: name.into(), current_ruler: ruler, heirs,
        stability: 0.7, governance: gov,
    });
    e
}
