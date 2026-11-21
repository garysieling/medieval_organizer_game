//! The organizer's own line (#1236): what each villager has heard you
//! keep saying hardens into how they read you - the patient one, the
//! firebrand, the merciful hand, the one who holds folk to account.
//! It's per villager: a pick only reaches the people who witnessed it,
//! and each keeps their own read. Opposite lines cost each other - take
//! the other side often enough and they'll name the turn.
use crate::engine::ecs::{Entity, World};
use crate::game::traits::{Personality, Trait};

/// Witnessed weight at which a line hardens into their read of you
pub const HARDEN: f32 = 3.0;
/// A hardened line softens back below this (the page can turn)
const SOFTENS: f32 = HARDEN * 0.5;
/// Daily keep - unrepeated, a line fades over a season or so
const FADE: f32 = 0.98;
/// What a contradicted hardened line keeps
const TURN_KEEPS: f32 = 0.5;

/// A rhetorical line; each has its opposite
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Line { Patient, Firebrand, Merciful, Exacting }

pub const LINES: [Line; 4] = [Line::Patient, Line::Firebrand, Line::Merciful, Line::Exacting];

impl Line {
    fn idx(self) -> usize { self as usize }
    pub fn opposite(self) -> Line {
        match self {
            Line::Patient => Line::Firebrand,
            Line::Firebrand => Line::Patient,
            Line::Merciful => Line::Exacting,
            Line::Exacting => Line::Merciful,
        }
    }
    /// The speaker status a hardened read carries (`other:status:firebrand`)
    pub fn status(self) -> &'static str {
        match self {
            Line::Patient => "patient",
            Line::Firebrand => "firebrand",
            Line::Merciful => "merciful",
            Line::Exacting => "exacting",
        }
    }
    fn preached(self) -> &'static str {
        match self {
            Line::Patient => "preached patience",
            Line::Firebrand => "called for fire",
            Line::Merciful => "spoke for mercy",
            Line::Exacting => "held folk to account",
        }
    }
}

/// One villager's read of the organizer's rhetoric
#[derive(Debug, Clone, Default)]
pub struct RhetoricMemory {
    pub weight: [f32; 4],
    /// The day each line hardened, while it holds
    pub hardened: [Option<u64>; 4],
    /// A hardened line you went back on, and the day it had hardened -
    /// waiting to be named at your next talk
    pub turn: Option<(Line, u64)>,
}

/// `v` saw you take `line`
pub fn witness(world: &mut World, v: Entity, line: Line, tick: u64) {
    let mut m = world.get_component::<RhetoricMemory>(v).cloned().unwrap_or_default();
    let opp = line.opposite().idx();
    if let Some(since) = m.hardened[opp] {
        m.weight[opp] *= TURN_KEEPS;
        if m.weight[opp] < HARDEN { m.hardened[opp] = None; }
        if m.turn.is_none() { m.turn = Some((line.opposite(), since)); }
    }
    let i = line.idx();
    m.weight[i] += 1.0;
    if m.weight[i] >= HARDEN && m.hardened[i].is_none() { m.hardened[i] = Some(tick); }
    world.add_component(v, m);
}

pub fn witness_all(world: &mut World, who: &[Entity], line: Line, tick: u64) {
    for &v in who { witness(world, v, line, tick); }
}

/// The lines `v` reads you by
pub fn hardened(world: &World, v: Entity) -> Vec<Line> {
    world.get_component::<RhetoricMemory>(v)
        .map(|m| LINES.iter().copied().filter(|l| m.hardened[l.idx()].is_some()).collect())
        .unwrap_or_default()
}

/// The day's fade: unrepeated lines soften back
pub fn fade(world: &mut World, v: Entity) {
    if let Some(m) = world.get_component_mut::<RhetoricMemory>(v) {
        for i in 0..LINES.len() {
            m.weight[i] *= FADE;
            if m.weight[i] < SOFTENS { m.hardened[i] = None; }
        }
    }
}

fn season_word(tick: u64) -> &'static str {
    match (tick % crate::game::desires::YEAR_DAYS) * 4 / crate::game::desires::YEAR_DAYS {
        0 => "spring",
        1 => "summer",
        2 => "autumn",
        _ => "winter",
    }
}

/// The turn, named once: "Thou preached patience in spring - and now this?"
pub fn take_turn(world: &mut World, v: Entity) -> Option<String> {
    let m = world.get_component_mut::<RhetoricMemory>(v)?;
    let (line, since) = m.turn.take()?;
    Some(format!("\"Thou {} in {} - and now this?\"", line.preached(), season_word(since)))
}

fn traits_of(world: &World, v: Entity) -> Vec<Trait> {
    world.get_component::<Personality>(v).map(|p| p.traits.clone()).unwrap_or_default()
}

/// How their read of you moves what they'll hear asked, a rung either
/// way: hotheads find the firebrand and the gentle shy off; the patient
/// one gets the pious and kind folk's deeper asks and the hothead's
/// shrug
pub fn ask_shift(world: &World, v: Entity) -> i8 {
    let t = traits_of(world, v);
    let has = |x: Trait| t.contains(&x);
    let mut s = 0i8;
    for l in hardened(world, v) {
        s += match l {
            Line::Firebrand if has(Trait::Hotheaded) => 1,
            Line::Firebrand if has(Trait::Kind) || has(Trait::Lazy) => -1,
            Line::Patient if has(Trait::Hotheaded) => -1,
            Line::Patient if has(Trait::Pious) || has(Trait::Kind) => 1,
            Line::Merciful if has(Trait::Kind) => 1,
            Line::Exacting if has(Trait::Kind) => -1,
            Line::Exacting if has(Trait::Ambitious) => 1,
            _ => 0,
        };
    }
    s.clamp(-1, 1)
}

/// The register they open a talk in, by the line they read you by
pub fn register_note(world: &World, v: Entity, name: &str) -> Option<String> {
    let l = *hardened(world, v).first()?;
    Some(match l {
        Line::Firebrand => format!("{name} eyes you warily - the firebrand, come to stir something."),
        Line::Patient => format!("{name} settles in - the patient one; you'll hear them out."),
        Line::Merciful => format!("{name} eases - word is you spare folk."),
        Line::Exacting => format!("{name} straightens - word is you hold folk to account."),
    })
}
