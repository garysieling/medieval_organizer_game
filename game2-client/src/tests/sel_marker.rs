use crate::*;

#[test]
fn a_clicked_building_wears_the_dart() {
    let game = Game::new_seeded(MAP_SEED);
    let view = View::new(Art::load_fs("../assets", 0));
    // a building well away from the player so its dart can't be the
    // player's own
    let (px, py) = game.player_pos();
    let m = game.ui_model();
    let (bi, bx, by) = m.buildings.iter().enumerate()
        .map(|(i, b)| (i, b.x, b.y))
        .filter(|&(_, x, y)| ((x - px).powi(2) + (y - py).powi(2)).sqrt() > 10.0)
        .next().expect("a building away from the player");
    let gold = |(sx, sy): (i32, i32), buf: &game2_core::engine::render::Buffer| {
        let cy = sy - (34.0 * view.zoom).round() as i32;
        let mut n = 0;
        for y in (cy - 12).max(0)..(cy + 4).min(buf.height as i32) {
            for x in (sx - 8).max(0)..(sx + 8).min(MAP_W as i32) {
                let k = (y as usize * W + x as usize) * 4;
                if (buf.pixels[k], buf.pixels[k + 1], buf.pixels[k + 2]) == (255, 215, 0) { n += 1; }
            }
        }
        n
    };
    let spot = iso_to_screen_cam(bx, by, view.cam_x, view.cam_y, view.zoom);
    // unselected: no dart over the building
    let buf = view.render(&m, (-1, -1));
    assert_eq!(gold(spot, &buf), 0, "a dart with nothing picked");
    // clicked: the dart rides it
    let mut m2 = m;
    m2.sel.building = Some(bi);
    let buf = view.render(&m2, (-1, -1));
    assert!(gold(spot, &buf) > 8, "the picked building wears the dart");
}
