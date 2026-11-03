use crate::session::*;

#[test]
fn test_the_third_defiance_is_a_gibbet() {
    let mut g = Game::new();
    g.lord_esteem = 0.8;
    let m = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<TownRole>(v).is_none()).unwrap();
    for p in [desires::Problem::Broke, desires::Problem::Blamed,
              desires::Problem::WritBorne, desires::Problem::LongWinter,
              desires::Problem::ShortChanged] {
        desires::add_grievance(&mut g.world, m, p, 100);
    }
    g.world.get_component_mut::<Organization>(g.org).unwrap().members.push(m);
    g.repression_debt = 2; // the third writ
    g.edict_tick(320); // summons day
    assert!(villager::is_dead(&g.world, m), "the bailiff comes back with knights");
    assert!(g.martyr_grave.is_some(), "a martyr's grave");
    assert!(g.log.iter().any(|l| l.contains("hanged for the defiance")));
}

#[test]
fn test_the_vigil_keeps_the_grave() {
    let mut g = Game::new();
    g.martyr_grave = Some((15, 15));
    let before = g.world.get_component::<crate::sim::organized_power::OrganizedPower>(g.org)
        .map(|p| p.capacity).unwrap_or(0.0);
    g.martyr_tick(20);
    let after = g.world.get_component::<crate::sim::organized_power::OrganizedPower>(g.org)
        .map(|p| p.capacity).unwrap_or(0.0);
    assert!(after > before, "the remembering is a kind of organizing");
}
