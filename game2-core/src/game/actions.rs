//! Public actions: the campaign steps out where everyone can see. A
//! demonstration outside a building, a strike vote across a trade, a
//! townhall where demands are put to a target. Each is a structure test -
//! the turnout is the org's hidden strength made legible, and the answer
//! that comes back shapes what the campaign is worth (the action is in
//! the reaction).

use crate::engine::ecs::{Entity, World};
use crate::game::campaign::{self, Role, ATTEND_MIN_TIE};
use crate::game::organizing::living_leader;
use crate::game::relationships::{find_relationship, Relationship};
use crate::game::dungeon::unavailable;
use crate::game::vocation::{Craft, Vocation};
use crate::sim::community::Organization;
use crate::sim::organized_power::{self, ActionQuality};

/// Who calls people out for an action: the turnout coordinator if the
/// seat's filled, else the shadow elite, else the leader. Same pull that
/// fills a meeting room fills a picket line.
fn rally_driver(world: &mut World, campaign: Entity) -> Option<Entity> {
    let c = world.get_component::<campaign::Campaign>(campaign)?;
    let members = world.get_component::<Organization>(c.org).map(|o| o.members.clone())?;
    let leader = living_leader(world, c.org)?;
    campaign::seat_holder(world, campaign, Role::TurnoutCoordinator)
        .filter(|&h| !unavailable(world, h) && (h == leader || members.contains(&h)))
        .or_else(|| campaign::shadow_elite(world, campaign))
        .or(Some(leader))
        .filter(|&d| !unavailable(world, d))
}

/// Who turns out for an action: the members - the driver if they're one,
/// and every other living member whose tie to them clears the bar. A
/// lone figure in the street is no show of power; the hidden strength is
/// revealed only as bodies the org actually fields.
fn turnout(world: &mut World, campaign: Entity) -> Vec<Entity> {
    let Some(c) = world.get_component::<campaign::Campaign>(campaign) else { return vec![] };
    let members = world.get_component::<Organization>(c.org)
        .map(|o| o.members.clone()).unwrap_or_default();
    let Some(driver) = rally_driver(world, campaign) else { return vec![] };
    let mut out = vec![];
    for m in members {
        if unavailable(world, m) { continue; }
        let tied = m == driver || find_relationship(world, m, driver)
            .and_then(|r| world.get_component::<Relationship>(r))
            .map(|r| r.strength >= ATTEND_MIN_TIE).unwrap_or(false);
        if tied { out.push(m); }
    }
    out
}

/// What the village saw when the campaign stepped out
#[derive(Debug)]
pub struct ActionOutcome {
    /// Who came - the structure test's reading
    pub turnout: Vec<Entity>,
    /// Turnout as a share of the membership (0-1)
    pub scale: f32,
    /// How the action landed for the org's people power
    pub quality: ActionQuality,
}

/// A demonstration outside a building (or anywhere people will see it).
/// Scale is the share of members who turn out: half or better is a real
/// show of power, a thin crowd barely registers, an empty street costs
/// the org.
pub fn demonstrate(world: &mut World, campaign: Entity, _at: Entity, _tick: u64) -> Option<ActionOutcome> {
    let c = world.get_component::<campaign::Campaign>(campaign)?;
    let (org, member_count) = (c.org, world.get_component::<Organization>(c.org)
        .map(|o| o.members.len()).unwrap_or(0));
    let out = turnout(world, campaign);
    if out.is_empty() {
        // nobody came: if there were members to call, that's a public
        // failure - the empty street costs the org
        if member_count > 0 {
            organized_power::record_action(world, org, ActionQuality::Overreach);
        }
        return None;
    }
    let scale = out.len() as f32 / (member_count.max(1)) as f32;
    // a real show of power takes more than the person who called it
    let quality = if scale >= 0.5 && out.len() >= 2 { ActionQuality::Transformational }
        else if scale >= 0.2 { ActionQuality::Transactional }
        else { ActionQuality::Overreach };
    organized_power::record_action(world, org, quality);
    Some(ActionOutcome { turnout: out, scale, quality })
}

