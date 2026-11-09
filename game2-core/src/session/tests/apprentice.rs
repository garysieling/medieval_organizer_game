use crate::session::*;

#[test]
fn test_the_term_accrues_under_a_master() {
    let mut g = Game::new();
    let m = g.villagers[0];
    let master = g.villagers[1];
    if let Some(c) = g.world.get_component_mut::<crate::game::vocation::Craft>(m) {
        c.master = Some(master);
        c.apprentice_years = 3;
        c.skill = 0.2;
    }
    g.apprentice_tick(54);
    let years = g.world.get_component::<crate::game::vocation::Craft>(m)
        .map(|c| c.apprentice_years).unwrap_or(3);
    assert!(years > 3, "the term never accrued: {}", years);
}

#[test]
fn test_the_graduation_lifts_the_skill() {
    let mut g = Game::new();
    let m = g.villagers[0];
    let master = g.villagers[1];
    if let Some(c) = g.world.get_component_mut::<crate::game::vocation::Craft>(m) {
        c.master = Some(master);
        c.apprentice_years = 6;
        c.skill = 0.2;
    }
    g.apprentice_tick(54);
    let skill = g.world.get_component::<crate::game::vocation::Craft>(m)
        .map(|c| c.skill).unwrap_or(0.2);
    assert!(skill >= 0.4, "the graduation never lifted: {}", skill);
    assert!(g.log.iter().any(|l| l.contains("the journeyman's own ceremony")),
        "the ceremony never logged");
}
