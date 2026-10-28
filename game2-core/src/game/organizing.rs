//! 1-1 organizing meetings + house meetings.
//! From lit review: relational interview reveals concerns/networks/leadership -> recruit.

use crate::engine::ecs::{Entity, World};
use crate::game::relationships::{record_interaction, Interaction, InteractionKind, Relationship};
use crate::sim::community::{Citizen, Organization};
use crate::sim::organized_power::{self, ActionQuality};

/// What a 1-1 reveals about a citizen
#[derive(Debug)]
pub struct OneOnOneResult {
    pub concerns: Vec<String>,       // what they care about
    pub networks: Vec<String>,       // who they know
    pub leadership_potential: f32,   // 0-1
    pub trust_gained: f32,
}

/// Conduct a 1-1 relational meeting (from NEA organizing guide)
pub fn one_on_one(world: &World, _organizer: Entity, citizen: Entity) -> OneOnOneResult {
    let c = world.get_component::<Citizen>(citizen);
    OneOnOneResult {
        concerns: vec![format!("worried about {}", c.map(|x| x.ideology.as_str()).unwrap_or("work"))],
        networks: vec!["neighbors".into(), "workmates".into()],
        leadership_potential: c.map(|x| x.influence * 1.5).unwrap_or(0.3).min(1.0),
        trust_gained: 0.2,
    }
}

/// A house meeting: small group, shared agenda emerges
#[derive(Debug)]
pub struct HouseMeetingResult {
    pub shared_concerns: Vec<String>,
    pub formed_committee: bool,
    pub attendees_recruited: u32,
}

use crate::game::villager::is_dead; // health 0, or marked Dead by the windowed game

/// The org's leader, handing leadership on if they've died: the living
/// member with the most influence (the earliest-joined on a tie) takes
/// over. None if the org is gone or no one living is left to lead.
pub fn living_leader(world: &mut World, org: Entity) -> Option<Entity> {
    let (leader, members) = world.get_component::<Organization>(org).map(|o| (o.leader, o.members.clone()))?;
    if !is_dead(world, leader) { return Some(leader); }
    let successor = members.iter().copied()
        .filter(|&m| !is_dead(world, m))
        .map(|m| (m, world.get_component::<Citizen>(m).map(|c| c.influence).unwrap_or(0.0)))
        .fold(None, |best: Option<(Entity, f32)>, (m, infl)| match best {
            Some((_, b)) if b >= infl => best,
            _ => Some((m, infl)),
        })
        .map(|(m, _)| m)?;
    world.get_component_mut::<Organization>(org)?.leader = successor;
    Some(successor)
}

/// An organization's meeting hall: with somewhere to gather, house meetings
/// become shared meals, which build trust faster than talk alone
#[derive(Debug)]
pub struct MeetingHall;

/// Gold it costs to build a meeting hall
pub const HALL_COST: f32 = 40.0;

/// Marks a meeting hall a storm has damaged: until it's repaired, house
/// meetings there are plain talk again
#[derive(Debug)]
pub struct HallDamaged;

/// Gold it costs to repair a storm-damaged meeting hall
pub const HALL_REPAIR_COST: f32 = 15.0;

/// A storm strikes: the org's meeting hall, if it has one, is damaged.
/// Returns whether there was a standing hall to damage.
pub fn storm_strikes_hall(world: &mut World, org: Entity) -> bool {
    let standing = world.get_component::<MeetingHall>(org).is_some()
        && world.get_component::<HallDamaged>(org).is_none();
    if standing { world.add_component(org, HallDamaged); }
    standing
}

