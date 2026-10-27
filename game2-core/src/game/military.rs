//! Military: simple abstracted battles, not a war sim.
//! From backlog: "very little in terms of military simulation - simple battles are fine"

use crate::engine::ecs::{Entity, World};

/// A military unit
#[derive(Debug)]
pub struct Unit {
    pub name: String,
    pub kind: UnitKind,
    pub strength: f32,
    pub morale: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnitKind {
    Infantry,
    Archers,
    Cavalry,
    Militia,
}

/// An army = collection of units
#[derive(Debug)]
pub struct Army {
    pub units: Vec<Entity>,
    pub commander: Option<Entity>,
    pub total_strength: f32,
}

/// Battle outcome
#[derive(Debug)]
pub struct BattleResult {
    pub winner: Option<Entity>,
    pub casualties_a: f32,
    pub casualties_b: f32,
    pub duration: u32, // ticks
}

/// Share of the enemy's power an army loses as casualties in one battle
pub const CASUALTY_RATE: f32 = 0.1;

/// An army's fighting power: each unit's strength weighted by its morale.
/// An army with no unit entities fights at its recorded total_strength.
pub fn army_power(world: &World, army: &Army) -> f32 {
    if army.units.is_empty() { return army.total_strength; }
    army.units.iter()
        .filter_map(|u| world.get_component::<Unit>(*u))
        .map(|u| u.strength * u.morale)
        .sum()
}

/// Resolve a battle simply between two army entities: strength + morale vs
/// strength + morale. The stronger side wins (a tie has no winner); each
/// side loses strength in proportion to the other's power, spread across
/// its units; the loser's morale breaks a little and the winner's rises.
/// Closer fights last longer.
pub fn resolve_battle(world: &mut World, a: Entity, b: Entity) -> BattleResult {
    if a == b {
        return BattleResult { winner: None, casualties_a: 0.0, casualties_b: 0.0, duration: 0 };
    }
    let (pa, pb) = match (world.get_component::<Army>(a), world.get_component::<Army>(b)) {
        (Some(x), Some(y)) => (army_power(world, x), army_power(world, y)),
        _ => return BattleResult { winner: None, casualties_a: 0.0, casualties_b: 0.0, duration: 0 },
    };
    let winner = if pa > pb { Some(a) } else if pb > pa { Some(b) } else { None };
    let casualties_a = take_losses(world, a, pb * CASUALTY_RATE, morale_shift(winner, a));
    let casualties_b = take_losses(world, b, pa * CASUALTY_RATE, morale_shift(winner, b));
    let closeness = 1.0 - (pa - pb).abs() / pa.max(pb).max(f32::EPSILON);
    BattleResult {
        winner,
        casualties_a,
        casualties_b,
        duration: 5 + (closeness * 10.0).round() as u32,
    }
}

fn morale_shift(winner: Option<Entity>, army: Entity) -> f32 {
    match winner {
        Some(w) if w == army => 0.1,
        Some(_) => -0.2,
        None => 0.0,
    }
}

/// Apply `losses` strength to an army, shared across its units in
/// proportion to their strength, and shift their morale; returns the
/// strength actually lost
fn take_losses(world: &mut World, army: Entity, losses: f32, morale: f32) -> f32 {
    let units = match world.get_component_mut::<Army>(army) {
        Some(a) if a.units.is_empty() => {
            let lost = losses.min(a.total_strength);
            a.total_strength -= lost;
            return lost;
        }
        Some(a) => a.units.clone(),
        None => return 0.0,
    };
    let before: f32 = units.iter()
        .filter_map(|u| world.get_component::<Unit>(*u).map(|u| u.strength))
        .sum();
    let lost = losses.min(before);
    for u in &units {
        if let Some(unit) = world.get_component_mut::<Unit>(*u) {
            if before > 0.0 { unit.strength -= lost * unit.strength / before; }
            unit.morale = (unit.morale + morale).clamp(0.0, 1.0);
        }
    }
    world.get_component_mut::<Army>(army).unwrap().total_strength = before - lost;
    lost
}

#[cfg(test)]
mod tests {
    use super::*;

    fn army(w: &mut World, units: &[(f32, f32)]) -> Entity {
        let ids = units.iter().map(|&(strength, morale)| {
            let u = w.create_entity();
            w.add_component(u, Unit { name: "u".into(), kind: UnitKind::Infantry, strength, morale });
            u
        }).collect::<Vec<_>>();
        let total = units.iter().map(|u| u.0).sum();
        let a = w.create_entity();
        w.add_component(a, Army { units: ids, commander: None, total_strength: total });
        a
    }

    #[test]
    fn test_stronger_army_wins_and_both_take_losses() {
        let mut w = World::new();
        let a = army(&mut w, &[(60.0, 1.0), (40.0, 1.0)]);
        let b = army(&mut w, &[(50.0, 1.0)]);
        let r = resolve_battle(&mut w, a, b);
        assert_eq!(r.winner, Some(a));
        assert!((r.casualties_a - 5.0).abs() < 1e-4);
        assert!((r.casualties_b - 10.0).abs() < 1e-4);
        assert!((w.get_component::<Army>(a).unwrap().total_strength - 95.0).abs() < 1e-4);
        let bu = w.get_component::<Army>(b).unwrap().units[0];
        assert!(w.get_component::<Unit>(bu).unwrap().morale < 1.0, "loser's morale breaks");
    }

    #[test]
    fn test_morale_can_beat_numbers() {
        let mut w = World::new();
        let many_broken = army(&mut w, &[(100.0, 0.3)]);
        let few_resolute = army(&mut w, &[(50.0, 0.9)]);
        let r = resolve_battle(&mut w, many_broken, few_resolute);
        assert_eq!(r.winner, Some(few_resolute));
    }

    #[test]
    fn test_even_fight_is_a_long_draw() {
        let mut w = World::new();
        let a = army(&mut w, &[(50.0, 0.8)]);
        let b = army(&mut w, &[(50.0, 0.8)]);
        let r = resolve_battle(&mut w, a, b);
        assert_eq!(r.winner, None);
        assert_eq!(r.duration, 15);
    }

    #[test]
    fn test_self_battle_does_not_double_damage_one_army() {
        let mut w = World::new();
        let a = army(&mut w, &[(100.0, 1.0)]);
        let r = resolve_battle(&mut w, a, a);
        assert_eq!(r.winner, None);
        assert_eq!(r.casualties_a, 0.0);
        assert_eq!(r.casualties_b, 0.0);
        assert!((w.get_component::<Army>(a).unwrap().total_strength - 100.0).abs() < 1e-4);
    }
}
