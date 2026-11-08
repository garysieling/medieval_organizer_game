use crate::session::*;

#[test]
fn test_the_next_man_stands_when_the_head_falls() {
    let mut g = Game::new();
    let org = g.org;
    let (a, b) = (g.villagers[1], g.villagers[2]);
    g.world.get_component_mut::<crate::sim::community::Organization>(org)
        .map(|o| o.successors = vec![a, b]);
    let up = crate::sim::community::decapitate(&mut g.world, org, 50);
    assert_eq!(up, Some(a), "the shadow leader never stood");
    assert_eq!(g.world.get_component::<crate::sim::community::Organization>(org)
        .map(|o| o.leader).unwrap(), a);
}

#[test]
fn test_the_boycott_cuts_by_the_web_it_leaves() {
    let mut g = Game::new();
    let org = g.org;
    let (m, defector) = (g.villagers[1], g.villagers[3]);
    // a dense web: the member's bond points inside the org
    g.world.add_component(m, crate::sim::psychology::Bond { target: g.villagers[2], security: 0.9 });
    g.world.add_component(g.villagers[2], crate::sim::psychology::Bond { target: m, security: 0.9 });
    g.world.get_component_mut::<crate::sim::community::Organization>(org)
        .map(|o| { o.members = vec![m, g.villagers[2]]; });
    g.world.add_component(defector, crate::sim::psychology::Bond { target: m, security: 0.9 });
    let dense = crate::sim::community::insider_boycott(&mut g.world, org, defector);
    assert!(dense > 0.5, "the dense web never counted");
    assert!(g.world.get_component::<crate::sim::psychology::Bond>(defector)
        .map(|b| b.security < 0.4).unwrap_or(false), "the boycott never cut");
}
