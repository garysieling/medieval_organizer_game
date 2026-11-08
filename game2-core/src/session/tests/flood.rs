use crate::session::*;

#[test]
fn test_the_water_takes_the_low_ground() {
    let mut g = Game::new();
    // a spell's flood: the level climbs, the cells fill where the land
    // lies low beside the water
    g.flood.raise(0.30);
    for t in 0..40 { g.flood_tick(1000 + t); }
    assert!(g.flood.active(), "the water never rose");
    // the cells it took are all land that lay low
    for &(x, y) in g.flood.cells.keys() {
        assert_ne!(g.map.get(x, y), Some(crate::game::tilemap::Tile::Water), "the water flooded water");
    }
}

#[test]
fn test_the_flood_turns_folk_round() {
    let mut g = Game::new();
    // flood the road: a tile under the water doesn't take a step
    let from = (g.village.0, g.village.1 + 1);
    g.flood.cells.insert((g.village.0, g.village.1 + 3), 1);
    let path = g.trail_path(from, (g.village.0, g.village.1 + 6));
    // the path either reroutes round the water or can't be made through it
    if let Some(p) = path {
        assert!(!p.contains(&(g.village.0, g.village.1 + 3)), "the path runs through the water");
    }
}

#[test]
fn test_a_crop_under_the_water_is_gone() {
    let mut g = Game::new();
    // find a crop field, drown its tile, tick: the year's work is lost
    let Some(&ce) = g.crop_fields.first() else { return };
    let Some(p) = g.world.get_component::<Pos>(ce).copied() else { return };
    let (x, y) = (p.x.round() as i32, p.y.round() as i32);
    g.world.get_component_mut::<crate::game::villager::Crop>(ce).map(|c| c.growth = 0.5);
    g.flood.cells.insert((x, y), 1);
    g.flood_tick(500);
    assert_eq!(g.world.get_component::<crate::game::villager::Crop>(ce)
        .map(|c| c.growth).unwrap_or(1.0), 0.0, "the field under the water kept its crop");
}
