use crate::*;

#[test]
fn test_hud_speaks_in_days_and_names_the_food_store() {
    let hud = hud_text(Season::Summer, 120, 495, Some("heavy rain"), false, 1);
    assert_eq!(hud, "Summer | day 120 | food store 495 | heavy rain");
    assert!(!hud.contains("t120"));
    assert!(hud_text(Season::Winter, 3, 0, None, true, 1).ends_with("[PAUSED]"));
}

#[test]
fn test_hud_sits_on_a_dark_strip() {
    let mut game = Game::new_seeded(MAP_SEED);
    let view = View::new(Art::load_fs("../assets", 0));
    while game.tick < 30 { game.update(); }
    let buf = view.render(&game.ui_model(), (-1, -1));
    // just past the end of the text, still on the strip
    let hud = hud_text(Season::Summer, game.tick, 0, game.precip.map(|k| k.label()), false, 1);
    let x = 10 + text_width(&hud, 1) - 20;
    let i = ((GLYPH_H + 10) as usize * W + x as usize) * 4;
    assert_eq!((buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]), (20, 20, 32), "no backing strip under the HUD");
}
