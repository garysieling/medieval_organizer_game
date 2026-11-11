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
    // a real conversation, like a Talk: the organizer's bond counts it
    crate::game::player::note_conversation(world, organizer, citizen);
    let after = world.get_component::<Relationship>(rel).map(|r| r.strength).unwrap_or(before);
    // running a real 1-1 is practice at the craft
    crate::game::org_skills::practice(world, organizer, crate::game::org_skills::OrgSkill::OneOnOne, crate::game::org_skills::ONE_ON_ONE_PRACTICE);

    // Their network: the people they know, by kind of tie (not counting the organizer)
    let mut kinds: Vec<(RelKind, usize)> = vec![];
    for r in crate::game::relationships::bonds_of(world, citizen) {
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
    // the head's fall costs the org - deeper anchors keep more of the pool
    let factor = anchor_factor(world, org);
    if let Some(o) = world.get_component_mut::<Organization>(org) {
        o.influence_pool *= 1.0 - 0.4 * (1.0 - factor);
        o.leader = successor;
    }
    Some(successor)
}

/// The institutional anchor: the deeper an organization is rooted, the
/// better it weathers a leader's loss and an idle season's atrophy -
/// the hall's the shallowest root, a congregation tie deeper, a guild
/// charter deepest
#[derive(Debug, Clone, Copy)]
pub enum AnchorTier { Hall, Congregation, Guild }

/// The anchor an organization carries - deeper tiers insure better
#[derive(Debug)]
pub struct Anchor(pub AnchorTier);

/// How much of the storm an org's anchor weathers: the hall a quarter,
/// a congregation tie half, a guild charter three quarters
pub fn anchor_factor(world: &World, org: Entity) -> f32 {
    if let Some(a) = world.get_component::<Anchor>(org) {
        return match a.0 {
            AnchorTier::Hall => 0.25,
            AnchorTier::Congregation => 0.5,
            AnchorTier::Guild => 0.75,
        };
    }
    // no formal anchor: an undamaged hall still roots it a little
    if world.get_component::<MeetingHall>(org).is_some()
        && world.get_component::<HallDamaged>(org).is_none() { 0.25 } else { 0.0 }
}

