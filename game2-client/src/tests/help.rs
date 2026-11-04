use crate::*;

#[test]
fn test_the_panel_points_to_help() {
    assert!(PANEL_HINT.contains("H help"));
    assert!(UI_X as i32 + 18 + text_width(PANEL_HINT, 1) < W as i32 - 20, "hint runs off the panel card");
}

#[test]
fn test_help_box_is_centred_on_the_map_and_fits_its_text() {
    let (x, y, w, h) = help_rect();
    assert!(((x + w / 2) - MAP_W as i32 / 2).abs() <= 1, "help box not centred on the map view");
    assert!(x >= 0 && x + w <= MAP_W as i32 && y >= 0 && y + h <= H as i32);
    let widest = HELP_LINES.iter().map(|l| text_width(l, 1)).max().unwrap();
    assert!(widest + 40 <= w, "help text too wide for its box");

    // in a real frame, the drawn text inside the box is balanced left/right
    let game = Game::new_seeded(MAP_SEED);
    let mut view = View::new(Art::load_fs("../assets", 0));
    view.help_open = true;
    let buf = view.render(&game.ui_model(), (-1, -1));
    let (mut lo, mut hi) = (i32::MAX, i32::MIN);
    for yy in y + 10..y + h - 5 { for xx in x + 4..x + w - 4 {
        let i = (yy as usize * W + xx as usize) * 4;
        if buf.pixels[i] > 140 { lo = lo.min(xx); hi = hi.max(xx); }
    }}
    let (left, right) = (lo - x, x + w - hi);
    assert!((left - right).abs() <= 12, "help text hugs one side: {}px left, {}px right", left, right);
}
