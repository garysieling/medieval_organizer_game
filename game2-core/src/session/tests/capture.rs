use crate::session::*;

#[test]
fn test_the_prophets_org_splits_on_the_capture() {
    let mut g = Game::new();
    g.prophet = true;
    if let Some(o) = g.world.get_component_mut::<crate::sim::community::Organization>(g.org) {
        for i in 0..8 { o.members.push(g.villagers[i % g.villagers.len()]); }
    }
    let before = g.world.get_component::<crate::sim::community::Organization>(g.org)
        .map(|o| o.members.len()).unwrap_or(0);
    g.capture_tick(101);
    let after = g.world.get_component::<crate::sim::community::Organization>(g.org)
        .map(|o| o.members.len()).unwrap_or(0);
    assert!(after < before, "the knife-fight never cost: {} -> {}", before, after);
    assert!(g.log.iter().any(|l| l.contains("the prophet's own capture")),
        "the capture never logged");
}

#[test]
fn test_the_covers_withdrawal_costs() {
    let mut g = Game::new();
    g.shield_cover = Some(g.tick);
    g.institutions = 2;
    if let Some(o) = g.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(g.org) {
        o.capacity = 0.8;
    }
    g.capture_tick(101);
    assert!(g.shield_cover.is_none(), "the cover never withdrew");
    assert_eq!(g.institutions, 1, "the shield never pulled a pillar");
    let after = g.world.get_component::<crate::sim::organized_power::OrganizedPower>(g.org)
        .map(|o| o.capacity).unwrap_or(0.8);
    assert!(after < 0.8, "the withdrawal never cost: {}", after);
}

#[test]
fn test_the_structural_org_resists() {
    let mut g = Game::new();
    g.prophet = false; // the structural org
    if let Some(o) = g.world.get_component_mut::<crate::sim::community::Organization>(g.org) {
        for i in 0..8 { o.members.push(g.villagers[i % g.villagers.len()]); }
    }
    let before = g.world.get_component::<crate::sim::community::Organization>(g.org)
        .map(|o| o.members.len()).unwrap_or(0);
    g.capture_tick(101);
    let after = g.world.get_component::<crate::sim::community::Organization>(g.org)
        .map(|o| o.members.len()).unwrap_or(0);
    assert_eq!(after, before, "the structural org lost members: {} -> {}", before, after);
}
