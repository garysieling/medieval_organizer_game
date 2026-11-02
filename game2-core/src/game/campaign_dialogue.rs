//! Campaign dialogue: the conversation framework's rule-database model
//! applied to collective decisions. Two separately counted libraries of
//! decision beats - house meetings (a room deliberating) and campaign arcs
//! (action and reaction against a target) - live in `campaign_beats`.
//!
//! Plan: wiki/software-design/campaign-dialogue-trees-plan.md (Stage 1).
//! What carries over from `conversation`:
//! - Declarative `Has`/`Lacks` criteria on flattened facts; the most
//!   specific matching beat leads, and every phase keeps a criteria-free
//!   beat so the walk always has something to say (Ruskin).
//! - Weighted alternatives biased by context, never gated by it (CiF):
//!   a consensus room *leans* toward talking a point out, it isn't forced.
//! - Spent material: a beat is used once per walk (Versu).
//!
//! What doesn't: person-to-person `Act`s. A beat belongs to a meeting or
//! campaign `Phase`, and each move names the phase it hands the room to.
//! Weights are authored priors, not historical frequencies.
//!
//! Selection is pure (`walk`); state changes happen only through
//! `apply_hook`, so a chosen line never mutates the world by itself.

use std::collections::HashSet;

use crate::engine::ecs::{Entity, World};
use crate::game::campaign::{Campaign, CampaignMeeting, DecisionRule, Framing, MeetingOutcome, Role};
use crate::game::conversation::{Criterion, Facts, Rng};

pub use crate::game::campaign_arc_beats::ARC_BEATS;
pub use crate::game::campaign_beats::SOURCE_PATTERNS;
pub use crate::game::campaign_meeting_beats::MEETING_BEATS;

/// The two separately counted libraries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Library {
    /// A room deliberating: agenda to follow-through
    HouseMeeting,
    /// A campaign's action/reaction cycle: target to renewal
    CampaignArc,
}

/// Where in a meeting or a campaign a beat sits
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    // House meeting
    Agenda,
    Floor,
    Testimony,
    Proposal,
    Amendment,
    Objection,
    Coalition,
    Decision,
    FollowThrough,
    // Campaign arc
    Target,
    Build,
    Test,
    Escalate,
    Counter,
    Negotiate,
    Outcome,
    Enforce,
    Learn,
    Renew,
}

impl Phase {
    pub const MEETING: [Phase; 9] = [Phase::Agenda, Phase::Floor, Phase::Testimony, Phase::Proposal,
        Phase::Amendment, Phase::Objection, Phase::Coalition, Phase::Decision, Phase::FollowThrough];
    pub const ARC: [Phase; 10] = [Phase::Target, Phase::Build, Phase::Test, Phase::Escalate,
        Phase::Counter, Phase::Negotiate, Phase::Outcome, Phase::Enforce, Phase::Learn, Phase::Renew];

    pub fn library(self) -> Library {
        if Phase::MEETING.contains(&self) { Library::HouseMeeting } else { Library::CampaignArc }
    }
}

/// Who speaks a line. Resolved to a person at runtime; a voice that needs
/// someone particular (a dissenter, the note taker) must be backed by the
/// criteria that guarantee they're in the room.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Voice {
    /// Whoever drove turnout - the organizer, a coordinator, or the pull
    Driver,
    /// Any other member present
    Member,
    /// A dissenter who came and was heard (needs `room:dissent_heard`)
    Dissenter,
    /// The seated note taker (needs `room:noted`)
    NoteTaker,
    /// The seated chair, present (needs `seat:chair`)
    Chair,
    /// The room together - a murmur, a show of hands
    Room,
    /// The other side: the steward, the reeve, the lord's man (arcs only)
    Opponent,
}

/// What a chosen move does to the world, applied by `apply_hook` only
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Hook {
    /// Talk only - the transition carries it
    Nothing,
    /// A point is settled and written down: one more resolve on record
    Resolve,
    /// The speaker isn't satisfied and goes back among the dissenters
    Dissent,
    /// The room puts the cause another way
    Reframe(Framing),
}

/// Where a move hands the room next
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Next {
    Phase(Phase),
    Close,
}

/// One weighted response to a beat
pub struct Move {
    /// Unique within its beat
    pub id: &'static str,
    /// Facts that must hold for this move to be on offer at all
    pub criteria: &'static [Criterion],
    /// Authored base weight (positive)
    pub weight: f32,
    /// (fact, multiplier) applied to the weight when the fact holds
    pub bias: &'static [(&'static str, f32)],
    pub voice: Voice,
    /// Interchangeable ways to say it; `{issue}` fills with the cause
    pub lines: &'static [&'static str],
    pub hook: Hook,
    pub next: Next,
}

/// A decision beat: a situation put to the room, and how it can answer.
/// The counted unit of a library - variants and moves don't inflate it.
pub struct Beat {
    /// Stable, globally unique
    pub id: &'static str,
    pub phase: Phase,
    /// All must hold; criterion count is the beat's specificity
    pub criteria: &'static [Criterion],
    pub voice: Voice,
    /// Ways to put it; `{issue}` fills with the cause
    pub lines: &'static [&'static str],
    pub moves: &'static [Move],
    /// Tags into `SOURCE_PATTERNS`: the historical mechanism it borrows
    pub sources: &'static [&'static str],
}

/// How firmly a source pattern is attested - kept in research notes,
/// never voiced in character
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Evidence {
    /// Scripture or sacred tradition
    Sacred,
    /// Ballad, legend, or later retelling
    Legendary,
    /// Happened, but what it caused is argued over
    Contested,
    /// Well documented by records and scholarship
    Documented,
}

/// A historical pattern a beat transfers: the mechanism, not the slogan
pub struct SourcePattern {
    pub tag: &'static str,
    /// The case or tradition it comes from
    pub origin: &'static str,
    pub evidence: Evidence,
    /// What carries over into the village
    pub mechanism: &'static str,
}

/// Every fact a beat or move may test. Runtime builders only emit these,
/// and the inventory audit rejects content that names anything else.
/// The `arc:*` facts are the campaign-arc vocabulary; they're wired to
/// state in Stage 4. Outcomes are exclusive: granted (`won`), denied
/// (`lost`), split (`partial`), put off (`delayed`) or granted in name
/// and emptied in practice (`diluted`). `countered`: the other side has
/// moved; `allies`: others stand with the campaign; `leader_taken`: its
/// leader is held or gone; `fatigue`: folk are worn; `first`/`veteran`:
/// the org's first campaign, or one of many; `expired`: a grant ran its
/// term and wants renewing (nobody broke it - `promise_broken` is for a
/// grant taken back while still in force).
/// - `room:trait:*`: someone in the room carries that temperament
/// - `room:want:*`: what the room is short of - bread, relief from dues,
///   justice for a wrong, a roof, work, time to mourn
/// - `org:*`: the org around the room - members taken, its standing
///   power, whether its leader came
pub const KNOWN_FACTS: &[&str] = &[
    "rule:majority", "rule:consensus",
    "framing:grievance", "framing:foe", "framing:duty",
    "room:drifted", "room:noted", "room:dissent_heard", "room:dissent_absent",
    "room:thin", "room:full",
    "seat:chair", "seat:coordinator", "seat:note_taker",
    "history:first", "history:seasoned",
    "arc:weak", "arc:strong", "arc:consent", "arc:crackdown", "arc:prisoners",
    "arc:won", "arc:lost", "arc:partial", "arc:promise_broken", "arc:split",
    "arc:delayed", "arc:diluted", "arc:countered", "arc:allies", "arc:leader_taken",
    "arc:fatigue", "arc:first", "arc:veteran", "arc:expired",
    "room:trait:proud", "room:trait:hotheaded", "room:trait:kind", "room:trait:pious",
    "room:trait:stingy", "room:trait:gregarious", "room:trait:ambitious", "room:trait:lazy",
    "room:want:bread", "room:want:relief", "room:want:justice", "room:want:roof",
    "room:want:work", "room:want:mourning",
    "room:faction", "org:jailed", "org:weak", "org:strong", "org:leader_absent",
];

