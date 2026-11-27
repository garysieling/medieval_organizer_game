//! Secret inner lives: the wants people carry (Maslow-flavoured, sampled
//! per vocation and class) and the problems that stand in their way.
//! Neither shows in the UI until revealed: a 1-1 has a small chance, and
//! at a house meeting each parallel complaint already voiced makes the
//! next speaker likelier to share their own. The organizer keeps what he
//! hears in a ledger that expires after a year; three or more people with
//! versions of the same trouble, heard inside the year, surface an issue.

use crate::engine::ecs::{Entity, World};
use crate::game::vocation::{self, Craft, Kid, SocialClass, TownRole};

/// Days in a year (4 seasons x 90 days); heard complaints expire after one
pub const YEAR_DAYS: u64 = 360;

/// Base chance a 1-1 or a meeting turn reveals one of someone's secrets
pub const REVEAL_BASE: f32 = 0.15;
/// Extra chance per parallel complaint already voiced at the meeting
pub const REVEAL_PARALLEL: f32 = 0.10;

/// An issue kind: complaints in the same family, however people put them
/// What an issue's about. Trade-flavored problems carry the trade too -
/// "no master (mason)" only echoes among masons; the commons (tax,
/// broke, the bad bread) carry no tag and echo for everyone
pub type IssueTag = String;

/// What a person wants, inspired by Maslow's hierarchy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Want {
    GoodFood,      // buys bread
    Finery,        // buys garments
    Faith,         // buys a service at the church
    Shelter,       // have a home (a problem when unhoused)
    Security,      // buys the watch's protection
    Mastery,       // rise in the craft
    Music,         // hear music played
    Instrument,    // own a lute
    BetterTools,   // own tools (works faster)
    DifferentTrade,// the assigned vocation bores them
    Love,          // a close bond
    Joy,           // play - mostly the children's want
    Drink,         // a cup at the alehouse - company in a bowl
}

/// What's in someone's way. Most are computed from their situation each
/// day; a few (a taxing) stick on them until it fades
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Problem {
    NoHome,
    NoWorkplace,
    /// A journeyman or apprentice with no master to learn from
    NoMaster,
    /// A day's input they can't lay hands on
    NoInput(&'static str),
    /// An empty shop near home that sours the street
    DerelictNearby,
    /// Working under an overseer who takes a cut
    OverseerCut,
    /// Paid the tax collector within the year
    Taxed,
    /// Under three gold to their name
    Broke,
    /// Carries a smell: the flock's, the cauldron's smoke - honest work
    /// hangs on them
    Smelly,
    /// Wolves got into the fold - stock lost in the night
    Loss,
    /// The village says their bread brought the fire
    Blamed,
    /// Ate spoiled stock - a sour gut for a poor soul
    SourGut,
    /// A child's hands in the fields when they should be at play -
    /// the village doesn't like to see it
    ChildLabor,
    /// A stranger's fist in the street - the season hands' nights
    /// end in the mud and the village remembers
    Rowdy,
    /// The lord took his fifth of what the dead one left
    DeathDuty,
    /// Someone they loved is gone - bereavement, not the trauma of
    /// watching a hanging (that's Grief)
    Bereaved,
    /// The wood's thin this side of the hill - gathers outpace regrowth
    ThinWood,
    /// The worked pit drowned - no stone to be had
    DryPit,
    /// The reach below town stinks - the leavings settle in it
    StinkingReach,
    /// The reckoning didn't add up - shorted in a foreign count
    ShortChanged,
    /// The circus's pull: the road's calling and the days here are gray
    Tempted,
    /// The bailiff's writ: a levy, a fee, a summons from the castle
    WritBorne,
    /// The winter runs long and the stores run thin
    LongWinter,
    /// No fuel for the hearth - the house eats cold all winter
    ColdHearth,
    /// The hermit's want: the dark keeps them
    TheDark,
    /// What they watched and can't unsee - the hanging's weight
    Grief,
    /// The good stock walked off in the night - a cellar worth robbing
    /// is a cellar worth watching
    Robbed,
    /// The pot's bare and the belly knows it - days of empty tables
    Hungry,
    /// The fevers ride the village - days shut in with the sickness
    Sick,
    /// The parish preaches one thing and lives another - a plainer word
    /// is heard on the green
    Doctrine,
}

/// One secret a person can share: a want or a problem
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Reveal {
    Want(Want),
    Problem(Problem),
}

/// A person's hidden inner life: the wants they carry, the sticky
/// grievances picked up along the way, and which wants are currently met
/// (with the day each stops counting as met)
#[derive(Debug)]
pub struct InnerLife {
    pub wants: Vec<Want>,
    /// (problem, the day it stops counting) - e.g. a taxing, fresh for a year
    pub grievances: Vec<(Problem, u64)>,
    /// (want, the day its satisfaction wears off)
    pub satisfied: Vec<(Want, u64)>,
}

/// The deep stat the pedagogy works on: the domination a villager's
/// own spirit's taken in - it lowers their own initiative, raises the
/// fear of freedom, and risks the oppressor-within flip; the agency
/// the consciousness-raising grows; and the experience the action
/// leaves, which only reflection turns into skill
#[derive(Debug, Default, Clone, Copy)]
pub struct Domination {
    /// How far the domination's taken in - resists joining, fears
    /// freedom, risks the flip
    pub internalized_domination: f32,
    /// The agency the consciousness-raising grows - the will to act
    pub critical_agency: f32,
    /// The experience the action leaves - only reflection turns it to
    /// skill
    pub experience: f32,
}

/// Where a ledger fact came from (E7-2, #1427): who put it in the
/// seat's ear, or whether the seat saw it themselves. Every entry
/// carries one - a fact with no source is a fact the ledger can't
/// trust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Source {
    /// They said it to this seat themselves
    Told { speaker: Entity },
    /// A third party carried it - hearsay, and hearsay can be wrong
    Gossip { speaker: Entity },
    /// The seat saw it happen
    Witnessed,
}

/// Certainty at the hearing by source: a telling straight from the
/// horse's mouth sits highest, a seen thing a little under (eyes
/// misread), a carried word lowest of all
fn source_confidence(s: &Source) -> f32 {
    match s {
        Source::Told { .. } => 0.9,
        Source::Witnessed => 0.7,
        Source::Gossip { .. } => 0.5,
    }
}

/// The fog-of-war clock a memory fades on (E7-2): each unrefreshed day
/// shaves a little off what the seat was once sure of
pub const LEDGER_FADE: f32 = 0.99;

/// One line in the seat's ledger (E7-2, #1427): a fact held about a
/// villager, who it came from, the day it landed, and how sure it still
/// is. `reveal` is the typed read when the fact's a want or a problem;
/// `fact` is its canonical id (`want:good_food`, `problem:no_input:*`,
/// or a fact-table key like `ghost:levy`) - the space gossip beliefs
/// and witnessed deeds write into
#[derive(Debug, Clone)]
pub struct LedgerEntry {
    pub fact: String,
    pub reveal: Option<Reveal>,
    pub source: Source,
    /// The day it landed - staleness derives off this
    pub day: u64,
    /// Certainty at the hearing; `confidence_of` fades it with age
    pub confidence: f32,
    /// A later entry said the opposite - both stay, both flag
    pub contradicted: bool,
}

/// The bounds a confidence travels under (E5-5, #1443): what the
/// owner asked of the seat when they told. A closed set - the
/// design's two shapes, nothing freer
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Rule {
    /// To no one: the whole world's out of bounds
    TellNoOne,
    /// Anywhere but there: "not the priest", "not the reeve" - the
    /// deep tells name the power they fear
    NotTo(Entity),
}

/// A disclosure held as an obligation (E5-5, #1443): what's told in
/// trust makes the seat a co-owner of it, bound by its rule until
/// it's broken. The keeping has a wage: a confidence stood long
/// enough proves the seat's integrity to the owner
#[derive(Debug, Clone)]
pub struct Confidence {
    /// The ledger fact it binds
    pub fact: String,
    /// Whose telling it was - the drop lands on their bond
    pub owner: Entity,
    pub rule: Rule,
    /// Held so far; a leak turns it false
    pub kept: bool,
    /// The day it was taken on
    pub day: u64,
    /// The keeping already paid its integrity out once
    pub honored: bool,
}

/// Days a kept confidence must stand before the holding itself is
/// proof (E5-5)
pub const CONFIDENCE_MARK_DAYS: u64 = 20;

/// How many turned-aside asks draw the shape (E5-2, #1434): the first
/// refusals read as manners; three in one area and the seat can see the
/// outline - "something about the lord"
pub const SILHOUETTE_AT: u32 = 3;

/// What they dodge on (E5-2 counts these up later; the record's kept
/// now): the shape a villager shows a seat that asks where they won't
/// go - a form without a name
#[derive(Debug, Clone)]
pub struct Silhouette {
    pub who: Entity,
    /// The ground they keep stepping around
    pub area: String,
    /// Times they've turned a question aside
    pub evasions: u32,
    /// The day they last did
    pub last_day: u64,
}

/// What an organizer has learned about the village, piece by piece:
/// the seat's own ledger of who told them what (E7-1, #1404), each
/// fact sourced and dated (E7-2, #1427). It lives on the seat's player
/// entity - one seat's hearing is invisible to another
#[derive(Debug, Default)]
pub struct Known {
    /// subject -> the seat's entries about them, oldest first
    pub entries: crate::engine::det::HashMap<Entity, Vec<LedgerEntry>>,
    /// The shapes that show without ever being told
    pub silhouettes: Vec<Silhouette>,
    /// The tellings the seat's bound to keep (E5-5)
    pub confidences: Vec<Confidence>,
}

/// The fact-table id a Reveal writes under in the ledger (E7-2):
/// `want:*` / `problem:*`, with NoInput carrying its input
pub fn reveal_key(r: &Reveal) -> String {
    fn snake(s: &str) -> String {
        s.chars().enumerate().flat_map(|(i, c)| {
            if c.is_uppercase() && i > 0 { vec!['_', c.to_ascii_lowercase()] }
            else { vec![c.to_ascii_lowercase()] }
        }).collect()
    }
    match r {
        Reveal::Want(w) => format!("want:{}", snake(&format!("{w:?}"))),
        Reveal::Problem(Problem::NoInput(input)) => format!("problem:no_input:{input}"),
        Reveal::Problem(p) => format!("problem:{}", snake(&format!("{p:?}"))),
    }
}

/// Write `fact` into `knower`'s ledger about `e`: sourced and dated
/// (E7-2). Dedups on the fact id - what the seat already knows says
/// nothing new. A claim and its negation (`x` against a held `!x`, or
/// the reverse) flag both sides contradicted rather than erasing
/// either
pub fn record(world: &mut World, knower: Entity, e: Entity, fact: String,
              reveal: Option<Reveal>, source: Source, day: u64) {
    if world.get_component::<Known>(knower).is_none() {
        world.add_component(knower, Known::default());
    }
    let k = world.get_component_mut::<Known>(knower).unwrap();
    let heard = k.entries.entry(e).or_default();
    if heard.iter().any(|en| en.fact == fact) { return; }
    let neg = fact.strip_prefix('!').map(str::to_string);
    let anti = format!("!{fact}");
    let contradicted = heard.iter_mut().fold(false, |hit, en| {
        let clash = Some(&en.fact) == neg.as_ref() || en.fact == anti;
        if clash { en.contradicted = true; }
        hit || clash
    });
    heard.push(LedgerEntry {
        fact: fact.clone(), reveal, source, day,
        confidence: source_confidence(&source),
        contradicted,
    });
    // a telling from their own mouth makes the seat a co-owner
    // (E5-5): what's confided is now the seat's to keep - but only a
    // genuine Reveal (a want, a problem) is a confidence; a plain
    // person:* fact (#1712's trade, kin, trait, hook) is firsthand
    // knowledge, not a secret entrusted to keep (#1721)
    if let Source::Told { speaker } = source {
        if speaker == e && reveal.is_some() {
            k.confidences.push(Confidence {
                fact, owner: e, rule: Rule::TellNoOne,
                kept: true, day, honored: false,
            });
        }
    }
}

