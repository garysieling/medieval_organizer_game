use crate::session::*;

#[test]
fn test_the_route_stocks_the_leaf() {
    let mut g = Game::new();
    g.tea = 0.2;
    g.tea_tick(43);
    // the route stocks 0.02, the cup consumes 0.01 - the net's a rise
    assert!(g.tea > 0.2, "the route never stocked: {}", g.tea);
}

#[test]
fn test_the_cup_lifts_the_condition() {
    let mut g = Game::new();
    g.tea = 0.3;
    let m = g.villagers[0];
    g.world.add_component(m, crate::sim::illness::Health {
        condition: 0.5, immunity: 0.5 });
    g.tea_tick(43);
    let after = g.world.get_component::<crate::sim::illness::Health>(m)
        .map(|h| h.condition).unwrap_or(0.5);
    assert!(after > 0.5, "the cup never lifted: {}", after);
}

#[test]
fn test_the_ceremony_bonds() {
    let mut g = Game::new();
    g.tea = 0.5;
    let m = g.villagers[0];
    g.world.add_component(m, crate::sim::psychology::Bond {
        target: g.villagers[1], security: 0.5 });
    g.tea_tick(43);
    let after = g.world.get_component::<crate::sim::psychology::Bond>(m)
        .map(|b| b.security).unwrap_or(0.5);
    assert!(after > 0.5, "the ceremony never bonded: {}", after);
}
