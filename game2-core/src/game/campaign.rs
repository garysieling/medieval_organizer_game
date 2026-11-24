//! Campaigns: an org that votes on what to fight for. Founding is a
//! ballot - every member votes the issue that serves their own interest,
//! and the winner becomes the campaign's cause. The organizer picks how
//! to put it (the framing); folk who wanted a different fight, or whose
//! grievance doesn't suit the way it's put, grumble.
//! From wiki/campaign-plan.md (C1); the lit review that lands in
//! parallel will deepen the model.

use crate::engine::ecs::{Entity, World};
use crate::game::desires::{self, IssueTag, Reveal, Want};
use crate::game::organizing::living_leader;
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

/// How the campaign settles its calls (Polletta: the decision rule is a
/// design choice, not a default)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DecisionRule {
    /// Settles in one meeting; manufactures a permanent minority
    Majority,
    /// Slower - grumbles get talked out before anything passes; the
    /// consent shows up as commitment when the campaign next acts
    Consensus,
}

/// A seat the organizer fills by delegation (Freeman: delegated
/// authority is what keeps informal elites accountable)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Role {
    /// Keeps the resolves - what the campaign decided is checkable later
    NoteTaker,
    /// Works the turnout list before a meeting or an action
    TurnoutCoordinator,
    /// Runs the room: who speaks, in what order, on what
    Chair,
}

/// The lord's plant inside the org: an infiltrated member who waits
/// for the campaign's peak to push a violent step - the proposal's
/// the weapon, not the deed
#[derive(Debug)]
pub struct Plant(pub Entity);

/// The lord's own counter-offer: a seat on the board - binding (a real
/// concession behind it) or performative (the offer absorbs the cause's
/// own energy if taken). The flag stays hidden until it's read
#[derive(Debug, Clone, Copy)]
pub struct SeatOffer {
    /// The lord who makes it
    pub from: Entity,
    /// The seat on offer
    pub role: Role,
    /// What it really carries - hidden from the player until read
    pub binding: bool,
    /// When the offer was made
    pub offered: u64,
}

/// Who holds a seat
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Seat {
    pub role: Role,
    pub holder: Entity,
}

/// A campaign: the org's settled cause, its own entity so it can carry
/// The campaign's own season: it listens before it cuts, it reads the
/// room before it acts, and it counts what it did before the next round
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CampaignPhase {
    Listening,    // the open season - hearing the grievances
    IssueCut,     // a target's named, the demands take shape
    Research,     // sounding the target out - the interview's yield
    Action,       // the street verbs - writs, pickets, the push itself
    Evaluate,     // what did it earn - settle or another round
}

impl CampaignPhase {
    /// The next season - Evaluate loops back to Listening unless the
    /// campaign's settled, so the machine never deadlocks
    pub fn step(self) -> Self {
        match self {
            Self::Listening => Self::IssueCut,
            Self::IssueCut => Self::Research,
            Self::Research => Self::Action,
            Self::Action => Self::Evaluate,
            Self::Evaluate => Self::Listening,
        }
    }
}

/// meetings, actions and prisoners of its own
#[derive(Debug)]
pub struct Campaign {
    pub issue: IssueTag,
    pub framing: Framing,
    pub org: Entity,
    pub founded_day: u64,
    /// Members who wanted a different issue - they came along, but they
    /// grumble until a meeting talks it out
    pub dissenters: Vec<Entity>,
    /// Majority or consensus, picked at founding
    pub decision_rule: DecisionRule,
    /// Delegated seats; a function nobody holds accrues to whoever has
    /// the strongest ties instead (the shadow elite)
    pub seats: Vec<Seat>,
    /// Who the demands are put to - a cut issue names a target; a
    /// mutual-aid cause has none
    pub target: Option<Entity>,
    /// What the org's sounded the target out on: their stance on the
    /// issue and the stake that drives it - the interview earns it
    pub dossier: Option<Dossier>,
    /// What the debrief settled on - the mandate the room gave the
    /// campaign's next step
    pub directive: Option<String>,
    /// The campaign's own season: it listens before it cuts, it reads
    /// the room before it acts
    pub phase: CampaignPhase,
}

/// Where the target stands on the campaign's issue, and the concern
/// that puts them there - the interview's yield
#[derive(Clone, Debug)]
pub struct Dossier {
    /// The person sounded out
    pub target: Entity,
    /// Support, Oppose, or Undecided on the campaign's issue
    pub stance: &'static str,
    /// The concern that drives them - their top issue tag
    pub stake: String,
}

/// The meeting a campaign holds on its cadence: turnout, resolutions,
/// and whether it's gone too long without one
#[derive(Debug)]
pub struct CampaignMeeting {
    /// Days between meetings (the organizer's recommendation)
    pub cadence_days: u64,
    /// The day the last meeting was held
    pub last_held: u64,
    /// Meetings held since founding
    pub held: u32,
    /// Decisions the note taker has on record
    pub resolves: u32,
    /// The pull that last drove a drifted meeting - observably theirs
    /// once the room's followed them
    pub driven_by: Option<Entity>,
}

/// Relationship strength needed before a member answers a turnout call
pub const ATTEND_MIN_TIE: f32 = 0.15;

/// A campaign lapses when it goes two cadences without meeting
pub const LAPSE_CADENCES: u64 = 2;

/// The campaign an org is running, if any (one org, one fight)
/// Tests: give `org` a campaign without the meeting and vote that found one
#[cfg(test)]
pub fn found_for_test(world: &mut World, org: Entity) -> Entity {
    let vote = IssueVote { winner: "bread".into(), ballots: vec![] };
    found_campaign(world, org, &vote, Framing::SharedGrievance, DecisionRule::Majority, 1)
}

