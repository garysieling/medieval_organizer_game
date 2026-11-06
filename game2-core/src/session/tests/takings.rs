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