/// An errand's answer settles what the seat heard (E5-7, #1438): a
/// fact held as word-of-mouth becomes the thing the seat saw proven -
/// witnessed source, the confidence of a seen thing, the day refreshed,
/// the contradiction cleared. Absent, the answer writes the fact fresh
/// as witnessed. Returns whether the ledger holds it now
pub fn confirm(world: &mut World, knower: Entity, subject: Entity, fact: &str, day: u64) -> bool {
    let held = world.get_component::<Known>(knower)
        .and_then(|k| k.entries.get(&subject))
        .map(|v| v.iter().any(|en| en.fact == fact)).unwrap_or(false);
    if !held {
        record(world, knower, subject, fact.to_string(), None, Source::Witnessed, day);
        return true;
    }
    if let Some(k) = world.get_component_mut::<Known>(knower) {
        if let Some(v) = k.entries.get_mut(&subject) {
            for en in v.iter_mut() {
                if en.fact == fact {
                    en.source = Source::Witnessed;
                    en.confidence = source_confidence(&Source::Witnessed).max(0.95);
                    en.day = day;
                    en.contradicted = false;
                }
            }
        }
    }
    true
}

/// The seat's ledger on `e`: every fact held, sourced and dated
pub fn entries(world: &World, knower: Entity, e: Entity) -> Vec<LedgerEntry> {
    world.get_component::<Known>(knower)
        .and_then(|k| k.entries.get(&e).cloned()).unwrap_or_default()
}

/// How sure the seat still is of an entry: the certainty at the
/// hearing, faded a little each day it sits unrefreshed
pub fn confidence_of(entry: &LedgerEntry, tick: u64) -> f32 {
    entry.confidence * LEDGER_FADE.powi(tick.saturating_sub(entry.day) as i32)
}

/// The newest day the seat heard anything about `e` - staleness reads
/// off this and the TalkMemory's last-sit-down day
pub fn last_heard(world: &World, knower: Entity, e: Entity) -> Option<u64> {
    world.get_component::<Known>(knower)
        .and_then(|k| k.entries.get(&e))
        .and_then(|v| v.iter().map(|en| en.day).max())
}

/// The seat's record of `e`: what `knower` has already heard them tell
pub fn known(world: &World, knower: Entity, e: Entity) -> Vec<Reveal> {
    entries(world, knower, e).into_iter().filter_map(|en| en.reveal).collect()
}

/// A `person:*` ledger fact already held (#1712): the learned-facts
/// ladder reads its own rungs off the ledger so nothing lands twice
fn person_known(world: &World, knower: Entity, e: Entity, fact: &str) -> bool {
    world.get_component::<Known>(knower)
        .and_then(|k| k.entries.get(&e))
        .map(|v| v.iter().any(|en| en.fact == fact)).unwrap_or(false)
}

/// The kin they'd name first: a Family bond, else who shares their roof
fn first_kin(world: &World, v: Entity) -> Option<Entity> {
    use crate::game::relationships::{bonds_of, RelKind, Relationship};
    // bonds_of hands back bond entities, not the people on them - read
    // each bond's own Relationship to find the Family one and its far
    // end, rather than asking find_relationship about the bond itself
    bonds_of(world, v).into_iter()
        .filter_map(|bond| world.get_component::<Relationship>(bond))
        .find(|r| r.kind == RelKind::Family)
        .map(|r| if r.from == v { r.other } else { r.from })
        .or_else(|| crate::game::household::home_of(world, v)
            .and_then(|h| world.get_component::<crate::game::household::Building>(h))
            .and_then(|b| b.occupants.iter().copied().find(|&o| o != v)))
}

/// What a talk teaches (#1712): each conversation's close reads one
/// more thing about them onto the seat's ledger - the trade on their
/// hands, who they keep house with, then the bend of their nature, and
/// the shape of what they carry only when the bond's deep enough for
/// it to show. One fact a talk: the shallowest unlearned rung their
/// stage affords. Returns the fact id that landed, if any
pub fn learn_in_talk(world: &mut World, knower: Entity, v: Entity, tick: u64) -> Option<String> {
    use crate::game::organizing::{stage_of, RelStage};
    let stage = stage_of(world, knower, v);
    let learn = |world: &mut World, fact: String| {
        record(world, knower, v, fact.clone(), None, Source::Told { speaker: v }, tick);
        Some(fact)
    };
    // the trade on their hands reads first
    if let Some(c) = world.get_component::<crate::game::vocation::Craft>(v) {
        let fact = format!("person:craft:{}", crate::game::vocation::vocation_name(c.vocation));
        if !person_known(world, knower, v, &fact) { return learn(world, fact); }
    }
    // who they keep house with
    if let Some(kin) = first_kin(world, v) {
        let fact = format!("person:kin:{kin}");
        if !person_known(world, knower, v, &fact) { return learn(world, fact); }
    }
    // the bend of their nature takes knowing them a little - a pair of
    // signature reads, the second a talk later
    if stage >= RelStage::Acquaintance {
        if let Some(p) = world.get_component::<crate::game::traits::Personality>(v) {
            for &t in p.traits.iter().take(2) {
                let fact = format!("person:trait:{t:?}");
                if !person_known(world, knower, v, &fact) { return learn(world, fact); }
            }
        }
    }
    // and the shape of what they carry only shows to the ones they know
    if stage >= RelStage::Known {
        if let Some(id) = crate::game::hooks::open(world, v) {
            let fact = format!("person:hook:{id}");
            if !person_known(world, knower, v, &fact) { return learn(world, fact); }
        }
    }
    None
}

/// Mark `r` known to `knower` about `e`, as if `e` said it themselves,
/// without `hear`'s ledger and issue bookkeeping - tests and seeded
/// knowledge take this road
pub fn note(world: &mut World, knower: Entity, e: Entity, r: Reveal) {
    record(world, knower, e, reveal_key(&r), Some(r), Source::Told { speaker: e }, 0);
}

/// A turned-aside ask lands on the seat's ledger (E5-2, #1434): each
/// evasion counts against the area that was closed - a real one, named
/// by the topic that asked past it. Returns the count in that area; at
/// `SILHOUETTE_AT` the shape's visible
pub fn note_evasion(world: &mut World, knower: Entity, e: Entity, area: &str, day: u64) -> u32 {
    if world.get_component::<Known>(knower).is_none() {
        world.add_component(knower, Known::default());
    }
    let k = world.get_component_mut::<Known>(knower).unwrap();
    match k.silhouettes.iter_mut().find(|s| s.who == e && s.area == area) {
        Some(s) => { s.evasions += 1; s.last_day = day; s.evasions }
        None => {
            k.silhouettes.push(Silhouette {
                who: e, area: area.into(), evasions: 1, last_day: day });
            1
        }
    }
}

/// The count of their turn-asides in `area` the seat's marked (E5-2)
pub fn evasions_of(world: &World, knower: Entity, e: Entity, area: &str) -> u32 {
    world.get_component::<Known>(knower)
        .map(|k| k.silhouettes.iter()
            .filter(|s| s.who == e && s.area == area).map(|s| s.evasions).sum())
        .unwrap_or(0)
}

/// The confidences a seat holds on `e` (E5-5)
pub fn confidences(world: &World, knower: Entity, e: Entity) -> Vec<Confidence> {
    world.get_component::<Known>(knower)
        .map(|k| k.confidences.iter().filter(|c| c.owner == e).cloned().collect())
        .unwrap_or_default()
}

/// Name the one place a confidence mustn't go (E5-5): the deep tells
/// carry it - "not the priest", "not the reeve"
pub fn restrict(world: &mut World, knower: Entity, owner: Entity, fact: &str, target: Entity) {
    if let Some(k) = world.get_component_mut::<Known>(knower) {
        if let Some(c) = k.confidences.iter_mut()
            .find(|c| c.owner == owner && c.fact == fact && c.kept) {
            c.rule = Rule::NotTo(target);
        }
    }
}

