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

#[test]
fn test_a_season_of_his_books_and_the_village_counts_his_way() {
    let mut g = Game::new();
    let c = g.villagers[3];
    g.translator = Some(c);
    // before the season's out the count's still his (12 free weeks = 84d)
    for t in (3..=80).step_by(7) { g.power_tick(t); }
    assert_eq!(g.numeracy, Numeracy::Tally, "the way spread too soon");
    // a season on, the village keeps the books the clerk's way
    g.power_tick(87);
    assert_eq!(g.numeracy, Numeracy::Positional, "the clerk's way never took");
    assert!(g.log.iter().any(|l| l.contains("the clerk's way")), "the turn went unsaid");
}

#[test]
fn test_a_dead_or_held_clerk_teaches_nothing() {
    let mut g = Game::new();
    let c = g.villagers[3];
    g.translator = Some(c);
    g.world.add_component(c, crate::game::villager::Dead);
    g.power_tick(3);
    assert_eq!(g.translator_days, 0, "the dead teach nothing");
    let mut g2 = Game::new();
    let c2 = g2.villagers[3];
    g2.translator = Some(c2);
    g2.world.add_component(c2, crate::game::dungeon::Imprisoned { since: 0, charge: "the count".into() });
    g2.power_tick(3);
    assert_eq!(g2.translator_days, 0, "the held teach nothing");
}