/// Run a house meeting with identified citizens. The meeting is a
/// conversation between the org's leader and each attendee; only attendees
/// whose relationship with the leader is already strong enough join. A
/// meeting where at least two members take part develops people and builds
/// the org's organized power; a thin one barely does. If the leader has
/// died, leadership passes on first (living_leader); with no one left to
/// lead, or for dead attendees, nothing happens.
pub fn house_meeting(world: &mut World, org: Entity, attendees: &[Entity]) -> HouseMeetingResult {
    // Shared concerns emerge from overlap
    let shared = vec!["fair wages".into(), "safe harvest".into()];
    let Some(leader) = living_leader(world, org) else {
        return HouseMeetingResult { shared_concerns: shared, formed_committee: false, attendees_recruited: 0 };
    };
    let world_has_hall = world.get_component::<MeetingHall>(org).is_some()
        && world.get_component::<HallDamaged>(org).is_none();
    // In a hall the meeting is a shared meal; otherwise it's just talk
    let kind = || if world_has_hall { InteractionKind::SharedMeal } else { InteractionKind::Conversation };
    let mut recruited = 0;
    let mut members_present = 0;
    for &a in attendees {
        if a == leader || is_dead(world, a) { continue; }
        let rel = record_interaction(world, leader, Interaction {
            other: a, kind: kind(), intensity: 1.0,
        });
        let trust = world.get_component::<Relationship>(rel).map(|r| r.strength).unwrap_or(0.0);
        if crate::sim::community::recruit(world, org, a, trust) {
            recruited += 1;
        }
        if world.get_component::<Organization>(org).map(|o| o.members.contains(&a)).unwrap_or(false) {
            members_present += 1;
        }
    }
    let quality = if members_present >= 2 { ActionQuality::Transformational } else { ActionQuality::Transactional };
    organized_power::record_action(world, org, quality);
    HouseMeetingResult {
        shared_concerns: shared,
        formed_committee: recruited >= 2,
        attendees_recruited: recruited,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::relationships::RelKind;

    fn guild(w: &mut World) -> (Entity, Entity) {
        let org = w.create_entity();
        let leader = w.create_entity();
        w.add_component(org, Organization {
            name: "Guild".into(), members: vec![], leader,
            influence_pool: 0.0, resources: 0.0, charter: "Wages".into(),
        });
        (org, leader)
    }

    #[test]
    fn test_house_meeting_does_not_recruit_strangers() {
        let mut w = World::new();
        let (org, _) = guild(&mut w);
        let strangers: Vec<Entity> = (0..3).map(|_| w.create_entity()).collect();
        let r = house_meeting(&mut w, org, &strangers);
        assert_eq!(r.attendees_recruited, 0);
        assert!(!r.formed_committee);
        assert!(w.get_component::<Organization>(org).unwrap().members.is_empty());
    }

    #[test]
    fn test_house_meeting_recruits_trusted_attendees() {
        let mut w = World::new();
        let (org, leader) = guild(&mut w);
        let friends: Vec<Entity> = (0..2).map(|_| w.create_entity()).collect();
        for &f in &friends {
            let rel = w.create_entity();
            w.add_component(rel, Relationship {
                from: leader, other: f, kind: RelKind::Friend, strength: 0.6, interactions: 5,
            });
        }
        let stranger = w.create_entity();
        let r = house_meeting(&mut w, org, &[friends[0], friends[1], stranger]);
        assert_eq!(r.attendees_recruited, 2);
        assert!(r.formed_committee);
        assert!(!w.get_component::<Organization>(org).unwrap().members.contains(&stranger));
    }

    #[test]
    fn test_house_meeting_with_members_builds_more_power_than_a_thin_one() {
        use crate::sim::organized_power::OrganizedPower;
        let mut w = World::new();
        let (strong, _) = guild(&mut w);
        let (thin, _) = guild(&mut w);
        for org in [strong, thin] {
            w.add_component(org, OrganizedPower::new());
        }
        let members: Vec<Entity> = (0..2).map(|_| w.create_entity()).collect();
        for &m in &members {
            crate::sim::community::recruit(&mut w, strong, m, 0.9);
        }
        house_meeting(&mut w, strong, &members);
        let strangers: Vec<Entity> = (0..2).map(|_| w.create_entity()).collect();
        house_meeting(&mut w, thin, &strangers);
        let cap = |o| w.get_component::<OrganizedPower>(o).unwrap().capacity;
        assert!(cap(strong) > cap(thin), "meeting with members should develop more power ({} vs {})", cap(strong), cap(thin));
        assert!(cap(thin) > 0.0, "even a thin meeting is some use of the muscle");
    }

    #[test]
    fn test_leadership_passes_to_most_influential_living_member() {
        use crate::sim::illness::Health;
        let mut w = World::new();
        let (org, leader) = guild(&mut w);
        w.add_component(leader, Health { condition: 0.0, immunity: 0.5 });
        let member = |w: &mut World, infl: f32, alive: bool| {
            let m = w.create_entity();
            w.add_component(m, Citizen { influence: infl, authority: 0.0, ideology: "laborer".into(), allegiance: "none".into() });
            w.add_component(m, Health { condition: if alive { 0.9 } else { 0.0 }, immunity: 0.5 });
            w.get_component_mut::<Organization>(org).unwrap().members.push(m);
            m
        };
        let _quiet = member(&mut w, 0.2, true);
        let _dead_star = member(&mut w, 0.9, false);
        let voice = member(&mut w, 0.5, true);
        let r = house_meeting(&mut w, org, &[]);
        assert_eq!(w.get_component::<Organization>(org).unwrap().leader, voice);
        assert_eq!(r.attendees_recruited, 0);
    }

    #[test]
    fn test_no_meeting_without_a_living_leader() {
        use crate::sim::illness::Health;
        let mut w = World::new();
        let (org, leader) = guild(&mut w);
        w.add_component(leader, Health { condition: 0.0, immunity: 0.5 });
        let stranger = w.create_entity();
        house_meeting(&mut w, org, &[stranger]);
        assert!(crate::game::relationships::find_relationship(&w, leader, stranger).is_none(),
            "a dead leader holds no conversations");
    }

    #[test]
    fn test_leader_marked_dead_hands_on_leadership() {
        // the windowed game removes Health on death and marks the villager Dead
        let mut w = World::new();
        let (org, leader) = guild(&mut w);
        w.add_component(leader, crate::game::villager::Dead);
        let heir = w.create_entity();
        w.add_component(heir, Citizen { influence: 0.3, authority: 0.0, ideology: "laborer".into(), allegiance: "none".into() });
        w.get_component_mut::<Organization>(org).unwrap().members.push(heir);
        assert_eq!(living_leader(&mut w, org), Some(heir));
    }

    #[test]
    fn test_a_meeting_hall_builds_trust_faster() {
        let meetings_to_win = |hall: bool| {
            let mut w = World::new();
            let (org, _) = guild(&mut w);
            if hall { w.add_component(org, MeetingHall); }
            let v = w.create_entity();
            (1..=20).find(|_| house_meeting(&mut w, org, &[v]).attendees_recruited == 1).unwrap()
        };
        let (with_hall, without) = (meetings_to_win(true), meetings_to_win(false));
        assert!(with_hall < without, "hall {} vs no hall {} meetings", with_hall, without);
        assert_eq!(without, 10);
        assert_eq!(with_hall, 7);
    }

    #[test]
    fn test_a_storm_damaged_hall_hosts_plain_talk_until_repaired() {
        let mut w = World::new();
        let (org, leader) = guild(&mut w);
        assert!(!storm_strikes_hall(&mut w, org), "no hall to damage");
        w.add_component(org, MeetingHall);
        assert!(storm_strikes_hall(&mut w, org));
        assert!(!storm_strikes_hall(&mut w, org), "already damaged");
        let v = w.create_entity();
        house_meeting(&mut w, org, &[v]);
        let r = crate::game::relationships::find_relationship(&w, leader, v).unwrap();
        assert!((w.get_component::<Relationship>(r).unwrap().strength - 0.05).abs() < 1e-6, "plain talk, not a shared meal");
    }
}
