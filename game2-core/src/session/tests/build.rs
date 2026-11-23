use crate::session::*;

#[test]
fn test_build_sites_stay_clear_and_near_the_player() {
    // the raising party (#1344) picks its plot through the same
    // build_site_at the dropped Build button used - keep its contract:
    // dry buildable ground near the player, clear of every building,
    // the windmill and the player himself, and a fresh plot each time
    let mut game = Game::new();
    let (px, py) = game.player_pos();
    let mut sites = 0;
    while let Some((x, y)) = game.build_site_at(0) {
        sites += 1;
        assert!(sites < 200, "build_site_at never ran out of plots");
        let tile = game.map.get(x as i32, y as i32);
        assert!(matches!(tile, Some(Tile::Grass) | Some(Tile::Hill)), "house on {:?} at ({},{})", tile, x, y);
        assert!((x - px).abs() + (y - py).abs() <= 2.0 * BUILD_SEARCH as f32, "house far from the player");
        let clear_of = |bx: f32, by: f32| (bx.round() - x).abs() > 1.0 || (by.round() - y).abs() > 1.0;
        assert!(game.buildings.iter().all(|(bx, by, _)| clear_of(*bx, *by)), "house on or beside an existing building at ({},{})", x, y);
        assert!(clear_of(game.mill.0, game.mill.1), "house on the windmill at ({},{})", x, y);
        assert!((x - px.round()).abs() + (y - py.round()).abs() >= 2.0, "house right on top of the player at ({},{})", x, y);
        // stand the house up so the next pick has to go elsewhere
        game.buildings.push((x, y, BSprite::House(0)));
        game.map.build_at(x as i32, y as i32);
    }
    assert!(sites > 0, "no build site found at all");
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
