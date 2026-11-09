use crate::session::*;

#[test]
fn test_ordination_lifts_a_soul_to_the_clergy() {
    let mut g = Game::new();
    let m = g.villagers[0];
    g.world.add_component(m, crate::game::vocation::Estate::Laity);
    if let Some(c) = g.world.get_component_mut::<crate::game::vocation::Craft>(m) {
        c.vocation = crate::game::vocation::Vocation::Priest;
    }
    g.estate_tick(8);
    let after = g.world.get_component::<crate::game::vocation::Estate>(m).copied();
    assert_eq!(after, Some(crate::game::vocation::Estate::Clergy),
        "the ordination never lifted them");
}

#[test]
fn test_the_clergys_read_brings_its_dues() {
    let mut g = Game::new();
    let m = g.villagers[0];
    g.world.add_component(m, crate::game::vocation::Estate::Clergy);
    g.world.add_component(m, crate::sim::religion::Faith {
        religion: "the parish".into(), devotion: 0.5, traditions: vec![] });
    g.estate_tick(8);
    let after = g.world.get_component::<crate::sim::religion::Faith>(m)
        .map(|f| f.devotion).unwrap_or(0.5);
    assert!(after > 0.5, "the clergy's dues never accrued: {}", after);
}
