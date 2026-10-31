//! Campaigns: an org that votes on what to fight for. Founding is a
//! ballot - every member votes the issue that serves their own interest,
//! and the winner becomes the campaign's cause. The organizer picks how
//! to put it (the framing); folk who wanted a different fight, or whose
//! grievance doesn't suit the way it's put, grumble.
//! From wiki/campaign-plan.md (C1); the lit review that lands in
//! parallel will deepen the model.

use crate::engine::ecs::{Entity, World};
use crate::game::desires::{self, IssueTag, Reveal, Want};
use crate::game::villager::is_dead;
use crate::sim::community::Organization;

/// How the organizer puts the cause: a burden to share, a foe to name,
/// or a duty the village owes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Framing {
    /// "We all go hungry together" - a shared condition, shared out
    SharedGrievance,
    /// "The collector is the boot on your neck" - someone to point at
    CommonFoe,
    /// "The village owes its own" - a moral claim on the powerful
    MoralDuty,
}

/// A campaign: the org's settled cause, its own entity so it can carry
/// meetings, actions and prisoners of its own
#[derive(Debug)]
pub struct Campaign {
    pub issue: IssueTag,
    pub framing: Framing,
    pub org: Entity,
    pub founded_day: u64,
    /// Members who wanted a different issue - they came along, but they
    /// grumble (a later round lets grumbling cost attendance)
    pub dissenters: Vec<Entity>,
}

/// One member's ballot: the issue they want the campaign to take up
/// (None = no stake, abstained), and whether the winning framing sits
/// right with them
#[derive(Debug)]
pub struct Ballot {
    pub member: Entity,
    pub choice: Option<IssueTag>,
    /// The reveal whose tag they voted - what it's really about for them
    pub stake: Option<Reveal>,
    /// True when the campaign's framing doesn't fit their stake - they
    /// still voted, but they complain about how it's put
    pub grumbles: bool,
}

/// The result of a founding vote
#[derive(Debug)]
pub struct IssueVote {
    pub winner: IssueTag,
    pub ballots: Vec<Ballot>,
}

/// A member's self-interest: the live problem or unmet want they'd vote
/// for, as a reveal (its `issue_tag` is what matches a candidate). Live
/// problems come first - what hurts now beats what they merely wish for.
pub fn stake_of(world: &World, member: Entity, tick: u64) -> Option<Reveal> {
    reveals_of(world, member, tick).into_iter().next()
}

/// Everything a member could vote on, strongest first: live problems,
/// then unmet wants
fn reveals_of(world: &World, member: Entity, tick: u64) -> Vec<Reveal> {
    let mut out: Vec<Reveal> = desires::problems_of(world, member, tick)
        .iter().map(|&p| Reveal::Problem(p)).collect();
    if let Some(life) = world.get_component::<desires::InnerLife>(member) {
        out.extend(life.wants.iter()
            .filter(|&&w| !desires::is_satisfied(world, member, w, tick))
            .map(|&w| Reveal::Want(w)));
    }
    out
}

/// The reveal a member's vote rests on: their live interest whose tag is
/// `tag`. What a vote is really about, for them - it's what they judge
/// the framing by.
fn stake_for(world: &World, member: Entity, tag: &IssueTag, tick: u64) -> Option<Reveal> {
    reveals_of(world, member, tick).into_iter()
        .find(|r| desires::issue_tag(world, member, r) == *tag)
}

/// The candidate a member votes for: the one whose tag matches their
/// stake, else the one matching the most of their live problems.
/// `candidates` is the organizer's recommended order - a tie goes to the
/// issue the organizer put first. None if they have no dog in the fight.
pub fn vote_for(world: &World, member: Entity, candidates: &[IssueTag], tick: u64) -> Option<IssueTag> {
    let mut tags: Vec<IssueTag> = desires::problems_of(world, member, tick)
        .iter().map(|&p| desires::issue_tag(world, member, &Reveal::Problem(p))).collect();
    if let Some(life) = world.get_component::<desires::InnerLife>(member) {
        tags.extend(life.wants.iter()
            .filter(|&&w| !desires::is_satisfied(world, member, w, tick))
            .map(|&w| desires::issue_tag(world, member, &Reveal::Want(w))));
    }
    candidates.iter()
        .filter(|c| tags.contains(*c))
        .max_by_key(|c| tags.iter().filter(|t| *t == *c).count())
        .cloned()
}

/// Does a grievance have someone to point at? These suit CommonFoe.
fn foe_backed(r: &Reveal) -> bool {
    use crate::game::desires::Problem::*;
    matches!(r, Reveal::Problem(Taxed | OverseerCut | WritBorne | DeathDuty | ShortChanged | Loss))
}

