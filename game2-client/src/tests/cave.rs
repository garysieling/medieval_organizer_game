use crate::*;
use game2_core::session::Game;

#[test]
fn test_the_cave_zone_draws_darker_than_the_world_around_it() {
    let game = Game::new_seeded(MAP_SEED);
    let view = View::new(Art::load_fs("../assets", 0));
    let m = game.ui_model();
    let Some(((ax, ay), (bx, by))) = m.cave_zone else { panic!("no cave zone") };
    // a tile at the zone's heart vs one inland of the mouth - the under
    // should read darker where the cave's shade lies
    let inside = ((ax + bx) as f32 / 2.0, (ay + by) as f32 / 2.0);
    let outside = (ax as f32 - 4.0, ay as f32 - 4.0);
    let (six, siy) = iso_to_screen_cam(inside.0, inside.1, 0, 0, 1.0);
    let (sox, soy) = iso_to_screen_cam(outside.0, outside.1, 0, 0, 1.0);
    let mut buf = Buffer::new(MAP_W, H);
    buf.rect(0, 0, MAP_W as i32, H as i32, 25, 25, 38);
    view.draw_ground(&m, &mut buf, 0, 0, 1.0);
    let px = |x: i32, y: i32| -> (u8, u8, u8) {
        let i = (y as usize * MAP_W + x as usize) * 4;
        (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2])
    };
    let (p_in, p_out) = (px(six, siy), px(sox, soy));
    let bright = |p: (u8, u8, u8)| p.0 as u32 + p.1 as u32 + p.2 as u32;
    assert!(bright(p_in) < bright(p_out),
        "the cave tile {:?} doesn't read darker than outside {:?}", p_in, p_out);
}
