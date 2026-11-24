//! The one-on-one argument (#1234): a private talk where the organizer
//! argues an issue with someone they know, and - given a few days to
//! turn it over - moves where they stand. Nothing here is a second
//! stance meter: a won argument lands as a `Sympathized` mark on the
//! issue, the same mark a moral shock leaves, so every reader of that
//! mark (recruiting, turnout, the campaign's woken count) sees it.
use crate::engine::ecs::{Entity, World};
use crate::game::desires::{self, IssueTag, Organizer};
use crate::game::organizing::{stage_of, RelStage};
use crate::game::reactions::Sympathized;
use crate::game::traits::{Personality, Trait};

/// Days they turn an argument over before it takes
pub const THINK_DAYS: u64 = 3;
/// The pull a position needs before it takes (either direction)
pub const TAKES_AT: f32 = 1.0;
/// Below this, a half-made case is forgotten
const FORGOTTEN: f32 = 0.1;
/// Most issues on offer to argue at once (the choice list holds three,
/// the last is "let it drop")
pub const ARGUABLE_MAX: usize = 2;

/// Which side of the issue the organizer argues
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stance { For, Against }

/// How the organizer answers their counter (Dwarf Fortress's moves)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgueMove { Press, Appeal, HearOut }

pub const MOVES: [ArgueMove; 3] = [ArgueMove::Press, ArgueMove::Appeal, ArgueMove::HearOut];

/// How the exchange came out
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Your case carried
    Won,
    /// Neither moved the other
    Stalemate,
    /// Their counter carried - if anything they lean the other way now
    Lost,
    /// Pushed too hard: they bristle, and nothing moves
    Pressed,
}

/// Arguments still being turned over: (issue, signed pull - positive is
/// for it - and the day it takes)
#[derive(Debug, Clone, Default)]
pub struct PendingPersuasion {
    pub items: Vec<(IssueTag, f32, u64)>,
}

/// Whether the bond's deep enough to argue with them: you know them
pub fn can_argue(stage: RelStage) -> bool { stage >= RelStage::Known }

/// Lines that are themselves an argument (#1348): the talk offers no
/// "argue a point" ask any more - the words carry it. Picking one of
/// these speaks it, then opens the argument the ask used to open.
/// The accusation picks of the event trees, the flat contradiction the
/// PushBack replies lean on, calling their word a liar's, and taking
/// a side in the Noah row all count.
pub const ARGUMENTATIVE_LINES: &[&str] = &[
    crate::game::dialogue_trees::FT_ACCUSE,
    crate::game::dialogue_trees::FT_ACCUSE_RUN,
    crate::game::dialogue_trees::NOAH_SIDE_THEM,
    "Nay, I see it differently, mark me.",
    "That is not the way the elders told it.",
    "I have heard that lie three times this week.",
    "Say that again, if {2:subj} {2:dare}.",
    "{2:subj} {2:have} a treacherous tongue in {2:poss} skull.",
    "The whole parish knows the measure of {2:poss} word.",
];

/// Whether a reply's own words are an argument's opening
pub fn is_argumentative(line: &str) -> bool { ARGUMENTATIVE_LINES.contains(&line) }

/// The live issues worth arguing with `person`: the campaign's, the
/// ones the organizer has found, then what they've told you of their
/// own and what's woken them - deduped, at most `ARGUABLE_MAX`
pub fn arguable_issues(world: &World, org: Entity, organizer: Entity, person: Entity) -> Vec<IssueTag> {
    let mut out: Vec<IssueTag> = vec![];
    let add = |t: IssueTag, out: &mut Vec<IssueTag>| { if !out.contains(&t) { out.push(t); } };
    if let Some(c) = crate::game::campaign::campaign_of(world, org)
        .and_then(|c| world.get_component::<crate::game::campaign::Campaign>(c)) {
        add(c.issue.clone(), &mut out);
    }
    if let Some(o) = world.get_component::<Organizer>(organizer) {
        for t in &o.issues { add(t.clone(), &mut out); }
    }
    for r in desires::known(world, organizer, person) {
        add(desires::issue_tag(world, person, &r), &mut out);
    }
    if let Some(s) = world.get_component::<Sympathized>(person) {
        for (t, _) in &s.issues { add(t.clone(), &mut out); }
    }
    out.truncate(ARGUABLE_MAX);
    out
}