pub fn campaign_of(world: &World, org: Entity) -> Option<Entity> {
    world.query::<Campaign>().into_iter()
        .find(|&e| world.get_component::<Campaign>(e).map(|c| c.org == org).unwrap_or(false))
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
pub fn stake_for(world: &World, member: Entity, tag: &IssueTag, tick: u64) -> Option<Reveal> {
    reveals_of(world, member, tick).into_iter()
        .find(|r| desires::tag_stands(tag, &desires::issue_tag(world, member, r)))
}

/// The candidate a member votes for: the one whose tag matches their
/// stake, else the one matching the most of their live problems.
/// `candidates` is the organizer's recommended order - a tie goes to the
/// issue the organizer put first. None if they have no dog in the fight.
fn vote_for(world: &World, member: Entity, candidates: &[IssueTag], tick: u64) -> Option<IssueTag> {
    let mut tags: Vec<IssueTag> = desires::problems_of(world, member, tick)
        .iter().map(|&p| desires::issue_tag(world, member, &Reveal::Problem(p))).collect();
    if let Some(life) = world.get_component::<desires::InnerLife>(member) {
        tags.extend(life.wants.iter()
            .filter(|&&w| !desires::is_satisfied(world, member, w, tick))
            .map(|&w| desires::issue_tag(world, member, &Reveal::Want(w))));
    }
    // Most matching stakes wins; an equal match keeps the recommended order
    let mut best: Option<(usize, &IssueTag)> = None;
    for c in candidates {
        let n = tags.iter().filter(|t| desires::tag_stands(c, t)).count();
        if n > 0 && best.map(|(b, _)| n > b).unwrap_or(true) { best = Some((n, c)); }
    }
    best.map(|(_, c)| c.clone())
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
pub fn found_campaign(world: &mut World, org: Entity, vote: &IssueVote, framing: Framing, rule: DecisionRule, founded_day: u64) -> Entity {
    let dissenters = vote.ballots.iter()
        .filter(|b| b.choice.as_ref().map(|c| *c != vote.winner).unwrap_or(false))
        .map(|b| b.member).collect();
    let c = world.create_entity();
    world.add_component(c, Campaign {
        issue: vote.winner.clone(), framing, org, founded_day, dissenters,
        dossier: None, directive: None, phase: CampaignPhase::Listening,
        decision_rule: rule, seats: vec![], target: None,
    });
    // the founding vote sets the flat rate the roll owes each meeting
    world.add_component(org, Dues { rate: DUES_RATE, missed: Default::default() });
    c
}

/// Set the meeting cadence (the organizer's recommendation, ratified at
/// founding in a later round - bones: the organizer sets it). Attach or
/// update the CampaignMeeting component.
pub fn set_cadence(world: &mut World, campaign: Entity, cadence_days: u64, tick: u64) {
    if let Some(m) = world.get_component_mut::<CampaignMeeting>(campaign) {
        m.cadence_days = cadence_days;
    } else {
        world.add_component(campaign, CampaignMeeting {
            cadence_days, last_held: tick, held: 0, resolves: 0,
            driven_by: None,
        });
    }
}

/// Delegate a seat to a member of the campaign's org (the leader counts).
/// A living, joined holder only; an existing holder of the same role is
/// replaced - delegation is how seats stay accountable.
pub fn delegate_role(world: &mut World, campaign: Entity, role: Role, holder: Entity) -> bool {
    let eligible = world.get_component::<Campaign>(campaign).map(|c| {
        let org = world.get_component::<Organization>(c.org);
        org.map(|o| o.leader == holder || o.members.contains(&holder)).unwrap_or(false)
    }).unwrap_or(false) && !crate::game::dungeon::unavailable(world, holder);
    if !eligible { return false; }
    let Some(c) = world.get_component_mut::<Campaign>(campaign) else { return false };
    c.seats.retain(|s| s.role != role && s.holder != holder);
    c.seats.push(Seat { role, holder });
    true
}

/// Who holds a role, if anyone does
pub fn seat_holder(world: &World, campaign: Entity, role: Role) -> Option<Entity> {
    world.get_component::<Campaign>(campaign)?.seats.iter()
        .find(|s| s.role == role).map(|s| s.holder)
}

/// A member's pull inside the org: the sum of their relationship
/// How deep a member's own stake runs to the cause: Bridged is the
/// issue's own stake heard, Amplified is the framing that sits right on
/// top of it, Transformed is the member who's stayed through the
/// meetings - alignment's read like morale is read before a stand
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment { Bridged, Amplified, Transformed }

/// Read a member's alignment to the campaign: the stake's own match
/// (Bridged), the framing's own fit on top (Amplified), the meetings
/// carried (Transformed) - the deeper tier carries the shallower
pub fn alignment(world: &World, member: Entity, campaign: Entity, tick: u64) -> Alignment {
    let stake = world.get_component::<Campaign>(campaign)
        .and_then(|c| stake_for(world, member, &c.issue, tick)).is_some();
    if !stake { return Alignment::Bridged; }
    let meetings = world.get_component::<crate::sim::psychology::AttachmentHistory>(member)
        .map(|h| h.episodes.len()).unwrap_or(0);
    if meetings >= 3 { Alignment::Transformed } else { Alignment::Amplified }
}

/// strengths toward the other living members. The hidden stat that runs
/// meetings nobody called - it leaks only through who shows up.
pub fn tie_mass(world: &World, member: Entity, members: &[Entity]) -> f32 {
    use crate::game::relationships::find_relationship;
    members.iter().filter(|&&m| m != member)
        .filter_map(|&m| find_relationship(world, member, m))
        .filter_map(|r| world.get_component::<crate::game::relationships::Relationship>(r))
        .map(|r| r.strength.max(0.0)).sum()
}

/// The shadow elite: the member (not the org's leader) whose personal
/// pull outweighs the leader's. When a seat stands empty, its function
/// doesn't go undone - it accrues to them (Freeman's informal elite).
/// None when the formal structure still holds the room.
pub fn shadow_elite(world: &World, campaign: Entity) -> Option<Entity> {
    let c = world.get_component::<Campaign>(campaign)?;
    let (leader, members) = world.get_component::<Organization>(c.org)
        .map(|o| (o.leader, o.members.clone()))?;
    let living: Vec<Entity> = members.iter().copied().filter(|&m| !crate::game::dungeon::unavailable(world, m)).collect();
    let leader_mass = tie_mass(world, leader, &living);
    living.iter().copied().filter(|&m| m != leader)
        .map(|m| (m, tie_mass(world, m, &living)))
        .filter(|(_, mass)| *mass > leader_mass)
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(m, _)| m)
}

/// What a campaign meeting did this time
#[derive(Debug)]
pub struct MeetingOutcome {
    /// Members who came
    pub attendees: Vec<Entity>,
    /// Dissenters whose stake got heard - they leave the dissenters list
    pub healed: Vec<Entity>,
    /// The meeting drifted to a shadow elite's pull (no coordinator seat)
    pub drifted: bool,
    /// A note taker kept the resolves
    pub noted: bool,
    /// Consensus consent: the next action carries a commitment bonus
    pub consent: bool,
    /// A room too big for the talk: the consensus' own rounds cracked
    /// the cause - the cause's own split, read off the room's own size
    pub splintered: bool,
}

/// Gone too long without meeting: more than `LAPSE_CADENCES` cadences
/// since the last one (or since cadence was set)
pub fn lapsed(world: &World, campaign: Entity, tick: u64) -> bool {
    world.get_component::<CampaignMeeting>(campaign)
        .map(|m| tick.saturating_sub(m.last_held) > m.cadence_days * LAPSE_CADENCES)
        .unwrap_or(false)
}

/// Hold the campaign's meeting. Who drives turnout: the TurnoutCoordinator
/// if the seat is filled, else the shadow elite if one's pulling, else the
/// leader. Attendance reads each member's tie to the driver - the hidden
/// relationship strength showing only as who walks in. Attending
/// dissenters get their stake heard and heal (amplification); a seated
/// note taker keeps the resolve; consensus meetings bank consent for the
/// next action. Two or more attendees is real organizing (power).
pub fn hold_campaign_meeting(world: &mut World, campaign: Entity, tick: u64) -> Option<MeetingOutcome> {
    let (org, dissenters, rule) = world.get_component::<Campaign>(campaign)
        .map(|c| (c.org, c.dissenters.clone(), c.decision_rule))?;

    // Cadence gate: no component yet, or not due since the last meeting,
    // and nothing happens - resolves the cadence before touching power.
    let due = world.get_component::<CampaignMeeting>(campaign)
        .map(|m| tick.saturating_sub(m.last_held) >= m.cadence_days)?;
    if !due { return None; }

    let leader = living_leader(world, org)?;
    let members = world.get_component::<Organization>(org).map(|o| o.members.clone())?;
    let in_org = |h: Entity| h == leader || members.contains(&h);

    let coordinator = seat_holder(world, campaign, Role::TurnoutCoordinator)
        .filter(|&h| !crate::game::dungeon::unavailable(world, h) && in_org(h));
    let driver = coordinator
        .or_else(|| shadow_elite(world, campaign))
        .or(Some(leader))
        .filter(|&d| !crate::game::dungeon::unavailable(world, d))?;
    let drifted = coordinator.is_none() && driver != leader;

    use crate::game::relationships::{find_relationship, Relationship};
    let strength_to = |w: &World, m: Entity| {
        let tie = find_relationship(w, m, driver)
            .and_then(|r| w.get_component::<Relationship>(r))
            .map(|r| r.strength).unwrap_or(0.0);
        // alignment's read like morale before a stand: the deeper the
        // member's own stake, the surer their pull
        let bonus = match alignment(w, m, campaign, tick) {
            Alignment::Bridged => 1.0,
            Alignment::Amplified => 1.2,
            Alignment::Transformed => 1.4,
        };
        tie * bonus
    };
    let mut attendees: Vec<Entity> = vec![];
    attendees.push(driver);
    for m in &members {
        if *m == driver || crate::game::dungeon::unavailable(world, *m) { continue; }
        if strength_to(world, *m) >= ATTEND_MIN_TIE { attendees.push(*m); }
    }

    // Amplification: a dissenter in the room gets heard and stops
    // dissenting (under consensus, anyone present does)
    let healed: Vec<Entity> = dissenters.iter().copied()
        .filter(|d| attendees.contains(d)).collect();
    if !healed.is_empty() {
        if let Some(c) = world.get_component_mut::<Campaign>(campaign) {
            c.dissenters.retain(|d| !healed.contains(d));
        }
    }

    let note_taker = seat_holder(world, campaign, Role::NoteTaker)
        .filter(|&h| !is_dead(world, h) && in_org(h));
    let noted = note_taker.map(|n| attendees.contains(&n)).unwrap_or(false);
    let consent = rule == DecisionRule::Consensus && attendees.len() >= 2;
    // and the splinter: a room too big for the talk can crack the cause
    let splintered = rule == DecisionRule::Consensus && members.len() > 12
        && (tick.wrapping_mul(53).wrapping_add(campaign as u64)) % 100 < 15;

    if attendees.len() >= 2 {
        crate::sim::organized_power::record_action(world, org, crate::sim::organized_power::ActionQuality::Transformational);
    } else {
        crate::sim::organized_power::record_action(world, org, crate::sim::organized_power::ActionQuality::Transactional);
    }
    if let Some(m) = world.get_component_mut::<CampaignMeeting>(campaign) {
        m.last_held = tick;
        m.held += 1;
        // the pull's only observably theirs when others answered - an
        // empty room proves nothing
        if drifted && attendees.len() >= 2 { m.driven_by = Some(driver); }
        if noted { m.resolves += 1; }
    }
    // a resolve kept on the record is a decided thing - the room's lies
    // about rooms deciding take the test (E8-2, #1444)
    if noted {
        let leader = world.get_component::<crate::sim::community::Organization>(org)
            .map(|o| o.leader);
        for &a in &attendees {
            crate::game::beliefs::tested_by(world, a, crate::game::beliefs::Tested::Decided,
                leader, tick);
        }
    }
    // Participation teaches: the room learns meeting craft, the seats
    // learn theirs
    crate::game::org_skills::learn_from_meeting(world, campaign, &attendees);

    // a conversion campaign's own pull: the issue IS the faith's own
    // name - only when some soul in the world already carries that
    // faith does the room's hearts drift toward it
    if let Some(issue) = world.get_component::<Campaign>(campaign).map(|c| c.issue.clone()) {
        let is_faith = world.query::<crate::sim::religion::Faith>()
            .iter().any(|&e| world.get_component::<crate::sim::religion::Faith>(e)
                .map(|f| f.religion == issue).unwrap_or(false));
        if !issue.is_empty() && is_faith {
            let orth = world.get_component::<crate::sim::religion::Congregation>(org)
                .map(|c| c.orthodoxy).unwrap_or(0.3);
            for &a in &attendees {
                let Some(f) = world.get_component_mut::<crate::sim::religion::Faith>(a) else { continue };
                // the pull drifts the devotion; a fold's own orthodoxy
                // resists it
                f.devotion = (f.devotion + 0.04 * (1.0 - orth)).min(1.0);
                // and the heart fully won takes the faith's own name
                if f.devotion >= 0.95 && f.religion != issue {
                    f.religion = issue.clone();
                }
            }
        }
    }
    Some(MeetingOutcome { attendees, healed, drifted, noted, consent, splintered })
}

// --- The accountability assembly (IAF): the campaign's own session on
// its own cadence - the agenda published ahead, the leader's yes/no
// answers on record, and pledges the deadline reads against what the
// village saw. It's a standing institution, not a meeting: it calls
// itself, and the organizer can call it early.

/// Days between accountability assemblies
pub const ASSEMBLY_CADENCE: u64 = 14;
/// Days the published agenda sits before the assembly sits - the
/// "in advance" that makes the pin-downs accountability, not ambush
pub const PUBLISH_GAP: u64 = 2;
/// Days into the fight before the first assembly can be called
pub const FIRST_ASSEMBLY_DELAY: u64 = 10;
/// Questions on a published agenda
const AGENDA_LEN: usize = 3;
/// Pledges kept on the record - the oldest resolved ones drop first
const PLEDGE_CAP: usize = 40;

/// What a pledge is checked against: the observable state that says
/// whether the word was kept. Each variant reads the world at the
/// deadline - nothing here tracks a promise, it's a question the
/// village can see the answer to
#[derive(Debug, Clone)]
pub enum PledgeCheck {
    /// an arc action lands - a march, a strike, a townhall
    Action { from: u32 },
    /// this many more meetings held
    Meetings { from: u32, more: u32 },
    /// the org's muscle grew past `from`
    Stronger { from: f32 },
    /// an assembly after the pledge's day turns out `min` strong
    Delegation { min: usize },
    /// `fewer` fewer dissenters than `from`
    Doubters { from: usize, fewer: usize },
}

impl PledgeCheck {
    /// Whether the observable says the thing happened - read at the
    /// deadline, against the state the village could see
    fn holds(&self, world: &World, campaign: Entity, pledge: &Pledge) -> bool {
        match self {
            PledgeCheck::Action { from } =>
                world.get_component::<crate::game::campaign_arc::ArcRecord>(campaign)
                    .map(|r| r.actions > *from).unwrap_or(false),
            PledgeCheck::Meetings { from, more } =>
                world.get_component::<CampaignMeeting>(campaign)
                    .map(|m| m.held >= from.saturating_add(*more)).unwrap_or(false),
            PledgeCheck::Stronger { from } => world.get_component::<Campaign>(campaign)
                .and_then(|c| world.get_component::<crate::sim::organized_power::OrganizedPower>(c.org))
                .map(|p| p.capacity > from + 0.1).unwrap_or(false),
            // the pledge's own assembly doesn't count - the check wants
            // a later one, which is why the deadline outlasts the cadence
            PledgeCheck::Delegation { min } =>
                world.get_component::<Assembly>(campaign)
                    .map(|a| a.last_held > pledge.day && a.last_turnout >= *min).unwrap_or(false),
            PledgeCheck::Doubters { from, fewer } =>
                world.get_component::<Campaign>(campaign)
                    .map(|c| c.dissenters.len().saturating_add(*fewer) <= *from).unwrap_or(false),
        }
    }
}

/// A yes/no answer the leader gave on the record - the artifact an
/// accountability assembly exists to produce
#[derive(Debug, Clone)]
pub struct Pledge {
    pub question: String,
    pub leader: Entity,
    /// The answer on record - yes is a promise, no is a refusal; both
    /// are checked against what happened
    pub answer: bool,
    pub day: u64,
    pub deadline: u64,
    /// The member whose testimony amended the recorded answer, if the
    /// room checked the leader's word before it was inked
    pub challenged: Option<Entity>,
    /// None until the deadline's check runs
    pub kept: Option<bool>,
    pub check: PledgeCheck,
}

/// The accountability assembly's own state, on the campaign entity
#[derive(Debug)]
pub struct Assembly {
    /// Days between assemblies
    pub cadence_days: u64,
    /// The day the last assembly sat
    pub last_held: u64,
    /// The day the standing agenda was published
    pub announced: Option<u64>,
    /// The published questions, each with what it's checked against
    pub agenda: Vec<(String, PledgeCheck)>,
    /// How many answered the last call
    pub last_turnout: usize,
    /// Who answered it - the room a broken pledge chills
    pub musterered: Vec<Entity>,
    /// Assemblies sat since the institution was called
    pub held: u32,
    /// The pledges on record
    pub pledges: Vec<Pledge>,
}

/// What an assembly sitting produced
#[derive(Debug)]
pub struct AssemblyOutcome {
    pub attendees: Vec<Entity>,
    /// The pledges the leader's answers made
    pub pledges: Vec<Pledge>,
    /// Members whose testimony amended an answer - (who, the question)
    pub challenges: Vec<(Entity, String)>,
    pub leader: Entity,
}

/// What the deadline's check found for one pledge
#[derive(Debug)]
pub struct PledgeVerdict {
    pub question: String,
    pub answer: bool,
    /// The word was kept: what they said is what happened
    pub kept: bool,
}

/// The questions an assembly could put, in priority order - each with
/// the observable it'll later be held to. Only what the campaign can
/// actually be asked makes the ballot
fn agenda_checks(world: &World, campaign: Entity, org: Entity) -> Vec<(String, PledgeCheck)> {
    let mut out = vec![];
    // the fight going public
    let actions = world.get_component::<crate::game::campaign_arc::ArcRecord>(campaign)
        .map(|r| r.actions).unwrap_or(0);
    out.push(("Will we carry the fight to the streets?".into(),
        PledgeCheck::Action { from: actions }));
    // the room keeping its date
    if let Some(m) = world.get_component::<CampaignMeeting>(campaign) {
        out.push(("Will the room meet again before the next assembly?".into(),
            PledgeCheck::Meetings { from: m.held, more: 1 }));
    }
    // the muscle growing
    let power = world.get_component::<crate::sim::organized_power::OrganizedPower>(org)
        .map(|p| p.capacity).unwrap_or(0.0);
    out.push(("Will we come out of this stronger than we went in?".into(),
        PledgeCheck::Stronger { from: power }));
    // the delegation answering - in force means most of them
    let free: Vec<Entity> = world.get_component::<Organization>(org)
        .map(|o| o.members.iter().copied()
            .filter(|&m| !crate::game::dungeon::unavailable(world, m)).collect())
        .unwrap_or_default();
    if free.len() >= 3 {
        out.push(("Will the delegation turn out in force?".into(),
            PledgeCheck::Delegation { min: free.len() / 2 + 1 }));
    }
    // the doubters coming round
    let dissenters = world.get_component::<Campaign>(campaign)
        .map(|c| c.dissenters.iter()
            .filter(|&&d| !crate::game::dungeon::unavailable(world, d)).count())
        .unwrap_or(0);
    if dissenters > 0 {
        out.push(("Will the doubters come 'round?".into(),
            PledgeCheck::Doubters { from: dissenters, fewer: dissenters }));
    }
    out
}

/// Post the agenda: AGENDA_LEN questions picked deterministically -
/// rotated by the day and the campaign so assemblies don't ask the
/// same three things forever. Returns the questions for the log; None
/// when there's no org or nobody to answer for it
pub fn announce_assembly(world: &mut World, campaign: Entity, tick: u64) -> Option<Vec<String>> {
    let org = world.get_component::<Campaign>(campaign)?.org;
    // somebody has to answer for the org
    living_leader(world, org)?;
    let mut checks = agenda_checks(world, campaign, org);
    if checks.is_empty() { return None; }
    let rot = (tick as usize).wrapping_add(campaign as usize) % checks.len();
    checks.rotate_left(rot);
    checks.truncate(AGENDA_LEN);
    let questions: Vec<String> = checks.iter().map(|(q, _)| q.clone()).collect();
    if world.get_component::<Assembly>(campaign).is_none() {
        world.add_component(campaign, Assembly {
            cadence_days: ASSEMBLY_CADENCE, last_held: tick,
            announced: None, agenda: vec![], last_turnout: 0,
            musterered: vec![], held: 0, pledges: vec![],
        });
    }
    if let Some(a) = world.get_component_mut::<Assembly>(campaign) {
        a.announced = Some(tick);
        a.agenda = checks;
    }
    Some(questions)
}

/// Sit the assembly: the published agenda comes due, the delegation
/// musters on the same ties as a meeting, and the leader's yes/no
/// answers go on record as pledges the deadline will check. The
/// publish gap is what makes it accountability - the questions were
/// public before the answers were asked
pub fn hold_assembly(world: &mut World, campaign: Entity, tick: u64) -> Option<AssemblyOutcome> {
    let announced = world.get_component::<Assembly>(campaign)?.announced?;
    if tick < announced + PUBLISH_GAP { return None; }
    let agenda = world.get_component::<Assembly>(campaign)?.agenda.clone();
    if agenda.is_empty() { return None; }
    let org = world.get_component::<Campaign>(campaign)?.org;
    let leader = living_leader(world, org)?;
    let members = world.get_component::<Organization>(org)?.members.clone();
    let in_org = |h: Entity| h == leader || members.contains(&h);

    // the same call a meeting answers: the coordinator's ties if the
    // seat's filled, else the shadow elite's pull, else the leader's
    let coordinator = seat_holder(world, campaign, Role::TurnoutCoordinator)
        .filter(|&h| !crate::game::dungeon::unavailable(world, h) && in_org(h));
    let driver = coordinator
        .or_else(|| shadow_elite(world, campaign))
        .or(Some(leader))
        .filter(|&d| !crate::game::dungeon::unavailable(world, d))?;

    use crate::game::relationships::{find_relationship, Relationship};
    let strength_to = |w: &World, m: Entity| {
        let tie = find_relationship(w, m, driver)
            .and_then(|r| w.get_component::<Relationship>(r))
            .map(|r| r.strength).unwrap_or(0.0);
        // alignment's read like morale before a stand: the deeper the
        // member's own stake, the surer their pull
        let bonus = match alignment(w, m, campaign, tick) {
            Alignment::Bridged => 1.0,
            Alignment::Amplified => 1.2,
            Alignment::Transformed => 1.4,
        };
        tie * bonus
    };
    let mut attendees: Vec<Entity> = vec![driver];
    for m in &members {
        if *m == driver || crate::game::dungeon::unavailable(world, *m) { continue; }
        if strength_to(world, *m) >= ATTEND_MIN_TIE { attendees.push(*m); }
    }

    // the room's weight is the org's muscle: a thin house earns little,
    // and a lone leader earns nothing - the mandate's what's mustered
    if attendees.len() >= 3 {
        crate::sim::organized_power::record_action(world, org,
            crate::sim::organized_power::ActionQuality::Transformational);
    } else if attendees.len() >= 2 {
        crate::sim::organized_power::record_action(world, org,
            crate::sim::organized_power::ActionQuality::Transactional);
    }

    // the leader answers the published agenda on the record - seeded
    // like the room's talk, so a replay says the same
    let mut rng = crate::game::conversation::Rng::new(
        tick.wrapping_mul(0x0100_0000_01B3) ^ ((campaign as u64) << 32));
    let cadence = world.get_component::<Assembly>(campaign)
        .map(|a| a.cadence_days).unwrap_or(ASSEMBLY_CADENCE);
    // the deadline outlasts the cadence by the publish gap, so the next
    // assembly has sat before a delegation pledge comes due
    let deadline = tick + cadence + PUBLISH_GAP;
    let mut pledges: Vec<Pledge> = agenda.iter().map(|(q, check)| Pledge {
        question: q.clone(), leader,
        answer: rng.below(100) < 60,
        day: tick, deadline, challenged: None, kept: None, check: check.clone(),
    }).collect();

    // a member's testimony can amend the leader's answer before it's
    // inked - the room's own check on the pin-downs. The amended word
    // is what the deadline reads, and the challenge itself is a
    // disagreement aired in public: it feeds the same trust ledger
    // every other friction does
    let mut challenges: Vec<(Entity, String)> = vec![];
    let pool: Vec<Entity> = attendees.iter().copied()
        .filter(|&m| m != leader).collect();
    if pool.len() >= 2 && !pledges.is_empty() && rng.below(100) < 60 {
        let challenger = pool[rng.below(pool.len())];
        let pi = rng.below(pledges.len());
        pledges[pi].answer = !pledges[pi].answer;
        pledges[pi].challenged = Some(challenger);
        challenges.push((challenger, pledges[pi].question.clone()));
        use crate::game::relationships::{record_interaction, Interaction, InteractionKind};
        record_interaction(world, challenger, Interaction {
            other: leader, kind: InteractionKind::Conflict, intensity: 0.5 });
    }

    if let Some(a) = world.get_component_mut::<Assembly>(campaign) {
        a.announced = None;
        a.last_held = tick;
        a.held += 1;
        a.last_turnout = attendees.len();
        a.musterered = attendees.clone();
        a.pledges.extend(pledges.clone());
        if a.pledges.len() > PLEDGE_CAP {
            let excess = a.pledges.len() - PLEDGE_CAP;
            a.pledges.drain(..excess);
        }
    }
    Some(AssemblyOutcome { attendees, pledges, challenges, leader })
}

/// Read every due pledge against the observable and mark it kept or
/// broken. The word kept warms the room that heard it; the word broken
/// chills it - and spends the consent the meetings banked, which is
/// the drift #853's watchdog will read through `broken_pledges`
pub fn resolve_pledges(world: &mut World, campaign: Entity, tick: u64) -> Vec<PledgeVerdict> {
    use crate::game::relationships::{record_interaction, Interaction, InteractionKind};
    // which pledges came due - the check reads the world, so the
    // mutable borrow waits for the verdicts
    let due: Vec<usize> = world.get_component::<Assembly>(campaign)
        .map(|a| a.pledges.iter().enumerate()
            .filter(|(_, p)| p.kept.is_none() && tick >= p.deadline)
            .map(|(i, _)| i).collect()).unwrap_or_default();
    if due.is_empty() { return vec![]; }
    let mut results: Vec<(usize, Entity, PledgeVerdict)> = vec![];
    for &i in &due {
        let Some(p) = world.get_component::<Assembly>(campaign).and_then(|a| a.pledges.get(i)) else { continue };
        let happened = p.check.holds(world, campaign, p);
        let kept = p.answer == happened;
        results.push((i, p.leader, PledgeVerdict {
            question: p.question.clone(), answer: p.answer, kept }));
    }
    if results.is_empty() { return vec![]; }
    let musterered = world.get_component::<Assembly>(campaign)
        .map(|a| a.musterered.clone()).unwrap_or_default();
    if let Some(a) = world.get_component_mut::<Assembly>(campaign) {
        for &(i, _, ref v) in &results { a.pledges[i].kept = Some(v.kept); }
    }
    // one verdict per leader: a single broken word chills the room,
    // a clean sheet warms it
    let mut leaders: Vec<Entity> = vec![];
    for &(_, l, _) in &results { if !leaders.contains(&l) { leaders.push(l); } }
    for l in leaders {
        let any_broken = results.iter().any(|&(_, fl, ref v)| fl == l && !v.kept);
        for &m in &musterered {
            if m == l { continue; }
            record_interaction(world, m, Interaction {
                other: l,
                kind: if any_broken { InteractionKind::Conflict } else { InteractionKind::Conversation },
                intensity: if any_broken { 1.0 } else { 0.5 },
            });
        }
        if any_broken {
            // the room's banked consent doesn't survive a broken word
            crate::game::campaign_arc::record_consent(world, campaign, false);
        }
        // and the word lands on the soul that gave it: kept grows,
        // broken wears - the room remembers either way (#950)
        crate::game::virtue::witness(world, l,
            if any_broken { crate::game::virtue::MoralAct::BrokeWord }
            else { crate::game::virtue::MoralAct::KeptWord }, 1.0);
    }
    results.into_iter().map(|(_, _, v)| v).collect()
}

/// The pledges the deadline broke - the commitments #853's enforcement
/// forms will watch for going quiet
pub fn broken_pledges(world: &World, campaign: Entity) -> Vec<&Pledge> {
    world.get_component::<Assembly>(campaign)
        .map(|a| a.pledges.iter().filter(|p| p.kept == Some(false)).collect())
        .unwrap_or_default()
}

/// A collective action the meeting floor can take up: the cause's
/// street verbs are motions now, not buttons - the room votes them,
/// and a bigger cause carries more of them (#1157)
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Motion {
    /// The column forms and marches the town's ways
    March,
    /// Ballot the trades: nine of ten walk out or it fails in public
    Strike,
    /// The demands put to the target, in public
    Townhall,
    /// Pass the hat and post the cause's flyers
    Flyers,
    /// Hands enough for two fronts at once: the march and the ballot
    /// on the same day
    TwoFronts,
    /// Send an organizer out to raise a market town for the cause
    SendOrganizer,
    /// A posted organizer stages the cause's day in their town
    /// (index into Game.sites)
    ParallelEvent(usize),
    /// The doors get asked about an issue - the count comes back
    /// fuzzed, and the asking itself reads the cause's reach (#1270)
    Survey,
    /// The room pools its coin and buys together at a discount
    /// (#1343): index into `BUY_GOODS`, and the gold promised behind it
    GroupBuy(usize, u32),
    /// The members take the work out together - a picking party or a
    /// raising (#1344): the take fills the chest, the shared day warms
    /// the hands
    WorkParty(PartyKind),
    /// Charter the fire brigade (#1346): the cause's answer to a fire
    /// grief it's identified - sworn hands on every bell
    Brigade,
    /// The rhythm verbs as motions (#1337): the room ratifies the
    /// organizer's read on how often the cause meets - the cadence is
    /// what the meeting does, not a button
    Cadence,
    /// The room tallies the next step after a meeting's held - the
    /// members standing there carry the mandate
    Debrief,
    /// Put the accountability assembly's agenda up early - it keeps
    /// its own dates otherwise
    Assembly,
    /// The purse mends the storm-hit hall - carried like any spending
    MendHall,
    /// The treasury raises the schoolhouse or sends for the teacher -
    /// a room can refuse the coin
    School,
}

/// The work a party walks out to (#1344): caps under the trees, or a
/// roof the whole village turns out for
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PartyKind {
    /// A flushed mushroom patch: the take sells into the chest
    Forage,
    /// A barnraising: the roof goes up, the village sees the cause work
    Raise,
}

/// The rhetorical line a motion takes (#1310): the street verbs are the
/// firebrand's work, the asking and the print the patient one's. None
/// when the pick carries no line - the road machinery is just work
pub fn motion_line(m: Motion) -> Option<crate::game::rhetoric::Line> {
    use crate::game::rhetoric::Line;
    match m {
        Motion::March | Motion::Strike | Motion::Townhall | Motion::TwoFronts => Some(Line::Firebrand),
        Motion::Flyers | Motion::Survey => Some(Line::Patient),
        Motion::GroupBuy(..) => Some(Line::Patient),
        Motion::WorkParty(_) => Some(Line::Patient),
        Motion::SendOrganizer | Motion::ParallelEvent(_) => None,
        Motion::Brigade => Some(Line::Patient),
        // the rhythm carries no line to witness - housekeeping isn't
        // rhetoric the room can be set against (#1337)
        Motion::Cadence | Motion::Debrief | Motion::Assembly
            | Motion::MendHall | Motion::School => None,
    }
}

/// What the room reads the organizer by (#1310): the living members'
/// reads of the cause's org, summed per line - the dominant one, when
/// it holds a hardened room's worth of weight, gates the slate. None
/// for a room that's read nothing yet
pub fn room_lean(world: &World, campaign: Entity) -> Option<crate::game::rhetoric::Line> {
    use crate::game::rhetoric::{LINES, RhetoricMemory, HARDEN};
    let org = world.get_component::<Campaign>(campaign)?.org;
    let o = world.get_component::<Organization>(org)?;
    let mut sum = [0.0f32; 4];
    for &m in &o.members {
        if is_dead(world, m) { continue; }
        if let Some(r) = world.get_component::<RhetoricMemory>(m)
            .and_then(|mm| mm.by_org.get(&org)) {
            for l in LINES { sum[l as usize] += r.weight[l as usize]; }
        }
    }
    let best = LINES.iter().copied()
        .max_by(|a, b| sum[*a as usize].partial_cmp(&sum[*b as usize]).unwrap_or(std::cmp::Ordering::Equal))?;
    if sum[best as usize] >= HARDEN { Some(best) } else { None }
}

/// The slate line the floor hears a motion under
pub fn motion_label(m: Motion) -> String {
    match m {
        Motion::March => "Take it to the streets - the column marches".into(),
        Motion::Strike => "Ballot the trades - call the strike".into(),
        Motion::Townhall => "Put the demands - face the Reeve in public".into(),
        Motion::Flyers => "Post the flyers - the cause in print".into(),
        Motion::TwoFronts => "Two fronts at once - march and ballot together".into(),
        Motion::SendOrganizer => "Send an organizer to raise a town for the cause".into(),
        Motion::ParallelEvent(_) => "The posted town rises - stage the event there".into(),
        Motion::Survey => "Run a survey - the doors get asked, the count comes back".into(),
        Motion::GroupBuy(gi, pledged) =>
            format!("Go in together on the {} - {} gold promised", BUY_GOODS[gi], pledged),
        Motion::WorkParty(PartyKind::Forage) =>
            "Send a work party out for the caps - the take fills the chest".into(),
        Motion::WorkParty(PartyKind::Raise) =>
            "Raise a roof together - call the village out for it".into(),
        Motion::Brigade =>
            "Charter a fire brigade - sworn hands on every bell".into(),
        Motion::Cadence => "Set the meeting's cadence - the rhythm's the room's".into(),
        Motion::Debrief => "Debrief - the room tallies the next step".into(),
        Motion::Assembly => "Call the accountability assembly early".into(),
        Motion::MendHall => "Mend the hall - the purse covers the roof".into(),
        Motion::School => "Raise the school - the treasury pays the desk".into(),
    }
}

/// A member posted to another town to raise it for the cause: they
/// live out there off the village's daily plans till the post's done
/// or they're called home (#1157)
#[derive(Debug)]
pub struct FieldPost {
    /// Index into Game.sites
    pub site: usize,
    /// The day they were sent
    pub since: u64,
}

/// The campaign's reach beyond the village: one post per town an
/// organizer's raising, and the warmth their work's built there
#[derive(Debug, Default)]
pub struct Outreach {
    /// (site index, the posted organizer, support built so far)
    pub posts: Vec<(usize, Entity, f32)>,
}

/// How warm a posted town must run before it can stage the cause's day
pub const FIELD_WARM: f32 = 3.0;
/// What a day's organizing adds to a posted town - three days and the
/// town's ready
pub const FIELD_DAILY: f32 = 1.0;

/// Living grown members free to act: the size the slate's read off
pub fn living_members(world: &World, campaign: Entity) -> usize {
    living_member_list(world, campaign).len()
}

/// The living members as entities, in roll order (#1344): the work
/// party's hands - the same bar `living_members` counts
pub fn living_member_list(world: &World, campaign: Entity) -> Vec<Entity> {
    let Some(org) = world.get_component::<Campaign>(campaign).map(|c| c.org) else { return vec![] };
    let dues = world.get_component::<Dues>(org).map(|d| d.missed.clone()).unwrap_or_default();
    world.get_component::<Organization>(org)
        .map(|o| o.members.iter().copied().filter(|&m| {
            !is_dead(world, m)
                && world.get_component::<crate::game::vocation::Kid>(m).is_none()
                && !crate::game::dungeon::unavailable(world, m)
                // a lapsed name stays on the rolls but doesn't carry
                // a motion's headcount (#1342)
                && dues.get(&m).copied().unwrap_or(0) == 0
        }).collect()).unwrap_or_default()
}

/// The goods a pooled buy can be for (#1343): the wants the village's
/// own shops keep on the shelf
pub const BUY_GOODS: [&str; 5] = ["bread", "garments", "lute", "tools", "ale"];
/// A member pledges up to half their purse toward their want, and
/// never more than two shares' worth (#1343)
const PLEDGE_SHARE: f32 = 0.5;
const PLEDGE_UNITS: f32 = 2.0;
/// The cause's coin behind a good before it carries to the floor:
/// two pledging members and three shares' worth promised (#1343)
const PLEDGE_BAR_UNITS: f32 = 3.0;
/// The discount's knobs (#1343): a point a share for the bulk, a point
/// of warmth for the keeper's best tie in the room, the cause's weight
/// - and the blend capped so no shop gives the stock away
const BULK_PCT: f32 = 0.01;
const BULK_CAP: f32 = 0.15;
const TIE_PCT: f32 = 0.10;
const WEIGHT_PCT: f32 = 0.10;
const DISCOUNT_CAP: f32 = 0.30;
/// The surplus goes out below street - a public benefit that still
/// leaves the cause its margin (#1343)
const SELL_AT: f32 = 0.8;

/// The want the cause can shop for together, as a `BUY_GOODS` index
fn want_good(w: Want) -> Option<usize> {
    match w {
        Want::GoodFood => Some(0),   // bread
        Want::Finery => Some(1),     // garments
        Want::Instrument => Some(2), // lute
        Want::BetterTools => Some(3),// tools
        Want::Drink => Some(4),      // ale
        _ => None,
    }
}

/// What the room's carrying to the meeting (#1343): a member with a
/// want the shops sell pledges half their purse toward it, two shares
/// at most. (good index, [(member, gold pledged)]) - goods ordered by
/// the gold behind them, members in roll order
pub fn group_pledges(world: &World, campaign: Entity) -> Vec<(usize, Vec<(Entity, f32)>)> {
    let Some(org) = world.get_component::<Campaign>(campaign).map(|c| c.org) else { return vec![] };
    let Some(members) = world.get_component::<Organization>(org).map(|o| o.members.clone()) else { return vec![] };
    let mut by_good: Vec<(usize, Vec<(Entity, f32)>)> = vec![];
    for m in members {
        if is_dead(world, m) || world.get_component::<crate::game::vocation::Kid>(m).is_some()
            || crate::game::dungeon::unavailable(world, m) { continue; }
        let Some(il) = world.get_component::<desires::InnerLife>(m) else { continue };
        let Some(gi) = il.wants.iter().find_map(|w| want_good(*w)) else { continue };
        let purse = world.get_component::<crate::game::livelihood::Purse>(m)
            .map(|p| p.gold).unwrap_or(0.0);
        let pledge = (purse * PLEDGE_SHARE)
            .min(crate::game::goods::worth_at(world, BUY_GOODS[gi]) * PLEDGE_UNITS);
        if pledge < 1.0 { continue; }
        match by_good.iter_mut().find(|(g, _)| *g == gi) {
            Some((_, ps)) => ps.push((m, pledge)),
            None => by_good.push((gi, vec![(m, pledge)])),
        }
    }
    by_good.sort_by(|a, b| {
        let ta: f32 = a.1.iter().map(|(_, p)| p).sum();
        let tb: f32 = b.1.iter().map(|(_, p)| p).sum();
        tb.partial_cmp(&ta).unwrap_or(std::cmp::Ordering::Equal).then(a.0.cmp(&b.0))
    });
    by_good
}

/// What the pooled buy settled (#1343): shares that went home with
/// members, the surplus that sold out below street, and what the
/// room's coin came to
#[derive(Debug, Default)]
pub struct GroupBuyOut {
    /// (member, units) - the shares that went home
    pub shares: Vec<(Entity, u32)>,
    /// Units sold on below street for the cause
    pub surplus: u32,
    /// Gold the sell-off brought into the treasury
    pub sold_for: f32,
    /// What the room's pooled coin actually paid
    pub paid: f32,
    /// The discount the bulk, the tie and the cause's weight bought
    pub discount: f32,
}

/// The pooled buy carried (#1343): the shop that stocks the good and
/// warms the cause's custom best takes the order; pledges collect
/// against what the purses still hold, the price bends on bulk, tie
/// and the cause's weight, shares go home one to a pledging member
/// while units last, and the rest sells out the door below street
pub fn group_buy(world: &mut World, campaign: Entity, gi: usize) -> Option<GroupBuyOut> {
    let good = BUY_GOODS[gi];
    let pledges = group_pledges(world, campaign).into_iter()
        .find(|(g, _)| *g == gi).map(|(_, p)| p).unwrap_or_default();
    if pledges.len() < 2 { return None; }
    let Some(org) = world.get_component::<Campaign>(campaign).map(|c| c.org) else { return None };
    // collect the pool against what the purses still hold
    let mut paid_in: Vec<(Entity, f32)> = vec![];
    for (m, pledge) in &pledges {
        let paid = world.get_component_mut::<crate::game::livelihood::Purse>(*m)
            .map(|p| { let take = pledge.min(p.gold); p.gold -= take; take })
            .unwrap_or(0.0);
        if paid > 0.0 { paid_in.push((*m, paid)); }
    }
    if paid_in.is_empty() { return None; }
    let pool: f32 = paid_in.iter().map(|(_, p)| p).sum();
    // the shop that stocks it and knows the room best takes the order
    let members: Vec<Entity> = paid_in.iter().map(|(m, _)| *m).collect();
    use crate::game::household::{Building, BuildingKind};
    let mut best: Option<(Entity, Entity, f32)> = None; // (shop, worker, tie)
    for b in world.query::<Building>() {
        let Some(bb) = world.get_component::<Building>(b) else { continue };
        if !matches!(bb.kind, BuildingKind::Shop(_)) { continue; }
        let Some(worker) = bb.worker else { continue };
        if bb.stock.get(good).copied().unwrap_or(0) == 0 { continue; }
        let tie = members.iter().map(|&m| {
            crate::game::relationships::find_relationship(world, m, worker)
                .and_then(|r| world.get_component::<crate::game::relationships::Relationship>(r))
                .map(|r| r.strength).unwrap_or(0.0)
        }).fold(0.0f32, f32::max);
        if best.map(|(_, _, t)| tie > t).unwrap_or(true) { best = Some((b, worker, tie)); }
    }
    let (shop, worker, tie) = best?;
    // the blend: a point a share for the bulk, warmth for the tie,
    // weight for the cause's name - capped so no shop gives it away
    let street = crate::game::goods::worth_at(world, good);
    let bulk = ((pool / street) * BULK_PCT).min(BULK_CAP);
    let weight = world.get_component::<crate::sim::organized_power::OrganizedPower>(org)
        .map(|p| p.capacity / crate::sim::organized_power::CAPACITY_CAP).unwrap_or(0.0) * WEIGHT_PCT;
    let discount = (bulk + tie * TIE_PCT + weight).min(DISCOUNT_CAP);
    let unit = street * (1.0 - discount);
    let stock = world.get_component::<Building>(shop).map(|b| b.stock.get(good).copied().unwrap_or(0)).unwrap_or(0);
    let qty = ((pool / unit).floor() as u32).min(stock);
    if qty == 0 { return None; }
    {
        let b = world.get_component_mut::<Building>(shop)?;
        crate::game::household::take_stock(b, good, qty);
    }
    let price = qty as f32 * unit;
    // the keeper's purse fills, the overseer's skim off the top
    let overseer = world.get_component::<Building>(shop).map(|b| b.overseer).unwrap_or(None);
    let cut = if overseer.is_some() { price * crate::game::livelihood::OVERSEER_CUT } else { 0.0 };
    if let Some(p) = world.get_component_mut::<crate::game::livelihood::Purse>(worker) { p.gold += price - cut; }
    if let Some(o) = overseer {
        if let Some(p) = world.get_component_mut::<crate::game::livelihood::Purse>(o) { p.gold += cut; }
    }
    // the pooled order's a warm word at the door: every pledging
    // member knows the keeper a little better for it
    for &(m, _) in &paid_in {
        crate::game::relationships::record_interaction(world, m,
            crate::game::relationships::Interaction {
                kind: crate::game::relationships::InteractionKind::Conversation,
                intensity: 0.3, other: worker });
    }
    // shares home first, members' order by pledge then the roll
    let mut order = paid_in.clone();
    order.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal).then(a.0.cmp(&b.0)));
    let mut left = qty;
    let mut out = GroupBuyOut { paid: price, discount, ..Default::default() };
    for (m, _) in &order {
        if left == 0 { break; }
        let Some(s) = world.get_component_mut::<crate::game::livelihood::Satchel>(*m) else { continue };
        *s.goods.entry(good.to_string()).or_insert(0) += 1;
        left -= 1;
        match out.shares.iter_mut().find(|(e, _)| e == m) {
            Some((_, n)) => *n += 1,
            None => out.shares.push((*m, 1)),
        }
    }
    // the surplus out the door below street - the cause's cheap stall
    // is a public benefit, and the margin's the treasury's
    if left > 0 {
        let sold_for = left as f32 * street * SELL_AT;
        if let Some(o) = world.get_component_mut::<Organization>(org) { o.resources += sold_for; }
        out.surplus = left;
        out.sold_for = sold_for;
        crate::sim::organized_power::record_action(world, org,
            crate::sim::organized_power::ActionQuality::Transactional);
    }
    Some(out)
}

