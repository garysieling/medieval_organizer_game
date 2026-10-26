//! Community organizing: influence networks, 1-1s, house meetings, power.
//! From backlog: "community organizers vs fascists/billionaires" central theme.
//! Player starts with influence without authority, builds organization.

use crate::engine::ecs::{Entity, World};

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

/// System: organizations accumulate influence from members
pub struct InfluenceSystem;
impl crate::engine::ecs::System for InfluenceSystem {
    fn run(&mut self, world: &mut World) {
        for o in world.query::<Organization>() {
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

/// System: 1-1s deepen relationships
pub struct OneOnOneSystem;
impl crate::engine::ecs::System for OneOnOneSystem {
    fn run(&mut self, world: &mut World) {
        for e in world.query::<OneOnOne>() {
            // Deep conversations build trust (emergent relationship)
            let _ = e;
        }
    }
}

/// Recruit a citizen into an organization via relationship
pub const RECRUIT_MIN_TRUST: f32 = 0.5;

pub fn recruit(world: &mut World, org: Entity, citizen: Entity, rel_quality: f32) -> bool {
    if rel_quality < RECRUIT_MIN_TRUST { return false; } // need trust to recruit
    if let Some(o) = world.get_component_mut::<Organization>(org) {
        if o.members.contains(&citizen) { return false; }
        o.members.push(citizen);
        return true;
    }
    false
}

/// Player exercises influence via an organization
pub fn exert_influence(world: &mut World, org: Entity, _target: &str) -> f32 {
    if let Some(o) = world.get_component::<Organization>(org) {
        return o.influence_pool; // sway toward target
    }
    0.0
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
            influence_pool: 0.0, resources: 100.0, charter: "Trade".into(),
        });
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
            influence_pool: 0.0, resources: 100.0, charter: "Trade".into(),
        });
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
}