/// The mentor's work: an org spends its actions growing a member's
/// influence - the successor the pipeline prefers when the chair empties
#[allow(dead_code)]
pub fn mentor(world: &mut World, member: Entity, boost: f32) {
    if let Some(c) = world.get_component_mut::<Citizen>(member) {
        c.influence = (c.influence + boost).min(1.0);
    }
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
        // they spoke at the meeting: the leader's bond with them grows a
        // talk's step, so a seat that hosted counts them as spoken to
        crate::game::player::note_conversation(world, leader, a);
        let trust = world.get_component::<Relationship>(rel).map(|r| r.strength).unwrap_or(0.0);
        if crate::sim::community::recruit(world, org, a, trust) {
            recruited += 1;
        }
        if world.get_component::<Organization>(org).map(|o| o.members.contains(&a)).unwrap_or(false) {
            members_present += 1;
        }
    }
    // everyone who came learns meeting craft a little; running it teaches
    // the leader agitation - working the room
    crate::game::org_skills::practice(world, leader, crate::game::org_skills::OrgSkill::Agitation, crate::game::org_skills::MEETING_PRACTICE);
    for &a in attendees {
        if a != leader && !is_dead(world, a) {
            crate::game::org_skills::practice(world, a, crate::game::org_skills::OrgSkill::HouseMeeting, crate::game::org_skills::MEETING_PRACTICE);
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

/// The ways people put a concern when it comes up at a meeting
fn meeting_sayings(concern: &str) -> [&'static str; 3] {
    match concern {
        "the food running short" => ["The store won't last the month.", "My children go to bed hungry.", "We need to share what grain there is."],
        "the sickness" => ["Half the village is coughing.", "Who'll work the fields if we're all sick?", "We should look after the sick together."],
        "fair wages" => ["A day's work hardly buys a day's bread.", "The miller takes more every season.", "If we asked together, they'd have to listen."],
        "safe roads" => ["The roads aren't safe to travel.", "No trader comes this way any more.", "We could watch the roads in turns."],
        "the faith" => ["We need something to hold on to.", "The church is the only thing that brings us together.", "Let's keep faith with each other."],
        "lower rents" => ["The rents go up every season.", "I can't pay what the lord asks.", "One of us alone can't argue the rent."],
        _ => ["If the harvest is good, we'll get through.", "We'll need every hand at harvest.", "Let's bring the harvest in together."],
    }
}

/// What `name` says at a house meeting on round `round`: one of their own
/// concerns, in words of their own (each person puts it differently)
pub fn meeting_remark(world: &World, person: Entity, name: &str, round: usize) -> String {
    let concerns = village_concerns(world, &[person]);
    let concern = &concerns[round % concerns.len()];
    let voice = name.bytes().map(|b| b as usize).sum::<usize>();
    meeting_sayings(concern)[(voice + round) % 3].to_string()
}

/// Small talk, for someone who has already told you what's on their mind
const SMALL_TALK: [(&str, &str); 5] = [
    ("chatted about the weather", "Looks like rain again, doesn't it?"),
    ("passed on some village gossip", "Did you hear who was seen down by the mill?"),
    ("asked how you were keeping", "Good to see you again."),
    ("talked about their family", "The little ones keep me on my toes."),
    ("swapped stories by the well", "Remember the winter the river froze?"),
];

/// A voice with a register: oaths, by-your-leaves, and the kind of
/// flourished word that makes a passing remark a line worth hearing
const REGISTERS: [&str; 8] = [
    "By Saint Crispin, the hedges are full of it.",
    "Thee's a good neighbor - there's my hand on it.",
    "A pox on this weather, and that's God's truth.",
    "I swear by the well it's the driest year in memory.",
    "As I live and breathe, the miller's a knave and no mistake.",
    "Good morrow to thee - the Lord keep thy purse fat.",
    "There's a tale for thee, if thy ears'll take it.",
    "Marry, the market's dear and the purse is thin.",
];

#[cfg(test)]
mod voice_tests {
    use super::*;

    #[test]
    fn test_a_voice_with_a_flourish_on_it() {
        // every fourth remark's a register line; the rest's small talk
        assert!(passing_remark(0).len() > 5);
        let flavored = passing_remark(3);
        assert!(flavored.contains("By") || flavored.contains("thee") || flavored.contains("pox")
            || flavored.contains("Marry") || flavored.contains("swear") || flavored.contains("morrow")
            || flavored.contains("neighbor"), "a word with a flourish on it: {}", flavored);
        // and the gossip reaches for an insult
        assert!(gossip_insult(0).contains("knave") || gossip_insult(0).len() > 8,
            "the talk turns flourished");
    }
}

/// The flourish aimed at a rung of the ladder: the words a gossip
/// reaches for when the talk turns on someone
const INSULTS: [&str; 5] = [
    "a knave and no mistake",
    "proud as a cock on a midden",
    "all mouth and no measure",
    "light-fingered as a weasel",
    "born for the stocks, that one",
];

/// What the gossip reaches for when the talk turns - a flourished
/// insult, picked by `n`
pub fn gossip_insult(n: usize) -> &'static str {
    INSULTS[n % INSULTS.len()]
}

/// Something said in passing, when people meet: one of the small-talk
/// lines - and now and then a word with a flourish on it - picked by `n`
pub fn passing_remark(n: usize) -> &'static str {
    if n % 4 == 3 { REGISTERS[n % REGISTERS.len()] } else { SMALL_TALK[n % SMALL_TALK.len()].1 }
}

/// What someone who knows you says as you pass: a hello in their own
/// voice, `{name}` being who they greet - picked by `n`
const GREETINGS: [&str; 8] = [
    "Well met, {name} - good to see thee.",
    "{name}! Small world, isn't it?",
    "Ah, {name} - out and about?",
    "{name}, there's a fine day for walking.",
    "There's {name} - I was saying only this morning...",
    "Hello, {name} - the village feels warmer for it.",
    "{name}! We haven't crossed paths in an age.",
    "God save thee, {name} - and thy purse.",
];

/// The hello a friend gives in passing
pub fn passing_greeting(n: usize, name: &str) -> String {
    GREETINGS[n % GREETINGS.len()].replace("{name}", name)
}

