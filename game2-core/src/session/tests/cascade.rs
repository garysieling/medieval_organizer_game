use crate::session::*;
use crate::session::stage;
use crate::engine::ecs::System;
use crate::sim::illness::IllnessSystem;
use crate::sim::story::{detect_cascade, EventKind, StoryEvent};

/// sickness on `v`, symptoms already showing
fn give_sick(g: &mut Game, v: Entity, kind: DiseaseKind, severity: f32) {
    let d = g.world.create_entity();
    let mut dis = Disease::new(kind, v, severity);
    dis.incubation = 0;
    dis.stage = Stage::Moderate;
    g.world.add_component(d, dis);
}

/// Link 1: the half-starved fight it worse - same sickness, same
/// immunity, but the famished one's fever clears slower
#[test]
fn test_hunger_halves_what_the_fever_clears() {
    let mut g = Game::new();
    let (fed, fam) = (g.villagers[0], g.villagers[1]);
    for (v, fullness) in [(fed, 90.0), (fam, 22.0)] {
        *g.world.get_component_mut::<Hunger>(v).unwrap() = Hunger { value: fullness };
        if let Some(h) = g.world.get_component_mut::<Health>(v) { h.immunity = 0.5; }
        give_sick(&mut g, v, DiseaseKind::Influenza, 0.5);
    }
    for _ in 0..40 { IllnessSystem.run(&mut g.world); }
    let severity = |v: Entity| g.world.query::<Disease>().iter()
        .filter_map(|&d| g.world.get_component::<Disease>(d))
        .find(|d| d.patient == v).map(|d| d.severity);
    let (s_fed, s_fam) = (severity(fed), severity(fam));
    assert!(s_fed.is_some() && s_fam.is_some(), "both still fighting it");
    assert!(s_fam.unwrap() > s_fed.unwrap(),
        "the hungry one's fever should clear slower: {:?} vs {:?}", s_fed, s_fam);
}

/// Link 2a: a corpse in the lanes fouls the reach like a patient does -
/// but the churchyard's interred stay out of it
#[test]
fn test_the_unburied_dead_foul_the_reach() {
    let corpse_tile = |g: &mut Game| {
        let v = g.villagers[0];
        *g.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: 12.0, y: 14.0 };
        g.world.add_component(v, villager::Dead);
        v
    };
    let mut unburied = stage::staged(stage::Stage::Ordinary);
    let mut buried = stage::staged(stage::Stage::Ordinary);
    corpse_tile(&mut unburied);
    let v2 = corpse_tile(&mut buried);
    buried.world.add_component(v2, villager::Buried);
    // the same nine censuses for both
    for t in (2..60).step_by(7) { unburied.land_tick(t); buried.land_tick(t); }
    let near = |g: &Game| g.reach.iter().copied()
        .min_by_key(|&(x, y)| ((x as f32 - 12.0).abs() + (y as f32 - 14.0).abs()) as i32)
        .expect("there's a reach");
    let t = near(&unburied);
    let e_open = unburied.land.effluent.get(&t).copied().unwrap_or(0.0);
    let e_closed = buried.land.effluent.get(&t).copied().unwrap_or(0.0);
    assert!(e_open > e_closed + 0.02,
        "the unburied corpse should foul the water the buried one doesn't: {} vs {}", e_open, e_closed);
}

/// Link 2b: famine doubles what the leavings do - the reach fills twice
/// as fast and the water's drawn twice as readily
#[test]
fn test_famine_fouls_the_water_faster() {
    let mut fam = stage::staged(stage::Stage::Famine);
    let mut ord = stage::staged(stage::Stage::Ordinary);
    let foul = |g: &Game| g.land.effluent.values().sum::<f32>();
    let (f0, o0) = (foul(&fam), foul(&ord)); // the same seed's same start
    for t in (2..60).step_by(7) { fam.land_tick(t); ord.land_tick(t); }
    let (grown_f, grown_o) = (foul(&fam) - f0, foul(&ord) - o0);
    assert!(grown_f > grown_o * 1.4,
        "famine should foul the reach faster: +{} vs +{}", grown_f, grown_o);
}

