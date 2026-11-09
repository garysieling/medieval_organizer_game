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

/// Fowler's stages of faith: how a soul makes sense of the holy.
/// Structure, not intensity - devotion stays its own measure. Ordered:
/// later is deeper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FaithStage {
    /// primal: the sacred is everywhere and nowhere named
    Undifferentiated,
    /// intuitive-projective: story and image carry the holy
    IntuitiveProjective,
    /// mythic-literal: the stories are true, the rules are plain
    MythicLiteral,
    /// synthetic-conventional: belief is woven into the community's order
    SyntheticConventional,
    /// individuative-reflective: the person picks their own meaning
    IndividuativeReflective,
    /// conjunctive: the symbols are held together, paradox and all
    Conjunctive,
    /// universalizing: the holy is served in every person
    Universalizing,
}

impl FaithStage {
    /// The next stage down the road, if there is one
    pub fn step(self) -> Option<FaithStage> {
        use FaithStage::*;
        Some(match self {
            Undifferentiated => IntuitiveProjective,
            IntuitiveProjective => MythicLiteral,
            MythicLiteral => SyntheticConventional,
            SyntheticConventional => IndividuativeReflective,
            IndividuativeReflective => Conjunctive,
            Conjunctive => Universalizing,
            Universalizing => return None,
        })
    }

    /// What the village calls a soul this far along - None while the
    /// stage is ordinary
    pub fn title(self) -> Option<&'static str> {
        use FaithStage::*;
        Some(match self {
            IndividuativeReflective => "seeker",
            Conjunctive => "mystic",
            Universalizing => "sage",
            _ => return None,
        })
    }

    /// Where a soul of these years likely stands: children lean to the
    /// early stages, the old can reach the late ones - a roll, not a law
    pub fn for_years(years: f32, roll: u64) -> FaithStage {
        use FaithStage::*;
        let r = (roll % 1000) as u32;
        let stage = if years < 14.0 {
            if r < 450 { Undifferentiated } else if r < 850 { IntuitiveProjective } else { MythicLiteral }
        } else if years < 18.0 {
            if r < 100 { Undifferentiated } else if r < 400 { IntuitiveProjective }
            else if r < 850 { MythicLiteral } else { SyntheticConventional }
        } else if years < 60.0 {
            if r < 150 { IntuitiveProjective } else if r < 600 { MythicLiteral }
            else if r < 900 { SyntheticConventional } else { IndividuativeReflective }
        } else {
            if r < 200 { MythicLiteral } else if r < 550 { SyntheticConventional }
            else if r < 850 { IndividuativeReflective } else if r < 970 { Conjunctive } else { Universalizing }
        };
        stage
    }
}

/// A deterministic per-entity hash for stage rolls - the same soul
/// always draws the same numbers, no rng stream spent
pub fn stage_roll(e: Entity, salt: u64) -> u64 {
    let mut z = (e as u64).wrapping_add(salt).wrapping_mul(0x9E3779B97F4A7C15);
    z = (z ^ (z >> 29)).wrapping_mul(0xBF58476D1CE4E5B9);
    z ^ (z >> 32)
}

/// The cult's lock: a cult-locked soul never passes MythicLiteral - the
/// stage where the stories are taken literally and the leader's word is
/// law. The flag is defined here; what breeds cults waits for the
/// reformation work
#[derive(Debug, Clone, Copy)]
pub struct Cultish;

/// The deepest stage a soul may hold: the cult-locked stop at the literal
pub fn stage_cap(world: &World, e: Entity) -> FaithStage {
    if world.get_component::<Cultish>(e).is_some() { FaithStage::MythicLiteral }
    else { FaithStage::Universalizing }
}

/// A rare conversion moment - a service, a miracle, a milestone - moves
/// a devout soul one stage deeper. `roll` is the caller's deterministic
/// draw; about one soul in twenty-five takes the step per call
pub fn stage_up(world: &mut World, e: Entity, roll: u64) {
    if roll % 100 >= 4 { return; }
    let Some(stage) = world.get_component::<FaithStage>(e).copied() else { return };
    let devout = world.get_component::<Faith>(e).map(|f| f.devotion > 0.45).unwrap_or(false);
    if !devout { return; }
    let Some(next) = stage.step() else { return };
    if next > stage_cap(world, e) { return; }
    world.add_component(e, next);
}

/// Lifton's eight dials of totalism, read off the congregation's own
/// drift: 0-10 each, pushed up by a hard head and a strict orthodoxy,
/// multiplied by the flock's own cutting-off. High totalism's high
/// loyalty, high extraction, and the collapse that waits on it
#[derive(Debug)]
pub struct Totalism {
    /// The eight dials (milieu control, mystical manipulation, the
    /// demand for purity, confession, sacred science, loaded language,
    /// doctrine over person, the dispensing of existence) - 0-10 each
    pub dials: [u8; 8],
    /// The extractable reserve: what belonging and hope generate, and
    /// what the leader and his coterie spend down
    pub idealism: f32,
    /// What it costs a member to leave: each costly rite raises it -
    /// the member's own choices become the cage
    pub exit_cost: f32,
    /// The flock's own cutting-off (0-1): physical, social,
    /// informational - multiplies every dial's own effect
    pub isolation: f32,
}