/// What `name` says when the player talks with them (`talks_before` is how
/// many times they've talked already). First the concerns actually on
/// their mind (village_concerns), one per talk; once they've said those,
/// small talk - though going hungry or sick still comes up every other
/// time. Returns the log line ("Bram complained about wages") and their
/// own words, for a speech bubble.
pub fn talk_response(world: &World, person: Entity, name: &str, talks_before: usize) -> (String, String) {
    let concerns = village_concerns(world, &[person]);
    let pressing = concerns.iter().find(|c| *c == "the food running short" || *c == "the sickness");
    let (did, said) = match (concerns.get(talks_before), pressing) {
        (Some(c), _) => concern_speech(c),
        (None, Some(c)) if talks_before % 2 == 0 => concern_speech(c),
        _ => SMALL_TALK[talks_before % SMALL_TALK.len()],
    };
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
        .into_iter().filter(|&m| !is_dead(world, m)
            && !crate::game::dungeon::unavailable(world, m)).collect();
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
        // after that it's small talk, though hunger keeps coming back up
        assert_eq!(talk_response(&w, v, "Bram", 2).0, "Bram complained the store is running low");
        let (chat, _) = talk_response(&w, v, "Bram", 3);
        assert!(SMALL_TALK.iter().any(|(d, _)| chat == format!("Bram {}", d)), "{}", chat);
        // someone fed and well just makes small talk once they've said their piece
        w.add_component(v, Hunger { value: 90.0 });
        for n in 1..6 {
            let (chat, said) = talk_response(&w, v, "Bram", n);
            assert!(SMALL_TALK.iter().any(|(d, s)| chat == format!("Bram {}", d) && said == *s), "{}", chat);
        }
    }

    #[test]
    fn test_people_put_the_same_concern_their_own_way_at_a_meeting() {
        let mut w = World::new();
        let (a, b) = (w.create_entity(), w.create_entity());
        for v in [a, b] { w.add_component(v, Citizen { influence: 0.2, authority: 0.0, ideology: "laborer".into(), allegiance: "none".into() }); }
        let (ra, rb) = (meeting_remark(&w, a, "Eliza", 0), meeting_remark(&w, b, "Bram", 0));
        assert_ne!(ra, rb, "two laborers shouldn't say the very same thing");
        assert!(meeting_sayings("fair wages").contains(&ra.as_str()));
        assert_ne!(meeting_remark(&w, a, "Eliza", 0), meeting_remark(&w, a, "Eliza", 1), "nor repeat themselves");
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
            assert!((w.get_component::<Relationship>(r).unwrap().strength - 0.20).abs() < 1e-6);
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
        assert!((r.trust_gained - 0.10).abs() < 1e-6, "the conversation really builds trust");
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
            influence_pool: 0.0, resources: 0.0, charter: "Wages".into(), successors: vec![] });
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
        assert_eq!(without, 5);
        assert_eq!(with_hall, 3);
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
        assert!((w.get_component::<Relationship>(r).unwrap().strength - 0.10).abs() < 1e-6, "plain talk, not a shared meal");
    }
}

/// How well the organizer knows a person: a ladder you climb by talking
/// and learning what they care about - stranger, acquaintance, known
/// (three pieces of their self-interest), follower (a real bond), and
/// co-leader (a follower who has taken on a commitment).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RelStage {
    Stranger,
    Acquaintance,
    Known,
    Follower,
    CoLeader,
}

/// The bond strength that makes someone a follower: the same trust the
/// org asks before it counts them
pub const FOLLOWER_TIE: f32 = crate::sim::community::RECRUIT_MIN_TRUST;
/// Pieces of a person's self-interest that make them "known" to you
pub const KNOWN_PIECES: usize = 3;
/// How far from the church "the village" reaches for an introduction's
/// fallback ear - the country settlements stand well outside it
const VILLAGE_EAR_REACH: f32 = 40.0;

/// Errands run for a person, kept on the bond with you: a job carried
/// through tells you who they are as surely as their secrets do - one
/// deed seen to reads as knowing them
#[derive(Debug, Clone, Copy, Default)]
pub struct Deeds { pub n: u32 }

/// "I'll come find you": when an introduction goes down well the person
/// you asked owes the organizer a return visit - the day loop sends
/// them over once `after` has come, and the note's spent only when the
/// greeting actually lands
#[derive(Debug, Clone, Copy)]
pub struct RoundBack {
    /// Who they said they'd come back to
    pub to: Entity,
    /// The first day the visit may start (ticks are days)
    pub after: u64,
    /// Empty doorsteps found so far - they don't chase forever
    pub misses: u8,
}

