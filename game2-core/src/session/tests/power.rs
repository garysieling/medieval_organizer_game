use crate::session::*;

fn capacity(g: &Game) -> f32 {
    g.world.get_component::<crate::sim::organized_power::OrganizedPower>(g.org)
        .map(|p| p.capacity).unwrap_or(0.0)
}

#[test]
fn test_power_builds_as_its_spent_and_atrophies_idle() {
    let mut g = Game::new();
    assert_eq!(capacity(&g), 0.0, "you're nobody until you've shown power");
    g.record_power(crate::sim::organized_power::ActionQuality::Transformational);
    g.record_power(crate::sim::organized_power::ActionQuality::Transformational);
    let high = capacity(&g);
    assert!(high > 0.2, "meetings in the street build it");
    for t in (3..60).step_by(7) { g.power_tick(t); }
    assert!(capacity(&g) < high, "power atrophies when idle - unlike gold");
}

#[test]
fn test_the_lords_estimate_lags_and_shifts_the_tax() {
    let mut g = Game::new();
    // a strong org takes weeks to register with the lord
    for _ in 0..10 { g.record_power(crate::sim::organized_power::ActionQuality::Transformational); }
    assert!(capacity(&g) > 1.0);
    for t in (3..200).step_by(7) { g.power_tick(t); }
    assert!(g.lord_esteem > 0.6, "eventually the lord learns");
    // and then the collector passes the player by
    g.world.get_component_mut::<Purse>(g.player).unwrap().gold = 100.0;
    g.arrive(g.collector.unwrap(), Goal::CollectTax(g.player));
    assert_eq!(g.world.get_component::<Purse>(g.player).unwrap().gold, 100.0,
        "a force is passed by");
}
