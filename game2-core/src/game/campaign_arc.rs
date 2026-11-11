//! Campaign arcs at runtime: read the campaign's state as arc facts, walk
//! the arc library after a public action, voice it through the org, and
//! apply the moves' hooks. Stage 4 of
//! wiki/software-design/campaign-dialogue-trees-plan.md.
//!
//! What the arc facts are read from:
//! - outcomes: a live `LordGrant` for the campaign's issue is a win
//!   (`arc:won`); the last townhall's answer gives `arc:lost`
//!   (doors shut) or `arc:delayed` (a clerk and regards). A grant of part
//!   of the demand is `arc:partial`; one hollowed because nobody kept
//!   watch (`reactions::hollow_if_unguarded`) is `arc:diluted` and a
//!   broken promise (`arc:promise_broken`) - taken back while still in
//!   force. One that simply ran its term is `arc:expired`: renew it.
//! - the other side: `GuardsPosted` is `arc:crackdown`, an `AssemblyBan`
//!   is `arc:countered`; members held are `arc:prisoners`, the leader
//!   held is `arc:leader_taken`.
//! - the campaign itself: organized power (`arc:weak`/`arc:strong`), two
//!   or more dissenters (`arc:split`), half the members demoralized
//!   (`arc:fatigue`), two or more outsiders woken to the issue
//!   (`arc:allies`), consent banked at the last meeting (`arc:consent`),
//!   and how many actions it has taken (`arc:first`/`arc:veteran`).

use std::collections::VecDeque;
use crate::engine::det::HashSet;

use crate::engine::ecs::{Entity, World};
use crate::game::actions::{Demoralized, TargetResponse};
use crate::game::campaign::{Campaign, DecisionRule, Framing, Role};
use crate::game::campaign_dialogue::{apply_hook, fill, Deliberation, Phase, Spoken, Voice, Walker,
    ARC_BEATS, ORG_STRONG, ORG_WEAK};
use crate::game::conversation::{Facts, Rng};
use crate::game::dungeon::unavailable;
use crate::game::reactions::{AssemblyBan, GuardsPosted, LordGrant, Reaction, Sympathized};
use crate::game::villager::is_dead;
use crate::sim::community::Organization;
use crate::sim::organized_power::OrganizedPower;

/// What the sim doesn't otherwise keep about a campaign's arc: its last
/// answer from the target, how many public actions it has taken, and
/// whether its last meeting banked consent
#[derive(Debug, Default)]
pub struct ArcRecord {
    pub actions: u32,
    pub last_response: Option<TargetResponse>,
    pub consent: bool,
}

/// Beats a campaign has heard lately, meetings and arcs both: a walk
/// prefers fresh ones so the same situation doesn't replay the same lines
#[derive(Debug, Default)]
pub struct DialogueMemory {
    pub recent: VecDeque<&'static str>,
}

/// How many recent beats a campaign remembers
pub const MEMORY_BEATS: usize = 40;
/// Most beats an arc walks after one action
pub const ARC_MAX_BEATS: usize = 3;
/// At least this many actions makes a campaign a veteran
pub const VETERAN_ACTIONS: u32 = 5;
/// Outsiders woken to the issue before the campaign has allies
pub const ALLIES_AT: usize = 2;

/// A public action the campaign has just taken
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArcEvent {
    /// A march, and how the powers answered it
    March(Reaction),
    /// A strike ballot - called, or failed in public
    Strike { called: bool },
    /// Demands put to the target, and its answer
    Townhall(TargetResponse),
}

impl ArcEvent {
    /// Where the arc picks up after this event: a march nobody noticed was
    /// a structure test; one that was noticed draws the other side's move;
    /// a called strike likewise; a failed one is a lesson; a townhall is
    /// an outcome
    pub fn start(self) -> Phase {
        match self {
            ArcEvent::March(Reaction::Ignored) => Phase::Test,
            ArcEvent::March(_) => Phase::Counter,
            ArcEvent::Strike { called: true } => Phase::Counter,
            ArcEvent::Strike { called: false } => Phase::Learn,
            ArcEvent::Townhall(_) => Phase::Outcome,
        }
    }
}

/// Note an action on the campaign's record (before reading its facts)
pub fn record_event(world: &mut World, campaign: Entity, ev: ArcEvent) {
    if world.get_component::<ArcRecord>(campaign).is_none() {
        world.add_component(campaign, ArcRecord::default());
    }
    if let Some(r) = world.get_component_mut::<ArcRecord>(campaign) {
        r.actions += 1;
        if let ArcEvent::Townhall(resp) = ev { r.last_response = Some(resp); }
    }
}