/// Where `organizer` stands with `who`, read off the world. A stage is
/// derived, not stored: bonds decay, so a follower who drifts is only a
/// follower while the tie holds.
pub fn stage_of(world: &World, organizer: Entity, who: Entity) -> RelStage {
    use crate::game::relationships::{find_relationship, Relationship};
    let Some(bond) = find_relationship(world, organizer, who) else { return RelStage::Stranger };
    let Some((strength, interactions)) = world.get_component::<Relationship>(bond)
        .map(|r| (r.strength, r.interactions)) else { return RelStage::Stranger };
    if interactions == 0 { return RelStage::Stranger; }
    if strength >= FOLLOWER_TIE {
        // a follower who's taken on work - an org membership, a campaign
        // seat, or a promise kept - has stepped up to co-lead
        let committed = world.query::<Organization>().iter().any(|&o|
            world.get_component::<Organization>(o)
                .map(|oo| oo.leader == organizer && oo.members.contains(&who)).unwrap_or(false))
            || world.get_component::<MeetingPlan>(organizer)
                .map(|p| p.holders().contains(&who)).unwrap_or(false)
            || world.query::<crate::game::campaign::Campaign>().iter().any(|&c|
                world.get_component::<crate::game::campaign::Campaign>(c)
                    .map(|cc| cc.seats.iter().any(|s| s.holder == who)).unwrap_or(false));
        return if committed { RelStage::CoLeader } else { RelStage::Follower };
    }
    let pieces = world.get_component::<crate::game::desires::Known>(who)
        .map(|k| k.revealed.len()).unwrap_or(0);
    // secrets heard, or deeds done for them - either makes them known
    let deeds = world.get_component::<Deeds>(bond).map(|d| d.n).unwrap_or(0);
    if pieces >= KNOWN_PIECES || deeds > 0 { RelStage::Known } else { RelStage::Acquaintance }
}

/// The stage word, for the inspector and the tone of a talk
pub fn stage_label(s: RelStage) -> &'static str {
    match s {
        RelStage::Stranger => "a stranger",
        RelStage::Acquaintance => "an acquaintance",
        RelStage::Known => "you know them",
        RelStage::Follower => "a follower",
        RelStage::CoLeader => "a co-leader",
    }
}

/// What a meeting needs before it can happen, gathered one ask at a
/// time. A promise lives on the org's plan; the more that come true,
/// the better the meeting runs.
#[derive(Debug, Default)]
pub struct MeetingPlan {
    /// Who said we could meet at their house, and the house they meant
    pub venue: Option<(Entity, Entity)>,
    /// Who said they'd bring something for the table
    pub snacks: Option<Entity>,
    /// Who said they'd keep the notes
    pub notes: Option<Entity>,
    /// Who said they'd open us with a word
    pub opener: Option<Entity>,
    /// Who said they'd give us a song
    pub song: Option<Entity>,
    /// Who said they'd close us out
    pub closer: Option<Entity>,
    /// Who said they'd come
    pub coming: Vec<Entity>,
    /// Who said they'd bring their friends
    pub spreaders: Vec<Entity>,
}

impl MeetingPlan {
    /// Everyone holding a promise on the plan
    pub fn holders(&self) -> Vec<Entity> {
        let mut v: Vec<Entity> = [self.venue.map(|(h, _)| h), self.snacks, self.notes,
            self.opener, self.song, self.closer].into_iter().flatten().collect();
        v.extend(self.coming.iter().copied());
        v.extend(self.spreaders.iter().copied());
        v
    }
    /// Promises that make the room warmer: everything past the venue and
    /// the bare attendance a meeting needs at all
    pub fn roles_filled(&self) -> usize {
        [self.snacks, self.notes, self.opener, self.song, self.closer]
            .iter().filter(|s| s.is_some()).count()
    }
    /// Can a meeting be called? Someone who'll come - an invite, a
    /// promised part, a word that they'll bring friends. A lent house
    /// makes it a house meeting, but folk can gather at the square
    /// without one (#1065)
    pub fn ready(&self) -> bool {
        !self.coming.is_empty() || !self.spreaders.is_empty() || self.roles_filled() > 0
    }
    /// What's still missing, for the button's reason
    pub fn missing(&self) -> &'static str {
        "no one's said they'll come"
    }
}

