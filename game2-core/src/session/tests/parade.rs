use crate::session::*;

#[test]
fn test_the_village_walks_it_in() {
    let mut g = Game::new();
    g.festival_tick(81);
    let Some(f) = &g.formation else { panic!("no column formed"); };
    assert!(matches!(f.kind, formation::FormationKind::Parade { feast: Some(..) }), "{:?}", f.kind);
    let members = f.members.clone();
    assert!(members.len() > 10, "a thin parade: {}", members.len());
    for &m in &members {
        assert!(g.world.get_component::<villager::InFormation>(m).is_some(), "a marcher never fell in");
    }
    // the column walks it out, the feast lands when it comes home
    for _ in 0..12000 { g.update(); if g.formation.is_none() { break; } }
    assert!(g.formation.is_none(), "the parade never came home");
    for &m in &members {
        assert!(g.world.get_component::<villager::InFormation>(m).is_none(), "a marcher never stood down");
    }
    let feasting = members.iter().filter(|&&m|
        g.world.get_component::<routine::Routine>(m).map(|r| r.mode == routine::Mode::Feasting).unwrap_or(false)).count();
    assert!(feasting > members.len() / 2, "the feast never gathered: {}/{}", feasting, members.len());
}

#[test]
fn test_a_carried_demand_walks_the_streets() {
    let mut g = Game::new();
    g.victory_parade();
    assert!(matches!(g.formation.as_ref().map(|f| f.kind),
        Some(formation::FormationKind::Parade { feast: None })), "{:?}", g.formation.as_ref().map(|f| f.kind));
    for _ in 0..30000 { g.update(); if g.formation.is_none() { break; } }
    assert!(g.formation.is_none(), "the victory parade never broke up");
}
