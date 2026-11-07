use crate::session::*;

#[test]
fn test_build_log_doesnt_quote_tile_coordinates() {
    let mut game = Game::new();
    // the Wanderer's starting gold is rolled by class and can come up
    // short - this test is about the log line, not the purse
    game.world.get_component_mut::<PlayerCharacter>(game.player).unwrap().resources = 100.0;
    game.act(2);
    let line = game.log.last().unwrap();
    assert!(line.contains("new house") && line.contains("1st roof.") && !line.contains('('), "{}", line);
}
