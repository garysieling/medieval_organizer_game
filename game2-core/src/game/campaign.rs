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
        .find(|r| desires::issue_tag(world, member, r) == *tag)
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
        let n = tags.iter().filter(|t| *t == c).count();
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
    let strength_to = |w: &World, m: Entity| find_relationship(w, m, driver)
        .and_then(|r| w.get_component::<Relationship>(r))
        .map(|r| r.strength).unwrap_or(0.0);
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
    // Participation teaches: the room learns meeting craft, the seats
    // learn theirs
    crate::game::org_skills::learn_from_meeting(world, campaign, &attendees);
    Some(MeetingOutcome { attendees, healed, drifted, noted, consent })
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
    let strength_to = |w: &World, m: Entity| find_relationship(w, m, driver)
        .and_then(|r| w.get_component::<Relationship>(r))
        .map(|r| r.strength).unwrap_or(0.0);
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
            tasked_by: None, pending: 0.0,
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
}
