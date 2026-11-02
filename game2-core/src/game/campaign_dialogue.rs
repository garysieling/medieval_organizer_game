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

pub use crate::game::campaign_beats::{ARC_BEATS, MEETING_BEATS, SOURCE_PATTERNS};

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
/// state in a later stage.
pub const KNOWN_FACTS: &[&str] = &[
    "rule:majority", "rule:consensus",
    "framing:grievance", "framing:foe", "framing:duty",
    "room:drifted", "room:noted", "room:dissent_heard", "room:dissent_absent",
    "room:thin", "room:full",
    "seat:chair", "seat:coordinator", "seat:note_taker",
    "history:first", "history:seasoned",
    "arc:weak", "arc:strong", "arc:consent", "arc:crackdown", "arc:prisoners",
    "arc:won", "arc:lost", "arc:partial", "arc:promise_broken", "arc:split",
];

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
        let phase = self.phase?;
        let Some((beat, mv)) = pick_beat(beats, phase, facts, &self.spent, rng)
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
/// framing, the seats whose holders came, its history, and who came
pub fn meeting_facts(world: &World, campaign: Entity, out: &MeetingOutcome) -> Facts {
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
    }
    if let Some(m) = world.get_component::<CampaignMeeting>(campaign) {
        if m.held <= 1 { f.insert("history:first"); }
        if m.held >= 4 { f.insert("history:seasoned"); }
    }
    if out.drifted { f.insert("room:drifted"); }
    if out.noted { f.insert("room:noted"); }
    if !out.healed.is_empty() { f.insert("room:dissent_heard"); }
    if out.attendees.len() < 3 { f.insert("room:thin"); }
    if out.attendees.len() >= 5 { f.insert("room:full"); }
    f
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
    /// Who said it; None for the room speaking together
    pub speaker: Option<Entity>,
    pub text: String,
    /// The beat it came from
    pub beat: &'static str,
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
    let voice_of = |v: Voice, rng: &mut Rng| -> Option<Entity> {
        match v {
            Voice::Driver => Some(driver),
            Voice::Member => Some(others.get(rng.below(others.len())).copied().unwrap_or(driver)),
            Voice::Dissenter => out.healed.first().copied().or(Some(driver)),
            Voice::NoteTaker => note_taker.or(Some(driver)),
            Voice::Chair => chair.or(Some(driver)),
            Voice::Room | Voice::Opponent => None,
        }
    };
    let mut d = Deliberation { said: vec![], applied: vec![], decided: false, consent: false };
    let mut w = Walker::new(Phase::Agenda);
    for _ in 0..MEETING_MAX_BEATS {
        let facts = meeting_facts(world, campaign, out);
        let Some(s) = w.step(MEETING_BEATS, &facts, &mut rng) else { break };
        let caller = voice_of(s.beat.voice, &mut rng);
        d.said.push(Spoken { speaker: caller, text: fill(s.call, &issue), beat: s.beat.id });
        let answerer = voice_of(s.mv.voice, &mut rng);
        d.said.push(Spoken { speaker: answerer, text: fill(s.reply, &issue), beat: s.beat.id });
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
    fn test_libraries_are_counted_separately_at_the_stage_one_tranche() {
        assert_eq!(MEETING_BEATS.len(), 50, "house-meeting beats");
        assert_eq!(ARC_BEATS.len(), 50, "campaign-arc beats");
        for l in [Library::HouseMeeting, Library::CampaignArc] {
            assert!(library(l).iter().all(|b| b.phase.library() == l), "{:?} holds another library's phase", l);
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
        let f = meeting_facts(&w, c, &out);
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
        assert!(a.len() >= 2);
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
    fn test_an_absent_chair_runs_nothing() {
        let (mut w, c, ms) = meeting_world(DecisionRule::Majority);
        // a chair with no tie to the coordinator stays home
        let stranger = w.create_entity();
        let org = w.get_component::<Campaign>(c).unwrap().org;
        w.get_component_mut::<Organization>(org).unwrap().members.push(stranger);
        assert!(delegate_role(&mut w, c, Role::Chair, stranger));
        let out = hold_campaign_meeting(&mut w, c, 8).unwrap();
        assert!(!out.attendees.contains(&stranger));
        let f = meeting_facts(&w, c, &out);
        assert!(!f.has("seat:chair"), "the chair isn't in the room");
        assert!(f.has("seat:coordinator") && f.has("seat:note_taker"));
        // seated and present: the chair's beats are theirs to speak
        let (mut w, c, ms2) = meeting_world(DecisionRule::Majority);
        assert!(delegate_role(&mut w, c, Role::Chair, ms2[3]));
        let out = hold_campaign_meeting(&mut w, c, 8).unwrap();
        assert!(meeting_facts(&w, c, &out).has("seat:chair"));
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
