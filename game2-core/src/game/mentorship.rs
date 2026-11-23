//! Mentorship (#1239): a long, strong bond with an older soul bends the
//! young one's temperament, a season at a time - CK3's Influence Ward,
//! not a talk. A formal master marks an apprentice soonest, but any tie
//! that's held long enough teaches (a kid shadowing a craftsman uncle).
//! Some marks come softened: a stolid master's pupil grows steady, a
//! cruel one's grows wary. A taught trait drives out its opposite. The
//! cause raises its own too: a young follower the organizer counsels,
//! or a member's child raised around it, leans toward the cause's
//! shape - the trait its members most share. Each mark is remembered
//! (who, what, when) so the pupil can name it in talk.
use crate::engine::ecs::{Entity, World};
use crate::game::desires::YEAR_DAYS;
use crate::game::lifecycle::{Age, LifeStage};
use crate::game::relationships::{bonds_of, RelKind, Relationship};
use crate::game::traits::{Personality, Trait, TRAITS};

/// The teaching clock: one chance a season
pub const SEASON: u64 = YEAR_DAYS / 4;
/// How strong a formal master/apprentice tie must hold to teach
pub const FORMAL_TIE: f32 = 0.3;
/// How strong, and how long-worn, any other bond must be
pub const BOND_TIE: f32 = 0.5;
pub const BOND_TALKS: u32 = 10;
/// A teacher's this many years older than the pupil, at least
pub const AGE_GAP: f32 = 8.0;
/// Past this age an adult is set in their ways
pub const YOUNG_YEARS: f32 = 25.0;
/// Marks a life can take
pub const MAX_MARKS: usize = 3;
/// A season's odds, one in N: a formal master, another bond, the cause
pub const FORMAL_IN: u64 = 3;
pub const BOND_IN: u64 = 5;
pub const CAUSE_IN: u64 = 4;

/// Who left the mark
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Teacher {
    Person(Entity),
    /// The organization (its entity) whose meetings raised them
    Cause(Entity),
}

/// One trait taken from a teacher
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mark {
    pub by: Teacher,
    pub took: Trait,
    /// The opposite it drove out, if they had it
    pub drove_out: Option<Trait>,
    pub day: u64,
    /// Named in talk yet
    pub told: bool,
}

/// What a villager was taught, oldest first
#[derive(Debug, Clone, Default)]
pub struct Taught {
    pub marks: Vec<Mark>,
}

fn age_of(world: &World, v: Entity) -> Option<(f32, LifeStage)> {
    world.get_component::<Age>(v).map(|a| (a.years, a.stage))
}

fn traits_of(world: &World, v: Entity) -> Vec<Trait> {
    world.get_component::<Personality>(v).map(|p| p.traits.clone()).unwrap_or_default()
}

/// Still being shaped: a child, an adolescent, or a young adult
pub fn is_young(world: &World, v: Entity) -> bool {
    if crate::game::villager::is_dead(world, v) { return false; }
    match age_of(world, v) {
        Some((_, LifeStage::Child | LifeStage::Adolescent)) => true,
        Some((y, LifeStage::Adult)) => y < YOUNG_YEARS,
        _ => false,
    }
}

pub fn marks(world: &World, v: Entity) -> usize {
    world.get_component::<Taught>(v).map(|t| t.marks.len()).unwrap_or(0)
}

/// The older souls whose bond with `pupil` can teach, in bond order,
/// each flagged formal (a master/apprentice tie) or not
pub fn teachers(world: &World, pupil: Entity) -> Vec<(Entity, bool)> {
    let Some((py, _)) = age_of(world, pupil) else { return vec![] };
    bonds_of(world, pupil).into_iter().filter_map(|b| {
        let r = world.get_component::<Relationship>(b)?;
        let t = if r.from == pupil { r.other } else { r.from };
        let (ty, stage) = age_of(world, t)?;
        if t == pupil || stage == LifeStage::Dead || crate::game::villager::is_dead(world, t)
            || ty < py + AGE_GAP { return None; }
        let formal = matches!(r.kind, RelKind::Mentor | RelKind::Apprentice);
        let holds = if formal { r.strength >= FORMAL_TIE }
            else { r.strength >= BOND_TIE && r.interactions >= BOND_TALKS };
        holds.then_some((t, formal))
    }).collect()
}

/// The first of the teacher's traits, as a pupil takes it, that the
/// pupil hasn't already got
pub fn lesson(teacher: &[Trait], pupil: &[Trait]) -> Option<Trait> {
    teacher.iter().map(|t| t.taught()).find(|t| !pupil.contains(t))
}