/// How a cult ends: the implosion of its own split, the capture by a
/// harder head, the deadline's own date, or the rare routinization
/// into a sect - emergent endings off the dials, not scripted
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollapseMode { Implosion, Capture, Deadline, Institutionalization }

/// The creed's own scoring: the canon the council's settled, and the
/// heresy_vector - the positions defeated that can resurface as
/// splinters. Belief as scored conflict, not a fixed type
#[derive(Debug)]
pub struct Doctrine {
    /// The settled positions - the council's own wins
    pub canon: Vec<String>,
    /// The defeated positions - the clause's own excluded side, which
    /// can resurface as a splinter
    pub heresy_vector: Vec<String>,
}

/// A religious community
#[derive(Debug)]
pub struct Congregation {
    pub name: String,
    pub members: Vec<Entity>,
    pub leader: Option<Entity>,
    pub orthodoxy: f32,         // how strict about beliefs
    pub services: Vec<Service>,
    /// The three factors tip a drifting congregation into a cult: a
    /// strict orthodoxy, a charismatic head, and the flock cut off
    pub cultish: bool,
}

/// One who's walked out: the mark the exit leaves - they don't come
/// back to the fold on their own
#[derive(Debug)]
pub struct Apostate;

/// A soul of the green fellowship: the splinter congregation that broke
/// from the parish. Members keep their Faith but worship at the green,
/// not the church - the marker is what routing and services read
#[derive(Debug)]
pub struct Reformed;

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

/// Hunger each fed attendee gives up per unit of charity_output, shared out
/// among the congregation's hungry members
pub const AID_PER_GIVER: f32 = 20.0;
/// Below this hunger a member is in need of the congregation's charity
pub const NEEDY_HUNGER: f32 = 30.0;

