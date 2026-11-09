use crate::session::*;

#[test]
fn test_the_drought_accrues() {
    let mut g = Game::new();
    g.drought = 10;
    g.event_pace_tick(35);
    assert!(g.drought > 10, "the drought never accrued: {}", g.drought);
}

#[test]
fn test_the_breaker_fires_at_the_long_stretch() {
    let mut g = Game::new();
    g.drought = 60;
    if let Some(o) = g.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(g.org) {
        o.capacity = 0.3;
    }
    g.event_pace_tick(35);
    assert_eq!(g.drought, 0, "the breaker never reset: {}", g.drought);
    let after = g.world.get_component::<crate::sim::organized_power::OrganizedPower>(g.org)
        .map(|o| o.capacity).unwrap_or(0.3);
    assert!(after > 0.3, "the breaker never paid: {}", after);
    assert!(g.log.iter().any(|l| l.contains("the drought-breaker lands")),
        "the breaker never logged");
}

#[test]
fn test_the_win_celebrates() {
    let mut g = Game::new();
    g.comeback = Some(g.tick);
    g.event_pace_tick(35);
    assert!(g.celebration.is_some(), "the win never celebrated");
    assert!(g.log.iter().any(|l| l.contains("the village's own celebration")),
        "the celebration never logged");
}
