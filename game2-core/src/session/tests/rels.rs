use crate::session::*;
use crate::game::relationships::*;

#[test]
fn test_bad_blood_settles_and_the_label_comes_off() {
    let mut g = Game::new();
    let (a, b) = (g.villagers[0], g.villagers[1]);
    // enough quarrels to make them rivals
    for _ in 0..6 {
        record_interaction(&mut g.world, a, Interaction {
            other: b, kind: InteractionKind::Conflict, intensity: 0.8 });
    }
    let rel = find_relationship(&g.world, a, b).expect("no tie");
    assert_eq!(g.world.get_component::<Relationship>(rel).unwrap().kind, RelKind::Rival);
    // a year and a half of quiet days settles it
    for t in 0..560 { g.rel_tick(); let _ = t; }
    let r = g.world.get_component::<Relationship>(rel).unwrap();
    assert_ne!(r.kind, RelKind::Rival, "the rivalry outlived the bad blood");
    assert!(r.strength > -0.3, "the strength held at {}", r.strength);
    // and a warm tie cools toward acquaintance too
    for _ in 0..9 {
        record_interaction(&mut g.world, a, Interaction {
            other: b, kind: InteractionKind::SharedMeal, intensity: 0.8 });
    }
    let s0 = g.world.get_component::<Relationship>(rel).unwrap().strength;
    for _ in 0..30 { g.rel_tick(); }
    let s1 = g.world.get_component::<Relationship>(rel).unwrap().strength;
    assert!(s1 < s0, "a warm tie didn't cool: {} -> {}", s0, s1);
}

#[test]
fn test_the_feast_half_hearts_a_few_random_souls() {
    let mut g = Game::new();
    // every bond's strength before the feast
    let mut before = crate::engine::det::HashMap::default();
    for e in g.world.query::<Relationship>() {
        let r = g.world.get_component::<Relationship>(e).unwrap();
        before.insert((r.from.min(r.other), r.from.max(r.other)), r.strength);
    }
    g.festival_tick(45);
    // each pair the feast warmed gained half a heart at most - one
    // bump a pair, however the crowd presses in
    let mut warmed = 0;
    for e in g.world.query::<Relationship>() {
        let r = g.world.get_component::<Relationship>(e).unwrap();
        let key = (r.from.min(r.other), r.from.max(r.other));
        let gain = r.strength - before.get(&key).copied().unwrap_or(0.0);
        if gain > 1e-6 {
            warmed += 1;
            assert!(gain <= 0.1 + 1e-4, "a feast warms a pair by half a heart, got {}", gain);
        }
    }
    assert!(warmed >= 10, "the feast only warmed {} pairs", warmed);
}

#[test]
fn test_a_player_at_the_feast_leaves_with_half_hearts() {
    let mut g = Game::new();
    // stand the player in the market crowd
    let mp = g.world.get_component::<Pos>(g.market_food).map(|p| (p.x, p.y)).unwrap();
    *g.world.get_component_mut::<Pos>(g.player).unwrap() = Pos { x: mp.0, y: mp.1 };
    g.festival_tick(45);
    let warmed = g.world.query::<Relationship>().iter()
        .filter(|&&e| g.world.get_component::<Relationship>(e)
            .map(|r| (r.from == g.player || r.other == g.player) && r.strength > 0.0).unwrap_or(false))
        .count();
    assert!(warmed >= 1 && warmed <= 3,
        "the feast gave the player {} half hearts", warmed);
}

#[test]
fn test_the_dead_cant_warm_the_feast() {
    let mut g = Game::new();
    // the player lies in state near the market on feast day
    let mp = g.world.get_component::<Pos>(g.market_food).map(|p| (p.x, p.y)).unwrap();
    *g.world.get_component_mut::<Pos>(g.player).unwrap() = Pos { x: mp.0, y: mp.1 };
    g.world.add_component(g.player, villager::Dead);
    g.festival_tick(45);
    let warmed = g.world.query::<Relationship>().iter()
        .filter(|&&e| g.world.get_component::<Relationship>(e)
            .map(|r| (r.from == g.player || r.other == g.player) && r.strength > 0.0).unwrap_or(false))
        .count();
    assert_eq!(warmed, 0, "the dead don't join the feast: {} bonds", warmed);
}

#[test]
fn test_the_card_counts_hearts_in_halves() {
    let mut g = Game::new();
    g.seats[0].selected_target = 3;
    // a stranger's card shows empty hearts
    assert_eq!(g.villager_inspector_at(0).hearts, Some(0.0));
    // one full talk is half a heart
    let pe = g.player;
    record_interaction(&mut g.world, pe, Interaction {
        other: g.villagers[3], kind: InteractionKind::Conversation, intensity: 1.0 });
    assert_eq!(g.villager_inspector_at(0).hearts, Some(0.5));
    // and a deed done for them is two more
    record_interaction(&mut g.world, pe, Interaction {
        other: g.villagers[3], kind: InteractionKind::Aid, intensity: 1.0 });
    assert_eq!(g.villager_inspector_at(0).hearts, Some(2.5));
}
