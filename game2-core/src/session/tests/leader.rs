use crate::session::*;

#[test]
fn test_the_dependence_grows_as_the_leader_holds() {
    let mut g = Game::new();
    g.leader_dependence = 0.4;
    g.leader_tick(57);
    assert!(g.leader_dependence > 0.4, "the dependence never grew: {}", g.leader_dependence);
}

#[test]
fn test_the_cold_anger_screens_the_brittle() {
    let mut g = Game::new();
    let leader = g.world.get_component::<crate::sim::community::Organization>(g.org)
        .map(|o| o.leader).unwrap_or(0);
    g.world.add_component(leader, crate::game::desires::Domination {
        internalized_domination: 0.8, critical_agency: 0.1, experience: 0.0 });
    g.leader_tick(57);
    assert!(g.log.iter().any(|l| l.contains("cold-anger screen")),
        "the screen never fired");
}

#[test]
fn test_the_high_dependence_drains_the_capacity() {
    let mut g = Game::new();
    g.leader_dependence = 0.9;
    if let Some(o) = g.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(g.org) {
        o.capacity = 0.8;
    }
    g.leader_tick(57);
    let after = g.world.get_component::<crate::sim::organized_power::OrganizedPower>(g.org)
        .map(|o| o.capacity).unwrap_or(0.8);
    assert!(after < 0.8, "the dependence never drained: {}", after);
}
