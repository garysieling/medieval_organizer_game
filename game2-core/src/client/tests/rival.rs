use crate::client::*;
use crate::game::villager::Pos;

#[test]
fn test_rivals_stay_in_their_own_village() {
    let _art = Art::default(); // no sprites needed to run the game, only to draw it
    let mut game = Game::new_seeded(MAP_SEED);
    while game.tick < 600 { game.update(); }
    for (i, r) in game.rivals.iter().enumerate() {
        let p = game.world.get_component::<Pos>(*r).unwrap();
        assert!(p.x >= 24.0, "{} wandered into our village at ({:.1},{:.1})", RIVAL_NAMES[i], p.x, p.y);
    }
}
