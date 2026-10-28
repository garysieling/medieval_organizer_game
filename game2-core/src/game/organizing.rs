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

/// Conduct a 1-1 relational meeting (from NEA organizing guide): a real
/// conversation between organizer and citizen, which builds the trust it
/// reports, and reveals what's on the citizen's mind (village_concerns),
/// who they're close to (their relationships, by kind), and how much they
/// could lead (their influence)
pub fn one_on_one(world: &mut World, organizer: Entity, citizen: Entity) -> OneOnOneResult {
    use crate::game::relationships::RelKind;
    let before = crate::game::relationships::find_relationship(world, organizer, citizen)
        .and_then(|r| world.get_component::<Relationship>(r)).map(|r| r.strength).unwrap_or(0.0);
    let rel = record_interaction(world, organizer, Interaction {
        other: citizen, kind: InteractionKind::Conversation, intensity: 1.0,
    });
    let after = world.get_component::<Relationship>(rel).map(|r| r.strength).unwrap_or(before);

    // Their network: the people they know, by kind of tie (not counting the organizer)
    let mut kinds: Vec<(RelKind, usize)> = vec![];
    for r in world.query::<Relationship>() {
        let Some(r) = world.get_component::<Relationship>(r) else { continue };
        let other = if r.from == citizen { r.other } else if r.other == citizen { r.from } else { continue };
        if other == organizer || r.kind == RelKind::Stranger { continue; }
        match kinds.iter_mut().find(|(k, _)| *k == r.kind) {
            Some(k) => k.1 += 1,
            None => kinds.push((r.kind, 1)),
        }
    }
    let networks = kinds.into_iter()
        .map(|(k, n)| {
            let (one, many) = match k {
                RelKind::Family => ("relative", "relatives"),
                RelKind::Acquaintance => ("acquaintance", "acquaintances"),
                RelKind::Friend => ("friend", "friends"),
                RelKind::Rival => ("rival", "rivals"),
                RelKind::Lover => ("lover", "lovers"),
                RelKind::Mentor => ("mentor", "mentors"),
                RelKind::Apprentice => ("apprentice", "apprentices"),
                RelKind::Stranger => ("stranger", "strangers"),
            };
            format!("{} {}", n, if n == 1 { one } else { many })
        })
        .collect();

    OneOnOneResult {
        concerns: village_concerns(world, &[citizen]),
        networks,
        leadership_potential: world.get_component::<Citizen>(citizen).map(|c| c.influence * 1.5).unwrap_or(0.3).min(1.0),
        trust_gained: after - before,
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
    // Shared concerns emerge from what the attendees are living through
    let shared = village_concerns(world, attendees);
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

/// What's on the minds of these people: the two concerns most of them
/// share - going hungry, sickness, or what their trade cares about -
/// most widespread first (the dead have no say). With nothing pressing,
/// talk turns to the coming harvest.
pub fn village_concerns(world: &World, people: &[Entity]) -> Vec<String> {
    use crate::game::villager::Hunger;
    use crate::sim::illness::Health;
    let mut tally: Vec<(&'static str, usize)> = vec![];
    let mut raise = |c: &'static str| match tally.iter_mut().find(|(n, _)| *n == c) {
        Some(t) => t.1 += 1,
        None => tally.push((c, 1)),
    };
    for &p in people {
        if is_dead(world, p) { continue; }
        if world.get_component::<Hunger>(p).map(|h| h.value < 30.0).unwrap_or(false) { raise("the food running short"); }
        if world.get_component::<Health>(p).map(|h| h.condition < 0.8).unwrap_or(false) { raise("the sickness"); }
        if let Some(c) = world.get_component::<Citizen>(p) {
            raise(match c.ideology.as_str() {
                "laborer" => "fair wages",
                "merchant" => "safe roads",
                "clergy" => "the faith",
                "lord" => "lower rents",
                _ => "the coming harvest",
            });
        }
    }
    // Most widespread first; ties keep the order concerns were first raised
    tally.sort_by(|a, b| b.1.cmp(&a.1));
    let top: Vec<String> = tally.into_iter().take(2).map(|(c, _)| c.to_string()).collect();
    if top.is_empty() { vec!["the coming harvest".into()] } else { top }
}

/// How a concern comes up in conversation: what the player saw them do
/// (for the log) and what they actually said (for their speech bubble)
fn concern_speech(concern: &str) -> (&'static str, &'static str) {
    match concern {
        "the food running short" => ("complained the store is running low", "There's hardly anything left in the store."),
        "the sickness" => ("worried about the sickness going round", "Half the village is coughing. Who's next?"),
        "fair wages" => ("complained about wages", "A day's work hardly buys a day's bread."),
        "safe roads" => ("complained the roads aren't safe", "Nobody will trade with us if the roads stay like this."),
        "the faith" => ("talked about keeping the faith", "We need something to believe in, these days."),
        "lower rents" => ("complained about the rents", "The rents go up every season."),
        _ => ("talked about the coming harvest", "If the harvest is good, we'll get through the winter."),
    }
}

/// What `name` says when the player talks with them: one of the concerns
/// actually on their mind (village_concerns), worked through in turn as
/// the player talks with them again (`talks_before`). Returns the log line
/// ("Bram complained about wages") and their own words, for a speech bubble.
pub fn talk_response(world: &World, person: Entity, name: &str, talks_before: usize) -> (String, String) {
    let concerns = village_concerns(world, &[person]);
    let (did, said) = concern_speech(&concerns[talks_before % concerns.len()]);
    (format!("{} {}", name, did), said.to_string())
}

/// Harvest: the org's living members bring in the harvest together. Working
/// side by side bonds each pair of them (an Aid interaction at half
/// strength), and a shared effort of two or more is real organizing that
/// builds the org's people power (one person alone barely does). Returns
/// how many members took part.
pub fn harvest_together(world: &mut World, org: Entity) -> usize {
    let workers: Vec<Entity> = world.get_component::<Organization>(org)
        .map(|o| o.members.clone()).unwrap_or_default()
        .into_iter().filter(|&m| !is_dead(world, m)).collect();
    for (i, &a) in workers.iter().enumerate() {
        for &b in &workers[i + 1..] {
            record_interaction(world, a, Interaction { other: b, kind: InteractionKind::Aid, intensity: 0.5 });
        }
    }
    match workers.len() {
        0 => {}
        1 => organized_power::record_action(world, org, ActionQuality::Transactional),
        _ => organized_power::record_action(world, org, ActionQuality::Transformational),
    }
    workers.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_talk_response_says_what_they_actually_said() {
        use crate::game::villager::Hunger;
        let mut w = World::new();
        let v = w.create_entity();
        w.add_component(v, Citizen { influence: 0.2, authority: 0.0, ideology: "laborer".into(), allegiance: "none".into() });
        w.add_component(v, Hunger { value: 10.0 });
        let (log, said) = talk_response(&w, v, "Bram", 0);
        assert_eq!(log, "Bram complained the store is running low");
        assert!(!said.is_empty());
        // talk again and they move on to the next thing on their mind
        assert_eq!(talk_response(&w, v, "Bram", 1).0, "Bram complained about wages");
        assert_eq!(talk_response(&w, v, "Bram", 2).0, "Bram complained the store is running low");
    }

    #[test]
    fn test_harvesting_together_bonds_members_and_builds_power() {
        use crate::game::relationships::find_relationship;
        use crate::sim::organized_power::OrganizedPower;
        let mut w = World::new();
        let (org, _) = guild(&mut w);
        w.add_component(org, OrganizedPower::new());
        assert_eq!(harvest_together(&mut w, org), 0);
        let (a, b, c) = (w.create_entity(), w.create_entity(), w.create_entity());
        w.get_component_mut::<Organization>(org).unwrap().members = vec![a, b, c];
        assert_eq!(harvest_together(&mut w, org), 3);
        for (x, y) in [(a, b), (a, c), (b, c)] {
            let r = find_relationship(&w, x, y).expect("members who harvest together bond");
            assert!((w.get_component::<Relationship>(r).unwrap().strength - 0.06).abs() < 1e-6);
        }
        assert!(w.get_component::<OrganizedPower>(org).unwrap().capacity > 0.1, "a shared effort builds people power");
    }

    #[test]
    fn test_one_on_one_reveals_real_concerns_and_network_and_builds_trust() {
        use crate::game::relationships::RelKind;
        use crate::game::villager::Hunger;
        let mut w = World::new();
        let organizer = w.create_entity();
        let c = w.create_entity();
        w.add_component(c, Citizen { influence: 0.4, authority: 0.0, ideology: "merchant".into(), allegiance: "none".into() });
        w.add_component(c, Hunger { value: 10.0 });
        for (kind, n) in [(RelKind::Friend, 2), (RelKind::Rival, 1), (RelKind::Family, 2)] {
            for _ in 0..n {
                let other = w.create_entity();
                let r = w.create_entity();
                w.add_component(r, Relationship { from: c, other, kind, strength: 0.5, interactions: 6 });
            }
        }
        let r = one_on_one(&mut w, organizer, c);
        assert_eq!(r.concerns, vec!["the food running short".to_string(), "safe roads".to_string()]);
        assert_eq!(r.networks, vec!["2 friends".to_string(), "1 rival".to_string(), "2 relatives".to_string()]);
        assert!((r.trust_gained - 0.05).abs() < 1e-6, "the conversation really builds trust");
        assert!((r.leadership_potential - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_concerns_come_from_what_attendees_are_living_through() {
        use crate::game::villager::Hunger;
        use crate::sim::illness::Health;
        let mut w = World::new();
        let person = |w: &mut World, ideology: &str, hunger: f32, health: f32| {
            let e = w.create_entity();
            w.add_component(e, Citizen { influence: 0.1, authority: 0.0, ideology: ideology.into(), allegiance: "none".into() });
            w.add_component(e, Hunger { value: hunger });
            w.add_component(e, Health { condition: health, immunity: 0.5 });
            e
        };
        let fed: Vec<Entity> = (0..3).map(|_| person(&mut w, "laborer", 90.0, 0.9)).collect();
        assert_eq!(village_concerns(&w, &fed), vec!["fair wages".to_string()]);
        // A famine and a merchant: hunger is what everyone talks about
        let mut famine: Vec<Entity> = (0..3).map(|_| person(&mut w, "laborer", 10.0, 0.9)).collect();
        famine.push(person(&mut w, "merchant", 10.0, 0.9));
        assert_eq!(village_concerns(&w, &famine), vec!["the food running short".to_string(), "fair wages".to_string()]);
        assert_eq!(village_concerns(&w, &[]), vec!["the coming harvest".to_string()]);
    }
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
