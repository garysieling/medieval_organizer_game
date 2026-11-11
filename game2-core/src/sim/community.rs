//! Community organizing: influence networks, 1-1s, house meetings, power.
//! From backlog: "community organizers vs fascists/billionaires" central theme.
//! Player starts with influence without authority, builds organization.

use crate::engine::ecs::{Entity, World};
use crate::sim::organized_power::{self, ActionQuality, OrganizedPower};

/// A person with social/political attributes
#[derive(Debug)]
pub struct Citizen {
    pub influence: f32,      // how much sway they have
    pub authority: f32,      // formal power
    pub ideology: String,    // beliefs (laborer, clergy, merchant, lord)
    pub allegiance: String,  // who they follow
}

/// A social relationship between citizens
#[derive(Debug)]
pub struct Relation {
    pub from: Entity,
    pub to: Entity,
    pub trust: f32,       // -1 to 1
    pub relationship: String, // friend, coworker, neighbor, rival
}

/// An organization the player can build
#[derive(Debug)]
pub struct Organization {
    pub name: String,
    pub members: Vec<Entity>,
    pub leader: Entity,
    pub influence_pool: f32,   // collective sway
    pub resources: f32,        // treasury
    pub charter: String,       // purpose/method
    /// The men named before the blow falls: when the head's cut off
    /// the next of them stands up in his place
    pub successors: Vec<Entity>,
}

/// A 1-1 conversation that builds relationship
#[derive(Debug)]
pub struct OneOnOne {
    pub with: Entity,
    pub depth: f32, // how personal/deep the convo got
}

/// A house meeting - small group recruitment
#[derive(Debug)]
pub struct HouseMeeting {
    pub attendees: Vec<Entity>,
    pub host: Entity,
    pub issues: Vec<String>,
}

/// System: organizations accumulate influence from their living members
/// (members who have died are taken off the rolls)
pub struct InfluenceSystem;
impl crate::engine::ecs::System for InfluenceSystem {
    fn run(&mut self, world: &mut World) {
        for o in world.query::<Organization>() {
            // a faction's people are read, not stored: its pool is set by
            // the weekly accounting, not the member rolls it doesn't keep
            if world.get_component::<crate::sim::factions::Faction>(o).is_some() { continue; }
            // The dead leave the rolls: they lend no sway and aren't counted
            // as members (villager::is_dead also sees the windowed game's
            // Dead marker, since it removes Health on death)
            let dead = |w: &World, e: Entity| crate::game::villager::is_dead(w, e);
            let gone: Vec<Entity> = world.get_component::<Organization>(o)
                .map(|org| org.members.iter().copied().filter(|&m| dead(world, m)).collect())
                .unwrap_or_default();
            if !gone.is_empty() {
                if let Some(org) = world.get_component_mut::<Organization>(o) {
                    org.members.retain(|m| !gone.contains(m));
                }
            }
            // Collect member influence first (immutable), then mutate
            let member_influence: f32 = {
                match world.get_component::<Organization>(o) {
                    Some(org) => org.members.iter()
                        .filter_map(|m| world.get_component::<Citizen>(*m).map(|c| c.influence))
                        .sum(),
                    None => continue,
                }
            };
            if let Some(org) = world.get_component_mut::<Organization>(o) {
                org.influence_pool = member_influence;
            }
        }
    }
}

/// System: 1-1s deepen relationships. A OneOnOne on an entity is a
/// conversation it has this tick with `with`; the system records it as a
/// conversation whose intensity is how deep it went, then clears it (the
/// meeting is over), so a personal 1-1 builds more trust than small talk.
pub struct OneOnOneSystem;
impl crate::engine::ecs::System for OneOnOneSystem {
    fn run(&mut self, world: &mut World) {
        use crate::game::relationships::{record_interaction, Interaction, InteractionKind};
        for e in world.query::<OneOnOne>() {
            let (with, depth) = match world.get_component::<OneOnOne>(e) {
                Some(m) => (m.with, m.depth),
                None => continue,
            };
            world.remove_component::<OneOnOne>(e);
            if with == e { continue; } // no 1-1 with yourself
            record_interaction(world, e, Interaction {
                other: with, kind: InteractionKind::Conversation, intensity: depth,
            });
        }
    }
}

/// Recruit a citizen into an organization via relationship
pub const RECRUIT_MIN_TRUST: f32 = 0.5;

pub fn recruit(world: &mut World, org: Entity, citizen: Entity, rel_quality: f32) -> bool {
    // the wronged need less persuading: repression's deferred debt arrives
    // as recruits - whoever watched the crackdown already half-believes
    let quality = rel_quality + if world.get_component::<crate::game::reactions::Radicalized>(citizen).is_some() {
        crate::game::reactions::RADICAL_RECRUIT_BONUS
    } else { 0.0 };
    if quality < RECRUIT_MIN_TRUST { return false; } // need trust to recruit
    if let Some(o) = world.get_component_mut::<Organization>(org) {
        if o.members.contains(&citizen) { return false; }
        o.members.push(citizen);
        return true;
    }
    false
}