/// Get the org's meeting plan, making it if needed
pub fn plan_for<'a>(world: &'a mut World, org: Entity) -> &'a mut MeetingPlan {
    if world.get_component::<MeetingPlan>(org).is_none() {
        world.add_component(org, MeetingPlan::default());
    }
    world.get_component_mut::<MeetingPlan>(org).unwrap()
}

/// A proposition the organizer can put to someone mid-talk
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PropKind {
    /// "Will you come along?"
    ComeAlong,
    /// "Could we meet at your house?"
    LendVenue,
    /// "Could you bring something for the table?"
    BringSnacks,
    /// "Would you keep the notes of what we settle?"
    TakeNotes,
    /// "Would you open us with a word?"
    OpenWithWord,
    /// "Would you give us a song?"
    GiveSong,
    /// "Would you see us out with a word?"
    CloseOut,
    /// "Could you bring your friends along?"
    InviteFriends,
    /// "Is there someone I should know?"
    Introduce,
}

/// The ask as the player sees it in the talk options
pub fn prop_label(p: PropKind) -> &'static str {
    match p {
        PropKind::ComeAlong => "Ask them to come to a meeting",
        PropKind::LendVenue => "Ask to meet at their house",
        PropKind::BringSnacks => "Ask them to bring something for the table",
        PropKind::TakeNotes => "Ask them to keep the notes",
        PropKind::OpenWithWord => "Ask them to open with a word",
        PropKind::GiveSong => "Ask them for a song",
        PropKind::CloseOut => "Ask them to close us out",
        PropKind::InviteFriends => "Ask them to bring their friends",
        PropKind::Introduce => "Ask who you should talk to next",
    }
}

/// Which asks are on the table: those you haven't already secured, plus
/// the always-on asks (come along, an introduction). Everything needs at
/// least one piece of their self-interest heard - you can't put a claim
/// on a stranger.
pub fn props_on_offer(world: &World, org: Entity, person: Entity) -> Vec<PropKind> {
    let heard = world.get_component::<crate::game::desires::Known>(person)
        .map(|k| !k.revealed.is_empty()).unwrap_or(false);
    if !heard { return vec![]; }
    let mut out = vec![PropKind::ComeAlong, PropKind::Introduce];
    let empty = MeetingPlan::default();
    let plan = world.get_component::<MeetingPlan>(org).unwrap_or(&empty);
    // the venue ask's free of the slot check - the house is a place,
    // not a job, and the host's usually at the table anyway. The five
    // roles are one job apiece (#1062): once they've promised a part
    // of the meeting, its other asks go to someone else - the work
    // spreads around the room
    if plan.venue.is_none() && crate::game::household::home_of(world, person).is_some() { out.push(PropKind::LendVenue); }
    let holds_a_role = [plan.snacks, plan.notes, plan.opener, plan.song, plan.closer]
        .contains(&Some(person));
    if !holds_a_role {
        if plan.snacks.is_none() { out.push(PropKind::BringSnacks); }
        if plan.notes.is_none() { out.push(PropKind::TakeNotes); }
        if plan.opener.is_none() { out.push(PropKind::OpenWithWord); }
        if plan.song.is_none() { out.push(PropKind::GiveSong); }
        if plan.closer.is_none() { out.push(PropKind::CloseOut); }
    }
    if !plan.spreaders.contains(&person) { out.push(PropKind::InviteFriends); }
    out
}

/// Whether the ask suits what you know of them: a pious soul opens, a
/// singer sings, a lonely heart gathers people. `r` is a known piece of
/// their self-interest.
fn ask_suits(p: PropKind, r: &crate::game::desires::Reveal) -> bool {
    use crate::game::desires::{Problem, Reveal, Want};
    match p {
        PropKind::OpenWithWord => matches!(r, Reveal::Want(Want::Faith)),
        PropKind::GiveSong => matches!(r, Reveal::Want(Want::Music | Want::Instrument)),
        PropKind::InviteFriends => matches!(r, Reveal::Want(Want::Love)),
        PropKind::BringSnacks => matches!(r, Reveal::Want(Want::GoodFood) | Reveal::Problem(Problem::LongWinter)),
        PropKind::LendVenue => matches!(r, Reveal::Want(Want::Shelter) | Reveal::Problem(Problem::NoHome)),
        PropKind::ComeAlong => matches!(r, Reveal::Problem(_) | Reveal::Want(Want::Love)),
        PropKind::TakeNotes | PropKind::CloseOut | PropKind::Introduce => false,
    }
}