/// The owner's close ears (E5-5): the folk under their roof and the
/// bonds that name them family or friend or lover - when the seat's
/// word leaks, it's theirs the story cools for
fn circle(world: &World, owner: Entity) -> Vec<Entity> {
    use crate::game::relationships::{RelKind, Relationship};
    let mut out = vec![];
    if let Some(home) = crate::game::household::home_of(world, owner) {
        if let Some(b) = world.get_component::<crate::game::household::Building>(home) {
            out.extend(b.occupants.iter().copied().filter(|&o| o != owner));
        }
    }
    for bond in crate::game::relationships::bonds_of(world, owner) {
        if let Some(r) = world.get_component::<Relationship>(bond) {
            if matches!(r.kind, RelKind::Family | RelKind::Friend | RelKind::Lover) {
                out.push(if r.from == owner { r.other } else { r.from });
            }
        }
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// A confidence broken (E5-5): word of it reached ears the rule
/// barred, and the owner's heard of it. Their trust in the seat drops
/// - the breach reads as a betrayal of the bond - and their circle
/// cools a little toward the seat: you can't hold a tongue, they
/// say. Returns whether a held confidence actually broke
pub fn leak(world: &mut World, knower: Entity, owner: Entity, fact: &str) -> bool {
    use crate::game::relationships::{record_interaction, Interaction, InteractionKind};
    let broke = world.get_component_mut::<Known>(knower)
        .map(|k| k.confidences.iter_mut()
            .any(|c| if c.owner == owner && c.fact == fact && c.kept {
                c.kept = false; true
            } else { false }))
        .unwrap_or(false);
    if !broke { return false; }
    record_interaction(world, owner, Interaction {
        other: knower, kind: InteractionKind::Betrayal, intensity: 0.8 });
    for c in circle(world, owner) {
        if c == knower { continue; }
        record_interaction(world, c, Interaction {
            other: knower, kind: InteractionKind::Betrayal, intensity: 0.3 });
    }
    true
}

/// The keeping's wage (E5-5): each confidence that's stood its time
/// and still holds pays out once in the owner's read of the seat's
/// integrity - the tongue held, and they saw it hold
pub fn honor_confidences(world: &mut World, knower: Entity, tick: u64) -> usize {
    let mut due = vec![];
    if let Some(k) = world.get_component_mut::<Known>(knower) {
        for c in k.confidences.iter_mut() {
            if c.kept && !c.honored && tick >= c.day + CONFIDENCE_MARK_DAYS {
                c.honored = true;
                due.push(c.owner);
            }
        }
    }
    crate::game::relationships::push_kept_word(world, &due, knower);
    due.len()
}

/// Days a villager has gone without: the streak that turns a hard day
/// into a grievance. Reset the day the want's met again - a single
/// missed meal isn't a crusade
#[derive(Debug, Default)]
pub struct Hardship {
    /// Consecutive days at Hunger < 40 (hungry or starving)
    pub hungry: u32,
    /// Consecutive days showing a sickness's symptoms
    pub sick: u32,
}

/// The organizer's ledger: what was heard, from whom, and when. Entries
/// expire after a year; three or more live entries in a family surface an
/// issue (kept in `issues` once found)
#[derive(Debug, Default)]
pub struct Organizer {
    pub heard: Vec<(IssueTag, Entity, u64)>,
    pub issues: Vec<IssueTag>,
}

/// Mark a want satisfied until `until` days from now (or refresh it)
pub fn satisfy(world: &mut World, e: Entity, want: Want, until: u64) {
    let Some(life) = world.get_component_mut::<InnerLife>(e) else { return };
    match life.satisfied.iter_mut().find(|(w, _)| *w == want) {
        Some((_, u)) => *u = (*u).max(until),
        None => life.satisfied.push((want, until)),
    }
}

/// Whether a want currently counts as met
pub fn is_satisfied(world: &World, e: Entity, want: Want, tick: u64) -> bool {
    world.get_component::<InnerLife>(e)
        .map(|l| l.satisfied.iter().any(|(w, u)| *w == want && tick <= *u))
        .unwrap_or(false)
}

/// Pin a grievance on someone until `until` (e.g. a taxing: the year out)
pub fn add_grievance(world: &mut World, e: Entity, p: Problem, until: u64) {
    let Some(life) = world.get_component_mut::<InnerLife>(e) else { return };
    match life.grievances.iter_mut().find(|(g, _)| *g == p) {
        Some((_, u)) => *u = (*u).max(until),
        None => life.grievances.push((p, until)),
    }
}

/// A problem in a clause the intro card can say - "the bailiff's writ
/// hangs over them"
pub fn wrong_phrase(p: Problem) -> &'static str {
    match p {
        Problem::Taxed => "the tax-man's tally",
        Problem::WritBorne => "the bailiff's writ",
        Problem::Blamed => "the village's blame",
        Problem::DeathDuty => "the lord's fifth",
        Problem::Bereaved => "a grave they still visit",
        Problem::ShortChanged => "a count that didn't add up",
        Problem::OverseerCut => "an overseer's cut",
        Problem::NoHome => "no roof of their own",
        Problem::NoWorkplace => "no bench to work",
        Problem::NoMaster => "no master to learn from",
        Problem::NoInput(_) => "materials they can't lay hands on",
        Problem::DerelictNearby => "an empty shop beside their door",
        Problem::Smelly => "a smell that hangs on them",
        Problem::Loss => "stock lost in the night",
        Problem::SourGut => "a sour gut from spoiled stock",
        Problem::ChildLabor => "a child's hands in the fields",
        Problem::Rowdy => "a fist that ended a night in mud",
        Problem::ThinWood => "woods gone thin",
        Problem::DryPit => "a drowned pit",
        Problem::StinkingReach => "a reach that stinks",
        Problem::Tempted => "the road's calling",
        Problem::LongWinter => "a winter that runs long",
        Problem::ColdHearth => "a hearth that eats cold",
        Problem::TheDark => "the dark that keeps them",
        Problem::Grief => "a hanging they can't unsee",
        Problem::Robbed => "a cellar worth robbing",
        Problem::Hungry => "a belly that knows the pot's bare",
        Problem::Sick => "the fevers riding the village",
        Problem::Broke => "less gold than a meal",
        Problem::Doctrine => "a word the parish calls crooked",
    }
}

/// The issue family a reveal belongs to - parallel complaints share one
pub fn issue_tag(world: &World, e: Entity, r: &Reveal) -> IssueTag {
    let base: &'static str = match r {
        Reveal::Want(Want::GoodFood) => "hunger",
        Reveal::Want(Want::Finery) => "comfort",
        Reveal::Want(Want::Faith) => "faith",
        Reveal::Want(Want::Shelter) | Reveal::Problem(Problem::NoHome) => "housing",
        Reveal::Want(Want::Security) => "security",
        Reveal::Want(Want::Mastery) => "mastery",
        Reveal::Want(Want::Music) | Reveal::Want(Want::Instrument) => "music",
        Reveal::Want(Want::BetterTools) => "tools",
        Reveal::Want(Want::DifferentTrade) => "restlessness",
        Reveal::Want(Want::Love) => "loneliness",
        Reveal::Want(Want::Joy) => "joy",
        Reveal::Want(Want::Drink) => "the drink",
        Reveal::Problem(Problem::NoWorkplace) => "workplaces",
        Reveal::Problem(Problem::NoMaster) => "apprenticeship",
        Reveal::Problem(Problem::NoInput(_)) => "materials",
        Reveal::Problem(Problem::DerelictNearby) => "empty shops",
        Reveal::Problem(Problem::OverseerCut) => "the overseers' cut",
        Reveal::Problem(Problem::Taxed) => "the taxes",
        Reveal::Problem(Problem::Broke) => "poverty",
        Reveal::Problem(Problem::Smelly) => "honest work's stink",
        Reveal::Problem(Problem::Loss) => "the wolves",
        Reveal::Problem(Problem::Bereaved) => "the loss",
        Reveal::Problem(Problem::Blamed) => "the bad bread",
        Reveal::Problem(Problem::SourGut) => "the sour gut",
        Reveal::Problem(Problem::DeathDuty) => "the death duty",
        Reveal::Problem(Problem::ThinWood) => "the shrinking wood",
        Reveal::Problem(Problem::ColdHearth) => "cold hearths",
        Reveal::Problem(Problem::DryPit) => "the drowned pit",
        Reveal::Problem(Problem::StinkingReach) => "the stinking reach",
        Reveal::Problem(Problem::ShortChanged) => "the reckoning that didn't add up",
        Reveal::Problem(Problem::Tempted) => "the road's pull",
        Reveal::Problem(Problem::WritBorne) => "the bailiff's writ",
        Reveal::Problem(Problem::LongWinter) => "the long winter",
        Reveal::Problem(Problem::TheDark) => "the dark",
        Reveal::Problem(Problem::Grief) => "the grief",
        Reveal::Problem(Problem::ChildLabor) => "child labor",
        Reveal::Problem(Problem::Rowdy) => "the rowdy",
        Reveal::Problem(Problem::Robbed) => "the thieves",
        Reveal::Problem(Problem::Hungry) => "the empty pot",
        Reveal::Problem(Problem::Sick) => "the fevers",
        Reveal::Problem(Problem::Doctrine) => "the true faith",
    };
    // a trade's grievance only echoes among the trade - the commons
    // (tax, poverty, the bad bread, the sour gut) echo for everyone
    let scoped = matches!(r,
        Reveal::Problem(Problem::NoWorkplace) | Reveal::Problem(Problem::NoMaster)
        | Reveal::Problem(Problem::NoInput(_)) | Reveal::Problem(Problem::DerelictNearby)
        | Reveal::Problem(Problem::OverseerCut) | Reveal::Problem(Problem::Smelly)
        | Reveal::Problem(Problem::Loss) | Reveal::Want(Want::BetterTools));
    if scoped {
        if let Some(v) = world.get_component::<crate::game::vocation::Craft>(e)
            .map(|c| crate::game::vocation::vocation_name(c.vocation).to_lowercase()) {
            return format!("{} ({})", base, v);
        }
    }
    base.into()
}

/// What the issue is called once the organizer sees the pattern. A
/// trade-scoped tag names its trade: "materials (baker)" reads "the
/// bakers' workshops starved of materials"
pub fn issue_name(tag: IssueTag) -> String {
    if let Some((base, voc)) = tag.strip_suffix(')').and_then(|s| s.split_once(" (")) {
        return format!("the {}s' {}", voc, issue_name(base.to_string()));
    }
    match tag.as_str() {
        "hunger" => "the bread shortage".into(),
        "comfort" => "plain living".into(),
        "faith" => "the neglected faith".into(),
        "housing" => "folk without a roof".into(),
        "security" => "folk who feel unsafe".into(),
        "mastery" => "trades without a path up".into(),
        "music" => "a village with no music".into(),
        "tools" => "folk short of tools".into(),
        "restlessness" => "trades that don't fit".into(),
        "loneliness" => "folk on their own".into(),
        "joy" => "children with nothing to do".into(),
        "workplaces" => "trades without a workshop".into(),
        "apprenticeship" => "youngsters with no master".into(),
        "materials" => "workshops starved of materials".into(),
        "empty shops" => "the boarded-up shops".into(),
        "the overseers' cut" => "the overseers' cut".into(),
        "the taxes" => "the tax collector's cut".into(),
        "the vote" => "a woman's say in the village".into(),
        "the rowdy" => "the strangers' nights".into(),
        "poverty" => "folk without a coin".into(),
        "honest work's stink" => "the stink of honest work".into(),
        "the wolves" => "wolves at the fold".into(),
        "the bad bread" => "the cursed bread".into(),
        "the sour gut" => "spoiled stock in the larder".into(),
        "the death duty" => "the lord's death-duty".into(),
        "the shrinking wood" => "the shrinking wood".into(),
        "the drowned pit" => "the drowned pit".into(),
        "the stinking reach" => "the stinking reach".into(),
        "the reckoning that didn't add up" => "the reckoning that didn't add up".into(),
        "the road's pull" => "the road's pull".into(),
        "the bailiff's writ" => "the bailiff's writ".into(),
        "the long winter" => "the long winter".into(),
        "the dark" => "the dark".into(),
        "the grief" => "the grief".into(),
        "the drink" => "a dry village".into(),
        "the thieves" => "thieves in the night".into(),
        "the true faith" => "the parish's crooked ways".into(),
        _ => tag,
    }
}

/// The themes an organizer reads into a tangle of grievances (#1072):
/// (key, the platform's name, the issue bases that fold into it). A
/// campaign fights a theme, not a thin tag - "the sickness and what it
/// took", not "the sour gut". Trade-scoped tags cluster on their base.
/// The faith and the suffrage keep their own names - a conversion
/// campaign's issue IS the faith's own and the vote's marker must read
/// back literally.
const CLUSTERS: &[(&str, &str, &[&str])] = &[
    ("bread", "the bread that runs short",
     &["hunger", "the empty pot", "the bad bread"]),
    ("sickness", "the sickness and what it took",
     &["the fevers", "the sour gut", "the grief", "the loss"]),
    ("lords_hand", "hard times under the lord's hand",
     &["the taxes", "the bailiff's writ", "the death duty", "poverty",
       "the reckoning that didn't add up"]),
    ("safety", "folk who can't sleep safe",
     &["security", "the thieves", "the wolves", "the rowdy", "the dark"]),
    ("shelter", "roofs and warm hearths",
     &["housing", "cold hearths"]),
    ("trades", "the trades squeezed dry",
     &["workplaces", "materials", "tools", "apprenticeship", "mastery",
       "restlessness", "empty shops", "the overseers' cut", "honest work's stink"]),
    ("land", "the land turning hard",
     &["the shrinking wood", "the drowned pit", "the stinking reach", "the long winter"]),
    ("hearts", "the village's weary heart",
     &["loneliness", "music", "comfort", "the drink", "the road's pull"]),
    ("children", "the little ones' lot",
     &["child labor", "joy"]),
];

/// The cluster key a tag folds into (a scoped tag counts on its base -
/// "materials (baker)" gathers with the trades); None for a tag no
/// theme owns - it keeps its own name on the ballot
pub fn cluster_of(tag: &IssueTag) -> Option<&'static str> {
    let base = tag.split_once(" (").map(|(b, _)| b).unwrap_or(tag.as_str());
    CLUSTERS.iter().find(|(_, _, ts)| ts.contains(&base)).map(|(k, _, _)| *k)
}

/// The platform name for a cluster key
fn cluster_name(key: &'static str) -> &'static str {
    CLUSTERS.iter().find(|(k, _, _)| *k == key).map(|(_, n, _)| *n).unwrap_or(key)
}

/// The cluster a synthesized platform name speaks for (a ballot
/// candidate that is a theme reads back to its key; anything else
/// isn't a platform at all)
pub fn platform_of(tag: &IssueTag) -> Option<&'static str> {
    CLUSTERS.iter().find(|(_, n, _)| tag == n).map(|(k, _, _)| *k)
}

/// Whether a member's tag counts toward a ballot candidate: an exact
/// match, or the member's tag folds into the theme the candidate names
/// (#1072 - a voter's sour gut stands for the sickness platform)
pub fn tag_stands(candidate: &IssueTag, tag: &IssueTag) -> bool {
    if candidate == tag { return true; }
    platform_of(candidate).map(|k| cluster_of(tag) == Some(k)).unwrap_or(false)
}