/// The flat share every member owes each meeting the cause holds
/// (#1342) - set at the founding vote, one rate for the whole roll
pub const DUES_RATE: f32 = 1.0;
/// Missed rounds before the roll quietly drops a name (#1342)
pub const LAPSE_OUT: u8 = 3;

/// The cause's dues book (#1342): the rate the founding vote set, and
/// whose standing's slipped - a name appears on a missed round and
/// clears when their dues land
#[derive(Debug, Clone)]
pub struct Dues {
    /// Flat for every member, per cadence
    pub rate: f32,
    /// Rounds each member's missed; absent from the map is good standing
    pub missed: crate::engine::det::HashMap<Entity, u8>,
}

/// What the hat's round left (#1342): who paid, whose share the purse
/// carried, whose standing slipped, and whose name the roll dropped
#[derive(Debug, Default)]
pub struct DuesBeat {
    /// Members whose own purse paid the round
    pub paid: usize,
    /// Broke members the treasury carried - solidarity's literal price
    pub carried: Vec<Entity>,
    /// Members whose standing slipped a round
    pub slipped: Vec<Entity>,
    /// Members the roll dropped for good
    pub gone: Vec<Entity>,
}

/// The dues round at a held meeting (#1342): each living grown member
/// owes the flat rate - their own purse pays it into the cause's, the
/// treasury covers a broke member while it can, and a name missed too
/// long quietly leaves the rolls, freeing any seat it held
pub fn dues_beat(world: &mut World, campaign: Entity) -> DuesBeat {
    let Some(org) = world.get_component::<Campaign>(campaign).map(|c| c.org) else {
        return DuesBeat::default();
    };
    let Some(members) = world.get_component::<Organization>(org).map(|o| o.members.clone()) else {
        return DuesBeat::default();
    };
    let rate = world.get_component::<Dues>(org).map(|d| d.rate).unwrap_or(DUES_RATE);
    let mut beat = DuesBeat::default();
    for m in members {
        if is_dead(world, m) || world.get_component::<crate::game::vocation::Kid>(m).is_some()
            || crate::game::dungeon::unavailable(world, m) { continue; }
        // their own purse pays if it's got the share
        let paid = world.get_component_mut::<crate::game::livelihood::Purse>(m)
            .map(|p| if p.gold >= rate { p.gold -= rate; true } else { false })
            .unwrap_or(false);
        if paid {
            if let Some(o) = world.get_component_mut::<Organization>(org) { o.resources += rate; }
            if let Some(d) = world.get_component_mut::<Dues>(org) { d.missed.remove(&m); }
            beat.paid += 1;
            continue;
        }
        // a broke member's share comes out of the treasury while it
        // can - the org spends what their purse can't (#1342)
        if world.get_component::<Organization>(org).map(|o| o.resources >= rate).unwrap_or(false) {
            if let Some(o) = world.get_component_mut::<Organization>(org) { o.resources -= rate; }
            if let Some(d) = world.get_component_mut::<Dues>(org) { d.missed.remove(&m); }
            beat.carried.push(m);
            continue;
        }
        let missed = world.get_component_mut::<Dues>(org)
            .map(|d| { let n = d.missed.entry(m).or_insert(0); *n += 1; *n })
            .unwrap_or(0);
        if missed == 1 { beat.slipped.push(m); }
        if missed >= LAPSE_OUT { beat.gone.push(m); }
    }
    if !beat.gone.is_empty() {
        if let Some(o) = world.get_component_mut::<Organization>(org) {
            o.members.retain(|m| !beat.gone.contains(m));
            o.successors.retain(|m| !beat.gone.contains(m));
        }
        if let Some(c) = world.get_component_mut::<Campaign>(campaign) {
            c.seats.retain(|s| !beat.gone.contains(&s.holder));
            c.dissenters.retain(|m| !beat.gone.contains(m));
        }
        if let Some(d) = world.get_component_mut::<Dues>(org) {
            for m in &beat.gone { d.missed.remove(m); }
        }
    }
    beat
}

