use crate::session::*;

#[test]
fn test_taken_is_held_not_gone() {
    let mut g = Game::new();
    let m = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<TownRole>(v).is_none()).unwrap();
    g.dungeon.push((m, 10));
    g.arrive(m, Goal::Exile); // they reach the castle's edge
    assert!(!villager::is_dead(&g.world, m), "held, not gone");
    assert!(g.log.iter().any(|l| l.contains("held at the castle")));
}

#[test]
fn test_the_org_pays_the_ransom() {
    let mut g = Game::new();
    let m = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<TownRole>(v).is_none()).unwrap();
    g.world.get_component_mut::<Organization>(g.org).unwrap().members.push(m);
    g.world.get_component_mut::<PlayerCharacter>(g.player).unwrap().resources = 50.0;
    g.dungeon.push((m, 10));
    g.dungeon_tick(50); // 40 days held, a %15==5 day - inside the window
    assert!(g.dungeon.is_empty(), "the ransom came");
    assert!(g.log.iter().any(|l| l.contains("pays the ransom")));
    let res = g.world.get_component::<PlayerCharacter>(g.player).unwrap().resources;
    assert_eq!(res, 30.0, "the purse answers for its own");
}

#[test]
fn test_the_ransomed_dont_walk_off_the_world() {
    let mut g = Game::new();
    let m = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<TownRole>(v).is_none()).unwrap();
    g.world.get_component_mut::<Organization>(g.org).unwrap().members.push(m);
    g.world.get_component_mut::<PlayerCharacter>(g.player).unwrap().resources = 50.0;
    g.dungeon.push((m, 10));
    assert!(g.send(m, Goal::Exile), "the writ marches them out");
    g.world.add_component(m, crate::game::dungeon::CastleHeld { since: 10 });
    // the ransom comes mid-march: the walk out is still live when the
    // cell's paid - the exile leg can't arrive a driving-out
    g.dungeon_tick(50);
    assert!(!villager::is_dead(&g.world, m), "the ransomed live");
    assert!(g.world.get_component::<crate::game::dungeon::CastleHeld>(m).is_none(),
        "the castle lets go");
    assert!(g.travels.iter().all(|t| t.who != m || !matches!(t.goal, Goal::Exile)),
        "the walk out is spent");
    // and the exile leg can't finish what the pardon stopped
    g.advance_travels();
    assert!(!villager::is_dead(&g.world, m), "the pardoned don't die at the edge");
}

#[test]
fn test_no_ransom_means_the_ladder_climbs() {
    let mut g = Game::new();
    let m = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<TownRole>(v).is_none()).unwrap();
    g.dungeon.push((m, 10));
    g.dungeon_tick(80); // 70 days - a %15==5 day, unanswered
    assert!(villager::is_dead(&g.world, m), "the ladder climbs to its top");
    assert!(g.log.iter().any(|l| l.contains("hanged at the castle")));
}
