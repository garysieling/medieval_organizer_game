//! The organizer's own line (#1236): what each villager has heard you
//! keep saying hardens into how they read you - the patient one, the
//! firebrand, the merciful hand, the one who holds folk to account.
//! It's per villager: a pick only reaches the people who witnessed it,
//! and each keeps their own read. Opposite lines cost each other - take
//! the other side often enough and they'll name the turn.
//!
//! Multiplayer keeps a separate read per organizer (`by_org`, keyed on
//! the seat's organization entity, like `player::Rapport::by_player`):
//! one seat's firebrand reputation with a villager is theirs alone, and
//! doesn't land on another seat's talk.
use crate::engine::det::HashMap;
use crate::engine::ecs::{Entity, World};
use crate::game::traits::{Personality, Trait};

/// Witnessed weight at which a line hardens into their read of you
pub const HARDEN: f32 = 3.0;
/// A hardened line softens back below this (the page can turn)
const SOFTENS: f32 = HARDEN * 0.5;
/// Daily keep per line - unrepeated, most lines fade over a season or
/// so. `Exacting`'s only production evidence is the burdened man's
/// dilemma, which recurs every 300-450 days (`EventId::Dilemma`); at
/// the general rate it would decay to nothing between visits and could
/// never harden, so it keeps far more of its weight day to day.
const FADE: [f32; 4] = [0.98, 0.98, 0.98, 0.9995];
/// What a contradicted hardened line keeps
const TURN_KEEPS: f32 = 0.5;
/// A member leaning with the line gets this much more of each witness -
/// the faithful harden faster (#1310)
const WITH_BONUS: f32 = 0.5;
/// A member not leaning with the line drifts toward its opposite this
/// much instead - the room splits around the organizer over time. At
/// this rate a fresh read hardens against after five opposed witnesses,
/// and the sixth is their last straw (#1310)
const AWAY_DRIFT: f32 = 0.6;

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

/// One villager's read of a single organizer's rhetoric
#[derive(Debug, Clone, Default)]
pub struct OrgRead {
    pub weight: [f32; 4],
    /// The day each line hardened, while it holds
    pub hardened: [Option<u64>; 4],
    /// A hardened line that organizer went back on, and the day it had
    /// hardened - waiting to be named at their next talk
    pub turn: Option<(Line, u64)>,
}

/// One villager's read of the organizers' rhetoric, kept separately per
/// seat's organization entity so one organizer's line can't be inherited
/// or spent by another's talk
#[derive(Debug, Clone, Default)]
pub struct RhetoricMemory {
    pub by_org: HashMap<Entity, OrgRead>,
}

/// `v` saw `org` take `line`
pub fn witness(world: &mut World, v: Entity, org: Entity, line: Line, tick: u64) {
    let mut m = world.get_component::<RhetoricMemory>(v).cloned().unwrap_or_default();
    let r = m.by_org.entry(org).or_default();
    let opp = line.opposite().idx();
    if let Some(since) = r.hardened[opp] {
        r.weight[opp] *= TURN_KEEPS;
        if r.weight[opp] < HARDEN { r.hardened[opp] = None; }
        if r.turn.is_none() { r.turn = Some((line.opposite(), since)); }
    }
    let i = line.idx();
    // a member's read is their lean (#1310): sworn folk polarize - the
    // already-won harden faster, the rest drift to the other line, so
    // the room splits around the line the organizer keeps taking
    if member_of(world, v, org) {
        if r.weight[i] >= r.weight[opp] {
            r.weight[i] += 1.0 + WITH_BONUS;
        } else {
            r.weight[opp] += AWAY_DRIFT;
        }
    } else {
        r.weight[i] += 1.0;
    }
    for j in 0..LINES.len() {
        if r.weight[j] >= HARDEN && r.hardened[j].is_none() { r.hardened[j] = Some(tick); }
    }
    world.add_component(v, m);
}

/// Whether `v` is sworn to `org` - their read of the organizer is also
/// their lean inside the cause, and it can walk them out the door
fn member_of(world: &World, v: Entity, org: Entity) -> bool {
    world.get_component::<crate::sim::community::Organization>(org)
        .map(|o| o.members.contains(&v)).unwrap_or(false)
}

/// The line `v` is sworn-set against `org` on: their hardened read sits
/// on `line`'s opposite - the defection trigger when `line` gets taken
/// again in their hearing (#1310)
pub fn set_against(world: &World, v: Entity, org: Entity, line: Line) -> bool {
    world.get_component::<RhetoricMemory>(v)
        .and_then(|m| m.by_org.get(&org))
        .map(|r| r.hardened[line.opposite().idx()].is_some())
        .unwrap_or_default()
}

