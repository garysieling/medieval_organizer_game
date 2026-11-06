use crate::session::*;

fn put_player(game: &mut Game, x: f32, y: f32) {
    *game.world.get_component_mut::<Pos>(game.player).unwrap() = Pos { x, y };
}

#[test]
fn test_the_player_cant_walk_into_a_building_but_can_go_round_behind() {
    let mut game = Game::new();
    // a building with open ground to the west and behind it - the town
    // is built up enough that the first cottage won't do
    let (bx, by, _) = *game.buildings.iter().find(|&&(bx, by, _)| {
        game.walkable(bx - 1.0, by) && game.walkable(bx - 2.0, by)
            && [(bx - 1.0, by), (bx, by - 1.0)].iter().any(|&(x, y)| game.walkable(x, y))
    }).expect("a building with open ground behind it");
    // walking straight at it (along +x) stops at the building's tile
    put_player(&mut game, bx - 2.0, by);
    for _ in 0..60 { game.walk(1.0, 0.0); }
    let (px, _) = game.player_pos();
    assert!(px.round() < bx, "walked into the building at {} (player at {})", bx, px);
    // but the tiles behind it (farther from the viewer) are open ground
    assert!([(bx - 1.0, by), (bx, by - 1.0)].iter().any(|&(x, y)| game.walkable(x, y)), "can't get behind the building");
}

#[test]
fn test_the_player_cant_walk_into_the_sea() {
    let mut game = Game::new();
    put_player(&mut game, 16.0, 16.0);
    for _ in 0..600 { game.walk(1.0, 1.0); }
    let (px, py) = game.player_pos();
    assert!(!matches!(game.map.get(px.round() as i32, py.round() as i32), Some(Tile::Water) | Some(Tile::River) | None),
        "the player walked into the water at ({:.1},{:.1})", px, py);
}

#[test]
fn test_a_still_walk_stick_leaves_the_player_where_they_stood() {
    // a zero Walk command (a held key let go, or noise on the wire)
    // used to divide by zero and NaN the player's position away
    let mut game = Game::new();
    let (x, y) = game.player_pos();
    game.walk(0.0, 0.0);
    let (nx, ny) = game.player_pos();
    assert!(nx.is_finite() && ny.is_finite(), "the player fell off the world ({},{})", nx, ny);
    assert_eq!((nx, ny), (x, y));
}

#[test]
fn test_roads_are_quicker() {
    let mut game = Game::new();
    let road = (0..game.map.height as i32).flat_map(|y| (0..game.map.width as i32).map(move |x| (x, y)))
        .find(|&(x, y)| game.map.get(x, y) == Some(Tile::Road) && game.map.get(x + 1, y) == Some(Tile::Road)).unwrap();
    let grass = (0..game.map.height as i32).flat_map(|y| (0..game.map.width as i32).map(move |x| (x, y)))
        .find(|&(x, y)| game.map.get(x, y) == Some(Tile::Grass) && !game.map.trees.contains(&(x, y)) && game.walkable(x as f32 + 0.2, y as f32)).unwrap();
    let step = |game: &mut Game, (x, y): (i32, i32)| {
        put_player(game, x as f32 - 0.2, y as f32);
        game.walk(1.0, 0.0);
        game.player_pos().0 - (x as f32 - 0.2)
    };
    let (on_road, off_road) = (step(&mut game, road), step(&mut game, grass));
    assert!((on_road - WALK_SPEED * ROAD_SPEEDUP).abs() < 1e-4 && (off_road - WALK_SPEED).abs() < 1e-4, "road {} vs grass {}", on_road, off_road);
}

#[test]
fn test_a_fast_clock_still_stops_at_a_building() {
    // at speed 16 a frame's stride is over two tiles: the wall between
    // here and the far side must still be walked, not leapt
    let mut game = Game::new();
    game.speed = 16;
    let (bx, by, _) = *game.buildings.iter().find(|&&(bx, by, _)| {
        game.walkable(bx - 1.0, by) && game.walkable(bx - 3.0, by) && game.walkable(bx + 1.0, by)
    }).expect("a building with open ground either side");
    put_player(&mut game, bx - 3.0, by);
    for _ in 0..60 { game.walk(1.0, 0.0); }
    let (px, _) = game.player_pos();
    assert!(px.round() < bx, "a fast stride leapt the building at {} (player at {})", bx, px);
}

#[test]
fn test_a_fast_clock_still_stops_at_the_river() {
    let mut game = Game::new();
    game.speed = 16;
    // somewhere the river runs one tile wide, open ground either side
    let river = (0..game.map.height as i32 - 1).flat_map(|y| (0..game.map.width as i32 - 1).map(move |x| (x, y)))
        .find(|&(x, y)| matches!(game.map.get(x, y), Some(Tile::River))
            && game.walkable(x as f32 - 1.0, y as f32) && game.walkable(x as f32 - 3.0, y as f32));
    let Some((rx, ry)) = river else { return };
    put_player(&mut game, rx as f32 - 3.0, ry as f32);
    for _ in 0..60 { game.walk(1.0, 0.0); }
    let (px, _) = game.player_pos();
    assert!(px.round() < rx as f32, "a fast stride crossed the river at {} (player at {})", rx, px);
}
