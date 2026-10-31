//! Organizing skills: a second craft, learned by doing. Running 1-1s,
//! working meetings, pulling turnout, agitating a room, and teaching are
//! each their own skill - they live on the person, not the campaign, so
//! what someone learned on one fight they keep for the next (Highlander's
//! citizenship schools: the answers come from the people, and the people
//! who just learned are the ones best placed to teach).

use crate::engine::ecs::{Entity, World};
use crate::game::campaign::{self, Role};

/// A person's organizing craft, separate from their vocation's `Craft`.
/// All 0..1, all grown by participation - nobody is born knowing how to
/// run a house meeting.
#[derive(Debug, Default)]
pub struct Organizing {
    /// Relational conversations: 1-1s that surface what someone cares about
    pub one_on_one: f32,
    /// Working a room: house meetings, campaign meetings
    pub house_meeting: f32,
    /// Getting people to show up
    pub turnout: f32,
    /// Moving a room to anger or action; coaching through a blow-off
    pub agitation: f32,
    /// Developing other organizers
    pub training: f32,
}

/// The skills, for passing one around by name
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrgSkill {
    OneOnOne,
    HouseMeeting,
    Turnout,
    Agitation,
    Training,
}

impl Organizing {
    /// Read a skill by name
    pub fn get(&self, s: OrgSkill) -> f32 {
        match s {
            OrgSkill::OneOnOne => self.one_on_one,
            OrgSkill::HouseMeeting => self.house_meeting,
            OrgSkill::Turnout => self.turnout,
            OrgSkill::Agitation => self.agitation,
            OrgSkill::Training => self.training,
        }
    }

    fn bump(&mut self, s: OrgSkill, amount: f32) {
        let v = match s {
            OrgSkill::OneOnOne => &mut self.one_on_one,
            OrgSkill::HouseMeeting => &mut self.house_meeting,
            OrgSkill::Turnout => &mut self.turnout,
            OrgSkill::Agitation => &mut self.agitation,
            OrgSkill::Training => &mut self.training,
        };
        *v = (*v + amount).min(1.0);
    }
}

/// How good they are at a skill, on the vocation's own ladder:
/// journeyman (<0.4), apprentice (<0.75), master
pub fn level_of(world: &World, person: Entity, skill: OrgSkill) -> crate::game::vocation::Level {
    let v = world.get_component::<Organizing>(person).map(|o| o.get(skill)).unwrap_or(0.0);
    crate::game::vocation::level_of(v)
}

/// Learn by doing: `amount` added to the skill, capped at 1. The
/// component appears the first time someone actually does the thing -
/// participation is what makes an organizer.
pub fn practice(world: &mut World, person: Entity, skill: OrgSkill, amount: f32) {
    if world.get_component::<Organizing>(person).is_none() {
        world.add_component(person, Organizing::default());
    }
    if let Some(o) = world.get_component_mut::<Organizing>(person) {
        o.bump(skill, amount);
    }
}

/// What a seat teaches: the coordinator works the list, the chair moves
/// the room, the note taker keeps the institutional memory (each is its
/// own craft)
pub fn role_skill(role: Role) -> OrgSkill {
    match role {
        Role::TurnoutCoordinator => OrgSkill::Turnout,
        Role::Chair => OrgSkill::Agitation,
        Role::NoteTaker => OrgSkill::Training,
    }
}

/// A little practice for turning up
pub const MEETING_PRACTICE: f32 = 0.02;
/// More for carrying a seat through the meeting
pub const SEAT_PRACTICE: f32 = 0.03;

/// A meeting's lessons, distributed: everyone who came gets meeting
/// practice; whoever held a seat gets practice at the seat's craft.
/// Called after a campaign meeting resolves.
pub fn learn_from_meeting(world: &mut World, campaign: Entity, attendees: &[Entity]) {
    let seats: Vec<crate::game::campaign::Seat> = world.get_component::<campaign::Campaign>(campaign)
        .map(|c| c.seats.clone()).unwrap_or_default();
    for &a in attendees {
        practice(world, a, OrgSkill::HouseMeeting, MEETING_PRACTICE);
        for s in &seats {
            if s.holder == a {
                practice(world, a, role_skill(s.role), SEAT_PRACTICE);
            }
        }
    }
}