/// Is the stake one of the spirit - faith, music, grief, the dark?
/// These suit MoralDuty.
fn of_the_spirit(r: &Reveal) -> bool {
    use crate::game::desires::Problem::*;
    matches!(r,
        Reveal::Problem(TheDark | Grief | Tempted)
            | Reveal::Want(Want::Faith | Want::Music | Want::Love | Want::Joy))
}

/// Whether the framing fits this member's stake. A wronged villager
/// wants a foe named; a broke one wants the burden shared; the grieving
/// want it put as a duty. A stakeless voter never grumbles.
pub fn framing_suits(framing: Framing, stake: &Reveal) -> bool {
    match framing {
        Framing::CommonFoe => foe_backed(stake),
        Framing::MoralDuty => of_the_spirit(stake),
        Framing::SharedGrievance => !foe_backed(stake) && !of_the_spirit(stake),
    }
}

/// The grumble a member makes when the framing doesn't sit right - for
/// the log. Bones: one line per way of missing.
pub fn grumble_line(framing: Framing) -> &'static str {
    match framing {
        Framing::CommonFoe => "mutters there's no one to hang for an empty larder",
        Framing::SharedGrievance => "mutters it isn't charity they want, it's what's owed",
        Framing::MoralDuty => "mutters fine words won't fill a belly",
    }
}

/// Hold the founding vote: each living member of the org votes their
/// self-interest among the candidate issues (the organizer's recommended
/// order breaks ties). Returns the winner and every ballot; None if the
/// org is gone, no candidates were put, or nobody has a stake to vote.
pub fn hold_issue_vote(world: &World, org: Entity, candidates: &[IssueTag], framing: Framing, tick: u64) -> Option<IssueVote> {
    let members = world.get_component::<Organization>(org)?.members.clone();
    if candidates.is_empty() { return None; }
    let mut ballots = vec![];
    let mut tally: Vec<usize> = vec![0; candidates.len()];
    for m in members {
        if is_dead(world, m) { continue; }
        let choice = vote_for(world, m, candidates, tick);
        let stake = choice.as_ref().and_then(|c| stake_for(world, m, c, tick));
        if let Some(c) = &choice {
            if let Some(i) = candidates.iter().position(|x| x == c) { tally[i] += 1; }
        }
        // They complain when the framing misses what their vote is really
        // about - abstainers have nothing to complain about yet
        let grumbles = match &stake {
            Some(s) => !framing_suits(framing, s),
            None => false,
        };
        ballots.push(Ballot { member: m, choice, stake, grumbles });
    }
    if tally.iter().all(|&n| n == 0) { return None; }
    let winner_i = tally.iter().enumerate()
        .max_by_key(|(i, &n)| (n, std::cmp::Reverse(*i))).map(|(i, _)| i)?;
    Some(IssueVote { winner: candidates[winner_i].clone(), ballots })
}

