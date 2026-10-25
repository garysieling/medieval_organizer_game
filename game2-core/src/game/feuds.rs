//! Feuds: vengeance cycles from Icelandic sagas.
//! Offense -> vengeance -> escalation -> peacemaking/resolution (or continues).
//! "Can't get even without going one better" - never equalizes.

use crate::engine::ecs::{Entity, World};

/// An active feud between two parties
#[derive(Debug)]
pub struct Feud {
    pub offender: Entity,
    pub aggrieved: Entity,
    pub offense: String,          // what started it
    pub intensity: f32,           // current escalation
    pub status: FeudStatus,
    pub tick: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FeudStatus {
    Active,        // escalating
    Peacemaking,   // arbitration happening
    Resolved,      // settled
    Burning,       // open conflict
}

/// An act of vengeance
#[derive(Debug)]
pub struct Vengeance {
    pub feud: Entity,
    pub by: Entity,
    pub target: Entity,
    pub kind: VengeanceKind,
}

#[derive(Debug, Clone, Copy)]
pub enum VengeanceKind {
    Slander,       // reputation damage
    Theft,         // property loss
    Assault,       // physical harm
    KinAttack,     // hits family/allies
    Arson,         // destroys property
    Killing,       // maximum
}

/// Vengeance escalates: getting even = going one better
pub fn vengeance_escalation(kind: &VengeanceKind) -> VengeanceKind {
    match kind {
        VengeanceKind::Slander => VengeanceKind::Theft,
        VengeanceKind::Theft => VengeanceKind::Assault,
        VengeanceKind::Assault => VengeanceKind::KinAttack,
        VengeanceKind::KinAttack => VengeanceKind::Arson,
        VengeanceKind::Arson => VengeanceKind::Killing,
        VengeanceKind::Killing => VengeanceKind::Killing, // max
    }
}

/// Peacemaking: arbitration resolves (or fails) the feud
pub fn peacemaking(world: &mut World, feud: Entity, _arbitrator: Entity) -> bool {
    if let Some(f) = world.get_component_mut::<Feud>(feud) {
        // Success depends on intensity - lower = easier to resolve
        f.status = if f.intensity < 0.5 { FeudStatus::Resolved } else { FeudStatus::Burning };
        return f.status == FeudStatus::Resolved;
    }
    false
}

/// Spawn a feud
pub fn spawn_feud(world: &mut World, offender: Entity, aggrieved: Entity, offense: &str) -> Entity {
    let e = world.create_entity();
    world.add_component(e, Feud {
        offender, aggrieved, offense: offense.into(), intensity: 0.3, status: FeudStatus::Active, tick: 0,
    });
    e
}

/// System: feuds escalate if unresolved
pub struct FeudSystem;
impl crate::engine::ecs::System for FeudSystem {
    fn run(&mut self, world: &mut World) {
        for e in world.query::<Feud>() {
            if let Some(f) = world.get_component_mut::<Feud>(e) {
                if f.status == FeudStatus::Active {
                    f.intensity = (f.intensity + 0.05).min(1.0);
                    if f.intensity > 0.7 { f.status = FeudStatus::Burning; }
                }
            }
        }
    }
}