/// The cause's shape: the trait its members most share (ties go to
/// the earlier trait)
pub fn cause_shape(world: &World, org: Entity) -> Option<Trait> {
    let members = world.get_component::<crate::sim::community::Organization>(org)
        .map(|o| o.members.clone()).unwrap_or_default();
    let mut best: Option<(Trait, usize)> = None;
    for t in TRAITS {
        let n = members.iter().filter(|&&m| traits_of(world, m).contains(&t)).count();
        if n > 0 && best.map(|(_, b)| n > b).unwrap_or(true) { best = Some((t, n)); }
    }
    best.map(|(t, _)| t)
}

/// Mark `pupil` with `took`: it drives out its opposite if they carry
/// it, else joins their traits. False if they already had it.
pub fn mark(world: &mut World, pupil: Entity, by: Teacher, took: Trait, day: u64) -> bool {
    let Some(p) = world.get_component::<Personality>(pupil) else { return false };
    if p.traits.contains(&took) { return false; }
    let drove_out = took.opposite().filter(|o| p.traits.contains(o));

    // A seeded pupil has a persona behind both views (#1412) - nudge the
    // axes the pass-3 table reads and re-derive both, so the taught
    // trait (and the opposite it drives out) land on the Big Five view
    // too. A fixed-cast pupil has no persona: fall back to mutating the
    // trait list directly, same as before #1412.
    if let Some(persona) = world.get_component_mut::<crate::game::persona::Persona>(pupil) {
        persona.teach(took);
    }
    if let Some(persona) = world.get_component::<crate::game::persona::Persona>(pupil).cloned() {
        let (big5, traits) = persona.personalities();
        world.add_component(pupil, big5);
        world.add_component(pupil, traits);
    } else {
        let Some(p) = world.get_component_mut::<Personality>(pupil) else { return false };
        match drove_out.and_then(|o| p.traits.iter().position(|&t| t == o)) {
            Some(i) => p.traits[i] = took,
            None => p.traits.push(took),
        }
    }
    let mut taught = world.get_component::<Taught>(pupil).cloned().unwrap_or_default();
    taught.marks.push(Mark { by, took, drove_out, day, told: false });
    world.add_component(pupil, taught);
    true
}

/// A season's one-in-`one_in` chance for `v` under `salt`: hashed off
/// the day and who, so it draws nothing from the shared streams
pub fn roll(day: u64, v: Entity, salt: u64, one_in: u64) -> bool {
    let mut z = day ^ ((v as u64) << 20) ^ salt.wrapping_mul(0xD6E8_FEB8_6659_FD93);
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    (z ^ (z >> 31)) % one_in == 0
}

/// The oldest mark they haven't named yet
pub fn untold(world: &World, v: Entity) -> Option<Mark> {
    world.get_component::<Taught>(v)?.marks.iter().find(|m| !m.told).copied()
}

/// Spend that mark: it's been named
pub fn tell(world: &mut World, v: Entity) {
    if let Some(m) = world.get_component_mut::<Taught>(v)
        .and_then(|t| t.marks.iter_mut().find(|m| !m.told)) {
        m.told = true;
    }
}

/// How the pupil names the mark, `who` being the teacher as said
/// aloud: "Old Wat beat the hurry out of me."
pub fn saying(m: Mark, who: &str) -> String {
    let what = match (m.took, m.drove_out) {
        (Trait::Steady, Some(Trait::Hotheaded)) => "beat the hurry out of me",
        (Trait::Kind, Some(Trait::Cruel)) => "took the hardness out of my hands",
        (Trait::Ambitious, Some(Trait::Lazy)) => "shook the idle out of me",
        (Trait::Gregarious, Some(Trait::Stolid)) => "drew me out of my shell",
        (Trait::Steady, _) => "taught me to take my time",
        (Trait::Wary, _) => "taught me to watch who's asking",
        (Trait::Kind, _) => "showed me the loaf goes out first",
        (Trait::Ambitious, _) => "told me I'd be more than I was born",
        (Trait::Lazy, _) => "taught me the clouds are worth the watching",
        (Trait::Stingy, _) => "taught me to count every penny twice",
        (Trait::Pious, _) => "brought me to the altar",
        (Trait::Hotheaded, _) => "taught me never to swallow an insult",
        (Trait::Gregarious, _) => "taught me every soul's name",
        (Trait::Proud, _) => "taught me to bow to no one",
        (Trait::SelfAbsorbed, _) => "taught me the floor's worth holding",
        (Trait::Stolid, _) => "taught me to keep my own counsel",
        (Trait::Cruel, _) => "taught me a hard hand gets things done",
    };
    let mut c = who.chars();
    let who = c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default();
    format!("{who} {what}.")
}
