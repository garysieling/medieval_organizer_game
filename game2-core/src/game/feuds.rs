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

/// Intensity below which arbitration settles a feud outright
pub const RESOLVABLE_INTENSITY: f32 = 0.5;
/// How much a failed arbitration still cools a feud
pub const ARBITRATION_COOLING: f32 = 0.2;

/// Peacemaking: arbitration resolves the feud if tempers are low enough.
/// Otherwise talks still cool it and pause escalation (Peacemaking), so
/// repeated arbitration can settle even a burning feud.
pub fn peacemaking(world: &mut World, feud: Entity, _arbitrator: Entity) -> bool {
    if let Some(f) = world.get_component_mut::<Feud>(feud) {
        if f.status == FeudStatus::Resolved { return true; }
        if f.intensity < RESOLVABLE_INTENSITY {
            f.status = FeudStatus::Resolved;
        } else {
            f.intensity = (f.intensity - ARBITRATION_COOLING).max(0.0);
            f.status = FeudStatus::Peacemaking;
        }
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

/// System: feuds escalate unless settled or under arbitration
pub struct FeudSystem;
impl crate::engine::ecs::System for FeudSystem {
    fn run(&mut self, world: &mut World) {
        for e in world.query::<Feud>() {
            // a temperate soul's feud cools where a wild one's climbs:
            // the offender's settled temper bends the escalation (#950)
            let offender = world.get_component::<Feud>(e).map(|f| f.offender);
            let damp = offender.and_then(|o| world.get_component::<crate::game::virtue::Virtues>(o))
                .map(|v| crate::game::virtue::weight(v, "feud")).unwrap_or(1.0);
            if let Some(f) = world.get_component_mut::<Feud>(e) {
                f.tick += 1;
                // Open conflict keeps escalating too; only talks or a
                // settlement stop it
                if matches!(f.status, FeudStatus::Active | FeudStatus::Burning) {
                    f.intensity = (f.intensity + 0.05 * damp).min(1.0);
                    if f.intensity > 0.7 { f.status = FeudStatus::Burning; }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::ecs::System;

    fn feud(w: &mut World) -> (Entity, Entity) {
        let a = w.create_entity();
        let b = w.create_entity();
        let arb = w.create_entity();
        (spawn_feud(w, a, b, "insult"), arb)
    }

    #[test]
    fn test_burning_feud_keeps_escalating() {
        let mut w = World::new();
        let (f, _) = feud(&mut w);
        for _ in 0..20 { FeudSystem.run(&mut w); }
        let fd = w.get_component::<Feud>(f).unwrap();
        assert_eq!(fd.status, FeudStatus::Burning);
        assert!(fd.intensity > 0.95, "burning feud stalled at {}", fd.intensity);
        assert_eq!(fd.tick, 20);
    }

    #[test]
    fn test_young_feud_resolves_by_arbitration() {
        let mut w = World::new();
        let (f, arb) = feud(&mut w);
        assert!(peacemaking(&mut w, f, arb));
        FeudSystem.run(&mut w);
        let fd = w.get_component::<Feud>(f).unwrap();
        assert_eq!(fd.status, FeudStatus::Resolved);
        assert!((fd.intensity - 0.3).abs() < 1e-6, "settled feuds don't escalate");
    }

    #[test]
    fn test_repeated_arbitration_settles_burning_feud() {
        let mut w = World::new();
        let (f, arb) = feud(&mut w);
        for _ in 0..20 { FeudSystem.run(&mut w); }
        assert!(!peacemaking(&mut w, f, arb));
        assert_eq!(w.get_component::<Feud>(f).unwrap().status, FeudStatus::Peacemaking);
        // Talks pause escalation
        let before = w.get_component::<Feud>(f).unwrap().intensity;
        FeudSystem.run(&mut w);
        assert_eq!(w.get_component::<Feud>(f).unwrap().intensity, before);
        // 1.0 -> 0.8 -> 0.6 -> 0.4, then the fourth round settles it
        assert!(!peacemaking(&mut w, f, arb));
        assert!(!peacemaking(&mut w, f, arb));
        assert!(peacemaking(&mut w, f, arb));
        assert_eq!(w.get_component::<Feud>(f).unwrap().status, FeudStatus::Resolved);
    }
}
