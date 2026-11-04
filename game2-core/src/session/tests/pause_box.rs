use crate::session::*;
use crate::client::*;

#[test]
fn test_pause_box_is_centred_with_balanced_text() {
    let (x, y, w, h) = pause_rect();
    assert!(((x + w / 2) - MAP_W as i32 / 2).abs() <= 1, "pause box not centred on the map view");
    let mut game = Game::new();
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
