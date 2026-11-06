use crate::*;
use game2_core::game::villager::Pos;
use game2_core::sim::illness::Health;

#[test]
fn test_the_dead_lie_under_a_gravestone_not_standing() {
    let mut game = Game::new_seeded(MAP_SEED);
    let view = View::new(Art::load_fs("../assets", 0));
    while game.tick < 60 { game.update(); }
    let corvin = game.villagers[3];
    game.world.get_component_mut::<Health>(corvin).unwrap().condition = 0.0;
    let t = game.tick;
    while game.tick < t + 2 { game.update(); }
    assert!(game.died_on[3].is_some(), "the day of death should be recorded");

    // quiet spots near the market, nearest first: buildings stand ~2 tiles
    // wide now, so a spot clear of the crowd can still sit inside a
    // neighbour's sprite - try each until the stone shows through
    let m = game.ui_model();
    let mut spots: Vec<(f32, f32)> = (4..48).flat_map(|y| (4..48).map(move |x| (x as f32 + 0.5, y as f32 + 0.5)))
        .filter(|&(x, y)| {
            let (tx, ty) = (x.round() as i32, y.round() as i32);
            m.map.passable(tx, ty) && !m.map.is_occupied(tx, ty)
        })
        .collect();
    spots.sort_by_key(|&(x, y)| (((x - 16.0).powi(2) + (y - 16.0).powi(2)) * 100.0) as i64);

    // the stone is there (the generated one's colours, else the drawn
    // stone's) - checked across a small patch so a passer-by's edge
    // can't hide it
    let stone: Vec<(u8, u8, u8)> = match variant(&view.art.graves, corvin) {
        Some(g) => g.pixels.chunks(4).filter(|c| c[3] > 128).map(|c| (c[0], c[1], c[2])).collect(),
        None => vec![GRAVE_STONE],
    };
    let figure_colours: Vec<(u8, u8, u8)> = view.art.villager_sprites[3].pixels.chunks(4)
        .filter(|c| c[3] > 128).map(|c| (c[0], c[1], c[2])).collect();
    let mut shown = false;
    for &(x, y) in spots.iter().take(60) {
        *game.world.get_component_mut::<Pos>(corvin).unwrap() = Pos { x, y };
        let buf = view.render(&game.ui_model(), (-1, -1));
        let (sx, sy) = iso_to_screen_cam(x, y, view.cam_x, view.cam_y, view.zoom);
        let px = |x: i32, y: i32| { let i = (y as usize * W + x as usize) * 4; (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]) };
        let stone_shows = (sy - 7..sy - 1).any(|yy| (sx - 6..sx + 1).any(|xx| stone.contains(&px(xx, yy))));
        if !stone_shows { continue; }
        // and nothing is standing on it: the space where a living head
        // would be shows no sprite colours from Corvin's figure
        let head = head_y(sy, view.zoom) + 4;
        assert!(!figure_colours.contains(&px(sx, head)), "Corvin is still drawn standing over his grave");
        shown = true;
        break;
    }
    assert!(shown, "no gravestone where Corvin fell");
}
