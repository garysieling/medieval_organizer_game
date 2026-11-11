use crate::*;

#[test]
fn test_pause_box_is_centred_with_balanced_text() {
    let (x, y, w, h) = pause_rect();
    assert!(((x + w / 2) - MAP_W as i32 / 2).abs() <= 1, "pause box not centred on the map view");
    let mut game = Game::new_seeded(MAP_SEED);
    let view = View::new(Art::load_fs("../assets", 0));
    game.paused = true;
    let buf = view.render(&game.ui_model(), (-1, -1));
    let (mut lo, mut hi) = (i32::MAX, i32::MIN);
    for yy in y + 10..y + h - 4 { for xx in x + 4..x + w - 4 {
        let i = (yy as usize * W + xx as usize) * 4;
        if buf.pixels[i] > 140 { lo = lo.min(xx); hi = hi.max(xx); }
    }}
    let (left, right) = (lo - x, x + w - hi);
    // was ~20px left vs ~120px right, and the box centred at x 410 not 330
    assert!((left - right).abs() <= 12, "pause text hugs one side: {}px left, {}px right", left, right);
}

#[test]
fn test_the_pause_menu_centres_and_its_rows_click() {
    let (x, y, w, h) = pause_menu_box();
    assert!(((x + w / 2) - MAP_W as i32 / 2).abs() <= 1, "menu not centred on the map view");
    let rows = pause_menu_rows();
    assert!(rows.iter().all(|&(rx, ry, rw, rh, _)|
            rx >= x && rx + rw <= x + w && ry >= y && ry + rh <= y + h),
        "a row sits inside the box");
    let (rx, ry, ..) = rows[0];
    assert_eq!(pause_menu_click(rx + 2, ry + 2), Some(MenuPick::Resume));
    let (rx, ry, rw, rh, _) = rows[1];
    assert_eq!(pause_menu_click(rx + rw - 1, ry + rh - 1), Some(MenuPick::Quit));
    assert_eq!(pause_menu_click(x - 5, y), None, "outside the box presses nothing");
    assert_eq!(pause_menu_click(rx, ry + rh + 1), None, "between the rows presses nothing");
}

#[test]
fn test_the_pause_menu_draws_its_rows() {
    let mut buf = Buffer::new(W, H);
    draw_pause_menu(&mut buf, (-1, -1), "PAUSED");
    let px = |xx: i32, yy: i32| buf.pixels[(yy as usize * W + xx as usize) * 4];
    let (x, y, w, _) = pause_menu_box();
    assert!(px(x + w / 2, y + 1) > 100, "the top bar drew");
    for &(rx, ry, rw, _, _) in &pause_menu_rows() {
        assert!(px(rx + rw / 2, ry) > 60, "a row's frame drew");
    }
}