/// What the ballot names: the issues as the organizer frames them, one
/// candidate per cluster holding two or more issues (the candidate is
/// the platform's name), each thinner issue keeping its own tag.
/// Order follows the issues' order - the cluster lands where its first
/// issue stood. Returns (candidate, its member tags) pairs.
pub fn platforms(issues: &[IssueTag]) -> Vec<(IssueTag, Vec<IssueTag>)> {
    let mut slots: Vec<(Option<&'static str>, Vec<IssueTag>)> = vec![];
    for t in issues {
        match cluster_of(t) {
            Some(k) => match slots.iter_mut().find(|(s, _)| *s == Some(k)) {
                Some((_, ts)) => ts.push(t.clone()),
                None => slots.push((Some(k), vec![t.clone()])),
            },
            None => slots.push((None, vec![t.clone()])),
        }
    }
    slots.into_iter().map(|(k, ts)| match k {
        Some(k) if ts.len() > 1 => (cluster_name(k).to_string(), ts),
        _ => (ts[0].clone(), ts),
    }).collect()
}

/// How a want reads on a character card once revealed
pub fn want_label(w: Want) -> &'static str {
    match w {
        Want::GoodFood => "wants good bread",
        Want::Finery => "wants fine clothes",
        Want::Faith => "wants the faith",
        Want::Shelter => "wants a home",
        Want::Security => "wants to feel safe",
        Want::Mastery => "wants to rise in the trade",
        Want::Music => "wants to hear music",
        Want::Instrument => "wants an instrument",
        Want::BetterTools => "wants better tools",
        Want::DifferentTrade => "bored with the trade",
        Want::Love => "wants love",
        Want::Joy => "wants to play",
        Want::Drink => "wants a drink",
    }
}

/// The want as a first-person object - "I'd like <this>" (#1269)
pub fn want_phrase(w: Want) -> &'static str {
    match w {
        Want::GoodFood => "good bread",
        Want::Finery => "fine clothes",
        Want::Faith => "the faith",
        Want::Shelter => "a home of my own",
        Want::Security => "to feel safe",
        Want::Mastery => "to rise in the trade",
        Want::Music => "music",
        Want::Instrument => "an instrument",
        Want::BetterTools => "better tools",
        Want::DifferentTrade => "a different trade",
        Want::Love => "love",
        Want::Joy => "a bit of play",
        Want::Drink => "a drink",
    }
}

/// How a problem reads on a character card once revealed
pub fn problem_label(p: &Problem) -> String {
    match p {
        Problem::NoHome => "has no home".into(),
        Problem::NoWorkplace => "has no workshop".into(),
        Problem::NoMaster => "has no master".into(),
        Problem::NoInput(g) => format!("can't get {}", g),
        Problem::DerelictNearby => "lives by an empty shop".into(),
        Problem::ChildLabor => "sees a child's hands in the rows".into(),
        Problem::Rowdy => "sees the strangers' nights turn ugly".into(),
        Problem::OverseerCut => "the overseer takes a cut".into(),
        Problem::Taxed => "paid the tax collector".into(),
        Problem::Broke => "is nearly broke".into(),
        Problem::Smelly => "smells of honest work".into(),
        Problem::Loss => "lost stock to wolves".into(),
        Problem::Bereaved => "lost someone close".into(),
        Problem::Blamed => "is blamed for the sickness".into(),
        Problem::SourGut => "ate spoiled stock".into(),
        Problem::DeathDuty => "paid the lord's fifth on the dead".into(),
        Problem::ThinWood => "the wood's thin this side of the hill".into(),
        Problem::DryPit => "the pit's drowned - no stone to be had".into(),
        Problem::StinkingReach => "the reach below town stinks".into(),
        Problem::ShortChanged => "shorted in a foreign count".into(),
        Problem::Tempted => "the road's calling".into(),
        Problem::WritBorne => "a writ came down from the castle".into(),
        Problem::LongWinter => "the winter runs long and the stores run thin".into(),
        Problem::ColdHearth => "no fuel for the hearth".into(),
        Problem::TheDark => "the dark keeps me".into(),
        Problem::Grief => "I watched it and can't unsee it".into(),
        Problem::Robbed => "was robbed in the night".into(),
        Problem::Hungry => "has known the empty pot too long".into(),
        Problem::Sick => "is sick and can't get well".into(),
        Problem::Doctrine => "hears the plain word".into(),
    }
}

/// What a revealed secret sounds like said aloud: (what the log saw them
/// do, what they said in the bubble)
pub fn reveal_speech(name: &str, r: &Reveal) -> (String, String) {
    match r {
        Reveal::Want(Want::GoodFood) => (format!("{} admits missing good bread", name), "A crust of real bread would be something.".into()),
        Reveal::Want(Want::Finery) => (format!("{} wishes for fine clothes", name), "One decent garment. Is that so much?".into()),
        Reveal::Want(Want::Faith) => (format!("{} misses the faith", name), "It's been too long since I felt anything.".into()),
        Reveal::Want(Want::Shelter) => (format!("{} longs for a home", name), "A roof of my own. That's all I ask.".into()),
        Reveal::Want(Want::Security) => (format!("{} feels unsafe", name), "I bar my door at night and still don't sleep.".into()),
        Reveal::Want(Want::Mastery) => (format!("{} wants to rise in the trade", name), "I'm better than this. Someone should see it.".into()),
        Reveal::Want(Want::Music) => (format!("{} misses music", name), "Not a song in the village for a year.".into()),
        Reveal::Want(Want::Instrument) => (format!("{} covets an instrument", name), "If I had a lute I'd never put it down.".into()),
        Reveal::Want(Want::BetterTools) => (format!("{} needs better tools", name), "My hands do the work my tools won't.".into()),
        Reveal::Want(Want::DifferentTrade) => (format!("{} is bored with the trade", name), "This isn't the life I pictured.".into()),
        Reveal::Want(Want::Love) => (format!("{} confides they're lonely", name), "Everyone's got someone. Not me.".into()),
        Reveal::Want(Want::Joy) => (format!("{} wants to play", name), "I just want to play a while.".into()),
        Reveal::Want(Want::Drink) => (format!("{} misses a cup", name), "A cup of something would sit well about now.".into()),
        Reveal::Problem(Problem::NoHome) => (format!("{} has no home", name), "Nowhere to lay my head. Nowhere.".into()),
        Reveal::Problem(Problem::NoWorkplace) => (format!("{} has no workshop", name), "No bench, no hearth - I work in the street.".into()),
        Reveal::Problem(Problem::NoMaster) => (format!("{} has no master", name), "No one to learn from. I'll be a journeyman forever.".into()),
        Reveal::Problem(Problem::NoInput(g)) => (format!("{} can't get {}", name, g), format!("No {} to be had anywhere.", g)),
        Reveal::Problem(Problem::DerelictNearby) => (format!("{} hates the empty shop", name), "That boarded shop on the corner. It sits there like a sore.".into()),
        Reveal::Problem(Problem::OverseerCut) => (format!("{} resents the overseer's cut", name), "I sweat and he takes his share for watching.".into()),
        Reveal::Problem(Problem::Taxed) => (format!("{} curses the tax collector", name), "The collector took his cut again. The lord eats well.".into()),
        Reveal::Problem(Problem::Broke) => (format!("{} is nearly broke", name), "Two coins left. Two.".into()),
        Reveal::Problem(Problem::Smelly) => (format!("{} stinks of honest work", name), "Aye, honest work hangs on - it'll wash off.".into()),
        Reveal::Problem(Problem::Loss) => (format!("{} lost stock to wolves", name), "The wolf came in the night. I found what's left of her.".into()),
        Reveal::Problem(Problem::Bereaved) => (format!("{} is grieving", name), "They were taken too soon. I keep going over it.".into()),
        Reveal::Problem(Problem::Blamed) => (format!("{} is blamed for the sickness", name), "They say my bread did it. My bread.".into()),
        Reveal::Problem(Problem::SourGut) => (format!("{} ate spoiled stock", name), "It was all there was. Still smells.".into()),
        Reveal::Problem(Problem::DeathDuty) => (format!("{} paid the lord's fifth", name), "A fifth of all he had. For dying.".into()),
        Reveal::Problem(Problem::ThinWood) => (format!("{}: the wood's thin", name), "Walked half a day for a bundle. It wasn't always so.".into()),
        Reveal::Problem(Problem::DryPit) => (format!("{}: the pit's drowned", name), "Water in the cut. No stone comes out of a pond.".into()),
        Reveal::Problem(Problem::StinkingReach) => (format!("{}: the reach stinks", name), "You can smell it on a still night. That wasn't always so.".into()),
        Reveal::Problem(Problem::ShortChanged) => (format!("{}: the reckoning's off", name), "He counted it his way. My way it's short.".into()),
        Reveal::Problem(Problem::Tempted) => (format!("{}'s feet itch", name), "The road looks shorter from the gate than the days look here.".into()),
        Reveal::Problem(Problem::WritBorne) => (format!("{} bore the writ", name), "Sealed by the castle. No arguing with wax.".into()),
        Reveal::Problem(Problem::LongWinter) => (format!("{}'s stores are thin", name), "Second winter running. The barrel knows it.".into()),
        Reveal::Problem(Problem::ColdHearth) => (format!("{} eats cold", name), "No bavin, no turf, not even the dry leavings. The pot's cold and the bed's colder.".into()),
        Reveal::Problem(Problem::TheDark) => (format!("{} keeps to the dark", name), "The village is loud with light. Down here it's honest.".into()),
        Reveal::Problem(Problem::Grief) => (format!("{} saw it", name), "They strung him up where we could see. I can't unsee it.".into()),
        Reveal::Problem(Problem::Robbed) => (format!("{} was robbed", name), "In the night, quiet as a cat. Took the good stock.".into()),
        Reveal::Problem(Problem::ChildLabor) => (format!("{} saw the child's hands", name), "Small hands in the rows when they should be at play. It isn't right.".into()),
        Reveal::Problem(Problem::Rowdy) => (format!("{} saw the strangers' brawl", name), "The season hands drink hard and the nights end in the mud. I saw it.".into()),
        Reveal::Problem(Problem::Hungry) => (format!("{}'s pot is bare", name), "Another day on thin broth. The pot's been bare a week.".into()),
        Reveal::Problem(Problem::Sick) => (format!("{} is sick", name), "The fever rides the village. Nobody's untouched.".into()),
        Reveal::Problem(Problem::Doctrine) => (format!("{} takes the green word", name), "The church sells what God gives free. The word is for all of us.".into()),
    }
}

