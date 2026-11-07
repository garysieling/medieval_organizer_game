use crate::session::*;

fn strong_org(g: &mut Game) {
    let org = g.org;
    let o = g.world.get_component_mut::<crate::sim::community::Organization>(org).unwrap();
    o.members = g.villagers[0..10].to_vec();
    o.resources = 500.0;
    o.influence_pool = 10.0;
}

#[test]
fn test_the_org_raises_its_ring() {
    let mut g = Game::new();
    strong_org(&mut g);
    g.monument_tick(50);
    assert!(g.monuments.iter().any(|m| m.kind == MonumentKind::Circle),
        "no ring was resolved");
    let m = g.monuments.iter().find(|m| m.kind == MonumentKind::Circle).unwrap();
    assert!(m.stood_on.is_none(), "the ring stood too soon");
    // the org paid from its own purse
    let org = g.org;
    let res = g.world.get_component::<crate::sim::community::Organization>(org).unwrap().resources;
    assert!((res - 300.0).abs() < 0.1, "the purse reads {}", res);
    // and it stands after its long build
    g.monument_tick(50 + 40);
    assert!(g.monuments.iter().find(|m| m.kind == MonumentKind::Circle).unwrap().stood_on.is_some(),
        "the ring never stood");
}

#[test]
fn test_a_poor_or_small_org_raises_nothing() {
    let mut g = Game::new();
    // standing but broke
    let org = g.org;
    {
        let o = g.world.get_component_mut::<crate::sim::community::Organization>(org).unwrap();
        o.members = g.villagers[0..10].to_vec();
        o.resources = 10.0;
        o.influence_pool = 10.0;
    }
    g.monument_tick(50);
    assert!(!g.monuments.iter().any(|m| m.kind == MonumentKind::Circle),
        "a broke org raised a ring");
    // rich but no standing
    {
        let o = g.world.get_component_mut::<crate::sim::community::Organization>(org).unwrap();
        o.members = vec![g.villagers[0]];
        o.resources = 500.0;
    }
    g.monument_tick(50);
    assert!(!g.monuments.iter().any(|m| m.kind == MonumentKind::Circle),
        "a one-man guild raised a ring");
}

#[test]
fn test_a_disbanded_org_loses_its_unbuilt_ring() {
    let mut g = Game::new();
    strong_org(&mut g);
    g.monument_tick(50);
    assert!(g.monuments.iter().any(|m| m.kind == MonumentKind::Circle));
    // the guild disbands mid-build
    g.world.remove_component::<crate::sim::community::Organization>(g.org);
    g.monument_tick(51);
    assert!(!g.monuments.iter().any(|m| m.kind == MonumentKind::Circle
        && m.stood_on.is_none()), "a dead guild's ring still rises");
}
