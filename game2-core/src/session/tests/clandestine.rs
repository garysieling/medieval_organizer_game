use crate::session::*;

#[test]
fn test_the_oath_binds_the_members() {
    let mut g = Game::new();
    g.oath_intensity = 2;
    // a member's own hands hold tighter by the oath's strength
    let m = g.villagers[0];
    g.world.add_component(m, crate::sim::psychology::Bond { target: g.villagers[1], security: 0.3 });
    if let Some(o) = g.world.get_component_mut::<crate::sim::community::Organization>(g.org) {
        if !o.members.contains(&m) { o.members.push(m); }
    }
    g.clandestine_tick(22);
    let after = g.world.get_component::<crate::sim::psychology::Bond>(m)
        .map(|b| b.security).unwrap_or(0.3);
    assert!(after > 0.3, "the oath never bound: {}", after);
}

#[test]
fn test_the_eyes_get_in_with_size() {
    let mut g = Game::new();
    g.penetration = 0.0;
    g.cover = false;
    g.oath_intensity = 0;
    // a fresh org may be thin: the eyes accrue on the members it has
    if let Some(o) = g.world.get_component_mut::<crate::sim::community::Organization>(g.org) {
        if o.members.is_empty() { o.members.extend_from_slice(&g.villagers[..5]); }
    }
    g.clandestine_tick(22);
    assert!(g.penetration > 0.0, "the eyes never got in");
}

#[test]
fn test_the_cover_keeps_the_eyes_thin() {
    let mut g = Game::new();
    g.penetration = 0.0;
    g.cover = true;
    g.oath_intensity = 0;
    g.clandestine_tick(22);
    let covered = g.penetration;
    let mut g2 = Game::new();
    g2.penetration = 0.0;
    g2.cover = false;
    g2.oath_intensity = 1;
    if let Some(o) = g2.world.get_component_mut::<crate::sim::community::Organization>(g2.org) {
        if o.members.is_empty() { o.members.extend_from_slice(&g2.villagers[..5]); }
    }
    g2.clandestine_tick(22);
    assert!(covered <= g2.penetration + 0.001,
        "the cover never thinned the eyes: {} vs {}", covered, g2.penetration);
}