/// The still-carrying line for a fact already heard (#1347): a re-told
/// trouble reads as catching up, not amnesia - "it still presses", not
/// a fresh confession
pub fn catchup_speech(name: &str, r: &Reveal) -> (String, String) {
    match r {
        Reveal::Want(Want::GoodFood) => (format!("{} still misses good bread", name), "The bread's no better, if {2:subj} {2:be} asking.".into()),
        Reveal::Want(Want::Finery) => (format!("{} still wishes for fine clothes", name), "Still not one decent garment to my name.".into()),
        Reveal::Want(Want::Faith) => (format!("{} still misses the faith", name), "I still can't feel it, whatever I do.".into()),
        Reveal::Want(Want::Shelter) => (format!("{} still longs for a home", name), "Still sleeping under another's roof.".into()),
        Reveal::Want(Want::Security) => (format!("{} still feels unsafe", name), "The door's barred still. It doesn't help.".into()),
        Reveal::Want(Want::Mastery) => (format!("{} still wants to rise in the trade", name), "Nobody's seen it yet. I'm still better than this.".into()),
        Reveal::Want(Want::Music) => (format!("{} still misses music", name), "Still not a song to be had.".into()),
        Reveal::Want(Want::Instrument) => (format!("{} still covets an instrument", name), "The lute's still a dream, friend.".into()),
        Reveal::Want(Want::BetterTools) => (format!("{} still needs better tools", name), "Same dull tools. Same hard work.".into()),
        Reveal::Want(Want::DifferentTrade) => (format!("{} still finds the trade stale", name), "This still isn't the life I pictured.".into()),
        Reveal::Want(Want::Love) => (format!("{} is still lonely", name), "Still no one, when the lamp goes out.".into()),
        Reveal::Want(Want::Joy) => (format!("{} still wants to play", name), "Still no play to be had.".into()),
        Reveal::Want(Want::Drink) => (format!("{} still misses a cup", name), "That cup would still sit well.".into()),
        Reveal::Problem(Problem::NoHome) => (format!("{} still has no home", name), "Still nowhere to lay my head.".into()),
        Reveal::Problem(Problem::NoWorkplace) => (format!("{} still has no workshop", name), "Still working in the street.".into()),
        Reveal::Problem(Problem::NoMaster) => (format!("{} still has no master", name), "Still no one to learn from.".into()),
        Reveal::Problem(Problem::NoInput(g)) => (format!("{} still can't get {}", name, g), format!("Still no {} to be had.", g)),
        Reveal::Problem(Problem::DerelictNearby) => (format!("{} still hates the empty shop", name), "The boarded shop sits there still.".into()),
        Reveal::Problem(Problem::OverseerCut) => (format!("{} still resents the overseer's cut", name), "He takes his share still, for watching.".into()),
        Reveal::Problem(Problem::Taxed) => (format!("{} still curses the tax collector", name), "The tolls press the same as ever.".into()),
        Reveal::Problem(Problem::Broke) => (format!("{} is still nearly broke", name), "The purse is no fatter, if {2:subj} {2:be} asking.".into()),
        Reveal::Problem(Problem::Smelly) => (format!("{} still stinks of honest work", name), "The smell still hangs on.".into()),
        Reveal::Problem(Problem::Loss) => (format!("{} still counts what the wolves took", name), "I still count what the wolves took.".into()),
        Reveal::Problem(Problem::Bereaved) => (format!("{} is still grieving", name), "The grief's still with me.".into()),
        Reveal::Problem(Problem::Blamed) => (format!("{} is still blamed for the sickness", name), "They still say my bread did it.".into()),
        Reveal::Problem(Problem::SourGut) => (format!("{} still suffers the spoiled stock", name), "The gut's still not done with me.".into()),
        Reveal::Problem(Problem::DeathDuty) => (format!("{} still feels the lord's fifth", name), "A fifth, still gone. For dying.".into()),
        Reveal::Problem(Problem::ThinWood) => (format!("{}: the wood's still thin", name), "Still half a day's walk for a bundle.".into()),
        Reveal::Problem(Problem::DryPit) => (format!("{}: the pit's still drowned", name), "Still water in the cut.".into()),
        Reveal::Problem(Problem::StinkingReach) => (format!("{}: the reach still stinks", name), "You can smell it still, on a quiet night.".into()),
        Reveal::Problem(Problem::ShortChanged) => (format!("{}: the reckoning's still off", name), "Counted his way again. Short again.".into()),
        Reveal::Problem(Problem::Tempted) => (format!("{}'s feet still itch", name), "The road still looks shorter from the gate.".into()),
        Reveal::Problem(Problem::WritBorne) => (format!("{} still bears the writ", name), "The wax still says what it says.".into()),
        Reveal::Problem(Problem::LongWinter) => (format!("{}'s stores are still thin", name), "The barrel still knows it.".into()),
        Reveal::Problem(Problem::ColdHearth) => (format!("{} still eats cold", name), "The pot's still cold. The bed's no warmer.".into()),
        Reveal::Problem(Problem::TheDark) => (format!("{} still keeps to the dark", name), "Still honest, down here in the dark.".into()),
        Reveal::Problem(Problem::Grief) => (format!("{} still sees it", name), "Strung up where we could see. I still see it.".into()),
        Reveal::Problem(Problem::Robbed) => (format!("{} still remembers the robbery", name), "Took the good stock, quiet as a cat. Nothing's come back.".into()),
        Reveal::Problem(Problem::ChildLabor) => (format!("{} still sees the child's hands", name), "Small hands in the rows still. It isn't right.".into()),
        Reveal::Problem(Problem::Rowdy) => (format!("{} still sees the strangers' brawls", name), "The nights still end in the mud.".into()),
        Reveal::Problem(Problem::Hungry) => (format!("{}'s pot is still bare", name), "Thin broth again today.".into()),
        Reveal::Problem(Problem::Sick) => (format!("{} is still sick", name), "The fever still rides.".into()),
        Reveal::Problem(Problem::Doctrine) => (format!("{} still takes the green word", name), "The word's still for all of us.".into()),
    }
}

/// What a person's carrying right now: their live problems and their
/// unsatisfied wants - the pool every reveal draw works from
pub(crate) fn live_pool(world: &World, e: Entity, tick: u64) -> Vec<Reveal> {
    let mut out: Vec<Reveal> = problems_of(world, e, tick).into_iter()
        .map(Reveal::Problem).collect();
    if let Some(life) = world.get_component::<InnerLife>(e) {
        for w in &life.wants {
            if is_satisfied(world, e, *w, tick) { continue; }
            out.push(Reveal::Want(*w));
        }
    }
    out
}

/// What a person can be made to reveal to `knower`: their unsatisfied
/// wants, their live problems, and their fresh grievances - skipping
/// anything that seat's already heard
pub fn unrevealed(world: &World, knower: Entity, e: Entity, tick: u64) -> Vec<Reveal> {
    let heard = known(world, knower, e);
    let mut out = live_pool(world, e, tick);
    out.retain(|r| !heard.contains(r));
    out
}

/// The live facts already told (#1347): what a catch-up draw can still
/// speak to - a burden they still carry that you've already heard
pub fn live_known(world: &World, knower: Entity, e: Entity, tick: u64) -> Vec<Reveal> {
    let heard = known(world, knower, e);
    let mut out = live_pool(world, e, tick);
    out.retain(|r| heard.contains(r));
    out
}

/// What they'd ask at the door (E5-3, #1435): the pressing thing a
/// worry question keys off - the live pool's a person's whole current
/// load, problems first. `seed` picks the one, stable across a talk's
/// replays; None when they're carrying nothing the talk could take up
pub fn worry_of(world: &World, e: Entity, tick: u64, seed: u64) -> Option<Reveal> {
    let pool = live_pool(world, e, tick);
    if pool.is_empty() { return None; }
    Some(pool[(seed % pool.len() as u64) as usize])
}

/// Pick one secret to share: problems first (they're what's pressing),
/// then wants. `roll` is 0..1 - callers give it a boosted chance
pub fn next_reveal(world: &World, knower: Entity, e: Entity, tick: u64, seed: u64) -> Option<Reveal> {
    let pool = unrevealed(world, knower, e, tick);
    if pool.is_empty() { return None; }
    Some(pool[(seed % pool.len() as u64) as usize])
}

/// Pick an old truth to come back around (#1347): once every live fact
/// is heard, asking can still catch up on what's still pressing. None
/// when nothing's been heard yet or what's heard has all resolved - a
/// healed trouble doesn't get re-mourned
pub fn catchup_reveal(world: &World, knower: Entity, e: Entity, tick: u64, seed: u64) -> Option<Reveal> {
    let pool = live_known(world, knower, e, tick);
    if pool.is_empty() { return None; }
    Some(pool[(seed % pool.len() as u64) as usize])
}

/// Record that the organizer heard `r` from `person`: mark it known on
/// the seat's own ledger, journal it on the organizer, prune entries
/// older than a year, and return the new issue if three or more people
/// have now said versions of the same thing inside the year
pub fn hear(world: &mut World, organizer: Entity, person: Entity, r: Reveal, tick: u64) -> Option<String> {
    hear_via(world, organizer, person, r, tick, Source::Told { speaker: person })
}

/// `hear` with the source named (E7-2/E5-5): a carried word lands as
/// `Gossip { speaker }` - the ledger hears it as hearsay, carries no
/// confidence with it, and the journal still counts the voice
pub fn hear_via(world: &mut World, organizer: Entity, person: Entity, r: Reveal, tick: u64, source: Source) -> Option<String> {
    // the hearing's the seat's own (E7-1): the record lands on the
    // organizer, so another seat's ledger stays blind to it
    if known(world, organizer, person).contains(&r) { return None; }
    // they said it themselves, today (E7-2): the entry carries its
    // teller, its day, and a telling's certainty
    record(world, organizer, person, reveal_key(&r), Some(r), source, tick);
    let tag = issue_tag(world, person, &r);
    let org = world.get_component_mut::<Organizer>(organizer)?;
    org.heard.retain(|(_, _, day)| tick <= *day + YEAR_DAYS);
    org.heard.push((tag.clone(), person, tick));
    if org.issues.contains(&tag) { return None; }
    let voices: Vec<Entity> = org.heard.iter()
        .filter(|(t, _, _)| *t == tag).map(|(_, e, _)| *e).collect();
    let mut dedup = voices.clone();
    dedup.sort_unstable();
    dedup.dedup();
    if dedup.len() >= 3 {
        org.issues.push(tag.clone());
        Some(issue_name(tag))
    } else {
        None
    }
}

/// Add an issue directly (e.g. a house meeting where 3+ voiced it) - the
/// same as hearing it out three times over
pub fn found_issue(world: &mut World, organizer: Entity, tag: IssueTag) -> Option<String> {
    let org = world.get_component_mut::<Organizer>(organizer)?;
    if org.issues.contains(&tag) { return None; }
    org.issues.push(tag.clone());
    Some(issue_name(tag))
}

/// A person's live problems, worked out from the world: where they live,
/// where they work, who teaches them, what they can't buy, the state of
/// their street, and what they've been through lately
pub fn problems_of(world: &World, e: Entity, tick: u64) -> Vec<Problem> {
    use crate::game::household::{self, Building};
    use crate::game::villager::Pos;
    let mut out = vec![];
    // a roof: they must be an occupant of some Home
    let housed = world.query::<Building>().iter().any(|&b| {
        world.get_component::<Building>(b)
            .map(|bb| bb.kind == household::BuildingKind::Home && bb.occupants.contains(&e))
            .unwrap_or(false)
    });
    if !housed { out.push(Problem::NoHome); }
    if let Some(c) = world.get_component::<Craft>(e) {
        if c.workplace.is_none() { out.push(Problem::NoWorkplace); }
        if c.master.is_none() && vocation::level_of(c.skill) < vocation::Level::Master {
            out.push(Problem::NoMaster);
        }
        // the fold's smell hangs on a shepherd; folk notice
        if c.vocation == vocation::Vocation::Shepherd { out.push(Problem::Smelly); }
    }
    // a derelict shop (workplace kind with no worker) within a stroll of home
    if let Some(p) = world.get_component::<Pos>(e).copied() {
        let near_derelict = world.query::<Building>().iter().any(|&b| {
            let Some(bb) = world.get_component::<Building>(b) else { return false };
            if !household::derelict(bb) { return false; }
            world.get_component::<Pos>(b)
                .map(|bp| (bp.x - p.x).abs() + (bp.y - p.y).abs() < 6.0)
                .unwrap_or(false)
        });
        if near_derelict { out.push(Problem::DerelictNearby); }
    }
    if let Some(life) = world.get_component::<InnerLife>(e) {
        out.extend(life.grievances.iter()
            .filter(|(_, until)| tick <= *until).map(|(p, _)| *p));
    }
    out
}

