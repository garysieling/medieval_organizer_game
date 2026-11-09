use crate::session::*;

#[test]
fn test_the_herald_brings_the_offer() {
    let mut g = Game::new();
    let m = g.villagers[0];
    let mut o = crate::game::org_skills::Organizing::default();
    o.turnout = 0.8;
    o.agitation = 0.8;
    g.world.add_component(m, o);
    if let Some(org) = g.world.get_component_mut::<crate::sim::community::Organization>(g.org) {
        if !org.members.contains(&m) { org.members.push(m); }
    }
    for site in &g.sites {
        if let Some(l) = site.lord {
            g.world.add_component(l, crate::session::subsystems::society::Watcher {
                paranoia: 0.8, legitimacy: 0.8 });
        }
    }
    g.parley_tick(31);
    assert!(g.parley.is_some(), "the offer never came");
    assert!(g.log.iter().any(|l| l.contains("white flag")),
        "the herald never logged");
}

#[test]
fn test_holding_the_distance_keeps_the_leader() {
    let mut g = Game::new();
    g.parley = Some(g.tick);
    let leader = g.world.get_component::<crate::sim::community::Organization>(g.org)
        .map(|o| o.leader).unwrap_or(0);
    g.decide_at(0, 1); // hold the distance
    assert!(g.parley.is_none(), "the parley never resolved");
    let after = g.world.get_component::<crate::sim::community::Organization>(g.org)
        .map(|o| o.leader).unwrap_or(0);
    assert_eq!(after, leader, "the leader was taken at a distance");
}
