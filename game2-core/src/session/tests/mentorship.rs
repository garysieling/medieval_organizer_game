//! Mentorship (#1239): a long bond with an older soul bends a young
//! one's temperament, a season at a time.
use crate::session::*;
use crate::engine::ecs::Entity;
use crate::game::lifecycle::{Age, LifeStage};
use crate::game::mentorship::{self as m, Teacher};
use crate::game::relationships::{add_relationship, bonds_of, RelKind, Relationship};
use crate::game::traits::{self, Personality, Trait};
use crate::game::villager::{self, Pos};
use crate::sim::community::Organization;

fn living(g: &Game) -> Vec<Entity> {
    g.villagers.iter().copied().filter(|&v| !villager::is_dead(&g.world, v)).collect()
}

/// `v` at `years`, carrying `traits`, with every bond they had cut
fn person(g: &mut Game, v: Entity, years: f32, stage: LifeStage, traits: Vec<Trait>) {
    for b in bonds_of(&g.world, v) { g.world.remove_component::<Relationship>(b); }
    let a = g.world.get_component_mut::<Age>(v).unwrap();
    a.years = years;
    a.stage = stage;
    g.world.add_component(v, Personality { traits });
}

fn bond(g: &mut Game, a: Entity, b: Entity, kind: RelKind, strength: f32, interactions: u32) {
    add_relationship(&mut g.world, Relationship { from: a, other: b, kind, strength, interactions, ..Default::default() });
}

fn seasons(g: &mut Game, n: u64) {
    for k in 1..=n { g.mentorship_tick(k * m::SEASON); }
}

fn traits_of(g: &Game, v: Entity) -> Vec<Trait> {
    g.world.get_component::<Personality>(v).unwrap().traits.clone()
}

fn taught(g: &Game, v: Entity) -> Vec<m::Mark> {
    g.world.get_component::<m::Taught>(v).map(|t| t.marks.clone()).unwrap_or_default()
}

/// A master and an apprentice: the pupil at 14, the master at 40
fn apprenticed(g: &mut Game, master: Vec<Trait>, pupil: Vec<Trait>) -> (Entity, Entity) {
    let vs = living(g);
    let (t, p) = (vs[0], vs[1]);
    person(g, t, 40.0, LifeStage::Adult, master);
    person(g, p, 14.0, LifeStage::Adolescent, pupil);
    bond(g, t, p, RelKind::Mentor, 0.4, 3);
    (t, p)
}

#[test]
fn test_a_stolid_master_raises_a_steady_apprentice() {
    let mut g = Game::new();
    let (t, p) = apprenticed(&mut g, vec![Trait::Stolid], vec![Trait::Pious]);
    seasons(&mut g, 16);
    let tr = traits_of(&g, p);
    assert!(tr.contains(&Trait::Steady), "a stolid master's pupil grows steady: {tr:?}");
    assert!(!tr.contains(&Trait::Stolid), "not a copy: {tr:?}");
    assert_eq!(taught(&g, p)[0].by, Teacher::Person(t));
}

#[test]
fn test_a_cruel_master_leaves_the_pupil_wary() {
    let mut g = Game::new();
    let (_, p) = apprenticed(&mut g, vec![Trait::Cruel], vec![Trait::Pious]);
    seasons(&mut g, 16);
    let tr = traits_of(&g, p);
    assert!(tr.contains(&Trait::Wary) && !tr.contains(&Trait::Cruel), "{tr:?}");
}

#[test]
fn test_any_long_bond_teaches_and_drives_out_the_opposite_then_is_named_in_talk() {
    let mut g = Game::new();
    let vs = living(&g);
    let (t, p) = (vs[0], vs[1]);
    person(&mut g, t, 45.0, LifeStage::Adult, vec![Trait::Steady]);
    person(&mut g, p, 12.0, LifeStage::Child, vec![Trait::Hotheaded, Trait::Pious]);
    // an uncle the kid shadows - no apprenticeship, just a long warm tie
    bond(&mut g, t, p, RelKind::Friend, 0.8, 20);
    seasons(&mut g, 24);
    assert_eq!(traits_of(&g, p), vec![Trait::Steady, Trait::Pious], "the hurry's gone, in its place");
    let mk = taught(&g, p)[0];
    assert_eq!(mk.drove_out, Some(Trait::Hotheaded));
    assert!(m::saying(mk, "old wat").starts_with("Old wat beat the hurry out of me"));

    // the next talk names who taught them, once
    let i = g.villagers.iter().position(|&x| x == p).unwrap();
    let at = *g.world.get_component::<Pos>(p).unwrap();
    *g.world.get_component_mut::<Pos>(g.player).unwrap() = Pos { x: at.x + 0.5, y: at.y };
    g.seats[0].selected_target = i;
    g.seats[0].picked = true;
    g.act_at(0, 0);
    let notes = g.seats[0].talk.as_ref().expect("talking").notes.clone();
    let named = format!("{} beat the hurry out of me", g.name_of(t));
    assert!(notes.iter().any(|n| n.contains(&named)), "{notes:?}");
    assert!(taught(&g, p)[0].told);
    assert!(m::untold(&g.world, p).is_none(), "named the once");
}

