use crate::*;

#[test]
fn test_the_panel_explains_the_map_marks() {
    // the legend fits the panel and sits below the log box
    let chars: usize = MAP_MARKS.iter().map(|(m, _, w)| m.len() + 1 + w.len() + 2).sum();
    assert!(UI_X as i32 + 14 + chars as i32 * GLYPH_W <= W as i32 - 4, "legend runs off the panel");
    assert!(LEGEND_Y >= 450 + 170 && LEGEND_Y + GLYPH_H <= H as i32);

    let mut game = Game::new_seeded(MAP_SEED);
    game.speed = 4;
    let art = Art::load_fs("../assets", 0);
    let view = View::new(art);
    while game.tick < 30 { game.update(); }
    let buf = view.render(&game.ui_model(), (-1, -1));
    // each mark's colour appears in the legend row - or its generated
    // icon does, where the icon set gave one
    for (i, (m, rgb, word)) in MAP_MARKS.iter().enumerate() {
        let found = (LEGEND_Y..LEGEND_Y + GLYPH_H).any(|y| (UI_X..W).any(|x| {
            let i = (y as usize * W + x) * 4;
            (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]) == *rgb
        }));
        assert!(found || view.art.icon(MARK_ICONS[i]).is_some(), "the legend doesn't show '{}' ({})", m, word);
    }
}
