use crate::session::*;

#[test]
fn test_the_pedlars_route_stocks_the_resin() {
    let mut g = Game::new();
    g.incense = 0.3;
    g.incense_tick(48);
    assert!(g.incense > 0.3, "the route never stocked: {}", g.incense);
}

#[test]
fn test_the_services_consume_it() {
    let mut g = Game::new();
    g.incense = 0.5;
    if let Some(c) = g.world.get_component_mut::<crate::sim::religion::Congregation>(g.congregation) {
c.services = (0..10).map(|_| crate::sim::religion::Service {
            name: "the rite".into(), frequency: "weekly".into(),
            participants: vec![], cohesion_boost: 0.1, charity_output: 0.1 }).collect();
    }
    g.incense_tick(48);
    // the route stocks 0.02, the 10 services burn 0.1 - the net's a drain
    assert!(g.incense < 0.5, "the services never consumed: {}", g.incense);
}

#[test]
fn test_the_lapse_darkens_the_church() {
    let mut g = Game::new();
    g.incense = 0.0;
    if let Some(c) = g.world.get_component_mut::<crate::sim::religion::Congregation>(g.congregation) {
c.services = (0..10).map(|_| crate::sim::religion::Service {
            name: "the rite".into(), frequency: "weekly".into(),
            participants: vec![], cohesion_boost: 0.1, charity_output: 0.1 }).collect();
    }
    g.separation_pressure = 0.2;
    g.incense_tick(48);
    assert!(g.log.iter().any(|l| l.contains("the censer hangs cold")),
        "the lapse never logged");
    assert!(g.separation_pressure > 0.2, "the lapse never pressured");
}
