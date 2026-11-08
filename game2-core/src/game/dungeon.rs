//! The dungeon: a crackdown's teeth. The state takes the loudest member,
//! the cause becomes a prisoner, and the release becomes a campaign of
//! its own - won by persistence, not spectacle (the Amnesty model:
//! pressure is volume times time, and the issue organizes people a
//! network never reached).

use crate::engine::ecs::{Entity, World};
use crate::game::campaign::{self, DecisionRule, Framing};
use crate::game::reactions::{GuardsPosted, Sympathized};
use crate::game::villager::is_dead;
use crate::sim::community::Organization;

/// Behind bars: can't meet, can't act, but not gone
#[derive(Debug)]
pub struct Imprisoned {
    pub since: u64,
    pub charge: String,
}

/// Held at the castle on a writ: off the rolls while the org raises the
/// ransom or the ladder climbs. Not an org cell - no guard tour frees
/// them early, and the ransom walk home isn't the jailer's errand
#[derive(Debug)]
pub struct CastleHeld {
    pub since: u64,
}

/// Can't take part: dead, in the dungeon, or held at the castle (the
/// ways off the rolls)
pub fn unavailable(world: &World, e: Entity) -> bool {
    is_dead(world, e)
        || world.get_component::<Imprisoned>(e).is_some()
        || world.get_component::<CastleHeld>(e).is_some()
}

/// The crackdown takes the loudest one: after knights are posted, arrest
/// the living member with the strongest pull inside the org. Returns who
/// was taken; None if no guards are posted or nobody's left to take.
pub fn arrest(world: &mut World, campaign: Entity, tick: u64) -> Option<Entity> {
    let c = world.get_component::<campaign::Campaign>(campaign)?;
    if world.get_component::<GuardsPosted>(c.org).is_none() { return None; }
    let members = world.get_component::<Organization>(c.org)
        .map(|o| o.members.clone()).unwrap_or_default();
    let loudest = members.iter().copied()
        .filter(|&m| !unavailable(world, m))
        .max_by(|&a, &b| {
            let am = campaign::tie_mass(world, a, &members);
            let bm = campaign::tie_mass(world, b, &members);
            am.partial_cmp(&bm).unwrap_or(std::cmp::Ordering::Equal)
        })?;
    world.add_component(loudest, Imprisoned { since: tick, charge: "sedition".into() });
    Some(loudest)
}

/// A release effort: the pressure piling up for a prisoner's freedom.
/// It lives on the prisoner, so two people inside each carry their own -
/// and the cause that put them there keeps its own campaign untouched.
#[derive(Debug)]
pub struct ReleaseCampaign {
    pub pressure: f32,
}

/// Which campaign exists to free this prisoner - a dedicated release
/// campaign is retired when they walk; a cause's own campaign isn't
/// touched
#[derive(Debug)]
pub struct ReleaseOf {
    pub campaign: Entity,
}

/// Found a campaign for a prisoner's release: the issue is a person, so
/// there are no dissenters and the framing is duty. The sympathetic join
/// directly - nobody needs to be organized into caring.
pub fn found_release_campaign(world: &mut World, org: Entity, prisoner: Entity, founded_day: u64) -> Entity {
    let vote = campaign::IssueVote { winner: "the prisoners".into(), ballots: vec![] };
    let c = campaign::found_campaign(world, org, &vote, Framing::MoralDuty, DecisionRule::Consensus, founded_day);
    world.add_component(prisoner, ReleaseCampaign { pressure: 0.0 });
    world.add_component(prisoner, ReleaseOf { campaign: c });
    c
}

/// Join a release campaign without being organized into it: a bystander
/// marked sympathetic to the issue, or anyone whose own stake it is,
/// signs on. The issue organizes them.
pub fn join(world: &mut World, campaign_e: Entity, person: Entity) -> bool {
    if unavailable(world, person) { return false; }
    let Some(c) = world.get_component::<campaign::Campaign>(campaign_e) else { return false };
    let (org, issue) = (c.org, c.issue.clone());
    if world.get_component::<Organization>(org)
        .map(|o| o.members.contains(&person)).unwrap_or(false) { return false; }
    let sympathetic = world.get_component::<Sympathized>(person)
        .map(|s| s.issues.iter().any(|(i, _)| *i == issue)).unwrap_or(false);
    let own_stake = campaign::stake_for(world, person, &issue, 0).is_some();
    if !sympathetic && !own_stake { return false; }
    if let Some(o) = world.get_component_mut::<Organization>(org) {
        o.members.push(person);
        return true;
    }
    false
}

/// Every letter, every vigil, every petition - small actions that never
/// stop. Pressure to free a prisoner accumulates on the prisoner; it
/// doesn't fade on its own (release efforts don't lapse while someone's
/// inside).
pub fn press(world: &mut World, prisoner: Entity, amount: f32) {
    if world.get_component::<Imprisoned>(prisoner).is_none() { return; }
    if world.get_component::<ReleaseCampaign>(prisoner).is_none() {
        world.add_component(prisoner, ReleaseCampaign { pressure: 0.0 });
    }
    if let Some(r) = world.get_component_mut::<ReleaseCampaign>(prisoner) {
        r.pressure += amount;
    }
}

