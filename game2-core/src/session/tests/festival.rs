use crate::session::*;

#[test]
fn test_the_feast_gathers_the_village() {
    let mut g = Game::new();
    g.festival_tick(45);
    assert!(g.log.iter().any(|l| l.contains("the village gathers")),
        "the log names the feast");
    // to a place in the ring round the market cross
    let walkers = g.villagers.iter().filter(|&&v| g.world.get_component::<routine::Routine>(v)
        .map(|r| r.mode == routine::Mode::Feasting && r.day == 45).unwrap_or(false)).count();
    assert!(walkers > 10, "the whole village walks to market, {} did", walkers);
    // and the collector comes uninvited
    let c = g.collector.unwrap();
    assert!(g.travels.iter().any(|t| t.who == c), "he always shows");
}

/// Each season's feast carries its own name - a summer one isn't
/// called the harvest, a winter one isn't Candlemas
#[test]
fn test_each_season_names_its_feast() {
    let mut g = Game::new();
    for (tick, name) in [(45, "May Day"), (135, "Midsummer"), (225, "the Harvest Home"), (315, "Yule")] {
        let n = g.log.len();
        g.festival_tick(tick);
        assert!(g.log[n..].iter().any(|l| l.contains(name)),
            "t{} should name {}, log says {:?}", tick, name, &g.log[n..]);
    }
}

#[test]
fn test_the_feast_feeds_the_wants() {
    let mut g = Game::new();
    // hungry and joyless before the feast, and the store stocked for it
    for &v in g.villagers.clone().iter() {
        if let Some(h) = g.world.get_component_mut::<Hunger>(v) { h.value = 20.0; }
    }
    if let Some(b) = g.world.get_component_mut::<Building>(g.building_entities[5]) {
        household::add_stock(b, "bread", 20);
    }
    g.festival_tick(45);
    let fed = g.villagers.iter().filter(|&&v| {
        g.world.get_component::<Hunger>(v).map(|h| h.value > 30.0).unwrap_or(false)
    }).count();
    assert!(fed > 10, "the good stock feeds the crowd");
}