/// An apprentice can teach; a journeyman still can't. The bar on the
/// ladder (vocation level names: <0.4 journeyman, <0.75 apprentice)
pub const TEACH_MIN_SKILL: f32 = 0.4;
/// What a student gains per lesson
pub const LEARN_PER_LESSON: f32 = 0.06;
/// How much more a student learns when the fight is theirs
pub const MOTIVATED_BONUS: f32 = 1.5;
/// What teaching does for the teacher
pub const TEACH_PRACTICE: f32 = 0.02;

/// Teach a skill: `trainer` teaches `student` a little of `skill`.
/// Requires the trainer to be an apprentice of it or better, and to
/// actually out-skill the student (nobody teaches past their own
/// craft). `campaign` is the context that motivates: a student whose
/// own stake matches the campaign's issue learns faster - the answers
/// come from the people. Returns whether the lesson happened.
pub fn teach(world: &mut World, campaign: Option<Entity>, trainer: Entity, student: Entity, skill: OrgSkill, tick: u64) -> bool {
    let (trainer_at, student_at) = {
        let t = world.get_component::<Organizing>(trainer).map(|o| o.get(skill)).unwrap_or(0.0);
        let s = world.get_component::<Organizing>(student).map(|o| o.get(skill)).unwrap_or(0.0);
        (t, s)
    };
    if trainer_at < TEACH_MIN_SKILL || student_at >= trainer_at || trainer == student {
        return false;
    }
    // A student fights for what they already feel: does the campaign's
    // issue match one of their live stakes?
    let motivated = campaign
        .and_then(|c| world.get_component::<campaign::Campaign>(c).map(|cc| cc.issue.clone()))
        .map(|issue| campaign::stake_for(world, student, &issue, tick).is_some())
        .unwrap_or(false);
    let gain = if motivated { LEARN_PER_LESSON * MOTIVATED_BONUS } else { LEARN_PER_LESSON };
    practice(world, student, skill, gain);
    practice(world, trainer, OrgSkill::Training, TEACH_PRACTICE);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::campaign::{DecisionRule, Framing, IssueVote, CampaignMeeting};
    use crate::game::desires::{add_grievance, InnerLife, Problem};
    use crate::game::vocation::Level;
    use crate::sim::community::Organization;

    fn person(w: &mut World) -> Entity { w.create_entity() }

    fn org(w: &mut World, members: &[Entity]) -> Entity {
        let o = w.create_entity();
        let leader = w.create_entity();
        w.add_component(o, Organization {
            name: "The League".into(), members: members.to_vec(), leader,
            influence_pool: 0.0, resources: 0.0, charter: "".into(),
        });
        o
    }

    #[test]
    fn test_practice_builds_the_craft_and_it_caps() {
        let mut w = World::new();
        let p = person(&mut w);
        practice(&mut w, p, OrgSkill::OneOnOne, 0.1);
        assert!((level_of(&w, p, OrgSkill::OneOnOne) == Level::Journeyman));
        practice(&mut w, p, OrgSkill::OneOnOne, 0.4);
        assert_eq!(level_of(&w, p, OrgSkill::OneOnOne), Level::Apprentice);
        practice(&mut w, p, OrgSkill::OneOnOne, 0.9);
        assert_eq!(level_of(&w, p, OrgSkill::OneOnOne), Level::Master);
        // the other crafts stay untouched
        assert!(w.get_component::<Organizing>(p).unwrap().turnout < 1e-6);
    }

    #[test]
    fn test_you_cant_teach_past_your_own_craft() {
        let mut w = World::new();
        let (journeyman, apprentice, student) = (person(&mut w), person(&mut w), person(&mut w));
        practice(&mut w, journeyman, OrgSkill::Turnout, 0.2);
        practice(&mut w, apprentice, OrgSkill::Turnout, 0.5);
        assert!(!teach(&mut w, None, journeyman, student, OrgSkill::Turnout, 1),
            "a journeyman can't teach yet");
        assert!(teach(&mut w, None, apprentice, student, OrgSkill::Turnout, 1));
        // and a student who caught up learns nothing more from them
        practice(&mut w, student, OrgSkill::Turnout, 0.5);
        assert!(!teach(&mut w, None, apprentice, student, OrgSkill::Turnout, 1));
        // teaching grows the teaching craft
        assert!(w.get_component::<Organizing>(apprentice).unwrap().training > 0.0);
    }

    #[test]
    fn test_a_student_fights_for_what_they_feel() {
        let mut w = World::new();
        let (teacher, believer, bystander) = (person(&mut w), person(&mut w), person(&mut w));
        practice(&mut w, teacher, OrgSkill::Agitation, 0.6);
        for v in [believer, bystander] {
            w.add_component(v, InnerLife { wants: vec![], grievances: vec![], satisfied: vec![] });
        }
        add_grievance(&mut w, believer, Problem::Taxed, 200);
        // a campaign about the taxes: the taxed student is motivated
        let o = org(&mut w, &[]);
        let vote = IssueVote { winner: "the taxes".into(), ballots: vec![] };
        let c = campaign::found_campaign(&mut w, o, &vote, Framing::CommonFoe, DecisionRule::Majority, 1);

        assert!(teach(&mut w, Some(c), teacher, believer, OrgSkill::Agitation, 10));
        assert!(teach(&mut w, Some(c), teacher, bystander, OrgSkill::Agitation, 10));
        let (b, s) = (
            w.get_component::<Organizing>(believer).unwrap().agitation,
            w.get_component::<Organizing>(bystander).unwrap().agitation);
        assert!((b - LEARN_PER_LESSON * MOTIVATED_BONUS).abs() < 1e-6);
        assert!((s - LEARN_PER_LESSON).abs() < 1e-6);
    }

    #[test]
    fn test_everyone_in_the_room_learns_and_the_seat_learns_more() {
        let mut w = World::new();
        let (coordinator, attendee) = (person(&mut w), person(&mut w));
        let o = org(&mut w, &[coordinator, attendee]);
        let vote = IssueVote { winner: "the taxes".into(), ballots: vec![] };
        let c = campaign::found_campaign(&mut w, o, &vote, Framing::CommonFoe, DecisionRule::Majority, 1);
        campaign::delegate_role(&mut w, c, Role::TurnoutCoordinator, coordinator);

        learn_from_meeting(&mut w, c, &[coordinator, attendee]);
        let coord = w.get_component::<Organizing>(coordinator).unwrap();
        assert!((coord.turnout - SEAT_PRACTICE).abs() < 1e-6);
        assert!((coord.house_meeting - MEETING_PRACTICE).abs() < 1e-6);
        let att = w.get_component::<Organizing>(attendee).unwrap();
        assert!((att.house_meeting - MEETING_PRACTICE).abs() < 1e-6);
        assert!(att.turnout < 1e-6, "the room doesn't learn the seat's craft");
    }

    #[test]
    fn test_the_craft_outlives_the_campaign() {
        let mut w = World::new();
        let p = person(&mut w);
        practice(&mut w, p, OrgSkill::Turnout, 0.6);
        // one campaign ends, another begins - the person keeps the craft
        let (o1, o2) = (org(&mut w, &[p]), org(&mut w, &[p]));
        for o in [o1, o2] {
            let vote = IssueVote { winner: "the taxes".into(), ballots: vec![] };
            let c = campaign::found_campaign(&mut w, o, &vote, Framing::CommonFoe, DecisionRule::Majority, 1);
            w.add_component(c, CampaignMeeting { cadence_days: 7, last_held: 0, held: 0, resolves: 0 });
        }
        assert_eq!(level_of(&w, p, OrgSkill::Turnout), Level::Apprentice,
            "the skills a person learns stay on the person");
    }
}
