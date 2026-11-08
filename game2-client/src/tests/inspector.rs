use crate::*;

#[test]
fn test_fullness_reads_the_right_way_round() {
    // Hunger stores fullness: a low number is a hungry villager
    assert_eq!(fullness_word(95.0), "full");
    assert_eq!(fullness_word(24.0), "hungry");
    assert_eq!(fullness_word(5.0), "starving");
    assert_eq!(meter_colour(0.24), (230, 80, 70), "a hungry villager's bar should be red");
    assert_eq!(meter_colour(0.95), (90, 200, 90));
}

/// The card's heart strip, top-right: empty hearts for a stranger, and
/// a talk's half heart lights the first one halfway
#[test]
fn test_the_card_shows_the_seats_hearts() {
    let mut game = Game::new_seeded(MAP_SEED);
    let view = View::new(Art::load_fs("../assets", 0));
    game.selected_target = 3;
    // the card floats on the map's bottom-left, its plate sized to its
    // rows; find its top by the first row that's plate edge to edge,
    // then the hearts sit in the band under the name row
    let card_top = |buf: &Buffer| -> usize {
        (300..940).find(|&y| (80..370usize).all(|x| {
            let i = (y * W + x) * 4;
            (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]) == (45, 45, 65)
        })).expect("the card's plate never drew")
    };
    let count = |buf: &Buffer, rgb: (u8, u8, u8)| -> usize {
        let top = card_top(buf);
        let mut n = 0;
        for y in top..top + 14 { for x in 330..376 {
            let i = (y as usize * W + x as usize) * 4;
            if (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]) == rgb { n += 1; }
        }}
        n
    };
    // a stranger: five dark hearts, nothing pink
    let buf = view.render(&game.ui_model(), (-1, -1));
    assert_eq!(count(&buf, (232, 92, 112)), 0, "no warmth lit yet");
    assert!(count(&buf, (72, 52, 62)) > 50, "the empty hearts still draw");
    // one full talk warms half a heart
    let pe = game.player;
    game2_core::game::relationships::record_interaction(&mut game.world, pe,
        game2_core::game::relationships::Interaction {
            other: game.villagers[3], kind: game2_core::game::relationships::InteractionKind::Conversation,
            intensity: 1.0 });
    let buf = view.render(&game.ui_model(), (-1, -1));
    let pink = count(&buf, (232, 92, 112));
    assert!(pink > 0 && pink < 30, "half a heart lit, got {} px", pink);
}

/// The page keeps its text inside the plate: a life's telling wraps
/// under LIFE rather than running off across the map the way the old
/// bio line did
#[test]
fn test_the_page_keeps_its_text_inside_the_plate() {
    let mut game = Game::new_seeded(MAP_SEED);
    let view = View::new(Art::load_fs("../assets", 0));
    game.selected_target = 3;
    // a bond close enough to be told things fills the page's sections
    let rel = game.world.create_entity();
    game.world.add_component(rel, game2_core::game::relationships::Relationship {
        from: game.player, other: game.villagers[3],
        kind: game2_core::game::relationships::RelKind::Friend,
        strength: 0.95, interactions: 5 });
    let buf = view.render(&game.ui_model(), (-1, -1));
    // the bio's colour, the run-on offender, inside the card and not past it
    let bio_px = |x0: usize, x1: usize| -> usize {
        let mut n = 0;
        for y in 500..945 { for x in x0..x1 {
            let i = (y * W + x) * 4;
            if (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]) == (160, 180, 200) { n += 1; }
        }}
        n
    };
    assert!(bio_px(20, 380) > 0, "no LIFE text on the page at all");
    assert_eq!(bio_px(384, MAP_W as usize), 0, "text ran past the card's edge");
}

/// For eyes: a known villager's properties page, rendered as
/// --dump-frame would. `INSP_FRAME=<path.bmp> cargo test -p
/// game2-client --lib inspector_page_frame -- --ignored`
#[test]
#[ignore]
fn inspector_page_frame() {
    let Ok(path) = std::env::var("INSP_FRAME") else { return };
    let mut game = Game::new_seeded(MAP_SEED);
    game.selected_target = 3;
    let rel = game.world.create_entity();
    game.world.add_component(rel, game2_core::game::relationships::Relationship {
        from: game.player, other: game.villagers[3],
        kind: game2_core::game::relationships::RelKind::Friend,
        strength: 0.95, interactions: 5 });
    let view = View::new(Art::load_fs("../assets", 0));
    view.render(&game.ui_model(), (-1, -1)).save_bmp(&path).unwrap();
}