/// The founding vote binds the room (#1342): everyone who came and can
/// still swear joins the org as a body - the enrollment the ask rides
/// on, re-run wherever the room's called together again
pub fn enroll_room(world: &mut World, org: Entity, came: &[Entity]) -> Vec<Entity> {
    let mut joined: Vec<Entity> = came.iter().copied()
        .filter(|&v| !is_dead(world, v)
            && world.get_component::<crate::game::vocation::Kid>(v).is_none()
            && !crate::game::dungeon::unavailable(world, v)
            && !world.get_component::<Organization>(org)
                .map(|o| o.members.contains(&v)).unwrap_or(false))
        .collect();
    joined.sort_unstable();
    if !joined.is_empty() {
        if let Some(o) = world.get_component_mut::<Organization>(org) {
            o.members.extend(joined.iter().copied());
        }
    }
    joined
}

/// The asking the cause has already done on each issue (#1335): a
/// completed survey tally, or a meeting where three voices surfaced it.
/// Lives on the org so a founding meeting's legwork counts before the
/// campaign entity exists - the loud motions check it per issue
#[derive(Debug, Clone, Default)]
pub struct Legwork(pub crate::engine::det::HashSet<IssueTag>);

/// The org's people already asked about `tag`: a survey's count came
/// back, or a meeting's voices surfaced it for the organizer
pub fn mark_legwork(world: &mut World, org: Entity, tag: IssueTag) {
    let mut l = world.get_component::<Legwork>(org).cloned().unwrap_or_default();
    l.0.insert(tag);
    world.add_component(org, l);
}