/// An org's current sway: its members' pooled influence, amplified by the
/// organized power it has built up. Read-only, for reporting.
pub fn influence(world: &World, org: Entity) -> f32 {
    let pool = match world.get_component::<Organization>(org) {
        Some(o) => o.influence_pool,
        None => return 0.0,
    };
    let capacity = world.get_component::<OrganizedPower>(org).map(|p| p.capacity).unwrap_or(0.0);
    pool * (1.0 + capacity)
}

/// Player exercises influence via an organization. Using power is itself an
/// organizing action: a routine ask with members behind it keeps the
/// muscle working, while acting with no members at all overreaches and
/// burns capacity.
pub fn exert_influence(world: &mut World, org: Entity, _target: &str) -> f32 {
    let sway = influence(world, org); // sway toward target
    let has_members = world.get_component::<Organization>(org)
        .map(|o| !o.members.is_empty()).unwrap_or(false);
    let quality = if has_members { ActionQuality::Transactional } else { ActionQuality::Overreach };
    organized_power::record_action(world, org, quality);
    sway
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recruit_needs_trust() {
        let mut w = World::new();
        let o = w.create_entity();
        let leader = w.create_entity();
        w.add_component(o, Organization {
            name: "Guild".into(), members: vec![], leader,
            influence_pool: 0.0, resources: 100.0, charter: "Trade".into(), successors: vec![] });
        let c = w.create_entity();
        assert!(!recruit(&mut w, o, c, 0.3)); // low trust fails
        assert!(recruit(&mut w, o, c, 0.8));  // high trust succeeds
        assert!(!recruit(&mut w, o, c, 0.8)); // already a member
        assert_eq!(w.get_component::<Organization>(o).unwrap().members.len(), 1);
    }

    #[test]
    fn test_influence_system_pools_member_influence() {
        use crate::engine::ecs::System;
        let mut w = World::new();
        let o = w.create_entity();
        let leader = w.create_entity();
        w.add_component(o, Organization {
            name: "Guild".into(), members: vec![], leader,
            influence_pool: 0.0, resources: 100.0, charter: "Trade".into(), successors: vec![] });
        for infl in [0.2, 0.3] {
            let c = w.create_entity();
            w.add_component(c, Citizen { influence: infl, authority: 0.0, ideology: "laborer".into(), allegiance: "none".into() });
            recruit(&mut w, o, c, 0.8);
        }
        // recruiting alone doesn't move the pool; the system must run
        assert_eq!(exert_influence(&mut w, o, "village"), 0.0);
        InfluenceSystem.run(&mut w);
        assert!((exert_influence(&mut w, o, "village") - 0.5).abs() < 1e-6);
    }

    fn guild_with_power(w: &mut World, members: usize, capacity: f32) -> Entity {
        let o = w.create_entity();
        let leader = w.create_entity();
        w.add_component(o, Organization {
            name: "Guild".into(), members: vec![], leader,
            influence_pool: 0.5, resources: 0.0, charter: "Wages".into(), successors: vec![] });
        w.add_component(o, OrganizedPower { capacity, idle_ticks: 7 });
        for _ in 0..members {
            let c = w.create_entity();
            recruit(w, o, c, 0.8);
        }
        o
    }

    #[test]
    fn test_organized_power_amplifies_influence() {
        let mut w = World::new();
        let weak = guild_with_power(&mut w, 2, 0.0);
        let strong = guild_with_power(&mut w, 2, 1.0);
        assert!((influence(&w, weak) - 0.5).abs() < 1e-6);
        assert!((influence(&w, strong) - 1.0).abs() < 1e-6, "capacity 1.0 should double sway");
    }

    #[test]
    fn test_exerting_influence_uses_the_muscle() {
        let mut w = World::new();
        let o = guild_with_power(&mut w, 2, 0.5);
        exert_influence(&mut w, o, "village");
        let p = w.get_component::<OrganizedPower>(o).unwrap();
        assert!(p.capacity > 0.5, "a routine ask with members should build a little power");
        assert_eq!(p.idle_ticks, 0);
    }

    #[test]
    fn test_exerting_influence_with_no_members_overreaches() {
        let mut w = World::new();
        let o = guild_with_power(&mut w, 0, 0.5);
        exert_influence(&mut w, o, "village");
        assert!(w.get_component::<OrganizedPower>(o).unwrap().capacity < 0.5, "acting alone should burn power");
    }

    #[test]
    fn test_one_on_one_builds_trust_by_depth_then_ends() {
        use crate::engine::ecs::System;
        use crate::game::relationships::{find_relationship, Relationship};
        let mut w = World::new();
        let organizer = w.create_entity();
        let (deep, shallow) = (w.create_entity(), w.create_entity());
        let strength = |w: &World, other| find_relationship(w, organizer, other)
            .and_then(|r| w.get_component::<Relationship>(r)).map(|r| r.strength).unwrap_or(0.0);

        w.add_component(organizer, OneOnOne { with: deep, depth: 1.0 });
        OneOnOneSystem.run(&mut w);
        assert!(w.get_component::<OneOnOne>(organizer).is_none(), "the meeting is over");
        OneOnOneSystem.run(&mut w); // nothing pending: no double count
        w.add_component(organizer, OneOnOne { with: shallow, depth: 0.2 });
        OneOnOneSystem.run(&mut w);

        assert!((strength(&w, deep) - 0.10).abs() < 1e-6);
        assert!(strength(&w, shallow) < strength(&w, deep), "a deeper 1-1 builds more trust");
    }

    #[test]
    fn test_dead_members_leave_the_org_and_its_influence() {
        use crate::engine::ecs::System;
        use crate::sim::illness::Health;
        let mut w = World::new();
        let o = w.create_entity();
        let leader = w.create_entity();
        w.add_component(o, Organization {
            name: "Guild".into(), members: vec![], leader,
            influence_pool: 0.0, resources: 0.0, charter: "".into(), successors: vec![] });
        let mut ms = vec![];
        for infl in [0.2, 0.3] {
            let c = w.create_entity();
            w.add_component(c, Citizen { influence: infl, authority: 0.0, ideology: "laborer".into(), allegiance: "none".into() });
            w.add_component(c, Health { condition: 0.9, immunity: 0.5 });
            recruit(&mut w, o, c, 0.8);
            ms.push(c);
        }
        InfluenceSystem.run(&mut w);
        assert!((w.get_component::<Organization>(o).unwrap().influence_pool - 0.5).abs() < 1e-6);
        w.get_component_mut::<Health>(ms[1]).unwrap().condition = 0.0;
        InfluenceSystem.run(&mut w);
        let org = w.get_component::<Organization>(o).unwrap();
        assert_eq!(org.members, vec![ms[0]]);
        assert!((org.influence_pool - 0.2).abs() < 1e-6, "a dead member lends no sway");

        // the windowed game removes Health on death and marks the villager Dead
        w.remove_component::<Health>(ms[0]);
        w.add_component(ms[0], crate::game::villager::Dead);
        InfluenceSystem.run(&mut w);
        assert!(w.get_component::<Organization>(o).unwrap().members.is_empty(), "a member marked Dead leaves too");
    }
}