/// How a strike vote went, in raw numbers
#[derive(Debug)]
pub struct StrikeOutcome {
    pub yes: usize,
    pub no: usize,
    /// Share of the trade that said yes (0-1; 0 if nobody balloted)
    pub share: f32,
    /// A strike needs nine of ten - anything less is a public failure
    pub called: bool,
}

/// A strike needs nine of ten voices
pub const STRIKE_QUORUM: f32 = 0.9;
/// A tie this strong pulls a vote even where the issue doesn't bite
pub const PULLED_TIE: f32 = 0.5;

/// A strike ballot across a vocation: every living worker of the trade
/// says yes or no. Yes if the campaign's issue is their own stake, or a
/// tie to any member strong enough to pull them along. Under nine of ten
/// and it fails in public - the org burns capacity for overreaching.
pub fn strike_vote(world: &mut World, campaign: Entity, vocation: Vocation, tick: u64) -> Option<StrikeOutcome> {
    let c = world.get_component::<campaign::Campaign>(campaign)?;
    let (issue, org) = (c.issue.clone(), c.org);
    // the dead and the jailed pull nobody - only free members' ties
    // drag a ballot
    let members: Vec<Entity> = world.get_component::<Organization>(org)
        .map(|o| o.members.clone()).unwrap_or_default()
        .into_iter().filter(|&m| !unavailable(world, m)).collect();
    let mut yes = 0usize;
    let mut no = 0usize;
    for v in world.query::<Craft>() {
        if unavailable(world, v) { continue; }
        let Some(craft) = world.get_component::<Craft>(v) else { continue };
        if craft.vocation != vocation { continue; }
        let own_fight = campaign::stake_for(world, v, &issue, tick).is_some();
        let pulled = members.iter().any(|&m| m != v && find_relationship(world, v, m)
            .and_then(|r| world.get_component::<Relationship>(r))
            .map(|r| r.strength >= PULLED_TIE).unwrap_or(false));
        if own_fight || pulled { yes += 1; } else { no += 1; }
    }
    let total = yes + no;
    if total == 0 { return None; }
    let share = yes as f32 / total as f32;
    let called = share >= STRIKE_QUORUM;
    organized_power::record_action(world, org,
        if called { ActionQuality::Transformational } else { ActionQuality::Overreach });
    Some(StrikeOutcome { yes, no, share, called })
}

/// What the target said when the demands were put
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TargetResponse {
    /// They take it up the chain - "the king will hear of this"
    Carries,
    /// It touches them, but the room's too thin to carry it whole: they
    /// grant part of it, and no more
    Concedes,
    /// They send regards and a clerk - a polite nothing
    Deflects,
    /// They won't hear it - the doors stay shut
    Refuses,
}

/// What a townhall produced
#[derive(Debug)]
pub struct TownhallOutcome {
    /// The room that showed for it
    pub turnout: Vec<Entity>,
    /// What the target did with the demands
    pub response: TargetResponse,
}

/// What a blow-off leaves on the people who came: the helplessness the
/// organizer has to coach them through
#[derive(Debug)]
pub struct Demoralized {
    pub since: u64,
}

/// How long helplessness lingers on its own: about a month, then the
/// sting fades whether or not anyone coached them through it
pub const HOLLOW_DAYS: u64 = 30;

/// Time heals what the organizer doesn't get to: a demoralized member
/// comes back to themselves once the mark has aged out. Returns whether
/// it lifted.
pub fn lifts_with_time(world: &mut World, member: Entity, tick: u64) -> bool {
    let stale = world.get_component::<Demoralized>(member)
        .map(|d| tick.saturating_sub(d.since) >= HOLLOW_DAYS)
        .unwrap_or(false);
    if stale { world.remove_component::<Demoralized>(member); }
    stale
}