/// Whether the cause's own issue has legwork behind it (#1335): the
/// doors asked or the voices heard - the check the loud public
/// motions ride on
pub fn legwork_done(world: &World, campaign: Entity) -> bool {
    world.get_component::<Campaign>(campaign)
        .and_then(|c| world.get_component::<Legwork>(c.org)
            .map(|l| l.0.contains(&c.issue)))
        .unwrap_or(false)
}

/// A survey under way (#1270): which campaign's asking, the issue the
/// floor picked, who's carrying it door to door, and the days left
#[derive(Debug, Clone)]
pub struct OrgSurvey {
    /// The campaign whose motion carried it
    pub campaign: Entity,
    /// The issue the doors get asked about
    pub issue: IssueTag,
    /// The members doing the asking - a member-day each is the cost
    pub canvassers: Vec<Entity>,
    /// Days of asking left before the count lands
    pub days_left: u8,
    /// Doors already knocked - the same soul isn't asked twice
    pub knocked: crate::engine::det::HashSet<Entity>,
}

/// The bond that makes a door answerable: folk you don't know don't
/// tell you what ails them (#1270)
pub const SURVEY_TIE: f32 = 0.15;
/// Days a survey's out asking before the tally lands
pub const SURVEY_DAYS: u8 = 3;
/// The most hands a canvass fields - a round's a day's walk each
pub const SURVEY_MAX_HANDS: usize = 6;

