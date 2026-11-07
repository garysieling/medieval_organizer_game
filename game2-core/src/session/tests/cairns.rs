use crate::session::*;

#[test]
fn test_a_death_unmourned_becomes_a_pile() {
    let mut g = Game::new();
    // lose someone folk loved
    let i = 0;
    let honoree = g.villagers[i];
    let pal = g.villagers[3];
    let r = g.world.create_entity();
    g.world.add_component(r, relationships::Relationship { from: pal, other: honoree, kind: relationships::RelKind::Friend, strength: 0.8, interactions: 5 });
    // the dead lie in the ground a week before the pile rises
    g.died_on[i] = Some(40);
    g.cairn_tick(46);
    assert!(g.cairns.is_empty(), "a week hasn't passed");
    g.cairn_tick(48);
    assert_eq!(g.cairns.len(), 1, "no cairn rose");
    let c = &g.cairns[0];
    assert_eq!(c.honoree, i);
    assert!(!c.deed.is_empty(), "the pile doesn't know who it keeps");
    assert!(g.log.iter().any(|l| l.contains("cairn rises")), "the log didn't mark it");
}

#[test]
fn test_the_pile_grows_a_stone_a_day() {
    let mut g = Game::new();
    let i = 0;
    let honoree = g.villagers[i];
    let pal = g.villagers[3];
    let r = g.world.create_entity();
    g.world.add_component(r, relationships::Relationship { from: pal, other: honoree, kind: relationships::RelKind::Friend, strength: 0.8, interactions: 5 });
    *g.world.get_component_mut::<Pos>(pal).unwrap() = Pos { x: g.village.0 as f32, y: g.village.1 as f32 };
    g.died_on[i] = Some(40);
    g.cairn_tick(48);
    let c = g.cairns[0].pos;
    *g.world.get_component_mut::<Pos>(pal).unwrap() = Pos { x: c.0 as f32, y: c.1 as f32 };
    let before = g.cairns[0].stones;
    g.cairn_tick(49);
    assert!(g.cairns[0].stones > before, "no one laid a stone");
}

#[test]
fn test_no_cairn_for_a_stranger_and_the_cap_holds() {
    let mut g = Game::new();
    // nobody loved them - every bond cut, no pile rises
    for &e in &g.world.query::<relationships::Relationship>() {
        g.world.remove_component::<relationships::Relationship>(e);
    }
    for i in 20..26 { g.died_on[i] = Some(40); }
    g.cairn_tick(48);
    assert!(g.cairns.iter().all(|c| !(20..26).contains(&c.honoree)), "a stranger got a pile");
    // loved ones: at most four piles stand
    for i in 0..8 {
        let h = g.villagers[i];
        let pal = g.villagers[(i + 5) % 12];
        let r = g.world.create_entity();
        g.world.add_component(r, relationships::Relationship { from: pal, other: h, kind: relationships::RelKind::Friend, strength: 0.8, interactions: 5 });
    }
    for _ in 0..8 { g.cairn_tick(48); }
    assert!(g.cairns.len() <= 4, "the village is all piles now");
}
