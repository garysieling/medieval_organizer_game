use crate::session::*;

#[test]
fn test_the_work_goes_on() {
    let mut g = Game::new();
    let succ = g.villagers[5];
    g.world.get_component_mut::<Organization>(g.org).unwrap().members.push(succ);
    let r = g.world.create_entity();
    g.world.add_component(r, relationships::Relationship {
        from: g.player, other: succ, kind: relationships::RelKind::Friend,
        strength: 0.9, interactions: 8 });
    g.world.add_component(g.player, villager::Dead);
    g.continuation_tick(1);
    assert_eq!(g.player, succ, "the deepest bond takes it up");
    assert!(g.log.iter().any(|l| l.contains("the work goes on")));
    assert!(g.world.get_component::<PlayerCharacter>(succ).is_some());
}

#[test]
fn test_the_standing_passes_half_on() {
    let mut g = Game::new();
    let succ = g.villagers[5];
    g.world.get_component_mut::<Organization>(g.org).unwrap().members.push(succ);
    g.world.get_component_mut::<PlayerCharacter>(g.player).unwrap().influence = 1.6;
    g.world.add_component(g.player, villager::Dead);
    g.continuation_tick(1);
    assert_eq!(g.player, succ);
    let p = g.world.get_component::<PlayerCharacter>(succ).unwrap();
    assert!((p.influence - 0.8).abs() < 1e-5,
        "the standing passes half on - got {}", p.influence);
}

#[test]
fn test_no_hands_means_the_work_dies() {
    let mut g = Game::new();
    // no org, no kin under the roof
    if let Some(home) = household::home_of(&g.world, g.player) {
        g.world.get_component_mut::<Building>(home).unwrap().occupants.clear();
    }
    g.world.add_component(g.player, villager::Dead);
    g.continuation_tick(1);
    assert!(g.log.iter().any(|l| l.contains("the work dies with them")));
    assert!(g.player_passed);
}
