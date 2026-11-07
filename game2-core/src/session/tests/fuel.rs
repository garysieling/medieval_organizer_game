use crate::session::*;

/// A month out from the cold, a thin woodpile gets a hand out ahead of
/// need - and what comes home goes on the stack before the hearth asks
#[test]
fn test_the_wise_stack_their_wood_before_the_cold() {
    let mut g = Game::new();
    // a home with nothing stacked, and a free hand under its roof
    let home = g.building_entities.iter().copied().find(|&h|
        g.world.get_component::<household::Building>(h)
            .map(|b| b.kind == household::BuildingKind::Home && !b.occupants.is_empty())
            .unwrap_or(false)).expect("a home");
    g.world.get_component_mut::<household::Building>(home).unwrap().stock.clear();
    // a cold-window day in the prep window: the prep sweep runs
    let t = 240 + 1; // tick % 360 == 241, % 3 == 1
    // a bundle in a home satchel goes on the stack
    let occ = g.world.get_component::<household::Building>(home).unwrap().occupants[0];
    let hp = g.world.get_component::<Pos>(home).copied().unwrap();
    if let Some(s) = g.world.get_component_mut::<Satchel>(occ) { s.goods.insert("bavin".into(), 2); }
    *g.world.get_component_mut::<Pos>(occ).unwrap() = Pos { x: hp.x, y: hp.y };
    g.tick = t;
    g.woodpile_tick(t);
    let stacked = g.world.get_component::<household::Building>(home).unwrap()
        .stock.get("bavin").copied().unwrap_or(0);
    assert!(stacked > 0, "the bundle never reached the stack");
    // and a thin pile sends someone out: a gather walk was raised
    let sent = g.travels.iter().any(|tr| {
        g.world.get_component::<household::Building>(home)
            .map(|b| b.occupants.contains(&tr.who)).unwrap_or(false)
    });
    assert!(sent || stacked >= 2, "nobody went for a bundle");
}

/// The renamed good burns and gathers the same: bavin is a fuel, on the
/// same trees, and the hearth takes it
#[test]
fn test_the_bavin_burns_where_the_bundle_did() {
    assert!(crate::game::goods::fills("bavin", crate::game::goods::Tag::Fuel));
    let g = Game::new();
    let hand = g.villagers[0];
    let op = g.world.get_component::<Pos>(hand).copied().unwrap();
    assert!(g.gather_site("bavin", op).is_some(), "no bavin gather site");
}
