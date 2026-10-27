//! Religion simulation: faith traits, gatherings, emergent cohesion.
//! From backlog: "emergent effects of religious gatherings not modelled in any game"
//! Bioshock-style satire possible; religion shapes community and power.

use crate::engine::ecs::{Entity, World};
use crate::game::relationships::{record_interaction, Interaction, InteractionKind};
use crate::game::villager::Hunger;

/// An agent's faith/religious identity
#[derive(Debug)]
pub struct Faith {
    pub religion: String,       // which faith
    pub devotion: f32,          // 0-1 intensity
    pub traditions: Vec<String>,// rituals/beliefs adopted
}

/// A religious community
#[derive(Debug)]
pub struct Congregation {
    pub name: String,
    pub members: Vec<Entity>,
    pub leader: Option<Entity>,
    pub orthodoxy: f32,         // how strict about beliefs
    pub services: Vec<Service>,
}

/// A religious service/ritual
#[derive(Debug)]
pub struct Service {
    pub name: String,
    pub frequency: String,      // daily/weekly/seasonal
    pub participants: Vec<Entity>,
    pub cohesion_boost: f32,    // emergent bonding effect
    pub charity_output: f32,    // emergent material aid
}

/// Emergent effect: gatherings produce social cohesion
pub struct GatherSystem;
impl crate::engine::ecs::System for GatherSystem {
    fn run(&mut self, world: &mut World) {
        // Congregations strengthen member bonds through services
        for c in world.query::<Congregation>() {
            if let Some(cong) = world.get_component::<Congregation>(c) {
                let cohesion = cong.services.iter()
                    .map(|s| s.cohesion_boost)
                    .sum::<f32>();
                let _ = cohesion; // would update member Bond components
            }
        }
    }
}

/// Devotion lost per tick without practice
pub const LAPSE_RATE: f32 = 0.002;

/// Faith lapses without practice: devotion slowly fades each tick, and only
/// attending services (perform_service) builds it back up
pub struct DevotionSystem;
impl crate::engine::ecs::System for DevotionSystem {
    fn run(&mut self, world: &mut World) {
        for e in world.query::<Faith>() {
            if let Some(f) = world.get_component_mut::<Faith>(e) {
                f.devotion = (f.devotion - LAPSE_RATE).max(0.0);
            }
        }
    }
}

/// Hunger a hungry member regains per giver per unit of charity_output
pub const AID_PER_GIVER: f32 = 20.0;
/// Below this hunger a member is in need of the congregation's charity
pub const NEEDY_HUNGER: f32 = 30.0;

