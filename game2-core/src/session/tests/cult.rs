use crate::session::*;

#[test]
fn test_the_flock_follows_its_shepherd_past_the_edge() {
    let mut g = Game::new();
    // force the three factors: strict orthodoxy, a leader the flock
    // holds close, and bonds that stay inside the parish
    let leader = g.villagers[0];
    for v in g.villagers.iter().take(6).copied().collect::<Vec<_>>() {
        // the flock's bond sits on the shepherd: the Bond component's
        // one-per-entity, so the leader's the one it keeps
        g.world.add_component(v, crate::sim::psychology::Bond { target: leader, security: 0.8 });
    }
    let flock: Vec<Entity> = g.villagers.iter().take(6).copied().collect();
    g.world.get_component_mut::<crate::sim::religion::Congregation>(g.congregation)
        .map(|c| { c.cultish = false; c.orthodoxy = 0.85; c.leader = Some(leader);
            c.members = flock; });
    g.cult_tick(50);
    let cult = g.world.get_component::<crate::sim::religion::Congregation>(g.congregation)
        .map(|c| c.cultish).unwrap_or(false);
    assert!(cult, "the parish never turned inward");
    assert!(g.villagers.iter().any(|&v|
        g.world.get_component::<crate::sim::religion::Cultish>(v).is_some()),
        "the flock never marked");
}

#[test]
fn test_the_fire_burns_out() {
    let mut g = Game::new();
    g.world.get_component_mut::<crate::sim::religion::Congregation>(g.congregation)
        .map(|c| { c.cultish = true; c.orthodoxy = 0.3; });
    g.cult_tick(51);
    assert!(!g.world.get_component::<crate::sim::religion::Congregation>(g.congregation)
        .map(|c| c.cultish).unwrap_or(true), "the cult never institutionalized");
}