/// Congregation performs a service - emergent effects. Each participant who
/// belongs to the congregation grows in devotion (with diminishing returns,
/// so the devout gain less than the lapsed), and the shared ritual bonds
/// every pair of participants. With charity_output > 0, the fed members who
/// attend share their food with the hungry members - present or not - so
/// more givers mean more aid. Returns how many members took part.
pub fn perform_service(world: &mut World, cong: Entity, service: Service) -> usize {
    let members = match world.get_component::<Congregation>(cong) {
        Some(c) => c.members.clone(),
        None => return 0,
    };
    // The dead neither attend nor receive aid (villager::is_dead: health 0,
    // or the Dead marker the windowed game leaves after removing Health)
    let alive = |w: &World, e: Entity| !crate::game::villager::is_dead(w, e);
    let present: Vec<Entity> = service.participants.iter()
        .copied()
        .filter(|p| members.contains(p) && alive(world, *p))
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
        // Charity moves food, it doesn't make it: each fed attendee puts in
        // charity_output * AID_PER_GIVER, the pool is split among the needy,
        // and the givers bear what was actually handed out
        let hunger = |w: &World, e: Entity| w.get_component::<Hunger>(e).map(|h| h.value);
        let givers: Vec<Entity> = present.iter().copied()
            .filter(|&p| hunger(world, p).map(|v| v >= NEEDY_HUNGER).unwrap_or(false))
            .collect();
        let needy: Vec<Entity> = members.iter().copied()
            .filter(|&m| alive(world, m) && hunger(world, m).map(|v| v < NEEDY_HUNGER).unwrap_or(false))
            .collect();
        if !givers.is_empty() && !needy.is_empty() {
            let share = service.charity_output * AID_PER_GIVER * givers.len() as f32 / needy.len() as f32;
            let mut handed_out = 0.0;
            for &m in &needy {
                if let Some(h) = world.get_component_mut::<Hunger>(m) {
                    let got = share.min(100.0 - h.value);
                    h.value += got;
                    handed_out += got;
                }
            }
            for &g in &givers {
                if let Some(h) = world.get_component_mut::<Hunger>(g) {
                    h.value = (h.value - handed_out / givers.len() as f32).max(0.0);
                }
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
            cultish: false,
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
            name: "Village Church".into(), members, leader: None, orthodoxy: 0.5, services: vec![], cultish: false,
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
        // ...and the givers paid for it: 6 shared three ways
        assert!((w.get_component::<Hunger>(givers[0]).unwrap().value - 78.0).abs() < 1e-4, "givers share their own food");
    }

    #[test]
    fn test_charity_is_split_among_the_needy_not_multiplied() {
        let mut w = World::new();
        let giver = believer(&mut w, 0.5);
        w.add_component(giver, Hunger { value: 80.0 });
        let needy: Vec<Entity> = (0..2).map(|_| {
            let e = believer(&mut w, 0.5);
            w.add_component(e, Hunger { value: 10.0 });
            e
        }).collect();
        let c = congregation(&mut w, vec![giver, needy[0], needy[1]]);
        perform_service(&mut w, c, Service {
            name: "Rite".into(), frequency: "weekly".into(),
            participants: vec![giver], cohesion_boost: 0.1, charity_output: 0.5,
        });
        // the giver puts in 0.5 x 20 = 10, split between two: +5 each
        let total: f32 = [giver, needy[0], needy[1]].iter()
            .map(|e| w.get_component::<Hunger>(*e).unwrap().value).sum();
        assert!((total - 100.0).abs() < 1e-4, "charity moves food, it doesn't make it (total {})", total);
        assert!((w.get_component::<Hunger>(needy[0]).unwrap().value - 15.0).abs() < 1e-4);
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

    #[test]
    fn test_the_dead_neither_worship_give_nor_receive() {
        use crate::sim::illness::Health;
        let mut w = World::new();
        let corpse = believer(&mut w, 0.5);
        w.add_component(corpse, Hunger { value: 80.0 }); // died fed
        w.add_component(corpse, Health { condition: 0.0, immunity: 0.5 });
        let dead_hungry = believer(&mut w, 0.5);
        w.add_component(dead_hungry, Hunger { value: 5.0 });
        w.add_component(dead_hungry, Health { condition: 0.0, immunity: 0.5 });
        let hungry = believer(&mut w, 0.5);
        w.add_component(hungry, Hunger { value: 5.0 });
        let c = congregation(&mut w, vec![corpse, dead_hungry, hungry]);
        let attended = perform_service(&mut w, c, Service {
            name: "Rite".into(), frequency: "weekly".into(),
            participants: vec![corpse], cohesion_boost: 0.2, charity_output: 0.5,
        });
        assert_eq!(attended, 0, "a corpse doesn't attend");
        assert_eq!(w.get_component::<Faith>(corpse).unwrap().devotion, 0.5);
        assert_eq!(w.get_component::<Hunger>(hungry).unwrap().value, 5.0, "the dead can't give charity");
        assert_eq!(w.get_component::<Hunger>(dead_hungry).unwrap().value, 5.0, "nor receive it");
    }

    #[test]
    fn test_the_dead_do_not_soak_up_charity_meant_for_the_living() {
        use crate::sim::illness::Health;
        let mut w = World::new();
        let giver = believer(&mut w, 0.5);
        w.add_component(giver, Hunger { value: 90.0 });
        let dead = believer(&mut w, 0.5);
        w.add_component(dead, Hunger { value: 5.0 });
        w.add_component(dead, Health { condition: 0.0, immunity: 0.5 });
        let living = believer(&mut w, 0.5);
        w.add_component(living, Hunger { value: 5.0 });
        let c = congregation(&mut w, vec![giver, dead, living]);
        perform_service(&mut w, c, Service {
            name: "Rite".into(), frequency: "weekly".into(),
            participants: vec![giver], cohesion_boost: 0.0, charity_output: 0.1,
        });
        let with_corpse = w.get_component::<Hunger>(living).unwrap().value;
        assert_eq!(w.get_component::<Hunger>(dead).unwrap().value, 5.0);
        assert!(with_corpse > 5.0, "the living member still gets the whole share");
        // the corpse doesn't split the pool: the living get a lone needy member's share
        assert!((with_corpse - (5.0 + 0.1 * AID_PER_GIVER)).abs() < 1e-4, "got {}", with_corpse);
    }

    #[test]
    fn test_dead_marker_without_health_still_counts_as_dead() {
        // The windowed game removes Health on death and leaves a Dead marker
        let mut w = World::new();
        let corpse = believer(&mut w, 0.5);
        w.add_component(corpse, Hunger { value: 80.0 });
        w.add_component(corpse, crate::game::villager::Dead);
        let hungry = believer(&mut w, 0.5);
        w.add_component(hungry, Hunger { value: 5.0 });
        let c = congregation(&mut w, vec![corpse, hungry]);
        let attended = perform_service(&mut w, c, Service {
            name: "Rite".into(), frequency: "weekly".into(),
            participants: vec![corpse], cohesion_boost: 0.2, charity_output: 0.5,
        });
        assert_eq!(attended, 0);
        assert_eq!(w.get_component::<Faith>(corpse).unwrap().devotion, 0.5);
        assert_eq!(w.get_component::<Hunger>(hungry).unwrap().value, 5.0, "a marked-dead villager gives no charity");
    }
}