/// Facts that can't hold together: a beat needing two of one group can
/// never play
pub const EXCLUSIVE_FACTS: &[&[&str]] = &[
    &["rule:majority", "rule:consensus"],
    &["framing:grievance", "framing:foe", "framing:duty"],
    &["room:thin", "room:full"],
    &["history:first", "history:seasoned"],
    &["org:weak", "org:strong"],
    &["arc:won", "arc:lost", "arc:partial", "arc:delayed", "arc:diluted"],
    &["arc:first", "arc:veteran"],
    &["arc:weak", "arc:strong"],
];

/// Organized power below this is a weak org, at or above `ORG_STRONG` a
/// strong one (a meeting attended builds 0.15)
pub const ORG_WEAK: f32 = 0.3;
pub const ORG_STRONG: f32 = 1.0;

/// Most beats a house meeting gets through before the light goes
pub const MEETING_MAX_BEATS: usize = 5;

/// One beat as it played: the situation put, and the move taken
pub struct Step {
    pub beat: &'static Beat,
    pub mv: &'static Move,
    /// The variant the beat was put with, and the one the move answered
    pub call: &'static str,
    pub reply: &'static str,
}

fn holds(criteria: &[Criterion], facts: &Facts) -> bool {
    criteria.iter().all(|c| c.holds(facts))
}

/// The beat a phase opens with: unspent beats whose criteria hold, the
/// most specific tier first; ties draw evenly (seeded)
pub fn pick_beat(beats: &'static [Beat], phase: Phase, facts: &Facts,
                 spent: &HashSet<&'static str>, rng: &mut Rng) -> Option<&'static Beat> {
    let pool: Vec<&'static Beat> = beats.iter()
        .filter(|b| b.phase == phase && !spent.contains(b.id) && holds(b.criteria, facts))
        .collect();
    let top = pool.iter().map(|b| b.criteria.len()).max()?;
    let tier: Vec<&'static Beat> = pool.into_iter().filter(|b| b.criteria.len() == top).collect();
    Some(tier[rng.below(tier.len())])
}

/// A move's draw weight here: its base weight times every bias whose
/// fact holds
pub fn move_weight(m: &Move, facts: &Facts) -> f32 {
    m.bias.iter().filter(|(f, _)| facts.has(f)).fold(m.weight, |w, (_, k)| w * k)
}

/// The room's answer to a beat: a weighted draw over the eligible moves
pub fn pick_move(beat: &'static Beat, facts: &Facts, rng: &mut Rng) -> Option<&'static Move> {
    let pool: Vec<&'static Move> = beat.moves.iter().filter(|m| holds(m.criteria, facts)).collect();
    if pool.is_empty() { return None; }
    let weights: Vec<f32> = pool.iter().map(|m| move_weight(m, facts)).collect();
    Some(pool[rng.weighted(&weights)])
}

/// Walk a library from `start`: each phase puts its beat, the room picks a
/// move, the move names the next phase, until one closes, a phase has
/// nothing unspent, or `max` beats have played. Pure - no world access.
pub fn walk(beats: &'static [Beat], start: Phase, facts: &Facts, rng: &mut Rng, max: usize) -> Vec<Step> {
    let mut w = Walker::new(start);
    let mut steps = vec![];
    while steps.len() < max {
        let Some(s) = w.step(beats, facts, rng) else { break };
        steps.push(s);
    }
    steps
}

/// A walk taken one beat at a time, so a caller can apply each move's hook
/// and re-read the facts before the next beat is picked
pub struct Walker {
    /// Where the next beat comes from; None once the walk has closed
    pub phase: Option<Phase>,
    spent: HashSet<&'static str>,
}

impl Walker {
    pub fn new(start: Phase) -> Self {
        Walker { phase: Some(start), spent: HashSet::new() }
    }

    /// Play the next beat against `facts`; None once the walk closed or the
    /// phase has nothing unspent left to say
    pub fn step(&mut self, beats: &'static [Beat], facts: &Facts, rng: &mut Rng) -> Option<Step> {
        self.step_avoiding(beats, facts, &HashSet::new(), rng)
    }

    /// As `step`, but a beat in `avoid` (heard lately) plays only when the
    /// phase has nothing fresher to say - so a campaign whose state hasn't
    /// changed hears new lines before old ones, and never dead-ends for it
    pub fn step_avoiding(&mut self, beats: &'static [Beat], facts: &Facts,
                         avoid: &HashSet<&'static str>, rng: &mut Rng) -> Option<Step> {
        let phase = self.phase?;
        let fresh: HashSet<&'static str> = self.spent.union(avoid).copied().collect();
        let Some((beat, mv)) = pick_beat(beats, phase, facts, &fresh, rng)
            .or_else(|| pick_beat(beats, phase, facts, &self.spent, rng))
            .and_then(|b| pick_move(b, facts, rng).map(|m| (b, m))) else {
            self.phase = None;
            return None;
        };
        self.spent.insert(beat.id);
        let call = beat.lines[rng.below(beat.lines.len())];
        let reply = mv.lines[rng.below(mv.lines.len())];
        self.phase = match mv.next {
            Next::Phase(p) => Some(p),
            Next::Close => None,
        };
        Some(Step { beat, mv, call, reply })
    }
}

/// A line put in a speaker's mouth
pub fn fill(line: &str, issue: &str) -> String {
    line.replace("{issue}", issue)
}

/// The facts a campaign meeting is judged by: the campaign's rule and
/// framing, the seats whose holders came, its history, who came - their
/// temperaments and what they're short of - and the org around them
pub fn meeting_facts(world: &World, campaign: Entity, out: &MeetingOutcome, tick: u64) -> Facts {
    let personal: HashSet<String> = out.attendees.iter()
        .flat_map(|&a| personal_facts(world, a, tick)).collect();
    meeting_facts_with(world, campaign, out, &personal)
}

/// `meeting_facts` with the attendees' own facts already gathered: those
/// don't change while the room talks, so a meeting reads them once and
/// re-reads only the campaign (which hooks can change) between beats
pub fn meeting_facts_with(world: &World, campaign: Entity, out: &MeetingOutcome, personal: &HashSet<String>) -> Facts {
    let mut f = Facts::default();
    if let Some(c) = world.get_component::<Campaign>(campaign) {
        f.insert(match c.decision_rule { DecisionRule::Majority => "rule:majority", DecisionRule::Consensus => "rule:consensus" });
        f.insert(match c.framing {
            Framing::SharedGrievance => "framing:grievance",
            Framing::CommonFoe => "framing:foe",
            Framing::MoralDuty => "framing:duty",
        });
        // a seat speaks to the room only when its holder is in it
        for s in c.seats.iter().filter(|s| out.attendees.contains(&s.holder)) {
            f.insert(match s.role {
                Role::Chair => "seat:chair",
                Role::TurnoutCoordinator => "seat:coordinator",
                Role::NoteTaker => "seat:note_taker",
            });
        }
        if c.dissenters.iter().any(|d| !out.attendees.contains(d)) { f.insert("room:dissent_absent"); }
        if let Some(o) = world.get_component::<crate::sim::community::Organization>(c.org) {
            if !out.attendees.contains(&o.leader) { f.insert("org:leader_absent"); }
            let taken = o.members.iter().any(|&m| !crate::game::villager::is_dead(world, m)
                && crate::game::dungeon::unavailable(world, m));
            if taken { f.insert("org:jailed"); }
        }
        if let Some(p) = world.get_component::<crate::sim::organized_power::OrganizedPower>(c.org) {
            if p.capacity < ORG_WEAK { f.insert("org:weak"); }
            if p.capacity >= ORG_STRONG { f.insert("org:strong"); }
        }
    }
    for x in personal { f.insert(x.clone()); }
    if let Some(m) = world.get_component::<CampaignMeeting>(campaign) {
        if m.held <= 1 { f.insert("history:first"); }
        if m.held >= 4 { f.insert("history:seasoned"); }
    }
    if out.drifted { f.insert("room:drifted"); }
    if out.noted { f.insert("room:noted"); }
    if !out.healed.is_empty() { f.insert("room:dissent_heard"); }
    if out.healed.len() >= 2 { f.insert("room:faction"); }
    if out.attendees.len() < 3 { f.insert("room:thin"); }
    if out.attendees.len() >= 5 { f.insert("room:full"); }
    f
}

/// The `room:trait:*` and `room:want:*` facts one attendee brings in.
/// The room holds the union; a line about them belongs in their mouth.
pub fn personal_facts(world: &World, who: Entity, tick: u64) -> HashSet<String> {
    let mut out = HashSet::new();
    if let Some(p) = world.get_component::<crate::game::traits::Personality>(who) {
        for t in &p.traits { out.insert(format!("room:trait:{}", trait_fact(*t))); }
    }
    for pr in crate::game::desires::problems_of(world, who, tick) {
        if let Some(w) = want_fact(pr) { out.insert(format!("room:want:{}", w)); }
    }
    out
}

/// The personal facts a beat's criteria call for
fn personal_needs(criteria: &[Criterion]) -> Vec<&'static str> {
    criteria.iter().filter_map(|c| match c {
        Criterion::Has(f) if f.starts_with("room:trait:") || f.starts_with("room:want:") => Some(*f),
        _ => None,
    }).collect()
}

fn trait_fact(t: crate::game::traits::Trait) -> &'static str {
    use crate::game::traits::Trait::*;
    match t {
        Lazy => "lazy", Ambitious => "ambitious", Kind => "kind", Stingy => "stingy",
        Pious => "pious", Hotheaded => "hotheaded", Gregarious => "gregarious", Proud => "proud",
    }
}