/// The chance a proposition lands: the stage they hold with you, lifted
/// when the ask touches something you know they want, and quietly
/// helped along by how practiced the organizer is at a 1-1.
pub fn yes_chance(world: &World, organizer: Entity, person: Entity, p: PropKind) -> f32 {
    let stage = stage_of(world, organizer, person);
    let base = match stage {
        RelStage::Stranger => 0.05,
        RelStage::Acquaintance => 0.45,
        RelStage::Known => 0.65,
        RelStage::Follower | RelStage::CoLeader => 0.9,
    };
    let suited = world.get_component::<crate::game::desires::Known>(person)
        .map(|k| k.revealed.iter().any(|r| ask_suits(p, r))).unwrap_or(false);
    let skill = world.get_component::<crate::game::org_skills::Organizing>(organizer)
        .map(|o| o.one_on_one).unwrap_or(0.0);
    (base + if suited { 0.2 } else { 0.0 } + skill * 0.15).min(0.98)
}

/// Put the proposition: the yes records the promise on the org's plan
/// (or warms a new contact for Introduce); the no costs a small chill.
/// `roll` is 0..1. Returns the log line, and for Introduce the friend
/// they named - the one the camera's wanted on.
pub fn propose(world: &mut World, organizer: Entity, org: Entity, person: Entity, p: PropKind, name: &str, roll: f32, tick: u64) -> (String, Option<Entity>) {
    use crate::game::relationships::{record_interaction, Interaction, InteractionKind};
    if yes_chance(world, organizer, person, p) <= roll {
        record_interaction(world, organizer, Interaction {
            other: person, kind: InteractionKind::Conflict, intensity: 0.1 });
        return (format!("{} won't hear it - not yet.", name), None);
    }
    // the ask itself is practice at the craft
    crate::game::org_skills::practice(world, organizer, crate::game::org_skills::OrgSkill::OneOnOne, 0.01);
    let home = if p == PropKind::LendVenue { crate::game::household::home_of(world, person) } else { None };
    let plan = plan_for(world, org);
    let msg = match p {
        PropKind::ComeAlong => {
            if !plan.coming.contains(&person) { plan.coming.push(person); }
            format!("{} says they'll come.", name)
        }
        PropKind::LendVenue => {
            if let Some(h) = home { plan.venue = Some((person, h)); }
            format!("{} says you can meet at their house.", name)
        }
        PropKind::BringSnacks => { plan.snacks = Some(person); format!("{} will bring something for the table.", name) }
        PropKind::TakeNotes => { plan.notes = Some(person); format!("{} will keep the notes.", name) }
        PropKind::OpenWithWord => { plan.opener = Some(person); format!("{} will open us with a word.", name) }
        PropKind::GiveSong => { plan.song = Some(person); format!("{} will give us a song.", name) }
        PropKind::CloseOut => { plan.closer = Some(person); format!("{} will see us out.", name) }
        PropKind::InviteFriends => {
            if !plan.spreaders.contains(&person) { plan.spreaders.push(person); }
            format!("{} will bring their friends.", name)
        }
        PropKind::Introduce => return introduce(world, organizer, person, name, tick),
    };
    // a yes is a small warmth too
    record_interaction(world, organizer, Interaction {
        other: person, kind: InteractionKind::Aid, intensity: 0.2 });
    (msg, None)
}

