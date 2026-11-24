use crate::*;
use game2_core::game::villager::Pos;

#[test]
fn test_talk_is_greyed_out_with_no_one_near() {
    let mut game = Game::new_seeded(MAP_SEED);
    let view = View::new(Art::load_fs("../assets", 0));
    // out on the map's edge, past any shout: nobody to talk to
    *game.world.get_component_mut::<Pos>(game.player).unwrap() = Pos { x: 1.0, y: 1.0 };
    assert_eq!(game.action_blocker(0).as_deref(), Some("no one near enough"));
    // the log line must say why, not read as a talk
    game.act(0);
    assert!(game.log.last().unwrap().contains("Can't talk"), "{}", game.log.last().unwrap());
    // and the button is drawn disabled, with the reason beside it
    let buf = view.render(&game.ui_model(), (-1, -1));
    let b = &action_buttons()[0];
    let i = ((b.y + 4) as usize * W + (b.x + 4) as usize) * 4;
    assert_ne!((buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]), b.color, "Talk still drawn in its live colour");
}