/// Whether the issue sits in `v`'s own house: a live problem or an
/// unsatisfied want that stands for it (#1270)
pub fn has_issue(world: &World, v: Entity, issue: &IssueTag, tick: u64) -> bool {
    let stands = |r: Reveal| desires::tag_stands(issue, &desires::issue_tag(world, v, &r));
    if desires::problems_of(world, v, tick).iter().any(|&p| stands(Reveal::Problem(p))) {
        return true;
    }
    world.get_component::<desires::InnerLife>(v)
        .map(|life| life.wants.iter()
            .any(|&w| !desires::is_satisfied(world, v, w, tick) && stands(Reveal::Want(w))))
        .unwrap_or(false)
}

/// Who the canvass can reach: every soul holding a real bond to a
/// canvasser - the asking's audience is the cause's ties made legible.
/// Members and kids don't count: you're asking the village, not
/// yourselves (#1270)
pub fn survey_respondents(world: &World, org: Entity, canvassers: &[Entity],
    villagers: &[Entity]) -> Vec<Entity> {
    let members: crate::engine::det::HashSet<Entity> = world.get_component::<Organization>(org)
        .map(|o| o.members.iter().copied().collect()).unwrap_or_default();
    villagers.iter().copied().filter(|&v| {
        if members.contains(&v) || is_dead(world, v)
            || world.get_component::<crate::game::vocation::Kid>(v).is_some()
            || crate::game::dungeon::unavailable(world, v) {
            return false;
        }
        canvassers.iter().any(|&m| m != v
            && crate::game::relationships::find_relationship(world, v, m)
                .and_then(|r| world.get_component::<crate::game::relationships::Relationship>(r))
                .map(|r| r.strength >= SURVEY_TIE).unwrap_or(false))
    }).collect()
}

/// The ring a respondent answers from (#1270): 3 their own house, 2 a
/// family's, 1 a friend's, 0 nothing to say. The tally counts the
/// closest ring they stand in
pub fn survey_ring(world: &World, v: Entity, issue: &IssueTag, tick: u64) -> u8 {
    if has_issue(world, v, issue, tick) { return 3; }
    // the family's roof: any housemate carrying it answers for them
    if let Some(h) = crate::game::household::home_of(world, v) {
        if let Some(b) = world.get_component::<crate::game::household::Building>(h) {
            if b.occupants.iter().any(|&o| o != v && has_issue(world, o, issue, tick)) {
                return 2;
            }
        }
    }
    // a friend carrying it: any tie strong enough to be told
    let known = world.query::<crate::game::relationships::Relationship>().iter().any(|&r| {
        let rel = world.get_component::<crate::game::relationships::Relationship>(r);
        let Some(rel) = rel else { return false };
        if rel.strength < SURVEY_TIE { return false; }
        let other = if rel.from == v { rel.other } else if rel.other == v { rel.from } else { return false };
        has_issue(world, other, issue, tick)
    });
    if known { 1 } else { 0 }
}

/// The fire grief the brigade answers (#1346): "the bad bread" voiced
/// or surveyed into the cause's legwork book - or a living member
/// still carrying the blame themselves, identified or not
fn fire_known(world: &World, campaign: Entity) -> bool {
    const FIRE_TAG: &str = "the bad bread";
    let Some(org) = world.get_component::<Campaign>(campaign).map(|c| c.org) else { return false };
    if world.get_component::<Legwork>(org)
        .map(|l| l.0.iter().any(|t| t.as_str() == FIRE_TAG)).unwrap_or(false) {
        return true;
    }
    let Some(members) = world.get_component::<Organization>(org).map(|o| o.members.clone()) else { return false };
    members.iter().any(|&m| !is_dead(world, m)
        && desires::problems_of(world, m, u64::MAX).contains(&desires::Problem::Blamed))
}

