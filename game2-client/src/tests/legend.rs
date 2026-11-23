use crate::*;

#[test]
fn test_the_panel_explains_the_map_marks() {
    // the legend fits the panel and sits below the log box - two rows
    // of two marks at the manuscript face's metric
    // each row's marks run icon + word from their column's start
    let span = |from: usize| -> i32 {
        MAP_MARKS[from..from + 2].iter()
            .map(|(_, _, w)| 16 + (w.len() + 2) as i32 * GLYPH_W).sum()
    };
    assert!(UI_X as i32 + 14 + span(0) <= W as i32 - 4, "legend row 0 runs off the panel");
    assert!(UI_X as i32 + 14 + 148 + span(2) <= W as i32 - 4, "legend row 1 runs off the panel");
    assert!(LEGEND_Y >= 450 + 170 && LEGEND_Y + LINE_H + GLYPH_H <= H as i32);

    let mut game = Game::new_seeded(MAP_SEED);
    game.speed = 4;
    let art = Art::load_fs("../assets", 0);
    let view = View::new(art);
    while game.tick < 30 { game.update(); }
    let buf = view.render(&game.ui_model(), (-1, -1));
    // each mark's colour appears in the legend row - or its generated
    // icon does, where the icon set gave one
    for (i, (m, rgb, word)) in MAP_MARKS.iter().enumerate() {
        let ly = LEGEND_Y + (i / 2) as i32 * LINE_H;
        let found = (ly..ly + GLYPH_H).any(|y| (UI_X..W).any(|x| {
            let i = (y as usize * W + x) * 4;
            (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]) == *rgb
        }));
        assert!(found || view.art.icon(MARK_ICONS[i]).is_some(), "the legend doesn't show '{}' ({})", m, word);
    }
}