/// "Who should I talk to?": the strongest of their bonds that shares
/// something with them - the same trade, or a trouble you've heard them
/// voice. The word of a friend opens a door: the named person starts a
/// little warm to you.
fn introduce(world: &mut World, organizer: Entity, person: Entity, name: &str, tick: u64) -> (String, Option<Entity>) {
    use crate::game::relationships::{bonds_of, record_interaction, Interaction, InteractionKind, Relationship};
    use crate::game::desires::{issue_tag, unrevealed, Known};
    // a word well spent comes back around: they said they'd think on it,
    // and come find you about it
    world.add_component(person, RoundBack { to: organizer, after: tick + 1, misses: 0 });
    let voc = world.get_component::<crate::game::vocation::Craft>(person).map(|c| c.vocation);
    let their_tags: Vec<String> = world.get_component::<Known>(person)
        .map(|k| k.revealed.iter().map(|r| issue_tag(world, person, r)).collect()).unwrap_or_default();
    // candidates: who they know, strongest first, sharing something
    let mut cands: Vec<(f32, Entity)> = bonds_of(world, person).iter().filter_map(|&r| {
        let rel = world.get_component::<Relationship>(r)?;
        let other = if rel.from == person { rel.other } else { rel.from };
        if other == organizer || crate::game::villager::is_dead(world, other) { return None; }
        let same_trade = voc.is_some() && world.get_component::<crate::game::vocation::Craft>(other)
            .map(|c| Some(c.vocation) == voc).unwrap_or(false);
        let mut their_reveals = world.get_component::<Known>(other)
            .map(|k| k.revealed.clone()).unwrap_or_default();
        their_reveals.extend(unrevealed(world, other, tick));
        let shared_trouble = their_reveals.iter()
            .any(|r| their_tags.contains(&issue_tag(world, other, r)));
        (same_trade || shared_trouble).then_some((rel.strength, other))
    }).collect();
    cands.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    match cands.first() {
        Some(&(_, friend)) => {
            // a friend's word opens a door: a little warmth to start on
            record_interaction(world, organizer, Interaction {
                other: friend, kind: InteractionKind::Conversation, intensity: 0.6 });
            (format!("{} names a friend - someone you should know. A word from {} opens the door.", name, name), Some(friend))
        }
        None => match someone_to_send_to(world, organizer, person) {
            // no friend comes to mind, but folk know where the village's
            // ear is - and they'd send you there
            Some((p, office)) => {
                record_interaction(world, organizer, Interaction {
                    other: p, kind: InteractionKind::Conversation, intensity: 0.6 });
                (format!("{} can't think of anyone - but \"go and tell the {} your piece, {}.\"", name, office, name), Some(p))
            }
            None => (format!("{} can't think of anyone to send you to.", name), None),
        },
    }
}

/// The village's open ears, for when a friend can't be named: the priest
/// first - he hears everything - then the heart-helper, who knows every
/// heart, then the collector, who counts every soul. "The village" is
/// measured from its church: the country settlements are deliberately
/// far off, and their chaplains aren't the parish ear
fn someone_to_send_to(world: &World, organizer: Entity, person: Entity) -> Option<(Entity, &'static str)> {
    let church = world.query::<crate::game::household::Building>().iter().copied()
        .filter(|&b| world.get_component::<crate::game::household::Building>(b)
            .map(|bb| bb.kind == crate::game::household::BuildingKind::Church).unwrap_or(false))
        .min()
        .and_then(|b| world.get_component::<crate::game::villager::Pos>(b).copied());
    let open = |e: Entity| {
        if e == organizer || e == person
            || crate::game::villager::is_dead(world, e)
            || crate::game::dungeon::unavailable(world, e) { return false; }
        match (church, world.get_component::<crate::game::villager::Pos>(e)) {
            (Some(c), Some(p)) => (p.x - c.x).abs() + (p.y - c.y).abs() <= VILLAGE_EAR_REACH,
            _ => true,
        }
    };
    let priest = world.query::<crate::game::vocation::Craft>().iter().copied()
        .filter(|&e| open(e) && world.get_component::<crate::game::vocation::Craft>(e)
            .map(|c| c.vocation == crate::game::vocation::Vocation::Priest).unwrap_or(false))
        .min();
    if let Some(p) = priest { return Some((p, "priest")); }
    let role = |e: Entity, r: crate::game::vocation::TownRole| open(e)
        && world.get_component::<crate::game::vocation::TownRole>(e) == Some(&r);
    let helper = world.query::<crate::game::vocation::TownRole>().iter().copied()
        .filter(|&e| role(e, crate::game::vocation::TownRole::Matchmaker)).min();
    if let Some(p) = helper { return Some((p, "heart-helper")); }
    world.query::<crate::game::vocation::TownRole>().iter().copied()
        .filter(|&e| role(e, crate::game::vocation::TownRole::TaxCollector)).min()
        .map(|p| (p, "collector"))
}

