use crate::*;

#[test]
fn test_notice_lines_wrap_a_long_posting() {
    let notices = vec![("writ".to_string(),
        "a fee of twelve gold for every barrel the river brought, set by the steward on market day".to_string())];
    let lines = notice_lines(&notices);
    assert!(lines.len() > 1, "a long posting didn't wrap");
    assert!(lines[0].starts_with("[writ]"), "the kind didn't lead: {}", lines[0]);
    for l in &lines {
        assert!(text_width(l, 1) < 480, "a wrapped row runs off the box: {}", l);
    }
}

#[test]
fn test_notice_lines_cap_with_a_more_tail() {
    let notices: Vec<(String, String)> = (0..40).map(|i| ("quest".to_string(),
        format!("posting {} asks a long favour of anyone who'll read this far into the board", i))).collect();
    let lines = notice_lines(&notices);
    assert_eq!(lines.len(), NOTICE_MAX, "the list didn't cap");
    assert!(lines.last().unwrap().starts_with('+'), "no '+n more' tail: {}", lines.last().unwrap());
    // and the box they size stays on screen
    let (x, y, w, h) = overlay_rect(480, lines.len(), NOTICE_ROW, true);
    assert!(x >= 0 && x + w <= MAP_W as i32 && y >= 0 && y + h <= H as i32,
        "the board's box escapes the map view: {}+{}x{}+{}", x, w, y, h);
}

#[test]
fn test_the_board_panel_draws_inside_the_map_view() {
    let mut game = Game::new_seeded(MAP_SEED);
    game.seats[0].notice_panel = true;
    for i in 0..40 {
        game.notices.push(("writ".to_string(),
            format!("a posting numbered {} with enough words to need the wrap, set by the steward", i),
            u64::MAX));
    }
    let view = View::new(Art::load_fs("../assets", 0));
    let m = game.ui_model();
    let lines = notice_lines(&m.notices);
    let (x, y, w, h) = overlay_rect(480, lines.len(), NOTICE_ROW, true);
    let buf = view.render(&m, (-1, -1));
    let px = |xx: i32, yy: i32| buf.pixels[(yy as usize * W + xx as usize) * 4];
    assert!(px(x + w / 2, y + 1) > 100, "the board's top bar never drew");
    // the title row and a body row drew their light text inside the box
    let lit = (y + 6..y + h - 6).any(|yy| (x + 6..x + w - 6).any(|xx| px(xx, yy) > 180));
    assert!(lit, "no posting text drew inside the box");
}
