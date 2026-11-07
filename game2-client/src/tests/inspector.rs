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
    // the card floats on the map's bottom-left; its hearts sit
    // top-right of it, same offsets the panel slot gave them
    let count = |buf: &Buffer, rgb: (u8, u8, u8)| -> usize {
        let mut n = 0;
        for y in 784..791 { for x in 292..332 {
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
