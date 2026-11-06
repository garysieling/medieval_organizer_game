use crate::*;

#[test]
fn test_hud_speaks_in_days_and_names_the_food_store() {
    let hud = hud_text(Season::Summer, 120, 495, Some("heavy rain"), false, 1, 1.0, false);
    assert_eq!(hud, "Summer | day 120 | food store 495 | heavy rain");
    assert!(!hud.contains("t120"));
    assert!(hud_text(Season::Winter, 3, 0, None, true, 1, 1.0, false).ends_with("[PAUSED]"));
    // a rich spring under a harsh winter carries both words
    let both = hud_text(Season::Spring, 400, 200, None, false, 1, 1.1, true);
    assert!(both.contains("rich fields") && both.contains("the store thins"), "{:?}", both);
}

#[test]
fn test_hud_sits_on_a_dark_strip() {
    let mut game = Game::new_seeded(MAP_SEED);
    game.speed = 4;
    let view = View::new(Art::load_fs("../assets", 0));
    while game.tick < 30 { game.update(); }
    let buf = view.render(&game.ui_model(), (-1, -1));
    // just past the end of the text, still on the strip
    let m = game.ui_model();
    let hud = hud_text(Season::Summer, game.tick, 0, game.precip.map(|k| k.label()), false, 1, m.food_yield, m.harsh_winter);
    let x = 10 + text_width(&hud, 1) - 20;
    let i = ((GLYPH_H + 10) as usize * W + x as usize) * 4;
    assert_eq!((buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]), (20, 20, 32), "no backing strip under the HUD");
}

#[test]
fn test_bond_lines_headline_and_breakdown() {
    let nobody = SocialV::default();
    assert_eq!(bond_lines(&nobody), vec!["0 folk you've spoken to".to_string()]);
    let s = SocialV { met: 12, acquaintances: 5, known: 4, followers: 2, coleaders: 1 };
    let lines = bond_lines(&s);
    assert_eq!(lines[0], "12 folk you've spoken to");
    assert_eq!(lines[1], "5 acquaintances · 4 known");
    assert_eq!(lines[2], "2 followers · 1 co-leader");
    for l in &lines { assert!(l.len() <= 46, "the line must fit the panel: {l}"); }
}

#[test]
fn test_meeting_lines_name_the_yeses() {
    let mp = MeetingPlanV {
        coming: vec!["Willa".into(), "Bram".into()],
        spreaders: vec!["Corvin".into()],
        roles: vec!["house: Aldric".into(), "notes: Nell".into()],
    };
    assert_eq!(meeting_lines(&mp), vec![
        "coming: Willa, Bram".to_string(),
        "bringing friends: Corvin".to_string(),
        "house: Aldric · notes: Nell".to_string(),
    ]);
    // an empty plan says nothing
    assert!(meeting_lines(&MeetingPlanV::default()).is_empty());
    // a crowd can't push the roles off: each category holds one row,
    // the overflow counted
    let crowd = MeetingPlanV {
        coming: vec!["Willa".into(), "Bram".into(), "Corvin".into(), "Nell".into(), "Aldric".into(), "Dagny".into()],
        spreaders: vec!["Eliza".into()],
        roles: vec!["house: Hob".into()],
    };
    let lines = meeting_lines(&crowd);
    assert_eq!(lines.len(), 3, "one row per category: {:?}", lines);
    assert!(lines[0].contains("+"), "the overflow is counted: {}", lines[0]);
    assert_eq!(lines[2], "house: Hob");
    for l in &lines { assert!(l.len() <= 40, "one row: {}", l); }
}
