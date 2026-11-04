use crate::client::*;
use crate::game::villager::Pos;
use crate::sim::community::Organization;

#[test]
fn test_recruit_is_greyed_out_for_a_member() {
    let mut game = Game::new_seeded(MAP_SEED);
    let view = View::new(Art::load_fs("../assets", 0));
    let v = game.villagers[2];
    game.world.get_component_mut::<Organization>(game.org).unwrap().members.push(v);
    // beside them, so Recruit doesn't turn to someone nearer (reach_target)
    let (vx, vy) = { let p = game.world.get_component::<Pos>(v).unwrap(); (p.x, p.y) };
    *game.world.get_component_mut::<Pos>(game.player).unwrap() = Pos { x: vx + 0.5, y: vy };
    game.selected_target = 2;
    assert_eq!(game.action_blocker(1).as_deref(), Some("already a member"));
    // the log line mustn't read as a success (log_colour greens "joined")
    game.act(1);
    assert_ne!(log_colour(game.log.last().unwrap()), (120, 255, 120), "{}", game.log.last().unwrap());
    // and the button is drawn disabled, with the reason beside it
    let buf = view.render(&game.ui_model(), (-1, -1));
    let b = &action_buttons()[1];
    let i = ((b.y + 4) as usize * W + (b.x + 4) as usize) * 4;
    assert_ne!((buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]), b.color, "Recruit still drawn in its live colour");
}
