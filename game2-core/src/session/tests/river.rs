use crate::session::*;

#[test]
fn test_the_river_can_only_be_crossed_at_the_bridge() {
    let game = Game::new();
    let mut river = 0;
    for y in 0..game.map.height as i32 { for x in 0..game.map.width as i32 {
        match game.map.get(x, y) {
            Some(Tile::River) => { river += 1; assert!(!game.walkable(x as f32, y as f32), "walked onto the river at ({},{})", x, y); }
            Some(Tile::Bridge) => assert!(game.walkable(x as f32, y as f32), "the bridge is blocked"),
            _ => {}
        }
    }}
    assert!(river > 10);
}
