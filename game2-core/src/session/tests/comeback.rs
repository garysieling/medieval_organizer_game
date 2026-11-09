use crate::session::*;

#[test]
fn test_the_misery_fires_the_windfall() {
    let mut g = Game::new();
    if let Some(o) = g.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(g.org) {
        o.capacity = 0.1;
    }
    g.comeback_tick(104);
    assert!(g.comeback.is_some(), "the windfall never fired");
    assert!(g.log.iter().any(|l| l.contains("the windfall's own turn")),
        "the turn never logged");
}

#[test]
fn test_the_windfall_relieves_the_ebb() {
    let mut g = Game::new();
    if let Some(o) = g.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(g.org) {
        o.capacity = 0.1;
    }
    g.comeback_tick(104);
    let after = g.world.get_component::<crate::sim::organized_power::OrganizedPower>(g.org)
        .map(|o| o.capacity).unwrap_or(0.1);
    assert!(after > 0.1, "the windfall never relieved: {}", after);
}

#[test]
fn test_the_well_off_get_no_windfall() {
    let mut g = Game::new();
    if let Some(o) = g.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(g.org) {
        o.capacity = 0.8;
    }
    g.comeback_tick(104);
    assert!(g.comeback.is_none(), "the well-off got a windfall");
}