/// What a live problem leaves a person short of, as the room hears it
fn want_fact(p: crate::game::desires::Problem) -> Option<&'static str> {
    use crate::game::desires::Problem::*;
    Some(match p {
        Broke | LongWinter | ColdHearth | SourGut => "bread",
        Taxed | WritBorne | DeathDuty | OverseerCut => "relief",
        Blamed | Robbed | ShortChanged | Loss => "justice",
        NoHome => "roof",
        NoWorkplace | NoMaster => "work",
        Grief => "mourning",
        _ => return None,
    })
}

/// Apply a chosen move's consequence. Returns whether anything changed.
/// `speaker` is who said it (None for the room or the other side).
pub fn apply_hook(world: &mut World, campaign: Entity, hook: Hook, speaker: Option<Entity>) -> bool {
    match hook {
        Hook::Nothing => false,
        Hook::Resolve => match world.get_component_mut::<CampaignMeeting>(campaign) {
            Some(m) => { m.resolves += 1; true }
            None => false,
        },
        Hook::Dissent => {
            let Some(who) = speaker else { return false };
            let leader = world.get_component::<Campaign>(campaign)
                .and_then(|c| world.get_component::<crate::sim::community::Organization>(c.org))
                .map(|o| o.leader);
            if leader == Some(who) { return false; }
            match world.get_component_mut::<Campaign>(campaign) {
                Some(c) if !c.dissenters.contains(&who) => { c.dissenters.push(who); true }
                _ => false,
            }
        }
        Hook::Reframe(fr) => match world.get_component_mut::<Campaign>(campaign) {
            Some(c) if c.framing != fr => { c.framing = fr; true }
            _ => false,
        },
    }
}

/// A line said at a campaign meeting
#[derive(Debug)]
pub struct Spoken {
    /// Who said it; None for the room speaking together or the other side
    pub speaker: Option<Entity>,
    /// The voice it was written for - tells the room from the other side
    pub voice: Voice,
    pub text: String,
    /// The beat it came from, and that beat's phase
    pub beat: &'static str,
    pub phase: Phase,
}

/// What a meeting talked through and what it changed
#[derive(Debug)]
pub struct Deliberation {
    pub said: Vec<Spoken>,
    /// The hooks that changed state, in order
    pub applied: Vec<Hook>,
    /// The room carried a decision (a Decision beat handed on to
    /// follow-through) rather than tabling, splitting or giving up
    pub decided: bool,
    /// Consent actually banked: a consensus room that decided with nobody
    /// going back to dissent. Supersedes `MeetingOutcome::consent`
    pub consent: bool,
}

impl Deliberation {
    /// The exchange that settled it: the last decision beat's two lines,
    /// or the closing beat's if the room never got to a decision
    pub fn decisive(&self) -> &[Spoken] {
        let at = self.said.iter().rposition(|s| s.phase == Phase::Decision)
            .map(|i| i - i % 2)
            .unwrap_or(self.said.len().saturating_sub(2));
        &self.said[at..(at + 2).min(self.said.len())]
    }
}

