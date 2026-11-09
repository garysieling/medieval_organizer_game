use crate::session::*;

#[test]
fn test_the_org_weakens_the_lord() {
    let mut g = Game::new();
    g.lord_strength = 0.8;
    if let Some(o) = g.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(g.org) {
        o.capacity = 0.8;
    }
    g.authority_tick(49);
    assert!(g.lord_strength < 0.8, "the org never dissolved: {}", g.lord_strength);
}

#[test]
fn test_the_weak_lord_feeds_the_org() {
    let mut g = Game::new();
    g.lord_strength = 0.2;
    if let Some(o) = g.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(g.org) {
        o.capacity = 0.3;
    }
    if let Some(org) = g.world.get_component_mut::<crate::sim::community::Organization>(g.org) {
        org.resources = 0.0;
    }
    g.authority_tick(49);
    let after = g.world.get_component::<crate::sim::organized_power::OrganizedPower>(g.org)
        .map(|o| o.capacity).unwrap_or(0.3);
    assert!(after > 0.3, "the weak lord never fed: {}", after);
    let resources = g.world.get_component::<crate::sim::community::Organization>(g.org)
        .map(|o| o.resources).unwrap_or(0.0);
    assert!(resources > 0.0, "the self-org never answered: {}", resources);
}