/// Note whether the campaign's last meeting banked consent
pub fn record_consent(world: &mut World, campaign: Entity, consent: bool) {
    if world.get_component::<ArcRecord>(campaign).is_none() {
        world.add_component(campaign, ArcRecord::default());
    }
    if let Some(r) = world.get_component_mut::<ArcRecord>(campaign) { r.consent = consent; }
}

/// The beats a campaign heard lately
pub fn recent_beats(world: &World, campaign: Entity) -> HashSet<&'static str> {
    world.get_component::<DialogueMemory>(campaign)
        .map(|m| m.recent.iter().copied().collect()).unwrap_or_default()
}

/// Remember beats a campaign just heard, forgetting the oldest
pub fn remember(world: &mut World, campaign: Entity, beats: &[&'static str]) {
    if world.get_component::<DialogueMemory>(campaign).is_none() {
        world.add_component(campaign, DialogueMemory::default());
    }
    if let Some(m) = world.get_component_mut::<DialogueMemory>(campaign) {
        for b in beats {
            m.recent.push_back(b);
            while m.recent.len() > MEMORY_BEATS { m.recent.pop_front(); }
        }
    }
}

/// The campaign's state as arc facts (see the module notes)
pub fn arc_facts(world: &World, campaign: Entity, tick: u64) -> Facts {
    let mut f = Facts::default();
    let Some(c) = world.get_component::<Campaign>(campaign) else { return f };
    f.insert(match c.decision_rule { DecisionRule::Majority => "rule:majority", DecisionRule::Consensus => "rule:consensus" });
    f.insert(match c.framing {
        Framing::SharedGrievance => "framing:grievance",
        Framing::CommonFoe => "framing:foe",
        Framing::MoralDuty => "framing:duty",
    });
    // a seat counts while its holder is free to act
    for s in c.seats.iter().filter(|s| !unavailable(world, s.holder)) {
        f.insert(match s.role {
            Role::Chair => "seat:chair",
            Role::TurnoutCoordinator => "seat:coordinator",
            Role::NoteTaker => "seat:note_taker",
        });
    }
    // a split is two dissenters free to say so - the held can't
    if c.dissenters.iter().filter(|&&d| !unavailable(world, d)).count() >= 2 { f.insert("arc:split"); }
    let (org, issue) = (c.org, c.issue.clone());

    if let Some(o) = world.get_component::<Organization>(org) {
        let held = |m: Entity| !is_dead(world, m) && unavailable(world, m);
        if o.members.iter().any(|&m| held(m)) {
            f.insert("arc:prisoners");
            f.insert("org:jailed");
        }
        if held(o.leader) { f.insert("arc:leader_taken"); }
        if unavailable(world, o.leader) { f.insert("org:leader_absent"); }
        let free: Vec<Entity> = o.members.iter().copied().filter(|&m| !unavailable(world, m)).collect();
        let low = free.iter().filter(|&&m| world.get_component::<Demoralized>(m).is_some()).count();
        if !free.is_empty() && low * 2 >= free.len() { f.insert("arc:fatigue"); }
        let woken = world.query::<Sympathized>().into_iter()
            .filter(|&v| !o.members.contains(&v) && v != o.leader && !unavailable(world, v))
            .filter(|&v| world.get_component::<Sympathized>(v)
                .map(|s| s.issues.iter().any(|(i, _)| crate::game::desires::covers(&issue, i))).unwrap_or(false))
            .count();
        if woken >= ALLIES_AT { f.insert("arc:allies"); }
    }
    if let Some(p) = world.get_component::<OrganizedPower>(org) {
        if p.capacity < ORG_WEAK { f.insert("arc:weak"); f.insert("org:weak"); }
        if p.capacity >= ORG_STRONG { f.insert("arc:strong"); f.insert("org:strong"); }
    }
    if world.get_component::<GuardsPosted>(org).is_some() { f.insert("arc:crackdown"); }
    if world.get_component::<AssemblyBan>(org).is_some() { f.insert("arc:countered"); }

    let record = world.get_component::<ArcRecord>(campaign);
    let actions = record.map(|r| r.actions).unwrap_or(0);
    if actions <= 1 { f.insert("arc:first"); }
    if actions >= VETERAN_ACTIONS { f.insert("arc:veteran"); }
    if record.map(|r| r.consent).unwrap_or(false) { f.insert("arc:consent"); }

    // the outcome - one at most (they're exclusive): a live grant beats
    // the last answer at the door
    match world.get_component::<LordGrant>(org).filter(|g| g.issue == issue) {
        Some(g) if g.active(tick) && g.hollow => {
            f.insert("arc:diluted");
            f.insert("arc:promise_broken");
        }
        Some(g) if g.active(tick) && g.partial => { f.insert("arc:partial"); }
        Some(g) if g.active(tick) => { f.insert("arc:won"); }
        Some(_) => {
            f.insert("arc:expired");
            outcome_from_response(&mut f, record.and_then(|r| r.last_response));
        }
        None => outcome_from_response(&mut f, record.and_then(|r| r.last_response)),
    }
    f
}

fn outcome_from_response(f: &mut Facts, resp: Option<TargetResponse>) {
    match resp {
        Some(TargetResponse::Refuses) => f.insert("arc:lost"),
        Some(TargetResponse::Deflects) => f.insert("arc:delayed"),
        // carried up the chain, or part granted: the grant decides it
        Some(TargetResponse::Carries) | Some(TargetResponse::Concedes) | None => {}
    }
}

/// Walk the arc library after `ev`: record the event, read the facts
/// (consent banked at the last meeting counts for this action, then is
/// spent),
/// walk from the event's phase (preferring beats the campaign hasn't
/// heard lately), voice each line through the org - the leader drives,
/// members answer, the other side speaks for itself - and apply the
/// hooks. Seeded by campaign and day. None if nobody's free to speak.
pub fn deliberate_arc(world: &mut World, campaign: Entity, ev: ArcEvent, tick: u64) -> Option<Deliberation> {
    record_event(world, campaign, ev);
    let d = walk_arc(world, campaign, ev, tick);
    // consent banked at a meeting is spent by the action that follows it -
    // whether or not anyone was free to talk it over
    if let Some(r) = world.get_component_mut::<ArcRecord>(campaign) { r.consent = false; }
    d
}

/// The walk itself, with the event already on the record
fn walk_arc(world: &mut World, campaign: Entity, ev: ArcEvent, tick: u64) -> Option<Deliberation> {
    let (org, issue, dissenters) = world.get_component::<Campaign>(campaign)
        .map(|c| (c.org, c.issue.clone(), c.dissenters.clone()))?;
    let (leader, members) = world.get_component::<Organization>(org).map(|o| (o.leader, o.members.clone()))?;
    let free: Vec<Entity> = members.into_iter().filter(|&m| !unavailable(world, m)).collect();
    let driver = if !unavailable(world, leader) { leader } else { *free.first()? };
    let others: Vec<Entity> = free.iter().copied().filter(|&m| m != driver).collect();
    let seated = |r: Role| crate::game::campaign::seat_holder(world, campaign, r).filter(|&h| !unavailable(world, h));
    let (note_taker, chair) = (seated(Role::NoteTaker), seated(Role::Chair));
    let dissenter = dissenters.iter().copied().find(|&d| !unavailable(world, d));
    let mut rng = Rng::new(tick.wrapping_mul(0x9E37_79B9) ^ ((campaign as u64) << 24) ^ 0xA5C);
    let voice_of = |v: Voice, not: Option<Entity>, rng: &mut Rng| -> Option<Entity> {
        match v {
            Voice::Driver => Some(driver),
            Voice::Member => {
                let pool: Vec<Entity> = others.iter().copied().filter(|&m| Some(m) != not).collect();
                Some(pool.get(rng.below(pool.len())).copied().unwrap_or(driver))
            }
            Voice::Dissenter => dissenter.or(Some(driver)),
            Voice::NoteTaker => note_taker.or(Some(driver)),
            Voice::Chair => chair.or(Some(driver)),
            Voice::Room | Voice::Opponent => None,
        }
    };

    let recent = recent_beats(world, campaign);
    let mut d = Deliberation { said: vec![], applied: vec![], decided: false, consent: false };
    let mut w = Walker::new(ev.start());
    let mut heard = vec![];
    for _ in 0..ARC_MAX_BEATS {
        let facts = arc_facts(world, campaign, tick);
        let Some(s) = w.step_avoiding(ARC_BEATS, &facts, &recent, &mut rng) else { break };
        let caller = voice_of(s.beat.voice, None, &mut rng);
        d.said.push(Spoken { speaker: caller, voice: s.beat.voice, text: fill(s.call, &issue), beat: s.beat.id, phase: s.beat.phase });
        let answerer = voice_of(s.mv.voice, caller, &mut rng);
        d.said.push(Spoken { speaker: answerer, voice: s.mv.voice, text: fill(s.reply, &issue), beat: s.beat.id, phase: s.beat.phase });
        if apply_hook(world, campaign, s.mv.hook, answerer) { d.applied.push(s.mv.hook); }
        heard.push(s.beat.id);
    }
    remember(world, campaign, &heard);
    if d.said.is_empty() { return None; }
    Some(d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::campaign::{found_campaign, IssueVote};
    use crate::game::desires::InnerLife;

    fn world_with(n: usize) -> (World, Entity, Entity, Vec<Entity>) {
        let mut w = World::new();
        let ms: Vec<Entity> = (0..n).map(|_| {
            let m = w.create_entity();
            w.add_component(m, InnerLife { wants: vec![], grievances: vec![], satisfied: vec![] });
            m
        }).collect();
        let o = w.create_entity();
        let leader = w.create_entity();
        w.add_component(o, Organization { name: "The League".into(), members: ms.clone(), leader,
            influence_pool: 0.0, resources: 0.0, charter: "".into(), successors: vec![] });
        w.add_component(o, OrganizedPower::new());
        let vote = IssueVote { winner: "the taxes".into(), ballots: vec![] };
        let c = found_campaign(&mut w, o, &vote, Framing::CommonFoe, DecisionRule::Majority, 1);
        (w, o, c, ms)
    }

    #[test]
    fn test_events_pick_up_the_arc_where_they_belong() {
        assert_eq!(ArcEvent::March(Reaction::Ignored).start(), Phase::Test);
        assert_eq!(ArcEvent::March(Reaction::Crackdown).start(), Phase::Counter);
        assert_eq!(ArcEvent::Strike { called: false }.start(), Phase::Learn);
        assert_eq!(ArcEvent::Townhall(TargetResponse::Refuses).start(), Phase::Outcome);
    }

    #[test]
    fn test_arc_facts_read_the_campaigns_state() {
        let (mut w, o, c, ms) = world_with(4);
        let f = arc_facts(&w, c, 10);
        for x in ["rule:majority", "framing:foe", "arc:weak", "arc:first"] { assert!(f.has(x), "missing {}", x); }
        for x in ["arc:won", "arc:lost", "arc:crackdown", "arc:prisoners", "arc:split", "arc:allies"] {
            assert!(!f.has(x), "{} without cause", x);
        }

        // the other side moves
        w.add_component(o, GuardsPosted { knights: 2, since: 5 });
        w.add_component(o, AssemblyBan { cap: 2 });
        w.add_component(ms[0], crate::game::dungeon::Imprisoned { since: 5, charge: "riot".into() });
        let f = arc_facts(&w, c, 10);
        for x in ["arc:crackdown", "arc:countered", "arc:prisoners", "org:jailed"] { assert!(f.has(x), "missing {}", x); }

        // the campaign's own state
        w.get_component_mut::<Campaign>(c).unwrap().dissenters = vec![ms[1], ms[2]];
        for &m in &ms[1..3] { w.add_component(m, Demoralized { since: 9 }); }
        for _ in 0..2 {
            let v = w.create_entity();
            w.add_component(v, Sympathized { issues: vec![("the taxes".into(), 8)] });
        }
        w.get_component_mut::<OrganizedPower>(o).unwrap().capacity = 2.0;
        let f = arc_facts(&w, c, 10);
        for x in ["arc:split", "arc:fatigue", "arc:allies", "arc:strong", "org:strong"] { assert!(f.has(x), "missing {}", x); }
        assert!(!f.has("arc:weak"));
    }

    #[test]
    fn test_outcomes_come_from_grants_and_answers() {
        let (mut w, o, c, _) = world_with(3);
        record_event(&mut w, c, ArcEvent::Townhall(TargetResponse::Refuses));
        assert!(arc_facts(&w, c, 10).has("arc:lost"));
        record_event(&mut w, c, ArcEvent::Townhall(TargetResponse::Deflects));
        let f = arc_facts(&w, c, 10);
        assert!(f.has("arc:delayed") && !f.has("arc:lost"));
        // carried up and granted: a win while the grant lasts...
        record_event(&mut w, c, ArcEvent::Townhall(TargetResponse::Carries));
        w.add_component(o, LordGrant::full("the taxes", 0, 100));
        let f = arc_facts(&w, c, 50);
        assert!(f.has("arc:won") && !f.has("arc:promise_broken"));
        // ...and once it has run its term, a grant to renew - nobody broke it
        let f = arc_facts(&w, c, 150);
        assert!(f.has("arc:expired") && !f.has("arc:won") && !f.has("arc:promise_broken"));
        // a grant for some other issue isn't this campaign's win
        w.add_component(o, LordGrant::full("the mill", 0, 500));
        assert!(!arc_facts(&w, c, 50).has("arc:won"));
        // part granted; then the grant hollowed while still on paper
        w.add_component(o, LordGrant::partial("the taxes", 0, 100));
        let f = arc_facts(&w, c, 50);
        assert!(f.has("arc:partial") && !f.has("arc:won"));
        w.get_component_mut::<LordGrant>(o).unwrap().hollow = true;
        let f = arc_facts(&w, c, 50);
        assert!(f.has("arc:diluted") && !f.has("arc:partial") && !f.has("arc:won"));
        assert!(f.has("arc:promise_broken"), "hollowed in force is a broken promise");
        w.add_component(o, LordGrant::full("the mill", 0, 500));
        assert!(!arc_facts(&w, c, 50).has("arc:won"));
        // and the record counts actions: three so far, five makes a veteran
        let f = arc_facts(&w, c, 50);
        assert!(!f.has("arc:first") && !f.has("arc:veteran"));
        record_event(&mut w, c, ArcEvent::Strike { called: true });
        record_event(&mut w, c, ArcEvent::March(Reaction::Watched));
        assert!(arc_facts(&w, c, 50).has("arc:veteran"));
    }

    #[test]
    fn test_an_arc_is_voiced_by_the_org_and_replays_by_seed() {
        let run = |day: u64| {
            let (mut w, o, c, ms) = world_with(4);
            let d = deliberate_arc(&mut w, c, ArcEvent::March(Reaction::Watched), day).unwrap();
            let leader = w.get_component::<Organization>(o).unwrap().leader;
            for s in &d.said {
                if let Some(e) = s.speaker { assert!(e == leader || ms.contains(&e), "{:?} isn't the org's", s); }
                if s.voice == Voice::Opponent { assert!(s.speaker.is_none(), "the other side speaks for itself"); }
                assert!(!s.text.contains("{issue}"));
            }
            assert!(d.said.len() >= 2 && d.said.len() % 2 == 0);
            assert_eq!(d.said[0].phase, Phase::Counter, "a noticed march draws the other side's move");
            d.said.iter().map(|s| (s.speaker, s.text.clone())).collect::<Vec<_>>()
        };
        for day in 5..25 { assert_eq!(run(day), run(day)); }
    }

    #[test]
    fn test_consent_is_spent_by_the_next_action() {
        let (mut w, _, c, _) = world_with(3);
        record_consent(&mut w, c, true);
        assert!(arc_facts(&w, c, 10).has("arc:consent"));
        let d = deliberate_arc(&mut w, c, ArcEvent::March(Reaction::Watched), 10).unwrap();
        assert!(!d.said.is_empty());
        assert!(!arc_facts(&w, c, 11).has("arc:consent"), "consent carried past the action it was banked for");
        // spent even when nobody's free to talk it over
        let (mut w, o, c, ms) = world_with(1);
        record_consent(&mut w, c, true);
        let leader = w.get_component::<Organization>(o).unwrap().leader;
        for e in [leader, ms[0]] {
            w.add_component(e, crate::game::dungeon::Imprisoned { since: 5, charge: "riot".into() });
        }
        assert!(deliberate_arc(&mut w, c, ArcEvent::Strike { called: true }, 12).is_none());
        assert!(!arc_facts(&w, c, 13).has("arc:consent"));
    }

    #[test]
    fn test_only_the_free_count_as_split_or_allies() {
        let (mut w, _, c, ms) = world_with(4);
        w.get_component_mut::<Campaign>(c).unwrap().dissenters = vec![ms[1], ms[2]];
        assert!(arc_facts(&w, c, 10).has("arc:split"));
        w.add_component(ms[2], crate::game::dungeon::Imprisoned { since: 5, charge: "riot".into() });
        assert!(!arc_facts(&w, c, 10).has("arc:split"), "a held dissenter can't voice the split");
        let woken: Vec<Entity> = (0..2).map(|_| {
            let v = w.create_entity();
            w.add_component(v, Sympathized { issues: vec![("the taxes".into(), 8)] });
            v
        }).collect();
        assert!(arc_facts(&w, c, 10).has("arc:allies"));
        w.add_component(woken[0], crate::game::villager::Dead);
        assert!(!arc_facts(&w, c, 10).has("arc:allies"), "the dead aren't allies");
    }

    #[test]
    fn test_a_jailed_leader_hands_the_voice_to_the_members() {
        let (mut w, o, c, ms) = world_with(3);
        let leader = w.get_component::<Organization>(o).unwrap().leader;
        w.add_component(leader, crate::game::dungeon::Imprisoned { since: 5, charge: "riot".into() });
        let d = deliberate_arc(&mut w, c, ArcEvent::Strike { called: true }, 9).unwrap();
        assert!(d.said.iter().all(|s| s.speaker != Some(leader)), "the held don't speak");
        assert!(arc_facts(&w, c, 9).has("arc:leader_taken"));
        let _ = ms;
    }

    use crate::game::campaign_dialogue::{Beat, Next, EXCLUSIVE_FACTS, KNOWN_FACTS, MEETING_BEATS, MEETING_MAX_BEATS};
    use crate::engine::det::HashMap;

    /// What a long run of one library looked like
    struct Balance {
        walks: usize,
        dead_ends: usize,
        beats_played: usize,
        repeats: usize,
        reached: HashSet<&'static str>,
        dominated: usize,
        judged: usize,
    }

    /// Run `campaigns` seeded campaigns of `events` walks each through
    /// `beats`: each campaign carries a consistent set of facts that drifts
    /// a little between events, starts each walk from `start(rng)`, and
    /// remembers its last MEMORY_BEATS beats the way the runtime does
    fn long_run(beats: &'static [Beat], pool: &[&'static str], start: &dyn Fn(&mut Rng) -> Phase,
                max: usize, campaigns: u64, events: usize) -> Balance {
        let mut b = Balance { walks: 0, dead_ends: 0, beats_played: 0, repeats: 0,
            reached: HashSet::default(), dominated: 0, judged: 0 };
        let mut picks: HashMap<&'static str, HashMap<&'static str, usize>> = HashMap::default();
        for seed in 0..campaigns {
            let mut rng = Rng::new(seed.wrapping_mul(7919) + 1);
            let mut held: Vec<&'static str> = vec![];
            let mut memory: VecDeque<&'static str> = VecDeque::new();
            for _ in 0..events {
                // the state drifts: a fact comes or goes
                let x = pool[rng.below(pool.len())];
                if let Some(i) = held.iter().position(|h| *h == x) { held.remove(i); }
                else if !EXCLUSIVE_FACTS.iter().any(|g| g.contains(&x) && held.iter().any(|h| g.contains(h))) && held.len() < 5 {
                    held.push(x);
                }
                let mut f = Facts::default();
                for h in &held { f.insert(*h); }
                let recent: HashSet<&'static str> = memory.iter().copied().collect();
                let mut w = Walker::new(start(&mut rng));
                let mut n = 0;
                for _ in 0..max {
                    let Some(st) = w.step_avoiding(beats, &f, &recent, &mut rng) else { break };
                    n += 1;
                    b.beats_played += 1;
                    if recent.contains(st.beat.id) { b.repeats += 1; }
                    b.reached.insert(st.beat.id);
                    *picks.entry(st.beat.id).or_default().entry(st.mv.id).or_default() += 1;
                    memory.push_back(st.beat.id);
                    while memory.len() > MEMORY_BEATS { memory.pop_front(); }
                    if st.mv.next == Next::Close { break; }
                }
                b.walks += 1;
                if n == 0 { b.dead_ends += 1; }
            }
        }
        // a beat that's played enough to judge, with more than one move,
        // where one move takes nine draws in ten, has a dominant choice
        for (id, by_move) in &picks {
            let total: usize = by_move.values().sum();
            let beat = beats.iter().find(|bb| bb.id == *id).unwrap();
            if total < 60 || beat.moves.len() < 2 { continue; }
            b.judged += 1;
            if *by_move.values().max().unwrap() as f32 / total as f32 > 0.9 {
                b.dominated += 1;
                eprintln!("  dominated: {} {:?}", id, by_move);
            }
        }
        b
    }

    fn report(name: &str, b: &Balance, total: usize) {
        eprintln!("{}: {} walks, {} dead ends, {} beats played, {:.1}% repeats within memory, \
                   {}/{} beats reached, {}/{} judged beats dominated by one move",
            name, b.walks, b.dead_ends, b.beats_played, 100.0 * b.repeats as f32 / b.beats_played.max(1) as f32,
            b.reached.len(), total, b.dominated, b.judged);
    }

    #[test]
    fn test_long_run_balance() {
        // campaign arcs, started where real events start them
        let arc_pool: Vec<&'static str> = KNOWN_FACTS.iter().copied()
            .filter(|f| ["arc:", "org:", "framing:", "rule:", "seat:"].iter().any(|p| f.starts_with(p))).collect();
        let starts = [ArcEvent::March(Reaction::Ignored), ArcEvent::March(Reaction::Watched),
            ArcEvent::Strike { called: true }, ArcEvent::Strike { called: false },
            ArcEvent::Townhall(TargetResponse::Refuses), ArcEvent::Townhall(TargetResponse::Carries)];
        let arc = long_run(ARC_BEATS, &arc_pool, &|r: &mut Rng| starts[r.below(starts.len())].start(),
            ARC_MAX_BEATS, 400, 30);
        report("arcs", &arc, ARC_BEATS.len());
        // house meetings, always from the agenda
        let meet_pool: Vec<&'static str> = KNOWN_FACTS.iter().copied().filter(|f| !f.starts_with("arc:")).collect();
        let meet = long_run(MEETING_BEATS, &meet_pool, &|_: &mut Rng| Phase::Agenda, MEETING_MAX_BEATS, 400, 30);
        report("meetings", &meet, MEETING_BEATS.len());

        for (name, b, total) in [("arcs", &arc, ARC_BEATS.len()), ("meetings", &meet, MEETING_BEATS.len())] {
            assert_eq!(b.dead_ends, 0, "{}: a walk with nothing to say", name);
            assert!((b.repeats as f32) < 0.05 * b.beats_played as f32, "{}: too many beats replayed within memory", name);
            assert!(b.reached.len() as f32 >= 0.85 * total as f32, "{}: only {} of {} beats reached", name, b.reached.len(), total);
            assert!((b.dominated as f32) <= 0.02 * b.judged as f32, "{}: {} of {} beats have a dominant move", name, b.dominated, b.judged);
        }
    }

    #[test]
    fn test_no_move_dominates_by_weight() {
        // exact, not sampled: with every bias a move could earn applied to
        // it alone, no move takes nine draws in ten from its alternatives
        let mut dominated = vec![];
        for b in ARC_BEATS.iter().chain(MEETING_BEATS) {
            if b.moves.len() < 2 { continue; }
            for (i, m) in b.moves.iter().enumerate() {
                let boosted = m.bias.iter().fold(m.weight, |w, (_, k)| if *k > 1.0 { w * k } else { w });
                let others: f32 = b.moves.iter().enumerate().filter(|(j, _)| *j != i)
                    .map(|(_, o)| o.bias.iter().fold(o.weight, |w, (_, k)| if *k < 1.0 { w * k } else { w })).sum();
                if boosted / (boosted + others) > 0.9 { dominated.push(format!("{}.{}", b.id, m.id)); }
            }
        }
        assert!(dominated.is_empty(), "moves that can crowd out every alternative: {:?}", dominated);
    }

    #[test]
    fn test_memory_keeps_a_campaign_from_replaying_itself() {
        // the same campaign in the same state, march after march: fresh
        // beats come before repeats
        let (mut w, _, c, _) = world_with(4);
        let mut seen: Vec<&str> = vec![];
        for day in 0..10 {
            let d = deliberate_arc(&mut w, c, ArcEvent::March(Reaction::Watched), 10 + day).unwrap();
            let mut beats: Vec<&str> = d.said.iter().map(|s| s.beat).collect();
            beats.dedup();
            seen.extend(beats);
        }
        let unique: HashSet<&str> = seen.iter().copied().collect();
        assert_eq!(unique.len(), seen.len(), "a beat replayed within {} beats", seen.len());
        assert!(w.get_component::<DialogueMemory>(c).unwrap().recent.len() <= MEMORY_BEATS);
    }
}