/// Sample a person's wants for seeding: `n` draws from a pool weighted by
/// class and vocation. Deterministic via `rng` (a splitmix64 next() value
/// per draw).
pub fn sample_wants(class: SocialClass, voc: Option<vocation::Vocation>, is_kid: bool, rng: &mut impl FnMut() -> u64) -> Vec<Want> {
    if is_kid {
        let mut w = vec![Want::Joy];
        if rng() % 3 == 0 { w.push(Want::Instrument); }
        return w;
    }
    // every want's weight for this person: base + vocation/class flavour
    let weight = |w: Want| -> u64 {
        let base: u64 = match w {
            Want::GoodFood => 4, Want::Security => 3, Want::Mastery => 3,
            Want::Love => 3, Want::Faith => 2, Want::Music => 2,
            Want::Finery => 2, Want::BetterTools => 2, Want::Shelter => 2,
            Want::DifferentTrade => 2, Want::Instrument => 1, Want::Joy => 1,
            Want::Drink => 3,
        };
        let bonus: u64 = match (w, voc) {
            (Want::Faith, Some(vocation::Vocation::Priest)) => 6,
            (Want::Music, Some(vocation::Vocation::Luthier)) => 6,
            (Want::Instrument, Some(vocation::Vocation::Luthier)) => 4,
            (Want::GoodFood, Some(vocation::Vocation::Baker)) => 2,
            (Want::BetterTools, Some(vocation::Vocation::Mason)) => 3,
            (Want::BetterTools, Some(vocation::Vocation::Toolmaker)) => 2,
            (Want::Mastery, Some(vocation::Vocation::Knight)) => 3,
            (Want::Finery, Some(vocation::Vocation::Tailor)) => 4,
            (Want::Finery, Some(vocation::Vocation::Glazier)) => 2,
            _ => 0,
        };
        let class_bonus: u64 = match (w, class) {
            (Want::Finery, SocialClass::ProfessionalMiddle) => 2,
            (Want::Finery, SocialClass::PetitBourgeois) => 3,
            (Want::Mastery, SocialClass::Working) => 2,
            (Want::Security, SocialClass::ProfessionalMiddle) => 0,
            _ => 0,
        };
        base + bonus + class_bonus
    };
    const ALL: [Want; 12] = [
        Want::GoodFood, Want::Finery, Want::Faith, Want::Shelter, Want::Security,
        Want::Mastery, Want::Music, Want::Instrument, Want::BetterTools,
        Want::DifferentTrade, Want::Love, Want::Drink,
    ];
    let mut pool: Vec<Want> = ALL.iter().copied()
        .flat_map(|w| std::iter::repeat(w).take(weight(w) as usize)).collect();
    // mostly two wants, sometimes one or three
    let n = match rng() % 10 { 0..=1 => 1, 2..=7 => 2, _ => 3 };
    let mut out = vec![];
    for _ in 0..n {
        if pool.is_empty() { break; }
        let i = (rng() % pool.len() as u64) as usize;
        let w = pool[i];
        pool.retain(|x| *x != w);
        out.push(w);
    }
    out
}

/// Whether a `Kid` marker is on the entity
pub fn is_kid(world: &World, e: Entity) -> bool {
    world.get_component::<Kid>(e).is_some()
}

/// What the posting is called over the village's tongue
pub fn town_role_name(role: TownRole) -> &'static str {
    match role {
        TownRole::Overseer => "Overseer",
        TownRole::TaxCollector => "Tax Collector",
        TownRole::Trader => "Trader",
        TownRole::Herbologist => "Herb-wife",
        TownRole::Matchmaker => "Heart-helper",
        TownRole::Teacher => "Schoolmistress",
        TownRole::Hermit => "Hermit",
    }
}

/// Their posting, for the inspector: the trade or PMC role they hold
pub fn occupation_of(world: &World, e: Entity) -> Option<String> {
    if let Some(c) = world.get_component::<Craft>(e) {
        let lvl = vocation::level_name(vocation::level_of(c.skill));
        return Some(format!("{} {}", lvl, vocation::vocation_name(c.vocation)));
    }
    if world.get_component::<Kid>(e).is_some() { return Some("Child".into()); }
    world.get_component::<TownRole>(e).map(|r| town_role_name(*r).to_string())
}

