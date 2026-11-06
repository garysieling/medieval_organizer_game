use crate::session::*;

#[test]
fn test_build_costs_gold_and_uses_free_land_near_the_player() {
    let mut game = Game::new();
    let gold0 = game.world.get_component::<PlayerCharacter>(game.player).unwrap().resources;
    let start = game.buildings.len();
    let mut built = 0;
    while game.action_blocker(2).is_none() {
        game.act(2);
        built += 1;
        assert!(built < 100, "Build never stopped");
    }
    assert_eq!(built, (gold0 / BUILD_COST) as usize, "one house per {} gold", BUILD_COST);
    let gold = game.world.get_component::<PlayerCharacter>(game.player).unwrap().resources;
    assert!((gold0 - gold - built as f32 * BUILD_COST).abs() < 1e-3, "gold should drop by the cost of each house");
    assert_eq!(game.action_blocker(2), Some(format!("need {:.0} gold", BUILD_COST)));
    let (px, py) = game.player_pos();
    let new: Vec<_> = game.buildings[start..].iter().map(|(x, y, _)| (*x, *y)).collect();
    for (i, &(x, y)) in new.iter().enumerate() {
        let tile = game.map.get(x as i32, y as i32);
        assert!(matches!(tile, Some(Tile::Grass) | Some(Tile::Hill)), "house on {:?} at ({},{})", tile, x, y);
        assert!((x - px).abs() + (y - py).abs() <= 2.0 * BUILD_SEARCH as f32, "house far from the player");
        let clear_of = |bx: f32, by: f32| (bx.round() - x).abs() > 1.0 || (by.round() - y).abs() > 1.0;
        assert!(game.buildings[..start].iter().all(|(bx, by, _)| clear_of(*bx, *by)), "house on or beside an existing building at ({},{})", x, y);
        assert!(clear_of(game.mill.0, game.mill.1), "house on the windmill at ({},{})", x, y);
        assert!((x - px.round()).abs() + (y - py.round()).abs() >= 2.0, "house right on top of the player at ({},{})", x, y);
        assert!(new[..i].iter().all(|&(ox, oy)| clear_of(ox, oy)), "two houses on or beside each other at ({},{})", x, y);
    }
}

#[test]
fn test_a_house_gone_derelict_is_said_once() {
    let mut g = Game::new();
    // empty the smithy of its worker - a workplace nobody works
    let shop = g.building_entities.iter().copied().find(|&e| {
        g.world.get_component::<Building>(e)
            .map(|b| household::is_workplace(b.kind) && b.worker.is_some())
            .unwrap_or(false)
    }).expect("no staffed workplace");
    let name = g.world.get_component::<Building>(shop).unwrap().name.clone();
    // let whatever was already standing empty say its piece first
    g.livelihood_tick(2);
    let before = g.log.iter().filter(|l| l.contains(&format!("{} stands derelict", name))).count();
    g.world.get_component_mut::<Building>(shop).unwrap().worker = None;
    g.livelihood_tick(3);
    assert_eq!(g.log.iter().filter(|l| l.contains(&format!("{} stands derelict", name))).count(), before + 1,
        "the derelict wasn't said, or said twice");
    // and it isn't said again next day
    g.livelihood_tick(4);
    assert_eq!(g.log.iter().filter(|l| l.contains(&format!("{} stands derelict", name))).count(), before + 1,
        "it was said again");
}
