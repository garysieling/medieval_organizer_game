use crate::session::*;

#[test]
fn test_a_new_house_lays_its_road_a_tile_at_a_time_then_rises() {
    let mut game = Game::new();
    // somewhere a road has to be laid to reach - the town's lanes reach
    // everywhere now, so hunt for a spot the road hasn't got to yet
    let spot = (8..MAP_TILES as i32 - 8).flat_map(|y| (8..MAP_TILES as i32 - 8).map(move |x| (x, y)))
        .find(|&(cx, cy)| {
            (-4..=4).all(|dy| (-4..=4).all(|dx| game.map.get(cx + dx, cy + dy) != Some(Tile::Road)))
        }).expect("a spot out past the roads");
    *game.world.get_component_mut::<Pos>(game.player).unwrap() = Pos { x: spot.0 as f32, y: spot.1 as f32 };
    // the plot a raising picks beside where the player stands, then what
    // the work party's sweep does when the hands arrive (#1344) - the
    // Build button that used to drive this is gone
    let (bx, by) = game.build_site_at(0).expect("a free plot near the player");
    game.buildings.push((bx, by, BSprite::House(0)));
    game.map.build_at(bx as i32, by as i32);
    let road = game.map.plan_road((bx as i32, by as i32));
    game.works.push(Works { building: game.buildings.len() - 1, road, frames: 0 });
    game.built_count += 1;
    let w = game.works.last().expect("building should start works");
    let road = w.road.clone();
    assert!(road.len() >= 1, "a house out here needs a road");
    assert!(road.iter().all(|&(x, y)| game.map.get(x, y) != Some(Tile::Road)), "the road isn't there at once");
    let mut laid_counts = vec![];
    for _ in 0..(road.len() as u32 * ROAD_TILE_FRAMES) {
        game.update();
        laid_counts.push(road.iter().filter(|&&(x, y)| game.map.get(x, y) == Some(Tile::Road)).count());
    }
    assert!(laid_counts.windows(2).all(|p| p[1] >= p[0] && p[1] - p[0] <= 1), "one tile at a time");
    assert_eq!(*laid_counts.last().unwrap(), road.len(), "the whole road is laid");
    assert!(!game.works.is_empty(), "the house is still going up");
    for _ in 0..RISE_FRAMES { game.update(); }
    assert!(game.works.is_empty(), "the house is finished");
}
