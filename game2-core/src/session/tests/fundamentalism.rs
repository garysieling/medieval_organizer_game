use crate::session::*;

#[test]
fn test_the_risk_drifts_the_absolutism() {
    let mut g = Game::new();
    if let Some(c) = g.world.get_component_mut::<crate::sim::religion::Congregation>(g.congregation) {
        c.orthodoxy = 0.7;
        c.cultish = true;
        c.absolutism = 0.3;
    }
    g.fundamentalism_tick(56);
    let after = g.world.get_component::<crate::sim::religion::Congregation>(g.congregation)
        .map(|c| c.absolutism).unwrap_or(0.3);
    assert!(after > 0.3, "the risk never drifted: {}", after);
}

#[test]
fn test_the_high_ebb_hardens_the_fold() {
    let mut g = Game::new();
    if let Some(c) = g.world.get_component_mut::<crate::sim::religion::Congregation>(g.congregation) {
        c.orthodoxy = 0.7;
        c.cultish = true;
        c.absolutism = 0.7;
    }
    g.separation_pressure = 0.3;
    g.fundamentalism_tick(56);
    assert!(g.separation_pressure > 0.3, "the hardness never pressured");
    let after = g.world.get_component::<crate::sim::religion::Congregation>(g.congregation)
        .map(|c| c.orthodoxy).unwrap_or(0.7);
    assert!(after > 0.7, "the hardness never hardened: {}", after);
}