/// Found the campaign from a settled vote: the Campaign entity records
/// the issue, the framing it was won on, and the members whose choice
/// lost (the dissenters - they came along, but they grumble).
pub fn found_campaign(world: &mut World, org: Entity, vote: &IssueVote, framing: Framing, founded_day: u64) -> Entity {
    let dissenters = vote.ballots.iter()
        .filter(|b| b.choice.as_ref().map(|c| *c != vote.winner).unwrap_or(false))
        .map(|b| b.member).collect();
    let c = world.create_entity();
    world.add_component(c, Campaign {
        issue: vote.winner.clone(), framing, org, founded_day, dissenters,
    });
    c
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::desires::{add_grievance, InnerLife, Problem};

    fn org(w: &mut World, members: &[Entity]) -> Entity {
        let o = w.create_entity();
        let leader = w.create_entity();
        w.add_component(o, Organization {
            name: "The League".into(), members: members.to_vec(), leader,
            influence_pool: 0.0, resources: 0.0, charter: "".into(),
        });
        o
    }

    fn member(w: &mut World) -> Entity {
        let m = w.create_entity();
        w.add_component(m, InnerLife { wants: vec![], grievances: vec![], satisfied: vec![] });
        m
    }

    #[test]
    fn test_members_vote_their_own_interest() {
        let mut w = World::new();
        let taxed = member(&mut w);
        add_grievance(&mut w, taxed, Problem::Taxed, 100);
        let broke = member(&mut w);
        add_grievance(&mut w, broke, Problem::Broke, 100);
        let o = org(&mut w, &[taxed, broke]);
        let candidates = vec!["the taxes".to_string(), "poverty".to_string()];

        let vote = hold_issue_vote(&w, o, &candidates, Framing::SharedGrievance, 10).unwrap();
        assert_eq!(vote.winner, "the taxes");
        // broke folk wanted the poverty campaign: a dissenter, and the
        // poverty issue's a live one for them
        assert_eq!(vote.ballots[1].choice.as_deref(), Some("poverty"));
    }

    #[test]
    fn test_plurality_wins_and_ties_take_the_recommendation() {
        let mut w = World::new();
        let (a, b, c) = (member(&mut w), member(&mut w), member(&mut w));
        add_grievance(&mut w, a, Problem::Taxed, 100);
        add_grievance(&mut w, b, Problem::Taxed, 100);
        add_grievance(&mut w, c, Problem::Broke, 100);
        let o = org(&mut w, &[a, b, c]);
        let candidates = vec!["the taxes".to_string(), "poverty".to_string()];
        let vote = hold_issue_vote(&w, o, &candidates, Framing::SharedGrievance, 10).unwrap();
        assert_eq!(vote.winner, "the taxes", "two taxed beat one broke");

        // a tie goes to the issue the organizer put first
        let mut w = World::new();
        let (a, b) = (member(&mut w), member(&mut w));
        add_grievance(&mut w, a, Problem::Taxed, 100);
        add_grievance(&mut w, b, Problem::Broke, 100);
        let o = org(&mut w, &[a, b]);
        let candidates = vec!["poverty".to_string(), "the taxes".to_string()];
        let vote = hold_issue_vote(&w, o, &candidates, Framing::SharedGrievance, 10).unwrap();
        assert_eq!(vote.winner, "poverty", "the recommendation breaks the tie");
    }

    #[test]
    fn test_no_stake_no_vote_no_campaign() {
        let mut w = World::new();
        let idle = member(&mut w);
        let o = org(&mut w, &[idle]);
        let candidates = vec!["the taxes".to_string()];
        assert!(hold_issue_vote(&w, o, &candidates, Framing::CommonFoe, 10).is_none(),
            "a member with no grievance abstains, and nobody voted");
    }

    #[test]
    fn test_losers_dissent_and_bad_framing_grumbles() {
        let mut w = World::new();
        let taxed = member(&mut w);
        add_grievance(&mut w, taxed, Problem::Taxed, 100);
        let broke = member(&mut w);
        add_grievance(&mut w, broke, Problem::Broke, 100);
        let o = org(&mut w, &[taxed, broke]);
        let candidates = vec!["the taxes".to_string(), "poverty".to_string()];

        // won on a foe framing: the taxed voter is content, the broke
        // one both lost the vote and hates how it's put
        let vote = hold_issue_vote(&w, o, &candidates, Framing::CommonFoe, 10).unwrap();
        assert!(!vote.ballots[0].grumbles, "a tax is a foe worth naming");
        assert!(vote.ballots[1].grumbles, "broke folk don't want a hanging, they want bread");

        let c = found_campaign(&mut w, o, &vote, Framing::CommonFoe, 42);
        let camp = w.get_component::<Campaign>(c).unwrap();
        assert_eq!(camp.issue, "the taxes");
        assert_eq!(camp.founded_day, 42);
        assert_eq!(camp.dissenters, vec![broke]);
    }

    #[test]
    fn test_the_dead_dont_vote() {
        let mut w = World::new();
        let taxed = member(&mut w);
        add_grievance(&mut w, taxed, Problem::Taxed, 100);
        let ghost = member(&mut w);
        add_grievance(&mut w, ghost, Problem::Broke, 100);
        w.add_component(ghost, crate::game::villager::Dead);
        let o = org(&mut w, &[taxed, ghost]);
        let candidates = vec!["poverty".to_string(), "the taxes".to_string()];
        let vote = hold_issue_vote(&w, o, &candidates, Framing::SharedGrievance, 10).unwrap();
        assert_eq!(vote.winner, "the taxes", "the dead cast no ballot");
        assert_eq!(vote.ballots.len(), 1);
    }

    #[test]
    fn test_trade_scoped_issues_only_echo_in_the_trade() {
        // a mason's NoMaster tags "apprenticeship (mason)" - it doesn't
        // match a candidate about the bakers
        let mut w = World::new();
        let m = member(&mut w);
        let shop = w.create_entity();
        w.add_component(m, crate::game::vocation::Craft {
            vocation: crate::game::vocation::Vocation::Mason,
            skill: 0.3, workplace: Some(shop), master: None,
            tasked_by: None, pending: 0.0,
        });
        let candidates = vec!["apprenticeship (baker)".to_string(), "apprenticeship (mason)".to_string()];
        assert_eq!(vote_for(&w, m, &candidates, 10).as_deref(), Some("apprenticeship (mason)"));
        let candidates = vec!["apprenticeship (baker)".to_string()];
        assert_eq!(vote_for(&w, m, &candidates, 10), None, "the bakers' fight isn't theirs");
    }
}