#[test]
fn test_peers_thin_bonds_and_set_adults_teach_or_learn_nothing() {
    let mut g = Game::new();
    let vs = living(&g);
    let (peer, thin, pupil, grown, master) = (vs[0], vs[1], vs[2], vs[3], vs[4]);
    person(&mut g, peer, 18.0, LifeStage::Adult, vec![Trait::Kind]);
    person(&mut g, thin, 50.0, LifeStage::Adult, vec![Trait::Pious]);
    person(&mut g, pupil, 14.0, LifeStage::Adolescent, vec![Trait::Lazy]);
    person(&mut g, master, 60.0, LifeStage::Elder, vec![Trait::Ambitious]);
    person(&mut g, grown, 40.0, LifeStage::Adult, vec![Trait::Stingy]);
    bond(&mut g, peer, pupil, RelKind::Mentor, 0.9, 30);   // too near in age
    bond(&mut g, thin, pupil, RelKind::Friend, 0.3, 30);   // too slight a tie
    bond(&mut g, master, grown, RelKind::Mentor, 0.9, 30); // set in their ways
    seasons(&mut g, 40);
    assert!(taught(&g, pupil).is_empty() && taught(&g, grown).is_empty());
    assert_eq!(traits_of(&g, pupil), vec![Trait::Lazy]);
}

#[test]
fn test_a_life_takes_at_most_three_marks() {
    let mut g = Game::new();
    let (_, p) = apprenticed(&mut g,
        vec![Trait::Kind, Trait::Pious, Trait::Proud, Trait::Gregarious, Trait::Stingy], vec![]);
    seasons(&mut g, 80);
    assert_eq!(taught(&g, p).len(), m::MAX_MARKS);
    assert_eq!(traits_of(&g, p).len(), m::MAX_MARKS);
}

/// The org's members all share `t`
fn shaped(g: &mut Game, members: &[Entity], t: Trait) {
    for &mm in members {
        person(g, mm, 35.0, LifeStage::Adult, vec![t]);
        g.world.get_component_mut::<Organization>(g.org).unwrap().members.push(mm);
    }
}

#[test]
fn test_a_members_child_grows_into_the_causes_shape() {
    let mut g = Game::new();
    let vs = living(&g);
    shaped(&mut g, &vs[0..3], Trait::Kind);
    let kid = vs[3];
    person(&mut g, kid, 9.0, LifeStage::Child, vec![Trait::Stingy]);
    // their mother's in the cause - but too thin a tie to teach on its own
    bond(&mut g, vs[0], kid, RelKind::Family, 0.2, 1);
    assert_eq!(m::cause_shape(&g.world, g.org), Some(Trait::Kind));
    seasons(&mut g, 24);
    assert!(traits_of(&g, kid).contains(&Trait::Kind), "{:?}", traits_of(&g, kid));
    assert_eq!(taught(&g, kid)[0].by, Teacher::Cause(g.org));
}

#[test]
fn test_the_organizers_counsel_raises_a_young_follower() {
    let mut g = Game::new();
    let vs = living(&g);
    shaped(&mut g, &vs[0..3], Trait::Pious);
    let (young, stranger) = (vs[3], vs[4]);
    person(&mut g, young, 20.0, LifeStage::Adult, vec![Trait::Lazy]);
    person(&mut g, stranger, 20.0, LifeStage::Adult, vec![Trait::Lazy]);
    let pe = g.player;
    bond(&mut g, pe, young, RelKind::Friend, 0.95, 12);
    assert_eq!(crate::game::organizing::stage_of(&g.world, pe, young),
        crate::game::organizing::RelStage::Follower);
    seasons(&mut g, 24);
    assert!(traits_of(&g, young).contains(&Trait::Pious), "{:?}", traits_of(&g, young));
    assert!(taught(&g, stranger).is_empty(), "nobody counselled them");
}

#[test]
fn test_new_temperaments_roll_beside_the_old_ones() {
    let old = [Trait::Lazy, Trait::Ambitious, Trait::Kind, Trait::Stingy, Trait::Pious,
        Trait::Hotheaded, Trait::Gregarious, Trait::Proud];
    let mut extra = 0;
    for seed in 0..400u64 {
        let p = traits::personality_from_seed(seed);
        let n = ((seed % 3) + 1) as usize;
        let first: Vec<Trait> = old.iter().cycle().skip((seed % 8) as usize).take(n).copied().collect();
        assert_eq!(p.traits[..n], first[..], "seed {seed}: the first eight roll as before");
        if p.traits.len() > n {
            extra += 1;
            let t = p.traits[n];
            assert!(t.opposite().map(|o| !first.contains(&o)).unwrap_or(true), "no opposite pair");
        }
    }
    assert!((40..160).contains(&extra), "about one in five carries a new one: {extra}");
    let p = Personality { traits: vec![Trait::Steady, Trait::Cruel] };
    assert!(traits::modifies(&p, "feud") < 1.0 && traits::modifies(&p, "share") < 1.0);
}

#[test]
fn test_the_dead_neither_teach_nor_learn() {
    let mut g = Game::new();
    let (t, p) = apprenticed(&mut g, vec![Trait::Kind], vec![]);
    g.world.add_component(t, crate::game::villager::Dead);
    seasons(&mut g, 24);
    assert!(taught(&g, p).is_empty(), "a dead master's lessons are done");
    let mut g = Game::new();
    let (_, p) = apprenticed(&mut g, vec![Trait::Pious], vec![]);
    g.world.add_component(p, crate::game::villager::Dead);
    seasons(&mut g, 24);
    assert!(taught(&g, p).is_empty(), "nor does a dead pupil grow");
}