/// The class a character reads as: PMC roles and PMC vocations read PMC;
/// masters who own their workshop read petit bourgeois
pub fn class_of(world: &World, e: Entity) -> SocialClass {
    if let Some(role) = world.get_component::<TownRole>(e) {
        // the herb-wife works her hedge like any labourer; the rest of the
        // postings carry the lord's or the profession's standing
        return match role { TownRole::Herbologist | TownRole::Matchmaker => SocialClass::Working, _ => SocialClass::ProfessionalMiddle };
    }
    if let Some(c) = world.get_component::<Craft>(e) {
        if vocation::is_pmc(c.vocation) { return SocialClass::ProfessionalMiddle; }
        if vocation::level_of(c.skill) == vocation::Level::Master && c.workplace.is_some() {
            return SocialClass::PetitBourgeois;
        }
    }
    SocialClass::Working
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::household::{Building, BuildingKind};
    use crate::game::villager::Pos;
    use crate::game::vocation::craft;

    fn person(w: &mut World, wants: &[Want]) -> Entity {
        let e = w.create_entity();
        w.add_component(e, Pos { x: 0.0, y: 0.0 });
        w.add_component(e, InnerLife { wants: wants.to_vec(), grievances: vec![], satisfied: vec![] });
        e
    }

    fn housed(w: &mut World, e: Entity) {
        let b = w.create_entity();
        w.add_component(b, Pos { x: 0.0, y: 0.0 });
        w.add_component(b, Building {
            kind: BuildingKind::Home, name: "a house".into(),
            occupants: vec![e], worker: None, overseer: None,
            stock: Default::default(), capacity: 20, flyer: None,
        });
    }

    #[test]
    fn test_problems_come_from_the_situation() {
        let mut w = World::new();
        let e = person(&mut w, &[]);
        assert_eq!(problems_of(&w, e, 0), vec![Problem::NoHome]);
        housed(&mut w, e);
        assert!(problems_of(&w, e, 0).is_empty());
        // an unpaired journeyman with no shop has two more problems
        let c = craft(vocation::Vocation::Mason, 0.2);
        w.add_component(e, c);
        let probs = problems_of(&w, e, 0);
        assert!(probs.contains(&Problem::NoWorkplace) && probs.contains(&Problem::NoMaster));
    }

    #[test]
    fn test_hearing_journals_and_discovers_the_pattern() {
        let mut w = World::new();
        let org = w.create_entity();
        w.add_component(org, Organizer::default());
        let (a, b, c) = (person(&mut w, &[]), person(&mut w, &[]), person(&mut w, &[]));
        let r = Reveal::Problem(Problem::Taxed);
        assert_eq!(hear(&mut w, org, a, r, 100), None);
        assert_eq!(hear(&mut w, org, b, r, 200), None);
        // the same person twice doesn't make a pattern
        assert_eq!(hear(&mut w, org, a, Reveal::Problem(Problem::Broke), 210), None);
        assert_eq!(hear(&mut w, org, c, r, 300).as_deref(), Some("the tax collector's cut"));
        // and it isn't discovered twice
        assert_eq!(hear(&mut w, org, c, r, 300), None);
        let heard = known(&w, org, a);
        assert!(heard.contains(&r) && heard.contains(&Reveal::Problem(Problem::Broke)));
    }

    #[test]
    fn test_heard_entries_expire_after_a_year() {
        let mut w = World::new();
        let org = w.create_entity();
        w.add_component(org, Organizer::default());
        let (a, b, c) = (person(&mut w, &[]), person(&mut w, &[]), person(&mut w, &[]));
        let r = Reveal::Problem(Problem::OverseerCut);
        hear(&mut w, org, a, r, 10);
        hear(&mut w, org, b, r, 20);
        // at day 371 the oldest complaint is stale (10 + a year), the
        // second one still counts - so c makes two voices, not three
        assert_eq!(hear(&mut w, org, c, r, 371), None,
            "a fresh complaint plus a stale one shouldn't surface an issue");
        // the fourth voice inside the year completes the pattern
        let d = person(&mut w, &[]);
        assert_eq!(hear(&mut w, org, d, r, 372).as_deref(), Some("the overseers' cut"));
    }

    #[test]
    fn test_unrevealed_skips_known_and_satisfied() {
        let mut w = World::new();
        let e = person(&mut w, &[Want::GoodFood, Want::Love]);
        let org = w.create_entity();
        w.add_component(org, Organizer::default());
        // NoHome is the only live problem; both wants unrevealed
        let pool = unrevealed(&w, org, e, 0);
        assert_eq!(pool.len(), 3);
        hear(&mut w, org, e, Reveal::Problem(Problem::NoHome), 0);
        satisfy(&mut w, e, Want::GoodFood, 30);
        let pool = unrevealed(&w, org, e, 10);
        assert_eq!(pool, vec![Reveal::Want(Want::Love)]);
    }

    #[test]
    fn test_wants_are_weighted_but_sparse() {
        let mut seed = 7u64;
        let mut rng = || { seed = seed.wrapping_add(0x9E3779B97F4A7C15); seed };
        for _ in 0..50 {
            let w = sample_wants(SocialClass::Working, Some(vocation::Vocation::Priest), false, &mut rng);
            assert!((1..=3).contains(&w.len()));
        }
        let kids = sample_wants(SocialClass::Working, None, true, &mut rng);
        assert!(kids.contains(&Want::Joy));
    }

    #[test]
    fn test_classes_read_off_the_components() {
        let mut w = World::new();
        let e = person(&mut w, &[]);
        assert_eq!(class_of(&w, e), SocialClass::Working);
        w.add_component(e, craft(vocation::Vocation::Knight, 0.5));
        assert_eq!(class_of(&w, e), SocialClass::ProfessionalMiddle);
        w.add_component(e, craft(vocation::Vocation::Baker, 0.9));
        assert_eq!(class_of(&w, e), SocialClass::Working, "a master with no shop is still working class");
    }

    #[test]
    fn test_the_thaw_gradient_runs_both_ways() {
        let mut w = World::new();
        let (v, asker) = (person(&mut w, &[]), person(&mut w, &[]));
        // cold: the issue's weather to them
        assert_eq!(rung_of(&w, v), ThawRung::Helpless);
        // the probe plants: warming while the want ripens
        plant_want(&mut w, v, asker, 0, 3);
        assert_eq!(rung_of(&w, v), ThawRung::Thawing);
        // ripened: the mark keeps them warm
        assert!(!ripen_planted(&mut w, v, PLANT_RIPEN_DAYS).is_empty());
        assert!(w.get_component::<Thawed>(v).is_some());
        assert_eq!(rung_of(&w, v), ThawRung::Thawing);
        // signed on: joined
        let org = w.create_entity();
        w.add_component(org, crate::sim::community::Organization {
            name: "the rooted".into(), members: vec![v], leader: asker,
            influence_pool: 1.0, resources: 0.0, charter: "".into(), successors: vec![] });
        assert_eq!(rung_of(&w, v), ThawRung::Joined);
        // holding the meeting's work: acting
        w.add_component(asker, crate::game::organizing::MeetingPlan {
            opener: Some(v), ..Default::default() });
        assert_eq!(rung_of(&w, v), ThawRung::Acting);
        // let the work go: back to joined
        w.remove_component::<crate::game::organizing::MeetingPlan>(asker);
        assert_eq!(rung_of(&w, v), ThawRung::Joined);
        // and falling back out re-chills - to the thaw the probe left
        w.get_component_mut::<crate::sim::community::Organization>(org).unwrap().members.clear();
        assert_eq!(rung_of(&w, v), ThawRung::Thawing);
    }

    #[test]
    fn test_the_probe_stays_open_while_theres_something_left_to_hear() {
        let mut w = World::new();
        let (v, asker) = (person(&mut w, &[]), person(&mut w, &[]));
        assert!(probe_open(&w, asker, v, 0));
        // carry every probeable want, all of them heard, nothing
        // pressing - the question's spent
        w.get_component_mut::<InnerLife>(v).unwrap().wants = PROBE_WANTS.to_vec();
        housed(&mut w, v);
        for x in PROBE_WANTS.iter() { note(&mut w, asker, v, Reveal::Want(*x)); }
        assert!(!probe_open(&w, asker, v, 0));
        // a kid never takes a planted want
        let kid = w.create_entity();
        w.add_component(kid, Pos { x: 0.0, y: 0.0 });
        w.add_component(kid, Kid);
        assert!(!probe_open(&w, asker, kid, 0));
        // a want already ripening still counts as covered
        w.get_component_mut::<InnerLife>(v).unwrap().wants.clear();
        plant_want(&mut w, v, asker, 0, 1);
        assert!(probe_open(&w, asker, v, 0), "other wants remain open");
        w.get_component_mut::<InnerLife>(v).unwrap().wants = PROBE_WANTS.to_vec();
        assert!(!probe_open(&w, asker, v, 0));
        // but the world moving re-opens it: a fresh grievance is
        // something to hear - heard or run out, it closes again
        add_grievance(&mut w, v, Problem::Loss, 60);
        assert!(probe_open(&w, asker, v, 0), "a new burden reopens the question");
        hear(&mut w, asker, v, Reveal::Problem(Problem::Loss), 0);
        assert!(!probe_open(&w, asker, v, 0), "heard, it's spent again");
    }

    #[test]
    fn test_every_ledger_fact_carries_a_source() {
        // the done-when: nothing in the seat's ledger floats free of
        // where it came from, or when it landed
        let mut w = World::new();
        let org = w.create_entity();
        w.add_component(org, Organizer::default());
        let v = person(&mut w, &[Want::GoodFood]);
        let carrier = person(&mut w, &[]);
        hear(&mut w, org, v, Reveal::Want(Want::GoodFood), 42);
        record(&mut w, org, v, "ghost:levy".into(), None,
            Source::Gossip { speaker: carrier }, 50);
        record(&mut w, org, v, "deed:watched_the_tithe".into(), None,
            Source::Witnessed, 60);
        let got = entries(&w, org, v);
        assert_eq!(got.len(), 3);
        assert!(got.iter().all(|en| en.confidence > 0.0));
        for en in &got {
            match en.source {
                Source::Told { speaker } => assert_eq!(speaker, v),
                Source::Gossip { .. } | Source::Witnessed => assert!(en.day > 0),
            }
        }
        // a telling lands first, the day it was said
        assert_eq!(got[0].fact, "want:good_food");
        assert_eq!(got[0].day, 42);
        assert_eq!(got[0].reveal, Some(Reveal::Want(Want::GoodFood)));
    }

    #[test]
    fn test_a_telling_outweighs_hearsay() {
        let mut w = World::new();
        let (knower, v, carrier) = (w.create_entity(), person(&mut w, &[]), person(&mut w, &[]));
        record(&mut w, knower, v, "problem:broke".into(),
            Some(Reveal::Problem(Problem::Broke)),
            Source::Told { speaker: v }, 10);
        record(&mut w, knower, v, "deed:kept_the_gate".into(), None,
            Source::Gossip { speaker: carrier }, 10);
        record(&mut w, knower, v, "deed:took_the_oath".into(), None,
            Source::Witnessed, 10);
        let got = entries(&w, knower, v);
        let told = got[0].confidence;
        let gossiped = got[1].confidence;
        let seen = got[2].confidence;
        assert!(told > seen && seen > gossiped,
            "told {told} should top witnessed {seen} over gossip {gossiped}");
    }

    #[test]
    fn test_the_memory_fades() {
        let mut w = World::new();
        let (knower, v) = (w.create_entity(), person(&mut w, &[]));
        record(&mut w, knower, v, "want:love".into(),
            Some(Reveal::Want(Want::Love)), Source::Told { speaker: v }, 0);
        let en = &entries(&w, knower, v)[0];
        assert_eq!(confidence_of(en, 0), 0.9);
        let old = confidence_of(en, 100);
        assert!(old < 0.9 && old > 0.0, "a year-old telling fades: {old}");
        // the newest hearing is the staleness clock's zero
        assert_eq!(last_heard(&w, knower, v), Some(0));
    }

    #[test]
    fn test_a_contradiction_flags_both_sides() {
        let mut w = World::new();
        let (knower, v, carrier) = (w.create_entity(), person(&mut w, &[]), person(&mut w, &[]));
        // the carrier says the debt's paid; the bailiff's roll says owed
        record(&mut w, knower, v, "debt:paid".into(), None,
            Source::Gossip { speaker: carrier }, 10);
        record(&mut w, knower, v, "!debt:paid".into(), None,
            Source::Witnessed, 20);
        let got = entries(&w, knower, v);
        assert_eq!(got.len(), 2);
        assert!(got[0].contradicted && got[1].contradicted,
            "a claim and its opposite both flag, neither erases");
        // same fact twice says nothing new - and keeps its first source
        record(&mut w, knower, v, "debt:paid".into(), None,
            Source::Witnessed, 30);
        let got = entries(&w, knower, v);
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].source, Source::Gossip { speaker: carrier });
    }

    #[test]
    fn test_known_still_reads_back_the_reveals() {
        let mut w = World::new();
        let (knower, v) = (w.create_entity(), person(&mut w, &[]));
        note(&mut w, knower, v, Reveal::Want(Want::Drink));
        note(&mut w, knower, v, Reveal::Problem(Problem::Taxed));
        record(&mut w, knower, v, "ghost:levy".into(), None,
            Source::Gossip { speaker: v }, 5);
        assert_eq!(known(&w, knower, v),
            vec![Reveal::Want(Want::Drink), Reveal::Problem(Problem::Taxed)],
            "only typed reveals come back - claim facts stay out of the pool");
        // seeded knowledge carries a telling's shape, before the days ran
        let en = &entries(&w, knower, v)[0];
        assert_eq!(en.source, Source::Told { speaker: v });
        assert_eq!(en.day, 0);
    }

    #[test]
    fn test_the_grief_clock_alone_doesnt_reopen_a_spent_probe() {
        let mut w2 = World::new();
        let v2 = person(&mut w2, &[]);
        let asker2 = w2.create_entity();
        w2.get_component_mut::<InnerLife>(v2).unwrap().wants = PROBE_WANTS.to_vec();
        housed(&mut w2, v2);
        for x in PROBE_WANTS.iter() { note(&mut w2, asker2, v2, Reveal::Want(*x)); }
        add_grievance(&mut w2, v2, Problem::Loss, 60);
        assert!(!probe_open(&w2, asker2, v2, 70), "the grief's run out");
    }

    #[test]
    fn test_an_errands_answer_confirms_what_was_carried() {
        // E5-7's done-when: the manor roll's answer turns hearsay into
        // a seen thing - confidence to the witnessed level, the day
        // refreshed, a contradiction settled
        let mut w = World::new();
        let (knower, v, carrier) = (person(&mut w, &[]), person(&mut w, &[]), person(&mut w, &[]));
        record(&mut w, knower, v, "ghost:miller_debt".into(), None,
            Source::Gossip { speaker: carrier }, 10);
        record(&mut w, knower, v, "!ghost:miller_debt".into(), None,
            Source::Gossip { speaker: carrier }, 20);
        // the contradiction stands before the answer
        assert!(entries(&w, knower, v).iter().all(|en| en.contradicted));
        confirm(&mut w, knower, v, "ghost:miller_debt", 60);
        let en = entries(&w, knower, v).into_iter()
            .find(|e| e.fact == "ghost:miller_debt").unwrap();
        assert_eq!(en.source, Source::Witnessed, "the roll was seen");
        assert_eq!(en.day, 60, "the sight refreshes the clock");
        assert!(en.confidence >= 0.9, "a seen thing sits high: {}", en.confidence);
        assert!(!en.contradicted, "the answer settles the doubt");
        // and what was never heard lands as witnessed, fresh
        confirm(&mut w, knower, v, "ghost:eviction", 60);
        let en2 = entries(&w, knower, v).into_iter()
            .find(|e| e.fact == "ghost:eviction").unwrap();
        assert_eq!(en2.source, Source::Witnessed);
    }

    #[test]
    fn test_a_telling_binds_the_seat_to_keep_it() {
        // E5-5 done-when: what they tell you themselves lands as an
        // obligation; hearsay about them carries none
        let mut w = World::new();
        let org = w.create_entity();
        w.add_component(org, Organizer::default());
        let (v, carrier) = (person(&mut w, &[]), person(&mut w, &[]));
        let r = Reveal::Problem(Problem::Taxed);
        hear(&mut w, org, v, r, 10);
        let held = confidences(&w, org, v);
        assert_eq!(held.len(), 1, "the telling makes the seat a co-owner");
        assert!(held[0].kept);
        assert_eq!(held[0].rule, Rule::TellNoOne);
        assert_eq!(held[0].fact, "problem:taxed");
        // gossip about another binds nothing
        hear_via(&mut w, org, carrier, r, 10,
            Source::Gossip { speaker: v });
        assert!(confidences(&w, org, carrier).is_empty(),
            "hearsay carries no obligation");
        // and the named bound lands: "not the collector"
        restrict(&mut w, org, v, "problem:taxed", carrier);
        let held = confidences(&w, org, v);
        assert_eq!(held[0].rule, Rule::NotTo(carrier));
    }

    #[test]
    fn test_a_leak_costs_the_owner_and_their_circle() {
        use crate::game::relationships::{find_relationship,
            Interaction, InteractionKind, RelKind, Relationship};
        let mut w = World::new();
        let (knower, owner, kin, neighbor) =
            (w.create_entity(), person(&mut w, &[]), person(&mut w, &[]), person(&mut w, &[]));
        // warm bonds to spend down: owner + circle all think well of the seat
        for e in [owner, kin, neighbor] {
            crate::game::relationships::record_interaction(&mut w, e,
                Interaction { other: knower, kind: InteractionKind::Aid, intensity: 0.8 });
        }
        // the circle: a friend-bond and a shared roof
        let bond = w.create_entity();
        w.add_component(bond, Relationship {
            from: owner, other: kin, kind: RelKind::Friend,
            ..Default::default()
        });
        let roof = w.create_entity();
        w.add_component(roof, Building {
            kind: BuildingKind::Home, name: "a house".into(),
            occupants: vec![owner, neighbor], worker: None, overseer: None,
            stock: Default::default(), capacity: 20, flyer: None,
        });
        let told = Reveal::Problem(Problem::WritBorne);
        record(&mut w, knower, owner, reveal_key(&told), Some(told),
            Source::Told { speaker: owner }, 5);
        let integ = |w: &World, e: Entity| find_relationship(w, e, knower)
            .and_then(|b| w.get_component::<Relationship>(b).map(|r| r.trust.integrity))
            .unwrap_or(0.0);
        let (o0, k0, n0) = (integ(&w, owner), integ(&w, kin), integ(&w, neighbor));
        assert!(leak(&mut w, knower, owner, "problem:writ_borne"));
        assert!(!confidences(&w, knower, owner)[0].kept, "the word's out");
        let (o1, k1, n1) = (integ(&w, owner), integ(&w, kin), integ(&w, neighbor));
        assert!(o1 < o0, "the owner's read of your integrity drops: {o0} -> {o1}");
        assert!(k1 < k0 && n1 < n0, "the circle cools too: {k0}->{k1}, {n0}->{n1}");
        assert!(k1 > o1 && n1 > o1, "but not as deep as the owner's");
        // nothing held, nothing breaks - a leak needs a living confidence
        let stranger = person(&mut w, &[]);
        assert!(!leak(&mut w, knower, stranger, "problem:broke"));
    }

    #[test]
    fn test_the_kept_word_pays_its_keeping() {
        use crate::game::relationships::{find_relationship,
            Interaction, InteractionKind, Relationship};
        let mut w = World::new();
        let (knower, owner) = (w.create_entity(), person(&mut w, &[]));
        crate::game::relationships::record_interaction(&mut w, owner,
            Interaction { other: knower, kind: InteractionKind::Aid, intensity: 0.5 });
        let integ = |w: &World| find_relationship(w, owner, knower)
            .and_then(|b| w.get_component::<Relationship>(b).map(|r| r.trust.integrity))
            .unwrap_or(0.0);
        let told = Reveal::Problem(Problem::Taxed);
        record(&mut w, knower, owner, reveal_key(&told), Some(told),
            Source::Told { speaker: owner }, 5);
        // not yet stood its time
        assert_eq!(honor_confidences(&mut w, knower, 5 + CONFIDENCE_MARK_DAYS - 1), 0);
        let before = integ(&w);
        assert_eq!(honor_confidences(&mut w, knower, 5 + CONFIDENCE_MARK_DAYS), 1);
        let after = integ(&w);
        assert!(after > before, "the keeping pays: {before} -> {after}");
        // pays once - not a pension
        assert_eq!(honor_confidences(&mut w, knower, 999), 0);
        assert_eq!(integ(&w), after);
        // and a broken one never pays
        let other = person(&mut w, &[]);
        let t2 = Reveal::Problem(Problem::Broke);
        record(&mut w, knower, other, reveal_key(&t2), Some(t2),
            Source::Told { speaker: other }, 5);
        leak(&mut w, knower, other, "problem:broke");
        assert_eq!(honor_confidences(&mut w, knower, 999), 0);
    }

    #[test]
    fn test_a_plain_person_fact_pays_no_kept_word_reward() {
        // #1721: learn_in_talk's person:* facts (trade, kin, trait,
        // hook) carry no Reveal - they're firsthand knowledge, not a
        // secret entrusted to keep, so they mint no confidence to honor
        let mut w = World::new();
        let (knower, owner) = (w.create_entity(), person(&mut w, &[]));
        record(&mut w, knower, owner, "person:craft:Weaver".to_string(), None,
            Source::Told { speaker: owner }, 5);
        assert!(confidences(&w, knower, owner).is_empty(),
            "a trade learned firsthand is no confidence");
        assert_eq!(honor_confidences(&mut w, knower, 5 + CONFIDENCE_MARK_DAYS), 0,
            "nothing was confided, so nothing pays a kept-word reward");
    }

    #[test]
    fn test_first_kin_reads_the_bond_not_the_bond_entity() {
        use crate::game::relationships::{RelKind, Relationship};
        let mut w = World::new();
        let (v, kin, housemate) =
            (person(&mut w, &[]), person(&mut w, &[]), person(&mut w, &[]));
        // a housemate who isn't kin would win by the fallback alone -
        // the Family bond must beat it even though the kin lives elsewhere
        let b = w.create_entity();
        w.add_component(b, Building {
            kind: BuildingKind::Home, name: "a house".into(),
            occupants: vec![v, housemate], worker: None, overseer: None,
            stock: Default::default(), capacity: 20, flyer: None,
        });
        let bond = w.create_entity();
        w.add_component(bond, Relationship {
            from: v, other: kin, kind: RelKind::Family, ..Default::default() });
        assert_eq!(first_kin(&w, v), Some(kin),
            "the family bond names the kin themselves, not the bond entity or a housemate");
    }

    #[test]
    fn test_turned_asides_count_into_a_shape() {
        let mut w = World::new();
        let (knower, who) = (w.create_entity(), person(&mut w, &[]));
        assert_eq!(note_evasion(&mut w, knower, who, "their dead", 5), 1);
        assert_eq!(note_evasion(&mut w, knower, who, "their dead", 6), 2);
        // a shape draws at three (E5-2 done-when)
        assert_eq!(note_evasion(&mut w, knower, who, "their dead", 7), SILHOUETTE_AT);
        assert_eq!(evasions_of(&w, knower, who, "their dead"), 3);
        // another area counts its own ledger
        assert_eq!(note_evasion(&mut w, knower, who, "the lord", 7), 1);
        assert_eq!(evasions_of(&w, knower, who, "the lord"), 1);
        // the last day moves with the latest turn-aside
        let s = &w.get_component::<Known>(knower).unwrap().silhouettes;
        assert_eq!(s.iter().find(|x| x.area == "their dead").unwrap().last_day, 7);
    }

    #[test]
    fn test_a_turn_aside_on_another_villager_is_theirs_not_yours() {
        let mut w = World::new();
        let (knower, a, b) = (w.create_entity(), person(&mut w, &[]), person(&mut w, &[]));
        note_evasion(&mut w, knower, a, "the lord", 3);
        assert_eq!(evasions_of(&w, knower, b, "the lord"), 0);
        assert_eq!(evasions_of(&w, a, a, "the lord"), 0);
    }
}

