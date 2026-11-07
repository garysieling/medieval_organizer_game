use crate::session::*;

#[test]
fn test_the_feast_gathers_the_village() {
    let mut g = Game::new();
    g.festival_tick(45);
    // the feast waits at the cross for the column to walk it in -
    // walk it straight out: the parade's own tests cover the march
    if let Some(f) = g.formation.take() {
        for &m in &f.members { g.world.remove_component::<villager::InFormation>(m); }
        if let formation::FormationKind::Parade { feast } = f.kind { g.parade_resolved(feast, &f.members); }
    }
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
    // the feast waits at the cross for the column to walk it in -
    // walk it straight out: the parade's own tests cover the march
    if let Some(f) = g.formation.take() {
        for &m in &f.members { g.world.remove_component::<villager::InFormation>(m); }
        if let formation::FormationKind::Parade { feast } = f.kind { g.parade_resolved(feast, &f.members); }
    }
    let fed = g.villagers.iter().filter(|&&v| {
        g.world.get_component::<Hunger>(v).map(|h| h.value > 30.0).unwrap_or(false)
    }).count();
    assert!(fed > 10, "the good stock feeds the crowd");
}

#[test]
fn test_a_local_celebration_gathers_the_neighbours() {
    let mut g = Game::new();
    let v = g.villagers[0];
    let at = g.world.get_component::<Pos>(v).map(|p| (p.x, p.y)).unwrap();
    g.local_celebration(10, at, Some(v), "a test feast");
    // the honoured one and the neighbours in reach hold a Feasting routine today
    let feasting = g.villagers.iter().filter(|&&o| g.world.get_component::<routine::Routine>(o)
        .map(|r| r.mode == routine::Mode::Feasting && r.day == 10).unwrap_or(false)).count();
    assert!(feasting >= 2, "{} neighbours celebrate", feasting);
    assert!(g.log.iter().any(|l| l.contains("a small celebration")), "the log names it");
    // and it's a neighbourhood affair, not the whole village at market
    assert!(feasting < g.villagers.len() / 2, "only the neighbours come: {} of {}", feasting, g.villagers.len());
}

#[test]
fn test_coming_of_age_draws_a_neighbourhood_celebration() {
    let mut g = Game::new();
    // age a kid to the day they take up a trade
    let kid = g.villagers.iter().copied().find(|&v| g.world.get_component::<Kid>(v).is_some()).unwrap();
    if let Some(kg) = g.world.get_component_mut::<KidGrowth>(kid) { kg.age_days = vocation::COMING_OF_AGE; }
    g.lifecycle_tick(77);
    // the rite comes first now: walk the initiate and the officiant to
    // the church or the stones, hold it, and the party is owed after
    for _ in 0..20000 {
        g.advance_travels();
        g.rites_tick(77);
        if g.log.iter().any(|l| l.contains("comes of age")) { break; }
    }
    assert!(g.log.iter().any(|l| l.contains("comes of age")), "they take up a trade");
    assert!(g.log.iter().any(|l| l.contains("a small celebration")), "and the neighbours mark it");
}
