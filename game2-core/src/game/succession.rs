//! Succession/inheritance: when a ruler dies, power transfers.
//! Emergent: who inherits affects governance style + stability.

use crate::engine::ecs::{Entity, World};
use crate::sim::community::Citizen;

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

/// Ruler dies -> succession happens. The heir chosen by the lineage's
/// governance becomes ruler and leaves the heir list, and stability drops
/// by how contested the succession was. Returns None (a crisis that still
/// costs stability) when there is no heir.
pub fn succeed(world: &mut World, lineage: Entity) -> Option<Entity> {
    let (heirs, pick_by) = {
        let l = world.get_component::<Lineage>(lineage)?;
        (l.heirs.clone(), match l.governance {
            Governance::Primogeniture | Governance::Sacred => None,
            Governance::Strongest => Some((|c: &Citizen| c.influence) as fn(&Citizen) -> f32),
            Governance::Election => Some((|c: &Citizen| c.authority) as fn(&Citizen) -> f32),
        })
    };
    // Primogeniture/Sacred follow birth order; otherwise the heir with the
    // most power (or council backing) wins, ties going to the earlier heir
    let chosen = match pick_by {
        None => heirs.first().copied(),
        Some(score) => heirs.iter().copied().fold(None, |best: Option<(Entity, f32)>, h| {
            let s = world.get_component::<Citizen>(h).map(score).unwrap_or(0.0);
            match best { Some((_, b)) if b >= s => best, _ => Some((h, s)) }
        }).map(|(h, _)| h),
    };
    let l = world.get_component_mut::<Lineage>(lineage)?;
    l.stability = (l.stability - succession_crisis(l)).max(0.0);
    let heir = chosen?;
    l.heirs.retain(|&h| h != heir);
    l.current_ruler = heir;
    Some(heir)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn citizen(w: &mut World, influence: f32, authority: f32) -> Entity {
        let e = w.create_entity();
        w.add_component(e, Citizen { influence, authority, ideology: "lord".into(), allegiance: "none".into() });
        e
    }

    #[test]
    fn test_primogeniture_crowns_eldest_and_advances_line() {
        let mut w = World::new();
        let king = w.create_entity();
        let (a, b) = (citizen(&mut w, 0.1, 0.0), citizen(&mut w, 0.9, 0.0));
        let l = spawn_lineage(&mut w, "Aldrics", king, vec![a, b], Governance::Primogeniture);
        assert_eq!(succeed(&mut w, l), Some(a));
        let lin = w.get_component::<Lineage>(l).unwrap();
        assert_eq!(lin.current_ruler, a);
        assert_eq!(lin.heirs, vec![b]);
        assert!(lin.stability < 0.7, "a contested succession costs stability");
        // Next death passes the crown down the line
        assert_eq!(succeed(&mut w, l), Some(b));
    }

    #[test]
    fn test_strongest_and_election_pick_by_power() {
        let mut w = World::new();
        let king = w.create_entity();
        let weak_elected = citizen(&mut w, 0.1, 0.9);
        let strong = citizen(&mut w, 0.8, 0.1);
        let l = spawn_lineage(&mut w, "Bram", king, vec![weak_elected, strong], Governance::Strongest);
        assert_eq!(succeed(&mut w, l), Some(strong));
        let l2 = spawn_lineage(&mut w, "Corvin", king, vec![strong, weak_elected], Governance::Election);
        assert_eq!(succeed(&mut w, l2), Some(weak_elected));
    }

    #[test]
    fn test_no_heir_is_a_crisis() {
        let mut w = World::new();
        let king = w.create_entity();
        let l = spawn_lineage(&mut w, "Dagny", king, vec![], Governance::Primogeniture);
        assert_eq!(succeed(&mut w, l), None);
        let lin = w.get_component::<Lineage>(l).unwrap();
        assert_eq!(lin.current_ruler, king);
        assert_eq!(lin.stability, 0.0);
    }
}
