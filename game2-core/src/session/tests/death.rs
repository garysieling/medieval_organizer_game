use crate::session::*;

#[test]
fn test_the_dead_lie_under_a_gravestone_not_standing() {
    let mut game = Game::new();
    let view = View::new(Art::load_fs("../assets", 0));
    while game.tick < 60 { game.update(); }
    let corvin = game.villagers[3];
    // a quiet spot away from the crowd, so no label or figure overlaps the grave
    *game.world.get_component_mut::<Pos>(corvin).unwrap() = Pos { x: 21.0, y: 21.5 };
    game.world.get_component_mut::<Health>(corvin).unwrap().condition = 0.0;
    let t = game.tick;
    while game.tick < t + 2 { game.update(); }
    assert!(game.died_on[3].is_some(), "the day of death should be recorded");

    let buf = view.render(&game.ui_model(), (-1, -1));
    let p = *game.world.get_component::<Pos>(corvin).unwrap();
    let (sx, sy) = iso_to_screen_cam(p.x, p.y, view.cam_x, view.cam_y, view.zoom);
    let px = |x: i32, y: i32| { let i = (y as usize * W + x as usize) * 4; (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]) };
    // the stone is there, beside the carved cross down its middle -
    // checked across a small patch so a passer-by's edge can't hide it
    let stone_shows = (sy - 7..sy - 1).any(|y| (sx - 6..sx + 1)
        .any(|x| px(x, y) == GRAVE_STONE));
    assert!(stone_shows, "no gravestone where Corvin fell");
    // and nothing is standing on it: the space where a living head would be
    // shows no sprite colours from Corvin's figure
    let fig = &view.art.villager_sprites[3];
    let figure_colours: Vec<(u8, u8, u8)> = fig.pixels.chunks(4).filter(|c| c[3] > 128).map(|c| (c[0], c[1], c[2])).collect();
    let head = head_y(sy, view.zoom) + 4;
    assert!(!figure_colours.contains(&px(sx, head)), "Corvin is still drawn standing over his grave");
}
