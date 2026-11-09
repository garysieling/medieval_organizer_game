use crate::session::*;

#[test]
fn test_the_suit_drains_the_cause() {
    let mut g = Game::new();
    g.lawfare = Some((g.tick, g.tick));
    if let Some(o) = g.world.get_component_mut::<crate::sim::community::Organization>(g.org) {
        o.resources = 1.0;
    }
    g.lawfare_tick(g.tick + 5);
    let after = g.world.get_component::<crate::sim::community::Organization>(g.org)
        .map(|o| o.resources).unwrap_or(1.0);
    assert!(after < 1.0, "the fees never drained: {}", after);
}

#[test]
fn test_a_favorable_ruling_leaves_the_cause_spent() {
    let mut g = Game::new();
    g.lawfare = Some((g.tick, g.tick));
    if let Some(o) = g.world.get_component_mut::<crate::sim::community::Organization>(g.org) {
        o.resources = 1.0;
    }
    // the ruling comes on day 15 - the drain's already spent
    g.lawfare_tick(g.tick + 15);
    assert!(g.lawfare.is_none(), "the case never closed");
    let after = g.world.get_component::<crate::sim::community::Organization>(g.org)
        .map(|o| o.resources).unwrap_or(1.0);
    assert!(after < 1.0, "the cause wasn't spent by the winning");
    assert!(g.log.iter().any(|l| l.contains("thinner for the winning")),
        "the ruling never logged");
}
