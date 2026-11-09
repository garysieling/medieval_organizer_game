use crate::session::*;

#[test]
fn test_the_supply_feeds_the_logistics() {
    let mut g = Game::new();
    let m = g.villagers[0];
    let mut o = crate::game::org_skills::Organizing::default();
    o.training = 0.8;
    o.signal_net = 0.8;
    g.world.add_component(m, o);
    if let Some(org) = g.world.get_component_mut::<crate::sim::community::Organization>(g.org) {
        if !org.members.contains(&m) { org.members.push(m); }
    }
    g.campaign_logistics = 0.0;
    g.campaign_stack_tick(24);
    assert!(g.campaign_logistics > 0.0, "the supply never fed: {}", g.campaign_logistics);
}

#[test]
fn test_the_trained_discipline_holds_the_plant() {
    let mut g = Game::new();
    g.discipline = 0.8;
    g.plant = Some(1);
    if let Some(o) = g.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(g.org) {
        o.capacity = 0.8;
    }
    let m = g.villagers[0];
    if let Some(org) = g.world.get_component_mut::<crate::sim::community::Organization>(g.org) {
        if !org.members.contains(&m) { org.members.push(m); }
    }
    g.campaign_stack_tick(24);
    let after = g.world.get_component::<crate::sim::organized_power::OrganizedPower>(g.org)
        .map(|o| o.capacity).unwrap_or(0.8);
    assert!(after >= 0.8, "the trained discipline let the plant break it: {}", after);
}