/// Lift a grievance: the load is answered, not just waited out - a
/// grievance that stops counting right now (a grant, a fix, a fight won)
pub fn lift_grievance(world: &mut World, e: Entity, p: Problem) {
    if let Some(life) = world.get_component_mut::<InnerLife>(e) {
        life.grievances.retain(|(g, _)| *g != p);
    }
}

/// Days a want the probing question planted takes to ripen into one
/// they carry (#1267)
pub const PLANT_RIPEN_DAYS: u64 = 10;

/// What "what do you actually want?" can put in a head that didn't
/// carry it before
const PROBE_WANTS: [Want; 5] = [Want::Security, Want::Mastery, Want::Love, Want::Finery, Want::BetterTools];

/// Wants a probing question put in someone's head (#1267): (the want,
/// the day it ripens, who asked). Till then it's only a thought - it
/// can't be drawn out; once ripe it's a want like any other
#[derive(Debug, Default)]
pub struct Planted {
    pub wants: Vec<(Want, u64, Entity)>,
}

/// Whether the probing question could still open something (#1269,
/// #1347): a want they neither carry nor already have ripening - or,
/// once those are all spoken for, a burden their life's taken on that
/// you've not yet heard. The world moves: a fresh grievance re-opens
/// the ask
pub fn probe_open(world: &World, knower: Entity, e: Entity, tick: u64) -> bool {
    if is_kid(world, e) { return false; }
    let carried = world.get_component::<InnerLife>(e).map(|l| l.wants.clone()).unwrap_or_default();
    let planted: Vec<Want> = world.get_component::<Planted>(e)
        .map(|p| p.wants.iter().map(|(w, _, _)| *w).collect()).unwrap_or_default();
    PROBE_WANTS.iter().any(|w| !carried.contains(w) && !planted.contains(w))
        || !unrevealed(world, knower, e, tick).is_empty()
}

/// Plant a want the hearer doesn't already carry, ripening in
/// PLANT_RIPEN_DAYS. `seed` picks which; None when there's nothing new
/// to plant (or a child heard it)
pub fn plant_want(world: &mut World, e: Entity, asker: Entity, tick: u64, seed: u64) -> Option<Want> {
    if is_kid(world, e) { return None; }
    let carried = world.get_component::<InnerLife>(e)?.wants.clone();
    let planted: Vec<Want> = world.get_component::<Planted>(e)
        .map(|p| p.wants.iter().map(|(w, _, _)| *w).collect()).unwrap_or_default();
    let open: Vec<Want> = PROBE_WANTS.iter().copied()
        .filter(|w| !carried.contains(w) && !planted.contains(w)).collect();
    if open.is_empty() { return None; }
    let w = open[(seed % open.len() as u64) as usize];
    if world.get_component::<Planted>(e).is_none() { world.add_component(e, Planted::default()); }
    world.get_component_mut::<Planted>(e).unwrap().wants.push((w, tick + PLANT_RIPEN_DAYS, asker));
    // the question's answer is itself a telling (#1625): they said the
    // want out loud, so the asker's ledger holds it from that day -
    // ripening only decides when it's a want they carry, not when
    // the seat got to hear of it
    let r = Reveal::Want(w);
    record(world, asker, e, reveal_key(&r), Some(r), Source::Told { speaker: e }, tick);
    Some(w)
}

/// The planted wants whose day has come: each becomes a want they
/// carry. Returns (want, who asked) for each that ripened
pub fn ripen_planted(world: &mut World, e: Entity, tick: u64) -> Vec<(Want, Entity)> {
    let Some(p) = world.get_component_mut::<Planted>(e) else { return vec![] };
    let (ripe, rest): (Vec<_>, Vec<_>) = p.wants.drain(..).partition(|(_, day, _)| tick >= *day);
    p.wants = rest;
    if ripe.is_empty() { return vec![]; }
    if let Some(life) = world.get_component_mut::<InnerLife>(e) {
        for (w, _, _) in &ripe {
            if !life.wants.contains(w) { life.wants.push(*w); }
        }
    }
    let asker = ripe[0].2;
    world.add_component(e, Thawed { asker, tick });
    ripe.into_iter().map(|(w, _, a)| (w, a)).collect()
}

/// The probe got through (#1269): the want they carry now is one the
/// question put there - their interest is connected to what they love
#[derive(Debug, Clone, Copy)]
pub struct Thawed {
    pub asker: Entity,
    pub tick: u64,
}

/// How far along the road out of helplessness they stand (#1269): the
/// gradient the talk reads - the same want lands as resignation until
/// it's connected to something they love
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThawRung {
    /// Un-activated: the issue's weather to them, not a thing to move
    Helpless,
    /// The question's taken root - a planted want ripening or ripened
    Thawing,
    /// Signed on: the interest's a cause now
    Joined,
    /// Acting: holds work in the cause - a seat of their own
    Acting,
}

/// Where a villager stands on the thaw gradient (#1269): derived live -
/// a member who falls back out re-chills rung by rung
pub fn rung_of(world: &World, e: Entity) -> ThawRung {
    let member = world.query::<crate::sim::community::Organization>().iter().any(|&o|
        world.get_component::<crate::sim::community::Organization>(o)
            .map(|org| org.members.contains(&e)).unwrap_or(false));
    if member {
        // acting's doing: a delegated seat, or work promised on the
        // meeting plan - the same "taken on work" read `stage_of` makes
        let holds = world.query::<crate::game::campaign::Campaign>().iter().any(|&c|
            world.get_component::<crate::game::campaign::Campaign>(c)
                .map(|cc| cc.seats.iter().any(|s| s.holder == e)).unwrap_or(false))
            || world.query::<crate::game::organizing::MeetingPlan>().iter().any(|&p|
                world.get_component::<crate::game::organizing::MeetingPlan>(p)
                    .map(|pp| pp.holders().contains(&e)).unwrap_or(false));
        return if holds { ThawRung::Acting } else { ThawRung::Joined };
    }
    let thawing = world.get_component::<Planted>(e).map(|p| !p.wants.is_empty()).unwrap_or(false)
        || world.get_component::<Thawed>(e).is_some();
    if thawing { ThawRung::Thawing } else { ThawRung::Helpless }
}

/// The status word the talk reads the rung by (#1269)
pub fn thaw_status(rung: ThawRung) -> &'static str {
    match rung {
        ThawRung::Helpless => "thaw_helpless",
        ThawRung::Thawing => "thaw_thawing",
        ThawRung::Joined => "thaw_joined",
        ThawRung::Acting => "thaw_acting",
    }
}