/// Whether the organizer's cause holds `issue`: the campaign's issue or
/// one they've taken up
fn org_holds(world: &World, org: Entity, organizer: Entity, issue: &IssueTag) -> bool {
    let campaign = crate::game::campaign::campaign_of(world, org)
        .and_then(|c| world.get_component::<crate::game::campaign::Campaign>(c))
        .map(|c| desires::tag_stands(&c.issue, issue) || desires::tag_stands(issue, &c.issue))
        .unwrap_or(false);
    campaign || world.get_component::<Organizer>(organizer)
        .map(|o| o.issues.iter().any(|t| desires::tag_stands(t, issue) || desires::tag_stands(issue, t)))
        .unwrap_or(false)
}

/// Arguing a position the cause doesn't hold - for an issue it hasn't
/// taken up, against one it has. Allowed, but it leaves a mark.
pub fn insincere(world: &World, org: Entity, organizer: Entity, issue: &IssueTag, stance: Stance) -> bool {
    let held = org_holds(world, org, organizer, issue);
    match stance { Stance::For => !held, Stance::Against => held }
}

fn traits_of(world: &World, who: Entity) -> Vec<Trait> {
    world.get_component::<Personality>(who).map(|p| p.traits.clone()).unwrap_or_default()
}

/// What the organizer says, opening the case
pub fn case_line(stance: Stance, issue: &IssueTag) -> String {
    let name = desires::issue_name(issue.clone());
    match stance {
        Stance::For => format!("You make the case for taking up {}.", name),
        Stance::Against => format!("You make the case against stirring up {}.", name),
    }
}

/// Their answer back, from their temper: the stingy want it costed, the
/// pious want it right, the hothead wants it fought, the proud won't be
/// told
pub fn counter_line(traits: &[Trait], stance: Stance, issue: &IssueTag) -> String {
    let name = desires::issue_name(issue.clone());
    let what = match stance {
        Stance::For => format!("a fuss over {}", name),
        Stance::Against => format!("letting {} lie", name),
    };
    for t in traits {
        let line = match t {
            Trait::Stingy => format!("And who pays for {}? Not me, I'll wager.", what),
            Trait::Pious => format!("Is {} right, though - or only useful to {{2:obj}}?", what),
            Trait::Hotheaded => format!("Talk's cheap. Who'd stand behind {} when it comes to it?", what),
            Trait::Proud => format!("I'll make my own mind up on {}, thank {{2:obj}}.", name),
            Trait::Lazy => format!("{} sounds like work. Who'd carry it?", capitalize(&what)),
            _ => continue,
        };
        return line;
    }
    format!("And what would {} change, for the likes of me?", what)
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}

/// The organizer's answer to their counter, as the choice reads
pub fn move_label(m: ArgueMove) -> &'static str {
    match m {
        ArgueMove::Press => "Press the point",
        ArgueMove::Appeal => "Appeal to their better nature",
        ArgueMove::HearOut => "Hear them out",
    }
}

/// How well a move suits them, -1..1: the ambitious respect a push the
/// hothead and the proud won't take; the kind and the pious hear an
/// appeal the stingy scoff at; the stingy, the lazy and the proud like
/// being let to have their say
pub fn move_fit(traits: &[Trait], m: ArgueMove) -> f32 {
    let mut fit = 0.0f32;
    for t in traits {
        fit += match (m, t) {
            (ArgueMove::Press, Trait::Ambitious) => 0.5,
            (ArgueMove::Press, Trait::Hotheaded | Trait::Proud) => -0.5,
            (ArgueMove::Appeal, Trait::Kind | Trait::Pious) => 0.5,
            (ArgueMove::Appeal, Trait::Stingy) => -0.5,
            (ArgueMove::HearOut, Trait::Stingy | Trait::Lazy | Trait::Proud) => 0.5,
            _ => 0.0,
        };
    }
    fit.clamp(-1.0, 1.0)
}

/// How the exchange resolves: the bond's depth sets the odds, the move's
/// fit and the organizer's practice shift them. Pushing a hothead or a
/// proud soul and missing is a pressed argument, not a lost one.
/// `roll` is 0..1.
pub fn resolve(world: &World, organizer: Entity, person: Entity, m: ArgueMove, roll: f32) -> Outcome {
    let traits = traits_of(world, person);
    let base = match stage_of(world, organizer, person) {
        RelStage::Stranger | RelStage::Acquaintance => 0.0,
        RelStage::Known => 0.45,
        RelStage::Follower => 0.6,
        RelStage::CoLeader => 0.7,
    };
    let skill = world.get_component::<crate::game::org_skills::Organizing>(organizer)
        .map(|o| o.one_on_one).unwrap_or(0.0);
    let chance = (base + move_fit(&traits, m) * 0.25 + skill * 0.15).clamp(0.0, 0.95);
    if roll < chance { return Outcome::Won; }
    if m == ArgueMove::Press && traits.iter().any(|t| matches!(t, Trait::Hotheaded | Trait::Proud)) {
        return Outcome::Pressed;
    }
    if roll < chance + 0.2 { Outcome::Stalemate } else { Outcome::Lost }
}