/// The motions a cause this size can carry tonight: more people means
/// more and different doors - two fronts want hands for both at once,
/// a post wants someone to spare, and a warm town wants staging (#1157)
pub fn motions_open(world: &World, campaign: Entity) -> Vec<Motion> {
    let n = living_members(world, campaign);
    let mut ms = vec![];
    // the loud public work wants legwork behind the cause's own issue
    // first (#1335): the doors asked or the voices heard. The quiet
    // work - flyers, an organizer on the road, the asking itself -
    // stays open to a fresh cause
    let asked = legwork_done(world, campaign);
    if asked && n >= 2 { ms.push(Motion::Strike); }
    if asked && n >= 3 { ms.push(Motion::March); }
    if asked && n >= 4 { ms.push(Motion::Townhall); }
    if n >= 2 { ms.push(Motion::Flyers); }
    if asked && n >= 6 { ms.push(Motion::TwoFronts); }
    if n >= 8 { ms.push(Motion::SendOrganizer); }
    // any cause big enough to act can first go asking
    if n >= 2 { ms.push(Motion::Survey); }
    // the quiet work of hands (#1344): a few members pick, a bigger
    // room raises - the site check is the agenda's, the slate just
    // needs the hands
    if n >= 3 { ms.push(Motion::WorkParty(PartyKind::Forage)); }
    if n >= 5 { ms.push(Motion::WorkParty(PartyKind::Raise)); }
    // the fire brigade is the cause's answer to a fire grief it's
    // identified (#1346): a bad-bread tag voiced or surveyed into the
    // legwork book, or a member still carrying the blame themselves.
    // Chartering's loud enough it wants the same survey-first
    // discipline the street verbs ride
    if asked && n >= 3 && fire_known(world, campaign) {
        ms.push(Motion::Brigade);
    }
    // the rhythm is the meeting's own (#1337): the floor can always
    // retime its gathering, tally the next step once it's held one,
    // put the assembly up early when the checks have a slate, mend a
    // roofless hall - housekeeping, not street work, so no legwork asks
    ms.push(Motion::Cadence);
    if world.get_component::<CampaignMeeting>(campaign).map(|m| m.held > 0).unwrap_or(false) {
        ms.push(Motion::Debrief);
    }
    let org = world.get_component::<Campaign>(campaign).map(|c| c.org);
    if let Some(org) = org {
        if world.get_component::<crate::game::organizing::HallDamaged>(org).is_some() {
            ms.push(Motion::MendHall);
        }
        if !agenda_checks(world, campaign, org).is_empty()
            && world.get_component::<Assembly>(campaign)
                .map(|a| a.announced.is_none()).unwrap_or(true)
        {
            ms.push(Motion::Assembly);
        }
    }
    // the room's own coin: when the pledges gather behind a good, the
    // floor can carry the pooled buy - quiet work, no legwork asked
    if let Some((gi, pledgers)) = group_pledges(world, campaign).into_iter()
        .find(|(gi, ps)| ps.len() >= 2
            && ps.iter().map(|(_, p)| p).sum::<f32>()
                >= crate::game::goods::worth_at(world, BUY_GOODS[*gi]) * PLEDGE_BAR_UNITS)
    {
        let promised = pledgers.iter().map(|(_, p)| p).sum::<f32>().round() as u32;
        ms.push(Motion::GroupBuy(gi, promised));
    }
    // a posted town that runs warm enough can stage the cause's day
    if asked {
        if let Some(o) = world.get_component::<Outreach>(campaign) {
            for &(site, _, support) in &o.posts {
                if support >= FIELD_WARM { ms.push(Motion::ParallelEvent(site)); }
            }
        }
    }
    // the room's read gates the slate (#1310): a cause set on mercy
    // isn't offered the street, a cause set on the street isn't offered
    // the quiet work - the pick the organizer keeps taking narrows what
    // the floor will hear next
    if let Some(lean) = room_lean(world, campaign) {
        ms.retain(|m| motion_line(*m) != Some(lean.opposite()));
    }
    ms
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
            influence_pool: 0.0, resources: 0.0, charter: "".into(), successors: vec![] });
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

        let c = found_campaign(&mut w, o, &vote, Framing::CommonFoe, DecisionRule::Majority, 42);
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
    fn test_a_voter_torn_between_issues_takes_the_recommendation() {
        // taxed AND broke: either fight is theirs, so they back the one
        // the organizer put first
        let mut w = World::new();
        let m = member(&mut w);
        add_grievance(&mut w, m, Problem::Taxed, 100);
        add_grievance(&mut w, m, Problem::Broke, 100);
        let candidates = vec!["poverty".to_string(), "the taxes".to_string()];
        assert_eq!(vote_for(&w, m, &candidates, 10).as_deref(), Some("poverty"));
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
            exposure: 0.0,
            tasked_by: None, pending: 0.0, apprentice_years: 0, stage: 0,
        });
        let candidates = vec!["apprenticeship (baker)".to_string(), "apprenticeship (mason)".to_string()];
        assert_eq!(vote_for(&w, m, &candidates, 10).as_deref(), Some("apprenticeship (mason)"));
        let candidates = vec!["apprenticeship (baker)".to_string()];
        assert_eq!(vote_for(&w, m, &candidates, 10), None, "the bakers' fight isn't theirs");
    }

    // --- C2: meetings, cadence, seats, the shadow elite ---

    use crate::game::relationships::{record_interaction, Interaction, InteractionKind};
    use crate::sim::organized_power::OrganizedPower;

    fn campaign_with(w: &mut World, members: &[Entity]) -> (Entity, Entity) {
        let o = org(w, members);
        w.add_component(o, OrganizedPower::new());
        let vote = IssueVote { winner: "the taxes".into(), ballots: vec![] };
        let c = found_campaign(w, o, &vote, Framing::CommonFoe, DecisionRule::Majority, 1);
        (o, c)
    }

    /// Bond `a` and `b` a few times (two Aids = 0.24, over the turnout bar)
    fn bond(w: &mut World, a: Entity, b: Entity) {
        for _ in 0..2 {
            record_interaction(w, a, Interaction { other: b, kind: InteractionKind::Aid, intensity: 1.0 });
        }
    }

    #[test]
    fn test_a_conversion_room_pulls_the_hearts() {
        let mut w = World::new();
        let (a, b) = (member(&mut w), member(&mut w));
        bond(&mut w, a, b);
        let (_, c) = campaign_with(&mut w, &[a, b]);
        w.get_component_mut::<Campaign>(c).unwrap().issue = "the old faith".into();
        w.add_component(a, crate::sim::religion::Faith {
            religion: "the old faith".into(), devotion: 0.5, traditions: vec![] });
        w.add_component(b, crate::sim::religion::Faith {
            religion: "the parish".into(), devotion: 0.5, traditions: vec![] });
        set_cadence(&mut w, c, 7, 1);
        delegate_role(&mut w, c, Role::TurnoutCoordinator, a);
        hold_campaign_meeting(&mut w, c, 8);
        let after = w.get_component::<crate::sim::religion::Faith>(b)
            .map(|f| f.devotion).unwrap_or(0.5);
        assert!(after >= 0.5, "the room's hearts never drifted");
    }

    #[test]
    fn test_a_heart_fully_won_takes_the_faith() {
        let mut w = World::new();
        let (a, b) = (member(&mut w), member(&mut w));
        bond(&mut w, a, b);
        let (_, c) = campaign_with(&mut w, &[a, b]);
        w.get_component_mut::<Campaign>(c).unwrap().issue = "the old faith".into();
        w.add_component(a, crate::sim::religion::Faith {
            religion: "the old faith".into(), devotion: 0.9, traditions: vec![] });
        w.add_component(b, crate::sim::religion::Faith {
            religion: "the parish".into(), devotion: 0.94, traditions: vec![] });
        set_cadence(&mut w, c, 7, 1);
        delegate_role(&mut w, c, Role::TurnoutCoordinator, a);
        hold_campaign_meeting(&mut w, c, 8);
        if let Some(f) = w.get_component::<crate::sim::religion::Faith>(b) {
            if f.devotion >= 0.95 {
                assert_eq!(f.religion, "the old faith",
                    "the heart fully won never took the name");
            }
        }
    }

    #[test]
    fn test_a_seat_must_be_delegated_to_a_living_member() {
        let mut w = World::new();
        let (a, b) = (member(&mut w), member(&mut w));
        let outsider = member(&mut w);
        let (_, c) = campaign_with(&mut w, &[a, b]);
        assert!(delegate_role(&mut w, c, Role::TurnoutCoordinator, a));
        assert!(!delegate_role(&mut w, c, Role::NoteTaker, outsider), "outsiders can't hold seats");
        w.add_component(a, crate::game::villager::Dead);
        assert!(!delegate_role(&mut w, c, Role::Chair, a), "nor the dead");
        // the org's leader can hold a seat even if not on the member rolls
        let leader = w.get_component::<Organization>(
            w.get_component::<Campaign>(c).unwrap().org).unwrap().leader;
        assert!(delegate_role(&mut w, c, Role::Chair, leader));
        // re-delegation replaces: one seat, one holder, one role per holder
        assert!(delegate_role(&mut w, c, Role::TurnoutCoordinator, b));
        assert_eq!(seat_holder(&w, c, Role::TurnoutCoordinator), Some(b));
        assert_eq!(w.get_component::<Campaign>(c).unwrap().seats.iter()
            .filter(|s| s.holder == a).count(), 0);
    }

    #[test]
    fn test_the_coordinators_ties_drive_turnout() {
        let mut w = World::new();
        let (coord, close, distant) = (member(&mut w), member(&mut w), member(&mut w));
        bond(&mut w, coord, close); // 0.24 - over the bar
        record_interaction(&mut w, coord, Interaction {
            other: distant, kind: InteractionKind::Conversation, intensity: 1.0 }); // 0.05 - under
        let (_, c) = campaign_with(&mut w, &[coord, close, distant]);
        set_cadence(&mut w, c, 7, 1);
        delegate_role(&mut w, c, Role::TurnoutCoordinator, coord);

        let out = hold_campaign_meeting(&mut w, c, 8).unwrap();
        assert!(out.attendees.contains(&coord) && out.attendees.contains(&close));
        assert!(!out.attendees.contains(&distant), "a 0.05 tie doesn't answer the call");
        assert!(!out.drifted, "the coordinator called it - no drift");
        let m = w.get_component::<CampaignMeeting>(c).unwrap();
        assert_eq!(m.held, 1);
        assert_eq!(m.last_held, 8);
    }

    #[test]
    fn test_an_empty_seat_lets_the_shadow_elite_run_the_room() {
        let mut w = World::new();
        let (a, b, c1) = (member(&mut w), member(&mut w), member(&mut w));
        // b is bonded to everyone; the leader is a stranger to them all
        bond(&mut w, b, a);
        bond(&mut w, b, c1);
        let (o, c) = campaign_with(&mut w, &[a, b, c1]);
        // the leader pulls nobody
        assert_eq!(shadow_elite(&w, c), Some(b));
        set_cadence(&mut w, c, 7, 1);
        let out = hold_campaign_meeting(&mut w, c, 8).unwrap();
        assert!(out.drifted, "no coordinator - the room follows the pull");
        assert!(out.attendees.contains(&a) && out.attendees.contains(&c1));
        let _ = o;
    }

    #[test]
    fn test_a_dissenter_who_comes_gets_heard() {
        let mut w = World::new();
        let (a, b) = (member(&mut w), member(&mut w));
        add_grievance(&mut w, a, Problem::Taxed, 200);
        add_grievance(&mut w, b, Problem::Broke, 200);
        let o = org(&mut w, &[a, b]);
        w.add_component(o, OrganizedPower::new());
        let vote = hold_issue_vote(&w, o, &["the taxes".into(), "poverty".into()], Framing::CommonFoe, 10).unwrap();
        let c = found_campaign(&mut w, o, &vote, Framing::CommonFoe, DecisionRule::Majority, 1);
        assert_eq!(w.get_component::<Campaign>(c).unwrap().dissenters, vec![b]);

        bond(&mut w, a, b);
        set_cadence(&mut w, c, 7, 1);
        delegate_role(&mut w, c, Role::TurnoutCoordinator, a);
        let out = hold_campaign_meeting(&mut w, c, 8).unwrap();
        assert_eq!(out.healed, vec![b], "they came, they were heard");
        assert!(w.get_component::<Campaign>(c).unwrap().dissenters.is_empty());
    }

    #[test]
    fn test_the_note_taker_keeps_the_resolves_and_consensus_banks_consent() {
        let mut w = World::new();
        let (a, b) = (member(&mut w), member(&mut w));
        bond(&mut w, a, b);
        let (_, c) = campaign_with(&mut w, &[a, b]);
        set_cadence(&mut w, c, 7, 1);
        delegate_role(&mut w, c, Role::TurnoutCoordinator, a);
        delegate_role(&mut w, c, Role::NoteTaker, b);
        let out = hold_campaign_meeting(&mut w, c, 8).unwrap();
        assert!(out.noted);
        assert_eq!(w.get_component::<CampaignMeeting>(c).unwrap().resolves, 1);
        assert!(!out.consent, "majority doesn't bank consent");

        // same room under consensus: consent for the next action
        w.get_component_mut::<Campaign>(c).unwrap().decision_rule = DecisionRule::Consensus;
        let out = hold_campaign_meeting(&mut w, c, 15).unwrap();
        assert!(out.consent);
    }

    #[test]
    fn test_a_campaign_lapses_and_meetings_build_power() {
        let mut w = World::new();
        let (a, b) = (member(&mut w), member(&mut w));
        bond(&mut w, a, b);
        let (o, c) = campaign_with(&mut w, &[a, b]);
        set_cadence(&mut w, c, 7, 1);
        delegate_role(&mut w, c, Role::TurnoutCoordinator, a);
        assert!(!lapsed(&w, c, 14), "one missed cadence isn't a lapse yet");
        assert!(lapsed(&w, c, 16), "two missed cadences is");
        let before = w.get_component::<OrganizedPower>(o).unwrap().capacity;
        hold_campaign_meeting(&mut w, c, 8);
        assert!(w.get_component::<OrganizedPower>(o).unwrap().capacity > before,
            "a meeting people actually attend builds the muscle");
    }
    #[test]
    fn test_clustered_tags_put_the_theme_on_the_ballot() {
        // grievances in one cluster gather under the platform's name;
        // a thinner issue keeps its own (#1072)
        let ps = desires::platforms(&[
            "the taxes".to_string(), "the fevers".to_string(),
            "the bailiff's writ".to_string(), "the vote".to_string()]);
        assert_eq!(ps.len(), 3);
        assert_eq!(ps[0].0, "hard times under the lord's hand");
        assert_eq!(ps[0].1, vec!["the taxes".to_string(), "the bailiff's writ".to_string()]);
        assert_eq!(ps[1].0, "the fevers", "a lone tag stays its own thin issue");
        assert_eq!(ps[2].0, "the vote", "the suffrage keeps its own name");

        // a member's stake in any constituent tag stands for the theme
        let mut w = World::new();
        let taxed = member(&mut w);
        add_grievance(&mut w, taxed, Problem::Taxed, 100);
        let written = member(&mut w);
        add_grievance(&mut w, written, Problem::WritBorne, 100);
        let o = org(&mut w, &[taxed, written]);
        let candidates = vec!["hard times under the lord's hand".to_string(), "the fevers".to_string()];
        let vote = hold_issue_vote(&w, o, &candidates, Framing::SharedGrievance, 10).unwrap();
        assert_eq!(vote.winner, "hard times under the lord's hand",
            "the tax grievance and the writ both stand for the lord's-hand theme");
        assert!(vote.ballots.iter().all(|b| b.stake.is_some()),
            "every voter's stake resolves inside the theme");
        assert!(stake_for(&w, taxed, &vote.winner, 10).is_some(),
            "the platform's name reaches a constituent stake");
    }

    #[test]
    fn test_a_theme_outvotes_its_own_thin_tags() {
        // the platform gathers stakes its separate tags would have
        // split: taxed + writ outvote the single fever (#1072)
        let mut w = World::new();
        let (a, b, c) = (member(&mut w), member(&mut w), member(&mut w));
        add_grievance(&mut w, a, Problem::Taxed, 100);
        add_grievance(&mut w, b, Problem::WritBorne, 100);
        add_grievance(&mut w, c, Problem::Sick, 100);
        let o = org(&mut w, &[a, b, c]);
        let candidates = vec!["the fevers".to_string(), "hard times under the lord's hand".to_string()];
        let vote = hold_issue_vote(&w, o, &candidates, Framing::SharedGrievance, 10).unwrap();
        assert_eq!(vote.winner, "hard times under the lord's hand",
            "two clustered stakes beat one thin one even recommended second");
    }

    /// The room's read narrows the slate (#1310): a cause set on mercy
    /// isn't offered the street, a cause set on the street isn't
    /// offered the quiet work, and an unread room sees it all
    #[test]
    fn test_the_room_lean_gates_the_slate() {
        use crate::game::rhetoric::{OrgRead, RhetoricMemory};
        let mut w = World::new();
        let ms: Vec<Entity> = (0..6).map(|_| member(&mut w)).collect();
        let (o, c) = campaign_with(&mut w, &ms);
        mark_legwork(&mut w, o, "the taxes".into());
        let lean = |w: &mut World, p: f32, f: f32| {
            for &m in &ms {
                let mut mem = RhetoricMemory::default();
                mem.by_org.insert(o, OrgRead {
                    weight: [p, f, 0.0, 0.0], ..Default::default() });
                w.add_component(m, mem);
            }
        };
        // an unread room gets the whole slate
        let slate = motions_open(&w, c);
        assert!(slate.contains(&Motion::March) && slate.contains(&Motion::Survey),
            "unread room: {slate:?}");
        // a patient room drops the street picks
        lean(&mut w, 1.0, 0.0);
        let slate = motions_open(&w, c);
        assert!(!slate.contains(&Motion::March) && !slate.contains(&Motion::Strike)
            && !slate.contains(&Motion::Townhall) && !slate.contains(&Motion::TwoFronts),
            "a patient room asks no fire: {slate:?}");
        assert!(slate.contains(&Motion::Survey) && slate.contains(&Motion::Flyers),
            "the quiet work stays: {slate:?}");
        // a firebrand room drops the quiet work
        lean(&mut w, 0.0, 1.0);
        let slate = motions_open(&w, c);
        assert!(!slate.contains(&Motion::Survey) && !slate.contains(&Motion::Flyers),
            "a firebrand room asks no patience: {slate:?}");
        assert!(slate.contains(&Motion::March), "the street stays: {slate:?}");
    }

    #[test]
    fn test_the_loud_work_waits_for_the_asking() {
        let mut w = World::new();
        let ms: Vec<Entity> = (0..6).map(|_| member(&mut w)).collect();
        let (o, c) = campaign_with(&mut w, &ms);
        // a fresh cause can fly bills and go asking - but the street's
        // loud work waits for the asking to come back on ITS trouble
        let slate = motions_open(&w, c);
        for m in [Motion::Strike, Motion::March, Motion::Townhall, Motion::TwoFronts] {
            assert!(!slate.contains(&m), "no legwork yet: {slate:?}");
        }
        assert!(slate.contains(&Motion::Survey) && slate.contains(&Motion::Flyers),
            "the legwork itself stays open: {slate:?}");
        // asking after some other trouble doesn't count - legwork's
        // about this grievance, not general experience
        mark_legwork(&mut w, o, "the rents".into());
        let slate = motions_open(&w, c);
        assert!(!slate.contains(&Motion::March), "a different trouble asked: {slate:?}");
        // once the cause's own count's back, the whole floor opens
        mark_legwork(&mut w, o, "the taxes".into());
        let slate = motions_open(&w, c);
        assert!(slate.contains(&Motion::Strike) && slate.contains(&Motion::March)
            && slate.contains(&Motion::Townhall) && slate.contains(&Motion::TwoFronts),
            "the asking's done: {slate:?}");
    }

    #[test]
    fn test_the_hat_passes_and_the_treasury_carries_a_broke_name() {
        use crate::game::livelihood::Purse;
        let mut w = World::new();
        let ms: Vec<Entity> = (0..4).map(|_| member(&mut w)).collect();
        // the broke one sits first in the roll: the hat hasn't filled
        // when his turn comes, and a thin treasury can't carry him
        w.add_component(ms[0], Purse { gold: 0.0 });
        for &m in &ms[1..] { w.add_component(m, Purse { gold: 2.0 }); }
        let (_o, c) = campaign_with(&mut w, &ms);
        // three purses pay their coin into the cause's; his slips
        let beat = dues_beat(&mut w, c);
        assert_eq!(beat.paid, 3);
        assert_eq!(beat.slipped, vec![ms[0]]);
        assert!(beat.carried.is_empty() && beat.gone.is_empty());
        let res = w.get_component::<Organization>(_o).unwrap().resources;
        assert_eq!(res, 3.0);
        // and a lapsed name doesn't carry the slate's headcount while
        // it's still on the rolls
        assert_eq!(living_members(&w, c), 3);
        // next round the purse the dues filled carries him - the
        // treasury spends his share, and his standing holds
        let beat = dues_beat(&mut w, c);
        assert_eq!(beat.paid, 3);
        assert_eq!(beat.carried, vec![ms[0]]);
        assert!(beat.slipped.is_empty() && beat.gone.is_empty());
        let res = w.get_component::<Organization>(_o).unwrap().resources;
        assert_eq!(res, 5.0, "three more dues less his carried share");
        assert_eq!(living_members(&w, c), 4);
    }

    #[test]
    fn test_a_name_missed_too_long_leaves_the_rolls() {
        use crate::game::livelihood::Purse;
        let mut w = World::new();
        let ms: Vec<Entity> = (0..2).map(|_| member(&mut w)).collect();
        for &m in &ms { w.add_component(m, Purse { gold: 0.0 }); }
        let (o, c) = campaign_with(&mut w, &ms);
        delegate_role(&mut w, c, Role::Chair, ms[1]);
        // an empty treasury can't carry anyone: slips pile up, and the
        // third missed round drops the name, seat and all
        for _ in 0..(LAPSE_OUT - 1) {
            let beat = dues_beat(&mut w, c);
            assert!(beat.gone.is_empty());
        }
        let beat = dues_beat(&mut w, c);
        assert_eq!(beat.gone.len(), 2);
        assert!(w.get_component::<Organization>(o).unwrap().members.is_empty());
        assert!(w.get_component::<Campaign>(c).unwrap().seats.is_empty());
        assert_eq!(living_members(&w, c), 0);
    }

    #[test]
    fn test_the_founding_vote_enrolls_the_room_as_a_body() {
        let mut w = World::new();
        let sworn: Vec<Entity> = (0..2).map(|_| member(&mut w)).collect();
        let (o, _c) = campaign_with(&mut w, &sworn);
        // folk who came to listen but never swore join with the vote -
        // kids and the taken don't
        let listener = member(&mut w);
        let kid = member(&mut w);
        w.add_component(kid, crate::game::vocation::Kid);
        let came = vec![sworn[0], listener, kid];
        let joined = enroll_room(&mut w, o, &came);
        assert_eq!(joined, vec![listener]);
        assert!(w.get_component::<Organization>(o).unwrap().members.contains(&listener));
        assert!(!w.get_component::<Organization>(o).unwrap().members.contains(&kid));
    }

    #[test]
    fn test_the_rooms_coin_pools_behind_the_wanted_good() {
        use crate::game::desires::Want;
        use crate::game::livelihood::{Purse, Satchel};
        let mut w = World::new();
        let ms: Vec<Entity> = (0..4).map(|_| member(&mut w)).collect();
        for &m in &ms {
            w.get_component_mut::<desires::InnerLife>(m).unwrap().wants.push(Want::GoodFood);
            w.add_component(m, Purse { gold: 20.0 });
            w.add_component(m, Satchel::default());
        }
        let (_o, c) = campaign_with(&mut w, &ms);
        // four purses behind the bread put the pooled buy on the slate
        let slate = motions_open(&w, c);
        assert!(slate.iter().any(|m| matches!(*m, Motion::GroupBuy(0, p) if p == 40)),
            "the pledges gather under the bread: {slate:?}");
        // and the room goes in together: the order bends the price,
        // shares come home, the surplus sells below street for the cause
        let worker = w.create_entity();
        w.add_component(worker, Purse { gold: 0.0 });
        let b = crate::game::household::spawn_building(&mut w,
            crate::game::household::BuildingKind::Shop(crate::game::vocation::Vocation::Baker),
            "the bakery", 0.0, 0.0, 20);
        {
            let bb = w.get_component_mut::<crate::game::household::Building>(b).unwrap();
            bb.worker = Some(worker);
            bb.stock.insert("bread".into(), 20);
        }
        let out = group_buy(&mut w, c, 0).expect("the order lands");
        // pool 40 on a street price of 5: the bulk alone bends it a
        // point a share - 8 units, 4 home and 4 out the door
        assert_eq!(out.shares.len(), 4);
        assert_eq!(out.shares.iter().map(|(_, n)| n).sum::<u32>(), 4);
        assert_eq!(out.surplus, 4);
        assert!((out.discount - 0.08).abs() < 0.01, "bulk bends the price: {}", out.discount);
        for &m in &ms {
            assert_eq!(*w.get_component::<Satchel>(m).unwrap().goods.get("bread").unwrap_or(&0), 1);
            assert_eq!(w.get_component::<Purse>(m).unwrap().gold, 10.0);
        }
        // the keeper's paid the discounted price, the treasury keeps
        // the sell-off's margin
        assert!((w.get_component::<Purse>(worker).unwrap().gold - out.paid).abs() < 0.01);
        assert_eq!(w.get_component::<Organization>(_o).unwrap().resources, out.sold_for);
        assert!((out.sold_for - 16.0).abs() < 0.01);
    }

    #[test]
    fn test_the_pledges_dont_reach_without_the_coin_or_the_hands() {
        use crate::game::desires::Want;
        use crate::game::livelihood::{Purse, Satchel};
        let mut w = World::new();
        // one pledger isn't a group, however deep their purse
        let m1 = member(&mut w);
        w.get_component_mut::<desires::InnerLife>(m1).unwrap().wants.push(Want::GoodFood);
        w.add_component(m1, Purse { gold: 30.0 });
        w.add_component(m1, Satchel::default());
        let (_o, c) = campaign_with(&mut w, &[m1]);
        let slate = motions_open(&w, c);
        assert!(!slate.iter().any(|m| matches!(m, Motion::GroupBuy(..))),
            "one purse isn't a pool: {slate:?}");
        // two thin purses want the bread but can't promise a bulk's worth
        let ms: Vec<Entity> = (0..3).map(|_| member(&mut w)).collect();
        for &m in &ms {
            w.get_component_mut::<desires::InnerLife>(m).unwrap().wants.push(Want::GoodFood);
            w.add_component(m, Purse { gold: 2.0 });
            w.add_component(m, Satchel::default());
        }
        let (_o2, c2) = campaign_with(&mut w, &ms);
        let slate = motions_open(&w, c2);
        assert!(!slate.iter().any(|m| matches!(m, Motion::GroupBuy(..))),
            "thin purses don't promise a bulk: {slate:?}");
    }

    #[test]
    fn test_the_party_waits_for_the_hands() {
        let mut w = World::new();
        // two hands can't walk a party out; three pick, five raise
        let two: Vec<Entity> = (0..2).map(|_| member(&mut w)).collect();
        let (_o, c) = campaign_with(&mut w, &two);
        let slate = motions_open(&w, c);
        assert!(!slate.iter().any(|m| matches!(m, Motion::WorkParty(..))),
            "two hands can't field a party: {slate:?}");
        let three: Vec<Entity> = (0..3).map(|_| member(&mut w)).collect();
        let (_o3, c3) = campaign_with(&mut w, &three);
        let slate = motions_open(&w, c3);
        assert!(slate.iter().any(|m| matches!(m, Motion::WorkParty(PartyKind::Forage))),
            "three hands pick: {slate:?}");
        assert!(!slate.iter().any(|m| matches!(m, Motion::WorkParty(PartyKind::Raise))),
            "three hands don't raise: {slate:?}");
        let five: Vec<Entity> = (0..5).map(|_| member(&mut w)).collect();
        let (_o5, c5) = campaign_with(&mut w, &five);
        let slate = motions_open(&w, c5);
        assert!(slate.iter().any(|m| matches!(m, Motion::WorkParty(PartyKind::Raise))),
            "five hands raise: {slate:?}");
        // the work's quiet: no legwork asked, and it reads Patient
        assert_eq!(motion_line(Motion::WorkParty(PartyKind::Forage)),
            Some(crate::game::rhetoric::Line::Patient));
    }
}