/// Congregation performs a service - emergent effects. Each participant who
/// belongs to the congregation grows in devotion (with diminishing returns,
/// so the devout gain less than the lapsed), and the shared ritual bonds
/// every pair of participants. With charity_output > 0, the fed members who
/// attend share food with every hungry member - present or not - so more
/// givers mean more aid. Returns how many members took part.
pub fn perform_service(world: &mut World, cong: Entity, service: Service) -> usize {
    let members = match world.get_component::<Congregation>(cong) {
        Some(c) => c.members.clone(),
        None => return 0,
    };
    let present: Vec<Entity> = service.participants.iter()
        .copied()
        .filter(|p| members.contains(p))
        .collect();
    for &p in &present {
        if let Some(f) = world.get_component_mut::<Faith>(p) {
            f.devotion = (f.devotion + service.cohesion_boost * (1.0 - f.devotion)).min(1.0);
        }
    }
    for (i, &a) in present.iter().enumerate() {
        for &b in &present[i + 1..] {
            record_interaction(world, a, Interaction {
                other: b, kind: InteractionKind::Conversation, intensity: service.cohesion_boost,
            });
        }
    }
    if service.charity_output > 0.0 {
        let fed = |w: &World, e: Entity| w.get_component::<Hunger>(e).map(|h| h.value >= NEEDY_HUNGER).unwrap_or(false);
        let givers = present.iter().filter(|&&p| fed(world, p)).count();
        let aid = service.charity_output * givers as f32 * AID_PER_GIVER;
        for &m in &members {
            if let Some(h) = world.get_component_mut::<Hunger>(m) {
                if h.value < NEEDY_HUNGER { h.value = (h.value + aid).min(100.0); }
            }
        }
    }
    present.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gather_produces_cohesion() {
        let mut w = World::new();
        let c = w.create_entity();
        w.add_component(c, Congregation {
            name: "Village Church".into(),
            members: vec![],
            leader: None,
            orthodoxy: 0.7,
            services: vec![Service {
                name: "Sunday Mass".into(),
                frequency: "weekly".into(),
                participants: vec![],
                cohesion_boost: 0.2,
                charity_output: 0.1,
            }],
        });
        // Service exists with cohesion effect
        assert!(w.get_component::<Congregation>(c).unwrap().services[0].cohesion_boost > 0.0);
    }

    fn believer(w: &mut World, devotion: f32) -> Entity {
        let e = w.create_entity();
        w.add_component(e, Faith { religion: "Old Faith".into(), devotion, traditions: vec![] });
        e
    }

    fn congregation(w: &mut World, members: Vec<Entity>) -> Entity {
        let c = w.create_entity();
        w.add_component(c, Congregation {
            name: "Village Church".into(), members, leader: None, orthodoxy: 0.5, services: vec![],
        });
        c
    }

    fn service(participants: Vec<Entity>) -> Service {
        Service {
            name: "Harvest Rite".into(), frequency: "weekly".into(),
            participants, cohesion_boost: 0.2, charity_output: 0.0,
        }
    }

    #[test]
    fn test_devotion_lapses_without_practice() {
        use crate::engine::ecs::System;
        let mut w = World::new();
        let b = believer(&mut w, 0.5);
        for _ in 0..50 { DevotionSystem.run(&mut w); }
        assert!(w.get_component::<Faith>(b).unwrap().devotion < 0.5, "faith with no services should fade");
    }

    #[test]
    fn test_service_raises_only_attending_members() {
        let mut w = World::new();
        let attends = believer(&mut w, 0.3);
        let stays_home = believer(&mut w, 0.3);
        let outsider = believer(&mut w, 0.3); // attends but isn't a member
        let c = congregation(&mut w, vec![attends, stays_home]);
        assert_eq!(perform_service(&mut w, c, service(vec![attends, outsider])), 1);
        let dev = |e| w.get_component::<Faith>(e).unwrap().devotion;
        assert!(dev(attends) > 0.3);
        assert_eq!(dev(stays_home), 0.3);
        assert_eq!(dev(outsider), 0.3);
    }

    #[test]
    fn test_service_bonds_participants() {
        use crate::game::relationships::{find_relationship, Relationship};
        let mut w = World::new();
        let a = believer(&mut w, 0.5);
        let b = believer(&mut w, 0.5);
        let c = congregation(&mut w, vec![a, b]);
        perform_service(&mut w, c, service(vec![a, b]));
        let rel = find_relationship(&w, a, b).expect("worshipping together should form a bond");
        assert!(w.get_component::<Relationship>(rel).unwrap().strength > 0.0);
    }

    #[test]
    fn test_regular_services_settle_below_full_devotion() {
        use crate::engine::ecs::System;
        let mut w = World::new();
        let b = believer(&mut w, 0.1);
        let c = congregation(&mut w, vec![b]);
        for tick in 0..700 {
            DevotionSystem.run(&mut w);
            if tick % 7 == 0 { perform_service(&mut w, c, service(vec![b])); }
        }
        let d = w.get_component::<Faith>(b).unwrap().devotion;
        assert!(d > 0.5 && d < 0.95, "weekly practice should hold devotion high but not pinned at 1, got {}", d);
    }

    #[test]
    fn test_charity_feeds_hungry_members_from_the_fed() {
        let mut w = World::new();
        let givers: Vec<Entity> = (0..3).map(|_| {
            let e = believer(&mut w, 0.5);
            w.add_component(e, Hunger { value: 80.0 });
            e
        }).collect();
        let hungry = believer(&mut w, 0.5);
        w.add_component(hungry, Hunger { value: 5.0 });
        let mut members = givers.clone();
        members.push(hungry);
        let c = congregation(&mut w, members);
        perform_service(&mut w, c, Service {
            name: "Harvest Rite".into(), frequency: "weekly".into(),
            participants: givers.clone(), cohesion_boost: 0.1, charity_output: 0.1,
        });
        // 3 givers x 0.1 x 20 = +6, though the hungry member stayed home
        assert!((w.get_component::<Hunger>(hungry).unwrap().value - 11.0).abs() < 1e-4);
        assert_eq!(w.get_component::<Hunger>(givers[0]).unwrap().value, 80.0, "the fed don't get aid");
    }

    #[test]
    fn test_no_charity_without_fed_givers() {
        let mut w = World::new();
        let a = believer(&mut w, 0.5);
        w.add_component(a, Hunger { value: 10.0 });
        let c = congregation(&mut w, vec![a]);
        perform_service(&mut w, c, Service {
            name: "Rite".into(), frequency: "weekly".into(),
            participants: vec![a], cohesion_boost: 0.1, charity_output: 0.5,
        });
        assert_eq!(w.get_component::<Hunger>(a).unwrap().value, 10.0, "the starving can't feed themselves");
    }
}
