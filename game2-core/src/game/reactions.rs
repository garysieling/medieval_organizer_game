//! Reactions: the action is in the reaction. A public action is seen -
//! bystanders whose own trouble rhymes with the cause wake to it (a
//! moral shock, no network needed), and power answers: at first it
//! watches, and when the street is big enough it sends knights to guard
//! it. The crackdown is itself a grievance - every response either moves
//! the village or moves against it.

use crate::engine::ecs::{Entity, World};
use crate::game::campaign;
use crate::game::desires::IssueTag;
use crate::game::relationships::{find_relationship, Relationship};
use crate::game::villager::is_dead;

/// A bystander woken to causes they never organized into: the issues
/// they've watched become theirs (moral-shock recruitment - the channel
/// beside the network). One person can carry more than one sympathy.
#[derive(Debug, Default)]
pub struct Sympathized {
    pub issues: Vec<(IssueTag, u64)>,
}

/// A tie strong enough that watching a friend in the street moves you
pub const WITNESS_TIE: f32 = 0.3;

/// Whether watching `participants` moves `observer` to the cause:
/// the issue is already their stake, or someone they care for stood
/// in the street. Returns the sympathetic mark, if it lands.
pub fn witness(world: &mut World, observer: Entity, issue: &IssueTag, participants: &[Entity], tick: u64) -> bool {
    if participants.is_empty() || is_dead(world, observer) || participants.contains(&observer) { return false; }
    let own_stake = campaign::stake_for(world, observer, issue, tick).is_some();
    let saw_a_friend = participants.iter().any(|&p| find_relationship(world, observer, p)
        .and_then(|r| world.get_component::<Relationship>(r))
        .map(|r| r.strength >= WITNESS_TIE).unwrap_or(false));
    if !own_stake && !saw_a_friend { return false; }
    if world.get_component::<Sympathized>(observer).is_none() {
        world.add_component(observer, Sympathized::default());
    }
    if let Some(s) = world.get_component_mut::<Sympathized>(observer) {
        s.issues.retain(|(i, _)| i != issue);
        s.issues.push((issue.clone(), tick));
    }
    true
}

/// How the powers answer a public action
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Reaction {
    /// Beneath notice
    Ignored,
    /// Watched - the names get taken
    Watched,
    /// Knights posted to the streets - violent backlash, and itself a
    /// grievance that recruits
    Crackdown,
}

/// The king's knights in the streets, posted after a crackdown
#[derive(Debug)]
pub struct GuardsPosted {
    pub knights: u32,
    pub since: u64,
}

/// Turnout big enough to take names
pub const WATCHED_AT: usize = 2;
/// Turnout big enough to fear
pub const CRACKDOWN_AT: usize = 4;

/// The state's answer to a public action, by how big it actually was:
/// under a handful it watches the names; at a real crowd it posts
/// knights (GuardsPosted lands on the org). Returns the reaction.
pub fn react(world: &mut World, campaign: Entity, turnout: &[Entity], tick: u64) -> Option<Reaction> {
    let org = world.get_component::<campaign::Campaign>(campaign)?.org;
    // under a ban, a crowd over the cap is broken up on sight - the rule
    // itself does the state's work
    let banned = !turnout.is_empty() && world.get_component::<AssemblyBan>(org)
        .map(|b| turnout.len() > b.cap).unwrap_or(false);
    let r = if turnout.len() >= CRACKDOWN_AT || banned { Reaction::Crackdown }
        else if turnout.len() >= WATCHED_AT { Reaction::Watched }
        else { Reaction::Ignored };
    if r == Reaction::Crackdown {
        world.add_component(org, GuardsPosted { knights: (turnout.len() / 2).max(1) as u32, since: tick });
    }
    Some(r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::campaign::{DecisionRule, Framing, IssueVote};
    use crate::sim::community::Organization;
    use crate::game::desires::{add_grievance, InnerLife, Problem};
    use crate::game::relationships::{record_interaction, Interaction, InteractionKind};

    fn member(w: &mut World) -> Entity {
        let m = w.create_entity();
        w.add_component(m, InnerLife { wants: vec![], grievances: vec![], satisfied: vec![] });
        m
    }

    fn campaign(w: &mut World, members: &[Entity]) -> (Entity, Entity) {
        let o = w.create_entity();
        let leader = w.create_entity();
        w.add_component(o, Organization {
            name: "The League".into(), members: members.to_vec(), leader,
            influence_pool: 0.0, resources: 0.0, charter: "".into(),
        });
        let vote = IssueVote { winner: "the taxes".into(), ballots: vec![] };
        (o, campaign::found_campaign(w, o, &vote, Framing::CommonFoe, DecisionRule::Majority, 1))
    }

    #[test]
    fn test_a_bystander_wakes_to_their_own_trouble() {
        let mut w = World::new();
        let taxed = member(&mut w);
        add_grievance(&mut w, taxed, Problem::Taxed, 200);
        let stranger = member(&mut w);
        let actor = member(&mut w);

        assert!(witness(&mut w, taxed, &"the taxes".into(), &[actor], 5),
            "their own grievance marching past wakes them");
        assert!(!witness(&mut w, stranger, &"the taxes".into(), &[actor], 5),
            "a stranger with no stake walks on by");
        assert!(!witness(&mut w, actor, &"the taxes".into(), &[actor], 5),
            "you can't shock yourself");
    }

    #[test]
    fn test_watching_a_friend_march_moves_you() {
        let mut w = World::new();
        let (friend, actor) = (member(&mut w), member(&mut w));
        for _ in 0..3 {
            record_interaction(&mut w, friend, Interaction {
                other: actor, kind: InteractionKind::Aid, intensity: 1.0 });
        }
        assert!(witness(&mut w, friend, &"the taxes".into(), &[actor], 5),
            "a friend's face in the crowd is its own argument");
    }

    #[test]
    fn test_power_answers_a_crowd_with_knights() {
        let mut w = World::new();
        let m = member(&mut w);
        let (o, camp) = campaign(&mut w, &[m]);
        let few: Vec<Entity> = (0..2).map(|_| member(&mut w)).collect();
        assert_eq!(react(&mut w, camp, &few, 5), Some(Reaction::Watched));
        assert!(w.get_component::<GuardsPosted>(o).is_none());

        let crowd: Vec<Entity> = (0..5).map(|_| member(&mut w)).collect();
        assert_eq!(react(&mut w, camp, &crowd, 5), Some(Reaction::Crackdown));
        assert_eq!(w.get_component::<GuardsPosted>(o).unwrap().knights, 2);
    }
}

/// The lord's answer to carried demands: the load the issue names comes
/// back light for the cause's people until `until`. A concession is
/// dated - the state gives relief, not surrender; the organizer has to
/// keep it won. Only the issue fought for is answered
#[derive(Debug)]
pub struct LordGrant {
    pub issue: String,
    pub until: u64,
}

/// The state's other answer: a rule made against the assembly itself.
/// A ban on gatherings over `cap` - the movement got big enough that
/// the law had to answer it. Public actions over the cap are broken up
/// on sight
#[derive(Debug)]
pub struct AssemblyBan {
    pub cap: usize,
}
