//! Regression checks on the frame a player actually sees: real sprites,
//! the real Game, Game::render(). (playtest_gate's UI checks use their own
//! stripped-down renderer and passed even when the real panel was empty.)
use crate::client::*;
use crate::engine::render::Buffer;

/// Pixels in a box that differ from the given background colour
fn drawn(buf: &Buffer, x0: usize, y0: usize, x1: usize, y1: usize, bg: (u8, u8, u8)) -> usize {
    let mut n = 0;
    for y in y0..y1 { for x in x0..x1 {
        let i = (y * W + x) * 4;
        if (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]) != bg { n += 1; }
    }}
    n
}

#[test]
fn test_rocks_read_as_rocks_not_holes() {
    // the pack's stone(1) is a tiny pebble that's nearly all dark outline:
    // on the map it read as a black hole in the ground
    let art = Art::load_fs("../assets", 0);
    assert!(!art.stones.is_empty(), "stone sprites should load");
    for stone in art.stones.iter() {
        let opaque: Vec<&[u8]> = stone.pixels.chunks(4).filter(|p| p[3] > 128).collect();
        // every stone has a thick dark outline; what makes it read as a rock
        // is a visible light face (stone(1): 10% of its pixels, the boulders 25-33%)
        let light = opaque.iter().filter(|p| (p[0] as u32 + p[1] as u32 + p[2] as u32) > 300).count();
        assert!(light * 5 >= opaque.len(), "stone sprite shows almost no rock face ({} of {} px light)", light, opaque.len());
    }
}

#[test]
fn test_real_frames_show_panel_log_minimap_and_hud() {
    // tests run from game2-core/, the art lives in the repo's assets/
    assert!(std::path::Path::new("../assets").exists(), "assets folder not found from {:?}", std::env::current_dir());
    let mut game = Game::new_seeded(MAP_SEED);
    let view = View::new(Art::load_fs("../assets", 0));
    // day 1, la grippe (150), rain (200, from the weather sim) and deep winter (330)
    for day in [1u64, 150, 200, 330] {
        while game.tick < day { game.update(); }
        let buf = view.render(&game.ui_model(), (-1, -1));
        let panel = drawn(&buf, UI_X, 0, W, H, (32, 32, 48));
        assert!(panel > (W - UI_X) * H / 6, "day {}: side panel is mostly empty ({} px drawn)", day, panel);
        let log = drawn(&buf, UI_X + 12, 466, UI_X + 278, 618, (25, 25, 40));
        assert!(log > 300, "day {}: event log shows no text ({} px)", day, log);
        let minimap = drawn(&buf, MM_LEFT as usize, MM_TOP as usize, (MM_LEFT + MM_W) as usize, (MM_TOP + MM_H) as usize, (20, 20, 30));
        assert!(minimap > (MM_W * MM_H) as usize / 3, "day {}: minimap has no terrain ({} px)", day, minimap);
        // the HUD line (season, day, food) is text over the map's top-left
        let hud_text = (0..120).flat_map(|x| (6..16).map(move |y| (x, y)))
            .filter(|&(x, y)| { let i = (y * W + x) * 4; buf.pixels[i] > 230 && buf.pixels[i + 1] > 230 && buf.pixels[i + 2] > 230 })
            .count();
        assert!(hud_text > 40, "day {}: HUD text missing ({} white px)", day, hud_text);
    }
}