/// The jailer's ledger: when accumulated pressure beats the hold, the
/// prisoner walks - Imprisoned lifted, the effort spent. `resistance` is
/// what the state spends to keep them (its legitimacy, translated: how
/// much pressure it absorbs before it folds).
pub fn release_check(world: &mut World, prisoner: Entity, resistance: f32) -> bool {
    let holds = world.get_component::<ReleaseCampaign>(prisoner)
        .map(|r| r.pressure >= resistance).unwrap_or(false);
    if !holds { return false; }
    world.remove_component::<Imprisoned>(prisoner);
    world.remove_component::<ReleaseCampaign>(prisoner);
    // a campaign raised just for this prisoner is spent with the release;
    // anything else the org fights for marches on
    if let Some(rc) = world.get_component::<ReleaseOf>(prisoner).map(|r| r.campaign) {
        world.remove_component::<campaign::Campaign>(rc);
        world.remove_component::<campaign::CampaignMeeting>(rc);
        world.remove_component::<ReleaseOf>(prisoner);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::campaign::IssueVote;
    use crate::game::desires::InnerLife;
    use crate::game::relationships::{record_interaction, Interaction, InteractionKind};

    fn member(w: &mut World) -> Entity {
        let m = w.create_entity();
        w.add_component(m, InnerLife { wants: vec![], grievances: vec![], satisfied: vec![] });
        m
    }

    fn bond(w: &mut World, a: Entity, b: Entity, n: usize) {
        for _ in 0..n {
            record_interaction(w, a, Interaction { other: b, kind: InteractionKind::Aid, intensity: 1.0 });
        }
    }

    fn org_and_campaign(w: &mut World, members: &[Entity]) -> (Entity, Entity) {
        let o = w.create_entity();
        let leader = w.create_entity();
        w.add_component(o, Organization {
            name: "The League".into(), members: members.to_vec(), leader,
            influence_pool: 0.0, resources: 0.0, charter: "".into(), successors: vec![] });
        let vote = IssueVote { winner: "the taxes".into(), ballots: vec![] };
        let c = campaign::found_campaign(w, o, &vote, Framing::CommonFoe, DecisionRule::Majority, 1);
        (o, c)
    }

    #[test]
    fn test_the_crackdown_takes_the_loudest() {
        let mut w = World::new();
        let (loud, quiet, third) = (member(&mut w), member(&mut w), member(&mut w));
        bond(&mut w, loud, quiet, 3);
        bond(&mut w, loud, third, 2); // loud pulls everyone - the loudest
        let (o, c) = org_and_campaign(&mut w, &[loud, quiet, third]);
        assert!(arrest(&mut w, c, 5).is_none(), "no guards, no arrests");
        w.add_component(o, GuardsPosted { knights: 2, since: 5 });
        assert_eq!(arrest(&mut w, c, 6), Some(loud), "the one who pulls the most gets taken");
        assert!(unavailable(&w, loud));
        // and the take is once: the next loudest can follow
        assert_eq!(arrest(&mut w, c, 7), Some(quiet));
    }

    #[test]
    fn test_the_sympathetic_join_the_release_without_an_invite() {
        let mut w = World::new();
        let (o, _) = org_and_campaign(&mut w, &[]);
        let prisoner = member(&mut w);
        w.add_component(prisoner, Imprisoned { since: 3, charge: "sedition".into() });
        let c = found_release_campaign(&mut w, o, prisoner, 4);

        let sympathizer = member(&mut w);
        w.add_component(sympathizer, Sympathized { issues: vec![("the prisoners".into(), 5)] });
        assert!(join(&mut w, c, sympathizer), "the shocked sign on without being organized");
        let untouched = member(&mut w);
        assert!(!join(&mut w, c, untouched), "the untouched walk on by");
        assert!(w.get_component::<Organization>(o).unwrap().members.contains(&sympathizer));
    }

    #[test]
    fn test_persistence_frees_the_prisoner() {
        let mut w = World::new();
        let (o, _) = org_and_campaign(&mut w, &[]);
        let prisoner = member(&mut w);
        w.add_component(prisoner, Imprisoned { since: 3, charge: "sedition".into() });
        let c = found_release_campaign(&mut w, o, prisoner, 4);

        press(&mut w, prisoner, 0.3);
        assert!(!release_check(&mut w, prisoner, 1.0), "one letter is not pressure");
        press(&mut w, prisoner, 0.8);
        assert!(release_check(&mut w, prisoner, 1.0), "but letters that never stop are");
        assert!(!unavailable(&w, prisoner), "the cell opens");
        // the spent campaign's purpose is done - the org can take a new cause
        assert!(w.get_component::<campaign::Campaign>(c).is_none());
        assert!(!release_check(&mut w, prisoner, 0.1), "and it's spent");
    }
}

/// What the cell puts to the keep: their one ask this round
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Demand {
    /// Let our people go
    Release,
    /// The slop is killing us slower than the sentence
    Rations,
    /// The sick get seen to
    Care,
}

/// The keep's answer to the cell's ask
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Grant { Full, Part, Refused }

/// The cell organizes: the held folk of a castle site run their own
/// meetings behind bars - no errands, no outside callers, just their
/// own business: what to ask of the keep this week
pub struct CellOrg {
    /// The org entity they deliberate under
    pub org: Entity,
    /// The castle site they're held under
    pub site: usize,
    /// The ask still on the road or unanswered
    pub outstanding: Option<Demand>,
    /// The kitchen's measure: 0.0 is slop, 1.0 is decent
    pub rations: f32,
}

/// Word on the cell road: the ask going out, or the answer coming back
#[derive(Debug, Clone, Copy)]
pub struct CellLetter {
    /// The castle site the letter belongs to
    pub site: usize,
    /// What the cell asked (or the answer's about)
    pub demand: Demand,
    /// The keep's answer - only set on the road back
    pub grant: Option<Grant>,
    /// Who in the keep answers - the site's knight-lord
    pub warden: Entity,
    /// The day it lands
    pub arrive: u64,
}
