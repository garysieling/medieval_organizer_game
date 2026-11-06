use crate::session::*;

#[test]
fn test_the_collector_rides_the_takings_out_himself() {
    let mut g = Game::new();
    let c = g.collector.unwrap();
    g.takings = 30.0;
    g.livelihood_tick(45); // the remit day
    assert_eq!(g.takings, 0.0, "the books never cleared");
    assert!(g.log.iter().any(|l| l.contains("rides to the castle")),
        "the ride went unsaid");
    let t = g.travels.iter().find(|t| t.who == c)
        .expect("the collector never left - the log lied");
    assert!(matches!(t.goal, Goal::Remit(_)), "his errand's no remit: {:?}", t.goal);
    // and the road's a real one: it leads off toward high ground
    assert!(!t.path.is_empty(), "a ride with no road under it");
}

#[test]
fn test_a_hungry_collector_rides_the_gold_through() {
    let mut g = Game::new();
    let c = g.collector.unwrap();
    if let Some(h) = g.world.get_component_mut::<Hunger>(c) { h.value = 20.0; }
    g.takings = 30.0;
    g.livelihood_tick(45);
    assert!(g.travels.iter().any(|t| t.who == c && matches!(t.goal, Goal::Remit(_))),
        "the hungry collector dropped the gold: {:?}",
        g.travels.iter().filter(|t| t.who == c).map(|t| format!("{:?}", t.goal)).collect::<Vec<_>>());
}

#[test]
fn test_the_ride_reaches_the_keep() {
    let mut g = Game::new();
    let c = g.collector.unwrap();
    if let Some(h) = g.world.get_component_mut::<Hunger>(c) { h.value = 20.0; }
    g.takings = 30.0;
    g.livelihood_tick(45);
    // ride the road: the flag holds through the hungry frames, and the
    // keep's table feeds him at the end
    for _ in 0..4000 {
        g.advance_travels();
        if !g.travels.iter().any(|t| t.who == c && matches!(t.goal, Goal::Remit(_))) { break; }
    }
    assert!(g.world.get_component::<villager::OnTheRoad>(c).is_none(), "the flag outlived the ride");
    let dist = |a: (f32, f32), b: (i32, i32)| (a.0 - b.0 as f32).abs() + (a.1 - b.1 as f32).abs();
    let pos = g.world.get_component::<Pos>(c).map(|p| (p.x, p.y)).unwrap();
    let castle = g.sites.iter().find(|s| s.kind == crate::session::country::SiteKind::Castle).unwrap().centre;
    assert!(dist(pos, castle) < 12.0 || g.travels.iter().any(|t| t.who == c),
        "he never got near the keep: {:?} vs {:?}", pos, castle);
}
