use crate::session::*;
use crate::client::*;

#[test]
fn test_log_entries_show_the_day_like_the_hud() {
    assert_eq!(log_parts("t120: Bram is taken with la grippe"), (Some("day 120".into()), "Bram is taken with la grippe"));
    assert_eq!(log_parts("Welcome to the village."), (None, "Welcome to the village."));
    assert_eq!(log_parts("tea: not a day"), (None, "tea: not a day"));
}

#[test]
fn test_the_log_draws_the_day_dim_and_the_message_in_colour() {
    let mut game = Game::new();
    let view = View::new(Art::load_fs("../assets", 0));
    game.log.push("t5: The Merchant Guild formed!".into());
    let buf = view.render(&game.ui_model(), (-1, -1));
    let row = |x0: i32, x1: i32, rgb: (u8, u8, u8)| (530..530 + GLYPH_H).any(|y| (x0..x1).any(|x| {
        let i = (y as usize * W + x as usize) * 4;
        (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]) == rgb
    }));
    let x = UI_X as i32 + 14;
    let msg_x = x + 6 * GLYPH_W; // after "day 5 "
    assert!(row(x, msg_x - GLYPH_W, (120, 120, 145)), "the day stamp should be dim");
    assert!(!row(x, msg_x - GLYPH_W, (255, 215, 0)), "the day stamp shouldn't take the message colour");
    assert!(row(msg_x, msg_x + 10 * GLYPH_W, (255, 215, 0)), "the guild message keeps its gold");
}