/// Talk a held meeting through: walk the house-meeting library against
/// the campaign's facts (seeded by campaign and day, so a replay says the
/// same), put each line in an attendee's mouth, and apply each move's hook
/// before the next beat is picked, so a reframe or a fresh dissent is what
/// the rest of the meeting hears. Call after `hold_campaign_meeting`; None
/// if nobody came.
///
/// Resolves: `hold_campaign_meeting` already counts one for a meeting the
/// note taker attended. It stands only if the room decided; a tabled or
/// split meeting takes it back. Meeting moves never add resolves of their
/// own (the audit keeps `Hook::Resolve` out of the meeting library), so a
/// meeting is at most one resolve.
pub fn deliberate_meeting(world: &mut World, campaign: Entity, out: &MeetingOutcome, tick: u64) -> Option<Deliberation> {
    let driver = *out.attendees.first()?;
    let issue = world.get_component::<Campaign>(campaign)?.issue.clone();
    let mut rng = Rng::new(tick.wrapping_mul(0x0100_0000_01B3) ^ ((campaign as u64) << 32));

    let others: Vec<Entity> = out.attendees.iter().copied().filter(|&a| a != driver).collect();
    let seated = |r: Role| crate::game::campaign::seat_holder(world, campaign, r)
        .filter(|n| out.attendees.contains(n));
    let (note_taker, chair) = (seated(Role::NoteTaker), seated(Role::Chair));
    // who brings which temperament and want - fixed for the evening
    let personal: Vec<(Entity, HashSet<String>)> = out.attendees.iter()
        .map(|&a| (a, personal_facts(world, a, tick))).collect();
    let room: HashSet<String> = personal.iter().flat_map(|(_, fs)| fs.iter().cloned()).collect();
    let holds_all = |a: Entity, needs: &[&str]| personal.iter()
        .find(|(e, _)| *e == a).map(|(_, fs)| needs.iter().all(|n| fs.contains(*n))).unwrap_or(false);
    // A member's line: the one the beat is about speaks it (`holders`);
    // whoever answers them is someone else when anyone else is there
    let member = |needs: &[&str], not: Option<Entity>, rng: &mut Rng| -> Entity {
        let pool = |cands: Vec<Entity>| -> Vec<Entity> { cands.into_iter().filter(|a| Some(*a) != not).collect() };
        let holders = pool(out.attendees.iter().copied().filter(|&a| !needs.is_empty() && holds_all(a, needs)).collect());
        let fallback = pool(others.clone());
        // in a two-person room the driver answers before anyone answers themself
        let anyone = pool(out.attendees.clone());
        let cands = [holders, fallback, anyone].into_iter().find(|c| !c.is_empty()).unwrap_or_else(|| others.clone());
        cands.get(rng.below(cands.len())).copied().unwrap_or(driver)
    };
    let voice_of = |v: Voice, needs: &[&str], not: Option<Entity>, rng: &mut Rng| -> Option<Entity> {
        match v {
            Voice::Driver => Some(driver),
            Voice::Member => Some(member(needs, not, rng)),
            Voice::Dissenter => out.healed.first().copied().or(Some(driver)),
            Voice::NoteTaker => note_taker.or(Some(driver)),
            Voice::Chair => chair.or(Some(driver)),
            Voice::Room | Voice::Opponent => None,
        }
    };
    let recent = crate::game::campaign_arc::recent_beats(world, campaign);
    let mut heard: Vec<&'static str> = vec![];
    let mut d = Deliberation { said: vec![], applied: vec![], decided: false, consent: false };
    let mut w = Walker::new(Phase::Agenda);
    for _ in 0..MEETING_MAX_BEATS {
        let facts = meeting_facts_with(world, campaign, out, &room);
        let Some(s) = w.step_avoiding(MEETING_BEATS, &facts, &recent, &mut rng) else { break };
        heard.push(s.beat.id);
        let beat_needs = personal_needs(s.beat.criteria);
        let caller = voice_of(s.beat.voice, &beat_needs, None, &mut rng);
        d.said.push(Spoken { speaker: caller, voice: s.beat.voice, text: fill(s.call, &issue), beat: s.beat.id, phase: s.beat.phase });
        // a reply's own needs pick its speaker; otherwise, when the call
        // came from someone else, the reply can be the one the beat's about
        let mv_needs = personal_needs(s.mv.criteria);
        let reply_needs: &[&str] = if !mv_needs.is_empty() { &mv_needs }
            else if s.beat.voice == Voice::Member { &[] } else { &beat_needs };
        let answerer = voice_of(s.mv.voice, reply_needs, caller, &mut rng);
        d.said.push(Spoken { speaker: answerer, voice: s.mv.voice, text: fill(s.reply, &issue), beat: s.beat.id, phase: s.beat.phase });
        if apply_hook(world, campaign, s.mv.hook, answerer) { d.applied.push(s.mv.hook); }
        if s.beat.phase == Phase::Decision && s.mv.next == Next::Phase(Phase::FollowThrough) {
            d.decided = true;
        }
    }
    if out.noted && !d.decided {
        if let Some(m) = world.get_component_mut::<CampaignMeeting>(campaign) {
            m.resolves = m.resolves.saturating_sub(1);
        }
    }
    d.consent = out.consent && d.decided && !d.applied.contains(&Hook::Dissent);
    // what the room settled carries into the campaign's arc, and what it
    // said is remembered so the next sitting hears fresh lines first
    crate::game::campaign_arc::record_consent(world, campaign, d.consent);
    crate::game::campaign_arc::remember(world, campaign, &heard);
    Some(d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::campaign::{delegate_role, found_campaign, hold_campaign_meeting, set_cadence, IssueVote};
    use crate::game::conversation::Criterion::{Has, Lacks};
    use crate::sim::community::Organization;
    use crate::sim::organized_power::OrganizedPower;
    use std::collections::HashMap;

    fn library(l: Library) -> &'static [Beat] {
        match l { Library::HouseMeeting => MEETING_BEATS, Library::CampaignArc => ARC_BEATS }
    }

    fn fact_of(c: &Criterion) -> &'static str {
        match c { Has(f) | Lacks(f) => f }
    }

    fn requires(criteria: &[&[Criterion]], fact: &str) -> bool {
        criteria.iter().flat_map(|cs| cs.iter()).any(|c| matches!(c, Has(f) if *f == fact))
    }

    // --- inventory: the audit every later stage reruns at scale ---

    #[test]
    fn test_libraries_are_counted_separately() {
        // at least the plan's thousand each - authors may add, not cut
        assert!(MEETING_BEATS.len() >= 1000, "house-meeting beats: {}", MEETING_BEATS.len());
        assert!(ARC_BEATS.len() >= 1000, "campaign-arc beats: {}", ARC_BEATS.len());
        for l in [Library::HouseMeeting, Library::CampaignArc] {
            assert!(library(l).iter().all(|b| b.phase.library() == l), "{:?} holds another library's phase", l);
        }
        // each library is spread across its phases, not piled in one
        for (beats, phases) in [(MEETING_BEATS, &Phase::MEETING[..]), (ARC_BEATS, &Phase::ARC[..])] {
            for p in phases {
                let n = beats.iter().filter(|b| b.phase == *p).count();
                assert!(n >= 75, "{:?} has only {} beats", p, n);
            }
        }
    }

    #[test]
    fn test_every_beat_can_match_some_room() {
        // no beat demands a fact and its absence, or two facts that can't
        // hold together - such a beat could never play
        for b in MEETING_BEATS.iter().chain(ARC_BEATS) {
            let has: Vec<&str> = b.criteria.iter().filter_map(|c| match c { Has(f) => Some(*f), _ => None }).collect();
            let lacks: Vec<&str> = b.criteria.iter().filter_map(|c| match c { Lacks(f) => Some(*f), _ => None }).collect();
            assert!(has.iter().all(|f| !lacks.contains(f)), "{} has and lacks one fact", b.id);
            for group in EXCLUSIVE_FACTS {
                assert!(has.iter().filter(|f| group.contains(f)).count() <= 1, "{} needs two of {:?}", b.id, group);
            }
        }
    }

    #[test]
    fn test_meeting_contexts_are_distinct_and_cover_the_room() {
        // many distinct situations, not one situation paraphrased
        let contexts: HashSet<(String, Vec<String>)> = MEETING_BEATS.iter().map(|b| {
            let mut cs: Vec<String> = b.criteria.iter().map(|c| format!("{:?}", c)).collect();
            cs.sort();
            (format!("{:?}", b.phase), cs)
        }).collect();
        eprintln!("{} distinct phase+criteria contexts", contexts.len());
        assert!(contexts.len() >= 450, "only {} distinct phase+criteria contexts", contexts.len());
        // every fact the runtime can read is something some beat speaks to
        // (in a meeting `seat:note_taker` and `room:noted` say the same thing -
        // both mean the note taker came - so content uses the latter)
        let runtime = KNOWN_FACTS.iter().filter(|f| !f.starts_with("arc:") && **f != "seat:note_taker");
        for f in runtime {
            let n = MEETING_BEATS.iter().filter(|b| requires(&[b.criteria], f)).count();
            assert!(n >= 3, "only {} meeting beats speak to {}", n, f);
        }
        // and every phase still has plenty to say when nothing's known
        for p in Phase::MEETING {
            let generic = MEETING_BEATS.iter().filter(|b| b.phase == p && b.criteria.is_empty()).count();
            assert!(generic >= 8, "{:?} has only {} criteria-free beats", p, generic);
        }
    }

    #[test]
    fn test_meeting_moves_cover_power_and_outcomes() {
        let moves: Vec<&Move> = MEETING_BEATS.iter().flat_map(|b| b.moves).collect();
        let dissents = moves.iter().filter(|m| m.hook == Hook::Dissent).count();
        let reframes = moves.iter().filter(|m| matches!(m.hook, Hook::Reframe(_))).count();
        let closes = moves.iter().filter(|m| m.next == Next::Close).count();
        assert!(dissents >= 30, "{} dissent moves", dissents);
        assert!(reframes >= 15, "{} reframe moves", reframes);
        assert!(closes >= 100, "{} moves can end a meeting", closes);
        for fr in [Framing::SharedGrievance, Framing::CommonFoe, Framing::MoralDuty] {
            assert!(moves.iter().any(|m| m.hook == Hook::Reframe(fr)), "nobody reframes to {:?}", fr);
        }
    }

    #[test]
    fn test_seeded_sample_reaches_most_of_the_meeting_library() {
        // a seeded spread of rooms: each draws a consistent handful of
        // facts, walks a meeting, and the sample is reported per phase
        let facts_pool: Vec<&str> = KNOWN_FACTS.iter().copied().filter(|f| !f.starts_with("arc:")).collect();
        let mut played: HashSet<&str> = HashSet::new();
        for seed in 0..6000u64 {
            let mut rng = Rng::new(seed);
            let mut f = Facts::default();
            let mut picked: Vec<&str> = vec![];
            for _ in 0..rng.below(5) {
                let x = facts_pool[rng.below(facts_pool.len())];
                let clash = EXCLUSIVE_FACTS.iter().any(|g| g.contains(&x) && picked.iter().any(|p| g.contains(p)));
                if !clash { picked.push(x); f.insert(x); }
            }
            for s in walk(MEETING_BEATS, Phase::Agenda, &f, &mut rng, MEETING_MAX_BEATS) {
                played.insert(s.beat.id);
            }
        }
        let share = played.len() as f32 / MEETING_BEATS.len() as f32;
        for p in Phase::MEETING {
            let all = MEETING_BEATS.iter().filter(|b| b.phase == p).count();
            let hit = MEETING_BEATS.iter().filter(|b| b.phase == p && played.contains(b.id)).count();
            eprintln!("{:?}: {}/{} beats played", p, hit, all);
        }
        eprintln!("sample reached {}/{} meeting beats", played.len(), MEETING_BEATS.len());
        assert!(share >= 0.9, "only {:.0}% of meeting beats ever played in the sample", share * 100.0);
    }

    // --- Stage 3: the campaign-arc library ---

    /// Facts an arc can be judged by: its own vocabulary plus the org,
    /// framing, rule and seat facts that exist outside any one meeting
    fn arc_fact(f: &str) -> bool {
        ["arc:", "org:", "framing:", "rule:", "seat:"].iter().any(|p| f.starts_with(p))
    }

    #[test]
    fn test_arc_beats_only_read_arc_scale_facts() {
        // a campaign isn't a room: no attendee temperaments, wants or
        // meeting history in the arc library
        for b in ARC_BEATS {
            for c in b.criteria.iter().chain(b.moves.iter().flat_map(|m| m.criteria)) {
                assert!(arc_fact(fact_of(c)), "{} reads meeting-scale fact {}", b.id, fact_of(c));
            }
        }
    }

    #[test]
    fn test_arc_contexts_are_distinct_and_cover_the_campaign() {
        let contexts: HashSet<(String, Vec<String>)> = ARC_BEATS.iter().map(|b| {
            let mut cs: Vec<String> = b.criteria.iter().map(|c| format!("{:?}", c)).collect();
            cs.sort();
            (format!("{:?}", b.phase), cs)
        }).collect();
        eprintln!("{} distinct arc phase+criteria contexts", contexts.len());
        assert!(contexts.len() >= 450, "only {} distinct arc contexts", contexts.len());
        for f in KNOWN_FACTS.iter().filter(|f| arc_fact(f)) {
            let n = ARC_BEATS.iter().filter(|b| requires(&[b.criteria], f)).count();
            assert!(n >= 3, "only {} arc beats speak to {}", n, f);
        }
        for p in Phase::ARC {
            let generic = ARC_BEATS.iter().filter(|b| b.phase == p && b.criteria.is_empty()).count();
            assert!(generic >= 8, "{:?} has only {} criteria-free beats", p, generic);
        }
    }

    #[test]
    fn test_every_outcome_kind_is_written_for() {
        // granted, denied, split, put off, hollowed out: each has its own
        // outcome beats
        for kind in ["arc:won", "arc:lost", "arc:partial", "arc:delayed", "arc:diluted"] {
            let n = ARC_BEATS.iter().filter(|b| b.phase == Phase::Outcome && requires(&[b.criteria], kind)).count();
            assert!(n >= 5, "only {} outcome beats for {}", n, kind);
        }
    }

    #[test]
    fn test_a_win_never_ends_the_campaign() {
        // won beats hand on to enforcement, renewal or more work - no move
        // from a beat that requires a win closes the arc
        let mut to_enforce = 0;
        for b in ARC_BEATS.iter().filter(|b| requires(&[b.criteria], "arc:won")) {
            for m in b.moves {
                assert_ne!(m.next, Next::Close, "{}.{}: a win that just ends", b.id, m.id);
                if m.next == Next::Phase(Phase::Enforce) { to_enforce += 1; }
            }
        }
        assert!(to_enforce >= 15, "only {} won moves hand on to enforcement", to_enforce);
    }

    #[test]
    fn test_a_loss_is_learned_from_and_still_costs() {
        // every denied outcome can hand on to learning; outside Renew,
        // nothing reached from a loss simply closes; and some losses carry
        // their cost forward as dissent rather than being smoothed over
        let lost: Vec<&Beat> = ARC_BEATS.iter().filter(|b| requires(&[b.criteria], "arc:lost")).collect();
        for b in lost.iter().filter(|b| b.phase == Phase::Outcome) {
            assert!(b.moves.iter().any(|m| m.next == Next::Phase(Phase::Learn)), "{}: a loss with no lesson", b.id);
        }
        for b in &lost {
            for m in b.moves {
                assert!(m.next != Next::Close || b.phase == Phase::Renew, "{}.{}: a loss that just ends", b.id, m.id);
            }
        }
        assert!(lost.iter().flat_map(|b| b.moves).any(|m| m.hook == Hook::Dissent), "losses never cost anyone's faith");
    }

    #[test]
    fn test_no_beat_is_keyed_only_on_absence() {
        // Lacks criteria still count toward specificity, so a beat that only
        // says what isn't so outranks every generic beat - and every beat
        // keyed on what did happen that it merely ties. Pair a Lacks with a Has.
        for b in MEETING_BEATS.iter().chain(ARC_BEATS) {
            if b.criteria.is_empty() { continue; }
            assert!(b.criteria.iter().any(|c| matches!(c, Has(_))), "{} is keyed only on absence", b.id);
        }
    }

    #[test]
    fn test_a_denied_campaign_never_guards_a_win() {
        // any outcome beat that can play once the campaign is lost - its own
        // criteria plus arc:lost - offers no move that hands on to Enforce
        let lost_group = EXCLUSIVE_FACTS.iter().find(|g| g.contains(&"arc:lost")).unwrap();
        let mut checked = 0;
        for b in ARC_BEATS.iter().filter(|b| b.phase == Phase::Outcome) {
            let needs: Vec<&str> = b.criteria.iter().filter_map(|c| match c { Has(f) => Some(*f), _ => None }).collect();
            if needs.iter().any(|f| *f != "arc:lost" && lost_group.contains(f)) { continue; }
            let mut f = facts(&needs);
            f.insert("arc:lost");
            if !holds(b.criteria, &f) { continue; }
            checked += 1;
            for m in b.moves.iter().filter(|m| holds(m.criteria, &f)) {
                assert_ne!(m.next, Next::Phase(Phase::Enforce), "{}.{}: a lost campaign guarding a win", b.id, m.id);
            }
            assert!(b.moves.iter().any(|m| holds(m.criteria, &f)), "{}: no way on after a loss", b.id);
        }
        assert!(checked >= 20, "only {} outcome beats can follow a loss", checked);
    }

    #[test]
    fn test_seeded_sample_reaches_most_of_the_arc_library() {
        let facts_pool: Vec<&str> = KNOWN_FACTS.iter().copied().filter(|f| arc_fact(f)).collect();
        let mut played: HashSet<&str> = HashSet::new();
        for seed in 0..8000u64 {
            let mut rng = Rng::new(seed);
            let mut f = Facts::default();
            let mut picked: Vec<&str> = vec![];
            for _ in 0..rng.below(5) {
                let x = facts_pool[rng.below(facts_pool.len())];
                let clash = EXCLUSIVE_FACTS.iter().any(|g| g.contains(&x) && picked.iter().any(|p| g.contains(p)));
                if !clash { picked.push(x); f.insert(x); }
            }
            let start = Phase::ARC[rng.below(Phase::ARC.len())];
            for s in walk(ARC_BEATS, start, &f, &mut rng, 12) {
                played.insert(s.beat.id);
            }
        }
        for p in Phase::ARC {
            let all = ARC_BEATS.iter().filter(|b| b.phase == p).count();
            let hit = ARC_BEATS.iter().filter(|b| b.phase == p && played.contains(b.id)).count();
            eprintln!("{:?}: {}/{} beats played", p, hit, all);
        }
        eprintln!("sample reached {}/{} arc beats", played.len(), ARC_BEATS.len());
        let share = played.len() as f32 / ARC_BEATS.len() as f32;
        assert!(share >= 0.9, "only {:.0}% of arc beats ever played in the sample", share * 100.0);
    }

    /// For authors: read a few campaigns as they'd play.
    /// `cargo test -p game2-core --lib sample_arcs -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn sample_arcs() {
        let arcs: [&[&str]; 5] = [
            &[],
            &["arc:strong", "arc:consent", "framing:foe"],
            &["arc:weak", "arc:crackdown", "arc:prisoners"],
            &["arc:won", "arc:allies"],
            &["arc:lost", "arc:split", "rule:majority"],
        ];
        for (i, arc) in arcs.iter().enumerate() {
            eprintln!("--- arc {:?}", arc);
            for s in walk(ARC_BEATS, Phase::Target, &facts(arc), &mut Rng::new(i as u64 + 3), 12) {
                eprintln!("[{}] {:?}: {}", s.beat.id, s.beat.voice, fill(s.call, "the mill toll"));
                eprintln!("    -> {:?}: {}", s.mv.voice, fill(s.reply, "the mill toll"));
            }
        }
    }

    /// For authors: read a few meetings as they'd play.
    /// `cargo test -p game2-core --lib sample_meetings -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn sample_meetings() {
        let rooms: [&[&str]; 6] = [
            &[],
            &["rule:consensus", "room:noted", "room:dissent_heard"],
            &["room:drifted", "org:leader_absent", "room:trait:hotheaded"],
            &["framing:foe", "room:want:bread", "org:weak"],
            &["org:jailed", "room:full", "room:trait:pious"],
            &["seat:chair", "room:faction", "rule:majority", "history:seasoned"],
        ];
        for (i, room) in rooms.iter().enumerate() {
            eprintln!("--- room {:?}", room);
            for s in walk(MEETING_BEATS, Phase::Agenda, &facts(room), &mut Rng::new(i as u64 + 7), MEETING_MAX_BEATS) {
                eprintln!("[{}] {:?}: {}", s.beat.id, s.beat.voice, fill(s.call, "the mill toll"));
                eprintln!("    -> {:?}: {}", s.mv.voice, fill(s.reply, "the mill toll"));
            }
        }
    }

    #[test]
    fn test_ids_unique_and_required_fields_present() {
        let mut ids = HashSet::new();
        for b in MEETING_BEATS.iter().chain(ARC_BEATS) {
            assert!(ids.insert(b.id), "duplicate beat id {}", b.id);
            assert!(!b.lines.is_empty() && !b.moves.is_empty() && !b.sources.is_empty(), "{} is missing a field", b.id);
            let mut mids = HashSet::new();
            for m in b.moves {
                assert!(mids.insert(m.id), "{} repeats move id {}", b.id, m.id);
                assert!(m.weight.is_finite() && m.weight > 0.0, "{}.{} weight", b.id, m.id);
                assert!(m.bias.iter().all(|(_, k)| k.is_finite() && *k > 0.0), "{}.{} bias", b.id, m.id);
                assert!(!m.lines.is_empty(), "{}.{} has no lines", b.id, m.id);
            }
            // every beat can always answer: one move with no criteria of its own
            assert!(b.moves.iter().any(|m| m.criteria.is_empty()), "{} can dead-end", b.id);
        }
    }

    #[test]
    fn test_every_fact_and_source_tag_resolves() {
        let tags: HashSet<&str> = SOURCE_PATTERNS.iter().map(|s| s.tag).collect();
        assert_eq!(tags.len(), SOURCE_PATTERNS.len(), "duplicate source tag");
        let mut cited = HashSet::new();
        for b in MEETING_BEATS.iter().chain(ARC_BEATS) {
            for c in b.criteria.iter().chain(b.moves.iter().flat_map(|m| m.criteria)) {
                assert!(KNOWN_FACTS.contains(&fact_of(c)), "{} tests unknown fact {}", b.id, fact_of(c));
            }
            for m in b.moves {
                for (f, _) in m.bias {
                    assert!(KNOWN_FACTS.contains(f), "{}.{} biases on unknown fact {}", b.id, m.id, f);
                }
            }
            for s in b.sources {
                assert!(tags.contains(s), "{} cites unknown source {}", b.id, s);
                cited.insert(*s);
            }
        }
        for s in SOURCE_PATTERNS {
            assert!(cited.contains(s.tag), "source {} is never cited", s.tag);
            assert!(!s.origin.is_empty() && !s.mechanism.is_empty());
        }
        // the record spans sacred, legendary, contested and documented
        for e in [Evidence::Sacred, Evidence::Legendary, Evidence::Contested, Evidence::Documented] {
            assert!(SOURCE_PATTERNS.iter().any(|s| s.evidence == e), "no {:?} source", e);
        }
    }

    #[test]
    fn test_every_phase_has_a_fallback_and_transitions_resolve() {
        for (l, phases, start) in [(Library::HouseMeeting, &Phase::MEETING[..], Phase::Agenda),
                                   (Library::CampaignArc, &Phase::ARC[..], Phase::Target)] {
            let beats = library(l);
            for p in phases {
                assert!(beats.iter().any(|b| b.phase == *p && b.criteria.is_empty()),
                    "{:?} has no criteria-free fallback", p);
            }
            for b in beats {
                for m in b.moves {
                    if let Next::Phase(p) = m.next {
                        assert_eq!(p.library(), l, "{}.{} hands off to another library", b.id, m.id);
                    }
                }
            }
            // every phase is reachable from the start
            let mut seen = HashSet::from([start]);
            let mut frontier = vec![start];
            while let Some(p) = frontier.pop() {
                for b in beats.iter().filter(|b| b.phase == p) {
                    for m in b.moves {
                        if let Next::Phase(n) = m.next {
                            if seen.insert(n) { frontier.push(n); }
                        }
                    }
                }
            }
            for p in phases { assert!(seen.contains(p), "{:?} is unreachable", p); }
            // and the walk can end
            assert!(beats.iter().flat_map(|b| b.moves).any(|m| m.next == Next::Close));
        }
    }

    #[test]
    fn test_voices_are_backed_by_criteria_and_hooks_are_sound() {
        for b in MEETING_BEATS.iter().chain(ARC_BEATS) {
            for m in b.moves {
                let crit: [&[Criterion]; 2] = [b.criteria, m.criteria];
                for v in [b.voice, m.voice] {
                    match v {
                        Voice::Dissenter => assert!(requires(&crit, "room:dissent_heard") || requires(&crit, "arc:split"),
                            "{}.{}: a dissenter speaks with none guaranteed", b.id, m.id),
                        Voice::NoteTaker => assert!(requires(&crit, "room:noted") || requires(&crit, "seat:note_taker"),
                            "{}.{}: the note taker speaks with none guaranteed", b.id, m.id),
                        Voice::Chair => assert!(requires(&crit, "seat:chair"),
                            "{}.{}: the chair speaks with none seated", b.id, m.id),
                        Voice::Opponent => assert_eq!(b.phase.library(), Library::CampaignArc,
                            "{}: the other side doesn't sit in the house meeting", b.id),
                        _ => {}
                    }
                }
                match m.hook {
                    // a meeting's one resolve is reconciled by deliberate_meeting
                    Hook::Resolve if b.phase.library() == Library::HouseMeeting =>
                        panic!("{}.{}: meeting moves don't add resolves", b.id, m.id),
                    Hook::Resolve => assert!(requires(&crit, "room:noted") || requires(&crit, "seat:note_taker"),
                        "{}.{}: a resolve with nobody to write it", b.id, m.id),
                    Hook::Dissent => assert!(matches!(m.voice, Voice::Member | Voice::Dissenter),
                        "{}.{}: only a member can go back to dissent", b.id, m.id),
                    _ => {}
                }
            }
        }
    }

    #[test]
    fn test_prompts_are_spoken_lines_not_captions() {
        // the line a beat opens with is said by someone; a three-word
        // heading ("The closed doors.") reads as a stage direction
        for b in MEETING_BEATS.iter().chain(ARC_BEATS) {
            for l in b.lines {
                assert!(l.split_whitespace().count() >= 4, "{}: caption-like prompt {:?}", b.id, l);
            }
        }
    }

    #[test]
    fn test_reading_attendee_facts_once_changes_nothing() {
        let (mut w, c, ms) = meeting_world(DecisionRule::Majority);
        crate::game::desires::add_grievance(&mut w, ms[1], crate::game::desires::Problem::Grief, 10_000);
        let out = hold_campaign_meeting(&mut w, c, 8).unwrap();
        let personal: HashSet<String> = out.attendees.iter().flat_map(|&a| personal_facts(&w, a, 8)).collect();
        let (a, b) = (meeting_facts(&w, c, &out, 8), meeting_facts_with(&w, c, &out, &personal));
        for f in KNOWN_FACTS { assert_eq!(a.has(f), b.has(f), "{} differs", f); }
        assert!(a.has("room:want:mourning"));
    }

    #[test]
    fn test_text_is_distinct_and_in_register() {
        let modern = ["okay", " ok ", "guys", "deadline", "meeting minutes", "agenda item", "stakeholder", "leverage", "vote yes"];
        for l in [Library::HouseMeeting, Library::CampaignArc] {
            let mut seen: HashMap<String, &str> = HashMap::new();
            for b in library(l) {
                for line in b.lines.iter().chain(b.moves.iter().flat_map(|m| m.lines)) {
                    let norm: String = line.to_lowercase().chars().filter(|c| c.is_alphanumeric() || *c == ' ').collect();
                    if let Some(prev) = seen.insert(norm, b.id) {
                        panic!("{} repeats a line from {}: {:?}", b.id, prev, line);
                    }
                    let lower = line.to_lowercase();
                    for w in modern { assert!(!lower.contains(w), "{}: modern diction {:?} in {:?}", b.id, w, line); }
                    let stripped = line.replace("{issue}", "");
                    assert!(!stripped.contains('{') && !stripped.contains('}'), "{}: stray placeholder in {:?}", b.id, line);
                    assert!(line.len() <= 140, "{}: overlong line {:?}", b.id, line);
                }
            }
        }
    }

    #[test]
    fn test_coverage_spans_power_outcomes_and_renewal() {
        let facts: HashSet<&str> = MEETING_BEATS.iter().chain(ARC_BEATS)
            .flat_map(|b| b.criteria.iter().chain(b.moves.iter().flat_map(|m| m.criteria)))
            .map(fact_of).collect();
        for f in ["room:drifted", "seat:chair", "room:dissent_heard", "rule:consensus",
                  "arc:won", "arc:lost", "arc:partial", "arc:crackdown", "arc:promise_broken"] {
            assert!(facts.contains(f), "no content speaks to {}", f);
        }
        let hooks: Vec<Hook> = MEETING_BEATS.iter().chain(ARC_BEATS).flat_map(|b| b.moves).map(|m| m.hook).collect();
        assert!(hooks.contains(&Hook::Resolve) && hooks.contains(&Hook::Dissent));
        assert!(hooks.iter().any(|h| matches!(h, Hook::Reframe(_))));
        // a won campaign doesn't just end: some won beat hands on to enforcement
        assert!(ARC_BEATS.iter().filter(|b| requires(&[b.criteria], "arc:won"))
            .flat_map(|b| b.moves).any(|m| m.next == Next::Phase(Phase::Enforce)));
    }

    // --- selection ---

    fn facts(xs: &[&str]) -> Facts {
        let mut f = Facts::default();
        for x in xs { f.insert(*x); }
        f
    }

    #[test]
    fn test_most_specific_beat_leads_and_generic_falls_back() {
        let spent = HashSet::new();
        let bare = facts(&[]);
        let b = pick_beat(MEETING_BEATS, Phase::Agenda, &bare, &spent, &mut Rng::new(1)).unwrap();
        assert!(b.criteria.is_empty(), "nothing known: the generic opener");
        let drifted = facts(&["room:drifted"]);
        let b = pick_beat(MEETING_BEATS, Phase::Agenda, &drifted, &spent, &mut Rng::new(1)).unwrap();
        assert!(b.criteria.len() >= 1, "a drifted room gets its own opener, got {}", b.id);
        // everything specific spent: the generic tail still answers
        let all_but_generic: HashSet<&'static str> = MEETING_BEATS.iter()
            .filter(|b| b.phase == Phase::Agenda && !b.criteria.is_empty()).map(|b| b.id).collect();
        let b = pick_beat(MEETING_BEATS, Phase::Agenda, &drifted, &all_but_generic, &mut Rng::new(1)).unwrap();
        assert!(b.criteria.is_empty());
    }

    #[test]
    fn test_seeded_walk_replays_exactly() {
        let f = facts(&["rule:consensus", "framing:foe", "room:noted", "seat:note_taker"]);
        let ids = |seed| walk(MEETING_BEATS, Phase::Agenda, &f, &mut Rng::new(seed), MEETING_MAX_BEATS)
            .iter().map(|s| (s.beat.id, s.mv.id, s.call, s.reply)).collect::<Vec<_>>();
        for seed in 0..20 {
            let a = ids(seed);
            assert!(!a.is_empty());
            assert_eq!(a, ids(seed), "seed {} didn't replay", seed);
        }
        assert!((0..20).map(ids).collect::<HashSet<_>>().len() > 5, "seeds should vary the walk");
    }

    #[test]
    fn test_walks_end_and_never_reuse_a_beat() {
        for (beats, start) in [(MEETING_BEATS, Phase::Agenda), (ARC_BEATS, Phase::Target)] {
            for seed in 0..200 {
                let f = facts(&KNOWN_FACTS[(seed as usize) % KNOWN_FACTS.len()..][..1]);
                let steps = walk(beats, start, &f, &mut Rng::new(seed), 12);
                assert!(!steps.is_empty());
                let ids: HashSet<&str> = steps.iter().map(|s| s.beat.id).collect();
                assert_eq!(ids.len(), steps.len(), "a beat replayed in one walk");
            }
        }
    }

    #[test]
    fn test_context_biases_without_gating() {
        // the consensus room leans to talking an objection out; a majority
        // room leans to calling the question - both stay possible
        let count = |f: &Facts| {
            let mut by_next: HashMap<String, usize> = HashMap::new();
            for seed in 0..2000 {
                let b = MEETING_BEATS.iter().find(|b| b.id == "hm_objection_generic").unwrap();
                let m = pick_move(b, f, &mut Rng::new(seed)).unwrap();
                *by_next.entry(format!("{:?}", m.next)).or_default() += 1;
            }
            by_next
        };
        let cons = count(&facts(&["rule:consensus"]));
        let maj = count(&facts(&["rule:majority"]));
        let talk = "Phase(Amendment)".to_string();
        let call = "Phase(Decision)".to_string();
        assert!(cons[&talk] > maj[&talk], "consensus should amend more: {:?} vs {:?}", cons, maj);
        assert!(maj[&call] > cons[&call], "majority should call the question more");
        assert!(cons.get(&call).copied().unwrap_or(0) > 0 && maj.get(&talk).copied().unwrap_or(0) > 0,
            "bias, not a gate");
    }

    #[test]
    fn test_arc_outcomes_continue_past_a_win_and_a_loss() {
        for (fact, want) in [("arc:won", Phase::Enforce), ("arc:lost", Phase::Learn)] {
            let f = facts(&[fact]);
            let reached = (0..200).any(|seed| {
                walk(ARC_BEATS, Phase::Outcome, &f, &mut Rng::new(seed), 2).iter()
                    .any(|s| s.mv.next == Next::Phase(want))
            });
            assert!(reached, "{} never hands on to {:?}", fact, want);
        }
    }

    // --- runtime: the meeting path ---

    fn meeting_world(rule: DecisionRule) -> (World, Entity, Vec<Entity>) {
        use crate::game::relationships::{record_interaction, Interaction, InteractionKind};
        let mut w = World::new();
        let ms: Vec<Entity> = (0..4).map(|_| {
            let m = w.create_entity();
            w.add_component(m, crate::game::desires::InnerLife { wants: vec![], grievances: vec![], satisfied: vec![] });
            m
        }).collect();
        for &m in &ms[1..] {
            for _ in 0..2 {
                record_interaction(&mut w, ms[0], Interaction { other: m, kind: InteractionKind::Aid, intensity: 1.0 });
            }
        }
        let o = w.create_entity();
        let leader = w.create_entity();
        w.add_component(o, Organization { name: "The League".into(), members: ms.clone(), leader,
            influence_pool: 0.0, resources: 0.0, charter: "".into() });
        w.add_component(o, OrganizedPower::new());
        let vote = IssueVote { winner: "the mill toll".into(), ballots: vec![] };
        let c = found_campaign(&mut w, o, &vote, Framing::CommonFoe, rule, 1);
        w.get_component_mut::<Campaign>(c).unwrap().dissenters.push(ms[2]);
        set_cadence(&mut w, c, 7, 1);
        delegate_role(&mut w, c, Role::TurnoutCoordinator, ms[0]);
        delegate_role(&mut w, c, Role::NoteTaker, ms[1]);
        (w, c, ms)
    }

    #[test]
    fn test_meeting_facts_read_campaign_state() {
        let (mut w, c, ms) = meeting_world(DecisionRule::Consensus);
        let out = hold_campaign_meeting(&mut w, c, 8).unwrap();
        let f = meeting_facts(&w, c, &out, 8);
        for x in ["rule:consensus", "framing:foe", "seat:coordinator", "seat:note_taker",
                  "room:noted", "room:dissent_heard", "history:first"] {
            assert!(f.has(x), "missing {}", x);
        }
        assert!(!f.has("room:drifted") && !f.has("room:thin") && !f.has("room:dissent_absent"));
        assert_eq!(out.healed, vec![ms[2]]);
    }

    #[test]
    fn test_deliberation_is_deterministic_and_speaks_through_attendees() {
        let run = || {
            let (mut w, c, _) = meeting_world(DecisionRule::Majority);
            let out = hold_campaign_meeting(&mut w, c, 8).unwrap();
            let d = deliberate_meeting(&mut w, c, &out, 8).unwrap();
            for s in &d.said {
                if let Some(e) = s.speaker { assert!(out.attendees.contains(&e), "{:?} wasn't there", s); }
                assert!(!s.text.contains("{issue}"));
            }
            d.said.iter().map(|s| (s.speaker, s.text.clone())).collect::<Vec<_>>()
        };
        let a = run();
        assert!(a.len() >= 2 && a.len() % 2 == 0, "every beat is a call and a reply");
        assert_eq!(a, run(), "same campaign, same day, same talk");
    }

    #[test]
    fn test_hooks_change_only_what_they_name() {
        let (mut w, c, ms) = meeting_world(DecisionRule::Majority);
        let before = w.get_component::<CampaignMeeting>(c).unwrap().resolves;
        assert!(apply_hook(&mut w, c, Hook::Resolve, None));
        assert_eq!(w.get_component::<CampaignMeeting>(c).unwrap().resolves, before + 1);

        assert!(!apply_hook(&mut w, c, Hook::Nothing, Some(ms[0])));
        assert!(!apply_hook(&mut w, c, Hook::Dissent, None), "the room can't dissent from itself");
        assert!(!apply_hook(&mut w, c, Hook::Dissent, Some(ms[2])), "already dissenting");
        assert!(apply_hook(&mut w, c, Hook::Dissent, Some(ms[3])));
        assert!(w.get_component::<Campaign>(c).unwrap().dissenters.contains(&ms[3]));
        let leader = w.get_component::<Organization>(w.get_component::<Campaign>(c).unwrap().org).unwrap().leader;
        assert!(!apply_hook(&mut w, c, Hook::Dissent, Some(leader)), "the leader doesn't dissent from their own cause");

        assert!(!apply_hook(&mut w, c, Hook::Reframe(Framing::CommonFoe), None), "same framing, no change");
        assert!(apply_hook(&mut w, c, Hook::Reframe(Framing::MoralDuty), None));
        assert_eq!(w.get_component::<Campaign>(c).unwrap().framing, Framing::MoralDuty);
    }

    #[test]
    fn test_deliberation_hooks_land_across_seeds() {
        // over many days the meeting path exercises its hooks for real, and
        // a meeting is one resolve if it decided and none if it didn't
        let mut applied: Vec<Hook> = vec![];
        let (mut decided, mut undecided, mut blocked) = (0, 0, 0);
        for day in 8..208 {
            let rule = if day % 2 == 0 { DecisionRule::Majority } else { DecisionRule::Consensus };
            let (mut w, c, _) = meeting_world(rule);
            let before = w.get_component::<CampaignMeeting>(c).unwrap().resolves;
            let out = hold_campaign_meeting(&mut w, c, day).unwrap();
            assert!(out.noted);
            let d = deliberate_meeting(&mut w, c, &out, day).unwrap();
            let after = w.get_component::<CampaignMeeting>(c).unwrap().resolves;
            assert_eq!(after, before + d.decided as u32, "day {}: {:?}", day, d.said);
            if d.decided { decided += 1 } else { undecided += 1 }
            let key = d.decisive();
            assert_eq!(key.len(), 2);
            assert_eq!(key[0].beat, key[1].beat, "the decisive pair is one beat's call and reply");
            if d.decided { assert_eq!(key[0].phase, Phase::Decision); }
            if rule == DecisionRule::Consensus && d.applied.contains(&Hook::Dissent) {
                assert!(!d.consent, "a dissent withholds consent");
                blocked += 1;
            }
            assert!(!d.consent || (d.decided && rule == DecisionRule::Consensus));
            applied.extend(d.applied);
        }
        assert!(decided > 0 && undecided > 0, "decided {} undecided {}", decided, undecided);
        assert!(blocked > 0, "no consensus meeting ever saw a dissent");
        assert!(applied.contains(&Hook::Dissent));
        assert!(applied.iter().any(|h| matches!(h, Hook::Reframe(_))));
    }

    #[test]
    fn test_personal_lines_go_to_the_person() {
        // only ms[1] grieves: a mourning beat's Member call is always theirs,
        // and the reply to it comes from someone else
        let mut checked = 0;
        for day in 8..408 {
            let (mut w, c, ms) = meeting_world(DecisionRule::Majority);
            crate::game::desires::add_grievance(&mut w, ms[1], crate::game::desires::Problem::Grief, 10_000);
            let out = hold_campaign_meeting(&mut w, c, day).unwrap();
            assert!(meeting_facts(&w, c, &out, day).has("room:want:mourning"));
            let d = deliberate_meeting(&mut w, c, &out, day).unwrap();
            for pair in d.said.chunks(2) {
                let b = MEETING_BEATS.iter().find(|b| b.id == pair[0].beat).unwrap();
                if b.voice == Voice::Member && personal_needs(b.criteria).contains(&"room:want:mourning") {
                    assert_eq!(pair[0].speaker, Some(ms[1]), "{}: grief voiced by someone else", b.id);
                    assert_ne!(pair[1].speaker, Some(ms[1]), "{}: the griever answered themself", b.id);
                    checked += 1;
                }
            }
        }
        assert!(checked > 0, "no mourning beat with a member's call ever played");
    }

    #[test]
    fn test_nobody_answers_themself_in_a_two_person_room() {
        for day in 8..408 {
            let (mut w, c, ms) = meeting_world(DecisionRule::Majority);
            let out = MeetingOutcome { attendees: vec![ms[0], ms[1]], healed: vec![],
                drifted: false, noted: false, consent: false };
            let d = deliberate_meeting(&mut w, c, &out, day).unwrap();
            for pair in d.said.chunks(2) {
                let b = MEETING_BEATS.iter().find(|b| b.id == pair[0].beat).unwrap();
                let mv = b.moves.iter().find(|m| m.lines.iter().any(|l| fill(l, "the mill toll") == pair[1].text)).unwrap();
                if mv.voice == Voice::Member && pair[0].speaker.is_some() {
                    assert_ne!(pair[0].speaker, pair[1].speaker, "day {}: {} answered itself", day, b.id);
                }
            }
        }
    }

    #[test]
    fn test_an_absent_chair_runs_nothing() {
        let (mut w, c, ms) = meeting_world(DecisionRule::Majority);
        // a chair with no tie to the coordinator stays home
        let stranger = w.create_entity();
        let org = w.get_component::<Campaign>(c).unwrap().org;
        w.get_component_mut::<Organization>(org).unwrap().members.push(stranger);
        assert!(delegate_role(&mut w, c, Role::Chair, stranger));
        let out = hold_campaign_meeting(&mut w, c, 8).unwrap();
        assert!(!out.attendees.contains(&stranger));
        let f = meeting_facts(&w, c, &out, 8);
        assert!(!f.has("seat:chair"), "the chair isn't in the room");
        assert!(f.has("seat:coordinator") && f.has("seat:note_taker"));
        // seated and present: the chair's beats are theirs to speak
        let (mut w, c, ms2) = meeting_world(DecisionRule::Majority);
        assert!(delegate_role(&mut w, c, Role::Chair, ms2[3]));
        let out = hold_campaign_meeting(&mut w, c, 8).unwrap();
        assert!(meeting_facts(&w, c, &out, 8).has("seat:chair"));
        let _ = ms;
    }

    #[test]
    fn test_a_reframe_is_heard_by_the_rest_of_the_meeting() {
        // walk the meeting step by step: after a reframe, no later beat is
        // picked on the old framing
        for day in 8..408 {
            let (mut w, c, _) = meeting_world(DecisionRule::Majority);
            let out = hold_campaign_meeting(&mut w, c, day).unwrap();
            let d = deliberate_meeting(&mut w, c, &out, day).unwrap();
            let beats: Vec<&str> = d.said.iter().map(|s| s.beat).collect();
            if let Some(i) = beats.iter().position(|b| *b == "hm_testimony_foe") {
                if d.applied.contains(&Hook::Reframe(Framing::SharedGrievance)) {
                    assert!(!beats[i + 2..].contains(&"hm_amend_foe_to_duty"),
                        "day {}: a foe-framed amendment after the room dropped the foe", day);
                }
            }
        }
    }
}