/// One argument's pull on them: deeper bonds move further, the proud,
/// the stingy and the wary barely shift, the undecided (no stake, no mark) move
/// most, and a move that suits them carries more
pub fn magnitude(world: &World, organizer: Entity, person: Entity, issue: &IssueTag, m: ArgueMove, tick: u64) -> f32 {
    let traits = traits_of(world, person);
    let stage = match stage_of(world, organizer, person) {
        RelStage::Stranger | RelStage::Acquaintance => 0.0,
        RelStage::Known => 0.4,
        RelStage::Follower => 0.7,
        RelStage::CoLeader => 0.9,
    };
    let resist: f32 = traits.iter().map(|t| match t {
        Trait::Proud => 0.5,
        Trait::Stingy => 0.7,
        Trait::Wary => 0.7,
        Trait::Stolid => 0.8,
        _ => 1.0,
    }).product();
    let staked = crate::game::campaign::stake_for(world, person, issue, tick).is_some()
        || world.get_component::<Sympathized>(person)
            .map(|s| s.issues.iter().any(|(t, _)| t == issue)).unwrap_or(false);
    let undecided = if staked { 1.0 } else { 1.3 };
    stage * resist * undecided * (1.0 + 0.5 * move_fit(&traits, m))
}

/// Set an argument's outcome turning over in their head: a won case
/// pulls toward the side argued, a lost one away from it. It takes
/// `THINK_DAYS` after the latest word on it.
pub fn push_argument(world: &mut World, person: Entity, issue: &IssueTag, stance: Stance, outcome: Outcome, mag: f32, tick: u64) {
    let sign = match stance { Stance::For => 1.0, Stance::Against => -1.0 };
    let pull = match outcome {
        Outcome::Won => sign * mag,
        Outcome::Lost => -sign * mag * 0.5,
        Outcome::Stalemate | Outcome::Pressed => return,
    };
    let mut p = world.get_component::<PendingPersuasion>(person).cloned().unwrap_or_default();
    match p.items.iter_mut().find(|(t, _, _)| t == issue) {
        Some((_, s, at)) => { *s += pull; *at = tick + THINK_DAYS; }
        None => p.items.push((issue.clone(), pull, tick + THINK_DAYS)),
    }
    world.add_component(person, p);
}

/// The thinking's done: a pull that's grown enough takes - for lands or
/// refreshes their `Sympathized` mark on the issue, against lifts it;
/// one that hasn't keeps half for the next look, and a faint one is
/// forgotten. Returns the issues that took, with their direction.
pub fn mature(world: &mut World, person: Entity, tick: u64) -> Vec<(IssueTag, Stance)> {
    let Some(mut p) = world.get_component::<PendingPersuasion>(person).cloned() else { return vec![] };
    let mut took = vec![];
    let mut keep = vec![];
    for (issue, s, at) in p.items.drain(..) {
        if tick < at { keep.push((issue, s, at)); continue; }
        if s >= TAKES_AT {
            if world.get_component::<Sympathized>(person).is_none() {
                world.add_component(person, Sympathized::default());
            }
            if let Some(m) = world.get_component_mut::<Sympathized>(person) {
                m.issues.retain(|(t, _)| *t != issue);
                m.issues.push((issue.clone(), tick));
            }
            took.push((issue, Stance::For));
        } else if s <= -TAKES_AT {
            let empty = world.get_component_mut::<Sympathized>(person).map(|m| {
                m.issues.retain(|(t, _)| *t != issue);
                m.issues.is_empty()
            }).unwrap_or(false);
            if empty { world.remove_component::<Sympathized>(person); }
            took.push((issue, Stance::Against));
        } else if (s * 0.5).abs() >= FORGOTTEN {
            keep.push((issue, s * 0.5, tick + THINK_DAYS));
        }
    }
    if keep.is_empty() { world.remove_component::<PendingPersuasion>(person); }
    else { world.add_component(person, PendingPersuasion { items: keep }); }
    took
}