/// Sick guests stay home
fn is_sick_guest(world: &World, guest: Entity) -> bool {
    world.query::<crate::sim::illness::Disease>().iter().any(|&d|
        world.get_component::<crate::sim::illness::Disease>(d)
            .map(|dd| dd.patient == guest && !matches!(dd.stage, crate::sim::illness::Stage::Recovering | crate::sim::illness::Stage::Latent))
            .unwrap_or(false))
}

/// The stake that draws an unpromised guest: a live grievance, a hard
/// streak of hunger or sickness, or a piece of their self-interest the
/// organizer's already heard. A contented stranger stays home.
fn has_stake(world: &World, guest: Entity, tick: u64) -> bool {
    use crate::game::desires::{Hardship, InnerLife, Known};
    if world.get_component::<InnerLife>(guest)
        .map(|l| l.grievances.iter().any(|(_, u)| *u >= tick)).unwrap_or(false) { return true; }
    if world.get_component::<Hardship>(guest)
        .map(|h| h.hungry > 0 || h.sick > 0).unwrap_or(false) { return true; }
    world.get_component::<Known>(guest).map(|k| !k.revealed.is_empty()).unwrap_or(false)
}

/// Whether an invited guest shows: the base rate is a coin's toss -
/// sickness and distance keep folk home, and a personal yes brings
/// them out. The dead and the held never come, and nobody comes
/// without a stake - a grievance, a hard streak, a want the
/// organizer's heard - unless they gave their yes or count the host
/// a true friend: a dragged-along friend who's content stays home.
pub fn shows_up(world: &World, organizer: Entity, guest: Entity, venue: (i32, i32), promised: bool, roll: f32, tick: u64) -> bool {
    use crate::game::relationships::{find_relationship, Relationship};
    if crate::game::dungeon::unavailable(world, guest) || is_sick_guest(world, guest) { return false; }
    let bond = find_relationship(world, organizer, guest)
        .and_then(|r| world.get_component::<Relationship>(r))
        .map(|r| r.strength).unwrap_or(0.0);
    if !promised && bond < 0.5 && !has_stake(world, guest, tick) { return false; }
    let mut p = if promised { 0.8 } else { 0.5 };
    p += bond.max(0.0) / 4.0;
    if let Some(pos) = world.get_component::<crate::game::villager::Pos>(guest) {
        let d = ((pos.x - venue.0 as f32).powi(2) + (pos.y - venue.1 as f32).powi(2)).sqrt();
        if d > 20.0 { p *= 0.5; } else if d > 10.0 { p *= 0.8; }
    }
    let skill = world.get_component::<crate::game::org_skills::Organizing>(organizer)
        .map(|o| o.turnout).unwrap_or(0.0);
    roll < (p + skill * 0.1).min(0.98)
}

/// Who a spreader brings: their strongest living bonds, strongest first
pub fn friends_of(world: &World, spreader: Entity, n: usize) -> Vec<Entity> {
    use crate::game::relationships::{bonds_of, Relationship};
    let mut out: Vec<(f32, Entity)> = bonds_of(world, spreader).iter().filter_map(|&r| {
        let rel = world.get_component::<Relationship>(r)?;
        let other = if rel.from == spreader { rel.other } else { rel.from };
        (!crate::game::villager::is_dead(world, other)).then_some((rel.strength, other))
    }).collect();
    out.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    out.into_iter().take(n).map(|(_, e)| e).collect()
}

/// Who stands over an issue - the target a cut issue can name. The
/// commons' hurts (the taxes, the writs, the death duty) point at the
/// Reeve; the overseers' cut points at whoever holds the posting; a
/// hurt nobody stands over is for the village to mend itself.
pub fn target_for(world: &World, tag: &crate::game::desires::IssueTag, collector: Option<Entity>) -> Option<Entity> {
    let base = tag.split_once(" (").map(|(b, _)| b).unwrap_or(tag.as_str());
    match base {
        "the taxes" | "the bailiff's writ" | "the death duty" => collector,
        "the overseers' cut" => world.query::<crate::game::vocation::TownRole>().iter().copied()
            .find(|&e| world.get_component::<crate::game::vocation::TownRole>(e)
                .map(|r| *r == crate::game::vocation::TownRole::Overseer).unwrap_or(false)),
        _ => None,
    }
}
