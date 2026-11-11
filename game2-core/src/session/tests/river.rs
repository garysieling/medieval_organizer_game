use crate::session::*;

#[test]
fn test_the_river_can_only_be_crossed_at_the_bridge() {
    let game = Game::new();
    let mut river = 0;
    let mut bridges = 0;
    let mut fords = 0;
    for y in 0..game.map.height as i32 { for x in 0..game.map.width as i32 {
        match game.map.get(x, y) {
            Some(Tile::River) => { river += 1; assert!(!game.walkable(x as f32, y as f32), "walked onto the river at ({},{})", x, y); }
            Some(Tile::Bridge) => { bridges += 1; assert!(game.walkable(x as f32, y as f32), "the bridge is blocked"); }
            Some(Tile::Ford) => { fords += 1; assert!(game.walkable(x as f32, y as f32), "the ford should be wadeable"); }
            _ => {}
        }
    }}
    assert!(river > 10);
    // the river runs through more than once: two dry crossings and two
    // wading stretches go in at setup (#1074)
    assert!(bridges >= 2, "only {} bridge tile(s) were placed", bridges);
    assert_eq!(fords, 2, "the seeded fords didn't land");
}

#[test]
fn test_the_ford_is_the_short_way_over_until_the_water_rises() {
    let game = Game::new();
    let (fx, fy) = {
        let mut f = None;
        for y in 0..game.map.height as i32 { for x in 0..game.map.width as i32 {
            if game.map.get(x, y) == Some(Tile::Ford) { f = Some((x, y)); break; }
        }}
        f.expect("no ford on the map")
    };
    // the ford's banks: passable neighbours on opposite sides
    let banks: Vec<(i32, i32)> = crate::game::tilemap::NEIGHBOURS.iter().map(|&(dx, dy)| (fx + dx, fy + dy))
        .filter(|&(x, y)| game.map.passable(x, y)).collect();
    assert!(banks.len() >= 2, "the ford has no wadeable banks");
    let empty = crate::engine::det::HashMap::default();
    let dry = Game::trail_path_on(&game.map, &empty, &game.flood, banks[0], banks[1]).expect("no way over");
    assert!(dry.contains(&(fx, fy)), "the nearest crossing is a wade through the ford");
    // the same walk when the water's up: the wading stretch is under it
    // and the path has to find dry crossings instead (#1074)
    let mut flooded = crate::game::flood::Flood::default();
    flooded.cells.insert((fx, fy), 2);
    if let Some(path) = Game::trail_path_on(&game.map, &empty, &flooded, banks[0], banks[1]) {
        assert!(!path.contains(&(fx, fy)), "walked the ford while it was under water");
    }
}
