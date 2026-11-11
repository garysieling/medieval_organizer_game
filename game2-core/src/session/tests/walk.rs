use crate::session::*;

fn put_player(game: &mut Game, x: f32, y: f32) {
    *game.world.get_component_mut::<Pos>(game.player).unwrap() = Pos { x, y };
}

#[test]
fn test_the_player_walks_straight_through_a_town_building() {
    let mut game = Game::new();
    // a building with open ground either side along +x
    let (bx, by, _) = *game.buildings.iter().find(|&&(bx, by, _)| {
        game.walkable(bx - 1.0, by) && game.walkable(bx - 2.0, by) && game.walkable(bx + 1.0, by)
    }).expect("a building with open ground either side");
    // walking straight at it goes through it and out the far side - a
    // building's walls are for the village, not the player's feet
    put_player(&mut game, bx - 2.0, by);
    for _ in 0..90 { game.walk(1.0, 0.0); }
    let (px, _) = game.player_pos();
    assert!(px.round() > bx, "the player didn't get through the building at {} (stuck at {})", bx, px);
}

#[test]
fn test_a_seat_stuck_inside_a_building_walks_out() {
    // the bug this fix is for: a seat that ends up on a building's tile
    // (a house raised round them, a bad spawn) could never step off -
    // every neighbour read unwalkable and they were pinned for good
    let mut game = Game::new();
    let (bx, by, _) = *game.buildings.iter().find(|&&(bx, by, _)| {
        game.walkable(bx - 1.0, by) || game.walkable(bx + 1.0, by) || game.walkable(bx, by - 1.0) || game.walkable(bx, by + 1.0)
    }).expect("a building with a way out");
    // at least one direction must let them leave the tile entirely
    let can_leave = [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)].iter().any(|&(dx, dy)| {
        put_player(&mut game, bx, by);
        for _ in 0..30 { game.walk(dx, dy); }
        let (px, py) = game.player_pos();
        (px.round() as i32, py.round() as i32) != (bx.round() as i32, by.round() as i32)
    });
    assert!(can_leave, "the seat is still trapped on the building tile ({},{})", bx, by);
}

#[test]
fn test_the_player_still_cant_walk_through_a_wall() {
    let mut game = Game::new();
    // a real wall tile (not a gate - those stay open) with open ground
    // beside it to stand on
    let (wx, wy) = game.map.walls.iter()
        .filter(|&&(_, piece)| !matches!(piece, WallPiece::GateAlongX | WallPiece::GateAlongY))
        .map(|&((x, y), _)| (x, y))
        .find(|&(x, y)| game.map.is_occupied(x, y)
            && (game.walkable_for(0, x as f32 - 1.0, y as f32) || game.walkable_for(0, x as f32 + 1.0, y as f32)))
        .expect("a wall tile with open ground beside it");
    let side = if game.walkable_for(0, wx as f32 - 1.0, wy as f32) { -1.0 } else { 1.0 };
    put_player(&mut game, wx as f32 + side, wy as f32);
    for _ in 0..60 { game.walk(-side, 0.0); }
    let (px, py) = game.player_pos();
    assert!((px.round() as i32, py.round() as i32) != (wx, wy),
        "the player walked through the wall at ({},{}) to ({:.1},{:.1})", wx, wy, px, py);
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
fn test_a_fast_clock_crosses_a_building_cleanly() {
    // at speed 16 a frame's stride is over two tiles: the building's
    // tile must still be stepped through, not hopped past and not
    // caught inside either - half-tile steps carry them cleanly across
    let mut game = Game::new();
    game.speed = 16;
    let (bx, by, _) = *game.buildings.iter().find(|&&(bx, by, _)| {
        game.walkable(bx - 1.0, by) && game.walkable(bx - 3.0, by) && game.walkable(bx + 1.0, by)
    }).expect("a building with open ground either side");
    put_player(&mut game, bx - 3.0, by);
    for _ in 0..60 { game.walk(1.0, 0.0); }
    let (px, _) = game.player_pos();
    assert!(px.round() > bx, "a fast stride stalled on the building at {} (player at {})", bx, px);
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