/// Link 3: days of hardship ripen into a grievance - and meeting the
/// want again lifts it with the streak
#[test]
fn test_days_of_hardship_become_a_grievance() {
    let mut g = stage::staged(stage::Stage::Famine);
    for t in 1..=7 { g.cascade_tick(t); }
    let grieved = |g: &Game, p: desires::Problem| g.villagers.iter().any(|&v|
        desires::problems_of(&g.world, v, 20).contains(&p));
    assert!(grieved(&g, desires::Problem::Hungry),
        "a week of empty bellies should be something to organize around");
    // the granary fills again - the streak breaks and the grievance lifts
    for &v in g.villagers.clone().iter() {
        if let Some(h) = g.world.get_component_mut::<Hunger>(v) { h.value = 90.0; }
    }
    for t in 8..=14 { g.cascade_tick(t); }
    assert!(!grieved(&g, desires::Problem::Hungry),
        "the met want lifts the grievance with the streak");
}

/// The whole arc: want, fever, anger - the census writes the dominos
/// and the village reaches for a name
#[test]
fn test_the_cascade_is_named_in_order() {
    let mut g = stage::staged(stage::Stage::Famine);
    // the fever rides the famine: three beds showing symptoms
    let first3: Vec<Entity> = g.villagers.iter().take(3).copied().collect();
    for v in first3 { give_sick(&mut g, v, DiseaseKind::Influenza, 0.6); }
    for t in 1..=30 { g.cascade_tick(t); }
    let kinds: Vec<EventKind> = g.cascade_tale.iter().map(|e| e.kind).collect();
    assert!(kinds.contains(&EventKind::Hardship), "the want fell: {:?}", kinds);
    assert!(kinds.contains(&EventKind::Illness), "the fever followed: {:?}", kinds);
    assert!(kinds.contains(&EventKind::Unrest), "the anger closed it: {:?}", kinds);
    let h = kinds.iter().position(|k| *k == EventKind::Hardship).unwrap();
    let i = kinds.iter().position(|k| *k == EventKind::Illness).unwrap();
    let u = kinds.iter().position(|k| *k == EventKind::Unrest).unwrap();
    assert!(h < i && i < u, "the dominos fell in order: {:?}", kinds);
    assert!(g.cascade_named_at.is_some(), "the stretch got its name");
    assert!(g.log.iter().any(|l| l.contains("hunger year")),
        "the log names it: {:?}", g.log.iter().rev().take(6).collect::<Vec<_>>());
}

/// The detector itself: order matters, and the name follows what led
#[test]
fn test_detect_cascade_names_the_arc() {
    let ev = |tick: u64, kind: EventKind| StoryEvent { tick, kind, description: String::new() };
    // want -> fever -> anger: the hunger year
    let tale = vec![
        ev(10, EventKind::Hardship), ev(20, EventKind::Illness), ev(30, EventKind::Unrest),
    ];
    let st = detect_cascade(&tale).expect("the arc reads");
    assert_eq!(st.title, "the hunger year");
    // fever alone closing on the dead: the pestilence, then the dying
    let pest = vec![
        ev(10, EventKind::Illness), ev(20, EventKind::Death), ev(30, EventKind::Death),
    ];
    let st = detect_cascade(&pest).expect("the fever's tale reads");
    assert_eq!(st.title, "the great dying");
    // out of order is no cascade: the fever came before the want
    let backwards = vec![
        ev(10, EventKind::Illness), ev(20, EventKind::Hardship), ev(30, EventKind::Unrest),
    ];
    assert!(detect_cascade(&backwards).is_none(), "the dominoes fell backwards");
    // and no closing loss or anger, no tale
    let open = vec![
        ev(10, EventKind::Hardship), ev(20, EventKind::Illness), ev(30, EventKind::Recovery),
    ];
    assert!(detect_cascade(&open).is_none(), "a fever that broke isn't a calamity");
}
