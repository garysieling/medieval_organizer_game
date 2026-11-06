use crate::*;
use game2_core::game::villager::Pos;

#[test]
fn test_rivals_stay_in_their_own_village() {
    let _art = Art::default(); // no sprites needed to run the game, only to draw it
    let mut game = Game::new_seeded(MAP_SEED);
    game.speed = 16; // rivals' pull home scales with ticks_per_frame
    while game.tick < 600 { game.update(); }
    for (i, r) in game.rivals.iter().enumerate() {
        let p = game.world.get_component::<Pos>(*r).unwrap();
        assert!(p.x >= 24.0, "{} wandered into our village at ({:.1},{:.1})", RIVAL_NAMES[i], p.x, p.y);
    }
}
