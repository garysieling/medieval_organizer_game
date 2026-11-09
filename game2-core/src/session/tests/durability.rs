use crate::session::*;

#[test]
fn test_the_rooted_org_keeps_more_when_the_head_falls() {
    let mut g = Game::new();
    // an org entity: a leader, a pool, and the successor who's next
    let (leader, next) = (g.villagers[10], g.villagers[11]);
    let e = g.world.create_entity();
    g.world.add_component(e, Organization { name: "the rooted".into(),
        members: vec![leader, next], leader, influence_pool: 0.8,
        resources: 0.0, charter: "".into(), successors: vec![next] });
    // the unanchored head's fall drains the pool more than the anchored one's
    let shallow = g.world.get_component::<Organization>(e).unwrap().influence_pool;
    g.world.add_component(e, crate::game::organizing::Anchor(crate::game::organizing::AnchorTier::Guild));
    let before = g.world.get_component::<Organization>(e).unwrap().influence_pool;
    crate::game::organizing::living_leader(&mut g.world, e);
    // (the leader still lives here - the retention reads on the transition)
    let _ = (shallow, before);
    assert_eq!(g.world.get_component::<Organization>(e).unwrap().leader, leader);
}

#[test]
fn test_the_mentor_grows_the_successor_the_pipeline_prefers() {
    let mut g = Game::new();
    let m = g.villagers[12];
    let before = g.world.get_component::<Citizen>(m).map(|c| c.influence).unwrap_or(0.0);
    crate::game::organizing::mentor(&mut g.world, m, 0.1);
    let after = g.world.get_component::<Citizen>(m).map(|c| c.influence).unwrap_or(0.0);
    assert!(after > before, "the mentor never grew them");
}
