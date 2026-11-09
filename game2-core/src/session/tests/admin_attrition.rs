use crate::session::*;

#[test]
fn test_the_burden_ticks_the_capacity_down() {
    let mut g = Game::new();
    g.compliance_burden = 0.5;
    if let Some(o) = g.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(g.org) {
        o.capacity = 0.8;
    }
    g.compliance_tick(35);
    let after = g.world.get_component::<crate::sim::organized_power::OrganizedPower>(g.org)
        .map(|o| o.capacity).unwrap_or(0.8);
    assert!(after < 0.8, "the burden never drained: {}", after);
}

#[test]
fn test_answering_the_postings_satisfies_the_burden() {
    let mut g = Game::new();
    g.compliance_burden = 0.5;
    if let Some(o) = g.world.get_component_mut::<crate::sim::community::Organization>(g.org) {
        o.resources = 1.0;
    }
    g.decide_at(0, 0); // answer the postings
    assert!(g.compliance_burden < 0.5, "the burden was never satisfied");
    assert!(g.log.iter().any(|l| l.contains("postings are answered")),
        "the answer never logged");
}
