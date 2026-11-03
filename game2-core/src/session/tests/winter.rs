use crate::session::*;

#[test]
fn test_the_winter_runs_long_and_the_store_thins() {
    let mut g = Game::new();
    g.winter_tick(400);
    assert!(g.harsh_winter, "every other winter's a hard one");
    assert!(g.log.iter().any(|l| l.contains("sets in hard")));
    if let Some(b) = g.world.get_component_mut::<Building>(g.market_food) {
        household::add_stock(b, "grain", 20);
    }
    for t in (400..410).step_by(5) { g.winter_tick(t); }
    let grain = g.world.get_component::<Building>(g.market_food)
        .map(|b| b.stock.get("grain").copied().unwrap_or(0)).unwrap_or(0);
    assert!(grain < 20, "the stores run thin");
}

#[test]
fn test_a_weak_village_looks_for_who_to_blame() {
    let mut g = Game::new();
    g.lord_esteem = 0.0;
    let herb = g.herbologist.unwrap();
    for t in (400..500).step_by(30) { g.winter_tick(t); } // 400,430,460,490 - all %30==10 in the window
    assert!(desires::problems_of(&g.world, herb, 500).contains(&desires::Problem::Blamed),
        "the herb-wife's hut looks different in a hard winter");
}

#[test]
fn test_an_org_worth_the_name_pulls_together() {
    let mut g = Game::new();
    g.lord_esteem = 0.6;
    for v in g.villagers.iter().take(6).copied().collect::<Vec<_>>() {
        if let Some(h) = g.world.get_component_mut::<Hunger>(v) { h.value = 40.0; }
    }
    // tick 1150: 1150 % 730 = 420 (in the harsh window), 1150 % 60 = 10 (the share-out day)
    g.winter_tick(1150);
    assert!(g.log.iter().any(|l| l.contains("pulls together")),
        "an org shares what it has");
    // and the herb-wife takes no blame for a shared winter
    let herb = g.herbologist.unwrap();
    assert!(!desires::problems_of(&g.world, herb, 1150).contains(&desires::Problem::Blamed),
        "belonging is the counterweight");
}