/// The head falls and the next man stands: a decapitation pulls the
/// first named successor into the leader's seat - and a hardened org
/// answers the blow with a harder edge
pub fn decapitate(world: &mut World, org: Entity, _day: u64) -> Option<Entity> {
    world.get_component_mut::<Organization>(org).and_then(|o| {
        let n = o.successors.first().copied();
        if let Some(n) = n { o.leader = n; o.successors.remove(0); }
        n
    })
}

/// The boycott measured by the web it leaves: a defector's ties strip
/// in a dense village's hands, and barely pull in a thin one's. The
/// effectiveness is the average bond strength the remaining members
/// still hold - a thin web can't cut a man out.
pub fn insider_boycott(world: &mut World, org: Entity, defector: Entity) -> f32 {
    let members = world.get_component::<Organization>(org)
        .map(|o| o.members.clone()).unwrap_or_default();
    let dense = if members.is_empty() { 0.0 } else {
        let inside = members.iter().filter(|&&m|
            world.get_component::<crate::sim::psychology::Bond>(m)
                .map(|b| members.contains(&b.target)).unwrap_or(false)).count();
        inside as f32 / members.len() as f32
    };
    if let Some(b) = world.get_component_mut::<crate::sim::psychology::Bond>(defector) {
        b.security = (b.security - dense * 0.6).max(0.0);
    }
    dense
}

/// The oath shared at the rite binds faster than the room's talk: the
/// members' bonds to each other take a step up - the Bois Cauman's own
/// kind of joining
pub fn sacred_rite(world: &mut World, org: Entity) {
    let members = world.get_component::<Organization>(org)
        .map(|o| o.members.clone()).unwrap_or_default();
    for i in 0..members.len() {
        for j in (i + 1)..members.len() {
            let (m, o) = (members[i], members[j]);
            if let Some(b) = world.get_component_mut::<crate::sim::psychology::Bond>(m) {
                if b.target == o { b.security = (b.security + 0.2).min(1.0); }
            }
        }
    }
}
