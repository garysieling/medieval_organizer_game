use crate::session::*;

#[test]
fn test_the_caravan_forms_between_the_sites() {
    let mut g = Game::new();
    g.caravan_tick(93);
    assert!(g.caravan.is_some(), "the caravan never formed");
    assert!(g.log.iter().any(|l| l.contains("the caravan reaches") || l.contains("the caravan's taken")),
        "the road never logged");
}

#[test]
fn test_the_far_market_pays() {
    let mut g = Game::new();
    // no bandits - the road's clear
    if let Some(o) = g.world.get_component_mut::<crate::sim::community::Organization>(g.org) {
        o.resources = 0.0;
    }
    g.caravan_tick(93);
    let after = g.world.get_component::<crate::sim::community::Organization>(g.org)
        .map(|o| o.resources).unwrap_or(0.0);
    assert!(after > 0.0, "the market never paid: {}", after);
}

#[test]
fn test_the_bandits_take_the_caravan() {
    let mut g = Game::new();
    for site in &g.sites {
        if let Some(l) = site.lord {
            g.world.add_component(l, crate::session::subsystems::society::Watcher {
                paranoia: 0.8, legitimacy: 0.8 });
        }
    }
    g.caravan_tick(93);
    assert!(g.log.iter().any(|l| l.contains("the caravan's taken")),
        "the bandits never took");
}
