use crate::session::*;

#[test]
fn test_the_reckoning_loses_a_little() {
    let mut g = Game::new();
    g.numeracy = Numeracy::Tally;
    let buyer = g.villagers[0];
    g.world.get_component_mut::<Purse>(buyer).unwrap().gold = 50.0;
    let stall = g.trader_stall.unwrap();
    g.world.get_component_mut::<Building>(stall).unwrap()
        .stock.insert("cloth".to_string(), 5);
    g.arrive(buyer, Goal::BuyFrom(stall, "cloth", 1));
    // the price plus the foreign count's bite
    let gold = g.world.get_component::<Purse>(buyer).unwrap().gold;
    assert!(gold < 50.0 - vocation::price("cloth"), "the reckoning takes its bite");
    assert!(g.reckonings >= 1);
}

#[test]
fn test_a_clerk_steps_up_for_the_tallies() {
    let mut g = Game::new();
    g.numeracy = Numeracy::Tally;
    let buyer = g.villagers[0];
    g.world.get_component_mut::<Purse>(buyer).unwrap().gold = 500.0;
    let stall = g.trader_stall.unwrap();
    g.world.get_component_mut::<Building>(stall).unwrap()
        .stock.insert("cloth".to_string(), 20);
    for t in 0..10 { g.arrive(buyer, Goal::BuyFrom(stall, "cloth", 1)); let _ = t; }
    assert!(g.translator.is_some(), "enough bad reckonings and a clerk steps up");
    // and with a clerk, the count comes out right
    let before = g.world.get_component::<Purse>(buyer).unwrap().gold;
    g.arrive(buyer, Goal::BuyFrom(stall, "cloth", 1));
    let gold = g.world.get_component::<Purse>(buyer).unwrap().gold;
    assert!(gold >= before - vocation::price("cloth") - 0.01,
        "he keeps the tallies straight");
}