/// A townhall: the campaign invites a target (a PMC member) and puts its
/// demands. The answer rides on the scale of the room and the target's
/// own interests: enough people and a stake that touches them and they
/// carry it up the chain; a thin room gets a clerk and regards; anything
/// else is refused outright. The last two are the usual case - they
/// leave everyone who came demoralized.
pub fn townhall(world: &mut World, campaign: Entity, target: Entity, tick: u64) -> Option<TownhallOutcome> {
    let issue = world.get_component::<campaign::Campaign>(campaign)?.issue.clone();
    let member_count = world.get_component::<campaign::Campaign>(campaign)
        .and_then(|c| world.get_component::<Organization>(c.org))
        .map(|o| o.members.len()).unwrap_or(0);
    let out = turnout(world, campaign);
    if out.is_empty() { return None; }
    let scale = out.len() as f32 / (member_count.max(1)) as f32;
    let their_fight = campaign::stake_for(world, target, &issue, tick).is_some();
    let response = if their_fight && scale >= 0.5 { TargetResponse::Carries }
        else if scale < 0.2 { TargetResponse::Deflects }
        else if their_fight { TargetResponse::Concedes }
        else { TargetResponse::Refuses };
    if matches!(response, TargetResponse::Deflects | TargetResponse::Refuses) {
        for &m in &out { world.add_component(m, Demoralized { since: tick }); }
    }
    let org = world.get_component::<campaign::Campaign>(campaign).unwrap().org;
    organized_power::record_action(world, org, match response {
        TargetResponse::Carries => ActionQuality::Transformational,
        TargetResponse::Concedes => ActionQuality::Transactional,
        TargetResponse::Deflects | TargetResponse::Refuses => ActionQuality::Overreach,
    });
    Some(TownhallOutcome { turnout: out, response })
}