pub fn witness_all(world: &mut World, who: &[Entity], org: Entity, line: Line, tick: u64) {
    for &v in who { witness(world, v, org, line, tick); }
}

/// The lines `v` reads `org` by
pub fn hardened(world: &World, v: Entity, org: Entity) -> Vec<Line> {
    world.get_component::<RhetoricMemory>(v)
        .and_then(|m| m.by_org.get(&org))
        .map(|r| LINES.iter().copied().filter(|l| r.hardened[l.idx()].is_some()).collect())
        .unwrap_or_default()
}

/// The day's fade: unrepeated lines soften back, for every organizer
/// `v` has a read on
pub fn fade(world: &mut World, v: Entity) {
    if let Some(m) = world.get_component_mut::<RhetoricMemory>(v) {
        for r in m.by_org.values_mut() {
            for i in 0..LINES.len() {
                r.weight[i] *= FADE[i];
                if r.weight[i] < SOFTENS { r.hardened[i] = None; }
            }
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
pub fn take_turn(world: &mut World, v: Entity, org: Entity) -> Option<String> {
    let m = world.get_component_mut::<RhetoricMemory>(v)?;
    let r = m.by_org.get_mut(&org)?;
    let (line, since) = r.turn.take()?;
    Some(format!("\"Thou {} in {} - and now this?\"", line.preached(), season_word(since)))
}

fn traits_of(world: &World, v: Entity) -> Vec<Trait> {
    world.get_component::<Personality>(v).map(|p| p.traits.clone()).unwrap_or_default()
}

/// How their read of `org` moves what they'll hear asked, a rung either
/// way: hotheads find the firebrand and the gentle shy off; the patient
/// one gets the pious and kind folk's deeper asks and the hothead's
/// shrug
pub fn ask_shift(world: &World, v: Entity, org: Entity) -> i8 {
    let t = traits_of(world, v);
    let has = |x: Trait| t.contains(&x);
    let mut s = 0i8;
    for l in hardened(world, v, org) {
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

/// The register they open a talk with `org` in, by the line they read
/// that organizer by
pub fn register_note(world: &World, v: Entity, org: Entity, name: &str) -> Option<String> {
    let l = *hardened(world, v, org).first()?;
    Some(match l {
        Line::Firebrand => format!("{name} eyes you warily - the firebrand, come to stir something."),
        Line::Patient => format!("{name} settles in - the patient one; you'll hear them out."),
        Line::Merciful => format!("{name} eases - word is you spare folk."),
        Line::Exacting => format!("{name} straightens - word is you hold folk to account."),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::community::Organization;

    fn org(w: &mut World, members: &[Entity]) -> Entity {
        let o = w.create_entity();
        w.add_component(o, Organization {
            name: "The League".into(), members: members.to_vec(), leader: members[0],
            influence_pool: 0.0, resources: 0.0, charter: "".into(), successors: vec![] });
        o
    }

    #[test]
    fn test_the_sworn_harden_faster_than_the_street() {
        let mut w = World::new();
        let m = w.create_entity();
        let stranger = w.create_entity();
        let o = org(&mut w, &[m]);
        witness(&mut w, m, o, Line::Firebrand, 10);
        witness(&mut w, m, o, Line::Firebrand, 11);
        witness(&mut w, stranger, o, Line::Firebrand, 10);
        witness(&mut w, stranger, o, Line::Firebrand, 11);
        // two tellings set the member's read; the unsworn need a third
        assert!(hardened(&w, m, o).contains(&Line::Firebrand));
        assert!(!hardened(&w, stranger, o).contains(&Line::Firebrand));
    }

    #[test]
    fn test_the_member_set_against_drifts_further_out() {
        let mut w = World::new();
        let m = w.create_entity();
        let o = org(&mut w, &[m]);
        // their read leans patient: each firebrand pick they hear only
        // feeds the opposite, the firebrand itself never lands
        witness(&mut w, m, o, Line::Patient, 10);
        witness(&mut w, m, o, Line::Firebrand, 11);
        let r = &w.get_component::<RhetoricMemory>(m).unwrap().by_org[&o];
        assert_eq!(r.weight[Line::Firebrand as usize], 0.0,
            "the opposed line should never accrue on a set member");
        assert!(r.weight[Line::Patient as usize] > 1.0,
            "the drift should feed their lean: {:?}", r.weight);
        // and set_against names the member whose drift fully hardens
        assert!(!set_against(&w, m, o, Line::Firebrand));
        for i in 0..2 { witness(&mut w, m, o, Line::Firebrand, 12 + i); }
        assert!(set_against(&w, m, o, Line::Firebrand),
            "three opposed tellings on a leaning read set them against it");
    }
}
