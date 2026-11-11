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
fn test_the_purse_passes_on() {
    let mut g = Game::new();
    let succ = g.villagers[5];
    g.world.get_component_mut::<Organization>(g.org).unwrap().members.push(succ);
    g.world.get_component_mut::<PlayerCharacter>(g.player).unwrap().resources = 42.0;
    g.world.add_component(g.player, villager::Dead);
    g.continuation_tick(1);
    assert_eq!(g.player, succ);
    let p = g.world.get_component::<PlayerCharacter>(succ).unwrap();
    assert_eq!(p.resources, 42.0, "the purse passes to whoever takes up the work");
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

#[test]
fn test_the_second_seats_death_passes_their_own_work_on() {
    // a second wanderer's passing used to go unanswered - the tick only
    // ever watched seat 0's hands (#962)
    let mut g = Game::new();
    let s1 = g.add_seat();
    let succ = g.villagers[5];
    g.world.get_component_mut::<Organization>(g.seats[s1].org).unwrap().members.push(succ);
    let r = g.world.create_entity();
    g.world.add_component(r, relationships::Relationship {
        from: g.seats[s1].player, other: succ, kind: relationships::RelKind::Friend,
        strength: 0.9, interactions: 8 });
    g.world.add_component(g.seats[s1].player, villager::Dead);
    g.continuation_tick(1);
    assert_eq!(g.seats[s1].player, succ, "seat 1's work goes to who knew them best");
    assert!(g.seats[s1].player_passed);
    assert!(!g.seats[0].player_passed, "seat 0 still stands, its own passing unrun");
    assert!(g.world.get_component::<PlayerCharacter>(succ).is_some());
}

#[test]
fn test_each_seats_passing_runs_once() {
    let mut g = Game::new();
    if let Some(home) = household::home_of(&g.world, g.player) {
        g.world.get_component_mut::<Building>(home).unwrap().occupants.clear();
    }
    g.world.add_component(g.player, villager::Dead);
    g.continuation_tick(1);
    assert!(g.player_passed);
    let dead = g.player;
    // a second tick doesn't run the passing again
    g.continuation_tick(1);
    assert_eq!(g.player, dead);
}