/// The organizer's job after a blow-off: talk the helplessness out of
/// them. Takes a real bond (the hidden strength, spent where it counts)
/// and practices the craft of agitation. Returns whether the member came
/// back to themselves.
pub fn coach(world: &mut World, organizer: Entity, member: Entity, _tick: u64) -> bool {
    let tie = find_relationship(world, organizer, member)
        .and_then(|r| world.get_component::<Relationship>(r))
        .map(|r| r.strength).unwrap_or(0.0);
    if tie < ATTEND_MIN_TIE { return false; }
    if world.get_component::<Demoralized>(member).is_none() { return false; }
    world.remove_component::<Demoralized>(member);
    crate::game::org_skills::practice(world, organizer,
        crate::game::org_skills::OrgSkill::Agitation, crate::game::org_skills::MEETING_PRACTICE);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::campaign::{DecisionRule, Framing, IssueVote};
    use crate::game::desires::{add_grievance, InnerLife, Problem};
    use crate::game::relationships::{record_interaction, Interaction, InteractionKind};

    fn member(w: &mut World) -> Entity {
        let m = w.create_entity();
        w.add_component(m, InnerLife { wants: vec![], grievances: vec![], satisfied: vec![] });
        m
    }

    fn bond(w: &mut World, a: Entity, b: Entity) {
        for _ in 0..2 {
            record_interaction(w, a, Interaction { other: b, kind: InteractionKind::Aid, intensity: 1.0 });
        }
    }

    fn campaign(w: &mut World, members: &[Entity], issue: &str) -> (Entity, Entity) {
        let o = w.create_entity();
        let leader = w.create_entity();
        w.add_component(o, Organization {
            name: "The League".into(), members: members.to_vec(), leader,
            influence_pool: 0.0, resources: 0.0, charter: "".into(),
        });
        w.add_component(o, organized_power::OrganizedPower::new());
        let vote = IssueVote { winner: issue.into(), ballots: vec![] };
        let c = campaign::found_campaign(w, o, &vote, Framing::CommonFoe, DecisionRule::Majority, 1);
        (o, c)
    }

    #[test]
    fn test_a_crowd_is_a_structure_test() {
        let mut w = World::new();
        let (a, b, c) = (member(&mut w), member(&mut w), member(&mut w));
        bond(&mut w, a, b);
        bond(&mut w, a, c);
        let (_, camp) = campaign(&mut w, &[a, b, c], "the taxes");
        campaign::delegate_role(&mut w, camp, Role::TurnoutCoordinator, a);

        let spot = w.create_entity();
        let out = demonstrate(&mut w, camp, spot, 5).unwrap();
        assert_eq!(out.turnout.len(), 3, "everyone who pulls answers the call");
        assert_eq!(out.quality, ActionQuality::Transformational);

        // a one-member org's coordinator is its whole crowd - no show
        let (_, thin) = campaign(&mut w, &[a], "poverty");
        campaign::delegate_role(&mut w, thin, Role::TurnoutCoordinator, a);
        let out = demonstrate(&mut w, thin, spot, 5).unwrap();
        assert_eq!(out.turnout.len(), 1);
        assert_ne!(out.quality, ActionQuality::Transformational,
            "one body in the street is no show of power");
    }

    #[test]
    fn test_an_empty_street_costs_the_org() {
        let mut w = World::new();
        let (a, b) = (member(&mut w), member(&mut w));
        let (o, camp) = campaign(&mut w, &[a, b], "the taxes");
        // members exist but nobody's tied to the leader who calls it
        w.get_component_mut::<organized_power::OrganizedPower>(o).unwrap().capacity = 0.5;
        let spot = w.create_entity();
        assert!(demonstrate(&mut w, camp, spot, 5).is_none());
        let after = w.get_component::<organized_power::OrganizedPower>(o).unwrap().capacity;
        assert!(after < 0.5, "an empty demonstration burns capacity, got {}", after);
    }

    #[test]
    fn test_the_dead_pull_no_ballots() {
        let mut w = World::new();
        let living_member = member(&mut w);
        let dead_member = member(&mut w);
        let (_, camp) = campaign(&mut w, &[living_member, dead_member], "the taxes");
        // a mason bonded hard to the dead member, indifferent to the issue
        let mason = member(&mut w);
        for _ in 0..5 { bond(&mut w, mason, dead_member); }
        w.add_component(mason, Craft {
            vocation: Vocation::Mason, skill: 0.5,
            workplace: None, master: None, tasked_by: None, pending: 0.0 });
        w.add_component(dead_member, crate::game::villager::Dead);
        let out = strike_vote(&mut w, camp, Vocation::Mason, 10).unwrap();
        assert_eq!(out.yes, 0, "a dead member's pull died with them");
        assert!(!out.called);
    }

    #[test]
    fn test_nine_of_ten_calls_the_strike_and_less_fails_public() {
        let mut w = World::new();
        // ten masons: eight taxed (their fight), one bonded to a member
        // (pulled), one untouched (no)
        let mut masons = vec![];
        for _ in 0..8 {
            let m = member(&mut w);
            add_grievance(&mut w, m, Problem::Taxed, 200);
            masons.push(m);
        }
        let pulled = member(&mut w);
        let unmoved = member(&mut w);
        for m in masons.iter().chain([&pulled, &unmoved]) {
            w.add_component(*m, Craft {
                vocation: Vocation::Mason, skill: 0.5,
                workplace: None, master: None, tasked_by: None, pending: 0.0 });
        }
        let (_, camp) = campaign(&mut w, &[masons[0]], "the taxes");
        bond(&mut w, pulled, masons[0]); // 0.24 - not enough to pull alone
        for _ in 0..4 { bond(&mut w, pulled, masons[0]); } // 0.24*5 > 0.5

        let out = strike_vote(&mut w, camp, Vocation::Mason, 10).unwrap();
        assert_eq!(out.yes + out.no, 10);
        assert_eq!(out.yes, 9, "their own fight, plus one pulled along");
        assert!(out.called);

        // a campaign about something else entirely fails in public
        let (_, other) = campaign(&mut w, &[masons[0]], "the shrinking wood");
        let out = strike_vote(&mut w, other, Vocation::Mason, 10).unwrap();
        assert_eq!(out.yes, 1, "only the pulled mason");
        assert!(!out.called, "one of ten is not a strike");
    }

    #[test]
    fn test_the_target_carries_what_touches_them_or_shuts_the_door() {
        let mut w = World::new();
        let (a, b) = (member(&mut w), member(&mut w));
        bond(&mut w, a, b);
        let (_, camp) = campaign(&mut w, &[a, b], "the taxes");
        campaign::delegate_role(&mut w, camp, Role::TurnoutCoordinator, a);

        // a target who pays the same taxes carries it up the chain
        let moved = member(&mut w);
        add_grievance(&mut w, moved, Problem::Taxed, 200);
        let out = townhall(&mut w, camp, moved, 10).unwrap();
        assert_eq!(out.response, TargetResponse::Carries);
        assert!(w.get_component::<Demoralized>(b).is_none());

        // the same target, facing a thinner room (one of three turns out),
        // grants part of it - nobody goes home hollow
        let c2 = member(&mut w);
        let lone = member(&mut w);
        let c3 = member(&mut w);
        let (_, thin) = campaign(&mut w, &[lone, c2, c3], "the taxes");
        campaign::delegate_role(&mut w, thin, Role::TurnoutCoordinator, lone);
        let out = townhall(&mut w, thin, moved, 10).unwrap();
        assert_eq!(out.response, TargetResponse::Concedes);
        assert!(w.get_component::<Demoralized>(lone).is_none());

        // a target it doesn't touch blows the room off - and the
        // demoralized need coaching
        let stone = member(&mut w);
        let out = townhall(&mut w, camp, stone, 10).unwrap();
        assert_eq!(out.response, TargetResponse::Refuses);
        assert!(w.get_component::<Demoralized>(b).is_some(), "a blow-off demoralizes");
        // a stranger can't coach; the bonded organizer can
        assert!(coach(&mut w, a, b, 10));
        assert!(w.get_component::<Demoralized>(b).is_none());
    }

    #[test]
    fn test_townhall_outcomes_rise_with_the_room() {
        // a probe over every room size and turnout: more people in the room
        // never earns a worse answer, a part-grant needs the target's own
        // stake, and it sits only between the thin and the full room
        let rank = |r: TargetResponse| match r {
            TargetResponse::Deflects | TargetResponse::Refuses => 0,
            TargetResponse::Concedes => 1,
            TargetResponse::Carries => 2,
        };
        let mut table: Vec<(usize, usize, bool, TargetResponse)> = vec![];
        for members in 2..=10 {
            for staked in [true, false] {
                let mut last = 0;
                for answering in 0..members {
                    let mut w = World::new();
                    let ms: Vec<Entity> = (0..members).map(|_| member(&mut w)).collect();
                    for &m in &ms[1..=answering.min(members - 1)] { bond(&mut w, ms[0], m); }
                    let (_, camp) = campaign(&mut w, &ms, "the taxes");
                    campaign::delegate_role(&mut w, camp, Role::TurnoutCoordinator, ms[0]);
                    let target = member(&mut w);
                    if staked { add_grievance(&mut w, target, Problem::Taxed, 200); }
                    let out = townhall(&mut w, camp, target, 10).unwrap();
                    let scale = out.turnout.len() as f32 / members as f32;
                    assert!(rank(out.response) >= last, "{} of {} did worse than fewer", out.turnout.len(), members);
                    last = rank(out.response);
                    if out.response == TargetResponse::Concedes {
                        assert!(staked && (0.2..0.5).contains(&scale), "a part-grant at {:.2}", scale);
                    }
                    if !staked { assert!(rank(out.response) == 0, "an unmoved target granted something"); }
                    table.push((members, out.turnout.len(), staked, out.response));
                }
            }
        }
        let concedes = table.iter().filter(|t| t.3 == TargetResponse::Concedes).count();
        let carries = table.iter().filter(|t| t.3 == TargetResponse::Carries).count();
        let staked = table.iter().filter(|t| t.2).count();
        eprintln!("townhall probe: of {} staked rooms, {} carried, {} conceded", staked, carries, concedes);
        assert!(concedes > 0 && concedes < carries, "part-grants should be the narrower band");
    }

    #[test]
    fn test_the_hollow_find_their_feet_in_about_a_month() {
        let mut w = World::new();
        let m = member(&mut w);
        w.add_component(m, Demoralized { since: 10 });
        assert!(!lifts_with_time(&mut w, m, 39), "the sting is still fresh");
        assert!(w.get_component::<Demoralized>(m).is_some());
        assert!(lifts_with_time(&mut w, m, 40), "a month on, they get up");
        assert!(w.get_component::<Demoralized>(m).is_none());
        // no mark, nothing to lift
        assert!(!lifts_with_time(&mut w, m, 400));
    }
}
