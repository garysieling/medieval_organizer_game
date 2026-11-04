use crate::session::*;
use crate::client::*;

#[test]
fn test_member_line_reads_and_fits_the_card() {
    assert_eq!(members_line(&[]), "nobody yet - recruit!");
    assert_eq!(members_line(&["Dagny", "Bram"]), "Dagny, Bram");
    let all = members_line(&["Dagny", "Bram", "Aldric", "Corvin", "Eliza"]);
    assert_eq!(all, "Dagny, Bram, Aldric, Corvin, Eliza");
    // past the width it stops at a whole name instead of mid-word
    let long = members_line(&["Dagny", "Bram", "Aldric", "Corvin", "Eliza", "Wulfric"]);
    assert!(long.ends_with(" ..") && long.len() <= MEMBERS_CHARS, "{}", long);
    // the card box ends at UI_X + 280
    let x = UI_X as i32 + 18 + (MEMBERS_LABEL.len() as i32 + 1) * GLYPH_W;
    for line in [&all, &long] {
        assert!(x + text_width(line, 1) <= UI_X as i32 + 278, "{} runs off the card", line);
    }
}

#[test]
fn test_build_log_doesnt_quote_tile_coordinates() {
    let mut game = Game::new();
    // the Wanderer's starting gold is rolled by class and can come up
    // short - this test is about the log line, not the purse
    game.world.get_component_mut::<PlayerCharacter>(game.player).unwrap().resources = 100.0;
    game.act(3);
    let line = game.log.last().unwrap();
    assert!(line.contains("new house") && !line.contains('('), "{}", line);
}
